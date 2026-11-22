//! Magnitudes in photometric bands, dimmed and reddened by dust (night sky
//! design 4.3, stage Y2, light side). A sky draw asks this for every star it
//! keeps, so a fill star costs a few Planck evaluations: its photosphere as a
//! blackbody, its flux density at each band's effective wavelength, the AB
//! magnitude from that, the Vega magnitude by the band's offset, and the
//! dust's dimming in that band from `sse.light.dust_extinction`. A resolved
//! system goes through the same path with its full light description, so a
//! point in the sky and its system's card give the same magnitude.

use crate::observe::{flux_density, Target};
use crate::table::Passband;
use crate::*;
use sse_atoms::extinction::ccm_ratio;

/// The AB system's zero point, W m⁻² Hz⁻¹: 3631 Jy, from m = −2.5 log₁₀ f_ν − 48.60
/// in cgs (Oke and Gunn 1983, a definition).
pub const AB_ZERO: f64 = 3.631e-23;
pub const R_OKE83: &str = "Oke, J. B. and Gunn, J. E. 1983, Secondary standard stars for absolute spectrophotometry, ApJ 266, 713: the AB magnitude m = −2.5 log₁₀ f_ν − 48.60 (cgs). doi:10.1086/160817";
pub const R_BR07: &str = "Blanton, M. R. and Roweis, S. 2007, K-corrections and filter transformations in the ultraviolet, optical, and near infrared, AJ 133, 734: effective wavelengths and AB minus Vega offsets (Table 1). doi:10.1086/510127";
pub const R_CCM89: &str = "Cardelli, J. A., Clayton, G. C. and Mathis, J. S. 1989, The relationship between infrared, optical, and ultraviolet extinction, ApJ 345, 245. doi:10.1086/167900";

/// A star as a blackbody photosphere: temperature and either radius or luminosity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Photosphere {
    pub temperature_k: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub luminosity_w: Option<f64>,
}

impl Photosphere {
    pub fn radius(&self) -> Result<f64, String> {
        if self.temperature_k.is_nan() || self.temperature_k <= 0.0 {
            return Err("temperature must be positive".into());
        }
        match (self.radius_m, self.luminosity_w) {
            (Some(r), _) if r > 0.0 => Ok(r),
            (_, Some(l)) if l > 0.0 => {
                let t2 = self.temperature_k * self.temperature_k;
                Ok(m::sqrt(l / (4.0 * m::PI * SIGMA * t2 * t2)))
            }
            _ => Err("give a positive radius or luminosity".into()),
        }
    }
}

/// How the star is seen: distance and the dust in between.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sight {
    pub distance_m: f64,
    /// Dimming in the V band by dust along the sight line, magnitudes.
    #[serde(default)]
    pub a_v: f64,
    /// R_V of that dust (3.1 for diffuse dust).
    #[serde(default = "default_rv")]
    pub r_v: f64,
}

fn default_rv() -> f64 {
    3.1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Magnitude {
    pub band: String,
    pub m_vega: f64,
    pub m_ab: f64,
    /// Dimming by dust in this band, magnitudes (included in both magnitudes).
    pub extinction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colour {
    /// "B-V", "V-K" and so on (Vega system).
    pub name: String,
    pub value: f64,
    /// The part of it made by dust (colour excess).
    pub excess: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Photometry {
    pub magnitudes: Vec<Magnitude>,
    pub colours: Vec<Colour>,
}

/// AB magnitude of a flux density f_λ (W m⁻² m⁻¹) at wavelength λ.
pub fn ab_magnitude(f_lambda: f64, lambda_m: f64) -> f64 {
    let f_nu = f_lambda * lambda_m * lambda_m / C;
    if f_nu <= 0.0 {
        return f64::INFINITY;
    }
    -2.5 * m::log10(f_nu / AB_ZERO)
}

/// Photometry from any flux density function, the one path for fill stars
/// and resolved systems alike.
pub fn photometry_of(
    f_lambda: impl Fn(f64) -> f64,
    sight: &Sight,
    bands: &[&Passband],
) -> Photometry {
    let magnitudes: Vec<Magnitude> = bands
        .iter()
        .map(|b| {
            let ext = sight.a_v.max(0.0) * ccm_ratio(b.effective_m, sight.r_v);
            let ab = ab_magnitude(f_lambda(b.effective_m), b.effective_m) + ext;
            Magnitude {
                band: b.id.clone(),
                m_vega: ab - b.ab_minus_vega,
                m_ab: ab,
                extinction: ext,
            }
        })
        .collect();
    let find = |id: &str| magnitudes.iter().find(|x| x.band == id);
    let mut colours = vec![];
    for (a, b) in [
        ("U", "B"),
        ("B", "V"),
        ("V", "R"),
        ("V", "I"),
        ("J", "Ks"),
        ("V", "Ks"),
    ] {
        if let (Some(x), Some(y)) = (find(a), find(b)) {
            colours.push(Colour {
                name: format!("{a}-{b}"),
                value: x.m_vega - y.m_vega,
                excess: x.extinction - y.extinction,
            });
        }
    }
    Photometry {
        magnitudes,
        colours,
    }
}

/// The bands asked for (all the table's when `ids` is empty).
pub fn bands_of<'a>(table: &'a LightTable, ids: &[String]) -> Result<Vec<&'a Passband>, String> {
    let all = table
        .passbands
        .as_ref()
        .ok_or("no pack provides photometric passbands")?;
    if ids.is_empty() {
        return Ok(all.list.iter().collect());
    }
    ids.iter()
        .map(|i| table.passband(i).ok_or(format!("unknown passband {i}")))
        .collect()
}

/// A blackbody photosphere seen across a sight line.
pub fn star_photometry(
    s: &Photosphere,
    sight: &Sight,
    bands: &[&Passband],
) -> Result<Photometry, String> {
    let r = s.radius()?;
    if sight.distance_m.is_nan() || sight.distance_m <= 0.0 {
        return Err("distance must be positive".into());
    }
    let dil = (r / sight.distance_m) * (r / sight.distance_m);
    let t = s.temperature_k;
    Ok(photometry_of(|l| m::PI * planck(l, t) * dil, sight, bands))
}

/// A resolved target (every component's full light, Doppler shifted) seen
/// across a sight line; the distance is the target's own.
pub fn target_photometry(
    t: &Target,
    a_v: f64,
    r_v: f64,
    bands: &[&Passband],
    table: &LightTable,
) -> Photometry {
    let sight = Sight {
        distance_m: t.distance_m,
        a_v,
        r_v,
    };
    photometry_of(|l| flux_density(t, l, table), &sight, bands)
}

pub fn references() -> Vec<Reference> {
    [R_OKE83, R_BR07, R_CCM89, R_PLANCK]
        .iter()
        .map(|t| Reference::parse(t))
        .collect()
}
