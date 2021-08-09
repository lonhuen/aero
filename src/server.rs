extern crate crypto;
extern crate merkle_light;
extern crate rand;
extern crate rsa;

use crypto::digest::Digest;
use crypto::sha3::{Sha3, Sha3Mode};
use merkle_light::hash::{Algorithm, Hashable};
use merkle_light::merkle::MerkleTree;
use merkle_light::proof::Proof;
use rand::rngs::OsRng;
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use std::convert::{From, Into};
use std::fmt;
use std::hash::Hasher;
use std::iter::FromIterator;
use std::sync::mpsc::channel;
use std::thread;

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
#[derive(Clone, Copy)]
pub struct CommitEntry {
    hash: [u8; 32],
}

impl CommitEntry {
    pub fn new(pk: &[u8; 451], c0: &[u8; 65536], c1: &[u8; 65536], r: &[u8; 16]) -> Self {
        let mut hasher = Sha3::sha3_256();

        hasher.input(pk);
        hasher.input(c0);
        hasher.input(c1);
        hasher.input(r);

        let mut h = [0u8; 32];
        hasher.result(&mut h);
        CommitEntry { hash: h }
    }
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

fn main() {
    // input from aggregation
    // TODO suppose a complete binary tree for now
    let nr_clients = 8;
    assert!(nr_clients > 2);
    let mut leaf_entry = vec![SummationLeaf::new(); nr_clients];
    let commit_array: Vec<_> = leaf_entry
        .iter()
        .map(|x| CommitEntry::new(&x.pk, &x.c0, &x.c1, &x.r))
        .collect();

    let commit_tree: MerkleTree<[u8; 32], ExampleAlgorithm> =
        MerkleTree::from_iter(commit_array.iter().map(|x| x.hash));
    let commit_root = commit_tree.root();

    let mut summation_array = vec![SummationNonLeaf::new(); nr_clients - 1];

    //       0
    //   1       2
    // 3   4   5   6
    //7 8 9 10 11 12 13 14
    //0 1 2 3 4 5 6 7
    // TODO 128-bit and mod addition here
    for i in (0..nr_clients - 1).rev() {
        // left-child
        if 2 * i + 1 >= nr_clients - 1 {
            // leaf
            for k in 0..summation_array[0].c0.len() {
                summation_array[i].c0[k] += leaf_entry[2 * i + 2 - nr_clients].c0[k];
                summation_array[i].c1[k] += leaf_entry[2 * i + 2 - nr_clients].c1[k];
            }
        } else {
            // non-leaf
            for k in 0..summation_array[0].c0.len() {
                summation_array[i].c0[k] += summation_array[2 * i + 1].c0[k];
                summation_array[i].c1[k] += summation_array[2 * i + 1].c1[k];
            }
        };
        if 2 * i + 2 >= nr_clients - 1 {
            for k in 0..summation_array[0].c0.len() {
                summation_array[i].c0[k] += leaf_entry[2 * i + 3 - nr_clients].c0[k];
                summation_array[i].c1[k] += leaf_entry[2 * i + 3 - nr_clients].c1[k];
            }
        } else {
            for k in 0..summation_array[0].c0.len() {
                summation_array[i].c0[k] += summation_array[2 * i + 2].c0[k];
                summation_array[i].c1[k] += summation_array[2 * i + 2].c1[k];
            }
        };
    }

    let summation_tree: MerkleTree<[u8; 32], ExampleAlgorithm> = MerkleTree::from_iter(
        summation_array
            .iter()
            .map(|x| x.hash())
            .chain(leaf_entry.iter().map(|f| f.hash())),
    );

    let summation_root = summation_tree.root();
    // check_aggregate

    //let (client_sender, server_receiver) = channel::<i32>();
    let (server_sender, client_receiver) = channel::<Vec<u8>>();

    // Spawn off an expensive computation
    let t = thread::spawn(move || {
        // published values
        let mut nr_bytes = 0;
        let c_root = commit_root.clone();
        let s_root = summation_root.clone();

        // check whether Commit_0 in Mc
        if let Ok(s) = client_receiver.recv() {
            nr_bytes += s.len();
            println!("One proof length {}", nr_bytes);
            // 784 for now
            // we estimate it to be 30 * 32
            let proof = construct_from_string(&String::from_utf8(s).unwrap());
            //println!("{:?}", proof);
            //println!("{:?}", proof.root());
            //println!("{:?}", proof.item());
            //println!("{:?}", commit_array[0].hash);
            //TODO check hash(commit.hash) == proof.item()
            if (proof.root() != c_root) || !proof.validate::<ExampleAlgorithm>() {
                println!("Commit not in merkle tree");
                std::process::exit(1);
            }
        }
        if let Ok(s) = client_receiver.recv() {
            nr_bytes += s.len();
            // parse all nodes and verify summation
        }
        println!("Total number of bytes: {}", nr_bytes);
        //TODO clients needs to send v_init and non-leaf nodes lists to the server
    });

    // commit_i in Mc
    let proof_commit_0 = commit_tree.gen_proof(0);
    server_sender
        .send(format!("{:?}", proof_commit_0).as_bytes().to_vec())
        .unwrap();

    // randomly select 6 leaf nodes and 3 non-leaf nodes + 3 * 3 non-leaf nodes
    // TODO here we just send leaf nodes 0-5 and non-leaf 3 4 5 and 0(1 2) 1(3 4) 2(5 6)
    let mut ct_vec: Vec<u8> = Vec::new();
    // summation array 0 - 6
    // leaf array 7 - 14
    for i in 0..6 {
        ct_vec.extend(leaf_entry[i].c0.iter());
        ct_vec.extend(leaf_entry[i].c1.iter());
        let proof = summation_tree.gen_proof(i + 7);
        ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
    }
    for i in 3..6 {
        ct_vec.extend(summation_array[i].c0.iter());
        ct_vec.extend(summation_array[i].c1.iter());
        let proof = summation_tree.gen_proof(i);
        ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
    }
    for i in 0..3 {
        {
            ct_vec.extend(summation_array[i].c0.iter());
            ct_vec.extend(summation_array[i].c1.iter());
            let proof = summation_tree.gen_proof(i);
            ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
        }
        {
            ct_vec.extend(summation_array[2 * i + 1].c0.iter());
            ct_vec.extend(summation_array[2 * i + 1].c1.iter());
            let proof = summation_tree.gen_proof(2 * i + 1);
            ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
        }
        {
            ct_vec.extend(summation_array[2 * i + 2].c0.iter());
            ct_vec.extend(summation_array[2 * i + 2].c1.iter());
            let proof = summation_tree.gen_proof(2 * i + 2);
            ct_vec.extend(format!("{:?}", proof).as_bytes().to_vec());
        }
    }
    server_sender.send(ct_vec).unwrap();

    t.join().unwrap();
}
