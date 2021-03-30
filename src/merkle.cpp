#include "merkle.h"
#include "picosha2.h"
#include "utils/log_utils.h"
#include <iostream>
#include <queue>

namespace quail {
MerkleTree::MerkleTree(const std::vector<std::string> &blocks)
    : nr_blocks(blocks.size()) {
  if (blocks.size() < 1) {
    LOG(FATAL) << "Empty blocks";
    return;
  }
  leaf.reserve(blocks.size());
  std::queue<MerkleNode *> level_nodes;
  for (int i = 0; i < blocks.size(); i++) {
    MerkleNode *n = new MerkleNode(picosha2::hash256_hex_string(blocks[i]));
    level_nodes.push(n);
    leaf.push_back(n);
  }
  this->root = constructTree(0, leaf.size());
}

MerkleNode *MerkleTree::constructTree(int start, int end) {
  if (start >= end)
    return nullptr;
  // leaf
  if (start + 1 == end) {
    return leaf[start];
  }
  // non-leaf node
  MerkleNode *t = new MerkleNode();
  t->left = constructTree(start, (start + end) / 2);
  t->right = constructTree((start + end) / 2, end);
  t->hash = picosha2::hash256_hex_string(t->left->hash + t->right->hash);
  return t;
}

MerkleProof MerkleTree::proofOfInclusion(int id) {
  if ((id >= nr_blocks) || (id < 0)) {
    LOG(FATAL) << "Index exceeds bound";
  }
  MerkleProof ret;
  int start = 0;
  int end = nr_blocks;
  MerkleNode *t = root;
  while (start + 1 < end) {
    int mid = (start + end) / 2;
    // left path
    if (id < mid) {
      ret.push_back(t->right->hash);
      t = t->left;
      end = mid;
    }
    // right path
    else {
      ret.push_back(t->left->hash);
      t = t->right;
      start = mid;
    }
  }
  return ret;
}

void MerkleTree::printMerkleTree() {
  // print the whole tree
  if (root == nullptr)
    return;
  std::queue<MerkleNode *> current_q;
  std::queue<MerkleNode *> next_q;

  current_q.push(root);
  while (current_q.size() != 0) {
    while (current_q.size() != 0) {
      MerkleNode *r = current_q.front();
      current_q.pop();

      std::cout << r->hash << " ";

      if (r->left != nullptr)
        next_q.push(r->left);
      if (r->right != nullptr)
        next_q.push(r->right);
    }
    std::cout << std::endl;
    current_q = next_q;
    while (!next_q.empty())
      next_q.pop();
  }
}

bool MerkleTree::proveInclusion(const std::string &root_hash,
                                const int block_id, const int total_blocks,
                                const std::string &block,
                                const MerkleProof &proof) {
  // start from root
  // if ((block_id >= nr_blocks) || (id < 0)) {
  //   LOG(FATAL) << "Index exceeds bound";
  // }
  std::vector<bool> path;
  int start = 0;
  int end = total_blocks;
  while (start + 1 < end) {
    int mid = (start + end) / 2;
    // left path
    if (block_id < mid) {
      path.push_back(true);
      end = mid;
    }
    // right path
    else {
      path.push_back(false);
      start = mid;
    }
  }
  std::string s = picosha2::hash256_hex_string(block);
  for (int i = path.size() - 1; i >= 0; i--) {
    // left path
    if (path[i]) {
      s = picosha2::hash256_hex_string(s + proof[i]);
    } else {
      s = picosha2::hash256_hex_string(proof[i] + s);
    }
  }
  return !s.compare(root_hash);
}
} // namespace quail