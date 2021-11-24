use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use quail::rlwe::context::{NTTContext, ShamirContext};
use quail::rlwe::NUM_DIMENSION;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::config::ConfigUtils;
use bincode::deserialize_from;
use bincode::serialize_into;
use rand::SeedableRng;
use ring_algorithm::chinese_remainder_theorem;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

pub const MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];

fn main() {
    let config = ConfigUtils::init("committee.yaml");
    // read the address of players
    let players: Vec<String> = config
        .settings
        .get_array("players")
        .unwrap()
        .into_iter()
        .map(|x| x.into_str().unwrap())
        .collect();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    let shamir_context = vec![
        ShamirContext::init(MODULUS[0], nr_players, threshold),
        ShamirContext::init(MODULUS[1], nr_players, threshold),
        ShamirContext::init(MODULUS[2], nr_players, threshold),
    ];

    let ntt_context = vec![
        NTTContext::init(MODULUS[0]),
        NTTContext::init(MODULUS[1]),
        NTTContext::init(MODULUS[2]),
    ];

    // first randomly generate the secret key
    // to verify correctness, the secret key now is [1, 0, ..., 0]
    let mut sk = vec![vec![0u64; 4096]; 3];
    sk[0][0] = 1u64;
    sk[1][0] = 1u64;
    sk[2][0] = 1u64;

    // first randomly sample A , e
    let pk_a: Vec<Vec<u64>> = vec![
        (0..NUM_DIMENSION)
            .into_iter()
            .map(|_| Scalar::sample_blw(&shamir_context[0].modulus).rep())
            //.map(|x| if x == 0 { 1u64 } else { 0u64 })
            .collect(),
        (0..NUM_DIMENSION)
            .into_iter()
            .map(|_| Scalar::sample_blw(&shamir_context[1].modulus).rep())
            .collect(),
        (0..NUM_DIMENSION)
            .into_iter()
            .map(|_| Scalar::sample_blw(&shamir_context[2].modulus).rep())
            .collect(),
    ];

    // instead of randomly sampling, let's just use [1,..,1] for simplicity
    let e: Vec<Vec<u64>> = vec![vec![1u64; 4096]; 3];
    // compute A * s + e
    let mut pk_b = vec![
        ntt_context[0].poly_mul(&pk_a[0], &sk[0]),
        ntt_context[1].poly_mul(&pk_a[1], &sk[1]),
        ntt_context[2].poly_mul(&pk_a[2], &sk[2]),
    ];
    for k in 0..3 {
        for i in 0..4096 {
            pk_b[k][i] = Scalar::add_mod(
                &Scalar::from(pk_b[k][i]),
                &Scalar::from(e[k][i]),
                &ntt_context[k].modulus,
            )
            .rep();
        }
    }

    // TODO generate pk_0 in i128 by crt decoding
    // TODO generate pk_1 in i128 by crt decoding
    // maybe do this via python

    // shamir share the secret key
    ntt_context[0].lazy_ntt_inplace(&mut sk[0]);
    ntt_context[1].lazy_ntt_inplace(&mut sk[1]);
    ntt_context[2].lazy_ntt_inplace(&mut sk[2]);
    let mut shares = vec![vec![vec![0u64; NUM_DIMENSION]; nr_players]; 3];
    for i in 0..NUM_DIMENSION {
        let ss0 = shamir_context[0].share(sk[0][i]);
        let ss1 = shamir_context[1].share(sk[1][i]);
        let ss2 = shamir_context[2].share(sk[2][i]);
        for j in 0..nr_players {
            shares[0][j][i] = ss0[j];
            shares[1][j][i] = ss1[j];
            shares[2][j][i] = ss2[j];
        }
    }
    // write to the files
    for k in 0..nr_players {
        let file_name = format!("./data/sk_share{}.txt", k);
        let mut f = BufWriter::new(File::create(file_name).unwrap());
        serialize_into(&mut f, &shares[0][k]).unwrap();
        serialize_into(&mut f, &shares[1][k]).unwrap();
        serialize_into(&mut f, &shares[2][k]).unwrap();
    }
    // write the public ct into a file
    {
        let file_name = format!("./data/ciphertext.txt");
        let mut f = BufWriter::new(File::create(file_name).unwrap());
        let mut ct = vec![vec![1u64; 4096]; 3];
        ntt_context[0].lazy_ntt_inplace(&mut ct[0]);
        ntt_context[1].lazy_ntt_inplace(&mut ct[1]);
        ntt_context[2].lazy_ntt_inplace(&mut ct[2]);
        serialize_into(&mut f, &ct).unwrap();
    }
}
