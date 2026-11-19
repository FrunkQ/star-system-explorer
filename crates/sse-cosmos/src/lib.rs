//! SSE4 cosmos suite (galaxy design). Stage G0: the default Milky-Way-like
//! patch envelope, so the physics suite can form systems before any galaxy
//! exists. Later stages (G1 to G8) add cosmology, haloes, galaxies, rings,
//! populations and events above it.
#![forbid(unsafe_code)]

pub mod birth;
pub mod cosmology;
pub mod envelope;
pub mod epochs;
pub mod galaxy;
pub mod haloes;
pub mod model;
pub mod patch;
pub mod rings;
pub mod scenario;
pub mod web;

pub use birth::SystemBirth;
pub use model::PatchModel;
pub use patch::{cohort_envelope, conditions, patch_envelope, Conditions, Site};

use sse_chronicle::{Chronicle, Publisher};
use sse_core::{Address, Seed};
use sse_flow::CostClass;
use sse_resolve::{resolve_recorded, ChildRef, Envelope, Recorded};

/// A patch resolved down to its systems and recorded.
pub struct Formed {
    pub envelope: Envelope,
    pub conditions: Conditions,
    pub cohorts: Vec<(Recorded, Envelope)>,
    pub systems: Vec<(Address, Recorded)>,
}

pub fn publisher() -> Publisher {
    Publisher::new("sse.base-cosmos", "patch", "1")
}

/// Resolve and record every cohort and system of a patch. Recorded facts
/// win (an earlier resolve is read back, never redrawn).
pub fn form_patch(
    chronicle: &mut Chronicle,
    seed: &Seed,
    model: &PatchModel,
    address: Address,
    site: &Site,
) -> Result<Formed, String> {
    let (env, cond) = patch_envelope(model, address, site);
    env.validate().map_err(|e| e.to_string())?;
    let pubr = publisher();
    let mut cohorts = Vec::new();
    let mut systems = Vec::new();
    let n = sse_resolve::children(seed, &env)
        .map_err(|e| e.to_string())?
        .generated;
    for k in 0..n {
        let rec = resolve_recorded(chronicle, seed, &env, &ChildRef::Generated(k), &pubr)
            .map_err(|e| e.to_string())?;
        let res = sse_resolve::Resolved {
            address: env
                .address
                .child("cohort", k)
                .map_err(|e| format!("{e:?}"))?,
            child: ChildRef::Generated(k),
            facts: rec.facts.clone(),
            envelope: None,
            work: 0,
        };
        let cenv = cohort_envelope(model, &cond, &res).ok_or("cohort envelope")?;
        cenv.validate().map_err(|e| e.to_string())?;
        let m = sse_resolve::children(seed, &cenv)
            .map_err(|e| e.to_string())?
            .generated;
        for i in 0..m {
            let s = resolve_recorded(chronicle, seed, &cenv, &ChildRef::Generated(i), &pubr)
                .map_err(|e| e.to_string())?;
            systems.push((
                cenv.address.child("sys", i).map_err(|e| format!("{e:?}"))?,
                s,
            ));
        }
        cohorts.push((rec, cenv));
    }
    Ok(Formed {
        envelope: env,
        conditions: cond,
        cohorts,
        systems,
    })
}

/// Every operation this crate adds, with its cost class (build plan 2).
pub fn ops() -> Vec<(&'static str, CostClass, &'static str)> {
    vec![
        ("cosmos.conditions", CostClass::Trivial, "closed form; any device"),
        ("cosmos.patch_envelope", CostClass::Trivial, "closed form; any device"),
        ("cosmos.galaxy", CostClass::Light, "about 1,400 steps of a few equations per galaxy; any device"),
        ("cosmos.rings", CostClass::Medium, "24 rings x 270 steps with delay convolutions (about 3 million multiply-adds, well under a second); any device"),
        ("cosmos.envelope", CostClass::Trivial, "interpolation and closed forms per place; any device"),
        ("cosmos.green_band", CostClass::Light, "about 90 envelope reads per time; any device"),
        ("cosmos.form_patch", CostClass::Medium, "linear in the systems a patch holds (about 330 in a 16 pc leaf near the Sun); weak devices resolve only the systems a view shows"),
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
#[cfg(test)]
mod g1_tests;
#[cfg(test)]
mod g2_tests;
#[cfg(test)]
mod g3_tests;
#[cfg(test)]
mod g4_tests;
