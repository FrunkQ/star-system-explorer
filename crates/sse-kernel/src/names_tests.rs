//! NM1: name schemes load with the stack and `name_sample` draws from them.

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
fn kernel(packs: Vec<serde_json::Value>) -> crate::Kernel {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": packs}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    k
}
fn base() -> serde_json::Value {
    pack("tests/fixtures/packs/base-physics.json")
}
fn names() -> serde_json::Value {
    pack("tests/fixtures/packs/names-toy.json")
}
fn sample(k: &mut crate::Kernel) -> serde_json::Value {
    let r = send(
        k,
        serde_json::json!({"op": "name_sample", "scheme": "vesh", "kind": "planet", "count": 2000}),
    );
    assert_eq!(r["type"], "names", "{r}");
    r
}

#[test]
fn a_sample_is_the_same_with_an_unrelated_pack_beside_it() {
    let a = sample(&mut kernel(vec![base(), names()]));
    let b = sample(&mut kernel(vec![
        base(),
        names(),
        pack("tests/fixtures/packs/reference-toy.json"),
    ]));
    assert_eq!(a["value"]["digest"], b["value"]["digest"]);
    assert_eq!(a["value"]["names"].as_array().unwrap().len(), 12);
    let first = &a["value"]["names"][0];
    assert!(first["text"].as_str().unwrap().len() > 1, "{first}");
    assert!(first["ipa"].as_str().unwrap().starts_with('/'), "{first}");
}

#[test]
fn names_change_no_physical_result() {
    let read = |packs: Vec<serde_json::Value>| {
        let mut k = kernel(packs);
        send(
            &mut k,
            serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
        );
        let form = serde_json::json!({"seed": "nm1", "system": "u:home/sys:5",
            "star_pins": {"primary_mass": 1.0}});
        let r = send(
            &mut k,
            serde_json::json!({"op": "adopt", "formation": form}),
        );
        assert_eq!(r["type"], "adopted", "{r}");
        let mut v = send(
            &mut k,
            serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
        );
        v.as_object_mut().unwrap().remove("state_hash");
        v
    };
    let a = read(vec![base()]);
    assert_eq!(a["type"], "system_resolved", "{a}");
    assert_eq!(a, read(vec![base(), names()]));
}

#[test]
fn a_hostile_scheme_refuses_the_stack_with_its_reason() {
    let mut bad = names();
    bad["provides"]["other"]["names"]["schemes"][0]["sounds"]["length"] =
        serde_json::json!([2, 40]);
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [base(), bad]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(r.to_string().contains("vesh"), "{r}");
    let mut odd = names();
    odd["provides"]["other"]["names"]["schemes"][1]["forms"]["planet"] =
        serde_json::json!([["{parent} \u{202e}II", 1]]);
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [base(), odd]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
}

#[test]
fn an_unknown_scheme_or_count_is_a_plain_error() {
    let mut k = kernel(vec![base(), names()]);
    for q in [
        serde_json::json!({"op": "name_sample", "scheme": "nobody", "kind": "planet"}),
        serde_json::json!({"op": "name_sample", "scheme": "vesh", "kind": "planet", "count": 0}),
    ] {
        let r = send(&mut k, q);
        assert_eq!(r["type"], "error", "{r}");
    }
}

/// NM3a: the shipped base-names pack loads with the stack, and its
/// Catalogue planets come back numbered under their parent.
#[test]
fn base_names_loads_with_the_stack() {
    let mut k = kernel(vec![base(), pack("packs/base-names/pack.json")]);
    let r = send(
        &mut k,
        serde_json::json!({"op": "name_sample", "scheme": "catalogue", "kind": "planet",
            "count": 3, "parent": "SX 000102"}),
    );
    assert_eq!(r["type"], "names", "{r}");
    let text = r.to_string();
    assert!(text.contains("SX 000102 I"), "{text}");
}
