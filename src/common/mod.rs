use ark_bls12_381::Parameters;
use ark_ec::bls12::Bls12;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
pub mod aggregation;
pub mod cipher;
pub mod server_service;

pub fn hash_commitment(rsa_pk: &Vec<u8>, cm: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha3::sha3_256();
    hasher.input(rsa_pk);
    hasher.input(cm);
    let mut h = [0u8; 32];
    hasher.result(&mut h);
    h
}
// T(2k) = T(k) + 2k
// T(2k+1) = T(k+1) + 2k
// T(n) = T(floor(n+1)/2) + n & ~0x1
pub fn summation_array_size(N: u32) -> u32 {
    let mut s: u32 = 0;
    let mut n = N;
    while n > 1 {
        s += n & (!0x1u32);
        n = (n + 1) / 2;
    }
    s + 1
}

pub type ZKProof = ark_groth16::Proof<Bls12<Parameters>>;
