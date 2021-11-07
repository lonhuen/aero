use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use quail::rlwe::context::{NTTContext, ShamirContext};
use quail::rlwe::NUM_DIMENSION;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::config::ConfigUtils;
use rand::SeedableRng;
use ring_algorithm::chinese_remainder_theorem;
use std::env;
use std::sync::{Arc, Mutex};

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
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("provide player id");
    }
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

    let id = usize::from_str_radix(&args[1], 10).unwrap();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    let listener = TcpListener::bind(&players[id]).await?;

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

    // first generate enough number of bits
    let gc = start_timer!(|| "generate shamir sharing");
    let mut rng = rand::rngs::StdRng::from_entropy();
    let random_bits: Vec<u64> = (0..nr_bits)
        .into_iter()
        //.map(|_| rng.gen_bool(0.5) as u64)
        //.map(|_| 1u64)
        .map(|_| 0u64)
        .collect();
    let mut shares = vec![vec![vec![0u64; random_bits.len()]; nr_players]; 3];
    for i in 0..random_bits.len() {
        let ss0 = shamir_context[0].share(random_bits[i]);
        let ss1 = shamir_context[1].share(random_bits[i]);
        let ss2 = shamir_context[2].share(random_bits[i]);
        for j in 0..nr_players {
            shares[0][j][i] = ss0[j];
            shares[1][j][i] = ss1[j];
            shares[2][j][i] = ss2[j];
        }
    }
    end_timer!(gc);
    //let gc = start_timer!(|| "serialization");
    //let mut buf = vec![0u8; share0[0].len() * 3 * 5];
    //serialize_shares_into(&share0[0], &share1[0], &share2[0], &mut buf);
    ////let buf = serialize_shares(&share0[0], &share1[0], &share2[0]);
    //end_timer!(gc);
    //let gc = start_timer!(|| "deserialization");
    //let (s0, s1, s2) = deserialize_shares(&buf);
    //end_timer!(gc);

    //for i in 0..nr_bits {
    //    assert_eq!(s0[i], share0[0][i]);
    //    assert_eq!(s1[i], share1[0][i]);
    //    assert_eq!(s2[i], share2[0][i]);
    //}
    let mut recv_bits: Vec<Vec<u64>> = vec![vec![0u64; nr_players * nr_bits]; 3];
    for i in 0..nr_bits {
        recv_bits[0][nr_bits * id + i] = shares[0][id][i];
        recv_bits[1][nr_bits * id + i] = shares[1][id][i];
        recv_bits[2][nr_bits * id + i] = shares[2][id][i];
    }

    let nr_bytes = nr_bits * 3 * 5;

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
                let (s0, s1, s2) = deserialize_shares(&buf[1..]);
                {
                    let mut l = mbits.as_ref().lock().unwrap();
                    for i in 0..nr_bits {
                        l[0][src * nr_bits + i] = s0[i];
                        l[1][src * nr_bits + i] = s1[i];
                        l[2][src * nr_bits + i] = s2[i];
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
                serialize_shares_into(&shares[0][i], &shares[1][i], &shares[2][i], &mut buf[1..]);
                stream.write_all(&buf).await?;
            }
        }
    }

    f.await?;
    //{
    //    println!("{:?}", mb.lock().unwrap()[0]);
    //}
    let rb = mb.lock().unwrap();
    let q = vec![
        &shamir_context[0].modulus,
        &shamir_context[1].modulus,
        &shamir_context[2].modulus,
    ];
    {
        // generate b*b - b and send this to the aggregator
        // connect to the aggregator to get a random number
        // let mut stream = TcpStream::connect(&aggregator_addr).await?;
        // stream.read(&mut buf).await?;
        // let r: i128 = bincode::deserialize(&buf).unwrap();
        let r = Scalar::from(1u64);

        // pre-generate the power vector
        let mut r_pow_vec: Vec<Vec<Scalar>> = vec![vec![Scalar::zero(); NUM_DIMENSION]; 3];
        let mut pow_r = vec![Scalar::from(1u64); 3];
        for i in 0..NUM_DIMENSION {
            for j in 0..3 {
                r_pow_vec[j][i] = pow_r[j].clone();
                pow_r[j] = Scalar::mul_mod(&pow_r[j], &r, &shamir_context[j].modulus);
            }
        }

        // apply the polynomial identity test
        let mut flag_bits: Vec<Vec<u64>> = vec![vec![0u64; rb[0].len() / NUM_DIMENSION]; 3];
        for i in 0..rb[0].len() / NUM_DIMENSION {
            for k in 0..3 {
                // let bit = Scalar::from(rb[k][i * NUM_DIMENSION]);
                // let sum = Scalar::mul_mod(&bit, &bit, q[k]);
                let mut sum = Scalar::zero();
                //for j in 0..NUM_DIMENSION {
                for j in 0..1 {
                    let bit = Scalar::from(rb[k][i * NUM_DIMENSION + j]);
                    // TODO fix the bug here
                    //let flag = Scalar::sub_mod(&Scalar::mul_mod(&bit, &bit, q[k]), &bit, q[k]);
                    let flag = Scalar::sub_mod(&Scalar::mul_mod(&bit, &bit, q[k]), &bit, q[k]);
                    sum = Scalar::add_mod(
                        &sum,
                        &Scalar::mul_mod(&r_pow_vec[k][j], &flag, q[k]),
                        q[k],
                    );
                }
                flag_bits[k][i] = sum.rep();
            }
        }

        // send to aggregator
        {
            let mut stream = TcpStream::connect(&aggregator_addr).await?;
            let mut buf = vec![0u8; flag_bits[0].len() * 5 * 3 + 1];
            buf[0] = id as u8;
            serialize_shares_into(&flag_bits[0], &flag_bits[1], &flag_bits[2], &mut buf[1..]);
            stream.write_all(&buf).await?;
            stream.shutdown().await?;
        }
    }

    // aggregate 40 bits into noise and apply NTT
    // assert!(rb[0].len() % 40 == 0);
    println!("rb len {}", rb[0].len());
    let mut noise: Vec<Vec<u64>> = Vec::new();
    for k in 0..3usize {
        noise.push(
            (0..rb[0].len())
                .step_by(40)
                .map(|x| {
                    let mut sum: u64 = 0;
                    for i in x..x + 20usize {
                        sum += rb[k][i];
                    }
                    for i in x + 20..x + 40usize {
                        sum -= rb[k][i];
                    }
                    Scalar::modulus(&Scalar::from(sum), q[k]).rep()
                })
                .collect(),
        );
    }
    // for each 4k numbers, run NTT
    // assert!(noise[0].len() % 40 == 0);
    for k in (0..noise[0].len()).step_by(NUM_DIMENSION) {
        ntt_context[0].lazy_ntt_inplace(&mut noise[0][k..k + NUM_DIMENSION]);
        ntt_context[1].lazy_ntt_inplace(&mut noise[1][k..k + NUM_DIMENSION]);
        ntt_context[2].lazy_ntt_inplace(&mut noise[2][k..k + NUM_DIMENSION]);
    }
    // write into a file
    // println!("{:?}", noise);
    Ok(())
}
