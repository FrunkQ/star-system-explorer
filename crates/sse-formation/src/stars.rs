//! Stage 2, star or stars: primary mass, multiplicity, companions and their
//! orbits, and the dated close-binary events that follow from the life lines.
//!
//! Chaotic outcomes are keyed draws from calibrated distributions (design
//! principle 3), labelled `drawn` with their spread.

use crate::lifeline::{LifeLine, MASS_RANGE};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

pub const REF_KROUPA: &str =
    "Kroupa, P. 2001, MNRAS 322, 231: initial mass function, slopes 1.3 below 0.5 and 2.3 above";
pub const REF_DK13: &str = "Duchene, G. and Kraus, A. 2013, ARA&A 51, 269, table 1: multiplicity fraction against primary mass";
pub const REF_RAGHAVAN: &str = "Raghavan, D. et al. 2010, ApJS 190, 1: solar-type multiples, log-normal periods (mean log P = 5.03 days, sigma 2.28), flat mass ratios, triples 8 of 44 percent";
pub const REF_EGGLETON: &str = "Eggleton, P. P. 1983, ApJ 268, 368: Roche-lobe radius";
pub const REF_MARDLING: &str =
    "Mardling, R. A. and Aarseth, S. J. 2001, MNRAS 321, 398: stability of hierarchical triples";

/// Kroupa 2001 mass draw on [0.1, 100] solar masses through its exact inverse
/// cumulative distribution (two power-law pieces, continuous at 0.5).
pub fn kroupa_quantile(u: f64) -> f64 {
    let (lo, br, hi) = (MASS_RANGE.0, 0.5, MASS_RANGE.1);
    // Piece 1: m^-1.3 on [lo, br]; piece 2: 0.5 m^-2.3 on [br, hi].
    let n1 = (m::pow(lo, -0.3) - m::pow(br, -0.3)) / 0.3;
    let n2 = 0.5 * (m::pow(br, -1.3) - m::pow(hi, -1.3)) / 1.3;
    let p1 = n1 / (n1 + n2);
    if u < p1 {
        let target = u * (n1 + n2) * 0.3;
        m::pow(m::pow(lo, -0.3) - target, -1.0 / 0.3)
    } else {
        let target = (u - p1) * (n1 + n2) * 1.3 / 0.5;
        m::pow(m::pow(br, -1.3) - target, -1.0 / 1.3)
    }
}

/// Multiplicity fraction (companion stars, any number) against primary mass,
/// Duchene and Kraus 2013, interpolated in log mass between their bins.
pub fn multiplicity_fraction(mass: f64) -> f64 {
    const PTS: [(f64, f64); 6] = [
        (0.1, 0.22),
        (0.3, 0.26),
        (1.0, 0.44),
        (3.0, 0.50),
        (12.0, 0.60),
        (30.0, 0.80),
    ];
    if mass <= PTS[0].0 {
        return PTS[0].1;
    }
    for w in PTS.windows(2) {
        let ((m0, f0), (m1, f1)) = (w[0], w[1]);
        if mass <= m1 {
            let x = (m::log10(mass) - m::log10(m0)) / (m::log10(m1) - m::log10(m0));
            return f0 + (f1 - f0) * x;
        }
    }
    PTS[5].1
}

/// A star's place in its system: the primary, or a companion orbiting the
/// barycentre of everything inside it (hierarchical, at most three here).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Star {
    pub address: Address,
    pub mass: f64,
    pub life: LifeLine,
    /// None for the primary.
    pub orbit: Option<BinaryOrbit>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BinaryOrbit {
    /// Days.
    pub period_days: f64,
    /// AU.
    pub semi_major_axis_au: f64,
    pub eccentricity: f64,
    /// Mass inside this orbit (the stars it circles), solar masses.
    pub inner_mass: f64,
}

/// Draws for stage 2: indexed counters, one stream, keyed by the system.
pub struct StarDraws(Draws);

impl StarDraws {
    pub fn new(seed: &Seed, system: &Address) -> Self {
        StarDraws(Draws::new(
            *seed,
            system.clone(),
            Stream::new("formation.stars.v1").expect("valid"),
        ))
    }
    fn u(&self, i: u64) -> f64 {
        self.0.unit(i)
    }
}

/// Pins on stage 2 (the rest is drawn).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct StarPins {
    pub primary_mass: Option<f64>,
    /// Number of stars, 1 to 3.
    pub count: Option<u32>,
}

/// Orbital period in days from separation (AU) and total mass (solar masses):
/// Kepler's third law in solar units.
pub fn period_days(a_au: f64, mass: f64) -> f64 {
    365.25 * m::sqrt(a_au * a_au * a_au / mass)
}

pub fn semi_major_axis_au(p_days: f64, mass: f64) -> f64 {
    let p = p_days / 365.25;
    m::cbrt(mass * p * p)
}

/// Roche-lobe radius over separation for mass ratio `q` = donor / accretor.
pub fn roche_lobe_fraction(q: f64) -> f64 {
    let q13 = m::cbrt(q);
    let q23 = q13 * q13;
    0.49 * q23 / (0.6 * q23 + m::ln(1.0 + q13))
}

fn companion_orbit(
    d: &StarDraws,
    base: u64,
    inner_mass: f64,
    total: f64,
    min_a: f64,
) -> BinaryOrbit {
    // Raghavan 2010 periods; resampled upward (counters base+10k) until the
    // orbit clears `min_a` (stability for the outer orbit of a triple).
    let mut k = 0;
    let (p, a) = loop {
        let z =
            sse_resolve::dist::std_normal_quantile(d.u(base + 10 * k).clamp(1e-12, 1.0 - 1e-12));
        let logp = 5.03 + 2.28 * z;
        let p = m::pow(10.0, logp.clamp(-0.5, 10.0));
        let a = semi_major_axis_au(p, total);
        if a >= min_a || k == 20 {
            break (p, a.max(min_a));
        }
        k += 1;
    };
    // Tidal circularisation below about 12 days (Raghavan figure 14).
    let e = if p < 12.0 { 0.0 } else { 0.8 * d.u(base + 1) };
    // The period follows from the (possibly raised) separation.
    BinaryOrbit {
        period_days: period_days(a, total),
        semi_major_axis_au: a,
        eccentricity: e,
        inner_mass,
    }
}

/// Stage 2: the stars of one system.
pub fn form_stars(
    fits: &crate::hurley::Fits,
    seed: &Seed,
    system: &Address,
    pins: &StarPins,
    system_mass: Option<f64>,
) -> Result<Vec<Star>, crate::lifeline::OutOfRange> {
    let d = StarDraws::new(seed, system);
    let ratio = |i: u64| 0.1 + 0.9 * d.u(100 * i);
    let drawn_count = |m: f64| {
        if d.u(1) >= multiplicity_fraction(m) {
            1
        } else if d.u(2) < 8.0 / 44.0 {
            3
        } else {
            2
        }
    };
    // A fixed cloud-core mass (G0) is shared by its stars: primary M / (1 + sum q).
    // Companions that would fall below the fits' 0.1 solar masses are dropped
    // (they would be brown dwarfs, which F5 places as substellar objects).
    let (primary, count, masses) = match (pins.primary_mass, system_mass) {
        (None, Some(total)) => {
            if total < MASS_RANGE.0 {
                return Err(crate::lifeline::OutOfRange(format!(
                    "the cloud core holds {total:.3} solar masses, below the 0.1 of the smallest star: a brown dwarf system"
                )));
            }
            let mut n = pins.count.unwrap_or_else(|| drawn_count(total)).clamp(1, 3) as u64;
            loop {
                let qsum: f64 = (1..n).map(ratio).sum();
                let p = total / (1.0 + qsum);
                if n == 1 || (1..n).all(|i| ratio(i) * p >= MASS_RANGE.0) {
                    break (p, n, (1..n).map(|i| ratio(i) * p).collect::<Vec<_>>());
                }
                n -= 1;
            }
        }
        _ => {
            let p = pins.primary_mass.unwrap_or_else(|| kroupa_quantile(d.u(0)));
            let n = pins.count.unwrap_or_else(|| drawn_count(p)).clamp(1, 3) as u64;
            (
                p,
                n,
                (1..n).map(|i| (ratio(i) * p).max(MASS_RANGE.0)).collect(),
            )
        }
    };
    let mut stars = vec![Star {
        address: system.child("star", 0).expect("valid"),
        mass: primary,
        life: LifeLine::new(fits, primary)?,
        orbit: None,
    }];
    let mut inner_mass = primary;
    let mut inner_a = 0.0_f64;
    for i in 1..count {
        // Flat mass ratio against the primary, 0.1 to 1 (Raghavan 2010).
        let mass = masses[i as usize - 1];
        let total = inner_mass + mass;
        // Outer orbit of a triple: periastron beyond about 3 inner separations
        // (Mardling and Aarseth 2001, order of magnitude).
        let min_a = if inner_a > 0.0 { 3.5 * inner_a } else { 0.0 };
        let orbit = companion_orbit(&d, 100 * i + 3, inner_mass, total, min_a);
        inner_a = orbit.semi_major_axis_au * (1.0 + orbit.eccentricity);
        inner_mass = total;
        stars.push(Star {
            address: system.child("star", i).expect("valid"),
            mass,
            life: LifeLine::new(fits, mass)?,
            orbit: Some(orbit),
        });
    }
    Ok(stars)
}

/// Mardling and Aarseth 2001 eq. 90: the smallest outer-to-inner
/// semi-major-axis ratio at which a hierarchical triple stays stable, for
/// outer mass ratio `q_out` = m_outer / m_inner pair, outer eccentricity
/// `e_out` and mutual inclination `incl_rad`.
pub fn critical_ratio(q_out: f64, e_out: f64, incl_rad: f64) -> f64 {
    let e = e_out.clamp(0.0, 0.999);
    2.8 * m::pow((1.0 + q_out) * (1.0 + e) / m::sqrt(1.0 - e), 0.4) * (1.0 - 0.3 * incl_rad / m::PI)
}

/// Stability of one outer orbit around the stars inside it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Stability {
    /// a_outer / a_inner.
    pub ratio: f64,
    /// The Mardling and Aarseth critical ratio.
    pub critical: f64,
    pub stable: bool,
    pub why: String,
}

/// Tag every outer orbit of a hierarchical multiple (generated or authored):
/// each companion after the second is tested against the orbit inside it.
/// Unstable configurations are tagged, never refused.
pub fn stability(stars: &[Star]) -> Vec<Option<Stability>> {
    let mut out = vec![];
    let mut inner: Option<&BinaryOrbit> = None;
    for s in stars {
        let tag = match (&s.orbit, inner) {
            (Some(o), Some(i)) if i.semi_major_axis_au > 0.0 => {
                let ratio = o.semi_major_axis_au / i.semi_major_axis_au;
                let crit = critical_ratio(s.mass / o.inner_mass.max(1e-9), o.eccentricity, 0.0);
                let stable = ratio > crit;
                Some(Stability {
                    ratio,
                    critical: crit,
                    stable,
                    why: format!(
                        "outer orbit {ratio:.1} times the inner one; {} needs more than {crit:.1} (Mardling and Aarseth 2001){}",
                        if stable { "stable: it" } else { "unstable: it" },
                        if stable { "" } else { "; the system would break up within a few outer orbits" }
                    ),
                })
            }
            _ => None,
        };
        out.push(tag);
        if s.orbit.is_some() {
            inner = s.orbit.as_ref();
        }
    }
    out
}

#[cfg(test)]
mod stability_tests {
    use super::*;
    use crate::hurley::Fits;

    fn star(i: u64, mass: f64, orbit: Option<(f64, f64, f64)>) -> Star {
        let fits = Fits::new(0.0142);
        Star {
            address: "u:t/sys:0"
                .parse::<Address>()
                .unwrap()
                .child("star", i)
                .unwrap(),
            mass,
            life: LifeLine::new(&fits, mass).unwrap(),
            orbit: orbit.map(|(a, e, inner)| BinaryOrbit {
                period_days: 0.0,
                semi_major_axis_au: a,
                eccentricity: e,
                inner_mass: inner,
            }),
        }
    }

    #[test]
    fn mardling_aarseth_tags_unstable_triples() {
        // Equal masses, circular: the critical ratio is 2.8 × 1.5^0.4 ≈ 3.3.
        let c = critical_ratio(0.5, 0.0, 0.0);
        assert!((c - 2.8 * m::pow(1.5, 0.4)).abs() < 1e-12, "{c}");
        // Alpha Centauri-like: a 23 au pair with Proxima at 8700 au is stable.
        let ok = stability(&[
            star(0, 1.1, None),
            star(1, 0.9, Some((23.0, 0.52, 1.1))),
            star(2, 0.12, Some((8700.0, 0.5, 2.0))),
        ]);
        assert!(ok[0].is_none() && ok[1].is_none());
        assert!(ok[2].as_ref().unwrap().stable);
        // An authored triple with the third star at twice the inner orbit is
        // tagged unstable, not refused.
        let bad = stability(&[
            star(0, 1.0, None),
            star(1, 1.0, Some((1.0, 0.0, 1.0))),
            star(2, 1.0, Some((2.0, 0.1, 2.0))),
        ]);
        let t = bad[2].as_ref().unwrap();
        assert!(!t.stable && t.why.contains("unstable"), "{t:?}");
    }
}
