//! `heat` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const MS95: &str = "McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: the composition of the Earth (bulk silicate Earth U 20.3 ppb, Th 79.5 ppb, K 240 ppm). doi:10.1016/0009-2541(94)00140-4";
const TS14: &str = "Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edn, Cambridge University Press, section 4.5 and table 4.2 (isotope heating rates). doi:10.1017/CBO9780511843877";
const FMM14: &str = "Frank, E. A., Meyer, B. S. and Mojzsis, S. J. 2014, Icarus 243, 274: a radiogenic heating evolution model for cosmochemically Earth-like exoplanets https://ui.adsabs.harvard.edu/abs/2014Icar..243..274F";
const UJP15: &str = "Unterborn, C. T., Johnson, J. A. and Panero, W. R. 2015, ApJ 806, 139: thorium abundances in solar twins and analogs, implications for the habitability of extrasolar planetary systems https://ui.adsabs.harvard.edu/abs/2015ApJ...806..139U";
const KAMLAND11: &str = "KamLAND Collaboration (Gando, A. et al.) 2011, Nature Geoscience 4, 647: partial radiogenic heat model for Earth revealed by geoneutrino measurements. doi:10.1038/ngeo1205";
const GM93: &str = "Grimm, R. E. and McSween, H. Y. 1993, Science 259, 653: heliocentric zoning of the asteroid belt by aluminum-26 heating. doi:10.1126/science.259.5095.653";
const L19: &str = "Lichtenberg, T. et al. 2019, Nature Astronomy 3, 307: a water budget dichotomy of rocky protoplanets from 26Al-heating. doi:10.1038/s41550-018-0688-5";
const R17: &str = "Ruedas, T. 2017, Geochemistry, Geophysics, Geosystems 18, 3530: radioactive heat production of six geologically important nuclides (isotope abundances, half-lives, 26Al heat 0.3583 W/kg). doi:10.1002/2017GC006997";
const KITA13: &str = "Kita, N. T. et al. 2013, Meteoritics and Planetary Science 48, 1383: 26Al-26Mg isotope systematics of the first solids in the early solar system (canonical 26Al/27Al about 5.2e-5). doi:10.1111/maps.12141";
const NEWTON: &str = "Spherical geometry: heat flow is power over surface area, e.g. Turcotte and Schubert 2014, chapter 4. doi:10.1017/CBO9780511843877";

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
/// Al-26 mean life, Myr (half-life 0.717 Myr).
pub const AL26_TAU_MYR: f64 = 0.717 / LN2;

fn rel(x: f64, what: &str) -> Result<f64, AtomError> {
    if x >= 0.0 && x.is_finite() {
        Ok(x)
    } else {
        Err(AtomError(format!("{what} must be zero or more, got {x}")))
    }
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let age = port("age", "body.age", D::TIME, Some((0.0, 100.0 * GYR)));
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
            id: "sse.heat.radiogenic_heating",
            version: "1.0.1",
            inputs: vec![age.clone(), k40.clone(), rp.clone()],
            outputs: vec![port("heating", "interior.radiogenic_heating", dims::SPECIFIC_POWER, None)],
            params: budget(),
            cost: CostClass::Trivial,
            references: &[MS95, TS14, R17, FMM14, UJP15],
            curious: "Rock is very slightly radioactive. Uranium, thorium and potassium-40 warm every kilogram of a planet's mantle by a few trillionths of a watt, and over a whole world that is enough to keep volcanoes and drifting continents going for billions of years. Young worlds run hotter, and a star born with more thorium gives its planets a longer-lived fire.",
            science: "Four long-lived isotopes. Today's bulk silicate Earth budget is wound back to the Sun's birth, scaled by the star's inheritance (K-40 for potassium; r-process for uranium and thorium, which are made in neutron-star mergers and some supernovae), then decayed to the body's age. U-235 and K-40 dominate in the first billion years, thorium in the far future. This is the radiogenic part only: Earth also loses about as much again as primordial heat.",
            formula: "H(t) = Σᵢ cᵢ,now · e^{λᵢ·4.567 Gyr} · relᵢ · hᵢ · e^{-λᵢ t},  λᵢ = ln2 / t½ᵢ",
            f: |i, p| {
                let k = rel(i[1], "k40_rel")?;
                let r = rel(i[2], "r_process_rel")?;
                Ok(vec![radiogenic_heating(i[0].max(0.0), k, r, p[0], p[1], p[2])])
            },
        }
        .build(),
        Def {
            id: "sse.heat.mantle_mass",
            version: "1.0.0",
            inputs: vec![
                port("mass", "body.mass", D::MASS, None),
                port("core_mass_fraction", "interior.core_mass_fraction", D::NONE, Some((0.0, 1.0))),
            ],
            outputs: vec![mantle.clone()],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[MS95],
            curious: "The radioactive elements shun iron, so almost all of them sit in the rocky mantle and crust, not the core.",
            science: "Uranium, thorium and potassium are lithophile: they partition into silicate, so the heat-producing mass is the silicate shell (mantle plus crust).",
            formula: "M_mantle = M · (1 − f_core)",
            f: |i, _| Ok(vec![positive(i[0], "mass")? * (1.0 - i[1])]),
        }
        .build(),
        Def {
            id: "sse.heat.radiogenic_power",
            version: "1.0.1",
            inputs: vec![
                port("heating", "interior.radiogenic_heating", dims::SPECIFIC_POWER, None),
                mantle.clone(),
            ],
            outputs: vec![port("power", "interior.radiogenic_power", D::POWER, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[KAMLAND11],
            curious: "Earth's rocks make about 20 terawatts of heat, roughly the whole of humanity's power use. Geoneutrinos, ghost particles from those decays, let physicists weigh it directly.",
            science: "Heating per kg times the silicate mass. KamLAND and Borexino geoneutrino counts put Earth's U and Th power at 20 (+9/−9) TW, and K-40 adds about 4 TW; this budget's U and Th part (about 16.5 TW today) is inside that range.",
            formula: "P = H · M_mantle",
            f: |i, _| Ok(vec![i[0] * i[1]]),
        }
        .build(),
        Def {
            id: "sse.heat.surface_heat_flow",
            version: "1.0.0",
            inputs: vec![port("power", "interior.radiogenic_power", D::POWER, None), radius.clone()],
            outputs: vec![port("flow", "interior.radiogenic_heat_flow", D::FLUX, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[NEWTON],
            curious: "Spread over the surface, Earth's radioactive heat is only a few hundredths of a watt per square metre, ten thousand times less than sunlight. Small worlds have more surface for their volume, so they cool and die first.",
            science: "Power over area. A smaller body has a larger area to volume ratio, so the same rock gives a lower flux; this is why the Moon and Mars froze geologically before Earth.",
            formula: "q = P / (4π R²)",
            f: |i, _| {
                let r = positive(i[1], "radius")?;
                Ok(vec![i[0] / (4.0 * m::PI * r * r)])
            },
        }
        .build(),
    ]
}
