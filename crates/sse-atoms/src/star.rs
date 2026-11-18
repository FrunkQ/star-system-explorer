//! Stars on the main sequence, from mass (P1 first wave).
//!
//! Metallicity is deliberately NOT an input yet: the first wave uses empirical
//! relations fitted to solar-neighbourhood stars, so a metallicity input would
//! change nothing and pretend otherwise. Metallicity and evolved stars need a
//! stellar track table (P5 issue, see handover/packages/P1.md).

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

pub const M_SUN: f64 = GM_SUN / G;
/// The Sun's age, 4.57 Gyr (Bonanno, Schlattl and Paternò 2002), in seconds.
pub const SUN_AGE: f64 = 4.57e9 * JULIAN_YEAR;

const EKER: &str = "Eker, Z. et al. 2018, MNRAS 479, 5491: interrelated main-sequence mass-luminosity, mass-radius and mass-effective temperature relations (six-piece MLR, table 4)";
const DK91: &str = "Demircan, O. and Kahraman, G. 1991, Ap&SS 181, 313: stellar mass-luminosity and mass-radius relations";
const GOUGH: &str =
    "Gough, D. O. 1981, Solar Physics 74, 21: solar interior structure and luminosity variations";
const HKT: &str = "Hansen, C. J., Kawaler, S. D. and Trimble, V. 2004, Stellar Interiors, 2nd edn, Springer, section 1.4 (main-sequence lifetime scaling)";

/// Eker et al. 2018 six-piece mass-luminosity relation, (upper mass bound in
/// M☉, slope, intercept) for log L = a log M + b.
pub const EKER_MLR: [(f64, f64, f64); 6] = [
    (0.45, 2.028, -0.976),
    (0.72, 4.572, -0.102),
    (1.05, 5.743, -0.007),
    (2.40, 4.329, 0.010),
    (7.0, 3.967, 0.093),
    (31.0, 2.865, 1.105),
];

pub fn luminosity_solar(m_solar: f64) -> f64 {
    let lm = m::log10(m_solar);
    let seg = EKER_MLR
        .iter()
        .find(|s| m_solar <= s.0)
        .unwrap_or(&EKER_MLR[5]);
    m::pow(10.0, seg.1 * lm + seg.2)
}

pub fn radius_solar(m_solar: f64) -> f64 {
    if m_solar < 1.66 {
        1.06 * m::pow(m_solar, 0.945)
    } else {
        1.33 * m::pow(m_solar, 0.555)
    }
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let mass = |valid| port("mass", "star.mass", D::MASS, valid);
    vec![
        Def {
            id: "sse.star.ms_luminosity",
            version: "1.0.0",
            inputs: vec![mass(Some((0.179 * M_SUN, 31.0 * M_SUN)))],
            outputs: vec![port("luminosity", "star.luminosity", D::POWER, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[EKER],
            curious: "Heavier stars squeeze their cores harder and burn far brighter: twice the Sun's mass is about twenty times as bright.",
            science: "Empirical six-piece power law fitted to 509 main-sequence components of detached eclipsing binaries with masses and radii good to 15 percent. It gives a typical main-sequence luminosity for the mass, averaged over age and solar-neighbourhood metallicity.",
            formula: "log L/L☉ = a log M/M☉ + b, (a, b) by mass range",
            f: |i, _| {
                let ms = positive(i[0], "mass")? / M_SUN;
                Ok(vec![L_SUN * luminosity_solar(ms)])
            },
        }
        .build(),
        Def {
            id: "sse.star.ms_radius",
            version: "1.0.0",
            inputs: vec![mass(Some((0.1 * M_SUN, 20.0 * M_SUN)))],
            outputs: vec![port("radius", "star.radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[DK91],
            curious: "Bigger stars are bigger, but not in proportion: ten Suns' worth of mass makes a star only about five times as wide.",
            science: "Two-piece power-law fit to detached binary components, valid for typical main-sequence ages. Stars swell as they age, so a young or old star of the same mass differs by tens of percent.",
            formula: "R/R☉ = 1.06 (M/M☉)^0.945 below 1.66 M☉, 1.33 (M/M☉)^0.555 above",
            f: |i, _| {
                let ms = positive(i[0], "mass")? / M_SUN;
                Ok(vec![R_SUN * radius_solar(ms)])
            },
        }
        .build(),
        Def {
            id: "sse.star.effective_temperature",
            version: "1.0.0",
            inputs: vec![
                port("luminosity", "star.luminosity", D::POWER, None),
                port("radius", "star.radius", D::LENGTH, None),
            ],
            outputs: vec![port("teff", "star.effective_temperature", D::TEMPERATURE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &["Stefan-Boltzmann law; definition of effective temperature, e.g. Carroll, B. W. and Ostlie, D. A. 2017, An Introduction to Modern Astrophysics, 2nd edn, eq. 3.17"],
            curious: "How hot a star's surface is follows from how much light leaves how much surface.",
            science: "Effective temperature is the temperature of a black body with the star's radius and luminosity. It defines the surface temperature rather than measuring it.",
            formula: "T_eff = (L / (4π R² σ))^¼",
            f: |i, _| {
                let l = positive(i[0], "luminosity")?;
                let r = positive(i[1], "radius")?;
                Ok(vec![m::pow(l / (4.0 * m::PI * r * r * STEFAN_BOLTZMANN), 0.25)])
            },
        }
        .build(),
        Def {
            id: "sse.star.ms_lifetime",
            version: "1.0.0",
            inputs: vec![
                mass(Some((0.179 * M_SUN, 31.0 * M_SUN))),
                port("luminosity", "star.luminosity", D::POWER, None),
            ],
            outputs: vec![port("lifetime", "star.main_sequence_lifetime", D::TIME, None)],
            params: vec![param(
                "sun_lifetime",
                D::TIME,
                1.0e10 * JULIAN_YEAR,
                (8.0e9 * JULIAN_YEAR, 1.2e10 * JULIAN_YEAR),
                "the Sun's main-sequence lifetime",
            )],
            cost: CostClass::Trivial,
            references: &[HKT],
            curious: "A star lives on its fuel divided by how fast it burns it. Bright heavy stars burn out in millions of years; dim red dwarfs outlast the present age of the universe.",
            science: "The usable hydrogen scales with mass and the burn rate with luminosity, so the lifetime scales as M/L, normalised to the Sun's 10 Gyr. Ignores the larger burnt fraction in fully convective dwarfs and mass loss in massive stars.",
            formula: "t_MS = t☉ (M/M☉) / (L/L☉)",
            f: |i, p| {
                let ms = positive(i[0], "mass")? / M_SUN;
                let ls = positive(i[1], "luminosity")? / L_SUN;
                Ok(vec![p[0] * ms / ls])
            },
        }
        .build(),
        Def {
            id: "sse.star.ms_luminosity_at_age",
            version: "1.0.0",
            inputs: vec![
                port("typical", "star.luminosity", D::POWER, None),
                port("age", "star.age", D::TIME, None),
                port("lifetime", "star.main_sequence_lifetime", D::TIME, None),
            ],
            outputs: vec![port("luminosity", "star.luminosity_at_age", D::POWER, None)],
            params: vec![param(
                "typical_fraction",
                D::NONE,
                0.457,
                (0.35, 0.8),
                "fraction of the main-sequence life at which the typical luminosity applies (the Sun's 4.57 of 10 Gyr)",
            )],
            cost: CostClass::Trivial,
            references: &[GOUGH],
            curious: "Stars brighten slowly as they age. The young Sun gave about 30 percent less light than today, which is the faint young Sun puzzle.",
            science: "Gough's fit to standard solar models, L(t) = L_now / (1 + 0.4 (1 - t/t_now)), applied by fraction of main-sequence life. Bounded agreement is claimed only for Sun-like stars; for other masses it is a reasonable shape, not a track. Before birth or past the end of the main sequence it refuses, since that needs the track table.",
            formula: "L(t) = L_typ / (1 + 0.4 (1 - x/x_typ)), x = t / t_MS",
            f: |i, p| {
                let l = positive(i[0], "typical luminosity")?;
                let life = positive(i[2], "lifetime")?;
                let x = i[1] / life;
                if !(0.0..=1.0).contains(&x) {
                    return Err(sse_flow::AtomError(format!(
                        "age is {x:.3} of the main-sequence lifetime; this atom covers 0 to 1 (post-main-sequence needs the track table)"
                    )));
                }
                let d = 1.0 + 0.4 * (1.0 - x / p[0]);
                Ok(vec![l / d])
            },
        }
        .build(),
    ]
}
