//! ST1, orbital stability (ruling of 23 Nov, Alex "yes to all six"): the
//! card's verdict when a body is added or a system is read. Every check is
//! a formula block in `packs/base-physics/formulas/stability.json`, so the
//! verdict shows its working: each check names its block, the numbers it
//! was given and what came out.
//!
//! - **Hill spacing** for neighbours round one host (Gladman 1993; Pu and
//!   Wu 2015), with the closest approach of their stretched orbits.
//! - **Angular momentum deficit** (Laskar and Petit 2017; Petit, Laskar and
//!   Boué 2017): the family's deficit against each neighbour pair's
//!   critical value. The collision root is found by the Newton step block,
//!   run [`COLLISION_STEPS`] times from e = 0.
//! - **Moon limit** (Domingos, Winter and Yokoyama 2006) for moons of
//!   worlds that circle a star.
//! - **Kozai-Lidov** swings where an outer body at least as heavy, or a
//!   companion star, is tilted more than 39.2 degrees against an orbit: a
//!   note with the largest stretch and the swing time, never a refusal.
//! - **Binaries** (Holman and Wiegert 1999): round one star of a pair, or
//!   round both.
//!
//! Resonance flags and a direct integration on request are later packages.
//! A body missing what a check needs is listed under `unchecked` with the
//! missing facts named, never guessed.

use sse_core::cbor::Value;
use sse_core::Address;
use sse_flow::Atom;
use std::collections::BTreeMap;

type Facts = BTreeMap<Address, BTreeMap<String, Value>>;
/// A block's inputs and outputs, by port.
type Ports = BTreeMap<String, f64>;

/// Newton steps for the collision root (16 reach it to about 1e-15 over
/// 0.01 <= alpha <= 0.99 and 1e-4 <= gamma <= 1e4).
pub const COLLISION_STEPS: usize = 16;

/// One check's outcome.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Check {
    /// `hill`, `amd`, `moon`, `kozai`, `binary_one_star`, `binary_both_stars`.
    pub check: String,
    /// The formula block that did the work.
    pub block: String,
    /// What the bodies circle.
    pub around: String,
    pub bodies: Vec<String>,
    /// 0 lasts, 1 worth watching, 2 will not last, 3 the orbits cross.
    pub level: u8,
    pub says: String,
    /// The block's inputs, by port.
    pub inputs: BTreeMap<String, f64>,
    /// The block's outputs, by port.
    pub results: BTreeMap<String, f64>,
}

/// A check that could not run.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Unchecked {
    pub check: String,
    pub bodies: Vec<String>,
    pub missing: Vec<String>,
}

/// The verdict word for a worst level.
pub fn verdict_word(level: u8) -> &'static str {
    match level {
        0 => "lasts",
        1 => "watch",
        _ => "unstable",
    }
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// Run one formula block by id: its outputs by port, or why it refused.
fn run(id: &str, inputs: &[f64]) -> Result<(Ports, Ports), String> {
    let a = sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .ok_or_else(|| format!("{id} is not loaded"))?;
    let d = a.decl();
    let p: Vec<f64> = d.params.iter().map(|p| p.default).collect();
    let out = a.eval(inputs, &p).map_err(|e| e.0)?;
    let ins = d
        .inputs
        .iter()
        .zip(inputs)
        .map(|(p, v)| (p.name.clone(), *v))
        .collect();
    let outs = d
        .outputs
        .iter()
        .zip(out)
        .map(|(p, v)| (p.name.clone(), v))
        .collect();
    Ok((ins, outs))
}

#[derive(Clone, Debug)]
struct Body {
    address: Address,
    star: bool,
    host: Option<Address>,
    mass: Option<f64>,
    a: Option<f64>,
    e: f64,
    i: f64,
    node: f64,
}

fn bodies(facts: &Facts, system: &Address) -> Vec<Body> {
    facts
        .iter()
        .filter(|(a, _)| system.contains(a))
        .map(|(a, row)| {
            let host = row
                .get("orbit.host")
                .or_else(|| row.get("parent"))
                .and_then(|v| match v {
                    Value::Text(t) => t.parse().ok(),
                    _ => None,
                })
                .or_else(|| a.parent());
            let f = |k: &str| row.get(k).and_then(num);
            Body {
                address: a.clone(),
                star: matches!(row.get("body.kind"), Some(Value::Text(t)) if t == "star"),
                host,
                mass: f("body.mass").filter(|m| *m > 0.0),
                a: f("orbit.semi_major_axis").filter(|x| *x > 0.0),
                e: f("orbit.eccentricity").unwrap_or(0.0),
                i: f("orbit.inclination").unwrap_or(0.0),
                node: f("orbit.ascending_node").unwrap_or(0.0),
            }
        })
        .collect()
}

struct Out {
    checks: Vec<Check>,
    unchecked: Vec<Unchecked>,
}

impl Out {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        check: &str,
        block: &str,
        around: &Address,
        bodies: &[&Address],
        inputs: &[f64],
        says: impl Fn(&BTreeMap<String, f64>) -> (u8, String),
    ) -> Option<BTreeMap<String, f64>> {
        let names: Vec<String> = bodies.iter().map(|a| a.to_string()).collect();
        match run(block, inputs) {
            Ok((ins, outs)) => {
                let (level, text) = says(&outs);
                self.checks.push(Check {
                    check: check.into(),
                    block: block.into(),
                    around: around.to_string(),
                    bodies: names,
                    level,
                    says: text,
                    inputs: ins,
                    results: outs.clone(),
                });
                Some(outs)
            }
            Err(e) => {
                self.unchecked.push(Unchecked {
                    check: check.into(),
                    bodies: names,
                    missing: vec![e],
                });
                None
            }
        }
    }
    fn missing(&mut self, check: &str, bodies: &[&Address], missing: Vec<String>) {
        self.unchecked.push(Unchecked {
            check: check.into(),
            bodies: bodies.iter().map(|a| a.to_string()).collect(),
            missing,
        });
    }
}

fn need(b: &Body, mass: bool, orbit: bool) -> Vec<String> {
    let mut m = vec![];
    if mass && b.mass.is_none() {
        m.push(format!("{} body.mass", b.address));
    }
    if orbit && b.a.is_none() {
        m.push(format!("{} orbit.semi_major_axis", b.address));
    }
    m
}

fn r(x: f64, places: i32) -> f64 {
    let k = sse_core::math::pow(10.0, f64::from(places));
    sse_core::math::round(x * k) / k
}

/// Every stability check for bodies at or under `system`.
pub fn checks(facts: &Facts, system: &Address) -> (Vec<Check>, Vec<Unchecked>) {
    let all = bodies(facts, system);
    let by: BTreeMap<&Address, &Body> = all.iter().map(|b| (&b.address, b)).collect();
    let mut out = Out {
        checks: vec![],
        unchecked: vec![],
    };
    // Children of each host, innermost first.
    let mut kids: BTreeMap<&Address, Vec<&Body>> = BTreeMap::new();
    for b in &all {
        if let (Some(h), Some(_)) = (&b.host, b.a) {
            if by.contains_key(h) {
                kids.entry(h).or_default().push(b);
            }
        }
    }
    for v in kids.values_mut() {
        v.sort_by(|x, y| {
            x.a.unwrap_or(0.0)
                .total_cmp(&y.a.unwrap_or(0.0))
                .then(x.address.cmp(&y.address))
        });
    }
    for (host, list) in &kids {
        let h = by[host];
        let Some(mh) = h.mass else {
            let a: Vec<&Address> = list.iter().map(|b| &b.address).collect();
            out.missing("hill", &a, vec![format!("{host} body.mass")]);
            continue;
        };
        let worlds: Vec<&Body> = list.iter().copied().filter(|b| !b.star).collect();
        // Hill spacing and the deficit, neighbour by neighbour.
        let mut amd = Some(0.0);
        for w in &worlds {
            let m = need(w, true, false);
            if !m.is_empty() {
                out.missing("amd", &[&w.address], m);
                amd = None;
                continue;
            }
            match run(
                "sse.stability.amd_term",
                &[w.a.unwrap_or(0.0), w.e, w.i, w.mass.unwrap_or(0.0), mh],
            ) {
                Ok((_, o)) => amd = amd.map(|s| s + o["amd"]),
                Err(e) => {
                    out.missing("amd", &[&w.address], vec![e]);
                    amd = None;
                }
            }
        }
        for pair in worlds.windows(2) {
            let (p, q) = (pair[0], pair[1]);
            let m: Vec<String> = need(p, true, true)
                .into_iter()
                .chain(need(q, true, true))
                .collect();
            if !m.is_empty() {
                out.missing("hill", &[&p.address, &q.address], m);
                continue;
            }
            let (a1, m1, a2, m2) = (
                p.a.unwrap_or(0.0),
                p.mass.unwrap_or(0.0),
                q.a.unwrap_or(0.0),
                q.mass.unwrap_or(0.0),
            );
            if a2 <= a1 {
                out.missing(
                    "hill",
                    &[&p.address, &q.address],
                    vec![
                        "two orbits of the same size: give one a different semi-major axis".into(),
                    ],
                );
                continue;
            }
            out.push(
                "hill",
                "sse.stability.hill_spacing",
                host,
                &[&p.address, &q.address],
                &[a1, p.e, m1, a2, q.e, m2, mh],
                |o| {
                    let (v, s, g) = (o["verdict"], r(o["spacing"], 1), r(o["closest_gap"], 1));
                    if v >= 3.0 {
                        (3, format!("{s} shared gravity bubbles apart, and the orbits cross: they will meet"))
                    } else if v >= 2.0 {
                        (2, format!("{s} shared gravity bubbles apart: under 2√3, too close to last"))
                    } else if v >= 1.0 {
                        (1, format!("{s} shared gravity bubbles apart (closest {g}): may scatter over long times"))
                    } else {
                        (0, format!("{s} shared gravity bubbles apart (closest {g}): lasts"))
                    }
                },
            );
            let Some(total) = amd else { continue };
            let mut e = 0.0;
            let mut ok = true;
            for _ in 0..COLLISION_STEPS {
                match run("sse.stability.amd_collision_step", &[e, a1, a2, m1, m2]) {
                    Ok((_, o)) => e = o["eccentricity_better"],
                    Err(x) => {
                        out.missing("amd", &[&p.address, &q.address], vec![x]);
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
            out.push(
                "amd",
                "sse.stability.amd_critical",
                host,
                &[&p.address, &q.address],
                &[e, a1, a2, m1, m2, mh, total],
                |o| {
                    let (c, k) = (o["relative_amd"], o["critical_amd"]);
                    if k <= 0.0 {
                        return (1, "so close that even round orbits sit where their resonances overlap: they could collide in time".into());
                    }
                    let beta = r(c / k, 3);
                    if o["verdict"] >= 1.0 {
                        (1, format!("the family's spare stretch is {beta} times what lets these two collide: they could, in time"))
                    } else {
                        (0, format!("the family's spare stretch is {beta} of what lets these two collide: they never can"))
                    }
                },
            );
        }
        // Moons of a world that circles a star.
        if !h.star {
            if let Some(star) = h.host.as_ref().and_then(|s| by.get(s)).filter(|s| s.star) {
                for moon in &worlds {
                    let m: Vec<String> = need(h, true, true)
                        .into_iter()
                        .chain(need(star, true, false))
                        .chain(need(moon, false, true))
                        .collect();
                    if !m.is_empty() {
                        out.missing("moon", &[&moon.address], m);
                        continue;
                    }
                    out.push(
                        "moon",
                        "sse.stability.moon_limit",
                        host,
                        &[&moon.address],
                        &[
                            h.a.unwrap_or(0.0),
                            h.e,
                            h.mass.unwrap_or(0.0),
                            star.mass.unwrap_or(0.0),
                            moon.a.unwrap_or(0.0),
                            moon.e,
                            moon.i,
                        ],
                        |o| {
                            let lim = r(o["limit"] / 1e3, 0);
                            if o["verdict"] >= 2.0 {
                                (2, format!("outside the widest lasting moon orbit ({lim} km): the star pulls it away"))
                            } else {
                                (0, format!("inside the widest lasting moon orbit ({lim} km): stays"))
                            }
                        },
                    );
                }
            }
        }
        // Kozai-Lidov: an outer body at least as heavy, or a companion star.
        for w in &worlds {
            let (Some(aw), Some(mw)) = (w.a, w.mass) else {
                continue;
            };
            for o in list.iter().skip_while(|b| b.address != w.address).skip(1) {
                let (Some(ao), Some(mo)) = (o.a, o.mass) else {
                    continue;
                };
                if ao <= aw || (!o.star && mo < mw) {
                    continue;
                }
                let pin = run("sse.orbit.period", &[aw, mh + mw]).map(|(_, x)| x["period"]);
                let pout = run("sse.orbit.period", &[ao, mh + mo]).map(|(_, x)| x["period"]);
                let (Ok(pin), Ok(pout)) = (pin, pout) else {
                    continue;
                };
                let c = sse_core::math::cos(w.i) * sse_core::math::cos(o.i)
                    + sse_core::math::sin(w.i)
                        * sse_core::math::sin(o.i)
                        * sse_core::math::cos(w.node - o.node);
                if c * c >= 0.6 {
                    continue;
                }
                out.push(
                    "kozai",
                    "sse.stability.kozai",
                    host,
                    &[&w.address, &o.address],
                    &[w.i, w.node, o.i, o.node, pin, pout, o.e, mh + mw, mo],
                    |x| {
                        let yr = sse_core::sheet::YR_JULIAN;
                        (
                            1,
                            format!(
                                "tilted {}° against the outer orbit: its stretch swings up to {} every {} years or so",
                                r(x["mutual_inclination"] * 180.0 / sse_core::math::PI, 1),
                                r(x["largest_eccentricity"], 2),
                                r(x["cycle_time"] / yr, 0)
                            ),
                        )
                    },
                );
            }
        }
        // Binaries: a companion star circling this star.
        if h.star {
            for c in list.iter().filter(|b| b.star) {
                let (Some(ab), Some(mc)) = (c.a, c.mass) else {
                    out.missing("binary_one_star", &[&c.address], need(c, true, true));
                    continue;
                };
                for w in &worlds {
                    let Some(ap) = w.a else { continue };
                    if ap < ab {
                        out.push(
                            "binary_one_star",
                            "sse.stability.binary_limit_one_star",
                            host,
                            &[&w.address, &c.address],
                            &[ab, c.e, mh, mc, ap],
                            hw_one,
                        );
                    } else {
                        out.push(
                            "binary_both_stars",
                            "sse.stability.binary_limit_both_stars",
                            host,
                            &[&w.address, &c.address],
                            &[ab, c.e, mh, mc, ap],
                            hw_both,
                        );
                    }
                }
                for w in kids
                    .get(&c.address)
                    .into_iter()
                    .flatten()
                    .filter(|b| !b.star)
                {
                    let Some(ap) = w.a else { continue };
                    out.push(
                        "binary_one_star",
                        "sse.stability.binary_limit_one_star",
                        &c.address,
                        &[&w.address, host],
                        &[ab, c.e, mc, mh, ap],
                        hw_one,
                    );
                }
            }
        }
    }
    (out.checks, out.unchecked)
}

fn hw_one(o: &BTreeMap<String, f64>) -> (u8, String) {
    let lim = r(o["limit"] / sse_core::sheet::AU, 3);
    if o["verdict"] >= 2.0 {
        (2, format!("outside the widest lasting orbit round its star ({lim} au): the other star throws it out"))
    } else {
        (
            0,
            format!("inside the widest lasting orbit round its star ({lim} au): lasts"),
        )
    }
}

fn hw_both(o: &BTreeMap<String, f64>) -> (u8, String) {
    let lim = r(o["limit"] / sse_core::sheet::AU, 3);
    if o["verdict"] >= 2.0 {
        (2, format!("inside the nearest lasting orbit round both stars ({lim} au): the pair throws it out"))
    } else {
        (
            0,
            format!("outside the nearest lasting orbit round both stars ({lim} au): lasts"),
        )
    }
}

/// The checks that concern `focus` (added bodies): those naming one, and
/// the deficit of any family one joined.
pub fn concerning(checks: Vec<Check>, focus: &[String], hosts: &[String]) -> Vec<Check> {
    if focus.is_empty() {
        return checks;
    }
    checks
        .into_iter()
        .filter(|c| {
            c.bodies.iter().any(|b| focus.contains(b))
                || (c.check == "amd" && hosts.contains(&c.around))
        })
        .collect()
}
