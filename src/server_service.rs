#[path = "cipher.rs"]
pub mod cipher;
use ark_bls12_381::Parameters;
use ark_ec::models::bls12::Bls12;
use cipher::CipherText;
use crypto::digest::Digest;
use crypto::sha3::{Sha3, Sha3Mode};
use merkle_light::merkle;
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use serde::ser::{SerializeSeq, SerializeStruct};
use tarpc::serde::{Deserialize, Serialize};
//use merkle_light::proof::Proof as MerkleProof;

pub fn hash_commitment(rsa_pk: &Vec<u8>, cm: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha3::sha3_256();
    hasher.input(rsa_pk);
    hasher.input(cm);
    let mut h = [0u8; 32];
    hasher.result(&mut h);
    h
}

pub type ZKProof = ark_groth16::Proof<Bls12<Parameters>>;
#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleProof {
    pub lemma: Vec<[u8; 32]>,
    pub path: Vec<bool>,
}

impl MerkleProof {
    pub fn to_proof(self) -> merkle_light::proof::Proof<[u8; 32]> {
        merkle_light::proof::Proof::<[u8; 32]>::from(self)
    }
}
impl From<merkle_light::proof::Proof<[u8; 32]>> for MerkleProof {
    fn from(proof: merkle_light::proof::Proof<[u8; 32]>) -> Self {
        Self {
            lemma: proof.lemma,
            path: proof.path,
        }
    }
}
impl From<MerkleProof> for merkle_light::proof::Proof<[u8; 32]> {
    fn from(proof: MerkleProof) -> Self {
        Self {
            lemma: proof.lemma,
            path: proof.path,
        }
    }
}
// This is the service definition
#[tarpc::service]
pub trait ServerService {
    // send the commitment in the aggregation phase
    async fn aggregate_commit(rsa_pk: Vec<u8>, commitment: [u8; 32]) -> MerkleProof;
    async fn aggregate_data(
        rsa_pk: Vec<u8>,
        cts: Vec<u8>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) -> MerkleProof;
}
