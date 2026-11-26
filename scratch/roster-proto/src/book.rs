//! The books of one class at a moment, and the deeds that change them.
//!
//! Three tiers inside a class, which partition its mass exactly:
//! - exact: the largest `n_exact` members, summed member by member, so the
//!   reservoir, what is in flight and what each sink received are exact at
//!   any moment, and each crater's impactor mass is the very mass its world
//!   gained;
//! - roster: smaller members down to the roster's size, which have names
//!   and fates (the roster) but whose books count members exactly and weigh
//!   them at their band's mean mass, read at the edge of a time bin;
//! - gravel: below the roster, a sum drained by the sketch.

use crate::roster::{member_sub, BandId, Cause, Roster};
use crate::system::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Deed {
    /// Players mine out a named member at a date.
    MineMember {
        band: BandId,
        rank: u64,
        t_myr: f64,
        by: String,
    },
    /// Players mine "some asteroid": a stated mass from a class's gravel,
    /// which becomes a pinned object they can return to.
    MineGravel {
        res: usize,
        class: usize,
        sub: u64,
        t_myr: f64,
        name: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Campaign {
    pub deeds: Vec<Deed>,
}

impl Campaign {
    pub fn mined_at(&self, band: BandId, rank: u64) -> Option<f64> {
        self.deeds.iter().find_map(|d| match d {
            Deed::MineMember {
                band: b,
                rank: r,
                t_myr,
                ..
            } if *b == band && *r == rank => Some(*t_myr),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum State {
    InReservoir,
    InFlight { to: String, lands_myr: f64 },
    Arrived { on: String, at_myr: f64 },
    Mined { at_myr: f64 },
}

pub fn sink_name(sys: &System, k: SinkKind) -> String {
    match k {
        SinkKind::Star => "the star".into(),
        SinkKind::Ejected => "thrown out of the system".into(),
        SinkKind::World(i) => sys.worlds[i].id.clone(),
    }
}

/// Where a member is at `t`, given the campaign's deeds.
pub fn state_at(ro: &mut Roster, cam: &Campaign, band: BandId, rank: u64, t: f64) -> State {
    let sys = ro.sys;
    let fate = ro.fate(band, rank);
    if let Some(td) = cam.mined_at(band, rank) {
        if t >= td {
            return State::Mined { at_myr: td };
        }
    }
    match fate {
        None => State::InReservoir,
        Some(f) if t < f.t_leave => State::InReservoir,
        Some(f) => {
            let to = sink_name(sys, sys.reservoirs[band.res].sinks[f.sink].0);
            if t < f.t_arrive {
                State::InFlight {
                    to,
                    lands_myr: f.t_arrive,
                }
            } else {
                State::Arrived {
                    on: to,
                    at_myr: f.t_arrive,
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ledger {
    pub t_myr: f64,
    pub start: [u64; 6],
    pub reservoir: [i128; 6],
    pub in_flight: [u64; 6],
    pub delivered: BTreeMap<String, [u64; 6]>,
    pub mined: [u64; 6],
    /// Exact-tier mass delivered to each sink (sub-units), for the two-ends check.
    pub exact_delivered: BTreeMap<String, u64>,
    pub tiers_sub: [u64; 3],
    pub exact_members_walked: u64,
}

impl Ledger {
    pub fn closes(&self) -> bool {
        (0..6).all(|g| {
            let d: u64 = self.delivered.values().map(|x| x[g]).sum();
            self.reservoir[g] >= 0
                && self.reservoir[g] as u128
                    + self.in_flight[g] as u128
                    + d as u128
                    + self.mined[g] as u128
                    == self.start[g] as u128
        })
    }
}

fn add(a: &mut [u64; 6], b: &[u64]) {
    for g in 0..6 {
        a[g] += b[g];
    }
}

fn shares(c: &ClassDef) -> Vec<f64> {
    c.groups.iter().map(|x| *x as f64).collect()
}

/// Band mass in closed form (no walk).
pub fn band_sub(c: &ClassDef, k0: u64, k1: u64) -> u64 {
    let w = weight_upto(k1 - 1, c.cloud.n_top) - weight_upto(k0 - 1, c.cloud.n_top);
    sse_core::math::floor(c.total_sub as f64 * w / c.cloud.norm) as u64
}

pub fn ledger(ro: &mut Roster, cam: &Campaign, res: usize, class: usize, t: f64) -> Ledger {
    let sys = ro.sys;
    let r = &sys.reservoirs[res];
    let c = &r.classes[class];
    let sh = shares(c);
    let start: [u64; 6] = std::array::from_fn(|g| c.groups[g] * SUB_PER_UNIT);
    let mut delivered: BTreeMap<String, [u64; 6]> = BTreeMap::new();
    let mut exact_delivered: BTreeMap<String, u64> = BTreeMap::new();
    let mut in_flight = [0u64; 6];
    let mut mined = [0u64; 6];
    let mut walked = 0u64;
    let tb = sys.bin_of(t);
    let t_edge = sys.edges[tb];

    // --- exact tier: walk the members that have left by t (and sum the rest).
    let mut exact_total = 0u64;
    for (bi, (k0, k1)) in c.bands.iter().enumerate() {
        if *k0 > c.n_exact {
            break;
        }
        let k1 = (*k1).min(c.n_exact + 1);
        for k in *k0..k1 {
            exact_total += member_sub(c, k);
        }
        let id = BandId {
            res,
            class,
            band: bi,
        };
        let plan = ro.plan(id);
        // Bins up to t's bin: members leaving later are still in the reservoir.
        for b in 0..=tb {
            if plan.starts[b + 1] == plan.starts[b] {
                continue;
            }
            for (si, (cause, s0, cnt)) in plan.segs(b).iter().enumerate() {
                if *cnt == 0 {
                    continue;
                }
                let counts = ro.sink_counts(id, plan.key, b, si, *cnt);
                let mut p = plan.starts[b] + s0;
                for (sink, n) in counts.iter().enumerate() {
                    for j in 0..*n {
                        let k = plan.k0 + plan.perm.at(p);
                        p += 1;
                        if k >= k1 {
                            continue; // the band's tail beyond the exact tier
                        }
                        walked += 1;
                        let f = ro.timed(id, k, b, *cause, si, sink, j);
                        let sub = member_sub(c, k);
                        let g = split_exact(sub, &sh);
                        if let Some(td) = cam.mined_at(id, k) {
                            if t >= td {
                                add(&mut mined, &g);
                                continue;
                            }
                        }
                        if t < f.t_leave {
                            continue;
                        }
                        if t < f.t_arrive {
                            add(&mut in_flight, &g);
                        } else {
                            let name = sink_name(sys, r.sinks[sink].0);
                            add(delivered.entry(name.clone()).or_default(), &g);
                            *exact_delivered.entry(name).or_default() += sub;
                        }
                    }
                }
            }
        }
        // Mined members that had not yet left by t's bin.
        for d in &cam.deeds {
            if let Deed::MineMember {
                band, rank, t_myr, ..
            } = d
            {
                if *band == id && t >= *t_myr && *rank < k1 {
                    let f = ro.fate(id, *rank);
                    let left_in_walk = f.as_ref().is_some_and(|f| f.bin <= tb);
                    if !left_in_walk {
                        add(&mut mined, &split_exact(member_sub(c, *rank), &sh));
                    }
                }
            }
        }
    }

    // --- roster tier: counts are exact, mass at the band's mean, at t's bin edge.
    let mut roster_total = 0u64;
    for (bi, (k0, k1)) in c.bands.iter().enumerate() {
        let k0 = (*k0).max(c.n_exact + 1);
        if k0 >= *k1 {
            continue;
        }
        let id = BandId {
            res,
            class,
            band: bi,
        };
        let whole = c.bands[bi];
        let bsub = band_sub(c, k0, *k1);
        roster_total += bsub;
        let n = *k1 - k0;
        let mean = bsub / n;
        let plan = ro.plan(id);
        // Share of this band's plan that is in the roster tier (a band can straddle the tiers).
        let frac = n as f64 / (whole.1 - whole.0) as f64;
        let mut per_sink = vec![0u64; r.sinks.len()];
        for b in 0..tb {
            if plan.starts[b + 1] == plan.starts[b] {
                continue;
            }
            for (si, (_, _, cnt)) in plan.segs(b).iter().enumerate() {
                if *cnt == 0 {
                    continue;
                }
                for (s, x) in ro.sink_counts(id, plan.key, b, si, *cnt).iter().enumerate() {
                    per_sink[s] += *x;
                }
            }
        }
        for (s, x) in per_sink.iter().enumerate() {
            let count = if frac < 1.0 {
                sse_core::math::floor(*x as f64 * frac) as u64
            } else {
                *x
            };
            let amt = (count * mean).min(bsub);
            add(
                delivered.entry(sink_name(sys, r.sinks[s].0)).or_default(),
                &split_exact(amt, &sh),
            );
        }
        for d in &cam.deeds {
            if let Deed::MineMember {
                band, rank, t_myr, ..
            } = d
            {
                if *band == id && t >= *t_myr && *rank >= k0 {
                    add(&mut mined, &split_exact(mean, &sh));
                }
            }
        }
    }

    // --- gravel: a sum, drained by the sketch (with the passes' kicks).
    let gravel = c.total_sub - exact_total - roster_total;
    let surv = |tt: f64| {
        let mut s = sys.survival(r, tt);
        if r.kicked {
            for p in sys
                .passes
                .iter()
                .filter(|p| p.t_myr < tt && p.t_myr >= r.t0_myr)
            {
                s *= 1.0 - p.kick;
            }
        }
        s
    };
    let mut left = (gravel as f64 * (1.0 - surv(t_edge))) as u64;
    let mut gmined = 0u64;
    for d in &cam.deeds {
        if let Deed::MineGravel {
            res: dr,
            class: dc,
            sub,
            t_myr,
            ..
        } = d
        {
            if *dr == res && *dc == class && t_edge >= *t_myr {
                let before = (gravel as f64 * (1.0 - surv(*t_myr))) as u64;
                let held = gravel - before - gmined;
                let take = (*sub).min(held);
                gmined += take;
                let after = held - take;
                left = before + (after as f64 * (1.0 - surv(t_edge) / surv(*t_myr))) as u64;
            }
        }
    }
    add(&mut mined, &split_exact(gmined, &sh));
    let sink_sh: Vec<f64> = r.sinks.iter().map(|x| x.1).collect();
    for (s, amt) in split_exact(left, &sink_sh).into_iter().enumerate() {
        add(
            delivered.entry(sink_name(sys, r.sinks[s].0)).or_default(),
            &split_exact(amt, &sh),
        );
    }

    let mut reservoir = [0i128; 6];
    for g in 0..6 {
        let d: u64 = delivered.values().map(|x| x[g]).sum();
        reservoir[g] = start[g] as i128 - d as i128 - in_flight[g] as i128 - mined[g] as i128;
    }
    Ledger {
        t_myr: t,
        start,
        reservoir,
        in_flight,
        delivered,
        mined,
        exact_delivered,
        tiers_sub: [exact_total, roster_total, gravel],
        exact_members_walked: walked,
    }
}

pub fn _unused(_: Cause) {}
