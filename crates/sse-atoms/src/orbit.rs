//! Two-body orbits (P1 first wave).

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const MD: &str =
    "Murray, C. D. and Dermott, S. F. 1999, Solar System Dynamics, Cambridge University Press. doi:10.1017/CBO9781139174817";

/// Fixed Newton iteration count for Kepler's equation: the deterministic
/// stopping rule. 30 steps converge to machine precision for e < 0.99 from the
/// starting guesses used below (Danby 1988, Fundamentals of Celestial Mechanics, section 6.6).
pub const KEPLER_ITERATIONS: u32 = 30;

/// Solve M = E - e sin E for E, with M reduced to (-π, π].
pub fn eccentric_anomaly(mean: f64, e: f64) -> f64 {
    let mut mm = m::fmod(mean, m::TAU);
    if mm > m::PI {
        mm -= m::TAU;
    } else if mm <= -m::PI {
        mm += m::TAU;
    }
    let mut big_e = if e < 0.8 {
        mm
    } else if mm >= 0.0 {
        m::PI
    } else {
        -m::PI
    };
    for _ in 0..KEPLER_ITERATIONS {
        let f = big_e - e * m::sin(big_e) - mm;
        let fp = 1.0 - e * m::cos(big_e);
        big_e -= f / fp;
    }
    big_e
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let ecc = port(
        "eccentricity",
        "orbit.eccentricity",
        D::NONE,
        Some((0.0, 0.99)),
    );
    vec![
        Def {
            id: "sse.orbit.kepler_position",
            version: "1.0.0",
            inputs: vec![
                port("mean_anomaly", "orbit.mean_anomaly", D::NONE, None),
                ecc,
            ],
            outputs: vec![
                port("eccentric_anomaly", "orbit.eccentric_anomaly", D::NONE, None),
                port("true_anomaly", "orbit.true_anomaly", D::NONE, None),
                port("radius_fraction", "orbit.radius_fraction", D::NONE, None),
            ],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[MD, "Danby, J. M. A. 1988, Fundamentals of Celestial Mechanics, 2nd edn, Willmann-Bell, section 6.6"],
            curious: "A world speeds up near its star and slows down far away. This works out where along its orbit it is after a given share of its year.",
            science: "Kepler's equation M = E - e sin E solved by Newton's method with a fixed 30 steps (start E = M, or ±π for e ≥ 0.8), then the true anomaly and r/a. Angles in radians.",
            formula: "M = E - e sin E; tan(ν/2) = √((1+e)/(1-e)) tan(E/2); r/a = 1 - e cos E",
            f: |i, _| {
                let e = i[1];
                if !(0.0..1.0).contains(&e) {
                    return Err(AtomError(format!("eccentricity {e} is not an ellipse")));
                }
                let big_e = eccentric_anomaly(i[0], e);
                let nu = 2.0
                    * m::atan2(
                        m::sqrt(1.0 + e) * m::sin(big_e / 2.0),
                        m::sqrt(1.0 - e) * m::cos(big_e / 2.0),
                    );
                Ok(vec![big_e, nu, 1.0 - e * m::cos(big_e)])
            },
        }
        .build(),
    ]
}
