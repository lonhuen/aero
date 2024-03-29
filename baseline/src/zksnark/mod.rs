#![warn(unused)]
use std::{
    borrow::{Borrow, BorrowMut},
    fs::File,
    io::{BufReader, BufWriter},
    num,
    sync::{Arc, RwLock},
};

use ark_ff::{Field, Fp256};
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef,
    OptimizationGoal, Result as R1CSResult,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::UniformRand;
// For benchmarking
use ark_bls12_381::{Bls12_381, Fr, FrParameters, Parameters};
use ark_ec::models::bls12::Bls12;
use ark_groth16::{
    create_random_proof, generate_random_parameters, lonhh_create_proof, verify_proof,
    PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
};
use ark_std::test_rng;
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
    join,
};
mod constraints;
use crate::zksnark::constraints::Circuit;
const NUM_DIMENSION: usize = 4096;

pub fn i128_to_field(x: i128) -> Fr {
    if x < 0 {
        //-Fp256::<Parameters>::from_random_bytes(&((-x).to_le_bytes())[..]).unwrap()
        -Fr::from_random_bytes(&((-x).to_le_bytes())[..]).unwrap()
    } else {
        Fr::from_random_bytes(&((x).to_le_bytes())[..]).unwrap()
    }
}

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
        // write the proving key
        {
            let mut buf = BufWriter::new(File::create("./data/proving_key.txt").unwrap());
            params.serialize_unchecked(&mut buf).unwrap();
        }
        {
            let mut buf = BufWriter::new(File::create("./data/verifying_key.txt").unwrap());
            params.vk.serialize_unchecked(&mut buf).unwrap();
        }
        Self {
            proving_key: params,
            circuit: c,
        }
    }
    pub fn new(enc_path: &str, pvk_path: &str) -> Self {
        let c = Circuit::<Fr>::new(NUM_DIMENSION, enc_path);

        let pvk = BufReader::new(File::open(pvk_path).unwrap());

        // read the proving key
        // ideally we should call deserialize; since this can be done offline, we just call deserialize_unchecked for simplicity here
        let params = ProvingKey::<Bls12<Parameters>>::deserialize_unchecked(pvk).unwrap();

        Self {
            proving_key: params,
            circuit: c,
        }
    }
    // // TODO we might need to set the inputs and witness of the circuit
    // pub fn create_proof(&self) -> Proof<Bls12<Parameters>> {
    //     //TODO use OsRng here
    //     let rng = &mut test_rng();
    //     create_random_proof(self.circuit.clone(), &self.proving_key, rng).unwrap()
    // }
    // pub fn get_circuit(&mut self) -> &mut Circuit<Fr> {
    //     &mut self.circuit
    // }
    pub fn create_n_proofs(
        nr_proof: usize,
        pk: &ProvingKey<Bls12<Parameters>>,
        circuit: &Circuit<Fr>,
    ) {
        let cs = ConstraintSystem::new_ref();
        cs.set_optimization_goal(OptimizationGoal::Constraints);
        circuit.clone().generate_constraints(cs.clone()).unwrap();
        cs.finalize();
        let matrices = cs.to_matrices().unwrap();
        let rng = &mut test_rng();
        for _ in 0..nr_proof {
            let rr = Fr::rand(rng);
            let rs = Fr::rand(rng);
            let proof = lonhh_create_proof::<Bls12<Parameters>, Circuit<Fr>>(
                cs.clone(),
                &matrices,
                pk,
                rr,
                rs,
            )
            .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            proof.serialize_unchecked(&mut buf).unwrap();
        }
    }

    pub fn create_proof_in_bytes(
        &self,
        c0: &Vec<Vec<i128>>,
        c1: &Vec<Vec<i128>>,
        r: &Vec<Vec<i128>>,
        e0: &Vec<Vec<i128>>,
        e1: &Vec<Vec<i128>>,
        delta0: &Vec<Vec<i128>>,
        delta1: &Vec<Vec<i128>>,
        m: &Vec<Vec<i128>>,
    ) -> Vec<Vec<u8>> {
        //let num_threads = rayon::current_num_threads();
        //let delta = (c0.len() as f32 / num_threads as f32).ceil() as usize;
        //let mut nr_proofs = vec![delta; num_threads - 1];
        //nr_proofs.push(c0.len() - delta * (num_threads - 1));
        //nr_proofs
        //    .into_par_iter()
        //    .for_each(|x| Self::create_n_proofs(x, &self.proving_key, &self.circuit));

        //vec![vec![0u8; 192]; c0.len()]
        let cs = ConstraintSystem::new_ref();
        cs.set_optimization_goal(OptimizationGoal::Constraints);
        self.circuit
            .clone()
            .generate_constraints(cs.clone())
            .unwrap();
        cs.finalize();
        let matrices = cs.to_matrices().unwrap();
        let rng = &mut test_rng();

        let mut ret = Vec::with_capacity(c0.len());
        for i in (0..c0.len()) {
            let cs = cs.clone();
            //    // // update the witness in constraint system
            //    // //r[i].iter().chain(e0[i].iter()).chain(e1[i].iter)
            //    // let e0_bit: Vec<i128> = e0[i]
            //    //     .iter()
            //    //     .flat_map(|x| (0..5).map(|l| (x >> l) & 0x1).collect::<Vec<i128>>())
            //    //     .collect();
            //    // let e1_bit: Vec<i128> = e1[i]
            //    //     .iter()
            //    //     .flat_map(|x| (0..5).map(|l| (x >> l) & 0x1).collect::<Vec<i128>>())
            //    //     .collect();
            //    // let m_bit: Vec<i128> = m[i]
            //    //     .iter()
            //    //     .flat_map(|x| (0..8).map(|l| (x >> l) & 0x1).collect::<Vec<i128>>())
            //    //     .collect();
            //    // let delta0_bit: Vec<i128> = delta0[i]
            //    //     .iter()
            //    //     .flat_map(|x| {
            //    //         (0..13)
            //    //             .map(|l| ((x + 4096) >> l) & 0x1)
            //    //             .collect::<Vec<i128>>()
            //    //     })
            //    //     .collect();
            //    // let delta1_bit: Vec<i128> = delta1[i]
            //    //     .iter()
            //    //     .flat_map(|x| {
            //    //         (0..13)
            //    //             .map(|l| ((x + 4096) >> l) & 0x1)
            //    //             .collect::<Vec<i128>>()
            //    //     })
            //    //     .collect();
            //    // r[i].iter()
            //    //     .chain(e0[i].iter())
            //    //     .chain(e1[i].iter())
            //    //     .chain(delta0[i].iter())
            //    //     .chain(delta1[i].iter())
            //    //     .chain(m[i].iter())
            //    //     .chain(e0_bit.iter())
            //    //     .chain(e1_bit.iter())
            //    //     .chain(m_bit.iter())
            //    //     .chain(delta0_bit.iter())
            //    //     .chain(delta1_bit.iter())
            //    //     .zip(cs.borrow_mut().unwrap().witness_assignment.iter_mut())
            //    //     .for_each(|(x, y)| *y = i128_to_field(*x));
            //    // c0[i]
            //    //     .iter()
            //    //     .chain(c1[0].iter())
            //    //     .zip(cs.borrow_mut().unwrap().instance_assignment[1..].iter_mut())
            //    //     .for_each(|(x, y)| *y = i128_to_field(*x));
            //    // first r
            let rr = Fr::rand(rng);
            let rs = Fr::rand(rng);
            let proof = lonhh_create_proof::<Bls12<Parameters>, Circuit<Fr>>(
                cs,
                &matrices,
                &self.proving_key,
                rr,
                rs,
            )
            .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            proof.serialize_unchecked(&mut buf).unwrap();
            ret.push(buf);
        }
        ret
    }
    pub fn deserialize_proof(pf: &Vec<u8>) -> Proof<Bls12<Parameters>> {
        Proof::<Bls12<Parameters>>::deserialize_unchecked(&**pf).unwrap()
    }
    pub fn serialize_pvk(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        self.proving_key.serialize_unchecked(&mut buf).unwrap();
        buf
    }
}
#[derive(Clone)]
pub struct Verifier {
    pub pvk: PreparedVerifyingKey<Bls12<Parameters>>,
}
impl Verifier {
    pub fn new(vk_path: &str) -> Self {
        let pvk = BufReader::new(File::open(vk_path).unwrap());

        // read the proving key
        // ideally we should call deserialize; since this can be done offline, we just call deserialize_unchecked for simplicity here
        let vk = VerifyingKey::<Bls12<Parameters>>::deserialize_unchecked(pvk).unwrap();
        Self { pvk: vk.into() }
    }

    pub fn verify_proof_from_bytes(&self, pf: &Vec<u8>, inputs: &[i128]) -> bool {
        let n_inputs: Vec<Fr> = inputs.iter().map(|x| i128_to_field(*x)).collect();
        let proof = Prover::deserialize_proof(pf);
        verify_proof(&self.pvk, &proof, &n_inputs).unwrap()
    }

    pub fn verify_proof(&self, pf: &Proof<Bls12<Parameters>>, inputs: &[i128]) -> bool {
        let n_inputs: Vec<Fr> = inputs.iter().map(|x| i128_to_field(*x)).collect();
        verify_proof(&self.pvk, &pf, &n_inputs).unwrap()
    }
}
