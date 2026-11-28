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

/// A baked table names the physics in force by its hash, which every new or
/// edited block anywhere moves; the golden keeps the rest of the line, and
/// masks the hash only when it is one (64 hex digits). That a new physics
/// bakes afresh is checked in `bake_tests`.
const BAKED: &str = r#"{"type":"baked""#;
const PHYSICS_FROM: &str = r#""physics":""#;
const PHYSICS_MASK: &str = "the physics in force";

/// What the golden keeps of a response line. Two lines list every op, so
/// every new op used to change them and golden-changing PRs queued behind
/// each other: the `hello` op list and the unknown-op message. The golden
/// keeps the rest of those lines byte for byte; the lists themselves are
/// checked against `protocol::ops()` (test `the_op_lists_are_every_op`).
/// The JavaScript runner (`packages/kernel-client/src/contract.ts`,
/// `maskOps`) masks the same way.
pub fn mask_ops(line: &str) -> String {
    if line.starts_with(BAKED) {
        if let Some(s) = line.find(PHYSICS_FROM) {
            let s = s + PHYSICS_FROM.len();
            let hex = line[s..].bytes().take_while(u8::is_ascii_hexdigit).count();
            if hex == 64 && line[s + 64..].starts_with('"') {
                return format!("{}{PHYSICS_MASK}{}", &line[..s], &line[s + 64..]);
            }
        }
    }
    if line.starts_with(HELLO) {
        if let (Some(s), Some(e)) = (line.find(OPS_FROM), line.find(OPS_TO)) {
            if s < e {
                return format!("{}{OPS_MASK}{}", &line[..s], &line[e + 1..]);
            }
        }
    }
    if line.starts_with(REACH) {
        if let Some(s) = line.find(REACH_FROM) {
            if line.ends_with("]}") {
                return format!("{}{REACH_MASK}}}", &line[..s]);
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

/// The constants reach report lists every block, so every new block used to
/// change its line. The golden keeps the line's head; the lists go one entry
/// per block to `tests/golden/reach/<constant>/<block>.txt` (`reach_entries`,
/// checked by `check_reach_golden`), where a new block is one new file. The
/// mask text still names the old one-file `contract-reach.txt` because it sits
/// in the core golden; change it with the next core golden change.
pub const REACH: &str = "{\"type\":\"reach\"";
pub const REACH_FROM: &str = "\"follows\":[";
pub const REACH_MASK: &str = "\"lists\":\"one line per block in tests/golden/contract-reach.txt\"";

/// A reach report's lists, one entry per line: `<constant> <list> <entry>`.
pub fn reach_entries(line: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return vec![];
    };
    let c = v["constant"].as_str().unwrap_or("?");
    let mut out = vec![];
    for list in ["follows", "estimated", "cannot_follow"] {
        for e in v[list].as_array().into_iter().flatten() {
            out.push(format!("{c} {list} {e}"));
        }
    }
    out
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
