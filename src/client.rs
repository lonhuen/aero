mod common;
mod util;
mod zksnark;
use crate::common::aggregation::{
    merkle::HashAlgorithm,
    merkle::MerkleProof,
    node::{SummationEntry, SummationLeaf, SummationNonLeaf},
};
use crate::common::server_service::ServerServiceClient;
use crate::common::{i128vec_to_le_bytes, summation_array_size, ZKProof};
use crate::util::{config::ConfigUtils, log::LogUtils};
use crate::zksnark::{Prover, Verifier};
use ark_std::{end_timer, start_timer};
use crypto::digest::Digest;
use crypto::sha3::{Sha3, Sha3Mode};
use futures::Future;
use log::info;
use merkle_light::hash::Algorithm;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use std::collections::HashSet;
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::id;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::{Instant, SystemTime};
use std::{net::SocketAddr, time::Duration};
use std::{thread, time};
use tarpc::{client, context, tokio_serde::formats::Json};

const DEADLINE_TIME: u64 = 600;
const NUM_DIMENSION: u32 = 4096;
pub struct Client {
    inner: ServerServiceClient,
    //prover: Prover,
    rsa_pk: Vec<u8>,
    rsa_vk: RsaPrivateKey,
    c0s: Vec<i128>,
    c1s: Vec<i128>,
    nonce: [u8; 16],
}

impl Client {
    // TODO random this nounce
    pub fn new(inner: ServerServiceClient) -> Self {
        // first download the proving key from the server

        let bits = 2048;
        //let mut rng = rand::rngs::StdRng::seed_from_u64(Instant::now().);
        let mut rng = rand::rngs::StdRng::from_entropy();
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);
        Self {
            inner,
            //prover: prover,
            rsa_pk: public_key.to_public_key_pem().unwrap().into_bytes(),
            rsa_vk: private_key,
            c0s: Vec::new(),
            c1s: Vec::new(),
            nonce: [0u8; 16],
        }
    }

    // TODO somehow get the file
    // TODO for now 1 u8 for 1 CT (just to measure performance)
    // we may use seal and process_log.py to get the desired file
    // but it takes about 16s. Also the process_log.py redoes the encryption work + crt conversion.
    // To simulate, let's
    // 1. call SEAL encryption
    // 2. return the encryption.txt
    pub fn encrypt(&mut self, xs: Vec<u8>) {
        // call SEAL here to get a log file
        // "data/encryption.txt"
        thread::sleep(
            time::Duration::from_millis(87) * (xs.len() / (NUM_DIMENSION as usize)) as u32,
        );
        for _ in 0..xs.len() / NUM_DIMENSION as usize {
            self.c0s.extend(vec![1i128; NUM_DIMENSION as usize]);
            self.c1s.extend(vec![1i128; NUM_DIMENSION as usize]);
        }
        info!("Atom: encryption get c0s len {}", self.c0s.len());
    }
    // TODO need to fix this with setup phase
    // but for now, let's read the proof and sleep some time to simulate the create proof
    pub fn generate_proof(&self) -> Vec<Vec<u8>> {
        // TODO here we assume we have 10 threads to do this proof generation
        thread::sleep(time::Duration::from_millis(
            //(self.c0s.len() as f64 / NUM_DIMENSION as f64) as u64 * 825,
            (self.c0s.len() as f64 / NUM_DIMENSION as f64) as u64,
        ));
        info!(
            "Atom: generate proofs for {} CTs",
            self.c0s.len() / NUM_DIMENSION as usize
        );
        vec![vec![0u8; 192]; self.c0s.len() / NUM_DIMENSION as usize]
    }

    // the whole aggregation phase (except the encryption)
    pub async fn upload(&mut self, xs: Vec<u8>) -> bool {
        // set the deadline of the context
        let gc1 = start_timer!(|| "encrypt the gradients");
        self.encrypt(xs);
        // generate commitment to all the CTs
        let cm = self.hash();
        end_timer!(gc1);

        let gc2 = start_timer!(|| "upload the mc+proof generation");
        let result_commit = {
            // send this commitment to the server
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .aggregate_commit(ctx, self.rsa_pk.clone(), cm)
                .await;
            self.inner.get_mc_proof(ctx, self.rsa_pk.clone(), 0u32)
        };
        // while waiting for the commitment, compute the zkproof
        let proofs = self.generate_proof();

        // wait for the Mc tree
        let mc_proof = result_commit.await.unwrap().to_proof();
        end_timer!(gc2);

        let gc3 = start_timer!(|| "upload the data");
        // proceed to summation tree
        let mut cts_bytes: Vec<i128> = Vec::with_capacity(self.c0s.len() * 2);
        // TODO this might needs to be changed
        let mut proof_bytes: Vec<u8> = Vec::with_capacity(self.c0s.len() / NUM_DIMENSION as usize);

        cts_bytes.extend(&self.c0s);
        cts_bytes.extend(&self.c1s);
        for i in 0..self.c0s.len() / NUM_DIMENSION as usize {
            proof_bytes.extend(proofs[i].iter());
        }
        let result_data = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .aggregate_data(ctx, self.rsa_pk.clone(), cts_bytes, self.nonce, proof_bytes)
                .await;
            self.inner.get_ms_proof(ctx, self.rsa_pk.clone(), 0u32)
        };

        let ms_proof = result_data.await.unwrap().to_proof();
        end_timer!(gc3);
        // TODO verify the proof by checking x.leaf for mc, ms
        //{
        //    let mut hasher = Sha3::sha3_256();
        //    hasher.input(&self.rsa_pk);
        //    hasher.input(&cm);
        //    let mut h = [0u8; 32];
        //    hasher.result(&mut h);
        //    let mut x = HashAlgorithm::new();
        //    info!("hash {:?}", x.leaf(h));
        //}
        ms_proof.validate::<HashAlgorithm>() && mc_proof.validate::<HashAlgorithm>()
    }
    fn hash(&mut self) -> [u8; 32] {
        // t = Hash(r, c0, c1,..., pi)
        let mut hasher = Sha3::sha3_256();
        hasher.input(&self.nonce);
        hasher.input(&i128vec_to_le_bytes(&self.c0s));
        hasher.input(&i128vec_to_le_bytes(&self.c1s));
        // TODO pem or der? or other ways to convert to [u8]
        hasher.input(&self.rsa_pk);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }

    // this N should be known from the board
    // s has to be at least 1
    //pub async fn verify(&self, N: u32, s: u32) {
    //    assert!(s >= 1, "s should be at least 1");
    //    // used to trace all the "retrieved" nodes
    //    // let mut set: HashSet<u32> = HashSet::new();
    //    // random the v_init
    //    // retrieve the leafs
    //    let mut rng = rand::rngs::StdRng::from_entropy();
    //    // TODO random here
    //    //let vinit: u32 = rng.gen::<u32>() % N;
    //    let vinit: u32 = 0;
    //    // receive the leafs
    //    // for i in vinit..vinit + s + 1 {
    //    //     set.insert(i);
    //    // }
    //    // [0..N): leafs
    //    // [N..N+N/2): non-leafs whose children are leafs
    //    // N+N/2: possibly 1 leaf + 1 non-leaf
    //    // [N+N/2..summation_array_size(N)): non-leafs with non-leaf children

    //    let mut non_leafs: Vec<u32> = Vec::new();
    //    let array_size = summation_array_size(N);
    //    // first pick about s/2 non-leafs whose children are leafs
    //    // if no such non-leafs nodes exist, just skip
    //    let mut idx = vinit;
    //    while idx <= vinit + s {
    //        if (idx & 0x1 == 0) && (idx + 1 <= vinit + s) {
    //            non_leafs.push(N + idx / 2);
    //            //set.insert(N + idx / 2);
    //            idx += 2;
    //        } else {
    //            idx += 1;
    //        }
    //    }

    //    // now randomly pick non-leaf nodes whose children are non-leafs
    //    // if s is even, just pick s/2; otherwise either (s+1)/2 or s/2
    //    // or put in another way, if s is odd and vmax is even, pick (s+1) / 2
    //    // otherwise, pick s/2
    //    let nr_gp = {
    //        if (s & 0x1 != 0) && ((vinit + s) & 0x1 == 0) {
    //            (s + 1) / 2
    //        } else {
    //            s / 2
    //        }
    //    };
    //    // [0..N)
    //    // a[N] = 0 + 1, a[N+1] = 2 + 3,...
    //    // a[id_gp - N] = 2 * (id_gp - N),
    //    for _ in 0..nr_gp + 1 {
    //        // id of grand parent
    //        let id_gp = rng.gen_range(N + N / 2..array_size);
    //        // the id of the children
    //        let left = (id_gp - N) * 2;
    //        let right = left + 1;
    //        //set.insert(left);
    //        //set.insert(right);
    //        non_leafs.push(id_gp);
    //        non_leafs.push(left);
    //        non_leafs.push(right);
    //    }

    //    let result = {
    //        let mut ctx = context::current();
    //        ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
    //        self.inner.verify(ctx, vinit, non_leafs)
    //    }
    //    .await
    //    .unwrap();

    //    for i in 0..s + 1 {
    //        let mc_node = &result[2 * i as usize];
    //        let ms_node = &result[(2 * i + 1) as usize];
    //        // Commit_i appears in Mc
    //        if !mc_node.1.clone().to_proof().validate::<HashAlgorithm>() {
    //            assert!(false, "wrong merkle proofs");
    //        }
    //        if let SummationEntry::Leaf(s) = &ms_node.0 {
    //            if s.c0.is_some() {
    //                let h = s.hash();
    //                if let SummationEntry::Commit(cm) = &mc_node.0 {
    //                    //TODO fix this
    //                    //assert_eq!(h, cm.hash);
    //                } else {
    //                    assert!(false, "not commitment entry");
    //                }
    //            }
    //        } else {
    //            assert!(false, "not leaf nodes!");
    //        }
    //    }

    //    // offset by the commit and leafs
    //    let mut i = (s + 1 + s + 1) as usize;
    //    let mut idx = vinit;
    //    //let mut ii = 0 as usize;
    //    while idx <= vinit + s {
    //        if (idx & 0x1 == 0) && (idx + 1 <= vinit + s) {
    //            // TODO check these nodes by ref to the leaf nodes
    //            let parent = &result[i];
    //            let left = &result[2 * (idx - vinit) as usize + 1];
    //            let right = &result[2 * (idx - vinit + 1) as usize + 1];
    //            // check the proofs
    //            assert!(parent.1.clone().to_proof().validate::<HashAlgorithm>());
    //            if let SummationEntry::NonLeaf(a) = &parent.0 {
    //                if let SummationEntry::Leaf(b) = &left.0 {
    //                    if let SummationEntry::Leaf(c) = &right.0 {
    //                        for j in 0..a.c0.len() {
    //                            let option_b = match &b.c0 {
    //                                Some(v) => v[j],
    //                                None => 0i128,
    //                            };
    //                            let option_c = match &c.c0 {
    //                                Some(v) => v[j],
    //                                None => 0i128,
    //                            };
    //                            assert_eq!(a.c0[j], option_b + option_c);
    //                            let option_b = match &b.c1 {
    //                                Some(v) => v[j],
    //                                None => 0i128,
    //                            };
    //                            let option_c = match &c.c1 {
    //                                Some(v) => v[j],
    //                                None => 0i128,
    //                            };
    //                            assert_eq!(a.c1[j], option_b + option_c);
    //                        }
    //                    }
    //                }
    //            }
    //            i = i + 1;
    //            //non_leafs.push(N + idx / 2);
    //            //set.insert(N + idx / 2);
    //            idx += 2;
    //        } else {
    //            idx += 1;
    //        }
    //    }
    //    while i < result.len() {
    //        let parent = &result[i];
    //        let left = &result[i + 1];
    //        let right = &result[i + 2];
    //        // check the proofs
    //        assert!(parent.1.clone().to_proof().validate::<HashAlgorithm>());
    //        assert!(left.1.clone().to_proof().validate::<HashAlgorithm>());
    //        assert!(right.1.clone().to_proof().validate::<HashAlgorithm>());
    //        if let SummationEntry::NonLeaf(a) = parent.0.clone() {
    //            if let SummationEntry::NonLeaf(b) = left.0.clone() {
    //                if let SummationEntry::NonLeaf(c) = right.0.clone() {
    //                    // check the sum
    //                    for j in 0..a.c0.len() {
    //                        assert_eq!(a.c0[j], b.c0[j] + c.c0[j]);
    //                        assert_eq!(a.c1[j], b.c1[j] + c.c1[j]);
    //                    }
    //                }
    //            }
    //        }
    //        i += 3;
    //    }
    //}

    pub async fn train_model(&mut self) -> Vec<u8> {
        let rm = start_timer!(|| "retrieve the model");
        let gradient = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner.retrieve_model(ctx)
        }
        .await
        .unwrap();
        end_timer!(rm);
        // training time
        // TODO set this time properly
        thread::sleep(Duration::from_secs(1));
        //thread::sleep(Duration::from_secs(45));
        gradient
    }
    pub async fn download_proving_key(&mut self) -> Vec<u8> {
        let mut ctx = context::current();
        ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
        self.inner.retrieve_model(ctx).await.unwrap()
        // also serialize here
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = start_timer!(|| "start of clients");

    let config = ConfigUtils::init("config.ini");
    LogUtils::init(&format!("client{}.log", id()));

    let nr_real = config.get_int("nr_real") as u32;
    let nr_sybil = config.get_int("nr_sybil") as u32;

    let server_addr = (
        IpAddr::V4(config.get_addr("server_addr")),
        config.get_int("server_port") as u16,
    );
    let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);

    // WorldClient is generated by the service attribute. It has a constructor `new` that takes a
    // config and any Transport as input.
    let inner_client =
        ServerServiceClient::new(client::Config::default(), transport.await?).spawn();
    let mut client = Client::new(inner_client);

    for _ in 0..1 {
        // begin uploading
        let train = start_timer!(|| "train model");
        let data = client.train_model().await;
        end_timer!(train);

        let rs = start_timer!(|| "upload data");
        let result = client.upload(data).await;
        end_timer!(rs);

        //let vr= start_timer!(|| "upload data");
        //client.verify(100, 5).await;
        //end_timer!(vr);
    }

    end_timer!(start);
    Ok(())
}
