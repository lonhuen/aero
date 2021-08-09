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
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, OptimizationGoal,
        SynthesisError, Variable,
    },
};
use std::marker::PhantomData;

pub struct Benchmark<F: Field> {
    num_dimension: usize,
    num_poly: usize,
    _engine: PhantomData<F>,
}

impl<F: Field> Benchmark<F> {
    pub fn new(num_dimension: usize, num_poly: usize) -> Self {
        Self {
            num_dimension,
            num_poly,
            _engine: PhantomData,
        }
    }
}

impl<F: Field> ConstraintSynthesizer<F> for Benchmark<F> {
    // fn range_proof(self, cs: ConstraintSystemRef<F>) -> Result<(),
    // SynthesisError> {

    //}
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // TODO this primitive root should be set properly, but now we assume it's one
        let pf = F::from(0xffffee001u64) * F::from(0xffffc4001u64) * F::from(0x1ffffe0001u64);
        cs.set_optimization_goal(OptimizationGoal::Constraints);
        // [1 1 1] * [1 1 1] + [0 0 0] = [1 1 1]
        // read NTT(pk)
        // let mut pk_val_vec = Vec::new();
        // let mut pk_var_vec = Vec::new();
        // for i in 0..self.num_dimension{
        //     pk_val_vec.push(F::one());
        //     pk_var_vec.push(cs.new_input_variable(|| Ok(pk_val_vec[i]))?);
        // }
        // Input: NTT(c1), NTT(c2), NTT(b1), NTT(b2), NTT(b3)
        // Witness: NTT(r), NTT(rc1), NTT(rc2), NTT(rc3), NTT(e1), NTT(e2)
        // NTT(c1)
        let mut c1_val_vec = Vec::new();
        let mut c1_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            c1_val_vec.push(pf);
            c1_var_vec.push(cs.new_input_variable(|| Ok(c1_val_vec[i]))?);
        }
        // NTT(c2)
        let mut c2_val_vec = Vec::new();
        let mut c2_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            c2_val_vec.push(pf);
            c2_var_vec.push(cs.new_input_variable(|| Ok(c2_val_vec[i]))?);
        }
        // NTT(r)
        let mut r_val_vec = Vec::new();
        let mut r_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            r_val_vec.push(pf);
            r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[i]))?);
        }
        // NTT(e1)
        let mut e1_val_vec = Vec::new();
        let mut e1_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            e1_val_vec.push(F::from(19u64));
            e1_var_vec.push(cs.new_witness_variable(|| Ok(e1_val_vec[i]))?);
        }
        let primitive_root = F::one();
        // for i in 0..self.num_dimension{
        //     let mut elc = LinearCombination::zero();
        //     let mut t = F::zero();
        //     for j in 0..self.num_dimension{
        //         elc = elc + (primitive_root, e1_var_vec[j]);
        //     }
        //     // TODO mod p
        //     // though mod p won't affect the # of constraints
        //     cs.enforce_constraint(lc!() + (F::one(),c1_var_vec[i]) +
        // (-F::one(),r_var_vec[i]), lc!() + Variable::One,lc!() + elc)?; }
        // NTT(e2)
        let mut e2_val_vec = Vec::new();
        let mut e2_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            e2_val_vec.push(F::from(19u64));
            e2_var_vec.push(cs.new_witness_variable(|| Ok(e2_val_vec[i]))?);
        }
        // for i in 0..self.num_dimension{
        //     let mut elc = LinearCombination::zero();
        //     let mut t = F::zero();
        //     for j in 0..self.num_dimension{
        //         elc = elc + (primitive_root, e2_var_vec[j]);
        //     }
        //     cs.enforce_constraint(lc!() + (F::one(),c1_var_vec[i]) +
        // (-F::one(),r_var_vec[i]), lc!() + Variable::One,lc!() + elc)?; }
        // m
        let mut m_val_vec = Vec::new();
        let mut m_var_vec = Vec::new();
        for i in 0..self.num_dimension {
            m_val_vec.push(F::from(255u64));
            m_var_vec.push(cs.new_witness_variable(|| Ok(m_val_vec[i]))?);
        }
        // NTT(m) + proof for ciphertext
        for i in 0..self.num_dimension {
            let mut elc = LinearCombination::zero();
            // let mut t = F::zero();
            for j in 0..self.num_dimension {
                elc = elc + (primitive_root, m_var_vec[j]);
            }
            // TODO mod p
            // though mod p won't affect the # of constraints
            cs.enforce_constraint(
                lc!() + (F::one(), r_var_vec[i]) + e1_var_vec[i],
                lc!() + Variable::One,
                lc!() + c1_var_vec[i],
            )?;
            cs.enforce_constraint(
                lc!() + (F::one(), r_var_vec[i]) + e2_var_vec[i] + elc,
                lc!() + Variable::One,
                lc!() + c2_var_vec[i],
            )?;
        }
        // range_proof
        // for i in 0..self.num_dimension{
        // let mut bit_val_vec = Vec::new();
        // let mut bit_var_vec = Vec::new();
        // let mut x = F::one();
        // for k in 0..4 {
        // bit_val_vec.push(F::zero());
        // bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
        // }
        // let mut tmp_lc = LinearCombination::zero();
        // for k in 0..4 {
        // tmp_lc = tmp_lc + (x, bit_var_vec[k]);
        // x = x + x;
        // bit
        // cs.enforce_constraint(lc!() + bit_var_vec[k],lc!() + bit_var_vec[k] +
        // (-F::one(), Variable::One), lc!())?; }
        // bit decompose
        // cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() +
        // e1_var_vec[i])?; }
        // for i in 0..self.num_dimension{
        // let mut bit_val_vec = Vec::new();
        // let mut bit_var_vec = Vec::new();
        // let mut x = F::one();
        // for k in 0..4 {
        // bit_val_vec.push(F::zero());
        // bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
        // }
        // let mut tmp_lc = LinearCombination::zero();
        // for k in 0..4 {
        // tmp_lc = tmp_lc + (x, bit_var_vec[k]);
        // x = x + x;
        //     cs.enforce_constraint(lc!() + bit_var_vec[k],lc!() + bit_var_vec[k] +
        // (-F::one(), Variable::One), lc!())?; }
        // bit decompose
        // cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() +
        // e2_var_vec[i])?; }
        for i in 0..self.num_dimension {
            let mut bit_val_vec = Vec::new();
            let mut bit_var_vec = Vec::new();
            let mut x = F::one();
            for k in 0..8 {
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

        // Proof for BDOP commitment
        // suppose k = 3, n = 1, commit to 2 polys (in crisp k=5,n=1, 4 error terms)
        // [1 1 1] [rc1=1]   [0]       [b1=1]
        // [1 1 1] [rc2=0] + [e1=0] =  [b2=1]
        // [1 1 1] [rc3=0]   [e2=0]    [b3=1]
        let k = 3;
        // NTT(rc)
        let mut rc_val_vec = vec![Vec::new(); 3];
        let mut rc_var_vec = vec![Vec::new(); 3];
        for i in 0..self.num_dimension {
            rc_val_vec[0].push(F::one());
            rc_var_vec[0].push(cs.new_witness_variable(|| Ok(rc_val_vec[0][i]))?);
        }
        for j in 1..k {
            for i in 0..self.num_dimension {
                rc_val_vec[j].push(F::zero());
                rc_var_vec[j].push(cs.new_witness_variable(|| Ok(rc_val_vec[j][i]))?);
            }
        }
        // NTT(bd)
        let mut bd_val_vec = vec![Vec::new(); 3];
        let mut bd_var_vec = vec![Vec::new(); 3];
        for j in 0..k {
            for i in 0..self.num_dimension {
                bd_val_vec[j].push(F::one());
                bd_var_vec[j].push(cs.new_input_variable(|| Ok(bd_val_vec[j][i]))?);
            }
        }
        for j in 0..k {
            for i in 0..self.num_dimension {
                cs.enforce_constraint(
                    lc!()
                        + (F::one(), rc_var_vec[0][i])
                        + (F::one(), rc_var_vec[1][i])
                        + (F::one(), rc_var_vec[2][i]),
                    lc!() + Variable::One,
                    lc!() + bd_var_vec[j][i],
                )?;
            }
        }
        // check commitment
        println!("# of constraints {}", cs.num_constraints());
        println!("# of instances {}", cs.num_instance_variables());
        println!("# of witness {}", cs.num_witness_variables());
        Ok(())
        // let primitive_root = F::one();
        // left polynomial
        // let mut pk_val_vec = Vec::new();
        // let mut pk_var_vec = Vec::new();
        // for i in 0..self.num_dimension{
        // pk_val_vec.push(F::one());
        // pk_var_vec.push(cs.new_input_variable(|| Ok(pk_val_vec[i]))?);
        // }
        //
        // result polynomial
        // let mut ct_val_vec = Vec::new();
        // let mut ct_var_vec = Vec::new();
        // for i in 0..self.num_dimension{
        // ct_val_vec.push(F::one());
        // ct_var_vec.push(cs.new_input_variable(|| Ok(ct_val_vec[i]))?);
        // }
        //
        // witness input
        // suppose f1(x)=1,f2(x)=0,f3(x)=0,...
        // let mut r_val_vec = Vec::new();
        // let mut r_var_vec = Vec::new();
        // r_val_vec.push(F::one());
        // r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[0]))?);
        // for i in 1..self.num_dimension{
        // r_val_vec.push(F::zero());
        // r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[i]))?);
        // }
        // for j in 0..self.num_poly - 1{
        // for i in 0..self.num_dimension{
        // r_val_vec.push(F::zero());
        // r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[i +
        // (j+1)*self.num_dimension]))?); }
        // }
        //
        // let mut agg_r_val_vec = Vec::new();
        // let mut agg_r_var_vec = Vec::new();
        //
        // aggregate several polynomials
        // for j in 0..self.num_dimension {
        // let mut tmp_lc = LinearCombination::zero();
        // let mut tmp_val = F::zero();
        // let x = F::one();
        // for i in 0..self.num_poly {
        // tmp_lc = tmp_lc + (x, r_var_vec[j+ self.num_dimension * i]);
        // tmp_val = tmp_val + x * r_val_vec[j+ self.num_dimension * i];
        // }
        // agg_r_val_vec.push(tmp_val);
        // agg_r_var_vec.push(cs.new_witness_variable(||
        // Ok(agg_r_val_vec[j]))?); cs.enforce_constraint(lc!() +
        // tmp_lc, lc!() + Variable::One, lc!() + agg_r_var_vec[j])?; }
        //
        // mod reduction
        // let mut agg_r_val_mod_vec = Vec::new();
        // let mut agg_r_var_mod_vec = Vec::new();
        // check commitment
        // println!("# of constraints {}",cs.num_constraints());
        // println!("# of instances {}",cs.num_instance_variables());
        // println!("# of witness {}",cs.num_witness_variables());
        // Ok(())
        // let primitive_root = F::one();
        // left polynomial
        // let mut pk_val_vec = Vec::new();
        // let mut pk_var_vec = Vec::new();
        // for i in 0..self.num_dimension{
        // pk_val_vec.push(F::one());
        // pk_var_vec.push(cs.new_input_variable(|| Ok(pk_val_vec[i]))?);
        // }
        //
        // result polynomial
        // let mut ct_val_vec = Vec::new();
        // let mut ct_var_vec = Vec::new();
        // for i in 0..self.num_dimension{
        // ct_val_vec.push(F::one());
        // ct_var_vec.push(cs.new_input_variable(|| Ok(ct_val_vec[i]))?);
        // }
        //
        // witness input
        // suppose f1(x)=1,f2(x)=0,f3(x)=0,...
        // let mut r_val_vec = Vec::new();
        // let mut r_var_vec = Vec::new();
        // r_val_vec.push(F::one());
        // r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[0]))?);
        // for i in 1..self.num_dimension{
        // r_val_vec.push(F::zero());
        // r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[i]))?);
        // }
        // for j in 0..self.num_poly - 1{
        // for i in 0..self.num_dimension{
        // r_val_vec.push(F::zero());
        // r_var_vec.push(cs.new_witness_variable(|| Ok(r_val_vec[i +
        // (j+1)*self.num_dimension]))?); }
        // }
        //
        // let mut agg_r_val_vec = Vec::new();
        // let mut agg_r_var_vec = Vec::new();
        //
        // aggregate several polynomials
        // for j in 0..self.num_dimension {
        // let mut tmp_lc = LinearCombination::zero();
        // let mut tmp_val = F::zero();
        // let x = F::one();
        // for i in 0..self.num_poly {
        // tmp_lc = tmp_lc + (x, r_var_vec[j+ self.num_dimension * i]);
        // tmp_val = tmp_val + x * r_val_vec[j+ self.num_dimension * i];
        // }
        // agg_r_val_vec.push(tmp_val);
        // agg_r_var_vec.push(cs.new_witness_variable(||
        // Ok(agg_r_val_vec[j]))?); cs.enforce_constraint(lc!() +
        // tmp_lc, lc!() + Variable::One, lc!() + agg_r_var_vec[j])?; }
        //
        // mod reduction
        // let mut agg_r_val_mod_vec = Vec::new();
        // let mut agg_r_var_mod_vec = Vec::new();
        // witness input
        // let mut agg_r_val_div_vec = Vec::new();
        // let mut agg_r_var_div_vec = Vec::new();
        // for i in 0..self.num_dimension{
        // agg_r_val_div_vec.push(F::zero());
        // agg_r_var_div_vec.push(cs.new_witness_variable(||
        // Ok(agg_r_val_div_vec[i]))?); agg_r_val_mod_vec.push(F::
        // zero()); agg_r_var_mod_vec.push(cs.new_witness_variable(||
        // Ok(agg_r_val_mod_vec[i]))?); agg_r_var_mod_vec.push(cs.
        // new_witness_variable(|| Ok(agg_r_val_vec[i]))?); }
        // for j in 0..self.num_dimension {
        // cs.enforce_constraint(lc!() + agg_r_var_vec[j], lc!() +
        // Variable::One, lc!() + agg_r_var_mod_vec[j] +
        // (pf,agg_r_var_div_vec[j]))?; }
        //
        // range proof
        //
        // each number for bit decomposition
        // let mut m_val_vec = Vec::new();
        // let mut m_var_vec = Vec::new();
        //
        // for j in 0..self.num_poly {
        // for i in 0..self.num_dimension{
        // m_val_vec.push(F::zero());
        // m_var_vec.push(cs.new_witness_variable(|| Ok(m_val_vec[i +
        // j*self.num_dimension]))?); }
        // }
        //
        // let mut mbit_val_vec = Vec::new();
        // let mut mbit_var_vec = Vec::new();
        // for j in 0..self.num_poly {
        // for i in 0..self.num_dimension{
        // let mut bit_val_vec = Vec::new();
        // let mut bit_var_vec = Vec::new();
        // let mut tmp_lc = LinearCombination::zero();
        // let mut x = F::one();
        // for k in 0..8 {
        // bit_val_vec.push(F::zero());
        // bit_var_vec.push(cs.new_witness_variable(|| Ok(bit_val_vec[k]))?);
        // }
        // for k in 0..8 {
        // tmp_lc = tmp_lc + (x, bit_var_vec[k]);
        // x = x + x;
        // bit
        // if k % 2 == 0 && k+1 == 7 {
        // cs.enforce_constraint(lc!() + bit_var_vec[k] + bit_var_vec[k+1],
        // lc!() + bit_var_vec[k] + (-F::one(), bit_var_vec[k+1]), lc!() +
        // bit_var_vec[k])?; }
        // else if k % 2 == 0 && k+1 < 7 {
        // cs.enforce_constraint(lc!() + bit_var_vec[k] +
        // (F::one(),bit_var_vec[k+1]),    lc!() + bit_var_vec[k] +
        // (-F::one(), bit_var_vec[k+1]), lc!() +
        // bit_var_vec[k]+(F::one(),bit_var_vec[k+1]))?; for simplicity,
        // assume k = 1 here cs.enforce_constraint(lc!() +
        // bit_var_vec[k] + bit_var_vec[k+1], lc!() + bit_var_vec[k] +
        // (-F::one(), bit_var_vec[k+1]), lc!() + bit_var_vec[k] +
        // bit_var_vec[k+1])?; }
        // if k == 7 {
        //     cs.enforce_constraint(lc!() + bit_var_vec[k], lc!() +
        // Variable::One + (-F::one(), bit_var_vec[k]), lc!())?; }
        // else {
        //     // x(x-1) = 0
        //     cs.enforce_constraint(lc!() + bit_var_vec[k], lc!() +
        // Variable::One + (-F::one(), bit_var_vec[k]), lc!())?; }
        // }
        // mbit_val_vec.push(bit_val_vec);
        // mbit_var_vec.push(bit_var_vec);
        // bit decompose
        // cs.enforce_constraint(lc!() + tmp_lc, lc!() + Variable::One, lc!() +
        // m_var_vec[i+j*self.num_dimension])?; }
        // }
        //
        // NTT for polynomial multiplication
        // /
        // let mut agg_m_val_vec = Vec::new();
        // let mut agg_m_var_vec = Vec::new();
        //
        // for j in 0..self.num_dimension {
        // let mut tmp_lc = LinearCombination::zero();
        // let mut tmp_val = F::zero();
        // let x = F::one();
        // for i in 0..self.num_poly {
        // tmp_lc = tmp_lc + (-x, m_var_vec[j+ self.num_dimension * i]);
        // tmp_val = tmp_val - x * m_val_vec[j+ self.num_dimension * i];
        // }
        // tmp_lc = tmp_lc + (F::one(),ct_var_vec[j]);
        // tmp_val = tmp_val + ct_val_vec[j];
        // agg_m_val_vec.push(tmp_val);
        // agg_m_var_vec.push(cs.new_witness_variable(||
        // Ok(agg_m_val_vec[j]))?); cs.enforce_constraint(lc!() +
        // tmp_lc, lc!() + Variable::One, lc!() + agg_m_var_vec[j])?; }
        // mod reduction
        // let mut agg_m_val_mod_vec = Vec::new();
        // let mut agg_m_var_mod_vec = Vec::new();
        // witness input
        // let mut agg_m_val_div_vec = Vec::new();
        // let mut agg_m_var_div_vec = Vec::new();
        // for i in 0..self.num_dimension{
        // agg_m_val_div_vec.push(F::zero());
        // agg_m_var_div_vec.push(cs.new_witness_variable(||
        // Ok(agg_m_val_div_vec[i]))?); agg_m_val_mod_vec.push(F::
        // zero()); agg_r_var_mod_vec.push(cs.new_witness_variable(||
        // Ok(agg_m_val_mod_vec[i]))?); agg_m_var_mod_vec.push(cs.
        // new_witness_variable(|| Ok(agg_m_val_vec[i]))?); }
        // for j in 0..self.num_dimension {
        // cs.enforce_constraint(lc!() + agg_m_var_vec[j], lc!() +
        // Variable::One, lc!() + agg_m_var_mod_vec[j] +
        // (pf,agg_m_var_div_vec[j]))?; }
        //
        //
        // linear combination of f(x)
        // A_lc[i] = f(root^i)
        // B_lc[i] = f(root^i)
        // t_root = root ^ i;
        // tmp_root = root ^ ij;
        // let mut t_root = primitive_root;
        // let mut A_lc = Vec::new();
        // let mut B_lc = Vec::new();
        // let mut C_lc = Vec::new();
        // for i in 0..self.num_constraints {
        // for _ in 0..self.num_dimension{
        // let mut tmp_root = t_root;
        // let mut left_lc = LinearCombination::zero();
        // let mut right_lc = LinearCombination::zero();
        // let mut result_lc = LinearCombination::zero();
        // for j in 0..self.num_dimension{
        // left_lc = left_lc + (tmp_root, pk_var_vec[j]);
        // right_lc = right_lc + (tmp_root, agg_r_var_mod_vec[j]);
        // result_lc = result_lc + (tmp_root, agg_m_var_mod_vec[j]);
        // tmp_root = tmp_root * t_root;
        // }
        // t_root = t_root * primitive_root;
        // A_lc.push(d_lc);
        // B_lc.push(e_lc);
        // C_lc.push(tmp_lc);
        // cs.enforce_constraint(lc!()+left_lc,lc!()+right_lc,lc!()+result_lc)?;
        // }
        //
        // new witness ct_var_vec = NTT(ct_var_vec)
        // for i in 0..self.num_constraints {
        //
        // }
        //
        //
        // This is naive implementation */
        // / intermediate result
        // / d[i] =a[0]*b[i] + a[1]*b[i-1] + ... + a[i]b[0] - a[i+1]b[n-1] -
        // a[i+2]b[n-2] -... for i in 0..self.num_constraints {
        //    let mut d_val = F::zero();
        //    let mut d_lc = LinearCombination::zero();
        //    for j in 0..self.num_constraints {
        //        if j  <= i {
        //            let t_val = pk_val_vec[j] * &r_val_vec[i-j];
        //            let t_var = cs.new_witness_variable(|| Ok(t_val))?;
        //            cs.enforce_constraint(lc!() + pk_var_vec[j], lc!() +
        // r_var_vec[i-j], lc!() + t_var)?;            d_val = d_val +
        // &t_val;            d_lc = d_lc + t_var;
        //        }
        //        else {
        //            let t_val = -pk_val_vec[j] *
        // &r_val_vec[self.num_constraints - j + i];            let
        // t_var = cs.new_witness_variable(|| Ok(t_val))?;
        // cs.enforce_constraint(lc!() + pk_var_vec[j], lc!() +
        // r_var_vec[self.num_constraints - j + i], lc!() + t_var)?;
        //            d_val = d_val + &t_val;
        //            d_lc = d_lc + t_var;
        //        }
        //    }
        //    let d_var = cs.new_witness_variable(|| Ok(d_val))?;
        //    cs.enforce_constraint(lc!() + d_lc, lc!() + Variable::One, lc!() +
        // d_var)?; }
        // println!("# of constraints {}",cs.num_constraints());
        // println!("# of instances {}",cs.num_instance_variables());
        // println!("# of witness {}",cs.num_witness_variables());
        // Ok(())
    }
}
