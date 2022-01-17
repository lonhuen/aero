mod rlwe;
mod zksnark;
use std::borrow::Borrow;
use std::io::BufRead;

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
use quail::zksnark::Verifier;
use ring_algorithm::chinese_remainder_theorem;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::rlwe::PublicKey;
use crate::util::config::ConfigUtils;
use bincode::deserialize_from;
use bincode::serialize_into;
use rand::SeedableRng;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

fn main() {
    //{
    let prover = Prover::setup("./data/encryption.txt");
    let enc_pk = {
        let (pk0, pk1) = {
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
            (pk_0.to_vec(), pk_1.to_vec())
        };
        PublicKey::new(&pk0, &pk1)
    };
    let (r, e0, e1, d0, d1, ct) = enc_pk.encrypt(&[0u8; 4096]);
    //let prover = Prover::new("./data/encryption.txt", "./data/proving_key.txt");
    let mut c0s = Vec::new();
    let mut c1s = Vec::new();
    let mut rs = Vec::new();
    let mut e0s = Vec::new();
    let mut e1s = Vec::new();
    let mut d0s = Vec::new();
    let mut d1s = Vec::new();
    let mut m = Vec::new();
    rs.push(r);
    e0s.push(e0);
    e1s.push(e1);
    d0s.push(d0);
    d1s.push(d1);
    c0s.push(ct.c_0);
    c1s.push(ct.c_1);
    m.push(vec![0i128; 4096]);
    let ret = prover.create_proof_in_bytes(&c0s, &c1s, &rs, &e0s, &e1s, &d0s, &d1s, &m);
    println!("{:?}", ret[0].len());
    let verifier = Verifier::new("./data/verifying_key.txt");
    let inputs: Vec<i128> = c0s[0]
        .iter()
        .cloned()
        .chain(c1s[0].iter().cloned())
        .collect();
    {
        let mut file = File::create("./data/proof.txt").unwrap();
        // Write a slice of bytes to the file
        file.write_all(&ret[0]).unwrap();
    }
    {
        let mut f = BufWriter::new(File::create("./data/ct0.txt").unwrap());
        serialize_into(&mut f, &c0s[0]).unwrap();
    }
    {
        let mut f = BufWriter::new(File::create("./data/ct1.txt").unwrap());
        serialize_into(&mut f, &c1s[0]).unwrap();
    }
    let ct0: Vec<i128> = {
        let mut f = BufReader::new(File::open("./data/ct0.txt").unwrap());
        deserialize_from(&mut f).unwrap()
    };
    let ct1: Vec<i128> = {
        let mut f = BufReader::new(File::open("./data/ct1.txt").unwrap());
        deserialize_from(&mut f).unwrap()
    };
    let inputs: Vec<i128> = ct0.iter().cloned().chain(ct1.iter().cloned()).collect();
    let mut file = File::open("./data/proof.txt").unwrap();
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer).unwrap();
    let result = verifier.verify_proof_from_bytes(&buffer, &inputs);
    println!("{}", result);
    //let prover = ProverOnline::setup("./data/encryption.txt");
    //let prover = ProverOffline::setup("./data/encryption.txt");
    //}
    //let config = ConfigUtils::init("config.yaml");
    //let players: Vec<String> = config
    //    .settings
    //    .get_array("players")
    //    .unwrap()
    //    .into_iter()
    //    .map(|x| x.into_str().unwrap())
    //    .collect();
    //// vec[player_id][0,1,2][4096]
    //let shares: Vec<Vec<Vec<u64>>> = (0..players.len())
    //    .into_iter()
    //    .map(|i| {
    //        let file_name = format!("./data/sk_share_new{}.txt", i);
    //        let mut f = BufReader::new(File::open(file_name).unwrap());
    //        let share0: Vec<u64> = deserialize_from(&mut f).unwrap();
    //        let share1: Vec<u64> = deserialize_from(&mut f).unwrap();
    //        let share2: Vec<u64> = deserialize_from(&mut f).unwrap();
    //        vec![share0, share1, share2]
    //    })
    //    .collect();
    //let nr_players = players.len();
    //let threshold = config.get_int("threshold") as usize;

    //let shamir_context = vec![
    //    ShamirContext::init(MODULUS[0], nr_players, threshold),
    //    ShamirContext::init(MODULUS[1], nr_players, threshold),
    //    ShamirContext::init(MODULUS[2], nr_players, threshold),
    //];

    //for i in 0..NUM_DIMENSION {
    //    let mut s = vec![0u64; nr_players];
    //    for j in 0..nr_players {
    //        s[j] = shares[j][0][i];
    //    }
    //    println!("id {} = {:?}", i, shamir_context[0].reconstruct(&s));
    //    for j in 0..nr_players {
    //        s[j] = shares[j][1][i];
    //    }
    //    println!("{:?}", shamir_context[1].reconstruct(&s));
    //    for j in 0..nr_players {
    //        s[j] = shares[j][2][i];
    //    }
    //    println!("{:?}", shamir_context[2].reconstruct(&s));
    //}
}
