//! E8: how far a world's air is out of chemical equilibrium (evolution
//! design section 10), feeding box I.
//!
//! A biosphere is visible because it holds gradients open that chemistry
//! alone would close, oxygen beside methane being the classic case. This is
//! the measure of Krissansen-Totton, Bergsman and Catling (2016,
//! doi:10.1089/ast.2015.1327): the **available Gibbs energy**, the Gibbs
//! energy of the observed air minus that of the same elements brought to
//! equilibrium, in joules per mole of air.
//!
//! - **Gibbs energy.** The air is an ideal gas: each species has
//!   μ = μ°(T) + RT ln(x P / P°), with μ° from E0's formation energies
//!   (constant-enthalpy van 't Hoff, as everywhere in E0). Liquid water, when
//!   the world has it, is a pure phase at activity 1 that can take up or give
//!   water.
//! - **Equilibrium.** The minimum is found by cycling a line search along
//!   each reaction of a basis: E0's gas reactions and those in
//!   `DiseqParams.extra`. G is convex in the reaction extents, so these
//!   coordinate steps converge to the one minimum. The basis has more
//!   directions than it needs, so no step stalls on a species that is
//!   exhausted. Each step's drop is credited to its reaction, so the
//!   result also says which pair carries the signal.
//!
//! Scope: the air (and surface water as a pure phase) only. Their full
//! atmosphere-ocean figure adds dissolved species, ions and pH, and comes
//! with E7's ocean chemistry. For Earth today the air alone gives about
//! 1.5 J/mol, carried by CH4 with O2 (their atmosphere-only value).

use crate::chemistry::ThermoTable;
use serde::{Deserialize, Serialize};
use sse_atoms::chem::{standard_gibbs_at, R_GAS};
use sse_core::math as m;
use std::collections::BTreeMap;

/// Settings (`life_signatures.disequilibrium`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DiseqParams {
    /// Reactions of E0's table used as search directions.
    pub reactions: Vec<String>,
    /// Further directions as `(id, terms)`, balanced on load.
    pub extra: Vec<(String, Vec<(String, f64)>)>,
    /// The liquid water species (a pure phase, activity 1).
    pub water: String,
    /// Sweeps over the basis, and bisection steps per line search.
    pub sweeps: u32,
    pub steps: u32,
    /// Above this, J per mol of air, the air reads as held out of balance.
    pub signal_j_mol: f64,
    /// Likelihood ratios for box I: air held out of balance, and gases a
    /// single-gas test would flag that turn out to be at equilibrium.
    pub lr_held: f64,
    pub lr_balanced: f64,
    pub sources: Vec<String>,
}

impl Default for DiseqParams {
    fn default() -> Self {
        DiseqParams {
            reactions: vec![
                "knallgas".into(),
                "methane-oxidation".into(),
                "methanogenesis".into(),
            ],
            extra: vec![
                (
                    "co-oxidation".into(),
                    vec![
                        ("co".into(), -2.0),
                        ("o2".into(), -1.0),
                        ("co2".into(), 2.0),
                    ],
                ),
                (
                    "water-gas-shift".into(),
                    vec![
                        ("co".into(), -1.0),
                        ("h2o".into(), -1.0),
                        ("co2".into(), 1.0),
                        ("h2".into(), 1.0),
                    ],
                ),
            ],
            water: "h2o".into(),
            sweeps: 60,
            steps: 200,
            signal_j_mol: 0.1,
            lr_held: 20.0,
            lr_balanced: 0.5,
            sources: vec![],
        }
    }
}

/// The result.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Disequilibrium {
    /// Available Gibbs energy, J per mol of air.
    pub available_j_mol: f64,
    /// Credited to each reaction, J per mol of air, largest first.
    pub by_reaction: Vec<(String, f64)>,
    /// The air at equilibrium, mole fractions.
    pub equilibrium: BTreeMap<String, f64>,
    pub out_of_balance: bool,
    pub why: String,
}

struct Dir {
    id: String,
    /// Gas species index and coefficient; liquid water left out.
    gas: Vec<(usize, f64)>,
    /// Standard Gibbs energy of the reaction at T, liquid water included, J.
    dg0: f64,
    /// The liquid water's part of it, J per unit extent.
    water_dg0: f64,
}

/// Available Gibbs energy of air with mole fractions `x` at `t_k` and
/// `p_bar`. `liquid_water` says whether surface water can take part.
pub fn disequilibrium(
    table: &ThermoTable,
    p: &DiseqParams,
    x: &BTreeMap<String, f64>,
    t_k: f64,
    p_bar: f64,
    liquid_water: bool,
) -> Result<Disequilibrium, String> {
    let ids: Vec<&String> = x.keys().collect();
    let mu0 = |id: &str| -> Result<f64, String> {
        let s = table
            .species(id)
            .ok_or_else(|| format!("unknown species {id}"))?;
        Ok(standard_gibbs_at(s.dgf_kj * 1000.0, s.dhf_kj * 1000.0, t_k))
    };
    let mut basis: Vec<(String, Vec<(String, f64)>)> = vec![];
    for r in &p.reactions {
        let r = table
            .reaction(r)
            .ok_or_else(|| format!("unknown reaction {r}"))?;
        basis.push((r.id.clone(), r.terms.clone()));
    }
    for (id, terms) in &p.extra {
        let r = crate::chemistry::Reaction {
            id: id.clone(),
            label: id.clone(),
            terms: terms.clone(),
            electrons: None,
            published_kj: None,
            published_ph: None,
            source: None,
        };
        table.balance(&r)?;
        basis.push((id.clone(), terms.clone()));
    }
    // Keep a direction only if every species it uses is in the air or is
    // the liquid water the world has.
    let mut dirs = vec![];
    'next: for (id, terms) in basis {
        let mut gas = vec![];
        let mut dg0 = 0.0;
        let mut water_dg0 = 0.0;
        for (s, nu) in &terms {
            if *s == p.water {
                if !liquid_water {
                    continue 'next;
                }
                water_dg0 += nu * mu0(s)?;
                dg0 += nu * mu0(s)?;
            } else if let Some(i) = ids.iter().position(|k| *k == s) {
                gas.push((i, *nu));
                dg0 += nu * mu0(s)?;
            } else {
                continue 'next;
            }
        }
        dirs.push(Dir {
            id,
            gas,
            dg0,
            water_dg0,
        });
    }
    let mut mu0s = vec![];
    for id in &ids {
        mu0s.push(mu0(id)?);
    }
    let rt = R_GAS * t_k;
    let ln_p = m::ln(p_bar.max(1e-12));
    let total: f64 = x.values().sum();
    if total <= 0.0 {
        return Err("the air has no gas".into());
    }
    let mut n: Vec<f64> = ids.iter().map(|k| x[*k] / total).collect();
    // Gibbs energy of the gas, per mol of the starting air, with the liquid
    // water's change counted through each reaction's dg0 (it is a pure phase).
    let gibbs = |n: &[f64]| -> f64 {
        let nt: f64 = n.iter().sum();
        n.iter()
            .zip(&mu0s)
            .filter(|(v, _)| **v > 0.0)
            .map(|(v, mu)| v * (mu + rt * (m::ln(v / nt) + ln_p)))
            .sum()
    };
    // ΔG of a direction at a composition: Σ ν μ (liquid water at activity 1).
    let slope = |d: &Dir, n: &[f64]| -> f64 {
        let nt: f64 = n.iter().sum();
        let mut s = d.dg0;
        for (i, nu) in &d.gas {
            let v = n[*i].max(1e-300);
            s += nu * rt * (m::ln(v / nt) + ln_p);
        }
        s
    };
    let mut credit: BTreeMap<String, f64> = BTreeMap::new();
    for _ in 0..p.sweeps {
        for d in &dirs {
            // Extents keep every gas amount at or above zero.
            let (mut lo, mut hi) = (f64::NEG_INFINITY, f64::INFINITY);
            for (i, nu) in &d.gas {
                let lim = -n[*i] / nu;
                if *nu < 0.0 {
                    hi = hi.min(lim);
                } else {
                    lo = lo.max(lim);
                }
            }
            if !(lo.is_finite() && hi.is_finite()) || hi - lo <= 0.0 {
                continue;
            }
            let at = |xi: f64| -> Vec<f64> {
                let mut v = n.clone();
                for (i, nu) in &d.gas {
                    v[*i] = (v[*i] + nu * xi).max(0.0);
                }
                v
            };
            // ΔG rises with the extent (convexity): bisect for ΔG = 0.
            let (mut a, mut b) = (lo, hi);
            for _ in 0..p.steps {
                let mid = 0.5 * (a + b);
                if slope(d, &at(mid)) < 0.0 {
                    a = mid;
                } else {
                    b = mid;
                }
            }
            let xi = 0.5 * (a + b);
            let next = at(xi);
            // The gas's change plus the liquid water made or used.
            let drop = gibbs(&n) - gibbs(&next) - xi * d.water_dg0;
            if drop > 0.0 {
                *credit.entry(d.id.clone()).or_default() += drop;
                n = next;
            }
        }
    }
    let available: f64 = credit.values().sum();
    let mut by: Vec<(String, f64)> = credit.into_iter().collect();
    by.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let nt: f64 = n.iter().sum();
    let equilibrium = ids
        .iter()
        .zip(&n)
        .map(|(k, v)| ((*k).clone(), v / nt))
        .collect();
    let out = available > p.signal_j_mol;
    let why = match (out, by.first()) {
        (true, Some((r, e))) => format!(
            "{available:.3} J per mol of air is held out of equilibrium, most of it ({e:.3}) in {r}: something keeps supplying what chemistry removes"
        ),
        _ => format!(
            "{available:.3} J per mol of air: the gases are close to equilibrium with each other, so they need no source that keeps them apart"
        ),
    };
    Ok(Disequilibrium {
        available_j_mol: available,
        by_reaction: by,
        equilibrium,
        out_of_balance: out,
        why,
    })
}

/// Box I with the measured air: the disequilibrium replaces the gas-pair
/// signals (`o2_ch4`, `o2`, `ch4`), which read the same evidence more
/// crudely, and the confidence is worked out again from the signals.
pub fn refine(
    s: &mut crate::signatures::Signatures,
    d: &Disequilibrium,
    dp: &DiseqParams,
    sp: &crate::signatures::SignatureParams,
) {
    let had_gas = s
        .signals
        .iter()
        .any(|g| matches!(g.id.as_str(), "o2_ch4" | "o2" | "ch4"));
    s.signals
        .retain(|g| !matches!(g.id.as_str(), "o2_ch4" | "o2" | "ch4"));
    if d.out_of_balance || had_gas {
        s.signals.retain(|g| g.id != "nothing");
        s.signals.push(crate::signatures::Signal {
            id: "disequilibrium".into(),
            value: d.available_j_mol,
            lr: if d.out_of_balance {
                dp.lr_held
            } else {
                dp.lr_balanced
            },
            why: d.why.clone(),
        });
    }
    let lr: f64 = s.signals.iter().map(|g| g.lr).product();
    let odds = sp.prior / (1.0 - sp.prior) * lr;
    s.confidence = odds / (1.0 + odds);
    s.candidate = s.signals.iter().any(|g| g.id != "nothing") && s.confidence > sp.prior;
}
