//! B1d: magnetic fields by condition. One dynamo law for any body with a
//! convecting, electrically conducting region: Christensen, Holzwarth and
//! Reiners (2009) showed that the field strength of Earth, Jupiter and
//! rapidly rotating low-mass stars follows from the convected energy flux and
//! the density of the dynamo region alone, not from rotation or
//! conductivity. So a planet and a star ask the same question here, as the
//! one-body plan wants.
//!
//! The chain: dynamo-region density, then the field inside the dynamo, then
//! the dipole at the body's surface and its moment (the fact the light
//! thread reads for aurorae, radio emission and magnetospheres).
//!
//! MF1: what drives the dynamo is the convected flux, not all the heat.
//! `sse.dynamo.convected_flux` takes off what conduction alone carries
//! along the adiabat; `sse.dynamo.compositional_power` adds the buoyancy of
//! light elements let out by a freezing inner core (or by magnesium oxide
//! coming out of solution), as an equivalent heat flux.

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const CHR09: &str = "Christensen, U. R., Holzwarth, V. and Reiners, A. 2009, Nature 457, 167: energy flux determines magnetic field strength of planets and stars. doi:10.1038/nature07626";
const C10: &str = "Christensen, U. R. 2010, Space Science Reviews 152, 565: dynamo scaling laws and applications to the planets (c about 0.63, f_ohm about 0.9). doi:10.1007/s11214-009-9553-2";

/// Vacuum permeability, N A⁻² (CODATA 2018).
pub const MU0: f64 = sse_core::sheet::MU0;

/// Field inside a dynamo, T, from Christensen (2010) eq. 12:
/// B²/(2μ₀) = c · f_ohm · ρ^(1/3) · (q)^(2/3), with q the convected heat
/// flux through the dynamo's outer surface.
pub fn dynamo_field_rms(density: f64, flux: f64, c: f64, f_ohm: f64) -> f64 {
    if density <= 0.0 || flux <= 0.0 {
        return 0.0;
    }
    let e = c * f_ohm * m::cbrt(density) * m::pow(flux, 2.0 / 3.0);
    m::sqrt(2.0 * MU0 * e)
}

/// The equatorial dipole field at `r_surface` from a dynamo of rms field
/// `b_rms` and outer radius `r_dynamo`, and the dipole moment, A m².
pub fn surface_dipole(b_rms: f64, r_dynamo: f64, r_surface: f64, dipole_ratio: f64) -> (f64, f64) {
    let x = (r_dynamo / r_surface).min(1.0);
    let b_eq = dipole_ratio * b_rms * x * x * x;
    let moment = 4.0 * m::PI / MU0 * b_eq * r_surface * r_surface * r_surface;
    (b_eq, moment)
}

/// Heat conducted along a core's adiabat at its top, W/m²: q = k α g T / c_p.
pub fn adiabatic_flux(k: f64, alpha: f64, g: f64, t: f64, cp: f64) -> f64 {
    k * alpha * g * t / cp.max(1e-9)
}

/// The convected flux: what crosses the top beyond conduction alone. Negative
/// means the top is stably layered (no thermal convection there).
pub fn convected_flux(q: f64, k: f64, alpha: f64, g: f64, t: f64, cp: f64) -> f64 {
    q - adiabatic_flux(k, alpha, g, t, cp)
}

/// Light elements' buoyancy as an equivalent heat flux, W/m²: a mass flux
/// `mass_flux` (kg m⁻² s⁻¹) with fractional density contrast `drho` gives
/// the buoyancy of q = (c_p / α) · drho · ṁ/A.
pub fn compositional_flux(mass_flux: f64, drho: f64, alpha: f64, cp: f64) -> f64 {
    if mass_flux <= 0.0 {
        return 0.0;
    }
    cp / alpha.max(1e-12) * drho * mass_flux
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let b = dims::MAGNETIC_FIELD;
    let density = port("density", "interior.dynamo_density", D::DENSITY, None);
    let flux = port("flux", "interior.dynamo_heat_flux", D::FLUX, None);
    let rms = port("field", "magnetic.dynamo_field", b, None);
    vec![
        Def {
            id: "sse.magnetic.dynamo_field",
            version: "1.0.0",
            inputs: vec![density, flux],
            outputs: vec![rms.clone()],
            params: vec![
                param("c", D::NONE, 0.63, (0.1, 3.0), "Christensen's constant, fitted to numerical dynamos (2010)"),
                param("f_ohm", D::NONE, 0.9, (0.0, 1.0), "share of the convected power lost to ohmic heating (about 0.9 for planets, 1 for stars)"),
            ],
            cost: CostClass::Trivial,
            references: &[CHR09, C10],
            curious: "A single rule sets the field strength of Earth, Jupiter and small, fast-spinning stars: how much heat the churning region carries out, and how dense it is. How fast the body spins decides only whether the field is a tidy dipole.",
            science: "In convection-driven dynamos the magnetic energy density is set by the power available to the flow, not by a balance of forces: B²/2μ₀ = c f_ohm ρ^(1/3) q^(2/3). The same fit runs from planets to stars across many orders of magnitude (Christensen, Holzwarth and Reiners 2009). It needs a convecting, conducting region; with no convected heat there is no dynamo and no field.",
            formula: "B_rms = √(2μ₀ · c · f_ohm · ρ^(1/3) · q^(2/3))",
            f: |i, p| Ok(vec![dynamo_field_rms(i[0], i[1], p[0], p[1])]),
        }
        .build(),
    ]
}
