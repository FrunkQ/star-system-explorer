//! Bulk properties and simple interiors of planets and moons (P1 first wave).

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

pub const M_EARTH: f64 = GM_EARTH / G;
const M_JUPITER: f64 = GM_JUPITER / G;
const M_SUN: f64 = GM_SUN / G;
/// Earth's mean radius, the R⊕ of Zeng et al. 2016 (table 2 notes: 6.371e6 m).
const R_EARTH_ZENG: f64 = 6.371e6;

const CK17: &str = "Chen, J. and Kipping, D. 2017, ApJ 834, 17: probabilistic forecasting of the masses and radii of other worlds (median posterior hyper-parameters, table 1). doi:10.3847/1538-4357/834/1/17";
const ZENG16: &str = "Zeng, L., Sasselov, D. D. and Jacobsen, S. B. 2016, ApJ 819, 127: mass-radius relation for rocky planets based on PREM, eq. 2 (inverse eq. 3). doi:10.3847/0004-637X/819/2/127";
const ZJ17: &str = "Zeng, L. and Jacobsen, S. B. 2017, ApJ 837, 164: a simple analytical model for rocky planet interiors (core radius fraction about the square root of core mass fraction). doi:10.3847/1538-4357/aa6218";
const NEWTON: &str = "Newtonian gravity for a spherical body, e.g. de Pater, I. and Lissauer, J. J. 2015, Planetary Sciences, 2nd edn, Cambridge University Press, chapter 2. doi:10.1017/CBO9781316165270";

/// Chen and Kipping 2017 relation at the posterior medians (table 1): (upper mass bound in M⊕, exponent).
/// R = 1.008 R⊕ at 1 M⊕ (the paper's Terran constant); the other segments'
/// constants follow from continuity, as in the paper.
pub const CK_SEGMENTS: [(f64, f64); 4] = [
    (2.04, 0.279),
    (0.414 * M_JUPITER / M_EARTH, 0.589),
    (0.080 * M_SUN / M_EARTH, -0.044),
    (f64::INFINITY, 0.881),
];

pub fn ck_radius_earth(m_earth: f64) -> f64 {
    let mut lo = 1.0;
    let mut r = 1.008;
    for (hi, s) in CK_SEGMENTS {
        if m_earth <= hi {
            return r * m::pow(m_earth / lo, s);
        }
        r *= m::pow(hi / lo, s);
        lo = hi;
    }
    r
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let mass = port("mass", "body.mass", D::MASS, None);
    let radius = port("radius", "body.radius", D::LENGTH, None);
    vec![
        Def {
            id: "sse.planet.radius_from_mass",
            version: "1.0.0",
            inputs: vec![port("mass", "body.mass", D::MASS, Some((1.0e-4 * M_EARTH, 0.08 * M_SUN)))],
            outputs: vec![port("radius", "body.radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CK17],
            curious: "Rocky worlds grow slowly with mass, gas-wrapped ones balloon, and past about Saturn's mass extra weight squeezes a giant rather than enlarging it.",
            science: "Median of a broken power law fitted to 316 objects from dwarf planets to late stars, with breaks at 2.0 M⊕ (rock to volatile envelope), 0.41 MJ (degeneracy pressure) and 0.08 M☉ (hydrogen fusion). Real worlds scatter around it by tens of percent; it is a forecast, not a composition model.",
            formula: "R ∝ M^0.279 | M^0.589 | M^-0.044 | M^0.881, continuous, R = 1.008 R⊕ at 1 M⊕",
            f: |i, _| {
                let me = positive(i[0], "mass")? / M_EARTH;
                Ok(vec![R_EARTH * ck_radius_earth(me)])
            },
        }
        .build(),
        Def {
            id: "sse.planet.bulk_density",
            version: "1.0.0",
            inputs: vec![mass.clone(), radius.clone()],
            outputs: vec![port("density", "body.bulk_density", D::DENSITY, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[NEWTON],
            curious: "Mass shared out over volume. Rock is around 3 to 5 grams per cubic centimetre, ice about 1, and Saturn would float.",
            science: "Mean density of a sphere. The volumetric mean radius should be used for oblate bodies.",
            formula: "ρ = 3M / (4π R³)",
            f: |i, _| {
                let r = positive(i[1], "radius")?;
                Ok(vec![3.0 * positive(i[0], "mass")? / (4.0 * m::PI * r * r * r)])
            },
        }
        .build(),
        Def {
            id: "sse.planet.surface_gravity",
            version: "1.0.0",
            inputs: vec![mass.clone(), radius.clone()],
            outputs: vec![port("g", "body.surface_gravity", D::ACCELERATION, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[NEWTON],
            curious: "How hard the ground pulls you down. More mass pulls harder; a bigger radius puts you farther from it.",
            science: "Newtonian gravity at the surface of a non-rotating sphere. Rotation and oblateness lower it at the equator by up to a few percent on fast spinners.",
            formula: "g = G M / R²",
            f: |i, _| {
                let r = positive(i[1], "radius")?;
                Ok(vec![G * positive(i[0], "mass")? / (r * r)])
            },
        }
        .build(),
        Def {
            id: "sse.planet.escape_velocity",
            version: "1.0.0",
            inputs: vec![mass.clone(), radius.clone()],
            outputs: vec![port("v_esc", "body.escape_velocity", D::VELOCITY, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[NEWTON],
            curious: "How fast you must throw something for it never to fall back. It decides which gases a world can keep.",
            science: "Speed at which kinetic energy equals the gravitational binding at the surface, ignoring air drag and rotation.",
            formula: "v = √(2 G M / R)",
            f: |i, _| {
                let r = positive(i[1], "radius")?;
                Ok(vec![m::sqrt(2.0 * G * positive(i[0], "mass")? / r)])
            },
        }
        .build(),
        Def {
            id: "sse.interior.rocky_radius",
            version: "1.0.1",
            inputs: vec![
                port("mass", "body.mass", D::MASS, Some((1.0 * M_EARTH, 8.0 * M_EARTH))),
                port("core_mass_fraction", "interior.core_mass_fraction", D::NONE, Some((0.0, 0.4))),
            ],
            outputs: vec![port("radius", "body.radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[ZENG16],
            curious: "An iron-rich rocky world is smaller than a rock-rich one of the same mass, because iron packs tighter.",
            science: "Fit to two-layer (iron core, silicate mantle) models scaled from Earth's seismic profile (PREM), for rocky planets without water or atmosphere. Agrees with the model curves to about 0.01 R⊕ in its range (1 to 8 M⊕, core mass fraction 0 to 0.4); the inverse gives core mass fraction to about ±0.02.",
            formula: "R/R⊕ = (1.07 - 0.21 CMF) (M/M⊕)^(1/3.7)",
            f: |i, _| {
                let me = positive(i[0], "mass")? / M_EARTH;
                Ok(vec![R_EARTH_ZENG * (1.07 - 0.21 * i[1]) * m::pow(me, 1.0 / 3.7)])
            },
        }
        .build(),
        Def {
            id: "sse.interior.core_mass_fraction",
            version: "1.0.1",
            inputs: vec![
                port("mass", "body.mass", D::MASS, Some((1.0 * M_EARTH, 8.0 * M_EARTH))),
                port("radius", "body.radius", D::LENGTH, None),
            ],
            outputs: vec![port(
                "core_mass_fraction",
                "interior.core_mass_fraction",
                D::NONE,
                None,
            )],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[ZENG16],
            curious: "Weigh a rocky world and measure it, and you can tell roughly how much of it is an iron core.",
            science: "The Zeng et al. 2016 relation inverted. Outside 0 to 0.4 the answer means the world is not a dry iron and rock planet (too fluffy: water or gas; too dense: more iron than the fit covers).",
            formula: "CMF = (1.07 - (R/R⊕) / (M/M⊕)^(1/3.7)) / 0.21",
            f: |i, _| {
                let me = positive(i[0], "mass")? / M_EARTH;
                let re = positive(i[1], "radius")? / R_EARTH_ZENG;
                Ok(vec![(1.07 - re / m::pow(me, 1.0 / 3.7)) / 0.21])
            },
        }
        .build(),
        Def {
            id: "sse.interior.core_radius",
            version: "1.0.0",
            inputs: vec![
                radius.clone(),
                port("core_mass_fraction", "interior.core_mass_fraction", D::NONE, Some((0.0, 0.4))),
            ],
            outputs: vec![port("core_radius", "interior.core_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[ZJ17],
            curious: "How far down you would dig before reaching the iron core.",
            science: "For rocky planets the core radius fraction is close to the square root of the core mass fraction (Earth: 0.57 predicted, 0.55 measured).",
            formula: "R_core ≈ R √CMF",
            f: |i, _| {
                if i[1] < 0.0 {
                    return Err(sse_flow::AtomError("core mass fraction is negative".into()));
                }
                Ok(vec![positive(i[0], "radius")? * m::sqrt(i[1])])
            },
        }
        .build(),
    ]
}
