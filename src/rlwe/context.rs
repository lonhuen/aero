use cupcake::integer_arith::scalar::Scalar;
use cupcake::integer_arith::ArithUtils;
use cupcake::polyarith::lazy_ntt::{lazy_inverse_ntt_u64, lazy_ntt_u64};
use cupcake::rqpoly::RqPolyContext;
use ring_algorithm::chinese_remainder_theorem;
use threshold_secret_sharing as tss;

use super::NUM_DIMENSION;

pub const NTT_MODULUS: [u64; 3] = [0xffffee001u64, 0xffffc4001u64, 0x1ffffe0001u64];

pub struct Context {
    pub modulus: Vec<Scalar>,
    pub roots_u64: Vec<Vec<u64>>,
    pub scaledroots_u64: Vec<Vec<u64>>,
    pub invroots_u64: Vec<Vec<u64>>,
    pub scaled_invroots_u64: Vec<Vec<u64>>,
    pub ninv: Vec<Scalar>,
}

impl Context {
    pub fn init_default() -> Self {
        let mut modulus = Vec::new();
        let mut roots_u64: Vec<Vec<u64>> = Vec::new();
        let mut scaledroots_u64: Vec<Vec<u64>> = Vec::new();
        let mut invroots_u64: Vec<Vec<u64>> = Vec::new();
        let mut scaled_invroots_u64: Vec<Vec<u64>> = Vec::new();
        let mut ninv = Vec::new();
        for i in 0..3usize {
            let q = Scalar::new_modulus(NTT_MODULUS[i]);
            let context = RqPolyContext::new(NUM_DIMENSION, &q);
            roots_u64.push(context.roots.iter().map(|elm| elm.rep()).collect());
            scaledroots_u64.push(context.scaled_roots.iter().map(|elm| elm.rep()).collect());
            invroots_u64.push(context.invroots.iter().map(|elm| elm.rep()).collect());
            scaled_invroots_u64.push(
                context
                    .scaled_invroots
                    .iter()
                    .map(|elm| elm.rep())
                    .collect(),
            );
            ninv.push(Scalar::inv_mod(&Scalar::from_u32(4096 as u32, &q), &q));
            modulus.push(q);
        }
        Context {
            modulus,
            roots_u64,
            scaledroots_u64,
            invroots_u64,
            scaled_invroots_u64,
            ninv,
        }
    }

    pub fn lazy_ntt_inplace(&self, a: &mut Vec<Vec<u64>>) {
        for i in 0..a.len() {
            lazy_ntt_u64(
                &mut a[i],
                &self.roots_u64[i],
                &self.scaledroots_u64[i],
                self.modulus[i].rep(),
            );
            a[i].iter_mut()
                .for_each(|x| *x = x.rem_euclid(self.modulus[i].rep()));
        }
    }

    pub fn lazy_inverse_ntt_inplace(&self, a: &mut Vec<Vec<u64>>) {
        for i in 0..a.len() {
            lazy_inverse_ntt_u64(
                &mut a[i],
                &self.invroots_u64[i],
                &self.scaled_invroots_u64[i],
                self.modulus[i].rep(),
            );
            a[i].iter_mut().for_each(|x| {
                *x = Scalar::mul_mod(
                    &self.ninv[i],
                    &Scalar::modulus(&Scalar::from(*x), &self.modulus[i]),
                    &self.modulus[i],
                )
                .rep()
            });
        }
    }

    pub fn coeffwise_mult(&self, a: &Vec<Vec<u64>>, b: &Vec<Vec<u64>>) -> Vec<Vec<u64>> {
        let mut ret = Vec::new();
        for i in 0..a.len() {
            let sa: Vec<Scalar> = a[i]
                .iter()
                .map(|x| Scalar::modulus(&Scalar::from(*x), &self.modulus[i]))
                .collect();
            let sb: Vec<Scalar> = b[i]
                .iter()
                .map(|x| Scalar::modulus(&Scalar::from(*x), &self.modulus[i]))
                .collect();

            let sc: Vec<u64> = sa
                .iter()
                .zip(sb.iter())
                .map(|(x, y)| Scalar::mul_mod(x, y, &self.modulus[i]).rep())
                .collect();
            ret.push(sc);
        }
        ret
    }

    pub fn crt_decode_vec(&self, v: &Vec<Vec<u64>>) -> Vec<i128> {
        let mut ret = Vec::with_capacity(v[0].len());
        let m = [
            self.modulus[0].rep() as i128,
            self.modulus[1].rep() as i128,
            self.modulus[2].rep() as i128,
        ];
        for i in 0..v[0].len() {
            ret.push(
                chinese_remainder_theorem::<i128>(
                    &[v[0][i] as i128, v[1][i] as i128, v[2][i] as i128],
                    &m,
                )
                .unwrap(),
            );
        }
        ret
    }
    pub fn crt_encode_vec(&self, v: &Vec<i128>) -> Vec<Vec<u64>> {
        let mut ret = Vec::new();
        ret.push(Vec::with_capacity(v.len()));
        ret.push(Vec::with_capacity(v.len()));
        ret.push(Vec::with_capacity(v.len()));
        let m = [
            self.modulus[0].rep() as i128,
            self.modulus[1].rep() as i128,
            self.modulus[2].rep() as i128,
        ];
        for i in 0..v.len() {
            ret[0].push(v[i].rem_euclid(m[0]) as u64);
            ret[1].push(v[i].rem_euclid(m[1]) as u64);
            ret[2].push(v[i].rem_euclid(m[2]) as u64);
        }
        ret
    }

    pub fn shamir_share_crt(
        &self,
        nr_players: usize,
        threshold: usize,
        poly: &Vec<Vec<u64>>,
    ) -> Vec<Vec<Vec<u64>>> {
        let mut ret: Vec<Vec<Vec<u64>>> = Vec::with_capacity(nr_players);
        for i in 0..nr_players {
            ret.push(poly.clone());
        }
        for i in 0..poly.len() {
            let ref tss = tss::shamir::ShamirSecretSharing {
                threshold: threshold,
                share_count: nr_players,
                prime: self.modulus[i].rep() as i64,
            };
            for j in 0..poly[0].len() {
                let shares = tss.share(poly[i][j] as i64);
                for k in 0..nr_players {
                    ret[k][i][j] = shares[k] as u64;
                }
            }
        }
        ret
    }
    pub fn shamir_share(
        &self,
        nr_players: usize,
        threshold: usize,
        poly: &Vec<i128>,
    ) -> Vec<Vec<i128>> {
        let to_share_crt = self.crt_encode_vec(poly);
        let shares_in_crt = self.shamir_share_crt(nr_players, threshold, &to_share_crt);
        let shares = shares_in_crt
            .iter()
            .map(|x| self.crt_decode_vec(x))
            .collect();
        shares
    }
    // pub fn shamir_reconstruct(&self, nr_players: usize, threshold: usize, poly: &Vec<Vec<u64>>) {
    // pub fn shamir_reconstruct(&self, nr_players: usize, threshold: usize, poly: &Vec<Vec<u64>>) {
    //     let mut ret: Vec<Vec<Vec<u64>>> = Vec::with_capacity(nr_players);
    //     for i in 0..nr_players {
    //         ret.push(poly.clone());
    //     }
    //     for i in 0..poly.len() {
    //         let ref tss = tss::shamir::ShamirSecretSharing {
    //             threshold: threshold,
    //             share_count: nr_players,
    //             prime: self.modulus[i].rep() as i64,
    //         };
    //         for j in 0..poly[0].len() {
    //             let shares = tss.share(poly[i][j] as i64);
    //             for k in 0..nr_players {
    //                 ret[k][i][j] = shares[k] as u64;
    //             }
    //         }
    //     }
    // }
}

//fn main() {
//    let q = Scalar::new_modulus(0xffffee001u64);
//    let context = RqPolyContext::new(4096, &q);
//    let mut a: Vec<u64> = vec![1u64; 4096];
//    let mut b: Vec<u64> = vec![0u64; 4096];
//    b[0] = 1;
//    let roots_u64: Vec<u64> = context.roots.iter().map(|elm| elm.rep()).collect();
//    let scaledroots_u64: Vec<u64> = context.scaled_roots.iter().map(|elm| elm.rep()).collect();
//    let invroots_u64: Vec<u64> = context.invroots.iter().map(|elm| elm.rep()).collect();
//    let scaled_invroots_u64: Vec<u64> = context
//        .scaled_invroots
//        .iter()
//        .map(|elm| elm.rep())
//        .collect();
//
//    let ninv = Scalar::inv_mod(&Scalar::from_u32(4096 as u32, &q), &q);
//    lazy_ntt_u64(&mut a, &roots_u64, &scaledroots_u64, q.rep());
//    lazy_ntt_u64(&mut b, &roots_u64, &scaledroots_u64, q.rep());
//
//    let sa: Vec<Scalar> = a
//        .iter()
//        .map(|x| Scalar::modulus(&Scalar::from(*x), &q))
//        .collect();
//    let sb: Vec<Scalar> = b
//        .iter()
//        .map(|x| Scalar::modulus(&Scalar::from(*x), &q))
//        .collect();
//
//    let mut sc: Vec<u64> = sa
//        .iter()
//        .zip(sb.iter())
//        .map(|(x, y)| Scalar::mul_mod(x, y, &q).rep())
//        .collect();
//    lazy_inverse_ntt_u64(&mut sc, &invroots_u64, &scaled_invroots_u64, q.rep());
//    sc.iter_mut().for_each(|x| {
//        *x = Scalar::mul_mod(&ninv, &Scalar::modulus(&Scalar::from(*x), &q), &q).rep()
//    });
//    println!("{:?}", sc);
//    // create instance of the Shamir scheme
//    let ref tss = tss::shamir::ShamirSecretSharing {
//        threshold: 8,    // privacy threshold
//        share_count: 20, // total number of shares to generate
//        prime: 41,       // prime field to use
//    };
//
//    let secret = 5;
//
//    // generate shares for secret
//    let all_shares = tss.share(secret);
//
//    // artificially remove some of the shares
//    let number_of_recovered_shared = 10;
//    assert!(number_of_recovered_shared >= tss.reconstruct_limit());
//    let recovered_indices: Vec<usize> = (0..number_of_recovered_shared).collect();
//    let recovered_shares: &[i64] = &all_shares[0..number_of_recovered_shared];
//
//    // reconstruct using remaining subset of shares
//    let reconstructed_secret = tss.reconstruct(&recovered_indices, recovered_shares);
//    assert_eq!(reconstructed_secret, secret);
//}
//
