use futures::{
    future::{self, Ready},
    prelude::*,
};
use std::{
    collections::BTreeMap,
    convert::{Into, TryInto},
    io,
    iter::FromIterator,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    process::exit,
    sync::{Arc, Condvar, Mutex, RwLock},
};

use tarpc::{
    context,
    server::{self, Channel, Incoming},
    tokio_serde::formats::Json,
};
mod common;
use crate::common::{
    aggregation::{merkle::*, CommitEntry, SummationEntry, SummationLeaf},
    hash_commitment, new_rsa_pub_key,
    server_service::ServerService,
};
pub enum STATE {
    Commit,
    Data,
    Verify,
}
const NR_COMMIT: u32 = 8;
const NR_SYBIL: u32 = 8;
const NR_PARAMETER: u32 = 4096 * 10;
pub struct Server {
    pub commit_array: BTreeMap<Vec<u8>, MerkleHash>,
    pub mc: Option<MerkleTree>,
    pub summation_array: Vec<SummationEntry>,
    pub ms: Option<MerkleTree>,
    pub state: STATE,
    //pub model: Vec<u8>,
}
impl Server {
    pub fn new() -> Self {
        Self {
            commit_array: BTreeMap::new(),
            mc: None,
            summation_array: Vec::new(),
            ms: None,
            state: STATE::Commit,
            //model: vec![0u8; NR_PARAMETER as usize],
        }
    }
}
#[derive(Clone)]
pub struct InnerServer {
    addr: SocketAddr,
    server: Arc<RwLock<Server>>,
    cond: Arc<(Mutex<u32>, Condvar)>,
}
impl InnerServer {
    pub fn new(
        addr: SocketAddr,
        server: &Arc<RwLock<Server>>,
        cond: &Arc<(Mutex<u32>, Condvar)>,
    ) -> Self {
        Self {
            addr,
            server: server.clone(),
            cond: cond.clone(),
        }
    }
}
#[tarpc::server]
impl ServerService for InnerServer {
    type AggregateCommitFut = Ready<MerkleProof>;
    type AggregateDataFut = Ready<MerkleProof>;
    type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    type RetrieveModelFut = Ready<Vec<u8>>;
    fn aggregate_commit(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        commitment: [u8; 32],
    ) -> Self::AggregateCommitFut {
        // wait for enough commitments
        // TODO maybe wait for some time rather than some # of commitments
        // TODO to fix: if another batch comes, the commit array will be modified
        println!("commit round connected");

        let mut num_clients = self.cond.0.lock().unwrap();

        if !matches!(self.server.read().as_ref().unwrap().state, STATE::Commit) {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        }

        if *num_clients == 0 {
            let mut s = self.server.write().unwrap();
            s.commit_array = BTreeMap::new();
            s.mc = None;
            s.summation_array = Vec::new();
            s.ms = None;
        }

        let idx = *num_clients;

        // push into the server
        {
            self.server
                .write()
                .unwrap()
                .commit_array
                .insert(rsa_pk, commitment);
        }

        *num_clients = *num_clients + 1;
        //println!("commit round {}", *num_clients);
        if *num_clients < NR_COMMIT {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        } else if *num_clients == NR_COMMIT {
            {
                let s = &mut *self.server.write().unwrap();
                // TODO add more sybil clients
                for _ in 0..NR_SYBIL {
                    s.commit_array.insert(new_rsa_pub_key(), [0u8; 32]);
                }
                // init the summation leaf array
                for _ in 0..s.commit_array.len() {
                    s.summation_array
                        .push(SummationEntry::Leaf(SummationLeaf::new()));
                }
                s.mc = Some(MerkleTree::from_iter(
                    s.commit_array.iter().map(|x| hash_commitment(&x.0, &x.1)),
                ));
                s.state = STATE::Data;
            }
            // notify all and generate the commit tree
            // also reset the num_clients for data
            *num_clients = 0;
            self.cond.1.notify_all();
        } else {
            //TODO maybe never reach here?
            assert!(false);
        }
        // unlock
        drop(num_clients);

        //println!("finish commit round");

        let proof_commit = {
            let s = &*self.server.write().unwrap();
            s.mc.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        };

        future::ready(proof_commit.into())
    }
    fn aggregate_data(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        cts: Vec<i128>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) -> Self::AggregateCommitFut {
        let mut num_clients = self.cond.0.lock().unwrap();

        while !matches!(self.server.read().as_ref().unwrap().state, STATE::Data) {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        }

        let mut s = self.server.write().unwrap();

        let sorted_keys: Vec<_> = s.commit_array.keys().cloned().collect();
        // TODO also verify the proof
        let idx = {
            if let Ok(id) = sorted_keys.binary_search(&rsa_pk) {
                // TODO no redundant should be sent
                s.summation_array[id] =
                    SummationEntry::Leaf(SummationLeaf::from_ct(rsa_pk, cts, nonce));
                id
            } else {
                assert!(false);
                // just to make the compiler happy
                0
            }
        };

        // unlock
        drop(s);

        *num_clients = *num_clients + 1;
        if *num_clients < NR_COMMIT {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        } else if *num_clients == NR_COMMIT {
            {
                let s = &mut *self.server.write().unwrap();
                // TODO add more sybil clients
                if let SummationEntry::Leaf(ll) = s.summation_array[idx].clone() {
                    for i in 0..NR_SYBIL + NR_COMMIT {
                        //s.summation_array.push(s.summation_array[idx].clone());
                        if let SummationEntry::Leaf(node) = &s.summation_array[i as usize] {
                            if node.c0.is_some() {
                                continue;
                            }
                        }
                        s.summation_array[i as usize] = SummationEntry::Leaf(SummationLeaf {
                            rsa_pk: sorted_keys[i as usize].clone(),
                            c0: ll.c0.clone(),
                            c1: ll.c1.clone(),
                            r: ll.r.clone(),
                        });
                        //println!("push {} into sumarray", i + NR_COMMIT);
                    }
                    //println!("len of commit array {}", s.commit_array.len());
                    //println!("len of summation array {}", s.summation_array.len());
                }
                // add the whole tree
                let mut left = 0;
                let mut right = s.summation_array.len();
                while left + 1 < right {
                    let a = match s.summation_array[left].clone() {
                        SummationEntry::NonLeaf(y) => y,
                        SummationEntry::Leaf(x) => x.into(),
                        _ => {
                            assert!(false, "commitment in summation array");
                            exit(1);
                        }
                    };
                    let b = match s.summation_array[left + 1].clone() {
                        SummationEntry::NonLeaf(y) => y,
                        SummationEntry::Leaf(x) => x.into(),
                        _ => {
                            assert!(false, "commitment in summation array");
                            exit(1);
                        }
                    };
                    let c = a + b;
                    s.summation_array.push(SummationEntry::NonLeaf(c));
                    left += 2;
                    right += 1;
                }
                // just for test purpose
                // for ii in 0..s.summation_array.len() {
                //     match s.summation_array[ii].clone() {
                //         SummationEntry::NonLeaf(x) => {
                //             println!("summation nonleaf [{}]={:?}", ii, x.c0[0]);
                //         }
                //         SummationEntry::Leaf(x) => {
                //             println!("summation leaf [{}]={:?}", ii, x.c0.unwrap()[0]);
                //         }
                //         _ => {
                //             println!("commit in sarray");
                //         }
                //     };
                // }

                s.ms = Some(MerkleTree::from_iter(s.summation_array.iter().map(
                    |x| match x {
                        SummationEntry::Leaf(y) => y.hash(),
                        SummationEntry::NonLeaf(y) => y.hash(),
                        // just to make compiler happy
                        // never reach here
                        _ => {
                            assert!(false, "commitment in summation array");
                            exit(1);
                        }
                    },
                )));
                s.state = STATE::Verify;
            }
            // notify all and generate the commit tree
            self.cond.1.notify_all();
        } else {
            //TODO maybe never reach here?
            assert!(false);
        }
        // unlock
        *num_clients = 0;
        drop(num_clients);

        let proof_leaf = {
            let s = &*self.server.write().unwrap();
            s.ms.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        };

        future::ready(proof_leaf.into())
    }

    // TODO needs sync here before starting next round
    fn verify(self, _: context::Context, vinit: u32, non_leaf_id: Vec<u32>) -> Self::VerifyFut {
        // TODO maybe RwLock? not able to directly read the content
        //first all the leafs
        let mut num_clients = self.cond.0.lock().unwrap();
        while !matches!(self.server.read().as_ref().unwrap().state, STATE::Verify) {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        }
        drop(num_clients);
        let mut ret: Vec<(SummationEntry, MerkleProof)> = Vec::new();
        let s = self.server.read().unwrap();
        for i in 0..5 + 1 {
            let node = s.summation_array[(i + vinit) as usize].clone();
            //println!("{:?}", s.mc.as_ref().unwrap().gen_proof(0));
            let mc_proof: MerkleProof =
                s.mc.as_ref()
                    .unwrap()
                    .gen_proof(((i + vinit) % NR_COMMIT) as usize)
                    .into();
            let ms_proof: MerkleProof =
                s.ms.as_ref()
                    .unwrap()
                    .gen_proof(((i + vinit) % NR_COMMIT) as usize)
                    .into();
            if let SummationEntry::Leaf(l) = node {
                let pk = l.rsa_pk;
                let hash = s.commit_array.get(&pk).unwrap().clone();
                //println!("commit_array len {}", s.commit_array.len());
                let hash = [0u8; 32];
                ret.push((
                    SummationEntry::Commit(CommitEntry {
                        rsa_pk: pk,
                        hash: hash,
                    }),
                    mc_proof,
                ));
                ret.push((s.summation_array[(i + vinit) as usize].clone(), ms_proof));
            }
        }
        for i in non_leaf_id {
            let ms_proof: MerkleProof = s.ms.as_ref().unwrap().gen_proof(i as usize).into();
            ret.push((s.summation_array[i as usize].clone(), ms_proof));
        }
        drop(s);
        // wait for all the threads to finish
        let mut num_clients = self.cond.0.lock().unwrap();
        *num_clients = *num_clients + 1;
        if *num_clients < NR_COMMIT {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        } else {
            *num_clients = 0;
            // TODO decrypt here and update the model here
            //self.server.write().unwrap().model = vec![0u8; NR_PARAMETER as usize];

            self.server.write().unwrap().state = STATE::Commit;
            self.cond.1.notify_all();
        }
        drop(num_clients);
        future::ready(ret)
    }

    fn retrieve_model(self, _: context::Context) -> Self::RetrieveModelFut {
        //future::ready(self.server.read().unwrap().model.clone())
        future::ready(vec![0u8; NR_PARAMETER as usize])
    }
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 38886u16);

    let mut server = Server::new();
    let mut server_ref = Arc::new(RwLock::new(server));
    let mut cond_ref = Arc::new((Mutex::new(0), Condvar::new()));

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
            let inner_server = InnerServer::new(
                channel.transport().peer_addr().unwrap(),
                &server_ref,
                &cond_ref,
            );
            channel.execute(inner_server.serve())
        })
        // Max 100 channels.
        .buffer_unordered(999)
        .for_each(|_| async {})
        .await;

    Ok(())
}
