//! ER11a: the `bake` op hands over `sse.table/1` tables from the physics
//! in force, kept by id, subject, size and physics hash.

use serde_json::{json, Value};
use std::path::PathBuf;

fn pack(p: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    serde_json::from_str(&std::fs::read_to_string(root.join(p)).unwrap()).unwrap()
}

fn kernel(packs: &[&str]) -> crate::Kernel {
    let mut k = crate::Kernel::new();
    let packs: Vec<Value> = packs.iter().map(|p| pack(p)).collect();
    let r = call(&mut k, json!({"op": "load_stack", "packs": packs}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    k
}

fn call(k: &mut crate::Kernel, req: Value) -> Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const BASE: [&str; 2] = [
    "tests/fixtures/packs/base-physics.json",
    "packs/base-reference/pack.json",
];

/// The 10 Sun hole fed by a companion, as `read_light_paths` gives it.
fn disc(k: &mut crate::Kernel) -> Value {
    let r = call(
        k,
        json!({"op": "read_light_paths", "fixture": "black-holes"}),
    );
    let p = &r["paths"][0];
    let f = &p["feed"];
    let share = |kind: &str| {
        f["flows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["kind"] == kind)
            .map_or(0.0, |x| x["strength"].as_f64().unwrap())
    };
    json!({
        "gravitational_radius_m": p["gravitational_radius_m"],
        "rate_kg_s": (share("thin") + share("thick")) * f["rate_kg_s"].as_f64().unwrap(),
        "inner_m": f["disc"]["inner_m"],
        "outer_m": f["disc"]["outer_m"],
    })
}

#[test]
fn the_disc_table_is_baked_kept_and_sized_by_tier() {
    let mut k = kernel(&BASE);
    let subject = disc(&mut k);
    let ask = |tier: &str| json!({"op": "bake", "table": "sse.light.disc", "subject": subject, "tier": tier});
    let a = call(&mut k, ask("gpu-high"));
    assert_eq!(a["type"], "baked", "{a}");
    assert_eq!(a["kept"], false);
    let t = &a["table"];
    assert_eq!(t["format"], "sse.table/1");
    assert_eq!(t["physics"], k.physics().hash().hex());
    assert_eq!(t["size"], "medium");
    assert_eq!(t["sampling"]["within"], true, "{}", t["sampling"]);
    let cells: u64 = t["axes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["n"].as_u64().unwrap())
        .product();
    assert_eq!(t["values"].as_array().unwrap().len() as u64, cells * 2);
    assert!(t["reads"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == "sse.light.disc_shift"));
    // Asked again: the kept copy, the same numbers.
    let b = call(&mut k, ask("gpu-high"));
    assert_eq!(b["kept"], true);
    assert_eq!(b["table"]["hash"], t["hash"]);
    // A phone's tier gets the small size; a still the large.
    let small = call(&mut k, ask("gpu-basic"));
    assert_eq!(small["table"]["size"], "small");
    let large = call(&mut k, ask("still"));
    assert_eq!(large["table"]["size"], "large");
    assert!(
        large["table"]["values"].as_array().unwrap().len()
            > small["table"]["values"].as_array().unwrap().len()
    );
    assert_eq!(k.baked.len(), 3);
}

#[test]
fn a_new_physics_bakes_afresh() {
    let live = "packs/base-physics/pack.json";
    let mut k = kernel(&[live]);
    let subject = disc(&mut k);
    let ask = json!({"op": "bake", "table": "sse.light.disc", "subject": subject});
    let a = call(&mut k, ask.clone());
    let r = call(
        &mut k,
        json!({"op": "load_stack", "packs": [pack(live), pack("tests/fixtures/packs/reference-strong-gravity.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let b = call(&mut k, ask);
    assert_eq!(
        b["kept"], false,
        "a different physics hash is a different table"
    );
    assert_ne!(a["table"]["physics"], b["table"]["physics"]);
}

#[test]
fn an_unknown_table_or_a_bad_subject_says_what_is_baked() {
    let mut k = kernel(&BASE);
    let r = call(&mut k, json!({"op": "bake", "table": "sse.nothing"}));
    assert_eq!(r["type"], "error");
    assert_eq!(r["code"], "bad_table");
    assert!(
        r["message"].as_str().unwrap().contains("sse.light.disc"),
        "{r}"
    );
    let r = call(
        &mut k,
        json!({"op": "bake", "table": "sse.light.disc", "subject": {"rate_kg_s": 1.0}}),
    );
    assert_eq!(r["code"], "bad_table", "{r}");
}
