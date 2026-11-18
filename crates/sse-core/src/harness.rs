//! Cross-device hash harness (K1, handoff section 12 experiment 1).
//!
//! Generates fixed case sets from integer-only pseudo-random inputs, runs them
//! through pinned maths, canonical encoding and SHA-256, and folds every result
//! into per-category digests. The same build run natively, in Node and in
//! every browser must print identical digests; the golden values live in
//! `tests/golden/harness.json`.

use crate::cbor::{self, Value};
use crate::hash::{ContentHash, Hasher};
use crate::math as m;
use serde::Serialize;
use std::collections::BTreeMap;

/// Bump when the case generation changes (that changes every digest).
pub const HARNESS_VERSION: &str = "sse.harness.v1";
pub const MATH_CASES: u32 = 10_000;
pub const ENCODING_CASES: u32 = 10_000;
pub const SHA_CASES: u32 = 1_000;
pub const DRAW_CASES: u32 = 10_000;

/// SplitMix64: integer-only, used only to make test inputs.
struct Mix(u64);
impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// Uniform in [0,1) from the top 53 bits (exact, no rounding).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
    /// An input double drawn from a mix of regimes that stress libm.
    fn input(&mut self) -> f64 {
        match self.next() % 8 {
            0 => self.unit() * 20.0 - 10.0,
            1 => self.unit() * 2.0 - 1.0,
            2 => (self.unit() * 2.0 - 1.0) * 1e6,
            3 => (self.unit() * 2.0 - 1.0) * 1e-6,
            4 => f64::from_bits(self.next()), // any bit pattern, incl. NaN/inf/subnormal
            5 => [
                0.0,
                -0.0,
                1.0,
                -1.0,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::MIN_POSITIVE,
                5e-324,
                m::PI,
                0.5,
            ][(self.next() % 10) as usize],
            6 => self.unit() * 700.0,
            _ => (self.next() % 2001) as f64 * 0.25 - 250.0,
        }
    }
}

/// Bits of a result, with every NaN folded to one value (NaN payloads and
/// signs are not specified identically across targets; the kernel never lets
/// a NaN out of an atom, so only "is NaN" matters).
fn bits(x: f64) -> u64 {
    if x.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        x.to_bits()
    }
}

/// A Kepler solve with a fixed iteration count: representative kernel work.
fn kepler(mean_anomaly: f64, e: f64) -> f64 {
    let mut ea = if e < 0.8 { mean_anomaly } else { m::PI };
    for _ in 0..12 {
        ea -= (ea - e * m::sin(ea) - mean_anomaly) / (1.0 - e * m::cos(ea));
    }
    ea
}

type Unary = (&'static str, fn(f64) -> f64);
const UNARY: &[Unary] = &[
    ("sin", m::sin),
    ("cos", m::cos),
    ("tan", m::tan),
    ("asin", m::asin),
    ("acos", m::acos),
    ("atan", m::atan),
    ("exp", m::exp),
    ("exp2", m::exp2),
    ("expm1", m::expm1),
    ("ln", m::ln),
    ("log2", m::log2),
    ("log10", m::log10),
    ("log1p", m::log1p),
    ("sqrt", m::sqrt),
    ("cbrt", m::cbrt),
    ("sinh", m::sinh),
    ("cosh", m::cosh),
    ("tanh", m::tanh),
    ("floor", m::floor),
    ("round", m::round),
];

type Binary = (&'static str, fn(f64, f64) -> f64);
const BINARY: &[Binary] = &[
    ("add", |a, b| a + b),
    ("sub", |a, b| a - b),
    ("mul", |a, b| a * b),
    ("div", |a, b| a / b),
    ("pow", m::pow),
    ("atan2", m::atan2),
    ("hypot", m::hypot),
    ("fmod", m::fmod),
    ("mul_add_unfused", |a, b| a * b + a),
];

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct HarnessReport {
    pub version: String,
    pub math_cases: u32,
    pub encoding_cases: u32,
    pub sha_cases: u32,
    pub draw_cases: u32,
    /// Per-category digests (hex SHA-256).
    pub digests: BTreeMap<String, String>,
    /// Encoding round-trip failures; must be 0.
    pub encoding_failures: u32,
    /// Digest over all category digests.
    pub overall: String,
}

fn random_value(r: &mut Mix, depth: u32) -> Value {
    let k = r.next() % if depth >= 4 { 6 } else { 8 };
    match k {
        0 => Value::Null,
        1 => Value::Bool(r.next() & 1 == 1),
        2 => {
            let raw = r.next();
            Value::Int(match r.next() % 4 {
                0 => (raw % 30) as i128 - 5,
                1 => raw as i128,
                2 => -(raw as i128) - 1,
                _ => (raw as i64) as i128,
            })
        }
        3 => {
            let f = r.input();
            Value::Float(if f.is_nan() { 0.0 } else { f })
        }
        4 => Value::Bytes((0..r.next() % 40).map(|_| r.next() as u8).collect()),
        5 => Value::Text(random_text(r)),
        6 => Value::Array(
            (0..r.next() % 6)
                .map(|_| random_value(r, depth + 1))
                .collect(),
        ),
        _ => {
            let mut map = BTreeMap::new();
            for _ in 0..r.next() % 6 {
                map.insert(random_text(r), random_value(r, depth + 1));
            }
            Value::Map(map)
        }
    }
}

fn random_text(r: &mut Mix) -> String {
    const ALPHABET: &[&str] = &["a", "b", "z", "A", "_", ".", "0", "é", "星", "🪐", " ", "/"];
    (0..r.next() % 12)
        .map(|_| ALPHABET[(r.next() % ALPHABET.len() as u64) as usize])
        .collect()
}

pub fn run() -> HarnessReport {
    let mut digests = BTreeMap::new();

    // --- maths ---
    let mut per_fn: Vec<Hasher> = UNARY
        .iter()
        .map(|(n, _)| Hasher::new(&format!("{HARNESS_VERSION}/math/{n}")))
        .chain(
            BINARY
                .iter()
                .map(|(n, _)| Hasher::new(&format!("{HARNESS_VERSION}/math/{n}"))),
        )
        .collect();
    let mut kep = Hasher::new(&format!("{HARNESS_VERSION}/math/kepler"));
    let mut r = Mix(0x5353_4534_0000_0001);
    for _ in 0..MATH_CASES {
        let (x, y) = (r.input(), r.input());
        for (i, (_, f)) in UNARY.iter().enumerate() {
            per_fn[i].update(&bits(f(x)).to_be_bytes());
        }
        for (j, (_, f)) in BINARY.iter().enumerate() {
            per_fn[UNARY.len() + j].update(&bits(f(x, y)).to_be_bytes());
        }
        let e = r.unit() * 0.99;
        kep.update(&bits(kepler(r.unit() * m::TAU, e)).to_be_bytes());
    }
    let names = UNARY
        .iter()
        .map(|(n, _)| *n)
        .chain(BINARY.iter().map(|(n, _)| *n));
    for (n, h) in names.zip(per_fn) {
        digests.insert(format!("math.{n}"), h.finish().hex());
    }
    digests.insert("math.kepler".into(), kep.finish().hex());

    // --- canonical encoding ---
    let mut enc = Hasher::new(&format!("{HARNESS_VERSION}/encoding"));
    let mut failures = 0;
    let mut r = Mix(0x5353_4534_0000_0002);
    for _ in 0..ENCODING_CASES {
        let v = random_value(&mut r, 0);
        match cbor::encode(&v) {
            Ok(b) => {
                let ok = cbor::decode(&b).map(|d| d == v).unwrap_or(false)
                    && cbor::encode(&cbor::decode(&b).unwrap()).ok().as_deref() == Some(&b[..]);
                if !ok {
                    failures += 1;
                }
                enc.update(&(b.len() as u64).to_be_bytes()).update(&b);
            }
            Err(_) => failures += 1,
        }
    }
    digests.insert("encoding.cbor".into(), enc.finish().hex());

    // --- SHA-256 itself ---
    let mut sha = Hasher::new(&format!("{HARNESS_VERSION}/sha256"));
    let mut r = Mix(0x5353_4534_0000_0003);
    for i in 0..SHA_CASES {
        let data: Vec<u8> = (0..i).map(|_| r.next() as u8).collect();
        sha.update(&ContentHash::of_bytes(&data).0);
    }
    digests.insert("hash.sha256".into(), sha.finish().hex());

    // --- keyed draws (added in K2) ---
    let mut dr = Hasher::new(&format!("{HARNESS_VERSION}/draws"));
    let seed = crate::draw::Seed::from_text("sse4 harness");
    let streams = ["planets.count.v1", "stars.mass.v1", "moons.orbit.v2"]
        .map(|s| crate::draw::Stream::new(s).unwrap());
    let root = crate::address::Address::universe("harness").unwrap();
    for i in 0..DRAW_CASES as u64 {
        let a = root
            .child("sys", i % 97)
            .unwrap()
            .child("body", i % 13)
            .unwrap();
        let d = crate::draw::Draws::new(seed, a, streams[(i % 3) as usize].clone());
        dr.update(&d.u64(i).to_be_bytes())
            .update(&bits(d.unit(i)).to_be_bytes())
            .update(&bits(d.normal(i)).to_be_bytes())
            .update(&d.below(i, 1 + i).to_be_bytes());
    }
    digests.insert("draws.keyed".into(), dr.finish().hex());

    let mut all = Hasher::new(&format!("{HARNESS_VERSION}/overall"));
    for (k, v) in &digests {
        all.update(k.as_bytes())
            .update(b"=")
            .update(v.as_bytes())
            .update(b";");
    }
    HarnessReport {
        version: HARNESS_VERSION.into(),
        math_cases: MATH_CASES,
        encoding_cases: ENCODING_CASES,
        sha_cases: SHA_CASES,
        draw_cases: DRAW_CASES,
        digests,
        encoding_failures: failures,
        overall: all.finish().hex(),
    }
}

/// The report as canonical JSON-ish text (stable key order) for printing.
pub fn report_json(r: &HarnessReport) -> String {
    let mut s = String::from("{\n");
    s += &format!("  \"version\": \"{}\",\n", r.version);
    s += &format!(
        "  \"math_cases\": {},\n  \"encoding_cases\": {},\n  \"sha_cases\": {},\n  \"draw_cases\": {},\n",
        r.math_cases, r.encoding_cases, r.sha_cases, r.draw_cases
    );
    s += &format!("  \"encoding_failures\": {},\n", r.encoding_failures);
    s += "  \"digests\": {\n";
    let n = r.digests.len();
    for (i, (k, v)) in r.digests.iter().enumerate() {
        s += &format!("    \"{k}\": \"{v}\"{}\n", if i + 1 < n { "," } else { "" });
    }
    s += "  },\n";
    s += &format!("  \"overall\": \"{}\"\n}}\n", r.overall);
    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn harness_matches_golden() {
        let r = super::run();
        assert_eq!(r.encoding_failures, 0);
        let golden = include_str!("../../../tests/golden/harness.json");
        assert!(super::report_json(&r) == golden, "harness digests changed; if intended, regenerate with `cargo run -p sse-cli -- harness > tests/golden/harness.json` and record why");
    }
}
