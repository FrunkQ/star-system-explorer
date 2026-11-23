//! A civilisation's technology as dated record events (track FT, future
//! tech design 7, FT2): its level on the setting's ladder, and exceptions,
//! advances gained early ("Develop it") or never taken up. Set by hand or by
//! a pack as plain data; nothing grows on its own (growth is 4.1).
//!
//! One `record.tech` event on an actor (or the universe root, for the
//! setting) sets any of: a level, advances gained, advances dropped. A child
//! inherits from its parents and the setting, the nearest saying wins.

use crate::{field, kinds, RecordError};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

/// A change to a civilisation's technology.
#[derive(Clone, PartialEq, Debug)]
pub struct TechDraft {
    /// An actor, or the universe root for the setting.
    pub holder: Address,
    pub level: Option<i64>,
    pub gain: Vec<String>,
    pub lose: Vec<String>,
    pub writer: String,
    pub reason: Option<String>,
}

fn text(v: &str) -> Value {
    Value::Text(v.into())
}

fn list(v: &[String]) -> Value {
    Value::Array(v.iter().map(|s| text(s)).collect())
}

/// Write one `record.tech` event.
pub fn set_tech(
    ch: &mut Chronicle,
    d: &TechDraft,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, RecordError> {
    let bad = |w: &str| RecordError::BadValue {
        kind: "tech".into(),
        why: w.into(),
    };
    if d.level.is_none() && d.gain.is_empty() && d.lose.is_empty() {
        return Err(bad("set a level, or gain or lose an advance"));
    }
    if d.level.is_some_and(|l| l < 0) {
        return Err(bad("a level is 0 or more"));
    }
    if d.gain.iter().any(|g| d.lose.contains(g)) {
        return Err(bad("an advance cannot be gained and lost at once"));
    }
    if d.writer.is_empty() {
        return Err(bad("no writer"));
    }
    let mut m = BTreeMap::new();
    if let Some(l) = d.level {
        m.insert("level".to_string(), Value::Int(l as i128));
    }
    if !d.gain.is_empty() {
        m.insert("gain".into(), list(&d.gain));
    }
    if !d.lose.is_empty() {
        m.insert("lose".into(), list(&d.lose));
    }
    m.insert("writer".into(), text(&d.writer));
    if let Some(r) = &d.reason {
        m.insert("reason".into(), text(r));
    }
    let e = Event::new(EventBody {
        kind: kinds::TECH.into(),
        subject: vec![d.holder.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: ch.next_stamp(),
        causes: vec![],
        payload: Payload {
            data: Some(Value::Map(m)),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    Ok(ch.append(e)?)
}

/// Where a part of a civilisation's technology came from.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Said {
    /// The holder that said it (the civilisation itself, a parent, or the
    /// setting at the universe root).
    pub by: Address,
    pub since_s: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A civilisation's technology at a date: the level it works at, and its
/// exceptions, each with who said it and when.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct TechOf {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<(i64, Said)>,
    /// Advances held above the level.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gained: BTreeMap<String, Said>,
    /// Advances at or below the level never taken up.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lost: BTreeMap<String, Said>,
}

fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| match x {
                Value::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

/// The technology of `holder` at `at`: its own events, then its parents',
/// then the setting's (the universe root); the nearest that says a thing
/// wins, and later events on one holder replace earlier ones.
pub fn tech_of<'a>(
    events: impl IntoIterator<Item = &'a Event>,
    holder: &Address,
    at: UniverseTime,
) -> TechOf {
    let mut sel: Vec<&Event> = events
        .into_iter()
        .filter(|e| e.body.kind == kinds::TECH && e.body.activated_at <= at)
        .filter(|e| e.body.subject.first().is_some_and(|s| s.contains(holder)))
        .collect();
    sel.sort_by_key(|e| e.order_key());
    // Each holder's own state, folded in time order.
    let mut own: BTreeMap<Address, TechOf> = BTreeMap::new();
    for e in sel {
        let Some(Value::Map(m)) = &e.body.payload.data else {
            continue;
        };
        let by = e.body.subject[0].clone();
        let said = Said {
            by: by.clone(),
            since_s: e.body.occurred_at.s,
            reason: field(m, "reason").map(str::to_string),
        };
        let t = own.entry(by).or_default();
        if let Some(Value::Int(l)) = m.get("level") {
            t.level = Some((*l as i64, said.clone()));
        }
        for g in strings(m.get("gain")) {
            t.lost.remove(&g);
            t.gained.insert(g, said.clone());
        }
        for l in strings(m.get("lose")) {
            t.gained.remove(&l);
            t.lost.insert(l, said.clone());
        }
    }
    // Nearest first: the holder, then each parent up to the root.
    let mut out = TechOf::default();
    let mut chain = vec![];
    let mut a = Some(holder.clone());
    while let Some(x) = a {
        a = x.parent();
        chain.push(x);
    }
    for h in &chain {
        let Some(t) = own.get(h) else { continue };
        if out.level.is_none() {
            out.level = t.level.clone();
        }
        for (k, s) in &t.gained {
            if !out.lost.contains_key(k) {
                out.gained.entry(k.clone()).or_insert_with(|| s.clone());
            }
        }
        for (k, s) in &t.lost {
            if !out.gained.contains_key(k) {
                out.lost.entry(k.clone()).or_insert_with(|| s.clone());
            }
        }
    }
    out
}
