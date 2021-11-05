from shamir import *


class Player:
    def __init__(self, sk, noise, u, i):
        self.id = i
        self.sk = sk
        self.noise = noise
        self.u = u

    def get_result_share(self):
        result = [0 for i in range(4096)]
        for i in range(4096):
            result[i] = self.sk[i] * self.u[i] + self.noise[i]

        return result

    def get_b2_b_share(self):
        # sk is ntt(1)
        result = [0 for i in range(4096)]
        for i in range(4096):
            result[i] = self.sk[i] * self.u[i] + self.noise[i]

        return result
