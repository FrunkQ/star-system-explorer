//! Trojans for any world round any star (package CO1): which worlds can hold
//! bodies at the points 60 degrees ahead (L4) and behind (L5), how wide the
//! swarm is, how much of a stirred belt each world catches, and how much of
//! the catch stays for billions of years.
//!
//! Nothing here asks which world it is or what kind. The physics is the
//! `sse.coorbital.*` Equations in `packs/base-physics/formulas/coorbital.json`
//! (written by `scripts/build-coorbital-blocks.py`, every number cited there);
//! this file only finds each block's inputs. Simplifications are in
//! `docs/fudges.md` (rows CO-*).

use crate::violent::World;
use sse_core::math as m;

/// The only resonances that count as "simple": order (p - q) at most this.
pub const MAX_ORDER: u32 = 3;
/// Largest inner number searched. Past it, first-order resonances are closer
/// together than the block's "near", so any pair that close is near one.
pub const MAX_Q: u32 = 12;

fn run(id: &str, inputs: &[f64]) -> Vec<f64> {
    // The block door (ER2): the block with the settings in force.
    sse_physics::run(id, inputs).unwrap_or_else(|e| panic!("{id}: {e}"))
}

/// A world's Trojan zone (`sse.coorbital.trojan_zone`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Zone {
    /// The world's share of the pair's mass.
    pub mu: f64,
    /// The points 60 degrees ahead and behind hold things.
    pub stable: bool,
    /// Half-width of the swarm in orbit size, as a share of the world's.
    pub width: f64,
}

/// Masses in any one unit (only their ratio enters).
pub fn zone(planet_mass: f64, host_mass: f64) -> Zone {
    let o = run("sse.coorbital.trojan_zone", &[planet_mass, host_mass]);
    Zone {
        mu: o[0],
        stable: o[1] > 0.5,
        width: o[2],
    }
}

/// Share of a stirred belt caught (`sse.coorbital.capture_share`).
pub fn capture_share(width: f64, giants_between: u32) -> f64 {
    run(
        "sse.coorbital.capture_share",
        &[width, f64::from(giants_between)],
    )[0]
}

/// Share of the catch still there after billions of years
/// (`sse.coorbital.long_lived_share`).
pub fn long_lived_share(mu: f64, disturbed: bool) -> f64 {
    run(
        "sse.coorbital.long_lived_share",
        &[mu, if disturbed { 1.0 } else { 0.0 }],
    )[0]
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// The simple resonance (order at most `MAX_ORDER`) nearest a period ratio
/// of at least 1: (p, q, offset).
pub fn nearest_resonance(ratio: f64) -> (u32, u32, f64) {
    let mut best = (1, 1, f64::INFINITY);
    for q in 1..=MAX_Q {
        for k in 1..=MAX_ORDER {
            let p = q + k;
            // 6:4 is 3:2, already seen.
            if gcd(p, q) > 1 {
                continue;
            }
            let off = (ratio * f64::from(q) / f64::from(p) - 1.0).abs();
            if off < best.2 {
                best = (p, q, off);
            }
        }
    }
    best
}

/// What the rules read of a world: its id, orbit size and mass.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Orbit<'a> {
    pub id: &'a str,
    pub a_au: f64,
    /// In any one unit (only ratios enter).
    pub mass: f64,
    pub giant: bool,
}

impl<'a> From<&'a World> for Orbit<'a> {
    fn from(w: &'a World) -> Self {
        Orbit {
            id: &w.id,
            a_au: w.a_au,
            mass: w.mass_earth(),
            giant: w.is_giant(),
        }
    }
}

/// Why a neighbour shakes a world's Trojans loose
/// (`sse.coorbital.neighbour_disturbs`).
#[derive(Clone, PartialEq, Debug)]
pub struct Disturbance {
    pub by: String,
    pub p: u32,
    pub q: u32,
    pub offset: f64,
    /// The neighbour's mass over the world's.
    pub mass_ratio: f64,
}

/// Does the adjacent world `n` disturb `w`'s swarm?
pub fn disturbance(w: &Orbit, n: &Orbit) -> Option<Disturbance> {
    let (inner, outer) = if w.a_au <= n.a_au { (w, n) } else { (n, w) };
    let ratio = m::pow(outer.a_au / inner.a_au, 1.5);
    let (p, q, _) = nearest_resonance(ratio);
    let o = run(
        "sse.coorbital.neighbour_disturbs",
        &[ratio, f64::from(p), f64::from(q), n.mass, w.mass],
    );
    (o[1] > 0.5).then(|| Disturbance {
        by: n.id.to_string(),
        p,
        q,
        offset: o[0],
        mass_ratio: n.mass / w.mass,
    })
}

/// The first of a world's two neighbours (worlds sorted by orbit) that
/// disturbs its swarm.
pub fn disturbed(worlds: &[Orbit], i: usize) -> Option<Disturbance> {
    let mut near = vec![];
    if i > 0 {
        near.push(i - 1);
    }
    if i + 1 < worlds.len() {
        near.push(i + 1);
    }
    near.into_iter()
        .find_map(|j| disturbance(&worlds[i], &worlds[j]))
}

/// Giant worlds strictly between world `i` and a belt at `belt_a` AU.
pub fn giants_between(worlds: &[Orbit], i: usize, belt_a: f64) -> u32 {
    let a = worlds[i].a_au;
    let (lo, hi) = if a <= belt_a {
        (a, belt_a)
    } else {
        (belt_a, a)
    };
    worlds
        .iter()
        .enumerate()
        .filter(|(j, w)| *j != i && w.giant && w.a_au > lo && w.a_au < hi)
        .count() as u32
}
