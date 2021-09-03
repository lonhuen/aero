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
use server_service::hash_commitment;
use std::io;
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

#[derive(Clone, Copy)]
pub struct SummationLeaf {
    pk: [u8; 451],
    c0: [u8; 65536],
    c1: [u8; 65536],
    r: [u8; 16],
}

impl SummationLeaf {
    pub fn new() -> Self {
        let mut pk = [0u8; 451];
        let mut c0 = [0u8; 65536];
        let mut c1 = [0u8; 65536];
        let mut r = [0u8; 16];
        //for i in 0..pk.len() {
        //    pk[i] = rand::random::<u8>();
        //}
        //for i in 0..pk.len() {
        //    pk[i] = rand::random::<u8>();
        //}
        SummationLeaf { pk, c0, c1, r }
    }
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        hasher.input(&self.pk);
        hasher.input(&self.c0);
        hasher.input(&self.c1);
        hasher.input(&self.r);

        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
}

#[derive(Clone, Copy)]
pub struct SummationNonLeaf {
    c0: [u8; 65536],
    c1: [u8; 65536],
}
impl SummationNonLeaf {
    pub fn new() -> Self {
        let c0 = [0u8; 65536];
        let c1 = [0u8; 65536];
        SummationNonLeaf { c0, c1 }
    }
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        hasher.input(&self.c0);
        hasher.input(&self.c1);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
}
fn construct_from_string(s: &String) -> Proof<[u8; 32]> {
    //Proof { lemma: [
    //  [11, 64, 41, 197, 101, 253, 82, 215, 87, 50, 138, 182, 223, 8, 67, 162, 216, 101, 115, 33, 214, 82, 24, 234, 215, 229, 225, 161, 3, 184, 23, 207],
    //  [11, 64, 41, 197, 101, 253, 82, 215, 87, 50, 138, 182, 223, 8, 67, 162, 216, 101, 115, 33, 214, 82, 24, 234, 215, 229, 225, 161, 3, 184, 23, 207],
    //  [239, 199, 156, 2, 242, 253, 77, 225, 241, 85, 164, 221, 197, 192, 149, 86, 213, 243, 251, 249, 82, 103, 164, 197, 183, 102, 30, 224, 0, 197, 31, 51],
    //  [213, 115, 214, 209, 255, 144, 254, 122, 135, 131, 127, 19, 90, 54, 237, 51, 110, 11, 11, 124, 123, 10, 97, 71, 209, 246, 185, 166, 12, 73, 65, 209]],
    // path:
    // [true, true] }
    let v: Vec<&str> = s.split(&['[', ' ', ',', ']'][..]).collect();
    let mut hash = Vec::new();
    let mut path = Vec::new();
    let mut tmp = [0u8; 32];
    let mut idx = 0;
    for ss in v {
        if let Ok(t) = ss.parse::<u8>() {
            tmp[idx] = t;
            idx = idx + 1;
            if idx == 32 {
                idx = 0;
                hash.push(tmp.clone());
            }
        } else if ss.trim() == "true" {
            path.push(true);
        } else if ss.trim() == "false" {
            path.push(false);
        }
    }

    //Proof::<[u8; 32]>::new(vec![[0u8; 32]; 3], vec![true; 2])
    Proof::<[u8; 32]>::new(hash, path)
}

pub struct Server {
    pub commit_array: BTreeMap<Vec<u8>, [u8; 32]>,
    pub mc: Option<MerkleTree<[u8; 32], ExampleAlgorithm>>,
}
impl Server {
    pub fn new() -> Self {
        Self {
            commit_array: BTreeMap::new(),
            mc: None,
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
            //    let commit_tree: MerkleTree<[u8; 32], ExampleAlgorithm> =
            //        MerkleTree::from_iter(commit_array.iter().map(|x| x.hash));
            {
                let s = &mut *self.server.lock().unwrap();
                println!("commit array len {:?}", s.commit_array.len());
                s.mc = Some(MerkleTree::from_iter(
                    s.commit_array.iter().map(|x| hash_commitment(&x.0, &x.1)),
                ));
            }
            // notify all and generate the commit tree
            self.cond.1.notify_all();
        } else {
            //TODO maybe never reach here?
            assert!(false);
        }
        // unlock
        drop(num_clients);

        let nr_clients: usize = NR_COMMIT.try_into().unwrap();

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
        //let mut num_clients = self.cond.0.lock().unwrap();

        //let s = self.server.lock().unwrap();

        //// TODO no redundant should be sent

        //if !s.commit_array.contains_key(&rsa_pk) {
        //    // TODO should not reach here
        //    assert!(false);
        //}

        //let idx = *num_clients;

        //// push into the server
        //{
        //    self.server
        //        .lock()
        //        .unwrap()
        //        .commit_array
        //        .insert(rsa_pk, commitment);
        //}

        //*num_clients = *num_clients + 1;
        //if *num_clients < NR_COMMIT {
        //    num_clients = self.cond.1.wait(num_clients).unwrap();
        //} else if *num_clients == NR_COMMIT {
        //    //    let commit_tree: MerkleTree<[u8; 32], ExampleAlgorithm> =
        //    //        MerkleTree::from_iter(commit_array.iter().map(|x| x.hash));
        //    {
        //        let s = &mut *self.server.lock().unwrap();
        //        println!("commit array len {:?}", s.commit_array.len());
        //        s.mc = Some(MerkleTree::from_iter(
        //            s.commit_array.iter().map(|x| hash_commitment(&x.0, &x.1)),
        //        ));
        //    }
        //    // notify all and generate the commit tree
        //    self.cond.1.notify_all();
        //} else {
        //    //TODO maybe never reach here?
        //    assert!(false);
        //}
        //// unlock
        //drop(num_clients);

        //let nr_clients: usize = NR_COMMIT.try_into().unwrap();

        //let proof_commit = {
        //    let s = &*self.server.lock().unwrap();
        //    s.mc.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        //};

        //future::ready(proof_commit.into())
        //let nr_clients = 8;
        //assert!(nr_clients > 2);
        //let mut leaf_entry = vec![SummationLeaf::new(); nr_clients];
        //let commit_array: Vec<_> = leaf_entry
        //    .iter()
        //    .map(|x| CommitEntry::new(&x.pk, &x.c0, &x.c1, &x.r))
        //    .collect();

        //let commit_tree: MerkleTree<[u8; 32], ExampleAlgorithm> =
        //    MerkleTree::from_iter(commit_array.iter().map(|x| x.hash));
        //let proof_commit_0 = commit_tree.gen_proof(0);
        future::ready(
            MerkleProof {
                lemma: Vec::new(),
                path: Vec::new(),
            }
            .into(),
        )
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
