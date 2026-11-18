//! SSE4 OBSERVE engine (H6.7): the difference between what is true and what
//! an audience has been shown.
//!
//! - A **ladder** of disclosure tiers (default obvious < scanned < surveyed < secret).
//! - A **disclosure map** from packs: every field and tag gets a tier.
//! - **Data views**: named filters (tiers, tags, per-field overrides with precision).
//! - **Grants**: `observe.grant` events set a body's view for an audience;
//!   children inherit unless they have their own. The Scanned button flips
//!   a body between the campaign's "before" and "after" views.
//! - **Scenes**: plain data built from state after the filter, the only thing views see.
#![forbid(unsafe_code)]

pub mod scenario;
pub mod scene;
pub mod view;

pub use scene::{build_scene, diff, Scene, SceneDelta, SceneFact, SceneInput, SceneSubject};
pub use view::{
    effective_view, scan_toggle, withheld, Campaign, DataView, Disclosure, Ladder, Override,
    Precision, TagMode, Target, UnknownStyle, EVERYONE,
};

use sse_flow::CostClass;

/// Operations this crate adds, with cost classes and edges (build plan section 2).
pub fn ops() -> Vec<(&'static str, CostClass, &'static str)> {
    vec![
        (
            "observe.scene",
            CostClass::Light,
            "linear in facts within the scope; any device builds its own scenes",
        ),
        (
            "observe.scan",
            CostClass::Trivial,
            "one grant event; GM only",
        ),
    ]
}

#[cfg(test)]
mod tests;
