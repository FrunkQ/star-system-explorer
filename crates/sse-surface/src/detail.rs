//! S16: detail levels (Rock 3 comparison, section 5.8, accepted 21 Nov).
//!
//! Most worlds are glanced at and skipped, so a world is worked out only as
//! far as someone looks at it:
//! - **Glance** (D0): the envelope and regime, already made for every world.
//! - **Orbit** (D1): every surface stage but the feature register, on a
//!   coarse grid (the base pack's `plates.guide`, 32 cells a face): plates,
//!   wear, the craters and landforms at least two cells across, cover and
//!   climate.
//! - **Survey** (D2): every stage (`survey::survey_world`), its plate run
//!   pinned to the Orbit run, so the continents seen from orbit are still
//!   there after the survey.
//! - **Site** (D3): rock columns and caves for the cells asked about (S9,
//!   S14, not built yet).
//!
//! Two rules sit over all of them: reuse a result before computing it (the
//! kernel keeps recent Orbit atlases; a result is reused only for the same
//! world key, code version and level, so it is identical to a fresh one),
//! and go up a level only on real need (approach, stay or an explicit
//! request; a click, a hover or a list never starts Survey work).

use crate::atlas::Atlas;
use crate::plates::{guide_settings, PLATES_VERSION};
use crate::survey::{SurveyOptions, SURVEY_VERSION};
use crate::{Body, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::Seed;
use sse_core::math as m;

pub const DETAIL_VERSION: &str = "surface-detail-v1";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Glance,
    Orbit,
    Survey,
    Site,
}

impl Level {
    /// The quality badge's words for a view at this level.
    pub fn badge(self) -> &'static str {
        match self {
            Level::Glance => "Glance: the kind of world and why, nothing about any place on it",
            Level::Orbit => "Orbit detail: continents, ranges and ocean floor from the plate history on a coarse grid; this world has not been surveyed",
            Level::Survey => "Survey detail: the full history, refined from the Orbit run",
            Level::Site => "Site detail: the ground under a site, layer by layer",
        }
    }

    /// What "why is this here?" may say about detail finer than the level
    /// resolves: never a cause it has not worked out.
    pub fn below_resolution(self) -> &'static str {
        match self {
            Level::Glance => "Not worked out: the globe is drawn from the world's kind only.",
            Level::Orbit => "Texture typical of the terrain around it, not yet resolved. Survey to see its history.",
            Level::Survey => "Below the survey's grid; land at a site to see the ground itself.",
            Level::Site => "Resolved at the site.",
        }
    }
}

/// The settings of the Orbit level: the plate run is the base pack's guide.
pub fn orbit_settings(s: &SurfaceSettings) -> SurfaceSettings {
    let mut o = s.clone();
    if let Some(p) = guide_settings(&s.plates) {
        o.plates = p;
    }
    o
}

/// The key under which an Orbit atlas may be reused: the world, the code
/// and the level. Equal keys give identical atlases.
pub fn orbit_key(env: &SurfaceEnvelope, seed: &Seed, epoch_gyr: f64) -> String {
    format!(
        "{DETAIL_VERSION}/{PLATES_VERSION}/{SURVEY_VERSION}/orbit/{}/{}/{}",
        serde_json::to_string(seed).expect("plain data"),
        env.address,
        m::round(epoch_gyr * 1.0e6) as i64
    )
}

/// The Orbit-level atlas of `body` at `epoch_gyr`: every surface stage but
/// the feature register, on the coarse grid (`survey::orbit_world`), with
/// only features at least two coarse cells across. A world that ever had
/// moving plates gets heights from the coarse plate run, the run a survey
/// refines, under `shell:orbit@`; one that never did keeps the regime
/// sketch as its height source at every level (`shell:sketch@`).
pub fn orbit_atlas(
    s: &SurfaceSettings,
    body: &Body,
    seed: &Seed,
    epoch_gyr: f64,
) -> Result<Atlas, String> {
    crate::survey::orbit_world(s, body, seed, epoch_gyr, &SurveyOptions::default())
        .map(|sv| sv.atlas)
}
