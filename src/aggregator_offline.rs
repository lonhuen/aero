use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use quail::rlwe::context::{NTTContext, ShamirContext};
use quail::rlwe::NUM_DIMENSION;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::{config::ConfigUtils, log::init_tracing};
use rand::{Rng, SeedableRng};
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use threshold_secret_sharing as tss;

pub const MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];

fn serialize_shares_into(s0: &Vec<u64>, s1: &Vec<u64>, s2: &Vec<u64>, buf: &mut [u8]) {
    assert!(buf.len() >= s0.len() * 5 * 3);
    s0.iter()
        .chain(s1.iter())
        .chain(s2.iter())
        .flat_map(|x| x.to_le_bytes()[0..5].to_vec())
        .zip(buf.iter_mut())
        .for_each(|(x, y)| *y = x);
}

fn deserialize_shares(buf: &[u8]) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let nr_bytes = buf.len() / 3;
    let s0 = (0..nr_bytes)
        .step_by(5)
        .map(|x| {
            buf[x] as u64
                | ((buf[x + 1] as u64) << 8)
                | ((buf[x + 2] as u64) << 16)
                | ((buf[x + 3] as u64) << 24)
                | ((buf[x + 4] as u64) << 32)
        })
        .collect();
    let s1 = (nr_bytes..2 * nr_bytes)
        .step_by(5)
        .map(|x| {
            buf[x] as u64
                | ((buf[x + 1] as u64) << 8)
                | ((buf[x + 2] as u64) << 16)
                | ((buf[x + 3] as u64) << 24)
                | ((buf[x + 4] as u64) << 32)
        })
        .collect();
    let s2 = (2 * nr_bytes..3 * nr_bytes)
        .step_by(5)
        .map(|x| {
            buf[x] as u64
                | ((buf[x + 1] as u64) << 8)
                | ((buf[x + 2] as u64) << 16)
                | ((buf[x + 3] as u64) << 24)
                | ((buf[x + 4] as u64) << 32)
        })
        .collect();
    (s0, s1, s2)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigUtils::init("committee.yaml");
    // read the address of players
    let players: Vec<String> = config
        .settings
        .get_array("players")
        .unwrap()
        .into_iter()
        .map(|x| x.into_str().unwrap())
        .collect();
    let aggregator_addr = config.get("aggregator");
    let nr_bits: usize = config.get_int("nBits") as usize;

    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    let listener = TcpListener::bind(&aggregator_addr).await?;

    let mut shamir_context = vec![
        ShamirContext::init(MODULUS[0], nr_players, threshold),
        ShamirContext::init(MODULUS[1], nr_players, threshold),
        ShamirContext::init(MODULUS[2], nr_players, threshold),
    ];

    let ntt_context = vec![
        NTTContext::init(MODULUS[0]),
        NTTContext::init(MODULUS[1]),
        NTTContext::init(MODULUS[2]),
    ];

    // recv_bits[0..2][0..nr_players][0..all bits]
    let mut recv_bits: Vec<Vec<Vec<u64>>> = vec![vec![Vec::new(); nr_players]; 3];

    let mutex_bits = Arc::new(Mutex::new(recv_bits));
    let mb = mutex_bits.clone();

    let f = tokio::spawn(async move {
        let mut handles = Vec::new();
        for _ in 0..nr_players {
            // maybe we can new a thread for each socket to improve latency
            let (mut socket, _) = listener.accept().await.unwrap();

            let mbits = mutex_bits.clone();

            handles.push(tokio::spawn(async move {
                let mut buf = vec![0u8; nr_bits * nr_players / NUM_DIMENSION * 15 + 1];

                let n = match socket.read_exact(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                let src = buf[0] as usize;
                let (s0, s1, s2) = deserialize_shares(&buf[1..n]);
                {
                    let mut l = mbits.as_ref().lock().unwrap();
                    l[0][src].extend(s0);
                    l[1][src].extend(s1);
                    l[2][src].extend(s2);
                }
            }));
        }
        futures::future::join_all(handles).await;
    });

    f.await?;
    let rb = mb.lock().unwrap();
    shamir_context[0].threshold *= 2;
    shamir_context[1].threshold *= 2;
    shamir_context[2].threshold *= 2;
    //let ret = shamir_context[0].reconstruct(&shares);
    //println!("flag : {}", ret);
    for k in 0..3usize {
        for i in 0..nr_bits * nr_players / NUM_DIMENSION {
            let mut shares = vec![0u64; nr_players];
            for j in 0..nr_players {
                shares[j] = rb[k][j][i];
            }
            let ret = shamir_context[k].reconstruct(&shares);
            assert_eq!(ret, 0);
        }
    }

    Ok(())
}
