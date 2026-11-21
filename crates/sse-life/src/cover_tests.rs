//! E6 part 1: cover and biomes from a region's climate.
//!
//! The sites are Earth stations with approximate climate normals:
//! - temperatures are the annual mean and the coldest and warmest months, in °C;
//! - precipitation is in metres a year;
//! - potential evaporation is in metres a year, roughly Priestley-Taylor;
//! - light at the ground is in MJ m-2 yr-1 of PAR.
//!
//! The values are rounded from standard climate tables, good to about 10 to
//! 20 percent, which is all a broad-type check needs. Actual evaporation comes
//! from the shared climate's own Fu curve, so these are the same α the
//! climate would give.

use crate::cover::*;
use sse_climate::water::fu_evaporation;

struct Site {
    name: &'static str,
    t: f64,
    cold: f64,
    warm: f64,
    p: f64,
    pet: f64,
    par: f64,
    ice: bool,
    broad: &'static str,
}

const SITES: &[Site] = &[
    Site {
        name: "Manaus",
        t: 27.5,
        cold: 26.5,
        warm: 28.5,
        p: 2.3,
        pet: 1.5,
        par: 2900.0,
        ice: false,
        broad: "closed forest",
    },
    Site {
        name: "Lubumbashi (miombo)",
        t: 20.5,
        cold: 16.0,
        warm: 23.5,
        p: 1.2,
        pet: 1.5,
        par: 2900.0,
        ice: false,
        broad: "open woodland and savannah",
    },
    Site {
        name: "In Salah (Sahara)",
        t: 25.0,
        cold: 13.0,
        warm: 35.0,
        p: 0.015,
        pet: 2.5,
        par: 3000.0,
        ice: false,
        broad: "scrub and desert",
    },
    Site {
        name: "Astana (steppe)",
        t: 3.0,
        cold: -15.0,
        warm: 20.0,
        p: 0.32,
        pet: 0.8,
        par: 2100.0,
        ice: false,
        broad: "grassland",
    },
    Site {
        name: "Frankfurt",
        t: 10.5,
        cold: 1.5,
        warm: 19.5,
        p: 0.65,
        pet: 0.65,
        par: 1800.0,
        ice: false,
        broad: "closed forest",
    },
    Site {
        name: "Yakutsk",
        t: -8.8,
        cold: -38.0,
        warm: 19.0,
        p: 0.24,
        pet: 0.35,
        par: 1500.0,
        ice: false,
        broad: "closed forest",
    },
    Site {
        name: "Utqiagvik (Barrow)",
        t: -11.0,
        cold: -26.0,
        warm: 4.0,
        p: 0.12,
        pet: 0.15,
        par: 1100.0,
        ice: false,
        broad: "tundra",
    },
    Site {
        name: "Vostok",
        t: -55.0,
        cold: -68.0,
        warm: -32.0,
        p: 0.02,
        pet: 0.0,
        par: 1200.0,
        ice: true,
        broad: "ice",
    },
];

fn input(s: &Site) -> CoverInput {
    let k = 273.15;
    CoverInput {
        t_k: s.t + k,
        t_cold_k: s.cold + k,
        t_warm_k: s.warm + k,
        pet_m_yr: s.pet,
        evap_m_yr: fu_evaporation(s.p, s.pet, 2.6),
        ice_balance_m_yr: if s.ice { s.p } else { -0.1 },
        par_mj_m2_yr: s.par,
        gravity_m_s2: 9.81,
        ignition_rel: 1.0,
    }
}

#[test]
fn earth_sites_get_their_broad_types() {
    let p = CoverParams::default();
    let mut right = 0;
    let mut report = vec![];
    for s in SITES {
        let x = input(s);
        let c = p.cover(&x, true);
        let b = p.biome(&c, &x);
        if b.broad == s.broad {
            right += 1;
        }
        report.push(format!(
            "{}: {} ({}), want {} | alpha {:.2} closure {:.2} canopy {:.0} m npp {:.0} fire {:.2}/yr",
            s.name, b.earth, b.broad, s.broad, c.alpha, c.closure, c.canopy_m, c.npp_gc_m2_yr, c.fire_per_yr
        ));
    }
    // The design's bar is 70 percent of land area; for stations, all eight.
    println!("{}", report.join("\n"));
    assert_eq!(right, SITES.len(), "{}", report.join("\n"));
}

#[test]
fn without_fire_the_savannah_closes_into_forest() {
    let p = CoverParams::default();
    let x = input(&SITES[1]);
    let burnt = p.cover(&x, true);
    let unburnt = p.cover(&x, false);
    assert!(
        burnt.closure < 0.6 && unburnt.closure >= 0.6,
        "{burnt:?} {unburnt:?}"
    );
    assert_eq!(p.biome(&unburnt, &x).broad, "closed forest");
    // No free oxygen, no fire: the same as switching it off.
    let anoxic = CoverInput {
        ignition_rel: 0.0,
        ..x
    };
    assert_eq!(p.cover(&anoxic, true), unburnt);
}

#[test]
fn production_and_mass_are_earthlike() {
    let p = CoverParams::default();
    let rain = p.cover(&input(&SITES[0]), true);
    // Tropical forest NPP about 800 to 1200 g C m-2 yr-1 and 10 to 25 kg C
    // m-2 of biomass (Saugier, Roy and Mooney 2001).
    assert!((700.0..1300.0).contains(&rain.npp_gc_m2_yr), "{rain:?}");
    assert!((8.0..30.0).contains(&rain.biomass_kg_c_m2), "{rain:?}");
    let tundra = p.cover(&input(&SITES[6]), true);
    assert!(
        tundra.npp_gc_m2_yr < 200.0 && tundra.npp_gc_m2_yr > 50.0,
        "{tundra:?}"
    );
}

#[test]
fn stronger_gravity_shortens_the_canopy_but_not_the_cover() {
    let p = CoverParams::default();
    let x = input(&SITES[4]);
    let heavy = CoverInput {
        gravity_m_s2: 2.0 * 9.80665,
        ..x.clone()
    };
    let (a, b) = (p.cover(&x, true), p.cover(&heavy, true));
    assert!(
        (b.canopy_m - a.canopy_m * 9.80665 / 19.6133).abs()
            < 1e-9 * a.canopy_m.max(1.0) + a.canopy_m * 0.01
    );
    assert_eq!(a.closure, b.closure);
}

#[test]
fn a_cold_dark_world_is_bare_and_an_ice_world_is_ice() {
    let p = CoverParams::default();
    let mut x = input(&SITES[2]);
    x.t_cold_k = 200.0;
    x.t_warm_k = 230.0;
    x.t_k = 215.0;
    let c = p.cover(&x, true);
    assert_eq!((c.npp_gc_m2_yr, c.closure), (0.0, 0.0));
    assert_eq!(p.biome(&c, &x).structure, "bare");
    let ice = input(&SITES[7]);
    assert_eq!(p.biome(&p.cover(&ice, true), &ice).structure, "ice");
}
