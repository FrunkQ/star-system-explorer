//! K13 `preview_tree` (editing design 10A, R1): the family tree of a system
//! before and after some edits, with each body's verdict, without writing
//! anything. The edits run on a copy of the universe, which is then put
//! back, so any edit op previews the same way it applies.

use crate::orbits;
use crate::stability;
use sse_chronicle::State;
use sse_core::cbor::Value;
use sse_core::{Address, Seed};
use std::collections::BTreeMap;

type Facts = BTreeMap<Address, BTreeMap<String, Value>>;

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// The tree of `system` in `state` at `at_s`: one row per body with its
/// host, kind, name, period, whether it and its host are drawn as a pair
/// (their centre lies outside the host's surface), and its worst verdict
/// with what said so; then the whole system's verdict.
pub fn tree(state: &State, system: &Address, at_s: i64, seed: Option<&Seed>) -> serde_json::Value {
    let facts: Facts = state
        .facts
        .iter()
        .filter(|(a, _)| system.contains(a))
        .map(|(a, r)| (a.clone(), r.clone()))
        .collect();
    let placed = orbits::orbits_at(&facts, system, &[at_s], seed);
    let (checks, _) = stability::checks(&facts, system);
    let mut worst: BTreeMap<String, (u8, Vec<String>)> = BTreeMap::new();
    for c in &checks {
        for b in &c.bodies {
            let w = worst.entry(b.clone()).or_insert((0, vec![]));
            if c.level > 0 {
                w.1.push(format!("{}: {}", c.check, c.says));
            }
            w.0 = w.0.max(c.level);
        }
    }
    let by: BTreeMap<String, &orbits::Placed> =
        placed.iter().map(|p| (p.address.clone(), p)).collect();
    let mut rows = vec![];
    let mut level = 0u8;
    for (a, row) in &facts {
        if !(row.contains_key("body.mass")
            || row.contains_key("orbit.semi_major_axis")
            || row.contains_key("body.kind"))
        {
            continue;
        }
        let key = a.to_string();
        let p = by.get(&key);
        let host = p.and_then(|p| p.parent.clone());
        let mass = row.get("body.mass").and_then(num);
        let pair = match (p, host.as_ref().and_then(|h| h.parse::<Address>().ok())) {
            (Some(p), Some(h)) if p.placed => {
                let hr = facts.get(&h);
                let hm = hr.and_then(|r| r.get("body.mass")).and_then(num);
                let rad = hr.and_then(|r| r.get("body.radius")).and_then(num);
                let sep = p.at.first().map(|x| {
                    let r = x.r_m;
                    sse_core::math::sqrt(r[0] * r[0] + r[1] * r[1] + r[2] * r[2])
                });
                match (hm, mass, rad, sep) {
                    (Some(hm), Some(m), Some(rad), Some(sep)) if hm + m > 0.0 => {
                        sep * m / (hm + m) > rad
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        let (lv, says) = worst.get(&key).cloned().unwrap_or((0, vec![]));
        let unplaced = p.is_some_and(|p| !p.placed);
        level = level.max(lv);
        rows.push(serde_json::json!({
            "address": key,
            "name": crate::names::typed_common(row),
            "kind": crate::names::kind_of(state, a),
            "host": host,
            "period_s": p.and_then(|p| p.period_s),
            "pair": pair,
            "verdict": if unplaced { "unplaced" } else { stability::verdict_word(lv) },
            "says": says,
            "missing": p.map(|p| p.missing.clone()).unwrap_or_default(),
        }));
    }
    serde_json::json!({
        "system": system.to_string(),
        "at_s": at_s,
        "verdict": stability::verdict_word(level),
        "bodies": rows,
    })
}

/// Rows whose host, verdict or presence differ between two trees.
pub fn changed(before: &serde_json::Value, after: &serde_json::Value) -> Vec<String> {
    let index = |t: &serde_json::Value| -> BTreeMap<String, serde_json::Value> {
        t["bodies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|b| {
                (
                    b["address"].as_str().unwrap_or("").to_string(),
                    serde_json::json!([b["host"], b["verdict"], b["pair"]]),
                )
            })
            .collect()
    };
    let (b, a) = (index(before), index(after));
    let mut out: Vec<String> = b
        .keys()
        .chain(a.keys())
        .filter(|k| b.get(*k) != a.get(*k))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}
