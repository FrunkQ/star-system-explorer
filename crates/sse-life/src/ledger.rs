//! The energy ledger life reads (LE2, life-from-energy design section 3):
//! per habitat and for the world, six lines in watts.
//!
//! - **arriving**: free energy reaching the habitat. Light at the ground
//!   times the habitat's area, weighted by Petela's exergy factor for light
//!   from a source at the light's Wien temperature used at the habitat's
//!   temperature; plus chemical supply, valued as the pack's supply
//!   metabolisms value it (the listed model has no independent free energy
//!   for a supply; the gradient catalogue does, and 4.1 may read it).
//! - **offered**: what life of this chemistry could tap: the energy cap of
//!   every metabolism the pack lists, as stored power.
//! - **captured**: what the biosphere actually runs on (production power,
//!   after nutrients and the metabolisms it has evolved).
//! - **keep-alive**: laboratory maintenance of the thinnest population life
//!   is found at, through the whole habitat (LE1).
//! - **surplus**: captured minus keep-alive. At or below zero the habitat is
//!   "alive, barely" (or closed, below the starved floor).
//! - **brought in**: power from outside the world's own budget (tech's seam,
//!   4.1). Always zero in 4.0.
//!
//! A line nobody could work out is `None`, never zero, and the habitat is
//! listed in `unknown`; world lines are sums over the habitats that know them.

use crate::budget::{self, BudgetParams, HabitatBudget, Metabolism, WorldBudget};
use crate::habitat::HabitatGraph;
use crate::*;
use sse_core::math as m;

use crate::budget::{WIEN_NM_K, YEAR_S};

/// Petela's exergy factor: the share of black-body radiation from a source
/// at `t_source` K that can do work in surroundings at `t0` K,
/// 1 - 4/3 x + 1/3 x^4 with x = t0 / t_source (Petela 1964, unverified; about
/// 0.93 for sunlight used at 288 K). Zero when the surroundings are as hot.
pub fn petela(t0: f64, t_source: f64) -> f64 {
    if t_source <= 0.0 || t0 >= t_source {
        return 0.0;
    }
    let x = t0 / t_source;
    (1.0 - 4.0 / 3.0 * x + m::pow(x, 4.0) / 3.0).clamp(0.0, 1.0)
}

/// The six lines, W.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Lines {
    pub arriving_w: Option<f64>,
    pub offered_w: Option<f64>,
    pub captured_w: Option<f64>,
    pub keep_alive_w: Option<f64>,
    pub surplus_w: Option<f64>,
    pub brought_in_w: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HabitatLedger {
    pub habitat: Address,
    pub kind: String,
    #[serde(flatten)]
    pub lines: Lines,
    /// Light and chemical parts of `arriving`, W.
    pub light_w: Option<f64>,
    pub chemical_w: Option<f64>,
    /// Petela's factor used for the light.
    pub exergy_factor: Option<f64>,
    /// "alive, barely": captured does not pay keep-alive (LE1).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub barely: bool,
    /// Why it is closed, if it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EnergyLedger {
    pub habitats: Vec<HabitatLedger>,
    /// Sums over the habitats that know each line.
    pub world: Lines,
    /// Habitats with a line nobody could work out.
    pub unknown: Vec<Address>,
    /// Sunlight the whole world absorbs, W, for the comparison with life's
    /// share: read from the climate's sunlight budget once it is published.
    pub absorbed_sunlight_w: Option<f64>,
    /// Why a world line is missing, in words.
    pub notes: Vec<String>,
}

fn mid_t(p: &LifeRequirementProfile) -> Option<f64> {
    p.temperature_k.map(|t| 0.5 * (t.lo + t.hi))
}

fn stored_w(gc_yr: f64, bp: &BudgetParams) -> f64 {
    gc_yr * bp.j_per_g_carbon / YEAR_S
}

/// Arriving free energy at a habitat: (light, chemical, Petela factor).
fn arriving(
    p: &LifeRequirementProfile,
    bp: &BudgetParams,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let lit = p.energy.available.iter().any(|c| c == "light");
    let (light, factor) = if !lit {
        (Some(0.0), None)
    } else {
        match (p.energy.light_w_m2, p.area_m2, mid_t(p)) {
            (Some(flux), Some(area), Some(t0)) => {
                let peak = p.energy.light_peak_nm.unwrap_or(500.0);
                let f = petela(t0, WIEN_NM_K / peak);
                (Some(flux.max(0.0) * area * f), Some(f))
            }
            _ => (None, None),
        }
    };
    let supply: Vec<&Metabolism> = bp
        .metabolisms
        .iter()
        .filter(|mb| mb.channel != "light")
        .collect();
    let mut chem = 0.0;
    let mut any = false;
    let mut scratch = vec![];
    for mb in supply {
        if let Some(g) = budget::metabolism_cap(p, bp, mb, &mut scratch) {
            chem += stored_w(g, bp);
            any = true;
        }
    }
    let chemical = any.then_some(chem);
    (light, chemical, factor)
}

/// The ledger of a world: `held` is the budget of what life holds now (the
/// biosphere's), `None` for a lifeless world (captured is then zero).
pub fn ledger(g: &HabitatGraph, bp: &BudgetParams, held: Option<&WorldBudget>) -> EnergyLedger {
    let all: Vec<String> = bp.metabolisms.iter().map(|mb| mb.id.clone()).collect();
    let offered = budget::world_budget(g, bp, &all);
    let mut habitats = vec![];
    let mut unknown = vec![];
    for h in &g.habitats {
        let p = &h.profile;
        let off: Option<&HabitatBudget> = offered.habitats.iter().find(|x| x.habitat == p.habitat);
        let now: Option<&HabitatBudget> =
            held.and_then(|w| w.habitats.iter().find(|x| x.habitat == p.habitat));
        let (light, chemical, factor) = arriving(p, bp);
        // As box B: the parts that are known count; a habitat with neither
        // is unknown.
        let arriving_w = match (light, chemical) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
        };
        let offered_w = off
            .and_then(|x| x.energy_cap_gc_yr)
            .map(|e| stored_w(e, bp));
        let captured_w = match (held, now) {
            (None, _) => Some(0.0),
            (Some(_), Some(x)) => x.power_w,
            (Some(_), None) => None,
        };
        let keep_alive_w = budget::keep_alive(p, bp).and_then(|k| k.growing_w);
        let surplus_w = match (captured_w, keep_alive_w) {
            (Some(c), Some(k)) => Some(c - k),
            _ => None,
        };
        let closed = now
            .and_then(|x| x.closed.clone())
            .or_else(|| off.and_then(|x| x.closed.clone()));
        let barely = now.is_some_and(|x| x.barely);
        let lines = Lines {
            arriving_w,
            offered_w,
            captured_w,
            keep_alive_w,
            surplus_w,
            brought_in_w: 0.0,
        };
        if arriving_w.is_none() || offered_w.is_none() || captured_w.is_none() {
            unknown.push(p.habitat.clone());
        }
        habitats.push(HabitatLedger {
            habitat: p.habitat.clone(),
            kind: p.kind.clone(),
            lines,
            light_w: light,
            chemical_w: chemical,
            exergy_factor: factor,
            barely,
            closed,
        });
    }
    let sum = |f: fn(&Lines) -> Option<f64>| -> Option<f64> {
        let v: Vec<f64> = habitats.iter().filter_map(|h| f(&h.lines)).collect();
        (!v.is_empty()).then(|| v.iter().sum())
    };
    let world = Lines {
        arriving_w: sum(|l| l.arriving_w),
        offered_w: sum(|l| l.offered_w),
        captured_w: sum(|l| l.captured_w),
        keep_alive_w: sum(|l| l.keep_alive_w),
        surplus_w: sum(|l| l.surplus_w),
        brought_in_w: 0.0,
    };
    EnergyLedger {
        habitats,
        world,
        unknown,
        absorbed_sunlight_w: None,
        notes: vec![
            "the sunlight the whole world absorbs is not published yet (the climate's sunlight budget is still to land)"
                .into(),
        ],
    }
}
