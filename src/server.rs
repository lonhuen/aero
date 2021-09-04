use futures::{
    future::{self, Ready},
    prelude::*,
};
use std::{
    collections::BTreeMap,
    convert::{Into, TryInto},
    io,
    iter::FromIterator,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::{Arc, Condvar, Mutex},
};

use tarpc::{
    context,
    server::{self, Channel, Incoming},
    tokio_serde::formats::Json,
};
mod common;
use crate::common::{
    aggregation::{merkle::*, SummationEntry, SummationLeaf},
    hash_commitment,
    server_service::ServerService,
};
const NR_COMMIT: u32 = 8;
pub struct Server {
    pub commit_array: BTreeMap<Vec<u8>, MerkleHash>,
    pub mc: Option<MerkleTree>,
    pub summation_array: Vec<SummationEntry>,
    pub ms: Option<MerkleTree>,
}
impl Server {
    pub fn new() -> Self {
        Self {
            commit_array: BTreeMap::new(),
            mc: None,
            summation_array: Vec::new(),
            ms: None,
        }
    }
}
#[derive(Clone)]
pub struct InnerServer {
    addr: SocketAddr,
    server: Arc<Mutex<Server>>,
    cond: Arc<(Mutex<u32>, Condvar)>,
}
impl InnerServer {
    pub fn new(
        addr: SocketAddr,
        server: &Arc<Mutex<Server>>,
        cond: &Arc<(Mutex<u32>, Condvar)>,
    ) -> Self {
        Self {
            addr,
            server: server.clone(),
            cond: cond.clone(),
        }
    }
}
#[tarpc::server]
impl ServerService for InnerServer {
    type AggregateCommitFut = Ready<MerkleProof>;
    type AggregateDataFut = Ready<MerkleProof>;
    type VerifyFut = Ready<Vec<(SummationEntry, MerkleProof)>>;
    fn aggregate_commit(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        commitment: [u8; 32],
    ) -> Self::AggregateCommitFut {
        // wait for enough commitments
        // TODO maybe wait for some time rather than some # of commitments
        // TODO to fix: if another batch comes, the commit array will be modified
        let mut num_clients = self.cond.0.lock().unwrap();

        let idx = *num_clients;

        // push into the server
        {
            self.server
                .lock()
                .unwrap()
                .commit_array
                .insert(rsa_pk, commitment);
        }

        *num_clients = *num_clients + 1;
        if *num_clients < NR_COMMIT {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        } else if *num_clients == NR_COMMIT {
            {
                let s = &mut *self.server.lock().unwrap();
                // init the summation leaf array
                for _ in 0..s.commit_array.len() {
                    s.summation_array
                        .push(SummationEntry::Leaf(SummationLeaf::new()));
                }
                s.mc = Some(MerkleTree::from_iter(
                    s.commit_array.iter().map(|x| hash_commitment(&x.0, &x.1)),
                ));
            }
            // notify all and generate the commit tree
            // also reset the num_clients for data
            *num_clients = 0;
            self.cond.1.notify_all();
        } else {
            //TODO maybe never reach here?
            assert!(false);
        }
        // unlock
        drop(num_clients);

        let proof_commit = {
            let s = &*self.server.lock().unwrap();
            s.mc.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        };

        future::ready(proof_commit.into())
    }
    fn aggregate_data(
        self,
        _: context::Context,
        rsa_pk: Vec<u8>,
        cts: Vec<u8>,
        nonce: [u8; 16],
        proofs: Vec<u8>,
    ) -> Self::AggregateCommitFut {
        let mut num_clients = self.cond.0.lock().unwrap();

        let mut s = self.server.lock().unwrap();

        let sorted_keys: Vec<_> = s.commit_array.keys().cloned().collect();
        // TODO also verify the proof
        let idx = {
            if let Ok(id) = sorted_keys.binary_search(&rsa_pk) {
                // TODO no redundant should be sent
                s.summation_array[id] =
                    SummationEntry::Leaf(SummationLeaf::from_ct(rsa_pk, cts, nonce));
                id
            } else {
                assert!(false);
                // just to make the compiler happy
                0
            }
        };

        // unlock
        drop(s);

        *num_clients = *num_clients + 1;
        if *num_clients < NR_COMMIT {
            num_clients = self.cond.1.wait(num_clients).unwrap();
        } else if *num_clients == NR_COMMIT {
            {
                let s = &mut *self.server.lock().unwrap();
                // add the whole tree
                let mut left = 0;
                let mut right = s.summation_array.len();
                while left + 1 < right {
                    let a = match s.summation_array[left].clone() {
                        SummationEntry::NonLeaf(y) => y,
                        SummationEntry::Leaf(x) => x.into(),
                    };
                    let b = match s.summation_array[left + 1].clone() {
                        SummationEntry::NonLeaf(y) => y,
                        SummationEntry::Leaf(x) => x.into(),
                    };
                    let c = a + b;
                    s.summation_array.push(SummationEntry::NonLeaf(c));
                    left += 2;
                    right += 1;
                }
                // just for test purpose
                //{
                //    let ii = s.summation_array.len() - 1;
                //    let result = match s.summation_array[ii].clone() {
                //        SummationEntry::NonLeaf(x) => Some(x),
                //        _ => None,
                //    };
                //    println!("{:?}", result.unwrap().c0[0]);
                //}

                s.ms = Some(MerkleTree::from_iter(s.summation_array.iter().map(
                    |x| match x {
                        SummationEntry::Leaf(y) => y.hash(),
                        SummationEntry::NonLeaf(y) => y.hash(),
                    },
                )));
            }
            // notify all and generate the commit tree
            self.cond.1.notify_all();
        } else {
            //TODO maybe never reach here?
            assert!(false);
        }
        // unlock
        drop(num_clients);

        let proof_leaf = {
            let s = &*self.server.lock().unwrap();
            s.ms.as_ref().unwrap().gen_proof(idx.try_into().unwrap())
        };

        future::ready(proof_leaf.into())
    }

    fn verify(self, _: context::Context, vinit: u32, non_leaf_id: Vec<u32>) -> Self::VerifyFut {
        // TODO maybe RwLock? not able to directly read the content
        //first all the leafs
        let mut ret: Vec<(Vec<SummationEntry>, MerkleProof)> = Vec::new();
        let s = self.server.lock().unwrap();
        for i in 0..5 {
            let proof: MerkleProof =
                s.ms.as_ref()
                    .unwrap()
                    .gen_proof((i + vinit).try_into().unwrap())
                    .into();
        }
        future::ready(Vec::new())
    }
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 38886u16);

    let mut server = Server::new();
    let mut server_ref = Arc::new(Mutex::new(server));
    let mut cond_ref = Arc::new((Mutex::new(0), Condvar::new()));

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(10, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let inner_server = InnerServer::new(
                channel.transport().peer_addr().unwrap(),
                &server_ref,
                &cond_ref,
            );
            channel.execute(inner_server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}
