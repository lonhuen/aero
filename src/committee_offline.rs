use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::{config::ConfigUtils, log::init_tracing};
use rand::{Rng, SeedableRng};
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use threshold_secret_sharing as tss;

pub const ONE: i128 = 1208925819615728686333953i128;
pub const MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];

fn sample_polynomial(secret: u64, threshold: usize, q: &Scalar) -> Vec<Scalar> {
    let mut poly = vec![Scalar::from(secret)];
    poly.extend((0..threshold).map(|_| Scalar::sample_blw(q)));
    poly
}
fn evaluate_polynomial(poly: &Vec<Scalar>, x: u64, q: &Scalar) -> u64 {
    let x_scalar = Scalar::from(x);
    let mut s = Scalar::zero();
    let mut x_pow = Scalar::one();
    for c in poly {
        // the addition will not exceed 64-bit
        s = Scalar::add(&s, &Scalar::mul_mod(&x_pow, &c, q));
        x_pow = Scalar::mul_mod(&x_pow, &x_scalar, q);
    }
    Scalar::modulus(&s, q).rep()
}

fn reconstruct_shamir(poly: &Vec<Scalar>, threshold: usize, q: &Scalar) -> u64 {
    0u64
}

fn shamir_share(
    nr_players: usize,
    threshold: usize,
    values: &Vec<(u64, u64, u64)>,
    modulus_scalar: &[Scalar; 3],
) -> Vec<Vec<(u64, u64, u64)>> {
    let mut ret = vec![values.clone(); nr_players];
    for i in 0..values.len() {
        let a: Vec<u64> = {
            let poly = sample_polynomial(values[i].0, threshold, &modulus_scalar[0]);
            (1..nr_players + 1)
                .into_iter()
                .map(|x| evaluate_polynomial(&poly, x as u64, &modulus_scalar[0]))
                .collect()
        };
        let b: Vec<u64> = {
            let poly = sample_polynomial(values[i].1, threshold, &modulus_scalar[1]);
            (1..nr_players + 1)
                .into_iter()
                .map(|x| evaluate_polynomial(&poly, x as u64, &modulus_scalar[1]))
                .collect()
        };
        let c: Vec<u64> = {
            let poly = sample_polynomial(values[i].2, threshold, &modulus_scalar[2]);
            (1..nr_players + 1)
                .into_iter()
                .map(|x| evaluate_polynomial(&poly, x as u64, &modulus_scalar[2]))
                .collect()
        };
        for j in 0..nr_players {
            ret[j][i] = (a[j], b[j], c[j]);
        }
    }
    ret
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

    let modulus_scalar = [
        Scalar::new_modulus(MODULUS[0]),
        Scalar::new_modulus(MODULUS[1]),
        Scalar::new_modulus(MODULUS[2]),
    ];

    //let context = Context::init_default();

    // first generate enough number of bits
    let nr_bits: usize = 1;
    let shares = {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let random_bits: Vec<(u64, u64, u64)> = (0..nr_bits)
            .into_iter()
            .map(|_| {
                if rng.gen_bool(0.5) {
                    (0u64, 0u64, 0u64)
                } else {
                    (1u64, 1u64, 1u64)
                }
            })
            .collect();
        let gc = start_timer!(|| "random bits");
        let r = shamir_share(nr_players, threshold, &random_bits, &modulus_scalar);
        end_timer!(gc);
        r
    };

    let one_share: Vec<i64> = (0..nr_players)
        .into_iter()
        .map(|x| shares[x][0].0 as i64)
        .collect();

    let tss = tss::shamir::ShamirSecretSharing {
        threshold: threshold,
        share_count: nr_players,
        prime: MODULUS[0] as i64,
    };

    let result = tss.reconstruct(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], &one_share);
    println!("{:?}", result);

    // let mut recv_bits: Vec<Field> = vec![Field::zero(); nr_players * nr_bits];
    /*
        for i in 0..nr_bits {
            recv_bits[nr_bits * id + i] = shares[id][i];
        }

        let nr_bytes = nr_bits * 16 + 256;

        let mutex_bits = Arc::new(Mutex::new(recv_bits));
        let mb = mutex_bits.clone();

        let f = tokio::spawn(async move {
            let mut handles = Vec::new();
            for _ in 0..nr_players - 1 {
                // maybe we can new a thread for each socket to improve latency
                let (mut socket, _) = listener.accept().await.unwrap();

                let mbits = mutex_bits.clone();

                handles.push(tokio::spawn(async move {
                    let mut buf = vec![0u8; nr_bytes + 2];

                    let _ = match socket.read(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };
                    let src = buf[0] as usize;
                    let s: Vec<Field> = {
                        let ts: Vec<i128> = bincode::deserialize(&buf[1..]).unwrap();
                        ts.iter().map(|x| Field::from_i128(*x)).collect()
                    };
                    {
                        let mut l = mbits.as_ref().lock().unwrap();
                        for i in 0..nr_bits {
                            l[src * nr_bits + i] = s[i];
                        }
                    }
                }));
            }
            futures::future::join_all(handles).await;
        });

        // sending data to other players
        let mut buf = vec![0u8; nr_bytes + 2];
        for i in 0..players.len() {
            buf[0] = id as u8;
            if i != id {
                let mut stream = TcpStream::connect(&players[i]).await?;
                {
                    let s: Vec<i128> = shares[i].iter().map(|x| x.as_i128()).collect();
                    bincode::serialize_into(&mut buf[1..], &s).unwrap();
                }
                stream.write_all(&buf).await?;
            }
        }

        f.await?;
    */
    /*
        // connect to the aggregator to get a random number
        let mut stream = TcpStream::connect(&aggregator_addr).await?;
        // stream.read(&mut buf).await?;
        // let r: i128 = bincode::deserialize(&buf).unwrap();
        let _r: i128 = 1i128;

        let mut _r_vec: Vec<i128> = vec![1i128; NUM_DIMENSION];
        // let mut rr: i128 = 1;
        // for _ in 0..NUM_DIMENSION {
        //     r_vec.push(rr);
        //     rr = (rr * r) % MODULUS;
        // }

        // bits validation
        let bits = mb.lock().unwrap();
        let ret: Vec<i128> = (0..bits.len())
            .step_by(NUM_DIMENSION)
            .map(|_x| {
                //let mut s: i128 = 0;
                let s: i128 = 0;
                // for i in 0..NUM_DIMENSION {
                //     // let delta = bits[i + x] * bits[i + x] - bits[i + x] * shamir(1);
                //     //s = s + (bits[i + x] * r_vec[i]) % MODULUS;
                // }
                s
            })
            .collect();
        let buf = bincode::serialize(&ret).unwrap();
        stream.write_all(&buf).await?;

        // update the local noise
    */
    Ok(())
}
