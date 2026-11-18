//! Keyed draws (H5.3): every random number is a pure function of
//! `(universe seed, address, stream name, counter)`. There is no generator
//! object to share or advance, so the order in which anything is resolved can
//! never change a result.
//!
//! Algorithm `sse.draw.sha256.v1` (pinned; docs/adr/0004-keyed-draws.md):
//! `u64 = first 8 bytes, big-endian, of SHA-256(
//!     len(domain) ‖ "sse.draw.sha256.v1" ‖ seed[32] ‖
//!     len(address) ‖ address text ‖ len(stream) ‖ stream ‖ counter as u64 BE)`
//! where every `len` is a u64 big-endian byte count.

use crate::address::Address;
use crate::hash::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const DRAW_ALGORITHM: &str = "sse.draw.sha256.v1";

/// The universe's master seed (32 bytes). Made from any text with [`Seed::from_text`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Seed(pub crate::hash::ContentHash);

impl Seed {
    /// A seed from user text (what a GM types). SHA-256 of "sse.seed.v1:" + text.
    pub fn from_text(text: &str) -> Self {
        let mut h = Hasher::new("sse.seed.v1");
        h.update(text.as_bytes());
        Seed(h.finish())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamError(pub String);
impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "stream name {:?} must look like `name.parts.v<N>`",
            self.0
        )
    }
}
impl std::error::Error for StreamError {}

/// A stream name such as `planets.count.v1`. The trailing version lets a pack
/// change an algorithm without silently changing old universes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Stream(String);

impl Stream {
    pub fn new(s: &str) -> Result<Self, StreamError> {
        let parts: Vec<&str> = s.split('.').collect();
        let ok_part = |p: &&str| {
            !p.is_empty()
                && p.bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
        };
        let ver = parts
            .last()
            .and_then(|v| v.strip_prefix('v'))
            .filter(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()));
        if parts.len() < 2 || ver.is_none() || !parts.iter().all(ok_part) {
            return Err(StreamError(s.into()));
        }
        Ok(Stream(s.into()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for Stream {
    type Error = StreamError;
    fn try_from(s: String) -> Result<Self, StreamError> {
        Stream::new(&s)
    }
}
impl From<Stream> for String {
    fn from(s: Stream) -> String {
        s.0
    }
}

/// The raw draw.
pub fn draw(seed: &Seed, address: &Address, stream: &Stream, counter: u64) -> u64 {
    let a = address.to_string();
    let mut h = Hasher::new(DRAW_ALGORITHM);
    h.update(&seed.0 .0)
        .update(&(a.len() as u64).to_be_bytes())
        .update(a.as_bytes())
        .update(&(stream.0.len() as u64).to_be_bytes())
        .update(stream.0.as_bytes())
        .update(&counter.to_be_bytes());
    let d = h.finish().0;
    u64::from_be_bytes(d[..8].try_into().unwrap())
}

/// Draws for one (seed, address, stream). Indexed, not stateful: `unit(5)`
/// is the same whether or not `unit(0..5)` were ever asked for.
#[derive(Clone, Debug)]
pub struct Draws {
    seed: Seed,
    address: Address,
    stream: Stream,
}

impl Draws {
    pub fn new(seed: Seed, address: Address, stream: Stream) -> Self {
        Draws {
            seed,
            address,
            stream,
        }
    }
    pub fn u64(&self, i: u64) -> u64 {
        draw(&self.seed, &self.address, &self.stream, i)
    }
    /// Uniform in [0, 1), exact from the top 53 bits.
    pub fn unit(&self, i: u64) -> f64 {
        (self.u64(i) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
    /// Uniform in [lo, hi).
    pub fn uniform(&self, i: u64, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit(i)
    }
    /// Uniform integer in [0, n) without modulo bias (Lemire's method; the
    /// rare retry uses counters `i + k * 2^32`, still a pure function of `i`).
    pub fn below(&self, i: u64, n: u64) -> u64 {
        assert!(n > 0);
        let mut k: u64 = 0;
        loop {
            let x = draw(
                &self.seed,
                &self.address,
                &self.stream,
                i.wrapping_add(k << 32),
            );
            let m = (x as u128) * (n as u128);
            let low = m as u64;
            if low >= n.wrapping_neg() % n {
                return (m >> 64) as u64;
            }
            k += 1;
        }
    }
    /// Standard normal from draws `2i` and `2i+1` (Box-Muller, pinned maths).
    pub fn normal(&self, i: u64) -> f64 {
        let u1 = 1.0 - self.unit(2 * i); // (0, 1]
        let u2 = self.unit(2 * i + 1);
        crate::math::sqrt(-2.0 * crate::math::ln(u1)) * crate::math::cos(crate::math::TAU * u2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn addr(n: u64) -> Address {
        Address::universe("t").unwrap().child("sys", n).unwrap()
    }

    #[test]
    fn golden_values_are_stable() {
        let d = Draws::new(
            Seed::from_text("tarn"),
            addr(7),
            Stream::new("planets.count.v1").unwrap(),
        );
        // If these change, every universe changes: never edit without a new algorithm version.
        assert_eq!(d.u64(0), 0x861c_eb1d_ba9a_7952, "{:#x}", d.u64(0));
        assert_eq!(d.below(1, 10), 0);
        assert_eq!(d.below(2, 1_000_000), d.below(2, 1_000_000));
    }

    #[test]
    fn stream_names_need_versions() {
        for ok in ["planets.count.v1", "a.v10", "stars.decor_2.v3"] {
            assert!(Stream::new(ok).is_ok(), "{ok}");
        }
        for bad in [
            "planets",
            "planets.count",
            "planets.v",
            "Planets.v1",
            "a..v1",
            "a.v1x",
            ".v1",
        ] {
            assert!(Stream::new(bad).is_err(), "{bad}");
        }
    }

    proptest! {
        #[test]
        fn independent_of_call_order(keys in proptest::collection::vec((0u64..50, 0u64..3, 0u64..100), 1..40), rot in 0usize..40) {
            let seed = Seed::from_text("order");
            let streams = ["a.v1", "b.v1", "c.v2"].map(|s| Stream::new(s).unwrap());
            let f = |(a, s, c): (u64, u64, u64)| draw(&seed, &addr(a), &streams[s as usize], c);
            let forward: Vec<u64> = keys.iter().copied().map(f).collect();
            let mut shuffled: Vec<(usize, (u64, u64, u64))> = keys.iter().copied().enumerate().collect();
            let len = shuffled.len();
            shuffled.rotate_left(rot % len);
            shuffled.reverse();
            for (i, k) in shuffled {
                prop_assert_eq!(f(k), forward[i]);
            }
        }

        #[test]
        fn adding_a_stream_changes_nothing_else(c in 0u64..1000) {
            let seed = Seed::from_text("x");
            let planets = Draws::new(seed, addr(1), Stream::new("planets.count.v1").unwrap());
            let before = planets.u64(c);
            let _decor = Draws::new(seed, addr(1), Stream::new("stars.decor.v1").unwrap()).u64(c);
            prop_assert_eq!(planets.u64(c), before);
        }

        #[test]
        fn ranges_hold(i in 0u64..10_000, n in 1u64..1_000_000) {
            let d = Draws::new(Seed::from_text("r"), addr(2), Stream::new("r.v1").unwrap());
            let u = d.unit(i);
            prop_assert!((0.0..1.0).contains(&u));
            prop_assert!(d.below(i, n) < n);
            prop_assert!(d.normal(i).is_finite());
        }
    }
}
