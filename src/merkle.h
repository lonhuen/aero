#pragma once
#include <string>
#include <vector>

namespace quail {
typedef std::vector<std::pair<bool, std::string>> MerkleProof;

struct MerkleNode {
  std::string hash;
  MerkleNode *left;
  MerkleNode *right;
  MerkleNode *parent;
  MerkleNode *pair;
  MerkleNode(std::string hash)
      : hash(hash), left(nullptr), right(nullptr), parent(nullptr),
        pair(nullptr){};
};

class MerkleTree {

public:
  int nr_blocks;
  std::vector<MerkleNode *> leaf;
  MerkleNode *root;
  MerkleTree(const std::vector<std::string> &blocks);
  std::vector<std::pair<bool, std::string>> proofOfInclusion(int id);
  // this function can be called by clients.
  static bool proveInclusion(const std::string &root_hash,
                             const std::string &block,
                             const MerkleProof &proof);

  // private:
  void printMerkleTree();
};
} // namespace quail