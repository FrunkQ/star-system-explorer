//! SB2: small bodies through the kernel.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const SYS: &str = "u:small/sys:1";

fn open() -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-reference/pack.json")).unwrap(),
    )
    .unwrap();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[sheet, packs["base-physics"].clone()]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"small","seed":"small"}),
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":"kernel-form-9","system":SYS,"star_pins":{"primary_mass":1.0}}}),
    );
    assert_ne!(r["type"], "error", "{r}");
    k
}

fn clouds(k: &mut crate::Kernel) -> Vec<serde_json::Value> {
    let r = send(k, serde_json::json!({"op":"resolved_system","system":SYS}));
    r["clouds"].as_array().cloned().unwrap_or_default()
}

#[test]
fn a_belt_member_resolves_the_same_every_time() {
    let mut k = open();
    let cs = clouds(&mut k);
    assert!(!cs.is_empty(), "the formed system has clouds");
    let cloud = cs[0]["address"].as_str().unwrap().to_string();
    let member = format!("{cloud}/obj:40");
    let a = send(
        &mut k,
        serde_json::json!({"op":"small_body","system":SYS,"member":member}),
    );
    assert_eq!(a["type"], "small_body_is", "{a}");
    let b = send(
        &mut k,
        serde_json::json!({"op":"small_body","system":SYS,"cloud":cloud,"rank":40}),
    );
    assert_eq!(a, b);
    let body = &a["body"];
    assert_eq!(body["address"], member);
    for f in [
        "mass_kg",
        "radius_m",
        "makeup.metal",
        "makeup.rock",
        "makeup.carbon",
        "makeup.ice",
    ] {
        assert!(body["facts"][f].is_number(), "{f}");
    }
    assert!(body["history"].as_array().is_some_and(|h| !h.is_empty()));
    let bad = send(
        &mut k,
        serde_json::json!({"op":"small_body","system":SYS,"member":format!("{cloud}/obj:0")}),
    );
    assert_eq!(bad["type"], "error");
}

#[test]
fn notables_list_resolves_and_made_bodies_follow_their_pins() {
    let mut k = open();
    let n = send(&mut k, serde_json::json!({"op":"small_body","system":SYS}));
    assert_eq!(n["type"], "small_body_is", "{n}");
    for b in n["notables"].as_array().unwrap() {
        assert_eq!(b["structure"]["kind"], "stripped-core");
        assert!(b["why"].as_str().unwrap().contains("iron core"));
    }
    let addr = format!("{SYS}/small:gm-1");
    let c = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"contact-binary","diameter_km":1.0,"a_au":1.5}),
    );
    assert_eq!(c["type"], "small_body_is", "{c}");
    assert_eq!(c["body"]["pair"]["kind"], "contact");
    assert_eq!(c["body"]["figure"]["lobes"].as_array().unwrap().len(), 2);
    assert_eq!(c["body"]["address"], addr);
    let d = c["body"]["diameter_km"].as_f64().unwrap();
    assert!(d > 0.7 && d < 1.4, "{d}");
    let p = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"binary-pair","mass_kg":1e12,"a_au":1.5}),
    );
    assert_eq!(p["body"]["pair"]["kind"], "separated", "{p}");
    let comet = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"comet-nucleus","diameter_km":4.0,"a_au":3.5,"e":0.7}),
    );
    assert_eq!(comet["type"], "small_body_is", "{comet}");
    // SB4: a comet nucleus at perihelion (3.5 x 0.3 = 1.05 AU) is active, water leading.
    let act = &comet["body"]["activity"];
    assert_eq!(act["active"], true, "{act}");
    assert_eq!(act["dominant"], "h2o", "{act}");
    assert!(act["coma_km"].as_f64().unwrap() > 1e4);
    let bad = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"dragon","diameter_km":1.0,"a_au":1.0}),
    );
    assert_eq!(bad["type"], "error");
}
