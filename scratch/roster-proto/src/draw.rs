//! Keyed draws for the roster's inner loops: integer mixing seeded once per
//! band from the kernel's SHA-based `Draws`, so a band costs one hash and
//! every count inside it is a pure function of (band key, counter).

use crate::perm::{mix, unit};
use sse_core::math as m;

pub fn normal(key: u64, i: u64) -> f64 {
    let u1 = 1.0 - unit(key, mix(i) ^ 1);
    let u2 = unit(key, mix(i) ^ 2);
    m::sqrt(-2.0 * m::ln(u1)) * m::cos(m::TAU * u2)
}

pub fn poisson(lam: f64, key: u64, i: u64) -> u64 {
    if lam <= 0.0 {
        return 0;
    }
    if lam < 30.0 {
        let u = unit(key, mix(i) ^ 3);
        let mut p = m::exp(-lam);
        let mut c = p;
        let mut k = 0u64;
        while u > c && k < 1000 {
            k += 1;
            p *= lam / k as f64;
            c += p;
        }
        k
    } else {
        m::round(lam + m::sqrt(lam) * normal(key, i)).max(0.0) as u64
    }
}

/// Binomial(n, p): exact below 33 trials, Poisson in either tail, normal in
/// the bulk. Deterministic, never above n.
pub fn binomial(n: u64, p: f64, key: u64, i: u64) -> u64 {
    if n == 0 || p <= 0.0 {
        return 0;
    }
    if p >= 1.0 {
        return n;
    }
    if n <= 32 {
        let base = mix(i);
        return (0..n)
            .filter(|j| unit(key, base ^ mix(*j + 100)) < p)
            .count() as u64;
    }
    let nf = n as f64;
    let mean = nf * p;
    let q = 1.0 - p;
    if mean < 30.0 {
        poisson(mean, key, i).min(n)
    } else if nf * q < 30.0 {
        n - poisson(nf * q, key, i).min(n)
    } else {
        m::round(mean + m::sqrt(mean * q) * normal(key, i)).clamp(0.0, nf) as u64
    }
}
