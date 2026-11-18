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
    serde_json::json!({ "systems": out, "solar_patch": solar_patch_digest() })
}

/// Every system of G0's solar patch (the Sun's neighbourhood now), formed
/// from its birth facts. Returns the histories in patch order, refusals as
/// `Err` with their reasons.
pub fn solar_patch() -> Vec<Result<FormationHistory, crate::FormationError>> {
    use sse_cosmos::{form_patch, PatchModel, Site, SystemBirth};
    let model = PatchModel::milky_way_like();
    let site = Site::solar(&model);
    let mut c = sse_chronicle::Chronicle::new();
    let formed = form_patch(
        &mut c,
        &sse_cosmos::scenario::seed(),
        &model,
        sse_cosmos::scenario::address(),
        &site,
    )
    .expect("the G0 fixture patch forms");
    formed
        .systems
        .iter()
        .map(|(a, r)| {
            let birth = SystemBirth::from_facts(a, &r.facts).expect("G0 systems carry birth facts");
            FormationHistory::form(Manifest::from_birth("g0", birth, site.epoch_gyr()))
        })
        .collect()
}

/// Digest of the solar patch's formation: counts and one hash over all histories.
pub fn solar_patch_digest() -> serde_json::Value {
    let all = solar_patch();
    let mut text = String::new();
    let (mut ok, mut refused, mut multiple, mut wd, mut bd) = (0, 0, 0, 0, 0);
    for h in &all {
        match h {
            Ok(h) => {
                ok += 1;
                let now = h.state_at(h.inheritance.age_gyr * 1000.0);
                multiple += usize::from(h.stars.len() > 1);
                wd += usize::from(now.iter().any(|s| s.phase == crate::Phase::WhiteDwarf));
                bd += usize::from(now[0].phase == crate::Phase::BrownDwarf);
                text.push_str(&h.hash().hex());
            }
            Err(e) => {
                refused += 1;
                text.push_str(&e.0);
            }
        }
        text.push('\n');
    }
    serde_json::json!({
        "formed": ok,
        "refused": refused,
        "multiple": multiple,
        "with_white_dwarf_now": wd,
        "brown_dwarf_primaries": bd,
        "digest": sse_core::ContentHash::of(&text).expect("hash").hex(),
    })
}
