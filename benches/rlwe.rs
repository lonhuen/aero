// use ark_std::{end_timer, start_timer};
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use quail::common::aggregation::node::*;
// use quail::rlwe::{PublicKey, MODULUS, NUM_DIMENSION};
// use rand::{rngs::StdRng, Rng, SeedableRng};
// use std::{
//     fs::File,
//     io::{prelude::*, BufReader},
// };
//
// fn encrypt(pk: &PublicKey) {
//     // read the key
//     //let prover = Prover::new("data/encryption.txt", "data/pk.txt");
//     pk.encrypt(&[0u8; 4096].to_vec());
// }
//
// fn matrix_mut(p: &Vec<i128>, q: &Vec<i128>) -> Vec<i128> {
//     let mut ret = Vec::<i128>::with_capacity(4096);
//     for i in 0..4096 {
//         let mut s: i128 = 0;
//         for j in 0..4096 {
//             s = s + q[i] * p[i * 4096 + j];
//         }
//         ret.push(s);
//     }
//     ret
// }
//
// fn sort(buf: &mut Vec<SummationLeaf>) {
//     let gc = start_timer!(|| "sort");
//     buf.sort_by_key(|l| l.rsa_pk.clone());
//     end_timer!(gc);
// }
//
// fn criterion_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("encrypt");
//     group.significance_level(0.1).sample_size(30);
//     /*
//     let pk = {
//         let mut pk_0 = [0i128; 4096];
//         let mut pk_1 = [0i128; 4096];
//         let file = match File::open("./data/encryption.txt") {
//             Ok(f) => f,
//             Err(_) => panic!(),
//         };
//         let reader = BufReader::new(file);
//         for line in reader.lines() {
//             if let Ok(l) = line {
//                 let vec = l.split(" ").collect::<Vec<&str>>();
//                 for i in 1..vec.len() {
//                     if l.contains("pk_0") {
//                         if let Ok(x) = i128::from_str_radix(vec[i], 10) {
//                             pk_0[i - 1] = x;
//                         }
//                     } else if l.contains("pk_1") {
//                         if let Ok(x) = i128::from_str_radix(vec[i], 10) {
//                             pk_1[i - 1] = x;
//                         }
//                     }
//                 }
//             }
//         }
//         PublicKey::new(pk_0.to_vec(), pk_1.to_vec())
//     };
//
//     let mut rng = rand::rngs::StdRng::from_entropy();
//     let mut p = Vec::<i128>::with_capacity(4096 * 4096);
//     for _ in 0..4096 * 4096 {
//         p.push(rng.gen_range(0..MODULUS));
//     }
//     let mut q = Vec::<i128>::with_capacity(4096);
//     for _ in 0..4096 {
//         q.push(rng.gen_range(0..3));
//     }
//     */
//     let mut rng = rand::rngs::StdRng::from_entropy();
//     let mut buf = {
//         (0..200)
//             .map(|_| {
//                 SummationLeaf::from_ct(
//                     (0..500).map(|_| rng.gen::<u8>()).collect(),
//                     (0..NUM_DIMENSION * 0).map(|_| rng.gen::<i128>()).collect(),
//                     [0u8; 16],
//                 )
//             })
//             .collect()
//     };
//     group.bench_function("sort", |b| b.iter(|| sort(&mut buf)));
//     //group.bench_function("encrypt", |b| b.iter(|| encrypt(&pk)));
//     //group.bench_function("matrix multiplication", |b| b.iter(|| matrix_mut(&p, &q)));
//     group.finish();
// }
//
// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
//
