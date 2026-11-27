//! Mounts: one construct carried by another (compound ships, CM1; design
//! `handover/design/SSE4-compound-ships-design.md` section 4, accepted
//! 27 Nov 2026 with Alex's rulings in its section 14A).
//!
//! Anything that can exist on its own (a ship, a stage, a lander, a station
//! module, a land structure) is a construct; a **compound** is constructs
//! held together by mounts at a date. A mount is facts on the **carried**
//! construct, because it has at most one carrier at a time and a carrier may
//! hold many:
//!
//! | Fact | Meaning |
//! |---|---|
//! | `mount.on` | the carrier's address |
//! | `mount.place` | `front`, `behind`, `inside`, `hull` or `outside` |
//! | `mount.port` | the carrier's port id (`port.<id>.*` facts), optional |
//! | `mount.own_port` | the carried construct's own port, optional |
//! | `mount.bay` | for `inside`: the carrier's bay part, optional |
//! | `mount.station.along_m`, `.clock_deg`, `.stand_off_m` | where on the hull, or held clear outside |
//! | `mount.bond` | `fixed`, `latch` or `rest` |
//! | `mount.lines` | what may pass across: `propellant`, `power`, `air`, `people` |
//! | `mount.push_m_s` | the separation push a release gives, when the mount says |
//!
//! Two rules cover every case: **capture** (`construct.mount`) starts a
//! mount at a dated second and **release** (`construct.release`) ends it.
//! Neither moves mass, propellant, power or people: each construct keeps
//! what it held, so nothing can be counted twice or lost. What crosses a
//! mount is a separate dated transfer (CM3).
//!
//! The compound is never stored: it is the tree of mounts read at a date
//! ([`compound`]), its root the one construct in it that is not carried.
//! An RS4 dock (`docked.to`) is read as a latch on the hull, so saves and
//! pack logs already written keep working with no migration.
//!
//! Ports are facts `port.<id>.place`, `.size_m` and `.lines` on a
//! construct; a bay is a part with any of `part.<id>.bay_length_m`,
//! `bay_radius_m`, `bay_door_m`, `bay_holds_kg`. A carried construct's own
//! size, when known, is `construct.length_m` and `construct.radius_m`.
//! Where a size or a capacity is not known the mount is made and a note
//! says what was not checked (steer, never stop).

use crate::split::{DOCKED, MASS, PART};
use crate::templates::fields;
use sse_chronicle::UniverseTime;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, State};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

pub mod kinds {
    pub const MOUNT: &str = "construct.mount";
    pub const RELEASE: &str = "construct.release";
}

pub const ON: &str = "mount.on";
pub const PLACE: &str = "mount.place";
pub const PORT: &str = "mount.port";
pub const OWN_PORT: &str = "mount.own_port";
pub const BAY: &str = "mount.bay";
pub const ALONG: &str = "mount.station.along_m";
pub const CLOCK: &str = "mount.station.clock_deg";
pub const STAND_OFF: &str = "mount.station.stand_off_m";
pub const BOND: &str = "mount.bond";
pub const LINES: &str = "mount.lines";
pub const PUSH: &str = "mount.push_m_s";
/// Every mount fact, so a release unsets exactly what a mount set.
pub const ALL: [&str; 11] = [
    ON, PLACE, PORT, OWN_PORT, BAY, ALONG, CLOCK, STAND_OFF, BOND, LINES, PUSH,
];
pub const PORTS: &str = "port.";
pub const LENGTH: &str = "construct.length_m";
pub const RADIUS: &str = "construct.radius_m";

/// Where a carried construct sits relative to its carrier (Alex, 27 Nov:
/// "in front, behind, inside, outside, and the hull").
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Place {
    /// On the carrier's axis ahead of its nose (a CSM on its adapter).
    Front,
    /// On the axis at the tail (a first stage under a second).
    Behind,
    /// In a bay (a lander in its adapter, fighters in a hangar).
    Inside,
    /// Touching the outer surface at a station round and along it (strap-on boosters, a docked Soyuz).
    Hull,
    /// Held clear on an arm, tether or truss (Curiosity on the sky crane's bridles).
    Outside,
}

impl Place {
    pub fn word(self) -> &'static str {
        match self {
            Place::Front => "front",
            Place::Behind => "behind",
            Place::Inside => "inside",
            Place::Hull => "hull",
            Place::Outside => "outside",
        }
    }
    fn parse(s: &str) -> Option<Place> {
        Some(match s {
            "front" => Place::Front,
            "behind" => Place::Behind,
            "inside" => Place::Inside,
            "hull" => Place::Hull,
            "outside" => Place::Outside,
            _ => return None,
        })
    }
    /// On the carrier's axis: one construct per port (or one with no port) at each end.
    fn axial(self) -> bool {
        matches!(self, Place::Front | Place::Behind)
    }
    /// How it is held when the mount does not say: structure on the axis,
    /// resting in a bay, a latch on the hull or at the end of an arm.
    pub fn usual_bond(self) -> Bond {
        match self {
            Place::Front | Place::Behind => Bond::Fixed,
            Place::Inside => Bond::Rest,
            Place::Hull | Place::Outside => Bond::Latch,
        }
    }
}

/// How a mount holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bond {
    /// Structure, opened only by a release mechanism; takes thrust.
    Fixed,
    /// A docking or berthing latch.
    Latch,
    /// Sitting in a bay or on a deck with no latch.
    Rest,
}

impl Bond {
    pub fn word(self) -> &'static str {
        match self {
            Bond::Fixed => "fixed",
            Bond::Latch => "latch",
            Bond::Rest => "rest",
        }
    }
    fn parse(s: &str) -> Option<Bond> {
        Some(match s {
            "fixed" => Bond::Fixed,
            "latch" => Bond::Latch,
            "rest" => Bond::Rest,
            _ => return None,
        })
    }
}

/// What may pass across a mount while it holds.
pub const LINE_WORDS: [&str; 4] = ["propellant", "power", "air", "people"];

/// A mount as read from the carried construct's facts.
#[derive(Clone, Debug, PartialEq)]
pub struct Mount {
    pub on: String,
    pub place: Place,
    pub port: Option<String>,
    pub own_port: Option<String>,
    pub bay: Option<String>,
    pub along_m: Option<f64>,
    pub clock_deg: Option<f64>,
    pub stand_off_m: Option<f64>,
    pub bond: Bond,
    pub lines: Vec<String>,
    pub push_m_s: Option<f64>,
    /// Read from an RS4 dock (`docked.to`), not written as a mount.
    pub docked: bool,
}

impl Mount {
    pub fn to_json(&self) -> serde_json::Value {
        let mut v = serde_json::json!({
            "on": self.on, "place": self.place.word(), "bond": self.bond.word(),
            "lines": self.lines,
        });
        let put = |v: &mut serde_json::Value, k: &str, x: &Option<String>| {
            if let Some(x) = x {
                v[k] = x.clone().into();
            }
        };
        put(&mut v, "port", &self.port);
        put(&mut v, "own_port", &self.own_port);
        put(&mut v, "bay", &self.bay);
        for (k, x) in [
            ("along_m", self.along_m),
            ("clock_deg", self.clock_deg),
            ("stand_off_m", self.stand_off_m),
            ("push_m_s", self.push_m_s),
        ] {
            if let Some(x) = x {
                v[k] = x.into();
            }
        }
        if self.docked {
            v["docked"] = true.into();
        }
        v
    }
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

fn text(row: &BTreeMap<String, Value>, k: &str) -> Option<String> {
    match row.get(k) {
        Some(Value::Text(t)) => Some(t.clone()),
        _ => None,
    }
}

fn words(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(xs)) => xs
            .iter()
            .filter_map(|x| match x {
                Value::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

/// The mount a construct's facts describe, if it is carried. A dock (RS4
/// `docked.to`) reads as a latch on the hull.
pub fn mount_of(row: &BTreeMap<String, Value>) -> Option<Mount> {
    if let Some(on) = text(row, ON) {
        let place = text(row, PLACE)
            .and_then(|p| Place::parse(&p))
            .unwrap_or(Place::Hull);
        return Some(Mount {
            on,
            place,
            port: text(row, PORT),
            own_port: text(row, OWN_PORT),
            bay: text(row, BAY),
            along_m: row.get(ALONG).and_then(num),
            clock_deg: row.get(CLOCK).and_then(num),
            stand_off_m: row.get(STAND_OFF).and_then(num),
            bond: text(row, BOND)
                .and_then(|b| Bond::parse(&b))
                .unwrap_or_else(|| place.usual_bond()),
            lines: words(row.get(LINES)),
            push_m_s: row.get(PUSH).and_then(num),
            docked: false,
        });
    }
    text(row, DOCKED).map(|on| Mount {
        on,
        place: Place::Hull,
        port: None,
        own_port: None,
        bay: None,
        along_m: None,
        clock_deg: None,
        stand_off_m: None,
        bond: Bond::Latch,
        lines: vec![],
        push_m_s: None,
        docked: true,
    })
}

fn retired(row: &BTreeMap<String, Value>) -> bool {
    row.get(fields::RETIRED) == Some(&Value::Bool(true))
}

/// One member of a compound: where it is in the tree and how it is held.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub address: Address,
    /// 0 for the root.
    pub depth: usize,
    pub mount: Option<Mount>,
    pub mass_kg: Option<f64>,
}

/// A compound at a date: its root and every construct mounted under it,
/// depth first in address order. Read, never stored.
#[derive(Clone, Debug, PartialEq)]
pub struct Compound {
    pub root: Address,
    pub members: Vec<Member>,
}

impl Compound {
    /// The sum of every member's `construct.mass_kg`, and the members whose mass is not known.
    pub fn mass_kg(&self) -> (f64, Vec<&Address>) {
        let mut sum = 0.0;
        let mut unknown = vec![];
        for m in &self.members {
            match m.mass_kg {
                Some(x) => sum += x,
                None => unknown.push(&m.address),
            }
        }
        (sum, unknown)
    }

    pub fn to_json(&self) -> serde_json::Value {
        let (mass, unknown) = self.mass_kg();
        serde_json::json!({
            "root": self.root.to_string(),
            "members": self.members.iter().map(|m| {
                let mut v = serde_json::json!({"address": m.address.to_string(), "depth": m.depth});
                if let Some(mt) = &m.mount { v["mount"] = mt.to_json(); }
                if let Some(x) = m.mass_kg { v["mass_kg"] = x.into(); }
                v
            }).collect::<Vec<_>>(),
            "mass_kg": mass,
            "mass_unknown": unknown.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
        })
    }
}

/// Constructs carried directly by `a` in `st`, in address order.
pub fn carried_by(st: &State, a: &Address) -> Vec<(Address, Mount)> {
    let here = a.to_string();
    st.facts
        .iter()
        .filter(|(_, row)| !retired(row))
        .filter_map(|(x, row)| {
            mount_of(row)
                .filter(|m| m.on == here)
                .map(|m| (x.clone(), m))
        })
        .collect()
}

/// The chain of carriers above `a` (nearest first). Stops at a loop or an
/// address that is not in `st`, so a broken record never hangs a read.
pub fn carriers_of(st: &State, a: &Address) -> Vec<Address> {
    let mut out: Vec<Address> = vec![];
    let mut at = a.clone();
    while let Some(m) = st.facts.get(&at).and_then(mount_of) {
        let Ok(up) = m.on.parse::<Address>() else {
            break;
        };
        if up == *a || out.contains(&up) || !st.facts.contains_key(&up) {
            break;
        }
        out.push(up.clone());
        at = up;
    }
    out
}

/// The compound `a` belongs to at the state's date.
pub fn compound(st: &State, a: &Address) -> Compound {
    let root = carriers_of(st, a)
        .last()
        .cloned()
        .unwrap_or_else(|| a.clone());
    let mut members = vec![];
    let mut stack = vec![(root.clone(), 0usize)];
    while let Some((x, depth)) = stack.pop() {
        if members.iter().any(|m: &Member| m.address == x) {
            continue;
        }
        let row = st.facts.get(&x);
        members.push(Member {
            address: x.clone(),
            depth,
            mount: if depth == 0 {
                None
            } else {
                row.and_then(mount_of)
            },
            mass_kg: row.and_then(|r| r.get(MASS)).and_then(num),
        });
        let kids = carried_by(st, &x);
        for (c, _) in kids.into_iter().rev() {
            stack.push((c, depth + 1));
        }
    }
    Compound { root, members }
}

/// What a capture asks for.
pub struct MountAsk<'a> {
    pub carried: &'a Address,
    pub carrier: &'a Address,
    pub place: Option<Place>,
    pub port: Option<&'a str>,
    pub own_port: Option<&'a str>,
    pub bay: Option<&'a str>,
    pub along_m: Option<f64>,
    pub clock_deg: Option<f64>,
    pub stand_off_m: Option<f64>,
    pub bond: Option<Bond>,
    pub lines: &'a [String],
    pub push_m_s: Option<f64>,
    pub at: UniverseTime,
}

/// What a mount or a release did.
pub struct Done {
    pub event: ContentHash,
    pub mount: Mount,
    /// The compound root of the carrier after the event.
    pub root: Address,
    /// What could not be checked, said plainly.
    pub notes: Vec<String>,
}

fn live<'s>(st: &'s State, a: &Address) -> Result<&'s BTreeMap<String, Value>, String> {
    let row = st
        .facts
        .get(a)
        .ok_or_else(|| format!("{a} is not in the universe at that time"))?;
    if retired(row) {
        return Err(format!("{a} is retired"));
    }
    Ok(row)
}

fn in_root(root: &Address, a: &Address) -> Result<(), String> {
    if !root.contains(a) || a == root {
        return Err(format!("{a} is not a construct in universe {root}"));
    }
    Ok(())
}

/// The facts of port `id` on a row.
fn port(row: &BTreeMap<String, Value>, id: &str) -> Option<BTreeMap<String, Value>> {
    let p = format!("{PORTS}{id}.");
    let fs: BTreeMap<String, Value> = row
        .iter()
        .filter_map(|(k, v)| k.strip_prefix(&p).map(|f| (f.to_string(), v.clone())))
        .collect();
    (!fs.is_empty()).then_some(fs)
}

fn port_ids(row: &BTreeMap<String, Value>) -> Vec<String> {
    let mut ids: Vec<String> = row
        .keys()
        .filter_map(|k| k.strip_prefix(PORTS))
        .filter_map(|r| r.split('.').next())
        .map(str::to_string)
        .collect();
    ids.dedup();
    ids
}

/// A bay part's fields (`bay_length_m` and so on, without the prefix).
fn bay(row: &BTreeMap<String, Value>, id: &str) -> Option<BTreeMap<String, f64>> {
    let p = format!("{PART}{id}.bay_");
    let fs: BTreeMap<String, f64> = row
        .iter()
        .filter_map(|(k, v)| {
            k.strip_prefix(&p)
                .and_then(|f| Some((f.to_string(), num(v)?)))
        })
        .collect();
    (!fs.is_empty()).then_some(fs)
}

fn bay_ids(row: &BTreeMap<String, Value>) -> Vec<String> {
    let mut ids: Vec<String> = row
        .keys()
        .filter_map(|k| k.strip_prefix(PART))
        .filter(|r| r.contains(".bay_"))
        .filter_map(|r| r.split('.').next())
        .map(str::to_string)
        .collect();
    ids.dedup();
    ids
}

fn finite(name: &str, x: Option<f64>, min: f64) -> Result<(), String> {
    match x {
        Some(v) if !(v.is_finite() && v >= min) => {
            Err(format!("{name}: a number of {min} or more"))
        }
        _ => Ok(()),
    }
}

/// The later mount or release that names `a`, if any (mounts are written in date order).
fn later_change(universe: &Chronicle, a: &Address, at: UniverseTime) -> Option<UniverseTime> {
    universe
        .events()
        .iter()
        .filter(|e| e.body.kind == kinds::MOUNT || e.body.kind == kinds::RELEASE)
        .filter(|e| e.body.subject.contains(a) && e.body.activated_at > at)
        .map(|e| e.body.activated_at)
        .min()
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

/// Capture: `ask.carried` is mounted on `ask.carrier` at `ask.at`.
/// Nothing moves; the carried construct gains its `mount.*` facts.
pub fn mount(
    universe: &mut Chronicle,
    root: &Address,
    ask: &MountAsk,
    publisher: &Publisher,
) -> Result<Done, String> {
    let (c, h) = (ask.carried, ask.carrier);
    let place = ask
        .place
        .ok_or_else(|| "place: one of front, behind, inside, hull, outside".to_string())?;
    in_root(root, c)?;
    in_root(root, h)?;
    if c == h {
        return Err(format!("{c} cannot carry itself"));
    }
    let st = universe.state_at(ask.at);
    let row = live(&st, c)?;
    let host = live(&st, h)?;
    if let Some(m) = mount_of(row) {
        return Err(format!(
            "{c} is already {} {}; release it first",
            if m.docked { "docked to" } else { "mounted on" },
            m.on
        ));
    }
    if let Some(t) = later_change(universe, c, ask.at) {
        return Err(format!(
            "{c} has a mount or release later, at {} s; write mounts in date order",
            t.s
        ));
    }
    let above = carriers_of(&st, h);
    if above.contains(c) {
        return Err(format!(
            "{c} already carries {h}; a construct cannot carry what carries it"
        ));
    }
    finite("along_m", ask.along_m, f64::MIN)?;
    finite("clock_deg", ask.clock_deg, f64::MIN)?;
    finite("stand_off_m", ask.stand_off_m, 0.0)?;
    finite("push_m_s", ask.push_m_s, 0.0)?;
    if !matches!(place, Place::Hull | Place::Outside)
        && (ask.along_m.is_some() || ask.clock_deg.is_some())
    {
        return Err(format!(
            "along_m and clock_deg place a hull or outside mount; {} sits on the axis or in a bay",
            place.word()
        ));
    }
    if place != Place::Outside && ask.stand_off_m.is_some() {
        return Err("stand_off_m is for an outside mount, held clear of the hull".into());
    }
    if place != Place::Inside && ask.bay.is_some() {
        return Err("a bay holds an inside mount".into());
    }
    let mut lines: Vec<String> = vec![];
    for l in ask.lines {
        if !LINE_WORDS.contains(&l.as_str()) {
            return Err(format!(
                "lines: {l:?} is not one of {}",
                LINE_WORDS.join(", ")
            ));
        }
        if !lines.contains(l) {
            lines.push(l.clone());
        }
    }
    lines.sort();
    let mut notes = vec![];
    let others = carried_by(&st, h);
    // Ports: the carrier's must exist, be free, sit at this place and carry these lines.
    if let Some(p) = ask.port {
        let Some(pf) = port(host, p) else {
            let have = port_ids(host);
            return Err(format!(
                "{h} has no port {p:?}; ports: {}",
                if have.is_empty() {
                    "none".into()
                } else {
                    have.join(", ")
                }
            ));
        };
        if let Some(Value::Text(pp)) = pf.get("place") {
            if pp != place.word() {
                return Err(format!(
                    "{h}'s port {p:?} is at its {pp}, not its {}",
                    place.word()
                ));
            }
        }
        if let Some((x, _)) = others.iter().find(|(_, m)| m.port.as_deref() == Some(p)) {
            return Err(format!("{h}'s port {p:?} already holds {x}"));
        }
        let allowed = words(pf.get("lines"));
        if pf.contains_key("lines") {
            if let Some(l) = lines.iter().find(|l| !allowed.contains(l)) {
                return Err(format!("{h}'s port {p:?} passes no {l}"));
            }
        }
        if let Some(op) = ask.own_port {
            let Some(of) = port(row, op) else {
                return Err(format!("{c} has no port {op:?}"));
            };
            if let (Some(a), Some(b)) = (
                pf.get("size_m").and_then(num),
                of.get("size_m").and_then(num),
            ) {
                if (a - b).abs() > 1e-9 * a.abs().max(b.abs()) {
                    return Err(format!(
                        "the ports do not fit: {h}'s {p:?} is {a} m, {c}'s {op:?} is {b} m"
                    ));
                }
            }
        }
    } else if let Some(op) = ask.own_port {
        if port(row, op).is_none() {
            return Err(format!("{c} has no port {op:?}"));
        }
    }
    // On the axis there is one construct at each end, unless ports say where.
    if place.axial() && ask.port.is_none() {
        if let Some((x, _)) = others
            .iter()
            .find(|(_, m)| m.place == place && m.port.is_none())
        {
            return Err(format!(
                "{h}'s {} already holds {x}; name a port to hold two",
                place.word()
            ));
        }
    }
    // A bay: named, or the carrier's only one; the carried must fit and the bay hold its weight.
    let mut bay_id = ask.bay.map(str::to_string);
    if place == Place::Inside {
        let bays = bay_ids(host);
        if bay_id.is_none() && bays.len() == 1 {
            bay_id = Some(bays[0].clone());
        }
        match &bay_id {
            None if bays.is_empty() => notes.push(format!(
                "{h} has no bay part, so whether {c} fits inside is not checked"
            )),
            None => {
                return Err(format!(
                    "{h} has bays {}; name which holds {c}",
                    bays.join(", ")
                ))
            }
            Some(b) => {
                let Some(bf) = bay(host, b) else {
                    return Err(format!(
                        "{h} has no bay {b:?}; bays: {}",
                        if bays.is_empty() {
                            "none".into()
                        } else {
                            bays.join(", ")
                        }
                    ));
                };
                let len = row.get(LENGTH).and_then(num);
                let rad = row.get(RADIUS).and_then(num);
                for (what, have, room) in [
                    ("long", len, bf.get("length_m")),
                    ("wide (radius)", rad, bf.get("radius_m")),
                ] {
                    match (have, room) {
                        (Some(x), Some(r)) if x > *r => {
                            return Err(format!("{c} is {x} m {what}; bay {b:?} of {h} has {r} m"))
                        }
                        (None, Some(_)) => notes.push(format!(
                            "{c} has no construct.{} so its fit in bay {b:?} is not checked",
                            if what == "long" {
                                "length_m"
                            } else {
                                "radius_m"
                            }
                        )),
                        _ => {}
                    }
                }
                if let (Some(r), Some(door)) = (rad, bf.get("door_m")) {
                    if 2.0 * r > *door {
                        return Err(format!(
                            "{c} is {} m across; the door of bay {b:?} is {door} m",
                            2.0 * r
                        ));
                    }
                }
                if let Some(cap) = bf.get("holds_kg") {
                    // Everything already resting in this bay, each with what it carries.
                    let mut load = 0.0;
                    let mut unknown = vec![];
                    let mut sum = |x: &Address| {
                        let cx = compound(&st, x);
                        let (m, u) = cx.mass_kg();
                        load += m;
                        unknown.extend(u.into_iter().map(|a| a.to_string()));
                    };
                    for (x, m) in &others {
                        if m.place == Place::Inside && m.bay.as_deref() == Some(b.as_str()) {
                            sum(x);
                        }
                    }
                    sum(c);
                    if load > *cap * (1.0 + 1e-9) {
                        return Err(format!(
                            "bay {b:?} of {h} holds {cap} kg; with {c} it would hold {load} kg"
                        ));
                    }
                    if !unknown.is_empty() {
                        notes.push(format!(
                            "no mass for {}, so bay {b:?}'s load is counted without it",
                            unknown.join(", ")
                        ));
                    }
                }
            }
        }
    }
    let bond = ask.bond.unwrap_or_else(|| place.usual_bond());
    let m = Mount {
        on: h.to_string(),
        place,
        port: ask.port.map(str::to_string),
        own_port: ask.own_port.map(str::to_string),
        bay: bay_id,
        along_m: ask.along_m,
        clock_deg: ask.clock_deg,
        stand_off_m: ask.stand_off_m,
        bond,
        lines,
        push_m_s: ask.push_m_s,
        docked: false,
    };
    let mut set_row: BTreeMap<String, Value> = BTreeMap::new();
    set_row.insert(ON.into(), Value::Text(m.on.clone()));
    set_row.insert(PLACE.into(), Value::Text(place.word().into()));
    set_row.insert(BOND.into(), Value::Text(bond.word().into()));
    set_row.insert(
        LINES.into(),
        Value::Array(m.lines.iter().map(|l| Value::Text(l.clone())).collect()),
    );
    for (k, x) in [(PORT, &m.port), (OWN_PORT, &m.own_port), (BAY, &m.bay)] {
        if let Some(x) = x {
            set_row.insert(k.into(), Value::Text(x.clone()));
        }
    }
    for (k, x) in [
        (ALONG, m.along_m),
        (CLOCK, m.clock_deg),
        (STAND_OFF, m.stand_off_m),
        (PUSH, m.push_m_s),
    ] {
        if let Some(x) = x {
            set_row.insert(k.into(), Value::Float(x));
        }
    }
    let mut data = BTreeMap::new();
    data.insert("place".to_string(), Value::Text(place.word().into()));
    data.insert("carrier".to_string(), Value::Text(m.on.clone()));
    let e = event(
        kinds::MOUNT,
        vec![h.clone(), c.clone()],
        ask.at,
        Payload {
            set: [(c.clone(), set_row)].into(),
            unset: BTreeMap::new(),
            data: Some(Value::Map(data)),
        },
        universe,
        publisher,
    )?;
    let id = universe.append(e).map_err(|e| e.to_string())?;
    let after = universe.state_at(ask.at);
    Ok(Done {
        event: id,
        root: compound(&after, h).root,
        mount: m,
        notes,
    })
}

/// Release: `carried` leaves its carrier at `at` (it must be `carrier`, when
/// named). Nothing moves; the carried construct becomes the root of what it
/// carries. A push given here wins over the mount's own.
pub fn release(
    universe: &mut Chronicle,
    root: &Address,
    carried: &Address,
    carrier: Option<&Address>,
    push_m_s: Option<f64>,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<Done, String> {
    in_root(root, carried)?;
    finite("push_m_s", push_m_s, 0.0)?;
    let st = universe.state_at(at);
    let row = live(&st, carried)?;
    let Some(mut m) = mount_of(row) else {
        return Err(format!("{carried} is not carried by anything at that time"));
    };
    if let Some(h) = carrier {
        if m.on != h.to_string() {
            return Err(format!("{carried} is carried by {}, not {h}", m.on));
        }
    }
    if let Some(t) = later_change(universe, carried, at) {
        return Err(format!(
            "{carried} has a mount or release later, at {} s; write releases in date order",
            t.s
        ));
    }
    let h: Address =
        m.on.parse()
            .map_err(|_| format!("{carried} is carried by {:?}, not an address", m.on))?;
    let gone: Vec<String> = if m.docked {
        vec![DOCKED.to_string()]
    } else {
        ALL.iter()
            .filter(|k| row.contains_key(**k))
            .map(|k| k.to_string())
            .collect()
    };
    if push_m_s.is_some() {
        m.push_m_s = push_m_s;
    }
    let mut data = BTreeMap::new();
    data.insert("place".to_string(), Value::Text(m.place.word().into()));
    data.insert("carrier".to_string(), Value::Text(m.on.clone()));
    if let Some(p) = m.push_m_s {
        data.insert("push_m_s".to_string(), Value::Float(p));
    }
    let e = event(
        kinds::RELEASE,
        vec![h.clone(), carried.clone()],
        at,
        Payload {
            set: BTreeMap::new(),
            unset: [(carried.clone(), gone)].into(),
            data: Some(Value::Map(data)),
        },
        universe,
        publisher,
    )?;
    let id = universe.append(e).map_err(|e| e.to_string())?;
    let after = universe.state_at(at);
    let mut notes = vec![];
    if m.push_m_s.is_none() {
        notes.push("no separation push given; the flight log uses the pack's".into());
    }
    Ok(Done {
        event: id,
        root: compound(&after, &h).root,
        mount: m,
        notes,
    })
}
