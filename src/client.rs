pub mod cipher;
pub mod proof;
use crate::cipher::CipherText;
use crate::proof::{Prover, Verifier};
#[path = "server.rs"]
mod server;
use crate::server::ExampleAlgorithm;
mod server_service;
use crate::server_service::{MerkleProof, ServerServiceClient, ZKProof};
use ark_std::{end_timer, start_timer};
use crypto::digest::Digest;
use crypto::sha3::{Sha3, Sha3Mode};
use rand::{rngs::StdRng, SeedableRng};
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use std::convert::TryInto;
use std::net::{IpAddr, Ipv6Addr};
use std::time::{Instant, SystemTime};
use std::{net::SocketAddr, time::Duration};
use std::{thread, time};
use tarpc::{client, context, tokio_serde::formats::Json};
use tokio::time::sleep;

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
        let result_commit = async {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner.aggregate_commit(ctx, self.rsa_pk.clone(), cm)
        };
        // while waiting for the commitment, compute the zkproof
        let proofs = self.generate_proof();

        // wait for the Mc tree
        let mc_proof = result_commit.await.await.unwrap().to_proof();

        // proceed to summation tree
        let mut cts_bytes: Vec<u8> = Vec::with_capacity(self.cts.len() * 65536 * 2);
        // TODO this might needs to be changed
        let mut proof_bytes: Vec<u8> = Vec::with_capacity(self.cts.len() * 192);

        for i in 0..self.cts.len() {
            cts_bytes.extend(self.cts[i].c0.iter());
            cts_bytes.extend(self.cts[i].c1.iter());
            proof_bytes.extend(proofs[i].iter());
        }

        let result_data = async {
            let mut ctx = context::current();
            ctx.deadline = SystemTime::now() + Duration::from_secs(DEADLINE_TIME);
            self.inner
                .aggregate_data(ctx, self.rsa_pk.clone(), cts_bytes, self.nonce, proof_bytes)
        };

        let ms_proof = result_data.await.await.unwrap().to_proof();
        // verify the proofs
        ms_proof.validate::<ExampleAlgorithm>() && mc_proof.validate::<ExampleAlgorithm>()
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
    println!("{}", result);

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
