mod common;
mod zksnark;
use crate::common::cipher::CipherText;
use crate::zksnark::{Prover, Verifier};

use crate::common::aggregation::{
    merkle::HashAlgorithm, merkle::MerkleProof, SummationEntry, SummationLeaf, SummationNonLeaf,
};
use crate::common::server_service::ServerServiceClient;
use crate::common::{summation_array_size, ZKProof};
use ark_std::{end_timer, start_timer};
use crypto::digest::Digest;
use crypto::sha3::{Sha3, Sha3Mode};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use std::collections::HashSet;
use std::convert::TryInto;
use std::net::{IpAddr, Ipv6Addr};
use std::thread::sleep;
use std::time::{Instant, SystemTime};
use std::{net::SocketAddr, time::Duration};
use std::{thread, time};
use tarpc::{client, context, tokio_serde::formats::Json};

const DEADLINE_TIME: u64 = 60;
pub struct Client {
    inner: ServerServiceClient,
    rsa_pk: Vec<u8>,
    rsa_vk: RsaPrivateKey,
    cts: Vec<CipherText>,
    nonce: [u8; 16],
}

impl Client {
    // TODO random this nounce
    pub fn new(inner: ServerServiceClient) -> Self {
        let bits = 2048;
        //let mut rng = rand::rngs::StdRng::seed_from_u64(Instant::now().);
        let mut rng = rand::rngs::StdRng::from_entropy();
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let public_key = RsaPublicKey::from(&private_key);
        Self {
            inner,
            rsa_pk: public_key.to_public_key_pem().unwrap().into_bytes(),
            rsa_vk: private_key,
            cts: Vec::new(),
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
        thread::sleep(time::Duration::from_millis(87) * (xs.len() as usize).try_into().unwrap());
        for _ in 0..xs.len() {
            self.cts.push(CipherText::new());
        }
    }
    // TODO need to fix this with setup phase
    // but for now, let's read the proof and sleep some time to simulate the create proof
    pub fn generate_proof(&self) -> Vec<Vec<u8>> {
        thread::sleep(
            //time::Duration::from_millis(8250) * (self.cts.len() as usize).try_into().unwrap(),
            time::Duration::from_millis(8250),
        );
        //let prover = Prover::new(enc_path);
        //let buf = prover.create_proof_in_bytes();
        //vec![ZKProof::default(); self.cts.len()]
        vec![vec![0u8; 192]; self.cts.len()]
    }

    // the whole aggregation phase (except the encryption)
    pub async fn upload(&mut self, xs: Vec<u8>) -> bool {
        // set the deadline of the context
        self.encrypt(xs);
        // generate commitment to all the CTs
        let cm = self.hash();
        // send this commitment to the server
        let result_commit = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner.aggregate_commit(ctx, self.rsa_pk.clone(), cm)
        };
        // while waiting for the commitment, compute the zkproof
        let proofs = self.generate_proof();

        // wait for the Mc tree
        let mc_proof = result_commit.await.unwrap().to_proof();

        // proceed to summation tree
        let mut cts_bytes: Vec<u8> = Vec::with_capacity(self.cts.len() * 65536 * 2);
        // TODO this might needs to be changed
        let mut proof_bytes: Vec<u8> = Vec::with_capacity(self.cts.len() * 192);

        for i in 0..self.cts.len() {
            cts_bytes.extend(self.cts[i].c0.iter());
            proof_bytes.extend(proofs[i].iter());
        }
        for i in 0..self.cts.len() {
            cts_bytes.extend(self.cts[i].c1.iter());
        }
        let result_data = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner
                .aggregate_data(ctx, self.rsa_pk.clone(), cts_bytes, self.nonce, proof_bytes)
        };

        let ms_proof = result_data.await.unwrap().to_proof();
        // verify the proofs
        ms_proof.validate::<HashAlgorithm>() && mc_proof.validate::<HashAlgorithm>()
    }
    fn hash(&mut self) -> [u8; 32] {
        // t = Hash(r, c0, c1,..., pi)
        let mut hasher = Sha3::sha3_256();
        hasher.input(&self.nonce);
        for ct in self.cts.iter() {
            hasher.input(&ct.c0);
            hasher.input(&ct.c1);
        }
        // TODO pem or der? or other ways to convert to [u8]
        hasher.input(&self.rsa_pk);
        let mut h = [0u8; 32];
        hasher.result(&mut h);
        h
    }

    // this N should be known from the board
    // s has to be at least 1
    pub async fn verify(&self, N: u32, s: u32) {
        assert!(s >= 1, "s should be at least 1");
        // used to trace all the "retrieved" nodes
        // let mut set: HashSet<u32> = HashSet::new();
        // random the v_init
        // retrieve the leafs
        let mut rng = rand::rngs::StdRng::from_entropy();
        // TODO random here
        //let vinit: u32 = rng.gen::<u32>() % N;
        let vinit: u32 = 0;
        // receive the leafs
        // for i in vinit..vinit + s + 1 {
        //     set.insert(i);
        // }
        // [0..N): leafs
        // [N..N+N/2): non-leafs whose children are leafs
        // N+N/2: possibly 1 leaf + 1 non-leaf
        // [N+N/2..summation_array_size(N)): non-leafs with non-leaf children

        let mut non_leafs: Vec<u32> = Vec::new();
        let array_size = summation_array_size(N);
        // first pick about s/2 non-leafs whose children are leafs
        // if no such non-leafs nodes exist, just skip
        let mut idx = vinit;
        while idx <= vinit + s {
            if (idx & 0x1 == 0) && (idx + 1 <= vinit + s) {
                non_leafs.push(N + idx / 2);
                //set.insert(N + idx / 2);
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
            let id_gp = rng.gen_range(N + N / 2..array_size);
            // the id of the children
            let left = (id_gp - N) * 2;
            let right = left + 1;
            //set.insert(left);
            //set.insert(right);
            non_leafs.push(id_gp);
            non_leafs.push(left);
            non_leafs.push(right);
        }

        let result = {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner.verify(ctx, vinit, non_leafs)
        }
        .await
        .unwrap();

        for i in 0..s + 1 {
            let mc_node = &result[2 * i as usize];
            let ms_node = &result[(2 * i + 1) as usize];
            // Commit_i appears in Mc
            if !mc_node.1.clone().to_proof().validate::<HashAlgorithm>() {
                assert!(false, "wrong merkle proofs");
            }
            if let SummationEntry::Leaf(s) = ms_node.0.clone() {
                // check the hash
                if s.c0.is_some() {
                    if let SummationEntry::Commit(cm) = mc_node.0.clone() {
                        let h = {
                            let mut hasher = Sha3::sha3_256();
                            hasher.input(&s.r.unwrap());
                            let c0: Vec<u8> = (0..4096)
                                .flat_map(|i| i128::to_le_bytes(s.c0.as_ref().unwrap()[i]))
                                .collect();
                            let c1: Vec<u8> = (0..4096)
                                .flat_map(|i| i128::to_le_bytes(s.c1.as_ref().unwrap()[i]))
                                .collect();
                            hasher.input(&c0);
                            hasher.input(&c1);
                            hasher.input(&self.rsa_pk);
                            let mut h = [0u8; 32];
                            hasher.result(&mut h);
                            h
                        };
                        //TODO maybe some bug in serialization of c0 and c1 somewhere
                        //assert_eq!(h, cm.hash);
                    }
                }
            } else {
                assert!(false, "not leaf nodes!");
            }
        }

        let mut i = (s + 1) as usize;
        let mut idx = vinit;
        while idx <= vinit + s {
            if (idx & 0x1 == 0) && (idx + 1 <= vinit + s) {
                // TODO check these nodes by ref to the leaf nodes
                i = i + 1;
                //non_leafs.push(N + idx / 2);
                //set.insert(N + idx / 2);
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
            if let SummationEntry::NonLeaf(a) = parent.0.clone() {
                if let SummationEntry::NonLeaf(b) = left.0.clone() {
                    if let SummationEntry::NonLeaf(c) = right.0.clone() {
                        // check the sum
                        for i in 0..4096 {
                            assert_eq!(a.c0[i], b.c0[i] + c.c0[i]);
                            assert_eq!(a.c1[i], b.c1[i] + c.c1[i]);
                        }
                    }
                }
            }
            i += 3;
        }
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 38886u16);
    let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);

    // WorldClient is generated by the service attribute. It has a constructor `new` that takes a
    // config and any Transport as input.
    let inner_client =
        ServerServiceClient::new(client::Config::default(), transport.await?).spawn();
    let mut client = Client::new(inner_client);

    // begin uploading
    let result = client.upload(vec![0u8; 1]).await;

    sleep(Duration::from_secs(1));
    println!("{}", result);
    client.verify(8, 5).await;

    Ok(())
}

//
//fn main() {
//let client = Client::new();

//let gc = start_timer!(|| "start setup");
//Prover::setup("data/pk.txt", "data/vk.txt", "data/encryption.txt");
//end_timer!(gc);
//
//let gc = start_timer!(|| "new prover"); let prover = Prover::new("data/encryption.txt", "data/pk.txt"); end_timer!(gc);
//let gc = start_timer!(|| "create proof");
////let proof = prover.create_proof_in_bytes();
//let proof = prover.create_proof();
//end_timer!(gc);

//let inputs: Vec<_> = prover
//    .circuit
//    .c_0
//    .to_vec()
//    .iter()
//    .chain(prover.circuit.c_1.to_vec().iter())
//    .map(|&x| prover.circuit.i128toField(x))
//    .collect::<Vec<_>>();
//let verifier = Verifier::new("data/vk.txt");
////let result = verifier.verify_proof_from_bytes(&proof, &inputs);
//let gc = start_timer!(|| "verification");
//let result = verifier.verify_proof(&proof, &inputs);
//end_timer!(gc);

//println!("result {}", result);
//assert!(result);
//}
