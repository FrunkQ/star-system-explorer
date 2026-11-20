//! O7: shifts (nebulae-and-light design section 5). Because light stays a
//! spectrum until the very last step, shifting it is exact and cheap: a shift
//! by factor `D` (observed frequency over emitted) with boost exponent `p`
//! turns the emitted spectral luminosity L'_λ into
//!
//!   L_λ(λ) = D^(p+2) L'_λ(λ D),
//!
//! so every band's observed power is `D^(p+1)` times the emitted power over
//! the band's edges multiplied by `D`. Lines move from λ₀ to λ₀ / D. No
//! resampling, no approximation.
//!
//! | Cause | D | p |
//! |---|---|---|
//! | motion of a discrete source (a star, a ship, a blob in a jet) | 1 / (γ (1 − β cos θ)) | 3 |
//! | motion of a steady continuous jet | same | 2 |
//! | climbing out of gravity from radius r | √(1 − r_s / r) | 1 |
//! | cosmological expansion, with the transverse comoving distance | 1 / (1 + z) | 1 |

use crate::*;
use serde::{Deserialize, Serialize};

pub const R_RL79_SHIFT: &str = "Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics, Wiley, chapter 4: Doppler factor, relativistic beaming and the transformation of specific intensity (I_ν / ν³ invariant)";
pub const R_HOGG99: &str = "Hogg, D. W. 1999, Distance measures in cosmology, arXiv:astro-ph/9905116: flux from luminosity with the luminosity distance d_L = (1 + z) D_M";
pub const R_GR: &str = "Gravitational redshift of light leaving a static spherical mass: 1 + z = (1 − 2GM / r c²)^(-½) (Schwarzschild); tested on Sirius B by Barstow, M. A. et al. 2005, MNRAS 362, 1134 (80.4 ± 4.8 km/s)";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shift {
    /// Observed frequency over emitted frequency.
    pub factor: f64,
    /// Boost exponent: 3 for a discrete moving source, 2 for a steady jet, 1 for gravity and expansion.
    pub exponent: f64,
    pub cause: String,
    pub why: String,
}

impl Shift {
    pub fn none() -> Self {
        Shift {
            factor: 1.0,
            exponent: 3.0,
            cause: "none".into(),
            why: "no shift".into(),
        }
    }

    /// Relativistic Doppler shift and beaming. `beta` is speed over c, and
    /// `cos_theta` the cosine of the angle between the velocity and the line
    /// towards the observer (1 = straight at the observer).
    pub fn doppler(beta: f64, cos_theta: f64, continuous_jet: bool) -> Self {
        let b = beta.clamp(0.0, 0.999_999_999);
        let gamma = 1.0 / m::sqrt(1.0 - b * b);
        let d = 1.0 / (gamma * (1.0 - b * cos_theta.clamp(-1.0, 1.0)));
        Shift {
            factor: d,
            exponent: if continuous_jet { 2.0 } else { 3.0 },
            cause: "motion".into(),
            why: format!(
                "Moving at {:.4} of light speed, {:.0}° from the line of sight: light arrives {} by a factor {:.4} in frequency and {:.3} times as bright in total.",
                b,
                m::acos(cos_theta.clamp(-1.0, 1.0)) * 180.0 / m::PI,
                if d >= 1.0 { "bluer" } else { "redder" },
                d,
                m::pow(d, if continuous_jet { 3.0 } else { 4.0 })
            ),
        }
    }

    /// Light climbing out of gravity from radius `r` round mass `mass_kg`.
    pub fn gravitational(mass_kg: f64, r_m: f64) -> Self {
        let rs = 2.0 * G * mass_kg / (C * C);
        let d = m::sqrt((1.0 - rs / r_m).max(0.0));
        Shift {
            factor: d,
            exponent: 1.0,
            cause: "gravity".into(),
            why: format!(
                "Climbing out of gravity from {:.3e} m (the event horizon would be {:.3e} m) reddens it by z = {:.3e}, as if receding at {:.1} km/s.",
                r_m,
                rs,
                1.0 / d - 1.0,
                (1.0 / d - 1.0) * C / 1e3
            ),
        }
    }

    /// Cosmological redshift `z`. Pair with the transverse comoving distance
    /// D_M when turning luminosity into flux (that is the luminosity distance
    /// d_L = (1 + z) D_M in effect).
    pub fn cosmological(z: f64) -> Self {
        Shift {
            factor: 1.0 / (1.0 + z.max(0.0)),
            exponent: 1.0,
            cause: "expansion".into(),
            why: format!(
                "The universe has stretched by {:.3} since this light left, so every wavelength is {:.3} times longer.",
                1.0 + z,
                1.0 + z
            ),
        }
    }

    /// Two shifts one after the other: factors multiply, and so do boosts.
    pub fn then(&self, o: &Shift) -> Composite {
        Composite(vec![self.clone(), o.clone()])
    }

    /// Bolometric luminosity multiplier, D^(p+1).
    pub fn boost(&self) -> f64 {
        m::pow(self.factor, self.exponent + 1.0)
    }

    /// Where a line emitted at `at_m` is seen.
    pub fn line_at(&self, at_m: f64) -> f64 {
        at_m / self.factor
    }
}

/// Several shifts in a row (for example a jet's beaming, then the host galaxy's redshift).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Composite(pub Vec<Shift>);

impl Composite {
    fn total(&self) -> (f64, f64) {
        self.0
            .iter()
            .fold((1.0, 1.0), |(d, b), s| (d * s.factor, b * s.boost()))
    }
}

/// Observed power between `a` and `b` after a shift: D^(p+1) times the emitted power over [aD, bD].
pub fn shifted_power(d: &LightDescription, a: f64, b: f64, s: &Shift, table: &LightTable) -> f64 {
    s.boost() * power_between(d, a * s.factor, b * s.factor, table)
}

/// The same for several shifts in a row.
pub fn composite_power(
    d: &LightDescription,
    a: f64,
    b: f64,
    c: &Composite,
    table: &LightTable,
) -> f64 {
    let (f, boost) = c.total();
    boost * power_between(d, a * f, b * f, table)
}

/// Observed spectral luminosity at λ (continuum only), W/m.
pub fn shifted_spectral_luminosity(
    d: &LightDescription,
    lambda_m: f64,
    s: &Shift,
    table: &LightTable,
) -> f64 {
    m::pow(s.factor, s.exponent + 2.0) * spectral_luminosity(d, lambda_m * s.factor, table)
}

/// Publish a description as seen after a shift: bands, slices, total, samples, peak.
pub fn publish_shifted(d: &LightDescription, s: &Shift, table: &LightTable) -> Published {
    let bands: Vec<BandPower> = table
        .bands
        .iter()
        .map(|b| BandPower {
            band: b.id.clone(),
            power_w: shifted_power(d, b.from_m, b.to_m, s, table),
        })
        .collect();
    let visible_slices = table
        .visible_slices
        .iter()
        .map(|b| BandPower {
            band: b.id.clone(),
            power_w: shifted_power(d, b.from_m, b.to_m, s, table),
        })
        .collect();
    let total_w = bands.iter().map(|b| b.power_w).sum();
    let samples: Vec<[f64; 2]> = table
        .grid()
        .into_iter()
        .map(|l| [l, shifted_spectral_luminosity(d, l, s, table)])
        .collect();
    let mut peak_m = 0.0;
    let mut best = 0.0;
    for [l, v] in &samples {
        if l * v > best {
            best = l * v;
            peak_m = *l;
        }
    }
    let mut description = d.clone();
    description.why.push(s.why.clone());
    description.references.push(Reference {
        text: match s.cause.as_str() {
            "gravity" => R_GR.into(),
            "expansion" => R_HOGG99.into(),
            _ => R_RL79_SHIFT.into(),
        },
        doi: None,
        url: Some(match s.cause.as_str() {
            "gravity" => "https://ui.adsabs.harvard.edu/abs/2005MNRAS.362.1134B".into(),
            "expansion" => "https://arxiv.org/abs/astro-ph/9905116".into(),
            _ => "https://doi.org/10.1002/9783527618170".into(),
        }),
    });
    Published {
        description,
        bands,
        visible_slices,
        total_w,
        samples,
        peak_m,
    }
}
