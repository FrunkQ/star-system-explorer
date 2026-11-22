//! Voyages between stars (V8 part 3, design `SSE4-voyages-design.md`).
//!
//! One search over one graph: the nodes are star systems, the edges are the
//! legs one ship can make, and the weight is what the GM or the autopilot
//! chose to minimise. A leg is a **jump** (a pack's FTL rule: a rating, fuel
//! per parsec and a fixed time; fictional, marked so) or a **flight** there
//! for real (constant proper acceleration with turnover, special relativity,
//! the relativistic rocket equation; coasting at the best speed the
//! propellant allows). Fuel is carried from leg to leg; a stop where the
//! crew knows of fuel refills both tanks. Unsurveyed systems are dry unless
//! gambles are allowed; then a system whose odds of holding fuel (from the
//! ship's sensors, or failing that from its star) reach the pack's floor is
//! an informed risk, marked with its odds.
//!
//! The model's edges, honestly: the two stars' own motion is ignored (start
//! and end at rest in the local standard of rest), the climb out of and the
//! fall into each system are NAV's in-system courses and are not counted
//! here, and the ship's mass during a flight leaves out its jump fuel.

use crate::{Ship, G0};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::quantity::consts::{C, DAY, JULIAN_YEAR};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};

/// One parsec in metres (IAU 2015 B2: 648000/π au).
pub const PC_M: f64 = 3.085_677_581_491_367e16;

/// A jump drive (a pack's FTL rule). "Jump-N" is the rating; the counted
/// parsecs of a jump are its length rounded up when `whole_parsecs`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JumpDrive {
    /// Longest single jump, parsecs (the rating N of "Jump-N").
    pub rating_pc: f64,
    /// Fuel burnt per parsec jumped, tonnes (the pack's rule).
    pub fuel_t_per_pc: f64,
    /// Jump fuel the tanks hold, tonnes.
    pub tank_t: f64,
    /// Jump fuel aboard now (default: full).
    #[serde(default)]
    pub fuel_t: Option<f64>,
    /// Days a jump takes whatever its length (the pack's rule).
    pub days_per_jump: f64,
    /// Count and pay for whole parsecs, rounded up (Traveller-like rules).
    #[serde(default = "yes")]
    pub whole_parsecs: bool,
    /// Which pack rule this is, shown with the leg ("jump drive, a pack rule").
    #[serde(default)]
    pub rule: String,
}
fn yes() -> bool {
    true
}

impl JumpDrive {
    pub fn fuel(&self) -> f64 {
        self.fuel_t
            .unwrap_or(self.tank_t)
            .clamp(0.0, self.tank_t.max(0.0))
    }
    /// The rating as a whole number for its wording ("Jump-2").
    pub fn rating_words(&self) -> String {
        format!("Jump-{}", fmt_pc(self.rating_pc))
    }
    fn counted(&self, d_pc: f64) -> f64 {
        if self.whole_parsecs {
            m::ceil(d_pc - 1e-9).max(1.0)
        } else {
            d_pc
        }
    }
}

fn fmt_pc(x: f64) -> String {
    if (x - m::round(x)).abs() < 1e-9 {
        format!("{}", m::round(x) as i64)
    } else {
        format!("{x:.1}")
    }
}

/// A star system the finder may visit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VoyageSystem {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub xyz_pc: [f64; 3],
    /// Fuel the crew knows of here ("gas giant", "ice", "port").
    #[serde(default)]
    pub fuel: Vec<String>,
    /// The crew has surveyed this system (so an empty `fuel` means dry).
    #[serde(default)]
    pub surveyed: bool,
    /// The chance this system holds fuel, 0 to 1, for an unsurveyed system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuel_odds: Option<f64>,
    /// Where the odds came from ("the ship's sensors", "its star's metallicity").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub odds_from: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Minimise {
    /// Time at home (the campaign clock).
    #[default]
    HomeTime,
    /// Time aboard (what the crew ages).
    ShipTime,
    /// Tonnes of fuel and propellant burnt.
    Fuel,
    /// Number of legs.
    Legs,
}

/// Pack settings for voyages (`nav.voyage`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VoyageSettings {
    /// Longest flight leg considered, parsecs.
    pub max_flight_pc: f64,
    /// Longest leg worth offering, years at home.
    pub max_leg_years: f64,
    /// Time to refuel at a stop, days (skimming, cracking ice or buying).
    pub refuel_days: f64,
    /// Kept labels per system (fuel states that are not beaten).
    pub labels_per_system: usize,
    /// Labels taken from the queue before the search gives up.
    pub max_steps: usize,
    /// Least odds of fuel at which a gamble is worth planning.
    #[serde(default = "half")]
    pub gamble_min_odds: f64,
    /// Odds of a gas giant before any survey, from the star (Johnson et al.
    /// 2010): base × (M/M☉)^mass_exp × 10^(feh_exp [Fe/H]), at most `cap`.
    #[serde(default)]
    pub giant_prior: GiantPrior,
    /// In-system voyages (part 3b).
    #[serde(default)]
    pub in_system: crate::insystem::InSystemSettings,
}
fn half() -> f64 {
    0.5
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GiantPrior {
    pub base: f64,
    pub mass_exp: f64,
    pub feh_exp: f64,
    pub cap: f64,
}

impl Default for GiantPrior {
    fn default() -> Self {
        GiantPrior {
            base: 0.07,
            mass_exp: 1.0,
            feh_exp: 1.2,
            cap: 0.9,
        }
    }
}

impl GiantPrior {
    /// Odds that a star of `mass_msun` and `feh` has a gas giant.
    pub fn odds(&self, mass_msun: f64, feh: f64) -> f64 {
        (self.base * m::pow(mass_msun.max(0.0), self.mass_exp) * m::pow(10.0, self.feh_exp * feh))
            .clamp(0.0, self.cap)
    }
}

impl Default for VoyageSettings {
    fn default() -> Self {
        VoyageSettings {
            max_flight_pc: 6.0,
            max_leg_years: 20_000.0,
            refuel_days: 2.0,
            labels_per_system: 6,
            max_steps: 200_000,
            gamble_min_odds: 0.5,
            giant_prior: GiantPrior::default(),
            in_system: crate::insystem::InSystemSettings::default(),
        }
    }
}

/// What the voyage is for.
#[derive(Clone, PartialEq, Debug)]
pub enum Goal {
    /// Reach this system.
    To(String),
    /// The nearest system not in `visited`; ties go to the shorter path,
    /// then to the one farther from `home`, so an explorer spirals out.
    NearestUnvisited {
        visited: Vec<String>,
        home: [f64; 3],
    },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Options {
    pub minimise: Minimise,
    pub gambles: bool,
    pub settings: VoyageSettings,
}

/// One flight there for real.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Flight {
    pub home_s: f64,
    pub ship_s: f64,
    pub propellant_t: f64,
    /// Peak speed as a fraction of c.
    pub peak_beta: f64,
    /// Coasted in the middle because the propellant could not reach turnover.
    pub coast: bool,
    pub accel_g: f64,
}

/// Why a flight cannot be made.
#[derive(Clone, PartialEq, Debug)]
pub enum NoFlight {
    NotSustained,
    NoThrust,
    AtReserve,
    TooLong { years: f64 },
}

/// A flight of `d_pc` from rest to rest with `prop_t` of propellant aboard:
/// constant proper acceleration (capped at the crew's limit), turnover at
/// the midpoint, or a coast at the best speed the propellant affords.
///
/// Rapidity at turnover w = acosh(1 + a d / 2c²); proper time 2(c/a)w; home
/// time 2(c/a) sinh w. The relativistic rocket: the rapidity change of a
/// burn is (v_e/c) ln(m0/m1), so speeding up and braking to w needs the
/// mass ratio exp(2 w c / v_e).
pub fn flight(ship: &Ship, prop_t: f64, d_pc: f64) -> Result<Flight, NoFlight> {
    if !ship.sustained {
        return Err(NoFlight::NotSustained);
    }
    let g = ship.accel_g.min(ship.g_limit);
    let a = g * G0;
    let ve = ship.exhaust_km_s * 1000.0;
    if a <= 0.0 || ve <= 0.0 || ship.dry_t <= 0.0 {
        return Err(NoFlight::NoThrust);
    }
    let full = (ship.wet_t - ship.dry_t).max(0.0);
    let floor = ship.dry_t + ship.reserve.clamp(0.0, 1.0) * full;
    let m0 = ship.dry_t + prop_t.max(0.0);
    if m0 <= floor * (1.0 + 1e-12) {
        return Err(NoFlight::AtReserve);
    }
    let d = d_pc * PC_M;
    let x = a * d / (2.0 * C * C);
    // acosh(1 + x), accurate for small x.
    let w_flip = m::log1p(x + m::sqrt(x * (2.0 + x)));
    let w_max = 0.5 * (ve / C) * m::ln(m0 / floor);
    let ca = C / a;
    let f = if w_flip <= w_max {
        let ratio = m::exp(2.0 * w_flip * C / ve);
        Flight {
            home_s: 2.0 * ca * m::sinh(w_flip),
            ship_s: 2.0 * ca * w_flip,
            propellant_t: m0 - m0 / ratio,
            peak_beta: m::tanh(w_flip),
            coast: false,
            accel_g: g,
        }
    } else {
        let w = w_max;
        // Distance covered by each burn, (c²/a)(cosh w - 1), written
        // 2 sinh²(w/2) so it stays exact for small w.
        let sh = m::sinh(0.5 * w);
        let s = C * ca * 2.0 * sh * sh;
        let v = C * m::tanh(w);
        let coast = ((d - 2.0 * s) / v).max(0.0);
        Flight {
            home_s: 2.0 * ca * m::sinh(w) + coast,
            ship_s: 2.0 * ca * w + coast / m::cosh(w),
            propellant_t: m0 - floor,
            peak_beta: m::tanh(w),
            coast: true,
            accel_g: g,
        }
    };
    Ok(f)
}

/// One leg of a voyage.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Leg {
    pub from: String,
    pub to: String,
    /// "jump" or "flight".
    pub means: String,
    pub distance_pc: f64,
    /// Parsecs counted by the jump rule (whole parsecs when it says so).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counted_pc: Option<f64>,
    pub home_days: f64,
    pub ship_days: f64,
    pub jump_fuel_t: f64,
    pub propellant_t: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_beta: Option<f64>,
    pub coast: bool,
    /// A pack's FTL rule, not physics (shown as an honesty mark on the GM view).
    pub fictional: bool,
    /// Arrives hoping for fuel in a system nobody has surveyed.
    pub gamble: bool,
    /// The odds that hope rests on, and where they came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odds: Option<f64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub odds_from: String,
    /// Refuelled on arrival, and from what.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refuel: Option<String>,
    /// Left in the tanks on arrival (after any refuel).
    pub jump_fuel_left_t: f64,
    pub propellant_left_t: f64,
    /// In-system legs: the NAV course taken (hohmann, lambert, torch, belt).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course: Option<String>,
    /// In-system legs: days waited for the departure window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_days: Option<f64>,
    /// The sentence the screens show.
    pub words: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Voyage {
    pub from: String,
    pub to: String,
    pub minimise: Minimise,
    pub legs: Vec<Leg>,
    pub distance_pc: f64,
    pub home_days: f64,
    pub ship_days: f64,
    pub jump_fuel_t: f64,
    pub propellant_t: f64,
    pub refuels: usize,
    pub gambles: usize,
    /// The chance every gamble pays off (1 when there are none).
    pub odds: f64,
    pub fictional: bool,
    /// The longest jump the plan makes, as the rating it needs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Key(f64, f64, u64);
impl Eq for Key {}
impl Ord for Key {
    // Reversed so the max-heap pops the cheapest first.
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0)
            .then(o.1.total_cmp(&self.1))
            .then(o.2.cmp(&self.2))
    }
}
impl PartialOrd for Key {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

#[derive(Clone, Debug)]
struct Label {
    node: usize,
    cost: f64,
    home_s: f64,
    ship_s: f64,
    dist_pc: f64,
    jf: f64,
    pr: f64,
    legs: u32,
    used: f64,
    parent: Option<usize>,
    leg: Option<Leg>,
    dead: bool,
}

pub fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let (x, y, z) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    m::sqrt(x * x + y * y + z * z)
}

fn days(s: f64) -> f64 {
    s / DAY
}

/// Words for a span of days.
pub fn span_words(d: f64) -> String {
    let y = d * DAY / JULIAN_YEAR;
    if y >= 2.0 {
        if y >= 100.0 {
            format!("{} years", group(m::round(y)))
        } else {
            format!("{y:.1} years")
        }
    } else if d >= 2.0 {
        format!("{} days", m::round(d) as i64)
    } else {
        format!("{:.0} hours", d * 24.0)
    }
}

fn group(x: f64) -> String {
    let s = format!("{}", x as i64);
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The cheapest voyage for `ship` from `from` to the goal, or the reasons
/// there is none, each naming what is missing.
pub fn plan(
    ship: &Ship,
    jump: Option<&JumpDrive>,
    systems: &[VoyageSystem],
    from: &str,
    goal: &Goal,
    o: &Options,
) -> Result<Voyage, Vec<String>> {
    let name = if ship.name.is_empty() {
        "the ship".to_string()
    } else {
        format!("the {}", ship.name)
    };
    let flies = ship.sustained && ship.accel_g > 0.0 && ship.exhaust_km_s > 0.0;
    if jump.is_none() && !flies {
        return Err(vec![format!(
            "{name} has no jump drive and no sustained drive, so it cannot cross between stars"
        )]);
    }
    let set = &o.settings;
    let jump_reach = jump.map(|j| j.rating_pc).unwrap_or(0.0);
    let reach = jump_reach.max(if flies { set.max_flight_pc } else { 0.0 });
    let maker = Interstellar {
        ship,
        jump,
        systems,
        grid: Grid::new(systems, reach.max(1e-6)),
        reach,
        flies,
        name: name.clone(),
        set,
    };
    search(ship, jump, systems, from, goal, o, &maker)
}

/// What prices the legs out of a place: between stars (jumps and flights)
/// or inside one system (NAV's courses). The search is the same for both.
pub trait LegMaker {
    /// Legs from place `from`, leaving `t_s` seconds after the voyage
    /// starts with `jf` t of jump fuel and `pr` t of propellant aboard.
    fn legs(&self, from: usize, t_s: f64, jf: f64, pr: f64) -> Vec<(usize, Leg)>;
    /// A cost the rest of the way from `from` to `to` can never beat (an
    /// admissible A* bound); 0 when nothing better is known.
    fn lower_bound(&self, _from: usize, _to: usize, _minimise: Minimise) -> f64 {
        0.0
    }
    /// Why no voyage was found, in words.
    fn refusals(&self, start: usize, target: Option<usize>, visited: &[usize]) -> Vec<String>;
}

struct Interstellar<'a> {
    ship: &'a Ship,
    jump: Option<&'a JumpDrive>,
    systems: &'a [VoyageSystem],
    grid: Grid,
    reach: f64,
    flies: bool,
    name: String,
    set: &'a VoyageSettings,
}

impl LegMaker for Interstellar<'_> {
    fn legs(&self, from: usize, _t_s: f64, jf: f64, pr: f64) -> Vec<(usize, Leg)> {
        let (ship, jump, set, systems) = (self.ship, self.jump, self.set, self.systems);
        let a = &systems[from];
        let mut out = vec![];
        for v in self.grid.near(&a.xyz_pc, self.reach) {
            if v == from {
                continue;
            }
            let b = &systems[v];
            let d = dist(&a.xyz_pc, &b.xyz_pc);
            if let Some(j) = jump {
                let n = j.counted(d);
                let need = j.fuel_t_per_pc * n;
                if n <= j.rating_pc + 1e-9 && need <= jf + 1e-9 {
                    let t = j.days_per_jump;
                    out.push((
                        v,
                        Leg {
                            means: "jump".into(),
                            counted_pc: Some(n),
                            home_days: t,
                            ship_days: t,
                            jump_fuel_t: need,
                            fictional: true,
                            jump_fuel_left_t: jf - need,
                            propellant_left_t: pr,
                            ..Leg::between(&a.id, &b.id, d)
                        },
                    ));
                }
            }
            if self.flies && d <= set.max_flight_pc {
                if let Ok(f) = flight(ship, pr, d) {
                    if f.home_s / JULIAN_YEAR <= set.max_leg_years {
                        out.push((
                            v,
                            Leg {
                                means: "flight".into(),
                                home_days: days(f.home_s),
                                ship_days: days(f.ship_s),
                                propellant_t: f.propellant_t,
                                peak_beta: Some(f.peak_beta),
                                coast: f.coast,
                                jump_fuel_left_t: jf,
                                propellant_left_t: pr - f.propellant_t,
                                ..Leg::between(&a.id, &b.id, d)
                            },
                        ));
                    }
                }
            }
        }
        out
    }
    fn refusals(&self, start: usize, target: Option<usize>, visited: &[usize]) -> Vec<String> {
        refusals(
            self.ship,
            self.jump,
            self.systems,
            start,
            target,
            visited,
            &self.name,
            self.flies,
            self.set,
            &self.grid,
        )
    }
}

impl Leg {
    /// A blank leg between two places, `d` parsecs apart, for a maker to fill.
    pub fn between(from: &str, to: &str, d: f64) -> Leg {
        Leg {
            from: from.to_string(),
            to: to.to_string(),
            means: String::new(),
            distance_pc: d,
            counted_pc: None,
            home_days: 0.0,
            ship_days: 0.0,
            jump_fuel_t: 0.0,
            propellant_t: 0.0,
            peak_beta: None,
            coast: false,
            fictional: false,
            gamble: false,
            odds: None,
            odds_from: String::new(),
            refuel: None,
            jump_fuel_left_t: 0.0,
            propellant_left_t: 0.0,
            course: None,
            wait_days: None,
            words: String::new(),
        }
    }
}

/// The search itself: one graph, any leg-maker.
pub fn search(
    ship: &Ship,
    jump: Option<&JumpDrive>,
    systems: &[VoyageSystem],
    from: &str,
    goal: &Goal,
    o: &Options,
    maker: &dyn LegMaker,
) -> Result<Voyage, Vec<String>> {
    let mut ids: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, s) in systems.iter().enumerate() {
        ids.insert(s.id.as_str(), i);
    }
    let start = *ids
        .get(from)
        .ok_or_else(|| vec![format!("{from} is not among the systems searched")])?;
    let target = match goal {
        Goal::To(t) => Some(
            *ids.get(t.as_str())
                .ok_or_else(|| vec![format!("{t} is not among the systems searched")])?,
        ),
        Goal::NearestUnvisited { .. } => None,
    };
    if target == Some(start) {
        return Err(vec![format!("{from} is where the voyage starts")]);
    }
    let visited: Vec<usize> = match goal {
        Goal::NearestUnvisited { visited, .. } => visited
            .iter()
            .filter_map(|v| ids.get(v.as_str()).copied())
            .collect(),
        Goal::To(_) => vec![],
    };
    let is_goal = |n: usize| match target {
        Some(t) => n == t,
        None => n != start && !visited.contains(&n),
    };

    let set = &o.settings;
    let full_pr = (ship.wet_t - ship.dry_t).max(0.0);
    let full_jf = jump.map(|j| j.tank_t).unwrap_or(0.0);

    let mut labels: Vec<Label> = vec![];
    let mut at: Vec<Vec<usize>> = vec![vec![]; systems.len()];
    let mut heap: BinaryHeap<(Key, usize)> = BinaryHeap::new();
    let mut seq = 0u64;
    let mut push = |labels: &mut Vec<Label>,
                    at: &mut Vec<Vec<usize>>,
                    heap: &mut BinaryHeap<(Key, usize)>,
                    l: Label| {
        let here = &at[l.node];
        // Beaten by a kept label: no worse cost, no less of either fuel.
        let eps = 1e-9;
        if here.iter().any(|&k| {
            let o = &labels[k];
            !o.dead && o.cost <= l.cost + eps && o.jf + eps >= l.jf && o.pr + eps >= l.pr
        }) {
            return;
        }
        for &k in here {
            let o = &labels[k];
            if !o.dead && l.cost <= o.cost + eps && l.jf + eps >= o.jf && l.pr + eps >= o.pr {
                labels[k].dead = true;
            }
        }
        let live = at[l.node].iter().filter(|&&k| !labels[k].dead).count();
        if live >= set.labels_per_system {
            return;
        }
        let id = labels.len();
        let h = target.map_or(0.0, |t| maker.lower_bound(l.node, t, o.minimise));
        heap.push((Key(l.cost + h, l.dist_pc, seq), id));
        seq += 1;
        at[l.node].push(id);
        labels.push(l);
    };

    let cost_of = |home_s: f64, ship_s: f64, fuel: f64, legs: u32| match o.minimise {
        Minimise::HomeTime => home_s,
        Minimise::ShipTime => ship_s,
        Minimise::Fuel => fuel,
        Minimise::Legs => legs as f64,
    };
    let jf0 = jump.map(|j| j.fuel()).unwrap_or(0.0);
    let pr0 = ship.propellant();
    push(
        &mut labels,
        &mut at,
        &mut heap,
        Label {
            node: start,
            cost: 0.0,
            home_s: 0.0,
            ship_s: 0.0,
            dist_pc: 0.0,
            jf: jf0,
            pr: pr0,
            legs: 0,
            used: 0.0,
            parent: None,
            leg: None,
            dead: false,
        },
    );

    let mut found: Vec<usize> = vec![];
    let mut best: Option<f64> = None;
    let mut steps = 0usize;
    while let Some((Key(cost, _, _), id)) = heap.pop() {
        if labels[id].dead {
            continue;
        }
        if let Some(b) = best {
            if cost > b + 1e-9 * b.abs().max(1.0) {
                break;
            }
        }
        steps += 1;
        if steps > set.max_steps {
            break;
        }
        let l = labels[id].clone();
        if is_goal(l.node) && l.parent.is_some() {
            best.get_or_insert(cost);
            found.push(id);
            if target.is_some() {
                break;
            }
            continue;
        }
        let used_here = l.used;
        for (v, leg) in maker.legs(l.node, l.home_s, l.jf, l.pr) {
            let b = &systems[v];
            let d = leg.distance_pc;
            {
                let home = l.home_s + leg.home_days * DAY;
                let shp = l.ship_s + leg.ship_days * DAY;
                let used = used_here + leg.jump_fuel_t + leg.propellant_t;
                let legs = l.legs + 1;
                let arrive = Label {
                    node: v,
                    cost: cost_of(home, shp, used, legs),
                    home_s: home,
                    ship_s: shp,
                    dist_pc: l.dist_pc + d,
                    jf: leg.jump_fuel_left_t,
                    pr: leg.propellant_left_t,
                    legs,
                    used,
                    parent: Some(id),
                    leg: Some(leg.clone()),
                    dead: false,
                };
                // Refuel here when the crew knows of fuel (or gambles on it).
                let known = !b.fuel.is_empty();
                let odds = b.fuel_odds.unwrap_or(0.0);
                let gamble = !b.surveyed
                    && o.gambles
                    && !known
                    && odds > 0.0
                    && odds + 1e-12 >= set.gamble_min_odds;
                let at_goal = is_goal(v);
                if (known || gamble) && !at_goal {
                    let mut r = leg.clone();
                    r.refuel = Some(if known {
                        b.fuel[0].clone()
                    } else {
                        "hoped for".into()
                    });
                    r.gamble = gamble;
                    if gamble {
                        r.odds = Some(odds);
                        r.odds_from = b.odds_from.clone();
                    }
                    r.jump_fuel_left_t = full_jf;
                    r.propellant_left_t = full_pr;
                    let rh = home + set.refuel_days * DAY;
                    let rs = shp + set.refuel_days * DAY;
                    let refuelled = Label {
                        cost: cost_of(rh, rs, used, legs),
                        home_s: rh,
                        ship_s: rs,
                        jf: full_jf,
                        pr: full_pr,
                        leg: Some(r),
                        ..arrive.clone()
                    };
                    push(&mut labels, &mut at, &mut heap, refuelled);
                }
                push(&mut labels, &mut at, &mut heap, arrive);
            }
        }
    }

    if found.is_empty() {
        return Err(maker.refusals(start, target, &visited));
    }
    // Ties: equal cost, then the shorter path, then farther from home, then id.
    let home = match goal {
        Goal::NearestUnvisited { home, .. } => *home,
        Goal::To(_) => systems[start].xyz_pc,
    };
    let pick = *found
        .iter()
        .min_by(|&&x, &&y| {
            let (a, b) = (&labels[x], &labels[y]);
            a.cost
                .total_cmp(&b.cost)
                .then(a.dist_pc.total_cmp(&b.dist_pc))
                .then(
                    dist(&systems[b.node].xyz_pc, &home)
                        .total_cmp(&dist(&systems[a.node].xyz_pc, &home)),
                )
                .then(systems[a.node].id.cmp(&systems[b.node].id))
        })
        .expect("found is not empty");
    let mut chain: Vec<Leg> = vec![];
    let mut k = Some(pick);
    while let Some(i) = k {
        if let Some(leg) = &labels[i].leg {
            chain.push(leg.clone());
        }
        k = labels[i].parent;
    }
    chain.reverse();
    let label = |id: &str| -> String {
        let s = &systems[ids[id]];
        if s.name.is_empty() {
            s.id.clone()
        } else {
            s.name.clone()
        }
    };
    for leg in &mut chain {
        leg.words = leg_words(leg, &label(&leg.from), &label(&leg.to));
    }
    let longest = chain
        .iter()
        .filter_map(|l| l.counted_pc)
        .fold(0.0_f64, f64::max);
    let v = Voyage {
        from: systems[start].id.clone(),
        to: systems[labels[pick].node].id.clone(),
        minimise: o.minimise,
        distance_pc: chain.iter().map(|l| l.distance_pc).sum(),
        home_days: chain.iter().map(|l| l.home_days).sum::<f64>()
            + set.refuel_days * chain.iter().filter(|l| l.refuel.is_some()).count() as f64,
        ship_days: chain.iter().map(|l| l.ship_days).sum::<f64>()
            + set.refuel_days * chain.iter().filter(|l| l.refuel.is_some()).count() as f64,
        jump_fuel_t: chain.iter().map(|l| l.jump_fuel_t).sum(),
        propellant_t: chain.iter().map(|l| l.propellant_t).sum(),
        refuels: chain.iter().filter(|l| l.refuel.is_some()).count(),
        gambles: chain.iter().filter(|l| l.gamble).count(),
        odds: chain.iter().filter_map(|l| l.odds).product(),
        fictional: chain.iter().any(|l| l.fictional),
        needs: (longest > 0.0).then(|| format!("Jump-{}", fmt_pc(longest))),
        legs: chain,
    };
    Ok(v)
}

fn leg_words(l: &Leg, from: &str, to: &str) -> String {
    let mut s = match l.means.as_str() {
        "course" => {
            let kind = match l.course.as_deref() {
                Some("hohmann") => "Hohmann transfer",
                Some("lambert") => "transfer",
                Some("torch") => "torch flight",
                _ => "course",
            };
            let wait = l.wait_days.unwrap_or(0.0);
            let on_way = l.home_days - wait;
            let wait_words = if wait >= 1.0 {
                format!("wait {} for the window, then ", span_words(wait))
            } else {
                String::new()
            };
            format!(
                "{} from {from} to {to}: {wait_words}{} on the way, {:.1} t of propellant",
                kind[..1].to_uppercase() + &kind[1..],
                span_words(on_way),
                l.propellant_t
            )
        }
        "jump" => format!(
            "Jump {} pc from {from} to {to} ({} counted), {}",
            fmt1(l.distance_pc),
            fmt_pc(l.counted_pc.unwrap_or(l.distance_pc)),
            span_words(l.home_days)
        ),
        _ => {
            let peak = l.peak_beta.unwrap_or(0.0);
            let speed = if peak >= 0.01 {
                format!("{:.1}% of light", peak * 100.0)
            } else {
                format!("{} km/s", group(m::round(peak * C / 1000.0)))
            };
            format!(
                "Fly {} pc from {from} to {to}, {} at home and {} aboard, peak {speed}{}",
                fmt1(l.distance_pc),
                span_words(l.home_days),
                span_words(l.ship_days),
                if l.coast { ", coasting" } else { "" }
            )
        }
    };
    if let Some(r) = &l.refuel {
        if l.gamble {
            let from = if l.odds_from.is_empty() {
                String::new()
            } else {
                format!(" from {}", l.odds_from)
            };
            s.push_str(&format!(
                "; refuel hoped for (a gamble: a {:.0}% chance of fuel{from})",
                l.odds.unwrap_or(0.0) * 100.0
            ));
        } else {
            s.push_str(&format!("; refuel ({r})"));
        }
    }
    s
}

fn fmt1(x: f64) -> String {
    format!("{x:.2}")
}

#[allow(clippy::too_many_arguments)]
fn refusals(
    ship: &Ship,
    jump: Option<&JumpDrive>,
    systems: &[VoyageSystem],
    start: usize,
    target: Option<usize>,
    visited: &[usize],
    name: &str,
    flies: bool,
    set: &VoyageSettings,
    grid: &Grid,
) -> Vec<String> {
    let mut out = vec![];
    let from = &systems[start];
    // The shortest whole-parsec reach that would connect (fuel ignored),
    // tried upward from the ship's own; `None` when even 12 pc does not.
    let is_target = |u: usize| match target {
        Some(t) => u == t,
        None => u != start && !visited.contains(&u),
    };
    let connects = |r: f64| -> bool {
        let mut seen = vec![false; systems.len()];
        let mut todo = vec![start];
        seen[start] = true;
        while let Some(u) = todo.pop() {
            if is_target(u) {
                return true;
            }
            for v in grid.near(&systems[u].xyz_pc, r) {
                if !seen[v] {
                    seen[v] = true;
                    todo.push(v);
                }
            }
        }
        false
    };
    let gap = |reach: f64| -> f64 {
        if connects(reach) {
            return reach;
        }
        let mut r = m::floor(reach) + 1.0;
        while r <= 12.0 {
            if connects(r) {
                return r;
            }
            r += 1.0;
        }
        f64::INFINITY
    };
    let reach_words = match (jump, flies) {
        (Some(j), true) => format!(
            "{} or a {} pc flight",
            j.rating_words(),
            fmt_pc(set.max_flight_pc)
        ),
        (Some(j), false) => j.rating_words(),
        (None, _) => format!("a {} pc flight", fmt_pc(set.max_flight_pc)),
    };
    let reach =
        jump.map(|j| j.rating_pc)
            .unwrap_or(0.0)
            .max(if flies { set.max_flight_pc } else { 0.0 });
    {
        let g = gap(reach);
        if g.is_infinite() {
            out.push(match target {
                Some(t) => format!(
                    "{} cannot be reached: no chain of legs shorter than 12 pc joins it",
                    nm(&systems[t])
                ),
                None => format!("nothing unvisited is within 12 pc hops of {}", nm(from)),
            });
            return out;
        }
        if g > reach + 1e-9 {
            let what = match jump {
                Some(j) => format!(
                    "needs Jump-{}, {name} is {}",
                    fmt_pc(j.counted(g)),
                    j.rating_words()
                ),
                None => format!(
                    "needs a {} pc leg, longer than the {} pc a flight is offered",
                    fmt1(g),
                    fmt_pc(set.max_flight_pc)
                ),
            };
            out.push(match target {
                Some(t) => format!("{} cannot be reached: it {what}", nm(&systems[t])),
                None => format!(
                    "nothing unvisited is within {reach_words} of {}: it {what}",
                    nm(from)
                ),
            });
            return out;
        }
    }
    // Reachable on paper, so fuel is what stops it.
    let near = grid.near(&from.xyz_pc, reach);
    let known: Vec<&VoyageSystem> = near
        .iter()
        .map(|&i| &systems[i])
        .filter(|s| s.id != from.id && !s.fuel.is_empty())
        .collect();
    if let Some(j) = jump {
        let one = j.fuel_t_per_pc * j.counted(1.0).min(j.rating_pc);
        if j.fuel() + 1e-9 < one && !flies {
            out.push(format!(
                "{name} has {} t of jump fuel; a jump needs at least {} t",
                fmt1(j.fuel()),
                fmt1(one)
            ));
            return out;
        }
    }
    if flies && jump.is_none() {
        if let Some(t) = target {
            let d = dist(&from.xyz_pc, &systems[t].xyz_pc);
            if d <= set.max_flight_pc {
                match flight(ship, ship.propellant(), d) {
                    Err(NoFlight::AtReserve) => out.push(format!(
                        "{name}'s propellant is down to its reserve, so it cannot set out"
                    )),
                    Ok(f) if f.home_s / JULIAN_YEAR > set.max_leg_years => out.push(format!(
                        "{name} would take {} to fly to {}, longer than the {} year limit",
                        span_words(days(f.home_s)),
                        nm(&systems[t]),
                        group(set.max_leg_years)
                    )),
                    _ => {}
                }
                if !out.is_empty() {
                    return out;
                }
            }
        }
    }
    out.push(if known.is_empty() {
        format!(
            "no known fuel within {reach_words} of {}, and {name} cannot get there on what it carries",
            nm(from)
        )
    } else {
        format!(
            "{name} runs dry on the way: the fuel it knows of ({}) does not bridge the gaps",
            known
                .iter()
                .take(3)
                .map(|s| nm(s))
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    out
}

fn nm(s: &VoyageSystem) -> String {
    if s.name.is_empty() {
        s.id.clone()
    } else {
        s.name.clone()
    }
}

/// Systems bucketed by cube so neighbours within `reach` are found quickly.
struct Grid {
    side: f64,
    cells: BTreeMap<[i64; 3], Vec<usize>>,
    xyz: Vec<[f64; 3]>,
}

impl Grid {
    fn new(systems: &[VoyageSystem], side: f64) -> Self {
        let mut cells: BTreeMap<[i64; 3], Vec<usize>> = BTreeMap::new();
        for (i, s) in systems.iter().enumerate() {
            cells.entry(Self::key(side, &s.xyz_pc)).or_default().push(i);
        }
        Grid {
            side,
            cells,
            xyz: systems.iter().map(|s| s.xyz_pc).collect(),
        }
    }
    fn key(side: f64, p: &[f64; 3]) -> [i64; 3] {
        [
            m::floor(p[0] / side) as i64,
            m::floor(p[1] / side) as i64,
            m::floor(p[2] / side) as i64,
        ]
    }
    /// Systems within `r` of `p`, in index order.
    fn near(&self, p: &[f64; 3], r: f64) -> Vec<usize> {
        let k = Self::key(self.side, p);
        let n = m::ceil(r / self.side) as i64;
        let mut out = vec![];
        for x in -n..=n {
            for y in -n..=n {
                for z in -n..=n {
                    if let Some(v) = self.cells.get(&[k[0] + x, k[1] + y, k[2] + z]) {
                        for &i in v {
                            if dist(&self.xyz[i], p) <= r + 1e-12 {
                                out.push(i);
                            }
                        }
                    }
                }
            }
        }
        out.sort_unstable();
        out
    }
}

#[cfg(test)]
#[path = "voyage_tests.rs"]
mod tests;
