pub mod merkle;
use log::{info, warn};
use merkle::MerkleTree;
pub mod node;
use self::merkle::MerkleProof;
use log::error;
use node::{CommitEntry, SummationEntry, SummationLeaf, SummationNonLeaf};
use std::iter::FromIterator;
use std::process::exit;

pub struct McTree {
    pub nr_real: u32,
    pub nr_sybil: u32,
    pub commit_array: Vec<CommitEntry>,
    pub mc: Option<MerkleTree>,
}

impl McTree {
    pub fn new(nr_real: u32, nr_sybil: u32) -> Self {
        McTree {
            nr_real,
            nr_sybil,
            commit_array: Vec::with_capacity(nr_real as usize),
            mc: None,
        }
    }

    pub fn get_node(&self, id: u32) -> CommitEntry {
        if id >= self.nr_real {
            info!("Atom: Mc get node more than nr_real");
            self.commit_array[(id % self.nr_real) as usize].clone()
        } else {
            self.commit_array[id as usize].clone()
        }
    }

    pub fn gen_tree(&mut self) -> bool {
        if self.commit_array.len() < self.nr_real as usize {
            false
        } else {
            self.commit_array
                .sort_by(|a, b| a.rsa_pk.partial_cmp(&b.rsa_pk).unwrap());
            self.mc = Some(MerkleTree::from_iter(
                self.commit_array
                    .iter()
                    .map(|x| x.hash())
                    .chain((0..self.nr_sybil).into_iter().map(|_| [0u8; 32])),
            ));
            true
        }
    }

    pub fn get_proof(&self, rsa_pk: &Vec<u8>) -> MerkleProof {
        if self.mc.is_none() {
            warn!("get_proof@McTree called while None Mc tree");
        }
        let id = self
            .commit_array
            .binary_search_by(|probe| probe.rsa_pk.cmp(rsa_pk))
            .unwrap();
        self.mc.as_ref().unwrap().gen_proof(id).into()
    }

    pub fn get_proof_by_id(&self, id: u32) -> MerkleProof {
        self.mc.as_ref().unwrap().gen_proof(id as usize).into()
    }

    pub fn insert_node(&mut self, node: CommitEntry) {
        if self.commit_array.len() >= self.nr_real as usize {
            error!("insert more then expected");
        }
        self.commit_array.push(node);
    }

    pub fn clear(&mut self) {
        self.commit_array.clear();
        self.mc = None;
    }
}

pub struct MsTree {
    pub nr_real: u32,
    pub nr_sybil: u32,
    pub summation_array: Vec<SummationEntry>,
    pub ms: Option<MerkleTree>,
}
impl MsTree {
    pub fn new(nr_real: u32, nr_sybil: u32) -> Self {
        MsTree {
            nr_real,
            nr_sybil,
            summation_array: Vec::with_capacity(nr_real as usize),
            ms: None,
        }
    }

    pub fn get_node(&self, id: u32) -> SummationEntry {
        if id >= self.nr_real {
            info!("Atom: Ms get node more than nr_real");
            self.summation_array[(id % self.nr_real) as usize].clone()
        } else {
            self.summation_array[id as usize].clone()
        }
    }

    pub fn gen_tree(&mut self) -> bool {
        if self.summation_array.len() < self.nr_real as usize {
            false
        } else {
            // first from the leafs to the tree first
            // TODO check the rsa_pk appears in Mc
            self.summation_array.sort_by(|a, b| {
                a.get_leaf_rsa_pk()
                    .partial_cmp(b.get_leaf_rsa_pk())
                    .unwrap()
            });
            // get the non-leaf nodes
            let mut left = 0;
            let mut right = self.summation_array.len();
            while left + 1 < right {
                let a = match self.summation_array[left].clone() {
                    SummationEntry::NonLeaf(y) => y,
                    SummationEntry::Leaf(x) => x.into(),
                    _ => {
                        error!("gen_tree: Not a leaf or nonleaf node");
                        exit(1);
                    }
                };
                let b = match self.summation_array[left + 1].clone() {
                    SummationEntry::NonLeaf(y) => y,
                    SummationEntry::Leaf(x) => x.into(),
                    _ => {
                        error!("gen_tree: Not a leaf or nonleaf node");
                        // just to make the compiler happy
                        exit(1);
                    }
                };
                let c = a + b;
                self.summation_array.push(SummationEntry::NonLeaf(c));
                left += 2;
                right += 1;
            }
            self.ms = Some(MerkleTree::from_iter(
                self.summation_array
                    .iter()
                    .map(|x| match x {
                        SummationEntry::Leaf(y) => y.hash(),
                        SummationEntry::NonLeaf(y) => y.hash(),
                        // just to make compiler happy
                        // never reach here
                        _ => {
                            error!("commitment in summation array");
                            [0u8; 32]
                        }
                    })
                    // TODO maybe more precise about the # of sybils here
                    .chain((0..(2 * self.nr_sybil)).into_iter().map(|_| [0u8; 32])),
            ));
            true
        }
    }

    pub fn get_proof(&self, rsa_pk: &Vec<u8>) -> MerkleProof {
        if self.ms.is_none() {
            warn!("get_proof@McTree called while None Mc tree");
        }
        let id = self.summation_array[0..self.nr_real as usize]
            .binary_search_by(|probe| probe.get_leaf_rsa_pk().cmp(rsa_pk))
            .unwrap();
        self.ms.as_ref().unwrap().gen_proof(id).into()
    }

    pub fn get_proof_by_id(&self, id: u32) -> MerkleProof {
        self.ms.as_ref().unwrap().gen_proof(id as usize).into()
    }

    pub fn insert_node(&mut self, node: SummationLeaf) {
        if self.summation_array.len() >= self.nr_real as usize {
            error!("insert more then expected");
        }
        self.summation_array.push(SummationEntry::Leaf(node));
    }

    pub fn clear(&mut self) {
        self.summation_array.clear();
        self.ms = None;
    }
}
