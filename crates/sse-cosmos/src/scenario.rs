//! The G0 golden scenario: the Sun's neighbourhood now, a 16 pc leaf,
//! resolved to its systems. Used by the tests and the browser harness.

use crate::{form_patch, PatchModel, Site, SystemBirth};
use sse_chronicle::Chronicle;
use sse_core::{Address, ContentHash, Seed};

pub fn seed() -> Seed {
    Seed::from_text("g0")
}
pub fn address() -> Address {
    "u:g0/patch:9:solar".parse().expect("valid address")
}

/// The envelope fixture formation stages read: the solar patch envelope and
/// its cohort envelopes, as JSON.
pub fn fixture() -> serde_json::Value {
    let model = PatchModel::milky_way_like();
    let mut c = Chronicle::new();
    let f = form_patch(&mut c, &seed(), &model, address(), &Site::solar(&model)).expect("forms");
    serde_json::json!({
        "about": "G0 envelope fixture: the default Milky-Way-like patch at the Sun's radius now (16 pc leaf), its conditions, and its cohort envelopes. Regenerate with SSE_BLESS=1 cargo test -p sse-cosmos.",
        "seed": "g0",
        "conditions": f.conditions,
        "patch": f.envelope,
        "cohorts": f.cohorts.iter().map(|(_, e)| e).collect::<Vec<_>>(),
    })
}

pub fn golden() -> serde_json::Value {
    let model = PatchModel::milky_way_like();
    let mut c = Chronicle::new();
    let f = form_patch(&mut c, &seed(), &model, address(), &Site::solar(&model)).expect("forms");
    let births: Vec<SystemBirth> = f
        .systems
        .iter()
        .map(|(a, r)| SystemBirth::from_facts(a, &r.facts).expect("typed"))
        .collect();
    let cond = serde_json::to_string(&f.conditions).expect("json");
    serde_json::json!({
        "systems": births.len(),
        "events": c.len(),
        "state": c.state_at(f.envelope.epoch).hash().hex(),
        "births_digest": ContentHash::of(&serde_json::to_string(&births).expect("json")).expect("hash").hex(),
        "conditions_digest": ContentHash::of(&cond).expect("hash").hex(),
    })
}
