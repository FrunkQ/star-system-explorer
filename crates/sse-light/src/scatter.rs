//! Phase tables per kind of particle (visual polish K-e, VP-A5): the Mie
//! series (`sse_atoms::mie`, block `sse.light.mie_scattering`) worked once
//! per kind from its size and refractive index, so every cloud, haze, dust,
//! plume and ring layer reads the same table rather than a one-lobe guess.
//! A spread of sizes (log-normal) smooths the series' ripples the way a
//! real cloud does.

use serde::{Deserialize, Serialize};
use sse_atoms::mie::mie;
use sse_core::math as m;
use sse_flow::Reference;

/// One kind of particle in one colour of light.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Particle {
    pub id: String,
    /// Radius, m (the middle of the spread).
    pub radius_m: f64,
    pub wavelength_m: f64,
    /// Refractive index, real part, relative to vacuum.
    pub n: f64,
    /// Imaginary part (absorption), 0 for clear.
    #[serde(default)]
    pub k: f64,
    /// Refractive index of what the particles float in (1 for a thin gas).
    #[serde(default = "one")]
    pub medium_n: f64,
    /// Width of the log-normal spread of radii (ln σ_g); 0 for one size.
    #[serde(default)]
    pub spread: f64,
    /// Angles to give the phase function at, degrees; 0 to 180 by 1 if left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angles_deg: Option<Vec<f64>>,
}

fn one() -> f64 {
    1.0
}

/// What the particle does to light.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scattering {
    pub id: String,
    pub size_parameter: f64,
    pub q_ext: f64,
    pub q_sca: f64,
    pub q_abs: f64,
    /// Share of the light taken out that is scattered rather than absorbed.
    pub single_scattering_albedo: f64,
    /// Mean cosine of the scattering angle.
    pub g: f64,
    /// Extinction cross-section of the mean particle, m².
    pub cross_section_m2: f64,
    pub angles_deg: Vec<f64>,
    /// Phase function at each angle; its mean over all directions is 1.
    pub phase: Vec<f64>,
    pub references: Vec<Reference>,
}

/// Abscissae and weights for the spread: nine points across ±3σ.
const SPREAD_POINTS: usize = 9;

/// The phase table and efficiencies for one kind of particle.
pub fn scattering(p: &Particle) -> Result<Scattering, String> {
    if !(p.radius_m > 0.0 && p.wavelength_m > 0.0 && p.medium_n > 0.0) {
        return Err("radius, wavelength and the medium's index must be positive".into());
    }
    if !(p.spread >= 0.0 && p.spread <= 1.5) {
        return Err(format!(
            "spread (ln σ_g) must be 0 to 1.5, got {}",
            p.spread
        ));
    }
    let angles = p
        .angles_deg
        .clone()
        .unwrap_or_else(|| (0..=180).map(|d| d as f64).collect());
    if angles.iter().any(|a| !(0.0..=180.0).contains(a)) {
        return Err("angles must be 0 to 180 degrees".into());
    }
    let rad: Vec<f64> = angles.iter().map(|a| a * m::PI / 180.0).collect();
    let k_wave = 2.0 * m::PI * p.medium_n / p.wavelength_m;
    let (nr, ki) = (p.n / p.medium_n, p.k / p.medium_n);
    // Radii and weights by number across the log-normal spread.
    let pts: Vec<(f64, f64)> = if p.spread > 0.0 {
        (0..SPREAD_POINTS)
            .map(|i| {
                let z = -3.0 + 6.0 * i as f64 / (SPREAD_POINTS - 1) as f64;
                (p.radius_m * m::exp(z * p.spread), m::exp(-0.5 * z * z))
            })
            .collect()
    } else {
        vec![(p.radius_m, 1.0)]
    };
    let (mut cext, mut csca, mut gw, mut wsum) = (0.0, 0.0, 0.0, 0.0);
    let mut phase = vec![0.0; rad.len()];
    for (r, w) in &pts {
        let o = mie(k_wave * r, nr, ki, &rad)?;
        let area = m::PI * r * r;
        let ce = w * o.q_ext * area;
        let cs = w * o.q_sca * area;
        cext += ce;
        csca += cs;
        gw += cs * o.g;
        for (ph, v) in phase.iter_mut().zip(&o.phase) {
            *ph += cs * v;
        }
        wsum += w * area;
    }
    if csca > 0.0 {
        for ph in &mut phase {
            *ph /= csca;
        }
    }
    let q_ext = cext / wsum;
    let q_sca = csca / wsum;
    let wsum_n: f64 = pts.iter().map(|(_, w)| w).sum();
    Ok(Scattering {
        id: p.id.clone(),
        size_parameter: k_wave * p.radius_m,
        q_ext,
        q_sca,
        q_abs: (q_ext - q_sca).max(0.0),
        single_scattering_albedo: if q_ext > 0.0 {
            (q_sca / q_ext).min(1.0)
        } else {
            0.0
        },
        g: if csca > 0.0 { gw / csca } else { 0.0 },
        cross_section_m2: cext / wsum_n,
        angles_deg: angles,
        phase,
        references: sse_atoms::decl_of("sse.light.mie_scattering").references,
    })
}
