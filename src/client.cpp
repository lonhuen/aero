#include "merkle.h"
#include "picosha2.h"
#include "utils/log_utils.h"
#include <iostream>
#include <queue>
#include "client.h"
#include <sstream>
#include "utils/helpers.h"
#include <bitset>
#include <tuple>

using namespace seal;

namespace quail {

Client::Client(const seal::SEALContext context) {
    // Initialize public key, secret key, encryptor and decryptor
    seal::KeyGenerator keygen(context);
    this->secret_key = keygen.secret_key();
    keygen.create_public_key(this->public_key);

    this->encryptor = new seal::Encryptor(context, this->public_key);
    this->decryptor = new seal::Decryptor(context, this->secret_key);
}

void Client::publishMc(const std::string &root_hash, const MerkleProof &proof) {

}

void Client::publishMs(const std::string &root_hash, const MerkleProof &proof) {
    
}

void Client::localUpdate() {
    // localUpdate updates global model with local data, sends server commitment
    
    // 1: Train Model Locally and measure time to train
    using std::chrono::high_resolution_clock;
    using std::chrono::duration_cast;
    using std::chrono::duration;
    using std::chrono::milliseconds;
    auto t1 = high_resolution_clock::now();
    trainModel();
    auto t2 = high_resolution_clock::now();
    auto ms_int = duration_cast<milliseconds>(t2 - t1);
    std::cout << "client finished model training in " << std::to_string(ms_int.count()) << " miliseconds." << std::endl;


    // 2: Encrypt data to send to server
    int randomInput = 2; // Placeholder for actual input to encrypt
    seal::Plaintext x_plain(std::to_string(randomInput));
    seal::Ciphertext x_encrypted;
    
    // Time to encrypt data
    t1 = high_resolution_clock::now();
    this->encryptor->encrypt(x_plain, x_encrypted);
    t2 = high_resolution_clock::now();
    ms_int = duration_cast<milliseconds>(t2 - t1);
    std::cout << "client finished encryption in " << std::to_string(ms_int.count()) << " miliseconds." << std::endl;
    std::cout << "    + size of freshly encrypted x: " << x_encrypted.size() << std::endl;
    std::cout << "    + noise budget in freshly encrypted x: " << decryptor->invariant_noise_budget(x_encrypted) << " bits"
        << std::endl;
    

    // 3. Generate commitment to ciphertext: Hash(r || ciphertext || public-key)
    srand(time(0));
    // Generate 128 bit random number
    std::string src_str;
    int random_num;
    for(int i = 0; i < 4; i++) {
        random_num = rand();
        std::string binary_str = std::bitset<32>(random_num).to_string();
        // std::cout << binary_str << std::endl;
        src_str = src_str + binary_str;
    }

    std::cout << "random num is: " << src_str << std::endl;
    std::cout << "length of src_str is" << src_str.size() << std::endl;

    // append ciphertext and public key to random number
    std::stringstream ct, pk;
    x_encrypted.save(ct);
    this->public_key.save(pk);
    src_str += ct.str();
    src_str += pk.str();

    // Take SHA256 hash of commitment    
    std::vector<unsigned char> hash(picosha2::k_digest_size);
    picosha2::hash256(src_str.begin(), src_str.end(), hash.begin(), hash.end());
    std::string hex_str = picosha2::bytes_to_hex_string(hash.begin(), hash.end());

    std::cout << "Hash of the commitment is: " << hex_str << std::endl;
    
    std::tuple<std::string, std::string> commitment (pk.str(),hex_str);
    // std::cout << "commitment contains: ";
    // std::cout << std::get<0>(commitment) << std::endl;
    // std::cout << std::get<1>(commitment) << '\n';

    // TODO: Send find commitment tuple to server. 
}

void Client::trainModel() {
    // Carry out one iteration of global model training using tensorflow.
}

}// namespace quail