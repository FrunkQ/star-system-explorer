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

/// base-sol as it ships: measured air, temperature and water reach the
/// chain, which then gives the planets we know. Plates and a field on Earth;
/// a still lid and no field on Venus and Mars.
#[test]
fn sol_from_its_catalogue_alone() {
    let mut k = open_sol(&[]);
    let e = world(&mut k, "Earth");
    assert_eq!(e["t_surface_now_k"], 288.0, "{e}");
    assert_eq!(e["regime_now"], "mobile-lid", "{e}");
    assert_eq!(e["core"]["dynamo_now"], true, "{e}");
    let v = world(&mut k, "Venus");
    assert_eq!(v["t_surface_now_k"], 737.0, "{v}");
    assert_ne!(v["regime_now"], "mobile-lid", "{v}");
    assert_eq!(v["core"]["dynamo_now"], false, "{v}");
    let m = world(&mut k, "Mars");
    assert_eq!(m["t_surface_now_k"], 214.0, "{m}");
    assert_eq!(m["regime_now"], "stagnant-lid", "{m}");
    assert_eq!(m["core"]["dynamo_now"], false, "{m}");
}

/// A world with no catalogued temperature (the Moon: NSSDCA prints only a
/// range) sits at its equilibrium temperature from its own Bond albedo.
#[test]
fn a_world_without_a_measured_mean_uses_its_own_albedo() {
    let mut k = open_sol(&[]);
    let e = world(&mut k, "Moon");
    let (teff, r_sun, a) = (5772.0_f64, 695_700e3_f64, 149.598e9_f64);
    let t_eq = teff * m::sqrt(r_sun / (2.0 * a)) * m::pow(1.0 - 0.11, 0.25);
    let t = e["t_surface_now_k"].as_f64().unwrap();
    assert!((t - t_eq).abs() < 0.01, "{t} against {t_eq}");
}

/// Facts typed over the catalogue win the same way.
#[test]
fn typed_air_and_temperature_win_over_estimates() {
    let mut k = open_sol(&[
        ("moon", "planet.surface_temperature", 250.0.into()),
        ("moon", "atmosphere.surface_pressure", 101_400.0.into()),
    ]);
    let e = world(&mut k, "Moon");
    assert_eq!(e["t_surface_now_k"], 250.0, "{e}");
}

/// A measurement belongs to the world it was made on: Mars moved to 1 AU by
/// the GM keeps its thin air's warming but answers to the brighter Sun.
#[test]
fn a_moved_world_keeps_its_air_but_not_its_old_temperature() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let mars = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == "Mars")
        .map(|(a, _)| a.clone())
        .unwrap();
    let au = sse_core::sheet::AU;
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "edits":[{"address":mars,"field":"orbit.semi_major_axis","value":au}]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let m = world(&mut k, "Mars");
    let t = m["t_surface_now_k"].as_f64().unwrap();
    let expect = 214.0 * m::sqrt(227.956e9 / au);
    assert!((t - expect).abs() < 0.01, "{t} against {expect}");
    // Typed by the GM, a temperature stands as typed, wherever the world is.
    send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "edits":[{"address":mars,"field":"planet.surface_temperature","value":230.0}]}),
    );
    assert_eq!(world(&mut k, "Mars")["t_surface_now_k"], 230.0);
}
