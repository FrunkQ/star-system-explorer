//! Contract test runner shared by the native test and the CLI. The JavaScript
//! runner (`packages/kernel-client/test/contract.test.ts`) does exactly the
//! same expansion, sends the same requests through the worker, and must get
//! byte-identical responses (`tests/golden/contract.jsonl`).
//!
//! Expansion rules for `tests/contract/requests.json`:
//! - a string `"$pack:<name>"` becomes the JSON of `tests/fixtures/packs/<name>.json`;
//! - the string `"$branch"` becomes the id from the latest `branch_opened` response.

use serde_json::Value;
use std::collections::BTreeMap;

fn expand(v: &Value, packs: &BTreeMap<String, Value>, branch: &str) -> Value {
    match v {
        Value::String(s) if s == "$branch" => Value::String(branch.to_string()),
        Value::String(s) if s.starts_with("$pack:") => packs
            .get(&s[6..])
            .cloned()
            .unwrap_or_else(|| panic!("no fixture pack {s}")),
        Value::Array(a) => Value::Array(a.iter().map(|x| expand(x, packs, branch)).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, x)| (k.clone(), expand(x, packs, branch)))
                .collect(),
        ),
        _ => v.clone(),
    }
}

/// Run every request on a fresh kernel; return the response lines.
pub fn run(requests: &Value, packs: &BTreeMap<String, Value>) -> Vec<String> {
    let mut k = crate::Kernel::new();
    let mut branch = String::new();
    let mut out = Vec::new();
    for r in requests.as_array().expect("requests is an array") {
        let req = serde_json::to_string(&expand(r, packs, &branch)).unwrap();
        let resp = k.handle_json(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        if parsed["type"] == "branch_opened" {
            branch = parsed["branch"].as_str().unwrap().to_string();
        }
        out.push(resp);
    }
    out
}

/// What the golden keeps of a response line. Two lines list every op, so
/// every new op used to change them and golden-changing PRs queued behind
/// each other: the `hello` op list and the unknown-op message. The golden
/// keeps the rest of those lines byte for byte; the lists themselves are
/// checked against `protocol::ops()` (test `the_op_lists_are_every_op`).
/// The JavaScript runner (`packages/kernel-client/src/contract.ts`,
/// `maskOps`) masks the same way.
pub fn mask_ops(line: &str) -> String {
    if line.starts_with(HELLO) {
        if let (Some(s), Some(e)) = (line.find(OPS_FROM), line.find(OPS_TO)) {
            if s < e {
                return format!("{}{OPS_MASK}{}", &line[..s], &line[e + 1..]);
            }
        }
    }
    if let Some(s) = line.find(UNKNOWN_FROM) {
        let s = s + UNKNOWN_FROM.len();
        if let Some(e) = line[s..].find(UNKNOWN_TO) {
            return format!("{}{UNKNOWN_MASK}{}", &line[..s], &line[s + e..]);
        }
    }
    line.to_string()
}

pub const HELLO: &str = "{\"type\":\"hello\"";
pub const OPS_FROM: &str = "\"ops\":[";
pub const OPS_TO: &str = "],\"tiers\":";
pub const OPS_MASK: &str = "\"ops\":\"every op, checked against ops()\"";
pub const UNKNOWN_FROM: &str = "unknown variant `nonsense`, expected one of ";
pub const UNKNOWN_TO: &str = " at line ";
pub const UNKNOWN_MASK: &str = "every op, checked against ops()";

/// Load the fixture packs from a directory of `<name>.json` files.
pub fn load_packs(dir: &std::path::Path) -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("packs dir")
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for p in entries {
        if p.extension().is_some_and(|e| e == "json") {
            let name = p.file_stem().unwrap().to_string_lossy().to_string();
            m.insert(
                name,
                serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap(),
            );
        }
    }
    m
}
