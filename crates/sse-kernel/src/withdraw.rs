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

/// Whether an address, or what it belongs to, is withdrawn in these facts.
/// A body that orbits something belongs to its `orbit.host` (a V3 moon sits
/// beside its planet, and a moon handed to another body keeps its address);
/// anything else belongs to the address above it. A ring stops after 64 steps.
pub fn is_withdrawn(facts: &BTreeMap<Address, BTreeMap<String, Value>>, a: &Address) -> bool {
    let mut at = Some(a.clone());
    let mut steps = 0;
    while let Some(x) = at {
        let row = facts.get(&x);
        if matches!(row.and_then(|f| f.get(WITHDRAWN)), Some(Value::Bool(true))) {
            return true;
        }
        steps += 1;
        if steps > 64 {
            return false;
        }
        at = match row.and_then(|f| f.get("orbit.host")) {
            Some(Value::Text(h)) => h.parse::<Address>().ok().filter(|h| *h != x),
            _ => None,
        }
        .or_else(|| x.parent());
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
