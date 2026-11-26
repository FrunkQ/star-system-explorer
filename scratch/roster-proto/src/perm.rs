//! A keyed bijection on [0, n) that is cheap both ways: a balanced Feistel
//! network on the smallest even bit width holding n, with cycle-walking.
//! Forward and inverse each cost a few rounds of integer mixing, at any n up
//! to 2^62, and never visit other members.

#[inline]
pub fn mix(mut z: u64) -> u64 {
    // splitmix64 finaliser: integer only, so bit-identical on every target.
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const ROUNDS: u64 = 6;

#[derive(Clone, Copy, Debug)]
pub struct Perm {
    n: u64,
    half: u32,
    mask: u64,
    key: u64,
}

impl Perm {
    pub fn new(n: u64, key: u64) -> Self {
        assert!(n > 0 && n < (1u64 << 62));
        let bits = 64 - (n - 1).max(1).leading_zeros();
        let half = bits.div_ceil(2).max(1);
        Perm {
            n,
            half,
            mask: (1u64 << half) - 1,
            key,
        }
    }
    #[inline]
    fn f(&self, r: u64, x: u64) -> u64 {
        mix(self.key ^ mix(r.wrapping_mul(0x1000_0000_01B3) ^ x)) & self.mask
    }
    #[inline]
    fn enc(&self, x: u64) -> u64 {
        let (mut l, mut r) = (x >> self.half, x & self.mask);
        for i in 0..ROUNDS {
            let t = l ^ self.f(i, r);
            l = r;
            r = t;
        }
        (l << self.half) | r
    }
    #[inline]
    fn dec(&self, y: u64) -> u64 {
        let (mut l, mut r) = (y >> self.half, y & self.mask);
        for i in (0..ROUNDS).rev() {
            let t = r ^ self.f(i, l);
            r = l;
            l = t;
        }
        (l << self.half) | r
    }
    /// Position to member.
    pub fn at(&self, p: u64) -> u64 {
        debug_assert!(p < self.n);
        let mut x = self.enc(p);
        while x >= self.n {
            x = self.enc(x);
        }
        x
    }
    /// Member to position.
    pub fn index_of(&self, k: u64) -> u64 {
        debug_assert!(k < self.n);
        let mut x = self.dec(k);
        while x >= self.n {
            x = self.dec(x);
        }
        x
    }
}

/// Uniform [0, 1) from a mixed counter.
#[inline]
pub fn unit(key: u64, i: u64) -> f64 {
    (mix(key ^ mix(i)) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_bijection_both_ways() {
        for n in [1u64, 2, 3, 7, 10, 64, 1000, 4097] {
            let p = Perm::new(n, 42 + n);
            let mut seen = vec![false; n as usize];
            for i in 0..n {
                let k = p.at(i);
                assert!(!seen[k as usize]);
                seen[k as usize] = true;
                assert_eq!(p.index_of(k), i);
            }
        }
        let big = Perm::new(8_560_000_000, 7);
        for i in [0u64, 1, 12_345_678, 8_559_999_999] {
            assert_eq!(big.index_of(big.at(i)), i);
        }
    }
}
