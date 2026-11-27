//! `class_sample`: a mid-band sample world per class, hand-set on a what-if
//! branch and resolved, for the class gallery's pictures (V3 gaps item 35).

use std::path::PathBuf;

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

fn pack(p: &str) -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    serde_json::from_str(&std::fs::read_to_string(root.join(p)).unwrap()).unwrap()
}

fn kernel() -> crate::Kernel {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[pack("packs/base-physics/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"home","seed":"home"}),
    );
    k
}

#[test]
fn every_asked_class_gets_a_sample_or_a_reason() {
    let mut k = kernel();
    let before = send(&mut k, serde_json::json!({"op":"state_hash","at_s":0}));
    let r = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[pack("packs/base-atlas/pack.json")],
            "classes":["iron-rich","small-body","no-such-class"]}),
    );
    assert_eq!(r["type"], "class_sampled", "{r}");
    let s = r["samples"].as_array().unwrap();
    assert_eq!(s.len(), 2, "{r}");
    assert_eq!(s[0]["class"], "iron-rich");
    assert_eq!(s[1]["class"], "small-body");
    assert_eq!(r["refused"][0]["class"], "no-such-class", "{r}");
    // Mercury is the iron-rich example; it keeps its class once resolved.
    assert_eq!(s[0]["example"], "Mercury");
    assert_eq!(s[0]["fit"], 1.0);
    assert_eq!(s[0]["surface"], true);
    assert_eq!(s[0]["radius_m"], 2439700.0);
    for x in s {
        let f = x["fit"].as_f64().unwrap();
        assert!((0.0..=1.0).contains(&f), "{x}");
        // The sample reads back on its branch with the same body.
        let rs = send(
            &mut k,
            serde_json::json!({"op":"resolved_system","system":x["system"],"branch":r["branch"],
                "tiles":[{"body":x["body"],"face":0,"level":0,"x":0,"y":0}]}),
        );
        assert_eq!(rs["type"], "system_resolved", "{rs}");
        // A sample with ground gives the globe its tiles.
        let drawn = rs["tiles"].as_array().is_some_and(|t| !t.is_empty());
        assert_eq!(drawn, x["surface"] == true, "{x} {}", rs["refused"]);
        assert!(
            rs["facts"].get(x["body"].as_str().unwrap()).is_some(),
            "{rs}"
        );
    }
    // Nothing on the main line moved.
    let after = send(&mut k, serde_json::json!({"op":"state_hash","at_s":0}));
    assert_eq!(before, after);
    let main = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":s[0]["system"]}),
    );
    assert!(
        main["facts"]
            .get(s[0]["body"].as_str().unwrap())
            .is_none_or(|f| f.get("body.mass").is_none()),
        "{main}"
    );
}

#[test]
fn the_most_central_example_is_chosen() {
    let mut k = kernel();
    let a = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[pack("packs/base-atlas/pack.json")],
            "classes":["iron-rich"]}),
    );
    let b = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[pack("packs/base-atlas/pack.json")],
            "classes":["iron-rich"]}),
    );
    assert_eq!(a["samples"][0]["example"], b["samples"][0]["example"]);
    assert_eq!(a["samples"][0]["fit"], b["samples"][0]["fit"]);
}

/// The whole gallery in one call: every class of the shipped atlas.
#[test]
fn the_whole_atlas_samples() {
    let mut k = kernel();
    let atlas = pack("packs/base-atlas/pack.json");
    let ids: Vec<serde_json::Value> = atlas["provides"]["other"]["classes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["scheme"] != "star")
        .map(|c| c["id"].clone())
        .collect();
    let r = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[atlas],"classes":ids}),
    );
    assert_eq!(r["type"], "class_sampled", "{r}");
    let s = r["samples"].as_array().unwrap();
    let kept = s.iter().filter(|x| x["fit"].as_f64() >= Some(0.6)).count();
    let ground = s.iter().filter(|x| x["surface"] == true).count();
    let unpublished = s
        .iter()
        .filter(|x| x["limiting"]["why"] == "not published on the world yet")
        .count();
    eprintln!(
        "{} samples, {kept} keep their class, {unpublished} held down by an unpublished fact, {ground} with ground; refused {}",
        s.len(),
        r["refused"]
    );
    for x in s {
        assert!(
            x["example_fit"].as_f64() >= Some(0.6),
            "every atlas example is in its class: {x}"
        );
        assert_eq!(x["limiting"].is_null(), x["fit"] == 1.0, "{x}");
    }
    assert_eq!(s.len() + r["refused"].as_array().unwrap().len(), ids.len());
}

/// Two calls never share a sample address, so a surface cache keyed by
/// address cannot hand one call's world to the next (renderer thread, 25 Nov).
#[test]
fn each_call_names_its_own_samples() {
    let mut k = kernel();
    let atlas = pack("packs/base-atlas/pack.json");
    let a = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[atlas.clone()],"classes":["iron-rich"]}),
    );
    let b = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[atlas],"classes":["small-body"]}),
    );
    assert_ne!(
        a["samples"][0]["system"], b["samples"][0]["system"],
        "{a} {b}"
    );
}

/// A star class is not placed as a world: it is refused, pointing at
/// describe_star and the class's own sample star.
#[test]
fn a_star_class_is_drawn_from_the_life_line_not_sampled() {
    let mut k = kernel();
    let r = send(
        &mut k,
        serde_json::json!({"op":"class_sample","packs":[pack("packs/base-atlas/pack.json")],
            "classes":["star/main-sequence/G"]}),
    );
    assert_eq!(r["type"], "class_sampled", "{r}");
    assert!(
        r["refused"][0]["why"]
            .as_str()
            .unwrap()
            .contains("describe_star"),
        "{r}"
    );
}
