//! The flight log (accepted 23 Nov 2026, real spaceflight pack question 11;
//! design: real spaceflight feasibility note, section 6A): one log of dated
//! pieces on any construct (ship, probe, station, stage, lander), with
//! events where pieces meet. Rewinding time looks up the piece in force and
//! works out where the craft was; nothing is integrated step by step.
//!
//! Detail is big burns and flybys only (Alex, 23 Nov): between them the craft
//! is on a stable orbit or a course solved from its ends, so a recorded log
//! never carries the arc itself. That is a fudge, in the register as
//! "flight logs: big burns and flybys only".
//!
//! Pieces and events carry a mark (recorded, planned, authored, estimated),
//! a source and a writer, so history from a pack, a GM's entry and a
//! committed voyage sit in one log. One split mechanism covers stages,
//! landers, probes and station modules: a split names the new constructs,
//! each with its own log from that moment. Times are whole universe seconds.

use crate::kepler::{lambert, propagate, state, Elements, V3};
use crate::NavSystem;
use serde::{Deserialize, Serialize};

/// Who stands behind a piece or event.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mark {
    /// From the historical record (a mission's published trajectory).
    Recorded,
    /// A committed plan not yet flown.
    Planned,
    /// Written by a GM or a pack author.
    Authored,
    /// Worked out where the record is silent.
    Estimated,
}

/// Orbital elements in the parent's frame, at `epoch_s`.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct OrbitElements {
    pub a_m: f64,
    pub e: f64,
    #[serde(default)]
    pub i_deg: f64,
    #[serde(default)]
    pub node_deg: f64,
    /// Longitude of periapsis.
    #[serde(default)]
    pub varpi_deg: f64,
    /// Mean longitude at `epoch_s`.
    #[serde(default)]
    pub mean_longitude_deg: f64,
    pub epoch_s: i64,
}

impl OrbitElements {
    fn kepler(&self) -> Elements {
        Elements {
            a_m: self.a_m,
            e: self.e,
            i_deg: self.i_deg,
            node_deg: self.node_deg,
            varpi_deg: self.varpi_deg,
            mean_longitude_deg: self.mean_longitude_deg,
        }
    }
}

/// How a log ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gone {
    Reentered,
    Impacted,
    LandedLost,
    Escaped,
    LostContact,
}

/// A course: solved from its ends, or as planned.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Course {
    /// Only the next meeting is given: the arc that leaves where the craft
    /// was at the piece's start and meets `to` at `arrive_s` (around the star).
    Lambert { to: String, arrive_s: i64 },
    /// A NAV course or voyage leg as planned, kept as given.
    Planned { course: serde_json::Value },
}

/// What a craft is doing over one piece of its log.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum State {
    Orbit {
        /// A body's name or address, or `star`.
        around: String,
        elements: OrbitElements,
    },
    Course {
        course: Course,
    },
    Landed {
        body: String,
        lat_deg: f64,
        lon_deg: f64,
    },
    Docked {
        to: String,
    },
    BetweenStars {
        leg: serde_json::Value,
    },
    /// A recorded path (a Horizons file), the yardstick for error only.
    Track {
        file: String,
    },
    Gone {
        how: Gone,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Piece {
    pub from_s: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_s: Option<i64>,
    pub state: State,
    pub mark: Mark,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer: Option<String>,
}

/// SSE3's journey stop, every field kept so its totals sum unchanged.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Stop {
    /// depart, arrive, load, unload, mine, refuel, loiter, stuck, disengage.
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journey_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tonnes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuel_kg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_at_s: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_construct_id: Option<String>,
}

/// What happens where pieces meet.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum What {
    Burn {
        dv_m_s: f64,
        #[serde(default)]
        propellant_t: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        engine: Option<String>,
    },
    Flyby {
        body: String,
        periapsis_m: f64,
    },
    /// The log's own construct keeps its identity; `into` are new constructs, each with its own log.
    Split {
        into: Vec<String>,
    },
    Join {
        with: String,
    },
    Entry,
    Landing,
    Launch,
    Stop(Stop),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub at_s: i64,
    #[serde(flatten)]
    pub what: What,
    pub mark: Mark,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer: Option<String>,
}

/// One construct's flight log.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FlightLog {
    pub construct: String,
    #[serde(default)]
    pub name: String,
    pub pieces: Vec<Piece>,
    #[serde(default)]
    pub events: Vec<LogEvent>,
}

/// Where a craft was, in the star's frame.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct At {
    /// The piece's kind (orbit, course, landed, docked, between_stars, track, gone), or `before` its log.
    pub kind: String,
    /// Position and velocity around the star, when the log can say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_m: Option<V3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_m_s: Option<V3>,
    /// What it is near or with (the body it orbits or stands on, the construct it is docked to).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<String>,
    pub mark: Option<Mark>,
    pub words: String,
}

/// Where bodies are: the star at the origin, each body on its orbit.
pub trait Ephemeris {
    /// A body's position and velocity around the star, and its GM; `star` is the origin.
    fn body(&self, name: &str, t_s: f64) -> Option<(V3, V3, f64)>;
    fn star_mu(&self) -> f64;
}

impl Ephemeris for NavSystem {
    fn body(&self, name: &str, t_s: f64) -> Option<(V3, V3, f64)> {
        if name == "star" || name.eq_ignore_ascii_case(&self.name) {
            return Some(([0.0; 3], [0.0; 3], self.mu()));
        }
        let b = NavSystem::body(self, name)?;
        let (r, v) = self.at(b, t_s);
        Some((r, v, b.mu()))
    }
    fn star_mu(&self) -> f64 {
        self.mu()
    }
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

impl FlightLog {
    /// The piece in force at `t_s`: the last one begun by then (a piece runs until the next begins, or its `to_s`).
    pub fn piece_at(&self, t_s: i64) -> Option<(usize, &Piece)> {
        let i = self.pieces.iter().rposition(|p| p.from_s <= t_s)?;
        let p = &self.pieces[i];
        match p.to_s {
            Some(end) if t_s > end => None,
            _ => Some((i, p)),
        }
    }

    /// The events between two times, inclusive, in order.
    pub fn events_between(&self, from_s: i64, to_s: i64) -> Vec<&LogEvent> {
        let mut out: Vec<&LogEvent> = self
            .events
            .iter()
            .filter(|e| e.at_s >= from_s && e.at_s <= to_s)
            .collect();
        out.sort_by_key(|e| e.at_s);
        out
    }

    /// Where the craft was at `t_s`.
    pub fn at(&self, t_s: i64, eph: &dyn Ephemeris) -> At {
        let Some((i, p)) = self.piece_at(t_s) else {
            let before = self.pieces.first().is_none_or(|p| t_s < p.from_s);
            return At {
                kind: if before { "before" } else { "unlogged" }.into(),
                r_m: None,
                v_m_s: None,
                with: None,
                mark: None,
                words: if before {
                    "before its log begins".into()
                } else {
                    "no piece of its log covers this time".into()
                },
            };
        };
        let blank = |kind: &str, with: Option<String>, words: String| At {
            kind: kind.into(),
            r_m: None,
            v_m_s: None,
            with,
            mark: Some(p.mark),
            words,
        };
        let t = t_s as f64;
        match &p.state {
            State::Orbit { around, elements } => match eph.body(around, t) {
                Some((rb, vb, mu)) => {
                    let (r, v) = state(&elements.kepler(), mu, (t_s - elements.epoch_s) as f64);
                    At {
                        kind: "orbit".into(),
                        r_m: Some(add(rb, r)),
                        v_m_s: Some(add(vb, v)),
                        with: Some(around.clone()),
                        mark: Some(p.mark),
                        words: format!("in orbit around {around}"),
                    }
                }
                None => blank(
                    "orbit",
                    Some(around.clone()),
                    format!("in orbit around {around}, which this system does not hold"),
                ),
            },
            State::Course {
                course: Course::Lambert { to, arrive_s },
            } => {
                // The arc from where the craft was as the piece began to where `to` is on arrival.
                let start = if i == 0 {
                    None
                } else {
                    self.at_piece(i - 1, p.from_s, eph)
                };
                let end = eph.body(to, *arrive_s as f64);
                let tof = (*arrive_s - p.from_s) as f64;
                match (start, end) {
                    (Some(r1), Some((r2, _, _))) if tof > 0.0 => {
                        match lambert(r1, r2, tof, eph.star_mu()) {
                            Some((v1, _)) => {
                                let (r, v) = propagate(r1, v1, eph.star_mu(), t - p.from_s as f64);
                                At {
                                    kind: "course".into(),
                                    r_m: Some(r),
                                    v_m_s: Some(v),
                                    with: Some(to.clone()),
                                    mark: Some(p.mark),
                                    words: format!("on its way to {to}, solved to arrive on time"),
                                }
                            }
                            None => blank(
                                "course",
                                Some(to.clone()),
                                format!("on its way to {to}; no transfer meets it in time"),
                            ),
                        }
                    }
                    _ => blank("course", Some(to.clone()), format!("on its way to {to}")),
                }
            }
            State::Course {
                course: Course::Planned { .. },
            } => blank("course", None, "on a planned course".into()),
            State::Landed { body, .. } => {
                let r = eph.body(body, t);
                At {
                    kind: "landed".into(),
                    r_m: r.map(|x| x.0),
                    v_m_s: r.map(|x| x.1),
                    with: Some(body.clone()),
                    mark: Some(p.mark),
                    words: format!("on the ground on {body}"),
                }
            }
            State::Docked { to } => blank("docked", Some(to.clone()), format!("docked with {to}")),
            State::BetweenStars { .. } => blank("between_stars", None, "between stars".into()),
            State::Track { file } => blank("track", None, format!("on a recorded track ({file})")),
            State::Gone { how } => blank(
                "gone",
                None,
                match how {
                    Gone::Reentered => "burnt up on entry",
                    Gone::Impacted => "struck its target",
                    Gone::LandedLost => "landed, and lost",
                    Gone::Escaped => "left the system",
                    Gone::LostContact => "contact lost",
                }
                .into(),
            ),
        }
    }

    /// Where piece `i` puts the craft at `t_s` (used as the start of a solved course).
    fn at_piece(&self, i: usize, t_s: i64, eph: &dyn Ephemeris) -> Option<V3> {
        let mut one = self.clone();
        one.pieces.truncate(i + 1);
        one.pieces[i].to_s = None;
        one.at(t_s, eph).r_m
    }

    /// The constructs this log splits off, with when.
    pub fn splits(&self) -> Vec<(i64, &str)> {
        let mut out = vec![];
        for e in &self.events {
            if let What::Split { into } = &e.what {
                for c in into {
                    out.push((e.at_s, c.as_str()));
                }
            }
        }
        out
    }
}
