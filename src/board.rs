//use ark_ff::BigInteger256;

//mod common;
//use crate::common::{
//    aggregation::{merkle::*, CommitEntry, SummationEntry, SummationLeaf},
//    hash_commitment, new_rsa_pub_key,
//    server_service::ServerService,
//};
//mod util;
//use common::aggregation::*;
//use common::board_service::BoardService;
//use futures::{
//    future::{self, Ready},
//    prelude::*,
//};
//use tarpc::{
//    context,
//    server::{self, Channel, Incoming},
//    tokio_serde::formats::Json,
//};
//use util::config::ConfigUtils;
//use util::log::LogUtils;
///// A class for the board, which is responsible for publishing merkle root along with answering merkle proof queries.
///// Ideally it's not necessary to answer merkle proof queries. but to make the system simpler, let's ask it to answer the queries.
//pub struct Board {}
//
//impl BoardService for Board {
//    type GetMcProofFut = Ready<MerkleProof>;
//    type GetMsProofFut = Ready<MerkleProof>;
//    fn get_mc_proof(self, _: context::Context, leaf_id: u32, round: u32) -> Self::GetMcProofFut {}
//    fn get_ms_proof(self, _: context::Context, leaf_id: u32, round: u32) -> Self::GetMsProofFut {}
//}
//
//fn main() {
//    LogUtils::init("board.log");
//    ConfigUtils::init("config.ini");
//    println!("hello world!");
//}
//
mod rlwe;
use crate::rlwe::PublicKey;
use rayon::iter::repeatn;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

fn add_two_vec(a: Option<&Vec<i128>>, b: Option<&Vec<i128>>) -> Vec<i128> {
    let tmp = Vec::<i128>::new();
    let left = a.unwrap_or(&tmp);
    let right = b.unwrap_or(&tmp);
    let (longer, shorter) = if left.len() > right.len() {
        (left, right)
    } else {
        (right, left)
    };
    shorter
        .par_iter()
        .chain(repeatn(&0i128, longer.len() - shorter.len()))
        .zip(longer.par_iter())
        .map(|(x, y)| x + y)
        .collect()
}

fn main() {
    let pk = {
        let mut pk_0 = [0i128; 4096];
        let mut pk_1 = [0i128; 4096];
        let file = match File::open("./data/encryption.txt") {
            Ok(f) => f,
            Err(_) => panic!(),
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(l) = line {
                let vec = l.split(" ").collect::<Vec<&str>>();
                for i in 1..vec.len() {
                    if l.contains("pk_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_0[i - 1] = x;
                        }
                    } else if l.contains("pk_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_1[i - 1] = x;
                        }
                    }
                }
            }
        }
        PublicKey::new(&pk_0.to_vec(), &pk_1.to_vec())
    };
    pk.encrypt([0u8; 4096].to_vec());
    // Hash an input all at once.
    let hash1 = blake3::hash(b"foobarbaz");

    // Hash an input incrementally.
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"foo");
    hasher.update(b"bar");
    hasher.update(b"baz");
    let hash2: [u8; 32] = hasher.finalize().into();
    println!("{:?}", hash2);
    //let a = None;
    let a = vec![1i128; 4096];
    let d = Some(vec![2i128; 4096]);
    let b = None;
    let c = add_two_vec(Some(&a), d.as_ref());
    println!("{:?}", c);
    let c = add_two_vec(b, Some(&a));
    println!("{:?}", c);
}
