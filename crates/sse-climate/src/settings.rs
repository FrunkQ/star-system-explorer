//! The climate pack's settings (`packs/base-climate`, `provides.other.climate`).
//! Every constant the energy balance and the shared circulation numbers use
//! lives here, with its source, so Orac and a GM can see and change it.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Reference {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// A substance that can condense, as the climate reads it: the weather
/// track's view (`sse.volatile.species`, W8 and matter pack MP3) of a row of
/// the master data sheet that has a triple point and a vapour curve. The
/// climate keeps no substance table of its own, so ground frost, cloud and
/// ice lines read one curve per substance.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Substance {
    /// The sheet's substance id, also the gas key in
    /// [`crate::Conditions::gases`] (`h2o`, `co2`, `ch4`, ...).
    pub gas: String,
    /// The sheet's plain name (`water`, `carbon dioxide`, ...).
    pub name: String,
    pub triple_k: f64,
    pub triple_pa: f64,
    /// Latent heat of the phase change that forms the cloud, J/kg: the
    /// sheet's heat of vaporisation where it has one, else its sublimation
    /// heat, else the vapour curve's slope at the triple point.
    pub latent_j_kg: f64,
    /// Molar mass worked out from the sheet's formula, kg/mol.
    pub molar_kg: f64,
    /// Density of the condensed phase, kg/m³ (liquid where the sheet has one,
    /// else solid).
    pub condensed_kg_m3: f64,
    /// Cloud colour, linear sRGB.
    pub colour: [f64; 3],
    pub source: String,
    /// The sheet row's `phases` group: the solid's vapour curves (read by
    /// [`crate::ebm::p_sat`] below the triple point).
    #[serde(default)]
    pub phases: serde_json::Map<String, serde_json::Value>,
}

/// The 10 m wind (`crate::wind`): the circulation's prevailing wind as its
/// mean, storms as its spread.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Wind10Settings {
    /// Spread of each 10 m wind component as a share of the eddy wind,
    /// where eddies carry all the heat (in air as dense as Earth's).
    pub gust_share: f64,
    pub source: String,
}

/// Tides and wind waves read for a world's card and a landing site.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SeaSettings {
    /// Longest stretch of open water one wind blows over, m: a storm's
    /// size, whatever the width of the sea.
    pub storm_fetch_m: f64,
    pub source: String,
}

/// W7c part 1: land and sea as two columns in each band, coupled by the
/// band's own diffusion across its coasts.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LandSeaSettings {
    /// Coasts a line along a band crosses when the world does not say
    /// (2: one continent and one sea per band).
    pub coasts_per_band: f64,
    /// Upper bound on the land-sea coupling per square metre of band,
    /// W m⁻² K⁻¹ (the formula grows as 1 / cos² φ toward the poles).
    pub max_coupling_w_m2_k: f64,
    /// Column-mean zonal wind as a share of the thermal wind at the top of
    /// the thermal depth (a wind rising linearly with height gives ½).
    pub column_wind_ratio: f64,
    pub source: String,
}

/// `sse.volatile.exchange` (W8 part 2): ice of the air's main gas between
/// ground and air.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExchangeSettings {
    /// Run the exchange when the main gas's frost point comes within this of
    /// the coldest band of the plain cycle, K.
    pub trigger_margin_k: f64,
    /// Most steps of the frost-point search per time step (regula falsi).
    pub bisection_steps: u32,
    /// The coldest frost point the search considers, K.
    pub lowest_frost_k: f64,
    /// With ground ice given: at most this many carried-forward years, and
    /// at most this long in all.
    pub migration_steps: u32,
    pub migration_span_yr: f64,
    /// Thermal emissivity of a frosted band (one value for every ice until
    /// the sheet carries per-ice optics).
    pub frost_emissivity: f64,
    pub source: String,
}

/// The base master data sheet (`packs/base-reference`).
pub fn base_reference_pack() -> sse_pack::PackManifest {
    sse_pack::PackManifest::from_json(include_str!("../../../packs/base-reference/pack.json"))
        .expect("base-reference pack")
}

fn num(v: Option<&serde_json::Value>) -> Option<f64> {
    v?.get("v")?.as_f64()
}

/// The volatile species (`sse.volatile.species`): every substance on the
/// sheet with a triple point and a vapour curve, in id order. A stack with
/// no sheet reads the base one.
pub fn volatile_species(stack: &[sse_pack::PackManifest]) -> Result<Vec<Substance>, String> {
    let r = if stack.iter().any(|p| !p.provides.reference.is_empty()) {
        sse_reference::Reference::from_packs(stack)
    } else {
        sse_reference::Reference::from_packs(&[base_reference_pack()])
    }
    .map_err(|e| format!("master data sheet: {}", e.join("; ")))?;
    let mut out = vec![];
    let Some(rows) = r.tables.get("substances") else {
        return Ok(out);
    };
    for (id, row) in rows {
        let Some(ph) = row.get("phases").and_then(|v| v.as_object()) else {
            continue;
        };
        let (Some(tk), Some(tp), Some(_)) = (
            num(ph.get("triple_point_k")),
            num(ph.get("triple_point_pa")),
            ph.get("vapour_pressure"),
        ) else {
            continue;
        };
        let Some(mk) = r.molar_mass_kg_mol(id) else {
            continue;
        };
        let latent = num(ph.get("latent_vaporisation"))
            .or_else(|| num(ph.get("latent_sublimation")))
            .or_else(|| sse_reference::sublimation_latent_j_mol(ph, tk).map(|l| l / mk))
            .ok_or_else(|| format!("substance {id}: no latent heat and no usable vapour curve"))?;
        let dens = num(ph.get("density_liquid"))
            .or_else(|| num(ph.get("density_solid")))
            .ok_or_else(|| format!("substance {id}: a volatile needs a liquid or solid density"))?;
        let colour = row
            .get("optical")
            .and_then(|o| o.get("condensed_colour_rgb"))
            .and_then(|v| v.get("v"))
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_f64()).collect::<Vec<_>>())
            .filter(|a| a.len() == 3)
            .map(|a| [a[0], a[1], a[2]])
            .ok_or_else(|| {
                format!("substance {id}: a volatile needs optical.condensed_colour_rgb")
            })?;
        let name = row
            .get("names")
            .and_then(|n| n.get("plain"))
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
        let mut srcs: Vec<String> = ["triple_point_k", "vapour_pressure"]
            .iter()
            .filter_map(|k| ph.get(*k)?.get("src")?.as_str().map(String::from))
            .collect();
        srcs.dedup();
        out.push(Substance {
            gas: id.clone(),
            name,
            triple_k: tk,
            triple_pa: tp,
            latent_j_kg: latent,
            molar_kg: mk,
            condensed_kg_m3: dens,
            colour,
            source: format!("master data sheet, substance {id} ({})", srcs.join(", ")),
            phases: ph.clone(),
        });
    }
    Ok(out)
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Defaults {
    /// Signed spin, hours.
    pub rotation_hours: f64,
    pub obliquity_deg: f64,
    pub gases: BTreeMap<String, f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Olr {
    /// B as a fraction of the world's blackbody slope 4 OLR / T.
    pub b_hat: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Diffusion {
    /// D on Earth, W m⁻² K⁻¹.
    pub d_earth_w_m2_k: f64,
    /// D scales as (column / Earth's)^`column_power` × (Ω_Earth / Ω)^`spin_power`.
    pub column_power: f64,
    pub spin_power: f64,
    /// Upper bound on D / B (beyond it the world is isothermal anyway).
    pub max_over_b: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ice {
    /// Albedo added where water freezes over, smoothly around `t_ice_k`.
    pub albedo_step: f64,
    pub t_ice_k: f64,
    pub width_k: f64,
    pub source: String,
}

/// How much heat a square metre stores through the year (seasons lag and
/// shrink where it is large).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HeatCapacity {
    /// Depth of the sea's mixed layer, m.
    pub mixed_layer_m: f64,
    /// Volumetric heat capacity of sea water, J m⁻³ K⁻¹.
    pub water_j_m3_k: f64,
    /// Thermal inertia of the ground, J m⁻² K⁻¹ s⁻½.
    pub ground_inertia_si: f64,
    pub steps_per_season: u32,
    /// Years stepped from the steady start before the last is read.
    pub spin_up_years: u32,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Moisture {
    /// Bulk exchange velocity C_E |U|, m/s.
    pub exchange_m_s: f64,
    /// Relative humidity of the air over the liquid.
    pub humidity: f64,
    /// Eddy moisture transport as a multiple of the heat diffusivity.
    pub eddy_ratio: f64,
    /// Width σ (in sine of latitude) of the Hadley cell's share of the
    /// transport, on a world whose Hadley edge is `hadley_reference_deg`.
    pub hadley_width_x: f64,
    pub hadley_reference_deg: f64,
    /// Upper-branch moist static energy over the equator's surface value.
    pub h_top_ratio: f64,
    /// Least gross moist stability, K (keeps the division finite).
    pub min_stability_k: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cloud {
    /// Cover from latent heat released: `cover_max` (1 − exp(−L P / `latent_scale_w_m2`)).
    pub cover_max: f64,
    pub latent_scale_w_m2: f64,
    /// Extra cover in baroclinic zones: `gradient_cover` × min(1, |dT/dφ| / `gradient_scale_k_deg`).
    pub gradient_cover: f64,
    pub gradient_scale_k_deg: f64,
    /// Worlds without a surface liquid: cover where a gas saturates,
    /// `saturation_cover` × min(1, ln(p / p_sat) / `saturation_ln_scale`).
    pub saturation_cover: f64,
    pub saturation_ln_scale: f64,
    /// Below this surface pressure there is no cloud and no wind, bar.
    pub min_pressure_bar: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Supply {
    /// Share of the sunlight reaching the ground that goes into evaporation
    /// where the ground is wet.
    pub latent_share: f64,
    /// Sunlight reaching the ground: `earth_fraction` × (column / Earth's)^(−`column_power`).
    pub earth_fraction: f64,
    pub column_power: f64,
    /// Relative humidity of air over open liquid (cloud base height).
    pub humidity_over_liquid: f64,
    /// Ocean fraction at which evaporation is no longer limited by wet area.
    pub wet_saturation: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Range {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

impl Range {
    pub fn holds(&self, v: f64) -> bool {
        self.min.is_none_or(|lo| v >= lo) && self.max.is_none_or(|hi| v < hi)
    }
}

/// One circulation rule: the first whose conditions all hold names the regime.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RegimeRule {
    pub regime: String,
    pub when: BTreeMap<String, Range>,
    pub because: String,
}

/// Quantities a regime rule may test.
/// What a label rule may read: facts and the strengths (W7b). The physics
/// never reads the label.
pub const QUANTITIES: [&str; 13] = [
    "pressure_bar",
    "solid",
    "star_locked",
    "hadley_edge_deg",
    "cells",
    "solar_day_hours",
    "radiative_days",
    "super_rotation",
    "baroclinic",
    "day_night",
    "collapse",
    "collisional",
    "thermal_rossby",
];

/// The process strength constants (W7b, `strengths.rs`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StrengthSettings {
    /// Eddy wind as a share of the thermal wind scale (the Rhines length's U).
    pub eddy_share: f64,
    /// Super-rotation: Ro_T where the strength is ½, and how steeply it turns on.
    pub super_rotation_critical: f64,
    pub super_rotation_steepness: f64,
    /// Lindzen and Farrell's 0.31 in σ = 0.31 f (dU/dz) / N.
    pub lindzen_farrell: f64,
    /// N² as a share of the isothermal (g/T)(g/c_p).
    pub stable_share: f64,
    /// Surface drag coefficient C_D and Komacek and Showman's Δln p.
    pub drag_coefficient: f64,
    pub log_pressure_span: f64,
    /// Wordsworth 2015's grey opacity κ, m²/kg.
    pub thin_opacity_m2_kg: f64,
    /// Width of the collapse step as a share of the condensation temperature.
    pub collapse_width: f64,
    /// Kinetic diameter of an air molecule, m (N₂ 3.7e-10).
    pub molecule_diameter_m: f64,
    /// The range each process's paper tested; a world outside it is marked.
    #[serde(default)]
    pub tested: Vec<Tested>,
    pub source: String,
}

/// One tested range: `process` was checked by `by` for `quantity` within
/// [min, max] (either bound may be open).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tested {
    pub process: String,
    pub quantity: String,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    pub by: String,
}

/// The water a place receives (W9, `water.rs`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WaterSettings {
    /// Snow falls below the freezing point plus this, K (Jennings et al. 2018).
    pub snow_offset_k: f64,
    /// Scatter of the air temperature about a season's mean, K.
    pub sigma_k: f64,
    /// Degree-day melt factors, mm of water per K per day (Braithwaite 1995).
    pub ddf_snow_mm: f64,
    pub ddf_ice_mm: f64,
    /// Fu's ω (2.6 matches Budyko's curve, Zhang et al. 2004).
    pub fu_omega: f64,
    pub source: String,
}

/// Orbital forcing and ice ages (W9 part 2, `forcing.rs`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ForcingSettings {
    /// Earth's dynamical ellipticity (C − A)/C (Zeebe 2022 Table 1).
    pub earth_ellipticity: f64,
    /// E_d scales as (ω / ω_Earth)^this (hydrostatic: 2).
    pub ellipticity_spin_power: f64,
    /// g_L in β = g_L (a/a_L)³ (m_L/M) (Zeebe 2022 eq. 5).
    pub moon_geometry: f64,
    /// Window of the history read for one epoch, years, sampled every
    /// `sample_yr` and integrated in steps of `step_yr`.
    pub window_yr: f64,
    pub sample_yr: f64,
    pub step_yr: f64,
    /// Huybers 2006: summer days are those of at least this insolation, W/m².
    pub summer_threshold_w_m2: f64,
    /// Steps through the year for the summer energy.
    pub year_steps: u32,
    /// The sensitivity probe: the climate at this much less tilt, degrees.
    pub probe_obliquity_deg: f64,
    /// Inclinations not published for a sibling are drawn up to this, degrees.
    pub drawn_inclination_deg: f64,
    pub source: String,
}

/// A sand grain saltation reads.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Grain {
    pub name: String,
    pub diameter_m: f64,
    pub density_kg_m3: f64,
    /// Ground colder than this is made of this grain (the last has none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub below_k: Option<f64>,
}

/// Lancaster's dune mobility index bounds.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Mobility {
    pub inactive: f64,
    pub active: f64,
    /// Least P / PE the index divides by.
    pub aridity_floor: f64,
}

/// `sse.wind.saltation` (W9, `saltation.rs`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SaltationSettings {
    /// Shao and Lu 2000 (Kok et al. 2012 eq. 2.8): u*ft = A_N sqrt(...), and the cohesion γ, N/m.
    pub a_n: f64,
    pub cohesion_n_m: f64,
    /// Impact over fluid threshold for loose sand on Earth (Kok 2010).
    pub impact_ratio: f64,
    /// The impact threshold is evaluated with air no thinner than this, kg/m³.
    pub impact_reference_kg_m3: f64,
    pub karman: f64,
    pub wind_height_m: f64,
    /// z0 = D × this (Kok et al. 2012: D/30).
    pub roughness_per_diameter: f64,
    /// Weibull shape of the wind about its mean (Kok 2010: k = 2).
    pub weibull_k: f64,
    /// The reference-height wind as a share of the climate's near-ground wind.
    pub ten_metre_share: f64,
    /// Share of the eddy wind felt as gusts near the ground (in air as dense as Earth's).
    pub gust_share: f64,
    /// Independent winds a year (ten-minute means: 52,596).
    pub gusts_per_year: f64,
    /// Width of the lofting step as a share of the main gas's frost point.
    pub lofting_width: f64,
    pub grains: Vec<Grain>,
    pub mobility: Mobility,
    pub source: String,
}

/// One area rule: the first whose conditions all hold names the area.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AreaRule {
    pub area: String,
    pub when: BTreeMap<String, Range>,
    pub because: String,
}

/// Per-area climate labels (W9): rules over a place's water supply.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Areas {
    pub rules: Vec<AreaRule>,
    pub labels: BTreeMap<String, String>,
    pub source: String,
}

/// Quantities an area rule may test (`water::quantity`).
pub const AREA_QUANTITIES: [&str; 12] = [
    "liquid",
    "liquid_water",
    "ice_balance_m_yr",
    "precip_m_yr",
    "runoff_m_yr",
    "aridity",
    "p_over_pth",
    "t_cold_c",
    "t_warm_c",
    "t_mean_c",
    "t_warm_over_freeze_k",
    "snow_share",
];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Circulation {
    /// Held and Hou: fractional equator-to-pole contrast of radiative
    /// equilibrium temperature, and tropopause depth in scale heights.
    pub delta_h: f64,
    pub tropopause_scale_heights: f64,
    /// The regime label: the first rule whose conditions all hold.
    pub rules: Vec<RegimeRule>,
    pub regimes: BTreeMap<String, String>,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClimateSettings {
    #[serde(default)]
    pub about: String,
    pub bands: u32,
    pub seasons: u32,
    pub iterations: u32,
    pub relax: f64,
    pub olr: Olr,
    pub diffusion: Diffusion,
    pub ice: Ice,
    pub heat_capacity: HeatCapacity,
    pub land_sea: LandSeaSettings,
    pub sea: SeaSettings,
    pub wind10: Wind10Settings,
    pub moisture: Moisture,
    pub cloud: Cloud,
    /// Thermal wind depth in scale heights, and the share of it felt near the ground.
    pub thermal_depth_scale_heights: f64,
    pub surface_wind_share: f64,
    pub supply: Supply,
    pub circulation: Circulation,
    pub strengths: StrengthSettings,
    pub water: WaterSettings,
    pub saltation: SaltationSettings,
    pub areas: Areas,
    pub forcing: ForcingSettings,
    pub exchange: ExchangeSettings,
    pub defaults: Defaults,
    /// Filled from the master data sheet (`volatile_species`); a climate
    /// pack may not list its own (retired with W8).
    #[serde(default)]
    pub substances: Vec<Substance>,
    pub references: Vec<Reference>,
}

impl ClimateSettings {
    /// The last pack in the stack that provides `climate` wins; a stack with
    /// none reads the base climate pack.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let species = volatile_species(stack)?;
        let base = [crate::base_pack()];
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("climate"))
            .or_else(|| base[0].provides.other.get("climate"))
            .ok_or("base climate pack provides no climate")?;
        let mut s: ClimateSettings = sse_reference::refs::with_sheet(stack, v)
            .and_then(|j| serde_json::from_value(j).map_err(|e| e.to_string()))
            .map_err(|e| format!("climate settings: {e}"))?;
        if s.bands < 4 || s.seasons == 0 || s.iterations == 0 {
            return Err("climate needs at least 4 bands, 1 season and 1 iteration".into());
        }
        let cc = &s.circulation;
        for r in &cc.rules {
            if !cc.regimes.contains_key(&r.regime) {
                return Err(format!("rule names unknown regime {}", r.regime));
            }
            for q in r.when.keys() {
                if !QUANTITIES.contains(&q.as_str()) {
                    return Err(format!("rule for {} reads unknown quantity {q}", r.regime));
                }
            }
        }
        if cc.rules.last().is_none_or(|r| !r.when.is_empty()) {
            return Err("the last circulation rule must have no conditions (the fallback)".into());
        }
        for r in &s.areas.rules {
            if !s.areas.labels.contains_key(&r.area) {
                return Err(format!("area rule names unknown area {}", r.area));
            }
            for q in r.when.keys() {
                if !AREA_QUANTITIES.contains(&q.as_str()) {
                    return Err(format!(
                        "area rule for {} reads unknown quantity {q}",
                        r.area
                    ));
                }
            }
        }
        if s.areas.rules.last().is_none_or(|r| !r.when.is_empty()) {
            return Err("the last area rule must have no conditions (the fallback)".into());
        }
        crate::saltation::check(&s.saltation)?;
        if !s.substances.is_empty() {
            return Err("climate.substances is retired (W8): substances come from the master data sheet's rows with a vapour curve".into());
        }
        s.substances = species;
        if s.substances.is_empty() {
            return Err(
                "climate needs at least one substance with a vapour curve on the master data sheet"
                    .into(),
            );
        }
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every climate reference needs a DOI or a URL".into());
        }
        Ok(s)
    }

    pub fn substance(&self, name: &str) -> Option<&Substance> {
        self.substances.iter().find(|s| s.name == name)
    }
}
