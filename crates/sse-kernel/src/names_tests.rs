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
    pack("packs/base-physics/pack.json")
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

// NM2: names on real addresses.

const SYS: &str = "u:home/sys:5";
const STAR: &str = "u:home/sys:5/star:0";
const P1: &str = "u:home/sys:5/body:0";
const P2: &str = "u:home/sys:5/body:1";
const P3: &str = "u:home/sys:5/body:2";
const MOON: &str = "u:home/sys:5/body:9";

/// A grown system (a star and five planets) with an authored moon on the
/// third planet, under the names campaign.
fn campaign() -> crate::Kernel {
    let mut k = kernel(vec![
        base(),
        pack("tests/fixtures/packs/names-campaign.json"),
    ]);
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let form =
        serde_json::json!({"seed": "nm1", "system": SYS, "star_pins": {"primary_mass": 1.0}});
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "set_authored_batch", "occurred_at_s": 0, "edits": [
            {"address": MOON, "field": "body.kind", "value": "moon"},
            {"address": MOON, "field": "parent", "value": P3}
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    k
}
fn names_of(k: &mut crate::Kernel, a: &str, audience: Option<&str>) -> BTreeMap<String, String> {
    let mut q = serde_json::json!({"op": "read_names", "address": a});
    if let Some(x) = audience {
        q["audience"] = x.into();
    }
    let r = send(k, q);
    assert_eq!(r["type"], "names", "{r}");
    r["value"]["names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["layer"].as_str().unwrap().to_string(),
                n["text"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}
fn ok(r: serde_json::Value) -> serde_json::Value {
    assert_eq!(r["type"], "authored", "{r}");
    r
}
use std::collections::BTreeMap;

#[test]
fn a_grown_system_has_designations_and_common_names() {
    let mut k = campaign();
    let star = names_of(&mut k, STAR, None);
    assert_eq!(star["designation"], "KS 0005");
    let p3 = names_of(&mut k, P3, None);
    assert_eq!(p3["designation"], "KS 0005 III");
    assert!(!p3["common"].is_empty());
    assert_eq!(names_of(&mut k, MOON, None)["designation"], "KS 0005 III a");
    // Siblings never share a common name.
    let commons: Vec<String> = [P1, P2, P3]
        .iter()
        .map(|a| names_of(&mut k, a, None)["common"].clone())
        .collect();
    assert_ne!(commons[0], commons[1]);
    assert_ne!(commons[1], commons[2]);
    // The catalogue has no form for a system; Vesh names anything.
    let sys = names_of(&mut k, SYS, None);
    assert!(
        !sys.contains_key("designation") && sys.contains_key("common"),
        "{sys:?}"
    );
}

#[test]
fn each_seat_sees_only_the_names_it_knows() {
    let mut k = campaign();
    ok(send(
        &mut k,
        serde_json::json!({"op": "set_naming", "address": "u:home", "layer": "local_vesh", "scheme": "vesh", "at_s": 0}),
    ));
    for (aud, view) in [("near", "scanned"), ("lander", "survey")] {
        let r = send(
            &mut k,
            serde_json::json!({"op": "grant", "address": SYS, "audience": aud, "view": view, "at_s": 0}),
        );
        assert_ne!(r["type"], "error", "{r}");
    }
    let layers = |k: &mut crate::Kernel, aud| -> Vec<String> {
        names_of(k, P3, Some(aud)).into_keys().collect()
    };
    assert_eq!(layers(&mut k, "far"), vec!["designation"]);
    assert_eq!(layers(&mut k, "near"), vec!["common", "designation"]);
    assert_eq!(
        layers(&mut k, "lander"),
        vec!["common", "designation", "local_vesh"]
    );
    assert_eq!(
        layers(&mut k, "gm"),
        vec!["common", "designation", "local_vesh"]
    );
    // The scene carries the same names, and no bookkeeping.
    let s = send(
        &mut k,
        serde_json::json!({"op": "scene", "audience": "near", "scope": P3, "at_s": 0}),
    );
    let fields: Vec<&str> = s["scene"]["subjects"][0]["facts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["field"].as_str().unwrap())
        .collect();
    assert!(fields.contains(&"name.common"), "{fields:?}");
    assert!(!fields
        .iter()
        .any(|f| f.starts_with("naming.") || f.starts_with("names.")));
}

#[test]
fn names_read_back_the_same_after_replay() {
    let read = || {
        let mut k = campaign();
        ok(send(
            &mut k,
            serde_json::json!({"op": "fix_names", "address": P2, "at_s": 5}),
        ));
        ok(send(
            &mut k,
            serde_json::json!({"op": "set_naming", "address": SYS, "layer": "common", "scheme": "catalogue", "at_s": 10}),
        ));
        let all: Vec<_> = [STAR, P1, P2, P3, MOON]
            .iter()
            .map(|a| names_of(&mut k, a, None))
            .collect();
        let h = send(&mut k, serde_json::json!({"op": "state_hash"}));
        (all, h)
    };
    assert_eq!(read(), read());
}

#[test]
fn a_typed_name_survives_its_world_being_grown_again() {
    let mut k = campaign();
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": P3, "text": "Haven", "at_s": 0}),
    ));
    // Grown again: new physical facts on the world.
    ok(send(
        &mut k,
        serde_json::json!({"op": "set_authored_batch", "occurred_at_s": 20, "edits": [
            {"address": P3, "field": "body.mass", "value": 7.0e24},
            {"address": P3, "field": "body.radius", "value": 6.9e6}
        ]}),
    ));
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_names", "address": P3}),
    );
    let common = r["value"]["names"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["layer"] == "common")
        .unwrap()
        .clone();
    assert_eq!(common["text"], "Haven");
    assert_eq!(common["source"], "typed");
    let original = common["original"].as_str().unwrap().to_string();
    // Taking the typed name away shows the drawn one again.
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": P3, "at_s": 30}),
    ));
    assert_eq!(names_of(&mut k, P3, None)["common"], original);
    // Names that are not safe text are refused.
    let r = send(
        &mut k,
        serde_json::json!({"op": "rename", "address": P3, "text": "Bad\u{202e}name", "at_s": 40}),
    );
    assert_eq!(r["code"], "bad_name", "{r}");
}

#[test]
fn a_setting_changes_only_unfixed_names_beneath_it() {
    let mut k = campaign();
    let before: Vec<_> = [P1, P2, P3, MOON]
        .iter()
        .map(|a| names_of(&mut k, a, None)["common"].clone())
        .collect();
    let fixed = ok(send(
        &mut k,
        serde_json::json!({"op": "fix_names", "address": P2, "at_s": 5}),
    ));
    assert_eq!(fixed["affected"], serde_json::json!([P2]));
    // The catalogue for common names on the third planet and what hangs from it.
    ok(send(
        &mut k,
        serde_json::json!({"op": "set_naming", "address": P3, "layer": "common", "scheme": "catalogue", "at_s": 10}),
    ));
    assert_eq!(names_of(&mut k, P1, None)["common"], before[0]);
    assert_eq!(names_of(&mut k, P2, None)["common"], before[1]);
    // The catalogue follows the star's name in the same layer: its Vesh name.
    let star = names_of(&mut k, STAR, None)["common"].clone();
    assert_eq!(names_of(&mut k, P3, None)["common"], format!("{star} III"));
    assert_eq!(
        names_of(&mut k, MOON, None)["common"],
        format!("{star} III a")
    );
    // On the whole system: the fixed planet keeps its name...
    ok(send(
        &mut k,
        serde_json::json!({"op": "set_naming", "address": SYS, "layer": "common", "scheme": "catalogue", "at_s": 20}),
    ));
    assert_eq!(names_of(&mut k, P2, None)["common"], before[1]);
    // The star is under the setting too, so its common name is now "KS 0005".
    assert_eq!(names_of(&mut k, STAR, None)["common"], "KS 0005");
    assert_eq!(names_of(&mut k, P1, None)["common"], "KS 0005 I");
    // ...until the GM renames fixed names from the new scheme, as of a date.
    let r = ok(send(
        &mut k,
        serde_json::json!({"op": "set_naming", "address": SYS, "layer": "common", "scheme": "catalogue", "at_s": 30, "rename_fixed": true}),
    ));
    assert!(
        r["affected"].as_array().unwrap().contains(&P2.into()),
        "{r}"
    );
    assert_eq!(names_of(&mut k, P2, None)["common"], "KS 0005 II");
    // The history keeps the old name before that date.
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_names", "address": P2, "at_s": 25}),
    );
    assert!(r.to_string().contains(&before[1]), "{r}");
    // An unknown scheme is refused.
    let r = send(
        &mut k,
        serde_json::json!({"op": "set_naming", "address": SYS, "layer": "common", "scheme": "nobody", "at_s": 40}),
    );
    assert_eq!(r["code"], "unknown_scheme", "{r}");
}

#[test]
fn renaming_a_star_renames_what_follows_it_in_every_layer() {
    let mut k = campaign();
    ok(send(
        &mut k,
        serde_json::json!({"op": "set_naming", "address": SYS, "layer": "common", "scheme": "catalogue", "at_s": 0}),
    ));
    let events = |k: &mut crate::Kernel, a: &str| {
        send(k, serde_json::json!({"op": "history", "address": a}))["events"]
            .as_array()
            .unwrap()
            .len()
    };
    let (e3, em) = (events(&mut k, P3), events(&mut k, MOON));
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": STAR, "layer": "designation", "text": "HD 102", "at_s": 1}),
    ));
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": STAR, "text": "Mystar", "at_s": 1}),
    ));
    let p3 = names_of(&mut k, P3, None);
    assert_eq!(p3["designation"], "HD 102 III");
    assert_eq!(p3["common"], "Mystar III");
    assert_eq!(names_of(&mut k, MOON, None)["common"], "Mystar III a");
    assert_eq!((events(&mut k, P3), events(&mut k, MOON)), (e3, em));
    // A typed planet name stays put; its moon follows it.
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": P3, "text": "Haven", "at_s": 2}),
    ));
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": STAR, "text": "Brightstar", "at_s": 3}),
    ));
    assert_eq!(names_of(&mut k, P3, None)["common"], "Haven");
    assert_eq!(names_of(&mut k, MOON, None)["common"], "Haven a");
    assert_eq!(names_of(&mut k, MOON, None)["designation"], "HD 102 III a");
    // A fixed name still follows: fixing stores the form, not the text.
    ok(send(
        &mut k,
        serde_json::json!({"op": "fix_names", "address": P1, "at_s": 4}),
    ));
    ok(send(
        &mut k,
        serde_json::json!({"op": "rename", "address": STAR, "layer": "designation", "text": "HD 103", "at_s": 5}),
    ));
    assert_eq!(names_of(&mut k, P1, None)["designation"], "HD 103 I");
}

#[test]
fn a_promoted_star_keeps_the_names_it_had() {
    let mut k = kernel(vec![
        base(),
        pack("tests/fixtures/packs/names-campaign.json"),
    ]);
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "aging", "seed": "aging"}),
    );
    // A drawn star of the home galaxy's own cell at the Sun's place (GZ12:
    // the galaxy, and so the address, is the universe's home halo's).
    let cell = send(&mut k, serde_json::json!({"op": "star_cell"}));
    let a = cell["cell"]["systems"][0]["address"]
        .as_str()
        .unwrap_or_else(|| panic!("no star in the home cell: {cell}"))
        .to_string();
    let a = a.as_str();
    let r = send(
        &mut k,
        serde_json::json!({"op": "promote_star", "address": a}),
    );
    assert_eq!(r["type"], "star_promoted", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_names", "address": a}),
    );
    let names = r["value"]["names"].as_array().unwrap();
    assert!(!names.is_empty(), "{r}");
    assert!(names.iter().all(|n| n["source"] == "fixed"), "{r}");
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

/// NM4 and the run-through: a V3 adoption has no kind facts and keeps its
/// names as `v3.name`. Its bodies take their catalogue kind from V3's role
/// (a planet gets a Roman numeral, not "{parent} {n}"), the common name is
/// the save's own, and a ship at the world names it, not its address.
#[test]
fn an_adopted_v3_world_keeps_its_name_and_its_kind() {
    let mut k = kernel(vec![base(), pack("packs/base-names/pack.json")]);
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let save = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "import_v3": save}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let sys = r["systems"][0].as_str().unwrap().to_string();
    let v = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": sys}),
    );
    let earth = v["spec"]["planets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "Earth")
        .unwrap_or_else(|| panic!("{}", v["spec"]))["address"]
        .as_str()
        .unwrap()
        .to_string();
    let n = names_of(&mut k, &earth, None);
    assert_eq!(n["common"], "Earth", "{n:?}");
    assert!(n["designation"].ends_with(" I"), "{n:?}");
    let moon = format!("{sys}/body:@3");
    let m = names_of(&mut k, &moon, None);
    assert_eq!(m["common"], "Moon", "{m:?}");
    assert!(m["designation"].ends_with(" I a"), "{m:?}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "ship_at_world", "system": sys,
            "ship": "chemical-transfer", "worlds": [earth]}),
    );
    assert_eq!(r["type"], "ship_at_world_read", "{r}");
    assert_eq!(r["report"]["worlds"][0]["name"], "Earth", "{r}");
}
