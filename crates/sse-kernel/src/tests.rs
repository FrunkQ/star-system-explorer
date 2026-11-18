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
