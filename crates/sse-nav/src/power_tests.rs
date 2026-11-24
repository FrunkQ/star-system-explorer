//! RK9: the four-bus power ledger, checked against flown hardware.
use crate::power::*;
use std::collections::BTreeMap;

const AU_FLUX: f64 = 1361.0;

fn at(flux: f64, t_s: i64) -> Conditions {
    Conditions {
        flux_w_m2: flux,
        t_s,
    }
}

fn nstar() -> EngineDef {
    // Dawn's NSTAR ion engine: 92 mN at 2.6 kW, Isp 3100 s (Brophy 2011).
    serde_json::from_value(serde_json::json!({
        "name": "NSTAR", "fuels": ["xenon"], "thrust_n": 0.092, "exhaust_km_s": 30.4,
        "energy_from": "electric", "electric_w": 2600.0, "efficiency": 0.54, "reject_k": 400.0
    }))
    .unwrap()
}

#[test]
fn ion_thrust_from_power_matches_nstar() {
    let f = thrust_from_power(2600.0, 0.54, 30.4);
    assert!((f - 0.092).abs() < 0.001, "{f}");
}

#[test]
fn a_solar_electric_ship_loses_thrust_as_the_sun_fades() {
    let arrays: Vec<PowerSource> = serde_json::from_value(serde_json::json!([{
        "id": "wings", "kind": "solar-array", "electric_w": 10000.0,
        "law": {"kind": "sunlight", "ref_flux_w_m2": AU_FLUX}
    }]))
    .unwrap();
    let mounts = vec![EngineMount {
        engine: EngineRef::Own(Box::new(nstar())),
        count: 1,
    }];
    let loads = vec![Load {
        name: "bus".into(),
        electric_w: 500.0,
        reject_k: None,
        mass_t: None,
        role: None,
        off: false,
    }];
    let defs = BTreeMap::new();
    let near = ledger(&arrays, &mounts, &loads, &defs, at(AU_FLUX, 0));
    assert!((near.thrust_n - 0.092).abs() < 1e-9);
    assert!(near.spare_w > 6000.0);
    assert_eq!(near.nowhere_w, 0.0, "{:?}", near.why);
    // At 3 au the arrays give a ninth; the bus is served first.
    let far = ledger(&arrays, &mounts, &loads, &defs, at(AU_FLUX / 9.0, 0));
    let share = (10000.0 / 9.0 - 500.0) / 2600.0;
    assert!(
        (far.thrust_n - 0.092 * share).abs() < 1e-9,
        "{}",
        far.thrust_n
    );
    assert!(far.spare_w < 0.0);
    assert!(
        far.why.iter().any(|w| w.contains("of its thrust")),
        "{:?}",
        far.why
    );
    // Every watt in ends as jet or heat.
    let heat: f64 = far.heat_by_k.values().sum();
    let jet: f64 = far.engines.iter().map(|e| e.jet_w).sum();
    assert!(
        (heat + jet - far.made_w).abs() < 1e-6,
        "{heat} + {jet} vs {}",
        far.made_w
    );
}

#[test]
fn a_radioisotope_source_halves_each_half_life() {
    // Voyager's three MHW-RTGs gave about 470 W at launch; Pu-238 halves in 87.7 years.
    let rtg: PowerSource = serde_json::from_value(serde_json::json!({
        "id": "rtg", "kind": "radioisotope", "electric_w": 470.0,
        "law": {"kind": "decay", "half_life_y": 87.7, "since_s": 1000}
    }))
    .unwrap();
    let yr = sse_core::sheet::YR_JULIAN;
    assert_eq!(source_w(&rtg, at(0.0, 0)), 470.0, "not before it was made");
    let later = source_w(&rtg, at(0.0, 1000 + (87.7 * yr) as i64));
    assert!((later - 235.0).abs() < 0.01, "{later}");
    let l = ledger(
        &[rtg],
        &[],
        &[],
        &BTreeMap::new(),
        at(0.0, 1000 + (47.0 * yr) as i64),
    );
    assert!(l.made_w > 320.0 && l.made_w < 330.0, "{}", l.made_w);
    assert!(l.why[0].contains("half-life"));
}

#[test]
fn sse3s_hall_thruster_shows_its_power_from_nowhere() {
    // SSE3's Hall thruster: 530 N at 20 km/s is a 5.3 MW jet on a 0.1 MW draw.
    let mut defs = BTreeMap::new();
    defs.insert(
        "hall".to_string(),
        serde_json::from_value::<EngineDef>(serde_json::json!({
            "name": "Hall thruster", "thrust_n": 530.0, "exhaust_km_s": 20.0,
            "energy_from": "electric", "electric_w": 100000.0
        }))
        .unwrap(),
    );
    let reactor: PowerSource = serde_json::from_value(serde_json::json!({
        "id": "core", "kind": "fission-reactor", "electric_w": 200000.0, "thermal_w": 600000.0, "reject_k": 900.0
    }))
    .unwrap();
    let mounts = vec![EngineMount {
        engine: EngineRef::Id("hall".into()),
        count: 1,
    }];
    let l = ledger(&[reactor], &mounts, &[], &defs, at(AU_FLUX, 0));
    assert!((l.nowhere_w - 5.2e6).abs() < 1.0, "{}", l.nowhere_w);
    assert!(l
        .flows
        .iter()
        .any(|f| f.from == "Nowhere" && f.to == "Hall thruster"));
    assert!(
        l.why
            .iter()
            .any(|w| w.contains("5.20 MW comes from nowhere")),
        "{:?}",
        l.why
    );
    assert_eq!(
        l.thrust_n, 530.0,
        "danger is drama: it still flies on its stated numbers"
    );
    assert_eq!(l.heat_by_k.get(&900), Some(&600000.0));
}

#[test]
fn a_chemical_engine_needs_no_electricity() {
    // RL10: 110 kN at Isp 465 s.
    let rl10: EngineDef = serde_json::from_value(serde_json::json!({
        "name": "RL10", "fuels": ["hydrogen-oxygen"], "thrust_n": 110000.0,
        "exhaust_km_s": 4.56, "energy_from": "fuel", "efficiency": 0.7
    }))
    .unwrap();
    let mounts = vec![EngineMount {
        engine: EngineRef::Own(Box::new(rl10)),
        count: 2,
    }];
    let l = ledger(&[], &mounts, &[], &BTreeMap::new(), at(0.0, 0));
    assert_eq!(l.thrust_n, 220000.0);
    assert_eq!(l.nowhere_w, 0.0);
    assert!(l.flows.iter().all(|f| f.kind != "electric"));
    assert!(l.flows.iter().any(|f| f.from == "propellant"));
    let (g, ve) = readings(&l, 22.0);
    assert!((g - 220000.0 / 22000.0 / crate::G0).abs() < 1e-12);
    assert_eq!(ve, 4.56);
}

#[test]
fn a_missing_engine_is_said_not_hidden() {
    let mounts = vec![EngineMount {
        engine: EngineRef::Id("warp".into()),
        count: 1,
    }];
    let l = ledger(&[], &mounts, &[], &BTreeMap::new(), at(0.0, 0));
    assert_eq!(l.thrust_n, 0.0);
    assert!(l.why[0].contains("warp"));
}

#[test]
fn a_ship_carries_its_ledger_and_old_ships_still_load() {
    let s: crate::Ship = serde_json::from_value(serde_json::json!({
        "accel_g": 0.0001, "exhaust_km_s": 30.4, "wet_t": 1.2, "dry_t": 0.75,
        "power": [{"kind": "solar-array", "electric_w": 10000.0, "law": {"kind": "sunlight", "ref_flux_w_m2": 1361.0}}],
        "engines": [{"engine": "nstar"}]
    }))
    .unwrap();
    assert_eq!(s.engines[0].count, 1);
    let back = serde_json::to_value(&s).unwrap();
    assert!(back.get("loads").is_none());
    let laser: Load = serde_json::from_value(serde_json::json!({
        "name": "laser", "electric_w": 5.0e7, "reject_k": 1500.0, "role": "weapon"
    }))
    .unwrap();
    assert_eq!(laser.role.as_deref(), Some("weapon"));
    let old: crate::Ship = serde_json::from_value(serde_json::json!({
        "accel_g": 1.0, "exhaust_km_s": 2000.0, "wet_t": 2.0, "dry_t": 1.0
    }))
    .unwrap();
    assert!(old.power.is_empty() && old.engines.is_empty());
}
