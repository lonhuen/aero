from ntt import NTT
from crt import crt
from player import Player
from shamir import Shamir
from shamir import Q
from shamir import T
import random

ntt = NTT()
N = 4096
modulus = [0xffffee001, 0xffffc4001, 0x1ffffe0001]
primitive_root = [0x1720701, 0x1baa271, 0x839eb4]  # 2N-primitive root
M = modulus[0] * modulus[1] * modulus[2]
root = crt(modulus, primitive_root)
inv_root = crt(modulus, [ntt.modInv(
    primitive_root[i], modulus[i]) for i in range(3)])
inv_N = crt(modulus, [ntt.modInv(N, modulus[i]) for i in range(3)])

share = Shamir(1).get_share()
print(share)
new_share = [(share[i] * share[i] - share[i]) % Q for i in range(len(share))]
x = Shamir.from_share(new_share, T*2)
print(x.reveal())

# verify ntt(sk) * ntt(v) + ntt(noise)
# sk = [1 for i in range(N)]
# v = [0 for i in range(N)]
# v[0] = 1
#
# ntt_sk = ntt.ntt(sk, M, N, root)
# ntt_v = ntt.ntt(v, M, N, root)
#
# ntt_sk_shares = [[] for i in range(20)]
# for i in range(N):
#    share = Shamir(ntt_sk[i]).get_share()
#    for j in range(20):
#        ntt_sk_shares[j].append(share[j])
#
# ntt_bit_shares = ntt_sk_shares.copy()
#
# players = []
# for i in range(20):
#    players.append(Player(ntt_sk_shares[i], ntt_bit_shares[i], ntt_v, i))
#
# result_shares = [players[i].get_result_share() for i in range(20)]
#
# result_shamir = []
# for i in range(4096):
#    slot_shamir_share = []
#    for j in range(20):
#        slot_shamir_share.append(result_shares[j][i])
#
#    s = Shamir.from_share(slot_shamir_share, 6).reveal()
#    result_shamir.append(s)
#
# print(ntt.intt(result_shamir, M, N, inv_root, inv_N))

# verify local ntt
# first convert into shamir_shares
# run ntt on each local share
# finally reconstruct and run intt
# noise = [1 for i in range(N)]
#
# noise_shares = [[] for i in range(20)]
# for i in range(N):
#     share = Shamir(noise[i]).get_share()
#     for j in range(20):
#         noise_shares[j].append(share[j])
#
# # local ntt
# for j in range(20):
#     noise_shares[j] = ntt.ntt(noise_shares[j], M, N, root)
#
# result_shamir = []
# for i in range(4096):
#     slot_shamir_share = []
#     for j in range(20):
#         slot_shamir_share.append(noise_shares[j][i])
#
#     s = Shamir.from_share(slot_shamir_share, 6).reveal()
#     result_shamir.append(s)
#
# print(ntt.intt(result_shamir, M, N, inv_root, inv_N))
#

# verify test b2 - b*1

# noise = [random.randint(0, 1) for i in range(N)]
# identity = [1 for i in range(N)]
# noise_shares = [[] for i in range(20)]
# for i in range(N):
#     share = Shamir(noise[i]).get_share()
#     for j in range(20):
#         noise_shares[j].append(share[j])
#
# id_shares = [[] for i in range(20)]
# for i in range(N):
#     share = Shamir(identity[i]).get_share()
#     for j in range(20):
#         id_shares[j].append(share[j])
#
# # local computation
# result_shares = [[0 for j in range(4096)] for i in range(20)]
# for j in range(20):
#     for i in range(4096):
#         result_shares[j][i] = noise_shares[j][i] * \
#             noise_shares[j][i] - noise_shares[j][i] * id_shares[j][i]
#
# result_shamir = []
# for i in range(4096):
#     slot_shamir_share = []
#     for j in range(20):
#         slot_shamir_share.append(result_shares[j][i])
#
#     s = Shamir.from_share(slot_shamir_share, 12).reveal()
#     result_shamir.append(s)
#
# print(result_shamir)
#
