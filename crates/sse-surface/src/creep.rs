//! How fast loose ground creeps downslope: one topographic diffusivity for a
//! world's conditions, from the block `sse.surface.creep_diffusivity`
//! (packs/base-physics/formulas/surface.json). Crater fading (impacts.rs)
//! and slope wear (wear.rs) read the same rates, which are on the data sheet
//! (mixtures `lunar-soil-mature-mare` and `sediment`, group `hillslope`) and
//! reach the surface settings as sheet references, so an edited sheet moves
//! both.

use serde::{Deserialize, Serialize};
use sse_flow::Atom;
use std::sync::{Arc, OnceLock};

/// The three creep rates, SI.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CreepRates {
    /// Dry airless regolith stirred by small impacts, m² s⁻¹.
    pub dry_m2_s: f64,
    /// Added while the ground has liquid water on it, m² s⁻¹.
    pub wet_m2_s: f64,
    /// Added per pascal of surface air (wind), m² s⁻¹ Pa⁻¹.
    pub air_m2_s_pa: f64,
}

const BLOCK: &str = "sse.surface.creep_diffusivity";

fn block() -> Arc<sse_physics::Physics> {
    // The physics in force inside a kernel call; outside one (unit tests) the
    // shipped blocks, whose maths is the same.
    sse_physics::now().unwrap_or_else(|| {
        static BARE: OnceLock<Arc<sse_physics::Physics>> = OnceLock::new();
        BARE.get_or_init(|| Arc::new(sse_physics::Physics::bare(Arc::default())))
            .clone()
    })
}

/// The creep diffusivity, m² s⁻¹, for ground wet `wet_share` of the time
/// (0 to 1) under `pressure_pa` of air.
pub fn diffusivity(c: &CreepRates, wet_share: f64, pressure_pa: f64) -> f64 {
    let p = block();
    let b = p
        .blocks()
        .iter()
        .find(|a| a.decl().id == BLOCK)
        .unwrap_or_else(|| panic!("base formula block {BLOCK} is missing"));
    b.eval(
        &[
            wet_share,
            pressure_pa,
            c.dry_m2_s,
            c.wet_m2_s,
            c.air_m2_s_pa,
        ],
        &[],
    )
    .unwrap_or_else(|e| panic!("{BLOCK}: {e}"))[0]
}

/// Seconds in a Julian year and a million of them (conventions).
pub const YR_S: f64 = sse_core::sheet::YR_JULIAN;
pub const MYR_S: f64 = 1.0e6 * sse_core::sheet::YR_JULIAN;
