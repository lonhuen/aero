use ark_groth16::verifier;
use rayon::ThreadPool;
use std::process::Command;

use crate::common::aggregation::{
    merkle::*,
    node::{CommitEntry, SummationEntry, SummationLeaf},
    McTree, MsTree,
};
use cancellable_timer::{Canceller, Timer};
use quail::zksnark::{Prover, Verifier};
use rand::{Rng, SeedableRng};
use std::{
    convert::Into,
    sync::{Arc, Condvar, Mutex, MutexGuard, RwLock},
    time::Duration,
};
use tracing::{error, event, instrument, span, warn, Level};

use crate::util::{config::ConfigUtils, log::init_tracing};

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
    mc: Arc<RwLock<McTree>>,
    cond: Arc<(Mutex<STATE>, Condvar)>,
    ms: Arc<RwLock<MsTree>>,
    nr_parameter: u32,
    pool: Arc<ThreadPool>,
    //pvk: Arc<Vec<u8>>,
    //verifier: Arc<Verifier>,
    canceller: Arc<RwLock<Canceller>>,
}

impl Server {
    pub fn setup(
        nr_real: u32,
        nr_sim: u32,
        nr_sybil: u32,
        nr_parameter: u32,
        pool: &Arc<ThreadPool>,
    ) -> Self {
        let mc = McTree::new(nr_real + nr_sim, nr_sybil);
        let ms = MsTree::new(nr_real + nr_sim, nr_sybil);

        //// get rid of prover
        //let (pvk, verifier) = {
        //    let prover = Prover::setup("./data/encryption.txt");
        //    let pvk = prover.serialize_pvk();
        //    let verifier = Verifier::new(&prover);
        //    (pvk, verifier)
        //};
        //let prover_ref = Arc::new(pvk);
        //let verifier_ref = Arc::new(verifier);

        let mc_ref = Arc::new(RwLock::new(mc));
        let ms_ref = Arc::new(RwLock::new(ms));

        let cond = Arc::new((Mutex::new((STAGE::Commit, 0)), Condvar::new()));
        //let timer_cond = cond.clone();

        let canceller = Timer::after(Duration::from_secs(WAITTIME), move |_| {}).unwrap();
        // TODO start random bit generation
        //warn!("Atom: Asking committee to generate random bits");
        //Command::new("bash")
        //    .arg("/home/ubuntu/quail/test.sh")
        //    .arg("offline")
        //    .output()
        //    .expect("failed to execute process");

        Self {
            mc: mc_ref,
            ms: ms_ref,
            cond: cond,
            nr_parameter,
            // pvk: pvk.clone(),
            //verifier: verifier.clone(),
            pool: pool.clone(),
            canceller: Arc::new(RwLock::new(canceller)),
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
    pub fn aggregate_commit(&self, round: u32, rsa_pk: Vec<u8>, commitment: [u8; 32]) {
        // wait for enough commitments
        // TODO to fix: should check if duplicate commitments come
        // TODO maybe wait for some time rather than some # of commitments

        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        match *state {
            (STAGE::Commit, round) => {}
            _ => return,
        }

        let mut mc = self.mc.as_ref().write().unwrap();
        let flag = {
            mc.insert_node(CommitEntry {
                rsa_pk,
                hash: commitment,
            });
            mc.gen_tree()
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
        cts: Vec<i128>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        // if never possible to get the lock, return
        match *state {
            (STAGE::Data, round) => {}
            _ => return,
        }
        let mut ms = self.ms.as_ref().write().unwrap();

        let flag = {
            ms.insert_node(SummationLeaf::from_ct(rsa_pk, cts, nonce));
            ms.gen_tree()
        };
        // TODO also verify the proof
        if flag {
            //let _ = self.canceller.as_ref().read().unwrap().cancel();
            *state = (STAGE::Verify, state.1);
            warn!("Server move to stage {:?}", *state);
            cvar.notify_all();
            let cond = self.cond.clone();
            let mc = self.ms.clone();
            let ms = self.mc.clone();
            let canceller = Timer::after(Duration::from_secs(10), move |_| {
                let (lock, cvar) = &*cond.clone();
                let mut state = lock.lock().unwrap();
                if let STAGE::Verify = state.0 {
                    // // TODO update the global model
                    // warn!("Atom: Asking committee to decrypt");
                    // Command::new("bash")
                    //     .arg("/home/ubuntu/quail/test.sh")
                    //     .arg("online")
                    //     .output()
                    //     .expect("failed to execute process");
                    // // TODO start random bit generation
                    // warn!("Atom: Asking committee to generate random bits");
                    // Command::new("bash")
                    //     .arg("/home/ubuntu/quail/test.sh")
                    //     .arg("offline")
                    //     .output()
                    //     .expect("failed to execute process");
                    *state = (STAGE::Commit, state.1 + 1);
                    //println!("Server move to stage {:?}", *state);
                    ms.write().unwrap().clear();
                    mc.write().unwrap().clear();
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
    pub fn get_mc_proof(&self, round: u32, rsa_pk: Vec<u8>) -> MerkleProof {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        // if never possible to get the lock, return
        if !Self::is_waitable(&*state, (STAGE::Data, round)) {
            return MerkleProof {
                lemma: Vec::new(),
                path: Vec::new(),
            };
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
        mc.get_proof(&rsa_pk)
    }

    //type GetMsProofFut = Ready<MerkleProof>;
    pub fn get_ms_proof(&self, round: u32, rsa_pk: Vec<u8>) -> MerkleProof {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        // if never possible to get the lock, return
        if !Self::is_waitable(&*state, (STAGE::Verify, round)) {
            return MerkleProof {
                lemma: Vec::new(),
                path: Vec::new(),
            };
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
        ms.get_proof(&rsa_pk)
    }

    //type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    pub fn verify(
        &self,
        round: u32,
        vinit: u32,
        non_leaf_id: Vec<u32>,
    ) -> Vec<(SummationEntry, MerkleProof)> {
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
        let mut ret: Vec<(SummationEntry, MerkleProof)> = Vec::new();
        for i in 0..5 + 1 {
            let node = ms.get_leaf_node(i + vinit);
            if let SummationEntry::Leaf(_) = node {
                let mc_proof: MerkleProof = mc.get_proof_by_id(i + vinit).into();
                let ms_proof: MerkleProof = ms.get_proof_by_id(i + vinit).into();
                ret.push((SummationEntry::Commit(mc.get_node(i + vinit)), mc_proof));
                ret.push((node, ms_proof));
            } else {
                warn!("Atom: verify not a leaf node");
            }
        }
        for i in non_leaf_id {
            let ms_proof: MerkleProof = ms.get_proof_by_id(i).into();
            ret.push((ms.get_nonleaf_node(i), ms_proof));
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
        (0..self.nr_parameter).map(|_| rng.gen::<u8>()).collect()
    }

    //type RetrieveProvingKeyFut = Ready<Vec<u8>>;
    pub fn retrieve_proving_key(&self, round: u32) -> Vec<u8> {
        //future::ready(self.pvk.as_ref().clone())
        vec![0u8; 1]
    }
}
