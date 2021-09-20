use ark_std::{end_timer, start_timer};
use log::error;
use ndarray::prelude::*;
use ndarray::ArcArray2;
use std::collections::BTreeMap;

pub const NUM_DIMENSION: usize = 4096;
pub const MODULUS: i128 = 649033470896967801447398927572993i128;

pub mod rand_poly;
//TODO make pk_0 and pk_1 matrix of scalar
///                       [p0, p1, ..., p4095]
/// [r0, r1,.., r4095] *  [-p4095, p0, p1, ..., p4094] = [c0, c1, ..., c4095]
///                         ...
///                       [-p1, -p2, ..., p0]
pub struct PublicKey {
    pub pk_0: ArcArray2<i128>,
    pub pk_1: ArcArray2<i128>,
}

pub struct Ciphertext {
    pub c_0: Vec<i128>,
    pub c_1: Vec<i128>,
}

impl PublicKey {
    pub fn new(mut pk0: Vec<i128>, mut pk1: Vec<i128>) -> Self {
        if pk0.len() < NUM_DIMENSION || pk1.len() < NUM_DIMENSION {
            error!("Not enough elements when creating the public key");
        }

        let mut tmp_pk = Vec::<i128>::with_capacity(NUM_DIMENSION * NUM_DIMENSION);
        tmp_pk.extend(pk0.iter());
        for _ in 0..NUM_DIMENSION - 1 {
            pk0.rotate_right(1);
            pk0[0] = -pk0[0];
            tmp_pk.extend(pk0.iter());
        }
        let pk_0 =
            ArcArray2::<i128>::from_shape_vec((NUM_DIMENSION, NUM_DIMENSION), tmp_pk).unwrap();

        let mut tmp_pk = Vec::<i128>::with_capacity(NUM_DIMENSION * NUM_DIMENSION);
        tmp_pk.extend(pk1.iter());
        for _ in 0..NUM_DIMENSION - 1 {
            pk1.rotate_right(1);
            pk1[0] = -pk1[0];
            tmp_pk.extend(pk1.iter());
        }
        let pk_1 =
            ArcArray2::<i128>::from_shape_vec((NUM_DIMENSION, NUM_DIMENSION), tmp_pk).unwrap();

        PublicKey { pk_0, pk_1 }
    }

    // TODO maybe accelerate the matrix multiplication here
    /// message will be consumed
    pub fn encrypt(
        &self,
        m: Vec<i128>,
    ) -> (
        Vec<i128>,
        Vec<i128>,
        Vec<i128>,
        Vec<i32>,
        Vec<i32>,
        Ciphertext,
    ) {
        let gc = start_timer!(|| "sample noise and r");
        let r = rand_poly::sample_ternary();
        let e0 = rand_poly::sample_gaussian();
        let e1 = rand_poly::sample_gaussian();
        end_timer!(gc);
        // 109-bit * 4096 * 2 = 122 bit
        let gc = start_timer!(|| "dot product with matrix");
        let array_r = Array1::<i128>::from_vec(r.clone());
        let mut pkr0 = array_r.dot(&self.pk_0.to_shared()) + Array1::<i128>::from_vec(e0.clone());
        let mut pkr1 = array_r.dot(&self.pk_1.to_shared())
            + Array1::<i128>::from_vec(e1.clone())
            + Array1::<i128>::from_vec(m);
        end_timer!(gc);
        let gc = start_timer!(|| "mod division");
        //pkr0.map_inplace(|x| *x = (*x + (NUM_DIMENSION * 2) as i128 * MODULUS) % MODULUS);
        //pkr1.map_inplace(|x| *x = (*x + (NUM_DIMENSION * 2) as i128 * MODULUS) % MODULUS);
        let delta_0 = pkr0.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();
        let delta_1 = pkr1.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();
        pkr0.map_inplace(|x| *x = x.rem_euclid(MODULUS));
        pkr1.map_inplace(|x| *x = x.rem_euclid(MODULUS));
        end_timer!(gc);
        (
            r,
            e0,
            e1,
            delta_0,
            delta_1,
            Ciphertext {
                c_0: pkr0.to_vec(),
                c_1: pkr1.to_vec(),
            },
        )
    }
    pub fn encrypt_internal(
        &self,
        m: Vec<i128>,
        r: Vec<i128>,
        e0: Vec<i128>,
        e1: Vec<i128>,
        d0: Vec<i128>,
        d1: Vec<i128>,
    ) -> Ciphertext {
        // 109-bit * 4096 * 2 = 122 bit
        let array_r = Array1::<i128>::from_vec(r.clone());
        let mut pkr0 = array_r.dot(&self.pk_0.to_shared()) + Array1::<i128>::from_vec(e0.clone());
        let mut pkr1 = array_r.dot(&self.pk_1.to_shared())
            + Array1::<i128>::from_vec(e1.clone())
            + Array1::<i128>::from_vec(m);
        //pkr0.map_inplace(|x| *x = (*x + 4096 * 2 * MODULUS) % MODULUS);
        //pkr1.map_inplace(|x| *x = (*x + 4096 * 2 * MODULUS) % MODULUS);
        let delta_0: Vec<i32> = pkr0.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();
        let delta_1: Vec<i32> = pkr1.iter().map(|x| x.div_euclid(MODULUS) as i32).collect();

        for i in 0..NUM_DIMENSION {
            assert_eq!(delta_0[i], d0[i] as i32, "different {}", i);
            assert_eq!(delta_1[i], d1[i] as i32, "different {}", i);
        }

        pkr0.map_inplace(|x| *x = x.rem_euclid(MODULUS));
        pkr1.map_inplace(|x| *x = x.rem_euclid(MODULUS));
        Ciphertext {
            c_0: pkr0.to_vec(),
            c_1: pkr1.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::File,
        io::{prelude::*, BufReader},
        time::Duration,
        time::Instant,
    };

    // run the following 2 tests with RUST_MIN_STACK=8388608 cargo test test_create_proof --release
    #[test]
    fn test_create_public_key() {
        let mut c_0 = [0i128; 4096];
        let mut c_1 = [0i128; 4096];
        let mut r = [1i128; 4096];
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
        let public_key = PublicKey::new(pk_0.to_vec(), pk_1.to_vec());
        let start = Instant::now();
        let ct = public_key.encrypt_internal(
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
