//! FTL rules as data (example packs note 4A, accepted by Alex 23 Nov 2026): one simple, flexible shape for every
//! setting's faster-than-light rule. Each rule states five plain things: where a leg may start and end, what the
//! crew does, what it costs, what can go wrong and how messages travel. Where a leg may start and end is a list of
//! places, any of which will do:
//!
//! - clear of a gravity well by a multiple of a body's size (Traveller's 100 diameters, of the star or of every
//!   world), or of its Hill sphere;
//! - a Lagrange point, where the pulls balance (L1 to L5 of a world about its star);
//! - a gate, a construct placed in the system (found, not built);
//! - anywhere.
//!
//! `jump_map` answers where in a system a leg may begin at a time, for the chart to draw as a HUD layer and for the
//! voyage finder to reach. The numbers in a rule are the pack's; nothing here is a law of physics (the honesty mark
//! is the pack's own `fictional`).

use crate::kepler::V3;
use crate::NavSystem;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const AU: f64 = sse_core::sheet::AU;

/// What a body's size is measured by, for a "clear of" limit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    Diameter,
    Radius,
    /// The body's Hill sphere about its star (its own gravity's reach).
    Hill,
}

/// Which bodies a limit applies to.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "which")]
pub enum Which {
    /// The primary star only.
    Star,
    /// The star and every world.
    Every,
    /// The star and the worlds at least this massive, Earth masses.
    Massive { min_earth: f64 },
}

/// One kind of place a leg may start or end.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Place {
    /// Outside every listed body's limit: `multiple` × its size.
    ClearOf {
        multiple: f64,
        of: Measure,
        bodies: Which,
    },
    /// At a world's Lagrange points (1 to 5), within `within` of its Hill radius (default 0.1).
    Lagrange {
        points: Vec<u8>,
        bodies: Which,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        within: Option<f64>,
    },
    /// Where gravity (star and worlds together) is below (or above, `above: true`) this level, m/s²: "out of
    /// the well", or a drive that must come into one to discharge.
    Gravity {
        m_s2: f64,
        #[serde(default)]
        above: bool,
    },
    /// Above or below the star's poles, this far out, au (fixed-point settings).
    Poles {
        au: f64,
        within_au: f64,
    },
    /// At a gate: a placed construct of this template or tag, and optionally only to its linked partner.
    Gate {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        construct: String,
        #[serde(default)]
        linked: bool,
    },
    Anywhere,
}

/// How messages travel in the setting.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "by")]
pub enum Messages {
    /// Radio at light speed: the delay is the distance.
    Light,
    /// Only aboard ships: news moves as fast as the fastest courier.
    Ships,
    /// FTL radio: `times_light` the speed of light, out to `range_pc` (none: no limit).
    Radio {
        times_light: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        range_pc: Option<f64>,
    },
    Instant,
}

/// How long a leg takes (apart from fuel). Time aboard is the same as time at home unless `aboard_per_home` says.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "form")]
pub enum Duration {
    Instant,
    /// The same whatever the distance (Traveller: about a week), with a spread for dice.
    Fixed {
        days: f64,
        #[serde(default)]
        spread_days: f64,
    },
    /// So many days per parsec at a drive rating of 1, divided by the ship's rating.
    PerParsec {
        days_at_rating_1: f64,
    },
    /// A speed from the rating, in light speeds: `times_light` × rating ^ `power` (the rating cubed: power 3).
    Formula {
        times_light: f64,
        power: f64,
    },
}

impl Duration {
    /// Days at home for a leg of `d_pc` by a drive of `rating`.
    pub fn home_days(&self, d_pc: f64, rating: f64) -> f64 {
        match self {
            Duration::Instant => 0.0,
            Duration::Fixed { days, .. } => *days,
            Duration::PerParsec { days_at_rating_1 } => days_at_rating_1 * d_pc / rating.max(1e-9),
            Duration::Formula { times_light, power } => {
                let c = times_light * m::pow(rating.max(1e-9), *power);
                // Distance over speed.
                d_pc * sse_core::sheet::PC / (sse_core::sheet::C * c) / sse_core::sheet::DAY
            }
        }
    }
}

/// A setting's FTL rule: the six plain fields every rule states.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FtlRule {
    pub id: String,
    pub name: String,
    /// none, jump, hop, formula, rate or gate (the family; the numbers live in the ship's drive).
    pub family: String,
    /// Where a leg may start, and where it may end: any listed place will do. Empty: anywhere.
    #[serde(default)]
    pub start: Vec<Place>,
    #[serde(default)]
    pub end: Vec<Place>,
    /// What the crew does on the way (awake, asleep, at risk), in words.
    #[serde(default)]
    pub crew: String,
    /// How long a leg takes, at home; time aboard is `aboard_per_home` of it (1 unless the pack says).
    pub duration: Duration,
    #[serde(default = "one")]
    pub aboard_per_home: f64,
    /// What it costs, in words (fuel; the figures live in the ship's drive).
    #[serde(default)]
    pub cost: String,
    /// What can go wrong, for the GM to weigh: never a refusal.
    #[serde(default)]
    pub risks: Vec<String>,
    pub messages: Messages,
    /// Past a named ceiling of physics: shown as fictional on the GM's view.
    #[serde(default)]
    pub fictional: bool,
}

fn one() -> f64 {
    1.0
}

/// A sphere a leg may not start inside.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Keepout {
    pub body: String,
    pub name: String,
    pub centre_m: V3,
    pub radius_m: f64,
}

/// A point a leg may start at.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Point {
    pub body: String,
    pub name: String,
    /// "L1" to "L5", or "gate".
    pub what: String,
    pub at_m: V3,
    /// How close counts as there, m.
    pub within_m: f64,
}

/// Where in a system a leg may begin, at one time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JumpMap {
    pub t_s: f64,
    /// Clear of these to start (any one place kind that is a "clear of").
    pub keepouts: Vec<Keepout>,
    pub points: Vec<Point>,
    /// Gate constructs the rule asks for: placed by the campaign, read by the caller.
    pub gates: Vec<String>,
    /// Gravity contours: below (or above) this level counts, m/s²; the caller draws each body's contour.
    pub gravity: Vec<(f64, bool)>,
    pub anywhere: bool,
    /// The rule in words, for the HUD's key.
    pub words: Vec<String>,
}

fn size_m(radius_m: f64, hill_m: f64, of: Measure) -> f64 {
    match of {
        Measure::Diameter => 2.0 * radius_m,
        Measure::Radius => radius_m,
        Measure::Hill => hill_m,
    }
}

fn picks(w: &Which, mass_earth: f64) -> bool {
    match w {
        Which::Star => false,
        Which::Every => true,
        Which::Massive { min_earth } => mass_earth >= *min_earth,
    }
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn norm(a: V3) -> f64 {
    m::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
}

/// A world's Lagrange point about its star at its position `r` (star at the origin), velocity `v`: L1 and L2 at
/// the Hill radius inside and outside, L3 opposite, L4 and L5 60° ahead and behind along the orbit.
fn lagrange(r: V3, v: V3, hill_m: f64, n: u8) -> Option<V3> {
    let d = norm(r);
    if d <= 0.0 {
        return None;
    }
    let u = scale(r, 1.0 / d);
    // The direction of motion, made square to r, in the orbit's plane.
    let along = {
        let k = v[0] * u[0] + v[1] * u[1] + v[2] * u[2];
        let t = [v[0] - k * u[0], v[1] - k * u[1], v[2] - k * u[2]];
        let tn = norm(t);
        if tn <= 0.0 {
            return None;
        }
        scale(t, 1.0 / tn)
    };
    let turn = |deg: f64| {
        let (s, c) = (m::sin(deg.to_radians()), m::cos(deg.to_radians()));
        add(scale(u, d * c), scale(along, d * s))
    };
    match n {
        1 => Some(scale(u, d - hill_m)),
        2 => Some(scale(u, d + hill_m)),
        3 => Some(scale(u, -d)),
        4 => Some(turn(60.0)),
        5 => Some(turn(-60.0)),
        _ => None,
    }
}

/// Where a leg may begin under `places` in `sys` at `t_s` (the start or the end list of a rule).
pub fn jump_map(sys: &NavSystem, places: &[Place], t_s: f64) -> JumpMap {
    let mut out = JumpMap {
        t_s,
        keepouts: vec![],
        points: vec![],
        gates: vec![],
        gravity: vec![],
        anywhere: places.is_empty(),
        words: vec![],
    };
    for p in places {
        match p {
            Place::Anywhere => {
                out.anywhere = true;
                out.words.push("anywhere".into());
            }
            Place::ClearOf {
                multiple,
                of,
                bodies,
            } => {
                let r_star = sys.star_radius_m();
                let star_size = size_m(
                    r_star,
                    0.0,
                    if *of == Measure::Hill {
                        Measure::Radius
                    } else {
                        *of
                    },
                );
                out.keepouts.push(Keepout {
                    body: String::new(),
                    name: sys.name.clone(),
                    centre_m: [0.0; 3],
                    radius_m: multiple * star_size,
                });
                for b in sys.bodies.iter().filter(|b| picks(bodies, b.mass_earth)) {
                    let (r, _) = sys.at(b, t_s);
                    let size = size_m(b.radius_m, b.hill_m(sys.star_mass_msun), *of);
                    out.keepouts.push(Keepout {
                        body: b.address.clone(),
                        name: b.name.clone(),
                        centre_m: r,
                        radius_m: multiple * size,
                    });
                }
                let what = match of {
                    Measure::Diameter => "diameters",
                    Measure::Radius => "radii",
                    Measure::Hill => "Hill radii",
                };
                let whose = match bodies {
                    Which::Star => "the star".to_string(),
                    Which::Every => "the star and every world".to_string(),
                    Which::Massive { min_earth } => {
                        format!("the star and every world of {min_earth} Earth masses or more")
                    }
                };
                out.words
                    .push(format!("clear of {whose} by {multiple} {what}"));
            }
            Place::Lagrange {
                points,
                bodies,
                within,
            } => {
                for b in sys.bodies.iter().filter(|b| picks(bodies, b.mass_earth)) {
                    let (r, v) = sys.at(b, t_s);
                    let hill = b.hill_m(sys.star_mass_msun);
                    for &n in points {
                        if let Some(at) = lagrange(r, v, hill, n) {
                            out.points.push(Point {
                                body: b.address.clone(),
                                name: b.name.clone(),
                                what: format!("L{n}"),
                                at_m: at,
                                within_m: within.unwrap_or(0.1) * hill,
                            });
                        }
                    }
                }
                let ls: Vec<String> = points.iter().map(|n| format!("L{n}")).collect();
                out.words.push(format!(
                    "at a world's {} (where the pulls balance)",
                    ls.join(", ")
                ));
            }
            Place::Gate { construct, linked } => {
                out.gates.push(construct.clone());
                let g = if construct.is_empty() {
                    "a gate".to_string()
                } else {
                    format!("a {construct}")
                };
                out.words.push(format!(
                    "at {g}{}",
                    if *linked {
                        ", to its linked partner only"
                    } else {
                        ""
                    }
                ));
            }
            Place::Gravity { m_s2, above } => {
                out.gravity.push((*m_s2, *above));
                out.words.push(format!(
                    "where gravity is {} {m_s2} m/s²",
                    if *above { "above" } else { "below" }
                ));
            }
            Place::Poles { au, within_au } => {
                for (n, z) in [("above the pole", 1.0), ("below the pole", -1.0)] {
                    out.points.push(Point {
                        body: String::new(),
                        name: sys.name.clone(),
                        what: n.into(),
                        at_m: [0.0, 0.0, z * au * AU],
                        within_m: within_au * AU,
                    });
                }
                out.words
                    .push(format!("{au} au above or below the star's poles"));
            }
        }
    }
    out
}

/// Is a position (star at the origin) a place a leg may begin, by this map? Outside every keepout of a
/// "clear of", or near a point. Gates are the caller's: it knows where its constructs are.
pub fn may_begin(map: &JumpMap, at: V3) -> bool {
    may_begin_in(map, None, at)
}

/// The total gravity at a point (star and worlds), m/s².
pub fn gravity_at(sys: &NavSystem, t_s: f64, at: V3) -> f64 {
    let mut g = [0.0; 3];
    let mut pull = |c: V3, mu: f64| {
        let d = [c[0] - at[0], c[1] - at[1], c[2] - at[2]];
        let r = norm(d).max(1.0);
        g = add(g, scale(d, mu / (r * r * r)));
    };
    pull([0.0; 3], sys.mu());
    for b in &sys.bodies {
        pull(sys.at(b, t_s).0, b.mu());
    }
    norm(g)
}

/// As `may_begin`, reading gravity conditions from the system when one is given.
pub fn may_begin_in(map: &JumpMap, sys: Option<&NavSystem>, at: V3) -> bool {
    if let Some(s) = sys {
        if map.gravity.iter().any(|&(lvl, above)| {
            let g = gravity_at(s, map.t_s, at);
            if above {
                g >= lvl
            } else {
                g <= lvl
            }
        }) {
            return true;
        }
    }
    if map.anywhere {
        return true;
    }
    let clear = !map.keepouts.is_empty()
        && map.keepouts.iter().all(|k| {
            norm([
                at[0] - k.centre_m[0],
                at[1] - k.centre_m[1],
                at[2] - k.centre_m[2],
            ]) >= k.radius_m
        });
    let near = map
        .points
        .iter()
        .any(|p| norm([at[0] - p.at_m[0], at[1] - p.at_m[1], at[2] - p.at_m[2]]) <= p.within_m);
    clear || near
}
