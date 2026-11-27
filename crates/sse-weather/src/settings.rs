//! The weather pack's settings (`packs/base-weather`, `provides.other.weather`).
//! Every threshold and constant a rule uses lives here, with its source, so
//! Orac and a GM can see and change it.

use serde::{Deserialize, Serialize};
pub use sse_climate::settings::{Range, Reference, RegimeRule, Substance};
use sse_climate::ClimateSettings;
use std::collections::BTreeMap;

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
    /// Typical particle radius, µm (0: no measured size held).
    #[serde(default)]
    pub particle_radius_um: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Circulation {
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

/// The zonal weather's cloud drawing (pack `weather.clouds`; fudge
/// W-clouds). Pairs are [sinking air, rising air], blended by how deep the
/// towers grow.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Clouds {
    /// Optical depth: `tau_floor + tau_rise` x the rain over the wettest band's.
    pub tau_floor: f64,
    pub tau_rise: f64,
    /// Deep towers grow in from this share of the peak rain, over this span.
    pub towers_from: f64,
    pub towers_span: f64,
    /// Winds strengthen by this share per layer up toward the jets.
    pub wind_rise_per_layer: f64,
    pub low_tau_share: [f64; 2],
    pub low_cover_share: [f64; 2],
    pub towers_cover_share: f64,
    pub high_cover_share: [f64; 2],
    pub high_tau: [f64; 2],
    /// Thin air: the frost haze's optical depth.
    pub thin_air_tau: f64,
    #[serde(default)]
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WeatherSettings {
    pub season_samples: u32,
    pub circulation: Circulation,
    /// Thin air: cover from frost and ice haze only (drawing).
    pub thin_cover: f64,
    /// How the zonal weather draws its cloud layers (pack `clouds`).
    pub clouds: Clouds,
    pub photochemical: Vec<Photochemical>,
    pub references: Vec<Reference>,
    /// The one climate this weather is drawn on (W7a): the climate pack in
    /// the same stack, or the base climate pack.
    #[serde(skip, default = "sse_climate::base_settings")]
    pub climate: ClimateSettings,
}

impl WeatherSettings {
    /// The last pack in the stack that provides `weather` wins.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("weather"))
            .ok_or("no pack provides weather settings")?;
        let mut s: WeatherSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("weather settings: {e}"))?;
        s.climate = ClimateSettings::from_packs(stack)?;
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every weather reference needs a DOI or a URL".into());
        }
        Ok(s)
    }
}
