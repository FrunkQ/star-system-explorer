//! ER1b: the reach gate (edits-reach design sections 7 and 12).
//!
//! Every editable number on the stack is nudged by one part in a thousand:
//! every constant of nature and of the Cosmos group, every material value on
//! the data sheet, and every non-whole number in every settings block. The
//! area contract cases are then run twice, on the shipped stack and on the
//! nudged one. An op whose answers do not move did not reach the edit.
//!
//! The ledger `handover/packages/ER-reach-ledger.txt` lists, one op a line:
//!   `still <op>`     runs, reads physics, and does not move: a side door
//!   `quiet <op>`     runs and reads no physics, so it should not move
//!   `untested <op>`  no area case runs it yet
//! The test fails when an op that does not move is not on the ledger as
//! still or quiet (a new side door), when an op on the ledger as still or
//! quiet now moves (take its line off), and when an op's untested line is
//! wrong either way. So the ledger only shrinks: fix a side door or add an
//! area case and delete the line. `SSE_REACH_WRITE=1` writes the ledger from
//! scratch (once, for ER1b); `SSE_REACH_SHRINK=1` only deletes stale lines
//! (and turns an untested op that now has a case but does not follow into
//! `still`, which changes the kind of a line, never the count).

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(p: &str) -> Value {
    let path = root().join(p);
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{p}: {e}")))
        .unwrap_or_else(|e| panic!("{p}: {e}"))
}

const LEDGER: &str = "handover/packages/ER-reach-ledger.txt";

/// The shipped stack the gate runs on: the frozen base-physics of the
/// contract, the master data sheet, and every base pack that provides a
/// settings block a kernel op could read.
const STACK: &[&str] = &[
    "packs/base-physics/pack.json",
    "packs/base-reference/pack.json",
    "packs/base-climate/pack.json",
    "packs/base-surface/pack.json",
    "packs/base-weather/pack.json",
    "packs/base-nav/pack.json",
    "packs/base-light/pack.json",
    "packs/base-small-bodies/pack.json",
    "packs/base-cosmology/pack.json",
    "packs/base-cosmos/pack.json",
    "packs/base-life/pack.json",
    "packs/base-time/pack.json",
    "packs/base-place/pack.json",
];

const NUDGE: f64 = 1.001;

/// Scale every number written as a real (not a whole-number count) by the nudge.
fn nudge_numbers(v: &mut Value) {
    match v {
        Value::Number(n) => {
            if let Some(x) = n.as_f64() {
                if n.is_f64() && x != 0.0 {
                    *v = json!(x * NUDGE);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(nudge_numbers),
        Value::Object(o) => {
            for (k, x) in o.iter_mut() {
                if !COUNTS.contains(&k.as_str()) {
                    nudge_numbers(x);
                }
            }
        }
        _ => {}
    }
}

/// Keys whose numbers are counts or bookkeeping that must balance, not
/// physics a GM would tune: a reaction's terms, a formula's atoms.
const COUNTS: &[&str] = &["terms", "formula", "charge", "electrons"];

/// The nudged copy of a pack: its settings blocks' numbers, and on the data
/// sheet every constant that is not a unit or convention, and every value
/// (`v`) of the other tables.
fn nudged(pack: &Value) -> Value {
    let mut p = pack.clone();
    let provides = p.get_mut("provides").and_then(Value::as_object_mut);
    let Some(provides) = provides else { return p };
    if let Some(other) = provides.get_mut("other").and_then(Value::as_object_mut) {
        for (k, v) in other.iter_mut() {
            if sse_physics::is_physics_key(k) {
                nudge_numbers(v);
            }
        }
    }
    if let Some(r) = provides.get_mut("reference").and_then(Value::as_object_mut) {
        for (table, rows) in r.iter_mut() {
            if matches!(
                table.as_str(),
                "about" | "sources" | "groups" | "units" | "origin_legend" | "bands" | "reactions"
            ) {
                continue;
            }
            for row in rows.as_array_mut().into_iter().flatten() {
                let group = row["group"].as_str().unwrap_or("").to_string();
                let id = row["id"].as_str().unwrap_or("").to_string();
                if table == "constants" && matches!(group.as_str(), "convention" | "factor") {
                    // A unit never moves (Alex, 26 Nov, edits-reach question
                    // 1). The sheet makes the solar, Earth and Jupiter mass
                    // units from GM / G, so each GM is nudged with G and the
                    // mass units stay where they are.
                    if id.starts_with("GM_") {
                        nudge_v(row);
                    }
                    continue;
                }
                if table == "constants" && id == "c" {
                    // Not c: the light year unit is made from c, and a unit
                    // never moves. ER4 gives it its IAU value.
                    continue;
                }
                nudge_v(row);
            }
        }
    }
    p
}

/// Scale every `v` that is a non-whole number inside `row`.
fn nudge_v(row: &mut Value) {
    match row {
        Value::Object(o) => {
            for (k, x) in o.iter_mut() {
                // A triple point is checked against the phase table's own
                // point, which is a fit, not a `v`: nudging one refuses the sheet.
                if k.starts_with("triple_point") {
                    continue;
                }
                if k == "v" {
                    nudge_numbers(x);
                } else {
                    nudge_v(x);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(nudge_v),
        _ => {}
    }
}

/// A sheet value the load checks say is derived: `constant X: typed as A
/// but ... gives B` sets X to B on the nudged sheet.
fn repair(packs: &mut [Value], issues: &[String]) -> bool {
    let mut fixed = false;
    for m in issues {
        let Some(rest) = m.split("constant ").nth(1) else {
            continue;
        };
        let Some((id, rest)) = rest.split_once(": typed as ") else {
            continue;
        };
        let Some(gives) = rest.rsplit(" gives ").next() else {
            continue;
        };
        let Ok(b) = gives.trim().parse::<f64>() else {
            continue;
        };
        for p in packs.iter_mut() {
            let Some(rows) = p
                .get_mut("provides")
                .and_then(|x| x.get_mut("reference"))
                .and_then(|x| x.get_mut("constants"))
                .and_then(Value::as_array_mut)
            else {
                continue;
            };
            for row in rows {
                if row["id"] == id {
                    if let Some(v) = row.get_mut("value").and_then(|x| x.get_mut("v")) {
                        *v = json!(b);
                        fixed = true;
                    }
                }
            }
        }
    }
    fixed
}

/// Drop what an edit always moves and is not a reach: hashes and ids.
fn masked(v: &Value) -> Value {
    match v {
        Value::Object(o) => Value::Object(
            o.iter()
                .filter(|(k, _)| !k.contains("hash"))
                .map(|(k, x)| (k.clone(), masked(x)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(masked).collect()),
        _ => v.clone(),
    }
}

/// Run every area case with each `load_stack` given `stack` instead; the
/// masked responses by op.
fn run_areas(stack: &[Value]) -> Vec<(String, String, Value)> {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let dir = root().join("tests/contract/areas");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut out = vec![];
    for f in files {
        let area = f.file_stem().unwrap().to_string_lossy().to_string();
        let mut reqs: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        for r in reqs.as_array_mut().unwrap() {
            if r["op"] == "load_stack" {
                r["packs"] = Value::Array(stack.to_vec());
            }
        }
        let lines = crate::contract::run(&reqs, &packs);
        for (r, line) in reqs.as_array().unwrap().iter().zip(lines) {
            let op = r["op"].as_str().unwrap_or("?").to_string();
            let v: Value = serde_json::from_str(&line).unwrap();
            out.push((area.clone(), op, masked(&v)));
        }
    }
    out
}

fn load_ok(stack: &[Value]) -> Result<(), Vec<String>> {
    let mut k = crate::Kernel::new();
    let r: Value = serde_json::from_str(
        &k.handle_json(&json!({"op": "load_stack", "packs": stack}).to_string()),
    )
    .unwrap();
    if r["type"] == "stack_loaded" {
        return Ok(());
    }
    let issues: Vec<String> = r["issues"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i.as_str().map(String::from))
        .collect();
    if issues.is_empty() {
        return Err(vec![r.to_string()]);
    }
    Err(issues)
}

/// The gate's finding: by op, moved or not; and the ops no case runs.
fn findings() -> (BTreeMap<String, bool>, BTreeSet<String>) {
    let base: Vec<Value> = STACK.iter().map(|p| read(p)).collect();
    load_ok(&base).unwrap_or_else(|e| panic!("the shipped stack loads: {e:?}"));
    let mut edited: Vec<Value> = base.iter().map(nudged).collect();
    let mut tries = 0;
    while let Err(issues) = load_ok(&edited) {
        tries += 1;
        assert!(
            tries < 6 && repair(&mut edited, &issues),
            "the nudged stack loads: {issues:#?}"
        );
    }
    let a = run_areas(&base);
    let b = run_areas(&edited);
    assert_eq!(a.len(), b.len());
    let mut moved: BTreeMap<String, bool> = BTreeMap::new();
    for ((_, op, x), (_, _, y)) in a.iter().zip(&b) {
        // A case that is refused on the shipped stack reads no physics.
        if x["type"] == "error" {
            continue;
        }
        let m = moved.entry(op.clone()).or_insert(false);
        *m |= x != y;
    }
    moved.remove("load_stack");
    let all: BTreeSet<String> = crate::protocol::ops().into_iter().map(|o| o.op).collect();
    let untested = all
        .into_iter()
        .filter(|o| !moved.contains_key(o) && o != "load_stack")
        .collect();
    (moved, untested)
}

#[test]
fn every_op_that_reads_physics_moves_when_the_physics_is_nudged() {
    let (moved, untested) = findings();
    let path = root().join(LEDGER);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut ledger: BTreeMap<String, String> = BTreeMap::new();
    let mut head = vec![];
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            head.push(line.to_string());
            continue;
        }
        let (state, op) = t
            .split_once(' ')
            .unwrap_or_else(|| panic!("bad ledger line {t:?}"));
        assert!(
            matches!(state, "still" | "quiet" | "untested"),
            "bad ledger line {t:?}"
        );
        let op = op.split('#').next().unwrap_or("").trim();
        ledger.insert(op.to_string(), state.to_string());
    }
    if std::env::var("SSE_REACH_WRITE").is_ok() {
        let mut out = vec![
            "# ER1b reach gate ledger (crates/sse-kernel/src/reach_gate_tests.rs). It only shrinks.".to_string(),
            "# still <op>: reads physics and does not follow a nudged stack (a side door).".to_string(),
            "# quiet <op>: reads no physics, so it should not move (say why after a #).".to_string(),
            "# untested <op>: no area contract case runs it yet.".to_string(),
        ];
        for (op, m) in &moved {
            if !m {
                let state = ledger
                    .get(op)
                    .map(String::as_str)
                    .filter(|s| *s == "quiet")
                    .unwrap_or("still");
                out.push(format!("{state} {op}"));
            }
        }
        for op in &untested {
            out.push(format!("untested {op}"));
        }
        std::fs::write(&path, out.join("\n") + "\n").unwrap();
        return;
    }
    let mut bad = vec![];
    let mut stale = BTreeSet::new();
    let mut to_still = BTreeSet::new();
    for (op, m) in &moved {
        match (ledger.get(op).map(String::as_str), m) {
            (None, false) => bad.push(format!(
                "NEW SIDE DOOR: {op} does not move when every physics number is nudged; read it through sse-physics"
            )),
            (Some("still" | "quiet"), true) => {
                stale.insert(op.clone());
                bad.push(format!("{op} now follows the physics: take its line off {LEDGER}"));
            }
            (Some("untested"), true) => {
                stale.insert(op.clone());
                bad.push(format!("{op} now has an area case and follows: take its untested line off {LEDGER}"));
            }
            (Some("untested"), false) => {
                to_still.insert(op.clone());
                bad.push(format!(
                    "{op} now has an area case and does not follow: its untested line becomes `still {op}` (or `quiet {op} # why` if it reads no physics)"
                ));
            }
            _ => {}
        }
    }
    for op in &untested {
        if ledger.get(op).map(String::as_str) != Some("untested") {
            bad.push(format!(
                "{op} has no area contract case: add one in tests/contract/areas/ (an untested line is never added)"
            ));
        }
    }
    for op in ledger.keys() {
        if !moved.contains_key(op) && !untested.contains(op) {
            stale.insert(op.clone());
            bad.push(format!(
                "{op} is not an op any more: take its line off {LEDGER}"
            ));
        }
    }
    if std::env::var("SSE_REACH_SHRINK").is_ok() && !(stale.is_empty() && to_still.is_empty()) {
        let kept: Vec<String> = text
            .lines()
            .map(|l| match l.split_whitespace().collect::<Vec<_>>()[..] {
                ["untested", op] if to_still.contains(op) => format!("still {op}"),
                _ => l.to_string(),
            })
            .filter(|l: &String| {
                let t = l.trim();
                t.is_empty()
                    || t.starts_with('#')
                    || !stale.contains(t.split_whitespace().nth(1).unwrap_or(""))
            })
            .collect();
        std::fs::write(&path, kept.join("\n") + "\n").unwrap();
        return;
    }
    assert!(bad.is_empty(), "reach gate:\n{}", bad.join("\n"));
}
