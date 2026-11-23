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
/// The Sun's age, 4.57 Gyr (Bonanno, Schlattl and Paternò 2002, A&A 390, 1115,
/// doi:10.1051/0004-6361:20020749), in seconds.
pub const SUN_AGE: f64 = 4.57e9 * JULIAN_YEAR;

const EKER: &str = "Eker, Z. et al. 2018, MNRAS 479, 5491: interrelated main-sequence mass-luminosity, mass-radius and mass-effective temperature relations (six-piece MLR, table 4). doi:10.1093/mnras/sty1834";
const DK91: &str = "Demircan, O. and Kahraman, G. 1991, Ap&SS 181, 313: stellar mass-luminosity and mass-radius relations https://ui.adsabs.harvard.edu/abs/1991Ap%26SS.181..313D";
const GOUGH: &str =
    "Gough, D. O. 1981, Solar Physics 74, 21: solar interior structure and luminosity variations. doi:10.1007/BF00151270";
const ADAMS: &str = "Adams, F. C. 2008, JCAP 08, 010: stars in other universes, stellar structure with different fundamental constants (homology scalings of luminosity and radius). doi:10.1088/1475-7516/2008/08/010";
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

/// MP1c, ESTIMATED: the luminosity a fitted relation gives, rescaled for
/// constants that differ from ours by stellar homology (Adams 2008). With
/// radiative diffusion and a fixed opacity per mass, L ∝ (a c / κ)
/// (G μ m_p / k_B)^4 M^3, and a c = 4σ, so at fixed mass L ∝ σ G^4 / k_B^4.
/// It is exactly 1 for the real constants.
pub fn homology_luminosity_factor() -> f64 {
    let real = sse_core::constants::Constants::REAL;
    let now = sse_core::constants::now();
    let gr = now.g / real.g;
    let kr = now.k_b / real.k_b;
    if gr == 1.0 && kr == 1.0 && now.sigma_sb == real.sigma_sb {
        return 1.0;
    }
    (now.sigma_sb / real.sigma_sb) * (gr * gr * gr * gr) / (kr * kr * kr * kr)
}

/// MP1c, ESTIMATED: the radius rescaled by homology. Hydrogen ignites at a
/// core thermal energy set by nuclear physics (k_B T_c fixed), and the
/// virial theorem gives k_B T_c ∝ G M μ m_p / R, so at fixed mass R ∝ G.
pub fn homology_radius_factor() -> f64 {
    let real = sse_core::constants::Constants::REAL;
    let now = sse_core::constants::now();
    if now.g == real.g {
        1.0
    } else {
        now.g / real.g
    }
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
            references: &[EKER, ADAMS],
            curious: "Heavier stars squeeze their cores harder and burn far brighter: twice the Sun's mass is about twenty times as bright.",
            science: "Empirical six-piece power law fitted to 509 main-sequence components of detached eclipsing binaries with masses and radii good to 15 percent. It gives a typical main-sequence luminosity for the mass, averaged over age and solar-neighbourhood metallicity.",
            formula: "log L/L☉ = a log M/M☉ + b, (a, b) by mass range; × (σ/σ₀)(G/G₀)⁴(k_B₀/k_B)⁴ with changed constants (ESTIMATED)",
            f: |i, _| {
                let ms = positive(i[0], "mass")? / M_SUN;
                Ok(vec![L_SUN * luminosity_solar(ms) * homology_luminosity_factor()])
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
            references: &[DK91, ADAMS],
            curious: "Bigger stars are bigger, but not in proportion: ten Suns' worth of mass makes a star only about five times as wide.",
            science: "Two-piece power-law fit to detached binary components, valid for typical main-sequence ages. Stars swell as they age, so a young or old star of the same mass differs by tens of percent.",
            formula: "R/R☉ = 1.06 (M/M☉)^0.945 below 1.66 M☉, 1.33 (M/M☉)^0.555 above; × G/G₀ with changed constants (ESTIMATED)",
            f: |i, _| {
                let ms = positive(i[0], "mass")? / M_SUN;
                Ok(vec![R_SUN * radius_solar(ms) * homology_radius_factor()])
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
