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

pub type ZKProof = ark_groth16::Proof<Bls12<Parameters>>;
