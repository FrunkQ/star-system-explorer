//! Paste support for the Build desk (top-level design 8.2): a batch of
//! authored edits as one event, and withdrawing a pasted body so Undo works
//! without deleting history.
//!
//! A withdrawn address carries the authored fact `withdrawn: true` from the
//! time of the withdrawal. It and everything under it drop out of `scene`,
//! `resolved_system` and `systems` from then on. `restore` writes
//! `withdrawn: false`, so Redo brings it back with every fact it had. The
//! history keeps both events.

use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::BTreeMap;

pub const WITHDRAWN: &str = "withdrawn";

/// Whether an address, any address above it, or anything it orbits (its
/// `orbit.host` chain: a V3 moon sits beside its planet, not under it) is
/// withdrawn in these facts. A ring of hosts stops after 64 steps.
pub fn is_withdrawn(facts: &BTreeMap<Address, BTreeMap<String, Value>>, a: &Address) -> bool {
    let mut todo = vec![(a.clone(), 0u32)];
    while let Some((x, depth)) = todo.pop() {
        let row = facts.get(&x);
        if matches!(row.and_then(|f| f.get(WITHDRAWN)), Some(Value::Bool(true))) {
            return true;
        }
        if depth >= 64 {
            continue;
        }
        if let Some(p) = x.parent() {
            todo.push((p, depth + 1));
        }
        if let Some(Value::Text(h)) = row.and_then(|f| f.get("orbit.host")) {
            if let Ok(h) = h.parse::<Address>() {
                if h != x {
                    todo.push((h, depth + 1));
                }
            }
        }
    }
    false
}

/// Drop every withdrawn subject (and those under one), and the bookkeeping
/// field from the rest.
pub fn drop_withdrawn(facts: &mut BTreeMap<Address, BTreeMap<String, Value>>) {
    let gone: Vec<Address> = facts
        .keys()
        .filter(|a| is_withdrawn(facts, a))
        .cloned()
        .collect();
    for a in gone {
        facts.remove(&a);
    }
    for f in facts.values_mut() {
        f.remove(WITHDRAWN);
    }
    facts.retain(|_, f| !f.is_empty());
}
