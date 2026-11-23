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
    let a = |valid| port("semi_major_axis", "orbit.semi_major_axis", D::LENGTH, valid);
    let ecc = port(
        "eccentricity",
        "orbit.eccentricity",
        D::NONE,
        Some((0.0, 0.99)),
    );
    vec![
        Def {
            id: "sse.orbit.semi_major_axis_from_period",
            version: "1.0.0",
            inputs: vec![
                port("period", "orbit.period", D::TIME, None),
                port("total_mass", "orbit.total_mass", D::MASS, None),
            ],
            outputs: vec![a(None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[MD],
            curious: "Knowing how long a year lasts and how heavy the star is tells you how far away the world must be.",
            science: "Kepler's third law solved for the semi-major axis. Used when a catalogue gives a period but not a distance.",
            formula: "a = (G (M + m) P² / 4π²)^⅓",
            f: |i, _| {
                let p = positive(i[0], "period")?;
                let mt = positive(i[1], "total mass")?;
                Ok(vec![m::cbrt(sse_core::constants::g() * mt * p * p / (4.0 * m::PI * m::PI))])
            },
        }
        .build(),
        Def {
            id: "sse.orbit.apsides",
            version: "1.0.0",
            inputs: vec![a(None), ecc.clone()],
            outputs: vec![
                port("periapsis", "orbit.periapsis", D::LENGTH, None),
                port("apoapsis", "orbit.apoapsis", D::LENGTH, None),
            ],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[MD],
            curious: "An oval orbit has a closest and a farthest point; the more stretched it is, the bigger the difference.",
            science: "Geometry of the ellipse: periapsis a(1 - e), apoapsis a(1 + e).",
            formula: "q = a (1 - e), Q = a (1 + e)",
            f: |i, _| {
                if !(0.0..1.0).contains(&i[1]) {
                    return Err(AtomError(format!("eccentricity {} is not an ellipse", i[1])));
                }
                Ok(vec![i[0] * (1.0 - i[1]), i[0] * (1.0 + i[1])])
            },
        }
        .build(),
        Def {
            id: "sse.orbit.kepler_position",
            version: "1.0.0",
            inputs: vec![
                port("mean_anomaly", "orbit.mean_anomaly", D::NONE, None),
                ecc.clone(),
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
        Def {
            id: "sse.orbit.hill_radius",
            version: "1.0.0",
            inputs: vec![
                a(None),
                ecc.clone(),
                port("body_mass", "body.mass", D::MASS, None),
                port("host_mass", "orbit.host_mass", D::MASS, None),
            ],
            outputs: vec![port("hill_radius", "body.hill_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[MD, "Hamilton, D. P. and Burns, J. A. 1992, Icarus 96, 43 (periapsis form) https://ui.adsabs.harvard.edu/abs/1992Icar...96...43H"],
            curious: "Around every world is a bubble where its own gravity beats its star's. Moons have to live well inside it.",
            science: "Hill sphere radius at periapsis, for a body much lighter than its host. Prograde moons are long-term stable out to roughly a third to a half of it.",
            formula: "r_H = a (1 - e) (m / 3M)^⅓",
            f: |i, _| {
                let a = positive(i[0], "semi-major axis")?;
                let mb = positive(i[2], "body mass")?;
                let mh = positive(i[3], "host mass")?;
                Ok(vec![a * (1.0 - i[1]) * m::cbrt(mb / (3.0 * mh))])
            },
        }
        .build(),
        Def {
            id: "sse.orbit.roche_limit",
            version: "1.0.0",
            inputs: vec![
                port("primary_radius", "body.radius", D::LENGTH, None),
                port("primary_density", "body.bulk_density", D::DENSITY, None),
                port("satellite_density", "satellite.bulk_density", D::DENSITY, None),
            ],
            outputs: vec![port("roche_limit", "body.roche_limit", D::LENGTH, None)],
            params: vec![param(
                "coefficient",
                D::NONE,
                2.44,
                (1.26, 2.46),
                "2.44 for a fluid satellite, 1.26 for a rigid one",
            )],
            cost: CostClass::Trivial,
            references: &[MD, "Chandrasekhar, S. 1969, Ellipsoidal Figures of Equilibrium, Yale University Press (fluid coefficient)"],
            curious: "Get too close to a big world and its tides pull a moon apart. That is where rings live.",
            science: "Roche limit for a satellite held together only by its own gravity. A fluid body breaks up at about 2.44 primary radii for equal densities, a rigid one at about 1.26.",
            formula: "d = k R (ρ_M / ρ_m)^⅓",
            f: |i, p| {
                let r = positive(i[0], "primary radius")?;
                let rp = positive(i[1], "primary density")?;
                let rs = positive(i[2], "satellite density")?;
                Ok(vec![p[0] * r * m::cbrt(rp / rs)])
            },
        }
        .build(),
    ]
}
