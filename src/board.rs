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
use crate::rlwe::context::*;
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use cupcake::polyarith::lazy_ntt::{lazy_inverse_ntt_u64, lazy_ntt_u64};
use cupcake::rqpoly::RqPolyContext;
//use quail::rlwe::context::{self, Context};
use ring_algorithm::chinese_remainder_theorem;
use threshold_secret_sharing as tss;

fn main() {
    // let context = Context::init_default();

    // //let mut a: Vec<u64> = vec![1u64; 4096];
    // //let mut aa = Vec::new();
    // //aa.push(a);
    // //context.lazy_ntt_inplace(&mut aa);
    // //context.lazy_inverse_ntt_inplace(&mut aa);
    // //println!("{:?}", aa[0]);

    // let mut a: Vec<i128> = vec![1i128; 4096];
    // let mut acrt = context.crt_encode_vec(&a);
    // context.lazy_ntt_inplace(&mut acrt);

    //let mut b: Vec<i128> = vec![1i128; 4096];
    ////b[0] = 1;
    //let mut bcrt = context.crt_encode_vec(&b);
    //context.lazy_ntt_inplace(&mut acrt);
    //context.lazy_ntt_inplace(&mut bcrt);
    //let mut c_ntt = context.coeffwise_mult(&acrt, &bcrt);
    //context.lazy_inverse_ntt_inplace(&mut c_ntt);
    //let c = context.crt_decode_vec(&c_ntt);
    //println!("{:?}", c);

    //let c = context.crt_decode_vec(&b);
    //let q = Scalar::new_modulus(0xffffee001u64);
    //let context = RqPolyContext::new(4096, &q);
    //let mut a: Vec<u64> = vec![1u64; 4096];
    ////    let mut b: Vec<u64> = vec![0u64; 4096];
    ////    b[0] = 1;
    //let roots_u64: Vec<u64> = context.roots.iter().map(|elm| elm.rep()).collect();
    //let scaledroots_u64: Vec<u64> = context.scaled_roots.iter().map(|elm| elm.rep()).collect();
    //let invroots_u64: Vec<u64> = context.invroots.iter().map(|elm| elm.rep()).collect();
    //let scaled_invroots_u64: Vec<u64> = context
    //    .scaled_invroots
    //    .iter()
    //    .map(|elm| elm.rep())
    //    .collect();
    ////
    //let ninv = Scalar::inv_mod(&Scalar::from_u32(4096 as u32, &q), &q);
    //lazy_ntt_u64(&mut a, &roots_u64, &scaledroots_u64, q.rep());
    ////    lazy_ntt_u64(&mut b, &roots_u64, &scaledroots_u64, q.rep());
    ////
    //let sa: Vec<Scalar> = a
    //    .iter()
    //    //.map(|x| Scalar::modulus(&Scalar::from(*x), &q))
    //    .map(|x| Scalar::from(*x))
    //    .collect();
    ////    let sb: Vec<Scalar> = b
    ////        .iter()
    ////        .map(|x| Scalar::modulus(&Scalar::from(*x), &q))
    ////        .collect();
    ////
    //let mut sc: Vec<u64> = sa.iter().map(|x| x.rep()).collect();
    ////        .zip(sb.iter())
    ////        .map(|(x, y)| Scalar::mul_mod(x, y, &q).rep())
    ////        .collect();
    //lazy_inverse_ntt_u64(&mut sc, &invroots_u64, &scaled_invroots_u64, q.rep());
    //sc.iter_mut().for_each(|x| {
    //    *x = Scalar::mul_mod(&ninv, &Scalar::modulus(&Scalar::from(*x), &q), &q).rep()
    //});
    //println!("{:?}", sc);
    //    // create instance of the Shamir scheme
    //    let ref tss = tss::shamir::ShamirSecretSharing {
    //        threshold: 8,    // privacy threshold
    //        share_count: 20, // total number of shares to generate
    //        prime: 41,       // prime field to use
    //    };
    //
    //    let secret = 5;
    //
    //    // generate shares for secret
    //    let all_shares = tss.share(secret);
    //
    //    // artificially remove some of the shares
    //    let number_of_recovered_shared = 10;
    //    assert!(number_of_recovered_shared >= tss.reconstruct_limit());
    //    let recovered_indices: Vec<usize> = (0..number_of_recovered_shared).collect();
    //    let recovered_shares: &[i64] = &all_shares[0..number_of_recovered_shared];
    //
    //    // reconstruct using remaining subset of shares
    //    let reconstructed_secret = tss.reconstruct(&recovered_indices, recovered_shares);
    //    assert_eq!(reconstructed_secret, secret);
}
