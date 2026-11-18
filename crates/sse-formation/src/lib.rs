//! SSE4 system formation suite (track F, `SSE4-system-formation-design.md`).
//!
//! F0: inheritance from a patch envelope (stage 1), stars with multiplicity
//! and life lines (stage 2), the FormationHistory record with time slices.
#![forbid(unsafe_code)]

pub mod disc;
pub mod history;
pub mod hurley;
pub mod inherit;
pub mod lifeline;
pub mod scenario;
pub mod stars;

pub use history::{FormationError, FormationHistory, Manifest};
pub use inherit::{Inheritance, InheritancePins, PatchEnvelope};
pub use lifeline::{LifeLine, Phase, StarState};

#[cfg(test)]
mod tests;

pub(crate) mod golden {
    /// Compare with `tests/golden/<name>`; `SSE_BLESS=1` rewrites it.
    #[cfg(test)]
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
