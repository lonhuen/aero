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
    pub c0: Vec<i128>,
    pub c1: Vec<i128>,
    pub r: [u8; 16],
    pub proof: Vec<u8>,
}

impl SummationLeaf {
    pub fn new() -> Self {
        SummationLeaf {
            rsa_pk: Vec::new(),
            c0: Vec::new(),
            c1: Vec::new(),
            r: [0u8; 16],
            proof: Vec::new(),
        }
    }

    pub fn from_ct(
        rsa_pk: Vec<u8>,
        c0: Vec<i128>,
        c1: Vec<i128>,
        r: [u8; 16],
        proof: Vec<u8>,
    ) -> Self {
        SummationLeaf {
            rsa_pk,
            c0,
            c1,
            r,
            proof: proof,
        }
    }
    #[cfg(not(feature = "hashfn_blake3"))]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();

        hasher.input(&self.rsa_pk);
        hasher.input(&i128vec_to_le_bytes(&self.c0));
        hasher.input(&i128vec_to_le_bytes(&self.c1));
        hasher.input(r);

        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }

    #[cfg(feature = "hashfn_blake3")]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.rsa_pk);
        hasher.update(&i128vec_to_le_bytes(&self.c0));
        hasher.update(&i128vec_to_le_bytes(&self.c1));
        hasher.update(&self.r);
        hasher.finalize().into()
    }

    pub fn evaluate_at(&self, r: i128) -> SummationNonLeaf {
        // TODO implement this
        assert_ne!(self.c0.len(), 0);
        SummationNonLeaf {
            c0: self.c0[0],
            c1: self.c1[0],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SummationNonLeaf {
    pub c0: i128,
    pub c1: i128,
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

        hasher.input(&self.c0.to_le_bytes());
        hasher.input(&self.c1.to_le_bytes());
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
    #[cfg(feature = "hashfn_blake3")]
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.c0.to_le_bytes());
        hasher.update(&self.c1.to_le_bytes());
        hasher.finalize().into()
    }
}

fn add_two_vec(a: &Vec<i128>, b: &Vec<i128>) -> Vec<i128> {
    // TODO need modulus here maybe
    let tmp = Vec::<i128>::new();
    let left = a;
    let right = b;
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
            c0: self.c0 + other.c0,
            c1: self.c1 + other.c1,
        }
    }
}

impl<'a, 'b> Add<&'b SummationNonLeaf> for &'a SummationNonLeaf {
    type Output = SummationNonLeaf;

    fn add(self, other: &'b SummationNonLeaf) -> SummationNonLeaf {
        SummationNonLeaf {
            c0: self.c0 + other.c0,
            c1: self.c1 + other.c1,
        }
    }
}

// impl<'a, 'b> Add<&'b SummationNonLeaf> for &'a SummationLeaf {
//     type Output = SummationNonLeaf;
//
//     fn add(self, other: &'b SummationNonLeaf) -> SummationNonLeaf {
//         SummationNonLeaf {
//             c0: add_two_vec(self.c0.as_ref(), Some(&other.c0)),
//             c1: add_two_vec(self.c1.as_ref(), Some(&other.c1)),
//         }
//     }
// }
//
// impl<'a, 'b> Add<&'b SummationLeaf> for &'a SummationLeaf {
//     type Output = SummationNonLeaf;
//
//     fn add(self, other: &'b SummationLeaf) -> SummationNonLeaf {
//         SummationNonLeaf {
//             c0: add_two_vec(self.c0.as_ref(), other.c0.as_ref()),
//             c1: add_two_vec(self.c0.as_ref(), other.c1.as_ref()),
//         }
//     }
// }
//
//
// impl From<SummationLeaf> for SummationNonLeaf {
//     fn from(leaf: SummationLeaf) -> SummationNonLeaf {
//         SummationNonLeaf {
//             c0: leaf.c0.unwrap_or(Vec::new()),
//             c1: leaf.c1.unwrap_or(Vec::new()),
//         }
//     }
// }

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
