#include "merkle.h"
#include "client.h"
#include "picosha2.h"
#include "seal/seal.h"
#include "utils/log_utils.h"
#include <iostream>
#include <string>
#include <vector>
#include <typeinfo>
#include <chrono>


using namespace seal;

int main(int argc, char **argv) {
  quail::LogUtils::init(argv[0]);
  EncryptionParameters params(scheme_type::bfv);
  size_t poly_modulus_degree = 4096;
  params.set_poly_modulus_degree(poly_modulus_degree);
  params.set_coeff_modulus(CoeffModulus::BFVDefault(poly_modulus_degree));
  params.set_plain_modulus(1024);
  SEALContext context(params);
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
  auto proof = mt.proofOfInclusion(1);
  std::cout << " -------- " << std::endl;
  for (int i = 0; i < blocks.size(); i++)
    std::cout << mt.leaf[i]->hash << std::endl;
  std::cout << " -------- " << std::endl;
  for (int i = 0; i < proof.size(); i++) {
    std::cout << proof[i] << std::endl;
  }
  if (mt.proveInclusion(mt.root->hash, 1, blocks.size(), std::string("2"),
                        proof)) {
    std::cout << "yeah" << std::endl;
  }
  
  
  quail::Client c(context);
  // Measure time for local client update: 
  using std::chrono::high_resolution_clock;
  using std::chrono::duration_cast;
  using std::chrono::duration;
  using std::chrono::milliseconds;

  auto t1 = high_resolution_clock::now();
  c.localUpdate();
  auto t2 = high_resolution_clock::now();
  auto ms_int = duration_cast<milliseconds>(t2 - t1);
  std::cout << "client finished local update in " << std::to_string(ms_int.count()) << " miliseconds." << std::endl;
}
