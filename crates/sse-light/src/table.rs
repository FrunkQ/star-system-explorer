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
    /// The element or substance that gives it off, by its id on the master
    /// data sheet (lower-case symbols for elements: "o", "h"; substances as
    /// they are: "n2", "co"). Absent for a line no sheet row stands for yet
    /// (annihilation, TiO, CH).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matter: Option<String>,
    /// How the line shines in an aurora (MF9 part 2b): absent for a line
    /// no falling electron excites.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aurora: Option<AuroraLine>,
    /// How the line shines in a nebula's glowing gas (O4 part 1): absent for
    /// a line electron knocks in a nebula do not make.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nebular: Option<NebularLine>,
}

/// The gases that take up ionising photons, each from its ionisation edge
/// upwards until the next gas's edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhotonTakers {
    pub list: Vec<PhotonTaker>,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhotonTaker {
    /// Element id on the master data sheet.
    pub matter: String,
    /// Its ionisation edge, eV.
    pub from_ev: f64,
    #[serde(default)]
    pub note: String,
}

/// A forbidden line excited by electron collisions in ionised gas: which ion
/// gives it (the photon energies that make and strip that ion), the atomic
/// data for the knock, and the share of the upper level's decays it takes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NebularLine {
    /// Photon energy that makes this ion from the one below, eV.
    pub makes_ev: f64,
    /// Photon energy that strips it to the next ion up, eV.
    pub strips_ev: f64,
    /// Effective collision strength from the lower term to the upper level.
    pub collision_strength: f64,
    pub lower_weight: f64,
    pub upper_weight: f64,
    /// Einstein A of the upper level's decay, s⁻¹; 0 when the line's upper
    /// level is not knocked down before it shines at the densities modelled
    /// (no de-excitation worked).
    #[serde(default)]
    pub rate_s: f64,
    /// Energy of the upper level above the lower term, cm⁻¹, when the line
    /// does not end on the term's centre (its excitation energy); 0 to use
    /// the line's own wavelength.
    #[serde(default)]
    pub upper_cm: f64,
    /// Share of the upper level's decays that give this line.
    #[serde(default = "one")]
    pub share: f64,
    /// For an ion that the starlight cannot use up (sulphur's photons are
    /// taken by hydrogen), its share against the next ion up from the local
    /// balance of stripping and rejoining, in place of photon counting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<IonBalance>,
    #[serde(default)]
    pub source: String,
}

/// The data for an ion's local balance with the next ion up.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IonBalance {
    /// Verner et al. 1996 fit to the ion's photoionisation cross-section.
    pub fit: CrossSectionFit,
    /// Rate coefficient for the next ion up catching an electron back, m³/s.
    pub recombination_m3_s: f64,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrossSectionFit {
    pub e0_ev: f64,
    pub sigma0_m2: f64,
    pub ya: f64,
    pub p: f64,
    pub yw: f64,
    pub y0: f64,
    pub y1: f64,
    /// The fit holds up to the next inner-shell edge, eV.
    pub max_ev: f64,
}

fn one() -> f64 {
    1.0
}

/// A line's part in aurorae: how long its upper state lives, how readily a
/// collision puts it out, and which gases give it off when electrons rain in.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuroraLine {
    /// Radiative lifetime of the upper state, s.
    pub lifetime_s: f64,
    /// Rate coefficient for collisions with the air taking the energy away
    /// before it shines, m³/s (0 for a prompt line).
    #[serde(default)]
    pub quench_m3_s: f64,
    /// The gases that give it off, each with the share of the falling
    /// electrons' energy that goes into the line (before quenching) when
    /// that gas is all of the air.
    pub from: Vec<AuroraParent>,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuroraParent {
    /// Element or substance id on the master data sheet.
    pub matter: String,
    #[serde(rename = "yield")]
    pub energy_share: f64,
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

/// A gas's optical data for Rayleigh scattering. The light pack names the
/// gas and its dispersion; the numbers are the master data sheet's (MP22):
/// `sheet` is the sheet row (a substance, or a mixture by volume such as
/// dry air), the gas's own id when absent. A pack may still type them, and a
/// typed number wins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GasDef {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
    #[serde(default)]
    pub molar_mass_g: f64,
    /// n − 1 at 589.3 nm, 273.15 K, 101325 Pa.
    #[serde(default)]
    pub refractivity_589: f64,
    /// King factor a + b λ⁻² + c λ⁻⁴ (λ in µm).
    #[serde(default)]
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

/// A photometric band for magnitudes (U, B, V, ... Ks).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Passband {
    pub id: String,
    pub name: String,
    /// Effective wavelength, m.
    pub effective_m: f64,
    /// m_AB − m_Vega for this band.
    pub ab_minus_vega: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Passbands {
    #[serde(default)]
    pub note: String,
    pub list: Vec<Passband>,
}

/// How the eye turns light into colour: the CIE 1931 2° observer as sums of
/// lopsided Gaussian lobes `[weight, centre nm, blue width nm, red width nm]`
/// (Wyman, Sloan and Shirley 2013), summed over `from_nm..=to_nm` in
/// `step_nm`, and the matrix from XYZ to linear sRGB (IEC 61966-2-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eye {
    #[serde(default)]
    pub note: String,
    pub lobes: EyeLobes,
    pub from_nm: f64,
    pub to_nm: f64,
    pub step_nm: f64,
    pub xyz_to_linear_srgb: [[f64; 3]; 3],
    #[serde(default)]
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EyeLobes {
    pub x: Vec<[f64; 4]>,
    pub y: Vec<[f64; 4]>,
    pub z: Vec<[f64; 4]>,
}

impl Eye {
    /// The wavelengths the eye is summed over, m.
    pub fn wavelengths_m(&self) -> Vec<f64> {
        let n = ((self.to_nm - self.from_nm) / self.step_nm + 0.5) as usize;
        (0..=n)
            .map(|i| (self.from_nm + self.step_nm * i as f64) * 1e-9)
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub from_m: f64,
    pub to_m: f64,
    pub points: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightTable {
    /// The fixed band set. Read from the master data sheet's Bands table
    /// (its `light` rows, RK10) when the pack gives none; a pack that
    /// gives its own list overrides the sheet.
    #[serde(default)]
    pub bands: Vec<Band>,
    pub visible_slices: Vec<Band>,
    pub lines: Vec<LineDef>,
    /// Which gas takes a nebula's ionising photons in each range of photon
    /// energy (O4 part 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photon_takers: Option<PhotonTakers>,
    pub grid: Grid,
    pub steps_per_decade: usize,
    pub disc_rings: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absorption: Option<Absorption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gases: Option<Gases>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passbands: Option<Passbands>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eye: Option<Eye>,
    #[serde(default)]
    pub references: Vec<Reference>,
}

pub const BASE_PACK: &str = include_str!("../../../packs/base-light/pack.json");

/// A gas's molar mass, refractivity and King factor from its sheet row,
/// where the pack left them out.
fn fill_from_sheet(g: &mut GasDef, sheet: &sse_reference::Reference) -> Result<(), String> {
    let id = g.sheet.clone().unwrap_or_else(|| g.id.clone());
    let row = sheet
        .row("substances", &id)
        .or_else(|| sheet.row("mixtures", &id))
        .ok_or_else(|| format!("gas {}: no sheet row {id}", g.id))?;
    let optical = row.get("optical").and_then(|o| o.as_object());
    if g.molar_mass_g == 0.0 {
        g.molar_mass_g = sheet
            .gas_molar_mass_kg_mol(&id)
            .ok_or_else(|| format!("gas {}: the sheet has no molar mass for {id}", g.id))?
            * 1000.0;
    }
    if g.refractivity_589 == 0.0 {
        g.refractivity_589 = optical
            .and_then(|o| o.get("refractivity_589nm")?.get("v")?.as_f64())
            .ok_or_else(|| format!("gas {}: the sheet has no refractivity for {id}", g.id))?;
    }
    if g.king == [0.0; 3] {
        let c = optical
            .and_then(|o| o.get("king_factor")?.get("coeffs")?.as_array().cloned())
            .ok_or_else(|| format!("gas {}: the sheet has no King factor for {id}", g.id))?;
        for (k, v) in g.king.iter_mut().zip(c) {
            *k = v.as_f64().unwrap_or(0.0);
        }
    }
    Ok(())
}

/// The light rows of the sheet's Bands table (RK10), in order of wavelength.
pub fn bands_from_sheet(sheet: &sse_reference::Reference) -> Result<Vec<Band>, String> {
    let num = |row: &serde_json::Map<String, serde_json::Value>, k: &str| {
        row.get(k)
            .and_then(|v| v.get("v").and_then(|x| x.as_f64()).or_else(|| v.as_f64()))
    };
    let mut out = vec![];
    for (id, row) in sheet.tables.get("bands").into_iter().flatten() {
        if row.get("kind").and_then(|k| k.as_str()) != Some("light") {
            continue;
        }
        let (Some(from_m), Some(to_m)) = (num(row, "from_m"), num(row, "to_m")) else {
            return Err(format!("sheet band {id} has no from_m or to_m"));
        };
        let name = row
            .get("names")
            .and_then(|n| n.get("physics"))
            .and_then(|n| n.as_str())
            .unwrap_or(id)
            .to_string();
        out.push(Band {
            id: id.clone(),
            name,
            from_m,
            to_m,
        });
    }
    if out.is_empty() {
        return Err("the sheet has no light bands (Bands table, RK10)".into());
    }
    out.sort_by(|a, b| a.from_m.total_cmp(&b.from_m));
    Ok(out)
}

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
        let mut t: LightTable = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("light table: {e}"))?;
        let sheet = || {
            sse_reference::Reference::from_packs(stack)
                .ok()
                .filter(|r| r.row("constants", "c").is_some())
                .unwrap_or_else(sse_reference::Reference::base)
        };
        if t.bands.is_empty() {
            t.bands = bands_from_sheet(&sheet())?;
        }
        if let Some(g) = t.gases.as_mut() {
            let sheet = sheet();
            for gas in &mut g.list {
                fill_from_sheet(gas, &sheet)?;
            }
        }
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
        if let Some(p) = &self.passbands {
            let mut ids: Vec<&str> = p.list.iter().map(|b| b.id.as_str()).collect();
            ids.sort_unstable();
            if ids.windows(2).any(|w| w[0] == w[1]) {
                return Err("passband ids repeat".into());
            }
            if let Some(b) = p
                .list
                .iter()
                .find(|b| b.effective_m.is_nan() || b.effective_m <= 0.0)
            {
                return Err(format!("passband {} needs a positive wavelength", b.id));
            }
        }
        if let Some(e) = &self.eye {
            if !(e.step_nm > 0.0 && e.to_nm > e.from_nm && e.from_nm > 0.0) {
                return Err("the eye's wavelengths need a positive step and a rising range".into());
            }
            for l in e.lobes.x.iter().chain(&e.lobes.y).chain(&e.lobes.z) {
                if !(l[2] > 0.0 && l[3] > 0.0) {
                    return Err("each eye lobe needs positive widths".into());
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

    pub fn passband(&self, id: &str) -> Option<&Passband> {
        self.passbands.as_ref()?.list.iter().find(|b| b.id == id)
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
