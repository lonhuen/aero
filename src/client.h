#pragma once
#include "merkle.h"
#include <string>
namespace quail {
class Client {
public:
  // when publish the root of commit tree, a proof of inclusion should also be
  // provided.
  void publishMc(const std::string &root_hash, const MerkleProof &proof);
  // when publish the root of summation tree, a proof of leaf node in Ms
  void publishMs(const std::string &root_hash, const MerkleProof &proof);
};

} // namespace quail