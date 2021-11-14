use std::convert::TryInto;

use ark_bls12_381::Bls12_381 as ArkBls12;
use ark_bls12_381::Fr as ArkFr;
use ark_ff::biginteger::BigInteger256 as ArkBigInteger256;
use ark_ff::Field as ArkField;
use ark_ff::PrimeField as ArkPrimeField;
use ark_relations::{
    lc,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, OptimizationGoal,
        SynthesisError, Variable,
    },
};
use ff::{BitIterator, Field, PrimeField, PrimeFieldRepr, ScalarEngine};
use std::ops::MulAssign;

pub struct AllocatedNum<E: ScalarEngine> {
    value: Option<E::Fr>,
    variable: Variable,
}

impl<E: ScalarEngine> Clone for AllocatedNum<E> {
    fn clone(&self) -> Self {
        AllocatedNum {
            value: self.value,
            variable: self.variable,
        }
    }
}
pub fn bp2ark<F: PrimeField>(u: F) -> ArkFr {
    //    ArkFr::from_repr()
    ArkFr::from_repr(ArkBigInteger256::new(
        u.into_repr().as_ref().try_into().unwrap(),
    ))
    .unwrap()
}
pub fn ark2bp<F: PrimeField>(u: ArkFr) -> F {
    let d: Vec<_> = u
        .into_repr()
        .as_ref()
        .iter()
        .map(|x| (*x).to_le_bytes())
        .collect();
    let mut e = [0u8; 32];
    for i in 0..4 {
        for j in 0..8 {
            e[i * 8 + j] = d[i][j];
        }
    }
    F::from_random_bytes(&e).unwrap()
}

impl<E: ScalarEngine> AllocatedNum<E> {
    pub fn alloc<F>(cs: &ConstraintSystemRef<ArkFr>, value: F) -> Result<Self, SynthesisError>
    where
        F: FnOnce() -> Result<E::Fr, SynthesisError>,
    {
        let mut new_value = None;
        let var = cs.new_witness_variable(|| {
            let tmp = value()?;
            new_value = Some(tmp);
            Ok(bp2ark(tmp))
        })?;
        Ok(AllocatedNum {
            value: new_value,
            variable: var,
        })
    }
    pub fn square(&self, cs: &ConstraintSystemRef<ArkFr>) -> Result<Self, SynthesisError> {
        let mut value = None;
        let var = cs.new_witness_variable(|| {
            let mut tmp = self.value.unwrap();
            tmp.square();
            value = Some(tmp);
            Ok(bp2ark(tmp))
        })?;
        cs.enforce_constraint(lc!() + self.variable, lc!() + self.variable, lc!() + var)?;
        Ok(AllocatedNum {
            value,
            variable: var,
        })
    }
    pub fn mul(
        &self,
        cs: &ConstraintSystemRef<ArkFr>,
        other: &Self,
    ) -> Result<Self, SynthesisError> {
        let mut value = None;
        let var = cs.new_witness_variable(|| {
            let mut tmp = self.value.unwrap();
            tmp.mul_assign(&other.value.unwrap());
            value = Some(tmp);
            Ok(bp2ark(tmp))
        })?;
        cs.enforce_constraint(lc!() + self.variable, lc!() + other.variable, lc!() + var)?;
        Ok(AllocatedNum {
            value,
            variable: var,
        })
    }

    pub fn get_value(&self) -> Option<E::Fr> {
        self.value
    }
    pub fn get_variable(&self) -> Variable {
        self.variable
    }
}

#[derive(Clone)]
pub struct Num<E: ScalarEngine> {
    value: Option<E::Fr>,
    lc: LinearCombination<ArkFr>,
}

impl<E: ScalarEngine> From<AllocatedNum<E>> for Num<E> {
    fn from(num: AllocatedNum<E>) -> Num<E> {
        Num {
            value: num.value,
            lc: lc!() + num.variable,
        }
    }
}

impl<E: ScalarEngine> Num<E> {
    pub fn zero() -> Self {
        Num {
            value: Some(E::Fr::zero()),
            lc: lc!(),
        }
    }
    pub fn from_fr(fr: E::Fr) -> Self {
        Num {
            value: Some(fr),
            lc: lc!() + (bp2ark(fr), Variable::One),
        }
    }

    pub fn get_value(&self) -> Option<E::Fr> {
        self.value
    }

    pub fn lc(&self, coeff: E::Fr) -> LinearCombination<ArkFr> {
        lc!() + (bp2ark(coeff), &self.lc)
    }

    pub fn add(self, other: &Self) -> Self {
        let lc = self.lc + &other.lc;
        let value = match (self.value, other.value) {
            (Some(v1), Some(v2)) => {
                let mut tmp = v1;
                tmp.add_assign(&v2);
                Some(tmp)
            }
            (Some(v), None) | (None, Some(v)) => Some(v),
            (None, None) => None,
        };

        Num { value, lc }
    }

    pub fn scale(mut self, scalar: E::Fr) -> Self {
        for (fr, _variable) in self.lc.0.iter_mut() {
            fr.mul_assign(bp2ark(scalar));
        }

        if let Some(ref mut v) = self.value {
            v.mul_assign(&scalar);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use ark_bls12_381::Bls12_381 as ArkBls12;
    use ark_bls12_381::Fr as ArkFr;
    use ark_relations::r1cs::{ConstraintSystem, ConstraintSystemRef};
    use bellperson::bls::{Bls12, Fr};
    #[test]
    fn test_allocated_num() {
        let cs_sys = ConstraintSystem::<ArkFr>::new();
        let cs = ConstraintSystemRef::new(cs_sys);

        let a = AllocatedNum::<Bls12>::alloc(&cs, || Ok(Fr::from_str("2").unwrap())).unwrap();
        let b = a.square(&cs).unwrap();
        let c = a.mul(&cs, &b).unwrap();

        {
            let d = Num::<Bls12>::zero();
            let e = Num::<Bls12>::from_fr(Fr::from_str("2").unwrap());
            let f = d.add(&e);
            println!("{}", f.get_value().unwrap());
        }
        {
            let e = Num::<Bls12>::from_fr(Fr::from_str("2").unwrap());
            let d = Fr::from_str("2").unwrap();
            let f = e.scale(d);
            println!("{}", f.get_value().unwrap());
        }

        println!("{}", cs.num_constraints());
        println!("{}", cs.num_witness_variables());
    }
}
