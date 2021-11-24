mod rlwe;
mod zksnark;
use std::borrow::Borrow;

use crate::rlwe::context::*;
//use crate::zksnark::*;
use crate::zksnark::Prover;
use crate::zksnark::ProverOffline;
use crate::zksnark::ProverOnline;
use ark_groth16::lonhh_create_proof;
use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use cupcake::polyarith::lazy_ntt::{lazy_inverse_ntt_u64, lazy_ntt_u64};
use cupcake::rqpoly::RqPolyContext;
use quail::rlwe::context::MODULUS;
//use quail::rlwe::context::{self, Context};
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef,
    OptimizationGoal, Result as R1CSResult,
};
use ark_std::{end_timer, start_timer};
use ring_algorithm::chinese_remainder_theorem;

fn main() {
    // let e0: Vec<i128> = vec![3i128; 4];
    // let e0_bit: Vec<i128> = e0
    //     .iter()
    //     .flat_map(|x| (0..2).map(|l| (x >> l) & 0x1).collect::<Vec<i128>>())
    //     .collect();
    // println!("{:?}", e0_bit);

    {
        //let prover = Prover::setup("./data/encryption.txt");
        let prover = ProverOnline::setup("./data/encryption.txt");
        //let prover = ProverOffline::setup("./data/encryption.txt");
    }
    //let gc = start_timer!(|| "deserialization");
    //let mut pv = Prover::new("./data/encryption.txt", "./data/proving_key.txt");
    // let mut ii = 0;
    // for i in 0..4096 {
    //     pv.circuit.r[i] = ii;
    //     ii += 1;
    // }
    // for i in 0..4096 {
    //     pv.circuit.e_0[i] = ii;
    //     ii += 1;
    // }
    // for i in 0..4096 {
    //     pv.circuit.e_1[i] = ii;
    //     ii += 1;
    // }
    // for i in 0..4096 {
    //     pv.circuit.delta_0[i] = ii;
    //     ii += 1;
    // }
    // for i in 0..4096 {
    //     pv.circuit.delta_1[i] = ii;
    //     ii += 1;
    // }
    // for i in 0..4096 {
    //     pv.circuit.m[i] = ii;
    //     ii += 1;
    // }
    // ii = 0;
    // for i in 0..4096 {
    //     pv.circuit.c_0[i] = ii;
    //     ii += 1;
    // }
    // for i in 0..4096 {
    //     pv.circuit.c_1[i] = ii;
    //     ii += 1;
    // }
    // let cs = ConstraintSystem::new_ref();
    // cs.set_optimization_goal(OptimizationGoal::Constraints);
    // pv.circuit.clone().generate_constraints(cs.clone()).unwrap();
    // cs.finalize();
    // let matrices = cs.to_matrices().unwrap();
    // println!(
    //     "witness len {:?}",
    //     cs.borrow_mut().unwrap().witness_assignment.len()
    // );
    // println!(
    //     "instance len {:?}",
    //     cs.borrow_mut().unwrap().instance_assignment.len()
    // );
    // println!("witness");
    // for i in cs.borrow().unwrap().witness_assignment.iter() {
    //     println!("{}", i);
    // }
    // println!("instance");
    // for i in cs.borrow().unwrap().instance_assignment.iter() {
    //     println!("{}", i);
    // }

    //end_timer!(gc);
    //let gc = start_timer!(|| "create proofs");
    //let proof = pv.create_proof_in_bytes();
    //end_timer!(gc);
    //let verifier = Verifier::new("./data/verifying_key.txt");
    //let mut inputs: Vec<_> = pv
    //    .circuit
    //    .c_0
    //    .to_vec()
    //    .iter()
    //    .chain(pv.circuit.c_1.to_vec().iter())
    //    //.map(|&x| pv.circuit.i128to_field(x))
    //    .map(|&x| x)
    //    .collect::<Vec<_>>();
    //let gc = start_timer!(|| "verify proofs");
    //println!("flag {}", verifier.verify_proof_from_bytes(&proof, &inputs));
    //end_timer!(gc);
}
