use super::aggregation::merkle::MerkleProof;
use super::aggregation::node::SummationEntry;
use std::env;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

// credit to https://github.com/google/tarpc/blob/master/example-service/src/lib.rs
pub fn init_tracing(service_name: &str) -> anyhow::Result<()> {
    env::set_var("OTEL_BSP_MAX_EXPORT_BATCH_SIZE", "12");

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
        .with_max_packet_size(2usize.pow(13))
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(())
}

// This is the service definition
#[tarpc::service]
pub trait ServerService {
    /// send the commitment in the aggregation phase, not block, call get_mc_proof later
    async fn aggregate_commit(rsa_pk: Vec<u8>, commitment: [u8; 32]);
    /// send the data in the aggregation phase, not block, call get_ms_proof later
    async fn aggregate_data(rsa_pk: Vec<u8>, cts: Vec<i128>, nonce: [u8; 16], proofs: Vec<u8>);
    /// Get the inclusion proof of a node inside the commitment merkle tree
    async fn get_mc_proof(rsa_pk: Vec<u8>, round: u32) -> MerkleProof;
    /// Get the inclusion proof of a node inside the summation merkle tree
    async fn get_ms_proof(rsa_pk: Vec<u8>, round: u32) -> MerkleProof;

    async fn verify(vinit: u32, non_leaf_id: Vec<u32>) -> Vec<(SummationEntry, MerkleProof)>;

    async fn retrieve_model() -> Vec<u8>;
    async fn retrieve_proving_key() -> Vec<u8>;
}
