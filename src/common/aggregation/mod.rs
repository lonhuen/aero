pub mod merkle;
use merkle::MerkleTree;
pub mod node;
use self::{merkle::MerkleProof, node::SummationNonLeaf};
use node::{CommitEntry, SummationEntry, SummationLeaf};
use rayon::prelude::*;
use std::iter::FromIterator;
use tracing::{error, instrument, warn};

use ark_std::{end_timer, start_timer};

use super::summation_array_size;

//TODO modify the # of real clients and also report back to the clients
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
            warn!(
                "Atom: Mc get node {} more than nr_real {}",
                id, self.nr_real
            );
            self.commit_array[(id % self.nr_real) as usize].clone()
        } else {
            self.commit_array[id as usize].clone()
        }
    }

    #[instrument(level = "warn", skip_all)]
    pub fn gen_tree(&mut self) -> bool {
        if self.commit_array.len() < self.nr_real as usize {
            warn!(
                "Gen commit tree: {} < {}",
                self.commit_array.len(),
                self.nr_real
            );
            false
        } else {
            self.commit_array
                .sort_by(|a, b| a.rsa_pk.partial_cmp(&b.rsa_pk).unwrap());
            self.mc = Some(MerkleTree::from_iter(
                self.commit_array
                    .par_iter()
                    .map(|x| x.hash())
                    .chain((0..self.nr_sybil).into_par_iter().map(|_| [0u8; 32]))
                    .collect::<Vec<[u8; 32]>>(),
            ));
            true
        }
    }

    #[instrument(level = "warn", skip_all)]
    pub fn gen_tree_timout(&mut self) -> usize {
        self.commit_array
            .sort_by(|a, b| a.rsa_pk.partial_cmp(&b.rsa_pk).unwrap());
        self.mc = Some(MerkleTree::from_iter(
            self.commit_array
                .par_iter()
                .map(|x| x.hash())
                .chain((0..self.nr_sybil).into_par_iter().map(|_| [0u8; 32]))
                .collect::<Vec<[u8; 32]>>(),
        ));
        self.commit_array.len()
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
            panic!("insert more then expected");
        }
        self.commit_array.push(node);
    }

    pub fn clear(&mut self) {
        self.commit_array.clear();
        self.mc = None;
    }
}
// since Vector is in heap, the sort won't move the actual data. just the pointer will be moved.
pub struct MsTree {
    pub nr_real: u32,
    pub nr_sybil: u32,
    pub nr_non_leaf: u32,
    pub summation_array: Vec<SummationEntry>,
    pub ms: Option<MerkleTree>,
}
impl MsTree {
    pub fn new(nr_real: u32, nr_sybil: u32) -> Self {
        let nr_non_leaf = summation_array_size(nr_real);
        MsTree {
            nr_real,
            nr_non_leaf,
            nr_sybil,
            summation_array: Vec::with_capacity(nr_real as usize),
            ms: None,
        }
    }

    pub fn get_leaf_node(&self, id: u32) -> SummationEntry {
        if id >= self.nr_real {
            warn!(
                "Atom: Ms get leaf node {} more than nr_real {}",
                id, self.nr_real
            );
            self.summation_array[(id % self.nr_real) as usize].clone()
        } else {
            self.summation_array[id as usize].clone()
        }
    }

    pub fn get_nonleaf_node(&self, id: u32) -> SummationEntry {
        if (id < self.nr_real) || (id >= self.nr_real + self.nr_non_leaf) {
            error!("Atom: Ms get leaf node more than nr_real+nr_non_leaf");
            self.summation_array[(id % (self.nr_real + self.nr_non_leaf)) as usize].clone()
        } else {
            self.summation_array[id as usize].clone()
        }
    }

    pub fn get_root(&self) -> &SummationNonLeaf {
        let ret = match self.summation_array.last().as_ref().unwrap() {
            SummationEntry::NonLeaf(x) => x,
            _ => {
                panic!("Root node is not a nonleaf node");
            }
        };
        ret
    }

    #[instrument(level = "warn", skip_all)]
    pub fn gen_tree(&mut self) -> bool {
        if self.summation_array.len() < self.nr_real as usize {
            warn!(
                "Gen summation tree: {} < {}",
                self.summation_array.len(),
                self.nr_real
            );
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
                let c = match (&self.summation_array[left], &self.summation_array[left + 1]) {
                    (SummationEntry::NonLeaf(left), SummationEntry::NonLeaf(right)) => left + right,
                    (SummationEntry::NonLeaf(left), SummationEntry::Leaf(right)) => left + right,
                    (SummationEntry::Leaf(left), SummationEntry::NonLeaf(right)) => right + left,
                    (SummationEntry::Leaf(left), SummationEntry::Leaf(right)) => left + right,
                    _ => {
                        panic!("gen_tree: Not a leaf or nonleaf node");
                    }
                };
                self.summation_array.push(SummationEntry::NonLeaf(c));
                left += 2;
                right += 1;
            }
            let gc = start_timer!(|| "gen tree of ms");

            self.ms = Some(MerkleTree::from_iter(
                self.summation_array
                    .par_iter()
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
                    .chain((0..(2 * self.nr_sybil)).into_par_iter().map(|_| [0u8; 32]))
                    .collect::<Vec<[u8; 32]>>(),
            ));
            end_timer!(gc);
            true
        }
    }

    #[instrument(level = "warn", skip_all)]
    pub fn gen_tree_timeout(&mut self) -> usize {
        // first from the leafs to the tree first
        let ret = self.summation_array.len();
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
            let c = match (&self.summation_array[left], &self.summation_array[left + 1]) {
                (SummationEntry::NonLeaf(left), SummationEntry::NonLeaf(right)) => left + right,
                (SummationEntry::NonLeaf(left), SummationEntry::Leaf(right)) => left + right,
                (SummationEntry::Leaf(left), SummationEntry::NonLeaf(right)) => right + left,
                (SummationEntry::Leaf(left), SummationEntry::Leaf(right)) => left + right,
                _ => {
                    panic!("gen_tree: Not a leaf or nonleaf node");
                }
            };
            self.summation_array.push(SummationEntry::NonLeaf(c));
            left += 2;
            right += 1;
        }
        let gc = start_timer!(|| "gen tree of ms");

        self.ms = Some(MerkleTree::from_iter(
            self.summation_array
                .par_iter()
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
                .chain((0..(2 * self.nr_sybil)).into_par_iter().map(|_| [0u8; 32]))
                .collect::<Vec<[u8; 32]>>(),
        ));
        end_timer!(gc);
        ret
    }
    pub fn get_proof(&self, rsa_pk: &Vec<u8>) -> MerkleProof {
        if self.ms.is_none() {
            warn!("get_proof@MsTree called while None Ms tree");
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
