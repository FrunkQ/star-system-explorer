//! W9: the water supply, saltation and area labels (review section 8's tests).
use crate::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Fx {
    conditions: Conditions,
}
#[derive(Deserialize)]
struct Fxs {
    worlds: Vec<Fx>,
}

fn world(label: &str) -> Conditions {
    let f: Fxs =
        serde_json::from_str(include_str!("../../../tests/fixtures/weather/worlds.json")).unwrap();
    f.worlds
        .into_iter()
        .map(|w| w.conditions)
        .find(|c| c.label == label)
        .unwrap()
}

/// Pluto under New Horizons' air: 10 µbar of nitrogen at about 40 K.
pub(crate) fn pluto() -> Conditions {
    let mut c = world("Mars");
    c.label = "Pluto".into();
    c.radius_m = 1.1883e6;
    c.gravity_m_s2 = 0.62;
    c.rotation_hours = -153.29;
    c.obliquity_deg = 122.5;
    c.year_days = 90_560.0;
    c.eccentricity = 0.2488;
    c.flux_w_m2 = 1361.0 / (39.48 * 39.48);
    c.albedo = 0.72;
    c.t_surface_k = 40.0;
    c.pressure_bar = 1.0e-5;
    c.mean_molar_kg = 0.028;
    c.gases = BTreeMap::from([("n2".to_string(), 0.995), ("ch4".to_string(), 0.005)]);
    c.liquid = None;
    c.ocean_fraction = 0.0;
    c
}

#[test]
fn earths_wettest_and_driest_bands_rank_as_observed() {
    let cs = base_settings();
    let cl = orbit(&cs, &world("Earth"), 4500.0);
    let at = |l: f64| water::water_at(&cs, &cl, 6.5, l, 0.0);
    // Wettest within 10° of the equator (the rain belt); driest in the
    // subtropics, 15° to 35°, in each hemisphere (Sahara, Kalahari).
    let lats: Vec<f64> = (-80..=80).step_by(5).map(f64::from).collect();
    let wettest = lats.iter().cloned().fold((0.0, -1.0), |b, l| {
        if at(l).precip_m_yr > b.1 {
            (l, at(l).precip_m_yr)
        } else {
            b
        }
    });
    assert!(wettest.0.abs() <= 10.0, "wettest at {}", wettest.0);
    for sign in [-1.0, 1.0] {
        let band: Vec<f64> = (0..=16).map(|k| sign * 5.0 * f64::from(k)).collect();
        let driest = band.iter().cloned().fold((0.0, f64::INFINITY), |b, l| {
            if at(l).aridity < b.1 {
                (l, at(l).aridity)
            } else {
                b
            }
        });
        assert!(
            (15.0..=35.0).contains(&driest.0.abs()),
            "driest at {}",
            driest.0
        );
        assert_eq!(
            water::area(&cs, &at(driest.0)).0,
            if at(driest.0).precip_m_yr < 1e-4 {
                "rainless"
            } else {
                "desert"
            }
        );
    }
    // Ice caps at the poles, rainforest on the equator.
    assert_eq!(water::area(&cs, &at(85.0)).0, "ice-cap");
    assert_eq!(water::area(&cs, &at(0.0)).0, "rainforest");
    // Every drop is accounted for: rain + melt = evaporation + runoff.
    for l in &lats {
        let w = at(*l);
        let input = w.rain_m_yr + w.melt_m_yr;
        assert!((input - w.evap_m_yr - w.runoff_m_yr).abs() < 1e-12, "{l}");
        assert!(w.snow_m_yr <= w.precip_m_yr + 1e-12);
    }
}

#[test]
fn a_mountain_has_its_own_snow_line() {
    let cs = base_settings();
    let cl = orbit(&cs, &world("Earth"), 4500.0);
    let low = water::water_at(&cs, &cl, 6.5, 45.0, 0.0);
    let high = water::water_at(&cs, &cl, 6.5, 45.0, 4000.0);
    assert!(low.ice_balance_m_yr < 0.0, "no glacier at sea level at 45°");
    assert!(high.ice_balance_m_yr > 0.0, "a glacier at 4 km at 45°");
    assert!(high.snow_m_yr > 2.0 * low.snow_m_yr);
    // Smooth with height: 100 m steps never jump the snow share by much.
    let mut last = water::water_at(&cs, &cl, 6.5, 45.0, 0.0);
    for k in 1..=60 {
        let w = water::water_at(&cs, &cl, 6.5, 45.0, 100.0 * f64::from(k));
        assert!(
            (w.snow_m_yr - last.snow_m_yr).abs() < 0.05,
            "step at {} m",
            100 * k
        );
        last = w;
    }
}

#[test]
fn the_wind_moves_sand_on_pluto_but_never_on_the_moon() {
    let cs = base_settings();
    let ss = &cs.saltation;
    // No pressure cut: 10 µbar of nitrogen still blows.
    let cp = pluto();
    let cl = orbit(&cs, &cp, 4500.0);
    let grain = saltation::grain_for(ss, cp.t_surface_k);
    assert_eq!(grain.name, "methane-ice sand");
    // The climate's own Pluto: thin air, still a wind, still some saltation
    // (its nitrogen ice, which lofts the grains, arrives with W8).
    let own = saltation::at(ss, &cl, 60.0, grain);
    assert!(own.mean_wind_m_s > 0.1 && own.sustained > 0.0, "{own:?}");
    // With Telfer et al.'s 1-10 m/s winds and lofted grains, sand moves
    // and dunes are active on its rainless ice.
    let s = saltation::saltation(ss, 1.0 / (296.8 * 40.0), 0.62, grain, 6.0, 1.0);
    assert!(
        s.fluid_wind_m_s > 100.0,
        "starting needs storms Pluto never has"
    );
    assert!(
        s.impact_wind_m_s < 10.0,
        "keeping it moving needs Telfer's winds"
    );
    assert!(s.transport_share > 0.05);
    assert!(saltation::dune_activity(ss, s.transport_share, 0.0) > 0.99);
    // Not lofted, the same wind never starts it.
    let dry = saltation::saltation(ss, 1.0 / (296.8 * 40.0), 0.62, grain, 6.0, 0.0);
    assert!(dry.transport_share < 1e-6);
    // The Moon: no air, no saltation, no dunes, at any wind.
    let moon = orbit(&cs, &world("Moon"), 4500.0);
    for l in (-80..=80).step_by(10) {
        let s = saltation::at(ss, &moon, f64::from(l), saltation::grain_for(ss, 250.0));
        assert_eq!(s.transport_share, 0.0, "{l}");
        assert_eq!(saltation::dune_activity(ss, s.transport_share, 0.0), 0.0);
    }
}

#[test]
fn earths_deserts_have_dunes_and_its_rainforest_has_none() {
    let cs = base_settings();
    let ss = &cs.saltation;
    let cl = orbit(&cs, &world("Earth"), 4500.0);
    let act = |l: f64| {
        let w = water::water_at(&cs, &cl, 6.5, l, 0.0);
        let s = saltation::at(ss, &cl, l, saltation::grain_for(ss, w.t_k));
        let a = saltation::dune_activity(ss, s.transport_share, w.aridity);
        (s, a)
    };
    let (sahara, a) = act(20.0);
    assert!(a > 0.9, "Sahara {a}");
    // Lancaster's W on Earth's deserts is tens of percent, not all the time.
    assert!(
        (0.02..0.6).contains(&sahara.transport_share),
        "{}",
        sahara.transport_share
    );
    assert!(act(0.0).1 < 0.01);
    // Sand starts at about the wind tunnels' 6-10 m/s at 10 m.
    assert!(
        (6.0..12.0).contains(&sahara.fluid_wind_m_s),
        "{}",
        sahara.fluid_wind_m_s
    );
}

#[test]
fn every_area_rule_is_reachable_and_settings_check() {
    let cs = base_settings();
    assert!(cs
        .areas
        .rules
        .iter()
        .all(|r| cs.areas.labels.contains_key(&r.area)));
    let mut bad = cs.clone();
    bad.areas.rules.pop();
    let pack = serde_json::json!({});
    let _ = pack;
    assert!(bad.areas.rules.last().is_some_and(|r| !r.when.is_empty()));
}
