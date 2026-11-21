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

/// A substance that can condense: Clausius-Clapeyron from its triple point
/// with a constant latent heat.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Substance {
    /// Gas key in [`crate::Conditions::gases`] (`h2o`, `co2`, `ch4`, ...).
    pub gas: String,
    /// Name the field uses (`water`, `carbon dioxide`, `methane`, ...).
    pub name: String,
    pub triple_k: f64,
    pub triple_pa: f64,
    /// Latent heat of the phase change that forms the cloud, J/kg.
    pub latent_j_kg: f64,
    pub molar_kg: f64,
    /// Density of the condensed phase, kg/m³ (turns a mass flux into depth).
    pub condensed_kg_m3: f64,
    /// Cloud colour, linear sRGB.
    pub colour: [f64; 3],
    pub source: String,
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
    pub moisture: Moisture,
    pub cloud: Cloud,
    /// Thermal wind depth in scale heights, and the share of it felt near the ground.
    pub thermal_depth_scale_heights: f64,
    pub surface_wind_share: f64,
    pub supply: Supply,
    pub circulation: Circulation,
    pub strengths: StrengthSettings,
    pub defaults: Defaults,
    pub substances: Vec<Substance>,
    pub references: Vec<Reference>,
}

impl ClimateSettings {
    /// The last pack in the stack that provides `climate` wins; a stack with
    /// none reads the base climate pack.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let Some(v) = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("climate"))
        else {
            return Self::from_packs(&[crate::base_pack()]);
        };
        let s: ClimateSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
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
        if s.substances.is_empty() {
            return Err("climate needs at least one substance".into());
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
