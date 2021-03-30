#pragma once
#include <string>
#include <vector>
namespace quail {
class Server {
public:
  /*
    Upload commitment to the server
    @param pi the public key of the client
    @param ti the commitment of the client
  */
  void uploadCommitment(const std::string &pi, const std::string &ti);
  /*
    Upload ciphertext to the server
    @param pi the public key of the client
    @param ci the public key of the client
    @param ri the random string from the client
  */
  void uploadCiphertext(const std::string &pi, const std::string &ci,
                        const std::string &ri);
  // void downloadSLeafNode(const std::vector<int> &index);
  // void downloadSNonLeafNode(const std::vector<int> &index);
};

} // namespace quail