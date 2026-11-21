//! SSE4 climate (W7a, weather review section 7): the **one** climate under
//! geology and weather. Before W7a there were four: erosion rained on a fixed
//! cosine, cover had its own temperature contrast and cell count, and each
//! weather block had its own temperature. Now:
//!
//! - [`conditions`]: the facts a climate reads, with a signed spin.
//! - [`ebm`]: the diffusive energy balance in latitude and season (North 1975).
//! - [`circulation`]: the Hadley edge, Rhines cells, their layout and the
//!   prevailing winds every reader shares.
//! - [`climate`]: the per-world result at a detail level (Glance, Orbit),
//!   keyed by world, code version, level and epoch, with a reuse cache.
//!
//! `sse-surface` (wear, cover) and `sse-weather` (both blocks) depend on this
//! crate and compute none of it themselves.

pub mod circulation;
pub mod climate;
pub mod conditions;
pub mod ebm;
pub mod settings;

pub use circulation::Circulation;
pub use climate::{glance, orbit, Cache, Climate, Glance, Level};
pub use conditions::{Conditions, Extra};
pub use settings::ClimateSettings;

pub const CLIMATE_VERSION: &str = "climate-v1";

/// The base climate pack (`packs/base-climate`).
pub fn base_pack() -> sse_pack::PackManifest {
    sse_pack::PackManifest::from_json(include_str!("../../../packs/base-climate/pack.json"))
        .expect("base-climate pack")
}

/// Settings from the base climate pack.
pub fn base_settings() -> ClimateSettings {
    ClimateSettings::from_packs(&[base_pack()]).expect("climate settings")
}

#[cfg(test)]
mod tests;
