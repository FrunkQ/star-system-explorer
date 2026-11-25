//! Colour to the eye (ND2): the colour a person would see from any light,
//! as a fact the kernel publishes, so no screen keeps its own colour maths.
//!
//! The eye's response is pack data (`eye` in the light table: the CIE 1931
//! 2° observer as sums of lopsided Gaussian lobes, Wyman, Sloan and Shirley
//! 2013) and two blocks: `sse.light.eye_lobe` (one lobe's shape) and
//! `sse.light.srgb_encode` (the screen curve, IEC 61966-2-1). The light is
//! summed against the three responses, turned into linear sRGB by the
//! table's matrix, its brightest channel set to one (colour, not
//! brightness), and encoded for a screen.

use crate::table::Eye;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::Atom;

/// A formula block from the base packs, found once.
pub(crate) fn block(id: &str) -> &'static dyn Atom {
    sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .map(|a| a as &dyn Atom)
        .unwrap_or_else(|| panic!("base formula block {id} is missing"))
}

/// Run a formula block on numbers.
pub(crate) fn run(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    block(id)
        .eval(inputs, &[])
        .map_err(|e| format!("{id}: {e}"))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colour {
    /// CIE XYZ, in the light's own units times nanometres (only ratios matter).
    pub xyz: [f64; 3],
    /// CIE chromaticity x, y.
    pub chromaticity: [f64; 2],
    /// Linear sRGB with the brightest channel at one; negative channels
    /// (colours a screen cannot show) clipped to zero.
    pub linear_srgb: [f64; 3],
    /// Encoded for a screen, 0 to 255.
    pub srgb: [u8; 3],
    pub hex: String,
    /// True when a channel had to be clipped: the colour is more saturated
    /// than a screen can show.
    pub outside_screen: bool,
}

/// The eye's three responses at one wavelength.
pub fn response(eye: &Eye, lambda_m: f64) -> Result<[f64; 3], String> {
    let nm = lambda_m * 1e9;
    let sum = |lobes: &[[f64; 4]]| -> Result<f64, String> {
        let mut s = 0.0;
        for l in lobes {
            s += l[0] * run("sse.light.eye_lobe", &[nm, l[1], l[2], l[3]])?[0];
        }
        Ok(s)
    };
    Ok([sum(&eye.lobes.x)?, sum(&eye.lobes.y)?, sum(&eye.lobes.z)?])
}

/// The colour of light given as its value at each of the eye's wavelengths
/// (`Eye::wavelengths_m`, in order).
pub fn colour_of(eye: &Eye, light: &[f64]) -> Result<Colour, String> {
    let wl = eye.wavelengths_m();
    if light.len() != wl.len() {
        return Err(format!(
            "the eye needs the light at {} wavelengths, got {}",
            wl.len(),
            light.len()
        ));
    }
    let mut xyz = [0.0; 3];
    for (l, v) in wl.iter().zip(light) {
        if !(v.is_finite() && *v >= 0.0) {
            return Err(format!(
                "light must be zero or more at every wavelength, got {v} at {:.0} nm",
                l * 1e9
            ));
        }
        let r = response(eye, *l)?;
        for k in 0..3 {
            xyz[k] += v * r[k] * eye.step_nm;
        }
    }
    from_xyz(eye, xyz)
}

/// The colour of light that is all in narrow lines, each given as its
/// wavelength (m) and power (any unit): an aurora's or a nebula's glow.
/// Lines outside the eye's range add nothing.
pub fn colour_of_lines(eye: &Eye, lines: &[(f64, f64)]) -> Result<Colour, String> {
    let mut xyz = [0.0; 3];
    for (l, p) in lines {
        if !(p.is_finite() && *p >= 0.0) {
            return Err(format!(
                "a line's power must be zero or more, got {p} at {:.0} nm",
                l * 1e9
            ));
        }
        let nm = l * 1e9;
        if nm < eye.from_nm || nm > eye.to_nm {
            continue;
        }
        let r = response(eye, *l)?;
        for k in 0..3 {
            xyz[k] += p * r[k];
        }
    }
    from_xyz(eye, xyz)
}

/// Chromaticity, screen colour and hex from summed CIE XYZ.
fn from_xyz(eye: &Eye, xyz: [f64; 3]) -> Result<Colour, String> {
    let total = xyz[0] + xyz[1] + xyz[2];
    let chromaticity = if total > 0.0 {
        [xyz[0] / total, xyz[1] / total]
    } else {
        [0.0, 0.0]
    };
    let mx = &eye.xyz_to_linear_srgb;
    let mut rgb = [0.0; 3];
    for (k, row) in mx.iter().enumerate() {
        rgb[k] = row[0] * xyz[0] + row[1] * xyz[1] + row[2] * xyz[2];
    }
    let outside_screen = rgb.iter().any(|&v| v < 0.0);
    for v in &mut rgb {
        *v = v.max(0.0);
    }
    let top = rgb[0].max(rgb[1]).max(rgb[2]);
    if top > 0.0 {
        for v in &mut rgb {
            *v /= top;
        }
    }
    let mut srgb = [0u8; 3];
    for k in 0..3 {
        let s = run("sse.light.srgb_encode", &[rgb[k].min(1.0)])?[0];
        srgb[k] = m::round(s * 255.0) as u8;
    }
    let hex = format!("#{:02X}{:02X}{:02X}", srgb[0], srgb[1], srgb[2]);
    Ok(Colour {
        xyz,
        chromaticity,
        linear_srgb: rgb,
        srgb,
        hex,
        outside_screen,
    })
}

/// The references for colour.
pub fn references(eye: &Eye) -> Vec<sse_flow::Reference> {
    eye.sources
        .iter()
        .map(|s| {
            let url = s.split_whitespace().find(|w| w.starts_with("https://"));
            sse_flow::Reference {
                text: s.trim_end_matches(url.unwrap_or("")).trim_end().to_string(),
                doi: None,
                url: url.map(String::from),
            }
        })
        .collect()
}
