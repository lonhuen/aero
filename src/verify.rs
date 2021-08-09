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

extern crate ark_ff;
use ark_ff::{Field, FromBytes, One, ToBytes};

// For benchmarking
use std::{
    error::Error,
    fs::File,
    io::{self, prelude::*, BufReader},
    time::{Duration, Instant},
};

// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-377 pairing-friendly elliptic curve.
// use ark_bls12_377::{Bls12_377, Fr};
extern crate ark_bls12_381;
use ark_bls12_381::{Bls12_381, Fr};

// We're going to use the Groth 16 proving system.
extern crate ark_groth16;
use ark_groth16::{verify_proof, PreparedVerifyingKey, Proof};
pub fn i128toField<F: Field>(x: i128) -> F {
    if x < 0 {
        -F::from_random_bytes(&((-x).to_le_bytes())[..]).unwrap()
    } else {
        F::from_random_bytes(&(x.to_le_bytes())[..]).unwrap()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut c_0 = [0i128; 4096];
    let mut c_1 = [0i128; 4096];
    let file = File::open("./data/data.output").unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(l) = line {
            let vec = l.split(" ").collect::<Vec<&str>>();
            for i in 1..vec.len() {
                if l.contains("c_0") {
                    if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                        c_0[i - 1] = x;
                    }
                } else if l.contains("c_1") {
                    if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                        c_1[i - 1] = x;
                    }
                }
            }
        }
    }
    let num_dimension: usize = 4096;
    let num_poly: usize = 1;
    let inputs: Vec<_> = vec![Fr::one(); num_dimension * (num_poly + 1)];
    let inputs: Vec<_> = c_0
        .to_vec()
        .iter()
        .chain(c_1.to_vec().iter())
        .map(|&x| i128toField::<ark_bls12_381::Fr>(x))
        .collect::<Vec<_>>();
    let pvk = {
        let file = File::open("./data/pvk").unwrap();
        let reader = BufReader::new(file);
        //PreparedVerifyingKey::<Bls12_381>::read(reader.buffer())
    };
    let proof = {
        let file = File::open("./data/proof").unwrap();
        let reader = BufReader::new(file);
        //Proof::<Bls12_381>::from()
    };

    let start = Instant::now();
    //let r = verify_proof::<Bls12_381>(&pvk, &proof, &inputs).unwrap();
    Ok(())
}
