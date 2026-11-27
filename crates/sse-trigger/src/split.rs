//! Split and join of constructs at a dated event (RS4, construct half).
//!
//! A construct's **parts** are its facts named `part.<id>.<field>` (a stage,
//! a lander, a service module, a cargo pod), each part's mass in
//! `part.<id>.mass_kg`. The construct's whole mass is `construct.mass_kg`
//! when it has one.
//!
//! - **Split** makes a new construct of the same kind beside the old one
//!   (under the same parent) and moves the named parts to it, with their
//!   mass and any further mass named (propellant left in a stage). The new
//!   construct carries `split.from`. One `construct.split` event writes
//!   both, so the record reads the same before and after the date.
//! - **Join** is one of three: `dock` (both stay, the other carries
//!   `docked.to`), `undock` (that fact goes), or `merge` (every part and the
//!   mass move across and the other is retired, `joined.into` naming where
//!   it went). One `construct.join` event.
//!
//! A mount (CM1, `mount.rs`) is the general form of a dock: a dock is
//! read as a latch on the hull, a mounted construct cannot also dock, and
//! a merge refuses while the merged construct still carries anything.
//!
//! Nothing is deleted: moved facts are unset on the old construct in the
//! same event, so the history still shows them where they were.
//! The flight log (the travel thread's `record.flight`) writes its own
//! Split and Join entries with the address returned here.

use crate::templates::fields;
use sse_chronicle::UniverseTime;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, State};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

pub mod kinds {
    pub const SPLIT: &str = "construct.split";
    pub const JOIN: &str = "construct.join";
}

pub const PART: &str = "part.";
pub const MASS: &str = "construct.mass_kg";
pub const PART_MASS: &str = "mass_kg";
pub const FROM: &str = "split.from";
pub const DOCKED: &str = "docked.to";
pub const JOINED: &str = "joined.into";

/// How two constructs join.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum How {
    Dock,
    Undock,
    Merge,
}

/// What a split asks for.
pub struct SplitAsk<'a> {
    pub address: &'a Address,
    pub key: Option<&'a str>,
    pub name: Option<&'a str>,
    pub parts: &'a [String],
    /// Mass moved beyond the named parts' own, kg (propellant, cargo).
    pub mass_kg: Option<f64>,
    pub at: UniverseTime,
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// Facts of `a` at `at`, refused when it is not there or retired.
fn live<'s>(st: &'s State, a: &Address) -> Result<&'s BTreeMap<String, Value>, String> {
    let row = st
        .facts
        .get(a)
        .ok_or_else(|| format!("{a} is not in the universe at that time"))?;
    if row.get(fields::RETIRED) == Some(&Value::Bool(true)) {
        return Err(format!("{a} is retired"));
    }
    Ok(row)
}

/// The fields of part `id` on a row.
fn part_fields(row: &BTreeMap<String, Value>, id: &str) -> Vec<(String, Value)> {
    let p = format!("{PART}{id}.");
    row.iter()
        .filter(|(k, _)| k.starts_with(&p))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Every part id on a row, in order.
pub fn parts_of(row: &BTreeMap<String, Value>) -> Vec<String> {
    let mut ids: Vec<String> = row
        .keys()
        .filter_map(|k| k.strip_prefix(PART))
        .filter_map(|r| r.split('.').next())
        .map(str::to_string)
        .collect();
    ids.dedup();
    ids
}

fn part_mass(fs: &[(String, Value)]) -> f64 {
    fs.iter()
        .filter(|(k, _)| k.rsplit('.').next() == Some(PART_MASS))
        .filter_map(|(_, v)| num(v))
        .sum()
}

fn event(
    kind: &str,
    subject: Vec<Address>,
    at: UniverseTime,
    payload: Payload,
    universe: &Chronicle,
    publisher: &Publisher,
) -> Result<Event, String> {
    Event::new(EventBody {
        kind: kind.into(),
        subject,
        occurred_at: at,
        activated_at: at,
        recognised_at: universe.next_stamp(),
        causes: vec![],
        payload,
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })
    .map_err(|e| e.to_string())
}

/// Split `ask.address` at `ask.at`: the new construct's address and the event.
pub fn split(
    universe: &mut Chronicle,
    root: &Address,
    ask: &SplitAsk,
    publisher: &Publisher,
) -> Result<(Address, ContentHash), String> {
    let a = ask.address;
    if !root.contains(a) || a == root {
        return Err(format!("{a} is not a construct in universe {root}"));
    }
    let st = universe.state_at(ask.at);
    let row = live(&st, a)?;
    let parent = a
        .parent()
        .ok_or_else(|| format!("{a} has no parent to split beside"))?;
    let kind = a.last().kind().to_string();
    let latest = universe.state_at(UniverseTime::seconds(i64::MAX));
    let taken = |x: &Address| latest.facts.contains_key(x);
    let child = match ask.key {
        Some(k) => {
            let c = parent
                .child_key(&kind, k)
                .map_err(|e| format!("{k:?}: {e}"))?;
            if taken(&c) {
                return Err(format!("{c} already exists"));
            }
            c
        }
        None => (1..=100_000u32)
            .map(|n| parent.child_key(&kind, &n.to_string()))
            .find(|c| c.as_ref().is_ok_and(|c| !taken(c)))
            .ok_or_else(|| format!("no free {kind} address under {parent}"))?
            .map_err(|e| e.to_string())?,
    };
    let mut moved: Vec<(String, Value)> = vec![];
    for id in ask.parts {
        let fs = part_fields(row, id);
        if fs.is_empty() {
            let have = parts_of(row);
            return Err(format!(
                "{a} has no part {id:?}; parts: {}",
                if have.is_empty() {
                    "none".to_string()
                } else {
                    have.join(", ")
                }
            ));
        }
        moved.extend(fs);
    }
    let extra = ask.mass_kg.unwrap_or(0.0);
    if !(extra >= 0.0 && extra.is_finite()) {
        return Err("mass_kg: a mass of 0 or more".into());
    }
    if moved.is_empty() && extra == 0.0 {
        return Err("name parts to move, or a mass".into());
    }
    let mass = part_mass(&moved) + extra;
    let mut old = BTreeMap::new();
    let had = row.get(MASS).and_then(num);
    if let Some(m) = had {
        if mass > m * (1.0 + 1e-9) {
            return Err(format!("{a} has {m} kg; the split takes {mass} kg"));
        }
        old.insert(MASS.to_string(), Value::Float((m - mass).max(0.0)));
    }
    let own = |k: &str| {
        row.get(k).and_then(|v| match v {
            Value::Text(t) => Some(t.clone()),
            _ => None,
        })
    };
    let name = ask
        .name
        .filter(|n| !n.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let base = own(fields::NAME).unwrap_or_else(|| a.to_string());
            match ask.parts.first() {
                Some(p) => format!("{base} {p}"),
                None => format!("{base} part"),
            }
        });
    let mut new: BTreeMap<String, Value> = moved.iter().cloned().collect();
    new.insert(fields::NAME.into(), Value::Text(name));
    new.insert(FROM.into(), Value::Text(a.to_string()));
    new.insert(format!("{}construct", crate::TAG_FIELD), Value::Bool(true));
    if had.is_some() || mass > 0.0 {
        new.insert(MASS.into(), Value::Float(mass));
    }
    let mut set = BTreeMap::new();
    if !old.is_empty() {
        set.insert(a.clone(), old);
    }
    set.insert(child.clone(), new);
    let mut unset = BTreeMap::new();
    if !moved.is_empty() {
        unset.insert(a.clone(), moved.iter().map(|(k, _)| k.clone()).collect());
    }
    let mut data = BTreeMap::new();
    data.insert(
        "parts".to_string(),
        Value::Array(ask.parts.iter().map(|p| Value::Text(p.clone())).collect()),
    );
    data.insert("mass_kg".to_string(), Value::Float(mass));
    let e = event(
        kinds::SPLIT,
        vec![a.clone(), child.clone()],
        ask.at,
        Payload {
            set,
            unset,
            data: Some(Value::Map(data)),
        },
        universe,
        publisher,
    )?;
    let id = universe.append(e).map_err(|e| e.to_string())?;
    Ok((child, id))
}

/// Join `with` to `a` at `at`.
pub fn join(
    universe: &mut Chronicle,
    root: &Address,
    a: &Address,
    with: &Address,
    how: How,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, String> {
    for x in [a, with] {
        if !root.contains(x) || x == root {
            return Err(format!("{x} is not a construct in universe {root}"));
        }
    }
    if a == with {
        return Err(format!("{a} cannot join itself"));
    }
    let st = universe.state_at(at);
    let host = live(&st, a)?;
    let other = live(&st, with)?;
    let docked = other.get(DOCKED).and_then(|v| match v {
        Value::Text(t) => Some(t.as_str()),
        _ => None,
    });
    let here = a.to_string();
    let mut set = BTreeMap::new();
    let mut unset = BTreeMap::new();
    match how {
        How::Dock => {
            if let Some(d) = docked {
                return Err(format!("{with} is already docked to {d}"));
            }
            if let Some(Value::Text(on)) = other.get(crate::mount::ON) {
                return Err(format!(
                    "{with} is already mounted on {on}; release it first"
                ));
            }
            set.insert(
                with.clone(),
                [(DOCKED.to_string(), Value::Text(here))].into(),
            );
        }
        How::Undock => {
            if docked != Some(here.as_str()) {
                return Err(format!("{with} is not docked to {a}"));
            }
            unset.insert(with.clone(), vec![DOCKED.to_string()]);
        }
        How::Merge => {
            // What `with` carries would be left on a retired construct.
            if let Some((x, _)) = crate::mount::carried_by(&st, with).first() {
                return Err(format!(
                    "{x} is carried by {with}; release it, or mount it on {a}, before the merge"
                ));
            }
            let ids = parts_of(other);
            if let Some(clash) = ids.iter().find(|id| !part_fields(host, id).is_empty()) {
                return Err(format!(
                    "{a} already has a part {clash:?}; split it under another id first"
                ));
            }
            let moved: Vec<(String, Value)> =
                ids.iter().flat_map(|id| part_fields(other, id)).collect();
            let mut row: BTreeMap<String, Value> = moved.iter().cloned().collect();
            let (mh, mo) = (host.get(MASS).and_then(num), other.get(MASS).and_then(num));
            if mh.is_some() || mo.is_some() {
                row.insert(
                    MASS.into(),
                    Value::Float(mh.unwrap_or(0.0) + mo.unwrap_or(0.0)),
                );
            }
            if !row.is_empty() {
                set.insert(a.clone(), row);
            }
            set.insert(
                with.clone(),
                [
                    (fields::RETIRED.to_string(), Value::Bool(true)),
                    (JOINED.to_string(), Value::Text(here)),
                ]
                .into(),
            );
            let mut gone: Vec<String> = moved.into_iter().map(|(k, _)| k).collect();
            if docked.is_some() {
                gone.push(DOCKED.into());
            }
            gone.extend(
                crate::mount::ALL
                    .iter()
                    .filter(|k| other.contains_key(**k))
                    .map(|k| k.to_string()),
            );
            if !gone.is_empty() {
                unset.insert(with.clone(), gone);
            }
        }
    }
    let how_word = match how {
        How::Dock => "dock",
        How::Undock => "undock",
        How::Merge => "merge",
    };
    let e = event(
        kinds::JOIN,
        vec![a.clone(), with.clone()],
        at,
        Payload {
            set,
            unset,
            data: Some(Value::Map(
                [("how".to_string(), Value::Text(how_word.into()))].into(),
            )),
        },
        universe,
        publisher,
    )?;
    universe.append(e).map_err(|e| e.to_string())
}
