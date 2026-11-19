//! F7: close binary stars by rule table (design section 4, "Stars": close
//! binaries use rule tables for mass transfer, common envelopes and mergers,
//! producing dated events). Sketch level: no detailed binary evolution.
//!
//! For each star on an orbit, the pair is the star and the mass it circles.
//! Each star swells as it ages (T1's ageing star, winds included); when one
//! fills its Roche lobe at periastron, the rule table decides:
//! - stable transfer when the mass ratio is below the critical one for the
//!   donor's phase (Hurley, Tout and Pols 2002): the envelope moves to the
//!   companion, conservatively, and the orbit changes with it;
//! - otherwise a common envelope: the orbit shrinks until the released
//!   orbital energy unbinds the envelope (Webbink 1984, alpha-lambda form);
//!   if what is left cannot fit inside its Roche lobe, or the donor is still
//!   on the main sequence, the pair merges.

use crate::epoch::TimeEvent;
use crate::star::{AgeingStar, StarSettings};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_formation::{FormationHistory, Phase};
use std::collections::BTreeMap;

/// One solar radius in au.
pub const RSUN_AU: f64 = 0.004_650_47;

pub const REF_EGGLETON: &str = "Eggleton, P. P. 1983, ApJ 268, 368: Roche lobe radius r_L/a = 0.49 q^(2/3) / (0.6 q^(2/3) + ln(1 + q^(1/3)))";
pub const REF_HTP02: &str = "Hurley, J. R., Tout, C. A. and Pols, O. R. 2002, MNRAS 329, 897: critical mass ratios for stable transfer (0.695 for low-mass main-sequence donors, 3 above about 0.7 solar masses as in BSE, 4 in the Hertzsprung gap, eq. 57 on the giant branch). arXiv:astro-ph/0201220";
pub const REF_WEBBINK: &str = "Webbink, R. F. 1984, ApJ 277, 355; de Kool, M. 1990, ApJ 358, 189: common-envelope energy budget, alpha = 1, lambda = 0.5";

/// Rule-table settings (pack data in a later step; defaults from the papers).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BinaryRules {
    pub q_crit_main_sequence: f64,
    /// Low-mass (deeply convective) main-sequence donors below 0.7 solar masses:
    /// 0.695 (Hurley, Tout and Pols 2002, section 2.6.1).
    pub q_crit_low_mass_main_sequence: f64,
    pub q_crit_gap: f64,
    /// Exponent x in HTP02 eq. 57 for giant donors.
    pub giant_x: f64,
    pub alpha_ce: f64,
    pub lambda_ce: f64,
    /// Time grid for the search: steps across each star's life.
    pub search_steps: u32,
}

impl Default for BinaryRules {
    fn default() -> Self {
        BinaryRules {
            q_crit_main_sequence: 3.0,
            q_crit_low_mass_main_sequence: 0.695,
            q_crit_gap: 4.0,
            giant_x: 0.30,
            alpha_ce: 1.0,
            lambda_ce: 0.5,
            search_steps: 600,
        }
    }
}

/// Eggleton 1983: Roche lobe radius over separation, for mass ratio q = M_donor / M_accretor.
pub fn roche_fraction(q: f64) -> f64 {
    let q3 = m::cbrt(q);
    let q23 = q3 * q3;
    0.49 * q23 / (0.6 * q23 + m::ln(1.0 + q3))
}

/// Critical mass ratio above which transfer is unstable.
pub fn q_crit(r: &BinaryRules, phase: Phase, mass: f64, core: f64) -> f64 {
    match phase {
        Phase::PreMainSequence | Phase::MainSequence if mass < 0.7 => {
            r.q_crit_low_mass_main_sequence
        }
        Phase::PreMainSequence | Phase::MainSequence => r.q_crit_main_sequence,
        Phase::HertzsprungGap => r.q_crit_gap,
        _ => (1.67 - r.giant_x + 2.0 * m::pow((core / mass).clamp(0.0, 1.0), 5.0)) / 2.13,
    }
}

/// A contact found for one pair.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Interaction {
    pub donor: String,
    pub accretor: String,
    pub age_myr: f64,
    pub outcome: String,
    pub a_before_au: f64,
    pub a_after_au: Option<f64>,
    pub event: TimeEvent,
}

/// One pair, given (address, initial mass) for each side, its orbit and metal fraction.
pub fn pair_contact(
    a: (String, f64),
    b: (String, f64),
    sep_au: f64,
    e: f64,
    z: f64,
    star: &StarSettings,
    rules: &BinaryRules,
) -> Option<Interaction> {
    let mk = |(address, mass): (String, f64)| -> Option<Side> {
        AgeingStar::new(mass, z, star.clone())
            .ok()
            .map(|star| Side { address, star })
    };
    contact(&mk(a)?, &mk(b)?, sep_au, e, rules)
}

struct Side {
    address: String,
    star: AgeingStar,
}

/// Search every close pair in a formed system for Roche-lobe contact and
/// apply the rule table. Dated by star age (Myr), in date order.
pub fn interactions(
    h: &FormationHistory,
    star: &StarSettings,
    rules: &BinaryRules,
) -> Vec<Interaction> {
    let z = h.inheritance.z;
    let mut out = vec![];
    for (k, s) in h.stars.iter().enumerate() {
        let Some(orbit) = &s.orbit else { continue };
        // The pair: this star and what it circles (star 0 for the first
        // companion; the inner mass as one body otherwise).
        let first = if k == 1 {
            h.stars[0].mass
        } else {
            orbit.inner_mass
        };
        if let Some(i) = pair_contact(
            (h.stars[0].address.to_string(), first),
            (s.address.to_string(), s.mass),
            orbit.semi_major_axis_au,
            orbit.eccentricity,
            z,
            star,
            rules,
        ) {
            out.push(i);
        }
    }
    out.sort_by(|x, y| x.age_myr.total_cmp(&y.age_myr));
    out
}

/// Overfill of `d` against its lobe at age t (positive when it overflows).
fn overfill(d: &Side, acc: &Side, peri: f64, t: f64) -> f64 {
    let sd = d.star.state_at(t);
    if sd.star.phase.is_remnant() || sd.star.phase == Phase::BrownDwarf {
        return -1.0;
    }
    let ma = acc.star.mass_at(t);
    // Orbits widen as the pair loses mass in winds (a M_total constant, Jeans 1924).
    let m0 = d.star.line.initial_mass + acc.star.line.initial_mass;
    let a_now = peri * m0 / (sd.mass + ma);
    sd.star.radius * RSUN_AU - roche_fraction(sd.mass / ma) * a_now
}

fn contact(a: &Side, b: &Side, sep: f64, e: f64, rules: &BinaryRules) -> Option<Interaction> {
    let peri = sep * (1.0 - e);
    // Whichever fills its lobe first is the donor.
    let first = |d: &Side, acc: &Side| -> Option<f64> {
        // The star's own clock: even steps to the giant branch, then steps even
        // in log luminosity up the giant branch (where radii change fast).
        let l = &d.star.line;
        let n = rules.search_steps.max(10);
        let cap = 50_000.0;
        let mut ts = vec![];
        let ms_end = l.t_bgb.min(cap);
        for i in 1..=n / 3 {
            ts.push(ms_end * f64::from(i) / f64::from(n / 3));
        }
        if l.t_end < cap && l.t_end > l.t_bgb {
            for i in 1..n - n / 3 {
                ts.push(
                    d.star
                        .age_at_giant_fraction(f64::from(i) / f64::from(n - n / 3)),
                );
            }
            ts.push(l.t_end - 1e-6);
        }
        let mut prev = 0.0;
        for t in ts {
            if t <= prev {
                continue;
            }
            if overfill(d, acc, peri, t) > 0.0 {
                let (mut lo, mut hi) = (prev, t);
                for _ in 0..60 {
                    let mid = 0.5 * (lo + hi);
                    if overfill(d, acc, peri, mid) > 0.0 {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                return Some(hi);
            }
            prev = t;
        }
        None
    };
    let ta = first(a, b);
    let tb = first(b, a);
    let (d, acc, t) = match (ta, tb) {
        (Some(x), Some(y)) if y < x => (b, a, y),
        (Some(x), _) => (a, b, x),
        (None, Some(y)) => (b, a, y),
        (None, None) => return None,
    };
    let sd = d.star.state_at(t);
    let (md, mc) = (sd.mass, sd.core_mass.min(sd.mass));
    let ma = acc.star.mass_at(t);
    let m0 = d.star.line.initial_mass + acc.star.line.initial_mass;
    let a_i = sep * m0 / (md + ma);
    let q = md / ma;
    let qc = q_crit(rules, sd.star.phase, md, mc);
    let phase = sd.star.phase.label();
    let mut numbers = BTreeMap::new();
    numbers.insert("donor_mass".into(), md);
    numbers.insert("accretor_mass".into(), ma);
    numbers.insert("core_mass".into(), mc);
    numbers.insert("mass_ratio".into(), q);
    numbers.insert("critical_ratio".into(), qc);
    numbers.insert("separation_au".into(), a_i);
    numbers.insert("donor_radius_au".into(), sd.star.radius * RSUN_AU);
    let lobe = roche_fraction(q) * a_i * (1.0 - e);
    let main_seq = matches!(sd.star.phase, Phase::MainSequence | Phase::PreMainSequence);
    let (outcome, a_f, why) = if q < qc {
        // Conservative transfer of the envelope: a M_d M_a constant squared.
        let (md_f, ma_f) = if main_seq {
            (ma, md)
        } else {
            (mc, ma + md - mc)
        };
        let a_f = a_i * m::pow(md * ma / (md_f * ma_f), 2.0);
        (
            "stable-mass-transfer",
            Some(a_f),
            format!(
                "The {phase} star swelled to fill its Roche lobe ({lobe:.3} au at periastron) and began pouring gas onto its {ma:.2} solar mass companion. The mass ratio {q:.2} is below the critical {qc:.2} for its phase, so the flow is stable: the orbit changes from {a_i:.3} to {a_f:.3} au as mass changes hands."
            ),
        )
    } else if main_seq {
        (
            "merger",
            None,
            format!(
                "The {phase} star filled its Roche lobe with a mass ratio of {q:.2}, above the critical {qc:.2}; the transfer runs away and the two main-sequence stars merge into one of about {:.2} solar masses.",
                md + ma
            ),
        )
    } else {
        let menv = md - mc;
        let r_d = sd.star.radius * RSUN_AU;
        let a_f = mc * ma
            / (2.0
                * (md * ma / (2.0 * a_i) + md * menv / (rules.alpha_ce * rules.lambda_ce * r_d)));
        // After the envelope: does the companion fit inside its lobe?
        let r_acc = acc.star.state_at(t).star.radius * RSUN_AU;
        let acc_fits = r_acc < roche_fraction(ma / mc) * a_f;
        if acc_fits {
            (
                "common-envelope",
                Some(a_f),
                format!(
                    "The {phase} star overflowed its Roche lobe faster than its {ma:.2} solar mass companion could take the gas (mass ratio {q:.2}, critical {qc:.2}). The companion was engulfed in a common envelope; drag pulled the orbit in from {a_i:.3} to {a_f:.4} au and the released energy blew the {menv:.2} solar mass envelope away, leaving the bare {mc:.2} solar mass core and its companion in a tight orbit."
                ),
            )
        } else {
            (
                "merger",
                None,
                format!(
                    "The {phase} star's envelope swallowed its companion; spiralling in, the orbit's energy was not enough to blow away {menv:.2} solar masses of envelope before the companion reached the core, and the two merged."
                ),
            )
        }
    };
    if let Some(x) = a_f {
        numbers.insert("separation_after_au".into(), x);
    }
    Some(Interaction {
        donor: d.address.clone(),
        accretor: acc.address.clone(),
        age_myr: t,
        outcome: outcome.into(),
        a_before_au: a_i,
        a_after_au: a_f,
        event: TimeEvent {
            age_myr: t,
            kind: outcome.into(),
            subjects: vec![d.address.clone(), acc.address.clone()],
            why,
            level: "sketch".into(),
            plausible_future: false,
            numbers,
        },
    })
}
