//! `formation_record` (the Formation screen, F10): a grown system's history read back by re-running the request its
//! adoption kept; an imported or authored system answers with no record and why. Read-only.

fn ask(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

#[test]
fn a_grown_system_gives_back_the_run_it_was_adopted_from() {
    let mut k = crate::Kernel::new();
    let r = ask(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"fr","seed":"fr"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    let req = serde_json::json!({"seed":"first light","system":"u:fr/sys:7"});
    let grown = ask(
        &mut k,
        serde_json::json!({"op":"run_formation","seed":"first light","system":"u:fr/sys:7"}),
    );
    assert_eq!(grown["type"], "formation_ran");
    let a = ask(&mut k, serde_json::json!({"op":"adopt","formation":req}));
    assert_eq!(a["type"], "adopted", "{a}");
    let hash = ask(&mut k, serde_json::json!({"op":"state_hash"}));

    let f = ask(
        &mut k,
        serde_json::json!({"op":"formation_record","system":"u:fr/sys:7"}),
    );
    assert_eq!(f["type"], "formation_record", "{f}");
    assert_eq!(f["source"], "formation");
    assert_eq!(f["request"]["seed"], "first light");
    assert!(f.get("why").is_none());
    // The same run as growing it fresh: the same worlds, the same history.
    assert_eq!(f["record"]["type"], "formation_ran");
    assert_eq!(
        f["record"]["run"], grown["run"],
        "the re-run is the run it was adopted from"
    );
    assert!(f["adopted_manifest"].is_string());
    // Reading writes nothing.
    assert_eq!(ask(&mut k, serde_json::json!({"op":"state_hash"})), hash);
}

#[test]
fn an_imported_or_unknown_system_says_why_it_has_no_record() {
    let mut k = crate::Kernel::new();
    ask(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"fr","seed":"fr"}),
    );
    let save = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/v3/synthetic-sol.json"
    ))
    .unwrap();
    let a = ask(&mut k, serde_json::json!({"op":"adopt","import_v3":save}));
    assert_eq!(a["type"], "adopted", "{a}");
    let sys = a["systems"][0].as_str().unwrap().to_string();
    let f = ask(
        &mut k,
        serde_json::json!({"op":"formation_record","system":sys}),
    );
    assert_eq!(f["type"], "formation_record", "{f}");
    assert_eq!(f["source"], "v3");
    assert!(f.get("record").is_none());
    assert!(f["why"].as_str().unwrap().contains("SSE 3 save"));
    let none = ask(
        &mut k,
        serde_json::json!({"op":"formation_record","system":"u:fr/sys:99"}),
    );
    assert_eq!(none["code"], "unknown_system", "{none}");
    let bad = ask(
        &mut k,
        serde_json::json!({"op":"formation_record","system":"u:elsewhere/sys:0"}),
    );
    assert_eq!(bad["code"], "bad_address", "{bad}");
}
