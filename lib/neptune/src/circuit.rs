use crate::hash_type::HashType;
use crate::matrix::Matrix;
use crate::mds::SparseMatrix;
use crate::poseidon::{Arity, PoseidonConstants};
use ark_bls12_381::Fr as ArkFr;
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, OptimizationGoal,
        SynthesisError, Variable,
    },
};
use ff::Field;
use ff::ScalarEngine as Engine;
use std::marker::PhantomData;

use crate::bls381num::AllocatedNum;
use crate::bls381num::{self, bp2ark};
//use bellperson::gadgets::num;
//use bellperson::gadgets::num::AllocatedNum;
//use bellperson::{ConstraintSystem, LinearCombination, SynthesisError};
//use bellperson::gadgets::boolean::Boolean;

/// Similar to `num::Num`, we use `Elt` to accumulate both values and linear combinations, then eventually
/// extract into a `num::AllocatedNum`, enforcing that the linear combination corresponds to the result.
/// In this way, all intermediate calculations are accounted for, with the restriction that we can only
/// accumulate linear (not polynomial) constraints. The set of operations provided here ensure this invariant is maintained.
#[derive(Clone)]
enum Elt<E: Engine> {
    Allocated(AllocatedNum<E>),
    Num(bls381num::Num<E>),
}

impl<E: Engine> Elt<E> {
    fn is_allocated(&self) -> bool {
        matches!(self, Self::Allocated(_))
    }

    fn is_num(&self) -> bool {
        matches!(self, Self::Num(_))
    }

    fn num_from_fr(fr: E::Fr) -> Self {
        Self::Num(bls381num::Num::from_fr(fr))
    }

    fn ensure_allocated(
        &self,
        cs: &ConstraintSystemRef<ArkFr>,
        enforce: bool,
    ) -> Result<AllocatedNum<E>, SynthesisError> {
        match self {
            Self::Allocated(v) => Ok(v.clone()),
            Self::Num(num) => {
                let v = AllocatedNum::alloc(cs, || {
                    num.get_value().ok_or(SynthesisError::AssignmentMissing)
                })?;

                if enforce {
                    cs.enforce_constraint(
                        lc!() + num.lc(E::Fr::one()),
                        lc!() + Variable::One,
                        lc!() + v.get_variable(),
                    )?;
                    //cs.enforce(
                    //    || "enforce num allocation preserves lc".to_string(),
                    //    |_| num.lc(E::Fr::one()),
                    //    |lc| lc + CS::one(),
                    //    |lc| lc + v.get_variable(),
                    //);
                }
                Ok(v)
            }
        }
    }

    fn val(&self) -> Option<E::Fr> {
        match self {
            Self::Allocated(v) => v.get_value(),
            Self::Num(num) => num.get_value(),
        }
    }

    fn lc(&self) -> LinearCombination<ArkFr> {
        match self {
            Self::Num(num) => num.lc(E::Fr::one()),
            Self::Allocated(v) => lc!() + v.get_variable(),
        }
    }

    /// Add two Nums and return a Num tracking the calculation. It is forbidden to invoke on an Allocated because the intended computation
    /// does not include that path.
    fn add(self, other: Elt<E>) -> Result<Elt<E>, SynthesisError> {
        match (self, other) {
            (Elt::Num(a), Elt::Num(b)) => Ok(Elt::Num(a.add(&b))),
            _ => panic!("only two numbers may be added"),
        }
    }

    /// Scale
    fn scale(self, scalar: E::Fr) -> Result<Elt<E>, SynthesisError> {
        match self {
            Elt::Num(num) => Ok(Elt::Num(num.scale(scalar))),
            Elt::Allocated(a) => Elt::Num(a.into()).scale(scalar),
        }
    }
}

/// Circuit for Poseidon hash.
pub struct PoseidonCircuit<'a, E, A>
where
    E: Engine,
    A: Arity<E::Fr>,
{
    constants_offset: usize,
    width: usize,
    elements: Vec<Elt<E>>,
    pos: usize,
    current_round: usize,
    constants: &'a PoseidonConstants<E, A>,
    _w: PhantomData<A>,
}

/// PoseidonCircuit implementation.
impl<'a, E, A> PoseidonCircuit<'a, E, A>
where
    E: Engine,
    A: Arity<E::Fr>,
{
    /// Create a new Poseidon hasher for `preimage`.
    fn new(elements: Vec<Elt<E>>, constants: &'a PoseidonConstants<E, A>) -> Self {
        let width = constants.width();

        PoseidonCircuit {
            constants_offset: 0,
            width,
            elements,
            pos: width,
            current_round: 0,
            constants,
            _w: PhantomData::<A>,
        }
    }

    fn hash(&mut self, cs: &ConstraintSystemRef<ArkFr>) -> Result<AllocatedNum<E>, SynthesisError> {
        self.full_round(cs, true, false)?;

        for i in 1..self.constants.full_rounds / 2 {
            self.full_round(cs, false, false)?;
        }

        for i in 0..self.constants.partial_rounds {
            self.partial_round(cs)?;
        }

        for i in 0..(self.constants.full_rounds / 2) - 1 {
            self.full_round(cs, false, false)?;
        }
        self.full_round(cs, false, true)?;

        self.elements[1].ensure_allocated(cs, true)
    }

    fn full_round(
        &mut self,
        cs: &ConstraintSystemRef<ArkFr>,
        first_round: bool,
        last_round: bool,
    ) -> Result<(), SynthesisError> {
        let mut constants_offset = self.constants_offset;

        let pre_round_keys = if first_round {
            (0..self.width)
                .map(|i| self.constants.compressed_round_constants[constants_offset + i])
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        constants_offset += pre_round_keys.len();

        let post_round_keys = if first_round || !last_round {
            (0..self.width)
                .map(|i| self.constants.compressed_round_constants[constants_offset + i])
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        constants_offset += post_round_keys.len();

        // Apply the quintic S-Box to all elements
        for i in 0..self.elements.len() {
            let pre_round_key = if first_round {
                let rk = pre_round_keys[i];
                Some(rk)
            } else {
                None
            };

            let post_round_key = if first_round || !last_round {
                let rk = post_round_keys[i];
                Some(rk)
            } else {
                None
            };

            if first_round {
                if i == 0 {
                    // The very first s-box for the constant arity tag can also be computed statically, as a constant.
                    self.elements[i] = constant_quintic_s_box_pre_add_tag::<E>(
                        &self.elements[i],
                        pre_round_key,
                        post_round_key,
                    );
                } else {
                    self.elements[i] = quintic_s_box_pre_add(
                        cs,
                        &self.elements[i],
                        pre_round_key,
                        post_round_key,
                    )?;
                }
            } else {
                self.elements[i] = quintic_s_box(cs, &self.elements[i], post_round_key)?;
            }
        }
        self.constants_offset = constants_offset;

        // Multiply the elements by the constant MDS matrix
        self.product_mds()?;
        Ok(())
    }

    fn partial_round(&mut self, cs: &ConstraintSystemRef<ArkFr>) -> Result<(), SynthesisError> {
        let round_key = self.constants.compressed_round_constants[self.constants_offset];
        self.constants_offset += 1;
        // Apply the quintic S-Box to the first element.
        self.elements[0] = quintic_s_box(cs, &self.elements[0], Some(round_key))?;

        // Multiply the elements by the constant MDS matrix
        self.product_mds()?;
        Ok(())
    }

    fn product_mds_m(&mut self) -> Result<(), SynthesisError> {
        self.product_mds_with_matrix(&self.constants.mds_matrices.m)
    }

    /// Set the provided elements with the result of the product between the elements and the appropriate
    /// MDS matrix.
    #[allow(clippy::collapsible_else_if)]
    fn product_mds(&mut self) -> Result<(), SynthesisError> {
        let full_half = self.constants.half_full_rounds;
        let sparse_offset = full_half - 1;
        if self.current_round == sparse_offset {
            self.product_mds_with_matrix(&self.constants.pre_sparse_matrix)?;
        } else {
            if (self.current_round > sparse_offset)
                && (self.current_round < full_half + self.constants.partial_rounds)
            {
                let index = self.current_round - sparse_offset - 1;
                let sparse_matrix = &self.constants.sparse_matrixes[index];

                self.product_mds_with_sparse_matrix(&sparse_matrix)?;
            } else {
                self.product_mds_m()?;
            }
        };

        self.current_round += 1;
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    fn product_mds_with_matrix(&mut self, matrix: &Matrix<E::Fr>) -> Result<(), SynthesisError> {
        let mut result: Vec<Elt<E>> = Vec::with_capacity(self.constants.width());

        for j in 0..self.constants.width() {
            let column = (0..self.constants.width())
                .map(|i| matrix[i][j])
                .collect::<Vec<_>>();

            let product = scalar_product::<E>(self.elements.as_slice(), &column)?;

            result.push(product);
        }

        self.elements = result;

        Ok(())
    }

    // Sparse matrix in this context means one of the form, M''.
    fn product_mds_with_sparse_matrix(
        &mut self,
        matrix: &SparseMatrix<E>,
    ) -> Result<(), SynthesisError> {
        let mut result: Vec<Elt<E>> = Vec::with_capacity(self.constants.width());

        result.push(scalar_product::<E>(
            self.elements.as_slice(),
            &matrix.w_hat,
        )?);

        for j in 1..self.width {
            result.push(
                self.elements[j].clone().add(
                    self.elements[0]
                        .clone() // First row is dense.
                        .scale(matrix.v_rest[j - 1])?, // Except for first row/column, diagonals are one.
                )?,
            );
        }

        self.elements = result;

        Ok(())
    }

    fn debug(&self) {
        let element_frs: Vec<_> = self.elements.iter().map(|n| n.val()).collect::<Vec<_>>();
        dbg!(element_frs, self.constants_offset);
    }
}

/// Create circuit for Poseidon hash.
pub fn poseidon_hash<E, A>(
    cs: &ConstraintSystemRef<ArkFr>,
    preimage: Vec<AllocatedNum<E>>,
    constants: &PoseidonConstants<E, A>,
) -> Result<AllocatedNum<E>, SynthesisError>
where
    E: Engine,
    A: Arity<E::Fr>,
{
    let arity = A::to_usize();
    let tag_element = Elt::num_from_fr(constants.domain_tag);
    let mut elements = Vec::with_capacity(arity + 1);
    elements.push(tag_element);
    elements.extend(preimage.into_iter().map(Elt::Allocated));

    if let HashType::ConstantLength(length) = constants.hash_type {
        assert!(length <= arity, "illegal length: constants are malformed");
        // Add zero-padding.
        for i in 0..(arity - length) {
            let allocated = AllocatedNum::alloc(cs, || Ok(E::Fr::zero()))?;
            let elt = Elt::Allocated(allocated);
            elements.push(elt);
        }
    }

    let mut p = PoseidonCircuit::new(elements, constants);

    p.hash(cs)
}

/// Compute l^5 and enforce constraint. If round_key is supplied, add it to result.
fn quintic_s_box<E: Engine>(
    cs: &ConstraintSystemRef<ArkFr>,
    e: &Elt<E>,
    post_round_key: Option<E::Fr>,
) -> Result<Elt<E>, SynthesisError> {
    let l = e.ensure_allocated(cs, true)?;

    // If round_key was supplied, add it after all exponentiation.
    let l2 = l.square(cs)?;
    let l4 = l2.square(cs)?;
    let l5 = mul_sum(cs, &l4, &l, None, post_round_key, true);

    Ok(Elt::Allocated(l5?))
}

/// Compute l^5 and enforce constraint. If round_key is supplied, add it to l first.
fn quintic_s_box_pre_add<E: Engine>(
    cs: &ConstraintSystemRef<ArkFr>,
    e: &Elt<E>,
    pre_round_key: Option<E::Fr>,
    post_round_key: Option<E::Fr>,
) -> Result<Elt<E>, SynthesisError> {
    if let (Some(pre_round_key), Some(post_round_key)) = (pre_round_key, post_round_key) {
        let l = e.ensure_allocated(cs, true)?;

        // If round_key was supplied, add it to l before squaring.
        let l2 = square_sum(cs, pre_round_key, &l, true)?;
        let l4 = l2.square(cs)?;
        let l5 = mul_sum(cs, &l4, &l, Some(pre_round_key), Some(post_round_key), true);

        Ok(Elt::Allocated(l5?))
    } else {
        panic!("pre_round_key and post_round_key must both be provided.");
    }
}

/// Compute l^5 and enforce constraint. If round_key is supplied, add it to l first.
fn constant_quintic_s_box_pre_add_tag<E: Engine>(
    tag: &Elt<E>,
    pre_round_key: Option<E::Fr>,
    post_round_key: Option<E::Fr>,
) -> Elt<E> {
    let mut tag = tag.val().expect("missing tag val");
    pre_round_key.expect("pre_round_key must be provided");
    post_round_key.expect("post_round_key must be provided");

    crate::quintic_s_box::<E>(&mut tag, pre_round_key.as_ref(), post_round_key.as_ref());

    Elt::num_from_fr(tag)
}

/// Calculates square of sum and enforces that constraint.
pub fn square_sum<E: Engine>(
    cs: &ConstraintSystemRef<ArkFr>,
    to_add: E::Fr,
    num: &AllocatedNum<E>,
    enforce: bool,
) -> Result<AllocatedNum<E>, SynthesisError> {
    let res = AllocatedNum::alloc(cs, || {
        let mut tmp = num.get_value().ok_or(SynthesisError::AssignmentMissing)?;
        tmp.add_assign(&to_add);
        tmp.square();

        Ok(tmp)
    })?;

    if enforce {
        cs.enforce_constraint(
            lc!() + num.get_variable() + (bp2ark(to_add), Variable::One),
            lc!() + num.get_variable() + (bp2ark(to_add), Variable::One),
            lc!() + res.get_variable(),
        )?;
        //cs.enforce(
        //    || "squared sum constraint",
        //    |lc| lc + num.get_variable() + (to_add, CS::one()),
        //    |lc| lc + num.get_variable() + (to_add, CS::one()),
        //    |lc| lc + res.get_variable(),
        //);
    }
    Ok(res)
}

/// Calculates (a * (pre_add + b)) + post_add — and enforces that constraint.
#[allow(clippy::collapsible_else_if)]
pub fn mul_sum<E: Engine>(
    cs: &ConstraintSystemRef<ArkFr>,
    a: &AllocatedNum<E>,
    b: &AllocatedNum<E>,
    pre_add: Option<E::Fr>,
    post_add: Option<E::Fr>,
    enforce: bool,
) -> Result<AllocatedNum<E>, SynthesisError> {
    let res = AllocatedNum::alloc(cs, || {
        let mut tmp = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
        if let Some(x) = pre_add {
            tmp.add_assign(&x);
        }
        tmp.mul_assign(&a.get_value().ok_or(SynthesisError::AssignmentMissing)?);
        if let Some(x) = post_add {
            tmp.add_assign(&x);
        }

        Ok(tmp)
    })?;

    if enforce {
        if let Some(x) = post_add {
            let mut neg = E::Fr::zero();
            neg.sub_assign(&x);

            if let Some(pre) = pre_add {
                cs.enforce_constraint(
                    lc!() + b.get_variable() + (bp2ark(pre), Variable::One),
                    lc!() + a.get_variable(),
                    lc!() + res.get_variable() + (bp2ark(neg), Variable::One),
                )?;
                //cs.enforce(
                //    || "mul sum constraint pre-post-add",
                //    |lc| lc + b.get_variable() + (pre, CS::one()),
                //    |lc| lc + a.get_variable(),
                //    |lc| lc + res.get_variable() + (neg, CS::one()),
                //);
            } else {
                cs.enforce_constraint(
                    lc!() + b.get_variable(),
                    lc!() + a.get_variable(),
                    lc!() + res.get_variable() + (bp2ark(neg), Variable::One),
                )?;
                //cs.enforce(
                //    || "mul sum constraint post-add",
                //    |lc| lc + b.get_variable(),
                //    |lc| lc + a.get_variable(),
                //    |lc| lc + res.get_variable() + (neg, CS::one()),
                //);
            }
        } else {
            if let Some(pre) = pre_add {
                cs.enforce_constraint(
                    lc!() + b.get_variable() + (bp2ark(pre), Variable::One),
                    lc!() + a.get_variable(),
                    lc!() + res.get_variable(),
                )?;
                //cs.enforce(
                //    || "mul sum constraint pre-add",
                //    |lc| lc + b.get_variable() + (pre, CS::one()),
                //    |lc| lc + a.get_variable(),
                //    |lc| lc + res.get_variable(),
                //);
            } else {
                cs.enforce_constraint(
                    lc!() + b.get_variable(),
                    lc!() + a.get_variable(),
                    lc!() + res.get_variable(),
                )?;
                //cs.enforce(
                //    || "mul sum constraint",
                //    |lc| lc + b.get_variable(),
                //    |lc| lc + a.get_variable(),
                //    |lc| lc + res.get_variable(),
                //);
            }
        }
    }
    Ok(res)
}

/// Calculates a * (b + to_add) — and enforces that constraint.
pub fn mul_pre_sum<E: Engine>(
    cs: &ConstraintSystemRef<ArkFr>,
    a: &AllocatedNum<E>,
    b: &AllocatedNum<E>,
    to_add: E::Fr,
    enforce: bool,
) -> Result<AllocatedNum<E>, SynthesisError> {
    let res = AllocatedNum::alloc(cs, || {
        let mut tmp = b.get_value().ok_or(SynthesisError::AssignmentMissing)?;
        tmp.add_assign(&to_add);
        tmp.mul_assign(&a.get_value().ok_or(SynthesisError::AssignmentMissing)?);

        Ok(tmp)
    })?;

    if enforce {
        cs.enforce_constraint(
            lc!() + b.get_variable() + (bp2ark(to_add), Variable::One),
            lc!() + a.get_variable(),
            lc!() + res.get_variable(),
        )?;
        //cs.enforce(
        //    || "mul sum constraint",
        //    |lc| lc + b.get_variable() + (to_add, CS::one()),
        //    |lc| lc + a.get_variable(),
        //    |lc| lc + res.get_variable(),
        //);
    }
    Ok(res)
}

fn scalar_product_with_add<E: Engine>(
    elts: &[Elt<E>],
    scalars: &[E::Fr],
    to_add: E::Fr,
) -> Result<Elt<E>, SynthesisError> {
    let tmp = scalar_product::<E>(elts, scalars)?;
    let tmp2 = tmp.add(Elt::<E>::num_from_fr(to_add))?;

    Ok(tmp2)
}

fn scalar_product<E: Engine>(elts: &[Elt<E>], scalars: &[E::Fr]) -> Result<Elt<E>, SynthesisError> {
    elts.iter()
        .zip(scalars)
        .try_fold(Elt::Num(bls381num::Num::zero()), |acc, (elt, &scalar)| {
            acc.add(elt.clone().scale(scalar)?)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poseidon::HashMode;
    use crate::{scalar_from_u64, Poseidon, Strength};
    use bellperson::bls::{Bls12, Fr};

    //use bellperson::util_cs::test_cs::TestConstraintSystem;
    //use bellperson::ConstraintSystem;
    use ark_relations::{
        lc,
        r1cs::{
            ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, LinearCombination,
            OptimizationGoal, SynthesisError, Variable,
        },
    };
    use generic_array::typenum;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    #[test]
    fn test_poseidon_hash() {
        test_poseidon_hash_aux::<typenum::U2>(Strength::Standard, 311, false);
        test_poseidon_hash_aux::<typenum::U4>(Strength::Standard, 377, false);
        test_poseidon_hash_aux::<typenum::U8>(Strength::Standard, 505, false);
        test_poseidon_hash_aux::<typenum::U16>(Strength::Standard, 761, false);
        test_poseidon_hash_aux::<typenum::U24>(Strength::Standard, 1009, false);
        test_poseidon_hash_aux::<typenum::U36>(Strength::Standard, 1385, false);

        test_poseidon_hash_aux::<typenum::U2>(Strength::Strengthened, 367, false);
        test_poseidon_hash_aux::<typenum::U4>(Strength::Strengthened, 433, false);
        test_poseidon_hash_aux::<typenum::U8>(Strength::Strengthened, 565, false);
        test_poseidon_hash_aux::<typenum::U16>(Strength::Strengthened, 821, false);
        test_poseidon_hash_aux::<typenum::U24>(Strength::Strengthened, 1069, false);
        test_poseidon_hash_aux::<typenum::U36>(Strength::Strengthened, 1445, false);

        test_poseidon_hash_aux::<typenum::U15>(Strength::Standard, 730, true);
    }

    fn test_poseidon_hash_aux<A>(
        strength: Strength,
        expected_constraints: usize,
        constant_length: bool,
    ) where
        A: Arity<<Bls12 as Engine>::Fr>,
    {
        let mut rng = XorShiftRng::from_seed(crate::TEST_SEED);
        let arity = A::to_usize();
        let constants_x = if constant_length {
            PoseidonConstants::<Bls12, A>::new_with_strength_and_type(
                strength,
                HashType::ConstantLength(arity),
            )
        } else {
            PoseidonConstants::<Bls12, A>::new_with_strength(strength)
        };

        let range = if constant_length {
            1..=arity
        } else {
            arity..=arity
        };
        for preimage_length in range {
            //let mut cs = TestConstraintSystem::<Bls12>::new();
            let cs_sys = ConstraintSystem::<ArkFr>::new();
            let cs = ConstraintSystemRef::new(cs_sys);

            let constants = if constant_length {
                constants_x.with_length(preimage_length)
            } else {
                constants_x.clone()
            };
            let expected_constraints_calculated = {
                let arity_tag_constraints = 0;
                let width = 1 + arity;
                // The '- 1' term represents the first s-box for the arity tag, which is a constant and needs no constraint.
                let s_boxes = (width * constants.full_rounds) + constants.partial_rounds - 1;
                let s_box_constraints = 3 * s_boxes;
                let mds_constraints =
                    (width * constants.full_rounds) + constants.partial_rounds - arity;
                arity_tag_constraints + s_box_constraints + mds_constraints
            };
            let mut i = 0;

            let mut fr_data = vec![Fr::zero(); preimage_length];
            let data: Vec<AllocatedNum<Bls12>> = (0..preimage_length)
                .enumerate()
                .map(|_| {
                    let fr = Fr::random(&mut rng);
                    fr_data[i] = fr;
                    i += 1;
                    AllocatedNum::alloc(&cs, || Ok(fr)).unwrap()
                })
                .collect::<Vec<_>>();

            let out = poseidon_hash(&cs, data, &constants).expect("poseidon hashing failed");

            let mut p = Poseidon::<Bls12, A>::new_with_preimage(&fr_data, &constants);
            let expected: Fr = p.hash_in_mode(HashMode::Correct);

            assert!(cs.is_satisfied().unwrap(), "constraints not satisfied");

            assert_eq!(
                expected,
                out.get_value().unwrap(),
                "circuit and non-circuit do not match"
            );

            assert_eq!(
                expected_constraints_calculated,
                cs.num_constraints(),
                "constraint number miscalculated"
            );

            assert_eq!(
                expected_constraints,
                cs.num_constraints(),
                "constraint number changed",
            );
            println!(
                "num of constraints {} for input length {}",
                cs.num_constraints(),
                preimage_length
            );
        }
    }

    fn fr(n: u64) -> <Bls12 as Engine>::Fr {
        scalar_from_u64::<<Bls12 as Engine>::Fr>(n)
    }

    fn efr(n: u64) -> Elt<Bls12> {
        Elt::num_from_fr(fr(n))
    }

    #[test]
    fn test_square_sum() {
        let cs_sys = ConstraintSystem::<ArkFr>::new();
        let cs = ConstraintSystemRef::new(cs_sys);

        let two = fr(2);
        let three = AllocatedNum::<Bls12>::alloc(&cs, || Ok(scalar_from_u64(3))).unwrap();
        let res = square_sum(&cs, two, &three, true).unwrap();

        let twenty_five: Fr = scalar_from_u64(25);
        assert_eq!(twenty_five, res.get_value().unwrap());
    }
    #[test]
    fn test_scalar_product() {
        {
            // Inputs are all linear combinations.
            let two = efr(2);
            let three = efr(3);
            let four = efr(4);

            let res = scalar_product::<Bls12>(&[two, three, four], &[fr(5), fr(6), fr(7)]).unwrap();

            assert!(res.is_num());
            assert_eq!(scalar_from_u64::<Fr>(56), res.val().unwrap());
        }
        {
            let cs_sys = ConstraintSystem::<ArkFr>::new();
            let cs = ConstraintSystemRef::new(cs_sys);

            // Inputs are linear combinations and an allocated number.
            let two = efr(2);

            let n3 = AllocatedNum::alloc(&cs, || Ok(scalar_from_u64(3))).unwrap();
            let three = Elt::Allocated(n3.clone());
            let n4 = AllocatedNum::alloc(&cs, || Ok(scalar_from_u64(4))).unwrap();
            let four = Elt::Allocated(n4.clone());

            let res = scalar_product::<Bls12>(&[two, three, four], &[fr(5), fr(6), fr(7)]).unwrap();

            assert!(res.is_num());
            assert_eq!(scalar_from_u64::<Fr>(56), res.val().unwrap());

            res.lc().iter().for_each(|(f, var)| {
                if var.eq(&n3.get_variable()) {
                    assert_eq!(bls381num::ark2bp::<Fr>(*f), fr(6));
                };
                if var.eq(&n4.get_variable()) {
                    assert_eq!(bls381num::ark2bp::<Fr>(*f), fr(7));
                };
            });

            res.ensure_allocated(&cs, true).unwrap();
            assert!(cs.is_satisfied().unwrap());
        }
        {
            let cs_sys = ConstraintSystem::<ArkFr>::new();
            let cs = ConstraintSystemRef::new(cs_sys);

            // Inputs are linear combinations and an allocated number.
            let two = efr(2);

            let n3 = AllocatedNum::alloc(&cs, || Ok(scalar_from_u64(3))).unwrap();
            let three = Elt::Allocated(n3.clone());
            let n4 = AllocatedNum::alloc(&cs, || Ok(scalar_from_u64(4))).unwrap();
            let four = Elt::Allocated(n4.clone());

            let mut res_vec = Vec::new();

            let res = scalar_product::<Bls12>(&[two, three, four], &[fr(5), fr(6), fr(7)]).unwrap();

            res_vec.push(res);

            assert!(res_vec[0].is_num());
            assert_eq!(fr(56), res_vec[0].val().unwrap());

            res_vec[0].lc().iter().for_each(|(f, var)| {
                if var.eq(&n3.get_variable()) {
                    assert_eq!(bls381num::ark2bp::<Fr>(*f), fr(6));
                };
                if var.eq(&n4.get_variable()) {
                    assert_eq!(bls381num::ark2bp::<Fr>(*f), fr(7));
                };
            });

            let four2 = Elt::Allocated(n4.clone());
            res_vec.push(efr(3));
            res_vec.push(four2);
            let res2 = scalar_product::<Bls12>(&res_vec, &[fr(7), fr(8), fr(9)]).unwrap();

            res2.lc().iter().for_each(|(f, var)| {
                if var.eq(&n3.get_variable()) {
                    assert_eq!(bls381num::ark2bp::<Fr>(*f), fr(42));
                };
                if var.eq(&n4.get_variable()) {
                    assert_eq!(bls381num::ark2bp::<Fr>(*f), fr(58));
                };
            });

            let allocated = res2.ensure_allocated(&cs, true).unwrap();

            let v = allocated.get_value().unwrap();
            assert_eq!(fr(452), v); // (7 * 56) + (8 * 3) + (9 * 4) = 448

            assert!(cs.is_satisfied().unwrap());
        }
    }

    #[test]
    fn test_scalar_product_with_add() {
        let two = efr(2);
        let three = efr(3);
        let four = efr(4);

        let res =
            scalar_product_with_add::<Bls12>(&[two, three, four], &[fr(5), fr(6), fr(7)], fr(3))
                .unwrap();

        assert!(res.is_num());
        assert_eq!(fr(59), res.val().unwrap());
    }
}
