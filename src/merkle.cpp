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
  while (level_nodes.size() > 1) {
    MerkleNode *l = level_nodes.front();
    level_nodes.pop();
    MerkleNode *r = level_nodes.front();
    level_nodes.pop();
    MerkleNode *t =
        new MerkleNode(picosha2::hash256_hex_string(l->hash + r->hash));
    t->left = l;
    t->right = r;
    l->parent = t;
    r->parent = t;
    l->pair = r;
    r->pair = l;
    level_nodes.push(t);
  }
  this->root = level_nodes.front();
}

std::vector<std::pair<bool, std::string>> MerkleTree::proofOfInclusion(int id) {
  if (id >= nr_blocks) {
    LOG(FATAL) << "Index exceeds bound";
  }
  std::vector<std::pair<bool, std::string>> ret;
  MerkleNode *ptr = leaf[id];
  while (ptr != root) {
    bool pair_left = (ptr->pair == ptr->parent->left);
    ret.push_back(std::make_pair(pair_left, ptr->pair->hash));
    ptr = ptr->parent;
  }
  return ret;
} // namespace quail

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

bool MerkleTree::proveInclusion(
    const std::string &root_hash, const std::string &block,
    const std::vector<std::pair<bool, std::string>> &proof) {

  std::string t = picosha2::hash256_hex_string(block);
  for (int i = 0; i < proof.size(); i++) {
    // pair is left
    if (proof[i].first) {
      t = picosha2::hash256_hex_string(proof[i].second + t);
    } else {
      t = picosha2::hash256_hex_string(t + proof[i].second);
    }
  }
  return t == root_hash;
}

} // namespace quail