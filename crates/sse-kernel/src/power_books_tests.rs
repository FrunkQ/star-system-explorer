//! RK9 part 2: a ship's power books through the API, and fitted engines
//! setting NAV's drive at the dimmer end of a leg.

use crate::Kernel;
use serde_json::{json, Value as J};

fn ask(k: &mut Kernel, v: J) -> J {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

/// SSE3's stock Hall thruster ship: 530 N at 20 km/s on a 0.1 MW plant.
fn hall_ship() -> J {
    json!({"name": "Hall tug", "accel_g": 0.05, "sustained": true, "exhaust_km_s": 20.0,
        "wet_t": 400.0, "dry_t": 200.0,
        "power": [{"id": "plant", "kind": "fission-reactor", "electric_w": 1.0e5}],
        "engines": [{"engine": {"name": "Hall thruster", "thrust_n": 530.0, "exhaust_km_s": 20.0,
            "energy_from": "electric", "electric_w": 1.0e5}}]})
}

/// Dawn-like: a 10 kW array at 1 au and one NSTAR (92 mN, 30.4 km/s, 2.6 kW, 54%).
fn solar_ship() -> J {
    json!({"name": "Sunfarer", "accel_g": 1.0e-5, "sustained": true, "exhaust_km_s": 30.4,
        "wet_t": 1.2, "dry_t": 0.75,
        "power": [{"id": "array", "kind": "solar-array", "electric_w": 1.0e4,
            "law": {"kind": "sunlight", "ref_flux_w_m2": 1361.0}}],
        "loads": [{"name": "bus", "electric_w": 1500.0}],
        "engines": [{"engine": {"name": "NSTAR", "thrust_n": 0.092, "exhaust_km_s": 30.4,
            "energy_from": "electric", "electric_w": 2600.0, "efficiency": 0.54}}]})
}

#[test]
fn a_drive_given_more_than_its_power_is_marked_fictional() {
    let mut k = Kernel::new();
    let r = ask(&mut k, json!({"op": "ship_power", "ship": hall_ship()}));
    assert_eq!(r["type"], "ship_power_is", "{r}");
    let p = &r["power"];
    // ½ × 530 N × 20 km/s = 5.3 MW of jet on 0.1 MW: 5.2 MW from nowhere.
    let nowhere = p["ledger"]["nowhere_w"].as_f64().unwrap();
    assert!((nowhere - 5.2e6).abs() < 1e4, "{nowhere}");
    assert_eq!(p["balance"]["ok"], false);
    assert_eq!(p["fictional"], true);
    assert!(
        p["balance"]["says"]
            .as_str()
            .unwrap()
            .contains("do not close by 5.2 MW"),
        "{p}"
    );
    // It still flies on its stated numbers: danger is drama.
    let g = p["accel_g"].as_f64().unwrap();
    assert!((g - 530.0 / 400_000.0 / 9.80665).abs() < 1e-9, "{g}");
}

#[test]
fn sunlight_power_falls_with_distance() {
    let mut k = Kernel::new();
    let near = ask(&mut k, json!({"op": "ship_power", "ship": solar_ship()}));
    let p = &near["power"];
    assert_eq!(p["balance"]["ok"], true, "{p}");
    assert_eq!(p["fictional"], false);
    let full = p["accel_g"].as_f64().unwrap();
    // At 3 au the array gives a ninth: 1.1 kW, less than the bus takes.
    let far = ask(
        &mut k,
        json!({"op": "ship_power", "ship": solar_ship(), "flux_w_m2": 1361.0 / 9.0}),
    );
    let a = far["power"]["accel_g"].as_f64().unwrap();
    assert!(a < full * 0.01, "{a} {full}");
    assert!(far["power"]["ledger"]["spare_w"].as_f64().unwrap() < 0.0);
}

#[test]
fn a_leg_reads_sunlight_at_its_dimmer_end() {
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        json!({"op": "plan_route", "fixture": "sol", "ship": solar_ship(),
        "from": "Earth", "to": "Mars", "window_days": 800.0}),
    );
    assert_eq!(r["type"], "route_planned", "{r}");
    let p = &r["plan"]["power"];
    let flux = p["flux_w_m2"].as_f64().unwrap();
    // Mars at 1.52 au: about 1361 / 2.32 = 590 W/m², not the Earth's 1361.
    assert!((500.0..700.0).contains(&flux), "{flux}");
    // A ship with no fitted engines keeps its typed drive and has no books.
    let r = ask(
        &mut k,
        json!({"op": "plan_route", "fixture": "sol", "ship": "chemical-transfer",
        "from": "Earth", "to": "Mars", "window_days": 800.0}),
    );
    assert!(r["plan"].get("power").is_none());
}

#[test]
fn engines_by_id_come_from_the_stack() {
    let mut k = Kernel::new();
    let pack = json!({"format": "sse.pack/1", "id": "t.engines", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"engines": {"rl10": {"name": "RL10", "fuels": ["hydrogen-oxygen"],
            "thrust_n": 110000.0, "exhaust_km_s": 4.5, "energy_from": "fuel"}}}}});
    let r = ask(&mut k, json!({"op": "load_stack", "packs": [pack]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let ship = json!({"name": "Upper stage", "accel_g": 0.0, "exhaust_km_s": 1.0,
        "wet_t": 20.0, "dry_t": 2.0, "engines": [{"engine": "rl10"}]});
    let r = ask(&mut k, json!({"op": "ship_power", "ship": ship}));
    let p = &r["power"];
    assert!(
        (p["exhaust_km_s"].as_f64().unwrap() - 4.5).abs() < 1e-12,
        "{p}"
    );
    assert!((p["accel_g"].as_f64().unwrap() - 110000.0 / 20000.0 / 9.80665).abs() < 1e-9);
    assert_eq!(p["balance"]["ok"], true);
    // An id the stack does not have is named, never a failure.
    let ship = json!({"name": "Mystery", "accel_g": 0.1, "exhaust_km_s": 3.0,
        "wet_t": 20.0, "dry_t": 2.0, "engines": [{"engine": "warp-core"}]});
    let r = ask(&mut k, json!({"op": "ship_power", "ship": ship}));
    assert!(r["power"]["why"].to_string().contains("warp-core"), "{r}");
}
