use ark_std::{end_timer, start_timer};
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use quail::rlwe::context::{NTTContext, ShamirContext};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
//mod util;
//use crate::util::{config::ConfigUtils, log::init_tracing};
//use rand::{Rng, SeedableRng};
//use std::env;
//use std::sync::{Arc, Mutex};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let context = ShamirContext::init(0xffffee001u64, 10, 4);
    //let gc = start_timer!(|| "init");
    //for _ in 0..(4096 * 342 * 40 / 20) {
    //    let secret = Scalar::sample_blw(&context.modulus).rep();
    //    let shares = context.share(secret);
    //    let ret = context.reconstruct(&shares);
    //    assert_eq!(ret, secret);
    //}
    //end_timer!(gc);
    let context = NTTContext::init(0xffffee001u64);
    let mut a = vec![1u64; 4096];
    let mut b = vec![0u64; 4096];
    b[0] = 1;
    context.lazy_ntt_inplace(&mut a);
    context.lazy_ntt_inplace(&mut b);
    let mut ntt_c = context.coeff_mul_mod(&a, &b);
    context.lazy_inverse_ntt_inplace(&mut ntt_c);
    //for _ in 0..(4096 * 342 * 40 / 20) {
    //    let secret = 0u64;
    //    let shares = context.share(secret);
    //}
    //end_timer!(gc);
    //    let config = ConfigUtils::init("committee.yaml");
    //    // read the address of players
    //    let players: Vec<String> = config
    //        .settings
    //        .get_array("players")
    //        .unwrap()
    //        .into_iter()
    //        .map(|x| x.into_str().unwrap())
    //        .collect();
    //    let aggregator = config.get("aggregator");
    //    //let nr_ct: usize = config.get_int("nr_ct") as usize;
    //    let nr_players = players.len();
    //    let threshold = config.get_int("threshold") as usize;
    //    let nr_bits = config.get_int("nBits") as usize;
    //    let listener = TcpListener::bind(&aggregator).await?;
    //
    //    // let mut shares: Vec<Vec<Vec<i128>>> =
    //    //     vec![vec![vec![0i128; nr_ct * NUM_DIMENSION * nr_bits]; nr_players]; nr_players];
    //    let shares: Vec<Arc<_>> = (0..nr_players)
    //        .into_iter()
    //        .map(|_| {
    //            Arc::new(vec![
    //                vec![0i128; nr_ct * NUM_DIMENSION * nr_bits];
    //                nr_players
    //            ])
    //        })
    //        .collect();
    //
    //    let nr_bytes = bincode::serialized_size(shares[0].as_ref()).unwrap() as usize;
    //
    //    let arc_shares = shares.clone();
    //
    //    let f = tokio::spawn(async move {
    //        let mut handles = Vec::new();
    //        for _ in 0..nr_players * nr_players {
    //            //for _ in 0..nr_players - 1 {
    //            // maybe we can new a thread for each socket to improve latency
    //            let (mut socket, _) = listener.accept().await.unwrap();
    //
    //            let mt = arc_shares.clone();
    //
    //            handles.push(tokio::spawn(async move {
    //                let mut buf = vec![0u8; nr_bytes + 2];
    //
    //                // In a loop, read data from the socket and write the data back.
    //                //loop {
    //                let _ = match socket.read(&mut buf).await {
    //                    // socket closed
    //                    Ok(n) if n == 0 => return,
    //                    Ok(n) => n,
    //                    Err(e) => {
    //                        eprintln!("failed to read from socket; err = {:?}", e);
    //                        return;
    //                    }
    //                };
    //                // let s: Vec<i128> = bincode::deserialize(&buf[2..]).unwrap();
    //                // let ss = mt[buf[0] as usize];
    //                // println!("from {:?} to {:?}", buf[0], buf[1]);
    //                // assign to the shares
    //                // mt.as_mut()[]
    //            }));
    //        }
    //        futures::future::join_all(handles).await;
    //    });
    //
    //    f.await?;
    //    // reconstruct and check all bits are 0
    Ok(())
}
//
