use crypto::digest::Digest;
use crypto::sha3::Sha3;
use std::convert::{From, TryInto};
use std::ops::Add;
use tarpc::serde::{Deserialize, Serialize};

pub mod merkle;
use crate::common::aggregation::merkle::MerkleHash;

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
    pub fn from_ct(rsa_pk: Vec<u8>, cts: Vec<u8>, r: [u8; 16]) -> Self {
        let c0: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| i128::from_le_bytes(cts[i..i + 16].try_into().expect("uncorrected length")))
            .collect();
        let c1: Vec<i128> = (0..65536)
            .step_by(16)
            .map(|i| {
                i128::from_le_bytes(
                    cts[(i + 65536)..(i + 16 + 65536)]
                        .try_into()
                        .expect("uncorrected length"),
                )
            })
            .collect();
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
        if self.c0.is_some() {
            let c0_bytes: Vec<u8> = (0..4096)
                .into_iter()
                .flat_map(|i| i128::to_le_bytes(self.c0.as_ref().unwrap()[i]))
                .collect();
            let c1_bytes: Vec<u8> = (0..4096)
                .into_iter()
                .flat_map(|i| i128::to_le_bytes(self.c1.as_ref().unwrap()[i]))
                .collect();
            hasher.input(&c0_bytes);
            hasher.input(&c1_bytes);
            hasher.input(&self.r.unwrap());
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
    pub fn new() -> Self {
        let c0 = vec![0i128; 4096];
        let c1 = vec![0i128; 4096];
        SummationNonLeaf { c0, c1 }
    }
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        let c0_bytes: Vec<u8> = (0..4096)
            .into_iter()
            .flat_map(|i| i128::to_le_bytes(self.c0[i]))
            .collect();
        let c1_bytes: Vec<u8> = (0..4096)
            .into_iter()
            .flat_map(|i| i128::to_le_bytes(self.c1[i]))
            .collect();

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
        let new_c0: Vec<i128> = (0..4096)
            .into_iter()
            .map(|i| self.c0[i] + other.c0[i])
            .collect();
        let new_c1: Vec<i128> = (0..4096)
            .into_iter()
            .map(|i| self.c1[i] + other.c1[i])
            .collect();
        SummationNonLeaf {
            c0: new_c0,
            c1: new_c1,
        }
    }
}
impl From<SummationLeaf> for SummationNonLeaf {
    fn from(leaf: SummationLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: leaf.c0.unwrap_or(vec![0i128; 4096]),
            c1: leaf.c1.unwrap_or(vec![0i128; 4096]),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SummationEntry {
    Leaf(SummationLeaf),
    NonLeaf(SummationNonLeaf),
    Commit(CommitEntry),
}
