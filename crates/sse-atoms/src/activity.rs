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
