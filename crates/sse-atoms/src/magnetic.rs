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
const CA06: &str = "Christensen, U. R. and Aubert, J. 2006, Geophysical Journal International 166, 97: scaling properties of convection-driven dynamos in rotating spherical shells and application to planetary magnetic fields (dipolar dynamos, the ratio of mean field to dipole field at the dynamo surface). doi:10.1111/j.1365-246X.2006.03009.x";
const IGRF: &str = "Alken, P. et al. 2021, Earth, Planets and Space 73, 49: the International Geomagnetic Reference Field, 13th generation (Earth's dipole moment about 7.7e22 A m²). doi:10.1186/s40623-020-01288-x";
const JRM09: &str = "Connerney, J. E. P. et al. 2018, Geophysical Research Letters 45, 2590: a new model of Jupiter's magnetic field from Juno's first nine orbits (dipole about 4.1 G at the equator). doi:10.1002/2018GL077312";
const LAB15: &str = "Labrosse, S. 2015, Physics of the Earth and Planetary Interiors 247, 36: thermal evolution of the core with a high thermal conductivity (the adiabatic heat flow q = k α g T / c_p). doi:10.1016/j.pepi.2015.02.002";
const DB14: &str = "Driscoll, P. and Bercovici, D. 2014, Physics of the Earth and Planetary Interiors 236, 36: on the thermal and magnetic histories of Earth and Venus (dynamo power from the CMB flux above the adiabat plus inner core buoyancy). doi:10.1016/j.pepi.2014.08.004";
const BR95: &str = "Buffett, B. A., Huppert, H. E., Lister, J. R. and Woods, A. W. 1996, Journal of Geophysical Research 101, 7989: on the thermal evolution of the Earth's core (light elements released at the inner core boundary as a buoyancy source). doi:10.1029/95JB03539";
const OS16: &str = "O'Rourke, J. G. and Stevenson, D. J. 2016, Nature 529, 387: powering Earth's dynamo with magnesium precipitation from the core. doi:10.1038/nature16495";
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
    let r_dyn = port("dynamo_radius", "interior.dynamo_radius", D::LENGTH, None);
    let conv = port("convected_flux", "interior.convected_flux", D::FLUX, None);
    vec![
        Def {
            id: "sse.dynamo.convected_flux",
            version: "1.0.0",
            inputs: vec![
                port("flux", "interior.core_heat_flux", D::FLUX, None),
                port("temperature", "interior.core_top_temperature", D::TEMPERATURE, None),
                port("gravity", "interior.core_top_gravity", D::ACCELERATION, None),
            ],
            outputs: vec![conv.clone()],
            params: vec![
                param("k", dims::CONDUCTIVITY, 130.0, (10.0, 300.0), "thermal conductivity of the liquid alloy (Pozzo et al. 2012 give about 150; Konopkova et al. 2016 about 30; until MP21's iron rows)"),
                param("alpha", dims::PER_KELVIN, 1.25e-5, (1e-6, 1e-4), "thermal expansivity of the liquid alloy"),
                param("cp", dims::SPECIFIC_HEAT, 840.0, (300.0, 2000.0), "specific heat of the liquid alloy"),
            ],
            cost: CostClass::Trivial,
            references: &[LAB15, DB14],
            curious: "A hot core can lose heat and still make no field: if conduction alone can carry the heat away, the iron does not need to churn.",
            science: "Liquid iron conducts heat well, so part of what leaves the core's top is carried by conduction along the adiabat, q_ad = k α g T / c_p. Only the rest is carried by convection, and only convection stirs a dynamo. When the flux falls below q_ad the top of the core is stably layered.",
            formula: "q_conv = q − k α g T / c_p",
            f: |i, p| Ok(vec![convected_flux(i[0], p[0], p[1], i[2], i[1], p[2])]),
        }
        .build(),
        Def {
            id: "sse.dynamo.compositional_power",
            version: "1.0.0",
            inputs: vec![
                conv.clone(),
                port("light_mass_flux", "interior.light_element_flux", dims::MASS_FLUX, None),
            ],
            outputs: vec![port("dynamo_flux", "interior.dynamo_heat_flux", D::FLUX, None)],
            params: vec![
                param("drho", D::NONE, 0.05, (0.0, 1.0), "fractional density contrast of what is let out (light elements at the inner core boundary, about 5%)"),
                param("alpha", dims::PER_KELVIN, 1.25e-5, (1e-6, 1e-4), "thermal expansivity of the liquid alloy"),
                param("cp", dims::SPECIFIC_HEAT, 840.0, (300.0, 2000.0), "specific heat of the liquid alloy"),
            ],
            cost: CostClass::Trivial,
            references: &[BR95, DB14, OS16],
            curious: "Earth's field may be older than its solid inner core. Once the inner core began to freeze, the light elements it pushed out rose like bubbles and gave the dynamo a second engine.",
            science: "A freezing inner core rejects light elements into the liquid above, and magnesium oxide can come out of solution at the core's top; either way buoyant fluid stirs the core. Its power is written as the heat flux that would give the same buoyancy, (c_p/α) Δρ/ρ ṁ/A, and added to the convected flux. A negative convected flux is a stable layer that the buoyancy must first overcome.",
            formula: "q_dyn = q_conv + (c_p / α) · (Δρ/ρ) · ṁ/A",
            f: |i, p| Ok(vec![i[0] + compositional_flux(i[1], p[0], p[1], p[2])]),
        }
        .build(),
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
