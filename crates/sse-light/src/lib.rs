//! SSE4 light (track O; nebulae-and-light design sections 1, 2 and 5B).
//!
//! Nothing is painted. Every glowing or lit thing publishes a small **light
//! description**: a few continuum parts and a list of named lines, in physical
//! units. Views choose how to show it at the very last step (band recipes, in
//! the UI); the kernel only ever publishes the spectrum and its integrals over
//! the fixed band set. That is what makes true colour, astro palettes, single
//! bands from radio to gamma, detection by sensor band and the spectrum chart
//! all come from one set of facts.
//!
//! Units: a part's spectral luminosity `L_λ` is in watts per metre of
//! wavelength, for the whole object, isotropic-equivalent. Flux at a distance
//! `d` is `L / (4π d²)`.
//!
//! Determinism: all transcendental maths goes through `sse_core::math`; band
//! integrals use a fixed number of steps per decade from the pack, never an
//! adaptive tolerance, so every device gives the same bits.
#![forbid(unsafe_code)]

pub mod air;
pub mod emit;
pub mod observe;
pub mod shift;
pub mod table;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use sse_core::math as m;
pub use sse_flow::Reference;
pub use table::{Band, LightTable, LineDef};

/// Planck constant, J s (CODATA 2018, exact).
pub const H: f64 = 6.626_070_15e-34;
/// Speed of light, m/s (exact).
pub const C: f64 = 299_792_458.0;
/// Boltzmann constant, J/K (CODATA 2018, exact).
pub const K_B: f64 = 1.380_649e-23;
/// Stefan-Boltzmann constant, W m⁻² K⁻⁴ (CODATA 2018, from h, k and c).
pub const SIGMA: f64 = sse_core::quantity::consts::STEFAN_BOLTZMANN;
/// Newton's constant (CODATA 2018).
pub const G: f64 = sse_core::quantity::consts::G;

pub const R_PLANCK: &str = "Planck's law and the Stefan-Boltzmann law: Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics, Wiley, chapter 1";
pub const R_CODATA: &str = "Tiesinga, E., Mohr, P. J., Newell, D. B. and Taylor, B. N. 2021, CODATA recommended values of the fundamental physical constants: 2018, Rev. Mod. Phys. 93, 025010";
pub const R_SS73: &str = "Shakura, N. I. and Sunyaev, R. A. 1973, Black holes in binary systems: observational appearance, A&A 24, 337: the thin disc temperature law T⁴ = 3GMṀ/(8πσr³)(1 - (r_in/r)^½)";

/// The references every light description carries for its maths.
pub fn base_references() -> Vec<Reference> {
    vec![
        Reference {
            text: R_PLANCK.into(),
            doi: Some("10.1002/9783527618170".into()),
            url: None,
        },
        Reference {
            text: R_CODATA.into(),
            doi: Some("10.1103/RevModPhys.93.025010".into()),
            url: None,
        },
    ]
}

/// One continuum part of a light description.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Part {
    /// The object's own heat: `L_λ = ε · area · π B_λ(T)`.
    Thermal {
        t_k: f64,
        area_m2: f64,
        emissivity: f64,
        cause: String,
    },
    /// Light from a luminous body, reflected. `area_m2` already carries the
    /// albedo and the dilution with distance: `L_λ = area · π B_λ(T_source)`.
    Reflected {
        source: String,
        t_k: f64,
        area_m2: f64,
        cause: String,
    },
    /// A power law, `F_ν ∝ ν^-alpha`, carrying `power_w` in total between
    /// `from_m` and `to_m`: synchrotron, coronae, cyclotron radio.
    PowerLaw {
        power_w: f64,
        from_m: f64,
        to_m: f64,
        alpha: f64,
        cause: String,
    },
    /// A thin accretion disc (Shakura and Sunyaev 1973), both faces, seen
    /// isotropically: rings from `r_inner_m` to `r_outer_m` round a mass.
    Disc {
        mass_kg: f64,
        accretion_kg_s: f64,
        r_inner_m: f64,
        r_outer_m: f64,
        cause: String,
    },
}

/// A named line's total power.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineEmission {
    pub line: String,
    pub power_w: f64,
    pub cause: String,
}

/// What an object gives off and reflects, as a spectrum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightDescription {
    pub address: String,
    pub parts: Vec<Part>,
    pub lines: Vec<LineEmission>,
    /// In words, what this light is made of.
    pub why: Vec<String>,
    /// Simplifications in force (each also in docs/fudges.md).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

impl LightDescription {
    pub fn new(address: &str) -> Self {
        LightDescription {
            address: address.into(),
            parts: vec![],
            lines: vec![],
            why: vec![],
            stand_ins: vec![],
            references: base_references(),
        }
    }
}

/// Planck spectral radiance `B_λ(T)`, W sr⁻¹ m⁻³.
pub fn planck(lambda_m: f64, t_k: f64) -> f64 {
    if t_k <= 0.0 || lambda_m <= 0.0 || !t_k.is_finite() {
        return 0.0;
    }
    let x = H * C / (lambda_m * K_B * t_k);
    if x > 700.0 {
        return 0.0;
    }
    let l2 = lambda_m * lambda_m;
    2.0 * H * C * C / (l2 * l2 * lambda_m) / m::expm1(x)
}

/// Thin-disc temperature at radius `r` (Shakura and Sunyaev 1973, zero torque at the inner edge).
pub fn disc_temperature(mass_kg: f64, accretion_kg_s: f64, r_inner_m: f64, r: f64) -> f64 {
    if r <= r_inner_m {
        return 0.0;
    }
    let t4 = 3.0 * G * mass_kg * accretion_kg_s / (8.0 * m::PI * SIGMA * r * r * r)
        * (1.0 - m::sqrt(r_inner_m / r));
    m::sqrt(m::sqrt(t4))
}

/// The rings a disc is split into: (temperature, area of both faces).
pub fn disc_rings(p: &Part, rings: usize) -> Vec<(f64, f64)> {
    let Part::Disc {
        mass_kg,
        accretion_kg_s,
        r_inner_m,
        r_outer_m,
        ..
    } = p
    else {
        return vec![];
    };
    let n = rings.max(1);
    let (a, b) = (m::ln(*r_inner_m), m::ln(*r_outer_m));
    (0..n)
        .map(|i| {
            let r0 = m::exp(a + (b - a) * i as f64 / n as f64);
            let r1 = m::exp(a + (b - a) * (i + 1) as f64 / n as f64);
            let rm = m::sqrt(r0 * r1);
            let t = disc_temperature(*mass_kg, *accretion_kg_s, *r_inner_m, rm);
            (t, 2.0 * m::PI * (r1 * r1 - r0 * r0))
        })
        .collect()
}

/// Power-law normalisation: `L_λ = k λ^(alpha-2)` carries `power` over [a, b].
fn power_law_k(power: f64, a: f64, b: f64, alpha: f64) -> f64 {
    let q = alpha - 1.0; // exponent of the integral
    let integral = if q.abs() < 1e-12 {
        m::ln(b / a)
    } else {
        (m::pow(b, q) - m::pow(a, q)) / q
    };
    if integral > 0.0 {
        power / integral
    } else {
        0.0
    }
}

/// Spectral luminosity of one part at one wavelength, W/m.
pub fn part_at(p: &Part, lambda_m: f64, table: &LightTable) -> f64 {
    match p {
        Part::Thermal {
            t_k,
            area_m2,
            emissivity,
            ..
        } => emissivity * area_m2 * m::PI * planck(lambda_m, *t_k),
        Part::Reflected { t_k, area_m2, .. } => area_m2 * m::PI * planck(lambda_m, *t_k),
        Part::PowerLaw {
            power_w,
            from_m,
            to_m,
            alpha,
            ..
        } => {
            if lambda_m < *from_m || lambda_m > *to_m {
                0.0
            } else {
                power_law_k(*power_w, *from_m, *to_m, *alpha) * m::pow(lambda_m, alpha - 2.0)
            }
        }
        Part::Disc { .. } => disc_rings(p, table.disc_rings)
            .iter()
            .map(|(t, a)| a * m::PI * planck(lambda_m, *t))
            .sum(),
    }
}

/// Continuum spectral luminosity at one wavelength (lines excluded), W/m.
pub fn spectral_luminosity(d: &LightDescription, lambda_m: f64, table: &LightTable) -> f64 {
    d.parts.iter().map(|p| part_at(p, lambda_m, table)).sum()
}

/// Simpson's rule in ln λ over [a, b] for a smooth function of λ, with a fixed
/// number of intervals per decade (pack `steps_per_decade`, even, at least 8).
fn integrate(f: impl Fn(f64) -> f64, a: f64, b: f64, per_decade: usize) -> f64 {
    if b <= a {
        return 0.0;
    }
    let (la, lb) = (m::ln(a), m::ln(b));
    let decades = (lb - la) / core::f64::consts::LN_10;
    let mut n = m::ceil(decades * per_decade as f64) as usize;
    n = n.max(16);
    if n % 2 == 1 {
        n += 1;
    }
    let h = (lb - la) / n as f64;
    let mut sum = 0.0;
    for i in 0..=n {
        let l = m::exp(la + h * i as f64);
        let w = if i == 0 || i == n {
            1.0
        } else if i % 2 == 1 {
            4.0
        } else {
            2.0
        };
        sum += w * f(l) * l;
    }
    sum * h / 3.0
}

/// One part's power between `a` and `b` (metres), W.
pub fn part_power(p: &Part, a: f64, b: f64, table: &LightTable) -> f64 {
    match p {
        Part::PowerLaw {
            power_w,
            from_m,
            to_m,
            alpha,
            ..
        } => {
            let (lo, hi) = (a.max(*from_m), b.min(*to_m));
            if hi <= lo {
                return 0.0;
            }
            let k = power_law_k(*power_w, *from_m, *to_m, *alpha);
            let q = alpha - 1.0;
            if q.abs() < 1e-12 {
                k * m::ln(hi / lo)
            } else {
                k * (m::pow(hi, q) - m::pow(lo, q)) / q
            }
        }
        _ => integrate(|l| part_at(p, l, table), a, b, table.steps_per_decade),
    }
}

/// Lines whose rest wavelength falls in [a, b), W.
pub fn line_power(d: &LightDescription, a: f64, b: f64, table: &LightTable) -> f64 {
    d.lines
        .iter()
        .filter_map(|l| table.line(&l.line).map(|def| (def.at_m, l.power_w)))
        .filter(|(at, _)| *at >= a && *at < b)
        .map(|(_, w)| w)
        .sum()
}

/// Total power between `a` and `b`: continuum plus lines, W.
pub fn power_between(d: &LightDescription, a: f64, b: f64, table: &LightTable) -> f64 {
    d.parts
        .iter()
        .map(|p| part_power(p, a, b, table))
        .sum::<f64>()
        + line_power(d, a, b, table)
}

/// One band's share of a description.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BandPower {
    pub band: String,
    pub power_w: f64,
}

/// What the kernel publishes for one object: the description, its power in
/// every band and visible slice, its total, and the sampled continuum for the
/// spectrum chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Published {
    pub description: LightDescription,
    pub bands: Vec<BandPower>,
    pub visible_slices: Vec<BandPower>,
    pub total_w: f64,
    /// `[wavelength m, L_λ W/m]` on the pack's grid, continuum only.
    pub samples: Vec<[f64; 2]>,
    /// The grid point where `λ L_λ` peaks (power per log wavelength), m.
    pub peak_m: f64,
}

/// Band powers of a description over a list of bands.
pub fn band_powers(d: &LightDescription, bands: &[Band], table: &LightTable) -> Vec<BandPower> {
    bands
        .iter()
        .map(|b| BandPower {
            band: b.id.clone(),
            power_w: power_between(d, b.from_m, b.to_m, table),
        })
        .collect()
}

/// Publish a description: band powers, total, samples and peak.
pub fn publish(d: &LightDescription, table: &LightTable) -> Published {
    let bands = band_powers(d, &table.bands, table);
    let visible_slices = band_powers(d, &table.visible_slices, table);
    let total_w = bands.iter().map(|b| b.power_w).sum();
    let samples: Vec<[f64; 2]> = table
        .grid()
        .into_iter()
        .map(|l| [l, spectral_luminosity(d, l, table)])
        .collect();
    let mut peak_m = 0.0;
    let mut best = 0.0;
    for [l, v] in &samples {
        if l * v > best {
            best = l * v;
            peak_m = *l;
        }
    }
    Published {
        description: d.clone(),
        bands,
        visible_slices,
        total_w,
        samples,
        peak_m,
    }
}

/// The temperature a blackbody of `area_m2` would need to give the same power
/// in [a, b] as this description (its brightness temperature in that band).
/// Bisection in ln T between 0.1 K and 10¹² K, fixed 200 steps.
pub fn brightness_temperature(
    d: &LightDescription,
    a: f64,
    b: f64,
    area_m2: f64,
    table: &LightTable,
) -> Option<f64> {
    let want = power_between(d, a, b, table);
    if want <= 0.0 || area_m2 <= 0.0 {
        return None;
    }
    let bb = |t: f64| {
        integrate(
            |l| area_m2 * m::PI * planck(l, t),
            a,
            b,
            table.steps_per_decade,
        )
    };
    let (mut lo, mut hi) = (m::ln(0.1), m::ln(1e12));
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if bb(m::exp(mid)) < want {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(m::exp(0.5 * (lo + hi)))
}
