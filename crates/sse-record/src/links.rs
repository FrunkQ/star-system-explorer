//! Links between places (space highways, V8; Alex 23 Nov 2026): a standing
//! connection ships travel along, such as a stargate pair, a charted trade
//! lane or a patrol line. The record keeps who made it, when it holds, its
//! ends, its name and kind, and who knows of it; the travel part (each end's
//! places, its own FTL rule, counted measures) rides as `nav`, JSON in
//! sse-nav's `Link` shape that the record stores and never reads.
//!
//! A link is one `record.link` event about its own address
//! (`u:<id>/link:<id>`), its two ends as further subjects. Nothing is edited
//! in place: a later event for the same link replaces it, and a withdrawal
//! ends it. Links live in the event's `data` only, never in the fold's facts,
//! so no physics flow can read them.

use crate::{field, kinds, Audience, RecordError, GM_WRITER};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

/// A link to write.
#[derive(Clone, PartialEq, Debug)]
pub struct LinkDraft {
    /// The universe it belongs to (`u:<id>`); the link's address is `u:<id>/link:<id>`.
    pub root: Address,
    pub id: String,
    pub a: Address,
    pub b: Address,
    pub name: String,
    /// A plain label ("gate", "trade lane", "courier route"); never a rule.
    pub kind: String,
    pub one_way: bool,
    /// The travel part, JSON in sse-nav's `Link` shape (ends' places, rule, counted).
    pub nav: Option<String>,
    pub tier: Option<String>,
    pub audience: Audience,
    pub writer: String,
    pub reason: Option<String>,
    /// The pack, import or rule it comes from, for "Why?".
    pub source: Option<String>,
    pub withdraw: bool,
}

/// A link as the record reads it back.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LinkRecord {
    pub link: Address,
    pub id: String,
    pub a: Address,
    pub b: Address,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
    #[serde(default)]
    pub one_way: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav: Option<String>,
    pub since_s: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_s: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    pub audience: Audience,
    pub writer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The event that wrote it.
    pub event: String,
}

impl LinkRecord {
    /// May this audience see it? The GM sees all; others need the audience
    /// and an untiered or `obvious` link, as holdings (SV1 refines this).
    pub fn visible_to(&self, audience: &str) -> bool {
        audience == GM_WRITER
            || (self.audience.includes(audience)
                && self.tier.as_deref().is_none_or(|t| t == "obvious"))
    }
}

fn text(v: &str) -> Value {
    Value::Text(v.into())
}

/// Write a link (or withdraw one): one `record.link` event. Returns the link's address.
pub fn set_link(
    ch: &mut Chronicle,
    d: &LinkDraft,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<(Address, ContentHash), RecordError> {
    let bad = |w: String| RecordError::BadValue {
        kind: "link".into(),
        why: w,
    };
    if d.root.segments().len() != 1 || d.root.last().kind() != "u" {
        return Err(bad(format!(
            "{}: a link belongs to a universe (u:<id>)",
            d.root
        )));
    }
    let link = d
        .root
        .child_key("link", &d.id)
        .map_err(|e| bad(format!("{:?}: {e}", d.id)))?;
    if d.a == d.b {
        return Err(bad("a link joins two different places".into()));
    }
    if !d.root.contains(&d.a) || !d.root.contains(&d.b) {
        return Err(bad(format!("both ends must lie in {}", d.root)));
    }
    if d.writer.is_empty() {
        return Err(bad("no writer".into()));
    }
    if let Some(n) = &d.nav {
        serde_json::from_str::<serde_json::Value>(n)
            .map_err(|e| bad(format!("nav is not JSON: {e}")))?;
    }
    let audience = d.audience.normal().map_err(bad)?;
    let mut m = BTreeMap::new();
    m.insert("id".to_string(), text(&d.id));
    m.insert("a".into(), text(&d.a.to_string()));
    m.insert("b".into(), text(&d.b.to_string()));
    if !d.name.is_empty() {
        m.insert("name".into(), text(&d.name));
    }
    if !d.kind.is_empty() {
        m.insert("kind".into(), text(&d.kind));
    }
    if d.one_way {
        m.insert("one_way".into(), Value::Bool(true));
    }
    if let Some(n) = &d.nav {
        m.insert("nav".into(), text(n));
    }
    if let Some(t) = &d.tier {
        m.insert("tier".into(), text(t));
    }
    m.insert("audience".into(), audience.to_value());
    m.insert("writer".into(), text(&d.writer));
    if let Some(r) = &d.reason {
        m.insert("reason".into(), text(r));
    }
    if let Some(s) = &d.source {
        m.insert("source".into(), text(s));
    }
    if d.withdraw {
        m.insert("withdraw".into(), Value::Bool(true));
    }
    let e = Event::new(EventBody {
        kind: kinds::LINK.into(),
        subject: vec![link.clone(), d.a.clone(), d.b.clone()],
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
    Ok((link, ch.append(e)?))
}

/// Read one `record.link` event: the link, and whether it withdraws.
fn link_of(e: &Event) -> Option<(LinkRecord, bool)> {
    if e.body.kind != kinds::LINK {
        return None;
    }
    let Some(Value::Map(m)) = &e.body.payload.data else {
        return None;
    };
    let l = LinkRecord {
        link: e.body.subject.first()?.clone(),
        id: field(m, "id")?.to_string(),
        a: field(m, "a")?.parse().ok()?,
        b: field(m, "b")?.parse().ok()?,
        name: field(m, "name").unwrap_or("").to_string(),
        kind: field(m, "kind").unwrap_or("").to_string(),
        one_way: matches!(m.get("one_way"), Some(Value::Bool(true))),
        nav: field(m, "nav").map(str::to_string),
        since_s: e.body.occurred_at.s,
        until_s: None,
        tier: field(m, "tier").map(str::to_string),
        audience: m
            .get("audience")
            .and_then(Audience::from_value)
            .unwrap_or_else(Audience::gm),
        writer: field(m, "writer").unwrap_or("").to_string(),
        reason: field(m, "reason").map(str::to_string),
        source: field(m, "source").map(str::to_string),
        event: e.id.hex(),
    };
    Some((l, matches!(m.get("withdraw"), Some(Value::Bool(true)))))
}

/// Every link begun by `at`, in fold order, with `until_s` on those a later
/// event (also by `at`) replaced or withdrew.
pub fn history<'a>(
    events: impl IntoIterator<Item = &'a Event>,
    at: UniverseTime,
) -> Vec<LinkRecord> {
    let mut sel: Vec<&Event> = events
        .into_iter()
        .filter(|e| e.body.kind == kinds::LINK && e.body.activated_at <= at)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut out: Vec<LinkRecord> = vec![];
    let mut live: BTreeMap<Address, usize> = BTreeMap::new();
    for e in sel {
        let Some((l, withdraw)) = link_of(e) else {
            continue;
        };
        if let Some(i) = live.remove(&l.link) {
            out[i].until_s = Some(l.since_s);
        }
        if !withdraw {
            live.insert(l.link.clone(), out.len());
            out.push(l);
        }
    }
    out
}

/// The links in force at `at` that `audience` may see.
pub fn links_at<'a>(
    events: impl IntoIterator<Item = &'a Event>,
    at: UniverseTime,
    audience: &str,
) -> Vec<LinkRecord> {
    history(events, at)
        .into_iter()
        .filter(|l| l.until_s.is_none() && l.visible_to(audience))
        .collect()
}
