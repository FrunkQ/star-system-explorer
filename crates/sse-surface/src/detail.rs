//! S16: detail levels (Rock 3 comparison, section 5.8, accepted 21 Nov).
//!
//! Most worlds are glanced at and skipped, so a world is worked out only as
//! far as someone looks at it:
//! - **Glance** (D0): the envelope and regime, already made for every world.
//! - **Orbit** (D1): the same plate code on a coarse grid (the base pack's
//!   `plates.guide`, 32 cells a face), heights from it; no wear, craters or
//!   cover yet (later parts of S16).
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
use crate::plates::{guide_settings, history, PLATES_VERSION};
use crate::{SurfaceEnvelope, SurfaceSettings};
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
        "{DETAIL_VERSION}/{PLATES_VERSION}/orbit/{}/{}/{}",
        serde_json::to_string(seed).expect("plain data"),
        env.address,
        m::round(epoch_gyr * 1.0e6) as i64
    )
}

/// The Orbit-level atlas of a world at `epoch_gyr`: heights from the coarse
/// plate run for a world that ever had moving plates, the regime sketch for
/// one that never did (its plates never moved, so the sketch is the height
/// source at every level, as `Atlas::from_plates` already rules). Its tiles
/// live under `shell:orbit@`, apart from the surveyed `shell:surface@`.
pub fn orbit_atlas(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    epoch_gyr: f64,
    oceans: f64,
) -> Atlas {
    let o = orbit_settings(s);
    let mobile = env.timeline.iter().any(|x| x.regime == "mobile-lid");
    if !mobile {
        return Atlas::new(&o, env, seed, epoch_gyr, vec![]);
    }
    let h = history(&o, env, seed, epoch_gyr);
    let mut a = Atlas::from_plates(&o, env, seed, &h, oceans, vec![]);
    if a.has_plates() {
        a.address = env
            .address
            .child_key("shell", &format!("orbit@{}", a.epoch_myr))
            .expect("valid shell key");
    }
    a
}
