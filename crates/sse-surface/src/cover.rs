//! S7: cover, deposits, the built layer, aerostats and the GM's reshape events
//! (surface design section 4: L3 cover and L5 built layer; the GM editing
//! paragraph). Named features (L4) are in [`crate::named`].
//!
//! Everything here is a rule over facts the earlier stages already hold, so a
//! tile stays a pure function of the world and its key:
//!
//! - **Climate**: the one climate (W7a, [`crate::climate`]): mean temperature
//!   by latitude from the energy balance in `sse-climate` (North 1975),
//!   falling with height at the adiabatic lapse rate g / c_p (moist worlds
//!   about two thirds of it).
//! - **Wind and wetness**: each band's wetness is the climate's rain against
//!   its wettest band, and its prevailing wind is the shared circulation's
//!   (reversed on a backward-spinning world); the wind carries moisture
//!   inland, losing it with distance from the sea and over high ground (rain
//!   shadow).
//! - **Ice**: sea ice and land ice below thresholds of mean temperature (land
//!   ice needs it colder where it is dry); on dry worlds ice survives only
//!   where colder than a cold trap.
//! - **Deposits**: ores where the plate history put arcs, collisions and flood
//!   basalts; nickel under big impact melt sheets; buried carbon where life
//!   and thick sediment meet; ice in permanently shadowed polar craters.
//! - **Colour** from material and cover, so the globe and the ground agree.
//! - **Built layer**: settlements, ports, mines and bases placed by pack rules
//!   (scored on water, flat ground, temperature, resources, coast) or by hand.
//! - **Aerostats**: the altitude band where an air profile (adiabat to the
//!   0.1 bar tropopause, isothermal above) gives tolerable pressure and
//!   temperature, drifting with that band's wind.
//! - **Reshape**: the GM raises ranges, drops craters, floods basins, pins
//!   heights and places bases. Each is an authored, dated event, stamped
//!   impossible when it contradicts the physics but kept.

use crate::atlas::{Atlas, COVERS, MATERIALS};
use crate::climate::WorldClimate;
use crate::impacts::Crater;
use crate::regimes::Landform;
use crate::{Range, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use std::collections::BTreeMap;

pub const COVER_VERSION: &str = "surface-cover-v1";

/// Deposit kinds, in the order of a tile's `deposit` codes (0 is none).
pub const DEPOSITS: [&str; 7] = [
    "none",
    "copper-arc",
    "gold-orogenic",
    "nickel-flood",
    "nickel-impact",
    "carbon-buried",
    "ice-shadowed",
];

const R_GAS: f64 = sse_core::sheet::R_GAS;

// ---------------------------------------------------------------- settings

/// A gas guess for a world's air until an atmosphere stage exists: the first
/// entry whose conditions hold is used.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Gas {
    pub gas: String,
    pub molar_kg: f64,
    pub cp_j_kg_k: f64,
    #[serde(default)]
    pub giant: bool,
    #[serde(default)]
    pub min_bar: f64,
    #[serde(default)]
    pub min_t_k: f64,
    #[serde(default)]
    pub max_t_k: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AerostatSettings {
    /// Tolerable pressure, bar (buoyancy and an unpressurised gondola).
    pub p_bar: [f64; 2],
    /// Tolerable temperature, K.
    pub t_k: [f64; 2],
    /// Drift wind at the band when nothing better is known, m/s.
    pub wind_m_s: f64,
    pub step_m: f64,
    pub source: String,
}

/// One kind of built feature placed by rule.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BuiltKind {
    pub kind: String,
    pub radius_km: f64,
    pub min_spacing_km: f64,
    /// Count per million km² of land (before `max`).
    pub per_mkm2: f64,
    pub max: u32,
    /// Score weights by factor: water, coast, flat, temperate, low, resource, equator.
    pub weights: BTreeMap<String, f64>,
    /// Conditions on t_k, height_m, lat_abs_deg, land, coast, deposit.
    #[serde(default)]
    pub needs: BTreeMap<String, Range>,
    #[serde(default)]
    pub source: String,
}

pub const BUILT_FACTORS: [&str; 7] = [
    "water",
    "coast",
    "flat",
    "temperate",
    "low",
    "resource",
    "equator",
];
pub const BUILT_NEEDS: [&str; 6] = ["t_k", "height_m", "lat_abs_deg", "land", "coast", "deposit"];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoverSettings {
    /// Below this surface pressure there is no lapse with height, bar.
    pub lapse_min_bar: f64,
    pub moist_lapse_fraction: f64,
    pub tropopause_bar: f64,
    pub land_ice_k: f64,
    pub sea_ice_k: f64,
    /// Land ice needs this much colder where it is fully dry, K.
    pub ice_dry_k: f64,
    /// Ice survives on a dry world only below this mean temperature, K.
    pub cold_trap_k: f64,
    pub ice_albedo: f64,
    pub grid_deg: f64,
    pub reach_km: f64,
    /// e-folding distance of moisture inland from the sea, km.
    pub inland_km: f64,
    /// e-folding height of a barrier upwind (rain shadow), m.
    pub barrier_m: f64,
    /// Wet ground darkens by up to this fraction.
    pub wet_darkening: f64,
    /// Polar craters can hold ice below this surface pressure, bar.
    pub shadow_ice_max_bar: f64,
    pub impact_ore_km: f64,
    pub arc_ore_myr: [f64; 2],
    pub orogenic_gold_min_myr: f64,
    pub colours: BTreeMap<String, [u8; 3]>,
    pub deposits: BTreeMap<String, String>,
    /// MP8: what each deposit kind holds (an ore process's enrichment over
    /// the crust); absent kinds hold nothing counted yet.
    #[serde(default)]
    pub ores: Ores,
    pub gases: Vec<Gas>,
    pub aerostat: AerostatSettings,
    pub built: Vec<BuiltKind>,
}

impl CoverSettings {
    /// Refuse settings that name unknown deposit kinds, factors or conditions.
    pub fn check(&self) -> Result<(), String> {
        for k in self.deposits.keys() {
            if !DEPOSITS.contains(&k.as_str()) {
                return Err(format!("cover: unknown deposit kind {k}"));
            }
        }
        for k in self.ores.kinds.keys() {
            if !DEPOSITS.contains(&k.as_str()) {
                return Err(format!("cover: ores name unknown deposit kind {k}"));
            }
        }
        for b in &self.built {
            for w in b.weights.keys() {
                if !BUILT_FACTORS.contains(&w.as_str()) {
                    return Err(format!("built kind {} weighs unknown factor {w}", b.kind));
                }
            }
            for q in b.needs.keys() {
                if !BUILT_NEEDS.contains(&q.as_str()) {
                    return Err(format!("built kind {} needs unknown quantity {q}", b.kind));
                }
            }
        }
        if self.gases.is_empty() {
            return Err("cover: no gas guesses".into());
        }
        Ok(())
    }
}

fn holds(r: &Range, v: f64) -> bool {
    r.min.is_none_or(|lo| v >= lo) && r.max.is_none_or(|hi| v <= hi)
}

// ---------------------------------------------------------------- air

/// A world's air as S7 reads it: surface pressure and temperature, and the
/// gas's molar mass and heat capacity (guessed from the pack until an
/// atmosphere stage exists).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Air {
    pub pressure_bar: f64,
    /// At the datum; for a giant, at the 1 bar level.
    pub t_surface_k: f64,
    pub gas: String,
    pub molar_kg: f64,
    pub cp_j_kg_k: f64,
    /// Water vapour condenses in it (moist lapse).
    pub moist: bool,
    pub giant: bool,
}

impl Air {
    pub fn guess(s: &CoverSettings, env: &SurfaceEnvelope, pressure_bar: f64, oceans: bool) -> Air {
        let giant = env.kind == "cloud-deck";
        let p = if giant { 1.0 } else { pressure_bar };
        let t = env.t_surface_now_k;
        let g = s
            .gases
            .iter()
            .find(|g| {
                g.giant == giant
                    && p >= g.min_bar
                    && t >= g.min_t_k
                    && g.max_t_k.is_none_or(|x| t <= x)
            })
            .unwrap_or(&s.gases[s.gases.len() - 1]);
        Air {
            pressure_bar: p,
            t_surface_k: t,
            gas: g.gas.clone(),
            molar_kg: g.molar_kg,
            cp_j_kg_k: g.cp_j_kg_k,
            moist: oceans,
            giant,
        }
    }

    /// Lapse rate, K per km: g / c_p, times the moist fraction on wet worlds.
    pub fn lapse_k_km(&self, s: &CoverSettings, g: f64) -> f64 {
        if self.pressure_bar < s.lapse_min_bar {
            return 0.0;
        }
        let f = if self.moist {
            s.moist_lapse_fraction
        } else {
            1.0
        };
        f * g / self.cp_j_kg_k * 1000.0
    }

    /// Temperature (K) and pressure (bar) at height z above the datum, m:
    /// the adiabat up to the tropopause pressure, isothermal above (Robinson
    /// and Catling 2014). Below the datum (giants) the adiabat continues.
    pub fn profile(&self, s: &CoverSettings, g: f64, z_m: f64) -> (f64, f64) {
        let gam = self.lapse_k_km(s, g) / 1000.0;
        let r = R_GAS / self.molar_kg;
        let (t0, p0) = (self.t_surface_k, self.pressure_bar);
        if gam <= 0.0 {
            let h = r * t0 / g;
            return (t0, p0 * m::exp(-z_m / h));
        }
        let ex = g / (r * gam);
        let at = |z: f64| {
            let t = t0 - gam * z;
            (t, p0 * m::pow(t / t0, ex))
        };
        let pt = s.tropopause_bar.min(p0);
        // Height of the tropopause: T where (T/T0)^ex = pt/p0.
        let t_tr = t0 * m::pow(pt / p0, 1.0 / ex);
        let z_tr = (t0 - t_tr) / gam;
        if z_m <= z_tr {
            let (t, p) = at(z_m);
            (t, p)
        } else {
            let h = r * t_tr / g;
            (t_tr, pt * m::exp(-(z_m - z_tr) / h))
        }
    }
}

// ---------------------------------------------------------------- climate

/// The mean climate S7 reads cover from: the one climate (W7a) and the
/// facts about the world's liquid cover needs.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClimateModel {
    pub world: WorldClimate,
    /// Liquid at the surface by the data sheet's plain name ("water",
    /// "methane", "helium"...) or none: the one climate's liquid when it
    /// names one, else water above 200 K and methane below.
    pub liquid: Option<String>,
    /// The world holds water at all (oceans or ice).
    pub water: bool,
    pub pressure_bar: f64,
    pub why: Vec<String>,
}

impl ClimateModel {
    /// Mean temperature at latitude (radians) and height above the datum (m).
    pub fn t_at(&self, lat: f64, h_m: f64) -> f64 {
        self.world.t_at(lat, h_m)
    }

    /// Wetness from the climate's rain alone at a latitude: 1 in the wettest
    /// band, 0 where nothing falls.
    pub fn band_wetness(&self, lat: f64) -> f64 {
        self.world.wetness(lat)
    }

    /// Whether the prevailing surface wind at a latitude blows from the east.
    pub fn easterly(&self, lat: f64) -> bool {
        self.world.easterly(lat)
    }

    pub fn lapse_k_km(&self) -> f64 {
        self.world.lapse_k_km
    }
}

fn climate(w: &WorldClimate, inp: &CoverInputs, any_liquid: Option<String>) -> ClimateModel {
    let k = &w.climate;
    let n = k.lat_deg.len();
    let c = &k.circulation;
    let mut why = vec![
        format!(
            "mean {:.0} K; equator {:.0} K, poles {:.0} K and {:.0} K (the one climate's energy balance, North 1975)",
            k.conditions.t_surface_k,
            k.t_k[n / 2],
            k.t_k[0],
            k.t_k[n - 1]
        ),
        format!(
            "{} circulation cell(s) per hemisphere from a spin of {:.1} h{}",
            c.cells,
            k.conditions.rotation_hours.abs(),
            if k.conditions.retrograde() {
                " (backwards)"
            } else {
                ""
            }
        ),
        format!(
            "air cools {:.2} K per km with height ({})",
            w.lapse_k_km, inp.air.gas
        ),
    ];
    for a in &w.assumed {
        why.push(format!("assumed: {a}"));
    }
    ClimateModel {
        world: w.clone(),
        liquid: any_liquid,
        water: inp.water,
        pressure_bar: inp.air.pressure_bar,
        why,
    }
}

// ---------------------------------------------------------------- inputs and layers

/// What S7 reads beyond the atlas and envelope.
#[derive(Clone, Debug)]
pub struct CoverInputs<'a> {
    pub air: Air,
    /// The world holds water (oceans now, or a water fraction).
    pub water: bool,
    /// The one climate (W7a): the same one wear read.
    pub climate: &'a WorldClimate,
    /// Biosphere productivity relative to Earth's (0: lifeless).
    pub life: f64,
    pub plates: Option<&'a crate::plates::PlateHistory>,
    pub reshape: Vec<Reshape>,
    /// Built kinds to place by rule (the pack's names); empty for an unsettled world.
    pub settle: Vec<String>,
}

/// MP8: how deposits hold elements (`sse.composition.deposit_amount`): the
/// ore in a footprint is `ore_share` of the crust to `depth_m`, at the crust's
/// concentration of each element times the kind's enrichment.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Ores {
    /// Depth the ore is counted to, m.
    #[serde(default)]
    pub depth_m: f64,
    /// Share of a footprint's crust to that depth that is ore.
    #[serde(default)]
    pub ore_share: f64,
    /// Per deposit kind, each element's grade over the crust's (enrichment).
    #[serde(default)]
    pub kinds: BTreeMap<String, BTreeMap<String, f64>>,
    /// Where the grades and the share come from.
    #[serde(default)]
    pub references: Vec<String>,
}

/// A deposit at a place (cell deposits live on the plate grid).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Deposit {
    pub id: String,
    pub kind: String,
    pub dir: [f64; 3],
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub radius_km: f64,
    pub cause: String,
}

/// A settlement, port, mine, base or anything else built.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Built {
    pub id: String,
    pub kind: String,
    pub dir: [f64; 3],
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub radius_km: f64,
    #[serde(default)]
    pub name: Option<String>,
    /// "rule" or "gm".
    pub placed_by: String,
    pub at_myr: f64,
    pub why: Vec<String>,
    #[serde(default)]
    pub impossible: Option<String>,
}

/// A flooded basin: below `level_m` within the radius is under liquid.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Flood {
    pub id: String,
    pub dir: [f64; 3],
    pub radius_km: f64,
    pub level_m: f64,
    /// Index into COVERS.
    pub cover: u8,
}

/// A GM edit. Every one is an authored event with a date (Myr since the
/// world formed); it shows only at epochs at or after its date.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum Reshape {
    RaiseRange {
        id: String,
        from: [f64; 2],
        to: [f64; 2],
        height_m: f64,
        width_km: f64,
        at_myr: f64,
    },
    DropCrater {
        id: String,
        lat_deg: f64,
        lon_deg: f64,
        d_km: f64,
        at_myr: f64,
    },
    FloodBasin {
        id: String,
        lat_deg: f64,
        lon_deg: f64,
        radius_km: f64,
        level_m: f64,
        at_myr: f64,
    },
    /// Fix the height at a place; the terrain flows around it.
    Pin {
        id: String,
        lat_deg: f64,
        lon_deg: f64,
        height_m: f64,
        radius_km: f64,
        at_myr: f64,
    },
    PlaceBase {
        id: String,
        kind: String,
        lat_deg: f64,
        lon_deg: f64,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        radius_km: Option<f64>,
        at_myr: f64,
    },
}

impl Reshape {
    pub fn id(&self) -> &str {
        match self {
            Reshape::RaiseRange { id, .. }
            | Reshape::DropCrater { id, .. }
            | Reshape::FloodBasin { id, .. }
            | Reshape::Pin { id, .. }
            | Reshape::PlaceBase { id, .. } => id,
        }
    }
    pub fn at_myr(&self) -> f64 {
        match self {
            Reshape::RaiseRange { at_myr, .. }
            | Reshape::DropCrater { at_myr, .. }
            | Reshape::FloodBasin { at_myr, .. }
            | Reshape::Pin { at_myr, .. }
            | Reshape::PlaceBase { at_myr, .. } => *at_myr,
        }
    }
}

/// What became of a GM edit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Applied {
    pub id: String,
    pub op: String,
    pub at_myr: f64,
    /// Shown at this epoch (false when dated after it).
    pub shown: bool,
    pub note: String,
    /// Why the physics would not allow it (the edit stays, tagged).
    #[serde(default)]
    pub impossible: Option<String>,
}

/// A pin waiting for the atlas to measure the height it must correct.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PendingPin {
    pub id: String,
    pub dir: [f64; 3],
    pub height_m: f64,
    pub radius_km: f64,
    pub at_myr: f64,
}

/// S7's layers on one atlas.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Layers {
    pub address: String,
    pub epoch_myr: f64,
    pub climate: ClimateModel,
    pub air: Air,
    /// Moisture on a latitude-longitude grid, 0 to 255, rows from the south.
    pub grid_deg: f64,
    pub wet: Vec<u8>,
    /// Cells per face of the plate grid the cell deposits sit on (0: none).
    pub cell_n: u32,
    /// Cell deposits on the plate grid (codes into DEPOSITS), when plates moved.
    pub cell_deposit: Vec<u8>,
    pub deposits: Vec<Deposit>,
    pub built: Vec<Built>,
    pub built_kinds: Vec<String>,
    pub landforms: Vec<Landform>,
    pub craters: Vec<Crater>,
    pub floods: Vec<Flood>,
    pub pins: Vec<PendingPin>,
    pub applied: Vec<Applied>,
    /// Colours by material and cover name.
    pub colours: BTreeMap<String, [u8; 3]>,
    pub ice_albedo: f64,
    pub land_ice_k: f64,
    pub sea_ice_k: f64,
    pub ice_dry_k: f64,
    pub cold_trap_k: f64,
    pub wet_darkening: f64,
    /// Tolerance terms for authored craters: (diameter km, fresh relief m).
    pub crater_relief: Vec<(f64, f64)>,
    /// Largest |height| a range or pin can stand at this gravity, m.
    pub max_relief_m: f64,
    /// The world has a solid surface for cover (not a giant's cloud deck).
    pub solid: bool,
    /// For authored craters' tolerances.
    pub impact_settings: crate::impacts::ImpactSettings,
    pub gravity: f64,
}

fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (la, lo) = (lat_deg.to_radians(), lon_deg.to_radians());
    [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)]
}

fn lat_lon(v: [f64; 3]) -> (f64, f64) {
    (
        m::asin(v[2].clamp(-1.0, 1.0)).to_degrees(),
        m::atan2(v[1], v[0]).to_degrees(),
    )
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = m::sqrt(dot(a, a)).max(1e-300);
    [a[0] / l, a[1] / l, a[2] / l]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Great-circle angle between two unit vectors, radians.
fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    m::acos(dot(a, b).clamp(-1.0, 1.0))
}

/// Boiling point of water at a pressure, K (Clausius-Clapeyron from 373.15 K at 1 atm).
pub fn water_boils_k(p_bar: f64) -> f64 {
    let l = 40.65e3;
    1.0 / (1.0 / 373.15 - R_GAS / l * m::ln(p_bar.max(1e-12) * 1e5 / sse_core::sheet::ATM))
}

/// Build S7's layers for an atlas. The atlas supplies heights, liquid and
/// features; nothing here changes the earlier stages' facts.
pub fn layers(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    atlas: &Atlas,
    inp: &CoverInputs,
) -> Layers {
    let cs = &s.cover;
    let rk = env.reference_radius_m / 1000.0;
    let g = env.gravity_m_s2.max(1e-6);
    let epoch = atlas.epoch_myr as f64;
    let solid = env.kind != "cloud-deck";
    let d = Draws::new(
        *seed,
        env.address.clone(),
        Stream::new("surface.cover.v1").expect("valid"),
    );
    // The coarse grid: heights and liquid at each cell centre.
    let nlat = (m::round(180.0 / cs.grid_deg) as usize).max(2);
    let nlon = 2 * nlat;
    let step = 180.0 / nlat as f64;
    let mut hgt = vec![0.0; nlat * nlon];
    let mut liq = vec![false; nlat * nlon];
    for i in 0..nlat {
        for j in 0..nlon {
            let la = -90.0 + (i as f64 + 0.5) * step;
            let lo = -180.0 + (j as f64 + 0.5) * step;
            let (h, p) = atlas.height(from_lat_lon(la, lo), 0);
            hgt[i * nlon + j] = h;
            liq[i * nlon + j] = p.liquid;
        }
    }
    let any_liquid = liq.iter().any(|x| *x);
    // The liquid is the one climate's (it picks it by phase); the old
    // temperature split stands in only when the climate names none.
    let liquid_kind = if !any_liquid || !solid {
        None
    } else if let Some(l) = &inp.climate.climate.conditions.liquid {
        Some(l.clone())
    } else if env.t_surface_now_k < 200.0 {
        Some("methane".to_string())
    } else {
        Some("water".to_string())
    };
    let cl = climate(inp.climate, inp, liquid_kind.clone());
    // Wetness by cell, marching upwind along each band's prevailing wind.
    let mut wet = vec![0u8; nlat * nlon];
    if any_liquid && solid {
        for i in 0..nlat {
            let la = (-90.0 + (i as f64 + 0.5) * step).to_radians();
            let w0 = cl.band_wetness(la);
            let dir: i64 = if cl.easterly(la) { 1 } else { -1 };
            let cell_km = (step.to_radians() * rk * m::cos(la)).max(1e-3);
            let steps = ((cs.reach_km / cell_km) as i64).clamp(1, nlon as i64);
            for j in 0..nlon {
                let c = i * nlon + j;
                let here = hgt[c].max(0.0);
                let w = if liq[c] {
                    w0
                } else {
                    let mut barrier: f64 = 0.0;
                    let mut sea_km = cs.reach_km;
                    for k in 1..=steps {
                        let jj = (j as i64 + dir * k).rem_euclid(nlon as i64) as usize;
                        let cc = i * nlon + jj;
                        if liq[cc] {
                            sea_km = k as f64 * cell_km;
                            break;
                        }
                        barrier = barrier.max(hgt[cc]);
                    }
                    w0 * m::exp(-sea_km / cs.inland_km)
                        * m::exp(-(barrier - here).max(0.0) / cs.barrier_m)
                };
                wet[c] = m::round(w.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
    }
    // Deposits on the plate grid.
    let mut cell_n = 0u32;
    let mut cell_deposit = vec![];
    let sediment = atlas.sediment_cells();
    if let Some(h) = inp.plates.filter(|h| h.steps > 0) {
        cell_deposit = h
            .cells
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                let age = |t: f64| epoch - t;
                if c.continental
                    && c.arc_myr
                        .is_some_and(|t| age(t) >= cs.arc_ore_myr[0] && age(t) <= cs.arc_ore_myr[1])
                {
                    1
                } else if c.continental
                    && c.orogeny
                        .iter()
                        .any(|o| age(o.0) >= cs.orogenic_gold_min_myr)
                {
                    2
                } else if c.flood_myr.is_some() {
                    3
                } else if inp.life > 0.0
                    && sediment
                        .as_ref()
                        .is_some_and(|sd| sd.get(ci).copied().unwrap_or(false))
                {
                    5
                } else {
                    0
                }
            })
            .collect();
        cell_n = h.n;
    }
    // Point deposits from craters.
    let mut deposits = vec![];
    // Tilt from the pole the spin runs about (a 177° tilt is 3° backwards).
    let o = inp.climate.climate.conditions.obliquity_deg.abs();
    let obl = o.min(180.0 - o).max(0.0);
    if let Some(f) = atlas.features() {
        for c in f.craters.iter().filter(|c| !c.ghost) {
            if c.d_km >= cs.impact_ore_km {
                deposits.push(Deposit {
                    id: format!("deposit:nickel-impact-{}", c.id),
                    kind: "nickel-impact".into(),
                    dir: c.dir,
                    lat_deg: c.lat_deg,
                    lon_deg: c.lon_deg,
                    radius_km: 0.5 * c.d_km,
                    cause: format!(
                        "impact melt sheet of {} ({:.0} km): metals settle out of a large melt (Sudbury-type)",
                        c.id, c.d_km
                    ),
                });
            }
            if inp.air.pressure_bar < cs.shadow_ice_max_bar && solid {
                let wall = m::atan((c.depth_m + c.rim_m) / (500.0 * c.d_km));
                let sun = (90.0 - c.lat_deg.abs() + obl).to_radians();
                if sun < wall && sun >= 0.0 {
                    let r = 0.5 * c.d_km * (1.0 - m::tan(sun) / m::tan(wall));
                    if r > 0.0 {
                        deposits.push(Deposit {
                            id: format!("deposit:ice-shadowed-{}", c.id),
                            kind: "ice-shadowed".into(),
                            dir: c.dir,
                            lat_deg: c.lat_deg,
                            lon_deg: c.lon_deg,
                            radius_km: r,
                            cause: format!(
                                "the floor of {} never sees the Sun (highest Sun {:.1}°, wall {:.1}°): a cold trap for delivered water",
                                c.id,
                                sun.to_degrees(),
                                wall.to_degrees()
                            ),
                        });
                    }
                }
            }
        }
    }
    let max_relief_m = s.impacts.volcanism.shield_height_m * (sse_core::sheet::G0 / g);
    let mut ly = Layers {
        address: env.address.to_string(),
        epoch_myr: epoch,
        climate: cl,
        air: inp.air.clone(),
        grid_deg: step,
        wet,
        cell_n,
        cell_deposit,
        deposits,
        built: vec![],
        built_kinds: cs.built.iter().map(|b| b.kind.clone()).collect(),
        landforms: vec![],
        craters: vec![],
        floods: vec![],
        pins: vec![],
        applied: vec![],
        colours: cs.colours.clone(),
        ice_albedo: cs.ice_albedo,
        land_ice_k: cs.land_ice_k,
        sea_ice_k: cs.sea_ice_k,
        ice_dry_k: cs.ice_dry_k,
        cold_trap_k: cs.cold_trap_k,
        wet_darkening: cs.wet_darkening,
        crater_relief: vec![],
        max_relief_m,
        solid,
        impact_settings: s.impacts.clone(),
        gravity: g,
    };
    // Built layer by rule.
    if solid {
        place_by_rule(cs, &d, &mut ly, &hgt, &liq, nlat, nlon, rk, inp);
    }
    // GM edits.
    for r in &inp.reshape {
        reshape(s, env, &mut ly, r, epoch, rk, g);
    }
    ly
}

#[allow(clippy::too_many_arguments)]
fn place_by_rule(
    cs: &CoverSettings,
    d: &Draws,
    ly: &mut Layers,
    hgt: &[f64],
    liq: &[bool],
    nlat: usize,
    nlon: usize,
    rk: f64,
    inp: &CoverInputs,
) {
    let step = ly.grid_deg;
    let cell_area = |i: usize| {
        let la0 = (-90.0 + i as f64 * step).to_radians();
        let la1 = (-90.0 + (i + 1) as f64 * step).to_radians();
        rk * rk * step.to_radians() * (m::sin(la1) - m::sin(la0))
    };
    let land_km2: f64 = (0..nlat * nlon)
        .filter(|c| !liq[*c])
        .map(|c| cell_area(c / nlon))
        .sum();
    for (ki, bk) in cs.built.iter().enumerate() {
        if !inp.settle.contains(&bk.kind) {
            continue;
        }
        let want = (m::round(bk.per_mkm2 * land_km2 / 1e6) as u32).min(bk.max);
        let mut scored: Vec<(f64, usize, Vec<String>)> = vec![];
        for c in 0..nlat * nlon {
            let (i, j) = (c / nlon, c % nlon);
            let la = -90.0 + (i as f64 + 0.5) * step;
            let lo = -180.0 + (j as f64 + 0.5) * step;
            let land = !liq[c];
            let coast = land
                && [
                    (i, (j + 1) % nlon),
                    (i, (j + nlon - 1) % nlon),
                    (i.saturating_sub(1), j),
                    ((i + 1).min(nlat - 1), j),
                ]
                .iter()
                .any(|&(a, b)| liq[a * nlon + b]);
            let h = hgt[c];
            let t = ly.climate.t_at(la.to_radians(), h);
            let dir = from_lat_lon(la, lo);
            let deposit = ly.deposit_at(dir, rk) != 0;
            let nb = |a: usize, b: usize| hgt[a * nlon + b];
            let dx = step.to_radians() * rk * 1000.0 * m::cos(la.to_radians()).max(0.05);
            let slope_deg = m::atan(
                ((nb(i, (j + 1) % nlon) - nb(i, (j + nlon - 1) % nlon)).abs() / (2.0 * dx)).max(
                    (nb((i + 1).min(nlat - 1), j) - nb(i.saturating_sub(1), j)).abs()
                        / (2.0 * step.to_radians() * rk * 1000.0),
                ),
            )
            .to_degrees();
            let q = |name: &str| match name {
                "t_k" => t,
                "height_m" => h,
                "lat_abs_deg" => la.abs(),
                "land" => land as u8 as f64,
                "coast" => coast as u8 as f64,
                "deposit" => deposit as u8 as f64,
                _ => 0.0,
            };
            if !bk.needs.iter().all(|(k, r)| holds(r, q(k))) {
                continue;
            }
            let wetness = ly.wet[c] as f64 / 255.0;
            let f = |name: &str| match name {
                "water" => wetness,
                "coast" => coast as u8 as f64,
                "flat" => (1.0 - slope_deg / 10.0).clamp(0.0, 1.0),
                "temperate" => m::exp(-((t - 288.0) / 15.0) * ((t - 288.0) / 15.0)),
                "low" => (1.0 - h.max(0.0) / 3000.0).clamp(0.0, 1.0),
                "resource" => deposit as u8 as f64,
                "equator" => m::cos(la.to_radians()),
                _ => 0.0,
            };
            let mut score = 0.0;
            let mut why = vec![];
            for (k, w) in &bk.weights {
                let v = f(k);
                score += w * v;
                why.push(format!("{k} {v:.2} × {w}"));
            }
            // A keyed jitter well below any weight breaks ties the same way everywhere.
            score += 1e-6 * d.unit(c as u64 + 1_000_000 * ki as u64);
            scored.push((score, c, why));
        }
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut placed: Vec<[f64; 3]> = vec![];
        for (score, c, why) in scored {
            if placed.len() as u32 >= want {
                break;
            }
            let (i, j) = (c / nlon, c % nlon);
            let jit = |k: u64| {
                (d.unit(2 * (c as u64) + k + 7_000_000 * (ki as u64 + 1)) - 0.5) * 0.8 * step
            };
            let la = -90.0 + (i as f64 + 0.5) * step + jit(0);
            let lo = -180.0 + (j as f64 + 0.5) * step + jit(1);
            let dir = from_lat_lon(la, lo);
            if placed
                .iter()
                .any(|p| angle(*p, dir) * rk < bk.min_spacing_km)
            {
                continue;
            }
            placed.push(dir);
            let n = placed.len();
            let mut w = vec![format!(
                "placed by the {} rule, score {:.2}",
                bk.kind, score
            )];
            w.extend(why);
            if !bk.source.is_empty() {
                w.push(bk.source.clone());
            }
            ly.built.push(Built {
                id: format!("built:{}-{n}", bk.kind),
                kind: bk.kind.clone(),
                dir,
                lat_deg: la,
                lon_deg: lo,
                radius_km: bk.radius_km,
                name: None,
                placed_by: "rule".into(),
                at_myr: ly.epoch_myr,
                why: w,
                impossible: None,
            });
        }
    }
}

fn reshape(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    ly: &mut Layers,
    r: &Reshape,
    epoch: f64,
    rk: f64,
    g: f64,
) {
    let shown = r.at_myr() <= epoch;
    let cause = |op: &str| format!("authored by the GM ({op}) at {:.0} Myr", r.at_myr());
    let mut note = String::new();
    let mut impossible = None;
    let op = match r {
        Reshape::RaiseRange {
            id,
            from,
            to,
            height_m,
            width_km,
            at_myr,
        } => {
            let a = from_lat_lon(from[0], from[1]);
            let b = from_lat_lon(to[0], to[1]);
            let mid = norm([a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
            let len = angle(a, b) * rk;
            // Tangent along the great circle at the midpoint, toward `to`.
            let n = norm(cross(a, b));
            let axis = norm(cross(n, mid));
            let (la, lo) = lat_lon(mid);
            if *height_m > ly.max_relief_m {
                impossible = Some(format!(
                    "a range {:.0} m high cannot stand at {:.2} m/s² (rock supports about {:.0} m)",
                    height_m, g, ly.max_relief_m
                ));
            }
            note = format!("range {len:.0} km long, {width_km:.0} km wide, {height_m:.0} m high");
            if shown {
                ly.landforms.push(Landform {
                    id: id.clone(),
                    kind: "range".into(),
                    dir: mid,
                    axis: Some(axis),
                    lat_deg: la,
                    lon_deg: lo,
                    size_km: len,
                    width_km: *width_km,
                    height_m: *height_m,
                    formed_myr: *at_myr,
                    cover: None,
                    material: None,
                    cause: cause("raise range"),
                });
            }
            "raise-range"
        }
        Reshape::DropCrater {
            id,
            lat_deg,
            lon_deg,
            d_km,
            at_myr,
        } => {
            let (shape, depth, rim, peak) = crate::impacts::crater_shape(&s.impacts, g, *d_km);
            if *d_km > 2.0 * rk {
                impossible = Some(format!(
                    "a crater {d_km:.0} km across is larger than the world ({:.0} km across) and would have shattered it",
                    2.0 * rk
                ));
            }
            note = format!("{shape} crater {d_km:.0} km across, {depth:.0} m deep");
            if shown {
                ly.craters.push(Crater {
                    id: id.clone(),
                    dir: from_lat_lon(*lat_deg, *lon_deg),
                    lat_deg: *lat_deg,
                    lon_deg: *lon_deg,
                    d_km: *d_km,
                    shape: shape.into(),
                    depth_m: depth,
                    rim_m: rim,
                    peak_m: peak,
                    formed_myr: *at_myr,
                    fresh: 1.0,
                    ghost: false,
                    flooded: false,
                    cause: cause("drop crater"),
                });
                ly.crater_relief.push((*d_km, depth + rim + peak));
            }
            "drop-crater"
        }
        Reshape::FloodBasin {
            id,
            lat_deg,
            lon_deg,
            radius_km,
            level_m,
            ..
        } => {
            let t = env.t_surface_now_k;
            let p = ly.air.pressure_bar;
            // Any liquid but water floods as the non-water liquid cover.
            let other_liquid = ly.climate.liquid.as_deref().is_some_and(|l| l != "water");
            let cover = if other_liquid {
                3
            } else if p * 1e5 < sse_core::sheet::triple_point_pa::H2O {
                impossible = Some(format!(
                    "liquid water boils away below 611 Pa; the air here is {:.0} Pa",
                    p * 1e5
                ));
                1
            } else if t > water_boils_k(p) {
                impossible = Some(format!(
                    "water boils at {:.0} K at {:.3} bar and the surface is {:.0} K",
                    water_boils_k(p),
                    p,
                    t
                ));
                1
            } else if t < ly.sea_ice_k {
                note = "frozen over: the mean surface temperature is below freezing".into();
                4
            } else {
                1
            };
            if note.is_empty() {
                note = format!("flooded to {level_m:.0} m within {radius_km:.0} km");
            }
            if shown {
                ly.floods.push(Flood {
                    id: id.clone(),
                    dir: from_lat_lon(*lat_deg, *lon_deg),
                    radius_km: *radius_km,
                    level_m: *level_m,
                    cover,
                });
            }
            "flood-basin"
        }
        Reshape::Pin {
            id,
            lat_deg,
            lon_deg,
            height_m,
            radius_km,
            at_myr,
        } => {
            note = format!("height pinned to {height_m:.0} m within {radius_km:.1} km");
            if shown {
                ly.pins.push(PendingPin {
                    id: id.clone(),
                    dir: from_lat_lon(*lat_deg, *lon_deg),
                    height_m: *height_m,
                    radius_km: *radius_km,
                    at_myr: *at_myr,
                });
            }
            "pin"
        }
        Reshape::PlaceBase {
            id,
            kind,
            lat_deg,
            lon_deg,
            name,
            radius_km,
            at_myr,
        } => {
            let rad = radius_km.unwrap_or(1.0);
            note = format!("{kind} placed by hand");
            if shown {
                if !ly.built_kinds.contains(kind) {
                    ly.built_kinds.push(kind.clone());
                }
                ly.built.push(Built {
                    id: id.clone(),
                    kind: kind.clone(),
                    dir: from_lat_lon(*lat_deg, *lon_deg),
                    lat_deg: *lat_deg,
                    lon_deg: *lon_deg,
                    radius_km: rad,
                    name: name.clone(),
                    placed_by: "gm".into(),
                    at_myr: *at_myr,
                    why: vec![cause("place base")],
                    impossible: None,
                });
            }
            "place-base"
        }
    };
    ly.applied.push(Applied {
        id: r.id().to_string(),
        op: op.into(),
        at_myr: r.at_myr(),
        shown,
        note,
        impossible,
    });
}

/// What a tile needs from the layers (gathered once per tile).
#[derive(Default)]
pub(crate) struct TileLayer {
    pub floods: Vec<Flood>,
    pub deposits: Vec<Deposit>,
    pub built: Vec<Built>,
}

/// One sample's S7 values.
pub(crate) struct SampleLayer {
    pub cover: u8,
    pub wet: u8,
    pub deposit: u8,
    pub structure: u8,
    pub albedo: f64,
    pub colour: u32,
}

impl Layers {
    /// The plate-cell deposit code at a direction (the plate grid's `cell_at`).
    fn cell_code(&self, v: [f64; 3]) -> u8 {
        if self.cell_n == 0 || self.cell_deposit.is_empty() {
            return 0;
        }
        let (f, u, w) = crate::atlas::face_of(v);
        let n = self.cell_n as f64;
        let i = (((u + 1.0) * 0.5 * n) as i64).clamp(0, self.cell_n as i64 - 1) as usize;
        let j = (((w + 1.0) * 0.5 * n) as i64).clamp(0, self.cell_n as i64 - 1) as usize;
        self.cell_deposit
            [(f as usize) * (self.cell_n * self.cell_n) as usize + j * self.cell_n as usize + i]
    }

    /// Moisture at a direction, 0 to 1 (bilinear on the grid, wrapping in longitude).
    pub fn wetness(&self, v: [f64; 3]) -> f64 {
        if self.wet.is_empty() {
            return 0.0;
        }
        let nlat = m::round(180.0 / self.grid_deg) as usize;
        let nlon = 2 * nlat;
        let (la, lo) = lat_lon(v);
        let fi = ((la + 90.0) / self.grid_deg - 0.5).clamp(0.0, (nlat - 1) as f64);
        let fj = (lo + 180.0) / self.grid_deg - 0.5;
        let i0 = m::floor(fi) as usize;
        let i1 = (i0 + 1).min(nlat - 1);
        let tj = m::floor(fj);
        let (j0, j1) = (
            (tj as i64).rem_euclid(nlon as i64) as usize,
            (tj as i64 + 1).rem_euclid(nlon as i64) as usize,
        );
        let (a, b) = (fi - i0 as f64, fj - tj);
        let w = |i: usize, j: usize| self.wet[i * nlon + j] as f64 / 255.0;
        let r0 = w(i0, j0) + (w(i0, j1) - w(i0, j0)) * b;
        let r1 = w(i1, j0) + (w(i1, j1) - w(i1, j0)) * b;
        r0 + (r1 - r0) * a
    }

    /// Deposit code at a direction from the plate cells and point deposits.
    pub fn deposit_at(&self, v: [f64; 3], radius_km: f64) -> u8 {
        for dp in &self.deposits {
            if angle(dp.dir, v) * radius_km <= dp.radius_km {
                return DEPOSITS.iter().position(|k| *k == dp.kind).unwrap_or(0) as u8;
            }
        }
        self.cell_code(v)
    }

    pub(crate) fn near_tile(
        &self,
        centre: [f64; 3],
        half: f64,
        rk: f64,
        tags: &mut Vec<String>,
    ) -> TileLayer {
        let near = |d: [f64; 3], r_km: f64| angle(d, centre) <= half + r_km / rk;
        let mut t = TileLayer::default();
        for f in &self.floods {
            if near(f.dir, f.radius_km) {
                tags.push(f.id.clone());
                t.floods.push(f.clone());
            }
        }
        for dp in &self.deposits {
            if near(dp.dir, dp.radius_km) {
                tags.push(dp.id.clone());
                t.deposits.push(dp.clone());
            }
        }
        for b in &self.built {
            if near(b.dir, b.radius_km) {
                tags.push(b.id.clone());
                t.built.push(b.clone());
            }
        }
        t
    }

    /// Cover, wetness, deposit, structure, albedo and colour of one sample.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sample(
        &self,
        t: &TileLayer,
        v: [f64; 3],
        h: f64,
        material: u8,
        cover: u8,
        albedo: f64,
        rk: f64,
    ) -> SampleLayer {
        let mut cov = cover;
        let wet = self.wetness(v);
        if self.solid && MATERIALS[material as usize] != "ice" && self.climate.water {
            let lat = m::asin(v[2].clamp(-1.0, 1.0));
            let temp = self.climate.t_at(lat, h);
            match COVERS[cov as usize] {
                "water" if temp < self.sea_ice_k => cov = 4,
                "none" => {
                    let limit = if self.climate.liquid.as_deref() == Some("water") {
                        self.land_ice_k - self.ice_dry_k * (1.0 - wet)
                    } else {
                        self.cold_trap_k
                    };
                    if temp < limit {
                        cov = 4;
                    }
                }
                _ => {}
            }
        }
        for f in &t.floods {
            if h < f.level_m && angle(f.dir, v) * rk <= f.radius_km {
                cov = f.cover;
            }
        }
        let mut deposit = 0u8;
        for dp in &t.deposits {
            if angle(dp.dir, v) * rk <= dp.radius_km {
                deposit = DEPOSITS.iter().position(|k| *k == dp.kind).unwrap_or(0) as u8;
                break;
            }
        }
        if deposit == 0 {
            deposit = self.cell_code(v);
        }
        let mut structure = 0u8;
        for b in &t.built {
            if angle(b.dir, v) * rk <= b.radius_km {
                structure = 1 + self
                    .built_kinds
                    .iter()
                    .position(|k| *k == b.kind)
                    .unwrap_or(0) as u8;
                break;
            }
        }
        let al = if COVERS[cov as usize] == "ice" {
            albedo.max(self.ice_albedo)
        } else {
            albedo
        };
        let name = if COVERS[cov as usize] == "none" {
            MATERIALS[material as usize]
        } else {
            COVERS[cov as usize]
        };
        let base = self.colours.get(name).copied().unwrap_or([128, 128, 128]);
        let k = if COVERS[cov as usize] == "none" {
            1.0 - self.wet_darkening * wet
        } else {
            1.0
        };
        let ch = |x: u8| (m::round(x as f64 * k).clamp(0.0, 255.0)) as u32;
        let colour = (ch(base[0]) << 16) | (ch(base[1]) << 8) | ch(base[2]);
        SampleLayer {
            cover: cov,
            wet: m::round(wet.clamp(0.0, 1.0) * 255.0) as u8,
            deposit,
            structure,
            albedo: al,
            colour,
        }
    }

    /// Why the ground at a place is covered and built as it is.
    pub fn explain(&self, lat_deg: f64, lon_deg: f64, h_m: f64, radius_km: f64) -> Vec<String> {
        let v = from_lat_lon(lat_deg, lon_deg);
        let mut out = vec![];
        if self.solid {
            let t = self.climate.t_at(lat_deg.to_radians(), h_m);
            out.push(format!(
                "mean temperature here {:.0} K at {:.0} m ({:.0} K at the datum and this latitude, {:.2} K per km lower with height)",
                t,
                h_m,
                self.climate.t_at(lat_deg.to_radians(), 0.0),
                self.climate.lapse_k_km()
            ));
            if !self.wet.is_empty() {
                out.push(format!(
                    "wetness {:.2}: {} wind band, {}",
                    self.wetness(v),
                    if self.climate.easterly(lat_deg.to_radians()) {
                        "easterly"
                    } else {
                        "westerly"
                    },
                    if self.climate.band_wetness(lat_deg.to_radians()) > 0.5 {
                        "under rising air"
                    } else {
                        "under sinking air"
                    }
                ));
            }
        }
        let mut any = false;
        for d in self
            .deposits
            .iter()
            .filter(|d| angle(d.dir, v) * radius_km <= d.radius_km)
        {
            out.push(format!("{}: {}", d.kind, d.cause));
            any = true;
        }
        let cell = self.cell_code(v);
        if !any && cell != 0 {
            out.push(format!(
                "{}: from this ground's plate history",
                DEPOSITS[cell as usize]
            ));
        }
        for b in &self.built {
            if angle(b.dir, v) * radius_km <= b.radius_km {
                out.push(format!("{} ({}): {}", b.id, b.placed_by, b.why.join("; ")));
            }
        }
        for a in &self.applied {
            if let Some(x) = &a.impossible {
                out.push(format!("{} is tagged impossible: {x}", a.id));
            }
        }
        out
    }
}

// ---------------------------------------------------------------- aerostats

/// The band an aerostat can hold: tolerable pressure and temperature.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AerostatBand {
    pub low_km: f64,
    pub high_km: f64,
    pub p_bar: [f64; 2],
    pub t_k: [f64; 2],
    pub wind_m_s: f64,
    /// One circuit of the world at the equator, days.
    pub circuit_days: f64,
    pub below: Vec<String>,
    pub why: Vec<String>,
}

/// Where an aerostat can float on this world, or why it cannot.
pub fn aerostat_band(
    s: &CoverSettings,
    env: &SurfaceEnvelope,
    air: &Air,
    wind_m_s: Option<f64>,
) -> Result<AerostatBand, String> {
    let a = &s.aerostat;
    let g = env.gravity_m_s2.max(1e-6);
    if air.pressure_bar < a.p_bar[0] && !air.giant {
        return Err(format!(
            "the air is too thin to float in: {:.3} bar at the surface, the band needs {} to {} bar",
            air.pressure_bar, a.p_bar[0], a.p_bar[1]
        ));
    }
    let (z0, z1) = if air.giant {
        (-400_000.0, 200_000.0)
    } else {
        (0.0, 200_000.0)
    };
    let ok = |z: f64| {
        let (t, p) = air.profile(s, g, z);
        p >= a.p_bar[0] && p <= a.p_bar[1] && t >= a.t_k[0] && t <= a.t_k[1]
    };
    let mut z = z0;
    let mut lo = None;
    let mut hi = None;
    while z <= z1 {
        if ok(z) {
            if lo.is_none() {
                lo = Some(z);
            }
            hi = Some(z);
        } else if lo.is_some() {
            break;
        }
        z += a.step_m;
    }
    let (Some(lo), Some(hi)) = (lo, hi) else {
        let (t1, _) = air.profile(s, g, 0.0);
        return Err(format!(
            "no height has both {}–{} bar and {}–{} K in this air ({} {:.0} K at {:.2} bar{})",
            a.p_bar[0],
            a.p_bar[1],
            a.t_k[0],
            a.t_k[1],
            if air.giant {
                "at the 1 bar level"
            } else {
                "at the surface"
            },
            t1,
            air.pressure_bar,
            if air.giant {
                ", hotter only deeper where the pressure is higher"
            } else {
                ""
            }
        ));
    };
    let (t_lo, p_lo) = air.profile(s, g, lo);
    let (t_hi, p_hi) = air.profile(s, g, hi);
    let wind = wind_m_s.unwrap_or(a.wind_m_s);
    let circuit = 2.0 * m::PI * (env.reference_radius_m + 0.5 * (lo + hi))
        / wind.max(1e-6)
        / sse_core::sheet::DAY;
    let mut below = vec![];
    if !air.giant {
        below.push(format!(
            "the surface {:.0} km below: {:.0} K and {:.2} bar",
            lo / 1000.0,
            air.t_surface_k,
            air.pressure_bar
        ));
    } else {
        below.push("no surface: the air gets hotter and denser all the way down".into());
    }
    Ok(AerostatBand {
        low_km: lo / 1000.0,
        high_km: hi / 1000.0,
        p_bar: [p_lo, p_hi],
        t_k: [t_lo, t_hi],
        wind_m_s: wind,
        circuit_days: circuit,
        below,
        why: vec![
            format!(
                "{} air (molar mass {:.4} kg/mol, c_p {:.0} J/kg/K) cooling {:.2} K per km to a {} bar tropopause",
                air.gas,
                air.molar_kg,
                air.cp_j_kg_k,
                air.lapse_k_km(s, g),
                s.tropopause_bar
            ),
            a.source.clone(),
        ],
    })
}
