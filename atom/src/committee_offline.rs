use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use quail::rlwe::context::{NTTContext, ShamirContext};
use quail::rlwe::NUM_DIMENSION;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::config::ConfigUtils;
use bincode::serialize_into;
use ring_algorithm::chinese_remainder_theorem;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];
pub const NOISE_MODULUS: [u64; 2] = [0x7ffffec001, 0x8000016001];

fn serialize_shares_into(s0: &Vec<u64>, s1: &Vec<u64>, buf: &mut [u8]) {
    assert!(buf.len() >= s0.len() * 5 * 2);
    s0.iter()
        .chain(s1.iter())
        .flat_map(|x| x.to_le_bytes()[0..5].to_vec())
        .zip(buf.iter_mut())
        .for_each(|(x, y)| *y = x);
}

fn deserialize_shares(buf: &[u8]) -> (Vec<u64>, Vec<u64>) {
    let nr_bytes = buf.len() / 2;
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
    (s0, s1)
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
    let nr_bits = config.get_int("nr_parameter_committee") as usize;

    let id = usize::from_str_radix(&args[1], 10).unwrap();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    let listener = TcpListener::bind(&players[id]).await?;

    let shamir_context = vec![
        ShamirContext::init(NOISE_MODULUS[0], nr_players, threshold),
        ShamirContext::init(NOISE_MODULUS[1], nr_players, threshold),
    ];

    let start = Instant::now();

    // first generate enough number of bits
    let gc = start_timer!(|| "generate shamir sharing");
    let mut shares = vec![vec![vec![0u64; nr_bits]; nr_players]; 2];
    {
        for i in 0..nr_bits {
            let ss0 = shamir_context[0].share(Scalar::sample_blw(&shamir_context[0].modulus).rep());
            let ss1 = shamir_context[1].share(Scalar::sample_blw(&shamir_context[1].modulus).rep());
            for j in 0..nr_players {
                shares[0][j][i] = ss0[j];
                shares[1][j][i] = ss1[j];
            }
        }
    }
    end_timer!(gc);

    let mut recv_bits: Vec<Vec<u64>> = vec![vec![0u64; nr_bits]; 2];
    for i in 0..nr_bits {
        recv_bits[0][i] = shares[0][id][i];
        recv_bits[1][i] = shares[1][id][i];
    }

    let nr_bytes = nr_bits * 2 * 5;

    let mutex_bits = Arc::new(Mutex::new(recv_bits));
    let mb = mutex_bits.clone();

    let f = tokio::spawn(async move {
        let mut handles = Vec::new();
        for _ in 0..nr_players - 1 {
            // maybe we can new a thread for each socket to improve latency
            let (mut socket, _) = listener.accept().await.unwrap();

            let mbits = mutex_bits.clone();

            handles.push(tokio::spawn(async move {
                let mut buf = vec![0u8; nr_bytes + 1];

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
                let (s0, s1) = deserialize_shares(&buf[1..]);
                {
                    let mut l = mbits.as_ref().lock().unwrap();
                    for i in 0..nr_bits {
                        l[0][i] += s0[i];
                        l[1][i] += s1[i];
                    }
                }
            }));
        }
        futures::future::join_all(handles).await;
    });

    // sending data to other players
    {
        let mut buf = vec![0u8; nr_bytes + 1];
        for i in 0..players.len() {
            buf[0] = id as u8;
            if i != id {
                let mut stream = TcpStream::connect(&players[i]).await?;
                serialize_shares_into(&shares[0][i], &shares[1][i], &mut buf[1..]);
                stream.write_all(&buf).await?;
            }
        }
        drop(shares);
    }

    f.await?;
    //{
    //    println!("{:?}", mb.lock().unwrap()[0]);
    //}
    let mut rb = mb.lock().unwrap();
    let q = vec![&shamir_context[0].modulus, &shamir_context[1].modulus];
    let ntt_context = vec![
        NTTContext::init(MODULUS[0]),
        NTTContext::init(MODULUS[1]),
        NTTContext::init(MODULUS[2]),
    ];
    let qq = vec![
        &ntt_context[0].modulus,
        &ntt_context[1].modulus,
        &ntt_context[2].modulus,
    ];

    // aggregate the noise
    let mut noise: Vec<Vec<u64>> = vec![vec![0u64; nr_bits]; 3];
    for k in 0..2usize {
        for i in 0..nr_bits {
            rb[k][i] = Scalar::modulus(&Scalar::from(rb[k][i]), q[k]).rep();
        }
    }
    // crt and modulus
    // for simplicity, let's simply write the multiplication
    for i in 0..nr_bits {
        for j in 0..3 {
            let noise_a = Scalar::modulus(&Scalar::from(rb[0][i]), qq[j]);
            let noise_b = Scalar::modulus(&Scalar::from(rb[1][i]), qq[j]);
            noise[j][i] = Scalar::mul_mod(&noise_a, &noise_b, qq[j]).rep();
        }
    }

    // println!("{} {} {}", noise[0][0], noise[1][0], noise[2][0]);
    // for each 4k numbers, run NTT
    // assert!(noise[0].len() % 40 == 0);
    for k in (0..noise[0].len()).step_by(NUM_DIMENSION) {
        ntt_context[0].lazy_ntt_inplace(&mut noise[0][k..k + NUM_DIMENSION]);
        ntt_context[1].lazy_ntt_inplace(&mut noise[1][k..k + NUM_DIMENSION]);
        ntt_context[2].lazy_ntt_inplace(&mut noise[2][k..k + NUM_DIMENSION]);
    }

    let elapsed_time = start.elapsed();
    println!(
        "Elapsed time: {:?} seconds",
        elapsed_time.subsec_nanos() as f64 / 1_000_000_000f64 + elapsed_time.as_secs() as f64
    );

    // write into a file
    {
        let file_name = format!("./data/noise{}.txt", id);
        let mut f = BufWriter::new(File::create(file_name).unwrap());
        serialize_into(&mut f, &noise).unwrap();
    }
    Ok(())
}
