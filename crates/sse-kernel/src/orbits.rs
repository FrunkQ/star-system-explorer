//! Orbits at any time, the campaign clock and pack calendars (asked for by
//! the chart room for human-scale time).
//!
//! `orbits_at`: each body with an `orbit.semi_major_axis` and a host mass
//! gets its position local to its host (f64 metres, the host's reference
//! plane as z = 0) at each asked time, from Kepler's equation solved by a
//! fixed number of Newton steps (deterministic on every device). A body
//! with no dated phase (`orbit.mean_anomaly_at_epoch`) takes one from the
//! universe's keyed draw on its address, labelled `resolved`: the same on
//! every device and every call, and pinnable like any fact. A body missing
//! anything else comes back unplaced with the missing fields named, never
//! with a guessed position.
//!
//! The epoch of a dated phase is `orbit.epoch_s` (universe seconds), else
//! an SSE 3 `orbit.epoch_v3_ms` read as milliseconds of universe time, else
//! the universe origin.

use crate::err;
use crate::protocol::Response;
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use std::collections::BTreeMap;

const G: f64 = sse_core::sheet::G;
/// Times one call may ask for.
pub const MAX_TIMES: usize = 64;
/// Event kind and root field of the campaign clock.
pub const CAMPAIGN_NOW: &str = "time.campaign_now";
pub const CAMPAIGN_NOW_FIELD: &str = "campaign.now_s";

type Facts = BTreeMap<Address, BTreeMap<String, Value>>;
type Points = BTreeMap<Address, [f64; 3]>;

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// One body at one time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct At {
    pub at_s: i64,
    /// Position relative to the host, metres.
    pub r_m: [f64; 3],
    /// Longitude in the host's reference plane, degrees in [0, 360).
    pub phase_deg: f64,
    /// Position from the barycentre of the whole hierarchy it belongs to, metres.
    pub sys_m: [f64; 3],
    /// The centre its orbit goes round (the barycentre of its host and
    /// everything further in), from the host's centre, metres.
    pub centre_m: [f64; 3],
    /// For a body with a motion model (RW17): `measured` inside the model's
    /// span, `plausible` outside it (a plain orbit fitted at the span's edge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Placed {
    pub address: String,
    pub placed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_s: Option<f64>,
    /// `authored` or `derived` (a dated phase in the facts) or `resolved` (the keyed draw).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_source: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    /// The motion model it follows (RW17), with its span, or why the one it
    /// names cannot be used (it then follows its plain orbit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion: Option<MotionNote>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub at: Vec<At>,
}

/// What `Placed` says about a body's motion model.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MotionNote {
    pub model: String,
    /// The span it holds over, universe seconds (absent when unusable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_s: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_s: Option<i64>,
    /// Why the model is not used, when it is not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unused: Option<String>,
}

/// Solve E - e sin E = M for E (radians), 16 fixed Newton steps from a safe start.
pub fn eccentric_anomaly(mean: f64, e: f64) -> f64 {
    let mean = mean - m::TAU * m::floor(mean / m::TAU);
    let mut ea = if e < 0.8 { mean } else { m::PI };
    for _ in 0..16 {
        ea -= (ea - e * m::sin(ea) - mean) / (1.0 - e * m::cos(ea));
    }
    ea
}

/// A body's own orbit, read from its facts (pass 1 of `orbits_at`).
struct Orbit {
    host: Address,
    a_m: f64,
    e: f64,
    m0: f64,
    /// The date of `m0`, on the whole-second axis (TP1: never a double).
    epoch: UniverseTime,
    /// R3(-node) R1(-inc) R3(-peri), rows x, y, z by perifocal (x, y).
    rot: [[f64; 2]; 3],
    mass: f64,
    source: &'static str,
    measured: Option<Measured>,
}

/// A plain orbit: elements and its own mean motion (the hand-over outside a
/// motion model's span).
struct Plain {
    a_m: f64,
    e: f64,
    m0: f64,
    epoch: i64,
    n: f64,
    rot: [[f64; 2]; 3],
}

impl Plain {
    fn from_elements(el: [f64; 6], epoch: i64, mu: f64) -> Plain {
        let [a_m, e, inc, node, peri, m0] = el;
        Plain {
            a_m,
            e,
            m0,
            epoch,
            n: m::sqrt(mu / (a_m * a_m * a_m)),
            rot: rotation(inc, node, peri),
        }
    }
    fn rel(&self, t: i64) -> [f64; 3] {
        let mean = self.m0 + self.n * (t - self.epoch) as f64;
        perifocal(self.a_m, self.e, mean, &self.rot)
    }
}

/// A body that follows a motion model (RW17).
struct Measured {
    series: &'static sse_time::series::Series,
    /// Universe second of J2000.0 in this system.
    anchor: i64,
    /// mu for the hand-over orbit.
    mu: f64,
    /// Hand-overs fitted at the span's start and end, worked out only when a
    /// time outside the span is asked for.
    before: std::cell::OnceCell<Option<Plain>>,
    after: std::cell::OnceCell<Option<Plain>>,
}

impl Measured {
    fn new(series: &'static sse_time::series::Series, anchor: i64, mu: f64) -> Measured {
        Measured {
            series,
            anchor,
            mu,
            before: std::cell::OnceCell::new(),
            after: std::cell::OnceCell::new(),
        }
    }
    fn fit(&self, jd: f64) -> Option<Plain> {
        let (r, v) = self.series.state_m(jd, 0.5);
        elements_from_state(r, v, self.mu)
            .map(|el| Plain::from_elements(el, crate::motion::at_s(jd, self.anchor), self.mu))
    }
    /// `measured` inside the span, `plausible` outside it.
    fn label(&self, t: i64) -> &'static str {
        if self.series.covers(crate::motion::jd(t, self.anchor)) {
            "measured"
        } else {
            "plausible"
        }
    }
    /// Where it is from its host's centre at `t`, and whether that is measured.
    fn at(&self, t: i64) -> Option<([f64; 3], &'static str)> {
        let jd = crate::motion::jd(t, self.anchor);
        if self.series.covers(jd) {
            return Some((self.series.position_m(jd), "measured"));
        }
        let p = if jd < self.series.from_jd {
            self.before.get_or_init(|| self.fit(self.series.from_jd))
        } else {
            self.after.get_or_init(|| self.fit(self.series.to_jd))
        };
        p.as_ref().map(|p| (p.rel(t), "plausible"))
    }
}

/// R3(-node) R1(-inc) R3(-peri), rows x, y, z by perifocal (x, y).
fn rotation(inc: f64, node: f64, peri: f64) -> [[f64; 2]; 3] {
    let (co, so) = (m::cos(node), m::sin(node));
    let (cw, sw) = (m::cos(peri), m::sin(peri));
    let (ci, si) = (m::cos(inc), m::sin(inc));
    [
        [co * cw - so * sw * ci, -co * sw - so * cw * ci],
        [so * cw + co * sw * ci, -so * sw + co * cw * ci],
        [sw * si, cw * si],
    ]
}

fn perifocal(a_m: f64, e: f64, mean: f64, r: &[[f64; 2]; 3]) -> [f64; 3] {
    let ea = eccentric_anomaly(mean, e);
    let b = a_m * m::sqrt(1.0 - e * e);
    let (xp, yp) = (a_m * (m::cos(ea) - e), b * m::sin(ea));
    [
        r[0][0] * xp + r[0][1] * yp,
        r[1][0] * xp + r[1][1] * yp,
        r[2][0] * xp + r[2][1] * yp,
    ]
}

impl Orbit {
    /// The relative vector at `t` for mean motion `n`.
    fn rel(&self, n: f64, t: i64) -> [f64; 3] {
        let mean = self.m0 + UniverseTime::seconds(t).angle_since(self.epoch, n);
        perifocal(self.a_m, self.e, mean, &self.rot)
    }
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// The barycentre hierarchy (Jacobi): each body orbits the centre of mass
/// of its host and everything round that host further in (with their own
/// moons), with mu = G(inner mass + its own group's mass). No threshold:
/// a small partner's pull just rounds towards nothing.
struct Tree<'a> {
    orbits: &'a BTreeMap<Address, Orbit>,
    /// Host -> children, innermost first.
    kids: BTreeMap<Address, Vec<Address>>,
    /// Own mass of every body in the tree (hosts included).
    mass: BTreeMap<Address, f64>,
    /// Mass of a body and everything round it.
    group: BTreeMap<Address, f64>,
    /// Mean motion of each child.
    n: BTreeMap<Address, f64>,
}

impl<'a> Tree<'a> {
    fn new(orbits: &'a BTreeMap<Address, Orbit>, host_mass: &BTreeMap<Address, f64>) -> Self {
        let mut kids: BTreeMap<Address, Vec<Address>> = BTreeMap::new();
        for (a, o) in orbits {
            kids.entry(o.host.clone()).or_default().push(a.clone());
        }
        for v in kids.values_mut() {
            v.sort_by(|x, y| orbits[x].a_m.total_cmp(&orbits[y].a_m).then(x.cmp(y)));
        }
        let mut mass: BTreeMap<Address, f64> = host_mass.clone();
        for (a, o) in orbits {
            mass.insert(a.clone(), o.mass);
        }
        let mut t = Tree {
            orbits,
            kids,
            mass,
            group: BTreeMap::new(),
            n: BTreeMap::new(),
        };
        for r in t.roots() {
            t.fill(&r, 0);
        }
        t
    }

    /// Hosts that orbit nothing placed: the tops of the hierarchy.
    fn roots(&self) -> Vec<Address> {
        self.kids
            .keys()
            .filter(|h| !self.orbits.contains_key(*h))
            .cloned()
            .collect()
    }

    fn fill(&mut self, h: &Address, depth: usize) -> f64 {
        let own = self.mass.get(h).copied().unwrap_or(0.0);
        if depth > 64 {
            return own;
        }
        let mut inner = own;
        for c in self.kids.get(h).cloned().unwrap_or_default() {
            let g = self.fill(&c, depth + 1);
            let a = self.orbits[&c].a_m;
            self.n
                .insert(c.clone(), m::sqrt(G * (inner + g) / (a * a * a)));
            inner += g;
        }
        self.group.insert(h.clone(), inner);
        inner
    }

    /// Positions and orbit centres of everything in the tree at each time,
    /// from each top's barycentre.
    fn frames(&self, times: &[i64]) -> Vec<(Points, Points)> {
        times
            .iter()
            .map(|&t| {
                let mut pos = BTreeMap::new();
                let mut cen = BTreeMap::new();
                for r in self.roots() {
                    self.place(&r, t, &mut pos, &mut cen, 0);
                }
                (pos, cen)
            })
            .collect()
    }

    /// `h` and everything round it, at any depth.
    fn subtree(&self, h: &Address, out: &mut Vec<Address>, depth: usize) {
        out.push(h.clone());
        if depth > 64 {
            return;
        }
        for c in self.kids.get(h).into_iter().flatten() {
            self.subtree(c, out, depth + 1);
        }
    }

    /// Positions of `h` and everything round it relative to their common
    /// barycentre at `t`, and each child's orbit centre (the barycentre of
    /// what lies inside it) in the same frame.
    fn place(
        &self,
        h: &Address,
        t: i64,
        pos: &mut BTreeMap<Address, [f64; 3]>,
        centre: &mut BTreeMap<Address, [f64; 3]>,
        depth: usize,
    ) {
        let mut local: BTreeMap<Address, [f64; 3]> = BTreeMap::new();
        let mut cen: BTreeMap<Address, [f64; 3]> = BTreeMap::new();
        local.insert(h.clone(), [0.0; 3]);
        let mut b = [0.0; 3];
        let mut mc = self.mass.get(h).copied().unwrap_or(0.0);
        if depth <= 64 {
            for c in self.kids.get(h).cloned().unwrap_or_default() {
                let (Some(n), Some(g)) = (self.n.get(&c), self.group.get(&c)) else {
                    continue;
                };
                // A measured body is placed from its host's centre directly
                // (its model already holds the inner bodies' pull); a plain
                // orbit goes round the barycentre of what lies inside it.
                let x = match self.orbits[&c].measured.as_ref().and_then(|ms| ms.at(t)) {
                    Some((v, _)) => v,
                    None => add(b, self.orbits[&c].rel(*n, t)),
                };
                cen.insert(c.clone(), b);
                let mut sp = BTreeMap::new();
                let mut sc = BTreeMap::new();
                self.place(&c, t, &mut sp, &mut sc, depth + 1);
                for (k, v) in sp {
                    local.insert(k, add(x, v));
                }
                for (k, v) in sc {
                    cen.insert(k, add(x, v));
                }
                let tot = mc + g;
                if tot > 0.0 {
                    b = scale(add(scale(b, mc), scale(x, *g)), 1.0 / tot);
                }
                mc = tot;
            }
        }
        for (k, v) in local {
            pos.insert(k, sub(v, b));
        }
        for (k, v) in cen {
            centre.insert(k, sub(v, b));
        }
    }
}

/// Every orbiting body at or under `system` in `facts`, at each time.
///
/// Barycentre always: positions come from the Jacobi hierarchy (`Tree`), so
/// a host moves by its children's pull (Pluto round its point outside
/// itself with Charon, a binary's stars round each other, a planet outside
/// a companion round the pair). `r_m` stays relative to the host's centre;
/// `sys_m` is from the barycentre of the whole hierarchy and `centre_m` the
/// orbit's own centre, from the host's centre.
pub fn orbits_at(
    facts: &Facts,
    system: &Address,
    times: &[i64],
    seed: Option<&Seed>,
) -> Vec<Placed> {
    let (mut out, orbits, host_mass, notes) = read_orbits(facts, system, seed);
    let tree = Tree::new(&orbits, &host_mass);
    let frames = tree.frames(times);
    for (a, o) in &orbits {
        let Some(n) = tree.n.get(a) else {
            // Never reached from a top: its hosts go round in a circle.
            out.push(Placed {
                address: a.to_string(),
                placed: false,
                parent: Some(o.host.to_string()),
                period_s: None,
                phase_source: None,
                missing: vec!["orbit.host chain that ends at a body orbiting nothing".into()],
                motion: None,
                at: vec![],
            });
            continue;
        };
        let at = times
            .iter()
            .zip(&frames)
            .map(|(&t, (pos, cen))| {
                let me = pos[a];
                let hp = pos[&o.host];
                let [x, y, z] = sub(me, hp);
                let mut ph = m::atan2(y, x) * (180.0 / m::PI);
                if ph < 0.0 {
                    ph += 360.0;
                }
                if ph >= 360.0 {
                    ph -= 360.0;
                }
                At {
                    at_s: t,
                    r_m: [x, y, z],
                    phase_deg: ph,
                    sys_m: me,
                    centre_m: sub(cen[a], hp),
                    motion: o.measured.as_ref().map(|ms| ms.label(t).to_string()),
                }
            })
            .collect();
        out.push(Placed {
            address: a.to_string(),
            placed: true,
            parent: Some(o.host.to_string()),
            period_s: Some(m::TAU / n),
            phase_source: Some(
                if o.measured.is_some() {
                    "measured"
                } else {
                    o.source
                }
                .into(),
            ),
            missing: vec![],
            motion: notes.get(a).cloned(),
            at,
        });
    }
    out.sort_by(|x, y| x.address.cmp(&y.address));
    out
}

type Read = (
    Vec<Placed>,
    BTreeMap<Address, Orbit>,
    BTreeMap<Address, f64>,
    BTreeMap<Address, MotionNote>,
);

/// The universe second of J2000.0 for `a`'s system: `time.j2000_s` on the
/// top of its host chain (the root star).
fn anchor_of(facts: &Facts, a: &Address) -> Option<i64> {
    let mut at = a.clone();
    for _ in 0..64 {
        if let Some(v) = facts
            .get(&at)
            .and_then(|r| r.get(crate::motion::ANCHOR_FIELD))
        {
            return match v {
                Value::Int(i) => i64::try_from(*i).ok(),
                v => num(v).map(|x| m::round(x) as i64),
            };
        }
        let row = facts.get(&at);
        let up = row
            .and_then(|r| r.get("orbit.host").or_else(|| r.get("parent")))
            .and_then(|v| match v {
                Value::Text(t) => t.parse().ok(),
                _ => None,
            })
            .or_else(|| at.parent())?;
        if up == at {
            return None;
        }
        at = up;
    }
    None
}

/// The motion model a body names, ready to place it, or why not.
fn motion_of(
    facts: &Facts,
    a: &Address,
    row: &BTreeMap<String, Value>,
    mu: f64,
) -> Option<(Option<Measured>, MotionNote)> {
    let id = match row.get(crate::motion::MOTION_FIELD)? {
        Value::Text(t) => t.clone(),
        _ => {
            return Some((
                None,
                MotionNote {
                    model: String::new(),
                    from_s: None,
                    to_s: None,
                    unused: Some("orbit.motion is not a model id".into()),
                },
            ))
        }
    };
    let unused = |why: String| {
        Some((
            None,
            MotionNote {
                model: id.clone(),
                from_s: None,
                to_s: None,
                unused: Some(why),
            },
        ))
    };
    let Some(series) = crate::motion::model(&id) else {
        return unused(format!("no motion model {id} in the kernel"));
    };
    let Some(anchor) = anchor_of(facts, a) else {
        return unused(format!(
            "no {} on its system's star to read the model's dates by",
            crate::motion::ANCHOR_FIELD
        ));
    };
    let note = MotionNote {
        model: id,
        from_s: Some(crate::motion::at_s(series.from_jd, anchor)),
        to_s: Some(crate::motion::at_s(series.to_jd, anchor)),
        unused: None,
    };
    Some((Some(Measured::new(series, anchor, mu)), note))
}

/// The date of a body's dated phase: `orbit.epoch_s` (universe seconds,
/// whole when it is an integer), else an SSE 3 `orbit.epoch_v3_ms`
/// (milliseconds), else the universe's origin. Kept as a date, never a double.
fn epoch_of(row: &BTreeMap<String, Value>) -> Result<UniverseTime, sse_core::time::OutOfRange> {
    match (row.get("orbit.epoch_s"), row.get("orbit.epoch_v3_ms")) {
        (Some(Value::Int(s)), _) => i64::try_from(*s).map(UniverseTime::seconds).map_err(|_| {
            sse_core::time::OutOfRange(format!("{s} seconds is beyond what dates can hold"))
        }),
        (Some(Value::Float(x)), _) => UniverseTime::from_seconds_f64(*x),
        (_, Some(Value::Int(ms))) => {
            let ms = *ms;
            let s = i64::try_from(ms.div_euclid(1000)).map_err(|_| {
                sse_core::time::OutOfRange(format!("{ms} ms is beyond what dates can hold"))
            })?;
            Ok(UniverseTime {
                s,
                ns: (ms.rem_euclid(1000) * 1_000_000) as u32,
            })
        }
        (_, Some(Value::Float(x))) => UniverseTime::from_seconds_f64(x / 1000.0),
        _ => Ok(UniverseTime::ORIGIN),
    }
}

/// Pass 1: every orbit under `system` read from its facts; the unplaced
/// come back with what they miss.
fn read_orbits(facts: &Facts, system: &Address, seed: Option<&Seed>) -> Read {
    let mut out: Vec<Placed> = vec![];
    let mut orbits: BTreeMap<Address, Orbit> = BTreeMap::new();
    let mut host_mass: BTreeMap<Address, f64> = BTreeMap::new();
    let mut notes: BTreeMap<Address, MotionNote> = BTreeMap::new();
    for (a, row) in facts.iter().filter(|(a, _)| system.contains(a)) {
        let Some(sma) = row.get("orbit.semi_major_axis") else {
            continue;
        };
        let mut missing = vec![];
        let host: Option<Address> = row
            .get("orbit.host")
            .or_else(|| row.get("parent"))
            .and_then(|v| match v {
                Value::Text(t) => t.parse().ok(),
                _ => None,
            })
            .or_else(|| a.parent());
        let mh = host
            .as_ref()
            .and_then(|h| facts.get(h))
            .and_then(|r| r.get("body.mass"))
            .and_then(num);
        let a_m = num(sma).filter(|x| *x > 0.0);
        if a_m.is_none() {
            missing.push("orbit.semi_major_axis".to_string());
        }
        if mh.is_none() {
            missing.push(match &host {
                Some(h) => format!("{h} body.mass"),
                None => "orbit.host".into(),
            });
        }
        if host.as_ref() == Some(a) {
            missing.push("orbit.host other than itself".into());
        }
        let e = row.get("orbit.eccentricity").and_then(num).unwrap_or(0.0);
        if !(0.0..1.0).contains(&e) {
            missing.push("orbit.eccentricity below 1".into());
        }
        let (m0, source) = match row.get("orbit.mean_anomaly_at_epoch").and_then(num) {
            Some(x) => (Some(x), "derived"),
            None => match seed {
                Some(s) => {
                    let d = Draws::new(
                        *s,
                        a.clone(),
                        Stream::new("orbit.phase.v1").expect("stream"),
                    );
                    (Some(d.unit(0) * m::TAU), "resolved")
                }
                None => {
                    missing.push("orbit.mean_anomaly_at_epoch (no universe to draw one)".into());
                    (None, "")
                }
            },
        };
        let (Some(h), Some(a_m), Some(mh), Some(m0), true) =
            (host.clone(), a_m, mh, m0, missing.is_empty())
        else {
            out.push(Placed {
                address: a.to_string(),
                placed: false,
                parent: host.map(|h| h.to_string()),
                period_s: None,
                phase_source: None,
                missing,
                motion: None,
                at: vec![],
            });
            continue;
        };
        let mass = row.get("body.mass").and_then(num).unwrap_or(0.0);
        let measured = match motion_of(facts, a, row, G * (mh + mass)) {
            Some((ms, note)) => {
                notes.insert(a.clone(), note);
                ms
            }
            None => None,
        };
        let epoch = match epoch_of(row) {
            Ok(t) => t,
            Err(e) => {
                out.push(Placed {
                    address: a.to_string(),
                    placed: false,
                    parent: Some(h.to_string()),
                    period_s: None,
                    phase_source: None,
                    missing: vec![format!("orbit.epoch_s within range: {e}")],
                    motion: None,
                    at: vec![],
                });
                continue;
            }
        };
        let inc = row.get("orbit.inclination").and_then(num).unwrap_or(0.0);
        let node = row.get("orbit.ascending_node").and_then(num).unwrap_or(0.0);
        let peri = row
            .get("orbit.argument_of_periapsis")
            .and_then(num)
            .unwrap_or(0.0);
        host_mass.insert(h.clone(), mh);
        orbits.insert(
            a.clone(),
            Orbit {
                host: h,
                a_m,
                e,
                m0,
                epoch,
                rot: rotation(inc, node, peri),
                mass,
                source,
                measured,
            },
        );
    }
    (out, orbits, host_mass, notes)
}

/// What a re-parent keeps (editing design 10A, R4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Keep {
    /// A round orbit at its present distance from the new centre.
    Tidy,
    /// Where it is and how it is moving now, re-expressed round the new host.
    Motion,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: [f64; 3]) -> f64 {
    m::sqrt(dot(a, a))
}
fn wrap(x: f64) -> f64 {
    let y = x - m::TAU * m::floor(x / m::TAU);
    if y >= m::TAU {
        0.0
    } else {
        y
    }
}

/// Keplerian elements from a relative position and velocity (the usual
/// vector route, e.g. Vallado, Fundamentals of Astrodynamics, alg. 9):
/// a, e, inclination, node, argument of periapsis and the mean anomaly, all
/// in the frame of the vectors. None when unbound (e >= 1).
pub fn elements_from_state(r: [f64; 3], v: [f64; 3], mu: f64) -> Option<[f64; 6]> {
    let rn = norm(r);
    let h = cross(r, v);
    let hn = norm(h);
    if rn <= 0.0 || hn <= 0.0 || mu <= 0.0 {
        return None;
    }
    let energy = dot(v, v) / 2.0 - mu / rn;
    if energy >= 0.0 {
        return None;
    }
    let a = -mu / (2.0 * energy);
    let ev = sub(scale(cross(v, h), 1.0 / mu), scale(r, 1.0 / rn));
    let e = norm(ev);
    if e >= 1.0 {
        return None;
    }
    let inc = m::acos((h[2] / hn).clamp(-1.0, 1.0));
    let nvec = [-h[1], h[0], 0.0];
    let nn = norm(nvec);
    let node = if nn > 1e-12 * hn {
        wrap(m::atan2(nvec[1], nvec[0]))
    } else {
        0.0
    };
    // Argument of latitude of the periapsis and of the body, measured in
    // the orbit's plane from the node (or the x axis when flat).
    let (p, q) = if nn > 1e-12 * hn {
        let p = scale(nvec, 1.0 / nn);
        (p, cross(scale(h, 1.0 / hn), p))
    } else {
        let p = [1.0, 0.0, 0.0];
        (p, cross(scale(h, 1.0 / hn), p))
    };
    let at = |x: [f64; 3]| wrap(m::atan2(dot(x, q), dot(x, p)));
    let peri = if e > 1e-12 { at(ev) } else { 0.0 };
    let nu = wrap(at(r) - peri);
    let ea = 2.0
        * m::atan2(
            m::sqrt(1.0 - e) * m::sin(nu / 2.0),
            m::sqrt(1.0 + e) * m::cos(nu / 2.0),
        );
    let mean = wrap(ea - e * m::sin(ea));
    Some([a, e, inc, node, peri, mean])
}

/// K13 `rehost`: the facts that put `body` (with everything round it) round
/// `to` at `t`, keeping a tidy orbit or its present motion. Worked out in the
/// same hierarchy `orbits_at` uses: the new orbit is about the barycentre of
/// `to` and what goes round `to` inside the body's present distance.
pub fn rehost(
    facts: &Facts,
    system: &Address,
    body: &Address,
    to: &Address,
    t: i64,
    seed: Option<&Seed>,
    keep: Keep,
) -> Result<BTreeMap<String, Value>, String> {
    if body == to {
        return Err(format!("{body} cannot orbit itself"));
    }
    let (_, orbits, host_mass, _) = read_orbits(facts, system, seed);
    if !orbits.contains_key(body) {
        return Err(format!(
            "{body} has no orbit to move (it needs one placed first)"
        ));
    }
    let tree = Tree::new(&orbits, &host_mass);
    let mut mine = vec![];
    tree.subtree(body, &mut mine, 0);
    if mine.contains(to) {
        return Err(format!(
            "{to} goes round {body}: a body cannot orbit what orbits it"
        ));
    }
    let dt = 30_i64;
    let fr = tree.frames(&[t - dt, t, t + dt]);
    if !fr[1].0.contains_key(to) {
        return Err(format!(
            "{to} is not placed at that time, so nothing can go round it"
        ));
    }
    let mass_of = |a: &Address| tree.mass.get(a).copied().unwrap_or(0.0);
    // The barycentre of a set of bodies in one frame.
    let bary = |set: &[Address], pos: &Points| -> ([f64; 3], f64) {
        let mut tot = 0.0;
        let mut b = [0.0; 3];
        for x in set {
            if let Some(p) = pos.get(x) {
                let w = mass_of(x);
                b = add(b, scale(*p, w));
                tot += w;
            }
        }
        if tot > 0.0 {
            (scale(b, 1.0 / tot), tot)
        } else {
            (pos.get(&set[0]).copied().unwrap_or([0.0; 3]), 0.0)
        }
    };
    let d = norm(sub(fr[1].0[body], fr[1].0[to]));
    let mut inner = vec![];
    tree.subtree(to, &mut inner, 0);
    inner.retain(|x| !mine.contains(x));
    // Only `to`'s own children further in than the body count (with theirs).
    let outer: Vec<Address> = tree
        .kids
        .get(to)
        .into_iter()
        .flatten()
        .filter(|c| !mine.contains(c) && orbits[*c].a_m >= d)
        .cloned()
        .collect();
    let mut drop = vec![];
    for c in &outer {
        tree.subtree(c, &mut drop, 0);
    }
    inner.retain(|x| !drop.contains(x));
    let state = |pos: &Points| {
        let (bi, mi) = bary(&inner, pos);
        let (bb, mb) = bary(&mine, pos);
        (sub(bb, bi), mi + mb)
    };
    let (r, mu_mass) = state(&fr[1].0);
    let (rm, _) = state(&fr[0].0);
    let (rp, _) = state(&fr[2].0);
    let v = scale(sub(rp, rm), 1.0 / (2.0 * dt as f64));
    let mu = G * mu_mass;
    let el = match keep {
        Keep::Tidy => {
            let rn = norm(r);
            [rn, 0.0, 0.0, 0.0, 0.0, wrap(m::atan2(r[1], r[0]))]
        }
        Keep::Motion => elements_from_state(r, v, mu).ok_or_else(|| {
            format!(
                "moving as it is, {body} is not bound to {to}: it would escape (keep a tidy orbit, or pick a heavier or nearer host)"
            )
        })?,
    };
    let f = Value::Float;
    let mut row = BTreeMap::new();
    row.insert("orbit.host".to_string(), Value::Text(to.to_string()));
    row.insert("orbit.semi_major_axis".to_string(), f(el[0]));
    row.insert("orbit.eccentricity".to_string(), f(el[1]));
    row.insert("orbit.inclination".to_string(), f(el[2]));
    row.insert("orbit.ascending_node".to_string(), f(el[3]));
    row.insert("orbit.argument_of_periapsis".to_string(), f(el[4]));
    row.insert("orbit.mean_anomaly_at_epoch".to_string(), f(el[5]));
    row.insert("orbit.epoch_s".to_string(), Value::Int(t.into()));
    Ok(row)
}

/// Facts from an inline `[{address, facts: {..}}]` list.
pub fn inline_facts(bodies: &[serde_json::Value]) -> Result<Facts, Response> {
    let mut f = Facts::new();
    for (i, b) in bodies.iter().enumerate() {
        let a: Address = b["address"]
            .as_str()
            .ok_or_else(|| err("bad_request", format!("body {i}: no address")))?
            .parse()
            .map_err(|e| err("bad_address", format!("body {i}: {e}")))?;
        let row: BTreeMap<String, Value> = serde_json::from_value(b["facts"].clone())
            .map_err(|e| err("bad_value", format!("body {i}: facts: {e}")))?;
        f.insert(a, row);
    }
    Ok(f)
}

/// The campaign clock event: `campaign.now_s` on the root, active from the
/// origin so every reading of the universe sees the latest one.
pub fn campaign_now_event(ch: &Chronicle, root: &Address, at_s: i64) -> Result<Event, Response> {
    let mut set = BTreeMap::new();
    set.insert(
        root.clone(),
        [(CAMPAIGN_NOW_FIELD.to_string(), Value::Int(at_s.into()))].into(),
    );
    Event::new(EventBody {
        kind: CAMPAIGN_NOW.into(),
        subject: vec![root.clone()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: ch.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("gm", "campaign-clock", "1"),
    })
    .map_err(|e| err("bad_event", format!("{e}")))
}

/// The campaign's now, and the event that set it.
pub fn campaign_now(ch: &Chronicle, root: &Address) -> (Option<i64>, Option<String>) {
    let e = ch
        .events()
        .iter()
        .rev()
        .find(|e| e.body.kind == CAMPAIGN_NOW && e.body.subject.first() == Some(root));
    let at = e
        .and_then(|e| e.body.payload.set.get(root))
        .and_then(|r| r.get(CAMPAIGN_NOW_FIELD))
        .and_then(|v| match v {
            Value::Int(i) => i64::try_from(*i).ok(),
            _ => None,
        });
    (at, e.map(|e| e.id.hex()))
}

/// A pack calendar (`provides.other.calendar`): the kernel checks and
/// carries it; the UI formats dates with it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calendar {
    pub name: String,
    /// Universe time of day 1 of year `epoch_year`.
    pub epoch_s: i64,
    #[serde(default)]
    pub epoch_year: i64,
    /// Seconds in a day and days in a year.
    pub day_s: f64,
    pub year_days: f64,
    /// Months in order; their days must add to `year_days` (whole years only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub months: Vec<Month>,
    /// For the UI, e.g. `"{day:03}-{year}"` (Traveller) or `"{mday} {month} {year}"`.
    pub format: String,
    /// Named days of the year (1-based), e.g. the Imperial Holiday.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named_days: Vec<NamedDay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub week: Option<Week>,
    /// Leap years: every `every` years, except every `except_every`, but every `but_every`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap: Option<Leap>,
    /// The month (0-based) that gains the leap day; needed when months and leap are both set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap_month: Option<usize>,
    /// A line for people on how the calendar reads (e.g. Sol today's note
    /// on UTC days counted on TT); carried, never read by the kernel.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub about: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedDay {
    pub day: u32,
    pub name: String,
    /// The day belongs to no week (needs a week that restarts each year).
    #[serde(default)]
    pub outside_weeks: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Week {
    pub names: Vec<String>,
    /// 1-based day of the year on which the first week starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_day: Option<u32>,
    /// True (the default): weeks run on across years. False: they restart each year.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuous: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Leap {
    pub every: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub except_every: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub but_every: Option<u32>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Month {
    pub name: String,
    pub days: u32,
}

impl Calendar {
    pub fn check(&self) -> Result<(), String> {
        if !(self.day_s.is_finite()
            && self.day_s > 0.0
            && self.year_days.is_finite()
            && self.year_days > 0.0)
        {
            return Err("day_s and year_days must be positive".into());
        }
        if self.format.trim().is_empty() {
            return Err("a calendar needs a format".into());
        }
        if !self.months.is_empty() {
            let sum: u32 = self.months.iter().map(|x| x.days).sum();
            if f64::from(sum) != self.year_days {
                return Err(format!(
                    "months add to {sum} days, the year has {}",
                    self.year_days
                ));
            }
        }
        let max_day = self.year_days + if self.leap.is_some() { 1.0 } else { 0.0 };
        let continuous = self
            .week
            .as_ref()
            .is_none_or(|w| w.continuous != Some(false));
        for d in &self.named_days {
            if d.day == 0 || f64::from(d.day) > max_day || d.name.trim().is_empty() {
                return Err(format!(
                    "named day {} must be named and lie in 1 to {max_day}",
                    d.day
                ));
            }
            if d.outside_weeks && continuous {
                return Err(format!(
                    "named day {}: outside_weeks needs a week with continuous false",
                    d.name
                ));
            }
        }
        if let Some(w) = &self.week {
            if w.names.is_empty() || w.names.iter().any(|n| n.trim().is_empty()) {
                return Err("a week needs its day names".into());
            }
            if let Some(f) = w.first_day {
                if f == 0 || f64::from(f) > max_day {
                    return Err(format!("week first_day {f} must lie in 1 to {max_day}"));
                }
            }
        }
        if let Some(l) = &self.leap {
            let ok = |x: Option<u32>, of: u32| x.is_none_or(|x| x > of && x % of == 0);
            if l.every == 0
                || !ok(l.except_every, l.every)
                || !ok(l.but_every, l.except_every.unwrap_or(u32::MAX))
                || (l.but_every.is_some() && l.except_every.is_none())
            {
                return Err("leap: every > 0; except_every a multiple of it; but_every a multiple of except_every".into());
            }
            if !self.months.is_empty() {
                match self.leap_month {
                    Some(i) if i < self.months.len() => {}
                    _ => {
                        return Err(
                            "leap_month (0-based) is needed when months and leap are both set"
                                .into(),
                        )
                    }
                }
            }
        } else if self.leap_month.is_some() {
            return Err("leap_month needs leap".into());
        }
        Ok(())
    }
}

/// The top pack's calendar, checked (None when no pack has one).
pub fn load_calendar(ms: &[sse_pack::PackManifest]) -> Result<Option<Calendar>, Vec<String>> {
    let Some(p) = ms
        .iter()
        .rev()
        .find(|p| p.provides.other.contains_key("calendar"))
    else {
        return Ok(None);
    };
    let v = serde_json::to_value(&p.provides.other["calendar"]).map_err(|e| vec![e.to_string()])?;
    let c: Calendar =
        serde_json::from_value(v).map_err(|e| vec![format!("{}: calendar: {e}", p.id)])?;
    c.check()
        .map_err(|e| vec![format!("{}: calendar: {e}", p.id)])?;
    Ok(Some(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth_sun() -> Facts {
        let mut f = Facts::new();
        let sun: Address = "u:o/sys:0/star:0".parse().unwrap();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        f.insert(
            sun.clone(),
            [("body.mass".to_string(), Value::Float(1.988_47e30))].into(),
        );
        f.insert(
            earth,
            [
                ("orbit.host".to_string(), Value::Text(sun.to_string())),
                (
                    "orbit.semi_major_axis".to_string(),
                    Value::Float(1.495_978_707e11),
                ),
                ("orbit.eccentricity".to_string(), Value::Float(0.0167)),
                ("orbit.mean_anomaly_at_epoch".to_string(), Value::Float(0.0)),
                ("body.mass".to_string(), Value::Float(5.972e24)),
            ]
            .into(),
        );
        f
    }

    #[test]
    fn kepler_orbits_land_where_they_should() {
        let f = earth_sun();
        let sys: Address = "u:o/sys:0".parse().unwrap();
        let year = 31_558_149;
        let p = orbits_at(&f, &sys, &[0, year / 4, year / 2, year], None);
        assert_eq!(p.len(), 1);
        let e = &p[0];
        assert!(e.placed);
        let days = e.period_s.unwrap() / 86_400.0;
        println!("Earth's period {days:.3} days, positions {:?}", e.at);
        assert!((days - 365.256).abs() < 0.01);
        let a = 1.495_978_707e11;
        let r = |x: &At| m::sqrt(x.r_m[0] * x.r_m[0] + x.r_m[1] * x.r_m[1] + x.r_m[2] * x.r_m[2]);
        assert!(
            (r(&e.at[0]) - a * (1.0 - 0.0167)).abs() < 1.0,
            "periapsis at M = 0"
        );
        assert!(
            (r(&e.at[2]) - a * (1.0 + 0.0167)).abs() < 1.0e6,
            "apoapsis half a year on"
        );
        assert!(e.at[0].phase_deg.abs() < 1e-9);
        assert!((e.at[2].phase_deg - 180.0).abs() < 0.05);
        assert!((e.at[3].phase_deg - e.at[0].phase_deg).abs() < 0.01 || e.at[3].phase_deg > 359.99);
        // Kepler's equation is solved to the bit at high eccentricity too.
        for &ecc in &[0.0, 0.3, 0.9, 0.99] {
            for k in 0..64 {
                let mm = k as f64 * 0.1;
                let ea = eccentric_anomaly(mm, ecc);
                let back = ea - ecc * m::sin(ea);
                let want = mm - m::TAU * m::floor(mm / m::TAU);
                assert!((back - want).abs() < 1e-12, "e {ecc}, M {mm}");
            }
        }
        // Vis-viva holds along an inclined, eccentric orbit.
        let mut g = earth_sun();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        let row = g.get_mut(&earth).unwrap();
        row.insert("orbit.eccentricity".into(), Value::Float(0.5));
        row.insert("orbit.inclination".into(), Value::Float(0.4));
        row.insert("orbit.ascending_node".into(), Value::Float(1.1));
        row.insert("orbit.argument_of_periapsis".into(), Value::Float(2.3));
        let times: Vec<i64> = (0..40).map(|k| k * 1_000_000).collect();
        let q = &orbits_at(&g, &sys, &times, None)[0];
        let mu = G * (1.988_47e30 + 5.972e24);
        for w in q.at.windows(2) {
            let (p0, p1) = (w[0].r_m, w[1].r_m);
            let v = [
                (p1[0] - p0[0]) / 1.0e6,
                (p1[1] - p0[1]) / 1.0e6,
                (p1[2] - p0[2]) / 1.0e6,
            ];
            let mid = r(&w[0]).min(r(&w[1]));
            let v2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
            let lo = mu * (2.0 / r(&w[0]).max(r(&w[1])) - 1.0 / a);
            let hi = mu * (2.0 / mid - 1.0 / a);
            assert!(
                v2 > lo * 0.97 && v2 < hi * 1.03,
                "vis-viva between the ends of each chord"
            );
        }
    }

    /// TP1: a date at a star's age, or at 13.8 Gyr, still moves a body every
    /// second. Before TP1 the date became a double first, and a low station
    /// took 3 distinct places in 64 one-second steps, jumping 424 km.
    #[test]
    fn one_second_moves_a_station_at_any_age() {
        let mut f = earth_sun();
        let sys: Address = "u:o/sys:0".parse().unwrap();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        let station: Address = "u:o/sys:0/body:3/body:1".parse().unwrap();
        f.insert(
            station.clone(),
            [
                ("orbit.host".to_string(), Value::Text(earth.to_string())),
                ("orbit.semi_major_axis".to_string(), Value::Float(6.778e6)),
                ("orbit.mean_anomaly_at_epoch".to_string(), Value::Float(0.3)),
                ("body.mass".to_string(), Value::Float(4.2e5)),
            ]
            .into(),
        );
        let yr = sse_core::time::YEAR_S;
        let star_age = 4_610_000_000 * yr;
        let cosmic = 13_800_000_000 * yr;
        // (date, the epoch of the dated phase): the epoch far back, and the epoch near.
        for (base, epoch) in [
            (star_age, None),
            (cosmic, None),
            (cosmic, Some(cosmic - 86_400)),
        ] {
            let mut g = f.clone();
            if let Some(ep) = epoch {
                for a in [&earth, &station] {
                    g.get_mut(a)
                        .unwrap()
                        .insert("orbit.epoch_s".into(), Value::Int(ep.into()));
                }
            }
            let times: Vec<i64> = (0..64).map(|k| base + k).collect();
            let got = orbits_at(&g, &sys, &times, None);
            for (who, speed) in [(&station, 7.67e3), (&earth, 2.98e4)] {
                let p = got.iter().find(|p| p.address == who.to_string()).unwrap();
                assert!(p.placed, "{p:?}");
                let mut distinct = 1;
                for w in p.at.windows(2) {
                    let d = sub(w[1].r_m, w[0].r_m);
                    let jump = m::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
                    if jump > 0.0 {
                        distinct += 1;
                    }
                    assert!(
                        (jump - speed).abs() < 0.02 * speed,
                        "{who} at {base} s, epoch {epoch:?}: moved {jump:.1} m in one second"
                    );
                }
                assert_eq!(distinct, 64, "{who} at {base} s, epoch {epoch:?}");
            }
        }
    }

    #[test]
    fn no_guessed_positions() {
        let mut f = earth_sun();
        let sys: Address = "u:o/sys:0".parse().unwrap();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        f.get_mut(&earth)
            .unwrap()
            .remove("orbit.mean_anomaly_at_epoch");
        let p = orbits_at(&f, &sys, &[0], None);
        assert!(!p[0].placed && p[0].missing[0].contains("mean_anomaly"));
        let s = Seed::from_text("o");
        let a = orbits_at(&f, &sys, &[0, 100], Some(&s));
        let b = orbits_at(&f, &sys, &[0, 100], Some(&s));
        assert_eq!(a, b);
        assert_eq!(a[0].phase_source.as_deref(), Some("resolved"));
        let sun: Address = "u:o/sys:0/star:0".parse().unwrap();
        f.remove(&sun);
        let p = orbits_at(&f, &sys, &[0], Some(&s));
        assert!(
            !p[0].placed && p[0].missing[0].contains("body.mass"),
            "{:?}",
            p[0]
        );
    }

    #[test]
    fn calendars_with_named_days_weeks_and_leaps() {
        let cal = |v: serde_json::Value| -> Result<(), String> {
            serde_json::from_value::<Calendar>(v)
                .map_err(|e| e.to_string())
                .and_then(|c| c.check())
        };
        // The Imperial calendar: Holiday outside the weeks, weeks restart each year on day 2.
        let imperial = serde_json::json!({
            "name": "Imperial", "epoch_s": 0, "epoch_year": 1105, "day_s": 86400.0,
            "year_days": 365.0, "format": "{day:03}-{year}",
            "named_days": [{"day": 1, "name": "Holiday", "outside_weeks": true}],
            "week": {"names": ["Wonday", "Tuday", "Thirday", "Forday", "Fiday", "Sixday", "Senday"],
                     "first_day": 2, "continuous": false}
        });
        assert_eq!(cal(imperial.clone()), Ok(()));
        let mut v = imperial.clone();
        v["week"]["continuous"] = serde_json::json!(true);
        assert!(cal(v).unwrap_err().contains("outside_weeks"));
        let mut v = imperial.clone();
        v["named_days"][0]["day"] = serde_json::json!(366);
        assert!(cal(v).is_err());
        let mut v = imperial;
        v["week"]["shape"] = serde_json::json!(1);
        assert!(cal(v).is_err(), "unknown keys are still refused");
        // Gregorian: 4, 100, 400, with February gaining the day.
        let months: Vec<serde_json::Value> = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            .iter()
            .enumerate()
            .map(|(i, d)| serde_json::json!({"name": format!("M{i}"), "days": d}))
            .collect();
        let greg = serde_json::json!({
            "name": "Gregorian", "epoch_s": 0, "day_s": 86400.0, "year_days": 365.0,
            "format": "{mday} {month} {year}", "months": months,
            "leap": {"every": 4, "except_every": 100, "but_every": 400}, "leap_month": 1,
            "named_days": [{"day": 366, "name": "Leap day end"}]
        });
        assert_eq!(cal(greg.clone()), Ok(()));
        let mut v = greg.clone();
        v.as_object_mut().unwrap().remove("leap_month");
        assert!(cal(v).unwrap_err().contains("leap_month"));
        let mut v = greg;
        v["leap"]["except_every"] = serde_json::json!(90);
        assert!(cal(v).is_err());
    }
}
