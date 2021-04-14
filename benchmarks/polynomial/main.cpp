#include <NTL/ZZ.h>
#include <NTL/ZZX.h>
#include <NTL/ZZ_p.h>
#include <NTL/ZZ_pX.h>
#include <iostream>

static void compute_a_vals(NTL::Vec<NTL::ZZ> &a, long p, long e) {
  NTL::ZZ p_to_e = NTL::power_ZZ(p, e);
  NTL::ZZ p_to_2e = NTL::power_ZZ(p, 2 * e);

  long len = (e - 1) * (p - 1) + 2;

  NTL::ZZ_pPush push(p_to_2e);

  NTL::ZZ_pX x_plus_1_to_p = power(NTL::ZZ_pX(NTL::INIT_MONO, 1) + 1, p);
  NTL::ZZ_pX denom =
      InvTrunc(x_plus_1_to_p - NTL::ZZ_pX(NTL::INIT_MONO, p), len);
  NTL::ZZ_pX poly = MulTrunc(x_plus_1_to_p, denom, len);
  poly *= p;

  a.SetLength(len);

  NTL::ZZ m_fac(1);
  for (long m = 2; m < p; m++) {
    m_fac = MulMod(m_fac, m, p_to_2e);
  }

  for (long m = p; m < len; m++) {
    m_fac = MulMod(m_fac, m, p_to_2e);
    NTL::ZZ c = rep(coeff(poly, m));
    NTL::ZZ d = GCD(m_fac, p_to_2e);
    if (d == 0 || d > p_to_e || c % d != 0)
      ;
    NTL::ZZ m_fac_deflated = (m_fac / d) % p_to_e;
    NTL::ZZ c_deflated = (c / d) % p_to_e;
    a[m] = MulMod(c_deflated, InvMod(m_fac_deflated, p_to_e), p_to_e);
  }
}

// This computes Chen and Han's magic polynomial G, which
// has the property that G(x) = (x mod p) (mod p^e).
// Here, (x mod p) is in the interval [0,1] if p == 2,
// and otherwise, is in the interval (-p/2, p/2).
void compute_magic_poly(NTL::ZZX &poly1, long p, long e) {
  NTL::Vec<NTL::ZZ> a;

  compute_a_vals(a, p, e);

  NTL::ZZ p_to_e = NTL::power_ZZ(p, e);
  long len = (e - 1) * (p - 1) + 2;

  NTL::ZZ_pPush push(p_to_e);

  NTL::ZZ_pX poly(0);
  NTL::ZZ_pX term(1);
  NTL::ZZ_pX X(NTL::INIT_MONO, 1);

  poly = 0;
  term = 1;

  for (long m = 0; m < p; m++) {
    term *= (X - m);
  }

  for (long m = p; m < len; m++) {
    poly += term * NTL::conv<NTL::ZZ_p>(a[m]);
    term *= (X - m);
  }

  // replace poly by poly(X+(p-1)/2) for odd p
  if (p % 2 == 1) {
    NTL::ZZ_pX poly2(0);

    for (long i = NTL::deg(poly); i >= 0; i--)
      poly2 = poly2 * (X + (p - 1) / 2) + poly[i];

    poly = poly2;
  }

  poly = X - poly;
  poly1 = NTL::conv<NTL::ZZX>(poly);
}

int main() {
  NTL::ZZX p;
  compute_magic_poly(p, 127, 3);
  for (int x = 127*127*127  - 127; x <= 127*127*127; x++) {
    NTL::ZZ sum;
    sum = 0;
    NTL::ZZ term;
    term = 1;
    for (int i = 0; i <= deg(p); i++) {
      sum = sum + p[i] * term;
      term *= x;
    }
    std::cout << sum % (127*127*127) << std::endl;
  }
  std::cout << p << std::endl;
  std::cout << deg(p) << std::endl;
  return 0;
}
