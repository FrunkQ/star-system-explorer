//! Light from bodies (track O2; nebulae-and-light design sections 1 and 5A):
//! the temperature a body glows at, how much starlight it sends back, and the
//! auroral radio a magnetised body gives off in a stellar wind. Keyed on
//! conditions only, never on a body's kind: a star, a brown dwarf, a giant
//! planet and a moon all go through the same blocks. The spectra themselves
//! are built from these numbers by `sse-light`.

use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;

/// Vacuum permeability, N A⁻² (CODATA 2018).
pub const MU0: f64 = sse_core::sheet::MU0;
/// Electron charge over electron mass, C/kg (CODATA 2018).
pub const E_OVER_ME: f64 = sse_core::sheet::E / sse_core::sheet::M_E;
/// Proton mass, kg (CODATA 2018).
pub const M_PROTON: f64 = sse_core::sheet::M_P;
/// Dimensions: magnetic moment (A m²), magnetic field (tesla = kg s⁻² A⁻¹), area.
pub const MOMENT: D = D([2, 0, 0, 1, 0, 0, 0]);
pub const TESLA: D = D([0, 1, -2, -1, 0, 0, 0]);
pub const AREA: D = D([2, 0, 0, 0, 0, 0, 0]);
/// The Sun's sidereal rotation rate (Carrington, 25.38 d), rad/s.
pub const OMEGA_SUN: f64 = 2.0 * m::PI / sse_core::sheet::P_ROT_SUN;

/// T⁴ = (1 − A) S / (4σ) + q / σ: absorbed light shared round the sphere plus
/// the body's own heat flowing out through its surface.
pub fn effective_temperature(insolation: f64, albedo: f64, internal_flux: f64) -> f64 {
    let t4 =
        (1.0 - albedo) * insolation / (4.0 * STEFAN_BOLTZMANN) + internal_flux / STEFAN_BOLTZMANN;
    m::sqrt(m::sqrt(t4.max(0.0)))
}

/// Magnetopause stand-off where the compressed dipole's pressure meets the
/// wind's ram pressure: (2 B_eq(r))² / 2μ₀ = ρv², B_eq(r) = μ₀M / (4π r³).
pub fn magnetopause_radius(moment: f64, density: f64, speed: f64) -> f64 {
    let ram = density * speed * speed;
    if moment <= 0.0 || ram <= 0.0 {
        return 0.0;
    }
    m::pow(
        MU0 * moment * moment / (8.0 * m::PI * m::PI * ram),
        1.0 / 6.0,
    )
}
