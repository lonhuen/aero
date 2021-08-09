// the coeff modulus used in seal for 4k 128-bit security is { 4096, {
// 0xffffee001, 0xffffc4001, 0x1ffffe0001 }} : 109-bit the modulus used here is
// 0x12AB655E9A2CA55660B44D1E5C37B00159AA76FED00000010A11800000000001 64 * 4 =
// 256 bit 	gaussianSampler := ring.NewGaussianSampler(prng, q, params.Sigma(),
// int(6*params.Sigma())) DefaultSigma in ckks = 3.2
// bound of error = 3.2 . * 6 = 19
use ark_ff::{Field, PrimeField};
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, OptimizationGoal,
        SynthesisError, Variable,
    },
};
use std::marker::PhantomData;
#[path = "num.rs"]
mod num;
use num::AllocatedNum;

#[derive(Clone)]
enum Elt<F: Field> {
    Allocated(AllocatedNum<F>),
    Num(num::Num<F>),
}

impl<F: Field> Elt<F> {
    fn is_allocated(&self) -> bool {
        matches!(self, Self::Allocated(_))
    }

    fn is_num(&self) -> bool {
        matches!(self, Self::Num(_))
    }

    fn num_from_fr(fr: F) -> Self {
        Self::Num(num::Num::from_fr(fr))
    }

    fn ensure_allocated(
        &self,
        cs: &ConstraintSystemRef<F>,
        enforce: bool,
    ) -> Result<AllocatedNum<F>, SynthesisError> {
        match self {
            Self::Allocated(v) => Ok(v.clone()),
            Self::Num(num) => {
                let v = AllocatedNum::alloc(cs, || {
                    num.get_value().ok_or(SynthesisError::AssignmentMissing)
                })?;

                if enforce {
                    cs.enforce_constraint(
                        num.lc(F::one()),
                        lc!() + Variable::One,
                        lc!() + v.get_variable(),
                    )?;
                }
                Ok(v)
            },
        }
    }

    fn val(&self) -> Option<F> {
        match self {
            Self::Allocated(v) => v.get_value(),
            Self::Num(num) => num.get_value(),
        }
    }

    fn lc(&self) -> LinearCombination<F> {
        match self {
            Self::Num(num) => num.lc(F::one()),
            Self::Allocated(v) => lc!() + v.get_variable(),
        }
    }

    /// Add two Nums and return a Num tracking the calculation. It is forbidden
    /// to invoke on an Allocated because the intended computation
    /// does not include that path.
    fn add(self, other: Elt<F>) -> Result<Elt<F>, SynthesisError> {
        match (self, other) {
            (Elt::Num(a), Elt::Num(b)) => Ok(Elt::Num(a.add(&b))),
            _ => panic!("only two numbers may be added"),
        }
    }

    /// Scale
    fn scale(self, scalar: F) -> Result<Elt<F>, SynthesisError> {
        match self {
            Elt::Num(num) => Ok(Elt::Num(num.scale(scalar))),
            Elt::Allocated(a) => Elt::Num(a.into()).scale(scalar),
        }
    }
}

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
        // let pf = F::from(0xffffee001u64) * F::from(0xffffc4001u64) *
        // F::from(0x1ffffe0001u64);
        let pf = F::from(0x2u64);
        let a = Elt::num_from_fr(pf);
        let b = a.ensure_allocated(&cs, true)?;
        // let a = AllocatedNum::alloc(&cs, || Ok(pf))?;
        // println!("{}", a.get_value().unwrap_or(F::zero()));
        // let b = a.square(&cs)?;
        // println!("{}", b.get_value().unwrap_or(F::zero()));
        // let c = a.mul(&cs, &b)?;
        // println!("{}", c.get_value().unwrap_or(F::zero()));
        // let d = Num::from(a).add(&b.into());
        // println!("{}", d.get_value().unwrap_or(F::zero()));
        // for k in d.scale(pf).lc(pf).0 {
        //    println!("{}", k.0);
        //}
    }
}
