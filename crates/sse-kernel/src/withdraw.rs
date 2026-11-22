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

/// Whether an address or any address above it is withdrawn in these facts.
pub fn is_withdrawn(facts: &BTreeMap<Address, BTreeMap<String, Value>>, a: &Address) -> bool {
    let mut at = Some(a.clone());
    while let Some(x) = at {
        if matches!(
            facts.get(&x).and_then(|f| f.get(WITHDRAWN)),
            Some(Value::Bool(true))
        ) {
            return true;
        }
        at = x.parent();
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
