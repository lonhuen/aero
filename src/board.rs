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
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::main]
async fn main() {
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();

    let notified1 = notify.notified();
    let notified2 = notify.notified();

    let handle = tokio::spawn(async move {
        println!("sending notifications");
        notify2.notify_waiters();
    });

    notified1.await;
    notified2.await;
    println!("received notifications");
    let notified3 = notify.notified();
    notified3.await;
    println!("received 3rd notifications");
}
