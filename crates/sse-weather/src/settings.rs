//! The weather pack's settings (`packs/base-weather`, `provides.other.weather`).
//! Every threshold and constant a rule uses lives here, with its source, so
//! Orac and a GM can see and change it.

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

/// A substance that can condense into cloud: Clausius-Clapeyron from its
/// triple point with a constant latent heat.
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

/// A cloud deck or haze made by chemistry rather than by condensation of a
/// gas already in the air (sulphuric acid on Venus, organic haze on Titan).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Photochemical {
    pub makes: String,
    /// Minimum mixing ratio of each gas it is made from.
    pub needs: BTreeMap<String, f64>,
    /// Least stellar flux to drive the chemistry, W/m².
    pub min_flux_w_m2: f64,
    /// Least surface pressure for it to hold together as a layer, bar.
    pub min_pressure_bar: f64,
    /// `deck` or `haze`.
    pub cloud_type: String,
    pub layer: usize,
    pub cover: f64,
    pub optical_depth: f64,
    pub colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Defaults {
    pub rotation_hours: f64,
    pub obliquity_deg: f64,
    pub gases: BTreeMap<String, f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Circulation {
    /// Wind scale in the Rhines length on worlds with a surface, m/s.
    pub wind_m_s: f64,
    /// Held and Hou: fractional equator-to-pole contrast of radiative
    /// equilibrium temperature, and tropopause depth in scale heights.
    pub delta_h: f64,
    pub tropopause_scale_heights: f64,
    /// Width of the rising belt and of each storm track, as a fraction of its cell.
    pub itcz_width: f64,
    pub storm_width: f64,
    /// Relative weight of rain in the rising belt, the storm tracks and everywhere.
    pub itcz_weight: f64,
    pub storm_weight: f64,
    pub rain_floor: f64,
    /// How far the rising belt follows the overhead sun: `k_ocean` over sea, `k_land` over land.
    pub itcz_follow_ocean: f64,
    pub itcz_follow_land: f64,
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
    /// Relative humidity of air over open liquid.
    pub humidity_over_liquid: f64,
    /// Ocean fraction at which evaporation is no longer limited by wet area.
    pub wet_saturation: f64,
    /// Cloud cover from moisture supply: `cover_max` × (1 − exp(−latent / `latent_scale_w_m2`)).
    pub cover_max: f64,
    pub latent_scale_w_m2: f64,
    /// Thin air: cover from frost and ice haze only.
    pub thin_cover: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WeatherSettings {
    pub bands: u32,
    pub season_samples: u32,
    pub defaults: Defaults,
    pub circulation: Circulation,
    pub supply: Supply,
    pub rules: Vec<RegimeRule>,
    pub regimes: BTreeMap<String, String>,
    pub substances: Vec<Substance>,
    pub photochemical: Vec<Photochemical>,
    pub references: Vec<Reference>,
}

/// Quantities a regime rule may test.
pub const QUANTITIES: [&str; 6] = [
    "pressure_bar",
    "solid",
    "star_locked",
    "hadley_edge_deg",
    "cells",
    "solar_day_hours",
];

impl WeatherSettings {
    /// The last pack in the stack that provides `weather` wins.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("weather"))
            .ok_or("no pack provides weather settings")?;
        let s: WeatherSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("weather settings: {e}"))?;
        for r in &s.rules {
            if !s.regimes.contains_key(&r.regime) {
                return Err(format!("rule names unknown regime {}", r.regime));
            }
            for q in r.when.keys() {
                if !QUANTITIES.contains(&q.as_str()) {
                    return Err(format!("rule for {} reads unknown quantity {q}", r.regime));
                }
            }
        }
        if s.rules.last().is_none_or(|r| !r.when.is_empty()) {
            return Err("the last weather rule must have no conditions (the fallback)".into());
        }
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every weather reference needs a DOI or a URL".into());
        }
        Ok(s)
    }
}
