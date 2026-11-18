//! Label changes as chronicle events (classification design section 5):
//! "Tarn stopped being a garden world 1.2 billion years from now".
//!
//! A `classify.relabel` event carries the scheme, the old and new base
//! labels, the fit and the hash of the atlas that decided it, in
//! `payload.data`. It sets no facts (`payload.set` is empty), so a label
//! is never the only record of anything (H7): drop every atlas and every
//! relabel event and the facts are exactly the same. Its cause is the
//! event that last set a fact on the body, so the story sifter can follow
//! why. Given the same authored chronicle and the same packs (the run
//! manifest), the same relabel events come out with the same ids.

use crate::classify::Atlas;
use crate::predicate::Subject;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

pub const RELABEL: &str = "classify.relabel";

/// One recorded change of base label.
#[derive(Clone, PartialEq, Debug)]
pub struct Relabel {
    pub event: ContentHash,
    pub at: UniverseTime,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn text(o: &Option<String>) -> Value {
    o.as_ref().map_or(Value::Null, |s| Value::Text(s.clone()))
}

/// Label `address` in `scheme` at each of `times` (ascending) from the
/// chronicle's own state, and append a relabel event wherever the base
/// label changes. The first reading sets the held label without an event.
pub fn record(
    ch: &mut Chronicle,
    atlas: &Atlas,
    scheme: &str,
    address: &Address,
    times: &[UniverseTime],
) -> Result<Vec<Relabel>, String> {
    let atlas_hash = atlas.content_hash();
    let mut held: Option<String> = None;
    let mut first = true;
    let mut out = Vec::new();
    for &t in times {
        let state = ch.state_at(t);
        let facts = state.facts.get(address).cloned().unwrap_or_default();
        let mut s = Subject::new(facts);
        s.address = Some(address.clone());
        s.now_s = t.s;
        let l = atlas.label(scheme, &s, held.as_deref());
        if first {
            first = false;
            held = l.base;
            continue;
        }
        if l.base == held {
            continue;
        }
        let cause = ch
            .events()
            .iter()
            .filter(|e| e.body.kind != RELABEL && e.body.occurred_at <= t)
            .filter(|e| e.body.payload.set.contains_key(address))
            .max_by(|a, b| a.order_key().cmp(&b.order_key()))
            .map(|e| e.id);
        let mut data = BTreeMap::new();
        data.insert("scheme".to_string(), Value::Text(scheme.into()));
        data.insert("from".to_string(), text(&held));
        data.insert("to".to_string(), text(&l.base));
        data.insert("fit".to_string(), Value::Float(l.fit));
        data.insert("atlas".to_string(), Value::Text(atlas_hash.hex()));
        let e = Event::new(EventBody {
            kind: RELABEL.into(),
            subject: vec![address.clone()],
            occurred_at: t,
            activated_at: t,
            recognised_at: ch.next_stamp(),
            causes: cause.into_iter().collect(),
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Derived,
            publisher: Publisher::new("sse.sift", &format!("classify.{scheme}"), "1"),
        })
        .map_err(|e| e.to_string())?;
        let id = ch.append(e).map_err(|e| e.to_string())?;
        out.push(Relabel {
            event: id,
            at: t,
            from: held.clone(),
            to: l.base.clone(),
        });
        held = l.base;
    }
    Ok(out)
}
