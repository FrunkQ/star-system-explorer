//! CM2: mounts joined up with the flight log (compound ships design 6.1 to
//! 6.3). A capture writes the carrier's `join` and the carried construct's
//! `mounted` piece; a release writes the carrier's `split` and seeds the
//! released construct from the compound root's state plus its push; a
//! capture checks reach and speed against both logs and offers a rendezvous.

use crate::Kernel;
use serde_json::{json, Value as J};

fn ask(k: &mut Kernel, v: J) -> J {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

const T0: i64 = 63_115_200_000; // Sol fixture's own epoch.
const CSM: &str = "u:sol/construct:columbia";
const LM: &str = "u:sol/construct:eagle";

/// A universe with base-nav on the stack (its `mounts` settings), Columbia
/// in a low circular orbit of Earth, and Eagle made beside it.
fn open() -> Kernel {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let packs = crate::contract::load_packs(&root.join("tests/fixtures/packs"));
    let nav: J = serde_json::from_str(
        &std::fs::read_to_string(root.join("packs/base-nav/pack.json")).unwrap(),
    )
    .unwrap();
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        json!({"op":"load_stack","packs":[packs["base-physics"], nav]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = ask(
        &mut k,
        json!({"op":"open_universe","id":"sol","seed":"compound logs"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    let r = ask(
        &mut k,
        json!({"op":"set_authored_batch","occurred_at_s":T0,"edits":[
            {"address":CSM,"field":"construct.mass_kg","value":28800.0},
            {"address":LM,"field":"construct.mass_kg","value":15100.0}]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let orbit = json!({"from_s": T0, "state": {"kind": "orbit", "around": "Earth",
        "elements": {"a_m": 6.778e6, "e": 0.0, "epoch_s": T0}}, "mark": "authored"});
    let r = ask(
        &mut k,
        json!({"op":"log_flight","construct":CSM,"piece":orbit,"at_s":T0}),
    );
    assert_eq!(r["type"], "flight_logged", "{r}");
    k
}

fn at(k: &mut Kernel, c: &str, t: i64) -> J {
    let r = ask(
        k,
        json!({"op":"flight_log","construct":c,"where_at":t,"fixture":"sol"}),
    );
    assert_eq!(r["type"], "flight_log_is", "{r}");
    r
}

fn v3(x: &J) -> [f64; 3] {
    let a = x.as_array().unwrap_or_else(|| panic!("no vector: {x}"));
    [
        a[0].as_f64().unwrap(),
        a[1].as_f64().unwrap(),
        a[2].as_f64().unwrap(),
    ]
}
fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[test]
fn a_capture_writes_both_logs_and_the_carried_craft_rides_its_carrier() {
    let mut k = open();
    let r = ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"front","fixture":"sol","at_s":T0+100}),
    );
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert_eq!(r["logged"].as_array().unwrap().len(), 2, "{r}");
    let e = at(&mut k, LM, T0 + 500);
    assert_eq!(e["log"]["pieces"][0]["state"]["kind"], "mounted", "{e}");
    assert_eq!(e["log"]["pieces"][0]["state"]["place"], "front");
    assert_eq!(e["at"]["kind"], "mounted");
    assert_eq!(e["at"]["with"], CSM);
    assert_eq!(e["at"]["words"], format!("carried at the front of {CSM}"));
    let c = at(&mut k, CSM, T0 + 500);
    assert_eq!(c["log"]["events"][0]["kind"], "join", "{c}");
    assert_eq!(c["log"]["events"][0]["with"], LM);
    assert_eq!(
        v3(&e["at"]["r_m"]),
        v3(&c["at"]["r_m"]),
        "where it is, is where its carrier is"
    );
}

#[test]
fn a_release_seeds_the_free_log_with_the_carriers_state_and_its_push() {
    let mut k = open();
    ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"front","at_s":T0+100}),
    );
    let t = T0 + 1000;
    let r = ask(
        &mut k,
        json!({"op":"release_construct","address":LM,"fixture":"sol","at_s":t}),
    );
    assert_eq!(r["type"], "construct_released", "{r}");
    assert_eq!(r["logged"].as_array().unwrap().len(), 2, "{r}");
    assert_eq!(r["notes"], json!([]), "the pack's push is used: {r}");
    let e = at(&mut k, LM, t);
    let c = at(&mut k, CSM, t);
    let p = e["log"]["pieces"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(p["state"]["kind"], "orbit", "{p}");
    assert_eq!(p["state"]["around"], "Earth");
    assert!(p["why"].as_str().unwrap().contains("pushed 0.5 m/s"), "{p}");
    assert_eq!(
        c["log"]["events"].as_array().unwrap().last().unwrap()["kind"],
        "split"
    );
    // At the release both are in the same place, moving 0.5 m/s apart along the flight.
    let (re, rc) = (v3(&e["at"]["r_m"]), v3(&c["at"]["r_m"]));
    let (ve, vc) = (v3(&e["at"]["v_m_s"]), v3(&c["at"]["v_m_s"]));
    assert!(dist(re, rc) < 0.01, "{} m apart", dist(re, rc));
    assert!((dist(ve, vc) - 0.5).abs() < 1e-6, "{} m/s", dist(ve, vc));
    let a = p["state"]["elements"]["a_m"].as_f64().unwrap();
    assert!(a > 6.778e6, "a prograde push raises the orbit: {a}");
    // An hour later they have drifted apart: a capture then is refused with a rendezvous offer.
    let r = ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"front","fixture":"sol","at_s":t+3600}),
    );
    assert_eq!(r["code"], "capture_apart", "{r}");
    let m = r["message"].as_str().unwrap();
    assert!(m.contains("a capture needs 10 m and 0.3 m/s"), "{m}");
    assert!(m.contains("a rendezvous leaving at"), "{m}");
    // The GM may still write it as authored, with a note of how far apart they were.
    let r = ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"front","fixture":"sol","authored":true,"at_s":t+3600}),
    );
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert!(
        r["notes"][0]
            .as_str()
            .unwrap()
            .starts_with("written as authored"),
        "{r}"
    );
    // Right after a release with no push they are within reach, so a capture passes.
    let r = ask(
        &mut k,
        json!({"op":"release_construct","address":LM,"push_m_s":0.0,"fixture":"sol","at_s":t+4000}),
    );
    assert_eq!(r["type"], "construct_released", "{r}");
    let r = ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"front","fixture":"sol","at_s":t+4001}),
    );
    assert_eq!(r["type"], "construct_mounted", "{r}");
}

#[test]
fn without_a_system_the_orbit_is_copied_and_the_push_noted() {
    let mut k = open();
    ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"behind","at_s":T0+100}),
    );
    let r = ask(
        &mut k,
        json!({"op":"release_construct","address":LM,"at_s":T0+1000}),
    );
    assert_eq!(r["type"], "construct_released", "{r}");
    assert_eq!(
        r["notes"],
        json!(["no system given, so the orbit is copied without the push"]),
        "{r}"
    );
    let e = at(&mut k, LM, T0 + 1000);
    let p = e["log"]["pieces"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(p["state"]["elements"]["a_m"], 6.778e6, "{p}");
}

#[test]
fn a_release_from_a_course_seeds_an_orbit_of_the_star() {
    let mut k = open();
    let course = json!({"from_s": T0 + 86_400, "state": {"kind": "course",
        "course": {"kind": "lambert", "to": "Mars", "arrive_s": T0 + 86_400 + 250 * 86_400}},
        "mark": "planned"});
    let r = ask(
        &mut k,
        json!({"op":"log_flight","construct":CSM,"piece":course,"at_s":T0}),
    );
    assert_eq!(r["type"], "flight_logged", "{r}");
    ask(
        &mut k,
        json!({"op":"mount_construct","address":LM,"on":CSM,"place":"inside","at_s":T0+100}),
    );
    let t = T0 + 30 * 86_400;
    let r = ask(
        &mut k,
        json!({"op":"release_construct","address":LM,"push_m_s":0.0,"fixture":"sol","at_s":t}),
    );
    assert_eq!(r["type"], "construct_released", "{r}");
    let e = at(&mut k, LM, t + 86_400);
    let c = at(&mut k, CSM, t + 86_400);
    let p = e["log"]["pieces"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(p["state"]["around"], "star", "{p}");
    // With no push it flies the carrier's arc: a day later they are still together.
    let d = dist(v3(&e["at"]["r_m"]), v3(&c["at"]["r_m"]));
    assert!(d < 100.0, "{d} m apart");
}

#[test]
fn with_no_flight_logs_nothing_is_written() {
    let mut k = open();
    let other = "u:sol/construct:tug";
    ask(
        &mut k,
        json!({"op":"set_authored_batch","occurred_at_s":T0,"edits":[
        {"address":other,"field":"construct.mass_kg","value":1000.0}]}),
    );
    let r = ask(
        &mut k,
        json!({"op":"mount_construct","address":other,"on":LM,"place":"behind","fixture":"sol","at_s":T0+100}),
    );
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert_eq!(r["logged"], json!([]));
    assert_eq!(r["notes"], json!([]), "{r}");
    let r = ask(
        &mut k,
        json!({"op":"release_construct","address":other,"at_s":T0+200}),
    );
    assert_eq!(r["logged"], json!([]), "{r}");
}
