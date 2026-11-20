//! Gas mixes (handoff 7A part 5): named atmosphere presets as pack data,
//! a composition (fraction ranges by gas) with an optional named taint, and
//! the code-sheet symbols each one can stand for. A preset is a starting
//! point: once written as facts the physics carries it forward.

use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GasMix {
    pub id: String,
    pub label: String,
    /// Gas formula (as written in facts, `o2`, `co2`) to its volume
    /// fraction range.
    pub composition: BTreeMap<String, [f64; 2]>,
    /// A named taint the gases alone do not show (biological, pollutants).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taint: Option<String>,
    /// Code-sheet segment to the symbols this mix can stand for.
    #[serde(default)]
    pub codes: BTreeMap<String, Vec<String>>,
    /// Relative weight when several mixes fit.
    #[serde(default = "one")]
    pub weight: f64,
    /// Where the preset came from, and any change made to it.
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn one() -> f64 {
    1.0
}

impl GasMix {
    /// Mid-range fractions, scaled to add up to one.
    pub fn fractions(&self) -> BTreeMap<String, f64> {
        let mid: BTreeMap<String, f64> = self
            .composition
            .iter()
            .map(|(g, [lo, hi])| (g.clone(), (lo + hi) / 2.0))
            .collect();
        let sum: f64 = mid.values().sum();
        mid.into_iter()
            .map(|(g, x)| (g, if sum > 0.0 { x / sum } else { 0.0 }))
            .collect()
    }

    /// Whether it can stand for `symbol` of `segment`.
    pub fn fits(&self, segment: &str, symbol: &str) -> bool {
        self.codes
            .get(segment)
            .is_some_and(|s| s.iter().any(|x| x == symbol))
    }

    /// Write the mix into a world's facts at its surface pressure: one
    /// partial pressure per gas (`atmosphere.pp.<gas>`, bar), the main gas,
    /// the mix and any taint. Earlier partial pressures are replaced.
    pub fn apply(&self, facts: &mut BTreeMap<String, Value>) {
        let p_bar = match facts.get("atmosphere.surface_pressure") {
            Some(Value::Float(x)) => *x / 1e5,
            Some(Value::Int(i)) => *i as f64 / 1e5,
            _ => 0.0,
        };
        facts.retain(|k, _| !k.starts_with("atmosphere.pp."));
        let fr = self.fractions();
        for (g, x) in &fr {
            facts.insert(format!("atmosphere.pp.{g}"), Value::Float(x * p_bar));
        }
        // Largest fraction; ties go to the first gas in name order.
        if let Some((g, _)) = fr
            .iter()
            .fold(None::<(&String, f64)>, |best, (g, x)| match best {
                Some((_, bx)) if bx >= *x => best,
                _ => Some((g, *x)),
            })
        {
            facts.insert("atmosphere.main".into(), Value::Text(g.clone()));
        }
        facts.insert("atmosphere.mix".into(), Value::Text(self.id.clone()));
        match &self.taint {
            Some(t) => {
                facts.insert("atmosphere.taint".into(), Value::Text(t.clone()));
            }
            None => {
                facts.remove("atmosphere.taint");
            }
        }
    }
}

/// Pick one of the mixes that fit, by weight, from a unit draw in [0, 1).
pub fn pick<'a>(mixes: &[&'a GasMix], u: f64) -> Option<&'a GasMix> {
    let total: f64 = mixes.iter().map(|m| m.weight.max(0.0)).sum();
    if total <= 0.0 {
        return mixes.first().copied();
    }
    let mut at = u * total;
    for m in mixes {
        let w = m.weight.max(0.0);
        if at < w {
            return Some(m);
        }
        at -= w;
    }
    mixes.last().copied()
}
