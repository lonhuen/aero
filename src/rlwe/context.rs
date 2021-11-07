use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use cupcake::polyarith::lazy_ntt::{lazy_inverse_ntt_u64, lazy_ntt_u64};
use cupcake::rqpoly::RqPolyContext;
use rand::{Rng, SeedableRng};

use super::NUM_DIMENSION;

pub const MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];

pub struct ShamirContext {
    pub modulus: Scalar,
    pub share_cnt: usize,
    pub threshold: usize,
    pub eval_matrix: Vec<Vec<Scalar>>,
}

impl ShamirContext {
    pub fn init(prime: u64, share_cnt: usize, threshold: usize) -> Self {
        let modulus = Scalar::new_modulus(prime);
        let mut eval_matrix = vec![vec![Scalar::zero(); threshold + 1]; share_cnt];
        for i in 0..share_cnt {
            eval_matrix[i][0] = Scalar::one();
            let pt = Scalar::from((i + 1) as u64);
            let mut x_pow = Scalar::from((i + 1) as u64);
            for j in 1..threshold + 1 {
                eval_matrix[i][j] = x_pow.clone();
                x_pow = Scalar::mul_mod(&x_pow, &pt, &modulus);
            }
        }
        Self {
            modulus,
            share_cnt,
            threshold,
            eval_matrix,
        }
    }
    fn sample_polynomial(&self, secret: u64) -> Vec<Scalar> {
        let mut poly = vec![Scalar::from(secret)];
        //poly.extend((0..self.threshold).map(|_| Scalar::sample_blw(&self.modulus)));
        //poly
        //poly.extend((0..self.threshold).map(|_| Scalar::one() * 2));
        let mut rng = rand::rngs::StdRng::from_entropy();
        poly.extend(
            (0..self.threshold).map(|_| Scalar::from(rng.gen_range(0..self.modulus.rep() - 1))),
            //(0..self.threshold).map(|_| Scalar::from(rng.gen_range(0..10u64))),
        );
        poly
    }

    fn evaluate_polynomial(&self, poly: &Vec<Scalar>, x: usize) -> u64 {
        let mut s = Scalar::zero();
        for i in 0..poly.len() {
            // the addition will not exceed 64-bit
            s = Scalar::add_mod(
                &s,
                &Scalar::mul_mod(&self.eval_matrix[x][i], &poly[i], &self.modulus),
                &self.modulus,
            );
        }
        //Scalar::modulus(&s, &self.modulus).rep()
        s.rep()
    }

    pub fn share(&self, secret: u64) -> Vec<u64> {
        let mut ret = Vec::with_capacity(self.share_cnt);
        let poly = self.sample_polynomial(secret);
        for i in 0..self.share_cnt {
            let y = self.evaluate_polynomial(&poly, i);
            ret.push(y);
        }
        ret
    }

    pub fn lagrange_interpolation_at_zero(
        &self,
        points: &Vec<Scalar>,
        values: &Vec<Scalar>,
    ) -> u64 {
        let q = &self.modulus;
        let mut acc = Scalar::zero();
        for i in 0..values.len() {
            let xi = points[i].clone();
            let yi = values[i].clone();
            let mut num = Scalar::one();
            let mut denum = Scalar::one();
            for j in 0..values.len() {
                if j != i {
                    let xj = points[j].clone();
                    //num = (num * xj) % prime;
                    num = Scalar::mul_mod(&num, &xj, q);
                    //denum = (denum * (xj - xi)) % prime;
                    denum = Scalar::mul_mod(&denum, &Scalar::sub_mod(&xj, &xi, q), q);
                }
            }
            acc = Scalar::add_mod(
                &acc,
                &Scalar::mul_mod(
                    &yi,
                    &Scalar::mul_mod(&num, &Scalar::inv_mod(&denum, q), q),
                    q,
                ),
                q,
            );
        }
        acc.rep()
    }

    pub fn reconstruct(&self, shares: &Vec<u64>) -> u64 {
        let recovered_indices: Vec<usize> = (0..self.threshold + 1).collect();
        let recovered_shares: Vec<Scalar> = shares[0..self.threshold + 1]
            .iter()
            .map(|x| Scalar::from(*x))
            .collect();

        let points: Vec<Scalar> = recovered_indices
            .iter()
            .map(|&i| Scalar::from((i as u64) + 1u64))
            .collect();
        let reconstructed_secret = self.lagrange_interpolation_at_zero(&points, &recovered_shares);
        //reconstructed_secret as u64
        Scalar::modulus(&Scalar::from(reconstructed_secret as u64), &self.modulus).rep()
    }
}

pub struct NTTContext {
    pub modulus: Scalar,
    pub roots_u64: Vec<u64>,
    pub scaledroots_u64: Vec<u64>,
    pub invroots_u64: Vec<u64>,
    pub scaled_invroots_u64: Vec<u64>,
    pub ninv: Scalar,
}

impl NTTContext {
    pub fn init(prime: u64) -> Self {
        let modulus = Scalar::new_modulus(prime);
        let q = &modulus;
        let context = RqPolyContext::new(NUM_DIMENSION, q);
        let roots_u64 = context.roots.iter().map(|elm| elm.rep()).collect();
        let scaledroots_u64 = context.scaled_roots.iter().map(|elm| elm.rep()).collect();
        let invroots_u64 = context.invroots.iter().map(|elm| elm.rep()).collect();
        let scaled_invroots_u64 = context
            .scaled_invroots
            .iter()
            .map(|elm| elm.rep())
            .collect();
        let ninv = Scalar::inv_mod(&Scalar::from_u32(NUM_DIMENSION as u32, q), q);
        Self {
            modulus,
            roots_u64,
            scaledroots_u64,
            invroots_u64,
            scaled_invroots_u64,
            ninv,
        }
    }

    pub fn lazy_ntt_inplace(&self, a: &mut [u64]) {
        lazy_ntt_u64(
            a,
            &self.roots_u64,
            &self.scaledroots_u64,
            self.modulus.rep(),
        );
        a.iter_mut()
            .for_each(|x| *x = Scalar::modulus(&Scalar::from(*x), &self.modulus).rep());
    }

    pub fn lazy_inverse_ntt_inplace(&self, a: &mut [u64]) {
        lazy_inverse_ntt_u64(
            a,
            &self.invroots_u64,
            &self.scaled_invroots_u64,
            self.modulus.rep(),
        );
        a.iter_mut()
            .for_each(|x| *x = Scalar::mul_mod(&self.ninv, &Scalar::from(*x), &self.modulus).rep());
    }

    pub fn coeff_mul_mod(&self, a: &Vec<u64>, b: &Vec<u64>) -> Vec<u64> {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                Scalar::mul_mod(&Scalar::from(*x), &Scalar::from(*y), &self.modulus).rep()
            })
            .collect()
    }
}

/*
/** Chinese remainder theorem

```
use ring_algorithm::chinese_remainder_theorem;
let u = vec![2, 3, 2];
let m = vec![3, 5, 7];
let a = chinese_remainder_theorem::<i32>(&u, &m).unwrap();
for (u, m) in u.iter().zip(m.iter()) {
    assert_eq!((a - u) % m, 0);
}
```
*/
pub fn chinese_remainder_theorem<T>(u: &[T], m: &[T]) -> Option<T>
where
    T: sealed::Sized + Clone + Eq + num_traits::Zero + num_traits::One + RingNormalize,
    for<'x> &'x T: EuclideanRingOperation<T>,
{
    if u.len() != m.len() {
        return None;
    }
    let mut v = Vec::with_capacity(u.len());
    for (i, (u_i, m_i)) in u.iter().zip(m.iter()).enumerate() {
        let coef_i = modulo_inverse::<T>(
            m[0..i].iter().fold(T::one(), |p, v| &(&p * v) % m_i),
            m_i.clone(),
        )?;
        let t = v
            .iter()
            .zip(m.iter())
            .rev()
            .fold(T::zero(), |t, (v_j, m_j)| &(&(m_j * &t) + v_j) % m_i);
        v.push(&(&(u_i - &t) * &coef_i) % m_i);
    }
    let mut ret = v.pop().unwrap();
    for (v_i, m_i) in v.iter().zip(m.iter()).rev() {
        ret = &(&ret * m_i) + v_i;
    }
    Some(ret)
}
*/
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_share_and_reshare() {
        let context = ShamirContext::init(0xffffee001u64, 10, 4);
        for _ in 0..(4096 * 342 * 40 / 20) {
            let secret = Scalar::sample_blw(&context.modulus).rep();
            let shares = context.share(secret);
            let ret = context.reconstruct(&shares);
            assert_eq!(ret, secret);
        }
    }

    #[test]
    fn test_ntt_inv_ntt() {
        let context = NTTContext::init(0xffffee001u64);
        let mut a = vec![1u64; 4096];
        let mut b = vec![0u64; 4096];
        b[0] = 1;
        context.lazy_ntt_inplace(&mut a);
        context.lazy_ntt_inplace(&mut b);
        let mut ntt_c = context.coeff_mul_mod(&a, &b);
        context.lazy_inverse_ntt_inplace(&mut ntt_c);
    }
}
