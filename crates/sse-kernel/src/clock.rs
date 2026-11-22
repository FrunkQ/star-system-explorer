//! One clock for the universe (top-level design 12.2 and 17.1).
//!
//! There is one campaign now (`time.campaign_now`, see `orbits`). Every
//! system's age is its own base age plus two things:
//! - the campaign time elapsed since the clock was first set (the latest
//!   `campaign.now_s` minus the first): "make this the campaign's now" moves
//!   the clock, so every system is that much older;
//! - its own `system.age_shift_myr` (authored): "make this system this old,
//!   today" leaves the clock alone and moves this star's birth earlier (or
//!   later) instead, with a reconstructed origin (`system.origin`) so the
//!   universe corrects itself around the tweak.
//!
//! This replaces T2's per-system `system.now_year`, which is still read on
//! old saves but never written.

use crate::orbits::{CAMPAIGN_NOW, CAMPAIGN_NOW_FIELD};
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, State, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::BTreeMap;

pub const AGE_SHIFT: &str = "system.age_shift_myr";
pub const AGE_IMPOSSIBLE: &str = "system.age_impossible";
pub const ORIGIN_KIND: &str = "system.origin";
/// A GM-pinned birthplace: the kernel never reconstructs over it.
pub const ORIGIN_PINNED: &str = "origin.pinned";
pub const JULIAN_YEAR_S: f64 = sse_core::sheet::YR_JULIAN;

fn int(v: &Value) -> Option<i64> {
    match v {
        Value::Int(i) => i64::try_from(*i).ok(),
        _ => None,
    }
}

/// The first and the latest campaign now, seconds.
pub fn campaign_span_s(ch: &Chronicle) -> Option<(i64, i64)> {
    let mut it = ch
        .events()
        .iter()
        .filter(|e| e.body.kind == CAMPAIGN_NOW)
        .filter_map(|e| {
            let root = e.body.subject.first()?;
            e.body
                .payload
                .set
                .get(root)
                .and_then(|r| r.get(CAMPAIGN_NOW_FIELD))
                .and_then(int)
        });
    let first = it.next()?;
    let last = it.next_back().unwrap_or(first);
    Some((first, last))
}

/// Campaign time elapsed since the clock was first set, Myr.
pub fn elapsed_myr(ch: &Chronicle) -> f64 {
    campaign_span_s(ch).map_or(0.0, |(a, b)| (b - a) as f64 / JULIAN_YEAR_S / 1.0e6)
}

pub fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// This system's own age shift, Myr.
pub fn age_shift_myr(state: &State, system: &Address) -> f64 {
    state.get(system, AGE_SHIFT).and_then(as_f64).unwrap_or(0.0)
}

/// An authored set on one address, timeless (active from the origin, as the
/// campaign clock is), so every reading of the universe sees it.
pub fn timeless_set(
    ch: &Chronicle,
    a: &Address,
    f: BTreeMap<String, Value>,
    evidence: Evidence,
    causes: Vec<sse_core::ContentHash>,
    kind: &str,
    publisher: Publisher,
) -> Result<Event, String> {
    let mut set = BTreeMap::new();
    set.insert(a.clone(), f);
    Event::new(EventBody {
        kind: if kind.is_empty() {
            kinds::AUTHORED_SET.into()
        } else {
            kind.into()
        },
        subject: vec![a.clone()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: ch.next_stamp(),
        causes,
        payload: Payload {
            set,
            ..Default::default()
        },
        evidence,
        publisher,
    })
    .map_err(|e| format!("{e}"))
}
