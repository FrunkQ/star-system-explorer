//! Surface convection of stars and other bodies with a churning outer layer
//! (light track, stellar surfaces): granules, supergranules, how fast they
//! churn and how long each lives. One process for any body: the sizes come
//! from the photosphere's pressure scale height (the same block as an
//! atmosphere's, `sse.atmosphere.scale_height`), so a red supergiant gets a
//! few huge cells and a red dwarf many small ones, with no type gates.
//! `sse-activity` reads these for the star's surface and its magnetic
//! network; the renderer animates cells from the sizes and lifetimes.

use sse_core::math as m;
use sse_core::quantity::consts::*;

const T_SUN: f64 = sse_core::sheet::T_SUN;
const G_SUN: f64 = GM_SUN / (R_SUN * R_SUN);
const K_B: f64 = sse_core::sheet::K_B;
const AMU: f64 = sse_core::sheet::M_U;

/// Granule size, m: a multiple of the pressure scale height.
pub fn granule_size(scale_height_m: f64, factor: f64) -> f64 {
    factor * scale_height_m
}

/// Speed of the churning gas near the surface, m/s. Convective flux
/// F = σT⁴ carried as ρv³ with photospheric density ρ ∝ g μ / T (constant
/// opacity) gives v ∝ T^(5/3) g^(-1/3); normalised to the Sun and never
/// faster than sound.
pub fn convective_speed(teff_k: f64, g: f64, v_sun: f64, mu: f64) -> f64 {
    let v = v_sun * m::pow(teff_k / T_SUN, 5.0 / 3.0) * m::cbrt(G_SUN / g);
    v.min(sound_speed(teff_k, mu))
}

/// Adiabatic sound speed of a monatomic ideal gas, m/s.
pub fn sound_speed(t_k: f64, mu: f64) -> f64 {
    m::sqrt(5.0 / 3.0 * K_B * t_k / (mu * AMU))
}

/// A cell's life: `k` times its size over the speed that drives it.
pub fn cell_lifetime(size_m: f64, speed_m_s: f64, k: f64) -> f64 {
    k * size_m / speed_m_s
}
