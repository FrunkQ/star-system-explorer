//! The campaign record (track J1, record design section 2): what a campaign
//! remembers that physics does not care about.
//!
//! - **Notes**: authored text at addresses and a universe time, with tags,
//!   links, an author seat and an audience. Never edited: a correction
//!   supersedes, a withdrawal supersedes with empty text.
//! - **Kept values**: stored, not derived, quantities a pack declares
//!   (`provides.other.record.kept`), changed only by `record.set` events from
//!   a listed writer. `misc.<anything>` is the schema-free corner (GM only).
//! - **Actors**: subjects that are not places (`u:<id>/actor:<id>`), with
//!   relations as kept values of shape `ref`.
//!
//! Nothing here is a second path: every change is a chronicle event, notes
//! ride in `payload.data` and kept values in `payload.set`, so the fold, the
//! state hash and branches work unchanged. Nothing here feeds physics or
//! RESOLVE (rules 2.2.1 and 2.2.2): no flow reads these fields.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sse_chronicle::{
    Chronicle, ChronicleError, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::{BTreeMap, BTreeSet};

/// Event kinds of the record.
pub mod kinds {
    pub const NOTE: &str = "record.note";
    pub const SET: &str = "record.set";
    pub const ACTOR: &str = "record.actor";
    /// A holding on a map layer (track OV): see [`crate::holdings`].
    pub const HOLD: &str = "record.hold";
    /// A civilisation's technology (track FT): see [`crate::tech`].
    pub const TECH: &str = "record.tech";
}

/// Fields an actor carries from `add_actor`.
pub const ACTOR_NAME: &str = "name";
pub const ACTOR_KIND: &str = "actor.kind";
/// The schema-free corner: any address may hold `misc.<anything>`.
pub const MISC_PREFIX: &str = "misc.";
/// The writer that is always allowed for `misc.*` and listed by default.
pub const GM_WRITER: &str = "gm";
/// Longest note text, bytes.
pub const MAX_NOTE_BYTES: usize = 16_384;

#[derive(Clone, Debug, PartialEq)]
pub enum RecordError {
    BadNote(String),
    UnknownNote(String),
    UnknownKind(String),
    WriterRefused { kind: String, writer: String },
    BadValue { kind: String, why: String },
    BadActor(String),
    Registry(Vec<String>),
    Chronicle(ChronicleError),
}
impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::BadNote(s) => write!(f, "bad note: {s}"),
            RecordError::UnknownNote(s) => write!(f, "no note {s} to supersede"),
            RecordError::UnknownKind(k) => write!(f, "no kept kind {k:?} in the pack stack"),
            RecordError::WriterRefused { kind, writer } => {
                write!(f, "{writer:?} may not change {kind:?} (not in its writers)")
            }
            RecordError::BadValue { kind, why } => write!(f, "{kind}: {why}"),
            RecordError::BadActor(s) => write!(f, "bad actor: {s}"),
            RecordError::Registry(v) => write!(f, "{}", v.join("; ")),
            RecordError::Chronicle(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for RecordError {}
impl From<ChronicleError> for RecordError {
    fn from(e: ChronicleError) -> Self {
        RecordError::Chronicle(e)
    }
}
impl From<sse_chronicle::EventError> for RecordError {
    fn from(e: sse_chronicle::EventError) -> Self {
        RecordError::Chronicle(e.into())
    }
}

// ---------------------------------------------------------------- notes

/// Who may see a note: everyone, the GM only, or listed audiences (the GM
/// always sees every note). JSON: `"everyone"`, `"gm"` or `["crew", ...]`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Audience {
    Named(String),
    Only(Vec<String>),
}

impl Default for Audience {
    fn default() -> Self {
        Audience::gm()
    }
}

impl Audience {
    pub fn everyone() -> Self {
        Audience::Named("everyone".into())
    }
    pub fn gm() -> Self {
        Audience::Named(GM_WRITER.into())
    }
    fn normal(&self) -> Result<Audience, String> {
        Ok(match self {
            Audience::Named(n) if n == "everyone" || n == GM_WRITER => self.clone(),
            Audience::Named(n) => {
                return Err(format!(
                    "audience {n:?}: use \"everyone\", \"gm\" or a list"
                ))
            }
            Audience::Only(v) if v.is_empty() => {
                return Err("an empty audience list; use \"gm\"".into())
            }
            Audience::Only(v) => {
                let s: BTreeSet<&String> = v.iter().collect();
                Audience::Only(s.into_iter().cloned().collect())
            }
        })
    }
    /// Does `audience` (a seat's audience id) get to see it? The GM always does.
    pub fn includes(&self, audience: &str) -> bool {
        if audience == GM_WRITER {
            return true;
        }
        match self {
            Audience::Named(n) => n == "everyone",
            Audience::Only(v) => v.iter().any(|a| a == audience),
        }
    }
    fn to_value(&self) -> Value {
        match self {
            Audience::Named(n) => Value::Text(n.clone()),
            Audience::Only(v) => Value::Array(v.iter().map(|a| Value::Text(a.clone())).collect()),
        }
    }
    fn from_value(v: &Value) -> Option<Audience> {
        match v {
            Value::Text(t) => Some(Audience::Named(t.clone())),
            Value::Array(a) => a
                .iter()
                .map(|x| match x {
                    Value::Text(t) => Some(t.clone()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
                .map(Audience::Only),
            _ => None,
        }
    }
}

/// A note to write.
#[derive(Clone, PartialEq, Debug)]
pub struct NoteDraft {
    /// The note's address first, then linked addresses.
    pub subjects: Vec<Address>,
    pub text: String,
    pub tags: Vec<String>,
    /// The seat that wrote it (`gm`, a seat id, `minerva`).
    pub by: String,
    pub audience: Audience,
    /// Shown only where the audience's data view shows this tier.
    pub tier: Option<String>,
    pub supersedes: Option<ContentHash>,
    /// Written by an agent (Minerva): evidence `narrative`, not `authored`.
    pub narrative: bool,
    /// When the news reaches its audience (the event's `activated_at`;
    /// default: when it happened). Not shown, and not sent, before then.
    pub arrives: Option<UniverseTime>,
}

/// A note as the record reads it back.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub subjects: Vec<Address>,
    pub at_s: i64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub by: String,
    pub audience: Audience,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub narrative: bool,
}

impl Note {
    /// Is this note at or linked to `a`?
    pub fn about(&self, a: &Address) -> bool {
        self.subjects.iter().any(|s| s == a)
    }
    /// May `audience` see it, given the tiers its data view shows at the
    /// note's address (`None`: no view applies, so only untiered notes)?
    pub fn visible_to(&self, audience: &str, shown_tiers: Option<&[String]>) -> bool {
        if audience == GM_WRITER {
            return true;
        }
        self.audience.includes(audience)
            && match &self.tier {
                None => true,
                Some(t) => shown_tiers.is_some_and(|s| s.iter().any(|x| x == t)),
            }
    }
}

fn text(v: &str) -> Value {
    Value::Text(v.into())
}

fn ident_ok(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"._-:@{}".contains(&c))
}

/// Write a note. The first subject is where it lives; the rest are links.
pub fn add_note(
    ch: &mut Chronicle,
    d: &NoteDraft,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, RecordError> {
    if d.subjects.is_empty() {
        return Err(RecordError::BadNote("a note needs an address".into()));
    }
    if d.text.len() > MAX_NOTE_BYTES {
        return Err(RecordError::BadNote(format!(
            "text over {MAX_NOTE_BYTES} bytes"
        )));
    }
    if d.text.trim().is_empty() && d.supersedes.is_none() {
        return Err(RecordError::BadNote(
            "empty text (an empty note only withdraws another)".into(),
        ));
    }
    if let Some(t) = d.tags.iter().find(|t| !ident_ok(t)) {
        return Err(RecordError::BadNote(format!(
            "tag {t:?}: lowercase letters, digits and . _ - :"
        )));
    }
    if d.by.is_empty() {
        return Err(RecordError::BadNote("no author seat".into()));
    }
    let audience = d.audience.normal().map_err(RecordError::BadNote)?;
    let arrives = d.arrives.unwrap_or(at);
    if arrives < at {
        return Err(RecordError::BadNote(
            "news cannot arrive before it happens".into(),
        ));
    }
    let mut causes = vec![];
    if let Some(s) = &d.supersedes {
        let found = ch
            .events()
            .iter()
            .any(|e| e.id == *s && e.body.kind == kinds::NOTE);
        if !found {
            return Err(RecordError::UnknownNote(s.hex()));
        }
        causes.push(*s);
    }
    let mut m = BTreeMap::new();
    m.insert("text".to_string(), text(&d.text));
    let tags: BTreeSet<&String> = d.tags.iter().collect();
    if !tags.is_empty() {
        m.insert(
            "tags".into(),
            Value::Array(tags.into_iter().map(|t| text(t)).collect()),
        );
    }
    m.insert("by".into(), text(&d.by));
    m.insert("audience".into(), audience.to_value());
    if let Some(t) = &d.tier {
        m.insert("tier".into(), text(t));
    }
    if let Some(s) = &d.supersedes {
        m.insert("supersedes".into(), text(&s.hex()));
    }
    let e = Event::new(EventBody {
        kind: kinds::NOTE.into(),
        subject: d.subjects.clone(),
        occurred_at: at,
        activated_at: arrives,
        recognised_at: ch.next_stamp(),
        causes,
        payload: Payload {
            data: Some(Value::Map(m)),
            ..Default::default()
        },
        evidence: if d.narrative {
            Evidence::Narrative
        } else {
            Evidence::Authored
        },
        publisher: publisher.clone(),
    })?;
    Ok(ch.append(e)?)
}

fn field<'a>(m: &'a BTreeMap<String, Value>, k: &str) -> Option<&'a str> {
    match m.get(k) {
        Some(Value::Text(t)) => Some(t),
        _ => None,
    }
}

/// Read one `record.note` event (None for any other event).
pub fn note_of(e: &Event) -> Option<Note> {
    if e.body.kind != kinds::NOTE {
        return None;
    }
    let Some(Value::Map(m)) = &e.body.payload.data else {
        return None;
    };
    Some(Note {
        id: e.id.hex(),
        subjects: e.body.subject.clone(),
        at_s: e.body.occurred_at.s,
        text: field(m, "text").unwrap_or("").to_string(),
        tags: match m.get("tags") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|x| match x {
                    Value::Text(t) => Some(t.clone()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        },
        by: field(m, "by").unwrap_or("").to_string(),
        audience: m
            .get("audience")
            .and_then(Audience::from_value)
            .unwrap_or_else(Audience::gm),
        tier: field(m, "tier").map(str::to_string),
        supersedes: field(m, "supersedes").map(str::to_string),
        narrative: e.body.evidence == Evidence::Narrative,
    })
}

/// The live notes at `at`: written (activated) by then and not superseded by
/// a note also active by then; withdrawals (empty text) are dropped. In fold
/// order.
pub fn notes_at<'a>(events: impl IntoIterator<Item = &'a Event>, at: UniverseTime) -> Vec<Note> {
    let mut sel: Vec<&Event> = events
        .into_iter()
        .filter(|e| e.body.kind == kinds::NOTE && e.body.activated_at <= at)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let all: Vec<Note> = sel.into_iter().filter_map(note_of).collect();
    let gone: BTreeSet<&str> = all.iter().filter_map(|n| n.supersedes.as_deref()).collect();
    all.iter()
        .filter(|n| !gone.contains(n.id.as_str()) && !n.text.trim().is_empty())
        .cloned()
        .collect()
}

/// The record is never sent to seats as raw events (record design 8A):
/// seats get per-audience disclosures built on the GM machine (the kernel's
/// `disclosures` op), which carry no event ids and no causes.

// ---------------------------------------------------------------- kept values

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    Number,
    Flag,
    Choice,
    Ref,
    Text,
    List,
}

/// A pack's declaration of a stored quantity (record design 2.1).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KeptKind {
    pub id: String,
    pub shape: Shape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<(f64, f64)>,
    /// Members of a `choice`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Disclosure tier, as for any quantity kind.
    pub tier: String,
    /// Who may change it: `gm`, intent kinds, effect ids.
    pub writers: Vec<String>,
    pub describes: String,
}

/// The `provides.other.record` block of a pack.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct RecordPack {
    #[serde(default)]
    pub kept: Vec<KeptKind>,
}

/// Every kept kind of the active stack, with the pack that keeps it.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Registry {
    pub kinds: BTreeMap<String, (String, KeptKind)>,
}

fn check_shape(k: &KeptKind, v: &Value) -> Result<(), String> {
    match (k.shape, v) {
        (Shape::Number, Value::Float(_) | Value::Int(_)) => {
            let x = match v {
                Value::Float(f) => *f,
                Value::Int(i) => *i as f64,
                _ => unreachable!(),
            };
            if !x.is_finite() {
                return Err("not a finite number".into());
            }
            if let Some((lo, hi)) = k.range {
                if x < lo || x > hi {
                    return Err(format!("{x} is outside {lo} to {hi}"));
                }
            }
            Ok(())
        }
        (Shape::Flag, Value::Bool(_)) => Ok(()),
        (Shape::Choice, Value::Text(t)) => {
            if k.choices.iter().any(|c| c == t) {
                Ok(())
            } else {
                Err(format!("{t:?} is not one of {:?}", k.choices))
            }
        }
        (Shape::Ref, Value::Text(t)) => t
            .parse::<Address>()
            .map(|_| ())
            .map_err(|e| format!("{t:?} is not an address: {e}")),
        (Shape::Text, Value::Text(_)) => Ok(()),
        (Shape::List, Value::Array(_)) => Ok(()),
        (s, _) => Err(format!("expected a {s:?} value").to_lowercase()),
    }
}

impl Registry {
    /// From each pack's `provides.other.record`, bottom of the stack first.
    /// Two packs declaring the same kind is refused: one keeper per kind.
    pub fn from_packs(packs: &[(String, serde_json::Value)]) -> Result<Registry, Vec<String>> {
        let mut r = Registry::default();
        let mut errs = vec![];
        for (pack, v) in packs {
            let rp: RecordPack = match serde_json::from_value(v.clone()) {
                Ok(x) => x,
                Err(e) => {
                    errs.push(format!("{pack}: record: {e}"));
                    continue;
                }
            };
            for k in rp.kept {
                let bad = |why: &str| format!("{pack}: kept kind {:?}: {why}", k.id);
                if !ident_ok(&k.id) || !k.id.contains('.') {
                    errs.push(bad("ids are dotted lowercase names (trade.credits)"));
                    continue;
                }
                if k.id.starts_with(MISC_PREFIX) {
                    errs.push(bad("misc.* is the GM's schema-free corner, not a kind"));
                    continue;
                }
                if k.writers.is_empty() {
                    errs.push(bad("no writers"));
                }
                if k.shape == Shape::Choice && k.choices.is_empty() {
                    errs.push(bad("a choice needs choices"));
                }
                if let Some((lo, hi)) = k.range {
                    if lo > hi || k.shape != Shape::Number {
                        errs.push(bad("a range is low to high, on numbers only"));
                    }
                }
                if let Some(d) = &k.default {
                    match serde_json::from_value::<Value>(d.clone()) {
                        Ok(v) => {
                            if let Err(e) = check_shape(&k, &v) {
                                errs.push(bad(&format!("default: {e}")));
                            }
                        }
                        Err(e) => errs.push(bad(&format!("default: {e}"))),
                    }
                }
                if let Some((other, _)) = r.kinds.get(&k.id) {
                    errs.push(bad(&format!(
                        "already kept by {other}; one keeper per kind"
                    )));
                    continue;
                }
                r.kinds.insert(k.id.clone(), (pack.clone(), k));
            }
        }
        if errs.is_empty() {
            Ok(r)
        } else {
            Err(errs)
        }
    }

    pub fn get(&self, kind: &str) -> Option<&KeptKind> {
        self.kinds.get(kind).map(|(_, k)| k)
    }

    /// May `writer` set `kind` to `value` (None: clear it)?
    pub fn check(
        &self,
        kind: &str,
        value: Option<&Value>,
        writer: &str,
    ) -> Result<(), RecordError> {
        if let Some(rest) = kind.strip_prefix(MISC_PREFIX) {
            if rest.is_empty() || !ident_ok(rest) {
                return Err(RecordError::BadValue {
                    kind: kind.into(),
                    why: "misc fields are misc.<lowercase name>".into(),
                });
            }
            if writer != GM_WRITER {
                return Err(RecordError::WriterRefused {
                    kind: kind.into(),
                    writer: writer.into(),
                });
            }
            return Ok(());
        }
        let k = self
            .get(kind)
            .ok_or_else(|| RecordError::UnknownKind(kind.into()))?;
        if !k.writers.iter().any(|w| w == writer) {
            return Err(RecordError::WriterRefused {
                kind: kind.into(),
                writer: writer.into(),
            });
        }
        if let Some(v) = value {
            check_shape(k, v).map_err(|why| RecordError::BadValue {
                kind: kind.into(),
                why,
            })?;
        }
        Ok(())
    }

    /// Disclosure entries for every kept kind (field id to tier), plus
    /// `misc.*` at `secret` (pass the ladder's GM-only rung).
    pub fn disclosure_fields(&self, secret: &str) -> BTreeMap<String, String> {
        let mut m: BTreeMap<String, String> = self
            .kinds
            .values()
            .map(|(_, k)| (k.id.clone(), k.tier.clone()))
            .collect();
        m.insert(format!("{MISC_PREFIX}*"), secret.into());
        m
    }
}

fn as_number(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// One change to a kept value.
#[derive(Clone, PartialEq, Debug)]
pub struct KeptSet {
    pub address: Address,
    pub kind: String,
    /// None clears the value.
    pub value: Option<Value>,
    pub writer: String,
    /// Why (free text): "fined at Tarn customs".
    pub reason: Option<String>,
    /// The events that caused it (an effect's trigger), if any.
    pub causes: Vec<ContentHash>,
}

/// Change a kept value: one `record.set` event, refused unless the writer
/// is listed and the value fits the kind. The event's data carries the
/// writer, reason, previous value and, for numbers, the delta.
pub fn set_kept(
    ch: &mut Chronicle,
    reg: &Registry,
    s: &KeptSet,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, RecordError> {
    reg.check(&s.kind, s.value.as_ref(), &s.writer)?;
    let before = ch.state_at(at);
    let prev = before.get(&s.address, &s.kind).cloned();
    let mut data = BTreeMap::new();
    data.insert("kind".to_string(), text(&s.kind));
    data.insert("writer".into(), text(&s.writer));
    if let Some(r) = &s.reason {
        data.insert("reason".into(), text(r));
    }
    data.insert("previous".into(), prev.clone().unwrap_or(Value::Null));
    if let (Some(a), Some(b)) = (
        prev.as_ref().and_then(as_number),
        s.value.as_ref().and_then(as_number),
    ) {
        data.insert("delta".into(), Value::Float(b - a));
    }
    let mut payload = Payload {
        data: Some(Value::Map(data)),
        ..Default::default()
    };
    match &s.value {
        Some(v) => {
            payload
                .set
                .insert(s.address.clone(), [(s.kind.clone(), v.clone())].into());
        }
        None => {
            payload
                .unset
                .insert(s.address.clone(), vec![s.kind.clone()]);
        }
    }
    let e = Event::new(EventBody {
        kind: kinds::SET.into(),
        subject: vec![s.address.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: ch.next_stamp(),
        causes: s.causes.clone(),
        payload,
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    Ok(ch.append(e)?)
}

/// One step in a kept value's history ("why is this 14?").
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KeptChange {
    pub event: String,
    pub at_s: i64,
    pub value: Option<Value>,
    pub writer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
}

/// A kept value as the record reads it back.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KeptValue {
    pub kind: String,
    pub value: Value,
    /// The pack that keeps it; none for `misc.*` and for kinds whose pack
    /// is not loaded (the value stays readable and exportable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kept_by: Option<String>,
    /// "kept by a pack that is not loaded", when so.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orphan: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub describes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub history: Vec<KeptChange>,
}

/// Every kept value at `a` as of `at`, with its history, in kind order.
pub fn kept_at(ch: &Chronicle, reg: &Registry, a: &Address, at: UniverseTime) -> Vec<KeptValue> {
    let mut sel: Vec<&Event> = ch
        .events()
        .iter()
        .filter(|e| e.body.kind == kinds::SET && e.body.activated_at <= at)
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut hist: BTreeMap<String, Vec<KeptChange>> = BTreeMap::new();
    for e in sel {
        let Some(Value::Map(d)) = &e.body.payload.data else {
            continue;
        };
        let Some(kind) = field(d, "kind") else {
            continue;
        };
        let value = e.body.payload.set.get(a).and_then(|m| m.get(kind)).cloned();
        let cleared = e
            .body
            .payload
            .unset
            .get(a)
            .is_some_and(|v| v.iter().any(|f| f == kind));
        if value.is_none() && !cleared {
            continue;
        }
        hist.entry(kind.to_string()).or_default().push(KeptChange {
            event: e.id.hex(),
            at_s: e.body.occurred_at.s,
            value,
            writer: field(d, "writer").unwrap_or("").to_string(),
            reason: field(d, "reason").map(str::to_string),
            delta: d.get("delta").and_then(as_number),
        });
    }
    hist.into_iter()
        .filter_map(|(kind, history)| {
            let value = history.last()?.value.clone()?;
            let k = reg.kinds.get(&kind);
            let misc = kind.starts_with(MISC_PREFIX);
            Some(KeptValue {
                kept_by: k.map(|(p, _)| p.clone()),
                orphan: (k.is_none() && !misc)
                    .then(|| "kept by a pack that is not loaded".to_string()),
                describes: k.map(|(_, x)| x.describes.clone()),
                unit: k.and_then(|(_, x)| x.unit.clone()),
                kind,
                value,
                history,
            })
        })
        .collect()
}

// ---------------------------------------------------------------- actors

/// Make an actor: an empire, a guild, a noble, the party, a crew. Its
/// address is `<root>/actor:<id>`, or under a parent actor. One
/// `record.actor` event sets its `name` and `actor.kind`.
#[allow(clippy::too_many_arguments)]
pub fn add_actor(
    ch: &mut Chronicle,
    root: &Address,
    parent: Option<&Address>,
    id: &str,
    name: &str,
    kind: &str,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<Address, RecordError> {
    let base = match parent {
        Some(p) => {
            if p.last().kind() != "actor" || !root.contains(p) {
                return Err(RecordError::BadActor(format!(
                    "{p} is not an actor in {root}"
                )));
            }
            p.clone()
        }
        None => root.clone(),
    };
    let a = base
        .child_key("actor", id)
        .map_err(|e| RecordError::BadActor(format!("{id:?}: {e}")))?;
    if name.trim().is_empty() || !ident_ok(kind) {
        return Err(RecordError::BadActor(
            "an actor needs a name and a lowercase kind".into(),
        ));
    }
    let state = ch.state_at(UniverseTime::seconds(i64::MAX));
    if let Some(p) = parent {
        if state.get(p, ACTOR_KIND).is_none() {
            return Err(RecordError::BadActor(format!("no actor {p}")));
        }
    }
    if state.get(&a, ACTOR_KIND).is_some() {
        return Err(RecordError::BadActor(format!("{a} already exists")));
    }
    let e = Event::new(EventBody {
        kind: kinds::ACTOR.into(),
        subject: vec![a.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: ch.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: [(
                a.clone(),
                [
                    (ACTOR_NAME.to_string(), text(name)),
                    (ACTOR_KIND.to_string(), text(kind)),
                ]
                .into(),
            )]
            .into(),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    ch.append(e)?;
    Ok(a)
}

/// Every actor at or under `scope` as of `at`: address, name, kind.
pub fn actors(ch: &Chronicle, scope: &Address, at: UniverseTime) -> Vec<(Address, String, String)> {
    let s = ch.state_at(at);
    s.facts
        .iter()
        .filter(|(a, _)| scope.contains(a) && a.last().kind() == "actor")
        .filter_map(|(a, f)| match (f.get(ACTOR_NAME), f.get(ACTOR_KIND)) {
            (Some(Value::Text(n)), Some(Value::Text(k))) => Some((a.clone(), n.clone(), k.clone())),
            _ => None,
        })
        .collect()
}

pub mod holdings;
pub mod tech;

#[cfg(test)]
mod holdings_tests;
#[cfg(test)]
mod tests;
