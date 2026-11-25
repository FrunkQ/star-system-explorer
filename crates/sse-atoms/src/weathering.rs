//! Space weathering of bare ground on airless worlds (SW1): the star's wind
//! and micrometeorites leave nanophase iron on soil grains, darkening and
//! reddening it as it matures. The blocks are Equations in
//! `packs/base-physics/formulas/surface.json`; these are the same sums for
//! the surface tiles (a test holds the two together):
//!
//! - `sse.surface.space_weathering`: maturity from exposure time, the wind's
//!   flux against 1 AU and whether the wind reaches the ground;
//! - `sse.surface.weathered_reflectance`: each band's reflectance factor,
//!   mature over fresh raised to the maturity (exactly 1 when fresh).

use sse_core::math as m;

/// Time for airless soil at 1 AU to come within 1/e of full maturity, s
/// (about 300 Myr; fitted to the Moon's rays fading over about 1 Gyr,
/// Grier et al. 2001). A stand-in in the fudge register.
pub const TAU_1AU_S: f64 = 9.5e15;
/// The wind's share of the weathering at 1 AU (Pieters and Noble 2016:
/// ions and micrometeorites comparable). A stand-in in the fudge register.
pub const WIND_SHARE: f64 = 0.5;
/// Mature over fresh reflectance of the same soil at 650, 550 and 450 nm
/// (Hapke 2001; Lucey and Riner 2011). A stand-in in the fudge register.
pub const MATURE_OVER_FRESH: [f64; 3] = [0.62, 0.55, 0.48];

/// Maturity, 0 fresh to 1 mature, after `t_s` seconds open to space.
pub fn maturity(t_s: f64, wind_ratio: f64, wind_open: bool, tau_s: f64, wind_share: f64) -> f64 {
    let o = if wind_open { 1.0 } else { 0.0 };
    let rate = wind_share * wind_ratio * o + (1.0 - wind_share);
    1.0 - m::exp(-t_s.max(0.0) * rate / tau_s)
}

/// Reflectance factors at 650, 550 and 450 nm for soil of maturity `mat`.
pub fn factors(mat: f64, k: [f64; 3]) -> [f64; 3] {
    let x = mat.clamp(0.0, 1.0);
    if x == 0.0 {
        return [1.0, 1.0, 1.0];
    }
    [m::pow(k[0], x), m::pow(k[1], x), m::pow(k[2], x)]
}
