#[tarpc::service]
pub trait CommitteeService {
    /// ask the committee to generate enough random bits and store locally, wait until the committee finishes it
    async fn generate_random_bits(nr_slots:u32);
    /// ask the committee to compute ntt(sk) * ntt(u) + ntt(e)
    async fn partial_decrypt(ntt_u:Vec<i128>) -> Vec<i128>;
}
