//! Physics blocks whose maths is a formula (package ND3b, equation block
//! design 8A). Each keeps its id, ports and results; its maths is an
//! Equation in its base pack (`packs/base-physics/formulas/`), under a name
//! that says what happens in the world. The blocks still written in Rust
//! are listed, each with why, in `handover/packages/ND3b-rust-atoms.txt`,
//! a list that only shrinks.

use sse_flow::Atom;
use sse_formula::FormulaAtom;
use std::sync::OnceLock;

/// The base packs' formula files, by family.
pub const FILES: &[(&str, &str)] = &[
    (
        "planet",
        include_str!("../../../packs/base-physics/formulas/planet.json"),
    ),
    (
        "orbit",
        include_str!("../../../packs/base-physics/formulas/orbit.json"),
    ),
    (
        "star",
        include_str!("../../../packs/base-physics/formulas/star.json"),
    ),
];

/// Read every formula file once: parsed, unit-checked and its worked
/// examples run (design 8B: no text is read when a block runs).
pub fn loaded() -> &'static [FormulaAtom] {
    static ALL: OnceLock<Vec<FormulaAtom>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut out = vec![];
        for (family, text) in FILES {
            let doc: serde_json::Value = serde_json::from_str(text)
                .unwrap_or_else(|e| panic!("formulas/{family}.json: {e}"));
            for a in doc["atoms"].as_array().into_iter().flatten() {
                out.push(
                    FormulaAtom::from_json(a)
                        .unwrap_or_else(|e| panic!("formulas/{family}.json: {e}")),
                );
            }
        }
        out
    })
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    loaded()
        .iter()
        .map(|a| Box::new(a.clone()) as Box<dyn Atom>)
        .collect()
}
