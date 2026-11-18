//! C2 checks: the base atlas (classification design sections 10 and 12).
//! Every class has passing examples, no two base classes overlap, the Solar
//! System and a reference exoplanet set classify as published descriptions
//! say, and the "tainted" air modifier and the "filter mask" profile agree.

use crate::classify::{Atlas, ClassKind, Expect};
use crate::predicate::Subject;
use crate::scenario::base_atlas;
use sse_core::cbor::Value;
use std::collections::BTreeMap;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
const SCHEMES: [&str; 4] = ["physical", "surface", "air", "habitability"];

fn fixture(name: &str) -> Vec<serde_json::Value> {
    let p = format!("{ROOT}tests/fixtures/classify/{name}");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    v["bodies"].as_array().unwrap().clone()
}
fn subject(b: &serde_json::Value) -> Subject {
    let f: BTreeMap<String, Value> = serde_json::from_value(b["facts"].clone()).unwrap();
    Subject::new(f)
}
fn strs(v: &serde_json::Value) -> Vec<String> {
    serde_json::from_value(v.clone()).unwrap_or_default()
}
/// Base label of one scheme. Physical ids are written bare in the fixtures.
fn base(a: &Atlas, scheme: &str, s: &Subject) -> Option<String> {
    a.label(scheme, s, None).base
}
fn modifiers(a: &Atlas, scheme: &str, s: &Subject) -> Vec<String> {
    let mut m = a.label(scheme, s, None).modifiers;
    m.sort();
    m
}

#[test]
fn base_atlas_is_valid_and_every_class_has_a_passing_example() {
    let a = base_atlas();
    a.validate().unwrap();
    assert_eq!(a.schemes.len(), 4);
    for c in &a.classes {
        assert!(SCHEMES.contains(&c.scheme.as_str()), "{}", c.id);
        assert!(!c.learn.summary.is_empty(), "{} has no learn text", c.id);
        assert!(
            c.examples.iter().any(|e| e.expect == Expect::In),
            "{} has no example",
            c.id
        );
    }
    let bad = a.check_examples();
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn no_two_base_classes_overlap() {
    let a = base_atlas();
    for s in SCHEMES {
        assert_eq!(a.overlaps(s), Vec::<(String, String)>::new(), "scheme {s}");
    }
    // The check does find an overlap when there is one.
    let mut b = a.clone();
    let mut twin = b.class("rocky").unwrap().clone();
    twin.id = "rocky-twin".into();
    twin.examples.clear();
    b.classes.push(twin);
    assert_eq!(
        a.overlaps("physical").len() + 1,
        b.overlaps("physical").len()
    );
}

#[test]
fn solar_system_classifies_as_expected() {
    let a = base_atlas();
    let mut bad = Vec::new();
    for b in fixture("solar-system.json") {
        let (name, e, s) = (b["name"].as_str().unwrap(), &b["expect"], subject(&b));
        for scheme in SCHEMES {
            let want = e[scheme].as_str().map(String::from);
            let got = base(&a, scheme, &s);
            if got != want {
                bad.push(format!("{name} {scheme}: {got:?}, expected {want:?}"));
            }
        }
        for (scheme, key) in [("physical", "physical_modifiers"), ("air", "air_modifiers")] {
            let got = modifiers(&a, scheme, &s);
            let mut want = strs(&e[key]);
            want.sort();
            if got != want {
                bad.push(format!("{name} {key}: {got:?}, expected {want:?}"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn reference_exoplanets_classify_as_published() {
    let a = base_atlas();
    let mut bad = Vec::new();
    for b in fixture("exoplanets.json") {
        let (name, s) = (b["name"].as_str().unwrap(), subject(&b));
        for (scheme, ok) in b["expect"].as_object().unwrap() {
            if scheme == "modifiers" {
                let got = modifiers(&a, "physical", &s);
                for m in strs(ok) {
                    if !got.contains(&m) {
                        bad.push(format!("{name}: no {m} in {got:?}"));
                    }
                }
                continue;
            }
            let got = base(&a, scheme, &s);
            let ok = strs(ok);
            if !got.as_ref().is_some_and(|g| ok.contains(g)) {
                bad.push(format!("{name} {scheme}: {got:?}, expected one of {ok:?}"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The air modifier "tainted" and the habitability profile "filter mask"
/// draw the same line: a breathable oxygen level with carbon dioxide the
/// shirt-sleeve profile will not take. Swept over both partial pressures.
#[test]
fn tainted_air_needs_a_filter_mask() {
    let a = base_atlas();
    let earth = fixture("solar-system.json")
        .into_iter()
        .find(|b| b["name"] == "Earth")
        .unwrap();
    for o2 in [0.1, 0.16, 0.21, 0.3, 0.5, 0.6] {
        for co2 in [0.0004, 0.004, 0.005, 0.01, 0.03, 0.05, 0.08] {
            let mut s = subject(&earth);
            s.facts.insert("atmosphere.pp.o2".into(), Value::Float(o2));
            s.facts
                .insert("atmosphere.pp.co2".into(), Value::Float(co2));
            let tainted = modifiers(&a, "air", &s).contains(&"air/tainted".to_string());
            let mask = base(&a, "habitability", &s).as_deref() == Some("habitable/filter-mask");
            assert_eq!(tainted, mask, "o2 {o2} co2 {co2}");
        }
    }
}

#[test]
fn every_scheme_has_bases_and_modifiers_where_the_design_says() {
    let a = base_atlas();
    let count = |s: &str, k: ClassKind| {
        a.classes
            .iter()
            .filter(|c| c.scheme == s && c.kind == k)
            .count()
    };
    assert!(count("physical", ClassKind::Modifier) >= 15);
    assert_eq!(count("habitability", ClassKind::Modifier), 0);
    assert_eq!(count("air", ClassKind::Modifier), 8);
}
