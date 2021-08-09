// the coeff modulus used in seal for 4k 128-bit security is { 4096, {
// 0xffffee001, 0xffffc4001, 0x1ffffe0001 }} : 109-bit the modulus used here is
// 0x12AB655E9A2CA55660B44D1E5C37B00159AA76FED00000010A11800000000001 64 * 4 =
// 256 bit 	gaussianSampler := ring.NewGaussianSampler(prng, q, params.Sigma(),
// int(6*params.Sigma())) DefaultSigma in ckks = 3.2
// bound of error = 3.2 . * 6 = 19
use ark_ff::{BigInteger, BigInteger256, Field};
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, OptimizationGoal,
        SynthesisError, Variable,
    },
};
use ark_std::{end_timer, start_timer, test_rng};
use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    marker::PhantomData,
};
pub struct Benchmark<F: Field> {
    pub num_dimension: usize,
    pub num_poly: usize,
    pub c_0: [i128; 4096],
    pub new_c_0: [i128; 4096],
    pub e_0: [i128; 4096],
    pub m: [i128; 4096],
    pub pk_0: [i128; 4096],
    pub delta_0: [i128; 4096],
    pub delta_1: [i128; 4096],
    pub bdop_0: [i128; 4096],
    pub bdop_1: [i128; 4096],
    pub _engine: PhantomData<F>,
}

impl<F: Field> Benchmark<F> {
    pub fn new(num_dimension: usize, num_poly: usize) -> Self {
        let mut c_0 = [0i128; 4096];
        let mut new_c_0 = [0i128; 4096];
        let mut e_0 = [0i128; 4096];
        let mut m = [0i128; 4096];
        let mut pk_0 = [0i128; 4096];
        let mut delta_0 = [0i128; 4096];
        let mut delta_1 = [0i128; 4096];
        let mut bdop_0 = [0i128; 4096];
        let mut bdop_1 = [0i128; 4096];
        let file = File::open("online.output").unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(l) = line {
                let vec = l.split(" ").collect::<Vec<&str>>();
                for i in 1..vec.len() {
                    if l.contains("bdop_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            bdop_0[i - 1] = x;
                        }
                    } else if l.contains("bdop_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            bdop_1[i - 1] = x;
                        }
                    } else if l.contains("c_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            c_0[i - 1] = x;
                            new_c_0[i - 1] = x;
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
                    } else if l.contains("delta_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            delta_1[i - 1] = x;
                        }
                    }
                }
            }
        }
        Self {
            num_dimension,
            num_poly,
            c_0,
            new_c_0,
            e_0,
            m,
            pk_0,
            delta_0,
            delta_1,
            bdop_0,
            bdop_1,
            _engine: PhantomData,
        }
    }
}
impl<F: Field> Benchmark<F> {
    pub fn i128toField(&self, x: i128) -> F {
        if x < 0 {
            -F::from_random_bytes(&((-x).to_le_bytes())[..]).unwrap()
        } else {
            F::from_random_bytes(&(x.to_le_bytes())[..]).unwrap()
        }
    }
}

impl<F: Field> ConstraintSynthesizer<F> for Benchmark<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let pk_0 = self
            .pk_0
            .to_vec()
            .iter()
            .map(|&x| self.i128toField(x))
            .collect::<Vec<_>>();
        let pf = F::from(0xffffee001u64) * F::from(0xffffc4001u64) * F::from(0x1ffffe0001u64);

        // bdop_0
        let mut bdop_0_val_vec = Vec::new();
        let mut bdop_0_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            bdop_0_val_vec.push(self.i128toField(self.bdop_0[i]));
            bdop_0_var_vec.push(cs.new_input_variable(|| Ok(bdop_0_val_vec[i]))?);
        }
        // bdop_1
        let mut bdop_1_val_vec = Vec::new();
        let mut bdop_1_var_vec = Vec::new();
        for i in 0..self.num_dimension * 2 {
            let j = i % self.num_dimension;
            bdop_1_val_vec.push(self.i128toField(self.bdop_1[j]));
            bdop_1_var_vec.push(cs.new_input_variable(|| Ok(bdop_1_val_vec[j]))?);
        }
        // new_c_0
        let mut new_c_0_val_vec = Vec::new();
        let mut new_c_0_var_vec = Vec::new();
        for i in 0..self.num_dimension * 2 {
            let j = i % self.num_dimension;
            new_c_0_val_vec.push(self.i128toField(self.new_c_0[j]));
            new_c_0_var_vec.push(cs.new_input_variable(|| Ok(new_c_0_val_vec[j]))?);
        }
        // e0
        let mut e_0_val_vec = Vec::new();
        let mut e_0_var_vec = Vec::new();
        for i in 0..self.num_dimension * 3 {
            let j = i % self.num_dimension;
            e_0_val_vec.push(self.i128toField(self.e_0[j]));
            e_0_var_vec.push(cs.new_witness_variable(|| Ok(e_0_val_vec[j]))?);
        }
        // delta_0
        let mut delta_0_val_vec = Vec::new();
        let mut delta_0_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            delta_0_val_vec.push(self.i128toField(self.delta_0[i]));
            delta_0_var_vec.push(cs.new_witness_variable(|| Ok(delta_0_val_vec[i]))?);
        }
        // c_0
        let mut c_0_val_vec = Vec::new();
        let mut c_0_var_vec = Vec::new();
        for i in 0..self.num_dimension * 2 {
            let j = i % self.num_dimension;
            c_0_val_vec.push(self.i128toField(self.c_0[j]));
            c_0_var_vec.push(cs.new_witness_variable(|| Ok(c_0_val_vec[j]))?);
        }
        // delta_1
        let mut delta_1_val_vec = Vec::new();
        let mut delta_1_var_vec = Vec::new();
        for i in 0..self.num_dimension * 2 {
            let j = i % self.num_dimension;
            delta_1_val_vec.push(self.i128toField(self.delta_1[j]));
            delta_1_var_vec.push(cs.new_witness_variable(|| Ok(delta_1_val_vec[j]))?);
        }
        // m
        let mut m_val_vec = Vec::new();
        let mut m_var_vec = Vec::new();
        for i in 0..self.num_dimension * 2 {
            let j = i % self.num_dimension;
            m_val_vec.push(self.i128toField(self.m[j]));
            m_var_vec.push(cs.new_witness_variable(|| Ok(m_val_vec[j]))?);
        }
        // pk_0 * e_0 * 5 + e_0 = c_0 + delta_0 * q
        for i in 0..self.num_dimension {
            let mut elc = LinearCombination::zero();
            // (pk * r) [i]
            // pk[j] * r[i-j%4096]
            for l in 0..3 {
                for j in 0..self.num_dimension {
                    let k = (i - j) % self.num_dimension;
                    if j + k >= self.num_dimension {
                        elc.0
                            .push((-pk_0[j], e_0_var_vec[k + l * self.num_dimension]));
                    } else {
                        elc.0
                            .push((pk_0[j], e_0_var_vec[k + l * self.num_dimension]));
                    }
                }
            }
            elc.0.sort_by_key(|e| e.1);
            cs.enforce_constraint(
                lc!() + elc,
                lc!() + Variable::One,
                lc!() + bdop_0_var_vec[i] + (pf, delta_0_var_vec[i]),
            )?;
        }
        // pk_0 * e_0 + c_0 = bdop_1 + delta_1 * q
        for ll in 0..2 {
            for i in 0..self.num_dimension {
                let mut elc = LinearCombination::zero();
                // (pk * r) [i]
                // pk[j] * r[i-j%4096]
                for l in 0..3 {
                    for j in 0..self.num_dimension {
                        let k = (i - j) % self.num_dimension;
                        if j + k >= self.num_dimension {
                            elc.0
                                .push((-pk_0[j], e_0_var_vec[k + l * self.num_dimension]));
                        } else {
                            elc.0
                                .push((pk_0[j], e_0_var_vec[k + l * self.num_dimension]));
                        }
                    }
                }
                elc.0.sort_by_key(|e| e.1);
                cs.enforce_constraint(
                    lc!() + elc + c_0_var_vec[i + ll * self.num_dimension],
                    lc!() + Variable::One,
                    lc!()
                        + bdop_1_var_vec[i + ll * self.num_dimension]
                        + (pf, delta_1_var_vec[i + ll * self.num_dimension]),
                )?;
            }
        }
        // range_proof of e0 [-19,19] -> [0,38]
        for l in 0..3 {
            for i in 0..self.num_dimension {
                let mut bit_val_vec = Vec::new();
                let mut bit_var_vec = Vec::new();
                let mut x = F::one();
                for k in 0..3 {
                    if (self.e_0[i] & (0x1 << k)) == 0 {
                        bit_val_vec.push(F::zero());
                    } else {
                        bit_val_vec.push(F::one());
                    }
                    bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
                }
                let mut tmp_lc = LinearCombination::zero();
                for k in 0..3 {
                    tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                    x = x + x;
                    // bit
                    cs.enforce_constraint(
                        lc!() + bit_var_vec[k],
                        lc!() + bit_var_vec[k] + (-F::one(), Variable::One),
                        lc!(),
                    )?;
                }
                // bit decompose
                cs.enforce_constraint(
                    lc!() + tmp_lc,
                    lc!() + Variable::One,
                    lc!() + e_0_var_vec[i + l * self.num_dimension],
                )?;
            }
        }

        // range_proof of m
        for l in 0..2 {
            for i in 0..self.num_dimension {
                let mut bit_val_vec = Vec::new();
                let mut bit_var_vec = Vec::new();
                let mut x = F::one();
                for k in 0..8 {
                    if (self.m[i] & (0x1 << k)) == 0 {
                        bit_val_vec.push(F::zero());
                    } else {
                        bit_val_vec.push(F::one());
                    }
                    bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
                }
                let mut tmp_lc = LinearCombination::zero();
                for k in 0..8 {
                    tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                    cs.enforce_constraint(
                        lc!() + bit_var_vec[k],
                        lc!() + bit_var_vec[k] + (-F::one(), Variable::One),
                        lc!(),
                    )?;
                    x = x + x;
                }
                // bit decompose
                cs.enforce_constraint(
                    lc!() + tmp_lc,
                    lc!() + Variable::One,
                    lc!() + m_var_vec[i + l * self.num_dimension],
                )?;
            }
        }
        // range_proof of delta_0
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..20 {
                if ((self.delta_0[i] + 4096i128) & (0x1 << k)) == 0 {
                    bit_val_vec.push(F::zero());
                } else {
                    bit_val_vec.push(F::one());
                }
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..20 {
                tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                cs.enforce_constraint(
                    lc!() + bit_var_vec[k],
                    lc!() + bit_var_vec[k] + (-F::one(), Variable::One),
                    lc!(),
                )?;
                x = x + x;
            }
            // bit decompose
            cs.enforce_constraint(
                lc!() + tmp_lc,
                lc!() + Variable::One,
                lc!() + delta_0_var_vec[i] + (F::from(4096u64), Variable::One),
            )?;
        }
        // range_proof of delta_1
        for l in 0..2 {
            for i in 0..self.num_dimension {
                let mut bit_val_vec = Vec::new();
                let mut bit_var_vec = Vec::new();
                let mut x = F::one();
                for k in 0..20 {
                    if ((self.delta_1[i] + 4096i128) & (0x1 << k)) == 0 {
                        bit_val_vec.push(F::zero());
                    } else {
                        bit_val_vec.push(F::one());
                    }
                    bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
                }
                let mut tmp_lc = LinearCombination::zero();
                for k in 0..20 {
                    tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                    cs.enforce_constraint(
                        lc!() + bit_var_vec[k],
                        lc!() + bit_var_vec[k] + (-F::one(), Variable::One),
                        lc!(),
                    )?;
                    x = x + x;
                }
                // bit decompose
                cs.enforce_constraint(
                    lc!() + tmp_lc,
                    lc!() + Variable::One,
                    lc!()
                        + delta_1_var_vec[i + l * self.num_dimension]
                        + (F::from(4096u64), Variable::One),
                )?;
            }
        }
        for l in 0..2 {
            for i in 0..self.num_dimension {
                cs.enforce_constraint(
                    lc!()
                        + m_var_vec[i + l * self.num_dimension]
                        + c_0_var_vec[i + l * self.num_dimension],
                    lc!() + Variable::One,
                    lc!() + new_c_0_var_vec[i + l * self.num_dimension],
                )?;
            }
        }
        println!("# of constraints {}", cs.num_constraints());
        println!("# of instances {}", cs.num_instance_variables());
        println!("# of witness {}", cs.num_witness_variables());
        println!("# of lc{}", cs.borrow().unwrap().num_linear_combinations);
        Ok(())
    }
}
