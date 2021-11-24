use rand::{thread_rng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};

use super::NUM_DIMENSION;
//constexpr double seal_he_std_parms_error_std_dev = 3.2;
//constexpr double noise_standard_deviation = seal_he_std_parms_error_std_dev;
//constexpr double noise_distribution_width_multiplier = 6;
//constexpr double noise_max_deviation = noise_standard_deviation * noise_distribution_width_multiplier;

pub const STDDEV: f64 = 3.2;
pub const MAXSTDDEV: f64 = 19.2;

/// sample the offset one from {0,1,2} directly
pub fn sample_ternary() -> Vec<i128> {
    //let mut rng = SeedableRng::from_entropy();
    let mut rng = rand::rngs::StdRng::from_entropy();
    (0..NUM_DIMENSION)
        .map(|_| rng.gen_range(0i128..3i128))
        .collect::<Vec<i128>>()
}

/// Sample a polynomial with Gaussian coefficients+offset
pub fn sample_gaussian() -> Vec<i128> {
    let normal = Normal::new(0.0, STDDEV).unwrap();
    let mut rng = thread_rng();

    (0..NUM_DIMENSION)
        .map(|_| {
            let tmp = normal.sample(&mut rng);
            if tmp < -MAXSTDDEV {
                0 as i128
            } else if tmp > MAXSTDDEV {
                2 * MAXSTDDEV as i128
            } else {
                (tmp + MAXSTDDEV) as i128
            }
        })
        .collect::<Vec<i128>>()
}
