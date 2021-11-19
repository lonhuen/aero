mod common;
mod rlwe;
mod util;
mod zksnark;
use crate::common::aggregation::{
    merkle::{HashAlgorithm, MerkleProof},
    node::{SummationEntry, SummationLeaf, SummationNonLeaf},
};
use crate::common::server_service::ServerServiceClient;
use crate::common::{i128vec_to_le_bytes, summation_array_size, ZKProof};
use crate::util::{config::ConfigUtils, log::init_tracing};
use crate::zksnark::Verifier;
use ark_std::{end_timer, start_timer};
#[cfg(not(feature = "online"))]
use quail::zksnark::Prover;
#[cfg(feature = "online")]
use quail::zksnark::ProverOnline as Prover;
#[cfg(feature = "hashfn_blake3")]
extern crate blake3;
use crate::rlwe::PublicKey;
#[cfg(not(feature = "hashfn_blake3"))]
use crypto::{digest::Digest, sha3::Sha3};
use tracing::{error, event, instrument, span, warn, Level};

use rand::{Rng, SeedableRng};
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::{fs::File, io::BufReader, net::IpAddr};
use std::{io::BufRead, process::id};
use std::{thread, time};
use tarpc::{client, context, tokio_serde::formats::Bincode};
use tracing_subscriber::filter::LevelFilter;

const DEADLINE_TIME: u64 = 6000;
const NUM_DIMENSION: u32 = 4096;
pub struct Client {
    inner: ServerServiceClient,
    rsa_pk: Vec<u8>,
    _rsa_vk: RsaPrivateKey,
    c0s: Vec<Vec<i128>>,
    c1s: Vec<Vec<i128>>,
    rs: Vec<Vec<i128>>,
    e0s: Vec<Vec<i128>>,
    e1s: Vec<Vec<i128>>,
    d0s: Vec<Vec<i128>>,
    d1s: Vec<Vec<i128>>,
    m: Vec<Vec<i128>>,
    nonce: Vec<[u8; 16]>,
    prover: Prover,
    //prover: ProverOnline,
    enc_pk: PublicKey,
}

impl Client {
    pub fn new(inner: ServerServiceClient) -> Self {
        let bits = 2048;
        let mut rng = rand::rngs::StdRng::from_entropy();
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);
        let enc_pk = {
            let (pk0, pk1) = {
                let mut pk_0 = [0i128; 4096];
                let mut pk_1 = [0i128; 4096];
                let file = match File::open("./data/encryption.txt") {
                    Ok(f) => f,
                    Err(_) => panic!(),
                };
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        let vec = l.split(" ").collect::<Vec<&str>>();
                        for i in 1..vec.len() {
                            if l.contains("pk_0") {
                                if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                                    pk_0[i - 1] = x;
                                }
                            } else if l.contains("pk_1") {
                                if let Ok(x) = i128::from_str_radix(vec[i], 10) {
                                    pk_1[i - 1] = x;
                                }
                            }
                        }
                    }
                }
                (pk_0.to_vec(), pk_1.to_vec())
            };
            PublicKey::new(&pk0, &pk1)
        };
        let prover = Prover::new("./data/encryption.txt", "./data/proving_key.txt");
        //let prover = ProverOnline::new("./data/encryption.txt", "./data/proving_key.txt");
        Self {
            inner,
            rsa_pk: public_key.to_public_key_pem().unwrap().into_bytes(),
            _rsa_vk: private_key,
            c0s: Vec::new(),
            c1s: Vec::new(),
            rs: Vec::new(),
            e0s: Vec::new(),
            e1s: Vec::new(),
            d0s: Vec::new(),
            d1s: Vec::new(),
            m: Vec::new(),
            nonce: Vec::new(),
            prover,
            enc_pk,
        }
    }
    #[inline(always)]
    fn clear(&mut self) {
        self.c0s.clear();
        self.c1s.clear();
        self.rs.clear();
        self.e0s.clear();
        self.e1s.clear();
        self.d0s.clear();
        self.d1s.clear();
    }

    #[instrument(skip_all, name = "encrypt")]
    pub fn encrypt(&mut self, xs: Vec<u8>) {
        self.clear();
        for i in 0..xs.len() / NUM_DIMENSION as usize {
            let (r, e0, e1, d0, d1, ct) = self
                .enc_pk
                .encrypt(&xs[i * NUM_DIMENSION as usize..(i + 1) * NUM_DIMENSION as usize]);
            self.rs.push(r);
            self.e0s.push(e0);
            self.e1s.push(e1);
            self.d0s.push(d0);
            self.d1s.push(d1);
            self.c0s.push(ct.c_0);
            self.c1s.push(ct.c_1);
            let m = xs[i * NUM_DIMENSION as usize..(i + 1) * NUM_DIMENSION as usize]
                .iter()
                .map(|x| *x as i128)
                .collect();
            self.m.push(m);
            // TODO random this nonce
            self.nonce.push([0u8; 16]);
        }
    }
    #[instrument(skip_all, name = "generate_proof")]
    pub fn generate_proof(&self) -> Vec<Vec<u8>> {
        let gc = start_timer!(|| "start proof generation");
        #[cfg(not(feature = "online"))]
        let ret = self.prover.create_proof_in_bytes(
            &self.c0s, &self.c1s, &self.rs, &self.e0s, &self.e1s, &self.d0s, &self.d1s, &self.m,
        );
        #[cfg(feature = "online")]
        let ret = self.prover.create_proof_in_bytes(
            &self.c0s,
            &self.rs,
            &self.e0s,
            &self.d0s,
            &self.m,
            &vec![[0u8; 224]],
        );
        //println!("proof len {} * {}", ret.len(), ret[0].len());
        // # of ct * 192
        end_timer!(gc);
        ret
    }

    // the whole aggregation phase (except the encryption)
    #[instrument(skip_all)]
    pub async fn upload(&mut self, round: u32, xs: Vec<u8>, pvk: Vec<u8>) -> bool {
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
                .aggregate_commit(ctx, round, self.rsa_pk.clone(), cm)
                .await;
            self.inner.get_mc_proof(ctx, round, self.rsa_pk.clone())
        };
        // while waiting for the commitment, compute the zkproof
        let proofs = self.generate_proof();

        // wait for the Mc tree
        let mc_proof = result_commit.await.unwrap();
        end_timer!(gc2);

        let gc3 = start_timer!(|| "upload the data");
        // proceed to summation tree

        warn!("data prepared");
        let result_data = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            let _ = self
                .inner
                .aggregate_data(
                    ctx,
                    round,
                    self.rsa_pk.clone(),
                    self.c0s.clone(),
                    self.c1s.clone(),
                    self.nonce.clone(),
                    proofs,
                )
                .await;
            warn!("data uploaded,receving ms proof");
            self.inner.get_ms_proof(ctx, round, self.rsa_pk.clone())
        };

        let ms_proof: Vec<MerkleProof> = result_data.await.unwrap();
        warn!("ms proof received");
        end_timer!(gc3);
        // TODO verify the proof by checking x.leaf for mc, ms maybe
        //{
        //    let mut hasher = Sha3::sha3_256();
        //    hasher.input(&self.rsa_pk);
        //    hasher.input(&cm);
        //    let mut h = [0u8; 32];
        //    hasher.result(&mut h);
        //    let mut x = HashAlgorithm::new();
        //    warn!("hash {:?}", x.leaf(h));
        //}
        let mut flag = true;
        for p in ms_proof {
            flag = flag & p.clone().to_proof().validate::<HashAlgorithm>();
        }
        for p in mc_proof {
            flag = flag & p.clone().to_proof().validate::<HashAlgorithm>();
        }
        flag
    }

    #[cfg(not(feature = "hashfn_blake3"))]
    fn hash(&mut self) -> [u8; 32] {
        // t = Hash(r, c0, c1,..., pi)
        let mut hasher = Sha3::sha3_256();
        hasher.input(&self.nonce);
        hasher.input(&i128vec_to_le_bytes(&self.c0s));
        hasher.input(&i128vec_to_le_bytes(&self.c1s));
        hasher.input(&self.rsa_pk);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }
    #[cfg(feature = "hashfn_blake3")]
    fn hash(&mut self) -> Vec<[u8; 32]> {
        (0..self.c0s.len())
            .into_iter()
            .map(|i| {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&self.nonce[i]);
                hasher.update(&i128vec_to_le_bytes(&self.c0s[i]));
                hasher.update(&i128vec_to_le_bytes(&self.c1s[i]));
                hasher.update(&self.rsa_pk);
                hasher.finalize().into()
            })
            .collect()
    }

    //TODO maybe fix when "# of clients < s"
    pub fn get_random_non_leafs(n: u32, s: u32, vinit: u32) -> Vec<u32> {
        let mut rng = rand::rngs::StdRng::from_entropy();
        // [0..N): leafs
        // [N..N+N/2): non-leafs whose children are leafs
        // N+N/2: possibly 1 leaf + 1 non-leaf
        // [N+N/2..summation_array_size(N)): non-leafs with non-leaf children
        let mut non_leafs: Vec<u32> = Vec::new();
        let array_size = summation_array_size(n);
        // first pick about s/2 non-leafs whose children are leafs
        // if no such non-leafs nodes exist, just skip
        let mut idx = 0;
        while idx <= s {
            let ii = (idx + vinit) % n;
            if (ii & 0x1 == 0) && (ii + 1 <= n) {
                non_leafs.push(n + ii / 2);
                idx += 2;
            } else {
                idx += 1;
            }
        }

        // now randomly pick non-leaf nodes whose children are non-leafs
        // if s is even, just pick s/2; otherwise either (s+1)/2 or s/2
        // or put in another way, if s is odd and vmax is even, pick (s+1) / 2
        // otherwise, pick s/2
        let nr_gp = {
            if (s & 0x1 != 0) && ((vinit + s) & 0x1 == 0) {
                (s + 1) / 2
            } else {
                s / 2
            }
        };
        // [0..N)
        // a[N] = 0 + 1, a[N+1] = 2 + 3,...
        // a[id_gp - N] = 2 * (id_gp - N),
        for _ in 0..nr_gp + 1 {
            // id of grand parent
            if n + n / 2 < array_size {
                let id_gp = rng.gen_range(n + n / 2..array_size);
                // the id of the children
                let left = (id_gp - n) * 2;
                let right = left + 1;
                non_leafs.push(id_gp);
                non_leafs.push(left);
                non_leafs.push(right);
            }
        }
        non_leafs
    }

    // this N should be known from the board
    // s has to be at least 1
    #[instrument(skip_all)]
    pub async fn verify(&self, round: u32, n: u32, s: u32) {
        let gc = start_timer!(|| "verify");

        assert!(s >= 1, "s should be at least 1");
        let mut rng = rand::rngs::StdRng::from_entropy();
        // TODO random here
        //let vinit: u32 = rng.gen::<u32>() % N;
        let vinit: u32 = 0;

        let non_leafs: Vec<u32> = Self::get_random_non_leafs(n, s, vinit);

        let gc1 = start_timer!(|| "receive verify");
        let ct_id = vec![0usize, 1usize, 2usize];
        let ret = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner
                .verify(ctx, round, vinit, non_leafs, ct_id.clone())
        }
        .await
        .unwrap();
        end_timer!(gc1);

        // if not able to retrieve the proof
        if ret.len() == 0 {
            return;
        }

        println!("leaf len {} * {}", ret.len(), ret[0].len());

        // verify all the leafs
        let gc2 = start_timer!(|| "verify the proofs");
        for result in &ret {
            for i in 0..s + 1 {
                let mc_node = &result[2 * i as usize];
                let ms_node = &result[(2 * i + 1) as usize];
                // Commit_i appears in Mc
                assert!(
                    mc_node.1.clone().to_proof().validate::<HashAlgorithm>(),
                    "wrong merkle proofs"
                );
                if let SummationEntry::Leaf(s) = &ms_node.0 {
                    let h = s.hash();
                    if let SummationEntry::Commit(cm) = &mc_node.0 {
                        //TODO fix this
                        // assert_eq!(h, cm.hash);
                    } else {
                        error!("Atom: Verify not commit entry!");
                    }
                } else {
                    error!("Atom: Verify not leaf nodes entry!");
                }
            }

            // offset by the commit and leafs
            let mut i = (s + 1 + s + 1) as usize;
            let mut idx = vinit;
            //let mut ii = 0 as usize;
            while idx <= vinit + s {
                let ii = idx % n;
                if (ii & 0x1 == 0) && (ii + 1 <= vinit + s) {
                    // check these nodes by ref to the leaf nodes
                    let parent = &result[i];
                    let left = &result[2 * (idx - vinit) as usize + 1];
                    let right = &result[2 * (idx - vinit + 1) as usize + 1];
                    // check the proofs
                    assert!(parent.1.clone().to_proof().validate::<HashAlgorithm>());

                    let c = match (&left.0, &right.0) {
                        // TODO add random pt here
                        (SummationEntry::Leaf(a), SummationEntry::Leaf(b)) => {
                            &a.evaluate_at(0) + &b.evaluate_at(0)
                        }
                        //(SummationEntry::Leaf(a), SummationEntry::NonLeaf(b)) => a + b,
                        //(SummationEntry::NonLeaf(a), SummationEntry::Leaf(b)) => a + b,
                        //(SummationEntry::NonLeaf(a), SummationEntry::NonLeaf(b)) => a + b,
                        _ => {
                            panic!("Not leaf nodes in verifying summation");
                        }
                    };

                    if let SummationEntry::NonLeaf(a) = &parent.0 {
                        assert_eq!(&c, a);
                    } else {
                        //error!("Parent not a nonleaf node when leaf");
                    }
                    i = i + 1;
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            while i < result.len() {
                let parent = &result[i];
                let left = &result[i + 1];
                let right = &result[i + 2];
                // check the proofs
                assert!(parent.1.clone().to_proof().validate::<HashAlgorithm>());
                assert!(left.1.clone().to_proof().validate::<HashAlgorithm>());
                assert!(right.1.clone().to_proof().validate::<HashAlgorithm>());
                // possibly leaf + non_leaf when the # of leafs is odd
                let c = match (&left.0, &right.0) {
                    (SummationEntry::NonLeaf(a), SummationEntry::NonLeaf(b)) => a + b,
                    (SummationEntry::Leaf(a), SummationEntry::NonLeaf(b)) => &a.evaluate_at(0) + b,
                    //(SummationEntry::Leaf(a), SummationEntry::Leaf(b)) => a + b,
                    //(SummationEntry::NonLeaf(a), SummationEntry::Leaf(b)) => a + b,
                    _ => {
                        panic!("Not leaf nodes in verifying summation");
                    }
                };
                if let SummationEntry::NonLeaf(a) = &parent.0 {
                    assert_eq!(&c, a);
                } else {
                    //error!("Parent not a nonleaf node when nonleaf");
                }
                i += 3;
            }
        }
        end_timer!(gc2);
    }

    #[instrument(skip_all)]
    pub async fn train_model(&mut self, round: u32) -> Vec<u8> {
        let rm = start_timer!(|| "retrieve the model");
        let gradient = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner.retrieve_model(ctx, round)
        }
        .await
        .unwrap();
        end_timer!(rm);
        // training time
        // TODO siumulation set this time properly
        thread::sleep(Duration::from_secs(1));
        //thread::sleep(Duration::from_secs(45));
        gradient
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ConfigUtils::init("config.yaml");
    init_tracing(
        &format!("Atom client {}", std::process::id()),
        config.get_agent_endpoint(),
        LevelFilter::WARN,
    )?;

    let _span = span!(Level::WARN, "Atom Client").entered();

    let start = start_timer!(|| "clients");

    let nr_real = config.get_int("nr_real") as u32;
    let nr_sim = config.get_int("nr_simulated") as u32;
    let nr_round = config.get_int("nr_round") as u32;

    let server_addr = (
        IpAddr::V4(config.get_addr("server_addr")),
        config.get_int("server_port") as u16,
    );
    #[cfg(feature = "json")]
    let mut transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
    #[cfg(not(feature = "json"))]
    let mut transport = tarpc::serde_transport::tcp::connect(server_addr, Bincode::default);
    transport.config_mut().max_frame_length(usize::MAX);

    let inner_client =
        ServerServiceClient::new(client::Config::default(), transport.await?).spawn();
    let mut client = Client::new(inner_client);
    for i in 0..nr_round {
        // begin uploading
        let sr = start_timer!(|| "one round");
        let train = start_timer!(|| "train model");
        let data = client.train_model(i).await;
        end_timer!(train);

        let rs = start_timer!(|| "upload data");
        //let result = client.upload(i, data, pvk.await.unwrap()).await;
        let result = client.upload(i, data, vec![0u8; 1]).await;
        end_timer!(rs);

        let vr = start_timer!(|| "verify the data");
        client.verify(i, nr_real + nr_sim, 5).await;
        end_timer!(vr);
        end_timer!(sr);
    }
    end_timer!(start);
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
