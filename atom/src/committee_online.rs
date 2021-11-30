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

fn serialize_shares_into(s0: &[u64], s1: &[u64], s2: &[u64], buf: &mut [u8]) {
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
    let aggregator_addr = config.get("aggregator");
    //let nr_bits: usize = config.get_int("nBits") as usize;

    let id = usize::from_str_radix(&args[1], 10).unwrap();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    //let listener = TcpListener::bind(&players[id]).await?;

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

    // read from file
    let mut noise: Vec<Vec<u64>> = {
        let file_name = format!("./data/noise{}.txt", id);
        let f = BufReader::new(File::open(file_name).unwrap());
        deserialize_from(f).unwrap()
    };
    let sk: Vec<Vec<u64>> = {
        let file_name = format!("./data/sk_share{}.txt", id);
        let mut f = BufReader::new(File::open(file_name).unwrap());
        let share0: Vec<u64> = deserialize_from(&mut f).unwrap();
        let share1: Vec<u64> = deserialize_from(&mut f).unwrap();
        let share2: Vec<u64> = deserialize_from(&mut f).unwrap();
        vec![share0, share1, share2]
    };
    //let ct: Vec<Vec<u64>> = {
    //    let file_name = format!("./data/ciphertext.txt");
    //    let mut f = BufReader::new(File::open(file_name).unwrap());
    //    deserialize_from(&mut f).unwrap()
    //};
    // send to aggregator
    {
        let mut stream = TcpStream::connect(&aggregator_addr).await?;
        let mut buf = vec![0u8; noise[0].len() * 5 * 3 + 1];
        // receive from aggregator
        stream.read_exact(&mut buf).await?;
        // read ciphertext first
        let ct: Vec<Vec<u64>> = {
            let (ct0, ct1, ct2) = deserialize_shares(&buf[1..]);
            vec![ct0, ct1, ct2]
        };

        // local compute and sends shares to the aggregator
        {
            for k in 0..3 {
                for j in (0..noise[0].len()).step_by(NUM_DIMENSION) {
                    let ct_sk = ntt_context[k].coeff_mul_mod(&sk[k], &ct[k]);
                    for i in 0..NUM_DIMENSION {
                        noise[k][j + i] = Scalar::add_mod(
                            &Scalar::from(noise[k][j + i]),
                            &Scalar::from(ct_sk[i]),
                            &ntt_context[k].modulus,
                        )
                        .rep();
                    }
                }
            }
        }
        buf[0] = id as u8;
        serialize_shares_into(&noise[0], &noise[1], &noise[2], &mut buf[1..]);
        stream.write_all(&buf).await?;
        stream.shutdown().await?;
    }
    Ok(())
}
