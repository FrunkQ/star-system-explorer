//! MP1: the master data sheet loads with the stack and `read_matter` reads it.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn pack(p: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
}
fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}
fn load(k: &mut crate::Kernel, top: &str) -> serde_json::Value {
    send(
        k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("tests/fixtures/packs/base-physics.json"), pack(top)]}),
    )
}

#[test]
fn a_sheet_with_five_mistakes_is_refused_with_five_messages() {
    let mut k = crate::Kernel::new();
    let r = load(&mut k, "tests/fixtures/packs/reference-five-errors.json");
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert_eq!(r["issues"].as_array().unwrap().len(), 5, "{r}");
}

#[test]
fn read_matter_gives_a_row_with_its_sources_and_packs() {
    let mut k = crate::Kernel::new();
    let a = load(&mut k, "tests/fixtures/packs/reference-toy.json");
    assert_eq!(a["type"], "stack_loaded", "{a}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "substances", "id": "h2o"}),
    );
    assert_eq!(r["type"], "matter", "{r}");
    let v = &r["value"];
    assert_eq!(v["packs"], serde_json::json!(["test.reference-toy"]));
    let mm = v["worked"]["molar_mass"]["v"].as_f64().unwrap();
    assert!((mm - 0.018015).abs() < 1e-9, "{mm}");
    // By id alone, and a listing.
    let r = send(&mut k, serde_json::json!({"op": "read_matter", "id": "o"}));
    assert_eq!(r["value"]["table"], "elements", "{r}");
    let r = send(&mut k, serde_json::json!({"op": "read_matter"}));
    assert_eq!(
        r["value"]["substances"],
        serde_json::json!(["h2", "h2o", "o2"]),
        "{r}"
    );
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "spells"}),
    );
    assert_eq!(r["type"], "error", "{r}");
    // The same stack hashes the same twice.
    let mut k2 = crate::Kernel::new();
    let b = load(&mut k2, "tests/fixtures/packs/reference-toy.json");
    assert_eq!(a["stack_hash"], b["stack_hash"]);
}

fn derive(k: &mut crate::Kernel, flow: &str, inputs: serde_json::Value) -> serde_json::Value {
    let r = send(
        k,
        serde_json::json!({"op": "derive", "flow": flow, "inputs": inputs}),
    );
    assert_eq!(r["type"], "derived", "{r}");
    r["values"].clone()
}

/// MP1c: with G doubled, the blocks that follow G give the right answer for
/// the new value; with the real sheet they give exactly today's numbers.
#[test]
fn doubling_g_shortens_the_year_by_root_two_and_doubles_gravity() {
    let orbit = serde_json::json!({
        "host_mass": {"value": 1, "unit": "m_sun"}, "body_mass": {"value": 1, "unit": "m_earth"},
        "semi_major_axis": {"value": 1, "unit": "au"}, "eccentricity": {"value": 0.0167}});
    let bulk = serde_json::json!({"mass": {"value": 1, "unit": "m_earth"}, "radius": {"value": 6371, "unit": "km"}});
    let base = || pack("packs/base-physics/pack.json");
    let mut real = crate::Kernel::new();
    let r = send(
        &mut real,
        serde_json::json!({"op": "load_stack", "packs": [base()]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let mut strong = crate::Kernel::new();
    let r = send(
        &mut strong,
        serde_json::json!({"op": "load_stack", "packs": [
        base(), pack("tests/fixtures/packs/reference-strong-gravity.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");

    let v = |vals: &serde_json::Value, q: &str| {
        vals[q]["value"]
            .as_f64()
            .unwrap_or_else(|| panic!("{q}: {vals}"))
    };
    let (a, b) = (
        derive(&mut real, "sse.base.orbit-basics", orbit.clone()),
        derive(&mut strong, "sse.base.orbit-basics", orbit),
    );
    let ratio = v(&b, "orbit.period") / v(&a, "orbit.period");
    assert!(
        (ratio - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12,
        "{ratio}"
    );
    let (a, b) = (
        derive(&mut real, "sse.base.planet-bulk", bulk.clone()),
        derive(&mut strong, "sse.base.planet-bulk", bulk),
    );
    let ratio = v(&b, "body.surface_gravity") / v(&a, "body.surface_gravity");
    assert!((ratio - 2.0).abs() < 1e-12, "{ratio}");
    // Nothing leaks: after the strong-gravity call, the real kernel still
    // gives the real year.
    assert_eq!(sse_core::constants::g(), sse_core::quantity::consts::G);
}

/// MP1c: the reach report puts every block in exactly one group, and every
/// declared block is a real atom.
#[test]
fn the_reach_report_lists_every_block_once() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-physics/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "G", "value": 1.33486e-10}),
    );
    assert_eq!(r["type"], "reach", "{r}");
    let mut seen = std::collections::BTreeSet::new();
    for g in ["follows", "estimated", "cannot_follow"] {
        for b in r[g].as_array().unwrap() {
            assert!(
                seen.insert(b["block"].as_str().unwrap().to_string()),
                "{b} twice"
            );
        }
    }
    let all: std::collections::BTreeSet<String> = k.atoms.decls().map(|d| d.id.clone()).collect();
    assert_eq!(seen, all);
    for (a, _) in sse_atoms::reach::DECLARED {
        assert!(all.contains(*a), "declared block {a} is not an atom");
    }
    let period = r["follows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["block"] == "sse.orbit.period")
        .unwrap();
    assert_eq!(period["uses_it"], true);
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "charm"}),
    );
    assert_eq!(r["type"], "error", "{r}");
}

/// MP1c (design 7A.6): a unit added on the sheet can be typed like any
/// built-in one.
#[test]
fn a_unit_from_the_sheet_can_be_typed() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
        pack("packs/base-physics/pack.json"), pack("tests/fixtures/packs/reference-toy.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let orbit = |a: serde_json::Value| {
        serde_json::json!({
        "host_mass": {"value": 1, "unit": "m_sun"}, "body_mass": {"value": 1, "unit": "m_earth"},
        "semi_major_axis": a, "eccentricity": {"value": 0.0}})
    };
    let by_league = derive(
        &mut k,
        "sse.base.orbit-basics",
        orbit(serde_json::json!({"value": 1.0e6, "unit": "league"})),
    );
    let by_m = derive(
        &mut k,
        "sse.base.orbit-basics",
        orbit(serde_json::json!({"value": 4.828032e9, "unit": "m"})),
    );
    assert_eq!(
        by_league["orbit.period"]["value"],
        by_m["orbit.period"]["value"]
    );
}

/// The shipped sheet (or the one `SSE_BASE_REFERENCE` names) loads in a
/// stack, its units included, and does not move a constant.
#[test]
fn the_base_sheet_loads_in_a_stack() {
    let path = std::env::var("SSE_BASE_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root().join("packs/base-reference/pack.json"));
    let Ok(s) = std::fs::read_to_string(&path) else {
        return;
    };
    let sheet: serde_json::Value = serde_json::from_str(&s).unwrap();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("tests/fixtures/packs/base-physics.json"), sheet]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let g = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "g"}),
    );
    assert_eq!(g["in_force"], g["real"], "{g}");
}
