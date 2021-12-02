mod rlwe;
mod zksnark;
use std::borrow::Borrow;

use crate::rlwe::context::*;
//use crate::zksnark::*;
use crate::zksnark::Prover;
use crate::zksnark::ProverOffline;
use crate::zksnark::ProverOnline;
use ark_groth16::lonhh_create_proof;
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use cupcake::polyarith::lazy_ntt::{lazy_inverse_ntt_u64, lazy_ntt_u64};
use cupcake::rqpoly::RqPolyContext;
use quail::rlwe::context::MODULUS;
//use quail::rlwe::context::{self, Context};
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef,
    OptimizationGoal, Result as R1CSResult,
};
use ark_std::{end_timer, start_timer};
use quail::rlwe::context::{NTTContext, ShamirContext};
use quail::rlwe::NUM_DIMENSION;
use ring_algorithm::chinese_remainder_theorem;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::config::ConfigUtils;
use bincode::deserialize_from;
use bincode::serialize_into;
use rand::SeedableRng;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

fn main() {
    //{
    //let prover = Prover::setup("./data/encryption.txt");
    //let prover = ProverOnline::setup("./data/encryption.txt");
    //let prover = ProverOffline::setup("./data/encryption.txt");
    //}
    let config = ConfigUtils::init("config.yaml");
    let players: Vec<String> = config
        .settings
        .get_array("players")
        .unwrap()
        .into_iter()
        .map(|x| x.into_str().unwrap())
        .collect();
    // vec[player_id][0,1,2][4096]
    let shares: Vec<Vec<Vec<u64>>> = (0..players.len())
        .into_iter()
        .map(|i| {
            let file_name = format!("./data/sk_share_new{}.txt", i);
            let mut f = BufReader::new(File::open(file_name).unwrap());
            let share0: Vec<u64> = deserialize_from(&mut f).unwrap();
            let share1: Vec<u64> = deserialize_from(&mut f).unwrap();
            let share2: Vec<u64> = deserialize_from(&mut f).unwrap();
            vec![share0, share1, share2]
        })
        .collect();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;

    let shamir_context = vec![
        ShamirContext::init(MODULUS[0], nr_players, threshold),
        ShamirContext::init(MODULUS[1], nr_players, threshold),
        ShamirContext::init(MODULUS[2], nr_players, threshold),
    ];

    for i in 0..NUM_DIMENSION {
        let mut s = vec![0u64; nr_players];
        for j in 0..nr_players {
            s[j] = shares[j][0][i];
        }
        println!("id {} = {:?}", i, shamir_context[0].reconstruct(&s));
        for j in 0..nr_players {
            s[j] = shares[j][1][i];
        }
        println!("{:?}", shamir_context[1].reconstruct(&s));
        for j in 0..nr_players {
            s[j] = shares[j][2][i];
        }
        println!("{:?}", shamir_context[2].reconstruct(&s));
    }
}
