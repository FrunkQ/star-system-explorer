//! Light off layers of particles (haze and cloud): the colour a layer of
//! tholin, chromophore, sulphuric acid or ammonia ice sends back, from what
//! the particles are made of. Same idea as `gas` for molecules: nothing
//! here names a kind of world or haze; a layer is a material on the data
//! sheet, a particle size, an optical depth and how far forward it
//! scatters, and those are inputs.
//!
//! The material's refractive index is its sheet row
//! (`optical.refractive_index`, a `table` of `lambda_nm`, `n`, `k`, shape
//! agreed with the data sheet thread on 24 Nov). The physics is two
//! formula blocks in `packs/base-physics/formulas/light.json`:
//! `sse.light.particle_albedo` (anomalous diffraction, van de Hulst 1957:
//! extinction and absorption of one particle, hence its single-scattering
//! albedo) and `sse.light.slab_reflectance` (two-stream reflectance and
//! transmission of a layer). Layers stack by the adding method over a floor,
//! wavelength by wavelength, and the eye (`colour`) turns the result into a
//! colour. A wavelength outside the measured table takes the nearest
//! measured value, and says so.

use crate::colour::{colour_of, run, Colour};
use crate::table::Eye;
use serde::{Deserialize, Serialize};

/// A material's refractive index against wavelength, as its sheet row gives it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexTable {
    pub lambda_nm: Vec<f64>,
    pub n: Vec<f64>,
    pub k: Vec<f64>,
    #[serde(default)]
    pub src: String,
}

impl IndexTable {
    /// The row's `optical.refractive_index`, from the substances or the
    /// mixtures table. `Ok(None)`: the material is on the sheet but its
    /// index has not been looked up yet.
    pub fn from_sheet(sheet: &sse_reference::Reference, id: &str) -> Result<Option<Self>, String> {
        let row = sheet
            .row("substances", id)
            .or_else(|| sheet.row("mixtures", id))
            .ok_or_else(|| {
                format!("material {id}: the data sheet has no substance or mixture row {id}")
            })?;
        let Some(v) = row.get("optical").and_then(|o| o.get("refractive_index")) else {
            return Ok(None);
        };
        let t: IndexTable = serde_json::from_value(v.clone())
            .map_err(|e| format!("material {id}: refractive index table: {e}"))?;
        t.check().map_err(|e| format!("material {id}: {e}"))?;
        Ok(Some(t))
    }

    pub fn check(&self) -> Result<(), String> {
        let n = self.lambda_nm.len();
        if n == 0 || self.n.len() != n || self.k.len() != n {
            return Err(
                "the index table needs lambda_nm, n and k of one length, at least one row".into(),
            );
        }
        if self
            .lambda_nm
            .windows(2)
            .any(|w| w[1].partial_cmp(&w[0]) != Some(std::cmp::Ordering::Greater))
        {
            return Err("the index table's wavelengths must rise".into());
        }
        if self.k.iter().any(|k| !(k.is_finite() && *k >= 0.0))
            || self.n.iter().any(|n| !(n.is_finite() && *n > 0.0))
        {
            return Err("n must be more than zero and k zero or more".into());
        }
        Ok(())
    }

    /// n and k at a wavelength, linear between rows; `true` when the
    /// wavelength is outside the table and the nearest row was used.
    pub fn at(&self, nm: f64) -> (f64, f64, bool) {
        let l = &self.lambda_nm;
        let last = l.len() - 1;
        if nm <= l[0] {
            return (self.n[0], self.k[0], nm < l[0]);
        }
        if nm >= l[last] {
            return (self.n[last], self.k[last], nm > l[last]);
        }
        let i = l.partition_point(|x| *x <= nm) - 1;
        let f = (nm - l[i]) / (l[i + 1] - l[i]);
        (
            self.n[i] + f * (self.n[i + 1] - self.n[i]),
            self.k[i] + f * (self.k[i + 1] - self.k[i]),
            false,
        )
    }
}

/// One layer of particles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Particles {
    /// A substance or mixture id on the data sheet.
    pub material: String,
    pub radius_m: f64,
    /// Optical depth straight down at `at_nm`.
    pub optical_depth: f64,
    pub at_nm: f64,
    /// Asymmetry parameter g of the particles' scattering.
    pub asymmetry: f64,
}

/// What a stack of layers sends back.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reflected {
    /// Reflectance at each wavelength asked.
    pub reflectance: Vec<f64>,
    /// The reflected light's colour to the eye.
    pub colour: Colour,
    /// Linear sRGB of the reflected light over the light's own: the
    /// layer's colour as a reflectance a renderer multiplies by, 0 to 1.
    pub reflectance_rgb: [f64; 3],
    /// Plain words: tables stretched to their nearest row, materials whose
    /// index is not looked up yet (drawn as non-absorbing).
    pub said: Vec<String>,
    pub references: Vec<String>,
}

/// Linear sRGB without normalising (the eye's matrix applied to XYZ).
fn rgb(eye: &Eye, c: &Colour) -> [f64; 3] {
    let mut o = [0.0; 3];
    for (k, row) in eye.xyz_to_linear_srgb.iter().enumerate() {
        o[k] = row[0] * c.xyz[0] + row[1] * c.xyz[1] + row[2] * c.xyz[2];
    }
    o
}

/// The layers (bottom first) over a floor of one reflectance, under a
/// light given at the eye's wavelengths.
pub fn reflect(
    sheet: &sse_reference::Reference,
    eye: &Eye,
    layers: &[Particles],
    floor: f64,
    light: &[f64],
) -> Result<Reflected, String> {
    if !(0.0..=1.0).contains(&floor) {
        return Err(format!(
            "the floor's reflectance must lie between 0 and 1, got {floor}"
        ));
    }
    let tables = layers
        .iter()
        .map(|p| IndexTable::from_sheet(sheet, &p.material))
        .collect::<Result<Vec<_>, _>>()?;
    reflect_with(eye, layers, &tables, floor, light)
}

/// As [`reflect`], with each layer's index table given (None: not looked up).
pub fn reflect_with(
    eye: &Eye,
    layers: &[Particles],
    tables: &[Option<IndexTable>],
    floor: f64,
    light: &[f64],
) -> Result<Reflected, String> {
    let wl = eye.wavelengths_m();
    if light.len() != wl.len() || tables.len() != layers.len() {
        return Err(format!(
            "the light is needed at the eye's {} wavelengths, and one table per layer",
            wl.len()
        ));
    }
    let mut said: Vec<String> = vec![];
    for (p, t) in layers.iter().zip(tables) {
        if !(p.radius_m > 0.0 && p.optical_depth >= 0.0 && p.at_nm > 0.0) {
            return Err(format!("layer of {}: radius and wavelength must be more than zero, optical depth zero or more", p.material));
        }
        if t.is_none() {
            said.push(format!(
                "{}: no refractive index on the data sheet yet, so drawn as grey and not absorbing",
                p.material
            ));
        }
    }
    // A particle's extinction efficiency and single-scattering albedo at a
    // wavelength; a material with no index is grey (the same extinction at
    // every wavelength) and absorbs nothing.
    let optics = |t: &Option<IndexTable>,
                  p: &Particles,
                  l: f64,
                  stretched: &mut bool|
     -> Result<(f64, f64), String> {
        match t {
            Some(t) => {
                let (n, k, s) = t.at(l * 1e9);
                *stretched |= s;
                let a = run("sse.light.particle_albedo", &[p.radius_m, l, n, k])?;
                Ok((a[0], a[2]))
            }
            None => Ok((1.0, 1.0)),
        }
    };
    let mut stretched = vec![false; layers.len()];
    // Each layer's extinction at its reference wavelength, to scale its depth.
    let mut qe_ref = vec![];
    for (i, p) in layers.iter().enumerate() {
        let mut s = false;
        qe_ref.push(optics(&tables[i], p, p.at_nm * 1e-9, &mut s)?.0);
    }
    let mut reflectance = vec![];
    for l in &wl {
        let mut r_tot = floor;
        for (i, p) in layers.iter().enumerate() {
            let (qe, om) = optics(&tables[i], p, *l, &mut stretched[i])?;
            let tau = if qe_ref[i] > 0.0 {
                p.optical_depth * qe / qe_ref[i]
            } else {
                0.0
            };
            let s = run("sse.light.slab_reflectance", &[tau, om, p.asymmetry])?;
            let (r, t) = (s[0], s[1]);
            // Adding method: the layer over what lies below, light bouncing between them.
            r_tot = r + t * t * r_tot / (1.0 - r * r_tot).max(1e-12);
        }
        reflectance.push(r_tot.clamp(0.0, 1.0));
    }
    for (i, p) in layers.iter().enumerate() {
        if stretched[i] {
            said.push(format!(
                "{}: its index table does not cover all of visible light, so the nearest measured value is used beyond it",
                p.material
            ));
        }
    }
    let out: Vec<f64> = light.iter().zip(&reflectance).map(|(i, r)| i * r).collect();
    let c_in = colour_of(eye, light)?;
    let c_out = colour_of(eye, &out)?;
    let (a, b) = (rgb(eye, &c_in), rgb(eye, &c_out));
    let reflectance_rgb = [0, 1, 2].map(|k| {
        if a[k] > 0.0 {
            (b[k] / a[k]).clamp(0.0, 1.0)
        } else {
            0.0
        }
    });
    let mut references = vec![
        "van de Hulst, H. C. 1957, Light Scattering by Small Particles (anomalous diffraction). doi:10.1063/1.3060205".to_string(),
        "Meador, W. E. and Weaver, W. R. 1980, J. Atmos. Sci. 37, 630 (two-stream). doi:10.1175/1520-0469(1980)037<0630:TSATRT>2.0.CO;2".to_string(),
    ];
    for t in tables.iter().flatten() {
        if !t.src.is_empty() && !references.contains(&t.src) {
            references.push(t.src.clone());
        }
    }
    Ok(Reflected {
        reflectance,
        colour: c_out,
        reflectance_rgb,
        said,
        references,
    })
}
