use super::aggregation::merkle::MerkleProof;
use super::aggregation::SummationEntry;

/// Trait for service of the bulletin board
#[tarpc::service]
pub trait BoardService{
    /// Get the inclusion proof of a node inside the commitment merkle tree
    async fn get_mc_proof(leaf_id: u32,round:u32) -> MerkleProof;
    /// Get the inclusion proof of a node inside the summation merkle tree
    async fn get_ms_proof(leaf_id: u32,round:u32) -> MerkleProof;
}
