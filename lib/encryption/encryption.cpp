// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

#include <algorithm>
#include <chrono>
#include <cstddef>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <memory>
#include <mutex>
#include <numeric>
#include <random>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#include "seal/seal.h"

using namespace std;
using namespace seal;

int main(int argc, char* argv[]) {
  EncryptionParameters parms(scheme_type::bfv);

  size_t poly_modulus_degree = 4096;
  parms.set_poly_modulus_degree(poly_modulus_degree);

  parms.set_coeff_modulus(CoeffModulus::BFVDefault(poly_modulus_degree));

  parms.set_plain_modulus(256 * 500);

  SEALContext context(parms);

  KeyGenerator keygen(context);
  SecretKey secret_key = keygen.secret_key();
  PublicKey public_key;
  keygen.create_public_key(public_key);

  Encryptor encryptor(context, public_key);

  Evaluator evaluator(context);

  Decryptor decryptor(context, secret_key);

  int x = 0;
  Plaintext x_plain(to_string(x));

  Ciphertext x_encrypted;

  encryptor.encrypt(x_plain, x_encrypted);


   for (int i = 0; i < parms.coeff_modulus().size(); i++) {
    for (int j = 0; j < public_key.data().size(); j++) {
      cout << "lonhh_data c_" << j << " " << i << " [4096]" << endl;
      for (int lonhh_i = 0; lonhh_i < 4096; lonhh_i++) {
        try {
          cout << std::hex
               << x_encrypted[j * parms.poly_modulus_degree() *
                                  parms.coeff_modulus().size() +
                              i * parms.poly_modulus_degree() + lonhh_i]
               << ", ";
        } catch (exception e) {
          cout << "i, j, lonhh_i " << i << " " << j << " " << lonhh_i << endl;
          exit(1);
        }
      }
      cout << endl;
    }
  }
}
