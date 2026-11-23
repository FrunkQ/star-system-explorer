//! The catalogued Solar System through the surface chain: Sol enters from
//! `packs/base-sol` as one catalogued batch per body, the way the start
//! sheet's door sends it, and its worlds resolve from those facts.

use sse_core::math as m;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const SOL: &str = "u:home/sys:sol";

/// The base-sol bodies at the start sheet's addresses (Sun star:0, bodies
/// body:N in sheet order, moons under their host), values as the pack has
/// them, plus `extra` facts by body id.
fn open_sol(extra: &[(&str, &str, serde_json::Value)]) -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["base-physics"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"home","seed":"home"}),
    );
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-sol/pack.json")).unwrap(),
    )
    .unwrap();
    let sol = &pack["provides"]["other"]["sol"];
    let mut addr: Vec<(String, String)> = vec![];
    let mut n = 0;
    let mut edits = vec![];
    for b in sol["bodies"].as_array().unwrap() {
        let id = b["id"].as_str().unwrap().to_string();
        let a = match b["host"].as_str() {
            None => format!("{SOL}/star:0"),
            Some("sun") => {
                n += 1;
                format!("{SOL}/body:{}", n - 1)
            }
            Some(h) => {
                let ha = &addr.iter().find(|(i, _)| i == h).unwrap().1;
                format!("{ha}/body:0")
            }
        };
        edits.push(serde_json::json!({"address":a,"field":"name","value":b["name"]}));
        if let Some(h) = b["host"].as_str() {
            let ha = addr.iter().find(|(i, _)| i == h).unwrap().1.clone();
            edits.push(serde_json::json!({"address":a,"field":"orbit.host","value":ha}));
        }
        for (f, v) in b["values"].as_object().unwrap() {
            if f == "system.age_gyr" {
                let s = v["v"].as_f64().unwrap() * 1e9 * sse_core::sheet::YR_JULIAN;
                edits.push(serde_json::json!({"address":SOL,"field":"system.age","value":s}));
            } else {
                edits.push(serde_json::json!({"address":a,"field":f,"value":v["v"]}));
            }
        }
        for (_, f, v) in extra.iter().filter(|(i, _, _)| *i == id) {
            edits.push(serde_json::json!({"address":a,"field":f,"value":v}));
        }
        addr.push((id, a));
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "evidence":"catalogued","source":"packs/base-sol","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    k
}

fn world(k: &mut crate::Kernel, name: &str) -> serde_json::Value {
    let r = send(k, serde_json::json!({"op":"resolved_system","system":SOL}));
    assert_eq!(r["type"], "system_resolved", "{r}");
    let addr = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == name)
        .map(|(a, _)| a.clone())
        .unwrap_or_else(|| panic!("no {name}"));
    r["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["address"] == addr.as_str())
        .cloned()
        .unwrap_or_else(|| panic!("no surface for {name}: {r}"))
}

/// With only what base-sol carries today (no air, no water), Earth sits at
/// its equilibrium temperature from its own Bond albedo: honest, but frozen.
#[test]
fn a_world_without_catalogued_air_uses_its_own_albedo() {
    let mut k = open_sol(&[]);
    let e = world(&mut k, "Earth");
    let (teff, r_sun, a) = (5772.0_f64, 695_700e3_f64, 149.598e9_f64);
    let t_eq = teff * m::sqrt(r_sun / (2.0 * a)) * m::pow(1.0 - 0.294, 0.25);
    let t = e["t_surface_now_k"].as_f64().unwrap();
    assert!((t - t_eq).abs() < 0.01, "{t} against {t_eq}");
}

/// Measured air, temperature and water win over every estimate, and the
/// chain then gives the planets we know: plates and a field on Earth, a
/// still lid and no field on Venus and Mars.
#[test]
fn catalogued_air_and_water_reach_the_surface_chain() {
    let mut k = open_sol(&[
        ("earth", "planet.surface_temperature", 288.0.into()),
        ("earth", "atmosphere.surface_pressure", 101_400.0.into()),
        ("earth", "body.surface_water_oceans", 1.0.into()),
        ("venus", "planet.surface_temperature", 737.0.into()),
        ("venus", "atmosphere.surface_pressure", 9.2e6.into()),
        ("mars", "planet.surface_temperature", 210.0.into()),
        ("mars", "atmosphere.surface_pressure", 636.0.into()),
    ]);
    let e = world(&mut k, "Earth");
    assert_eq!(e["t_surface_now_k"], 288.0, "{e}");
    assert_eq!(e["regime_now"], "mobile-lid", "{e}");
    assert_eq!(e["core"]["dynamo_now"], true, "{e}");
    let v = world(&mut k, "Venus");
    assert_eq!(v["t_surface_now_k"], 737.0, "{v}");
    assert_ne!(v["regime_now"], "mobile-lid", "{v}");
    assert_eq!(v["core"]["dynamo_now"], false, "{v}");
    let m = world(&mut k, "Mars");
    assert_eq!(m["t_surface_now_k"], 210.0, "{m}");
    assert_eq!(m["regime_now"], "stagnant-lid", "{m}");
    assert_eq!(m["core"]["dynamo_now"], false, "{m}");
}
