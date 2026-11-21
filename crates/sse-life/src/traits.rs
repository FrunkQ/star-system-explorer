//! E5, functional traits only (evolution design section 6, cut to what E6
//! needs: no lineage art, no body plans).
//!
//! A guild is a strategy in one habitat with up to eight trait values. Each
//! trait has an optimum set by the habitat's conditions and limits set by
//! physics. The value relaxes towards the optimum,
//! `v(t + dt) = opt + (v(t) - opt) exp(-dt / tau)`, with `tau = tau0 / opportunity`.
//! The gap is the guild's **strain**, in units of each trait's tolerance.
//! Strain lowers the guild's share of production (`exp(-s^2 / 2)`).
//! - Past `lost_strain` the guild is lost, an extinction with a physical
//!   cause.
//! - If strain climbs past `rescue_from` and comes back under 1, that is
//!   evolutionary rescue.
//!
//! "Seed everything, let the environment choose" (section 6.4): candidates
//! are keyed draws inside the limits (`life.guild.v1`, keyed by habitat and
//! candidate number), so adding a habitat or a candidate never moves another's
//! draw. Each guild runs on its own, so the order guilds are listed in
//! changes nothing.
//!
//! Branching (section 6.3) and the lineage tree come later, with E5's full
//! form.

use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use std::collections::BTreeMap;

/// A habitat's conditions, as far as traits are concerned.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Conditions {
    /// Mean temperature and its typical day or season swing, K.
    pub temperature_k: f64,
    pub swing_k: f64,
    /// Water balance, -1 (dry: evaporation wins) to 1 (wet).
    pub water_balance: f64,
    /// Surface gravity, m s-2.
    pub gravity_m_s2: f64,
    /// Nutrient supply relative to today's Earth (E4's `nutrient_rel`).
    pub nutrient_rel: f64,
    /// Disturbance (fire, frost, flood) relative to today's Earth.
    pub disturbance_rel: f64,
    /// Production density relative to today's Earth land, the "opportunity"
    /// that sets how fast traits move.
    pub opportunity: f64,
}

/// One trait's rule (`life_traits.traits[]`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TraitDef {
    pub id: String,
    /// What the optimum follows (one of the built-in optima below).
    pub optimum: String,
    /// The gap that counts as one unit of strain.
    pub tolerance: f64,
    /// Relaxation time at opportunity 1, Myr.
    pub tau0_myr: f64,
    pub explain: String,
}

/// Settings (`life_traits`). Defaults are the starting set of section 6.1.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TraitParams {
    pub traits: Vec<TraitDef>,
    /// Candidate strategies drawn per habitat.
    pub candidates: u32,
    pub lost_strain: f64,
    pub rescue_from: f64,
    /// Life's temperature limits, K (about -20 C to 122 C: Takai et al. 2008
    /// for the upper limit; fudge F32 for the lower).
    pub t_min_k: f64,
    pub t_max_k: f64,
    /// Tallest a tree can grow at Earth's gravity, m: the hydraulic limit of
    /// Koch et al. 2004. It scales as 1/g (the water column's weight).
    pub stature_limit_earth_m: f64,
    pub sources: Vec<String>,
}

fn def(id: &str, optimum: &str, tolerance: f64, tau0: f64, explain: &str) -> TraitDef {
    TraitDef {
        id: id.into(),
        optimum: optimum.into(),
        tolerance,
        tau0_myr: tau0,
        explain: explain.into(),
    }
}

impl Default for TraitParams {
    fn default() -> Self {
        TraitParams {
            traits: vec![
                def("temperature_optimum_k", "temperature", 10.0, 0.1, "matched to the habitat's mean temperature"),
                def("temperature_breadth_k", "swing", 10.0, 0.1, "wider where the habitat swings more"),
                def("water_strategy", "water", 0.25, 0.1, "0 saves water, 1 spends it: follows the water balance"),
                def("tissue_economics", "nutrients", 0.25, 0.1, "0 slow and tough, 1 fast and cheap (Wright et al. 2004): fast where nutrients are rich"),
                def("stature_m", "stature", 0.2, 0.5, "as tall as light competition pays for, up to the hydraulic limit and the water there is (tolerance is a share of the limit)"),
                def("dormancy", "variability", 0.25, 0.1, "rewarded by a variable habitat"),
                def("disturbance_strategy", "disturbance", 0.25, 0.1, "0 resist, 1 reseed: reseeding pays where fire, frost or flood come often"),
            ],
            candidates: 16,
            lost_strain: 3.0,
            rescue_from: 2.0,
            t_min_k: 253.0,
            t_max_k: 395.0,
            stature_limit_earth_m: 130.0,
            sources: vec![],
        }
    }
}

/// Earth's standard gravity, m s-2 (the stature limit's reference).
const G_EARTH: f64 = 9.806_65;

impl TraitParams {
    /// A trait's limits in these conditions.
    pub fn limits(&self, t: &TraitDef, c: &Conditions) -> (f64, f64) {
        match t.optimum.as_str() {
            "temperature" => (self.t_min_k, self.t_max_k),
            "swing" => (1.0, 60.0),
            "stature" => (0.0, self.stature_limit(c)),
            _ => (0.0, 1.0),
        }
    }

    pub fn stature_limit(&self, c: &Conditions) -> f64 {
        if c.gravity_m_s2 <= 0.0 {
            self.stature_limit_earth_m
        } else {
            self.stature_limit_earth_m * G_EARTH / c.gravity_m_s2
        }
    }

    /// The optimum, inside the limits.
    pub fn optimum(&self, t: &TraitDef, c: &Conditions) -> f64 {
        let raw = match t.optimum.as_str() {
            "temperature" => c.temperature_k,
            "swing" => c.swing_k,
            "water" => 0.5 * (c.water_balance + 1.0),
            "nutrients" => c.nutrient_rel / (c.nutrient_rel + 1.0),
            "stature" => self.stature_limit(c) * (0.5 * (c.water_balance + 1.0)).clamp(0.0, 1.0),
            "variability" => c.swing_k / (c.swing_k + 20.0),
            "disturbance" => c.disturbance_rel / (c.disturbance_rel + 1.0),
            _ => 0.5,
        };
        let (lo, hi) = self.limits(t, c);
        raw.clamp(lo, hi)
    }

    /// The gap that is one unit of strain (stature's tolerance is a share
    /// of its limit).
    fn tolerance(&self, t: &TraitDef, c: &Conditions) -> f64 {
        if t.optimum == "stature" {
            (t.tolerance * self.stature_limit(c)).max(1e-9)
        } else {
            t.tolerance
        }
    }

    /// The cheap envelope for a habitat nobody has opened: every trait at
    /// its optimum, at no strain.
    pub fn envelope(&self, c: &Conditions) -> BTreeMap<String, f64> {
        self.traits
            .iter()
            .map(|t| (t.id.clone(), self.optimum(t, c)))
            .collect()
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub traits: BTreeMap<String, f64>,
    pub strain: f64,
    /// Share of production the guild keeps, `exp(-strain^2 / 2)`.
    pub share: f64,
    pub alive: bool,
    /// Strain has passed `rescue_from` and not yet come back under 1.
    #[serde(default)]
    pub at_risk: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TraitEvent {
    pub at_myr: f64,
    pub guild: String,
    /// `lost` or `rescued`.
    pub kind: String,
    /// The trait that strained most, and by how much.
    pub trait_id: String,
    pub strain: f64,
}

/// The candidate strategies for one habitat, drawn inside the limits.
pub fn seed_guilds(
    p: &TraitParams,
    c: &Conditions,
    habitat: &Address,
    seed: &Seed,
) -> Result<Vec<Guild>, String> {
    let stream = Stream::new("life.guild.v1").map_err(|e| e.to_string())?;
    let draws = Draws::new(*seed, habitat.clone(), stream);
    let mut out = vec![];
    for k in 0..p.candidates {
        let mut traits = BTreeMap::new();
        for t in &p.traits {
            let (lo, hi) = p.limits(t, c);
            let u = draws.unit(key_of(&format!("{k}#{}", t.id)));
            traits.insert(t.id.clone(), lo + (hi - lo) * u);
        }
        let mut g = Guild {
            id: format!("{habitat}#guild:{k}"),
            traits,
            strain: 0.0,
            share: 1.0,
            alive: true,
            at_risk: false,
        };
        let (s, _) = strain(p, &g, c);
        g.strain = s;
        g.share = m::exp(-s * s / 2.0);
        // A candidate the habitat cannot hold at all never takes hold.
        g.alive = s <= p.lost_strain;
        out.push(g);
    }
    Ok(out)
}

/// A draw counter from a key, so a new trait or candidate moves no other draw.
fn key_of(id: &str) -> u64 {
    let mut h = sse_core::hash::Hasher::new("sse.life.guild-key.v1");
    h.update(id.as_bytes());
    u64::from_be_bytes(h.finish().0[..8].try_into().expect("8 bytes"))
}

/// A guild's strain and the trait that strains most.
pub fn strain(p: &TraitParams, g: &Guild, c: &Conditions) -> (f64, String) {
    let mut worst = (0.0, String::new());
    for t in &p.traits {
        let Some(v) = g.traits.get(&t.id) else {
            continue;
        };
        let s = (v - p.optimum(t, c)).abs() / p.tolerance(t, c);
        if s > worst.0 {
            worst = (s, t.id.clone());
        }
    }
    worst
}

/// Move every living guild over one step of `dt_myr` under `c`, recording
/// losses and rescues at `at_myr` (the step's end).
pub fn step(
    p: &TraitParams,
    guilds: &mut [Guild],
    c: &Conditions,
    at_myr: f64,
    dt_myr: f64,
) -> Vec<TraitEvent> {
    let mut events = vec![];
    let opp = c.opportunity.max(1e-12);
    for g in guilds.iter_mut().filter(|g| g.alive) {
        for t in &p.traits {
            let Some(v) = g.traits.get_mut(&t.id) else {
                continue;
            };
            let opt = p.optimum(t, c);
            let (lo, hi) = p.limits(t, c);
            let tau = t.tau0_myr / opp;
            *v = (opt + (*v - opt) * m::exp(-dt_myr / tau)).clamp(lo, hi);
        }
        let (s, worst) = strain(p, g, c);
        g.strain = s;
        g.share = m::exp(-s * s / 2.0);
        if s > p.lost_strain {
            g.alive = false;
            g.share = 0.0;
            events.push(TraitEvent {
                at_myr,
                guild: g.id.clone(),
                kind: "lost".into(),
                trait_id: worst,
                strain: s,
            });
        } else if s > p.rescue_from {
            g.at_risk = true;
        } else if g.at_risk && s < 1.0 {
            g.at_risk = false;
            events.push(TraitEvent {
                at_myr,
                guild: g.id.clone(),
                kind: "rescued".into(),
                trait_id: worst,
                strain: s,
            });
        }
    }
    events
}

/// The living guilds' share-weighted mean of each trait (what E6 reads as
/// the habitat's cover strategy).
pub fn mean_traits(guilds: &[Guild]) -> BTreeMap<String, f64> {
    let mut sum: BTreeMap<String, f64> = BTreeMap::new();
    let mut w = 0.0;
    for g in guilds.iter().filter(|g| g.alive && g.share > 0.0) {
        w += g.share;
        for (k, v) in &g.traits {
            *sum.entry(k.clone()).or_default() += g.share * v;
        }
    }
    if w > 0.0 {
        for v in sum.values_mut() {
            *v /= w;
        }
    }
    sum
}
