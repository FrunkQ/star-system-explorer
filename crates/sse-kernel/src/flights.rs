//! The flight log on the record (V8 flight log, part 2): `log_flight` writes
//! one entry of a construct's log (sse-record `flights`), checked against
//! sse-nav's `Piece` and `LogEvent` shapes; `flight_log` assembles the log
//! and, given a time and a system, says where the craft was.
//!
//! Splitting and joining are one mechanism: a `split` event seeds each new
//! construct's log with the state its parent was in at that moment (an
//! orbit, a landing, a dock, a leg between the stars), and a `join` docks the
//! construct it names to the one whose log it is. The construct half (parts
//! and mass moving) is the kernel's construct ops; this is only the log.

use crate::{err, record_err, Response};
use sse_chronicle::{Chronicle, Publisher, UniverseTime};
use sse_core::Address;
use sse_nav::flightlog::{Ephemeris, FlightLog, LogEvent, Piece, State, What};
use sse_record::flights::{self, Entry, FlightDraft, FlightEntry};

fn bad(m: impl Into<String>) -> Response {
    err("bad_flight", m)
}

/// A construct a split or join names: an address, or a bare key for a
/// sibling of the log's own construct (`centaur` beside `u:sol/construct:voyager-2`).
fn other(root: &Address, construct: &Address, c: &str) -> Result<Address, Response> {
    if c.contains(':') {
        return crate::in_universe(root, c);
    }
    let kind = construct.last().kind().to_string();
    construct
        .parent()
        .and_then(|p| p.child_key(&kind, c).ok())
        .ok_or_else(|| {
            bad(format!(
                "{c}: give an address, or a key for a sibling of {construct}"
            ))
        })
}

/// The log of `construct` from the record's entries written by `at`.
pub(crate) fn assemble(
    record: &Chronicle,
    construct: &Address,
    at: UniverseTime,
) -> (FlightLog, Vec<FlightEntry>) {
    let es = flights::entries(record.events(), construct, at);
    let mut log = FlightLog {
        construct: construct.to_string(),
        name: String::new(),
        about: String::new(),
        pieces: vec![],
        events: vec![],
    };
    for e in &es {
        match e.entry {
            Entry::Piece => {
                if let Ok(p) = serde_json::from_str::<Piece>(&e.body) {
                    log.pieces.push(p);
                }
            }
            Entry::Event => {
                if let Ok(v) = serde_json::from_str::<LogEvent>(&e.body) {
                    log.events.push(v);
                }
            }
            Entry::About => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&e.body) {
                    if let Some(n) = v["name"].as_str() {
                        log.name = n.to_string();
                    }
                    if let Some(a) = v["about"].as_str() {
                        log.about = a.to_string();
                    }
                }
            }
            Entry::Withdraw => {}
        }
    }
    // By time; pieces written for the same start keep the order written.
    log.pieces.sort_by_key(|p| p.from_s);
    log.events.sort_by_key(|e| e.at_s);
    (log, es)
}

/// An entry as the reply lists it.
pub(crate) fn entry_json(e: &FlightEntry) -> serde_json::Value {
    let mut v = serde_json::json!({
        "event": e.event, "entry": e.entry, "written_s": e.written_s, "writer": e.writer,
        "body": serde_json::from_str::<serde_json::Value>(&e.body).unwrap_or(serde_json::Value::Null),
    });
    if !e.also.is_empty() {
        v["also"] = e
            .also
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .into();
    }
    if let Some(r) = &e.reason {
        v["reason"] = r.clone().into();
    }
    v
}

pub(crate) struct Write<'a> {
    pub root: &'a Address,
    pub construct: &'a str,
    pub piece: Option<serde_json::Value>,
    pub event: Option<serde_json::Value>,
    pub name: Option<String>,
    pub about: Option<String>,
    pub withdraw: Option<String>,
    pub writer: String,
    pub reason: Option<String>,
    pub at_s: i64,
}

/// Write one entry (and, for a split or a join, the entries it seeds in the
/// other constructs' logs). Returns the entry's id, the seeded ids and notes.
pub(crate) fn write(
    record: &mut Chronicle,
    w: Write,
) -> Result<(String, Vec<String>, Vec<String>), Response> {
    let construct = crate::in_universe(w.root, w.construct)?;
    let at = UniverseTime::seconds(w.at_s);
    let publisher = Publisher::new(&w.writer, "log_flight", "1");
    let given = [
        w.piece.is_some(),
        w.event.is_some(),
        w.name.is_some() || w.about.is_some(),
        w.withdraw.is_some(),
    ];
    if given.iter().filter(|g| **g).count() != 1 {
        return Err(bad(
            "give one of a piece, an event, a name and about line, or an entry to withdraw",
        ));
    }
    let mut also = vec![];
    let mut seeds: Vec<(Address, Piece)> = vec![];
    let mut notes = vec![];
    let (entry, body) = if let Some(p) = w.piece {
        let piece: Piece = serde_json::from_value(p).map_err(|e| bad(format!("piece: {e}")))?;
        if piece.to_s.is_some_and(|t| t < piece.from_s) {
            return Err(bad("a piece ends after it starts"));
        }
        (Entry::Piece, serde_json::to_string(&piece).expect("plain"))
    } else if let Some(v) = w.event {
        let ev: LogEvent = serde_json::from_value(v).map_err(|e| bad(format!("event: {e}")))?;
        let (log, _) = assemble(record, &construct, UniverseTime::seconds(i64::MAX));
        let seed = |state: State, why: String| Piece {
            from_s: ev.at_s,
            to_s: None,
            state,
            mark: ev.mark,
            src: ev.src.clone(),
            why: Some(why),
            writer: Some(w.writer.clone()),
        };
        match &ev.what {
            What::Split { into } => {
                let here = log.piece_at(ev.at_s).map(|(_, p)| p.state.clone());
                for c in into {
                    let a = other(w.root, &construct, c)?;
                    if a == construct {
                        return Err(bad("a construct does not split into itself"));
                    }
                    also.push(a.clone());
                    match &here {
                        Some(s @ (State::Orbit { .. } | State::Landed { .. } | State::Docked { .. } | State::BetweenStars { .. })) => {
                            seeds.push((a.clone(), seed(s.clone(), format!("split from {construct}"))));
                        }
                        Some(_) => notes.push(format!("{a} split from {construct} on a course or past its end: give it its own first piece")),
                        None => notes.push(format!("{construct}'s log says nothing at the split, so {a}'s log starts empty")),
                    }
                }
            }
            What::Join { with } => {
                let a = other(w.root, &construct, with)?;
                if a == construct {
                    return Err(bad("a construct does not join itself"));
                }
                also.push(a.clone());
                seeds.push((
                    a,
                    seed(
                        State::Docked {
                            to: construct.to_string(),
                        },
                        format!("joined {construct}"),
                    ),
                ));
            }
            _ => {}
        }
        (Entry::Event, serde_json::to_string(&ev).expect("plain"))
    } else if let Some(id) = w.withdraw {
        let d = FlightDraft {
            construct: construct.clone(),
            entry: Entry::Withdraw,
            body: String::new(),
            also: vec![],
            withdraws: Some(id),
            writer: w.writer.clone(),
            reason: w.reason.clone(),
        };
        let h = flights::write(record, &d, at, &publisher).map_err(record_err)?;
        return Ok((h.hex(), vec![], vec![]));
    } else {
        let mut v = serde_json::json!({});
        if let Some(n) = w.name {
            v["name"] = n.into();
        }
        if let Some(a) = w.about {
            v["about"] = a.into();
        }
        (Entry::About, v.to_string())
    };
    let d = FlightDraft {
        construct: construct.clone(),
        entry,
        body,
        also,
        withdraws: None,
        writer: w.writer.clone(),
        reason: w.reason.clone(),
    };
    let h = flights::write(record, &d, at, &publisher).map_err(record_err)?;
    let mut seeded = vec![];
    for (a, p) in seeds {
        let d = FlightDraft {
            construct: a,
            entry: Entry::Piece,
            body: serde_json::to_string(&p).expect("plain"),
            also: vec![construct.clone()],
            withdraws: None,
            writer: w.writer.clone(),
            reason: w.reason.clone(),
        };
        seeded.push(
            flights::write(record, &d, at, &publisher)
                .map_err(record_err)?
                .hex(),
        );
    }
    Ok((h.hex(), seeded, notes))
}

/// Where the craft was at `t_s` in this system.
pub(crate) fn where_at(log: &FlightLog, t_s: i64, eph: &dyn Ephemeris) -> serde_json::Value {
    serde_json::to_value(log.at(t_s, eph)).expect("plain")
}
