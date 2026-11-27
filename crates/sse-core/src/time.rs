//! Dates on the one time axis (track TP, package TP1).
//!
//! A date is integers: whole seconds from the universe's origin plus
//! nanoseconds. It is never a floating-point number, because a double holds
//! 53 bits: at a star's age (about 1.5e17 s) the smallest step a double can
//! take is 32 s, and at 13.8 Gyr it is 64 s, so a position read from an
//! absolute double date can only move once a minute.
//!
//! Physics works on "seconds since my anchor". `Instant::since` does the
//! subtraction in integers first and turns only the small difference into a
//! double, so one second is always one second, at any age.
//!
//! Range: `i64` seconds reach about 292 billion years either side of the
//! origin. Conversions that could leave that range are checked and refuse
//! with a plain sentence; nothing wraps.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Seconds in one Julian year (365.25 days), as a whole number.
pub const YEAR_S: i64 = crate::sheet::YR_JULIAN as i64;
const NS_PER_S: u32 = 1_000_000_000;

/// A date: signed whole seconds (plus nanoseconds) from the universe's origin.
/// Integer, so ordering is exact and one second is always resolved.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
pub struct Instant {
    pub s: i64,
    #[serde(default)]
    pub ns: u32,
}

/// A date that would fall off the whole-second axis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutOfRange(pub String);

impl fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn refuse(what: impl fmt::Display) -> OutOfRange {
    OutOfRange(format!(
        "{what} is beyond what dates can hold (about 292 billion years either side of the Big Bang)"
    ))
}

impl Instant {
    pub const ORIGIN: Instant = Instant { s: 0, ns: 0 };

    pub const fn seconds(s: i64) -> Self {
        Instant { s, ns: 0 }
    }

    /// Whole Julian years from the origin, refused past the axis's end.
    pub fn from_years(y: i64) -> Result<Self, OutOfRange> {
        y.checked_mul(YEAR_S)
            .map(Instant::seconds)
            .ok_or_else(|| refuse(format_args!("year {y}")))
    }

    /// Whole Julian years from the origin, held at the axis's end rather
    /// than wrapped. For callers that cannot refuse; prefer `from_years`.
    pub fn years(y: i64) -> Self {
        Instant::seconds(y.saturating_mul(YEAR_S))
    }

    /// A date given as floating-point seconds (old saves, JSON, SSE 3
    /// epochs). The double's own value is kept exactly: its whole seconds
    /// and the nanoseconds of its fraction.
    pub fn from_seconds_f64(x: f64) -> Result<Self, OutOfRange> {
        // 2^63 is exact as a double; anything at or past it does not fit.
        const LIMIT: f64 = 9_223_372_036_854_775_808.0;
        if !(-LIMIT..LIMIT).contains(&x) {
            return Err(refuse(format_args!("{x:e} seconds")));
        }
        let whole = x.floor();
        let mut s = whole as i64;
        let mut ns = ((x - whole) * 1.0e9).round() as u32;
        if ns >= NS_PER_S {
            s = s
                .checked_add(1)
                .ok_or_else(|| refuse(format_args!("{x:e} seconds")))?;
            ns = 0;
        }
        Ok(Instant { s, ns })
    }

    /// Seconds from `anchor` to this date. The whole seconds are subtracted
    /// as integers before anything becomes a double, so the answer is exact
    /// to the nanosecond while it is under 2^53 s (285 million years) and
    /// as close as a double can be beyond.
    pub fn since(self, anchor: Instant) -> f64 {
        let ds = self.s as i128 - anchor.s as i128;
        let dns = self.ns as i64 - anchor.ns as i64;
        ds as f64 + dns as f64 * 1.0e-9
    }

    /// The angle turned at `rate_rad_s` from `anchor` to this date, in
    /// [0, 2π). This is what an orbit's mean anomaly or a spin phase needs:
    /// `rate * since(anchor)` in plain doubles loses whole seconds once the
    /// span is long (an orbit 4.6 Gyr from its epoch has turned some 1e10
    /// radians, where a double's step is 4e-6 rad, hundreds of km). Here the
    /// product is formed exactly (Dekker's two-product on a split of the
    /// whole seconds, no fused multiply-add) and the whole turns are taken
    /// off before rounding, so the angle is good to about 1e-16 rad and a
    /// one-second step is always the rate times one second.
    ///
    /// The span is still only as good as the rate: a period known to one
    /// part in 1e9 says nothing about phase after 1e9 turns. That is the
    /// physics's limit, honest and labelled; a date that cannot move by one
    /// second is not.
    pub fn angle_since(self, anchor: Instant, rate_rad_s: f64) -> f64 {
        use crate::math::{floor, TAU};
        /// 2π minus its nearest double.
        const TAU_LO: f64 = 2.449_293_598_294_706_4e-16;
        const LOW_BITS: u32 = 26;
        let ds = self.s as i128 - anchor.s as i128;
        let hi = ((ds >> LOW_BITS) as f64) * (1u64 << LOW_BITS) as f64;
        let lo = (ds & ((1i128 << LOW_BITS) - 1)) as f64;
        let dns = (self.ns as i64 - anchor.ns as i64) as f64 * 1.0e-9;
        let a = two_prod(rate_rad_s, hi);
        let b = two_prod(rate_rad_s, lo);
        let sum = dd_add(dd_add(a, b), (rate_rad_s * dns, 0.0));
        let k = floor(sum.0 / TAU);
        let whole = two_prod(k, TAU);
        let (d, de) = two_sum(sum.0, -whole.0);
        let mut r = d + (de + sum.1 - whole.1 - k * TAU_LO);
        if r < 0.0 {
            r += TAU;
        }
        if r >= TAU {
            r -= TAU;
        }
        r
    }

    /// This date moved by a span in seconds (a double is fine for a span:
    /// it is a difference, not a date), refused past the axis's end.
    pub fn after(self, span_s: f64) -> Result<Self, OutOfRange> {
        let d = Instant::from_seconds_f64(span_s)?;
        let mut ns = self.ns + d.ns;
        let mut s = self.s.checked_add(d.s);
        if ns >= NS_PER_S {
            ns -= NS_PER_S;
            s = s.and_then(|s| s.checked_add(1));
        }
        s.map(|s| Instant { s, ns })
            .ok_or_else(|| refuse(format_args!("{span_s:e} seconds on from {}", self.s)))
    }

    /// This date moved by whole seconds, refused past the axis's end.
    pub fn checked_add_seconds(self, ds: i64) -> Result<Self, OutOfRange> {
        self.s
            .checked_add(ds)
            .map(|s| Instant { s, ns: self.ns })
            .ok_or_else(|| refuse(format_args!("{} seconds on from {}", ds, self.s)))
    }
}

/// Reading a date from a request: a whole number of seconds is taken
/// exactly, a fractional one as its double's own value, and `{s, ns}` as
/// itself. Use with `#[serde(deserialize_with = "sse_core::time::seconds")]`,
/// or hold a `Seconds`.
pub fn seconds<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Instant, D::Error> {
    d.deserialize_any(SecondsVisitor)
}

/// A date as it arrives in a request or leaves in a reply: a plain number
/// of seconds. A whole number is read exactly; written back, a date with no
/// nanoseconds is a whole number again.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Seconds(pub Instant);

impl<'de> Deserialize<'de> for Seconds {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        seconds(d).map(Seconds)
    }
}

impl Serialize for Seconds {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if self.0.ns == 0 {
            s.serialize_i64(self.0.s)
        } else {
            s.serialize_f64(self.0.since(Instant::ORIGIN))
        }
    }
}

struct SecondsVisitor;

impl<'de> serde::de::Visitor<'de> for SecondsVisitor {
    type Value = Instant;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a date in seconds (a number, or {s, ns})")
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Instant, E> {
        Ok(Instant::seconds(v))
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Instant, E> {
        i64::try_from(v)
            .map(Instant::seconds)
            .map_err(|_| E::custom(refuse(format_args!("{v} seconds"))))
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Instant, E> {
        Instant::from_seconds_f64(v).map_err(E::custom)
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Instant, A::Error> {
        Instant::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}

/// a + b as an exact pair (Knuth's two-sum).
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

/// A double cut into two halves of 26 bits (Veltkamp), so their products are exact.
fn split(a: f64) -> (f64, f64) {
    let c = 134_217_729.0 * a; // 2^27 + 1
    let hi = c - (c - a);
    (hi, a - hi)
}

/// a * b as an exact pair (Dekker), without a fused multiply-add.
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (ah, al) = split(a);
    let (bh, bl) = split(b);
    (p, ((ah * bh - p) + ah * bl + al * bh) + al * bl)
}

/// Sum of two double-double numbers.
fn dd_add(x: (f64, f64), y: (f64, f64)) -> (f64, f64) {
    let (s, e) = two_sum(x.0, y.0);
    let e = e + x.1 + y.1;
    let hi = s + e;
    (hi, e - (hi - s))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 13.8 Gyr, the present age of the universe, in seconds.
    const NOW_COSMIC: i64 = 13_800_000_000 * YEAR_S;

    #[test]
    fn one_second_is_one_second_at_any_age() {
        for base in [0, 4_610_000_000 * YEAR_S, NOW_COSMIC, -NOW_COSMIC] {
            let a = Instant::seconds(base);
            for k in 0..64 {
                assert_eq!(Instant::seconds(base + k).since(a), k as f64);
            }
        }
        // The same step through a double date is lost: the fault this fixes.
        let t = NOW_COSMIC as f64;
        assert_eq!(t + 1.0, t);
    }

    /// Difference of two angles, wrapped to (-π, π].
    fn turn(a: f64, b: f64) -> f64 {
        let mut d = a - b;
        while d > crate::math::PI {
            d -= crate::math::TAU;
        }
        while d <= -crate::math::PI {
            d += crate::math::TAU;
        }
        d
    }

    #[test]
    fn angles_move_every_second_at_any_age() {
        let station = 1.131e-3; // rad/s, a 92-minute orbit
        let earth = 1.991e-7;
        let origin = Instant::ORIGIN;
        for rate in [station, earth, -station] {
            // Short spans agree with the plain product.
            for dt in [0, 1, 3_600, 86_400 * 365] {
                let want = (rate * dt as f64).rem_euclid(crate::math::TAU);
                let got = Instant::seconds(dt).angle_since(origin, rate);
                assert!(turn(got, want).abs() < 1e-12, "{rate} {dt}: {got} {want}");
            }
            for base in [4_610_000_000 * YEAR_S, NOW_COSMIC, -NOW_COSMIC] {
                let t = |k: i64| Instant::seconds(base + k).angle_since(origin, rate);
                for k in 0..64 {
                    let step = turn(t(k + 1), t(k));
                    assert!((step - rate).abs() < 1e-12, "{rate} at {base}+{k}: {step}");
                    assert!((0.0..crate::math::TAU).contains(&t(k)));
                }
                // Additive: origin -> base, then base -> base + a day.
                let mid = Instant::seconds(base);
                let end = Instant::seconds(base + 86_400);
                let two = mid.angle_since(origin, rate) + end.angle_since(mid, rate);
                assert!(turn(two, end.angle_since(origin, rate)).abs() < 1e-9);
            }
        }
        // Nanoseconds count too.
        let half = Instant {
            s: NOW_COSMIC,
            ns: 500_000_000,
        };
        let whole = Instant::seconds(NOW_COSMIC);
        let d = turn(
            half.angle_since(origin, station),
            whole.angle_since(origin, station),
        );
        assert!((d - 0.5 * station).abs() < 1e-12);
    }

    #[test]
    fn nanoseconds_and_float_dates() {
        let a = Instant::from_seconds_f64(1.5).unwrap();
        assert_eq!(
            a,
            Instant {
                s: 1,
                ns: 500_000_000
            }
        );
        let b = Instant::from_seconds_f64(-1.25).unwrap();
        assert_eq!(
            b,
            Instant {
                s: -2,
                ns: 750_000_000
            }
        );
        assert_eq!(a.since(b), 2.75);
        assert_eq!(b.since(a), -2.75);
        let big = Instant::from_seconds_f64(NOW_COSMIC as f64).unwrap();
        assert_eq!(big.since(Instant::ORIGIN), NOW_COSMIC as f64);
        assert!(Instant::from_seconds_f64(f64::NAN).is_err());
        assert!(Instant::from_seconds_f64(1.0e19).is_err());
    }

    #[test]
    fn spans_move_dates() {
        let a = Instant {
            s: 5,
            ns: 600_000_000,
        };
        assert_eq!(
            a.after(1.5).unwrap(),
            Instant {
                s: 7,
                ns: 100_000_000
            }
        );
        assert_eq!(
            a.after(-0.25).unwrap(),
            Instant {
                s: 5,
                ns: 350_000_000
            }
        );
        let big = Instant::seconds(NOW_COSMIC);
        assert_eq!(big.after(1.0).unwrap().since(big), 1.0);
        assert!(Instant::seconds(i64::MAX).after(1.0).is_err());
    }

    #[test]
    fn range_is_refused_not_wrapped() {
        let edge = i64::MAX / YEAR_S;
        assert!(Instant::from_years(edge).is_ok());
        let e = Instant::from_years(edge + 1).unwrap_err();
        assert!(e.0.contains("292 billion years"), "{e}");
        assert_eq!(Instant::years(edge + 1).s, i64::MAX);
        assert_eq!(Instant::years(-edge - 2).s, i64::MIN);
        assert!(Instant::seconds(i64::MAX).checked_add_seconds(1).is_err());
        // Far-apart dates still subtract without overflow.
        let d = Instant::seconds(i64::MAX).since(Instant::seconds(i64::MIN));
        assert!(d > 1.8e19);
    }
}
