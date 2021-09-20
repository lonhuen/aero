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
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
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
        PublicKey::new(pk_0.to_vec(), pk_1.to_vec())
    };
    pk.encrypt([0i128; 4096].to_vec());
}
