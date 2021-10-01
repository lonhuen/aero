mod common;
mod util;
mod zksnark;
use crate::common::aggregation::{
    merkle::HashAlgorithm,
    node::{SummationEntry, SummationLeaf, SummationNonLeaf},
};
use crate::common::server_service::ServerServiceClient;
use crate::common::{i128vec_to_le_bytes, summation_array_size, ZKProof};
use crate::util::{config::ConfigUtils, log::init_tracing};
use crate::zksnark::{Prover, Verifier};
use ark_std::{end_timer, start_timer};
#[cfg(feature = "hashfn_blake3")]
extern crate blake3;
#[cfg(not(feature = "hashfn_blake3"))]
use crypto::{digest::Digest, sha3::Sha3};
use tracing::{error, event, instrument, span, warn, Level};

use rand::{Rng, SeedableRng};
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::{fs::File, io::BufReader, net::IpAddr};
use std::{io::BufRead, process::id};
use std::{thread, time};
use tarpc::{client, context, tokio_serde::formats::Bincode};
mod rlwe;
use rlwe::PublicKey;
use tracing_subscriber::filter::LevelFilter;

const DEADLINE_TIME: u64 = 6000;
const NUM_DIMENSION: u32 = 4096;
pub struct LightClient {
    inner: ServerServiceClient,
    nr_lc: u32,
    // 451 bytes
    rsa_pk: Vec<Vec<u8>>,
    //c0s: Vec<i128>,
    //c1s: Vec<i128>,
    cts: Vec<i128>,
    // 192 bytes per proof
    proofs: Vec<u8>,
    nonce: [u8; 16],
}

impl LightClient {
    // TODO random this nounce
    pub fn new(inner: ServerServiceClient, nr_lc: u32, nr_parameter: u32) -> Self {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let cts = (0..nr_parameter * 2).map(|_| rng.gen::<i128>()).collect();
        let proofs = (0..nr_parameter).map(|_| rng.gen::<u8>()).collect();
        let rsa_pk = (0..nr_lc)
            .map(|_| (0..451).map(|_| rng.gen::<u8>()).collect())
            .collect();
        let nonce = rand::thread_rng().gen::<[u8; 16]>();
        Self {
            inner,
            nr_lc,
            rsa_pk: rsa_pk,
            cts,
            proofs,
            nonce,
        }
    }

    #[inline]
    pub fn random_hash() -> [u8; 32] {
        rand::thread_rng().gen::<[u8; 32]>()
    }

    pub async fn train_model(&self, round: u32) {
        let rm = start_timer!(|| "retrieve the model");
        for _ in 0..self.nr_lc as usize {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self.inner.retrieve_model(ctx, round).await;
        }
        end_timer!(rm);
    }

    pub async fn upload(&self, round: u32) {
        // set the deadline of the context
        // generate commitment to all the CTs
        let cm = Self::random_hash();

        // upload all the commitment first
        for i in 0..self.nr_lc as usize {
            // send this commitment to the server
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .aggregate_commit(ctx, round, self.rsa_pk[i].clone(), cm)
                .await;
        }
        // get all the mc_proof
        for i in 0..self.nr_lc as usize {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .get_mc_proof(ctx, round, self.rsa_pk[i].clone())
                .await;
        }

        // upload all the data
        for i in 0..self.nr_lc as usize {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .aggregate_data(
                    ctx,
                    round,
                    self.rsa_pk[i].clone(),
                    self.cts.clone(),
                    self.nonce,
                    self.proofs.clone(),
                )
                .await;
        }

        // get all ms proof
        for i in 0..self.nr_lc as usize {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .get_ms_proof(ctx, round, self.rsa_pk[i].clone())
                .await;
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ConfigUtils::init("config.ini");
    init_tracing(
        &format!("LightWeight Atom client {}", std::process::id()),
        config.get_agent_endpoint(),
        LevelFilter::WARN,
    )?;

    let _span = span!(Level::WARN, "LightWeight Atom Client").entered();

    let start = start_timer!(|| "clients");

    let nr_parameter = config.get_int("nr_parameter") as u32;
    let nr_lc = config.get_int("nr_simulated") as u32;
    let nr_round = config.get_int("nr_round") as u32;

    let server_addr = (
        IpAddr::V4(config.get_addr("server_addr")),
        config.get_int("server_port") as u16,
    );
    #[cfg(feature = "json")]
    let mut transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
    #[cfg(not(feature = "json"))]
    let mut transport = tarpc::serde_transport::tcp::connect(server_addr, Bincode::default);
    transport.config_mut().max_frame_length(usize::MAX);

    let inner_client =
        ServerServiceClient::new(client::Config::default(), transport.await?).spawn();
    let mut client = LightClient::new(inner_client, nr_lc, nr_parameter);

    for i in 0..nr_round {
        // begin uploading
        let sr = start_timer!(|| "one round");
        let train = start_timer!(|| "train model");
        client.train_model(i).await;
        end_timer!(train);

        let rs = start_timer!(|| "upload data");
        let _ = client.upload(i).await;
        end_timer!(rs);

        //let vr = start_timer!(|| "verify the data");
        //client.verify(nr_real, 5).await;
        //end_timer!(vr);
        //end_timer!(sr);
    }
    end_timer!(start);
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
