#[cfg(feature = "hashfn_blake3")]
extern crate blake3;
#[cfg(not(feature = "hashfn_blake3"))]
use crypto::{digest::Digest, sha3::Sha3, sha3::Sha3Mode};
use tarpc::serde::{Deserialize, Serialize};

//pub type MerkleHash = [u8; 32];
pub type HashAlgorithm = hash_fn::HashAlgorithm;
pub type MerkleTree = merkle_light::merkle::MerkleTree<[u8; 32], HashAlgorithm>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleProof {
    pub lemma: Vec<[u8; 32]>,
    pub path: Vec<bool>,
}

impl MerkleProof {
    pub fn to_proof(self) -> merkle_light::proof::Proof<[u8; 32]> {
        merkle_light::proof::Proof::<[u8; 32]>::from(self)
    }
}
impl From<merkle_light::proof::Proof<[u8; 32]>> for MerkleProof {
    fn from(proof: merkle_light::proof::Proof<[u8; 32]>) -> Self {
        Self {
            lemma: proof.lemma,
            path: proof.path,
        }
    }
}
impl From<MerkleProof> for merkle_light::proof::Proof<[u8; 32]> {
    fn from(proof: MerkleProof) -> Self {
        Self {
            lemma: proof.lemma,
            path: proof.path,
        }
    }
}
#[cfg(feature = "hashfn_blake3")]
mod hash_fn {
    use merkle_light::hash::Algorithm;
    use std::hash::Hasher;
    pub struct HashAlgorithm(blake3::Hasher);
    impl HashAlgorithm {
        pub fn new() -> HashAlgorithm {
            HashAlgorithm(blake3::Hasher::new())
        }
    }

    impl Default for HashAlgorithm {
        fn default() -> HashAlgorithm {
            HashAlgorithm::new()
        }
    }

    impl Hasher for HashAlgorithm {
        #[inline]
        fn write(&mut self, msg: &[u8]) {
            self.0.update(msg);
        }

        #[inline]
        fn finish(&self) -> u64 {
            unimplemented!()
        }
    }

    impl Algorithm<[u8; 32]> for HashAlgorithm {
        #[inline]
        fn hash(&mut self) -> [u8; 32] {
            self.0.finalize().into()
        }

        #[inline]
        fn reset(&mut self) {
            self.0.reset();
        }
    }
}
#[cfg(not(feature = "hashfn_blake3"))]
mod hash_fn {
    use merkle_light::hash::Algorithm;
    use std::hash::Hasher;
    pub struct HashAlgorithm(Sha3);
    impl HashAlgorithm {
        pub fn new() -> HashAlgorithm {
            HashAlgorithm(Sha3::new(Sha3Mode::Sha3_256))
        }
    }

    impl Default for HashAlgorithm {
        fn default() -> HashAlgorithm {
            HashAlgorithm::new()
        }
    }

    impl Hasher for HashAlgorithm {
        #[inline]
        fn write(&mut self, msg: &[u8]) {
            self.0.input(msg)
        }

        #[inline]
        fn finish(&self) -> u64 {
            unimplemented!()
        }
    }

    impl Algorithm<[u8; 32]> for HashAlgorithm {
        #[inline]
        fn hash(&mut self) -> [u8; 32] {
            let mut h = [0u8; 32];
            self.0.result(&mut h);
            h
        }

        #[inline]
        fn reset(&mut self) {
            self.0.reset();
        }
    }
}
