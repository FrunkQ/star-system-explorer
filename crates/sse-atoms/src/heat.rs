//! Heat-producing elements: the radioactive budget that keeps a world's interior
//! warm, drives volcanism and plate motion, and (for Al-26) decides whether the
//! first planetesimals keep their water. Every isotope constant is cited; the
//! star's inheritance (K-40, r-process and Al-26 relative to the Sun's birth
//! cloud) scales the budget.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const TS14: &str = "Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edn, Cambridge University Press, section 4.5 and table 4.2 (isotope heating rates). doi:10.1017/CBO9780511843877";
const FMM14: &str = "Frank, E. A., Meyer, B. S. and Mojzsis, S. J. 2014, Icarus 243, 274: a radiogenic heating evolution model for cosmochemically Earth-like exoplanets https://ui.adsabs.harvard.edu/abs/2014Icar..243..274F";
const UJP15: &str = "Unterborn, C. T., Johnson, J. A. and Panero, W. R. 2015, ApJ 806, 139: thorium abundances in solar twins and analogs, implications for the habitability of extrasolar planetary systems https://ui.adsabs.harvard.edu/abs/2015ApJ...806..139U";
const GM93: &str = "Grimm, R. E. and McSween, H. Y. 1993, Science 259, 653: heliocentric zoning of the asteroid belt by aluminum-26 heating. doi:10.1126/science.259.5095.653";
const L19: &str = "Lichtenberg, T. et al. 2019, Nature Astronomy 3, 307: a water budget dichotomy of rocky protoplanets from 26Al-heating. doi:10.1038/s41550-018-0688-5";
const R17: &str = "Ruedas, T. 2017, Geochemistry, Geophysics, Geosystems 18, 3530: radioactive heat production of six geologically important nuclides (isotope abundances, half-lives, 26Al heat 0.3583 W/kg). doi:10.1002/2017GC006997";
const KITA13: &str = "Kita, N. T. et al. 2013, Meteoritics and Planetary Science 48, 1383: 26Al-26Mg isotope systematics of the first solids in the early solar system (canonical 26Al/27Al about 5.2e-5). doi:10.1111/maps.12141";

/// Age of the Solar System (CAI condensation), Gyr. Connelly et al. 2012, Science 338, 651.
pub const SOLAR_AGE_GYR: f64 = 4.567;
const GYR: f64 = 1e9 * JULIAN_YEAR;
const MYR: f64 = 1e6 * JULIAN_YEAR;
const LN2: f64 = core::f64::consts::LN_2;

/// One heat-producing isotope: share of its element by mass, heat per kg of the
/// isotope (W/kg) and half-life (Gyr). Heat: Turcotte and Schubert 2014, table 4.2;
/// shares by mass (from atom abundances) and half-lives: Ruedas 2017, tables 1 and 2.
struct Isotope {
    share: f64,
    heat: f64,
    half_life_gyr: f64,
}

const U238: Isotope = Isotope {
    share: 0.9929,
    heat: 9.46e-5,
    half_life_gyr: 4.468,
};
const U235: Isotope = Isotope {
    share: 0.00711,
    heat: 5.69e-4,
    half_life_gyr: 0.704,
};
const TH232: Isotope = Isotope {
    share: 1.0,
    heat: 2.64e-5,
    half_life_gyr: 14.05,
};
const K40: Isotope = Isotope {
    share: 1.19e-4,
    heat: 2.92e-5,
    half_life_gyr: 1.248,
};

/// Heat per kg of silicate at `age_s` after formation, for an element budget
/// given as today's bulk silicate Earth (u, th in ppb; k in ppm) scaled by the
/// star's inheritance. Each isotope is wound back to the Sun's birth
/// (4.567 Gyr ago), scaled, then decayed forward to the body's age.
pub fn radiogenic_heating(
    age_s: f64,
    k40_rel: f64,
    r_rel: f64,
    u_ppb: f64,
    th_ppb: f64,
    k_ppm: f64,
) -> f64 {
    let part = |iso: &Isotope, conc: f64, rel: f64| {
        let lambda = LN2 / iso.half_life_gyr;
        let birth = conc * iso.share * m::exp(lambda * SOLAR_AGE_GYR) * rel;
        birth * iso.heat * m::exp(-lambda * age_s / GYR)
    };
    part(&U238, u_ppb * 1e-9, r_rel)
        + part(&U235, u_ppb * 1e-9, r_rel)
        + part(&TH232, th_ppb * 1e-9, r_rel)
        + part(&K40, k_ppm * 1e-6, k40_rel)
}

/// Radiogenic surface heat flow (W/m²) of a body at `age_s`.
pub fn radiogenic_flux(age_s: f64, rel: (f64, f64), mantle: f64, radius: f64, p: &[f64]) -> f64 {
    radiogenic_heating(age_s, rel.0, rel.1, p[0], p[1], p[2]) * mantle
        / (4.0 * m::PI * radius * radius)
}

/// Default budget params, shared by the atoms that use it.
fn budget() -> Vec<sse_flow::Param> {
    vec![
        param(
            "u_ppb",
            D::NONE,
            20.3,
            (0.0, 1000.0),
            "uranium in the silicate mantle today, parts per billion by mass (Earth: 20.3)",
        ),
        param(
            "th_ppb",
            D::NONE,
            79.5,
            (0.0, 4000.0),
            "thorium in the silicate mantle today, parts per billion by mass (Earth: 79.5)",
        ),
        param(
            "k_ppm",
            D::NONE,
            240.0,
            (0.0, 10000.0),
            "potassium in the silicate mantle today, parts per million by mass (Earth: 240)",
        ),
    ]
}

/// Al-26 heating per kg of chondritic rock at CAI time zero, W/kg:
/// Al 0.865 wt% (CI chondrite, McDonough and Sun 1995) × canonical 26Al/27Al
/// 5.25e-5 (Kita et al. 2013) × 0.3583 W per kg of Al-26 (Ruedas 2017, table 2).
pub const AL26_H0: f64 = 0.00865 * 5.25e-5 * 0.3583;
/// Al-26 mean life, Myr: the sheet's half-life over ln 2 (0.717 Myr half-life).
pub const AL26_TAU_MYR: f64 = sse_core::sheet::isotope_half_life_s::AL26 / LN2 / MYR;

fn rel(x: f64, what: &str) -> Result<f64, AtomError> {
    if x >= 0.0 && x.is_finite() {
        Ok(x)
    } else {
        Err(AtomError(format!("{what} must be zero or more, got {x}")))
    }
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let _age = port("age", "body.age", D::TIME, Some((0.0, 100.0 * GYR)));
    let k40 = port(
        "k40_rel",
        "inheritance.k40_rel",
        D::NONE,
        Some((0.0, 100.0)),
    );
    let rp = port(
        "r_process_rel",
        "inheritance.r_process_rel",
        D::NONE,
        Some((0.0, 100.0)),
    );
    let mantle = port("mantle_mass", "interior.mantle_mass", D::MASS, None);
    let radius = port("radius", "body.radius", D::LENGTH, None);
    vec![
        Def {
            id: "sse.heat.active_geology_lifetime",
            version: "1.0.1",
            inputs: vec![k40, rp, mantle, radius],
            outputs: vec![port("lifetime", "interior.active_geology_lifetime", D::TIME, None)],
            params: {
                let mut p = budget();
                p.push(param("q_crit", D::FLUX, 0.019, (0.001, 1.0), "radiogenic heat flow below which volcanism and plate motion stop, W/m² (default: half of Earth's today)"));
                p
            },
            cost: CostClass::Light,
            references: &[FMM14, UJP15, TS14, R17],
            curious: "How long a world stays geologically alive: volcanoes, mountain building, a recycled crust, and the carbon cycle that keeps a climate steady. Mars went quiet early because it is small; a planet with extra thorium can stay lively billions of years longer than Earth.",
            science: "The age at which radiogenic surface heat flow falls to a threshold, found by bisection on 0 to 100 Gyr. The default threshold, half of Earth's present radiogenic flux, is a calibration, not a law: it puts the end of Mars's widespread volcanism about a billion years ago and the Moon's mare volcanism in its first two billion years, and keeps Earth active for billions of years more. Primordial heat, tides and lid style are left out.",
            formula: "find t with q(t) = q_crit,  q(t) = H(t)·M_mantle / (4π R²)",
            f: |i, p| {
                let r = (rel(i[0], "k40_rel")?, rel(i[1], "r_process_rel")?);
                let mantle = positive(i[2], "mantle_mass")?;
                let radius = positive(i[3], "radius")?;
                let q = |t: f64| radiogenic_flux(t, r, mantle, radius, p);
                let crit = p[3];
                let (mut lo, mut hi) = (0.0, 100.0 * GYR);
                if q(lo) < crit {
                    return Ok(vec![0.0]);
                }
                if q(hi) >= crit {
                    return Ok(vec![hi]);
                }
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    if q(mid) >= crit {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                Ok(vec![0.5 * (lo + hi)])
            },
        }
        .build(),
        Def {
            id: "sse.heat.al26_planetesimal_heating",
            version: "1.0.1",
            inputs: vec![
                port("al26_rel", "inheritance.al26_rel", D::NONE, Some((0.0, 100.0))),
                port("formation_time", "planetesimal.formation_time", D::TIME, Some((0.0, 100.0 * MYR))),
            ],
            outputs: vec![
                port("energy", "planetesimal.al26_energy", dims::SPECIFIC_ENERGY, None),
                port("water_kept", "planetesimal.water_kept_fraction", D::NONE, Some((0.0, 1.0))),
            ],
            params: vec![
                param("e_dry_start", dims::SPECIFIC_ENERGY, 6.0e5, (1e4, 1e7), "Al-26 energy per kg at which hydrated minerals start losing water, J/kg"),
                param("e_dry", dims::SPECIFIC_ENERGY, 1.8e6, (1e4, 1e8), "Al-26 energy per kg at which the rock melts and outgasses, J/kg"),
                param("dry_residual", D::NONE, 0.1, (0.0, 1.0), "share of water left in a fully melted planetesimal"),
            ],
            cost: CostClass::Trivial,
            references: &[GM93, L19, R17, KITA13],
            curious: "In the first million years, short-lived aluminium-26 heated young asteroids from inside like a slow oven. Bodies that formed early melted and boiled off their water; those that formed a few million years later stayed cold and wet. How much Al-26 a star was born with may decide whether its rocky planets are dry or drowned.",
            science: "Adiabatic upper bound for a body larger than a few tens of kilometres: all the Al-26 energy left after its formation time stays in the rock. Starting heating is Al 0.865 wt% (CI chondrite) times canonical 26Al/27Al = 5.25e-5 times 0.358 W per kg of Al-26, about 1.6e-7 W/kg, scaled by the star's inheritance. Water loss is log-linear in energy between dehydration and melting.",
            formula: "E = H₀ · rel · τ · e^{-t/τ},  τ = 1.034 Myr;  kept = 1 → residual, log-linear from e_dry_start to e_dry",
            f: |i, p| {
                let r = rel(i[0], "al26_rel")?;
                let tau = AL26_TAU_MYR * MYR;
                let e = AL26_H0 * r * tau * m::exp(-i[1].max(0.0) / tau);
                let (lo, hi, resid) = (p[0], p[1], p[2]);
                if hi <= lo {
                    return Err(AtomError("e_dry must exceed e_dry_start".into()));
                }
                let kept = if e <= lo {
                    1.0
                } else if e >= hi {
                    resid
                } else {
                    let s = m::ln(e / lo) / m::ln(hi / lo);
                    1.0 - s * (1.0 - resid)
                };
                Ok(vec![e, kept])
            },
        }
        .build(),
    ]
}
