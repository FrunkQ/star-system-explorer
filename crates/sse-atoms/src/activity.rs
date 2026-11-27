//! Stellar activity (track O10; nebulae-and-light design section 4A): how
//! strongly a star with a churning outer layer is magnetically active, from
//! its mass and spin. The Rossby number (spin period over convective
//! turnover time) sets the X-ray brightness, the activity cycle and the
//! flare rate. `sse-activity` builds spots, cycles, flares and ejections
//! from these blocks.

use sse_core::math as m;
use sse_core::quantity::consts::*;

/// The Sun's X-ray fraction (Judge et al. 2003, as used by Wright et al. 2011).
pub const SUN_LOG_RX: f64 = -6.24;
/// Reference flare energy, J (10³⁴ erg).
pub const E_REF_J: f64 = 1e27;
/// Flares above E_REF per second per watt of X-ray luminosity, times E_REF:
/// k = N(>E_REF) E_REF / L_X, fixed by Vasilyev et al. 2024 (8.63e-3 per
/// year above 10³⁴ erg) on a star with the Sun's L_X (10^-6.24 L☉).
pub const FLARE_K: f64 = 8.63e-3 / JULIAN_YEAR * E_REF_J / (5.754_399e-7 * L_SUN);

const M_SUN: f64 = GM_SUN / G;

/// Convective turnover time, s, from mass in kg (Wright et al. 2011 eq. 11).
/// Outside 0.09 to 1.36 M☉ the fit is extrapolated; the caller says so.
pub fn convective_turnover(mass_kg: f64) -> f64 {
    let x = m::log10(mass_kg / M_SUN);
    m::pow(10.0, 1.16 - 1.49 * x - 0.54 * x * x) * DAY
}

/// L_X / L_bol from the Rossby number, with saturation.
pub fn xray_fraction(ro: f64, log_rx_sat: f64, ro_sat: f64, beta: f64) -> f64 {
    let sat = m::pow(10.0, log_rx_sat);
    if ro <= ro_sat {
        sat
    } else {
        sat * m::pow(ro / ro_sat, beta)
    }
}

/// Activity cycle period, s, from the rotation period, s: log P_cyc = a + n log P_rot (days).
pub fn cycle_period(p_rot_s: f64, a: f64, n: f64) -> f64 {
    m::pow(10.0, a + n * m::log10(p_rot_s / DAY)) * DAY
}

/// Flares per second above energy `e` J for X-ray luminosity `lx` W:
/// N(>E) = k (L_X / E_ref) (E / E_ref)^(1 − α).
pub fn flare_rate_above(lx: f64, e: f64, k: f64, alpha: f64) -> f64 {
    k * lx / E_REF_J * m::pow(e / E_REF_J, 1.0 - alpha)
}

// SG6 prominences: the star's field, its corona, and where gas can be held
// up. Each sum is an Equation in `packs/base-physics/formulas/activity.json`
// (a test holds the two together).

/// Mean surface field, T, from the Rossby number (Reiners et al. 2022): a
/// steep slow branch and a nearly flat fast one, meeting at `ro_sat`.
/// Fields in gauss at Ro = 1: `b_fast_g`, `b_slow_g`.
pub fn surface_field(
    ro: f64,
    b_fast_g: f64,
    slope_fast: f64,
    b_slow_g: f64,
    slope_slow: f64,
    ro_sat: f64,
) -> f64 {
    let g = if ro <= ro_sat {
        b_fast_g * m::pow(ro, slope_fast)
    } else {
        b_slow_g * m::pow(ro, slope_slow)
    };
    g * 1e-4
}

/// Mean coronal temperature, K, from the X-ray power and radius (Johnstone
/// and Güdel 2015): 0.11 F_X^0.26 MK, F_X in erg s⁻¹ cm⁻².
pub fn corona_temperature(lx_w: f64, radius_m: f64, k: f64, n: f64) -> f64 {
    let fx = lx_w.max(0.0) / (4.0 * m::PI * radius_m * radius_m) * 1000.0;
    k * m::pow(fx, n) * 1e6
}

/// Particles per m³ at the corona's foot: one hydrostatic scale height over
/// the whole star glowing optically thin, L_X = n² Λ 4πR² H.
pub fn corona_base_density(lx_w: f64, radius_m: f64, t_k: f64, g: f64, loss: f64, mu: f64) -> f64 {
    let h = sse_core::sheet::K_B * t_k / (mu * sse_core::sheet::M_U * g);
    m::sqrt(lx_w.max(0.0) / (loss * 4.0 * m::PI * radius_m * radius_m * h))
}

/// Co-rotation radius, m: where a Kepler orbit keeps pace with the spin (the
/// `sse.orbit.semi_major_axis_from_period` block with the star's spin period).
pub fn corotation_radius(mass_kg: f64, period_s: f64) -> f64 {
    m::cbrt(G * mass_kg * period_s * period_s / (4.0 * m::PI * m::PI))
}

/// Reach of the closed field, stellar radii: the Sun's source surface
/// `r_ref` at field `b_ref_g` gauss, growing as `index` power of the field.
pub fn closed_corona_radius(field_t: f64, r_ref: f64, b_ref_g: f64, index: f64) -> f64 {
    (r_ref * m::pow(field_t / (b_ref_g * 1e-4), index)).max(1.0)
}

/// Surface spin rate at latitude `lat` (radians) over the equator's:
/// (A + B sin²φ + C sin⁴φ) / A (Snodgrass and Ulrich 1990).
pub fn differential_rotation(lat: f64, a: f64, b: f64, c: f64) -> f64 {
    let s = m::sin(lat);
    let s2 = s * s;
    (a + b * s2 + c * s2 * s2) / a
}

/// Mean latitude of new spot groups, radians, a share `progress` of the way
/// through the cycle from its minimum (Spörer's law, a straight line from
/// `start_deg` to `end_deg`).
pub fn butterfly_latitude(progress: f64, start_deg: f64, end_deg: f64) -> f64 {
    (start_deg - (start_deg - end_deg) * progress.clamp(0.0, 1.0)) * m::PI / 180.0
}

/// Joy's law: a spot group's tilt, radians, at latitude `lat`.
pub fn joy_tilt(lat: f64, scale_deg: f64) -> f64 {
    scale_deg * m::PI / 180.0 * m::sin(lat.abs())
}

/// Gnevyshev-Waldmeier: a spot group's life, s, from its peak area in
/// millionths of a hemisphere and the decay rate in the same a day.
pub fn spot_group_life(area_msh: f64, decay_msh_day: f64) -> f64 {
    area_msh / decay_msh_day * sse_core::sheet::DAY
}
