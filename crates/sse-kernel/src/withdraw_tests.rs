//! Paste support for the Build desk: one-event batches, withdraw and restore.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

fn kernel() -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["tarn-observe"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    k
}

fn subjects(k: &mut crate::Kernel, at: i64) -> Vec<String> {
    let s = send(
        k,
        serde_json::json!({"op":"scene","audience":"gm","scope":"u:kestrel","at_s":at}),
    );
    s["scene"]["subjects"]
        .as_array()
        .unwrap_or_else(|| panic!("{s}"))
        .iter()
        .map(|x| x["address"].as_str().unwrap().to_string())
        .collect()
}

const B3: &str = "u:kestrel/sys:0/body:3";
const B4: &str = "u:kestrel/sys:0/body:4";
const M1: &str = "u:kestrel/sys:0/body:4/moon:1";

#[test]
fn a_paste_is_one_event_and_undo_and_redo_keep_history() {
    let mut k = kernel();
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            {"address": B3, "field": "name", "value": "Tarn"},
            {"address": B4, "field": "name", "value": "Kell"},
            {"address": B4, "field": "pressure_bar", "value": 2.0},
            {"address": M1, "field": "name", "value": "Kell a"}
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    assert_eq!(r["affected"].as_array().unwrap().len(), 3, "{r}");
    let h = send(
        &mut k,
        serde_json::json!({"op":"history","address":"u:kestrel"}),
    );
    assert_eq!(h["events"].as_array().unwrap().len(), 1, "one event: {h}");
    let all = subjects(&mut k, 10);
    assert!(
        all.contains(&B4.to_string()) && all.contains(&M1.to_string()),
        "{all:?}"
    );
    // Undo: Kell and its moon leave the scene from t = 5 on, not before.
    let w = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":B4,"at_s":5}),
    );
    assert_eq!(w["type"], "authored", "{w}");
    let after = subjects(&mut k, 10);
    assert_eq!(after, vec![B3.to_string()], "{after:?}");
    assert_eq!(subjects(&mut k, 0), all);
    let sys = send(&mut k, serde_json::json!({"op":"systems"}));
    assert_eq!(sys["systems"][0]["addresses"], 1, "{sys}");
    // Redo: back with every fact it had.
    send(
        &mut k,
        serde_json::json!({"op":"restore","address":B4,"at_s":7}),
    );
    assert_eq!(subjects(&mut k, 10), all);
    let h = send(
        &mut k,
        serde_json::json!({"op":"history","address":"u:kestrel"}),
    );
    assert_eq!(
        h["events"].as_array().unwrap().len(),
        3,
        "nothing deleted: {h}"
    );
    // A whole system withdrawn leaves the systems list.
    send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":"u:kestrel/sys:0","at_s":8}),
    );
    let sys = send(&mut k, serde_json::json!({"op":"systems"}));
    assert_eq!(sys["systems"], serde_json::json!([]), "{sys}");
}

#[test]
fn a_bad_edit_refuses_the_whole_batch() {
    let mut k = kernel();
    let h0 = send(
        &mut k,
        serde_json::json!({"op":"state_hash","at_s":i64::MAX}),
    );
    for edits in [
        serde_json::json!([{"address": B3, "field": "name", "value": "Tarn"},
                           {"address": "u:elsewhere/sys:0", "field": "name", "value": "X"}]),
        serde_json::json!([{"address": B3, "field": "name", "value": "Tarn"},
                           {"address": B4, "field": "name", "value": "Y", "impossible": true}]),
        serde_json::json!([]),
    ] {
        let r = send(
            &mut k,
            serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":edits}),
        );
        assert_eq!(r["type"], "error", "{r}");
    }
    let h1 = send(
        &mut k,
        serde_json::json!({"op":"state_hash","at_s":i64::MAX}),
    );
    assert_eq!(h0, h1);
    // Nothing to withdraw where nothing is.
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":B4,"at_s":0}),
    );
    assert_eq!(r["type"], "error", "{r}");
}

/// The start sheet's Sol enters as one catalogued batch with its citation,
/// and resolved_system stamps each fact catalogued with that source.
#[test]
fn a_catalogued_batch_carries_its_source() {
    let mut k = kernel();
    let src = "NASA Planetary Fact Sheet, https://nssdc.gsfc.nasa.gov/planetary/factsheet/";
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "evidence":"catalogued","source":src,
            "edits":[{"address":B3,"field":"name","value":"Earth"},
                     {"address":B3,"field":"mass_kg","value":5.972e24}]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":"u:kestrel/sys:0"}),
    );
    let who = &r["evidence"][B3];
    for f in ["name", "mass_kg"] {
        assert_eq!(who[f]["evidence"], "catalogued", "{r}");
        assert_eq!(who[f]["source"], src, "{r}");
    }
    // A plain batch stays authored, with no source.
    send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "edits":[{"address":B3,"field":"name","value":"Tarn"}]}),
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":"u:kestrel/sys:0"}),
    );
    assert_eq!(r["evidence"][B3]["name"]["evidence"], "authored", "{r}");
    assert!(r["evidence"][B3]["name"].get("source").is_none(), "{r}");
    assert_eq!(
        r["evidence"][B3]["mass_kg"]["evidence"], "catalogued",
        "{r}"
    );
    // Catalogued and impossible do not mix; unknown labels and blank sources are refused.
    for extra in [
        serde_json::json!({"evidence":"catalogued","edits":[{"address":B3,"field":"x","value":1,"impossible":true}]}),
        serde_json::json!({"evidence":"resolved","edits":[{"address":B3,"field":"x","value":1}]}),
        serde_json::json!({"source":" ","edits":[{"address":B3,"field":"x","value":1}]}),
    ] {
        let mut req = extra.clone();
        req["op"] = "set_authored_batch".into();
        req["occurred_at_s"] = 0.into();
        let r = send(&mut k, req);
        assert_eq!(r["type"], "error", "{extra} {r}");
    }
}

#[test]
fn a_history_can_name_what_each_edit_changed_at_one_address() {
    let mut k = kernel();
    send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            {"address": B4, "field": "pressure_bar", "value": 2.0},
            {"address": B4, "field": "picture", "value": "x".repeat(5000)},
            {"address": M1, "field": "name", "value": "Kell a"}
        ]}),
    );
    // Plain history is unchanged: no changes asked, none given.
    let plain = send(&mut k, serde_json::json!({"op":"history","address":B4}));
    assert!(plain["events"][0].get("changes").is_none(), "{plain}");
    let h = send(
        &mut k,
        serde_json::json!({"op":"history","address":B4,"changes":true}),
    );
    let c = h["events"][0]["changes"].as_array().unwrap().clone();
    // Only B4's own facts (not its moon's); a picture-sized value is named but not sent.
    assert_eq!(c.len(), 2, "{h}");
    let p = c.iter().find(|x| x["field"] == "pressure_bar").unwrap();
    assert_eq!(p["value"], 2.0);
    let pic = c.iter().find(|x| x["field"] == "picture").unwrap();
    assert_eq!(pic["large"], true, "{pic}");
    assert!(pic["value"].is_null());
}
