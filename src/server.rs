use futures::{
    future::{self, Ready},
    prelude::*,
};
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

use tarpc::{
    context,
    server::{self, Channel, Incoming},
    tokio_serde::formats::Json,
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
    nr_parameter: u32
}
impl InnerServer {
    pub fn new(addr: SocketAddr, mc: &Arc<RwLock<McTree>>, ms: &Arc<RwLock<MsTree>>,nr_parameter:u32) -> Self {
        Self {
            addr,
            mc: mc.clone(),
            ms: ms.clone(),
            nr_parameter,
        }
    }
}
#[tarpc::server]
impl ServerService for InnerServer {
    type RetrieveModelFut = Ready<Vec<u8>>;

    type AggregateCommitFut = Ready<()>;
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
    fn get_mc_proof(self, _: context::Context, rsa_pk:Vec<u8>, round: u32) -> Self::GetMcProofFut {
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
    fn get_ms_proof(self, _: context::Context, rsa_pk:Vec<u8>, round: u32) -> Self::GetMsProofFut {
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

    //type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    // TODO needs sync here before starting next round
    //fn verify(self, _: context::Context, vinit: u32, non_leaf_id: Vec<u32>) -> Self::VerifyFut {
    //    // TODO maybe RwLock? not able to directly read the content
    //    //first all the leafs
    //    let mut ret: Vec<(SummationEntry, MerkleProof)> = Vec::new();
    //    let ms = self.ms.as_ref().read().unwrap();
    //    let mc = self.mc.as_ref().read().unwrap();
    //    for i in 0..5 + 1 {
    //        let node = ms.summation_array[(i + vinit) as usize].clone();
    //        //println!("{:?}", s.mc.as_ref().unwrap().gen_proof(0));
    //        let mc_proof: MerkleProof = mc.gen_proof(((i + vinit) % NR_COMMIT) as usize).into();
    //        let ms_proof: MerkleProof =
    //            s.ms.as_ref()
    //                .unwrap()
    //                .gen_proof(((i + vinit) % NR_COMMIT) as usize)
    //                .into();
    //        if let SummationEntry::Leaf(l) = node {
    //            let pk = l.rsa_pk;
    //            let hash = s.commit_array.get(&pk).unwrap().clone();
    //            //println!("commit_array len {}", s.commit_array.len());
    //            let hash = [0u8; 32];
    //            ret.push((
    //                SummationEntry::Commit(CommitEntry {
    //                    rsa_pk: pk,
    //                    hash: hash,
    //                }),
    //                mc_proof,
    //            ));
    //            ret.push((s.summation_array[(i + vinit) as usize].clone(), ms_proof));
    //        }
    //    }
    //    for i in non_leaf_id {
    //        let ms_proof: MerkleProof = s.ms.as_ref().unwrap().gen_proof(i as usize).into();
    //        ret.push((s.summation_array[i as usize].clone(), ms_proof));
    //    }
    //    drop(s);
    //    // wait for all the threads to finish
    //    let mut num_clients = self.cond.0.lock().unwrap();
    //    *num_clients = *num_clients + 1;
    //    if *num_clients < NR_COMMIT {
    //        num_clients = self.cond.1.wait(num_clients).unwrap();
    //    } else {
    //        *num_clients = 0;
    //        // TODO decrypt here and update the model here
    //        //self.server.write().unwrap().model = vec![0u8; NR_PARAMETER as usize];

    //        self.server.write().unwrap().state = STATE::Commit;
    //        self.cond.1.notify_all();
    //    }
    //    drop(num_clients);
    //    future::ready(ret)
    //}

    fn retrieve_model(self, _: context::Context) -> Self::RetrieveModelFut {
        //future::ready(self.server.read().unwrap().model.clone())
        future::ready(vec![0u8; self.nr_parameter as usize])
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

    let mut mc = McTree::new(nr_real, nr_sybil);
    let mut ms = MsTree::new(nr_real, nr_sybil);

    let mut mc_ref = Arc::new(RwLock::new(mc));
    let mut ms_ref = Arc::new(RwLock::new(ms));

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(999, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let inner_server =
                InnerServer::new(channel.transport().peer_addr().unwrap(), &mc_ref, &ms_ref,nr_parameter);
            channel.execute(inner_server.serve())
        })
        // Max 100 channels.
        .buffer_unordered(999)
        .for_each(|_| async {})
        .await;

    Ok(())
}
