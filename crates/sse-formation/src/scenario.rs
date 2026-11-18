//! Golden scenario, run natively and in every browser by the harness: a
//! fixture set of systems formed from the stand-in envelope.

use crate::history::{FormationHistory, Manifest};
use crate::inherit::PatchEnvelope;
use sse_core::Address;

pub const ENVELOPE_JSON: &str =
    include_str!("../../../tests/fixtures/formation/solar-neighbourhood.json");

pub fn envelope() -> PatchEnvelope {
    PatchEnvelope::from_json(ENVELOPE_JSON).expect("fixture envelope is valid")
}

pub fn system(n: u64) -> Address {
    Address::universe("f0")
        .expect("valid")
        .child("sys", n)
        .expect("valid")
}

/// 24 systems: their history hashes, star counts and a few slice values.
pub fn golden() -> serde_json::Value {
    let env = envelope();
    let mut out = vec![];
    for n in 0..24 {
        let h = FormationHistory::form(Manifest::new("f0-golden", system(n), env.clone()));
        out.push(match h {
            Ok(h) => serde_json::json!({
                "system": n,
                "stars": h.stars.len(),
                "primary_mass": h.stars[0].mass,
                "events": h.events.len(),
                "slices": h.slices.len(),
                "hash": h.hash().hex(),
            }),
            Err(e) => serde_json::json!({ "system": n, "refused": e.0 }),
        });
    }
    serde_json::json!({ "systems": out })
}
