use ark_bls12_381::Fr as ArkFr;
use ark_ff::{BigInteger, BigInteger256, Field, One, Zero};
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, OptimizationGoal,
        SynthesisError, Variable,
    },
};
use ark_std::{end_timer, start_timer, test_rng};
use bellperson::bls::{Bls12, Fr as BPFr};
use ff::{Field as BPField, ScalarEngine as BPEngine};
use neptune::{self, bls381num::AllocatedNum, poseidon::PoseidonConstants, Arity};
use std::{
    any::type_name,
    fs::File,
    io::{self, prelude::*, BufReader},
    marker::PhantomData,
};
use typenum::*;

#[derive(Clone)]
pub struct CircuitOffline {
    pub num_dimension: usize,
    pub c_0: [i128; 4096],
    pub r: [i128; 4096],
    pub e_0: [i128; 4096],
    pub pk_0: [i128; 4096],
    pub delta_0: [i128; 4096],
    pub nonce: [u8; 32],
    pub hash: [u8; 32],
    constants: PoseidonConstants<Bls12, typenum::U34>,
    arity: usize,
    // TODO hash result should be here
    pub _engine: PhantomData<ArkFr>,
}
impl CircuitOffline {
    pub fn new(enc_path: &str) -> Self {
        let num_dimension = 4096;
        let mut c_0 = [0i128; 4096];
        let mut r = [0i128; 4096];
        let mut e_0 = [0i128; 4096];
        let mut pk_0 = [0i128; 4096];
        let mut delta_0 = [0i128; 4096];
        let nonce = [0u8; 32];
        let hash = [0u8; 32];
        let file = File::open(enc_path).unwrap();
        let reader = BufReader::new(file);
        let constants = PoseidonConstants::<Bls12, typenum::U34>::new_with_strength(
            neptune::Strength::Standard,
        );
        let arity = typenum::U34::to_usize();
        for line in reader.lines() {
            if let Ok(l) = line {
                let vec = l.split(" ").collect::<Vec<&str>>();
                for i in 1..vec.len() {
                    if l.contains("r") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            r[i - 1] = x;
                        }
                    } else if l.contains("c_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            c_0[i - 1] = x;
                        }
                    } else if l.contains("pk_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_0[i - 1] = x;
                        }
                    } else if l.contains("e_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            e_0[i - 1] = x;
                        }
                    } else if l.contains("delta_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            delta_0[i - 1] = x;
                        }
                    }
                }
            }
        }
        Self {
            num_dimension,
            c_0,
            r,
            e_0,
            pk_0,
            delta_0,
            nonce,
            hash,
            constants,
            arity,
            _engine: PhantomData,
        }
    }
}

impl CircuitOffline {
    pub fn i128toField(&self, x: i128) -> ArkFr {
        if x < 0 {
            -ArkFr::from_random_bytes(&((-x).to_le_bytes())[..]).unwrap()
        } else {
            ArkFr::from_random_bytes(&(x.to_le_bytes())[..]).unwrap()
        }
    }
}

impl ConstraintSynthesizer<ArkFr> for CircuitOffline {
    // TODO maybe c1 should be used for online
    fn generate_constraints(self, cs: ConstraintSystemRef<ArkFr>) -> Result<(), SynthesisError> {
        // TODO this should be moved into circuit field
        // init constants

        // begin

        let pk_0 = self
            .pk_0
            .to_vec()
            .iter()
            .map(|&x| self.i128toField(x))
            .collect::<Vec<_>>();
        let pf = ArkFr::from(0xffffee001u64)
            * ArkFr::from(0xffffc4001u64)
            * ArkFr::from(0x1ffffe0001u64);

        // c0
        let mut c0_val_vec = Vec::new();
        let mut c0_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            c0_val_vec.push(self.i128toField(self.c_0[i]));
            c0_var_vec.push(cs.new_input_variable(|| Ok(c0_val_vec[i]))?);
        }
        // r
        // r_bit
        let mut r_bit_val_vec = Vec::new();
        let mut r_bit_var_vec = Vec::new();
        let mut r_agg_val_vec = Vec::new();
        let mut r_agg_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            r_bit_val_vec.push(self.i128toField(self.r[i] & 0x1));
            r_bit_var_vec.push(cs.new_witness_variable(|| Ok(r_bit_val_vec[2 * i]))?);
            r_bit_val_vec.push(self.i128toField(self.r[i] & 0x2));
            r_bit_var_vec.push(cs.new_witness_variable(|| Ok(r_bit_val_vec[2 * i + 1]))?);
        }
        // aggregate of all bits into 34 elements
        {
            let mut l = lc!();
            let mut t = ArkFr::zero();
            let mut x = ArkFr::one();
            let mut ii = 0;
            let y = ArkFr::one().double();
            for i in 0..r_bit_val_vec.len() {
                l = l + (x, r_bit_var_vec[i]);
                t = t + x * r_bit_val_vec[i];
                x = x * y;
                if i % 255 == 0 {
                    r_agg_val_vec.push(t);
                    r_agg_var_vec.push(cs.new_witness_variable(|| Ok(r_agg_val_vec[ii]))?);
                    cs.enforce_constraint(
                        lc!() + &l,
                        lc!() + Variable::One,
                        lc!() + r_agg_var_vec[ii],
                    )?;
                    l.clear();
                    x = ArkFr::one();
                    t = ArkFr::zero();
                    ii = ii + 1;
                }
            }
            if l.len() != 0 {
                r_agg_val_vec.push(t);
                r_agg_var_vec.push(cs.new_witness_variable(|| Ok(r_agg_val_vec[ii]))?);
                cs.enforce_constraint(lc!() + l, lc!() + Variable::One, lc!() + r_agg_var_vec[ii])?;
            }
        }
        // e0
        let mut e0_val_vec = Vec::new();
        let mut e0_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            e0_val_vec.push(self.i128toField(self.e_0[i]));
            e0_var_vec.push(cs.new_witness_variable(|| Ok(e0_val_vec[i]))?);
        }
        // delta_0
        let mut delta_0_val_vec = Vec::new();
        let mut delta_0_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            delta_0_val_vec.push(self.i128toField(self.delta_0[i]));
            delta_0_var_vec.push(cs.new_witness_variable(|| Ok(delta_0_val_vec[i]))?);
        }
        // pk_0 * r + e_0 = c_0 + delta_0 * q
        for i in 0..self.num_dimension {
            let mut elc = LinearCombination::zero();
            //(pk * r) [i]
            //pk[j] * r[i-j%4096]
            for j in 0..self.num_dimension {
                let k = (i - j) % self.num_dimension;
                if j + k >= self.num_dimension {
                    elc.0.push((-pk_0[j], r_bit_var_vec[2 * k]));
                    elc.0
                        .push((-pk_0[j] * ArkFr::from(2u64), r_bit_var_vec[2 * k + 1]));
                } else {
                    elc.0.push((pk_0[j], r_bit_var_vec[2 * k]));
                    elc.0
                        .push((pk_0[j] * ArkFr::from(2u64), r_bit_var_vec[2 * k + 1]));
                }
            }
            elc.0.sort_by_key(|e| e.1);
            cs.enforce_constraint(
                lc!() + elc + e0_var_vec[i],
                lc!() + Variable::One,
                lc!() + c0_var_vec[i] + (pf, delta_0_var_vec[i]),
            )?;
        }
        // range_proof of e0 [-19,19] -> [0,38]
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = ArkFr::one();
            for k in 0..5 {
                if (self.e_0[i] & (0x1 << k)) == 0 {
                    bit_val_vec.push(ArkFr::zero());
                } else {
                    bit_val_vec.push(ArkFr::one());
                }
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..5 {
                tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                x = x + x;
                // bit
                cs.enforce_constraint(
                    lc!() + bit_var_vec[k],
                    lc!() + bit_var_vec[k] + (-ArkFr::one(), Variable::One),
                    lc!(),
                )?;
            }
            // bit decompose
            cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() + e0_var_vec[i])?;
        }
        // range_proof of delta_0
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = ArkFr::one();
            for k in 0..13 {
                if ((self.delta_0[i] + 4096i128) & (0x1 << k)) == 0 {
                    bit_val_vec.push(ArkFr::zero());
                } else {
                    bit_val_vec.push(ArkFr::one());
                }
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..13 {
                tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                cs.enforce_constraint(
                    lc!() + bit_var_vec[k],
                    lc!() + bit_var_vec[k] + (-ArkFr::one(), Variable::One),
                    lc!(),
                )?;
                x = x + x;
            }
            // bit decompose
            cs.enforce_constraint(
                lc!() + tmp_lc,
                lc!() + Variable::One,
                lc!() + delta_0_var_vec[i] + (ArkFr::from(4096u64), Variable::One),
            )?;
        }
        let data: Vec<AllocatedNum<Bls12>> = r_agg_val_vec
            .iter()
            .map(|x| AllocatedNum::alloc(&cs, || Ok(neptune::bls381num::ark2bp(*x))).unwrap())
            .collect::<Vec<_>>();
        let _out = neptune::circuit::poseidon_hash(&cs, data, &self.constants)
            .expect("poseidon hashing failed");
        println!("# of constraints {}", cs.num_constraints());
        println!("# of instances {}", cs.num_instance_variables());
        println!("# of witness {}", cs.num_witness_variables());
        println!("# of lc{}", cs.borrow().unwrap().num_linear_combinations);
        Ok(())
    }
}
