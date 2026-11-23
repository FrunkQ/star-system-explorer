//! What stops a wind at a body, and how conducting layers answer a changing
//! field (magnetic fields design 6.1 and 6.6, stage MF6). Every body gets all
//! three possible obstacles, computed: its own field (the stand-off block
//! `sse.light.magnetopause_radius`, or the field-pressure balance here for a
//! moon inside a host's bubble), its ionosphere pushed against by the wind,
//! and its bare surface. The outermost is the obstacle. No switch on "has a
//! magnetosphere".

use crate::light::MU0;
use sse_core::math as m;

/// Height, m, at which the air's own pressure P₀ exp(−h/H) falls to `k`
/// times the wind's ram pressure: a stand-in for the ionopause (ionospheric
/// plasma pressure is a small, fairly steady share of the neutral pressure
/// there). Zero if the air cannot hold the wind off above the ground.
pub fn ionopause_altitude(surface_pressure: f64, scale_height: f64, ram: f64, k: f64) -> f64 {
    if surface_pressure <= 0.0 || ram <= 0.0 {
        return 0.0;
    }
    (scale_height * m::ln(surface_pressure / (k * ram))).max(0.0)
}

/// Shue et al. 1998 flaring exponent, from the wind's ram pressure (Pa) and
/// the north-south field (T, positive northward).
pub fn shue_alpha(ram_pa: f64, bz_t: f64) -> f64 {
    let p_npa = (ram_pa * 1e9).max(1e-6);
    (0.58 - 0.007 * bz_t * 1e9) * (1.0 + 0.024 * m::ln(p_npa))
}

/// Shue boundary distance at angle θ from the nose.
pub fn shue_radius(r0: f64, alpha: f64, theta: f64) -> f64 {
    r0 * m::pow(2.0 / (1.0 + m::cos(theta)), alpha)
}

/// Stand-off of a dipole moment M (A m²) in an outside field B (T) and
/// plasma pressure p (Pa): the compressed dipole (2 B_dip) balances
/// B² / 2μ₀ + p.
pub fn standoff_in_field(moment: f64, outside_b: f64, plasma_pa: f64) -> f64 {
    let b_need = m::sqrt(outside_b * outside_b + 2.0 * MU0 * plasma_pa.max(0.0));
    if moment <= 0.0 || b_need <= 0.0 {
        return 0.0;
    }
    m::cbrt(MU0 * moment / (2.0 * m::PI * b_need))
}

/// Skin depth, m, of a conductor σ (S/m) at angular frequency ω.
pub fn skin_depth(sigma: f64, omega: f64) -> f64 {
    m::sqrt(2.0 / (MU0 * sigma * omega))
}

/// Induced response of a conducting shell (radius r, thickness h, σ) to a
/// field changing with period T: thin-shell form A = κ / √(1 + κ²),
/// κ = μ₀ σ ω r h_eff / 3, with the thickness capped at the skin depth.
pub fn induction_amplitude(sigma: f64, thickness: f64, radius: f64, period: f64) -> f64 {
    if sigma <= 0.0 || thickness <= 0.0 || period <= 0.0 {
        return 0.0;
    }
    let omega = 2.0 * m::PI / period;
    let h = thickness.min(skin_depth(sigma, omega));
    let k = MU0 * sigma * omega * radius * h / 3.0;
    k / m::sqrt(1.0 + k * k)
}
