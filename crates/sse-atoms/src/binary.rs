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

/// BX3: Peters 1964: m/s at which a circular orbit of separation `a` shrinks
/// by gravitational waves (negative), masses in kg.
pub fn gw_decay(m1: f64, m2: f64, a: f64, k: f64) -> f64 {
    let c = C;
    let c5 = c * c * c * c * c;
    -k * G * G * G * m1 * m2 * (m1 + m2) / (c5 * a * a * a)
}

/// BX3: Hurley, Tout and Pols 2002 eq. 50 (after Rappaport, Verbunt and Joss
/// 1983): kg m² s⁻² of spin carried off by a magnetised wind (negative), for
/// a star of `mass` with `envelope` of it convective, radius `radius` m,
/// turning at `omega` rad/s.
pub fn magnetic_braking(mass: f64, envelope: f64, radius: f64, omega: f64, k: f64) -> f64 {
    let msun = sse_core::sheet::M_SUN;
    let yr = JULIAN_YEAR;
    let rw = radius / R_SUN * omega * yr;
    -k * (envelope / mass) * rw * rw * rw * msun * R_SUN * R_SUN / (yr * yr)
}

/// BX2: Blaauw 1961, Brandt and Podsiadlowski 1995: the orbit right after one
/// of two bodies loses mass at once and is kicked. `ma` is the two's mass
/// after (kg), `r` the distance between them (m), (`vt`, `vr`) the relative
/// speed across the line between them and along it just before, and the kick
/// is (`kt` along the motion, `kr` outward, `kz` out of the plane), m/s.
/// Returns (new semi-major axis, negative when they come apart, and new
/// eccentricity).
pub fn kick_jump(ma: f64, r: f64, vt: f64, vr: f64, k: [f64; 3]) -> (f64, f64) {
    let ut = vt + k[0];
    let ur = vr + k[1];
    let v2 = ut * ut + ur * ur + k[2] * k[2];
    let inv = 2.0 / r - v2 / (G * ma);
    let h2 = r * r * (ut * ut + k[2] * k[2]);
    (1.0 / inv, m::sqrt((1.0 - h2 * inv / (G * ma)).max(0.0)))
}
