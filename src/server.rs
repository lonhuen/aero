extern crate crypto;
extern crate merkle_light;
extern crate rand;
extern crate rsa;
mod server_service;
use crypto::digest::Digest;
use crypto::sha3::{Sha3, Sha3Mode};
use futures::{
    future::{self, Ready},
    prelude::*,
};
use rsa::pkcs1::der::bigint::generic_array::typenum::private::IsEqualPrivate;
use server_service::hash_commitment;
use std::io;
use std::ops::Add;
use std::rc::Rc;
use std::time::SystemTime;
use tarpc::server::Serve;

use merkle_light::hash::{Algorithm, Hashable};
use merkle_light::merkle::MerkleTree;
use merkle_light::proof::Proof;
use rand::rngs::OsRng;
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use server_service::{MerkleProof, ServerService};
use std::collections::BTreeMap;
use std::convert::{From, Into, TryInto};
use std::fmt;
use std::hash::Hasher;
use std::iter::FromIterator;
use std::sync::mpsc::channel;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use tarpc::{
    context,
    server::{self, Channel, Incoming},
    tokio_serde::formats::Json,
};
const NR_COMMIT: u32 = 8;
pub struct ExampleAlgorithm(Sha3);

impl ExampleAlgorithm {
    pub fn new() -> ExampleAlgorithm {
        ExampleAlgorithm(Sha3::new(Sha3Mode::Sha3_256))
    }
}

impl Default for ExampleAlgorithm {
    fn default() -> ExampleAlgorithm {
        ExampleAlgorithm::new()
    }
}

impl Hasher for ExampleAlgorithm {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.input(msg)
    }

    #[inline]
    fn finish(&self) -> u64 {
        unimplemented!()
    }
}

impl Algorithm<[u8; 32]> for ExampleAlgorithm {
    #[inline]
    fn hash(&mut self) -> [u8; 32] {
        let mut h = [0u8; 32];
        self.0.result(&mut h);
        h
    }

    #[inline]
    fn reset(&mut self) {
        self.0.reset();
    }
}
#[derive(Clone)]
pub struct CommitEntry {
    rsa_pk: Vec<u8>,
    hash: [u8; 32],
}

#[derive(Clone)]
pub struct SummationLeaf {
    pub rsa_pk: Vec<u8>,
    pub c0: Option<Vec<i128>>,
    pub c1: Option<Vec<i128>>,
    pub r: Option<[u8; 16]>,
}

impl SummationLeaf {
    pub fn new() -> Self {
        SummationLeaf {
            rsa_pk: Vec::new(),
            c0: None,
            c1: None,
            r: None,
        }
    }
    pub fn from_ct(rsa_pk: Vec<u8>, cts: Vec<u8>, r: [u8; 16]) -> Self {
        let c0: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| i128::from_le_bytes(cts[i..i + 16].try_into().expect("uncorrected length")))
            .collect();
        let c1: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(
                    cts[(i + 65536)..(i + 16 + 65536)]
                        .try_into()
                        .expect("uncorrected length"),
                )
            })
            .collect();
        SummationLeaf {
            rsa_pk,
            c0: Some(c0),
            c1: Some(c1),
            r: Some(r),
        }
    }
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        hasher.input(&self.rsa_pk);
        if self.c0.is_some() {
            let c0_bytes: Vec<u8> = (0..4096)
                .into_iter()
                .flat_map(|i| i128::to_le_bytes(self.c0.as_ref().unwrap()[i]))
                .collect();
            let c1_bytes: Vec<u8> = (0..4096)
                .into_iter()
                .flat_map(|i| i128::to_le_bytes(self.c1.as_ref().unwrap()[i]))
                .collect();
            hasher.input(&c0_bytes);
            hasher.input(&c1_bytes);
            hasher.input(&self.r.unwrap());
        }

        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
}

#[derive(Clone)]
pub struct SummationNonLeaf {
    pub c0: Vec<i128>,
    pub c1: Vec<i128>,
}
impl SummationNonLeaf {
    pub fn new() -> Self {
        let c0 = vec![0i128; 4096];
        let c1 = vec![0i128; 4096];
        SummationNonLeaf { c0, c1 }
    }
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        let c0_bytes: Vec<u8> = (0..4096)
            .into_iter()
            .flat_map(|i| i128::to_le_bytes(self.c0[i]))
            .collect();
        let c1_bytes: Vec<u8> = (0..4096)
            .into_iter()
            .flat_map(|i| i128::to_le_bytes(self.c1[i]))
            .collect();

        hasher.input(&c0_bytes);
        hasher.input(&c1_bytes);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
}

impl Add for SummationNonLeaf {
    type Output = SummationNonLeaf;
    fn add(self, other: Self) -> Self {
        // TODO need modulus here maybe
        let new_c0: Vec<i128> = (0..4096)
            .into_iter()
            .map(|i| self.c0[i] + other.c0[i])
            .collect();
        let new_c1: Vec<i128> = (0..4096)
            .into_iter()
            .map(|i| self.c1[i] + other.c1[i])
            .collect();
        SummationNonLeaf {
            c0: new_c0,
            c1: new_c1,
        }
    }
}
impl From<SummationLeaf> for SummationNonLeaf {
    fn from(leaf: SummationLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: leaf.c0.unwrap_or(vec![0i128; 4096]),
            c1: leaf.c1.unwrap_or(vec![0i128; 4096]),
        }
    }
}
#[derive(Clone)]
pub enum SummationEntry {
    Leaf(SummationLeaf),
    NonLeaf(SummationNonLeaf),
}
pub struct Server {
    pub commit_array: BTreeMap<Vec<u8>, [u8; 32]>,
    pub mc: Option<MerkleTree<[u8; 32], ExampleAlgorithm>>,
    pub summation_array: Vec<SummationEntry>,
    pub ms: Option<MerkleTree<[u8; 32], ExampleAlgorithm>>,
}
impl Server {
    pub fn new() -> Self {
        Self {
            commit_array: BTreeMap::new(),
            mc: None,
            summation_array: Vec::new(),
            ms: None,
        }
    }
}
#[derive(Clone)]
pub struct InnerServer {
    addr: SocketAddr,
    server: Arc<Mutex<Server>>,
    cond: Arc<(Mutex<u32>, Condvar)>,
}
impl InnerServer {
    pub fn new(
        addr: SocketAddr,
        server: &Arc<Mutex<Server>>,
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
    fn aggregate_commit(
        self,
        ctx: context::Context,
        rsa_pk: Vec<u8>,
        commitment: [u8; 32],
    ) -> Self::AggregateCommitFut {
        // wait for enough commitments
        // TODO maybe wait for some time rather than some # of commitments
        // TODO to fix: if another batch comes, the commit array will be modified
        let mut num_clients = self.cond.0.lock().unwrap();

        let idx = *num_clients;

        // push into the server
        {
            self.server
                .lock()
                .unwrap()
                .commit_array
                .insert(rsa_pk, commitment);
        }

        *num_clients = *num_clients + 1;
        if *num_clients < NR_COMMIT {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        } else if *num_clients == NR_COMMIT {
            {
                let s = &mut *self.server.lock().unwrap();
                // init the summation leaf array
                for _ in 0..s.commit_array.len() {
                    s.summation_array
                        .push(SummationEntry::Leaf(SummationLeaf::new()));
                }
                s.mc = Some(MerkleTree::from_iter(
                    s.commit_array.iter().map(|x| hash_commitment(&x.0, &x.1)),
                ));
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

        let proof_commit = {
            let s = &*self.server.lock().unwrap();
            s.mc.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        };

        future::ready(proof_commit.into())
    }
    fn aggregate_data(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        cts: Vec<u8>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) -> Self::AggregateCommitFut {
        let mut num_clients = self.cond.0.lock().unwrap();

        let mut s = self.server.lock().unwrap();

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
                let s = &mut *self.server.lock().unwrap();
                // add the whole tree
                let mut left = 0;
                let mut right = s.summation_array.len();
                while left + 1 < right {
                    let a = match s.summation_array[left].clone() {
                        SummationEntry::NonLeaf(y) => y,
                        SummationEntry::Leaf(x) => x.into(),
                    };
                    let b = match s.summation_array[left + 1].clone() {
                        SummationEntry::NonLeaf(y) => y,
                        SummationEntry::Leaf(x) => x.into(),
                    };
                    let c = a + b;
                    s.summation_array.push(SummationEntry::NonLeaf(c));
                    left += 2;
                    right += 1;
                }

                {
                    let ii = s.summation_array.len() - 1;
                    let result = match s.summation_array[ii].clone() {
                        SummationEntry::NonLeaf(x) => Some(x),
                        _ => None,
                    };
                    println!("{:?}", result.unwrap().c0[0]);
                }

                s.ms = Some(MerkleTree::from_iter(s.summation_array.iter().map(
                    |x| match x {
                        SummationEntry::Leaf(y) => y.hash(),
                        SummationEntry::NonLeaf(y) => y.hash(),
                    },
                )));
            }
            // notify all and generate the commit tree
            self.cond.1.notify_all();
        } else {
            //TODO maybe never reach here?
            assert!(false);
        }
        // unlock
        drop(num_clients);

        let proof_leaf = {
            let s = &*self.server.lock().unwrap();
            s.ms.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        };

        future::ready(proof_leaf.into())
    }
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 38886u16);

    let mut server = Server::new();
    let mut server_ref = Arc::new(Mutex::new(server));
    let mut cond_ref = Arc::new((Mutex::new(0), Condvar::new()));

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(10, |t| t.transport().peer_addr().unwrap().ip())
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
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

//fn main() {
//    // input from aggregation
//    // TODO suppose a complete binary tree for now
//    let nr_clients = 8;
//    assert!(nr_clients > 2);
//    let mut leaf_entry = vec![SummationLeaf::new(); nr_clients];
//    let commit_array: Vec<_> = leaf_entry
//        .iter()
//        .map(|x| CommitEntry::new(&x.pk, &x.c0, &x.c1, &x.r))
//        .collect();
//
//    let commit_tree: MerkleTree<[u8; 32], ExampleAlgorithm> =
//        MerkleTree::from_iter(commit_array.iter().map(|x| x.hash));
//    let commit_root = commit_tree.root();
//
//    let mut summation_array = vec![SummationNonLeaf::new(); nr_clients - 1];
//
//    //       0
//    //   1       2
//    // 3   4   5   6
//    //7 8 9 10 11 12 13 14
//    //0 1 2 3 4 5 6 7
//    // TODO 128-bit and mod addition here
//    for i in (0..nr_clients - 1).rev() {
//        // left-child
//        if 2 * i + 1 >= nr_clients - 1 {
//            // leaf
//            for k in 0..summation_array[0].c0.len() {
//                summation_array[i].c0[k] += leaf_entry[2 * i + 2 - nr_clients].c0[k];
//                summation_array[i].c1[k] += leaf_entry[2 * i + 2 - nr_clients].c1[k];
//            }
//        } else {
//            // non-leaf
//            for k in 0..summation_array[0].c0.len() {
//                summation_array[i].c0[k] += summation_array[2 * i + 1].c0[k];
//                summation_array[i].c1[k] += summation_array[2 * i + 1].c1[k];
//            }
//        };
//        if 2 * i + 2 >= nr_clients - 1 {
//            for k in 0..summation_array[0].c0.len() {
//                summation_array[i].c0[k] += leaf_entry[2 * i + 3 - nr_clients].c0[k];
//                summation_array[i].c1[k] += leaf_entry[2 * i + 3 - nr_clients].c1[k];
//            }
//        } else {
//            for k in 0..summation_array[0].c0.len() {
//                summation_array[i].c0[k] += summation_array[2 * i + 2].c0[k];
//                summation_array[i].c1[k] += summation_array[2 * i + 2].c1[k];
//            }
//        };
//    }
//
//    let summation_tree: MerkleTree<[u8; 32], ExampleAlgorithm> = MerkleTree::from_iter(
//        summation_array
//            .iter()
//            .map(|x| x.hash())
//            .chain(leaf_entry.iter().map(|f| f.hash())),
//    );
//
//    let summation_root = summation_tree.root();
//    // check_aggregate
//
//    //let (client_sender, server_receiver) = channel::<i32>();
//    let (server_sender, client_receiver) = channel::<Vec<u8>>();
//
//    // Spawn off an expensive computation
//    let t = thread::spawn(move || {
//        // published values
//        let mut nr_bytes = 0;
//        let c_root = commit_root.clone();
//        let s_root = summation_root.clone();
//
//        // check whether Commit_0 in Mc
//        if let Ok(s) = client_receiver.recv() {
//            nr_bytes += s.len();
//            println!("One proof length {}", nr_bytes);
//            // 784 for now
//            // we estimate it to be 30 * 32
//            let proof = construct_from_string(&String::from_utf8(s).unwrap());
//            //println!("{:?}", proof);
//            //println!("{:?}", proof.root());
//            //println!("{:?}", proof.item());
//            //println!("{:?}", commit_array[0].hash);
//            //TODO check hash(commit.hash) == proof.item()
//            if (proof.root() != c_root) || !proof.validate::<ExampleAlgorithm>() {
//                println!("Commit not in merkle tree");
//                std::process::exit(1);
//            }
//        }
//        if let Ok(s) = client_receiver.recv() {
//            nr_bytes += s.len();
//            // parse all nodes and verify summation
//        }
//        println!("Total number of bytes: {}", nr_bytes);
//        //TODO clients needs to send v_init and non-leaf nodes lists to the server
//    });
//
//    // commit_i in Mc
//    let proof_commit_0 = commit_tree.gen_proof(0);
//    server_sender
//        .send(format!("{:?}", proof_commit_0).as_bytes().to_vec())
//        .unwrap();
//
//    // randomly select 6 leaf nodes and 3 non-leaf nodes + 3 * 3 non-leaf nodes
//    // TODO here we just send leaf nodes 0-5 and non-leaf 3 4 5 and 0(1 2) 1(3 4) 2(5 6)
//    let mut ct_vec: Vec<u8> = Vec::new();
//    // summation array 0 - 6
//    // leaf array 7 - 14
//    for i in 0..6 {
//        ct_vec.extend(leaf_entry[i].c0.iter());
//        ct_vec.extend(leaf_entry[i].c1.iter());
//        let proof = summation_tree.gen_proof(i + 7);
//        ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
//    }
//    for i in 3..6 {
//        ct_vec.extend(summation_array[i].c0.iter());
//        ct_vec.extend(summation_array[i].c1.iter());
//        let proof = summation_tree.gen_proof(i);
//        ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
//    }
//    for i in 0..3 {
//        {
//            ct_vec.extend(summation_array[i].c0.iter());
//            ct_vec.extend(summation_array[i].c1.iter());
//            let proof = summation_tree.gen_proof(i);
//            ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
//        }
//        {
//            ct_vec.extend(summation_array[2 * i + 1].c0.iter());
//            ct_vec.extend(summation_array[2 * i + 1].c1.iter());
//            let proof = summation_tree.gen_proof(2 * i + 1);
//            ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
//        }
//        {
//            ct_vec.extend(summation_array[2 * i + 2].c0.iter());
//            ct_vec.extend(summation_array[2 * i + 2].c1.iter());
//            let proof = summation_tree.gen_proof(2 * i + 2);
//            ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
//        }
//    }
//    server_sender.send(ct_vec).unwrap();
//
//    t.join().unwrap();
//}
//
