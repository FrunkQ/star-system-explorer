//! SSE4 flows (H6.2, H6.5): atoms are compiled operators, flows are data.
#![forbid(unsafe_code)]

pub mod decl;
pub mod flow;
pub mod toy;

pub use decl::{Atom, AtomDecl, AtomError, AtomRegistry, CostClass, Param, Port, Reference};
pub use flow::{
    Context, EvalError, Evaluation, Explanation, Flow, FlowDoc, FlowError, InputValue, SourceLabel,
    Validity,
};

#[cfg(test)]
mod tests;
