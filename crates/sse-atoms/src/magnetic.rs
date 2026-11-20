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

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const CHR09: &str = "Christensen, U. R., Holzwarth, V. and Reiners, A. 2009, Nature 457, 167: energy flux determines magnetic field strength of planets and stars. doi:10.1038/nature07626";
const C10: &str = "Christensen, U. R. 2010, Space Science Reviews 152, 565: dynamo scaling laws and applications to the planets (c about 0.63, f_ohm about 0.9). doi:10.1007/s11214-009-9553-2";
const CA06: &str = "Christensen, U. R. and Aubert, J. 2006, Geophysical Journal International 166, 97: scaling properties of convection-driven dynamos in rotating spherical shells (dipolar dynamos, the ratio of mean field to dipole field at the dynamo surface). doi:10.1111/j.1365-246X.2006.02972.x";
const IGRF: &str = "Alken, P. et al. 2021, Earth, Planets and Space 73, 49: the International Geomagnetic Reference Field, 13th generation (Earth's dipole moment about 7.7e22 A m²). doi:10.1186/s40623-020-01288-x";
const JRM09: &str = "Connerney, J. E. P. et al. 2018, Geophysical Research Letters 45, 2590: a new model of Jupiter's magnetic field from Juno's first nine orbits (dipole about 4.1 G at the equator). doi:10.1002/2018GL077312";
const SPHERE: &str = "A uniform sphere: density is mass over volume, ρ = M / (4/3 π R³).";
const DIPOLE: &str = "A dipole field falls as r⁻³; the equatorial field B at radius r of a moment μ is B = μ₀ μ / (4π r³). Jackson, J. D. 1999, Classical Electrodynamics, 3rd edn, Wiley, section 5.6.";

/// Vacuum permeability, N A⁻² (CODATA 2018).
pub const MU0: f64 = 1.256_637_062_12e-6;

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

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let b = dims::MAGNETIC_FIELD;
    let density = port("density", "interior.dynamo_density", D::DENSITY, None);
    let flux = port("flux", "interior.dynamo_heat_flux", D::FLUX, None);
    let rms = port("field", "magnetic.dynamo_field", b, None);
    let r_dyn = port("dynamo_radius", "interior.dynamo_radius", D::LENGTH, None);
    vec![
        Def {
            id: "sse.magnetic.dynamo_density",
            version: "1.0.0",
            inputs: vec![
                port("mass", "body.mass", D::MASS, None),
                port("core_mass_fraction", "interior.core_mass_fraction", D::NONE, Some((0.0, 1.0))),
                port("core_radius", "interior.core_radius", D::LENGTH, None),
            ],
            outputs: vec![density.clone()],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[SPHERE, C10],
            curious: "Earth's iron core is about twice as dense as the rock above it, and that dense, churning metal is where the compass field is made.",
            science: "For a rocky world the dynamo runs in the liquid iron core, so its density is the core's mass over its volume. For a giant the dynamo region is the metallic hydrogen inside about 85% of the radius, and its density can be given directly.",
            formula: "ρ = M f_core / (4/3 π R_core³)",
            f: |i, _| {
                let r = positive(i[2], "core_radius")?;
                Ok(vec![positive(i[0], "mass")? * i[1] / (4.0 / 3.0 * m::PI * r * r * r)])
            },
        }
        .build(),
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
        Def {
            id: "sse.magnetic.surface_dipole",
            version: "1.0.0",
            inputs: vec![
                rms,
                r_dyn,
                port("radius", "body.radius", D::LENGTH, None),
            ],
            outputs: vec![
                port("surface_field", "body.surface_magnetic_field", b, None),
                port("moment", "body.magnetic_moment", dims::MAGNETIC_MOMENT, None),
            ],
            params: vec![param(
                "dipole_ratio",
                D::NONE,
                0.085,
                (0.0, 1.0),
                "dipole field at the dynamo surface over the rms field inside (Earth and Jupiter, docs/fudges.md)",
            )],
            cost: CostClass::Trivial,
            references: &[CA06, DIPOLE, IGRF, JRM09],
            curious: "Only a small part of the field made in the core reaches the surface as the familiar north-south dipole, and it fades fast with distance: Earth's field at the surface is about seven times weaker than at the top of the core.",
            science: "In dipolar dynamos the dipole at the dynamo's outer surface is a roughly fixed share of the field inside (Christensen and Aubert 2006); outside it falls as (R_dynamo/r)³. The moment follows from the equatorial surface field. The share is set here so that Earth and Jupiter come out within about 15% of their measured moments.",
            formula: "B_eq = k · B_rms · (R_dyn / R)³,  μ = 4π B_eq R³ / μ₀",
            f: |i, p| {
                let (b, mu) = surface_dipole(i[0], positive(i[1], "dynamo_radius")?, positive(i[2], "radius")?, p[0]);
                Ok(vec![b, mu])
            },
        }
        .build(),
    ]
}
