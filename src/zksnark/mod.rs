#![warn(unused)]
use ark_ff::Fp256;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::test_rng;
// For benchmarking
use ark_bls12_381::{Bls12_381, Fr, FrParameters, Parameters};
use ark_ec::models::bls12::Bls12;
use ark_groth16::{
    create_random_proof, generate_random_parameters, verify_proof, PreparedVerifyingKey, Proof,
    ProvingKey, VerifyingKey,
};
mod constraints;
use crate::zksnark::constraints::Circuit;
const NUM_DIMENSION: usize = 4096;

#[derive(Clone)]
pub struct Prover {
    pub proving_key: ProvingKey<Bls12<Parameters>>,
    pub circuit: Circuit<Fr>,
}
impl Prover {
    pub fn setup(enc_path: &str) -> Self {
        let c = Circuit::<Fr>::new(NUM_DIMENSION, enc_path);
        //TODO use OsRng here
        let rng = &mut test_rng();

        let params = generate_random_parameters::<Bls12_381, _, _>(c.clone(), rng).unwrap();
        Self {
            proving_key: params,
            circuit: c,
        }
        // write the proving key
        //{
        //    let mut buf: Vec<u8> = Vec::new();
        //    params.serialize(&mut buf).unwrap();
        //    std::fs::write(pk_path, &buf[..]).unwrap();
        //}
        //let buf: Vec<u8> = std::fs::read(pk_path).unwrap();
        //let new_params = ProvingKey::<Bls12<Parameters>>::deserialize(&*buf).unwrap();
        //println!("params == new_params? {}", params == new_params);
        // write the verification key
        //{
        //    let mut buf: Vec<u8> = Vec::new();
        //    params.vk.serialize(&mut buf).unwrap();
        //    std::fs::write(vk_path, &buf[..]).unwrap();
        //}
    }
    pub fn new(enc_path: &str, pvk: Vec<u8>) -> Self {
        let c = Circuit::<Fr>::new(NUM_DIMENSION, enc_path);

        // read the proving key
        let params = ProvingKey::<Bls12<Parameters>>::deserialize(&*pvk).unwrap();

        Self {
            proving_key: params,
            circuit: c,
        }
    }
    // TODO we might need to set the inputs and witness of the circuit
    pub fn create_proof(&self) -> Proof<Bls12<Parameters>> {
        //TODO use OsRng here
        let rng = &mut test_rng();
        create_random_proof(self.circuit.clone(), &self.proving_key, rng).unwrap()
    }
    pub fn get_circuit(&mut self) -> &mut Circuit<Fr> {
        &mut self.circuit
    }
    pub fn create_proof_in_bytes(&self) -> Vec<u8> {
        let rng = &mut test_rng();
        let proof = create_random_proof(self.circuit.clone(), &self.proving_key, rng).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        proof.serialize(&mut buf).unwrap();
        buf
    }
    pub fn deserialize_proof(pf: &Vec<u8>) -> Proof<Bls12<Parameters>> {
        Proof::<Bls12<Parameters>>::deserialize(&**pf).unwrap()
    }
    pub fn serialize_pvk(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        self.proving_key.serialize(&mut buf).unwrap();
        buf
    }
}
#[derive(Clone)]
pub struct Verifier {
    pub pvk: PreparedVerifyingKey<Bls12<Parameters>>,
}
impl Verifier {
    pub fn new(prover: &Prover) -> Self {
        //let buf = std::fs::read(vk_path).unwrap();
        let vk = prover.proving_key.vk.clone();
        Self { pvk: vk.into() }
    }

    pub fn verify_proof_from_bytes(&self, pf: &Vec<u8>, inputs: &[Fp256<FrParameters>]) -> bool {
        //let proof = Proof::<Bls12<Parameters>>::deserialize(&**pf).unwrap();
        let proof = Prover::deserialize_proof(pf);
        verify_proof(&self.pvk, &proof, inputs).unwrap()
    }

    pub fn verify_proof(
        &self,
        pf: &Proof<Bls12<Parameters>>,
        inputs: &[Fp256<FrParameters>],
    ) -> bool {
        verify_proof(&self.pvk, pf, inputs).unwrap()
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;

    // run the following 2 tests with RUST_MIN_STACK=8388608 cargo test test_create_proof --release
    #[test]
    fn test_create_proof() {
        Prover::setup("data/pk.txt", "data/vk.txt", "data/encryption.txt");
        let prover = Prover::new("data/encryption.txt", "data/pk.txt");
        //let proof = prover.create_proof_in_bytes();
        let proof = prover.create_proof();
        let inputs: Vec<_> = prover
            .circuit
            .c_0
            .to_vec()
            .iter()
            .chain(prover.circuit.c_1.to_vec().iter())
            .map(|&x| prover.circuit.i128toField(x))
            .collect::<Vec<_>>();
        let verifier = Verifier::new("data/vk.txt");
        //let result = verifier.verify_proof_from_bytes(&proof, &inputs);
        let result = verifier.verify_proof(&proof, &inputs);
        println!("result {}", result);
        assert!(result);
    }
    #[test]
    fn test_create_proof_in_bytes() {
        Prover::setup("data/pk.txt", "data/vk.txt", "data/encryption.txt");
        let prover = Prover::new("data/encryption.txt", "data/pk.txt");
        //let proof = prover.create_proof_in_bytes();
        let proof = prover.create_proof_in_bytes();
        let inputs: Vec<_> = prover
            .circuit
            .c_0
            .to_vec()
            .iter()
            .chain(prover.circuit.c_1.to_vec().iter())
            .map(|&x| prover.circuit.i128toField(x))
            .collect::<Vec<_>>();
        let verifier = Verifier::new("data/vk.txt");
        //let result = verifier.verify_proof_from_bytes(&proof, &inputs);
        let result = verifier.verify_proof_from_bytes(&proof, &inputs);
        println!("result {}", result);
        assert!(result);
    }
}
*/
