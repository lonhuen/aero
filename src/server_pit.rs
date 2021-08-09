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
use std::convert::{From, Into, TryInto};
use std::fmt;
use std::hash::Hasher;
use std::iter::FromIterator;
use std::ops::Add;
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

impl Add for SummationLeaf {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let left_c0: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(self.c0[i..i + 16].try_into().expect("uncorrected length"))
                    % MODULUS
            })
            .collect();
        let left_c1: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(self.c1[i..i + 16].try_into().expect("uncorrected length"))
                    % MODULUS
            })
            .collect();
        let right_c0: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(other.c0[i..i + 16].try_into().expect("uncorrected length"))
                    % MODULUS
            })
            .collect();
        let right_c1: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(other.c1[i..i + 16].try_into().expect("uncorrected length"))
                    % MODULUS
            })
            .collect();

        let result_c0: Vec<u8> = left_c0
            .iter()
            .zip(right_c0.iter())
            .map(|x| (x.0 + x.1) % MODULUS)
            .map(|x| i128::to_le_bytes(x))
            .flatten()
            .collect();
        let result_c1: Vec<u8> = left_c1
            .iter()
            .zip(right_c1.iter())
            .map(|x| (x.0 + x.1) % MODULUS)
            .map(|x| i128::to_le_bytes(x))
            .flatten()
            .collect();

        SummationLeaf {
            pk: self.pk,
            c0: result_c0.try_into().expect("uncorrected length"),
            c1: result_c1.try_into().expect("uncorrected length"),
            r: self.r,
        }
    }
}

impl SummationLeaf {
    pub fn zero() -> Self {
        let pk = [0u8; 451];
        let c0 = [0u8; 65536];
        let c1 = [0u8; 65536];
        let r = [0u8; 16];
        SummationLeaf { pk, c0, c1, r }
    }
    pub fn new() -> Self {
        let pk = [0u8; 451];
        let mut c0 = [0u128; 4096];
        let mut c1 = [0u128; 4096];
        let r = [0u8; 16];
        for i in 0..c0.len() {
            c0[i] = rand::random::<u32>().into();
        }
        for i in 0..c1.len() {
            c1[i] = rand::random::<u32>().into();
        }
        let vc0: Vec<u8> = c0.iter().map(|x| x.to_le_bytes()).flatten().collect();
        let vc1: Vec<u8> = c1.iter().map(|x| x.to_le_bytes()).flatten().collect();
        SummationLeaf {
            pk: pk,
            c0: vc0.try_into().expect("not correct length"),
            c1: vc1.try_into().expect("not correct length"),
            r: r,
        }
    }
    pub fn new_with_poly(c0: &[i128; 4096], c1: &[i128; 4096]) -> Self {
        let pk = [0u8; 451];
        let r = [0u8; 16];
        let vc0: Vec<u8> = c0
            .to_vec()
            .iter()
            .map(|x| i128::to_le_bytes(*x))
            .flatten()
            .collect();
        let vc1: Vec<u8> = c1
            .to_vec()
            .iter()
            .map(|x| i128::to_le_bytes(*x))
            .flatten()
            .collect();
        SummationLeaf {
            pk: pk,
            r: r,
            c0: vc0.try_into().expect("uncorrected length"),
            c1: vc1.try_into().expect("uncorrected length"),
        }
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
    pub fn display(&self) {
        {
            let mut v = Vec::new();
            for i in (0..65536).step_by(16) {
                v.push(i128::from_le_bytes(
                    self.c0[i..i + 16]
                        .try_into()
                        .expect("slice with incorrect length"),
                ));
            }
            println!("c0 {:?}", v);
        }
        {
            let mut v = Vec::new();
            for i in (0..65536).step_by(16) {
                v.push(i128::from_le_bytes(
                    self.c1[i..i + 16]
                        .try_into()
                        .expect("slice with incorrect length"),
                ));
            }
            println!("c1 {:?}", v);
        }
        println!("{:?}", self.c0[0]);
    }
}

#[derive(Clone, Copy)]
pub struct SummationNonLeaf {
    c0: [u8; 16],
    c1: [u8; 16],
}
impl SummationNonLeaf {
    pub fn new() -> Self {
        let c0 = [0u8; 16];
        let c1 = [0u8; 16];
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

//TODO consider overflow
pub fn evaluate_at(poly: &[i128], x: i128) -> i128 {
    assert!(x > 0 && x < MODULUS);
    let mut ret = 0i128;
    let mut exp = 1i128;
    for i in 0..poly.len() {
        ret = (ret + poly[i] * exp) % MODULUS;
        exp = (exp * x) % MODULUS;
    }
    ret
}

impl Add for SummationNonLeaf {
    type Output = SummationNonLeaf;
    fn add(self, other: Self) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: i128::to_le_bytes(
                (i128::from_le_bytes(self.c0) + i128::from_le_bytes(other.c0)) % MODULUS,
            ),
            c1: i128::to_le_bytes(
                (i128::from_le_bytes(self.c1) + i128::from_le_bytes(other.c1)) % MODULUS,
            ),
        }
    }
}

impl From<SummationLeaf> for SummationNonLeaf {
    fn from(leaf: SummationLeaf) -> SummationNonLeaf {
        let poly0: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(leaf.c0[i..i + 16].try_into().expect("uncorrected length"))
                    % MODULUS
            })
            .collect();
        let poly1: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(leaf.c1[i..i + 16].try_into().expect("uncorrected length"))
                    % MODULUS
            })
            .collect();
        let c0 = evaluate_at(&poly0, RANDOM_POINT);
        let c1 = evaluate_at(&poly1, RANDOM_POINT);
        SummationNonLeaf {
            c0: c0.to_le_bytes(),
            c1: c1.to_le_bytes(),
        }
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

const RANDOM_POINT: i128 = 10;
//const MODULUS: i128 = 649033470896967801447398927572993i128;
const MODULUS: i128 = 0xffffee001;

fn main() {
    // input from aggregation
    // TODO suppose a complete binary tree for now
    let nr_clients = 8;
    assert!(nr_clients > 2);
    //let poly = [MODULUS + 1; 4096];
    //let mut leaf_entry = vec![SummationLeaf::new_with_poly(&poly, &poly); nr_clients];
    let leaf_entry = vec![SummationLeaf::new(); nr_clients];
    let commit_array: Vec<_> = leaf_entry
        .iter()
        .map(|x| CommitEntry::new(&x.pk, &x.c0, &x.c1, &x.r))
        .collect();

    let commit_tree: MerkleTree<[u8; 32], ExampleAlgorithm> =
        MerkleTree::from_iter(commit_array.iter().map(|x| x.hash));
    let commit_root = commit_tree.root();

    let mut summation_array = vec![SummationNonLeaf::new(); nr_clients - 1];

    // test purpose
    // let mut leaf_sum = SummationLeaf::zero();
    // for i in 0..leaf_entry.len() {
    //     leaf_sum = leaf_sum + leaf_entry[i];
    // }
    // let non_leaf_sum: SummationNonLeaf = leaf_sum.into();
    // println!(
    //     "{} {}",
    //     i128::from_le_bytes(non_leaf_sum.c0),
    //     i128::from_le_bytes(non_leaf_sum.c1)
    // );
    //let mut s0 = 0i128;
    //let mut s1 = 0i128;
    //for i in 0..leaf_entry.len() {
    //    let non_leaf_entry: SummationNonLeaf = leaf_entry[i].into();
    //    s0 += i128::from_le_bytes(non_leaf_entry.c0);
    //    s1 += i128::from_le_bytes(non_leaf_entry.c1);
    //}
    //println!("{} {}", s0 % MODULUS, s1 % MODULUS);

    //       0
    //   1       2
    // 3   4   5   6
    //7 8 9 10 11 12 13 14
    //0 1 2 3 4 5 6 7
    // TODO 128-bit and mod addition here
    for i in (0..nr_clients - 1).rev() {
        // left-child
        if 2 * i + 1 >= nr_clients - 1 {
            // leaf + leaf
            summation_array[i] = summation_array[i] + leaf_entry[2 * i + 2 - nr_clients].into();
        } else {
            // non-leaf
            summation_array[i] = summation_array[i] + summation_array[2 * i + 1];
        };
        if 2 * i + 2 >= nr_clients - 1 {
            summation_array[i] = summation_array[i] + leaf_entry[2 * i + 3 - nr_clients].into();
        } else {
            summation_array[i] = summation_array[i] + summation_array[2 * i + 2];
        };
    }

    // for test purpose
    //println!(
    //    "{} {}",
    //    i128::from_le_bytes(summation_array[0].c0),
    //    i128::from_le_bytes(summation_array[0].c1),
    //);

    let summation_tree: MerkleTree<[u8; 32], ExampleAlgorithm> = MerkleTree::from_iter(
        summation_array
            .iter()
            .map(|x| x.hash())
            .chain(leaf_entry.iter().map(|f| f.hash())),
    );

    let summation_root = summation_tree.root();
    //// check_aggregate

    ////let (client_sender, server_receiver) = channel::<i32>();
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
