use ark_groth16::verifier;

use futures::{
    future::{self, Join, Ready},
    prelude::*,
};
use rand::{Rng, SeedableRng};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    convert::Into,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex, RwLock},
    thread::sleep,
    time::Duration,
};
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Bincode,
};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{error, event, instrument, span, warn, Level};
use tracing_subscriber::filter::LevelFilter;
mod common;
use crate::common::{
    aggregation::{
        merkle::*,
        node::{CommitEntry, SummationEntry, SummationLeaf},
        McTree, MsTree,
    },
    server_service::ServerService,
};

mod util;
use crate::util::{config::ConfigUtils, log::init_tracing};
mod back_server;
use back_server::Server;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum STATE {
    Commit(u32),
    Data(u32),
    Verify(u32),
}
//pub struct Server {
//    pub commit_array: BTreeMap<Vec<u8>, MerkleHash>,
//    pub mc: Option<MerkleTree>,
//    pub summation_array: Vec<SummationEntry>,
//    pub ms: Option<MerkleTree>,
//    pub state: STATE,
//    //pub model: Vec<u8>,
//}
//impl Server {
//    pub fn new() -> Self {
//        Self {
//            commit_array: BTreeMap::new(),
//            mc: None,
//            summation_array: Vec::new(),
//            ms: None,
//            state: STATE::Commit,
//            //model: vec![0u8; NR_PARAMETER as usize],
//        }
//    }
//}
#[derive(Clone)]
pub struct InnerServer {
    addr: SocketAddr,
    pool: Arc<ThreadPool>,
    server: Server,
}
impl InnerServer {
    pub fn new(addr: SocketAddr, pool: &Arc<ThreadPool>, server: &Server) -> Self {
        Self {
            addr,
            pool: pool.clone(),
            server: server.clone(),
        }
    }
}
#[tarpc::server]
impl ServerService for InnerServer {
    //type AggregateCommitFut = Ready<()>;
    async fn aggregate_commit(
        self,
        _: context::Context,
        round: u32,
        rsa_pk: Vec<u8>,
        commitment: Vec<[u8; 32]>,
    ) {
        //self.pool
        //    .as_ref()
        //    .install(|| self.server.aggregate_commit(round, rsa_pk, commitment))
        std::thread::spawn(move || self.server.aggregate_commit(round, rsa_pk, commitment))
            .join()
            .unwrap()
    }

    async fn aggregate_data(
        self,
        _: context::Context,
        round: u32,
        rsa_pk: Vec<u8>,
        c0: Vec<Vec<i128>>,
        c1: Vec<Vec<i128>>,
        nonce: Vec<[u8; 16]>,
        proofs: Vec<Vec<u8>>,
    ) {
        std::thread::spawn(move || {
            self.server
                .aggregate_data(round, rsa_pk, c0, c1, nonce, proofs)
        })
        .join()
        .unwrap()
        //self.pool.as_ref().install(|| {
        //});
    }

    //type GetMcProofFut = Ready<MerkleProof>;
    //async fn get_mc_proof(
    //    self,
    //    _: context::Context,
    //    round: u32,
    //    rsa_pk: Vec<u8>,
    //) -> Vec<MerkleProof> {
    async fn get_mc_proof(self, _: context::Context, round: u32, rsa_pk: Vec<u8>) {
        std::thread::spawn(move || self.server.get_mc_proof(round, rsa_pk))
            .join()
            .unwrap()
        //self.pool
        //    .as_ref()
        //    .install(|| self.server.get_mc_proof(round, rsa_pk))
    }

    //type GetMsProofFut = Ready<MerkleProof>;
    //async fn get_ms_proof(
    //    self,
    //    _: context::Context,
    //    round: u32,
    //    rsa_pk: Vec<u8>,
    //) -> Vec<MerkleProof> {
    async fn get_ms_proof(self, _: context::Context, round: u32, rsa_pk: Vec<u8>) {
        //self.pool
        //    .as_ref()
        //    .install(|| self.server.get_ms_proof(round, rsa_pk))
        std::thread::spawn(move || self.server.get_ms_proof(round, rsa_pk))
            .join()
            .unwrap()
    }

    //type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    async fn verify(
        self,
        _: context::Context,
        round: u32,
        vinit: u32,
        non_leaf_id: Vec<u32>,
        ct_id: Vec<usize>,
    ) -> Vec<Vec<(SummationEntry, MerkleProof)>> {
        //self.pool
        //    .as_ref()
        //    .install(|| self.server.verify(round, vinit, non_leaf_id))
        std::thread::spawn(move || self.server.verify(round, vinit, non_leaf_id, ct_id))
            .join()
            .unwrap()
    }

    //type RetrieveModelFut = Ready<Vec<u8>>;
    async fn retrieve_model(self, _: context::Context, round: u32) -> Vec<u8> {
        std::thread::spawn(move || self.server.retrieve_model(round))
            .join()
            .unwrap()
        //self.pool
        //    .as_ref()
        //    .install(|| self.server.retrieve_model(round))
    }

    //type RetrieveProvingKeyFut = Ready<Vec<u8>>;
    async fn retrieve_proving_key(self, _: context::Context, round: u32) -> Vec<u8> {
        std::thread::spawn(move || self.server.retrieve_proving_key(round))
            .join()
            .unwrap()
        //self.pool
        //    .as_ref()
        //    .install(|| self.server.retrieve_proving_key(round))
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ConfigUtils::init("config.yaml");
    init_tracing(
        "Atom Server",
        config.get_agent_endpoint(),
        LevelFilter::WARN,
    )?;

    let _span = span!(Level::WARN, "Atom Server").entered();

    let nr_real = config.get_int("nr_real") as u32;
    let nr_sim = config.get_int("nr_simulated") as u32;
    let nr_sybil = config.get_int("nr_sybil") as u32;
    let nr_parameter = config.get_int("nr_parameter") as u32;

    let server_addr = (
        IpAddr::V4(config.get_addr("server_addr")),
        config.get_int("server_port") as u16,
    );

    let pool = Arc::new(ThreadPoolBuilder::new().build().unwrap());
    let server = Server::setup(nr_real, nr_sim, nr_sybil, nr_parameter, &pool);

    #[cfg(feature = "json")]
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    #[cfg(not(feature = "json"))]
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Bincode::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);

    println!("Atom: server starts listening");

    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let inner_server =
                InnerServer::new(channel.transport().peer_addr().unwrap(), &pool, &server);
            channel.execute(inner_server.serve())
            //tokio::spawn(channel.execute(inner_server.serve()))
        })
        // Max 100 channels.
        .buffer_unordered(999)
        .for_each(|_| async {})
        .await;

    Ok(())
}
