use ark_std::{end_timer, start_timer};
use quail::rlwe::MODULUS;
use quail::rlwe::{context::Context, NUM_DIMENSION};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod util;
use crate::util::{config::ConfigUtils, log::init_tracing};
use rand::{Rng, SeedableRng};
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
    let args: Vec<String> = env::args().collect();
    let nr_bits: usize = config.get_int("nBits") as usize;

    if args.len() < 2 {
        panic!("provide player id");
    }
    let id = usize::from_str_radix(&args[1], 10).unwrap();
    let nr_players = players.len();
    let threshold = config.get_int("threshold") as usize;
    let listener = TcpListener::bind(&players[id]).await?;

    // first generate enough number of bits
    let gc = start_timer!(|| "random bits");
    let shares = {
        let mut random_bits: Vec<i128> = Vec::with_capacity(nr_bits);
        let mut rng = rand::rngs::StdRng::from_entropy();
        for _ in 0..nr_bits {
            random_bits.push(rng.gen_bool(0.5f64) as i128);
        }
        let context = Context::init_default();
        context.shamir_share(nr_players, threshold, &random_bits)
    };
    end_timer!(gc);

    let mut recv_bits: Vec<i128> = vec![0i128; nr_players * nr_bits];

    for i in 0..nr_bits {
        recv_bits[nr_bits * id + i] = shares[id][i];
    }

    let nr_bytes = bincode::serialized_size(&shares[0]).unwrap() as usize;

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
                let s: Vec<i128> = bincode::deserialize(&buf[1..]).unwrap();
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
            bincode::serialize_into(&mut buf[1..], &shares[i]).unwrap();
            stream.write_all(&buf).await?;
        }
    }

    f.await?;
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
