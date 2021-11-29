use super::aggregation::merkle::MerkleProof;
use super::aggregation::node::{CommitEntry, SummationEntry, SummationLeaf, SummationNonLeaf};
// This is the service definition
#[tarpc::service]
pub trait ServerService {
    /// send the commitment in the aggregation phase, not block, call get_mc_proof later
    async fn aggregate_commit(round: u32, rsa_pk: Vec<u8>, commitment: Vec<[u8; 32]>);
    /// send the data in the aggregation phase, not block, call get_ms_proof later
    async fn aggregate_data(
        round: u32,
        rsa_pk: Vec<u8>,
        ct0: Vec<Vec<i128>>,
        ct1: Vec<Vec<i128>>,
        nonce: Vec<[u8; 16]>,
        proofs: Vec<Vec<u8>>,
    );
    /// Get the inclusion proof of a node inside the commitment merkle tree
    async fn get_mc_proof(round: u32, rsa_pk: Vec<u8>) -> Vec<MerkleProof>;
    /// Get the inclusion proof of a node inside the summation merkle tree
    async fn get_ms_proof(round: u32, rsa_pk: Vec<u8>) -> Vec<MerkleProof>;

    async fn verify(
        round: u32,
        vinit: u32,
        non_leaf_id: Vec<u32>,
        ct_id: Vec<usize>,
    ) -> Vec<Vec<(SummationEntry, MerkleProof)>>;

    async fn retrieve_model(round: u32) -> Vec<u8>;
    async fn retrieve_proving_key(round: u32) -> Vec<u8>;
}