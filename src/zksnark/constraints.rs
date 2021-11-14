// the coeff modulus used in seal for 4k 128-bit security is { 4096, {
// 0xffffee001, 0xffffc4001, 0x1ffffe0001 }} : 109-bit the modulus used here is
// 0x12AB655E9A2CA55660B44D1E5C37B00159AA76FED00000010A11800000000001 64 * 4 =
// 256 bit 	gaussianSampler := ring.NewGaussianSampler(prng, q, params.Sigma(),
// int(6*params.Sigma())) DefaultSigma in ckks = 3.2
// bound of error = 3.2 . * 6 = 19
use ark_ff::Field;
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
    },
};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    marker::PhantomData,
};
#[derive(Clone)]
pub struct Circuit<F: Field> {
    pub num_dimension: usize,
    //pub c_0: [i128; 4096],
    //pub c_1: [i128; 4096],
    //pub r: [i128; 4096],
    //pub e_0: [i128; 4096],
    //pub e_1: [i128; 4096],
    //pub m: [i128; 4096],
    pub pk_0: [i128; 4096],
    pub pk_1: [i128; 4096],
    //pub delta_0: [i128; 4096],
    //pub delta_1: [i128; 4096],
    pub _engine: PhantomData<F>,
}

impl<F: Field> Circuit<F> {
    pub fn new(num_dimension: usize, file_path: &str) -> Self {
        let mut pk_0 = [0i128; 4096];
        let mut pk_1 = [0i128; 4096];
        //let mut c_0 = [0i128; 4096];
        //let mut c_1 = [0i128; 4096];
        //let mut r = [0i128; 4096];
        //let mut e_0 = [0i128; 4096];
        //let mut e_1 = [0i128; 4096];
        //let mut m = [0i128; 4096];
        //let mut delta_0 = [0i128; 4096];
        //let mut delta_1 = [0i128; 4096];
        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => panic!(),
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(l) = line {
                let vec = l.split(" ").collect::<Vec<&str>>();
                for i in 1..vec.len() {
                    if l.contains("pk_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_0[i - 1] = x;
                        }
                    } else if l.contains("pk_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_1[i - 1] = x;
                        }
                    }
                }
            }
        }
        Self {
            num_dimension,
            // c_0,
            // c_1,
            // r,
            // e_0,
            // e_1,
            // m: [0i128; 4096],
            pk_0,
            pk_1,
            // delta_0,
            // delta_1,
            _engine: PhantomData,
        }
    }

    pub fn i128to_field(&self, x: i128) -> F {
        if x < 0 {
            -F::from_random_bytes(&((-x).to_le_bytes())[..]).unwrap()
        } else {
            F::from_random_bytes(&(x.to_le_bytes())[..]).unwrap()
        }
    }
}

impl<F: Field> ConstraintSynthesizer<F> for Circuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let pk_0 = self
            .pk_0
            .to_vec()
            .iter()
            .map(|&x| self.i128to_field(x))
            .collect::<Vec<_>>();
        let pk_1 = self
            .pk_1
            .to_vec()
            .iter()
            .map(|&x| self.i128to_field(x))
            .collect::<Vec<_>>();
        let pf = F::from(0xffffee001u64) * F::from(0xffffc4001u64) * F::from(0x1ffffe0001u64);

        // c0
        //let mut c0_val_vec = Vec::new();
        let mut c0_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            //c0_val_vec.push(self.i128to_field(self.c_0[i]));
            //c0_var_vec.push(cs.new_input_variable(|| Ok(c0_val_vec[i]))?);
            c0_var_vec.push(cs.new_input_variable(|| Ok(F::zero()))?);
        }
        // c1
        //let mut c1_val_vec = Vec::new();
        let mut c1_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            //c1_val_vec.push(self.i128to_field(self.c_1[i]));
            //c1_var_vec.push(cs.new_input_variable(|| Ok(c1_val_vec[i]))?);
            c1_var_vec.push(cs.new_input_variable(|| Ok(F::zero()))?);
        }
        // r
        //let mut r_val_vec = Vec::new();
        let mut r_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            //r_val_vec.push(self.i128to_field(self.r[i]));
            //r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[i]))?);
            r_var_vec.push(cs.new_witness_variable(|| Ok(F::zero()))?);
        }
        // e0
        //let mut e0_val_vec = Vec::new();
        let mut e0_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            //e0_val_vec.push(self.i128to_field(self.e_0[i]));
            //e0_var_vec.push(cs.new_witness_variable(|| Ok(e0_val_vec[i]))?);
            e0_var_vec.push(cs.new_witness_variable(|| Ok(F::zero()))?);
        }
        // e2
        //let mut e1_val_vec = Vec::new();
        let mut e1_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            //e1_val_vec.push(self.i128to_field(self.e_1[i]));
            //e1_var_vec.push(cs.new_witness_variable(|| Ok(e1_val_vec[i]))?);
            e1_var_vec.push(cs.new_witness_variable(|| Ok(F::zero()))?);
        }
        // delta_0
        //let mut delta_0_val_vec = Vec::new();
        let mut delta_0_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            // delta_0_val_vec.push(self.i128to_field(self.delta_0[i]));
            // delta_0_var_vec.push(cs.new_witness_variable(|| Ok(delta_0_val_vec[i]))?);
            delta_0_var_vec.push(cs.new_witness_variable(|| Ok(F::zero()))?);
        }
        // delta_1
        //let mut delta_1_val_vec = Vec::new();
        let mut delta_1_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            //delta_1_val_vec.push(self.i128to_field(self.delta_1[i]));
            //delta_1_var_vec.push(cs.new_witness_variable(|| Ok(delta_1_val_vec[i]))?);
            delta_1_var_vec.push(cs.new_witness_variable(|| Ok(F::zero()))?);
        }
        // pk_0 * r + e_0 = c_0 + delta_0 * q
        for i in 0..self.num_dimension {
            let mut elc = LinearCombination::zero();
            // (pk * r) [i]
            // pk[j] * r[i-j%4096]
            for j in 0..self.num_dimension {
                let k = (i - j) % self.num_dimension;
                if j + k >= self.num_dimension {
                    elc.0.push((-pk_0[j], r_var_vec[k]));
                } else {
                    elc.0.push((pk_0[j], r_var_vec[k]));
                }
            }
            elc.0.sort_by_key(|e| e.1);
            cs.enforce_constraint(
                lc!() + elc + e0_var_vec[i],
                lc!() + Variable::One,
                lc!() + c0_var_vec[i] + (pf, delta_0_var_vec[i]),
            )?;
        }
        // m
        //let mut m_val_vec = Vec::new();
        let mut m_var_vec = Vec::new();
        for _i in 0..self.num_dimension {
            //m_val_vec.push(self.i128to_field(self.m[i]));
            //m_var_vec.push(cs.new_witness_variable(|| Ok(m_val_vec[i]))?);
            m_var_vec.push(cs.new_witness_variable(|| Ok(F::zero()))?);
        }
        // pk_1 * r + e_0 + m = c_1 + delta_1 * q
        for i in 0..self.num_dimension {
            let mut elc = LinearCombination::zero();
            // (pk * r) [i]
            // pk[j] * r[i-j%4096]
            for j in 0..self.num_dimension {
                let k = (i - j) % self.num_dimension;
                if j + k >= self.num_dimension {
                    elc.0.push((-pk_1[j], r_var_vec[k]));
                } else {
                    elc.0.push((pk_1[j], r_var_vec[k]));
                }
            }
            elc.0.sort_by_key(|e| e.1);
            cs.enforce_constraint(
                lc!() + elc + e1_var_vec[i] + m_var_vec[i],
                lc!() + Variable::One,
                lc!() + c1_var_vec[i] + (pf, delta_1_var_vec[i]),
            )?;
        }
        // range_proof of e1 [-19,19] -> [0,38]
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..5 {
                //if (self.e_0[i] & (0x1 << k)) == 0 {
                //    bit_val_vec.push(F::zero());
                //} else {
                //    bit_val_vec.push(F::one());
                //}
                bit_val_vec.push(F::zero());
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..5 {
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
            cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() + e0_var_vec[i])?;
        }

        // range_proof of e2
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..5 {
                //if (self.e_1[i] & (0x1 << k)) == 0 {
                //    bit_val_vec.push(F::zero());
                //} else {
                //    bit_val_vec.push(F::one());
                //}
                bit_val_vec.push(F::zero());
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..5 {
                tmp_lc = tmp_lc + (x, bit_var_vec[k]);
                x = x + x;
                cs.enforce_constraint(
                    lc!() + bit_var_vec[k],
                    lc!() + bit_var_vec[k] + (-F::one(), Variable::One),
                    lc!(),
                )?;
            }
            // bit decompose
            cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() + e1_var_vec[i])?;
        }

        // range_proof of m
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..8 {
                //if (self.m[i] & (0x1 << k)) == 0 {
                //    bit_val_vec.push(F::zero());
                //} else {
                //    bit_val_vec.push(F::one());
                //}
                bit_val_vec.push(F::zero());
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
            cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() + m_var_vec[i])?;
        }
        // range_proof of delta_0
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..13 {
                //if ((self.delta_0[i] + 4096i128) & (0x1 << k)) == 0 {
                //    bit_val_vec.push(F::zero());
                //} else {
                //    bit_val_vec.push(F::one());
                //}
                bit_val_vec.push(F::zero());
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..13 {
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
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..13 {
                //if ((self.delta_1[i] + 4096i128) & (0x1 << k)) == 0 {
                //    bit_val_vec.push(F::zero());
                //} else {
                //    bit_val_vec.push(F::one());
                //}
                bit_val_vec.push(F::zero());
                bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
            }
            let mut tmp_lc = LinearCombination::zero();
            for k in 0..13 {
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
                lc!() + delta_1_var_vec[i] + (F::from(4096u64), Variable::One),
            )?;
        }
        //eprintln!("# of constraints {}", cs.num_constraints());
        //eprintln!("# of instances {}", cs.num_instance_variables());
        //eprintln!("# of witness {}", cs.num_witness_variables());
        Ok(())
    }
}
