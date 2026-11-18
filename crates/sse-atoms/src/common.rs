//! Shared building blocks for atom definitions.

use sse_core::Dimension as D;
use sse_flow::{Atom, AtomDecl, AtomError, CostClass, Param, Port};

/// Physical constants not in `sse_core::quantity::consts`, kept here so this
/// crate does not edit the core. Each is cited.
pub mod k {
    /// Boltzmann constant, J/K. Exact in the 2019 SI (CODATA 2018).
    pub const BOLTZMANN: f64 = 1.380_649e-23;
    /// Atomic mass constant, kg (CODATA 2018).
    pub const AMU: f64 = 1.660_539_066_60e-27;
}

pub type EvalFn = fn(&[f64], &[f64]) -> Result<Vec<f64>, AtomError>;

pub struct FnAtom {
    pub decl: AtomDecl,
    pub f: EvalFn,
}

impl Atom for FnAtom {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, i: &[f64], p: &[f64]) -> Result<Vec<f64>, AtomError> {
        (self.f)(i, p)
    }
}

pub fn port(name: &str, kind: &str, dim: D, valid: Option<(f64, f64)>) -> Port {
    Port {
        name: name.into(),
        kind: kind.into(),
        dim,
        valid,
    }
}

pub fn param(name: &str, dim: D, default: f64, range: (f64, f64), description: &str) -> Param {
    Param {
        name: name.into(),
        dim,
        default,
        range: Some(range),
        description: description.into(),
    }
}

/// Everything an atom needs, written as one literal per atom.
pub struct Def {
    pub id: &'static str,
    pub version: &'static str,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub params: Vec<Param>,
    pub cost: CostClass,
    pub references: &'static [&'static str],
    pub curious: &'static str,
    pub science: &'static str,
    pub formula: &'static str,
    pub f: EvalFn,
}

impl Def {
    pub fn build(self) -> Box<dyn Atom> {
        Box::new(FnAtom {
            decl: AtomDecl {
                id: self.id.into(),
                version: self.version.into(),
                inputs: self.inputs,
                outputs: self.outputs,
                params: self.params,
                cost_class: self.cost,
                references: self.references.iter().map(|s| s.to_string()).collect(),
                explain: sse_flow::decl::ExplainText {
                    curious: self.curious.into(),
                    science: self.science.into(),
                    formula: self.formula.into(),
                },
            },
            f: self.f,
        })
    }
}

/// Error unless `x` is strictly positive.
pub fn positive(x: f64, what: &str) -> Result<f64, AtomError> {
    if x > 0.0 {
        Ok(x)
    } else {
        Err(AtomError(format!("{what} must be positive, got {x}")))
    }
}

/// Dimensions used here that `sse_core` does not name.
pub mod dims {
    use sse_core::Dimension as D;
    /// s^-1 (angular rate, mean motion).
    pub const RATE: D = D([0, 0, -1, 0, 0, 0, 0]);
}
