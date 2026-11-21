//! E4: soil as state, made by life (evolution design section 8).

use crate::feedback::coupled;
use crate::scenario::coupled_manifest;
use crate::soil::{Soil, SoilParams, SoilStep};

fn with_soil(seed: &str) -> crate::feedback::CoupledRun {
    let mut man = coupled_manifest("earth");
    man.air.soil = Some(SoilParams::default());
    man.seed = seed.into();
    coupled(&man).expect("coupled run")
}

fn step(npp_m2: f64, o2_rel: f64, t: f64) -> SoilStep {
    SoilStep {
        dt_myr: 5.0,
        land_area_m2: 1.0e14,
        temperature_k: t,
        land_npp_gc_yr: npp_m2 * 1.0e14,
        o2_rel,
    }
}

/// A barren, wet, rocky world that life colonises steadily: soil deepens,
/// fines and carbon build, and none of it runs away.
#[test]
fn barren_to_soil_is_monotonic_and_bounded() {
    let p = SoilParams::default();
    let mut s = Soil::default();
    let bound = p.h0_m * sse_core::math::ln(p.p0_m_myr / p.erosion_m_myr);
    let mut prev = s.clone();
    for i in 0..80 {
        let npp = 10.0 * i as f64; // 0 to 790 g C m-2 yr-1
        s = p.step(&s, &step(npp, 1.0, 288.0));
        assert!(s.depth_m >= prev.depth_m, "depth fell at {i}: {s:?}");
        assert!(s.fines >= prev.fines - 1e-12, "fines fell at {i}");
        assert!(
            s.organic_c_kg_m2 >= prev.organic_c_kg_m2,
            "carbon fell at {i}"
        );
        assert!(s.weathering_factor >= prev.weathering_factor);
        assert!(s.depth_m <= bound + 1e-12 && s.fines <= p.fines_max);
        assert!(s.weathering_factor <= p.factor_max);
        prev = s.clone();
    }
    // Barren rock at first: production with no life cannot beat erosion.
    let bare = p.step(&Soil::default(), &step(0.0, 0.0, 288.0));
    assert_eq!(bare.depth_m, 0.0);
    assert_eq!(bare.weathering_factor, 1.0);
    assert!(prev.depth_m > 0.3, "{prev:?}");
}

/// Without free oxygen only anaerobic decomposers work: the same litter
/// piles up as far more carbon (the coal-swamp lever), and warmth burns it faster.
#[test]
fn carbon_piles_up_without_oxygen_and_burns_off_in_warmth() {
    let p = SoilParams::default();
    let oxic = p.step(&Soil::default(), &step(400.0, 1.0, 288.0));
    let anoxic = p.step(&Soil::default(), &step(400.0, 0.0, 288.0));
    let warm = p.step(&Soil::default(), &step(400.0, 1.0, 298.0));
    assert!((anoxic.organic_c_kg_m2 / oxic.organic_c_kg_m2 - 10.0).abs() < 1e-9);
    assert!((oxic.organic_c_kg_m2 / warm.organic_c_kg_m2 - 2.0).abs() < 1e-9);
}

/// Earth today: soil carbon within a factor of three of the published
/// totals (about 1500 Gt C in the top metre, Scharlemann et al. 2014;
/// about 2400 Gt C to 3 m), and a weathering factor near Berner's 4.
#[test]
fn earth_soil_carbon_within_a_factor_of_three() {
    let r = with_soil("c");
    let now = r.air.soil_series.last().unwrap();
    let gt = now.soil.carbon_mol * 12.011 / 1.0e15;
    assert!((500.0..=4500.0).contains(&gt), "{gt} Gt C");
    assert!(
        (2.0..=6.0).contains(&now.soil.weathering_factor),
        "{:?}",
        now.soil
    );
    // The oxygen ledger still balances with soil carbon in it.
    assert!(r.air.worst_residual < 1.0e6, "{}", r.air.worst_residual);
    assert!(r.air.totals.soil_store > 0.0);
}

/// Some seed has land life long before plants, and that land life cools
/// the world by drawing CO2 down. Others never hold the land, and stay as
/// they were. The model allows it and does not require it.
#[test]
fn a_seed_exists_where_early_land_life_cools_the_world() {
    let mut cooled_early = false;
    let mut stayed_bare = false;
    for seed in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        let r = with_soil(seed);
        let first = r.air.soil_series.iter().find(|p| p.soil.weatherers > 0.0);
        match first {
            Some(p) if p.at_myr < 3000.0 && p.delta_t_k < -1.0 => cooled_early = true,
            None => {
                stayed_bare = true;
                assert!(r.air.soil_series.iter().all(|p| p.delta_t_k == 0.0));
            }
            _ => {}
        }
    }
    assert!(cooled_early && stayed_bare);
}

/// Off by default: a run without soil settings is L2 exactly as it was.
#[test]
fn without_soil_settings_nothing_changes() {
    let man = coupled_manifest("earth");
    assert!(man.air.soil.is_none());
    let r = coupled(&man).unwrap();
    assert!(r.air.soil_series.is_empty() && r.air.soil.is_empty());
    assert_eq!(r.air.totals.soil_store, 0.0);
}
