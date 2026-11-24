//! SSE4 OBSERVE engine (H6.7): the difference between what is true and what
//! an audience has been shown.
//!
//! - A **ladder** of disclosure tiers (default obvious < scanned < surveyed < secret).
//! - A **disclosure map** from packs: every field and tag gets a tier.
//! - **Data views**: named filters (tiers, tags, per-field overrides with precision).
//! - **Grants**: `observe.grant` events set (or take back) a body's view for
//!   an audience, or an audience's starting view; children inherit unless they
//!   have their own. Campaign tag grants show tagged bodies to an audience
//!   (an empire's discovery tag), after explicit grants. The Scanned button flips
//!   a body between the campaign's "before" and "after" views.
//! - **Scenes**: plain data built from state after the filter, the only thing views see.
#![forbid(unsafe_code)]

pub mod known;
pub mod pack;
pub mod scenario;
pub mod scene;
pub mod view;

pub use known::{set_known, undo_known, KnownChange, Landmark, Landmarks, GM_ONLY_VIEW};
pub use pack::ObserveSetup;
pub use scene::{
    build_scene, build_scene_through, diff, Scene, SceneDelta, SceneFact, SceneInput, SceneNote,
    SceneSubject,
};
pub use view::{
    effective_view, effective_view_tagged, grant, no_tags, scan_toggle, withheld, Campaign,
    DataView, Disclosure, Ladder, Override, Precision, TagGrant, TagMode, Target, UnknownStyle,
    EVERYONE,
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
        (
            "observe.grant",
            CostClass::Trivial,
            "one grant event (set or take back a view for an audience); GM only",
        ),
    ]
}

#[cfg(test)]
mod grant_tests;
#[cfg(test)]
mod tests;
