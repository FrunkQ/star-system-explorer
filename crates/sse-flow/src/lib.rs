//! SSE4 flows (H6.2, H6.5): atoms are compiled operators, flows are data.
#![forbid(unsafe_code)]

pub mod decl;
pub mod flow;
pub mod toy;
pub mod value;

pub use decl::{
    Atom, AtomDecl, AtomError, AtomRegistry, BlockClass, Call, CostClass, DimRule, Env, Out, Param,
    Port, Reference, World,
};
pub use flow::{
    Context, EvalError, Evaluation, Explanation, Flow, FlowDoc, FlowError, InputValue, SourceLabel,
    Validity,
};
pub use value::{Shape, Val};

#[cfg(test)]
mod tests;
