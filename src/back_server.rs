use ark_groth16::verifier;
use rayon::ThreadPool;

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
    sync::{Arc, Condvar, Mutex, RwLock},
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

    #[instrument(level = "warn", skip_all)]
    pub fn next_stage_to(&self, target: STAGE) {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        match (state.0, target) {
            (STAGE::Commit, STAGE::Data) => {
                //let _ = self.canceller.as_ref().read().unwrap().cancel();
                //self.next_stage_to(STAGE::Data);
                //let cond = self.cond.clone();
                //let canceller = Timer::after(Duration::from_secs(WAITTIME), move |_| {
                //    let (lock, cvar) = &*cond.clone();
                //    let mut state = lock.lock().unwrap();
                //    if let STATE::Data(x) = *state {
                //        *state = STATE::Verify(x);
                //        cvar.notify_all();
                //    }
                //})
                //.unwrap();
                *state = (STAGE::Data, state.1);
            }
            (STAGE::Data, STAGE::Verify) => *state = (STAGE::Verify, state.1),
            (STAGE::Verify, STAGE::Commit) => *state = (STAGE::Commit, state.1 + 1),
            _ => {}
        };
        warn!("Server move to stage {:?}", *state);
        cvar.notify_all();
    }

    //#[instrument(level = "warn", skip_all)]
    //pub fn next_stage_after_timeout(&self) {
    //    let (lock, cvar) = &*self.cond;
    //    let mut state = lock.lock().unwrap();
    //    match *state {
    //        STATE::Commit(x) => *state = STATE::Data(x),
    //        STATE::Data(x) => *state = STATE::Verify(x),
    //        STATE::Verify(x) => *state = STATE::Commit(x + 1),
    //    };
    //    warn!("Server move to stage {:?}", *state);
    //    cvar.notify_all();
    //}
    /// wait till target stage; if exceed, return false
    #[instrument(level = "warn", skip_all)]
    pub fn wait_till(&self, target: STAGE, round: u32) -> bool {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        if round != state.1 && state.1 + 1 != round {
            false
        } else {
            loop {
                match (state.0, target) {
                    (STAGE::Commit, STAGE::Commit) => return true,
                    (STAGE::Data, STAGE::Data) => return true,
                    (STAGE::Verify, STAGE::Verify) => return true,
                    _ => {
                        warn!("in {:?}{:?} wait {:?}{:?}", state.0, state.1, target, round);
                        state = cvar.wait(state).unwrap();
                    }
                }
            }
        }
    }

    #[instrument(level = "warn", skip_all)]
    pub fn aggregate_commit(&self, round: u32, rsa_pk: Vec<u8>, commitment: [u8; 32]) {
        // wait for enough commitments
        // TODO to fix: should check if duplicate commitments come
        // TODO maybe wait for some time rather than some # of commitments
        // TODO fix this
        let ret = self.wait_till(STAGE::Commit, round);
        // data from previous round
        if !ret {
            return;
        }
        let flag = {
            let mut mc = self.mc.as_ref().write().unwrap();
            mc.insert_node(CommitEntry {
                rsa_pk,
                hash: commitment,
            });
            mc.gen_tree()
        };
        // if we've got enough elements, move to next stage
        if flag {
            //let _ = self.canceller.as_ref().read().unwrap().cancel();
            self.next_stage_to(STAGE::Data);
            //let cond = self.cond.clone();
            //let canceller = Timer::after(Duration::from_secs(WAITTIME), move |_| {
            //    let (lock, cvar) = &*cond.clone();
            //    let mut state = lock.lock().unwrap();
            //    if let STAGE::Data = state.0 {
            //        *state = (STAGE::Verify, state.1);
            //        cvar.notify_all();
            //    }
            //})
            //.unwrap();
            //*self.canceller.as_ref().write().unwrap() = canceller;
        }
    }

    #[instrument(level = "warn", skip_all)]
    pub fn aggregate_data(
        &self,
        round: u32,
        rsa_pk: Vec<u8>,
        cts: Vec<i128>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) {
        let ret = self.wait_till(STAGE::Data, round);
        if !ret {
            return;
        }
        // TODO also verify the proof
        let flag = {
            let mut ms = self.ms.as_ref().write().unwrap();
            ms.insert_node(SummationLeaf::from_ct(rsa_pk, cts, nonce));

            ms.gen_tree()
        };
        if flag {
            //let _ = self.canceller.as_ref().read().unwrap().cancel();
            //self.next_stage_to(STAGE::Verify);
            self.next_stage_to(STAGE::Verify);
            self.ms.write().unwrap().clear();
            self.mc.write().unwrap().clear();
            self.next_stage_to(STAGE::Commit);
            //let cond = self.cond.clone();
            //let mc = self.ms.clone();
            //let ms = self.mc.clone();
            //let canceller = Timer::after(Duration::from_secs(10), move |_| {
            //    let (lock, cvar) = &*cond.clone();
            //    let mut state = lock.lock().unwrap();
            //    if let STAGE::Verify = state.0 {
            //        ms.write().unwrap().clear();
            //        mc.write().unwrap().clear();
            //        *state = (STAGE::Commit, state.1 + 1);
            //        //println!("Server move to stage {:?}", *state);
            //        warn!("Server move to stage {:?}", *state);
            //        cvar.notify_all();
            //    }
            //})
            //.unwrap();
            //*self.canceller.as_ref().write().unwrap() = canceller;
        }
    }

    //type GetMcProofFut = Ready<MerkleProof>;
    pub fn get_mc_proof(&self, round: u32, rsa_pk: Vec<u8>) -> MerkleProof {
        // TODO fix with condition variable
        let ret = self.wait_till(STAGE::Data, round);
        if !ret {
            return MerkleProof {
                lemma: Vec::new(),
                path: Vec::new(),
            };
        }
        let mc = self.mc.as_ref().read().unwrap();
        mc.get_proof(&rsa_pk)
    }

    //type GetMsProofFut = Ready<MerkleProof>;
    pub fn get_ms_proof(&self, round: u32, rsa_pk: Vec<u8>) -> MerkleProof {
        //let ret = self.wait_till(STAGE::Verify, round);
        //if !ret {
        //    return MerkleProof {
        //        lemma: Vec::new(),
        //        path: Vec::new(),
        //    };
        //}
        //let ms = self.ms.as_ref().read().unwrap();
        //ms.get_proof(&rsa_pk)
        MerkleProof {
            lemma: Vec::new(),
            path: Vec::new(),
        }
    }

    //type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    pub fn verify(
        &self,
        round: u32,
        vinit: u32,
        non_leaf_id: Vec<u32>,
    ) -> Vec<(SummationEntry, MerkleProof)> {
        let ret = self.wait_till(STAGE::Verify, round);
        if !ret {
            return Vec::new();
        }
        // TODO the client should call get_ms_proof before verify. Fix this for SGD
        //first all the leafs
        let mut ret: Vec<(SummationEntry, MerkleProof)> = Vec::new();
        let ms = self.ms.as_ref().read().unwrap();
        let mc = self.mc.as_ref().read().unwrap();
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
        let mut rng = rand::rngs::StdRng::from_entropy();
        (0..self.nr_parameter).map(|_| rng.gen::<u8>()).collect()
    }

    //type RetrieveProvingKeyFut = Ready<Vec<u8>>;
    pub fn retrieve_proving_key(&self, round: u32) -> Vec<u8> {
        //future::ready(self.pvk.as_ref().clone())
        vec![0u8; 1]
    }
}
