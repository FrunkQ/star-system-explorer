//! SSE4 SIFT engine (H6.6): rule predicates, tags with hysteresis, region
//! classification machinery, sifters with evidence and significance.
//! Generic: suites and packs supply the rules, atlases and sifters as data.
#![forbid(unsafe_code)]

pub mod classic;
pub mod classify;
pub mod pack;
pub mod predicate;
pub mod relabel;
pub mod rules;
pub mod scenario;
pub mod sifter;

pub use classify::{Atlas, Axis, Bound, Class, ClassFit, ClassKind, Label, Scale};
pub use pack::{load, SiftContent};
pub use predicate::{Eval, Op, Predicate, Read, Subject};
pub use rules::{Rule, RuleSet, TagChange, TagState};
pub use sifter::{score, sift, Hit, Pattern, Persistence, Reach, Sifter, SignificanceSpec, World};

use sse_flow::CostClass;

/// Operations this crate adds, with cost classes and edges (build plan section 2).
pub fn ops() -> Vec<(&'static str, CostClass, &'static str)> {
    vec![
        ("rules.apply", CostClass::Trivial, "a few comparisons per rule per subject; any device"),
        ("classify", CostClass::Trivial, "classes x axes arithmetic per subject; any device"),
        ("sift", CostClass::Light, "linear in subjects x sifters within the scope; weak devices sift the open system, the GM machine sifts the campaign"),
    ]
}

#[cfg(test)]
mod base_atlas_tests;
#[cfg(test)]
mod classic_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod golden {
    /// Compare with `tests/golden/<name>`; `SSE_BLESS=1` rewrites it.
    pub fn check(name: &str, got: &serde_json::Value) {
        let path = format!("{}/../../tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
        let text = serde_json::to_string_pretty(got).unwrap() + "\n";
        if std::env::var("SSE_BLESS").is_ok() {
            std::fs::write(&path, &text).unwrap();
            return;
        }
        let want = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing {path}; run with SSE_BLESS=1"));
        assert_eq!(
            text.replace("\r\n", "\n"),
            want.replace("\r\n", "\n"),
            "golden {name} changed"
        );
    }
}
