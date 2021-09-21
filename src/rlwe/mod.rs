//use ark_std::{end_timer, start_timer};
use log::error;

pub const NUM_DIMENSION: usize = 4096;
pub const MODULUS: i128 = 649033470896967801447398927572993i128;

pub mod rand_poly;
//TODO make pk_0 and pk_1 matrix of scalar
/// [p0, -p4095, ..., -p1]
/// [p1, p0, -p4095, ..., -p2] * [r0 r1 ... r4095] = [c0, c1, ..., c4095]
///        ...
/// [p4095, p4094, ..., p0]
pub struct PublicKey {
    pub pk_0: Vec<i128>,
    pub pk_1: Vec<i128>,
}

pub struct Ciphertext {
    pub c_0: Vec<i128>,
    pub c_1: Vec<i128>,
}

impl PublicKey {
    pub fn new(opk0: &Vec<i128>, opk1: &Vec<i128>) -> Self {
        if opk0.len() < NUM_DIMENSION || opk1.len() < NUM_DIMENSION {
            error!("Not enough elements when creating the public key");
        }

        let mut pk0 = opk0.clone();
        let mut pk1 = opk1.clone();

        let pk_0 = {
            let mut tmp_pk = Vec::<i128>::with_capacity(NUM_DIMENSION * NUM_DIMENSION);
            pk0.reverse();
            pk0.rotate_right(1);
            pk0.iter_mut().for_each(|x| *x = -*x);
            pk0[0] = -pk0[0];
            tmp_pk.extend(pk0.iter());
            for _ in 0..NUM_DIMENSION - 1 {
                pk0.rotate_right(1);
                pk0[0] = -pk0[0];
                tmp_pk.extend(pk0.iter());
            }
            tmp_pk
        };
        let pk_1 = {
            let mut tmp_pk = Vec::<i128>::with_capacity(NUM_DIMENSION * NUM_DIMENSION);
            pk1.reverse();
            pk1.rotate_right(1);
            pk1.iter_mut().for_each(|x| *x = -*x);
            pk1[0] = -pk1[0];
            tmp_pk.extend(pk1.iter());
            for _ in 0..NUM_DIMENSION - 1 {
                pk1.rotate_right(1);
                pk1[0] = -pk1[0];
                tmp_pk.extend(pk1.iter());
            }
            tmp_pk
        };

        PublicKey { pk_0, pk_1 }
    }

    pub fn matrix_mut(mat: &Vec<i128>, v: &Vec<i128>) -> Vec<i128> {
        let mut ret = Vec::<i128>::with_capacity(NUM_DIMENSION);
        for i in 0..NUM_DIMENSION {
            let mut s: i128 = 0;
            for j in 0..NUM_DIMENSION {
                //s = s + v[i] * mat[i * NUM_DIMENSION + j];
                s = s + v[j] * mat[i * NUM_DIMENSION + j];
            }
            ret.push(s);
        }
        ret
    }
    // TODO maybe accelerate the matrix multiplication here
    /// message will be consumed
    pub fn encrypt(
        &self,
        m: Vec<u8>,
    ) -> (
        Vec<i128>,
        Vec<i128>,
        Vec<i128>,
        Vec<i32>,
        Vec<i32>,
        Ciphertext,
    ) {
        let r = rand_poly::sample_ternary();
        let e0 = rand_poly::sample_gaussian();
        let e1 = rand_poly::sample_gaussian();
        // 109-bit * 4096 * 2 = 122 bit
        let mut pkr0 = PublicKey::matrix_mut(&self.pk_0, &r);
        let mut pkr1 = PublicKey::matrix_mut(&self.pk_1, &r);
        for i in 0..NUM_DIMENSION {
            pkr0[i] += e0[i];
            pkr1[i] += e1[i] + m[i] as i128;
        }
        let delta_0 = pkr0.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();
        let delta_1 = pkr1.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();
        pkr0.iter_mut().for_each(|x| *x = x.rem_euclid(MODULUS));
        pkr1.iter_mut().for_each(|x| *x = x.rem_euclid(MODULUS));
        (
            r,
            e0,
            e1,
            delta_0,
            delta_1,
            Ciphertext {
                c_0: pkr0,
                c_1: pkr1,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::File,
        io::{prelude::*, BufReader},
        time::Instant,
    };
    fn encrypt_internal(
        pk: &PublicKey,
        m: Vec<i128>,
        r: Vec<i128>,
        e0: Vec<i128>,
        e1: Vec<i128>,
        d0: Vec<i128>,
        d1: Vec<i128>,
    ) -> Ciphertext {
        // 109-bit * 4096 * 2 = 122 bit
        let mut pkr0 = PublicKey::matrix_mut(pk.pk_0.as_ref(), &r);
        let mut pkr1 = PublicKey::matrix_mut(pk.pk_1.as_ref(), &r);
        for i in 0..NUM_DIMENSION {
            pkr0[i] += e0[i];
            pkr1[i] += e1[i] + m[i];
        }
        let delta_0: Vec<i32> = pkr0.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();
        let delta_1: Vec<i32> = pkr1.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();

        for i in 0..NUM_DIMENSION {
            assert_eq!(delta_0[i], d0[i] as i32, "different {}", i);
            assert_eq!(delta_1[i], d1[i] as i32, "different {}", i);
        }

        pkr0.iter_mut().for_each(|x| *x = x.rem_euclid(MODULUS));
        pkr1.iter_mut().for_each(|x| *x = x.rem_euclid(MODULUS));
        Ciphertext {
            c_0: pkr0.to_vec(),
            c_1: pkr1.to_vec(),
        }
    }

    // run the following 2 tests with RUST_MIN_STACK=8388608 cargo test test_create_proof --release
    #[test]
    fn test_create_public_key() {
        let mut c_0 = [0i128; 4096];
        let mut c_1 = [0i128; 4096];
        let mut r = [0i128; 4096];
        let mut e_0 = [0i128; 4096];
        let mut e_1 = [0i128; 4096];
        let mut m = [0i128; 4096];
        let mut pk_0 = [1i128; 4096];
        let mut pk_1 = [0i128; 4096];
        let mut delta_0 = [0i128; 4096];
        let mut delta_1 = [0i128; 4096];
        let file = match File::open("./data/encryption.txt") {
            Ok(f) => f,
            Err(_) => panic!(),
        };
        let reader = BufReader::new(file);
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
                    } else if l.contains("c_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            c_1[i - 1] = x;
                        }
                    } else if l.contains("pk_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_0[i - 1] = x;
                        }
                    } else if l.contains("pk_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            pk_1[i - 1] = x;
                        }
                    } else if l.contains("e_0") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            e_0[i - 1] = x;
                        }
                    } else if l.contains("e_1") {
                        if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                            e_1[i - 1] = x;
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
        let public_key = PublicKey::new(&pk_0.to_vec(), &pk_1.to_vec());
        let start = Instant::now();
        let ct = encrypt_internal(
            &public_key,
            m.to_vec(),
            r.to_vec(),
            e_0.to_vec(),
            e_1.to_vec(),
            delta_0.to_vec(),
            delta_1.to_vec(),
        );
        let d = start.elapsed();
        let t = d.subsec_nanos() as f64 / 1_000_000_000f64 + (d.as_secs() as f64);
        println!("time to encrypt {} seconds", t);

        for i in 0..NUM_DIMENSION {
            assert_eq!(ct.c_0[i], c_0[i]);
            assert_eq!(ct.c_1[i], c_1[i]);
        }
    }
}
