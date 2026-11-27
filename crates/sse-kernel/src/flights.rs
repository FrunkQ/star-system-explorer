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
use sse_nav::flightlog::{
    elements_from_state, push_direction, Ephemeris, FlightLog, LogEvent, Mark, MountSettings,
    Piece, State, What,
};
use sse_nav::kepler::{lambert, norm, sub, V3};
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
                        Some(s @ (State::Orbit { .. } | State::Landed { .. } | State::Docked { .. } | State::Mounted { .. } | State::BetweenStars { .. })) => {
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

/// Where the craft was at `t_s` in this system; while it is docked or
/// mounted, at its compound root's position (CM2).
pub(crate) fn where_at(
    record: &Chronicle,
    root: &Address,
    log: &FlightLog,
    t_s: i64,
    eph: &dyn Ephemeris,
) -> serde_json::Value {
    serde_json::to_value(where_followed(record, root, log, t_s, eph)).expect("plain")
}

// ------------------------------------------------------------ CM2: mounts

/// The nav settings' `mounts` block in force (base-nav), if the stack has one.
pub(crate) fn mount_settings() -> Option<MountSettings> {
    sse_physics::settings("nav")
        .and_then(|n| serde_json::to_value(n).ok())
        .and_then(|n| MountSettings::from_nav(&n).ok())
}

/// The construct a docked or mounted piece names, as an address.
fn carrier_of(root: &Address, c: &Address, s: &State) -> Option<Address> {
    match s {
        State::Docked { to } | State::Mounted { to, .. } => other(root, c, to).ok(),
        _ => None,
    }
}

/// How deep a chain of carriers is followed (a broken record never hangs a read).
const DEPTH: usize = 32;

/// The piece that says where `c` is at `t_s`: its own, or, while it is
/// docked or mounted, its carrier's, up to the root of the compound. With the
/// construct whose log holds it.
fn root_piece(
    record: &Chronicle,
    root: &Address,
    c: &Address,
    t_s: i64,
) -> Option<(Address, FlightLog, Piece)> {
    let mut cur = c.clone();
    for _ in 0..DEPTH {
        let (log, _) = assemble(record, &cur, UniverseTime::seconds(i64::MAX));
        let p = log.piece_at(t_s).map(|(_, p)| p.clone())?;
        match carrier_of(root, &cur, &p.state) {
            Some(up) if up != *c => cur = up,
            _ => return Some((cur, log, p)),
        }
    }
    None
}

/// Where `c` was at `t_s`: its own log's answer, with the position and
/// velocity of the compound's root while it is docked or mounted (design 6.1).
pub(crate) fn where_followed(
    record: &Chronicle,
    root: &Address,
    log: &FlightLog,
    t_s: i64,
    eph: &dyn Ephemeris,
) -> sse_nav::flightlog::At {
    let mut at = log.at(t_s, eph);
    if at.r_m.is_none() {
        let own: Option<Address> = log.construct.parse().ok();
        let up = own
            .zip(log.piece_at(t_s))
            .and_then(|(own, (_, p))| carrier_of(root, &own, &p.state));
        if let Some((_, l, _)) = up.and_then(|up| root_piece(record, root, &up, t_s)) {
            let there = l.at(t_s, eph);
            at.r_m = there.r_m;
            at.v_m_s = there.v_m_s;
        }
    }
    at
}

fn draft(construct: &Address, entry: Entry, body: String, also: Vec<Address>) -> FlightDraft {
    FlightDraft {
        construct: construct.clone(),
        entry,
        body,
        also,
        withdraws: None,
        writer: "gm".into(),
        reason: None,
    }
}

/// Write a release into the flight logs (design 6.2): a `split` event on the
/// carrier's log naming the released construct, and the released
/// construct's first free piece, seeded from the compound root's state at
/// that second, changed by the separation push (along the flight for the
/// front, against it behind, outward for the rest). With no system the
/// state is copied and the push noted; on a course a system is needed.
/// Nothing is written when neither has a log. Returns the entry ids and notes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn on_release(
    record: &mut Chronicle,
    root: &Address,
    carrier: &Address,
    carried: &Address,
    place: &str,
    push_m_s: Option<f64>,
    at_s: i64,
    eph: Option<&dyn Ephemeris>,
) -> Result<(Vec<String>, Vec<String>), Response> {
    let mut notes = vec![];
    let Some((who, wlog, piece)) = root_piece(record, root, carrier, at_s) else {
        let (own, _) = assemble(record, carried, UniverseTime::seconds(i64::MAX));
        if !own.pieces.is_empty() {
            notes.push(format!(
                "{carrier}'s flight log says nothing at that second, so {carried}'s log gets no free piece: give it one"
            ));
        }
        return Ok((vec![], notes));
    };
    let push = match push_m_s.or_else(|| mount_settings().map(|s| s.push_m_s)) {
        Some(p) => p,
        None => {
            notes.push(
                "no separation push and no nav.mounts settings on the stack, so no push".into(),
            );
            0.0
        }
    };
    let seeded = |r: V3, v: V3, mu: f64, around: &str| -> Option<State> {
        let d = push_direction(place, r, v);
        let v2 = [v[0] + d[0] * push, v[1] + d[1] * push, v[2] + d[2] * push];
        elements_from_state(r, v2, mu, at_s).map(|elements| State::Orbit {
            around: around.to_string(),
            elements,
        })
    };
    let state = match (&piece.state, eph) {
        (State::Orbit { around, elements }, Some(e)) => match e.body(around, at_s as f64) {
            Some((_, _, mu)) => {
                let (r, v) = elements.state(mu, (at_s - elements.epoch_s) as f64);
                seeded(r, v, mu, around)
            }
            None => {
                notes.push(format!(
                    "the system holds no {around}, so the orbit is copied without the push"
                ));
                Some(piece.state.clone())
            }
        },
        (State::Orbit { .. }, None) => {
            if push > 0.0 {
                notes.push("no system given, so the orbit is copied without the push".into());
            }
            Some(piece.state.clone())
        }
        (State::Course { .. }, Some(e)) => {
            let at = wlog.at(at_s, e);
            match (at.r_m, at.v_m_s) {
                (Some(r), Some(v)) => seeded(r, v, e.star_mu(), "star"),
                _ => None,
            }
        }
        (s @ (State::Landed { .. } | State::BetweenStars { .. }), _) => Some(s.clone()),
        _ => None,
    };
    let Some(state) = state else {
        notes.push(format!(
            "{who}'s log at that second cannot seed {carried} (a course needs a system; a track or an end cannot): give it its own first piece"
        ));
        return Ok((vec![], notes));
    };
    let why = format!(
        "released from {carrier} ({place}){}",
        if push > 0.0 {
            format!(", pushed {push} m/s")
        } else {
            String::new()
        }
    );
    let at = UniverseTime::seconds(at_s);
    let publisher = Publisher::new("gm", "release", "1");
    let ev = LogEvent {
        at_s,
        what: What::Split {
            into: vec![carried.to_string()],
        },
        mark: Mark::Authored,
        src: None,
        why: Some(why.clone()),
        writer: Some("gm".into()),
    };
    let p = Piece {
        from_s: at_s,
        to_s: None,
        state,
        mark: Mark::Authored,
        src: None,
        why: Some(why),
        writer: Some("gm".into()),
    };
    let a = flights::write(
        record,
        &draft(
            carrier,
            Entry::Event,
            serde_json::to_string(&ev).expect("plain"),
            vec![carried.clone()],
        ),
        at,
        &publisher,
    )
    .map_err(record_err)?;
    let b = flights::write(
        record,
        &draft(
            carried,
            Entry::Piece,
            serde_json::to_string(&p).expect("plain"),
            vec![carrier.clone()],
        ),
        at,
        &publisher,
    )
    .map_err(record_err)?;
    Ok((vec![a.hex(), b.hex()], notes))
}

/// How far apart and how fast two constructs were at `t_s`, by their logs.
fn gap(
    record: &Chronicle,
    root: &Address,
    a: &Address,
    b: &Address,
    t_s: i64,
    eph: &dyn Ephemeris,
) -> Option<(f64, f64)> {
    let pos = |c: &Address| {
        let (log, _) = assemble(record, c, UniverseTime::seconds(i64::MAX));
        let at = where_followed(record, root, &log, t_s, eph);
        Some((at.r_m?, at.v_m_s?))
    };
    let ((ra, va), (rb, vb)) = (pos(a)?, pos(b)?);
    Some((norm(sub(ra, rb)), norm(sub(va, vb))))
}

/// The cheapest rendezvous of one transfer arc from where `chaser` is at
/// `t_s` to where `target` will be, over flight times that are fractions of
/// the target's orbital period: `(leave_s, arrive_s, dv_m_s)`, the Δv to
/// leave plus the Δv to match. None unless the target's compound root is on
/// a closed orbit.
fn rendezvous(
    record: &Chronicle,
    root: &Address,
    chaser: &Address,
    target: &Address,
    t_s: i64,
    eph: &dyn Ephemeris,
    fractions: &[f64],
) -> Option<(i64, i64, f64)> {
    let (_, _, p) = root_piece(record, root, target, t_s)?;
    let State::Orbit { around, elements } = &p.state else {
        return None;
    };
    let (_, _, mu) = eph.body(around, t_s as f64)?;
    if !(elements.e < 1.0 && elements.a_m > 0.0) {
        return None;
    }
    let a = elements.a_m;
    let period = sse_core::math::TAU * sse_core::math::sqrt(a * a * a / mu);
    let rel = |c: &Address, t: i64| {
        let (log, _) = assemble(record, c, UniverseTime::seconds(i64::MAX));
        let at = where_followed(record, root, &log, t, eph);
        let (rb, vb, _) = eph.body(around, t as f64)?;
        Some((sub(at.r_m?, rb), sub(at.v_m_s?, vb)))
    };
    let (r1, v1c) = rel(chaser, t_s)?;
    let mut best: Option<(i64, i64, f64)> = None;
    for f in fractions {
        let tof = sse_core::math::round(f * period) as i64;
        if tof <= 0 {
            continue;
        }
        let Some((r2, v2t)) = rel(target, t_s + tof) else {
            continue;
        };
        let Some((v1, v2)) = lambert(r1, r2, tof as f64, mu) else {
            continue;
        };
        let dv = norm(sub(v1, v1c)) + norm(sub(v2t, v2));
        if dv.is_finite() && best.is_none_or(|b| dv < b.2) {
            best = Some((t_s, t_s + tof, dv));
        }
    }
    best
}

/// Check a capture against both flight logs (design 6.3): both craft within
/// `capture_reach_m` and `capture_speed_m_s` of each other at that second.
/// Not within reach: refused, with how far apart they are and a rendezvous
/// offer, unless the GM writes it `authored`. Unchecked (with a note) when
/// there is no system, no settings or a log that cannot say.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_capture(
    record: &Chronicle,
    root: &Address,
    carrier: &Address,
    carried: &Address,
    at_s: i64,
    eph: Option<&dyn Ephemeris>,
    authored: bool,
    notes: &mut Vec<String>,
) -> Result<(), Response> {
    let Some(eph) = eph else {
        notes.push("no system given, so where the two craft were is not checked".into());
        return Ok(());
    };
    let Some(s) = mount_settings() else {
        notes.push(
            "no nav.mounts settings on the stack, so the capture's reach is not checked".into(),
        );
        return Ok(());
    };
    let Some((d, v)) = gap(record, root, carried, carrier, at_s, eph) else {
        notes.push(
            "a flight log cannot say where one of the craft was, so the capture is not checked"
                .into(),
        );
        return Ok(());
    };
    if d <= s.capture_reach_m && v <= s.capture_speed_m_s {
        return Ok(());
    }
    let apart = format!(
        "{carried} is {} m from {carrier} and {} m/s apart at that second (a capture needs {} m and {} m/s)",
        sse_core::math::round(d), sse_core::math::round(v * 100.0) / 100.0,
        s.capture_reach_m, s.capture_speed_m_s
    );
    if authored {
        notes.push(format!("written as authored: {apart}"));
        return Ok(());
    }
    let offer = match rendezvous(record, root, carried, carrier, at_s, eph, &s.rendezvous_tof_fractions) {
        Some((l, a, dv)) => format!(
            "; a rendezvous leaving at {l} s and meeting at {a} s costs {} m/s from {carried}'s own tanks",
            sse_core::math::round(dv * 10.0) / 10.0
        ),
        None => String::new(),
    };
    Err(err(
        "capture_apart",
        format!("{apart}{offer}; plan the rendezvous, or write the capture with authored: true"),
    ))
}

/// Write a capture into the flight logs: a `join` event on the carrier's
/// log and a `mounted` piece on the carried construct's, when either has a log.
pub(crate) fn on_capture(
    record: &mut Chronicle,
    root: &Address,
    carrier: &Address,
    carried: &Address,
    place: &str,
    port: Option<&str>,
    at_s: i64,
) -> Result<Vec<String>, Response> {
    let any = |c: &Address| {
        !assemble(record, c, UniverseTime::seconds(i64::MAX))
            .0
            .pieces
            .is_empty()
    };
    if !(any(carried) || root_piece(record, root, carrier, at_s).is_some() || any(carrier)) {
        return Ok(vec![]);
    }
    let at = UniverseTime::seconds(at_s);
    let publisher = Publisher::new("gm", "mount", "1");
    let why = format!("captured by {carrier} ({place})");
    let ev = LogEvent {
        at_s,
        what: What::Join {
            with: carried.to_string(),
        },
        mark: Mark::Authored,
        src: None,
        why: Some(why.clone()),
        writer: Some("gm".into()),
    };
    let p = Piece {
        from_s: at_s,
        to_s: None,
        state: State::Mounted {
            to: carrier.to_string(),
            place: place.to_string(),
            port: port.map(str::to_string),
        },
        mark: Mark::Authored,
        src: None,
        why: Some(why),
        writer: Some("gm".into()),
    };
    let a = flights::write(
        record,
        &draft(
            carrier,
            Entry::Event,
            serde_json::to_string(&ev).expect("plain"),
            vec![carried.clone()],
        ),
        at,
        &publisher,
    )
    .map_err(record_err)?;
    let b = flights::write(
        record,
        &draft(
            carried,
            Entry::Piece,
            serde_json::to_string(&p).expect("plain"),
            vec![carrier.clone()],
        ),
        at,
        &publisher,
    )
    .map_err(record_err)?;
    Ok(vec![a.hex(), b.hex()])
}
