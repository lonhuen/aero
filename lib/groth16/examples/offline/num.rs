use ark_ff::{BigInteger, BigInteger256, Field};
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
    },
};
use ark_std::{end_timer, start_timer, test_rng};
use std::marker::PhantomData;

pub struct AllocatedNum<F: Field> {
    value: Option<F>,
    variable: Variable,
}

impl<F: Field> Clone for AllocatedNum<F> {
    fn clone(&self) -> Self {
        AllocatedNum {
            value: self.value,
            variable: self.variable,
        }
    }
}

impl<F: Field> AllocatedNum<F> {
    pub fn alloc<E>(cs: &ConstraintSystemRef<F>, value: E) -> Result<Self, SynthesisError>
    where
        E: FnOnce() -> Result<F, SynthesisError>,
    {
        let mut new_value = None;
        let var = cs.new_witness_variable(|| {
            let tmp = value()?;
            new_value = Some(tmp);
            Ok(tmp)
        })?;
        Ok(AllocatedNum {
            value: new_value,
            variable: var,
        })
    }
    pub fn square(&self, cs: &ConstraintSystemRef<F>) -> Result<Self, SynthesisError> {
        let mut value = None;
        let var = cs.new_witness_variable(|| {
            let mut tmp = self.value.unwrap();
            tmp.square_in_place();
            value = Some(tmp);
            Ok(tmp)
        })?;
        cs.enforce_constraint(lc!() + self.variable, lc!() + self.variable, lc!() + var)?;
        Ok(AllocatedNum {
            value,
            variable: var,
        })
    }
    pub fn mul(&self, cs: &ConstraintSystemRef<F>, other: &Self) -> Result<Self, SynthesisError> {
        let mut value = None;
        let var = cs.new_witness_variable(|| {
            let tmp = self.value.unwrap() * other.value.unwrap();
            value = Some(tmp);
            Ok(tmp)
        })?;
        cs.enforce_constraint(lc!() + self.variable, lc!() + other.variable, lc!() + var)?;
        Ok(AllocatedNum {
            value,
            variable: var,
        })
    }

    pub fn get_value(&self) -> Option<F> {
        self.value
    }
    pub fn get_variable(&self) -> Variable {
        self.variable
    }
}

#[derive(Clone)]
pub struct Num<F: Field> {
    value: Option<F>,
    lc: LinearCombination<F>,
}

impl<F: Field> From<AllocatedNum<F>> for Num<F> {
    fn from(num: AllocatedNum<F>) -> Num<F> {
        Num {
            value: num.value,
            lc: lc!() + num.variable,
        }
    }
}

impl<F: Field> Num<F> {
    pub fn zero() -> Self {
        Num {
            value: Some(F::zero()),
            lc: lc!(),
        }
    }
    pub fn from_fr(fr: F) -> Self {
        Num {
            value: Some(fr),
            lc: lc!() + (fr, Variable::One),
        }
    }

    pub fn get_value(&self) -> Option<F> {
        self.value
    }

    pub fn lc(&self, coeff: F) -> LinearCombination<F> {
        LinearCombination::zero() + (coeff, &self.lc)
    }

    pub fn add(self, other: &Self) -> Self {
        let lc = self.lc + &other.lc;
        let value = match (self.value, other.value) {
            (Some(v1), Some(v2)) => {
                let mut tmp = v1;
                tmp.add_assign(&v2);
                Some(tmp)
            },
            (Some(v), None) | (None, Some(v)) => Some(v),
            (None, None) => None,
        };

        Num { value, lc }
    }

    pub fn scale(mut self, scalar: F) -> Self {
        for (fr, _variable) in self.lc.0.iter_mut() {
            fr.mul_assign(&scalar);
        }

        if let Some(ref mut v) = self.value {
            v.mul_assign(&scalar);
        }

        self
    }
}
