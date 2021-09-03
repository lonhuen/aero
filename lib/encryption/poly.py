def polyMul(a, b):
    c = [0] * 4096
    for i in range(4096):
        for j in range(4096):
            # a[i] x^i * b[j] x^j
            if i+j >= 4096:
                c[i+j-4096] -= a[i] * b[j]
            else:
                c[i+j] += a[i] * b[j]
    for i in range(4096):
        c[i] = c[i]
    return c


def polyAdd(a, b):
    c = [0] * 4096
    for i in range(4096):
        c[i] = (a[i] + b[i])
    return c
