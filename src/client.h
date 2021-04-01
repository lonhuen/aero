#pragma once
#include "merkle.h"
#include "seal/seal.h"
#include <string>
#include <chrono>

namespace quail {

class Client {
public:
  Client(const seal::SEALContext context);
  // when publish the root of commit tree, a proof of inclusion should also be
  // provided.
  void publishMc(const std::string &root_hash, const MerkleProof &proof);
  // when publish the root of summation tree, a proof of leaf node in Ms
  void publishMs(const std::string &root_hash, const MerkleProof &proof);
  // Uploads to server updated parameters from local training,  
  void localUpdate();
  // Trains received global with local data 
  void trainModel();

private:

  seal::PublicKey public_key;
  seal::SecretKey secret_key;

  seal::Encryptor* encryptor;
  seal::Decryptor* decryptor;

  // TODO: Mock local data
  
};

} // namespace quail