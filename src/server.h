#pragma once
#include <string>
#include <vector>
namespace quail {
class Server {
public:
  void uploadCommitment(const std::string &pi, const std::string &ti);
  void uploadCipher(const std::string &ci, const std::string &ti);
  // void downloadSLeafNode(const std::vector<int> &index);
  // void downloadSNonLeafNode(const std::vector<int> &index);
};

} // namespace quail