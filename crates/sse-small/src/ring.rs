//! One ring per world (track SB, VP-R3 with the grain colours VP-R2 reads).
//!
//! A world's ring is one object that describes its own structure: zones
//! (catalogued bands, authored bands, or seeded ones for a generated ring),
//! the marks its moons leave (edges, gaps, density-wave trains), optical
//! depth varying with radius inside every zone, and the colour of its grains
//! from what they are made of. [`structure`] is a pure function of the ask:
//! the same host, moons, ring and `variant` give the same ring on every
//! device. Variant 0 is the generator's own pick; other variants are other
//! plausible wholes with the same inputs. Marks made by moons are the same in
//! every variant: only the seeded part (generated zones, the fine structure)
//! changes.
//!
//! Grades (visual polish design section 3):
//! - fact: resonance radii (first-order inner Lindblad resonances, with the
//!   world's J2 when known), each resonance's torque against the ring's
//!   viscous torque (Tajeddine et al. 2017 eqs. 5 and 6), the gap an embedded
//!   moon clears, the density wave's wavelength from the dispersion relation
//!   (Shu 1984) and its viscous damping, the edge waves beside a gap
//!   (wavelength 3 pi times the half-width, Tiscareno 2013), the zones' mass
//!   from their optical depth and grain size, and the grains' colour mixed
//!   from the sheet's materials;
//! - drawn from the numbers: the fine structure inside each zone (a seeded
//!   fractal whose spread is the zone's measured range of optical depth, or a
//!   share of its depth when no range is on record), wave amplitudes.
//!
//! Stand-ins are in docs/fudges.md (rows SB-ring-*).

use crate::settings::RingSettings;
use crate::Makeup;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use sse_reference::Reference;

pub const REF_TAJEDDINE17: &str = "Tajeddine, R. et al. 2017, ApJS 232, 28: the resonant torque of a first-order resonance (eq. 5, f about 8.6 m^2) against the ring's viscous torque (eq. 6) sets the A and B rings' outer edges";
pub const REF_SHU84: &str = "Shu, F. H. 1984, in Planetary Rings (Greenberg and Brahic, eds.), 513: density waves at Lindblad resonances, wavenumber k = D (r - r_L) / (2 pi G Sigma r_L) with r dD/dr = 3 (m - 1) Omega^2";
pub const REF_TISCARENO13: &str = "Tiscareno, M. S. 2013, in Planets, Stars and Stellar Systems vol. 3, 309: edge waves beside an embedded moon repeat every 3 pi times the gap's half-width";
pub const REF_HAPKE: &str = "Hapke, B. 2012, Theory of Reflectance and Emittance Spectroscopy (2nd ed.): single-scattering albedo of an equivalent slab from n and k, mixed by cross-section; bihemispherical reflectance r = (1 - gamma) / (1 + gamma), gamma = sqrt(1 - w)";

/// The world the ring goes round.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct RingHost {
    pub mass_kg: f64,
    pub radius_m: f64,
    /// Its second zonal harmonic, when on record.
    #[serde(default)]
    pub j2: Option<f64>,
    /// Else J2 is estimated from its flattening and spin (Darwin-Radau to first order).
    #[serde(default)]
    pub flattening: Option<f64>,
    #[serde(default)]
    pub rotation_period_s: Option<f64>,
}

/// A body whose gravity shapes the ring: a moon round a world (or, later, a planet round a star's belt).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Sculptor {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub a_m: f64,
    pub mass_kg: f64,
}

impl Sculptor {
    /// What marks name it by: the id, or the name when no id was given.
    fn key(&self) -> String {
        if self.id.is_empty() {
            self.name.clone()
        } else {
            self.id.clone()
        }
    }
}

/// A zone the ask gives: a catalogued band, or one a GM authored.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ZoneIn {
    #[serde(default)]
    pub name: Option<String>,
    pub inner_m: f64,
    pub outer_m: f64,
    #[serde(default)]
    pub tau: Option<f64>,
    #[serde(default)]
    pub tau_range: Option<[f64; 2]>,
    #[serde(default)]
    pub albedo: Option<f64>,
    /// Share of the ring's mass (authored zones; renormalised so the ring's mass holds).
    #[serde(default)]
    pub share: Option<f64>,
    #[serde(default)]
    pub makeup: Option<Makeup>,
    /// The sheet's id for what dirties this zone's ice (else the ring's, else the pack's).
    #[serde(default)]
    pub contaminant: Option<String>,
    /// Size of the contaminant's grains, m (else the pack's): fine pigment is redder than coarse.
    #[serde(default)]
    pub contaminant_grain_m: Option<f64>,
    /// "catalogued" or "authored".
    #[serde(default)]
    pub source: Option<String>,
}

/// The ring's own inputs.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct RingIn {
    pub inner_m: f64,
    pub outer_m: f64,
    #[serde(default)]
    pub mass_kg: Option<f64>,
    #[serde(default)]
    pub makeup: Option<Makeup>,
    #[serde(default)]
    pub grain_radius_m: Option<f64>,
    #[serde(default)]
    pub contaminant: Option<String>,
    #[serde(default)]
    pub zones: Vec<ZoneIn>,
}

/// Everything [`structure`] reads.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct RingAsk {
    /// Seeds the drawn parts: usually the host's address.
    pub key: String,
    #[serde(default)]
    pub variant: u64,
    pub host: RingHost,
    #[serde(default)]
    pub sculptors: Vec<Sculptor>,
    pub ring: RingIn,
    #[serde(default)]
    pub samples: Option<usize>,
    /// Sample only this span (closer views ask for finer detail over less ring).
    #[serde(default)]
    pub window: Option<[f64; 2]>,
}

/// Why a zone is there.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cause {
    /// "base", "seeded", "catalogued", "authored", "resonance" or "embedded".
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sculptor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m: Option<u32>,
}

/// One zone of the ring.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Zone {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub inner_m: f64,
    pub outer_m: f64,
    pub mass_kg: f64,
    /// Normal optical depth (the zone's mean; the profile varies it).
    pub tau: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tau_range: Option<[f64; 2]>,
    pub sigma_kg_m2: f64,
    /// Reflectance at 550 nm of a layer of its grains.
    pub albedo: f64,
    /// Linear reflectance at the pack's red, green and blue bands.
    pub rgb: [f64; 3],
    /// Share of the grains' cross-section that is contaminant.
    pub dark_share: f64,
    pub contaminant: String,
    /// Share of its optical depth that is micron dust (forward scattering); the rest is metre-sized particles.
    pub dust_share: f64,
    pub cause: Cause,
    /// True when its depth or albedo is a fallback, not on record.
    pub assumed: bool,
}

/// How the ring's light turns with the angle between sun and eye: the drawing reads these (rings.ts `ringLook`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Phase {
    pub back_g: f64,
    pub forward_g: f64,
    pub surge_b0: f64,
    pub surge_width_deg: f64,
}

/// A mark a sculptor leaves.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Mark {
    /// "edge" (the resonance holds the ring's edge), "wave" (a density-wave train) or "gap" (an embedded moon's).
    pub kind: String,
    /// The sculptor's id from the ask (its name when the ask gave no id): ids survive a GM's renaming.
    pub sculptor: String,
    /// The sculptor's name, for the words.
    pub sculptor_name: String,
    /// The resonance's m (m:m-1); 0 for a gap.
    pub m: u32,
    pub r_m: f64,
    pub half_width_m: f64,
    /// Resonant torque over the ring's viscous torque (a gap: infinite, sent as null).
    pub ratio: Option<f64>,
    /// How much of the ring it clears or modulates there, 0 to 1.
    pub depth: f64,
    /// Waves: the wavelength one wavelength out from the resonance, and the distance over which the train dies away.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wave_lambda_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damping_m: Option<f64>,
    /// Gaps: the edge waves' wavelength along the ring and their radial amplitude.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_wave_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_amplitude_m: Option<f64>,
}

/// The ring sampled across a span: optical depth per sample and which zone holds it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub lo_m: f64,
    pub hi_m: f64,
    pub tau: Vec<f64>,
    /// Index into the zones; -1 where no zone is (a gap, or outside the ring).
    pub zone: Vec<i32>,
}

/// One ring, whole.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RingSystem {
    pub inner_m: f64,
    pub outer_m: f64,
    pub mass_kg: f64,
    pub j2: f64,
    pub variant: u64,
    pub zones: Vec<Zone>,
    pub marks: Vec<Mark>,
    pub phase: Phase,
    pub profile: Profile,
    pub why: Vec<String>,
}

/// The share of a zone's depth that is dust: all of it below `dusty_below_tau`, the clean share above
/// `clean_above_tau`, and between them a straight line in log depth.
fn dust_share(tau: f64, p: &crate::settings::RingPhaseSettings) -> f64 {
    if tau <= p.dusty_below_tau {
        return 1.0;
    }
    if tau >= p.clean_above_tau {
        return p.dust_share_clean;
    }
    let f = m::ln(tau / p.dusty_below_tau) / m::ln(p.clean_above_tau / p.dusty_below_tau);
    1.0 + (p.dust_share_clean - 1.0) * f
}

const STREAM: &str = "small.ring.v1";

fn interp(l: &[f64], v: &[f64], x: f64) -> Option<f64> {
    if l.is_empty() || l.len() != v.len() {
        return None;
    }
    if x <= l[0] {
        return Some(v[0]);
    }
    for i in 0..l.len() - 1 {
        if x <= l[i + 1] {
            let f = (x - l[i]) / (l[i + 1] - l[i]);
            return Some(v[i] + (v[i + 1] - v[i]) * f);
        }
    }
    v.last().copied()
}

fn nums(j: Option<&serde_json::Value>) -> Vec<f64> {
    j.and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
        .unwrap_or_default()
}

/// Hapke: single-scattering albedo of an isotropic layer whose bihemispherical reflectance is `r`.
pub fn w_of_reflectance(r: f64) -> f64 {
    let r = r.clamp(0.0, 0.999_999);
    let g = (1.0 - r) / (1.0 + r);
    1.0 - g * g
}

/// Hapke: the bihemispherical reflectance of a layer of single-scattering albedo `w`.
pub fn reflectance_of_w(w: f64) -> f64 {
    let g = m::sqrt((1.0 - w).clamp(0.0, 1.0));
    (1.0 - g) / (1.0 + g)
}

/// Hapke's equivalent slab: a grain's single-scattering albedo from its refractive index n + ik at `nm`, size `d_m`.
pub fn slab_w(n: f64, k: f64, nm: f64, d_m: f64) -> f64 {
    let se = ((n - 1.0) * (n - 1.0) + k * k) / ((n + 1.0) * (n + 1.0) + k * k) + 0.05;
    let si = 1.0 - 4.0 / (n * (n + 1.0) * (n + 1.0));
    let alpha = 4.0 * m::PI * k / (nm * 1e-9);
    let th = m::exp(-alpha * d_m);
    (se + (1.0 - se) * (1.0 - si) * th / (1.0 - si * th)).clamp(0.0, 1.0)
}

/// A material's single-scattering albedo at the three bands, read with the sheet's Look up: from its refractive
/// index (equivalent slab of `d_m`), else its reflectance table, else a substance's condensed colour.
pub fn material_w(sheet: &Reference, id: &str, bands: &[f64; 3], d_m: f64) -> Option<[f64; 3]> {
    let row = sheet
        .row("mixtures", id)
        .or_else(|| sheet.row("substances", id))?;
    let o = row.get("optical")?;
    if let Some(ri) = o.get("refractive_index") {
        let l = nums(ri.get("lambda_nm"));
        let n = nums(ri.get("n"));
        let k = nums(ri.get("k"));
        if !l.is_empty() && n.len() == l.len() && k.len() == l.len() {
            let mut out = [0.0; 3];
            for (i, b) in bands.iter().enumerate() {
                out[i] = slab_w(interp(&l, &n, *b)?, interp(&l, &k, *b)?.max(0.0), *b, d_m);
            }
            return Some(out);
        }
    }
    if let Some(t) = o.get("reflectance") {
        let l = nums(t.get("lambda_nm"));
        let r = nums(t.get("r"));
        let mut out = [0.0; 3];
        for (i, b) in bands.iter().enumerate() {
            out[i] = w_of_reflectance(interp(&l, &r, *b)?);
        }
        return Some(out);
    }
    let c = nums(o.get("condensed_colour_rgb").and_then(|c| c.get("v")));
    if c.len() == 3 {
        return Some([
            w_of_reflectance(c[0]),
            w_of_reflectance(c[1]),
            w_of_reflectance(c[2]),
        ]);
    }
    None
}

/// The rest of the ring's numbers: the ice's and a contaminant's single-scattering albedos.
struct Optics {
    ice: [f64; 3],
}

fn contaminant_w(
    sheet: &Reference,
    s: &RingSettings,
    id: &str,
    grain_m: f64,
    why: &mut Vec<String>,
) -> [f64; 3] {
    material_w(sheet, id, &s.bands_nm, grain_m).unwrap_or_else(|| {
        why.push(format!(
            "the data sheet holds no optics for {id}, so its grains are drawn as the pack's {}",
            s.contaminant
        ));
        material_w(sheet, &s.contaminant, &s.bands_nm, s.contaminant_grain_m).unwrap_or([0.1; 3])
    })
}

/// The share of cross-section that is contaminant to give an albedo (550 nm) of `albedo`.
fn dark_share_for(ice: f64, dark: f64, albedo: f64) -> f64 {
    let wt = w_of_reflectance(albedo);
    if (ice - dark).abs() < 1e-12 {
        return 0.0;
    }
    ((ice - wt) / (ice - dark)).clamp(0.0, 1.0)
}

/// Cross-section share of everything but ice for a makeup (grains of one size: share of volume).
fn dark_share_of(mk: &Makeup, rho: &[f64; 4]) -> f64 {
    let v = [
        mk.metal / rho[0],
        mk.rock / rho[1],
        mk.carbon / rho[2],
        mk.ice / rho[3],
    ];
    let t: f64 = v.iter().sum();
    if t > 0.0 {
        1.0 - v[3] / t
    } else {
        0.0
    }
}

fn mix_rgb(ice: &[f64; 3], dark: &[f64; 3], f: f64) -> [f64; 3] {
    let mut out = [0.0; 3];
    for i in 0..3 {
        out[i] = reflectance_of_w((1.0 - f) * ice[i] + f * dark[i]);
    }
    out
}

/// The world's J2: on record, else from its flattening and spin, J2 = (2f - q) / 3 with q = w^2 R^3 / GM.
fn j2_of(h: &RingHost, g: f64) -> f64 {
    if let Some(j) = h.j2 {
        return j.max(0.0);
    }
    if let (Some(f), Some(p)) = (h.flattening, h.rotation_period_s) {
        if p > 0.0 && h.mass_kg > 0.0 {
            let w = 2.0 * m::PI / p;
            let q = w * w * h.radius_m * h.radius_m * h.radius_m / (g * h.mass_kg);
            return ((2.0 * f - q) / 3.0).max(0.0);
        }
    }
    0.0
}

struct Grav {
    gm: f64,
    r: f64,
    j2: f64,
}

impl Grav {
    fn omega(&self, a: f64) -> f64 {
        let x = self.r / a;
        m::sqrt(self.gm / (a * a * a) * (1.0 + 1.5 * self.j2 * x * x))
    }
    fn kappa(&self, a: f64) -> f64 {
        let x = self.r / a;
        m::sqrt(self.gm / (a * a * a) * (1.0 - 1.5 * self.j2 * x * x).max(0.0))
    }
    /// Where m (Omega - n_s) = kappa: the m:m-1 inner Lindblad resonance of a moon at `a_s`.
    fn ilr(&self, a_s: f64, mm: u32) -> f64 {
        let ns = self.omega(a_s);
        let f = |r: f64| mm as f64 * (self.omega(r) - ns) - self.kappa(r);
        let mut lo = self.r.max(1.0) * 0.5;
        let mut hi = a_s;
        if f(lo) <= 0.0 {
            return a_s * m::pow((mm as f64 - 1.0) / mm as f64, 2.0 / 3.0);
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if f(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// A seeded fractal along the radius, about -1 to 1, with octaves from `scale_m` down; only octaves whose cells are
/// at least two samples wide are added (finer ones average to zero over a sample), and the sum is normalised over all
/// of them, so a closer look keeps the coarse shape and adds detail.
fn fractal(d: &Draws, salt: u64, r: f64, s: &RingSettings, spacing: f64) -> f64 {
    let st = &s.structure;
    let mut sum = 0.0;
    let mut norm = 0.0;
    let mut amp = 1.0;
    let mut cell = st.scale_m;
    for o in 0..st.octaves {
        norm += amp;
        if cell >= 2.0 * spacing {
            let x = r / cell;
            let i = m::floor(x);
            let t = smooth(x - i);
            let at = |c: f64| {
                let k = (salt << 40) ^ ((o as u64) << 34) ^ ((c as i64 as u64) & 0x3_ffff_ffff);
                2.0 * d.unit(k) - 1.0
            };
            sum += amp * (at(i) * (1.0 - t) + at(i + 1.0) * t);
        }
        amp *= m::pow(0.5, st.hurst);
        cell *= 0.5;
    }
    if norm > 0.0 {
        sum / norm * 2.0
    } else {
        0.0
    }
}

/// Mean grain density, kg/m^3: the makeup's grains at the pack's porosity.
fn particle_density(mk: &Makeup, rho: &[f64; 4], porosity: f64) -> f64 {
    let v = mk.metal / rho[0] + mk.rock / rho[1] + mk.carbon / rho[2] + mk.ice / rho[3];
    let solid = if v > 0.0 { 1.0 / v } else { rho[3] };
    solid * (1.0 - porosity)
}

fn area(inner: f64, outer: f64) -> f64 {
    m::PI * (outer * outer - inner * inner)
}

fn km(x: f64) -> String {
    format!("{:.0} km", x / 1000.0)
}

/// The ring, whole, from its ask. `rho` are the grain densities of metal, rock, carbon and ice (the sheet's).
pub fn structure(
    ask: &RingAsk,
    s: &RingSettings,
    rho: &[f64; 4],
    seed: &Seed,
    sheet: &Reference,
) -> Result<RingSystem, String> {
    let h = &ask.host;
    if !(h.mass_kg > 0.0 && h.radius_m > 0.0) {
        return Err("the ring's world needs a mass and a radius".into());
    }
    let mut why: Vec<String> = vec![];
    let ring = &ask.ring;
    let mut zin: Vec<ZoneIn> = ring
        .zones
        .iter()
        .filter(|z| z.outer_m > z.inner_m && z.inner_m > 0.0)
        .cloned()
        .collect();
    zin.sort_by(|a, b| a.inner_m.total_cmp(&b.inner_m));
    let (inner, outer) = if zin.is_empty() {
        (ring.inner_m, ring.outer_m)
    } else {
        (
            zin.iter().map(|z| z.inner_m).fold(f64::INFINITY, f64::min),
            zin.iter().map(|z| z.outer_m).fold(0.0, f64::max),
        )
    };
    if !(outer > inner && inner > 0.0) {
        return Err("the ring needs an inner and an outer edge".into());
    }
    let j2 = j2_of(h, s.g);
    let gv = Grav {
        gm: s.g * h.mass_kg,
        r: h.radius_m,
        j2,
    };
    let address = Address::universe("ring").map_err(|e| e.to_string())?;
    let stream = Stream::new(STREAM).map_err(|e| e.to_string())?;
    let keyed = Seed::from_text(&format!("{}|{}", hex(seed), ask.key));
    let d = Draws::new(keyed, address, stream);
    let vbase = ask.variant.wrapping_mul(0x9e37_79b9) << 8;

    let ice_w = material_w(sheet, &s.ice, &s.bands_nm, s.contaminant_grain_m)
        .ok_or_else(|| format!("the data sheet holds no optics for {}", s.ice))?;
    let optics = Optics { ice: ice_w };
    let grain = ring.grain_radius_m.unwrap_or(s.grain_radius_m).max(1e-6);
    let ring_dark = ring
        .contaminant
        .clone()
        .unwrap_or_else(|| s.contaminant.clone());
    let default_mk = ring.makeup.unwrap_or(Makeup {
        metal: 0.0,
        rock: 0.0,
        carbon: 0.0,
        ice: 1.0,
    });

    // The base zones: the ask's (catalogued or authored), else seeded ones for a generated ring.
    let mut zones: Vec<Zone> = vec![];
    let make_zone = |name: Option<String>,
                     inner_m: f64,
                     outer_m: f64,
                     tau: f64,
                     tau_range: Option<[f64; 2]>,
                     albedo: Option<f64>,
                     mk: Option<Makeup>,
                     dark_id: &str,
                     dark_grain_m: f64,
                     cause: Cause,
                     assumed: bool,
                     why: &mut Vec<String>|
     -> Zone {
        let dark = contaminant_w(sheet, s, dark_id, dark_grain_m, why);
        let mkz = mk.unwrap_or(default_mk);
        let f = match (albedo, mk) {
            (Some(a), None) => dark_share_for(optics.ice[1], dark[1], a),
            _ => dark_share_of(&mkz, rho),
        };
        let rgb = mix_rgb(&optics.ice, &dark, f);
        let rp = particle_density(&mkz, rho, s.grain_porosity);
        let sigma = 4.0 / 3.0 * rp * grain * tau;
        Zone {
            name,
            inner_m,
            outer_m,
            mass_kg: sigma * area(inner_m, outer_m),
            tau,
            tau_range,
            sigma_kg_m2: sigma,
            albedo: rgb[1],
            rgb,
            dark_share: f,
            contaminant: dark_id.to_string(),
            dust_share: dust_share(tau, &s.phase),
            cause,
            assumed,
        }
    };
    if !zin.is_empty() {
        let shares: f64 = zin.iter().filter_map(|z| z.share).sum();
        for z in &zin {
            let src = z.source.clone().unwrap_or_else(|| "catalogued".into());
            let mid = z.tau_range.map(|r| m::sqrt((r[0] * r[1]).max(0.0)));
            let mut assumed = false;
            let mut tau = z.tau.or(mid).unwrap_or_else(|| {
                assumed = true;
                s.fallback_tau
            });
            if let (Some(sh), Some(mass)) = (z.share, ring.mass_kg) {
                // An authored share of the ring's mass: its depth follows from that mass.
                let mk = z.makeup.unwrap_or(default_mk);
                let rp = particle_density(&mk, rho, s.grain_porosity);
                let sigma = mass * sh / shares.max(1e-12) / area(z.inner_m, z.outer_m);
                tau = 3.0 * sigma / (4.0 * rp * grain);
            }
            let albedo = if z.makeup.is_some() {
                None
            } else {
                Some(z.albedo.unwrap_or_else(|| {
                    assumed = true;
                    s.fallback_albedo
                }))
            };
            let dark_id = z.contaminant.clone().unwrap_or_else(|| ring_dark.clone());
            zones.push(make_zone(
                z.name.clone(),
                z.inner_m,
                z.outer_m,
                tau.max(0.0),
                z.tau_range,
                albedo,
                z.makeup,
                &dark_id,
                z.contaminant_grain_m.unwrap_or(s.contaminant_grain_m),
                Cause {
                    kind: src,
                    sculptor: None,
                    m: None,
                },
                assumed,
                &mut why,
            ));
        }
    } else {
        let g = &s.generated;
        let n = g.zones_min + d.below(vbase ^ 1, (g.zones_max - g.zones_min + 1).max(1));
        let mut cuts: Vec<f64> = (1..n).map(|i| d.unit(vbase ^ (16 + i))).collect();
        cuts.sort_by(f64::total_cmp);
        let mut edges = vec![inner];
        edges.extend(cuts.iter().map(|c| inner + c * (outer - inner)));
        edges.push(outer);
        let weights: Vec<f64> = (0..n)
            .map(|i| m::exp(g.ln_sigma * d.normal(vbase ^ (64 + i))))
            .collect();
        let mass = ring.mass_kg.unwrap_or(0.0);
        let rp = particle_density(&default_mk, rho, s.grain_porosity);
        let wa: f64 = (0..n as usize)
            .map(|i| weights[i] * area(edges[i], edges[i + 1]))
            .sum();
        for i in 0..n as usize {
            let sigma = if wa > 0.0 {
                mass * weights[i] / wa
            } else {
                0.0
            };
            let tau = 3.0 * sigma / (4.0 * rp * grain);
            zones.push(make_zone(
                None,
                edges[i],
                edges[i + 1],
                tau,
                None,
                None,
                ring.makeup,
                &ring_dark,
                s.contaminant_grain_m,
                Cause {
                    kind: "seeded".into(),
                    sculptor: None,
                    m: None,
                },
                false,
                &mut why,
            ));
        }
        why.push(format!(
            "a ring of {:.2e} kg from {} to {} round its world; with no record of its bands, its history is drawn as {n} zones of differing density (variant {})",
            mass,
            km(inner),
            km(outer),
            ask.variant
        ));
    }

    // The marks the sculptors leave.
    let mut marks: Vec<Mark> = vec![];
    for sc in &ask.sculptors {
        if !(sc.a_m > 0.0 && sc.mass_kg > 0.0) {
            continue;
        }
        let mu = sc.mass_kg / h.mass_kg;
        if sc.a_m > inner && sc.a_m < outer {
            let hw = s.gap_per_root_mu * m::sqrt(mu) * sc.a_m;
            let hill = sc.a_m * m::cbrt(mu / 3.0);
            if hw < s.gap_min_hill_radii * hill {
                continue;
            }
            marks.push(Mark {
                kind: "gap".into(),
                sculptor: sc.key(),
                sculptor_name: sc.name.clone(),
                m: 0,
                r_m: sc.a_m,
                half_width_m: hw,
                ratio: None,
                depth: 1.0,
                wave_lambda_m: None,
                damping_m: None,
                edge_wave_m: Some(3.0 * m::PI * hw),
                edge_amplitude_m: Some(
                    s.waves.edge_wave_coefficient * mu * sc.a_m * sc.a_m * sc.a_m / (hw * hw),
                ),
            });
            continue;
        }
        if sc.a_m < inner {
            continue;
        }
        for mm in 2..=s.m_max {
            let r = gv.ilr(sc.a_m, mm);
            if r < inner || r > outer {
                continue;
            }
            let n = gv.omega(r);
            let ratio = s.resonance_f * (mm * mm) as f64 * mu * mu * n * r * r
                / (3.0 * m::PI * s.collisional_nu_m2_s);
            if ratio < s.min_ratio {
                continue;
            }
            let hw = (r * m::sqrt(mm as f64 * mu)).max(1e3);
            if ratio >= 1.0 {
                marks.push(Mark {
                    kind: "edge".into(),
                    sculptor: sc.key(),
                    sculptor_name: sc.name.clone(),
                    m: mm,
                    r_m: r,
                    half_width_m: hw,
                    ratio: Some(ratio),
                    depth: ratio / (1.0 + ratio),
                    wave_lambda_m: None,
                    damping_m: None,
                    edge_wave_m: None,
                    edge_amplitude_m: None,
                });
                continue;
            }
            // A density-wave train launched outward: k = a x with a = D_L / (2 pi G Sigma r_L), D_L = 3 (m-1) Omega^2,
            // damped by viscosity as exp(-(x / x_D)^3), x_D = (9 pi G Sigma / (7 nu a^2 Omega))^(1/3).
            let sigma = zones
                .iter()
                .find(|z| r >= z.inner_m && r < z.outer_m)
                .map(|z| z.sigma_kg_m2)
                .unwrap_or(0.0);
            if sigma <= 0.0 {
                continue;
            }
            let dl = 3.0 * (mm as f64 - 1.0) * n * n;
            let a = dl / (2.0 * m::PI * s.g * sigma * r);
            let xd = m::cbrt(9.0 * m::PI * s.g * sigma / (7.0 * s.collisional_nu_m2_s * a * a * n));
            let x1 = m::sqrt(4.0 * m::PI / a);
            marks.push(Mark {
                kind: "wave".into(),
                sculptor: sc.key(),
                sculptor_name: sc.name.clone(),
                m: mm,
                r_m: r,
                half_width_m: hw,
                ratio: Some(ratio),
                depth: (m::sqrt(ratio)).min(s.waves.max_amplitude),
                wave_lambda_m: Some(2.0 * m::PI / (a * x1)),
                damping_m: Some(xd),
                edge_wave_m: None,
                edge_amplitude_m: None,
            });
        }
    }
    marks.sort_by(|a, b| a.r_m.total_cmp(&b.r_m));

    // Cut the marks into the zones: an edge ends the zone holding it, a gap splits the ring round its moon.
    for k in &marks {
        match k.kind.as_str() {
            "edge" => {
                for z in zones.iter_mut() {
                    // A resonance within the slack of a recorded boundary is that boundary already.
                    if k.r_m > z.inner_m * (1.0 + s.edge_slack)
                        && k.r_m < z.outer_m * (1.0 - s.edge_slack)
                    {
                        let keep = area(z.inner_m, k.r_m) / area(z.inner_m, z.outer_m);
                        z.mass_kg *= keep;
                        z.outer_m = k.r_m;
                        why.push(format!(
                            "{}'s {}:{} resonance at {} pushes {:.1} times harder than the ring's viscosity spreads it, so the ring ends there",
                            k.sculptor_name,
                            k.m,
                            k.m - 1,
                            km(k.r_m),
                            k.ratio.unwrap_or(0.0)
                        ));
                    }
                }
            }
            "gap" => {
                let lo = k.r_m - k.half_width_m;
                let hi = k.r_m + k.half_width_m;
                let mut out = vec![];
                for z in zones.drain(..) {
                    if hi <= z.inner_m || lo >= z.outer_m {
                        out.push(z);
                        continue;
                    }
                    let whole = area(z.inner_m, z.outer_m);
                    if lo > z.inner_m {
                        let mut a = z.clone();
                        a.outer_m = lo;
                        a.mass_kg = z.mass_kg * area(a.inner_m, a.outer_m) / whole;
                        out.push(a);
                    }
                    if hi < z.outer_m {
                        let mut b = z.clone();
                        b.inner_m = hi;
                        b.mass_kg = z.mass_kg * area(b.inner_m, b.outer_m) / whole;
                        out.push(b);
                    }
                }
                zones = out;
                why.push(format!(
                    "{} orbits inside the ring at {} and clears a gap {} wide; its edges ripple every {}",
                    k.sculptor_name,
                    km(k.r_m),
                    km(2.0 * k.half_width_m),
                    km(k.edge_wave_m.unwrap_or(0.0))
                ));
            }
            _ => {}
        }
    }
    zones.retain(|z| z.outer_m > z.inner_m);
    zones.sort_by(|a, b| a.inner_m.total_cmp(&b.inner_m));
    let waves: Vec<&Mark> = marks.iter().filter(|k| k.kind == "wave").collect();
    if let Some(w) = waves.iter().max_by(|a, b| a.depth.total_cmp(&b.depth)) {
        why.push(format!(
            "{} density-wave trains; the strongest, {}'s {}:{} at {}, starts {} long and dies away over about {}",
            waves.len(),
            w.sculptor_name,
            w.m,
            w.m - 1,
            km(w.r_m),
            km(w.wave_lambda_m.unwrap_or(0.0)),
            km(w.damping_m.unwrap_or(0.0))
        ));
    }

    // The profile.
    let n = ask
        .samples
        .unwrap_or(s.samples_default)
        .clamp(16, s.samples_max.max(16));
    let (lo, hi) = match ask.window {
        Some([a, b]) if b > a => (a, b),
        _ => (inner, outer),
    };
    let dx = (hi - lo) / n as f64;
    let mut tau = Vec::with_capacity(n);
    let mut zone_ix = Vec::with_capacity(n);
    let st = &s.structure;
    const SUB: usize = 4;
    for i in 0..n {
        let mut acc = 0.0;
        let mut best: (i32, usize) = (-1, 0);
        let mut counts: Vec<(i32, usize)> = vec![];
        for j in 0..SUB {
            let r = lo + (i as f64 + (j as f64 + 0.5) / SUB as f64) * dx;
            let zi = zones.iter().position(|z| r >= z.inner_m && r < z.outer_m);
            let t = match zi {
                None => 0.0,
                Some(zi) => {
                    let z = &zones[zi];
                    let spread = match z.tau_range {
                        Some([a, b]) if a > 0.0 && b > a => m::ln(b / a) / (2.0 * st.range_sigmas),
                        Some([a, b]) if b > a => {
                            m::ln(b / (0.1 * b).max(1e-9)) / (2.0 * st.range_sigmas)
                        }
                        _ => st.ln_amplitude * z.tau / (z.tau + st.tau_half),
                    };
                    let mut t = z.tau * m::exp(spread * fractal(&d, vbase ^ 3, r, s, dx));
                    if let Some([a, b]) = z.tau_range {
                        t = t.clamp(a, b.max(a));
                    }
                    for w in &waves {
                        let x = r - w.r_m;
                        if x <= 0.0 {
                            continue;
                        }
                        let xd = w.damping_m.unwrap_or(1.0);
                        if x > 3.0 * xd {
                            continue;
                        }
                        // The first wavelength is sqrt(pi / a) (see the mark), so a follows from it.
                        let lam1 = w.wave_lambda_m.unwrap_or(1.0);
                        let a = m::PI / (lam1 * lam1);
                        let k = a * x;
                        // The mean over a sample of cos: sinc(k dx / 2), so waves finer than a sample average out.
                        let q = 0.5 * k * dx / SUB as f64;
                        let filt = if q > 1e-6 { m::sin(q) / q } else { 1.0 };
                        let env = m::exp(-(x / xd) * (x / xd) * (x / xd));
                        t *= 1.0 + w.depth * env * filt * m::cos(0.5 * a * x * x);
                    }
                    t.max(0.0)
                }
            };
            acc += t;
            let id = zi.map(|x| x as i32).unwrap_or(-1);
            match counts.iter_mut().find(|c| c.0 == id) {
                Some(c) => c.1 += 1,
                None => counts.push((id, 1)),
            }
        }
        for c in &counts {
            if c.1 > best.1 || (c.1 == best.1 && c.0 >= 0 && best.0 < 0) {
                best = *c;
            }
        }
        tau.push(f64::from((acc / SUB as f64) as f32));
        zone_ix.push(best.0);
    }
    let mass_kg = zones.iter().map(|z| z.mass_kg).sum();
    if j2 > 0.0 {
        why.push(format!(
            "resonances are placed with the world's J2 of {j2:.4}, which moves them outward by a few hundred km"
        ));
    }
    Ok(RingSystem {
        inner_m: zones.first().map(|z| z.inner_m).unwrap_or(inner),
        outer_m: zones.last().map(|z| z.outer_m).unwrap_or(outer),
        mass_kg,
        j2,
        variant: ask.variant,
        zones,
        marks,
        phase: Phase {
            back_g: s.phase.back_g,
            forward_g: s.phase.forward_g,
            surge_b0: s.phase.surge_b0,
            surge_width_deg: s.phase.surge_width_deg,
        },
        profile: Profile {
            lo_m: lo,
            hi_m: hi,
            tau,
            zone: zone_ix,
        },
        why,
    })
}

fn hex(s: &Seed) -> String {
    s.0 .0.iter().map(|b| format!("{b:02x}")).collect()
}
