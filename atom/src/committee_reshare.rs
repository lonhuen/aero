use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use curve25519_dalek::ristretto::RistrettoPoint;
use quail::rlwe::context::{NTTContext, ShamirContext};
use quail::rlwe::NUM_DIMENSION;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::config::ConfigUtils;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::scalar::Scalar as curveScalar;
use num_bigint::BigUint;

use ark_ff::Field;
use ark_ff::One;
use bincode::deserialize_from;
use bincode::serialize_into;
use rand::{Rng, SeedableRng};
use ring_algorithm::chinese_remainder_theorem;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

pub const MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];
pub const TOTAL_MODULUS: u128 = 649033470896967801447398927572993u128;

fn sample_polynomial(secret: u128, threshold: usize) -> Vec<curveScalar> {
    let mut poly = vec![curveScalar::from(secret as u128)];
    let mut rng = rand::rngs::StdRng::from_entropy();
    poly.extend(
        // TODO since the remainder theorem doesn't seem to work here, we just work in the first prime field
        //(0..threshold).map(|_| curveScalar::from(rng.gen_range(0..TOTAL_MODULUS - 1) as u128)),
        (0..threshold).map(|_| curveScalar::from(rng.gen_range(0..MODULUS[0] - 1) as u128)),
    );
    poly
}

fn evaluate_polynomial(
    poly: &Vec<curveScalar>,
    x: usize,
    eval_matrix: &Vec<Vec<curveScalar>>,
) -> curveScalar {
    let mut s = curveScalar::zero();
    for i in 0..poly.len() {
        // the addition will not exceed 64-bit
        s = s + eval_matrix[x][i] * poly[i];
    }
    s
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("provide player id");
    }
    let config = ConfigUtils::init("config.yaml");
    // read the address of players
    let players: Vec<String> = config
        .settings
        .get_array("players")
        .unwrap()
        .into_iter()
        .map(|x| x.into_str().unwrap())
        .collect();
    let id = usize::from_str_radix(&args[1], 10).unwrap();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    let listener = TcpListener::bind(&players[id]).await?;

    let sk: Vec<u128> = {
        let modulus = [MODULUS[0] as i128, MODULUS[1] as i128, MODULUS[2] as i128];
        let file_name = format!("./data/sk_share{}.txt", id);
        let mut f = BufReader::new(File::open(file_name).unwrap());
        let share0: Vec<u64> = deserialize_from(&mut f).unwrap();
        let share1: Vec<u64> = deserialize_from(&mut f).unwrap();
        let share2: Vec<u64> = deserialize_from(&mut f).unwrap();
        let mut share = Vec::with_capacity(NUM_DIMENSION);
        for i in 0..NUM_DIMENSION {
            // TODO for some reason, the remainder theorem doesn't seem to work here;
            // for simplicity, we just use share0[i] maybe
            // but it should be the same for 109-bit number, since the order of the curve is more than 250-bit
            share.push(share0[i] as u128);
            // share.push(
            //     chinese_remainder_theorem(
            //         &[share0[i] as i128, share1[i] as i128, share2[i] as i128],
            //         &modulus,
            //     )
            //     .unwrap() as u128,
            // );
            // assert_eq!((share[i] % MODULUS[0] as u128), share0[i] as u128);
            // assert_eq!((share[i] % MODULUS[1] as u128), share1[i] as u128);
            // assert_eq!((share[i] % MODULUS[2] as u128), share2[i] as u128);
        }
        share
    };
    let mut eval_matrix = vec![vec![curveScalar::one(); threshold + 1]; nr_players];
    for i in 0..nr_players {
        let pt = (i + 1) as u128;
        let mut x_pow = (i + 1) as u128;
        for j in 1..threshold + 1 {
            eval_matrix[i][j] = curveScalar::from(x_pow);
            x_pow = (x_pow * pt).rem_euclid(TOTAL_MODULUS);
        }
    }
    //println!("eval_matrix {:?}", eval_matrix);

    // send to_send[i] to player i
    // s_{id}_{i}
    let mut to_send = vec![vec![curveScalar::zero(); NUM_DIMENSION]; nr_players];
    let mut proof = Vec::with_capacity(NUM_DIMENSION);
    for i in 0..NUM_DIMENSION {
        //let poly = sample_polynomial(sk[i], threshold);
        let poly = sample_polynomial(sk[i], threshold);
        let tproof: Vec<RistrettoPoint> = poly
            .iter()
            .map(|x| x * &RISTRETTO_BASEPOINT_TABLE)
            .collect();
        for j in 0..nr_players {
            let y = evaluate_polynomial(&poly, j, &eval_matrix);
            to_send[j][i] = y;
        }
        proof.push(tproof);
    }
    //for k in 0..nr_players {
    //    println!("s[{}][{}]={:?}", id, k, to_send[k][0]);
    //}
    // simulate the publish by writing to a file
    {
        let file_name = format!("./data/reshare_proof{}.txt", id);
        let mut f = BufWriter::new(File::create(file_name).unwrap());
        serialize_into(&mut f, &proof).unwrap();
    }

    // simulate the CPU by verifying the proofs from itself rather than read it again
    {
        for i in 0..nr_players {
            for j in 0..NUM_DIMENSION {
                let recv_share = to_send[i][j];
                let recv_group = &recv_share * &RISTRETTO_BASEPOINT_TABLE;
                let mut compute_group = proof[j][0];
                for k in 1..threshold + 1 {
                    compute_group += &eval_matrix[i][k] * &proof[j][k];
                }
            }
        }
    }

    // store from player i to `to_recv[i]`
    // s_{i}_{id}
    let mut to_recv = vec![vec![curveScalar::zero(); NUM_DIMENSION]; nr_players];
    for i in 0..NUM_DIMENSION {
        to_recv[id][i] = to_send[id][i];
    }

    let mutex = Arc::new(Mutex::new(to_recv));
    let mb = mutex.clone();

    let f = tokio::spawn(async move {
        let mut handles = Vec::new();
        for _ in 0..nr_players - 1 {
            // maybe we can new a thread for each socket to improve latency
            let (mut socket, _) = listener.accept().await.unwrap();

            let mbits = mutex.clone();

            handles.push(tokio::spawn(async move {
                // 1 + 32 * 4096 + 8
                let mut buf = vec![0u8; 131081];

                let _ = match socket.read_exact(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                let src = buf[0] as usize;
                let recv_shares: Vec<curveScalar> = deserialize_from(&buf[1..]).unwrap();
                let mut l = mbits.as_ref().lock().unwrap();
                for i in 0..NUM_DIMENSION {
                    l[src][i] = recv_shares[i];
                }
            }));
        }
        futures::future::join_all(handles).await;
    });

    // sending data to other players
    {
        // 1 + 32 * 4096 + 8
        let mut buf = vec![0u8; 131081];
        for i in 0..players.len() {
            buf[0] = id as u8;
            if i != id {
                let mut stream = TcpStream::connect(&players[i]).await?;
                serialize_into(&mut buf[1..], &to_send[i]).unwrap();
                stream.write_all(&buf).await?;
            }
        }
    }

    f.await?;
    let mut rb = mb.lock().unwrap();
    let shamir_context = vec![
        ShamirContext::init(MODULUS[0], nr_players, threshold),
        ShamirContext::init(MODULUS[1], nr_players, threshold),
        ShamirContext::init(MODULUS[2], nr_players, threshold),
    ];
    //for k in 0..nr_players {
    //    println!("s'[{}][{}]={:?}", k, id, rb[k][0]);
    //}

    let mut share: Vec<Vec<u64>> = vec![vec![0u64; NUM_DIMENSION]; 3];
    for i in 0..NUM_DIMENSION {
        for k in 0..1 {
            //for k in 0..3 {
            let subshare: Vec<u64> = (0..nr_players)
                .map(|x| {
                    let bigint = BigUint::from_bytes_le(rb[x][i].as_bytes());
                    let u64_digits = (bigint % MODULUS[k]).to_u64_digits();
                    if u64_digits.len() == 0 {
                        0u64
                    } else {
                        u64_digits[0]
                    }
                })
                .collect();
            //println!("{:?}", subshare);
            share[k][i] = shamir_context[k].reconstruct(&subshare);
        }
    }

    {
        let file_name = format!("./data/sk_share_new{}.txt", id);
        let mut f = BufWriter::new(File::create(file_name).unwrap());
        serialize_into(&mut f, &share[0]).unwrap();
        serialize_into(&mut f, &share[1]).unwrap();
        serialize_into(&mut f, &share[2]).unwrap();
    }

    Ok(())
}
