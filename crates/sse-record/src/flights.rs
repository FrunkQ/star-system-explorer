//! Flight logs on the record (V8 flight log, part 2): every construct's
//! dated log of where it was and what happened to it, kept as the record's
//! own events so it is replayed, rewound and secret-able like any other
//! record fact. The travel part (sse-nav's `Piece` and `LogEvent` shapes)
//! rides as JSON text the record stores and never reads, as links do.
//!
//! One `record.flight` event per entry, about the construct (further
//! subjects: the constructs a split or join touches). An entry is a piece,
//! an event, or the log's name and about line; a withdrawal names the event
//! it takes back. Nothing is edited in place.

use crate::{field, kinds, RecordError};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::{BTreeMap, BTreeSet};

/// What an entry holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entry {
    /// A stretch of the craft's state (sse-nav `Piece`).
    Piece,
    /// Something that happens where pieces meet (sse-nav `LogEvent`).
    Event,
    /// The log's name and what it is (`{"name", "about"}`).
    About,
    /// Takes back an earlier entry (`body` is empty; `withdraws` names it).
    Withdraw,
}

impl Entry {
    fn word(self) -> &'static str {
        match self {
            Entry::Piece => "piece",
            Entry::Event => "event",
            Entry::About => "about",
            Entry::Withdraw => "withdraw",
        }
    }
    fn from_word(w: &str) -> Option<Entry> {
        Some(match w {
            "piece" => Entry::Piece,
            "event" => Entry::Event,
            "about" => Entry::About,
            "withdraw" => Entry::Withdraw,
            _ => return None,
        })
    }
}

/// An entry to write.
#[derive(Clone, PartialEq, Debug)]
pub struct FlightDraft {
    pub construct: Address,
    pub entry: Entry,
    /// JSON text (a piece, an event or `{name, about}`); empty for a withdrawal.
    pub body: String,
    /// Other constructs this entry touches (a split's new pieces, a join's partner).
    pub also: Vec<Address>,
    /// The entry this takes back (an event id), for `Entry::Withdraw`.
    pub withdraws: Option<String>,
    pub writer: String,
    pub reason: Option<String>,
}

/// An entry as the record reads it back.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FlightEntry {
    pub event: String,
    pub construct: Address,
    pub entry: Entry,
    pub body: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also: Vec<Address>,
    /// When it was written (the campaign clock), not the times inside it.
    pub written_s: i64,
    pub writer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

fn text(v: &str) -> Value {
    Value::Text(v.into())
}

/// Write one entry of a construct's flight log.
pub fn write(
    ch: &mut Chronicle,
    d: &FlightDraft,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, RecordError> {
    let bad = |w: String| RecordError::BadValue {
        kind: "flight".into(),
        why: w,
    };
    if d.writer.is_empty() {
        return Err(bad("no writer".into()));
    }
    match (d.entry, d.withdraws.as_deref()) {
        (Entry::Withdraw, None) => {
            return Err(bad("a withdrawal names the entry it takes back".into()))
        }
        (Entry::Withdraw, Some(id)) => {
            let found = ch.events().iter().any(|e| {
                e.id.hex() == id
                    && e.body.kind == kinds::FLIGHT
                    && e.body.subject.first() == Some(&d.construct)
            });
            if !found {
                return Err(bad(format!(
                    "{id} is not an entry of {}'s log",
                    d.construct
                )));
            }
        }
        (_, Some(_)) => return Err(bad("only a withdrawal names another entry".into())),
        _ => {
            serde_json::from_str::<serde_json::Value>(&d.body)
                .map_err(|e| bad(format!("the entry is not JSON: {e}")))?;
        }
    }
    let mut m = BTreeMap::new();
    m.insert("entry".to_string(), text(d.entry.word()));
    if !d.body.is_empty() {
        m.insert("body".into(), text(&d.body));
    }
    if let Some(w) = &d.withdraws {
        m.insert("withdraws".into(), text(w));
    }
    m.insert("writer".into(), text(&d.writer));
    if let Some(r) = &d.reason {
        m.insert("reason".into(), text(r));
    }
    let mut subject = vec![d.construct.clone()];
    let also: BTreeSet<&Address> = d.also.iter().filter(|a| **a != d.construct).collect();
    subject.extend(also.into_iter().cloned());
    let e = Event::new(EventBody {
        kind: kinds::FLIGHT.into(),
        subject,
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

fn entry_of(e: &Event) -> Option<(FlightEntry, Option<String>)> {
    if e.body.kind != kinds::FLIGHT {
        return None;
    }
    let Some(Value::Map(m)) = &e.body.payload.data else {
        return None;
    };
    let entry = Entry::from_word(field(m, "entry")?)?;
    Some((
        FlightEntry {
            event: e.id.hex(),
            construct: e.body.subject.first()?.clone(),
            entry,
            body: field(m, "body").unwrap_or("").to_string(),
            also: e.body.subject.iter().skip(1).cloned().collect(),
            written_s: e.body.occurred_at.s,
            writer: field(m, "writer").unwrap_or("").to_string(),
            reason: field(m, "reason").map(str::to_string),
        },
        field(m, "withdraws").map(str::to_string),
    ))
}

/// A construct's entries written by `at`, in the order written, without
/// withdrawn ones or the withdrawals themselves.
pub fn entries<'a>(
    events: impl IntoIterator<Item = &'a Event>,
    construct: &Address,
    at: UniverseTime,
) -> Vec<FlightEntry> {
    let mut sel: Vec<&Event> = events
        .into_iter()
        .filter(|e| {
            e.body.kind == kinds::FLIGHT
                && e.body.activated_at <= at
                && e.body.subject.first() == Some(construct)
        })
        .collect();
    sel.sort_by_key(|e| e.order_key());
    let mut out = vec![];
    let mut gone = BTreeSet::new();
    for e in sel {
        let Some((f, withdraws)) = entry_of(e) else {
            continue;
        };
        if let Some(w) = withdraws {
            gone.insert(w);
        } else {
            out.push(f);
        }
    }
    out.retain(|f| !gone.contains(&f.event));
    out
}

/// The constructs with a flight log under `bounds`, by address.
pub fn logged<'a>(events: impl IntoIterator<Item = &'a Event>, bounds: &Address) -> Vec<Address> {
    let set: BTreeSet<Address> = events
        .into_iter()
        .filter(|e| e.body.kind == kinds::FLIGHT)
        .filter_map(|e| e.body.subject.first().cloned())
        .filter(|a| bounds.contains(a))
        .collect();
    set.into_iter().collect()
}
