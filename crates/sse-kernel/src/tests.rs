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
    assert_eq!(gm["skipped"], serde_json::json!([]));
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
