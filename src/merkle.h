#pragma once
#include <string>
#include <vector>

namespace quail {
typedef std::vector<std::string> MerkleProof;

struct MerkleNode {
  std::string hash;
  MerkleNode *left;
  MerkleNode *right;
  MerkleNode(std::string hash) : hash(hash), left(nullptr), right(nullptr){};
  MerkleNode(){};
};

class MerkleTree {

public:
  int nr_blocks;
  std::vector<MerkleNode *> leaf;
  MerkleNode *root;
  MerkleTree(const std::vector<std::string> &blocks);
  MerkleProof proofOfInclusion(int id);
  // this function can be called by clients.
  static bool proveInclusion(const std::string &root_hash, const int block_id,
                             const int total_blocks, const std::string &block,
                             const MerkleProof &proof);

  // private:
  void printMerkleTree();
  MerkleNode *constructTree(int start, int end);
};
} // namespace quail