//! The band set, visible slices, line table and sampling grid (pack
//! `provides.other.light`, base pack `packs/base-light`). Pack data, so a pack
//! can add lines or finer bands without code.

use crate::Reference;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_pack::PackManifest;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Band {
    pub id: String,
    pub name: String,
    pub from_m: f64,
    pub to_m: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineDef {
    pub id: String,
    pub name: String,
    pub species: String,
    /// Rest wavelength, m.
    pub at_m: f64,
    /// "air" or "vacuum".
    pub medium: String,
}

/// Photospheric absorption: each line's strength (0 to 1) at the listed temperatures.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Absorption {
    pub teff_k: Vec<f64>,
    #[serde(default)]
    pub note: String,
    pub lines: Vec<AbsorptionLine>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbsorptionLine {
    pub line: String,
    pub strength: Vec<f64>,
}

impl Absorption {
    /// Strength of each line at `teff`, interpolated in log temperature, held flat beyond the ends.
    pub fn at(&self, teff: f64) -> Vec<(&str, f64)> {
        let t = &self.teff_k;
        let x = m::ln(teff.max(1.0));
        self.lines
            .iter()
            .map(|l| {
                let v = if teff <= t[0] {
                    l.strength[0]
                } else if teff >= t[t.len() - 1] {
                    l.strength[t.len() - 1]
                } else {
                    let j = t
                        .iter()
                        .position(|&a| a > teff)
                        .unwrap_or(t.len() - 1)
                        .max(1);
                    let (x0, x1) = (m::ln(t[j - 1]), m::ln(t[j]));
                    let f = (x - x0) / (x1 - x0);
                    l.strength[j - 1] + f * (l.strength[j] - l.strength[j - 1])
                };
                (l.line.as_str(), v)
            })
            .collect()
    }
}

/// A gas's optical data for Rayleigh scattering.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GasDef {
    pub id: String,
    pub name: String,
    pub molar_mass_g: f64,
    /// n − 1 at 589.3 nm, 273.15 K, 101325 Pa.
    pub refractivity_589: f64,
    /// King factor a + b λ⁻² + c λ⁻⁴ (λ in µm).
    pub king: [f64; 3],
    /// "peck-reeder" (dry air, exact) or "scaled" (air's dispersion, scaled).
    pub dispersion: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gases {
    #[serde(default)]
    pub note: String,
    pub list: Vec<GasDef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub from_m: f64,
    pub to_m: f64,
    pub points: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightTable {
    pub bands: Vec<Band>,
    pub visible_slices: Vec<Band>,
    pub lines: Vec<LineDef>,
    pub grid: Grid,
    pub steps_per_decade: usize,
    pub disc_rings: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absorption: Option<Absorption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gases: Option<Gases>,
    #[serde(default)]
    pub references: Vec<Reference>,
}

pub const BASE_PACK: &str = include_str!("../../../packs/base-light/pack.json");

impl LightTable {
    /// The base pack's table.
    pub fn base() -> Self {
        let p = PackManifest::from_json(BASE_PACK).expect("base-light pack parses");
        Self::from_pack(&p).expect("base-light pack has a light table")
    }

    /// Read `provides.other.light` from a pack.
    pub fn from_pack(p: &PackManifest) -> Result<Self, String> {
        Self::from_packs(std::slice::from_ref(p))
    }

    /// The last pack in a stack that provides `light` wins.
    pub fn from_packs(stack: &[PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("light"))
            .ok_or("no pack provides a light table")?;
        let t: LightTable = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("light table: {e}"))?;
        t.check()?;
        Ok(t)
    }

    /// Bands must be in order and touch end to end; lines must be unique.
    pub fn check(&self) -> Result<(), String> {
        for w in self.bands.windows(2) {
            if w[0].to_m != w[1].from_m {
                return Err(format!("bands {} and {} do not meet", w[0].id, w[1].id));
            }
        }
        for b in self.bands.iter().chain(&self.visible_slices) {
            if !(b.from_m > 0.0 && b.to_m > b.from_m) {
                return Err(format!("band {} has a bad range", b.id));
            }
        }
        let mut ids: Vec<&str> = self.lines.iter().map(|l| l.id.as_str()).collect();
        ids.sort_unstable();
        if ids.windows(2).any(|w| w[0] == w[1]) {
            return Err("line ids repeat".into());
        }
        if let Some(a) = &self.absorption {
            if a.teff_k.len() < 2 || a.teff_k.windows(2).any(|w| w[1] <= w[0]) {
                return Err("absorption temperatures must rise, at least two".into());
            }
            for l in &a.lines {
                if l.strength.len() != a.teff_k.len() || self.line(&l.line).is_none() {
                    return Err(format!(
                        "absorption line {} is unknown or has the wrong number of strengths",
                        l.line
                    ));
                }
            }
        }
        if self.steps_per_decade < 8 || self.grid.points < 2 {
            return Err(
                "steps_per_decade must be at least 8 and the grid at least 2 points".into(),
            );
        }
        Ok(())
    }

    pub fn line(&self, id: &str) -> Option<&LineDef> {
        self.lines.iter().find(|l| l.id == id)
    }

    pub fn band(&self, id: &str) -> Option<&Band> {
        self.bands
            .iter()
            .chain(&self.visible_slices)
            .find(|b| b.id == id)
    }

    /// The sampling grid, log spaced.
    pub fn grid(&self) -> Vec<f64> {
        let (a, b) = (m::ln(self.grid.from_m), m::ln(self.grid.to_m));
        let n = self.grid.points - 1;
        (0..=n)
            .map(|i| m::exp(a + (b - a) * i as f64 / n as f64))
            .collect()
    }
}
