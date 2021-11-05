from math import e
import math
import random
Q = 649033470896967801447398927572993
N = 20
T = 6
SHARE_POINTS = [p for p in range(1, N+1)]


def encode(x):
    return x % Q


def decode(x):
    return x if x <= Q/2 else x-Q

# using Horner's rule


def evaluate_at_point(coefs, point):
    result = 0
    for coef in reversed(coefs):
        result = (coef + point * result) % Q
    return result


def egcd(a, b):
    if a == 0:
        return (b, 0, 1)
    else:
        g, y, x = egcd(b % a, a)
        return (g, x - (b // a) * y, y)

# from http://www.ucl.ac.uk/~ucahcjm/combopt/ext_gcd_python_programs.pdf


def egcd_binary(a, b):
    u, v, s, t, r = 1, 0, 0, 1, 0
    while (a % 2 == 0) and (b % 2 == 0):
        a, b, r = a//2, b//2, r+1
    alpha, beta = a, b
    while (a % 2 == 0):
        a = a//2
        if (u % 2 == 0) and (v % 2 == 0):
            u, v = u//2, v//2
        else:
            u, v = (u + beta)//2, (v - alpha)//2
    while a != b:
        if (b % 2 == 0):
            b = b//2
            if (s % 2 == 0) and (t % 2 == 0):
                s, t = s//2, t//2
            else:
                s, t = (s + beta)//2, (t - alpha)//2
        elif b < a:
            a, b, u, v, s, t = b, a, s, t, u, v
        else:
            b, s, t = b - a, s - u, t - v
    return (2 ** r) * a, s, t


def inverse(a):
    _, b, _ = egcd_binary(a, Q)
    return b

# see https://en.wikipedia.org/wiki/Lagrange_polynomial


def lagrange_constants_for_point(points, point):
    constants = [0] * len(points)
    for i in range(len(points)):
        xi = points[i]
        num = 1
        denum = 1
        for j in range(len(points)):
            if j != i:
                xj = points[j]
                num = (num * (xj - point)) % Q
                denum = (denum * (xj - xi)) % Q
        constants[i] = (num * inverse(denum)) % Q
    return constants


def interpolate_at_point(points_values, point):
    points, values = zip(*points_values)
    constants = lagrange_constants_for_point(points, point)
    return sum(vi * ci for vi, ci in zip(values, constants)) % Q


def sample_shamir_polynomial(zero_value):
    coefs = [zero_value] + [random.randrange(Q) for _ in range(T)]
    return coefs


def shamir_share(secret):
    polynomial = sample_shamir_polynomial(secret)
    shares = [evaluate_at_point(polynomial, p) for p in SHARE_POINTS]
    return shares


def shamir_reconstruct(shares):
    polynomial = [(p, v)
                  for p, v in zip(SHARE_POINTS, shares) if v is not None]
    secret = interpolate_at_point(polynomial, 0)
    return secret


def shamir_add(x, y):
    return [(xi + yi) % Q for xi, yi in zip(x, y)]


def shamir_sub(x, y):
    return [(xi - yi) % Q for xi, yi in zip(x, y)]


def shamir_mul(x, y):
    return [(xi * yi) % Q for xi, yi in zip(x, y)]


class Shamir:

    def __init__(self, secret=None):
        self.shares = shamir_share(
            encode(secret)) if secret is not None else []
        self.degree = T

    def reveal(self):
        assert(self.degree+1 <= N)
        return decode(shamir_reconstruct(self.shares))

    def __repr__(self):
        return "Shamir(%d)" % self.reveal()

    def __add__(x, y):
        z = Shamir()
        z.shares = shamir_add(x.shares, y.shares)
        z.degree = max(x.degree, y.degree)
        return z

    def __sub__(x, y):
        z = Shamir()
        z.shares = shamir_sub(x.shares, y.shares)
        z.degree = max(x.degree, y.degree)
        return z

    def __mul__(x, y):
        z = Shamir()
        z.shares = shamir_mul(x.shares, y.shares)
        z.degree = x.degree + y.degree
        return z

    def get_share(self):
        return self.shares.copy()

    def from_share(shares, T):
        s = Shamir()
        s.shares = shares
        s.degree = T
        return s


# probability computation
# e^-fC * (3 * ef)^C/3
def compute_probability_with_threshold(f, C, t_inv):
    p = math.exp(-f*C)
    p = p * math.pow(t_inv * e * f, C/t_inv)
    return p
