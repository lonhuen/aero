use ark_bls12_381::Parameters;
use ark_ec::bls12::Bls12;
use rand::SeedableRng;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};

pub mod aggregation;
pub mod board_service;
pub mod server_service;

// T(2k) = T(k) + 2k
// T(2k+1) = T(k+1) + 2k
// T(n) = T(floor(n+1)/2) + n & ~0x1
pub fn summation_array_size(n: u32) -> u32 {
    let mut s: u32 = 0;
    let mut t = n;
    while t > 1 {
        s += t & (!0x1u32);
        t = (t + 1) / 2;
    }
    s + 1
}

pub fn i128vec_to_le_bytes(v: &Vec<i128>) -> Vec<u8> {
    let ret: Vec<u8> = v.iter().flat_map(|x| i128::to_le_bytes(*x)).collect();
    ret
}
#[inline]
pub fn new_rsa_pub_key() -> Vec<u8> {
    let bits = 2048;
    //let mut rng = rand::rngs::StdRng::seed_from_u64(Instant::now().);
    let mut rng = rand::rngs::StdRng::from_entropy();
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    public_key.to_public_key_pem().unwrap().into_bytes()
}

pub type ZKProof = ark_groth16::Proof<Bls12<Parameters>>;
