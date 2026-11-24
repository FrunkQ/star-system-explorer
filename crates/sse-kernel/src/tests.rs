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
fn read_world_stack_serves_fixtures_and_checks_given_stacks() {
    let mut k = crate::Kernel::new();
    let a = k.handle_json(r#"{"op":"read_world_stack","fixture":"europa"}"#);
    let v: serde_json::Value = serde_json::from_str(&a).unwrap();
    assert_eq!(v["type"], "world_stack_read", "{a:.300}");
    assert_eq!(v["source"], "fixture:europa");
    assert_eq!(v["stack"]["format"], "sse.world-stack/1");
    assert!(!v["ladder"].as_array().unwrap().is_empty());
    assert!(!v["notes"].as_array().unwrap().is_empty(), "{}", v["notes"]);
    assert!(
        v["invalid"].as_array().unwrap().is_empty(),
        "{}",
        v["invalid"]
    );
    // By the fixture's body address: the same stack.
    let b = k.handle_json(r#"{"op":"read_world_stack","body":"u:sol/sys:0/body:8"}"#);
    let w: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert_eq!(w["stack"], v["stack"]);
    // A given stack is checked: freeze Europa's ocean and physics objects.
    let mut st = v["stack"].clone();
    for s in st["shells"].as_array_mut().unwrap() {
        if s["properties"].get("salinity").is_some() {
            s["properties"]["temperature_k"] = serde_json::json!(200.0);
        }
    }
    let req = serde_json::json!({"op":"read_world_stack","stack":st}).to_string();
    let c: serde_json::Value = serde_json::from_str(&k.handle_json(&req)).unwrap();
    assert_eq!(c["source"], "given");
    assert!(
        !c["problems"].as_array().unwrap().is_empty(),
        "{}",
        c["problems"]
    );
    let none = k.handle_json(r#"{"op":"read_world_stack","body":"u:nowhere/sys:0/body:1"}"#);
    assert!(none.contains("no_world_stack"), "{none}");
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
    // F5's leftovers come with the run: at least one cloud round the star,
    // each with a sensible span.
    let clouds = r["clouds"]
        .as_array()
        .expect("a grown system has leftovers");
    assert!(clouds.iter().any(|c| c["around"].is_null()), "{clouds:?}");
    for c in clouds {
        let (lo, hi) = (
            c["a_min_au"].as_f64().unwrap(),
            c["a_max_au"].as_f64().unwrap(),
        );
        assert!(lo > 0.0 && hi >= lo, "{c}");
        assert!(c["kind"].is_string() && c["address"].is_string(), "{c}");
        let shell = c["shape"] == "shell";
        assert!(shell || c["shape"] == "disc", "{c}");
        assert_eq!(shell, c["i_sigma_deg"].is_null(), "{c}");
        if c["kind"] == "oort" {
            assert!(shell, "{c}");
        }
    }
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
    // Evidence per fact: the aged luminosity is derived by the write-back; the orbit host is from the adoption.
    assert_eq!(aged["evidence"][star]["star.luminosity"]["by"], "time.aged");
    assert_eq!(
        aged["evidence"][star]["star.luminosity"]["evidence"],
        "derived"
    );
    assert_eq!(
        aged["evidence"][&world]["orbit.host"]["evidence"],
        "resolved"
    );
    assert_eq!(
        r["evidence"][star]["star.luminosity"]["by"],
        "adopt.formation"
    );

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
    // The universe's systems, with their sources.
    let l = send(&mut k, serde_json::json!({"op": "systems"}));
    let got: Vec<(String, String)> = l["systems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["system"].as_str().unwrap().to_string(),
                s["source"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(
        got.contains(&("u:home/sys:5".to_string(), "formation".to_string())),
        "{got:?}"
    );
    assert!(got.contains(&(sys.clone(), "v3".to_string())), "{got:?}");
    // The imported Earth carries its oceans (importer hydrosphere facts), so it has plates.
    let earth = v["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["label"] == "Earth")
        .expect("Earth's surface");
    assert_eq!(earth["regime_now"], "mobile-lid", "{earth:.400}");
    // W7b: the imported Earth's climate spins at V3's own day, marked authored (V3).
    let w = send(
        &mut k,
        serde_json::json!({"op": "weather", "want": "climate", "system": sys, "body": earth["address"]}),
    );
    let c = &w["data"]["core"]["conditions"];
    assert!(
        (c["rotation_hours"].as_f64().unwrap() - 23.9345).abs() < 1e-9,
        "{c}"
    );
    let sv = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "system": sys, "body": earth["address"], "quick": true}),
    );
    let assumed = sv["survey"]["climate"]["assumed"].to_string();
    assert!(assumed.contains("authored (V3)"), "{assumed}");
    // W-sea: the card's tides and waves, from the formula blocks.
    let sea = &sv["survey"]["climate"]["sea"];
    let by = sea["tides"]["by"].as_array().expect("tides by raiser");
    assert!(by.iter().any(|t| t["raiser"] == "host"), "{sea}");
    // The imported Moon, listed beside Earth, raises the larger tide: about
    // half a metre every 12.42 hours, and the Sun's is about 0.46 of it.
    let moon = &by[0];
    assert_eq!(moon["raiser"], "moon 1", "{sea}");
    assert!(
        (moon["range_m"].as_f64().unwrap() - 0.535).abs() < 0.01,
        "{moon}"
    );
    assert!(
        (moon["period_h"].as_f64().unwrap() - 12.42).abs() < 0.01,
        "{moon}"
    );
    for t in by {
        let h = t["range_m"].as_f64().unwrap();
        assert!(h > 0.0 && h < 10.0, "{t}");
    }
    let typical = sea["waves"]["typical_m"].as_f64().unwrap();
    assert!((0.3..8.0).contains(&typical), "{sea}");
    // And the landing site's sea state under the wind blowing there.
    let r = send(
        &mut k,
        serde_json::json!({"op": "weather", "want": "report", "system": sys, "body": earth["address"], "lat": -50.0, "lon": 160.0}),
    );
    let site = &r["data"]["sea"];
    let wave = site["wave_m"].as_f64().unwrap_or(-1.0);
    assert!((0.0..20.0).contains(&wave), "{r:.600}");
    assert!(
        site["tides"]["spring_range_m"].as_f64().unwrap() > 0.0,
        "{site}"
    );
    // A formation world spins at its settled spin.
    let f = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let fb = f["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] != "cloud-deck")
        .expect("a solid world")["address"]
        .clone();
    let sv = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "system": "u:home/sys:5", "body": fb, "quick": true}),
    );
    let assumed = sv["survey"]["climate"]["assumed"].to_string();
    assert!(assumed.contains("(formation)"), "{assumed}");
    // W9 part 2: a formation world reads its ice ages from T1's secular
    // theory over its siblings, and each band says how often ice advances.
    let ia = &sv["survey"]["climate"]["ice_ages"];
    assert!(
        ia["source"].as_str().unwrap().contains("T1 secular"),
        "{ia}"
    );
    let swing =
        ia["obliquity_max_deg"].as_f64().unwrap() - ia["obliquity_min_deg"].as_f64().unwrap();
    assert!(swing > 0.0 && swing < 30.0, "{ia}");
    let places = sv["survey"]["climate"]["places"].as_array().unwrap();
    assert!(places.iter().all(|p| p["ice_share"].is_number()));
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

#[test]
fn save_and_reopen_a_universe_and_read_life_views() {
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-time/pack.json")).unwrap(),
    )
    .unwrap();
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack]}),
    );
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home seed"}),
    );
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    send(
        &mut k,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    let r = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let la = send(
        &mut k,
        serde_json::json!({"op": "look_ahead", "system": r["spec"], "target_myr": r["spec"]["age_myr"].as_f64().unwrap() + 1000.0}),
    );
    send(
        &mut k,
        serde_json::json!({"op": "commit_look_ahead", "branch": la["branch"], "move_now_myr": r["spec"]["age_myr"].as_f64().unwrap() + 500.0, "write_back": true}),
    );
    let before = send(&mut k, serde_json::json!({"op": "state_hash"}));
    let read_before = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let man = send(&mut k, serde_json::json!({"op": "manifest"}));
    let ex = send(&mut k, serde_json::json!({"op": "export_chronicle"}));
    assert_eq!(ex["type"], "chronicle_exported", "{ex}");
    // Through text, as a browser would save it.
    let saved: serde_json::Value = serde_json::from_str(&ex["export"].to_string()).unwrap();

    let mut k2 = crate::Kernel::new();
    send(
        &mut k2,
        serde_json::json!({"op": "load_stack", "packs": [pack]}),
    );
    let im = send(
        &mut k2,
        serde_json::json!({"op": "import_chronicle", "export": saved}),
    );
    assert_eq!(im["type"], "chronicle_imported", "{im}");
    assert_eq!(im["stack_matches"], true);
    assert_eq!(
        send(&mut k2, serde_json::json!({"op": "state_hash"})),
        before
    );
    assert_eq!(
        send(
            &mut k2,
            serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"})
        ),
        read_before
    );
    assert_eq!(im["manifest_id"], ex["export"]["manifest_id"]);
    let _ = man;
    // Tampering is refused.
    let mut bad = saved.clone();
    let c = bad["events"][0]["cbor"].as_str().unwrap().to_string();
    let flipped = format!(
        "{}{}",
        &c[..c.len() - 2],
        if c.ends_with("00") { "01" } else { "00" }
    );
    bad["events"][0]["cbor"] = flipped.into();
    let e = send(
        &mut crate::Kernel::new(),
        serde_json::json!({"op": "import_chronicle", "export": bad}),
    );
    assert_eq!(e["code"], "bad_export", "{e}");
    // A different stack is flagged, not refused.
    let im3 = send(
        &mut crate::Kernel::new(),
        serde_json::json!({"op": "import_chronicle", "export": saved}),
    );
    assert_eq!(im3["stack_matches"], false);

    // Life views by fixture.
    let v = send(
        &mut k,
        serde_json::json!({"op": "life_view", "fixture": "europa"}),
    );
    assert_eq!(v["type"], "life_viewed", "{v}");
    assert_eq!(v["source"], "fixture:europa");
    let hooks: Vec<&str> = v["view"]["hooks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["id"].as_str().unwrap())
        .collect();
    assert!(hooks.contains(&"hidden_life"), "{hooks:?}");
    let e = send(
        &mut k,
        serde_json::json!({"op": "life_view", "fixture": "mars"}),
    );
    assert_eq!(e["code"], "unknown_fixture");
    let e = send(
        &mut k,
        serde_json::json!({"op": "life_view", "body": "u:home/sys:5/body:1"}),
    );
    assert_eq!(e["code"], "no_habitats");
}

/// S7 served: a fixture world's survey (layers, register, aerostat band,
/// tiles with the new layers), a GM rename and edit, a giant, refusals, and
/// a body of an adopted system read at its date.
#[test]
fn survey_surface_serves_layers_register_and_tiles() {
    let mut k = crate::Kernel::new();
    let opts = serde_json::json!({
        "rotation_hours": 23.93, "life": 1.0, "settle": ["settlement", "port"],
        "reshape": [{"op": "raise-range", "id": "gm:r", "from": [10.0, 20.0], "to": [10.0, 40.0],
                     "height_m": 40000.0, "width_km": 150.0, "at_myr": 0.0}],
    });
    let req = serde_json::json!({"op": "survey_surface", "fixture": "earth", "quick": true,
        "options": opts, "tiles": [{"face": 0, "level": 0, "x": 0, "y": 0}, {"face": 9, "level": 0, "x": 0, "y": 0}]});
    let r = send(&mut k, req.clone());
    assert_eq!(r["type"], "surface_surveyed", "{r}");
    let s = &r["survey"];
    assert_eq!(s["source"], "fixture:Earth");
    assert_eq!(s["stages"].as_array().unwrap().len(), 7);
    assert_eq!(s["climate"]["cells"], 3, "{}", s["climate"]);
    assert!(s["aerostat"]["low_km"].as_f64().is_some(), "{s}");
    assert!(!s["built"].as_array().unwrap().is_empty());
    assert!(
        s["applied"][0]["impossible"].is_string(),
        "a 40 km range is tagged"
    );
    assert_eq!(s["tiles"].as_array().unwrap().len(), 1);
    assert!(s["tiles"][0]["colour"]
        .as_array()
        .is_some_and(|c| !c.is_empty()));
    assert_eq!(s["refused"][0]["why"], "bad tile key");
    let feats = s["register"]["features"].as_array().unwrap();
    let ocean = feats
        .iter()
        .find(|f| f["kind"] == "ocean")
        .expect("an ocean");
    // Same request, same bytes; a rename changes names only.
    assert_eq!(send(&mut k, req.clone()), r);
    let mut renamed = req.clone();
    renamed["options"]["renames"] =
        serde_json::json!([{"feature": ocean["id"], "name": "Big Blue", "at_myr": 0.0}]);
    let r2 = send(&mut k, renamed);
    assert_eq!(r2["survey"]["tiles"], s["tiles"]);
    assert_eq!(
        r2["survey"]["register"]["renames"][0]["name"], "Big Blue",
        "{}",
        r2["survey"]["register"]["renames"]
    );

    // A giant: cloud deck only, no layers or register; the band is refused with why.
    let j = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "fixture": "Jupiter", "quick": true}),
    );
    assert_eq!(
        j["survey"]["stages"],
        serde_json::json!(["envelope", "cloud-deck"])
    );
    assert!(j["survey"].get("register").is_none());
    assert!(j["survey"]["aerostat_refused"].is_string(), "{j}");

    // Refusals.
    let e = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "fixture": "Krypton"}),
    );
    assert_eq!(e["code"], "unknown_fixture");
    let e = send(&mut k, serde_json::json!({"op": "survey_surface"}));
    assert_eq!(e["code"], "bad_request");
    let e = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "fixture": "Earth", "options": {"life": "lots"}}),
    );
    assert_eq!(e["code"], "bad_request");
    let e = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "system": "u:home/sys:5", "body": "u:home/sys:5/body:1"}),
    );
    assert_eq!(e["code"], "no_universe");

    // A body of an adopted system.
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "adopt", "formation": form})
        )["type"],
        "adopted"
    );
    let v = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let solid = v["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["kind"] != "cloud-deck")
        .expect("a solid world");
    let body = solid["address"].as_str().unwrap();
    let r = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "system": "u:home/sys:5", "body": body, "quick": true}),
    );
    assert_eq!(r["survey"]["source"], "formation", "{r}");
    assert_eq!(r["survey"]["body"], body);
    let total = r["survey"]["register"]["features_total"].as_u64().unwrap();
    let listed = r["survey"]["register"]["features"]
        .as_array()
        .unwrap()
        .len() as u64;
    assert_eq!(listed, total.min(200));
    let few = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "system": "u:home/sys:5", "body": body, "quick": true, "limit": 3}),
    );
    let f = few["survey"]["register"]["features"].as_array().unwrap();
    assert_eq!(f.len() as u64, total.min(3));
    assert!(few["survey"]["deposits"].as_array().unwrap().len() <= 3);
    let e = send(
        &mut k,
        serde_json::json!({"op": "survey_surface", "system": "u:home/sys:5", "body": "u:home/sys:5/body:999"}),
    );
    assert_eq!(e["code"], "no_surface_body");

    // W7a: the weather of the same body, by address, is the ground's climate.
    let w = send(
        &mut k,
        serde_json::json!({"op": "weather", "want": "climate", "system": "u:home/sys:5", "body": body}),
    );
    assert_eq!(w["type"], "weather_read", "{w}");
    let core = &w["data"]["core"];
    assert!(core["key"].as_str().unwrap().starts_with(body), "{core}");
    let ground = &r["survey"]["climate"];
    let wet_past = ground["assumed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|a| a.as_str().unwrap().starts_with("while wet"));
    assert!(!wet_past, "this body is no wetter in the past");
    assert_eq!(core["key"], ground["key"]);
    assert_eq!(core["t_k"], ground["t_k"]);
    assert_eq!(core["precip_m_yr"], ground["precip_m_yr"]);
    let again = send(
        &mut k,
        serde_json::json!({"op": "weather", "want": "climate", "system": "u:home/sys:5", "body": body}),
    );
    assert_eq!(again, w, "a re-read reuses the kept climate");
    let e = send(
        &mut k,
        serde_json::json!({"op": "weather", "want": "climate", "system": "u:home/sys:5"}),
    );
    assert_eq!(e["code"], "bad_request");
}

/// J1 through the API: notes by audience, news that arrives later, kept
/// values that fold, explain themselves and reach players only as their view
/// allows (a band stays a band), the writer rule, actors, disclosures with no
/// event ids, the universe hash untouched by the record, and a save and
/// reopen that keeps the record.
#[test]
fn the_record_through_the_api() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-record"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    let tarn = "u:kestrel/sys:0/body:3";
    let a0 = send(
        &mut k,
        serde_json::json!({"op":"set_authored","edit":{"address":tarn,"field":"name","value":"Tarn","occurred_at_s":0}}),
    );
    let universe = a0["state_hash"].clone();
    assert!(universe.is_string(), "{a0}");
    let before = send(
        &mut k,
        serde_json::json!({"op":"scene","audience":"crew","scope":"u:kestrel","at_s":10}),
    );
    let n1 = send(
        &mut k,
        serde_json::json!({"op":"add_note","address":tarn,"text":"the crew peed on the statue","tags":["insult"],"by":"crew-seat","audience":["crew"],"at_s":5}),
    );
    assert_eq!(n1["type"], "note_added", "{n1}");
    assert_eq!(n1["audience"], serde_json::json!(["crew"]));
    let n2 = send(
        &mut k,
        serde_json::json!({"op":"add_note","address":tarn,"text":"the statue is a sleeping machine","at_s":5}),
    );
    assert_eq!(n2["audience"], "gm", "GM only by default");
    let n3 = send(
        &mut k,
        serde_json::json!({"op":"add_note","address":tarn,"text":"the Calla was taken off Tarn","audience":"everyone","at_s":5,"arrives_at_s":20}),
    );
    // The record never moves the universe hash; it has its own.
    for n in [&n1, &n2, &n3] {
        assert_eq!(n["state_hash"], universe);
    }
    assert_ne!(n1["record_hash"], n2["record_hash"]);
    let notes = |k: &mut crate::Kernel, aud: &str, at: i64| {
        let s = send(
            k,
            serde_json::json!({"op":"scene","audience":aud,"scope":"u:kestrel","at_s":at}),
        );
        s["scene"]["subjects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["address"] == tarn)
            .map(|x| x["notes"].as_array().map(|a| a.len()).unwrap_or(0))
            .unwrap_or(0)
    };
    assert_eq!(notes(&mut k, "crew", 10), 1);
    assert_eq!(notes(&mut k, "vesh", 10), 0);
    assert_eq!(notes(&mut k, "gm", 10), 2);
    assert_eq!(notes(&mut k, "vesh", 20), 1, "the news has arrived");
    let crew_early = send(
        &mut k,
        serde_json::json!({"op":"scene","audience":"crew","scope":"u:kestrel","at_s":4}),
    );
    assert_eq!(
        crew_early["scene_hash"], before["scene_hash"],
        "before the note, nothing changed"
    );
    // A correction supersedes.
    let fix = send(
        &mut k,
        serde_json::json!({"op":"add_note","address":tarn,"text":"the crew only threatened to","audience":["crew"],"supersedes":n1["note_id"],"at_s":6}),
    );
    assert_eq!(fix["type"], "note_added", "{fix}");
    // Actors and kept values.
    let party = send(
        &mut k,
        serde_json::json!({"op":"add_actor","id":"party","name":"The crew of the Calla","kind":"party","at_s":0}),
    );
    assert_eq!(party["address"], "u:kestrel/actor:party", "{party}");
    assert_eq!(party["state_hash"], universe);
    let p = "u:kestrel/actor:party";
    for (v, w, why) in [
        (1000, "gm", "starting purse"),
        (1400, "intent.trade", "sold ore"),
        (1386, "gm", "docking fee"),
    ] {
        let r = send(
            &mut k,
            serde_json::json!({"op":"set_kept","address":p,"kind":"trade.credits","value":v,"writer":w,"reason":why,"at_s":7}),
        );
        assert_eq!(r["type"], "kept_set", "{r}");
        assert_eq!(r["state_hash"], universe);
    }
    let refused = send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":p,"kind":"politics.standing","value":-1,"writer":"intent.trade","at_s":7}),
    );
    assert_eq!(refused["code"], "writer_refused", "{refused}");
    let bad = send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":p,"kind":"politics.standing","value":-11,"at_s":7}),
    );
    assert_eq!(bad["code"], "bad_value");
    let unknown = send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":p,"kind":"x4.metal","value":1,"at_s":7}),
    );
    assert_eq!(unknown["code"], "unknown_kind");
    send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":p,"kind":"politics.standing","value":-2,"writer":"effect.consequence","reason":"the insult","at_s":7}),
    );
    send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":p,"kind":"misc.ship_cat","value":"Jonesy","at_s":7}),
    );
    let rec = send(&mut k, serde_json::json!({"op":"record","address":p}));
    assert_eq!(rec["type"], "record_read", "{rec}");
    let kept = rec["kept"].as_array().unwrap();
    assert_eq!(kept.len(), 3);
    let credits = kept.iter().find(|x| x["kind"] == "trade.credits").unwrap();
    assert_eq!(credits["value"], 1386);
    assert_eq!(credits["history"][1]["reason"], "sold ore");
    assert_eq!(credits["history"][2]["delta"], -14.0);
    // What the crew's seat is sent: at long range the credits as a band,
    // nothing secret, the corrected note, and no event ids anywhere.
    let d = send(
        &mut k,
        serde_json::json!({"op":"disclosures","audience":"crew","at_s":10}),
    );
    assert_eq!(d["type"], "disclosed", "{d}");
    let text = d.to_string();
    let item = |d: &serde_json::Value, a: &str| {
        d["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["subject"] == a)
            .cloned()
    };
    let field = |d: &serde_json::Value, f: &str| {
        item(d, p).unwrap()["kept"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["field"] == f)
            .cloned()
    };
    // The actor's obvious name is disclosed too; it is record, not universe.
    assert_eq!(field(&d, "name").unwrap()["value"], "The crew of the Calla");
    let c = field(&d, "trade.credits").unwrap();
    assert!(c["value"].is_null());
    assert_eq!(c["band"], serde_json::json!([500.0, 1500.0]));
    assert_eq!(item(&d, p).unwrap()["kept"].as_array().unwrap().len(), 2);
    assert!(
        !text.contains("1386") && !text.contains("Jonesy") && !text.contains("politics.standing")
    );
    let tn = item(&d, tarn).unwrap()["notes"].clone();
    assert_eq!(tn.as_array().unwrap().len(), 1);
    assert_eq!(tn[0]["text"], "the crew only threatened to");
    assert_eq!(tn[0]["id"], "n1");
    for id in [
        &n1["note_id"],
        &n2["note_id"],
        &n3["note_id"],
        &fix["note_id"],
    ] {
        assert!(!text.contains(id.as_str().unwrap()), "an event id leaked");
    }
    assert!(!text.contains("sleeping machine") && !text.contains("Calla was taken"));
    // The Vesh get nothing of the crew's, and the news only once it arrives.
    let dv = send(
        &mut k,
        serde_json::json!({"op":"disclosures","audience":"vesh","at_s":10}),
    );
    assert!(!dv.to_string().contains("crew only"), "{dv}");
    assert!(item(&dv, tarn).is_none());
    let dv20 = send(
        &mut k,
        serde_json::json!({"op":"disclosures","audience":"vesh","at_s":20}),
    );
    assert_eq!(
        item(&dv20, tarn).unwrap()["notes"][0]["text"],
        "the Calla was taken off Tarn"
    );
    assert_ne!(dv["stream_hash"], dv20["stream_hash"]);
    // Once the crew have the party scanned, the exact value.
    send(
        &mut k,
        serde_json::json!({"op":"grant","address":p,"audience":"crew","view":"scanned","at_s":8}),
    );
    let d2 = send(
        &mut k,
        serde_json::json!({"op":"disclosures","audience":"crew","at_s":10}),
    );
    assert_eq!(field(&d2, "trade.credits").unwrap()["value"], 1386, "{d2}");
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op":"disclosures","audience":"gm","at_s":10})
        )["code"],
        "bad_audience"
    );
    let tg = send(
        &mut k,
        serde_json::json!({"op":"record","address":"u:kestrel"}),
    );
    assert_eq!(tg["actors"][0]["name"], "The crew of the Calla");
    // Save and reopen: the record comes back; the universe hash is the same.
    let ex = send(&mut k, serde_json::json!({"op":"export_chronicle"}));
    let mut k2 = crate::Kernel::new();
    send(
        &mut k2,
        serde_json::json!({"op":"load_stack","packs":[packs["tarn-observe"]]}),
    );
    let im = send(
        &mut k2,
        serde_json::json!({"op":"import_chronicle","export":ex["export"]}),
    );
    assert_eq!(im["type"], "chronicle_imported", "{im}");
    // Without the record pack the values stay readable, marked as orphans.
    let back = send(&mut k2, serde_json::json!({"op":"record","address":p}));
    let c2 = back["kept"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["kind"] == "trade.credits")
        .unwrap()
        .clone();
    assert_eq!(c2["value"], 1386);
    assert_eq!(c2["orphan"], "kept by a pack that is not loaded");
    assert_eq!(back["kept"].as_array().unwrap().len(), 3);
    // A tampered record is refused.
    let mut bad_ex = ex["export"].clone();
    bad_ex["record_head"] = serde_json::json!("00");
    let mut k3 = crate::Kernel::new();
    assert_eq!(
        send(
            &mut k3,
            serde_json::json!({"op":"import_chronicle","export":bad_ex})
        )["code"],
        "bad_export"
    );
}

/// T3 through the API: a fixture course, a drifting ship's ways out, a ship
/// object, refusals, and a course between two worlds of an adopted system.
#[test]
fn plan_route_gives_courses_and_ways_out() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": "chemical-transfer",
            "from": "Earth", "to": "Mars", "prefer": "save-fuel", "window_days": 800.0}),
    );
    assert_eq!(r["type"], "route_planned", "{r}");
    let p = &r["plan"];
    assert_eq!(p["from"], "u:sol/sys:sun/body:earth");
    let c = &p["candidates"][0];
    assert!(c["days"].as_f64().unwrap() > 100.0);
    assert!(c["difficulty"]["label"].is_string());
    assert_eq!(p["recommended"], 0);
    // Same question, same bytes.
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": "chemical-transfer",
            "from": "Earth", "to": "Mars", "prefer": "save-fuel", "window_days": 800.0})
        ),
        r
    );
    // Drifting at 1 au with no speed: falls into the Sun; rescue from a port.
    let d = send(
        &mut k,
        serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": "chemical-transfer",
            "from": {"r_m": [1.495978707e11, 0.0, 0.0], "v_m_s": [0.0, 0.0, 0.0]}}),
    );
    let drift = &d["plan"]["drift"];
    assert!(
        drift["fate"]
            .as_str()
            .unwrap()
            .starts_with("falls into the star"),
        "{d}"
    );
    assert!(drift["ways_out"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w["kind"] == "rescue" && w["feasible"] == true));
    // A ship object, and refusals.
    let ship = serde_json::json!({"name": "Skiff", "accel_g": 0.01, "exhaust_km_s": 3.0, "wet_t": 11.0, "dry_t": 10.0});
    let w = send(
        &mut k,
        serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": ship, "from": "Earth", "to": "Neptune"}),
    );
    assert!(
        w["plan"]["candidates"].as_array().unwrap().is_empty(),
        "{w}"
    );
    assert!(!w["plan"]["refused"].as_array().unwrap().is_empty());
    for (req, code) in [
        (
            serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": "warp-barge", "from": "Earth", "to": "Mars"}),
            "unknown_ship",
        ),
        (
            serde_json::json!({"op": "plan_route", "fixture": "tatooine", "ship": "fusion-torch", "from": "Earth", "to": "Mars"}),
            "unknown_fixture",
        ),
        (
            serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": "fusion-torch", "from": "Vulcan", "to": "Mars"}),
            "route_refused",
        ),
        (
            serde_json::json!({"op": "plan_route", "ship": "fusion-torch", "from": "Earth", "to": "Mars"}),
            "bad_request",
        ),
        (
            serde_json::json!({"op": "plan_route", "fixture": "sol", "ship": "fusion-torch", "from": "Earth", "to": "Mars", "prefer": "scenic"}),
            "bad_request",
        ),
        (
            serde_json::json!({"op": "plan_route", "system": "u:home/sys:5", "ship": "fusion-torch", "from": "Earth", "to": "Mars"}),
            "no_universe",
        ),
    ] {
        assert_eq!(send(&mut k, req)["code"], code);
    }
    // Between two worlds of an adopted system.
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op": "adopt", "formation": form})
        )["type"],
        "adopted"
    );
    let v = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let planets = v["spec"]["planets"].as_array().unwrap();
    assert!(planets.len() >= 2);
    let (a, b) = (
        planets[0]["address"].as_str().unwrap(),
        planets[1]["address"].as_str().unwrap(),
    );
    let r = send(
        &mut k,
        serde_json::json!({"op": "plan_route", "system": "u:home/sys:5", "ship": "fusion-torch",
            "from": a, "to": b, "prefer": "fastest", "ports": [a]}),
    );
    assert_eq!(r["type"], "route_planned", "{r}");
    assert_eq!(r["plan"]["candidates"][0]["kind"], "torch");
}

/// J2: the logic blocks are listed with their class and shapes; physics
/// atoms list as before (no new keys).
#[test]
fn logic_blocks_are_listed() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"atoms","ids":["logic.select","world.read","toy.luminosity_from_mass"]}),
    );
    let atoms = r["atoms"].as_array().unwrap_or_else(|| panic!("{r}"));
    assert_eq!(atoms.len(), 3, "{r}");
    let by = |id: &str| atoms.iter().find(|a| a["id"] == id).unwrap().clone();
    let sel = by("logic.select");
    assert_eq!(sel["class"], "logic");
    assert_eq!(sel["inputs"][0]["shape"], "flag");
    assert_eq!(sel["inputs"][1]["any_dim"], true);
    let toy = by("toy.luminosity_from_mass");
    assert!(toy.get("class").is_none() && toy["inputs"][0].get("shape").is_none());
}

/// S5 served: descents onto fixture worlds through the survey cache. Mars
/// shows why real landers go to low ground: the same lander lands in a low
/// basin and crashes on the uplands.
#[test]
fn plan_descent_flies_landers_onto_surveyed_sites() {
    let mut k = crate::Kernel::new();
    let ask = |w: &str, lander: &str, lat: f64, lon: f64| {
        serde_json::json!({"op": "plan_descent", "fixture": w, "quick": true,
            "lander": lander, "lat_deg": lat, "lon_deg": lon})
    };
    let get = |k: &mut crate::Kernel, q: serde_json::Value| {
        let r = send(k, q);
        assert_eq!(r["type"], "descent_planned", "{r}");
        r["descent"].clone()
    };
    // Earth: a capsule on its parachute; this site is at sea, so it is a
    // splashdown flagged on the site.
    let d = get(&mut k, ask("Earth", "capsule", 45.0, 60.0));
    let o = &d["descent"]["outcome"];
    assert_eq!(o["result"], "landed", "{o}");
    assert_eq!(o["method"], "chute");
    assert_eq!(d["site"]["under_liquid"], "water");
    assert_eq!(d["verdict"], "landed-on-a-bad-site");
    assert!(d["hazards"]
        .as_array()
        .unwrap()
        .iter()
        .any(|h| h["kind"] == "liquid" && h["level"] == "danger"));
    assert_eq!(d["entry"]["flight_path_deg"], -2.0);
    // Mars: low ground lands, high ground crashes.
    let low = get(&mut k, ask("Mars", "mars-lander", 0.0, 120.0));
    let high = get(&mut k, ask("Mars", "mars-lander", 0.0, 180.0));
    assert!(
        low["site"]["height_m"].as_f64().unwrap()
            < high["site"]["height_m"].as_f64().unwrap() - 3000.0
    );
    assert_eq!(
        low["descent"]["outcome"]["result"], "landed",
        "{}",
        low["descent"]["outcome"]
    );
    assert_eq!(low["descent"]["outcome"]["method"], "chute-and-engines");
    assert_eq!(high["descent"]["outcome"]["result"], "crash");
    assert!(
        high["air_at_site"]["density_kg_m3"].as_f64()
            < low["air_at_site"]["density_kg_m3"].as_f64()
    );
    // Moon: engines only, from a low orbit.
    let d = get(&mut k, ask("Moon", "moon-lander", 0.7, 23.5));
    assert_eq!(d["descent"]["outcome"]["result"], "landed");
    assert_eq!(d["descent"]["outcome"]["method"], "engines");
    assert_eq!(d["air_at_site"]["density_kg_m3"], 0.0);
    // Titan: a long parachute descent.
    let d = get(&mut k, ask("Titan", "titan-probe", -10.0, 192.0));
    assert_eq!(d["verdict"], "landed", "{}", d["site"]);
    assert!(d["descent"]["outcome"]["duration_s"].as_f64().unwrap() > 3600.0);
    // Deterministic, and refusals.
    assert_eq!(get(&mut k, ask("Titan", "titan-probe", -10.0, 192.0)), d);
    let r = send(&mut k, ask("Titan", "glider", 0.0, 0.0));
    assert_eq!(r["code"], "unknown_lander", "{r}");
    let r = send(&mut k, ask("Jupiter", "capsule", 0.0, 0.0));
    assert_eq!(r["code"], "no_ground", "{r}");
    let r = send(&mut k, ask("Earth", "capsule", 100.0, 0.0));
    assert_eq!(r["code"], "bad_request", "{r}");
}

/// Run-through blocker 4: the party's own ship lands on its own numbers
/// (`lander: {ship}`), a stock id or a whole nav ship, as ship_at_world judges it.
#[test]
fn plan_descent_lands_the_partys_own_ship() {
    let mut k = crate::Kernel::new();
    let ask = |lander: serde_json::Value| {
        serde_json::json!({"op": "plan_descent", "fixture": "Moon", "quick": true,
            "lander": lander, "lat_deg": 0.7, "lon_deg": 23.5})
    };
    let r = send(
        &mut k,
        ask(serde_json::json!({"ship": "chemical-transfer"})),
    );
    assert_eq!(r["type"], "descent_planned", "{r}");
    let o = &r["descent"]["descent"]["outcome"];
    assert_eq!(o["result"], "landed", "{o}");
    assert_eq!(o["method"], "engines");
    // The same ship sent whole lands the same way; a lighter copy lands too.
    let whole = serde_json::json!({"name": "Chemical transfer ship", "accel_g": 0.3,
        "exhaust_km_s": 4.4, "wet_t": 500.0, "dry_t": 100.0});
    let r2 = send(&mut k, ask(serde_json::json!({"ship": whole})));
    assert_eq!(r2["descent"], r["descent"]);
    let r = send(&mut k, ask(serde_json::json!({"ship": "no-such-ship"})));
    assert_eq!(r["code"], "unknown_ship", "{r}");
}

/// J3: a stack with boxes loads (box golden cases run, flows are compiled
/// with their boxes inlined); a box whose own case fails is refused by name.
#[test]
fn boxes_load_with_the_stack() {
    let root = root();
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let boxes: serde_json::Value =
        serde_json::from_str(&read("tests/fixtures/boxes/toy-boxes.json")).unwrap();
    let flow: serde_json::Value =
        serde_json::from_str(&read("tests/fixtures/boxes/toy-boxed.json")).unwrap();
    let kinds: Vec<sse_core::QuantityKind> = sse_flow::toy::kinds().into_values().collect();
    let pack = |boxes: serde_json::Value| {
        serde_json::json!({
            "format": "sse.pack/1", "id": "sse.boxed-toy", "version": "0.1.0", "kind": ["base-physics"],
            "kernel_range": "^0.1", "licence": "Apache-2.0",
            "provides": {"quantity_kinds": kinds, "flows": [flow], "boxes": boxes}
        })
    };
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[pack(boxes.clone())]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let mut bad = boxes;
    bad[1]["tests"][0]["expect"]["teff"]["value"] = serde_json::json!(6000);
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[pack(bad)]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(
        r["report"]
            .as_str()
            .unwrap()
            .contains("(box toy.star): test toy.star: the Sun: teff = 577"),
        "{r}"
    );
}

/// J4 through the API: the sensor ladder grants the crew the next rung after an
/// hour in range; an `ask` consequence changes nothing until the GM says yes;
/// the rules run on the GM machine and a seat's disclosures carry results
/// only, never the trail, its ids or its inputs; a replay is identical.
#[test]
fn rules_through_the_api() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let run = || {
        let mut k = crate::Kernel::new();
        let r = send(
            &mut k,
            serde_json::json!({"op":"load_stack","packs":[packs["rpg-record"], packs["rpg-rules"]]}),
        );
        assert_eq!(r["type"], "stack_loaded", "{r}");
        send(
            &mut k,
            serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
        );
        let tarn = "u:kestrel/sys:0/body:3";
        let ship = "u:kestrel/ship:calla";
        let edit = |k: &mut crate::Kernel, a: &str, f: &str, v: serde_json::Value, t: i64| {
            let r = send(
                k,
                serde_json::json!({"op":"set_authored","edit":{"address":a,"field":f,"value":v,"occurred_at_s":t}}),
            );
            assert_eq!(r["type"], "authored", "{r}");
        };
        edit(&mut k, tarn, "name", serde_json::json!("Tarn"), 0);
        for (f, v) in [("pos.x_m", 0.0), ("pos.y_m", 0.0), ("pos.z_m", 0.0)] {
            edit(&mut k, tarn, f, serde_json::json!(v), 0);
        }
        edit(&mut k, ship, "tag.ship", serde_json::json!(true), 0);
        edit(&mut k, ship, "sensor.range_m", serde_json::json!(1.0e9), 0);
        for (f, v) in [("pos.x_m", 5.0e9), ("pos.y_m", 0.0), ("pos.z_m", 0.0)] {
            edit(&mut k, ship, f, serde_json::json!(v), 0);
        }
        edit(&mut k, ship, "pos.x_m", serde_json::json!(5.0e8), 1000);
        let listed = send(&mut k, serde_json::json!({"op":"rules"}));
        assert_eq!(listed["rules"].as_array().unwrap().len(), 2, "{listed}");
        let view = |k: &mut crate::Kernel, t: i64| {
            let s = send(
                k,
                serde_json::json!({"op":"scene","audience":"crew","scope":"u:kestrel","at_s":t}),
            );
            s["scene"]["subjects"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["address"] == tarn)
                .map(|x| x["view"].clone())
                .unwrap()
        };
        let mut steps = vec![];
        for t in [0, 1000, 3000, 4600] {
            let r = send(&mut k, serde_json::json!({"op":"rules_step","at_s":t}));
            assert_eq!(r["type"], "rules_stepped", "{r}");
            steps.push(r);
        }
        assert_eq!(view(&mut k, 3000), "long-range");
        assert_eq!(
            view(&mut k, 4600),
            "scanned",
            "an hour in range: one rung up"
        );
        assert_eq!(steps[3]["report"]["passes"][0][0]["rule"], "sensor-ladder");
        // A step back is refused.
        let back = send(&mut k, serde_json::json!({"op":"rules_step","at_s":10}));
        assert_eq!(back["code"], "rules", "{back}");
        // An insult: proposed, nothing changes until the GM says yes.
        send(
            &mut k,
            serde_json::json!({"op":"add_note","address":tarn,"text":"the crew peed on the statue","tags":["insult"],"by":"crew-seat","audience":["crew"],"at_s":5000}),
        );
        let before = send(&mut k, serde_json::json!({"op":"record","address":tarn}));
        let s = send(&mut k, serde_json::json!({"op":"rules_step","at_s":5100}));
        assert_eq!(s["report"]["passes"][0][0]["outcome"], "proposed", "{s}");
        assert_eq!(
            s["state_hash"], steps[3]["state_hash"],
            "the universe is untouched"
        );
        let after = send(&mut k, serde_json::json!({"op":"record","address":tarn}));
        assert_eq!(before["kept"], after["kept"]);
        let p = send(&mut k, serde_json::json!({"op":"rules_pending"}));
        let id = p["proposals"][0]["id"].as_str().unwrap().to_string();
        let d = send(
            &mut k,
            serde_json::json!({"op":"rules_decide","proposal":id,"approve":true,"at_s":5200}),
        );
        assert_eq!(d["written"].as_array().unwrap().len(), 2, "{d}");
        let rec = send(&mut k, serde_json::json!({"op":"record","address":tarn}));
        let standing = rec["kept"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["kind"] == "politics.standing")
            .unwrap()["value"]
            .clone();
        assert_eq!(standing, serde_json::json!(-1.0));
        let again = send(
            &mut k,
            serde_json::json!({"op":"rules_decide","proposal":id,"approve":true}),
        );
        assert_eq!(again["code"], "rules");
        let pol = send(
            &mut k,
            serde_json::json!({"op":"rules_policy","rule":"consequence","approval":{"policy":"auto"},"at_s":5200}),
        );
        assert_eq!(pol["type"], "rules_policy_set", "{pol}");
        // What the crew's device is sent: results only.
        let disc = send(
            &mut k,
            serde_json::json!({"op":"disclosures","audience":"crew","at_s":6000}),
        );
        let text = disc.to_string();
        let listed = send(&mut k, serde_json::json!({"op":"rules"}));
        for e in listed["trail"].as_array().unwrap() {
            assert!(
                !text.contains(e["id"].as_str().unwrap()),
                "a rules event id reached the crew"
            );
        }
        for word in [
            "rules.",
            "sensor-ladder",
            "consequence",
            "the Vesh remember",
            "politics.standing",
        ] {
            assert!(!text.contains(word), "{word} reached the crew: {text}");
        }
        let export = send(&mut k, serde_json::json!({"op":"export_chronicle"}));
        (steps, d, listed, export)
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "the same session replays to the same bytes");
}

/// Orbits at any time, the campaign clock and a pack calendar (asked for by the chart room).
#[test]
fn orbits_clock_and_calendar() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-time/pack.json"), pack("tests/fixtures/packs/imperial-calendar.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let c = send(&mut k, serde_json::json!({"op": "calendar"}));
    assert_eq!(c["calendar"]["format"], "{day:03}-{year}", "{c}");
    assert_eq!(c["calendar"]["year_days"], 365.0);

    // A calendar whose months do not fill the year is refused at load.
    let mut bad = pack("tests/fixtures/packs/imperial-calendar.json");
    bad["provides"]["other"]["calendar"]["months"] =
        serde_json::json!([{"name": "Only", "days": 30}]);
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [bad]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(r["issues"][0]
        .as_str()
        .unwrap()
        .contains("months add to 30"));

    // No universe: inline bodies still place (no keyed draw without a seed).
    let inline = serde_json::json!([
        {"address": "u:x/sys:0/star:0", "facts": {"body.mass": 1.98847e30}},
        {"address": "u:x/sys:0/body:0", "facts": {"orbit.host": "u:x/sys:0/star:0", "orbit.semi_major_axis": 1.495978707e11, "orbit.mean_anomaly_at_epoch": 0.0}},
        {"address": "u:x/sys:0/body:1", "facts": {"orbit.host": "u:x/sys:0/star:0", "orbit.semi_major_axis": 7.0e11}}
    ]);
    let r = send(
        &mut k,
        serde_json::json!({"op": "orbits_at", "system": "u:x/sys:0", "at_s": [0, 1000], "bodies": inline}),
    );
    assert_eq!(r["type"], "orbits_placed", "{r}");
    assert_eq!(r["bodies"][0]["placed"], true);
    assert_eq!(r["bodies"][1]["placed"], false);
    assert!(r["bodies"][1]["missing"][0]
        .as_str()
        .unwrap()
        .contains("mean_anomaly"));
    let r = send(
        &mut k,
        serde_json::json!({"op": "orbits_at", "system": "u:x/sys:0", "at_s": []}),
    );
    assert_eq!(r["code"], "bad_request");

    // An adopted formation system: every body placed, phases dated, the
    // same answer twice, and the keyed draw when a phase is missing.
    send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-time/pack.json")]}),
    );
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let ask = serde_json::json!({"op": "orbits_at", "system": "u:home/sys:5", "at_s": [0, 86400, 31557600]});
    let a = send(&mut k, ask.clone());
    let b = send(&mut k, ask);
    assert_eq!(a, b);
    let bodies = a["bodies"].as_array().unwrap();
    assert!(!bodies.is_empty(), "{a}");
    for x in bodies {
        assert_eq!(x["placed"], true, "{x}");
        assert_eq!(x["at"].as_array().unwrap().len(), 3);
        assert!(x["period_s"].as_f64().unwrap() > 0.0);
    }
    eprintln!(
        "formation orbits: {:?}",
        bodies
            .iter()
            .map(|x| (
                x["address"].as_str().unwrap(),
                x["period_s"].as_f64().unwrap() / 86400.0,
                x["phase_source"].as_str().unwrap()
            ))
            .collect::<Vec<_>>()
    );

    // An SSE 3 import: dated phases from the save, and a planet a year long.
    let save = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "import_v3": save}),
    );
    let sys = r["systems"][0].as_str().unwrap().to_string();
    let v = send(
        &mut k,
        serde_json::json!({"op": "orbits_at", "system": sys, "at_s": [0]}),
    );
    let days: Vec<(String, f64, String)> = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["placed"] == true)
        .map(|x| {
            (
                x["address"].as_str().unwrap().to_string(),
                x["period_s"].as_f64().unwrap() / 86400.0,
                x["phase_source"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    eprintln!("v3 orbits: {days:?} of {}", v["bodies"]);
    assert!(days.iter().any(|d| (d.1 - 365.25).abs() < 1.0), "{days:?}");

    // The campaign clock: unset, set, moved back, read.
    let n = send(&mut k, serde_json::json!({"op": "campaign_now"}));
    assert!(n.get("at_s").is_none(), "{n}");
    let s1 = send(
        &mut k,
        serde_json::json!({"op": "set_campaign_now", "at_s": 5000}),
    );
    let s2 = send(
        &mut k,
        serde_json::json!({"op": "set_campaign_now", "at_s": 4000}),
    );
    assert_ne!(s1["state_hash"], s2["state_hash"]);
    let n = send(&mut k, serde_json::json!({"op": "campaign_now"}));
    assert_eq!(n["at_s"], 4000);
    assert_eq!(n["event_id"], s2["event_id"]);
}

/// J5 through the API: the registry, a seat plan, built-ins applied by the
/// kernel's own requests, a pack kind queued, approved and taken up by a rule.
#[test]
fn intents_through_the_api() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    // A rule listening for an undeclared intent is refused at load.
    let mut stray = packs["rpg-intents"].clone();
    stray["provides"]["other"]["intents"]["kinds"] = serde_json::json!([]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-record"], stray]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(
        r["issues"][0]
            .as_str()
            .unwrap()
            .contains("intent hail is not declared"),
        "{r}"
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-record"], packs["rpg-intents"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    let reg = send(&mut k, serde_json::json!({"op":"intents"}));
    let kinds: Vec<&str> = reg["kinds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "set_value",
            "adopt_physics",
            "stack_packs",
            "scan_toggle",
            "grant",
            "set_known",
            "undo_known",
            "add_note",
            "set_campaign_now",
            "hail"
        ]
    );
    assert_eq!(reg["seats"][0]["id"], "gm");

    let calla = "u:kestrel/ship:calla";
    let rock = "u:kestrel/sys:0/body:3";
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"gm","at_s":0,
        "intent":{"kind":"set_value","address":calla,"field":"tag.crewed","value":true,"at_yr":0}}),
    );
    assert_eq!(r["decision"]["action"], "apply", "{r}");
    assert_eq!(r["applied_by"], "kernel");
    assert_eq!(
        r["applied"]["type"], "authored",
        "the GM's set_value is set_authored"
    );

    let r = send(
        &mut k,
        serde_json::json!({"op":"set_seats","at_s":0,"seats":[
        {"id":"crew","label":"Crew","audience":"crew","authority":"player","may_send":["hail","set_value"],
         "auto_approve":[{"intent":"set_value","scope":["u:kestrel/sys:0"],"fields":["name"]}],"sealed":false}]}),
    );
    assert_eq!(r["type"], "seats_set", "{r}");
    // Inside the crew's naming scope: applied at once.
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"crew","at_s":10,
        "intent":{"kind":"set_value","address":rock,"field":"name","value":"Grit","at_yr":0}}),
    );
    assert_eq!(
        (r["decision"]["action"].as_str(), r["applied_by"].as_str()),
        (Some("apply"), Some("kernel")),
        "{r}"
    );
    // Hailing a rock is not valid; check_only writes nothing.
    let hail = |a: &str| serde_json::json!({"kind":"hail","address":a,"words":"this is Kestrel"});
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"crew","at_s":20,"intent":hail(rock)}),
    );
    assert_eq!(r["decision"]["action"], "refuse", "{r}");
    let before = send(&mut k, serde_json::json!({"op":"intents"}));
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"crew","at_s":20,"intent":hail(calla),"check_only":true}),
    );
    assert_eq!(r["decision"]["action"], "ask_gm", "{r}");
    assert_eq!(send(&mut k, serde_json::json!({"op":"intents"})), before);
    // Sent for real: queued, then approved, then the rule answers.
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"crew","at_s":20,"intent":hail(calla)}),
    );
    let proposal = r["proposal"].as_str().unwrap().to_string();
    let p = send(&mut k, serde_json::json!({"op":"intents"}));
    assert_eq!(p["pending"][0]["id"], proposal.as_str());
    let r = send(
        &mut k,
        serde_json::json!({"op":"intent_decide","proposal":proposal,"approve":true}),
    );
    assert_eq!(r["applied_by"], "record", "{r}");
    let again = send(
        &mut k,
        serde_json::json!({"op":"intent_decide","proposal":proposal,"approve":true}),
    );
    assert_eq!(again["code"], "unknown_proposal");
    let step = send(&mut k, serde_json::json!({"op":"rules_step","at_s":30}));
    let fired = &step["report"]["passes"][0];
    assert_eq!(fired[0]["rule"], "answer-hail", "{step}");
    assert_eq!(fired[0]["outcome"], "applied");
    let rec = send(&mut k, serde_json::json!({"op":"record","address":calla}));
    assert!(
        rec.to_string()
            .contains("u:kestrel/ship:calla answered the hail"),
        "{rec}"
    );
    // An unknown seat.
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"nobody","at_s":0,"intent":hail(calla)}),
    );
    assert_eq!(r["code"], "unknown_seat");
    // The campaign clock travels as an intent: the GM's applies at once,
    // through set_campaign_now; a bad time is refused.
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"gm","at_s":0,"intent":{"kind":"set_campaign_now","now_s":86400}}),
    );
    assert_eq!(r["decision"]["action"], "apply", "{r}");
    assert_eq!(r["applied"]["type"], "campaign_now_is", "{r}");
    let n = send(&mut k, serde_json::json!({"op": "campaign_now"}));
    assert_eq!(n["at_s"], 86400, "{n}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"send_intent","seat":"gm","at_s":0,"intent":{"kind":"set_campaign_now","now_s":1.5}}),
    );
    assert!(r.to_string().contains("whole number"), "{r}");
}

/// J7 through the API: a ship from a template carries its facts, tags,
/// programs and kept values; a rule wrecks it (create and retire, asked);
/// removing the pack leaves the objects valid.
#[test]
fn templates_through_the_api() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    // A template whose kept kind does not list it as a writer is refused.
    let mut bad = packs["rpg-templates"].clone();
    bad["provides"]["other"]["templates"]["templates"][0]["kept"] =
        serde_json::json!({"ship.cargo_t": 5});
    let r = send(&mut k, serde_json::json!({"op":"load_stack","packs":[bad]}));
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(
        r["issues"]
            .to_string()
            .contains("template raider: kept ship.cargo_t"),
        "{r}"
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-templates"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    let t = send(&mut k, serde_json::json!({"op":"templates"}));
    assert_eq!(t["templates"].as_array().unwrap().len(), 3);

    let sys = "u:kestrel/sys:0";
    let c = send(
        &mut k,
        serde_json::json!({"op":"create","template":"raider","under":sys,"name":"Black Gull","at_s":100}),
    );
    assert_eq!(c["type"], "created", "{c}");
    let ship = c["address"].as_str().unwrap().to_string();
    assert_eq!(ship, "u:kestrel/sys:0/ship:1");
    let c2 = send(
        &mut k,
        serde_json::json!({"op":"create","template":"freighter","under":sys,"at_s":100}),
    );
    assert_eq!(c2["address"], "u:kestrel/sys:0/ship:2");
    assert_eq!(c2["kept"].as_array().unwrap().len(), 2);
    let dup = send(
        &mut k,
        serde_json::json!({"op":"create","template":"freighter","under":sys,"key":"1","at_s":100}),
    );
    assert_eq!(dup["code"], "create_refused");
    let h = send(&mut k, serde_json::json!({"op":"history","address":ship}));
    assert!(h.to_string().contains("template.create"), "{h}");
    let rec = send(&mut k, serde_json::json!({"op":"record","address":ship}));
    assert!(rec.to_string().contains("ship.hull"), "{rec}");

    // Hull to zero: the wreck rule asks (create and retire), then a yes applies both.
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":ship,"kind":"ship.hull","value":0.0,"at_s":200}),
    );
    assert_eq!(r["type"], "kept_set", "{r}");
    let step = send(&mut k, serde_json::json!({"op":"rules_step","at_s":300}));
    assert_eq!(
        step["report"]["passes"][0][0]["outcome"], "proposed",
        "{step}"
    );
    let pending = send(&mut k, serde_json::json!({"op":"rules_pending"}));
    let id = pending["proposals"][0]["id"].as_str().unwrap().to_string();
    let r = send(
        &mut k,
        serde_json::json!({"op":"rules_decide","proposal":id,"approve":true}),
    );
    assert_eq!(r["written"].as_array().unwrap().len(), 2, "{r}");
    let again = send(
        &mut k,
        serde_json::json!({"op":"retire","address":ship,"at_s":400}),
    );
    assert_eq!(again["code"], "retire_refused", "already retired");
    let r = send(
        &mut k,
        serde_json::json!({"op":"retire","address":sys,"at_s":400}),
    );
    assert_eq!(
        r["code"], "retire_refused",
        "only objects made from templates retire"
    );

    // The pack goes; the objects stay readable, with their facts and kept values.
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-record"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let v = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":sys}),
    );
    assert_eq!(v["frame"]["evidence"], "drawn", "{v}");
    let to_g = &v["frame"]["to_galactic"];
    assert_eq!(to_g.as_array().map(|r| r.len()), Some(3), "{v}");
    let f = &v["facts"];
    assert_eq!(f[&ship]["retired"], true, "{v}");
    assert_eq!(f[&ship]["name"], "Black Gull");
    assert_eq!(f[&ship]["template.programs"], serde_json::json!(["pirate"]));
    assert_eq!(f[&ship]["tag.raider"], true);
    assert_eq!(
        f[format!("{ship}/wreck:1")]["name"],
        format!("wreck of {ship}")
    );
    let rec = send(
        &mut k,
        serde_json::json!({"op":"record","address":"u:kestrel/sys:0/ship:2"}),
    );
    assert!(
        rec.to_string().contains("ship.cargo_t") && rec.to_string().contains("not loaded"),
        "{rec}"
    );
}

#[test]
fn sift_through_the_api() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-record"], packs["rpg-sifters"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(&mut k, serde_json::json!({"op":"sift","at_s":0}));
    assert_eq!(r["code"], "no_universe", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    let tarn = "u:kestrel/sys:0/body:3";
    for (field, value) in [
        ("name", serde_json::json!("Tarn")),
        ("mass_kg", serde_json::json!(6.0e24)),
        ("gm.threat", serde_json::json!(3.0)),
    ] {
        let r = send(
            &mut k,
            serde_json::json!({"op":"set_authored","edit":{"address":tarn,"field":field,"value":value,"occurred_at_s":0}}),
        );
        assert!(r["state_hash"].is_string(), "{r}");
    }
    let ids = |r: &serde_json::Value| -> Vec<String> {
        r["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["sifter"].as_str().unwrap().to_string())
            .collect()
    };
    // The GM sees everything, with the events behind the evidence.
    let gm = send(&mut k, serde_json::json!({"op":"sift","at_s":10}));
    assert_eq!(gm["type"], "sifted", "{gm}");
    let mut got = ids(&gm);
    got.sort();
    assert_eq!(got, ["heavy-world", "sleeping-threat"], "{gm}");
    assert!(gm["hits"][0]["evidence"][0]["event"].is_string(), "{gm}");
    assert!(gm.get("skipped").is_none(), "nothing skipped for the GM");
    // Only one sifter, and a missing one.
    let one = send(
        &mut k,
        serde_json::json!({"op":"sift","sifters":["heavy-world"],"scope":tarn,"at_s":10}),
    );
    assert_eq!(ids(&one), ["heavy-world"]);
    let bad = send(
        &mut k,
        serde_json::json!({"op":"sift","sifters":["nope"],"at_s":10}),
    );
    assert_eq!(bad["code"], "unknown_sifter", "{bad}");
    let out = send(
        &mut k,
        serde_json::json!({"op":"sift","scope":"u:elsewhere","at_s":10}),
    );
    assert_eq!(out["code"], "bad_address", "{out}");
    // The crew's long-range view shows no mass and never the GM's secrets.
    let crew = send(
        &mut k,
        serde_json::json!({"op":"sift","audience":"crew","at_s":10}),
    );
    assert_eq!(crew["type"], "sifted", "{crew}");
    assert_eq!(crew["audience"], "crew");
    assert!(ids(&crew).is_empty(), "{crew}");
    assert_eq!(crew["skipped"], serde_json::json!(["granted"]));
    // Scanned: the mass is shown, so the heavy world is a hit; the threat stays secret.
    let s = send(
        &mut k,
        serde_json::json!({"op":"scan_toggle","address":tarn,"audience":"crew","at_s":5}),
    );
    assert!(s["state_hash"].is_string(), "{s}");
    let crew = send(
        &mut k,
        serde_json::json!({"op":"sift","audience":"crew","at_s":10}),
    );
    assert_eq!(ids(&crew), ["heavy-world"], "{crew}");
    let text = crew.to_string();
    assert!(
        !text.contains("gm.threat") && !text.contains("\"event\":\""),
        "{crew}"
    );
    // Before the scan, nothing.
    let early = send(
        &mut k,
        serde_json::json!({"op":"sift","audience":"crew","at_s":4}),
    );
    assert!(ids(&early).is_empty());
}

/// J6: the mining drone surveys, travels, mines and returns; it acts only on
/// what its sensors have shown; a replay gives the same record; the limit holds.
#[test]
fn programs_through_the_api() {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let run = |scan: bool| -> (Vec<serde_json::Value>, String, crate::Kernel) {
        let mut k = crate::Kernel::new();
        let r = send(
            &mut k,
            serde_json::json!({"op":"load_stack","packs":[packs["rpg-programs"]]}),
        );
        assert_eq!(r["type"], "stack_loaded", "{r}");
        send(
            &mut k,
            serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
        );
        let sys = "u:kestrel/sys:0";
        for (b, grade) in [("body:1", 0.9), ("body:2", 0.6), ("body:3", 0.2)] {
            let r = send(
                &mut k,
                serde_json::json!({"op":"set_authored","edit":{"address":format!("{sys}/{b}"),"field":"ore.grade","value":grade,"occurred_at_s":0}}),
            );
            assert!(r["state_hash"].is_string(), "{r}");
        }
        let c = send(
            &mut k,
            serde_json::json!({"op":"create","template":"drone","under":sys,"name":"Pick","at_s":0}),
        );
        let drone = c["address"].as_str().unwrap().to_string();
        let seats = serde_json::json!([{
            "id": "drone-ops", "label": "Drone ops", "audience": "drone-ops", "authority": "player",
            "may_send": ["go", "mine", "unload"], "program": "miner",
            "auto_approve": [{"intent": "go", "scope": [drone]}, {"intent": "mine", "scope": [drone]}, {"intent": "unload", "scope": [drone]}]
        }]);
        let r = send(
            &mut k,
            serde_json::json!({"op":"set_seats","seats":seats,"at_s":0}),
        );
        assert_eq!(r["type"], "seats_set", "{r}");
        let r = send(
            &mut k,
            serde_json::json!({"op":"attach_program","address":drone,"program":"miner","seat":"drone-ops","at_s":0}),
        );
        assert_eq!(r["type"], "program_attached", "{r}");
        if scan {
            // The GM scans the middling body for the drone's operators only.
            let s = send(
                &mut k,
                serde_json::json!({"op":"scan_toggle","address":format!("{sys}/body:2"),"audience":"drone-ops","at_s":5}),
            );
            assert!(s["state_hash"].is_string(), "{s}");
        }
        let mut log = vec![];
        for t in 1..=12 {
            let r = send(
                &mut k,
                serde_json::json!({"op":"programs_step","at_s":t * 100}),
            );
            assert_eq!(r["type"], "programs_stepped", "{r}");
            log.push(r["programs"][0].clone());
        }
        let head = send(&mut k, serde_json::json!({"op":"record"}))["record_hash"].to_string();
        (log, head, k)
    };
    // Nothing shown: the drone waits at the depot, though the GM knows of rich ore.
    let (blind, _, _) = run(false);
    for l in &blind {
        assert_eq!(l["to"], "survey", "{l}");
        assert!(l["intent"].is_null(), "{l}");
    }
    // Body 2 scanned: it goes there (not to body 1, richer but unseen), mines four
    // shifts, comes home, unloads and sets off again.
    let (log, head, mut k) = run(true);
    let states: Vec<&str> = log.iter().map(|l| l["to"].as_str().unwrap()).collect();
    println!("{states:?}");
    assert_eq!(log[0]["picked"], "u:kestrel/sys:0/body:2", "{}", log[0]);
    assert_eq!(
        states,
        [
            "survey", "mine", "mine", "mine", "mine", "return", "unload", "survey", "mine", "mine",
            "mine", "mine"
        ]
    );
    for l in &log {
        assert_eq!(l["applied_by"], "record", "{l}");
    }
    // Replay: the same steps give the same record (a look-ahead branch equals the run).
    let (_, again, _) = run(true);
    assert_eq!(head, again);
    // The limit: two programs at most, and only objects whose template lists the program.
    let rock = send(
        &mut k,
        serde_json::json!({"op":"attach_program","address":"u:kestrel/sys:0/body:2","program":"miner","seat":"drone-ops","at_s":2000}),
    );
    assert_eq!(rock["code"], "program_refused", "{rock}");
    for (n, want) in [(2, "program_attached"), (3, "error")] {
        let c = send(
            &mut k,
            serde_json::json!({"op":"create","template":"drone","under":"u:kestrel/sys:0","key":n.to_string(),"at_s":2000}),
        );
        let r = send(
            &mut k,
            serde_json::json!({"op":"attach_program","address":c["address"],"program":"miner","seat":"drone-ops","at_s":2000}),
        );
        assert_eq!(r["type"], want, "{r}");
        if want == "error" {
            assert_eq!(r["code"], "program_limit");
        }
    }
    let list = send(&mut k, serde_json::json!({"op":"programs"}));
    assert_eq!(list["running"].as_array().unwrap().len(), 2, "{list}");
    assert_eq!(list["limit"], 2);
}

/// `codes`: digits from the stack's code sheet, authored codes win with the
/// physics kept beside them, and counts read the body's own system.
#[test]
fn codes_read_every_body_with_its_why() {
    let l: Vec<serde_json::Value> = contract_lines()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let all: Vec<&serde_json::Value> = l.iter().filter(|r| r["type"] == "codes").collect();
    assert_eq!(all.len(), 3, "two here, one after the sector import");
    let r = &all[0]["readings"];
    let line = |i: usize| {
        r[i]["reading"]["lines"]["code"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(r.as_array().unwrap().len(), 3);
    assert_eq!(
        (line(0), line(1), line(2)),
        ("X800000-2".into(), "X800000-2".into(), "X800000-2".into())
    );
    let size = &r[1]["reading"]["segments"][0]["why"];
    assert_eq!(size["kind"], "authored");
    assert_eq!(size["physics"], "1");
    assert_eq!(all[1]["readings"].as_array().unwrap().len(), 1);
    assert!(l
        .iter()
        .any(|x| x["type"] == "error" && x["code"] == "no_code_sheets"));
    assert!(l
        .iter()
        .any(|x| x["type"] == "error" && x["code"] == "unknown_sheet"));
}

/// The Traveller pack loads on the real base packs and reads UWPs through
/// the kernel.
#[test]
fn traveller_pack_reads_codes_through_the_kernel() {
    let mut k = crate::Kernel::new();
    let pack = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(format!("packs/{n}/pack.json"))).unwrap(),
        )
        .unwrap()
    };
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "load_stack", "packs": [
        pack("base-physics"), pack("base-atlas"), pack("base-time"), pack("traveller")]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "trav", "seed": "t"}));
    for (f, v) in [
        ("body.radius", serde_json::json!(6.371e6)),
        ("atmosphere.surface_pressure", serde_json::json!(1.013e5)),
        ("surface.liquid_fraction", serde_json::json!(0.71)),
    ] {
        let r = send(serde_json::json!({"op": "set_authored", "edit": {
            "address": "u:trav/sys:0/body:3", "field": f, "value": v, "occurred_at_s": 0}}));
        assert_eq!(r["type"], "authored", "{r}");
    }
    let r = send(serde_json::json!({"op": "codes", "scope": "u:trav/sys:0", "at_s": 0}));
    assert_eq!(r["type"], "codes", "{r}");
    assert_eq!(r["sheet"], "traveller.world");
    let uwp = r["readings"][0]["reading"]["lines"]["uwp"]
        .as_str()
        .unwrap();
    assert_eq!(&uwp[1..2], "8", "{uwp}");
    assert_eq!(&uwp[3..4], "7", "71% water reads hydrographics 7: {uwp}");
}

/// `import_sector`: a preview changes nothing; applying writes authored
/// events whose facts read back as the listed codes.
#[test]
fn sector_import_previews_then_applies_as_authored_events() {
    let l: Vec<serde_json::Value> = contract_lines()
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let imps: Vec<&serde_json::Value> = l.iter().filter(|r| r["type"] == "sector_import").collect();
    assert_eq!(imps.len(), 2);
    let preview = imps[0];
    assert!(preview["events"].as_array().unwrap().is_empty());
    assert!(preview.get("state_hash").is_none());
    assert_eq!(preview["import"]["summary"]["refused"], 1);
    let applied = imps[1];
    assert_eq!(applied["events"].as_array().unwrap().len(), 2);
    assert!(applied["state_hash"].is_string());
    let c = l.iter().rfind(|r| r["type"] == "codes").unwrap();
    assert_eq!(c["readings"][0]["address"], "u:toy/sys:0101/body:@1");
    assert_eq!(
        &c["readings"][0]["reading"]["lines"]["code"]
            .as_str()
            .unwrap()[..2],
        "X8"
    );
}

/// The Traveller pack's knowledge ladder: a ship with a spectral analyser
/// that stays within range for an hour lifts a world to "scanned" for the
/// players; a ship without one changes nothing.
#[test]
fn traveller_sensors_move_a_world_up_the_ladder() {
    let mut k = crate::Kernel::new();
    let pack = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(format!("packs/{n}/pack.json"))).unwrap(),
        )
        .unwrap()
    };
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "load_stack", "packs": [
        pack("base-physics"), pack("base-atlas"), pack("base-time"), pack("traveller")]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "trav", "seed": "t"}));
    let world = "u:trav/sys:0/body:3";
    let ship = "u:trav/sys:0/ship:@1";
    for (a, f, v) in [
        (world, "name", serde_json::json!("Ashgrove")),
        (
            world,
            "atmosphere.surface_pressure",
            serde_json::json!(1.0e5),
        ),
        (world, "pos.x_m", serde_json::json!(0.0)),
        (world, "pos.y_m", serde_json::json!(0.0)),
        (world, "pos.z_m", serde_json::json!(0.0)),
        (ship, "tag.ship", serde_json::json!(true)),
        (ship, "crew.audience", serde_json::json!("players")),
        (
            ship,
            "sensor.spectral_1au.range_m",
            serde_json::json!(1.496e11),
        ),
        (ship, "pos.x_m", serde_json::json!(1.0e10)),
        (ship, "pos.y_m", serde_json::json!(0.0)),
        (ship, "pos.z_m", serde_json::json!(0.0)),
    ] {
        let r = send(serde_json::json!({"op": "set_authored", "edit": {
            "address": a, "field": f, "value": v, "occurred_at_s": 0}}));
        assert_eq!(r["type"], "authored", "{r}");
    }
    // Whether the players see the world's air, and through which view.
    let scene = |send: &mut dyn FnMut(serde_json::Value) -> serde_json::Value, at: i64| {
        let r = send(
            serde_json::json!({"op": "scene", "audience": "players", "scope": "u:trav/sys:0", "at_s": at}),
        );
        assert_eq!(r["type"], "scene", "{r}");
        let w = r["scene"]["subjects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["address"] == "u:trav/sys:0/body:3")
            .unwrap()
            .clone();
        let air = w["facts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["field"] == "atmosphere.surface_pressure" && f.get("value").is_some());
        (air, w["view"].as_str().unwrap().to_string())
    };
    assert_eq!(scene(&mut send, 10), (false, "long-range".to_string()));
    let r = send(serde_json::json!({"op": "rules_step", "at_s": 0}));
    assert_ne!(r["type"], "error", "{r}");
    let r = send(serde_json::json!({"op": "rules_step", "at_s": 4000}));
    assert_ne!(r["type"], "error", "{r}");
    assert_eq!(
        scene(&mut send, 5000),
        (true, "scanned".to_string()),
        "scanned after an hour in range"
    );
}

/// The Traveller pack's stock ships and stations are J7 templates: a trader
/// made from the pack carries its own sensors, so once its crew is named it
/// lifts a world up the ladder like any hand-built ship.
#[test]
fn traveller_stock_ships_are_templates_that_sense() {
    let mut k = crate::Kernel::new();
    let pack = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(format!("packs/{n}/pack.json"))).unwrap(),
        )
        .unwrap()
    };
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "load_stack", "packs": [
        pack("base-physics"), pack("base-atlas"), pack("base-time"), pack("traveller")]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "trav", "seed": "t"}));
    let t = send(serde_json::json!({"op": "templates"}));
    let ids: Vec<&str> = t["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect();
    for want in ["courier", "trader", "highport_a", "downport", "scout_base"] {
        assert!(ids.contains(&want), "{want} in {ids:?}");
    }
    let c = send(serde_json::json!({"op": "create", "template": "trader",
        "under": "u:trav/sys:0", "name": "Kestrel Dawn", "at_s": 0}));
    assert_eq!(c["type"], "created", "{c}");
    let ship = c["address"].as_str().unwrap().to_string();
    assert_eq!(ship, "u:trav/sys:0/ship:1");
    let st = send(serde_json::json!({"op": "create", "template": "highport_a",
        "under": "u:trav/sys:0/body:3", "at_s": 0}));
    assert_eq!(st["type"], "created", "{st}");
    assert_eq!(
        st["kept"].as_array().unwrap().len(),
        3,
        "crew aboard, readiness and discipline: {st}"
    );
    // The chart room's Ships and stations screen finds both in the GM scene,
    // each naming its template, with the template's facts.
    let g = send(
        serde_json::json!({"op": "scene", "audience": "gm", "scope": "u:trav/sys:0", "at_s": 0}),
    );
    let subjects = g["scene"]["subjects"].as_array().unwrap();
    for (a, t) in [
        ("u:trav/sys:0/ship:1", "trader"),
        ("u:trav/sys:0/body:3/station:1", "highport_a"),
    ] {
        let s = subjects
            .iter()
            .find(|s| s["address"] == a)
            .unwrap_or_else(|| panic!("{a} in {g}"));
        let fact = |f: &str| {
            s["facts"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["field"] == f)
                .map(|x| x["value"].clone())
        };
        assert_eq!(fact("template.id"), Some(serde_json::json!(t)), "{s}");
        assert!(fact("construct.crew_max").is_some(), "{s}");
    }
    let world = "u:trav/sys:0/body:3";
    for (a, f, v) in [
        (
            world,
            "atmosphere.surface_pressure",
            serde_json::json!(1.0e5),
        ),
        (world, "pos.x_m", serde_json::json!(0.0)),
        (world, "pos.y_m", serde_json::json!(0.0)),
        (world, "pos.z_m", serde_json::json!(0.0)),
        (ship.as_str(), "crew.audience", serde_json::json!("players")),
        (ship.as_str(), "pos.x_m", serde_json::json!(1.0e10)),
        (ship.as_str(), "pos.y_m", serde_json::json!(0.0)),
        (ship.as_str(), "pos.z_m", serde_json::json!(0.0)),
    ] {
        let r = send(serde_json::json!({"op": "set_authored", "edit": {
            "address": a, "field": f, "value": v, "occurred_at_s": 0}}));
        assert_eq!(r["type"], "authored", "{r}");
    }
    send(serde_json::json!({"op": "rules_step", "at_s": 0}));
    send(serde_json::json!({"op": "rules_step", "at_s": 4000}));
    let r = send(
        serde_json::json!({"op": "scene", "audience": "players", "scope": "u:trav/sys:0", "at_s": 5000}),
    );
    let w = r["scene"]["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["address"] == world)
        .unwrap()
        .clone();
    assert_eq!(w["view"], "scanned", "{w}");
}

/// Importing with `grow`: each line's whole system is grown and adopted, the
/// coded facts land on the grown main world, and the codes read back.
#[test]
fn a_sector_imports_as_grown_systems() {
    let mut k = crate::Kernel::new();
    let pack = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(format!("packs/{n}/pack.json"))).unwrap(),
        )
        .unwrap()
    };
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "load_stack", "packs": [
        pack("base-physics"), pack("base-atlas"), pack("base-time"), pack("traveller")]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "kestrel", "seed": "k"}));
    let text =
        std::fs::read_to_string(root().join("tests/fixtures/sector/lanternfall.tab")).unwrap();
    let three: String = text.lines().take(13).map(|l| format!("{l}\n")).collect();
    let r = send(
        serde_json::json!({"op": "import_sector", "text": three, "apply": true, "grow": true, "at_s": 0}),
    );
    assert_eq!(r["type"], "sector_import", "{r}");
    let worlds = r["import"]["worlds"].as_array().unwrap();
    assert_eq!(worlds.len(), 12);
    for w in worlds {
        let main = w["main_world"].as_str().unwrap();
        assert!(!main.ends_with("@1"), "the grown main world: {main}");
        let c = send(serde_json::json!({"op": "codes", "scope": main, "at_s": 0}));
        let uwp = c["readings"][0]["reading"]["lines"]["uwp"]
            .as_str()
            .unwrap()
            .to_string();
        let listed = w["raw"].as_str().unwrap().split('\t').nth(4).unwrap();
        assert_eq!(uwp, listed, "{}", w["name"]);
        let s = send(serde_json::json!({"op": "codes", "scope": w["system"], "at_s": 0}));
        assert!(
            s["readings"].as_array().unwrap().len() >= 2,
            "a whole system: {}",
            w["name"]
        );
    }
    let h = send(serde_json::json!({"op": "systems"}));
    assert!(h.to_string().contains("u:kestrel/sys:0912"), "{h}");
    // And back out: every grown system writes the line it came in on.
    let e = send(
        serde_json::json!({"op": "export_sector", "scope": "u:kestrel", "format": "t5_tab", "at_s": 0}),
    );
    assert_eq!(e["type"], "sector_export", "{e}");
    assert_eq!(e["written"], 12, "{e}");
    let out = e["text"].as_str().unwrap();
    for w in worlds {
        let raw = w["raw"].as_str().unwrap();
        let f: Vec<&str> = raw.split('\t').collect();
        let (hex, uwp) = (f[2], f[4]);
        assert!(
            out.lines()
                .any(|l| l.starts_with(&format!("Kstr\tF\t{hex}\t")) && l.contains(uwp)),
            "{hex} {uwp} in\n{out}"
        );
    }
    // The hex map reads the same worlds, by hex, each with its reading.
    let m = send(serde_json::json!({"op": "sector_map", "scope": "u:kestrel", "at_s": 0}));
    assert_eq!(m["type"], "sector_mapped", "{m}");
    let rows = m["worlds"].as_array().unwrap();
    assert_eq!(rows.len(), 12, "{m}");
    let hexes: Vec<&str> = rows.iter().map(|r| r["hex"].as_str().unwrap()).collect();
    let mut sorted = hexes.clone();
    sorted.sort();
    assert_eq!(hexes, sorted);
    for r in rows {
        assert!(
            r["body"]
                .as_str()
                .unwrap()
                .starts_with(r["system"].as_str().unwrap()),
            "{r}"
        );
        assert!(r["reading"]["lines"]["uwp"].is_string(), "{r}");
    }
}

#[test]
fn sift_finds_story_hooks_with_their_evidence() {
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "s", "seed": "s"}),
    );
    for (a, f, v) in [
        ("u:s/sys:1/body:2", "name", serde_json::json!("Harbour")),
        (
            "u:s/sys:1/body:2",
            "story.rumour",
            serde_json::json!("a ship went down in the ice"),
        ),
        (
            "u:s/sys:1/body:2",
            "settlements.count",
            serde_json::json!(3),
        ),
        (
            "u:s/sys:1/body:2",
            "hazards",
            serde_json::json!("ice quakes"),
        ),
        ("u:s/sys:1/body:3", "name", serde_json::json!("Bare")),
    ] {
        let r = send(
            &mut k,
            serde_json::json!({"op": "set_authored", "edit": {"address": a, "field": f, "value": v, "occurred_at_s": 10}}),
        );
        assert_eq!(r["type"], "authored", "{r}");
    }
    let sifters = serde_json::json!([
        {"id": "rumour", "label": "A rumour to chase", "pattern": {"match": "facts", "when": {"if": "has", "field": "story.rumour"}},
         "significance": {"persistence": {"from": "fixed", "s": 3.15e7}, "reach": {"from": "fixed", "n": 1}, "reversibility": 0.5}},
        {"id": "danger-at-home", "label": "Danger where people live", "pattern": {"match": "facts", "when": {"if": "all", "of": [
            {"if": "has", "field": "hazards"}, {"if": "range", "field": "settlements.count", "min": 1}]}},
         "significance": {"persistence": {"from": "fixed", "s": 3.15e9}, "reach": {"from": "fixed", "n": 1000}, "reversibility": 0.2}}
    ]);
    let r = send(
        &mut k,
        serde_json::json!({"op": "sift", "sifters": sifters}),
    );
    assert_eq!(r["type"], "sifted", "{r}");
    let hits = r["hits"].as_array().unwrap();
    assert_eq!(hits.len(), 2, "{r}");
    // Danger at home lasts longer, reaches more and is harder to undo: it comes first.
    assert_eq!(hits[0]["sifter"], "danger-at-home");
    assert!(hits.iter().all(|h| h["subject"] == "u:s/sys:1/body:2"));
    let ev = hits[0]["evidence"].as_array().unwrap();
    assert!(
        ev.iter()
            .any(|e| e["field"] == "settlements.count" && !e["event"].is_null()),
        "evidence names the event: {r}"
    );
    let bad = send(
        &mut k,
        serde_json::json!({"op": "sift", "sifters": [{"id": "x"}]}),
    );
    assert_eq!(bad["code"], "bad_sifter", "{bad}");
}

/// One clock for the universe (top-level design 12.2 and 17.1): the two named
/// ways to accept a look-ahead, and ageing one system with a reconstructed origin.
#[test]
fn one_clock_and_the_two_ways_to_accept() {
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-time/pack.json")).unwrap(),
    )
    .unwrap();
    let run = |k: &mut crate::Kernel| -> Vec<serde_json::Value> {
        let mut log = vec![];
        send(k, serde_json::json!({"op": "load_stack", "packs": [pack]}));
        send(
            k,
            serde_json::json!({"op": "open_universe", "id": "clock", "seed": "clock"}),
        );
        for (sys, m) in [("u:clock/sys:5", 1.0), ("u:clock/sys:6", 0.8)] {
            let r = send(
                k,
                serde_json::json!({"op": "adopt", "formation": {"seed": sys, "system": sys, "star_pins": {"primary_mass": m}}}),
            );
            assert_eq!(r["type"], "adopted", "{r}");
        }
        let spec = |k: &mut crate::Kernel, s: &str| {
            send(k, serde_json::json!({"op": "resolved_system", "system": s}))["spec"].clone()
        };
        let a0 = spec(k, "u:clock/sys:5");
        let b0 = spec(k, "u:clock/sys:6");
        let (aa, ba) = (
            a0["age_myr"].as_f64().unwrap(),
            b0["age_myr"].as_f64().unwrap(),
        );

        // "Make this system this old, today": only A changes, the clock stays.
        let la = send(
            k,
            serde_json::json!({"op": "look_ahead", "system": a0, "target_myr": aa + 3000.0}),
        );
        let c = send(
            k,
            serde_json::json!({"op": "commit_look_ahead", "branch": la["branch"],
            "accept": "system_age", "move_now_myr": aa + 2000.0, "write_back": true}),
        );
        assert_eq!(c["type"], "committed", "{c}");
        let sa = &c["system_age"];
        assert_eq!(sa["type"], "system_age_set", "{sa}");
        assert!(
            (sa["shift_myr"].as_f64().unwrap() - 2000.0).abs() < 1e-6,
            "{sa}"
        );
        assert!(sa["impossible"].is_null(), "{sa}");
        let o = &sa["origin"];
        assert_eq!(o["label"], "one plausible past");
        assert!(
            o["why"].as_str().unwrap().contains("One plausible past"),
            "{o}"
        );
        assert!(
            c.get("campaign_now_s").is_none(),
            "the clock did not move: {c}"
        );
        let a1 = spec(k, "u:clock/sys:5");
        assert!(
            (a1["age_myr"].as_f64().unwrap() - (aa + 2000.0)).abs() < 1e-6,
            "{a1}"
        );
        assert_eq!(
            a1["birth_year"].as_i64().unwrap(),
            a0["birth_year"].as_i64().unwrap() - 2_000_000_000,
            "born 2 Gyr earlier"
        );
        assert_eq!(
            spec(k, "u:clock/sys:6"),
            b0,
            "the other system is untouched"
        );
        let read = send(
            k,
            serde_json::json!({"op": "resolved_system", "system": "u:clock/sys:5"}),
        );
        assert!(
            read["at_s"].as_i64().is_some(),
            "read at its present, not the kept future: {read}"
        );
        let h = send(
            k,
            serde_json::json!({"op": "history", "address": "u:clock/sys:5"}),
        );
        let kinds: Vec<&str> = h["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["kind"].as_str().unwrap())
            .collect();
        assert!(
            kinds.contains(&"system.origin") && kinds.contains(&"time.request"),
            "{kinds:?}"
        );
        log.push(c);

        // "Make this the campaign's now": the clock moves, so A ages too.
        let la = send(
            k,
            serde_json::json!({"op": "look_ahead", "system": b0, "target_myr": ba + 1000.0}),
        );
        let c = send(
            k,
            serde_json::json!({"op": "commit_look_ahead", "branch": la["branch"],
            "accept": "campaign_now", "move_now_myr": ba + 500.0}),
        );
        assert_eq!(
            c["campaign_now_s"].as_i64().unwrap(),
            500_000_000 * 31_557_600,
            "{c}"
        );
        let now = send(k, serde_json::json!({"op": "campaign_now"}));
        assert_eq!(now["at_s"], c["campaign_now_s"]);
        let a2 = spec(k, "u:clock/sys:5");
        let b2 = spec(k, "u:clock/sys:6");
        assert!(
            (a2["age_myr"].as_f64().unwrap() - (aa + 2500.0)).abs() < 1e-6,
            "{a2}"
        );
        assert!(
            (b2["age_myr"].as_f64().unwrap() - (ba + 500.0)).abs() < 1e-6,
            "{b2}"
        );
        assert_eq!(
            b2["birth_year"], b0["birth_year"],
            "the clock moves, births stay"
        );
        log.push(c);

        // Directly: keep the composition, then an age no galaxy could give.
        let kept = send(
            k,
            serde_json::json!({"op": "set_system_age", "system": "u:clock/sys:6", "age_myr": 9000.0, "composition": "keep"}),
        );
        assert_eq!(kept["type"], "system_age_set", "{kept}");
        assert_eq!(kept["composition"]["mode"], "keep");
        assert_eq!(kept["composition"]["rederive"], false);
        assert_eq!(kept["origin"]["composition"], "kept");
        let follow = send(
            k,
            serde_json::json!({"op": "set_system_age", "system": "u:clock/sys:6", "age_myr": 11000.0}),
        );
        assert_eq!(follow["composition"]["mode"], "follow");
        assert!(
            follow["origin"]["birth_r_kpc"].as_f64().unwrap()
                < follow["origin"]["now_r_kpc"].as_f64().unwrap(),
            "{follow}"
        );
        let b3 = spec(k, "u:clock/sys:6");
        assert!(
            (b3["age_myr"].as_f64().unwrap() - 11000.0).abs() < 1e-6,
            "{b3}"
        );
        if follow["composition"]["rederive"] == true {
            assert!(
                (b3["z"].as_f64().unwrap() - follow["composition"]["z_after"].as_f64().unwrap())
                    .abs()
                    < 1e-12
            );
        }
        let bad = send(
            k,
            serde_json::json!({"op": "set_system_age", "system": "u:clock/sys:6", "age_myr": 20000.0}),
        );
        assert!(
            bad["impossible"].as_str().unwrap().contains("Big Bang"),
            "{bad}"
        );
        let ev = send(
            k,
            serde_json::json!({"op": "resolved_system", "system": "u:clock/sys:6"}),
        );
        assert_eq!(
            ev["evidence"]["u:clock/sys:6"]["system.age_shift_myr"]["evidence"],
            "authored-impossible",
            "{}",
            ev["evidence"]["u:clock/sys:6"]
        );
        // Younger again: allowed, and the stamp goes.
        let young = send(
            k,
            serde_json::json!({"op": "set_system_age", "system": "u:clock/sys:6", "age_myr": 1000.0}),
        );
        assert!(young["impossible"].is_null(), "{young}");
        log.push(young);
        // Refusals.
        let r = send(
            k,
            serde_json::json!({"op": "set_system_age", "system": "u:clock/sys:6", "age_myr": 1000.0, "composition": "swap"}),
        );
        assert_eq!(r["code"], "bad_composition");
        let b4 = spec(k, "u:clock/sys:6");
        let la = send(
            k,
            serde_json::json!({"op": "look_ahead", "system": b4, "target_myr": 2000.0}),
        );
        let r = send(
            k,
            serde_json::json!({"op": "commit_look_ahead", "branch": la["branch"], "accept": "tomorrow"}),
        );
        assert_eq!(r["code"], "bad_accept");
        log.push(send(k, serde_json::json!({"op": "state_hash"})));
        log
    };
    let mut k = crate::Kernel::new();
    let first = run(&mut k);
    let mut k2 = crate::Kernel::new();
    assert_eq!(first, run(&mut k2), "the same session gives the same bytes");
}

#[test]
fn classify_preview_labels_the_campaign_under_a_candidate_stack() {
    let mut k = crate::Kernel::new();
    let time: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-time/pack.json")).unwrap(),
    )
    .unwrap();
    let atlas: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-atlas/pack.json")).unwrap(),
    )
    .unwrap();
    let no_universe = send(
        &mut k,
        serde_json::json!({"op":"classify_preview","packs":[atlas],"scheme":"physical"}),
    );
    assert_eq!(no_universe["code"], "no_universe", "{no_universe}");
    send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[time]}),
    );
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"home","seed":"home"}),
    );
    for (sys, m) in [("u:home/sys:5", 1.0), ("u:home/sys:6", 0.8)] {
        let form =
            serde_json::json!({"seed": sys, "system": sys, "star_pins": {"primary_mass": m}});
        let r = send(&mut k, serde_json::json!({"op":"adopt","formation":form}));
        assert_eq!(r["type"], "adopted", "{r}");
    }
    // A drafted table: worlds by mass alone (the shipped atlas also wants
    // makeup facts that formation does not publish yet).
    let table = serde_json::json!({
        "format": "sse.pack/1", "id": "campaign.tables", "version": "0.1.0",
        "kind": ["game-system"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {
            "axes": [["mass", {"field": "body.mass", "scale": "log", "span": [1.0e20, 1.0e32]}]],
            "schemes": [{"id": "heft", "question": "how heavy?"}],
            "classes": [
                {"id": "heft/world", "scheme": "heft", "kind": "base",
                 "region": [{"axis": "mass", "core": [1.0e20, 1.0e28], "fringe": [1.0e19, 2.0e28]}],
                 "examples": [{"name": "Earth", "values": {"body.mass": 5.97e24}, "expect": "in"}]},
                {"id": "heft/star", "scheme": "heft", "kind": "base",
                 "region": [{"axis": "mass", "core": [1.0e29, 1.0e32], "fringe": [5.0e28, 2.0e32]}],
                 "examples": [{"name": "Sun", "values": {"body.mass": 1.989e30}, "expect": "in"}]}
            ]
        }}
    });
    let base = send(
        &mut k,
        serde_json::json!({"op":"classify_preview","packs":[atlas],"scheme":"physical"}),
    );
    assert_eq!(base["type"], "classify_previewed", "{base}");
    assert_eq!(base["failing_examples"], serde_json::json!([]));
    let atlas = table;
    let before = send(&mut k, serde_json::json!({"op":"state_hash"}));
    let r = send(
        &mut k,
        serde_json::json!({"op":"classify_preview","packs":[atlas],"scheme":"heft"}),
    );
    assert_eq!(r["type"], "classify_previewed", "{r}");
    let bodies = r["bodies"].as_array().unwrap();
    assert!(bodies.len() >= 4, "{r}");
    let labelled = bodies.iter().filter(|b| !b["base"].is_null()).count();
    assert_eq!(labelled, bodies.len(), "{r}");
    for b in bodies {
        let star = b["address"].as_str().unwrap().contains("/star:");
        assert_eq!(
            b["base"],
            if star { "heft/star" } else { "heft/world" },
            "{b}"
        );
    }
    assert_eq!(
        r["unclassified"].as_u64().unwrap() as usize,
        bodies.len() - labelled
    );
    assert_eq!(r["failing_examples"], serde_json::json!([]));
    // Nothing loaded or written; the same ask gives the same answer.
    assert_eq!(send(&mut k, serde_json::json!({"op":"state_hash"})), before);
    assert_eq!(
        send(
            &mut k,
            serde_json::json!({"op":"classify_preview","packs":[atlas],"scheme":"heft"})
        ),
        r
    );
    // Scope narrows to one system.
    let one = send(
        &mut k,
        serde_json::json!({"op":"classify_preview","packs":[atlas],"scheme":"heft","scope":"u:home/sys:6"}),
    );
    let one = one["bodies"].as_array().unwrap();
    assert!(!one.is_empty() && one.len() < bodies.len());
    assert!(one
        .iter()
        .all(|b| b["address"].as_str().unwrap().starts_with("u:home/sys:6")));
    // A draft whose example disagrees with its own regions is reported.
    let mut bad = atlas.clone();
    let classes = bad["provides"]["other"]["classes"].as_array_mut().unwrap();
    let c = classes
        .iter_mut()
        .find(|c| c["id"] == "heft/world")
        .expect("heft/world");
    let ex = &mut c["examples"][0]["expect"];
    *ex = if ex == "in" {
        "out".into()
    } else {
        "in".into()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op":"classify_preview","packs":[bad],"scheme":"heft"}),
    );
    assert_eq!(r["failing_examples"].as_array().unwrap().len(), 1, "{r}");
    let unknown = send(
        &mut k,
        serde_json::json!({"op":"classify_preview","packs":[atlas],"scheme":"nope"}),
    );
    assert_eq!(unknown["code"], "unknown_scheme", "{unknown}");
}

#[test]
fn the_window_says_how_its_frame_sits_in_the_galaxy() {
    let mut k = crate::Kernel::new();
    let v = send(
        &mut k,
        serde_json::json!({"op":"window_view","fixture":"sol","from":"Earth"}),
    );
    let f = &v["view"]["frame"];
    assert_eq!(f["evidence"], "measured", "{v}");
    let pl = f["pole_l_deg"].as_f64().unwrap();
    assert!((pl - 96.38).abs() < 0.01, "{f}");
    // An inline system with no open universe has no keyed frame.
    let nav = serde_json::to_value(sse_nav::scenario::sol()).unwrap();
    let v = send(
        &mut k,
        serde_json::json!({"op":"window_view","nav":nav,"from":"Earth","system":"u:x/sys:1"}),
    );
    assert!(v["view"]["frame"].is_null(), "{v}");
}

/// `read_field` with fixture "solar-system" (MF14a, for the renderer's
/// field lines): six magnetised planets with their measured dipoles, read
/// with the real data sheet's constants.
#[test]
fn read_field_solar_system_fixture() {
    let mut k = crate::Kernel::new();
    let pack = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
    };
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_field", "fixture": "solar-system"}),
    );
    assert_eq!(r["type"], "field_read", "{r}");
    let b = r["bodies"].as_array().unwrap();
    let names: Vec<&str> = b.iter().map(|x| x["name"].as_str().unwrap()).collect();
    assert_eq!(names.len(), 6, "{names:?}");
    let earth = b.iter().find(|x| x["address"] == "sol/earth").unwrap();
    let g10 = earth["field"]["g10_t"].as_f64().unwrap();
    // Earth's g10 is about −29,400 nT (IGRF); the dipole read from the
    // moment and tilt lands within a few per cent.
    assert!((-31e-6..-28e-6).contains(&g10), "{g10}");
    assert!(earth["belts"].is_object(), "{earth}");
    // Parker's spiral at 1 au sits near 45 degrees (Parker 1958).
    let angle = earth["wind_field"]["spiral_angle_deg"].as_f64().unwrap();
    assert!((38.0..52.0).contains(&angle), "{angle}");
    let bt = earth["wind_field"]["magnitude_t"].as_f64().unwrap();
    assert!((3e-9..7e-9).contains(&bt), "{bt}");
    let jupiter = b.iter().find(|x| x["address"] == "sol/jupiter").unwrap();
    assert!(
        jupiter["field"]["g10_t"].as_f64().unwrap() > 3e-4,
        "{jupiter}"
    );
}

/// Gap item 7, "which worlds are shielded, at a glance": `read_field` with
/// `system` reads every planet of an adopted system and gives each a glance.
#[test]
fn read_field_for_a_whole_system() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-time/pack.json"), pack("packs/base-reference/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "formation": form}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_field", "system": "u:home/sys:5"}),
    );
    assert_eq!(r["type"], "field_read", "{r}");
    let bodies = r["bodies"].as_array().unwrap();
    assert!(!bodies.is_empty());
    for b in bodies {
        let g = &b["glance"];
        assert!(b["name"].is_string(), "{b}");
        assert!(g["bubble_m"].as_f64().unwrap() > 0.0, "{g}");
        let by = g["held_off_by"].as_str().unwrap();
        assert!(["field", "air", "surface"].contains(&by), "{g}");
        assert_eq!(g["shielded"], by == "field");
        let f = g["field"].as_str().unwrap();
        assert!(["computed", "none", "unresolved"].contains(&f), "{g}");
        if f == "unresolved" {
            assert!(b["stand_ins"].to_string().contains("MF2"), "{b}");
        }
        // MF1's core history says why a rocky world has no dynamo now.
        if g["dynamo_now"] == false {
            assert!(b["why"].to_string().contains("MF1"), "{b}");
        }
    }
    // Outside the universe, or with nothing open, it says so.
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_field", "system": "u:elsewhere/sys:1"}),
    );
    assert_eq!(r["code"], "bad_address", "{r}");
}

/// RK11 through `detect`: a magnetometer with the sheet's μ0 finds a dark
/// magnetised body, and the setting's range bands label the readings.
#[test]
fn detect_on_the_magnetic_channel_with_range_bands() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "detect",
            "targets": [{"address": "u:t/sys:0/body:2", "distance_m": 1e8, "magnetic_moment_a_m2": 8e22}],
            "sensors": [{"name": "fluxgate", "from_m": 1.0, "to_m": 2.0, "aperture_m": 1.0,
                "method": "magnetometer", "noise_density": 1e-11, "field_clutter": 1e-9, "stare_s": 100.0}],
            "range_bands": [{"id": "close", "label": "close", "range_m": 1e4},
                {"id": "distant", "label": "distant", "range_m": 3e8}]}),
    );
    assert_eq!(r["type"], "detected", "{r}");
    let d = &r["detections"][0];
    assert_eq!(d["channel"], "magnetic", "{d}");
    assert_eq!(d["detectable"], true, "{d}");
    assert_eq!(d["distance_band"], "distant", "{d}");
}

/// X3i (RK11, our share): every Traveller sensor kind carries an instrument
/// the kernel's `detect` reads. Each computes its range from what the target
/// gives off (only the datalink, a handshake, keeps V3's authored range), and
/// each detection is labelled with the pack's own range bands.
#[test]
fn the_traveller_sensors_compute_their_ranges() {
    let mut k = crate::Kernel::new();
    let pack = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
    };
    let traveller = pack("packs/traveller/pack.json");
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let sensors = &traveller["provides"]["other"]["sensors"];
    let kinds = sensors["kinds"].as_array().unwrap();
    assert_eq!(kinds.len(), 12);
    // A 200-tonne ship 1 au from a Sun-like star: sunlit hull, warm radiators,
    // a transponder, a radar cross-section and its mass.
    let au = 1.495978707e11;
    let ship = |d: f64| {
        serde_json::json!({"address": "u:t/sys:0/ship:0", "distance_m": d,
            "conditions": {"address": "u:t/sys:0/ship:0", "radius_m": 15.0, "bond_albedo": 0.3,
                "lit_by": [{"address": "u:t/sys:0/star:0", "luminosity_w": 3.828e26, "t_k": 5772.0, "distance_m": au}],
                "radiators": [{"t_k": 400.0, "area_m2": 200.0}],
                "transmitters": [{"frequency_hz": 3e9, "bandwidth_hz": 1e6, "power_w": 100.0}]},
            "radar_cross_section_m2": 100.0, "mass_kg": 2.0e6})
    };
    let instruments: Vec<serde_json::Value> =
        kinds.iter().map(|s| s["instrument"].clone()).collect();
    let r = send(
        serde_json::json!({"op": "detect", "targets": [ship(1.0e7)], "sensors": instruments,
        "range_bands": sensors["range_bands"]}),
    );
    assert_eq!(r["type"], "detected", "{r}");
    let ds = r["detections"].as_array().unwrap();
    assert_eq!(ds.len(), 12);
    for (s, d) in kinds.iter().zip(ds) {
        let id = s["id"].as_str().unwrap();
        let range = d["range_m"].as_f64().unwrap();
        assert!(range.is_finite() && range > 0.0, "{id}: {d}");
        let want = if id == "datalink" {
            "authored"
        } else {
            "computed"
        };
        assert_eq!(d["range_source"], want, "{id}: {d}");
        assert_eq!(d["authored_range_m"], s["range_m"], "{id}: {d}");
        assert!(d["distance_band"].is_string(), "{id}: {d}");
        eprintln!(
            "{id}: {} reaches {range:.3e} m ({}), V3 typed {:.3e} m",
            d["method"],
            d["range_band"],
            s["range_m"].as_f64().unwrap()
        );
    }
    let by = |id: &str| &ds[kinds.iter().position(|s| s["id"] == id).unwrap()];
    // The radar is heard further than it sees; the radar's power is reported
    // for the power ledger.
    let radar = by("search_radar");
    assert!(radar["heard_to_m"].as_f64().unwrap() > radar["range_m"].as_f64().unwrap());
    assert_eq!(radar["transmit_power_w"], 1e5);
    // A spectrum reaches less far than an image through the same telescope.
    assert!(
        by("spectral_5au")["range_m"].as_f64().unwrap()
            < by("telescope_15au")["range_m"].as_f64().unwrap()
    );
    // The gravimetric grid finds a ship's mass only close by.
    assert!(by("gravimetric")["range_m"].as_f64().unwrap() < 1.0e5);
}

/// ND2: light through gas reads each gas's absorption from the data sheet
/// and gives the colour to the eye before and after.
#[test]
fn light_through_gas_gives_the_colour_to_the_eye() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let pack = |p: &str| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "light_through_gas", "passages": [
            {"address": "sun", "t_k": 5800.0, "layers": []},
            {"address": "giant", "t_k": 5772.0, "path_factor": 2.0,
             "layers": [{"t_k": 80.0, "path_m": 5e4, "gases": [
                 {"gas": "h2", "number_density_m3": 1e25}, {"gas": "ch4", "number_density_m3": 2e22}]}],
             "wavelengths_m": [5.5e-7, 8.9e-7]}]}),
    );
    assert_eq!(r["type"], "gas_passed", "{r}");
    assert_eq!(r["passages"][0]["colour_in"]["hex"], "#FFF1EB", "{r}");
    let g = &r["passages"][1]["passed"];
    assert_eq!(g["wavelengths_m"].as_array().unwrap().len(), 2, "{g}");
    // Methane's bands are read from the sheet; hydrogen has no rows yet,
    // which is said, and it passes the light.
    assert_eq!(g["features"][0]["feature"], "ch4.visible-bands", "{g}");
    assert!(g["not_looked_up"].to_string().contains("h2"), "{g}");
    let out = &r["passages"][1]["colour_out"]["srgb"];
    assert!(out[2].as_u64() > out[0].as_u64(), "{out}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "light_through_gas", "passages": [
            {"layers": [{"t_k": 80.0, "path_m": 1.0, "gases": [{"gas": "unobtainium", "number_density_m3": 1.0}]}], "t_k": 5000.0}]}),
    );
    assert_eq!(r["code"], "bad_gas", "{r}");
}

/// RK12: effect at range gives the reading, the odds from light lag, the
/// rings and a beam's load on the power ledger.
#[test]
fn effect_at_range_reads_a_beam_against_a_skin() {
    let mut k = crate::Kernel::new();
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "effect_at_range", "asks": [{
        "weapon": {"kind": "beam", "beam_power_w": 2.0e7, "wavelength_m": 1.0e-6,
                   "aperture_m": 1.0, "efficiency": 0.3, "reject_k": 600.0},
        "target": {"skin": {"takes_j_m2": 1.0e7}, "radius_m": 10.0, "accel_m_s2": 10.0},
        "distance_m": 1.0e5}]}));
    assert_eq!(r["type"], "effects_at_range", "{r}");
    let e = &r["effects"][0];
    assert_eq!(e["reading"], "in range", "{e}");
    assert_eq!(e["takes_from"], "authored", "{e}");
    assert!(e["rings"]["enough_m"].as_f64().unwrap() > 1.0e5, "{e}");
    let heat = e["load"]["electric_w"].as_f64().unwrap() - 2.0e7;
    assert!((heat / 1.0e6 - 46.667).abs() < 1.0e-3, "{e}");
    assert_eq!(e["odds"], 1.0, "{e}");
    let r = send(serde_json::json!({"op": "effect_at_range", "asks": [{
        "weapon": {"kind": "projectile", "mass_kg": 1.0, "muzzle_m_s": 3000.0, "face_m2": 0.01},
        "target": {"skin": {"material": "unobtainium"}, "radius_m": 10.0},
        "distance_m": 1.0e5}]}));
    assert_eq!(r["code"], "bad_ask", "{r}");
}

/// X3e: the chart room's Packs screen adds the Traveller pack on top of the
/// app's opening stack (no atlas pack), so it must load there and read codes.
#[test]
fn the_traveller_pack_stacks_on_the_apps_opening_packs() {
    let mut k = crate::Kernel::new();
    let pack = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(format!("packs/{n}/pack.json"))).unwrap(),
        )
        .unwrap()
    };
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "load_stack", "packs": [
        pack("base-physics"), pack("base-time"), pack("base-reference"), pack("traveller")]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "k", "seed": "k"}));
    let m = send(serde_json::json!({"op": "sector_map", "scope": "u:k", "at_s": 0}));
    assert_eq!(m["type"], "sector_mapped", "{m}");
    assert_eq!(m["sheet"], "traveller.world", "{m}");
}

/// TG9 (record-logic-programs design 8.1): readiness and discipline are kept
/// values with template defaults; status tags set readiness only through the
/// pack's visible triggers; the GM can set readiness otherwise with the tag
/// left in place; and no pack rule reads a number from a tag.
#[test]
fn ship_readiness_follows_status_tags_through_visible_triggers() {
    let mut k = crate::Kernel::new();
    let pack = |n: &str| -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(root().join(format!("packs/{n}/pack.json"))).unwrap(),
        )
        .unwrap()
    };
    let traveller = pack("traveller");
    let mut send = |v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let r = send(serde_json::json!({"op": "load_stack", "packs": [
        pack("base-physics"), pack("base-time"), traveller.clone()]}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(serde_json::json!({"op": "open_universe", "id": "tg9", "seed": "t"}));
    let make = |send: &mut dyn FnMut(serde_json::Value) -> serde_json::Value, t: &str| {
        let c = send(
            serde_json::json!({"op": "create", "template": t, "under": "u:tg9/sys:0", "at_s": 0}),
        );
        assert_eq!(c["type"], "created", "{c}");
        c["address"].as_str().unwrap().to_string()
    };
    let kept =
        |send: &mut dyn FnMut(serde_json::Value) -> serde_json::Value, a: &str, kind: &str| {
            let r = send(serde_json::json!({"op": "record", "address": a}));
            r["kept"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["kind"] == kind)
                .map(|x| x["value"].as_f64().unwrap())
        };
    let patrol = make(&mut send, "patrol");
    let trader = make(&mut send, "trader");
    assert_eq!(kept(&mut send, &patrol, "ship.readiness"), Some(1.0));
    assert_eq!(kept(&mut send, &patrol, "ship.discipline"), Some(0.9));
    assert_eq!(kept(&mut send, &trader, "ship.discipline"), Some(0.6));
    let tag = |send: &mut dyn FnMut(serde_json::Value) -> serde_json::Value,
               a: &str,
               t: &str,
               at: i64| {
        let r = send(serde_json::json!({"op": "set_authored", "edit": {
            "address": a, "field": format!("tag.{t}"), "value": true, "occurred_at_s": at}}));
        assert_eq!(r["type"], "authored", "{r}");
    };
    let step = |send: &mut dyn FnMut(serde_json::Value) -> serde_json::Value, at: i64| {
        let r = send(serde_json::json!({"op": "rules_step", "at_s": at}));
        assert_ne!(r["type"], "error", "{r}");
    };
    let set = |send: &mut dyn FnMut(serde_json::Value) -> serde_json::Value,
               a: &str,
               v: f64,
               at: i64| {
        let r = send(
            serde_json::json!({"op": "set_kept", "address": a, "kind": "ship.readiness", "value": v, "at_s": at}),
        );
        assert_ne!(r["type"], "error", "{r}");
    };
    // Derelict: readiness 0, once; the GM may then set it otherwise, tag in place.
    tag(&mut send, &patrol, "status.derelict", 10);
    step(&mut send, 10);
    assert_eq!(kept(&mut send, &patrol, "ship.readiness"), Some(0.0));
    let r = send(serde_json::json!({"op": "record", "address": patrol}));
    let last = r["kept"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["kind"] == "ship.readiness")
        .and_then(|x| x["history"].as_array()?.last().cloned())
        .unwrap();
    assert_eq!(
        last["writer"], "effect.traveller.status.derelict",
        "the cause shows: {last}"
    );
    set(&mut send, &patrol, 0.7, 20);
    step(&mut send, 30);
    assert_eq!(
        kept(&mut send, &patrol, "ship.readiness"),
        Some(0.7),
        "quietly made spaceworthy"
    );
    // Damaged: held at 0.5 or below while the tag is on.
    tag(&mut send, &trader, "status.damaged", 40);
    step(&mut send, 40);
    assert_eq!(kept(&mut send, &trader, "ship.readiness"), Some(0.5));
    set(&mut send, &trader, 0.3, 50);
    step(&mut send, 50);
    assert_eq!(
        kept(&mut send, &trader, "ship.readiness"),
        Some(0.3),
        "below the cap stays"
    );
    // No rule in the pack reads a number from a tag: tag conditions are only `tag` tests.
    fn numeric_on_tag(p: &serde_json::Value) -> bool {
        match p {
            serde_json::Value::Object(m) => {
                let field_is_tag = ["field", "left", "right", "a", "b"].iter().any(|k| {
                    m.get(*k)
                        .and_then(|v| v.as_str())
                        .is_some_and(|f| f.starts_with("tag."))
                });
                field_is_tag || m.values().any(numeric_on_tag)
            }
            serde_json::Value::Array(a) => a.iter().any(numeric_on_tag),
            _ => false,
        }
    }
    let other = &traveller["provides"]["other"];
    assert!(
        !numeric_on_tag(&other["triggers"]),
        "a trigger reads a tag as a number"
    );
    assert!(
        !numeric_on_tag(&other["programs"]),
        "a program reads a tag as a number"
    );
}
