//! ND2 part 2b: the limb law, per colour. How a disc's brightness and
//! colour change from its centre to its edge, from the same physics
//! everywhere and with no kind of body named:
//!
//! - **A disc that glows** (a star, a hot world): at slant μ you see down to
//!   optical depth about μ (Eddington-Barbier), where a grey atmosphere is
//!   at T⁴ = (3/4) T_eff⁴ (μ + 2/3) (block `sse.light.limb_temperature`). The
//!   light at each wavelength is the Planck function at that temperature, so
//!   the edge is darker and redder, more so in the blue.
//! - **A disc that is lit** (a planet, a moon): Minnaert's law
//!   μ₀^k μ^(k−1) (block `sse.light.reflected_limb`), the same at every
//!   wavelength unless a channel gives its own k.

use crate::colour::{colour_of, run, Colour};
use crate::table::Eye;
use serde::{Deserialize, Serialize};

/// One point across a disc.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LimbPoint {
    pub mu: f64,
    /// A glowing disc: the temperature of the layer seen there.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_k: Option<f64>,
    /// Brightness against the centre (glowing) or against a matt surface
    /// in full sun seen square on (lit), at each wavelength asked for.
    pub share: Vec<f64>,
    /// The colour to the eye there (glowing discs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<Colour>,
}

/// A glowing disc of effective temperature `teff_k`: brightness at each
/// `mu` and wavelength against the centre, and the colour to the eye.
pub fn glowing(
    teff_k: f64,
    mus: &[f64],
    wavelengths_m: &[f64],
    eye: &Eye,
) -> Result<Vec<LimbPoint>, String> {
    let t1 = run("sse.light.limb_temperature", &[teff_k, 1.0])?[0];
    let eye_wl = eye.wavelengths_m();
    let mut out = vec![];
    for &mu in mus {
        let t = run("sse.light.limb_temperature", &[teff_k, mu])?[0];
        let share = wavelengths_m
            .iter()
            .map(|&l| {
                let c = crate::planck(l, t1);
                if c > 0.0 {
                    crate::planck(l, t) / c
                } else {
                    0.0
                }
            })
            .collect();
        let light: Vec<f64> = eye_wl.iter().map(|&l| crate::planck(l, t)).collect();
        out.push(LimbPoint {
            mu,
            temperature_k: Some(t),
            share,
            colour: Some(colour_of(eye, &light)?),
        });
    }
    Ok(out)
}

/// A lit disc: Minnaert's law with one `k`, or one `k` per wavelength.
pub fn lit(k: &[f64], mu0: f64, mus: &[f64]) -> Result<Vec<LimbPoint>, String> {
    if k.is_empty() {
        return Err("Minnaert's law needs a k (one, or one per wavelength)".into());
    }
    let mut out = vec![];
    for &mu in mus {
        let mut share = vec![];
        for &kk in k {
            share.push(run("sse.light.reflected_limb", &[kk, mu0, mu])?[0]);
        }
        out.push(LimbPoint {
            mu,
            temperature_k: None,
            share,
            colour: None,
        });
    }
    Ok(out)
}

pub fn references() -> Vec<sse_flow::Reference> {
    let mut r: Vec<sse_flow::Reference> =
        sse_atoms::decls_of(&["sse.light.limb_temperature", "sse.light.reflected_limb"])
            .into_iter()
            .flat_map(|d| d.references)
            .collect();
    r.dedup_by(|a, b| a.text == b.text);
    r
}
