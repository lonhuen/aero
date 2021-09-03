import sys
from crt import *
from ntt import *
from poly import *

ntt = NTT()
modulus = [0xffffee001, 0xffffc4001, 0x1ffffe0001]
primitive_root = [0x1720701, 0x1baa271, 0x839eb4]
N = 4096

total_M = modulus[0]*modulus[1]*modulus[2]
root = crt(modulus, primitive_root)
inv_root = crt(modulus, [ntt.modInv(
    primitive_root[i], modulus[i]) for i in range(3)])
inv_N = crt(modulus, [ntt.modInv(N, modulus[i]) for i in range(3)])

in_fpath = sys.argv[1]
out_fpath = sys.argv[2]

# read the raw output from seal
# ['r', 'ntt_r', 'ntt_pk_0', 'ntt_pk_r_0', 'ntt_pk_1', 'ntt_pk_r_1', 'e_0', 'pk_r_e_0', 'e_1', 'pk_r_e_1', 'c_0', 'c_1']
with open(in_fpath, "r") as f:
    m = {}
    while True:
        l = f.readline()
        if not l:
            break
        if "lonhh_data" in l:
            variable_name = l.lower().split(' ')[1]
            coeff_idx = int(l.lower().split(' ')[2])
            if variable_name not in m:
                m[variable_name] = [[], [], []]
            # read 4096
            l = f.readline().strip()
            m[variable_name][coeff_idx] = [
                int(e, 16) for e in l.split(',') if e != '']

# M = 0xffffee001
# r = 0x1720701

# change to normal order (seal outputs in bit reversed order)
for k in m.keys():
    if 'ntt' in k:
        for i in range(3):
            m[k][i] = ntt.orderReverse(m[k][i], N.bit_length()-1)

# compute pk_0 and pk_1 from ntt
ntt_pk_0 = [crt(modulus, [m['ntt_pk_0'][i][j] for i in range(3)])
            for j in range(N)]
ntt_pk_1 = [crt(modulus, [m['ntt_pk_1'][i][j] for i in range(3)])
            for j in range(N)]
# primitive_root = [0x1720701, 0x1baa271, 0x839eb4]
# inv_root = [ntt.modInv(primitive_root[i], modulus[i]) for i in range(3)]
#inv_N = [ntt.modInv(N, modulus[i]) for i in range(3)]

pk_0 = ntt.intt(ntt_pk_0, total_M, N,
                inv_root, inv_N)
pk_1 = ntt.intt(ntt_pk_1, total_M, N,
                inv_root, inv_N)
r = [crt(modulus, [m['r'][i][j] for i in range(3)])
     for j in range(N)]
e_0 = [crt(modulus, [m['e_0'][i][j] for i in range(3)])
       for j in range(N)]
e_1 = [crt(modulus, [m['e_1'][i][j] for i in range(3)])
       for j in range(N)]
c_0 = [crt(modulus, [m['c_0'][i][j] for i in range(3)])
       for j in range(N)]
c_1 = [crt(modulus, [m['c_1'][i][j] for i in range(3)])
       for j in range(N)]

# update r,e  and c_0 first
# pk_0 * (r+1) + (e_0+19) = c_0'
M = total_M
for i in range(4096):
    r[i] = (r[i] + 1) % M
    e_0[i] = (e_0[i] + 19) % M
    e_1[i] = (e_1[i] + 19) % M
new_c_0 = [x % M for x in polyAdd(polyMul(pk_0, r), e_0)]
new_c_1 = [x % M for x in polyAdd(polyMul(pk_1, r), e_1)]

# now compute pk * r + e = c + delta_q * M
delta_q_0 = [0] * 4096
delta_q_1 = [0] * 4096
new_c_0_raw = polyAdd(polyMul(pk_0, r), e_0)
new_c_1_raw = polyAdd(polyMul(pk_1, r), e_1)
for i in range(4096):
    td = (new_c_0_raw[i] - new_c_0[i])
    if td % M != 0:
        print("not multiple of modulus")
        break
    delta_q_0[i] = td // M
    td = (new_c_1_raw[i] - new_c_1[i])
    if td % M != 0:
        print("not multiple of modulus")
        break
    delta_q_1[i] = td // M


def writeOutput(f, s, l):
    f.write(s)
    f.write(' ')
    f.write(" ".join([str(x) for x in l]))
    f.write('\n')


with open(out_fpath, "w") as f:
    writeOutput(f, 'r', r)
    writeOutput(f, 'pk_0', pk_0)
    writeOutput(f, 'pk_1', pk_1)
    writeOutput(f, 'c_0', new_c_0)
    writeOutput(f, 'c_1', new_c_1)
    writeOutput(f, 'e_0', e_0)
    writeOutput(f, 'e_1', e_1)
    writeOutput(f, 'delta_0', delta_q_0)
    writeOutput(f, 'delta_1', delta_q_1)
