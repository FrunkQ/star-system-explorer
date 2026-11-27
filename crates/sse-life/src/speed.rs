//! The speed read-out (LE4, life-from-energy design sections 6.1 and 7):
//! energy as life's speed control.
//!
//! - **Fill time per habitat** from Fisher's front: v = 2 sqrt(r D)
//!   (Fisher 1937; Skellam 1951). r is the growth rate of a thin pioneer
//!   population, the smaller of the biology's best (a cell's r_max, moved
//!   with temperature by the metabolic theory's activation energy) and what
//!   the surplus pays for (surplus per gram of the thinnest living population
//!   / the free energy built into a gram). Rich habitat: biology sets the
//!   speed; starved: energy does, falling to zero at the keep-alive line. D
//!   is how fast the medium mixes things, from a short cited table. The
//!   span is the habitat's width (the square root of its area), or for
//!   rock its thickness.
//! - **One world line**: opportunity per Myr now, in Earth-Myr (the same
//!   rule the history steps, `history::opportunity_rate`), so present-day
//!   Earth reads about one.

use crate::budget::{BudgetParams, WorldBudget, YEAR_S};
use crate::habitat::HabitatGraph;
use crate::history::HistoryParams;
use crate::ledger::EnergyLedger;
use crate::*;
use sse_core::math as m;

/// `life_history.spread` (LE4, and LE5's links).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SpreadParams {
    /// A pioneer cell's best growth rate at the history's reference
    /// temperature, per day.
    pub r_max_per_day: f64,
    /// Mixing by medium, m^2 per s.
    pub mixing_m2_s: BTreeMap<String, f64>,
    /// Habitat media whose span is their thickness, not their width.
    pub thickness_media: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HabitatSpeed {
    pub habitat: Address,
    pub kind: String,
    /// Growth rate of a thin pioneer population, per s, and what set it
    /// (`biology`, `energy` or `none`).
    pub r_per_s: Option<f64>,
    pub r_limited_by: String,
    pub mixing_m2_s: Option<f64>,
    pub front_m_s: Option<f64>,
    pub span_m: Option<f64>,
    /// Time for the front to cross the habitat, years. `None` when no front
    /// moves (no surplus) or an input is missing.
    pub fill_yr: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Speed {
    pub habitats: Vec<HabitatSpeed>,
    /// Opportunity per Myr now, Earth-Myr (Earth today about 1).
    pub pace_x_earth: f64,
    /// The world line in words.
    pub words: String,
}

fn mid_t(p: &LifeRequirementProfile) -> Option<f64> {
    p.temperature_k.map(|t| 0.5 * (t.lo + t.hi))
}

/// A pioneer's growth rate, per s: (rate, what limited it). `surplus_w`
/// over the thinnest living population pays for new biomass at
/// `j_per_g_carbon`.
pub fn growth_rate(
    sp: &SpreadParams,
    hp: &HistoryParams,
    bp: &BudgetParams,
    t_k: f64,
    surplus_w: f64,
    population_gc: f64,
) -> (f64, &'static str) {
    let k_ev = 8.617_333e-5;
    let r_max = sp.r_max_per_day / 86_400.0
        * m::exp(-hp.activation_ev / k_ev * (1.0 / t_k - 1.0 / hp.reference_k));
    if surplus_w <= 0.0 || population_gc <= 0.0 {
        return (0.0, "none");
    }
    let r_energy = surplus_w / population_gc / bp.j_per_g_carbon;
    if r_energy < r_max {
        (r_energy, "energy")
    } else {
        (r_max, "biology")
    }
}

/// The speed read-out of a world: `held` is what life holds now.
pub fn speed(
    g: &HabitatGraph,
    bp: &BudgetParams,
    hp: &HistoryParams,
    ledger: &EnergyLedger,
    held: Option<&WorldBudget>,
) -> Option<Speed> {
    let sp = hp.spread.as_ref()?;
    let mut habitats = vec![];
    for h in &g.habitats {
        let p = &h.profile;
        let line = ledger.habitats.iter().find(|x| x.habitat == p.habitat);
        let surplus = line.and_then(|l| l.lines.surplus_w);
        let pop = bp
            .maintenance
            .as_ref()
            .and_then(|mt| mt.growing_population_gc(p));
        let (r, by) = match (mid_t(p), surplus, pop) {
            (Some(t), Some(s), Some(pg)) => {
                let (r, by) = growth_rate(sp, hp, bp, t, s, pg);
                (Some(r), by)
            }
            _ => (None, "none"),
        };
        let d = sp.mixing_m2_s.get(&p.medium).copied();
        let front = match (r, d) {
            (Some(r), Some(d)) => Some(2.0 * m::sqrt(r * d)),
            _ => None,
        };
        let span = if sp.thickness_media.contains(&p.medium) {
            match (p.volume_m3, p.area_m2) {
                (Some(v), Some(a)) if a > 0.0 => Some(v / a),
                _ => None,
            }
        } else {
            p.area_m2.map(m::sqrt)
        };
        let fill = match (front, span) {
            (Some(v), Some(x)) if v > 0.0 => Some(x / v / YEAR_S),
            _ => None,
        };
        habitats.push(HabitatSpeed {
            habitat: p.habitat.clone(),
            kind: p.kind.clone(),
            r_per_s: r,
            r_limited_by: by.into(),
            mixing_m2_s: d,
            front_m_s: front,
            span_m: span,
            fill_yr: fill,
        });
    }
    // The world line: opportunity per Myr now, as the history steps it.
    let production = held.map_or(0.0, |w| w.production_gc_yr);
    // Open as the history counts it: not closed by the habitat's conditions.
    let open: Vec<usize> = g
        .habitats
        .iter()
        .enumerate()
        .filter(|(_, h)| crate::budget::closed_reason(&h.profile, bp).is_none())
        .map(|(i, _)| i)
        .collect();
    let mut kinds: Vec<&str> = open.iter().map(|&i| g.habitats[i].kind.as_str()).collect();
    kinds.sort();
    kinds.dedup();
    let temp = crate::history::mean_temperature(g, &open).unwrap_or(hp.reference_k);
    let pace = crate::history::opportunity_rate(hp, production, temp, kinds.len());
    let words = if pace <= 0.0 {
        "nothing alive here moves or changes".to_string()
    } else {
        let n = if pace >= 0.1 {
            format!("{pace:.1}")
        } else {
            format!("{pace:.1e}")
        };
        format!("life here moves and changes at about {n} times Earth's pace")
    };
    Some(Speed {
        habitats,
        pace_x_earth: pace,
        words,
    })
}
