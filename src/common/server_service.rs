use super::aggregation::merkle::MerkleProof;
use super::aggregation::SummationEntry;

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
    async fn verify(vinit: u32, non_leaf_id: Vec<u32>) -> Vec<(SummationEntry, MerkleProof)>;
}
