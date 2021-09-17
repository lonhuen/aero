use ark_groth16::verifier;
use futures::{
    future::{self, Ready},
    prelude::*,
};
use log::{error, info, warn};
use quail::zksnark::{Prover, Verifier};
use std::{
    collections::BTreeMap,
    convert::{Into, TryInto},
    io,
    iter::FromIterator,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread::sleep,
    time::Duration,
};

use bincode::Options;
use tarpc::{
    context,
    server::{self, Channel, Incoming},
    tokio_serde::formats::{Bincode, Json},
};
mod common;
use crate::common::{
    aggregation::{
        merkle::*,
        node::{CommitEntry, SummationEntry, SummationLeaf},
        McTree, MsTree,
    },
    hash_commitment, new_rsa_pub_key,
    server_service::ServerService,
};
mod util;
use crate::util::{config::ConfigUtils, log::LogUtils};
pub enum STATE {
    Commit,
    Data,
    Verify,
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
    mc: Arc<RwLock<McTree>>,
    ms: Arc<RwLock<MsTree>>,
    nr_parameter: u32,
    pvk: Arc<Vec<u8>>,
    verifier: Arc<Verifier>,
}
impl InnerServer {
    pub fn new(
        addr: SocketAddr,
        mc: &Arc<RwLock<McTree>>,
        ms: &Arc<RwLock<MsTree>>,
        nr_parameter: u32,
        pvk: &Arc<Vec<u8>>,
        verifier: &Arc<Verifier>,
    ) -> Self {
        Self {
            addr,
            mc: mc.clone(),
            ms: ms.clone(),
            nr_parameter,
            pvk: pvk.clone(),
            verifier: verifier.clone(),
        }
    }
}
#[tarpc::server]
impl ServerService for InnerServer {
    type RetrieveModelFut = Ready<Vec<u8>>;
    fn aggregate_commit(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        commitment: [u8; 32],
    ) -> Self::AggregateCommitFut {
        // wait for enough commitments
        // TODO maybe wait for some time rather than some # of commitments
        // TODO to fix: should check if duplicate commitments come
        let mut mc = self.mc.as_ref().write().unwrap();
        mc.insert_node(CommitEntry {
            rsa_pk,
            hash: commitment,
        });
        mc.gen_tree();
        drop(mc);
        future::ready(())
    }

    type AggregateDataFut = Ready<()>;
    fn aggregate_data(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        cts: Vec<i128>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) -> Self::AggregateDataFut {
        // TODO also verify the proof
        let mut ms = self.ms.as_ref().write().unwrap();
        ms.insert_node(SummationLeaf::from_ct(rsa_pk, cts, nonce));
        ms.gen_tree();
        drop(ms);
        future::ready(())
    }

    type GetMcProofFut = Ready<MerkleProof>;
    // TODO for now assume only 1 round
    fn get_mc_proof(self, _: context::Context, rsa_pk: Vec<u8>, round: u32) -> Self::GetMcProofFut {
        loop {
            let mc = self.mc.as_ref().read().unwrap();
            if mc.commit_array.len() >= mc.nr_real as usize {
                break;
            }
            drop(mc);
            sleep(Duration::from_millis(100));
        }
        let mc = self.mc.as_ref().read().unwrap();
        future::ready(mc.get_proof(&rsa_pk))
    }

    type GetMsProofFut = Ready<MerkleProof>;
    fn get_ms_proof(self, _: context::Context, rsa_pk: Vec<u8>, round: u32) -> Self::GetMsProofFut {
        loop {
            let ms = self.ms.as_ref().read().unwrap();
            if ms.summation_array.len() >= ms.nr_real as usize {
                break;
            }
            drop(ms);
            sleep(Duration::from_millis(100));
        }
        let ms = self.ms.as_ref().read().unwrap();
        future::ready(ms.get_proof(&rsa_pk))
    }

    type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    fn verify(self, _: context::Context, vinit: u32, non_leaf_id: Vec<u32>) -> Self::VerifyFut {
        // TODO the client should call get_ms_proof before verify. Fix this for SGD
        //first all the leafs
        let mut ret: Vec<(SummationEntry, MerkleProof)> = Vec::new();
        let ms = self.ms.as_ref().read().unwrap();
        let mc = self.mc.as_ref().read().unwrap();
        for i in 0..5 + 1 {
            let node = ms.get_node(i + vinit);
            if let SummationEntry::Leaf(_) = node {
                let mc_proof: MerkleProof = mc.get_proof_by_id(i + vinit).into();
                let ms_proof: MerkleProof = ms.get_proof_by_id(i + vinit).into();
                ret.push((SummationEntry::Commit(mc.get_node(i + vinit)), mc_proof));
                ret.push((node, ms_proof));
            } else {
                warn!("Atom: verify not a leaf node");
            }
        }
        for i in non_leaf_id {
            let ms_proof: MerkleProof = ms.get_proof_by_id(i).into();
            ret.push((ms.get_node(i), ms_proof));
        }
        future::ready(ret)
    }

    type AggregateCommitFut = Ready<()>;
    fn retrieve_model(self, _: context::Context) -> Self::RetrieveModelFut {
        self.mc.as_ref().write().unwrap().clear();
        self.ms.as_ref().write().unwrap().clear();
        future::ready(vec![0u8; self.nr_parameter as usize])
    }

    type RetrieveProvingKeyFut = Ready<Vec<u8>>;
    fn retrieve_proving_key(self, _: context::Context) -> Self::RetrieveModelFut {
        //future::ready(self.pvk.as_ref().clone())
        future::ready(vec![0u8; 1])
    }
}
#[tokio::main]
async fn main() -> io::Result<()> {
    LogUtils::init("server.log");

    let config = ConfigUtils::init("config.ini");
    let nr_real = config.get_int("nr_real") as u32;
    let nr_sybil = config.get_int("nr_sybil") as u32;
    let nr_parameter = config.get_int("nr_parameter") as u32;

    let server_addr = (
        IpAddr::V4(config.get_addr("server_addr")),
        config.get_int("server_port") as u16,
    );

    let mc = McTree::new(nr_real, nr_sybil);
    let ms = MsTree::new(nr_real, nr_sybil);
    let prover = Prover::setup("./data/encryption.txt");
    let pvk = prover.serialize_pvk();
    let verifier = Verifier::new(&prover);

    drop(prover);

    let mc_ref = Arc::new(RwLock::new(mc));
    let ms_ref = Arc::new(RwLock::new(ms));

    let prover_ref = Arc::new(pvk);
    let verifier_ref = Arc::new(verifier);

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
        .max_channels_per_key(999, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let inner_server = InnerServer::new(
                channel.transport().peer_addr().unwrap(),
                &mc_ref,
                &ms_ref,
                nr_parameter,
                &prover_ref,
                &verifier_ref,
            );
            channel.execute(inner_server.serve())
        })
        // Max 100 channels.
        .buffer_unordered(999)
        .for_each(|_| async {})
        .await;

    Ok(())
}
