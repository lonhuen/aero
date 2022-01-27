use ark_groth16::verifier;
use rayon::ThreadPool;
use std::{borrow::BorrowMut, process::Command};

use crate::common::aggregation::{
    merkle::*,
    node::{CommitEntry, SummationEntry, SummationLeaf},
    McTree, MsTree,
};
use cancellable_timer::{Canceller, Timer};
use quail::{
    rlwe::NUM_DIMENSION,
    zksnark::{Prover, Verifier},
};
use rand::{Rng, SeedableRng};
use std::{
    convert::Into,
    sync::{Arc, Condvar, Mutex, MutexGuard, RwLock},
    time::Duration,
};
use tracing::{error, event, instrument, span, warn, Level};

use crate::util::{config::ConfigUtils, log::init_tracing};
use std::process::Child;

// 2 hours
const WAITTIME: u64 = 7200;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum STAGE {
    Commit,
    Data,
    Verify,
}
pub type STATE = (STAGE, u32);
#[derive(Clone)]
pub struct Server {
    mc: Arc<RwLock<Vec<McTree>>>,
    cond: Arc<(Mutex<STATE>, Condvar)>,
    ms: Arc<RwLock<Vec<MsTree>>>,
    nr_parameter: u32,
    pool: Arc<ThreadPool>,
    //pvk: Arc<Vec<u8>>,
    //verifier: Arc<Verifier>,
    canceller: Arc<RwLock<Canceller>>,
    child: Arc<Mutex<Child>>,
}

impl Server {
    pub fn setup(
        nr_real: u32,
        nr_sim: u32,
        nr_sybil: u32,
        nr_parameter: u32,
        pool: &Arc<ThreadPool>,
    ) -> Self {
        let nr_ct = (nr_parameter / 4096) as usize;
        let mc = (0..nr_ct)
            .into_iter()
            .map(|_| McTree::new(nr_real + nr_sim, nr_sybil))
            .collect();
        let ms = (0..nr_ct)
            .into_iter()
            .map(|_| MsTree::new(nr_real + nr_sim, nr_sybil))
            .collect();
        //let verifier = Verifier::new("./data/verifying_key");
        //let verifier_ref = Arc::new(verifier);

        let mc_ref = Arc::new(RwLock::new(mc));
        let ms_ref = Arc::new(RwLock::new(ms));

        let cond = Arc::new((Mutex::new((STAGE::Commit, 0)), Condvar::new()));
        //let timer_cond = cond.clone();

        let canceller = Timer::after(Duration::from_secs(WAITTIME), move |_| {}).unwrap();
        warn!("Atom: Asking committee to generate random bits");
        let child = Command::new("bash")
            .arg("/home/ubuntu/quail/test.sh")
            .arg("offline")
            .spawn()
            .unwrap();

        Self {
            mc: mc_ref,
            ms: ms_ref,
            cond: cond,
            nr_parameter,
            // pvk: pvk.clone(),
            //verifier: verifier.clone(),
            pool: pool.clone(),
            canceller: Arc::new(RwLock::new(canceller)),
            child: Arc::new(Mutex::new(child)),
        }
    }
    #[inline]
    pub fn is_waitable(current: &STATE, target: STATE) -> bool {
        // get model: in Verify/Commit wait for commit
        // aggregate commit: in Commit wait for Commit
        // ----enough commits or timeout----
        // get mc proof: in Commit/Data wait for data
        // aggregate data: in Data wait for data
        // ----enough data or timeout----
        // get ms proof: in Data/verify wait for verify
        // ----anounce the summed CT
        // verify: in Verify wait for verify
        match (current.0, target.0) {
            (STAGE::Verify, STAGE::Commit) => current.1 + 1 == target.1,
            (STAGE::Commit, STAGE::Commit) => current.1 == target.1,
            (STAGE::Commit, STAGE::Data) => current.1 == target.1,
            (STAGE::Data, STAGE::Data) => current.1 == target.1,
            (STAGE::Data, STAGE::Verify) => current.1 == target.1,
            (STAGE::Verify, STAGE::Verify) => current.1 == target.1,
            _ => true,
        }
    }

    #[instrument(skip_all)]
    pub fn aggregate_commit(&self, round: u32, rsa_pk: Vec<u8>, commitment: Vec<[u8; 32]>) {
        // wait for enough commitments
        // TODO should check if duplicate commitments exist
        // TODO maybe wait for some time rather than some # of commitments

        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        match *state {
            (STAGE::Commit, round) => {}
            _ => return,
        }

        let mut mc = self.mc.as_ref().write().unwrap();
        for i in 0..commitment.len() {
            mc[i].insert_node(CommitEntry {
                rsa_pk: rsa_pk.clone(),
                hash: commitment[i],
            });
        }
        // TODO ideally all gen_tree should return true; maybe check this for robutness
        let flag = {
            let mut flag = false;
            for i in 0..commitment.len() {
                flag = mc[i].gen_tree()
            }
            flag
        };
        // if we've got enough elements, move to next stage
        if flag {
            *state = (STAGE::Data, state.1);
            warn!("Server move to stage {:?}", *state);
            cvar.notify_all();
        }
        drop(mc);
        drop(state);
    }

    #[instrument(skip_all)]
    pub fn aggregate_data(
        &self,
        round: u32,
        rsa_pk: Vec<u8>,
        c0: Vec<Vec<i128>>,
        c1: Vec<Vec<i128>>,
        nonce: Vec<[u8; 16]>,
        proofs: Vec<Vec<u8>>,
    ) {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        // if never possible to get the lock, return
        match *state {
            (STAGE::Data, round) => {}
            _ => return,
        }
        let mut ms = self.ms.as_ref().write().unwrap();

        // TODO ideally all gen_tree should return true; maybe check this for robutness
        let mut flag = false;
        for i in 0..nonce.len() {
            flag = {
                ms[i].insert_node(SummationLeaf::from_ct(
                    rsa_pk.clone(),
                    c0[i].clone(),
                    c1[i].clone(),
                    nonce[i],
                    proofs[i].clone(),
                ));
                ms[i].gen_tree()
            };
        }

        // TODO also verify the proofs
        // Since verification cost is little and we don't focus on aggregator cost, we don't really check the proofs here
        if flag {
            //let _ = self.canceller.as_ref().read().unwrap().cancel();
            *state = (STAGE::Verify, state.1);
            warn!("Server move to stage {:?}", *state);
            cvar.notify_all();
            let cond = self.cond.clone();
            let mc = self.mc.clone();
            let ms = self.ms.clone();
            let child_ref = self.child.clone();
            let canceller = Timer::after(Duration::from_secs(20), move |_| {
                let (lock, cvar) = &*cond.clone();
                let mut state = lock.lock().unwrap();
                if let STAGE::Verify = state.0 {
                    // TODO update the global model
                    warn!("Atom: Asking committee to decrypt");
                    let mut child = child_ref.lock().unwrap();
                    child.wait().unwrap();
                    // decrypt
                    Command::new("bash")
                        .arg("/home/ubuntu/quail/test.sh")
                        .arg("online")
                        .output()
                        .expect("failed to execute process");
                    // start next random bit generation
                    warn!("Atom: Asking committee to generate random bits");
                    *child = Command::new("bash")
                        .arg("/home/ubuntu/quail/test.sh")
                        .arg("offline")
                        .spawn()
                        .unwrap();
                    *state = (STAGE::Commit, state.1 + 1);
                    //println!("Server move to stage {:?}", *state);
                    mc.write().unwrap().iter_mut().for_each(|t| t.clear());
                    ms.write().unwrap().iter_mut().for_each(|t| t.clear());
                    warn!("Server move to stage {:?}", *state);
                    cvar.notify_all();
                }
            })
            .unwrap();
            *self.canceller.as_ref().write().unwrap() = canceller;
        }
        drop(state);
    }

    //type GetMcProofFut = Ready<MerkleProof>;
    pub fn get_mc_proof(&self, round: u32, rsa_pk: Vec<u8>) -> Vec<MerkleProof> {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        // if never possible to get the lock, return
        if !Self::is_waitable(&*state, (STAGE::Data, round)) {
            return vec![MerkleProof {
                lemma: Vec::new(),
                path: Vec::new(),
            }];
        }
        // otherwise wait till the state
        state = cvar
            .wait_while(state, |state| match state.0 {
                STAGE::Data => false,
                _ => true,
            })
            .unwrap();
        let mc = self.mc.as_ref().read().unwrap();
        drop(state);
        //let ret = match state.0 {
        //    STAGE::Data => {
        //        let mc = self.mc.as_ref().read().unwrap();
        //        mc.iter().map(|x| x.get_proof(&rsa_pk)).collect()
        //    }
        //    _ => Vec::new(),
        //};
        //ret
        mc.iter().map(|x| x.get_proof(&rsa_pk)).collect()
    }

    //type GetMsProofFut = Ready<MerkleProof>;
    pub fn get_ms_proof(&self, round: u32, rsa_pk: Vec<u8>) -> Vec<MerkleProof> {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        //if never possible to get the lock, return
        if !Self::is_waitable(&*state, (STAGE::Verify, round)) {
            return vec![MerkleProof {
                lemma: Vec::new(),
                path: Vec::new(),
            }];
        }
        // otherwise wait till the state
        state = cvar
            .wait_while(state, |state| match state.0 {
                STAGE::Verify => false,
                _ => true,
            })
            .unwrap();
        let ms = self.ms.as_ref().read().unwrap();
        drop(state);
        ms.iter().map(|x| x.get_proof(&rsa_pk)).collect()
        //let ret = match state.0 {
        //    STAGE::Verify => {
        //        let ms = self.ms.as_ref().read().unwrap();
        //        drop(state);
        //        ms.iter().map(|x| x.get_proof(&rsa_pk)).collect()
        //    }
        //    _ => Vec::new(),
        //};
        //ret
    }

    //type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    pub fn verify(
        &self,
        round: u32,
        vinit: u32,
        non_leaf_id: Vec<u32>,
        ct_id: Vec<usize>,
    ) -> Vec<Vec<(SummationEntry, MerkleProof)>> {
        let (lock, _cvar) = &*self.cond;
        let state = lock.lock().unwrap();

        // if not verify phase, just return
        match *state {
            (STAGE::Verify, round) => {}
            _ => return Vec::new(),
        };
        let ms = self.ms.as_ref().read().unwrap();
        let mc = self.mc.as_ref().read().unwrap();
        drop(state);
        //first all the leafs
        let mut ret: Vec<Vec<(SummationEntry, MerkleProof)>> = Vec::new();
        for k in ct_id {
            let mut t: Vec<(SummationEntry, MerkleProof)> = Vec::new();
            if k >= mc.len() {
                warn!("K larger than Mc len");
                continue;
            }
            for i in 0..5 + 1 {
                let node: SummationEntry = ms[k].get_leaf_node(i as u32 + vinit);
                if let SummationEntry::Leaf(_) = node {
                    let mc_proof: MerkleProof = mc[k].get_proof_by_id(i + vinit).into();
                    let ms_proof: MerkleProof = ms[k].get_proof_by_id(i + vinit).into();
                    t.push((SummationEntry::Commit(mc[k].get_node(i + vinit)), mc_proof));
                    t.push((node, ms_proof));
                } else {
                    warn!("Atom: verify not a leaf node");
                }
            }
            for i in &non_leaf_id {
                let ms_proof: MerkleProof = ms[k].get_proof_by_id(*i).into();
                t.push((ms[k].get_nonleaf_node(*i), ms_proof));
            }
            ret.push(t);
        }
        ret
    }

    //type RetrieveModelFut = Ready<Vec<u8>>;
    pub fn retrieve_model(&self, round: u32) -> Vec<u8> {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        // if never possible to get the lock, return
        if !Self::is_waitable(&*state, (STAGE::Commit, round)) {
            return Vec::new();
        }
        // otherwise wait till the state
        let _state = cvar
            .wait_while(state, |state| match *state {
                (STAGE::Commit, round) => false,
                _ => true,
            })
            .unwrap();

        let mut rng = rand::rngs::StdRng::from_entropy();
        //(0..self.nr_parameter).map(|_| rng.gen::<u8>()).collect()
        (0..self.nr_parameter).map(|_| 0u8).collect()
    }

    //type RetrieveProvingKeyFut = Ready<Vec<u8>>;
    pub fn retrieve_proving_key(&self, round: u32) -> Vec<u8> {
        //future::ready(self.pvk.as_ref().clone())
        vec![0u8; 1]
    }
}
