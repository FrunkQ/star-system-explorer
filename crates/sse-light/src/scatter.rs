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
    /// Refractive index, real part, relative to vacuum. Left out (0), it
    /// is read from the sheet for `substance` in `phase`.
    #[serde(default)]
    pub n: f64,
    /// Imaginary part (absorption), 0 for clear.
    #[serde(default)]
    pub k: f64,
    /// The particle's substance on the master data sheet ("h2o"), read
    /// when `n` is left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substance: Option<String>,
    /// "liquid" (default) or "solid".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
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
    let (n, k) = if p.n > 0.0 {
        (p.n, p.k)
    } else {
        let s = p
            .substance
            .as_deref()
            .ok_or("give n (and k), or a substance on the sheet")?;
        sheet_index(
            &sse_reference::Reference::base(),
            s,
            p.phase.as_deref().unwrap_or("liquid"),
            p.wavelength_m,
        )?
    };
    let k_wave = 2.0 * m::PI * p.medium_n / p.wavelength_m;
    let (nr, ki) = (n / p.medium_n, k / p.medium_n);
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

/// A substance's refractive index n + ik at a wavelength from its master
/// data sheet table (`optical.refractive_index_liquid` or `_solid`, kind
/// "table" with `lambda_nm`, `n`, `k`), linear between rows and held at the
/// ends. Errors name what the sheet lacks.
pub fn sheet_index(
    sheet: &sse_reference::Reference,
    substance: &str,
    phase: &str,
    wavelength_m: f64,
) -> Result<(f64, f64), String> {
    let key = match phase {
        "liquid" => "refractive_index_liquid",
        "solid" => "refractive_index_solid",
        other => return Err(format!("phase must be liquid or solid, got {other}")),
    };
    let t = sheet
        .row("substances", substance)
        .and_then(|r| r.get("optical"))
        .and_then(|o| o.get(key))
        .ok_or_else(|| format!("the sheet has no {key} for {substance}: give n (and k) instead"))?;
    let col = |c: &str| -> Result<Vec<f64>, String> {
        t.get(c)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
            .ok_or_else(|| format!("{substance} {key}: no {c} column"))
    };
    let (l, n, k) = (col("lambda_nm")?, col("n")?, col("k")?);
    if l.is_empty() || l.len() != n.len() || l.len() != k.len() {
        return Err(format!("{substance} {key}: columns differ in length"));
    }
    let nm = wavelength_m * 1e9;
    if nm <= l[0] {
        return Ok((n[0], k[0]));
    }
    for i in 1..l.len() {
        if nm <= l[i] {
            let f = (nm - l[i - 1]) / (l[i] - l[i - 1]);
            return Ok((
                n[i - 1] + f * (n[i] - n[i - 1]),
                k[i - 1] + f * (k[i] - k[i - 1]),
            ));
        }
    }
    let e = l.len() - 1;
    Ok((n[e], k[e]))
}
