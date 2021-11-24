use crate::common::i128vec_to_le_bytes;
#[cfg(feature = "hashfn_blake3")]
extern crate blake3;
#[cfg(not(feature = "hashfn_blake3"))]
use crypto::{digest::Digest, sha3::Sha3};
use rayon::iter::repeatn;
use rayon::prelude::*;
use std::ops::Add;
use tarpc::serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitEntry {
    pub rsa_pk: Vec<u8>,
    pub hash: [u8; 32],
}

impl CommitEntry {
    pub fn new() -> Self {
        CommitEntry {
            rsa_pk: vec![0u8; 500],
            hash: [0u8; 32],
        }
    }
    #[cfg(not(feature = "hashfn_blake3"))]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();
        hasher.input(&self.rsa_pk);
        hasher.input(&self.hash);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
    #[cfg(feature = "hashfn_blake3")]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.rsa_pk);
        hasher.update(&self.hash);
        hasher.finalize().into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SummationLeaf {
    pub rsa_pk: Vec<u8>,
    pub c0: Option<Vec<i128>>,
    pub c1: Option<Vec<i128>>,
    pub r: Option<[u8; 16]>,
}

impl SummationLeaf {
    pub fn new() -> Self {
        SummationLeaf {
            rsa_pk: Vec::new(),
            c0: None,
            c1: None,
            r: None,
        }
    }
    pub fn from_ct(rsa_pk: Vec<u8>, cts: Vec<i128>, r: [u8; 16]) -> Self {
        let mut c0: Vec<i128> = Vec::with_capacity(cts.len() / 2);
        c0.extend(&cts[0..cts.len() / 2]);
        let mut c1: Vec<i128> = Vec::with_capacity(cts.len() / 2);
        c1.extend(&cts[cts.len() / 2..]);
        SummationLeaf {
            rsa_pk,
            c0: Some(c0),
            c1: Some(c1),
            r: Some(r),
        }
    }
    #[cfg(not(feature = "hashfn_blake3"))]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        hasher.input(&self.rsa_pk);
        if let Some(c0) = self.c0.as_ref() {
            hasher.input(&i128vec_to_le_bytes(c0));
        }
        if let Some(c1) = self.c1.as_ref() {
            hasher.input(&i128vec_to_le_bytes(c1));
        }
        if let Some(r) = self.r.as_ref() {
            hasher.input(r);
        }

        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
    #[cfg(feature = "hashfn_blake3")]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.rsa_pk);
        if let Some(c0) = self.c0.as_ref() {
            hasher.update(&i128vec_to_le_bytes(c0));
        }
        if let Some(c1) = self.c1.as_ref() {
            hasher.update(&i128vec_to_le_bytes(c1));
        }
        if let Some(r) = self.r.as_ref() {
            hasher.update(r);
        }
        hasher.finalize().into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SummationNonLeaf {
    pub c0: Vec<i128>,
    pub c1: Vec<i128>,
}
impl SummationNonLeaf {
    //pub fn new() -> Self {
    //    let c0 = vec![0i128; 4096];
    //    let c1 = vec![0i128; 4096];
    //    SummationNonLeaf { c0, c1 }
    //}
    #[cfg(not(feature = "hashfn_blake3"))]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        let c0_bytes = i128vec_to_le_bytes(&self.c0);
        let c1_bytes = i128vec_to_le_bytes(&self.c1);

        hasher.input(&c0_bytes);
        hasher.input(&c1_bytes);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
    #[cfg(feature = "hashfn_blake3")]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        let c0_bytes = i128vec_to_le_bytes(&self.c0);
        let c1_bytes = i128vec_to_le_bytes(&self.c1);
        hasher.update(&c0_bytes);
        hasher.update(&c1_bytes);
        hasher.finalize().into()
    }
}

fn add_two_vec(a: Option<&Vec<i128>>, b: Option<&Vec<i128>>) -> Vec<i128> {
    // TODO need modulus here maybe
    let tmp = Vec::<i128>::new();
    let left = a.unwrap_or(&tmp);
    let right = b.unwrap_or(&tmp);
    let (longer, shorter) = if left.len() > right.len() {
        (left, right)
    } else {
        (right, left)
    };
    shorter
        .par_iter()
        .chain(repeatn(&0i128, longer.len() - shorter.len()))
        .zip(longer.par_iter())
        .map(|(x, y)| x + y)
        .collect()
    //shorter
    //    .iter()
    //    //.chain(repeatn(&0i128, longer.len() - shorter.len()))
    //    .zip(longer.iter())
    //    .map(|(x, y)| x + y)
    //    .collect()
}

impl Add for SummationNonLeaf {
    type Output = SummationNonLeaf;
    fn add(self, other: Self) -> Self {
        SummationNonLeaf {
            c0: add_two_vec(Some(&self.c0), Some(&other.c0)),
            c1: add_two_vec(Some(&self.c1), Some(&other.c1)),
        }
    }
}

impl<'a, 'b> Add<&'b SummationNonLeaf> for &'a SummationNonLeaf {
    type Output = SummationNonLeaf;

    fn add(self, other: &'b SummationNonLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: add_two_vec(Some(&self.c0), Some(&other.c0)),
            c1: add_two_vec(Some(&self.c1), Some(&other.c1)),
        }
    }
}

impl<'a, 'b> Add<&'b SummationLeaf> for &'a SummationNonLeaf {
    type Output = SummationNonLeaf;

    fn add(self, other: &'b SummationLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: add_two_vec(Some(&self.c0), other.c0.as_ref()),
            c1: add_two_vec(Some(&self.c1), other.c1.as_ref()),
        }
    }
}

impl<'a, 'b> Add<&'b SummationNonLeaf> for &'a SummationLeaf {
    type Output = SummationNonLeaf;

    fn add(self, other: &'b SummationNonLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: add_two_vec(self.c0.as_ref(), Some(&other.c0)),
            c1: add_two_vec(self.c1.as_ref(), Some(&other.c1)),
        }
    }
}

impl<'a, 'b> Add<&'b SummationLeaf> for &'a SummationLeaf {
    type Output = SummationNonLeaf;

    fn add(self, other: &'b SummationLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: add_two_vec(self.c0.as_ref(), other.c0.as_ref()),
            c1: add_two_vec(self.c0.as_ref(), other.c1.as_ref()),
        }
    }
}

impl PartialEq for SummationNonLeaf {
    fn eq(&self, other: &Self) -> bool {
        if (self.c0.len() != other.c0.len()) || (self.c1.len() != other.c1.len()) {
            false
        } else {
            let a = self
                .c0
                .par_iter()
                .zip(other.c0.par_iter())
                .map(|(x, y)| x == y)
                .reduce(|| true, |x, y| x && y);
            let b = self
                .c1
                .par_iter()
                .zip(other.c1.par_iter())
                .map(|(x, y)| x == y)
                .reduce(|| true, |x, y| x && y);
            a && b
        }
    }
}

impl From<SummationLeaf> for SummationNonLeaf {
    fn from(leaf: SummationLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: leaf.c0.unwrap_or(Vec::new()),
            c1: leaf.c1.unwrap_or(Vec::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SummationEntry {
    Leaf(SummationLeaf),
    NonLeaf(SummationNonLeaf),
    Commit(CommitEntry),
}

impl SummationEntry {
    pub fn new_leaf() -> Self {
        SummationEntry::Leaf(SummationLeaf::new())
    }

    pub fn get_leaf_rsa_pk(&self) -> &Vec<u8> {
        if let SummationEntry::Leaf(l) = self {
            &l.rsa_pk
        } else {
            panic!("get_leaf_rsa_pk@SummationEntry: not a leaf");
        }
    }
}
