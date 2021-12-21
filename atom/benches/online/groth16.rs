#![warn(unused)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    variant_size_differences,
    stable_features,
    non_shorthand_field_patterns,
    renamed_and_removed_lints,
    private_in_public,
    unsafe_code
)]

// For randomness (during paramgen and proof generation)
use ark_ff::{One, ToBytes};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::test_rng;
// For benchmarking
use std::{
    error::Error,
    time::{Duration, Instant},
};

// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-377 pairing-friendly elliptic curve.
// use ark_bls12_377::{Bls12_377, Fr};
use ark_bls12_381::{Bls12_381, Fr};
// use bellperson::bls::{Bls12, Fr};

// We're going to use the Groth 16 proving system.
use ark_groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
};

use std::{env, fs::OpenOptions, path::PathBuf, process};

mod constraints;
use crate::constraints::Benchmark;
use cpu_time::ProcessTime;
#[inline]
pub fn duration_to_sec(d: &Duration) -> f64 {
    d.subsec_nanos() as f64 / 1_000_000_000f64 + (d.as_secs() as f64)
}

fn main() -> Result<(), Box<dyn Error>> {
    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut test_rng();
    // Let's benchmark stuff!
    let num_dimension = 4096;

    let c = Benchmark::new(num_dimension);
    // Create parameters for our circuit
    let params = {
        generate_random_parameters::<Bls12_381, _, _>(c.clone(), rng)?
        // generate_random_parameters::<MNT4_753, _, _>(c, rng)?
    };

    let cpu_start = ProcessTime::now();
    // proof_vec.truncate(0);
    let proof = {
        // Create a proof with our parameters.
        create_random_proof(c, &params, rng)?
    };
    let cpu_time = cpu_start.elapsed();
    println!("proof cpu time {}", duration_to_sec(&cpu_time));

    let mut buf: Vec<u8> = Vec::new();
    proof.serialize(&mut buf).unwrap();

    println!("len of proof: {}", buf.len());

    Ok(())
}
