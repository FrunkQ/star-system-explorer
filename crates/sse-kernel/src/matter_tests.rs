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
