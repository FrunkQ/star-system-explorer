//! Look-ahead and commit (handoff 6.3, build plan T2).
//!
//! A look-ahead is a chronicle branch: an authored `time.request` (what was
//! asked: the system, the settings, how far), then the run's epoch steps as
//! `evolve.transition` snapshots and its named transitions as `time.<kind>`
//! events, every one derived, dated in universe time and carrying its
//! reason. Nothing touches the main chronicle until the GM commits; commit
//! re-stamps the branch onto the chronicle and writes `commit.branch`, and
//! may move the system's "now" there (an authored `system.now` fact).
//!
//! Replay: the request is in the chronicle, so `replay` rebuilds every
//! derived event from the authored ones and must arrive at the same head.

use crate::epoch::{Ageing, AgeingRun, SystemSpec, TimeEvent, TimeSettings};
use serde::{Deserialize, Serialize};
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, SessionStamp, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

pub const REQUEST: &str = "time.request";
pub const SYSTEM_NOW: &str = "system.now_year";

/// What a look-ahead was asked to do. Stored whole in the chronicle.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Request {
    pub format: String,
    pub system: SystemSpec,
    pub settings: TimeSettings,
    /// Star age to run to, Myr.
    pub target_myr: f64,
}

impl Request {
    pub fn new(system: SystemSpec, settings: TimeSettings, target_myr: f64) -> Self {
        Request {
            format: "sse.time-request/1".into(),
            system,
            settings,
            target_myr,
        }
    }
}

/// An open look-ahead.
#[derive(Clone, PartialEq, Debug)]
pub struct LookAhead {
    pub branch: ContentHash,
    pub run: AgeingRun,
    pub request: Request,
}

fn publisher() -> Publisher {
    Publisher::new("sse.time", "age", "1")
}

fn to_value(v: &impl Serialize) -> Result<Value, String> {
    serde_json::to_value(v)
        .and_then(serde_json::from_value)
        .map_err(|e| format!("value: {e}"))
}

fn addr(s: &str) -> Result<Address, String> {
    s.parse().map_err(|e| format!("address {s}: {e:?}"))
}

/// Universe time of a star age (Myr), to the whole year.
pub fn universe_time(spec: &SystemSpec, age_myr: f64) -> UniverseTime {
    UniverseTime::years(spec.birth_year + m::round(age_myr * 1.0e6) as i64)
}

/// The derived events of a request, in order, stamped from `first`.
fn derived(
    req: &Request,
    run: &AgeingRun,
    request_id: ContentHash,
    first: u64,
) -> Result<Vec<Event>, String> {
    let sp = &req.system;
    let star = addr(&sp.star_address)?;
    let planets: Vec<Address> = sp
        .planets
        .iter()
        .map(|p| addr(&p.address))
        .collect::<Result<_, _>>()?;
    let mut out: Vec<(UniverseTime, u8, EventBody)> = Vec::new();
    for snap in &run.snapshots {
        let at = universe_time(sp, snap.age_myr);
        let mut set = BTreeMap::new();
        let s = &snap.star;
        let sf: BTreeMap<String, Value> = [
            ("star.phase", Value::Text(s.phase.clone())),
            ("star.luminosity", Value::Float(s.luminosity)),
            ("star.radius", Value::Float(s.radius_rsun)),
            ("star.teff", Value::Float(s.teff)),
            ("star.mass", Value::Float(s.mass)),
            ("star.colour", Value::Text(s.colour.clone())),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        set.insert(star.clone(), sf);
        for (p, a) in snap.planets.iter().zip(&planets) {
            let mut f = BTreeMap::new();
            f.insert("body.engulfed".to_string(), Value::Bool(p.engulfed));
            if !p.engulfed {
                f.insert("orbit.a".to_string(), Value::Float(p.a_au));
                f.insert("orbit.e".to_string(), Value::Float(p.e));
                f.insert("orbit.i".to_string(), Value::Float(p.i_deg));
            }
            set.insert(a.clone(), f);
        }
        let mut subject = vec![star.clone()];
        subject.extend(planets.iter().cloned());
        out.push((
            at,
            1,
            EventBody {
                kind: kinds::EVOLVE_TRANSITION.into(),
                subject,
                occurred_at: at,
                activated_at: at,
                recognised_at: SessionStamp(0),
                causes: vec![request_id],
                payload: Payload {
                    set,
                    data: Some(to_value(&serde_json::json!({"age_myr": snap.age_myr}))?),
                    ..Default::default()
                },
                evidence: Evidence::Derived,
                publisher: publisher(),
            },
        ));
    }
    for e in &run.events {
        out.push((e.age_myr_time(sp), 0, named(e, sp, request_id)?));
    }
    // Named transitions before the snapshot at the same instant; stable otherwise.
    out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    out.into_iter()
        .enumerate()
        .map(|(k, (_, _, mut body))| {
            body.recognised_at = SessionStamp(first + k as u64);
            Event::new(body).map_err(|e| format!("{e:?}"))
        })
        .collect()
}

impl TimeEvent {
    fn age_myr_time(&self, sp: &SystemSpec) -> UniverseTime {
        universe_time(sp, self.age_myr)
    }
}

fn named(e: &TimeEvent, sp: &SystemSpec, request_id: ContentHash) -> Result<EventBody, String> {
    let at = universe_time(sp, e.age_myr);
    let subject = e
        .subjects
        .iter()
        .map(|s| addr(s))
        .collect::<Result<_, _>>()?;
    let mut set = BTreeMap::new();
    if e.kind == "engulfed" {
        let mut f = BTreeMap::new();
        f.insert("body.engulfed".to_string(), Value::Bool(true));
        set.insert(addr(&e.subjects[0])?, f);
    }
    Ok(EventBody {
        kind: format!("time.{}", e.kind),
        subject,
        occurred_at: at,
        activated_at: at,
        recognised_at: SessionStamp(0),
        causes: vec![request_id],
        payload: Payload {
            set,
            data: Some(to_value(&serde_json::json!({
                "why": e.why,
                "level": e.level,
                "plausible_future": e.plausible_future,
                "age_myr": e.age_myr,
                "numbers": e.numbers,
            }))?),
            ..Default::default()
        },
        evidence: Evidence::Derived,
        publisher: publisher(),
    })
}

fn request_event(
    ch: &Chronicle,
    req: &Request,
    stamp: u64,
    at: UniverseTime,
) -> Result<Event, String> {
    Event::new(EventBody {
        kind: REQUEST.into(),
        subject: vec![addr(&req.system.system)?],
        occurred_at: at,
        activated_at: at,
        recognised_at: SessionStamp(stamp),
        causes: vec![],
        payload: Payload {
            data: Some(to_value(req)?),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("gm", "look-ahead", "1"),
    })
    .map_err(|e| format!("{e:?} at {}", ch.len()))
}

/// Open a look-ahead on a branch. `go` sees the work done (deterministic
/// units) and may cancel; a cancelled look-ahead leaves no branch.
pub fn look_ahead(
    ch: &mut Chronicle,
    req: Request,
    go: &mut dyn FnMut(u64) -> bool,
) -> Result<Option<LookAhead>, String> {
    let ageing = Ageing::new(&req.settings, &req.system)?;
    let Some(run) = ageing.run_with(req.target_myr, go) else {
        return Ok(None);
    };
    let label = format!("age {} to {:.0} Myr", req.system.name, req.target_myr);
    let branch = ch.branch(&label);
    let first = ch.next_stamp().0;
    let now = universe_time(&req.system, req.system.age_myr);
    let r = request_event(ch, &req, first, now)?;
    let rid = ch.branch_append(&branch, r).map_err(|e| format!("{e:?}"))?;
    for e in derived(&req, &run, rid, first + 1)? {
        ch.branch_append(&branch, e).map_err(|e| format!("{e:?}"))?;
    }
    Ok(Some(LookAhead {
        branch,
        run,
        request: req,
    }))
}

/// Commit a look-ahead. With `move_now`, the system's "now" moves to that
/// star age (an authored fact on the system), the GM's "age the system".
pub fn commit(
    ch: &mut Chronicle,
    la: &LookAhead,
    move_now: Option<f64>,
) -> Result<ContentHash, String> {
    let sp = &la.request.system;
    let now = universe_time(sp, sp.age_myr);
    let id = ch
        .commit(
            &la.branch,
            now,
            vec![addr(&sp.system)?],
            Publisher::new("gm", "look-ahead", "1"),
        )
        .map_err(|e| format!("{e:?}"))?;
    if let Some(age) = move_now {
        let mut f = BTreeMap::new();
        let year = sp.birth_year + m::round(age * 1.0e6) as i64;
        f.insert(SYSTEM_NOW.to_string(), Value::Int(i128::from(year)));
        let mut set = BTreeMap::new();
        set.insert(addr(&sp.system)?, f);
        let e = Event::new(EventBody {
            kind: kinds::AUTHORED_SET.into(),
            subject: vec![addr(&sp.system)?],
            occurred_at: now,
            activated_at: now,
            recognised_at: ch.next_stamp(),
            causes: vec![id],
            payload: Payload {
                set,
                ..Default::default()
            },
            evidence: Evidence::Authored,
            publisher: Publisher::new("gm", "edit", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        return ch.append(e).map_err(|e| format!("{e:?}"));
    }
    Ok(id)
}

/// Rebuild a chronicle from its authored events alone: every `time.request`
/// is run again and its derived events regenerated in place. Any difference
/// from the original shows as a different head (or a commit whose causes are
/// missing).
pub fn replay(original: &Chronicle) -> Result<Chronicle, String> {
    let mut ch = Chronicle::new();
    let evs = original.events();
    let mut k = 0;
    while k < evs.len() {
        let e = &evs[k];
        match e.body.evidence {
            Evidence::Authored | Evidence::AuthoredImpossible => {
                ch.append(e.clone()).map_err(|x| format!("{x:?}"))?;
                if e.body.kind == REQUEST {
                    let v = e.body.payload.data.clone().ok_or("request without data")?;
                    let json = serde_json::to_value(&v).map_err(|x| x.to_string())?;
                    let req: Request = serde_json::from_value(json).map_err(|x| x.to_string())?;
                    let ageing = Ageing::new(&req.settings, &req.system)?;
                    let run = ageing.run(req.target_myr);
                    for d in derived(&req, &run, e.id, e.body.recognised_at.0 + 1)? {
                        ch.append(d).map_err(|x| format!("{x:?}"))?;
                    }
                }
            }
            _ => {
                // Derived: regenerated above (or from another engine: copied).
                if !e
                    .body
                    .causes
                    .iter()
                    .any(|c| evs.iter().any(|x| x.id == *c && x.body.kind == REQUEST))
                {
                    ch.append(e.clone()).map_err(|x| format!("{x:?}"))?;
                }
            }
        }
        k += 1;
    }
    Ok(ch)
}
