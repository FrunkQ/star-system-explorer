//! SSE4 suite socket and coordinator (H11).
//!
//! A **suite** supplies a domain (life, cosmos, technosphere) behind one
//! interface. The kernel is the **coordinator**: it owns the seed, the clock,
//! the common time boundaries, ordering and persistence. At each boundary it
//! lets the physics provider (the authoritative owner of physical truth)
//! step, hands every suite a pinned, content-hashed input dataset, takes back
//! events, fluxes and signatures, checks them against the suite's declared
//! rights, and commits both sides' outputs to the chronicle in a fixed order.
//!
//! Determinism classes (H9.2): class A suites are bit-exact and may be re-run
//! anywhere; class B suites run once on the GM machine and everyone else
//! folds their recorded output. Either way the chronicle alone replays the run.
#![forbid(unsafe_code)]

mod coordinator;
pub mod scenario;
mod socket;

pub use coordinator::{replay_state, Coordinator, CoordinatorError, RunReport, Violation};
pub use socket::{
    AdvanceOutput, DeterminismClass, Flux, InputDataset, NullSuite, Physics, Requirement,
    SuiteAdapter, SuiteError, SuiteEvent, SuiteManifest,
};

use sse_flow::CostClass;

/// Operations this crate adds, with cost classes and edges (build plan section 2).
pub fn ops() -> Vec<(&'static str, CostClass, &'static str)> {
    vec![
        ("suite.advance", CostClass::Medium, "per boundary per suite, as the suite declares; class B suites run only on the GM machine and players fold the recorded results"),
        ("suite.replay", CostClass::Light, "folding recorded boundaries; any device"),
    ]
}

#[cfg(test)]
mod tests;
