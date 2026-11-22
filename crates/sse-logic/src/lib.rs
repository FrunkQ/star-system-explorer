//! SSE4 logic blocks (track J2, record design section 3).
//!
//! - [`blocks`]: the small, fixed logic atom set (compare, combine, choose,
//!   numbers, lists, world reads), marked `class: "logic"`.
//! - [`condition`]: one condition language. A K8 predicate becomes the same
//!   condition as blocks, with identical results and evidence.
//! - [`dice`]: `roll` as a keyed draw on the GM-only table secret, and the
//!   commitment seats receive.
//! - [`engine`]: engine blocks, the third kind of block beside atoms and
//!   boxes: `engine.classify` (the class atlas) and `engine.place` (reverse
//!   placement), each a heavy block with its own editor.
//! - [`equation`]: the Equation block, a formula as pack data, and its
//!   expansion into the maths blocks.
//! - [`model`]: the Model number block, a number that belongs to the model
//!   (a fitted slope), with its unit, range, grade and source.
#![forbid(unsafe_code)]

pub mod blocks;
pub mod condition;
pub mod dice;
pub mod engine;
pub mod equation;
pub mod lookup;
pub mod model;

pub use condition::{to_blocks, Condition, SubjectWorld};

/// Every logic atom and the dice, ready to add to a kernel's registry.
pub fn register(reg: &mut sse_flow::AtomRegistry) {
    for a in blocks::all() {
        reg.insert(a);
    }
    for a in dice::all() {
        reg.insert(a);
    }
    reg.insert(Box::new(lookup::Lookup::new()));
    reg.insert(Box::new(equation::Equation::new()));
    reg.insert(Box::new(model::ModelNumber::new()));
}

/// A registry holding only the logic set (tests, condition flows).
pub fn registry() -> sse_flow::AtomRegistry {
    let mut r = sse_flow::AtomRegistry::default();
    register(&mut r);
    r
}

#[cfg(test)]
mod equation_tests;
#[cfg(test)]
mod tests;
