//! Air optics for a planet's air (Rayleigh scattering), from the gas table
//! in the light pack and the blocks in `sse_atoms::air`: cross-section,
//! scattering per metre at the surface, column optical depth and scale
//! height, at chosen wavelengths and across the visible slices. The
//! renderer's sky reads this instead of its own table.

use crate::table::{GasDef, LightTable};
use crate::*;
use serde::{Deserialize, Serialize};
use sse_atoms::air as a;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Share {
    pub gas: String,
    /// Fraction by number (mole fraction).
    pub fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Air {
    pub address: String,
    pub gases: Vec<Share>,
    pub surface_pressure_pa: f64,
    pub surface_temperature_k: f64,
    pub gravity_m_s2: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AtWavelength {
    pub wavelength_m: f64,
    /// Mean cross-section per molecule, m².
    pub cross_section_m2: f64,
    /// Scattering per metre at the surface.
    pub beta_per_m: f64,
    /// Optical depth straight up from the surface.
    pub tau: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AirOptics {
    pub address: String,
    pub mean_molecular_mass_kg: f64,
    pub scale_height_m: f64,
    pub surface_density_m3: f64,
    pub at: Vec<AtWavelength>,
    /// The same at the geometric centre of each visible slice.
    pub visible_slices: Vec<(String, AtWavelength)>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

/// n − 1 of a gas at λ, at Loschmidt density.
fn refractivity(g: &GasDef, lambda_m: f64) -> f64 {
    let air_now = a::air_refractivity(lambda_m) * a::N_LOSCHMIDT / a::N_S_288;
    if g.dispersion == "peck-reeder" {
        return air_now;
    }
    let air_589 = a::air_refractivity(589.3e-9) * a::N_LOSCHMIDT / a::N_S_288;
    g.refractivity_589 * air_now / air_589
}

fn king(g: &GasDef, lambda_m: f64) -> f64 {
    let um = lambda_m * 1e6;
    let s2 = 1.0 / (um * um);
    g.king[0] + g.king[1] * s2 + g.king[2] * s2 * s2
}

/// Mean Rayleigh cross-section per molecule of the mix at λ.
pub fn cross_section(air: &Air, lambda_m: f64, table: &LightTable) -> Result<f64, String> {
    let gases = table.gases.as_ref().ok_or("the light table has no gases")?;
    let mut s = 0.0;
    for sh in &air.gases {
        let g = gases
            .list
            .iter()
            .find(|g| g.id == sh.gas)
            .ok_or_else(|| format!("unknown gas {}", sh.gas))?;
        s += sh.fraction
            * a::rayleigh_cross_section(
                lambda_m,
                refractivity(g, lambda_m),
                a::N_LOSCHMIDT,
                king(g, lambda_m),
            );
    }
    Ok(s)
}

pub fn optics(air: &Air, wavelengths_m: &[f64], table: &LightTable) -> Result<AirOptics, String> {
    let gases = table.gases.as_ref().ok_or("the light table has no gases")?;
    let total: f64 = air.gases.iter().map(|s| s.fraction).sum();
    if air.gases.is_empty() || (total - 1.0).abs() > 0.02 {
        return Err(format!(
            "{}: gas fractions must add to 1 (they add to {total:.4})",
            air.address
        ));
    }
    if !(air.surface_pressure_pa >= 0.0
        && air.surface_temperature_k > 0.0
        && air.gravity_m_s2 > 0.0)
    {
        return Err(format!(
            "{}: pressure must be zero or more, temperature and gravity positive",
            air.address
        ));
    }
    let mut mm = 0.0;
    for sh in &air.gases {
        let g = gases
            .list
            .iter()
            .find(|g| g.id == sh.gas)
            .ok_or_else(|| format!("unknown gas {}", sh.gas))?;
        mm += sh.fraction * g.molar_mass_g;
    }
    let m_kg = mm * a::AMU;
    let h = a::scale_height(air.surface_temperature_k, m_kg, air.gravity_m_s2);
    let n = air.surface_pressure_pa / (a::K_B * air.surface_temperature_k);
    let at_l = |l: f64| -> Result<AtWavelength, String> {
        let sigma = cross_section(air, l, table)?;
        Ok(AtWavelength {
            wavelength_m: l,
            cross_section_m2: sigma,
            beta_per_m: n * sigma,
            tau: a::column_optical_depth(air.surface_pressure_pa, sigma, m_kg, air.gravity_m_s2),
        })
    };
    let at: Vec<AtWavelength> = wavelengths_m
        .iter()
        .map(|&l| at_l(l))
        .collect::<Result<_, _>>()?;
    let visible_slices = table
        .visible_slices
        .iter()
        .map(|b| Ok((b.id.clone(), at_l(m::sqrt(b.from_m * b.to_m))?)))
        .collect::<Result<Vec<_>, String>>()?;
    let mut why = vec![format!(
        "Mean molecule {:.2} u; at {:.0} K and {:.3e} Pa there are {:.3e} molecules per m³, thinning by e every {:.1} km.",
        mm,
        air.surface_temperature_k,
        air.surface_pressure_pa,
        n,
        h / 1e3
    )];
    if let Some(x) = at
        .iter()
        .find(|x| (x.wavelength_m - 550e-9).abs() < 1e-9)
        .or(at.first())
    {
        why.push(format!(
            "At {:.0} nm light travels {:.3e} m before scattering, and straight up the air scatters {:.1} % of it (optical depth {:.4}). Blue scatters most, as λ⁻⁴.",
            x.wavelength_m * 1e9,
            1.0 / x.beta_per_m.max(1e-300),
            100.0 * (1.0 - m::exp(-x.tau)),
            x.tau
        ));
    }
    let mut references: Vec<Reference> = sse_atoms::decls_of(&[
        "sse.air.rayleigh_cross_section",
        "sse.air.dry_air_refractivity",
        "sse.air.column_optical_depth",
        "sse.air.scale_height",
        "sse.air.scattering_coefficient",
    ])
    .into_iter()
    .flat_map(|d| d.references)
    .collect();
    references.dedup();
    if let Some(r) = table.references.iter().find(|r| r.text.starts_with("Kaye")) {
        references.push(r.clone());
    }
    Ok(AirOptics {
        address: air.address.clone(),
        mean_molecular_mass_kg: m_kg,
        scale_height_m: h,
        surface_density_m3: n,
        at,
        visible_slices,
        why,
        stand_ins: vec![
            "isothermal air at the surface temperature for the scale height".into(),
            "gases other than dry air follow air's dispersion scaled to their 589 nm refractivity; King factors constant except N₂ and O₂".into(),
            "gravity taken at the surface, not at the column's mass-weighted height (about 0.2 % on Earth)".into(),
            "molecular (Rayleigh) scattering only: no haze, dust, cloud or absorption bands".into(),
        ],
        references,
    })
}
