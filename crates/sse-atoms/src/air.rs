//! Air optics (track O, for the renderer's sky and the spectrum chart): how
//! strongly a planet's air scatters light by wavelength (Rayleigh), how
//! thick it is, and its optical depth. Gas data (refractivity and King
//! factor per gas) is pack data in `base-light`; these blocks are the
//! physics.

use sse_core::math as m;

/// Boltzmann constant, J/K.
pub const K_B: f64 = sse_core::sheet::K_B;
/// Atomic mass unit, kg.
pub const AMU: f64 = sse_core::sheet::M_U;
/// Number density at which dry-air refractivity (Peck and Reeder) is given: 288.15 K, 101325 Pa, m⁻³.
pub const N_S_288: f64 = 2.546_899e25;
/// Loschmidt's number, 273.15 K and 101325 Pa, m⁻³: p₀ / (k_B T₀) from the sheet.
pub const N_LOSCHMIDT: f64 = sse_core::sheet::ATM / (sse_core::sheet::K_B * sse_core::sheet::T0C);

/// Dry-air refractivity n − 1 at `lambda_m` (Peck and Reeder 1972), at N_S_288.
pub fn air_refractivity(lambda_m: f64) -> f64 {
    let s2 = 1.0 / (lambda_m * 1e6 * lambda_m * 1e6);
    (8060.51 + 2_480_990.0 / (132.274 - s2) + 17455.7 / (39.329_57 - s2)) * 1e-8
}

/// Rayleigh cross-section per molecule, m², from the refractivity `n1` (n − 1)
/// measured at number density `n_ref`, and the King factor.
pub fn rayleigh_cross_section(lambda_m: f64, n1: f64, n_ref: f64, king: f64) -> f64 {
    let n = 1.0 + n1;
    let a = (n * n - 1.0) / (n * n + 2.0);
    let l2 = lambda_m * lambda_m;
    24.0 * m::PI * m::PI * m::PI * a * a / (l2 * l2 * n_ref * n_ref) * king
}

/// Optical depth of a whole air column: τ = P σ / (m̄ g).
pub fn column_optical_depth(pressure_pa: f64, sigma_m2: f64, mean_mass_kg: f64, g: f64) -> f64 {
    pressure_pa * sigma_m2 / (mean_mass_kg * g)
}

/// Pressure scale height, m: H = kT / (m̄ g).
pub fn scale_height(t_k: f64, mean_mass_kg: f64, g: f64) -> f64 {
    sse_core::constants::k_b() * t_k / (mean_mass_kg * g)
}
