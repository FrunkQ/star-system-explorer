//! The golden scenario: a full two-level resolve of the fixture patch plus
//! its location-first outlook. Run by the crate's tests and by the browser
//! harness, so native and browser digests are compared.

use crate::query::location_first;
use crate::{children, resolve_recorded, ChildRef, Envelope};
use sse_chronicle::{Chronicle, Publisher, UniverseTime};
use sse_core::{ContentHash, Seed};

pub const PATCH_JSON: &str = include_str!("../../../tests/fixtures/resolve/patch.json");

pub fn patch() -> Envelope {
    serde_json::from_str(PATCH_JSON).expect("fixture patch is valid")
}

fn refs(seed: &Seed, env: &Envelope) -> Vec<ChildRef> {
    let k = children(seed, env).expect("valid");
    (0..k.pins.len())
        .map(ChildRef::Pinned)
        .chain((0..k.generated).map(ChildRef::Generated))
        .collect()
}

pub fn golden() -> serde_json::Value {
    let seed = Seed::from_text("k7");
    let publisher = Publisher::new("test.resolve", "patch", "1");
    let env = patch();
    let mut c = Chronicle::new();
    for r in refs(&seed, &env) {
        let rec = resolve_recorded(&mut c, &seed, &env, &r, &publisher).expect("resolves");
        let sys = rec.envelope.expect("systems have bodies");
        for b in refs(&seed, &sys) {
            resolve_recorded(&mut c, &seed, &sys, &b, &publisher).expect("resolves");
        }
    }
    let outlook = ContentHash::of(
        &serde_json::to_string(&location_first(&env).expect("valid")).expect("json"),
    )
    .expect("hash");
    serde_json::json!({
        "resolve_tree_state": c.state_at(UniverseTime::ORIGIN).hash().hex(),
        "events": c.len(),
        "outlook": outlook.hex(),
    })
}
