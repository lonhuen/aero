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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum STATE {
    Commit(u32),
    Data(u32),
    Verify(u32),
}
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

        let cond = Arc::new((Mutex::new(STATE::Commit(0)), Condvar::new()));
        let timer_cond = Arc::new((Mutex::new(STATE::Commit(0)), Condvar::new()));
        let timer_mc = Arc::new((Mutex::new(STATE::Commit(0)), Condvar::new()));

        let canceller = Timer::after(Duration::from_secs(10), move |_| {
            let (lock, cvar) = &*timer_cond;
            let mut state = lock.lock().unwrap();
            if let STATE::Commit(x) = *state {
                *state = STATE::Data(x);
                cvar.notify_all();
            }
        })
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
        }
    }

    #[instrument(level = "warn", skip_all)]
    pub fn next_stage_to(&self, target: STATE) {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        match (*state, target) {
            (STATE::Commit(x), STATE::Data(_)) => *state = STATE::Data(x),
            (STATE::Data(x), STATE::Verify(_)) => *state = STATE::Verify(x),
            (STATE::Verify(x), STATE::Commit(_)) => *state = STATE::Commit(x + 1),
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
    #[instrument(level = "warn", skip_all)]
    pub fn wait_till(&self, target: STATE) {
        let (lock, cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        loop {
            match (*state, target) {
                (STATE::Commit(_), STATE::Commit(_)) => break,
                (STATE::Data(_), STATE::Data(_)) => break,
                (STATE::Verify(_), STATE::Verify(_)) => break,
                _ => {
                    state = cvar.wait(state).unwrap();
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
        self.wait_till(STATE::Commit(0));
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
            let _ = self.canceller.as_ref().read().unwrap().cancel();
            self.next_stage_to(STATE::Data(0));
            let cond = self.cond.clone();
            let canceller = Timer::after(Duration::from_secs(10), move |_| {
                let (lock, cvar) = &*cond.clone();
                let mut state = lock.lock().unwrap();
                if let STATE::Data(x) = *state {
                    *state = STATE::Verify(x);
                    cvar.notify_all();
                }
            })
            .unwrap();
            *self.canceller.as_ref().write().unwrap() = canceller;
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
        self.wait_till(STATE::Data(round));
        // TODO also verify the proof
        let flag = {
            let mut ms = self.ms.as_ref().write().unwrap();
            ms.insert_node(SummationLeaf::from_ct(rsa_pk, cts, nonce));

            ms.gen_tree()
        };
        if flag {
            let _ = self.canceller.as_ref().read().unwrap().cancel();
            self.next_stage_to(STATE::Verify(0));
            let cond = self.cond.clone();
            let mc = self.ms.clone();
            let ms = self.mc.clone();
            let canceller = Timer::after(Duration::from_secs(10), move |_| {
                let (lock, cvar) = &*cond.clone();
                let mut state = lock.lock().unwrap();
                if let STATE::Verify(x) = *state {
                    ms.write().unwrap().clear();
                    mc.write().unwrap().clear();
                    *state = STATE::Commit(x + 1);
                    cvar.notify_all();
                }
            })
            .unwrap();
            *self.canceller.as_ref().write().unwrap() = canceller;
        }
    }

    //type GetMcProofFut = Ready<MerkleProof>;
    // TODO for now assume only 1 round
    pub fn get_mc_proof(&self, round: u32, rsa_pk: Vec<u8>) -> MerkleProof {
        // TODO fix with condition variable
        let _ = self.wait_till(STATE::Data(0));
        let mc = self.mc.as_ref().read().unwrap();
        mc.get_proof(&rsa_pk)
    }

    //type GetMsProofFut = Ready<MerkleProof>;
    pub fn get_ms_proof(&self, round: u32, rsa_pk: Vec<u8>) -> MerkleProof {
        self.wait_till(STATE::Verify(0));
        let ms = self.ms.as_ref().read().unwrap();
        ms.get_proof(&rsa_pk)
    }

    //type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    pub fn verify(&self, vinit: u32, non_leaf_id: Vec<u32>) -> Vec<(SummationEntry, MerkleProof)> {
        self.wait_till(STATE::Verify(0));
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
