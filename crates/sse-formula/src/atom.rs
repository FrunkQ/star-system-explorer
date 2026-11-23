//! A physics atom whose maths is a formula (package ND3b, equation block
//! design 8A): the block keeps its id, ports and results, and its maths is
//! an Equation written as data in its base pack, under a name that says
//! what happens in the world.
//!
//! One entry of a formula file (`packs/base-*/formulas/<family>.json`):
//!
//! ```json
//! { "id": "sse.planet.escape_velocity", "version": "1.0.0", "cost": "trivial",
//!   "inputs":  [ {"port": "mass", "letter": "M"}, {"port": "radius", "letter": "R"} ],
//!   "positive": [ {"port": "radius", "says": "radius"}, {"port": "mass", "says": "mass"} ],
//!   "outputs": [ {"port": "v_esc", "letter": "v"} ],
//!   "constants": { "G": "constants/G" },
//!   "references": ["Newtonian gravity ... doi:..."],
//!   "shown": "v = √(2 G M / R)",
//!   "equation": { "name": "Speed needed to escape", "formula": "v = sqrt(2 * G * M / R)", ... } }
//! ```
//!
//! A port's kind, dimension and valid range come from its letter in the
//! Equation. A constant letter reads the constant in force (the sheet's row,
//! so a changed G flows through), never a typed copy. `positive` keeps
//! the old block's refusals, in their order and word for word. The formula runs through
//! the same pinned maths as the Equation block, in the order written, so a
//! moved block gives the same bits as its Rust maths did.

use crate::{Parsed, PortSpec};
use serde::Deserialize;
use serde_json::Value as J;
use sse_flow::decl::ExplainText;
use sse_flow::{Atom, AtomDecl, AtomError, CostClass, Port, Reference};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortEntry {
    port: String,
    letter: String,
}

/// A refusal kept from the old block: the port must be above zero, and
/// the refusal names it in these words ("semi-major axis must be positive").
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Positive {
    port: String,
    says: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    version: String,
    cost: CostClass,
    inputs: Vec<PortEntry>,
    outputs: Vec<PortEntry>,
    #[serde(default)]
    constants: BTreeMap<String, String>,
    /// Checked in this order before the formula runs.
    #[serde(default)]
    positive: Vec<Positive>,
    references: Vec<String>,
    /// The formula as the block's face has always shown it.
    shown: String,
    equation: J,
}

/// Where each of the formula's letters comes from.
#[derive(Clone, Debug)]
enum From {
    Port(usize),
    Constant(fn() -> f64),
}

/// The constants a formula may read, as the sheet names them, each the
/// value in force.
pub fn constant(row: &str) -> Option<fn() -> f64> {
    Some(match row {
        "constants/G" => sse_core::constants::g,
        "constants/c" => sse_core::constants::c,
        "constants/sigma_SB" => sse_core::constants::sigma_sb,
        "constants/k_B" => sse_core::constants::k_b,
        _ => return None,
    })
}

/// A physics atom defined by a formula.
#[derive(Clone, Debug)]
pub struct FormulaAtom {
    decl: AtomDecl,
    parsed: Parsed,
    /// For each of the formula's inputs (in its order), where it comes from.
    from: Vec<From>,
    /// For each output port, the formula's output index.
    out: Vec<usize>,
    /// Ports that must be above zero, with their names.
    positive: Vec<(usize, String)>,
}

fn port(p: &PortEntry, spec: &PortSpec, dim: sse_core::Dimension, scale: f64) -> Port {
    Port {
        name: p.port.clone(),
        kind: spec.kind.clone(),
        dim,
        valid: spec.valid.map(|(a, b)| (a * scale, b * scale)),
        shape: None,
        any_dim: false,
    }
}

impl FormulaAtom {
    /// Read and check one entry of a formula file.
    pub fn from_json(v: &J) -> Result<FormulaAtom, String> {
        let e: Entry = serde_json::from_value(v.clone()).map_err(|e| format!("{e}"))?;
        let at = |m: String| format!("{}: {m}", e.id);
        // A constant letter is never typed into a worked example: the
        // example reads the sheet's value in force, like the block does.
        let mut equation = e.equation.clone();
        if let Some(J::Array(exs)) = equation.get_mut("examples") {
            for ex in exs.iter_mut().filter_map(J::as_object_mut) {
                for (l, row) in &e.constants {
                    if ex.contains_key(l) {
                        return Err(at(format!(
                            "a worked example types {l}; it reads {row} from the sheet"
                        )));
                    }
                    if let Some(f) = constant(row) {
                        ex.insert(l.clone(), J::from(f()));
                    }
                }
            }
        }
        let parsed = Parsed::new(Some(&equation)).map_err(at)?;
        if parsed.list {
            return Err(at(
                "a physics atom's formula takes single numbers, not lists".into(),
            ));
        }
        let letter_in = |l: &str| parsed.inputs.iter().position(|x| x == l);
        let mut from = vec![None; parsed.inputs.len()];
        let mut inputs = vec![];
        let mut positive = vec![];
        for (k, p) in e.inputs.iter().enumerate() {
            let i = letter_in(&p.letter)
                .ok_or_else(|| at(format!("port {} names no letter {}", p.port, p.letter)))?;
            if from[i].is_some() {
                return Err(at(format!("letter {} is fed twice", p.letter)));
            }
            from[i] = Some(From::Port(k));
            let spec = &parsed.cfg.inputs[&p.letter];
            inputs.push(port(p, spec, parsed.in_dims[i], parsed.in_scales[i]));
        }
        for q in &e.positive {
            let k = e
                .inputs
                .iter()
                .position(|p| p.port == q.port)
                .ok_or_else(|| at(format!("positive names no port {}", q.port)))?;
            positive.push((k, q.says.clone()));
        }
        for (l, row) in &e.constants {
            let i = letter_in(l).ok_or_else(|| at(format!("constant {l} is no letter")))?;
            if from[i].is_some() {
                return Err(at(format!("letter {l} is fed twice")));
            }
            if parsed.in_scales[i] != 1.0 {
                return Err(at(format!("constant {l} is written in SI units")));
            }
            let f = constant(row).ok_or_else(|| {
                at(format!("{row} is not a constant a formula can read (constants/G, constants/c, constants/sigma_SB, constants/k_B)"))
            })?;
            from[i] = Some(From::Constant(f));
        }
        let from: Vec<From> = from
            .into_iter()
            .enumerate()
            .map(|(i, f)| f.ok_or_else(|| at(format!("letter {} has no line", parsed.inputs[i]))))
            .collect::<Result<_, _>>()?;
        let mut outputs = vec![];
        let mut out = vec![];
        for p in &e.outputs {
            let o = parsed
                .outputs
                .iter()
                .position(|x| *x == p.letter)
                .ok_or_else(|| at(format!("port {} names no output {}", p.port, p.letter)))?;
            let spec = &parsed.cfg.outputs[&p.letter];
            outputs.push(port(p, spec, parsed.out_dims[o], parsed.out_scales[o]));
            out.push(o);
        }
        if out.len() != parsed.outputs.len() {
            return Err(at("every output of the formula is a port".into()));
        }
        let explain = ExplainText {
            curious: parsed.cfg.explain.curious.clone(),
            science: parsed.cfg.explain.science.clone(),
            formula: e.shown.clone(),
        };
        Ok(FormulaAtom {
            decl: AtomDecl {
                id: e.id.clone(),
                version: e.version.clone(),
                inputs,
                outputs,
                params: vec![],
                cost_class: e.cost,
                references: e.references.iter().map(|s| Reference::parse(s)).collect(),
                explain,
                class: None,
                dims: None,
            },
            parsed,
            from,
            out,
            positive,
        })
    }

    /// The Equation's worded name ("Speed needed to escape").
    pub fn name(&self) -> &str {
        &self.parsed.cfg.name
    }
    /// The formula as written.
    pub fn formula(&self) -> &str {
        &self.parsed.cfg.formula
    }
    pub fn parsed(&self) -> &Parsed {
        &self.parsed
    }
}

impl Atom for FormulaAtom {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, inputs: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        for (k, what) in &self.positive {
            let x = inputs[*k];
            if x > 0.0 {
                continue;
            }
            return Err(AtomError(format!("{what} must be positive, got {x}")));
        }
        let x: Vec<f64> = self
            .from
            .iter()
            .map(|f| match f {
                From::Port(k) => inputs[*k],
                From::Constant(c) => c(),
            })
            .collect();
        let nums = self
            .parsed
            .run_numbers(&x)
            .map_err(|e| AtomError(format!("{}: {e}", self.parsed.me())))?;
        Ok(self.out.iter().map(|o| nums[*o]).collect())
    }
}
