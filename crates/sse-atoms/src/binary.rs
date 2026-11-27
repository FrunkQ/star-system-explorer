//! BX1: Rust copies of the pair blocks (`sse.orbit.roche_lobe`,
//! `sse.orbit.overflow_rate`, `sse.star.thermal_timescale`), held to them by
//! `pair_blocks_match_their_sums`.

use sse_core::math as m;
use sse_core::quantity::consts::*;

/// Eggleton 1983: Roche lobe radius for mass ratio `q` (this star over its
/// partner) at separation `a`, in a's unit.
pub fn roche_lobe(q: f64, a: f64) -> f64 {
    let q3 = m::cbrt(q);
    let q23 = q3 * q3;
    a * 0.49 * q23 / (0.6 * q23 + m::ln(1.0 + q3))
}

/// Hurley, Tout and Pols 2002 eq. 58: kg/s off a star of `mass_kg` whose
/// radius is `fill` times its lobe's; the rate stops growing above `cap_msun`.
pub fn overflow_rate(mass_kg: f64, fill: f64, cap_msun: f64) -> f64 {
    let msun = sse_core::sheet::M_SUN;
    let mc = (mass_kg / msun).min(cap_msun);
    let over = m::ln(fill.max(1.0));
    3e-6 * mc * mc * over * over * over * msun / JULIAN_YEAR
}

/// Thermal (Kelvin-Helmholtz) timescale, s: `share` G M (M − Mc) / (R L).
pub fn thermal_timescale(mass: f64, core: f64, radius: f64, lum: f64, share: f64) -> f64 {
    share * G * mass * (mass - core).max(0.0) / (radius * lum)
}
