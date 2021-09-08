use crypto::digest::Digest;
use crypto::sha3::Sha3;
use std::convert::{From, TryInto};
use std::ops::Add;
use tarpc::serde::{Deserialize, Serialize};

pub mod merkle;
use crate::common::aggregation::merkle::MerkleHash;

use super::i128vec_to_le_bytes;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitEntry {
    pub rsa_pk: Vec<u8>,
    pub hash: [u8; 32],
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
    pub fn hash(&self) -> MerkleHash {
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
}

impl Add for SummationNonLeaf {
    type Output = SummationNonLeaf;
    fn add(self, other: Self) -> Self {
        // TODO need modulus here maybe
        let len = std::cmp::max(self.c0.len(), other.c0.len());
        let mut new_c0: Vec<i128> = Vec::with_capacity(len);
        let mut new_c1: Vec<i128> = Vec::with_capacity(len);
        if self.c0.len() == other.c0.len() {
            for i in 0..len {
                new_c0.push(self.c0[i] + other.c0[i]);
                new_c1.push(self.c1[i] + other.c1[i]);
            }
        } else {
            for i in 0..len {
                let (a, b) = if i > self.c0.len() {
                    (0i128, 0i128)
                } else {
                    (self.c0[i], self.c1[i])
                };
                let (c, d) = if i > other.c0.len() {
                    (0i128, 0i128)
                } else {
                    (other.c0[i], other.c1[i])
                };
                new_c0.push(a + c);
                new_c1.push(b + d);
            }
        }
        SummationNonLeaf {
            c0: new_c0,
            c1: new_c1,
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
