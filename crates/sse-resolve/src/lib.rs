//! SSE4 RESOLVE and QUERY engines (H6.1, H6.4): clouds to facts, and
//! read-only feasibility questions over clouds.
//!
//! Generic: knows envelopes, budgets, distributions and pins, not stars.
//! Suites supply envelopes (as data or by building them in code).
#![forbid(unsafe_code)]

pub mod dist;
pub mod envelope;
pub mod partition;
pub mod query;
pub mod resolve;
pub mod scenario;

pub use dist::{Distribution, Sample};
pub use envelope::{
    Budget, ChildTemplate, CountSpec, Envelope, EnvelopeError, FieldSpec, LifeSpec, PartitionSpec,
    Pin, Weights,
};
pub use partition::BudgetSplit;
pub use query::{Criterion, Outlook, QueryOutcome, Wish};
pub use resolve::{
    children, phase_at, recorded, resolve, resolve_all, resolve_recorded, ChildRef, Children,
    Phase, Recorded, Resolved,
};

use sse_flow::CostClass;

/// Every operation this crate adds, with its cost class and edge behaviour
/// (build plan section 2).
pub fn ops() -> Vec<(&'static str, CostClass, &'static str)> {
    vec![
        ("resolve", CostClass::Light, "one child: a few draws, plus one weight per sibling for flat splits; any device"),
        ("resolve_all", CostClass::Medium, "linear in the child count; weak devices resolve only what a view shows, or take recorded facts from the GM"),
        ("query.location", CostClass::Trivial, "closed-form outlook from the envelope; any device"),
        ("query.outcome", CostClass::Medium, "bounded by the caller's work budget; returns 'not found within budget' when it runs out"),
        ("query.search_children", CostClass::Medium, "bounded by the work budget; weak devices get a smaller budget and see where the search stopped"),
    ]
}

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
