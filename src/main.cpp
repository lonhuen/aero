#include "merkle.h"
#include "picosha2.h"
#include "seal/seal.h"
#include "utils/log_utils.h"
#include <iostream>
#include <string>
#include <vector>

using namespace seal;

int main(int argc, char **argv) {
  quail::LogUtils::init(argv[0]);
  // EncryptionParameters parms(scheme_type::bfv);
  // size_t poly_modulus_degree = 4096;
  // parms.set_poly_modulus_degree(poly_modulus_degree);
  // parms.set_coeff_modulus(CoeffModulus::BFVDefault(poly_modulus_degree));
  // parms.set_plain_modulus(1024);
  // SEALContext context(parms);
  // KeyGenerator keygen(context);
  // SecretKey secret_key = keygen.secret_key();
  // PublicKey public_key;
  // keygen.create_public_key(public_key);
  // Encryptor encryptor(context, public_key);
  // Evaluator evaluator(context);
  // Decryptor decryptor(context, secret_key);
  // int x = 6;
  // string s("5x^1 + 6");
  // Plaintext x_plain(s);
  // cout << "Express x = " + to_string(x) + " as a plaintext polynomial 0x" +
  //             x_plain.to_string() + "."
  //      << endl;
  // Ciphertext x_encrypted;
  // encryptor.encrypt(x_plain, x_encrypted);
  // Plaintext x_decrypted;
  // cout << "    + decryption of x_encrypted: ";
  // decryptor.decrypt(x_encrypted, x_decrypted);
  // cout << "0x" << x_decrypted.to_string() << " ...... Correct." << endl;
  // LOG(INFO) << "hello world";
  // std::cout << "hello world" << std::endl;
  std::vector<std::string> blocks;
  blocks.push_back("1");
  blocks.push_back("2");
  blocks.push_back("3");
  blocks.push_back("4");
  blocks.push_back("5");
  blocks.push_back("6");
  quail::MerkleTree mt(blocks);
  mt.printMerkleTree();
  auto proof = mt.proofOfInclusion(0);
  std::cout << " -------- " << std::endl;
  std::cout << mt.leaf[0]->hash << std::endl;
  std::cout << " -------- " << std::endl;
  for (int i = 0; i < proof.size(); i++) {
    std::cout << proof[i].second << std::endl;
  }
  if (mt.proveInclusion(mt.root->hash, std::string("1"), proof)) {
    std::cout << "yeah" << std::endl;
  }
}
