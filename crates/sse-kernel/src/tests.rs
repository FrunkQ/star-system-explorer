use crate::protocol::*;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn contract_lines() -> Vec<String> {
    let reqs: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("tests/contract/requests.json")).unwrap(),
    )
    .unwrap();
    crate::contract::run(
        &reqs,
        &crate::contract::load_packs(&root().join("tests/fixtures/packs")),
    )
}

/// Native half of the contract test. Set SSE_BLESS=1 to rewrite the golden.
#[test]
fn contract_matches_golden() {
    let lines = contract_lines();
    let path = root().join("tests/golden/contract.jsonl");
    let text = lines.join("\n") + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
    }
    let golden = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1 once");
    for (i, (a, b)) in lines.iter().zip(golden.lines()).enumerate() {
        assert_eq!(a, b, "response {i} differs");
    }
    assert_eq!(lines.len(), golden.lines().count());
}

#[test]
fn every_op_is_covered_by_the_contract() {
    let reqs: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("tests/contract/requests.json")).unwrap(),
    )
    .unwrap();
    let used: std::collections::BTreeSet<String> = reqs
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["op"].as_str().unwrap().to_string())
        .collect();
    for o in ops() {
        assert!(used.contains(&o.op), "op {} has no contract case", o.op);
    }
}

#[test]
fn contract_behaviours() {
    let l: Vec<serde_json::Value> = contract_lines()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let ty = |i: usize| l[i]["type"].as_str().unwrap().to_string();
    assert_eq!(ty(1), "error"); // derive before a stack
    assert_eq!(ty(2), "stack_rejected");
    assert!(l[2]["report"]
        .as_str()
        .unwrap()
        .contains("only one publisher"));
    assert_eq!(
        l[3]["fictional_kinds"],
        serde_json::json!(["planet.force_temperature"])
    );
    let ts = &l[4]["values"]["planet.surface_temperature"];
    assert!((ts["value"].as_f64().unwrap() - 286.3).abs() < 0.5 && ts["fictional"] == false);
    assert_eq!(
        l[5]["values"]["planet.force_temperature"]["fictional"],
        true
    );
    assert_eq!(ty(7), "explanation"); // node.port also explains
    assert_eq!(l[11]["code"], "bad_stream");
    assert_eq!(l[15]["code"], "bad_address"); // other universe
    assert_eq!(l[16]["events"].as_array().unwrap().len(), 2);
    assert_ne!(l[17]["hash"], l[18]["hash"]);
    assert_eq!(
        l[24]["hash"], l[18]["hash"],
        "branch then discard leaves state unchanged"
    );
    assert_ne!(l[29]["hash"], l[18]["hash"], "commit changes state");
    assert_eq!(l[28]["code"], "unknown_branch", "committed branch is gone");
    assert_eq!(ty(33), "job_done");
    assert_eq!(l[33]["done_units"], 5000);
    assert_eq!(l[34]["code"], "unknown_job", "a finished job is gone");
    assert_eq!(ty(37), "job_cancelled");
    assert_eq!(l[38]["code"], "unknown_job");
    assert_eq!(l[40]["code"], "bad_request");
    // A stack change in an open universe is chronicled (U3 gap 1).
    let adopted = &l[41]["adopted"];
    assert!(adopted["event_id"].is_string(), "{}", l[41]);
    assert_eq!(l[44]["manifest_id"], adopted["manifest_id"]);
    let packs = l[44]["manifest"]["packs"].as_array().unwrap();
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0]["id"], "sse.base-physics");
    assert!(
        l[45].get("adopted").is_none(),
        "same stack again records nothing"
    );
    let kinds: Vec<&str> = l[46]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds.iter().filter(|k| **k == "stack.adopt").count(), 1);
    assert_ne!(l[47]["adopted"]["state_hash"], adopted["state_hash"]);
    assert_eq!(l[48]["manifest"]["packs"][0]["id"], "sse.base-toy");
    // Scenes through the API (V1 request): Tarn before and after one tap of
    // Scanned are exactly the K9 fixtures the player views build against.
    let fixture = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join("tests/fixtures/observe").join(n)).unwrap(),
        )
        .unwrap()
    };
    // Compared as JavaScript sees them: a JS client cannot send 281.0, only
    // 281, so numbers are compared by value (see packages/K6-additions.md).
    assert_eq!(as_js(&l[89]["scene"]), as_js(&fixture("tarn-before.json")));
    assert_eq!(l[90]["view"], "scanned");
    assert_eq!(as_js(&l[91]["scene"]), as_js(&fixture("tarn-after.json")));
    let gm = l[92]["scene"]["subjects"].to_string();
    assert!(
        gm.contains("Pirate cache") && gm.contains("ancient-ruins"),
        "the GM sees secrets"
    );
    assert!(
        !l[91]["scene"].to_string().contains("Pirate cache"),
        "players never do"
    );
    assert!(l[93]["scene"]["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|s| s["view"] == "survey"));
    assert_eq!(l[94]["view"], "long-range");
    assert_eq!(l[95], l[89], "a second tap undoes");
    assert_eq!(l[96]["code"], "bad_data_view");
    assert_eq!(l[97]["code"], "bad_address");
    // Draft observe setup (preview before Apply): a different scene, and
    // nothing recorded, so the stack's scene and the state hash are unchanged.
    assert!(l[99]["scene"]["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["view"] == "scanned"));
    assert_ne!(l[99], l[95]);
    assert_eq!(l[100], l[95]);
    assert_eq!(l[101], l[98]);
    assert_eq!(l[102]["code"], "bad_observe");
    // Atom descriptions (U3 gap 2): ports with kinds and readable dimensions, parameters with ranges.
    let atom = &l[103]["atoms"][0];
    assert_eq!(atom["id"], "toy.luminosity_from_mass");
    assert_eq!(atom["outputs"][0]["kind"], "star.luminosity");
    assert!(atom["params"][0]["range"].is_array() && atom["params"][0]["description"].is_string());
    assert_eq!(l[104]["code"], "unknown_atom");
    // Flow tests at load_stack (gap 5) and readable validation text (gap 3).
    assert!(l[105]["issues"][0]
        .as_str()
        .unwrap()
        .contains("test sun and earth: planet.surface_temperature"));
    let range = l[106]["issues"][0].as_str().unwrap();
    assert!(
        range.contains("lum.exponent: value 99 is outside the allowed range 1 to 6")
            && !range.contains("Some(")
    );
    // Explain a node that publishes nothing (gap 4), by node.port.
    assert_eq!(l[108]["type"], "explanation");
    assert_eq!(l[109]["code"], "unknown_kind");
}

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

/// U3 gap 1: adopting a stack after open_universe moves the state hash and
/// the manifest, and the same session replays to the same ids on a fresh
/// kernel; a rejected stack records nothing.
#[test]
fn stack_changes_are_chronicled_and_replay() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let session = |k: &mut crate::Kernel| -> Vec<serde_json::Value> {
        let mut out = vec![];
        out.push(send(
            k,
            serde_json::json!({"op":"load_stack","packs":[packs["base-toy"]]}),
        ));
        out.push(send(
            k,
            serde_json::json!({"op":"open_universe","id":"rp","seed":"replay"}),
        ));
        out.push(send(k, serde_json::json!({"op":"state_hash","at_s":0})));
        out.push(send(k, serde_json::json!({"op":"manifest"})));
        out.push(send(
            k,
            serde_json::json!({"op":"load_stack","packs":[packs["base-physics"]]}),
        ));
        out.push(send(k, serde_json::json!({"op":"state_hash","at_s":0})));
        out.push(send(k, serde_json::json!({"op":"manifest"})));
        out.push(send(k, serde_json::json!({"op":"load_stack","packs":[packs["acme-force"], packs["acme-rival"]]})));
        out.push(send(k, serde_json::json!({"op":"manifest"})));
        out
    };
    let a = session(&mut crate::Kernel::new());
    let b = session(&mut crate::Kernel::new());
    assert_eq!(a, b, "the same session replays identically");
    assert_eq!(
        a[0].get("adopted"),
        None,
        "no universe yet: nothing to record"
    );
    assert_ne!(
        a[2]["hash"], a[5]["hash"],
        "adopting physics moves the state hash"
    );
    assert_ne!(a[3]["manifest_id"], a[6]["manifest_id"]);
    assert_eq!(a[6]["manifest_id"], a[4]["adopted"]["manifest_id"]);
    assert_eq!(a[6]["manifest"]["packs"][0]["id"], "sse.base-physics");
    assert_eq!(a[7]["type"], "stack_rejected");
    assert_eq!(a[8], a[6], "a rejected stack changes nothing");
}

/// The contract loads the real base-physics pack from a fixture copy (both
/// runners read `tests/fixtures/packs/`). The copy must be the pack itself.
/// After re-blessing the pack: `cp packs/base-physics/pack.json tests/fixtures/packs/base-physics.json`.
#[test]
fn base_physics_fixture_is_the_real_pack() {
    let real = std::fs::read_to_string(root().join("packs/base-physics/pack.json")).unwrap();
    let copy =
        std::fs::read_to_string(root().join("tests/fixtures/packs/base-physics.json")).unwrap();
    assert!(
        real == copy,
        "tests/fixtures/packs/base-physics.json is stale: cp packs/base-physics/pack.json tests/fixtures/packs/base-physics.json"
    );
}

/// Tags in scenes also come from the stack's rule pack (K8), not only from
/// authored `tag.*` facts.
#[test]
fn scene_tags_come_from_stack_rules() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let rules: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("tests/fixtures/sift/rules-pack.json")).unwrap(),
    )
    .unwrap();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["tarn-observe"], rules]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    for (f, v) in [
        ("surface_temp_k", serde_json::json!(290.0)),
        ("pressure_bar", serde_json::json!(1.0)),
        ("has_ocean", serde_json::json!(true)),
        ("name", serde_json::json!("Tarn")),
    ] {
        let e = serde_json::json!({"op":"set_authored","edit":{"address":"u:kestrel/sys:0/body:3","field":f,"value":v,"occurred_at_s":0}});
        assert_eq!(send(&mut k, e)["type"], "authored");
    }
    let s = send(
        &mut k,
        serde_json::json!({"op":"scene","audience":"gm","scope":"u:kestrel","at_s":0}),
    );
    assert_eq!(
        s["scene"]["subjects"][0]["tags"],
        serde_json::json!(["habitable"]),
        "{s}"
    );
}

/// A JSON value with every number made an f64, as JavaScript holds it.
fn as_js(v: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        J::Number(n) => serde_json::json!(n.as_f64().unwrap()),
        J::Array(a) => J::Array(a.iter().map(as_js).collect()),
        J::Object(o) => J::Object(o.iter().map(|(k, x)| (k.clone(), as_js(x))).collect()),
        other => other.clone(),
    }
}

/// Cross-track number rule (packages/numbers.md): the same value written as
/// JavaScript writes it ("281") or as Rust and Python write it ("281.0",
/// "2.81e2") is one fact with one event id and one state hash; a float
/// stays a float; and an inexact-parse case ("1.2e27") is exact.
#[test]
fn number_spellings_hash_identically() {
    let run = |text: &str| -> (String, String) {
        let mut k = crate::Kernel::new();
        k.handle_json(r#"{"op":"open_universe","id":"n","seed":"numbers"}"#);
        let r: serde_json::Value = serde_json::from_str(&k.handle_json(&format!(
            r#"{{"op":"set_authored","edit":{{"address":"u:n/sys:0","field":"x","value":{text},"occurred_at_s":0}}}}"#
        )))
        .unwrap();
        (
            r["event_id"].as_str().unwrap().into(),
            r["state_hash"].as_str().unwrap().into(),
        )
    };
    let int = run("281");
    // Pinned: packages/kernel-client/test/contract.test.ts expects the same
    // state hash through the worker.
    assert_eq!(
        int.1,
        "c0b8445be888aeca4bbf63634feec6907958eafb1b4155030629cc71442d3dee"
    );
    for t in ["281.0", "2.81e2", "2.810E+2"] {
        assert_eq!(run(t), int, "{t}");
    }
    assert_ne!(run("281.5"), int);
    assert_eq!(run("1.2e27"), run("1.20e+27"));
    assert_eq!(run("1.2e27"), run("1200000000000000000000000000"));
}

/// Numbers change (cross-track): a whole-valued physical input, however it is
/// spelled, reaches the atoms as the same float, through flow inputs and
/// through pack flow parameters.
#[test]
fn whole_valued_physical_inputs_reach_the_atoms() {
    let pack_text = std::fs::read_to_string(root().join("packs/base-physics/pack.json")).unwrap();
    let derive = |pack: &str, req: &str| -> String {
        let mut k = crate::Kernel::new();
        let r = k.handle_json(&format!(r#"{{"op":"load_stack","packs":[{pack}]}}"#));
        assert!(r.contains("stack_loaded"), "{r}");
        k.handle_json(req)
    };
    let bulk = |m: &str, r: &str| {
        format!(
            r#"{{"op":"derive","flow":"sse.base.planet-bulk","inputs":{{"mass":{{"value":{m},"unit":"m_earth"}},"radius":{{"value":{r},"unit":"km"}}}}}}"#
        )
    };
    let a = derive(&pack_text, &bulk("1", "6371"));
    assert!(a.contains("body.surface_gravity"), "{a}");
    for (m, r) in [("1.0", "6371.0"), ("1e0", "6.371e3"), ("1.0E+0", "6371")] {
        assert_eq!(derive(&pack_text, &bulk(m, r)), a, "{m} {r}");
    }
    // The pack itself with its whole-valued floats written as JavaScript writes
    // them (1.0 as 1) is the same stack: same manifest, every golden case passes.
    let js = pack_text.replace("\"value\": 1.0", "\"value\": 1");
    assert_ne!(js, pack_text);
    let manifest = |pack: &str| derive(pack, r#"{"op":"manifest"}"#);
    assert_eq!(manifest(&js), manifest(&pack_text));
    let pack = sse_pack::PackManifest::from_json(&js).unwrap();
    let stack = sse_pack::resolve(&[pack], "0.1.0").unwrap();
    let atoms = crate::kernel_atoms();
    let cx = sse_flow::Context {
        atoms: &atoms,
        kinds: &stack.kinds,
        units: &stack.units,
    };
    for f in stack.flows.values() {
        let flow = sse_flow::Flow::compile(f.doc.clone(), &cx).unwrap();
        let fails = flow.run_tests(&cx);
        assert!(fails.is_empty(), "{}: {fails:?}", f.doc.id);
    }
}

/// V10's kernel gap: grants as proper `observe.grant` events through the API.
#[test]
fn grants_through_the_api() {
    let l: Vec<serde_json::Value> = contract_lines()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(l[111]["type"], "granted");
    assert_eq!(l[111]["view"], "rumour");
    assert_eq!(l[112]["view"], "scanned");
    let subjects = l[113]["scene"]["subjects"].as_array().unwrap();
    let view = |a: &str| {
        subjects
            .iter()
            .find(|s| s["address"] == a)
            .map(|s| s["view"].clone())
    };
    assert_eq!(view("u:kestrel/sys:0/body:3").unwrap(), "scanned");
    assert_eq!(
        view("u:kestrel/sys:0/body:3/body:0").unwrap(),
        "scanned",
        "moons follow"
    );
    assert_eq!(l[114]["view"], "rumour", "taken back to the starting view");
    assert_ne!(l[114]["state_hash"], l[112]["state_hash"]);
    assert_eq!(l[115]["code"], "unknown_view");
    assert_eq!(l[116]["code"], "bad_address");
}

/// T2 look-ahead through the API: age the Sun on a branch, read its fate,
/// commit with "now" moved, and the error cases.
#[test]
fn look_ahead_through_the_api() {
    let mut k = crate::Kernel::new();
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-time/pack.json")).unwrap(),
    )
    .unwrap();
    let system: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("tests/fixtures/time/solar-system.json")).unwrap(),
    )
    .unwrap();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "sol", "seed": "sol"}),
    );
    assert_eq!(r["type"], "universe_opened");
    let ask = serde_json::json!({"op": "look_ahead", "system": system, "target_myr": 13000.0});
    // No stack yet: no settings.
    assert_eq!(send(&mut k, ask.clone())["code"], "no_time_settings");
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let before = send(
        &mut k,
        serde_json::json!({"op": "state_hash", "at_s": i64::MAX}),
    );

    let r = send(&mut k, ask.clone());
    assert_eq!(r["type"], "looked_ahead", "{r}");
    let kinds: Vec<&str> = r["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    for want in ["oceans_lost", "engulfed", "planetary_nebula"] {
        assert!(kinds.contains(&want), "{want} in {kinds:?}");
    }
    assert!(r["events"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| !e["why"].as_str().unwrap().is_empty()));
    let snaps = r["snapshots"].as_array().unwrap();
    assert!(snaps.len() > 10);
    assert_eq!(snaps[0]["star"]["phase"], "main sequence");
    assert_eq!(snaps.last().unwrap()["star"]["phase"], "white dwarf");
    assert_eq!(r["now_myr"], 4570.0);
    assert!(r["work_units"].as_u64().unwrap() > 1000);
    let e0 = &r["events"][0];
    assert_eq!(
        e0["year"].as_i64().unwrap(),
        9_230_000_000 + (e0["age_myr"].as_f64().unwrap() * 1.0e6).round() as i64
    );
    // The main chronicle is untouched until commit.
    let mid = send(
        &mut k,
        serde_json::json!({"op": "state_hash", "at_s": i64::MAX}),
    );
    assert_eq!(before, mid);
    let b = r["branch"].as_str().unwrap().to_string();
    let bh = send(
        &mut k,
        serde_json::json!({"op": "branch_state_hash", "branch": b, "at_s": i64::MAX}),
    );
    assert_ne!(bh["hash"], mid["hash"]);

    // Discard one, commit another with now moved.
    let d = send(&mut k, serde_json::json!({"op": "discard", "branch": b}));
    assert_eq!(d["type"], "discarded");
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "commit_look_ahead", "branch": b})
        )["code"],
        "unknown_branch"
    );
    let r = send(&mut k, ask);
    let b = r["branch"].as_str().unwrap().to_string();
    let c = send(
        &mut k,
        serde_json::json!({"op": "commit_look_ahead", "branch": b, "move_now_myr": 12000.0}),
    );
    assert_eq!(c["type"], "committed", "{c}");
    assert_ne!(c["state_hash"], mid["hash"]);
    let h = send(
        &mut k,
        serde_json::json!({"op": "history", "address": "u:sol/sys:0"}),
    );
    let hk: Vec<&str> = h["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert!(hk.contains(&"time.request"), "{hk:?}");

    // Errors: target not ahead, a system outside the universe, a bad spec.
    let mut s2 = system.clone();
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "look_ahead", "system": s2, "target_myr": 100.0})
        )["code"],
        "bad_target"
    );
    s2["system"] = "u:other/sys:0".into();
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "look_ahead", "system": s2, "target_myr": 6000.0})
        )["code"],
        "bad_address"
    );
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "look_ahead", "system": {"name": 1}, "target_myr": 6000.0})
        )["code"],
        "bad_system"
    );
}

#[test]
fn run_formation_runs_the_pipeline_with_surfaces() {
    let mut k = crate::Kernel::new();
    let a = k.handle_json(
        r#"{"op":"run_formation","seed":"kernel-form-4","star_pins":{"primary_mass":1.0}}"#,
    );
    let b = crate::Kernel::new().handle_json(
        r#"{"op":"run_formation","seed":"kernel-form-4","star_pins":{"primary_mass":1.0}}"#,
    );
    assert_eq!(a, b, "same request, same response");
    let v: serde_json::Value = serde_json::from_str(&a).unwrap();
    assert_eq!(v["type"], "formation_ran", "{a:.300}");
    assert_eq!(v["system"], "u:generated/sys:0");
    let worlds = v["run"]["worlds"].as_array().unwrap().len();
    assert!(worlds > 0);
    assert_eq!(v["surfaces"].as_array().unwrap().len(), worlds);
    assert!(v.get("pinned").is_none());
    eprintln!("run_formation response {} bytes", a.len());
    // With a pin: Earth at 1 au is honoured.
    let p = k.handle_json(
        r#"{"op":"run_formation","seed":"kernel-form-4","star_pins":{"primary_mass":1.0},"pins":{"worlds":[{"label":"Home","mass_earth":1.0,"a_au":1.0}],"tries":3}}"#,
    );
    let v: serde_json::Value = serde_json::from_str(&p).unwrap();
    assert_eq!(v["type"], "formation_ran", "{p:.300}");
    let m = &v["pinned"]["matches"][0];
    assert!(m["world"].is_string());
    let w = v["run"]["worlds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["address"] == m["world"])
        .unwrap();
    assert!(
        (w["mass_earth"].as_f64().unwrap() - 1.0).abs() < 0.05,
        "{w}"
    );
    // Errors are protocol errors.
    let e = k.handle_json(r#"{"op":"run_formation","seed":"x","block":"sideways"}"#);
    assert!(e.contains("\"type\":\"error\""), "{e}");
}

#[test]
fn import_v3_returns_facts_and_the_report() {
    let save = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");
    let req = serde_json::json!({"op":"import_v3","save":save,"reconstruct":2}).to_string();
    let a = crate::Kernel::new().handle_json(&req);
    assert_eq!(a, crate::Kernel::new().handle_json(&req));
    let v: serde_json::Value = serde_json::from_str(&a).unwrap();
    assert_eq!(v["type"], "v3_imported", "{a:.300}");
    assert!(v["facts"].as_object().unwrap().len() > 3);
    assert!(v["report"]["differences"].is_array());
    assert_eq!(v["reconstructed"].as_array().unwrap().len(), 1);
    assert!(v["reconstructed"][0]["run"]["worlds"].is_array());
    eprintln!("import_v3 response {} bytes", a.len());
    let bad = crate::Kernel::new().handle_json(r#"{"op":"import_v3","save":"not json"}"#);
    assert!(bad.contains("import_failed"), "{bad}");
}

#[test]
fn adopt_age_and_read_back_a_system() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-time/pack.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "load_stack", "packs": [pack]})
        )["type"],
        "stack_loaded"
    );
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );

    // Formation: one event, facts in the importer's names, read back with surfaces and tiles.
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    assert_eq!(r["events"].as_array().unwrap().len(), 1);
    assert!(r["facts"].as_u64().unwrap() > 10);
    let again = send(
        &mut k,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    assert_eq!(again["code"], "already_adopted");
    let h = send(
        &mut k,
        serde_json::json!({"op": "history", "address": "u:home/sys:5"}),
    );
    assert_eq!(h["events"].as_array().unwrap().len(), 1, "{h}");

    let r = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5",
        "tiles": [{"body": "nowhere", "face": 0, "level": 0, "x": 0, "y": 0}]}),
    );
    assert_eq!(r["type"], "system_resolved", "{r}");
    assert_eq!(r["source"], "formation");
    let surfaces = r["surfaces"].as_array().unwrap();
    assert!(!surfaces.is_empty());
    assert_eq!(r["refused"].as_array().unwrap().len(), 1);
    let world = surfaces[0]["address"].as_str().unwrap().to_string();
    let spec = r["spec"].clone();
    // Facts and spec agree on the first world's orbit.
    let a_m = r["facts"][&world]["orbit.semi_major_axis"]
        .as_f64()
        .unwrap();
    let a_au = spec["planets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["address"] == world.as_str())
        .unwrap()["a_au"]
        .as_f64()
        .unwrap();
    assert!((a_m / 149_597_870_700.0 - a_au).abs() < 1e-12);
    let tiles = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5",
        "tiles": [{"body": world, "face": 0, "level": 0, "x": 0, "y": 0},
                  {"body": world, "face": 0, "level": 30, "x": 0, "y": 0}]}),
    );
    assert_eq!(tiles["tiles"].as_array().unwrap().len(), 1, "{tiles}");
    assert_eq!(tiles["refused"].as_array().unwrap().len(), 1);
    // A fresh kernel cache gives the same read (the request is in the event).
    let mut k2 = crate::Kernel::new();
    send(
        &mut k2,
        serde_json::json!({"op": "load_stack", "packs": [pack]}),
    );
    send(
        &mut k2,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    send(
        &mut k2,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    let fresh = send(
        &mut k2,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5",
        "tiles": [{"body": world, "face": 0, "level": 0, "x": 0, "y": 0},
                  {"body": world, "face": 0, "level": 30, "x": 0, "y": 0}]}),
    );
    let t2 = tiles.clone();
    assert_eq!(fresh, t2);

    // Age it with the spec it gave, and write the aged state back.
    let target = spec["age_myr"].as_f64().unwrap() + 3000.0;
    let la = send(
        &mut k,
        serde_json::json!({"op": "look_ahead", "system": spec, "target_myr": target}),
    );
    assert_eq!(la["type"], "looked_ahead", "{la}");
    let move_to = spec["age_myr"].as_f64().unwrap() + 2000.0;
    let c = send(
        &mut k,
        serde_json::json!({"op": "commit_look_ahead", "branch": la["branch"],
        "move_now_myr": move_to, "write_back": true}),
    );
    assert_eq!(c["type"], "committed", "{c}");
    let at = c["aged_at_myr"].as_f64().unwrap();
    assert!(at <= move_to && at > move_to - 500.0, "{at}");
    let aged = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let star = spec["star_address"].as_str().unwrap();
    let l0 = r["facts"][star]["star.luminosity"].as_f64().unwrap();
    let l1 = aged["facts"][star]["star.luminosity"].as_f64().unwrap();
    assert!(l1 > l0, "a Sun-like star brightens: {l0} to {l1}");
    assert!(aged["at_s"].as_i64().unwrap() > 0, "read at the moved now");

    // V3: a handful of events (one per evidence class), the first with the report.
    let save = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "import_v3": save}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let n = r["events"].as_array().unwrap().len();
    assert!((1..=3).contains(&n));
    let sys = r["systems"][0].as_str().unwrap().to_string();
    assert!(sys.starts_with("u:home/"));
    let v = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": sys}),
    );
    assert_eq!(v["source"], "v3", "{v}");
    assert!(
        v["spec"]["planets"]
            .as_array()
            .map(|p| !p.is_empty())
            .unwrap_or(false),
        "{}",
        v["spec"]
    );
    let names: Vec<&str> = v["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["regime_now"].as_str().unwrap())
        .collect();
    eprintln!("v3 adopted regimes: {names:?}");
    assert!(!names.is_empty());
    // Errors.
    assert_eq!(
        send(&mut k, serde_json::json!({"op": "adopt"}))["code"],
        "bad_request"
    );
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "resolved_system", "system": "u:home/sys:99"})
        )["code"],
        "unknown_system"
    );
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "adopt", "formation": {"seed": "x", "system": "u:elsewhere/sys:1"}})
        )["code"],
        "bad_address"
    );
}
