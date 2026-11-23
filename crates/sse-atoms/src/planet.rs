//! Bulk properties and simple interiors of planets and moons (P1 first wave).

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

pub const M_EARTH: f64 = GM_EARTH / G;
const M_JUPITER: f64 = GM_JUPITER / G;
const M_SUN: f64 = GM_SUN / G;

const CK17: &str = "Chen, J. and Kipping, D. 2017, ApJ 834, 17: probabilistic forecasting of the masses and radii of other worlds (median posterior hyper-parameters, table 1). doi:10.3847/1538-4357/834/1/17";

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
    let _radius = port("radius", "body.radius", D::LENGTH, None);
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
    ]
}
