//! SV1 checks (secrets and visibility design 11): a hidden thing and a
//! kept-back field are absent from every byte sent to a seat; a scoped
//! reveal is one undo; counts never include hidden things; news dated later
//! waits on the campaign clock.

fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

const SYS: &str = "u:kestrel/sys:0";
const TARN: &str = "u:kestrel/sys:0/body:3";
const BASE: &str = "u:kestrel/sys:0/body:3/body:0";
const T: &str = "everyone";

fn kernel() -> crate::Kernel {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let packs = crate::contract::load_packs(&root.join("tests/fixtures/packs"));
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
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            {"address": TARN, "field": "name", "value": "Tarn"},
            {"address": TARN, "field": "settlements.population", "value": 31337},
            {"address": BASE, "field": "name", "value": "Tarn Deep"},
            {"address": BASE, "field": "kind", "value": "base"}
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    k
}

fn set(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    let mut q = serde_json::json!({"op": "set_known", "audience": T, "at_s": 0});
    for (key, x) in v.as_object().unwrap() {
        q[key] = x.clone();
    }
    let r = send(k, q);
    assert_eq!(r["type"], "known_set", "{r}");
    r
}

fn known(k: &mut crate::Kernel, a: &str, field: Option<&str>, at: i64) -> serde_json::Value {
    let mut q = serde_json::json!({"op": "known_to", "address": a, "at_s": at});
    if let Some(f) = field {
        q["field"] = f.into();
    }
    let r = send(k, q);
    assert_eq!(r["type"], "known_to", "{r}");
    r["value"]["audiences"][0].clone()
}

/// Everything one seat of the table could be sent.
fn wire(k: &mut crate::Kernel, at: i64) -> String {
    let mut out = String::new();
    for q in [
        serde_json::json!({"op":"scene","audience":T,"scope":"u:kestrel","at_s":at}),
        serde_json::json!({"op":"disclosures","audience":T,"at_s":at}),
        serde_json::json!({"op":"read_names","address":BASE,"audience":T,"at_s":at}),
        serde_json::json!({"op":"read_names","address":TARN,"audience":T,"at_s":at}),
    ] {
        out.push_str(&send(k, q).to_string());
    }
    out
}

#[test]
fn hidden_things_and_kept_back_fields_never_reach_a_seat() {
    let mut k = kernel();
    set(
        &mut k,
        serde_json::json!({"address": TARN, "landmark": "known"}),
    );
    set(
        &mut k,
        serde_json::json!({"address": BASE, "landmark": "gm_only"}),
    );
    set(
        &mut k,
        serde_json::json!({"address": TARN, "field": "settlements.population", "landmark": "gm_only"}),
    );
    let t = known(&mut k, TARN, None, 0);
    assert_eq!(t["landmark"], "known", "{t}");
    assert_eq!(t["rung"], "survey");
    assert_eq!(t["top"], true);
    assert_eq!(t["label"], "The table");
    assert_eq!(
        t["exceptions"],
        serde_json::json!([{"field": "settlements.population", "landmark": "gm_only"}])
    );
    assert_eq!(
        t["withheld"],
        serde_json::json!(["settlements.population"]),
        "{t}"
    );
    let p = known(&mut k, TARN, Some("settlements.population"), 0);
    assert_eq!(p["landmark"], "gm_only");
    let n = known(&mut k, TARN, Some("name"), 0);
    assert_eq!(n["landmark"], "known");
    let b = known(&mut k, BASE, None, 0);
    assert_eq!(b["landmark"], "gm_only", "{b}");
    assert_eq!(b["rung_label"], "GM only");
    // Not one byte of either secret goes to the table.
    let w = wire(&mut k, 0);
    assert!(w.contains("Tarn"), "Tarn itself is known");
    assert!(!w.contains("Tarn Deep"), "{w}");
    assert!(
        !w.contains("31337") && !w.contains("settlements.population"),
        "{w}"
    );
    // The GM still sees both.
    let gm = send(
        &mut k,
        serde_json::json!({"op":"scene","audience":"gm","scope":"u:kestrel","at_s":0}),
    )
    .to_string();
    assert!(gm.contains("Tarn Deep") && gm.contains("31337"));
    // Heard of on a field: the line is there, its value is not.
    set(
        &mut k,
        serde_json::json!({"address": TARN, "field": "settlements.population", "landmark": "heard_of"}),
    );
    let w = wire(&mut k, 0);
    assert!(
        w.contains("settlements.population") && !w.contains("31337"),
        "{w}"
    );
    // A null landmark drops the exception: the field follows Tarn again.
    set(
        &mut k,
        serde_json::json!({"address": TARN, "field": "settlements.population", "landmark": null}),
    );
    assert!(wire(&mut k, 0).contains("31337"));
    assert_eq!(
        known(&mut k, TARN, None, 0)["exceptions"],
        serde_json::json!([])
    );
}

#[test]
fn a_scoped_reveal_is_one_undo_and_counts_never_include_hidden_things() {
    let mut k = kernel();
    set(
        &mut k,
        serde_json::json!({"address": BASE, "landmark": "gm_only"}),
    );
    let c = send(
        &mut k,
        serde_json::json!({"op":"known_in","bounds":SYS,"at_s":0}),
    );
    assert_eq!(c["type"], "known_counted", "{c}");
    let row = &c["audiences"][0];
    let seen = send(
        &mut k,
        serde_json::json!({"op":"scene","audience":T,"scope":SYS,"at_s":0}),
    )["scene"]["subjects"]
        .as_array()
        .unwrap()
        .len() as u64;
    assert_eq!(row["gm_only"], 1, "{c}");
    assert_eq!(
        row["known"].as_u64().unwrap() + row["heard_of"].as_u64().unwrap(),
        seen,
        "{c}"
    );
    // The system and everything in it, in one event.
    let r = set(
        &mut k,
        serde_json::json!({"address": SYS, "landmark": "known", "scope": "children"}),
    );
    assert_eq!(r["changed"], 2, "{r}");
    assert_eq!(known(&mut k, BASE, None, 0)["landmark"], "known");
    assert!(wire(&mut k, 0).contains("Tarn Deep"));
    // One undo puts both back.
    let u = send(
        &mut k,
        serde_json::json!({"op":"undo_known","undo":r["undo"],"at_s":0}),
    );
    assert_eq!(u["type"], "known_set", "{u}");
    assert_eq!(known(&mut k, BASE, None, 0)["landmark"], "gm_only");
    assert_eq!(known(&mut k, TARN, None, 0)["landmark"], "heard_of");
    assert!(!wire(&mut k, 0).contains("Tarn Deep"));
    // The same landmark again changes nothing and writes nothing.
    let r = set(
        &mut k,
        serde_json::json!({"address": BASE, "landmark": "gm_only"}),
    );
    assert_eq!(r["changed"], 0);
    assert!(r.get("undo").is_none());
    // A list.
    let r = set(
        &mut k,
        serde_json::json!({"address": TARN, "landmark": "known", "scope": {"list": [BASE]}}),
    );
    assert_eq!(r["changed"], 2, "{r}");
    // Bad asks are refused.
    for (q, code) in [
        (
            serde_json::json!({"address": TARN, "landmark": "seen"}),
            "bad_landmark",
        ),
        (
            serde_json::json!({"address": TARN, "landmark": "known", "scope": "near"}),
            "bad_scope",
        ),
        (
            serde_json::json!({"address": "u:other/sys:0", "landmark": "known"}),
            "bad_address",
        ),
        (
            serde_json::json!({"address": TARN, "landmark": "known", "audience": "gm"}),
            "bad_audience",
        ),
        (
            serde_json::json!({"address": TARN, "landmark": "known", "field": "observe.view"}),
            "bad_field",
        ),
    ] {
        let mut full = serde_json::json!({"op": "set_known", "audience": T, "at_s": 0});
        for (key, x) in q.as_object().unwrap() {
            full[key] = x.clone();
        }
        assert_eq!(send(&mut k, full)["code"], code);
    }
}

#[test]
fn news_dated_later_waits_and_an_undo_before_it_means_it_never_arrives() {
    let mut k = kernel();
    set(
        &mut k,
        serde_json::json!({"address": BASE, "landmark": "gm_only"}),
    );
    let r = set(
        &mut k,
        serde_json::json!({"address": BASE, "landmark": "heard_of", "from_s": 1000}),
    );
    let b = known(&mut k, BASE, None, 0);
    assert_eq!(b["landmark"], "gm_only");
    assert_eq!(b["from_s"], 1000, "{b}");
    assert!(!wire(&mut k, 999).contains("Tarn Deep"));
    assert_eq!(known(&mut k, BASE, None, 1000)["landmark"], "heard_of");
    assert!(wire(&mut k, 1000).contains("Tarn Deep"));
    // Undone at 10, before it arrives: it never does.
    send(
        &mut k,
        serde_json::json!({"op":"undo_known","undo":r["undo"],"at_s":10}),
    );
    assert_eq!(known(&mut k, BASE, None, 2000)["landmark"], "gm_only");
    assert!(!wire(&mut k, 2000).contains("Tarn Deep"));
}

/// The chart room's shared fixture (SV2): every key it reads is in the
/// kernel's answer, and every request it sends is taken. Its Kestrel is the
/// test drive's (system 1); here Tarn is system 0's.
#[test]
fn the_shared_fixture_is_what_the_kernel_answers_and_takes() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fx: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("tests/fixtures/sv/known-to.json")).unwrap(),
    )
    .unwrap();
    let here = |v: &serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&v.to_string().replace("u:kestrel/sys:1", SYS)).unwrap()
    };
    let mut k = kernel();
    for a in fx["answers"].as_array().unwrap() {
        let r = send(&mut k, here(&a["ask"]));
        assert_eq!(r["type"], "known_to", "{r}");
        let want = &a["value"]["audiences"][0];
        let got = &r["value"]["audiences"][0];
        for key in want.as_object().unwrap().keys() {
            assert!(got.get(key).is_some(), "known_to lacks {key}: {got}");
        }
        for key in a["value"].as_object().unwrap().keys() {
            assert!(r["value"].get(key).is_some(), "known_to lacks {key}");
        }
    }
    for c in fx["changes"].as_array().unwrap() {
        let r = send(&mut k, here(&c["request"]));
        assert_eq!(r["type"], "known_set", "{}: {r}", c["about"]);
    }
}
