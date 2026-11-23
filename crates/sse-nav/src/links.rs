//! Links between places (SSE3's "space highways"): a standing connection a
//! ship travels along, such as a stargate pair, a charted trade lane or a
//! patrol line. Alex, 23 Nov 2026: two-way by default with a one-way option;
//! either end may be any place an address names, a system or something in
//! it down to a surface site, with the FTL rules' start and end conditions
//! as options; a link with no rule of its own is a charted lane the ship
//! flies with its own drive.
//!
//! A link is campaign data (the record's `link` subject), never physics.
//! The voyage finder reads it in two ways:
//! - **a link with a rule** (a gate pair, a hyperspace corridor) makes its
//!   own leg, timed by the rule's `duration` from the real distance and the
//!   ship's rating, and marked fictional when the rule is;
//! - **a charted lane** (no rule) marks the ship's own jumps and flights
//!   that run along it, so a voyage can be held to the lanes.
//!
//! Dates and who knows of a link are the record's: the caller passes only
//! the links in force at the voyage's start that the crew knows.

use crate::ftl::{FtlRule, Place};
use crate::voyage::{dist, Leg, VoyageSystem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One end of a link.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LinkEnd {
    /// An address: a system, or a body, construct or surface site in it.
    pub at: String,
    /// Where in that place a leg may begin or end (the FTL rules' places); empty: anywhere.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub place: Vec<Place>,
}

/// A standing link between two places.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Link {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// A plain label ("gate", "trade lane", "courier route", "patrol line"); never read as a rule.
    #[serde(default)]
    pub kind: String,
    pub a: LinkEnd,
    pub b: LinkEnd,
    /// Only from `a` to `b`.
    #[serde(default)]
    pub one_way: bool,
    /// The link's own way of travel; none: a charted lane, flown with the ship's own drive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<FtlRule>,
    /// Counted measures the setting gives this link ("traveller.jumps": 2), beside the real distance (MP16).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub counted: BTreeMap<String, f64>,
}

/// The links a voyage may use, and whether it must keep to them.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Links {
    #[serde(default)]
    pub links: Vec<Link>,
    /// Every leg must run along a link (a setting where travel needs a gate or a lane).
    #[serde(default)]
    pub only: bool,
}

/// The system an address lies in: its own id, or the id it starts with.
pub fn system_of(systems: &[VoyageSystem], at: &str) -> Option<usize> {
    systems.iter().position(|s| {
        at == s.id
            || (at.len() > s.id.len()
                && at.starts_with(s.id.as_str())
                && at.as_bytes()[s.id.len()] == b'/')
    })
}

/// A link's ends as systems, in the directions it may be travelled.
fn ways(systems: &[VoyageSystem], l: &Link) -> Vec<(usize, usize)> {
    match (system_of(systems, &l.a.at), system_of(systems, &l.b.at)) {
        (Some(a), Some(b)) if a != b => {
            if l.one_way {
                vec![(a, b)]
            } else {
                vec![(a, b), (b, a)]
            }
        }
        _ => vec![],
    }
}

impl Links {
    /// The link, if any, running from system `from` to system `to`.
    pub fn between(&self, systems: &[VoyageSystem], from: usize, to: usize) -> Option<&Link> {
        self.links
            .iter()
            .find(|l| ways(systems, l).contains(&(from, to)))
    }

    /// Legs out of `from` made by links with a rule of their own. `rating` is the ship's drive rating (1 without one).
    pub fn ruled_legs(
        &self,
        systems: &[VoyageSystem],
        from: usize,
        rating: f64,
        jf: f64,
        pr: f64,
    ) -> Vec<(usize, Leg)> {
        let mut out = vec![];
        for l in &self.links {
            let Some(rule) = &l.rule else { continue };
            for (a, b) in ways(systems, l) {
                if a != from {
                    continue;
                }
                let (sa, sb) = (&systems[a], &systems[b]);
                let d = dist(&sa.xyz_pc, &sb.xyz_pc);
                let home = rule.duration.home_days(d, rating);
                let name = if l.name.is_empty() {
                    l.id.clone()
                } else {
                    l.name.clone()
                };
                out.push((
                    b,
                    Leg {
                        means: "link".into(),
                        home_days: home,
                        ship_days: home * rule.aboard_per_home,
                        fictional: rule.fictional,
                        jump_fuel_left_t: jf,
                        propellant_left_t: pr,
                        link: Some(l.id.clone()),
                        words: format!("along {name} ({}), {}", rule.name, counted_words(l, d)),
                        ..Leg::between(&sa.id, &sb.id, d)
                    },
                ));
            }
        }
        out
    }

    /// Mark the ship's own legs that run along a charted lane, and keep only
    /// legs on links when the voyage must keep to them.
    pub fn mark(
        &self,
        systems: &[VoyageSystem],
        from: usize,
        legs: Vec<(usize, Leg)>,
    ) -> Vec<(usize, Leg)> {
        legs.into_iter()
            .filter_map(|(to, mut leg)| {
                if leg.link.is_none() {
                    if let Some(l) = self.between(systems, from, to).filter(|l| l.rule.is_none()) {
                        leg.link = Some(l.id.clone());
                    }
                }
                (!self.only || leg.link.is_some()).then_some((to, leg))
            })
            .collect()
    }
}

/// "2 jumps · 7.3 pc measured": the setting's counts beside the real distance.
pub fn counted_words(l: &Link, d_pc: f64) -> String {
    let mut s: Vec<String> = l
        .counted
        .iter()
        .map(|(k, v)| {
            let word = k.rsplit('.').next().unwrap_or(k);
            format!("{v} {word}")
        })
        .collect();
    s.push(format!("{d_pc:.1} pc measured"));
    s.join(" · ")
}
