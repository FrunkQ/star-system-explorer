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
