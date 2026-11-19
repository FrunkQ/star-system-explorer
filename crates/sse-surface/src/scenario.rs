//! S0 scenarios: the base-surface pack, the fixture bodies and the golden.

use crate::{envelope, envelopes_of, sketch, Body, SurfaceSettings, SurveyState};
use serde::Deserialize;
use sse_core::draw::Seed;
use sse_pack::PackManifest;

pub fn base_surface_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-surface/pack.json"))
        .expect("base-surface pack")
}

pub fn settings() -> SurfaceSettings {
    SurfaceSettings::from_packs(&[base_surface_pack()]).expect("surface settings")
}

#[derive(Deserialize)]
pub struct Fixture {
    pub expect: String,
    pub body: Body,
}

#[derive(Deserialize)]
struct Fixtures {
    bodies: Vec<Fixture>,
}

/// Solar System and reference worlds with the regime each should get.
pub fn fixtures() -> Vec<Fixture> {
    let f: Fixtures =
        serde_json::from_str(include_str!("../../../tests/fixtures/surface/bodies.json"))
            .expect("surface fixtures");
    f.bodies
}

fn digest<T: serde::Serialize>(t: &T) -> String {
    let text = serde_json::to_string(t).expect("serialisable");
    sse_core::ContentHash::of(&text).expect("hash").hex()
}

/// Golden: every fixture's envelope and scanned sketch, and the envelopes of
/// four G0 systems through the formation pipeline.
pub fn golden() -> serde_json::Value {
    let s = settings();
    let seed = Seed::from_text("sse4-surface-fixtures");
    let fx: Vec<_> = fixtures()
        .iter()
        .map(|f| {
            let e = envelope(&s, &f.body);
            let sk = sketch(&s, &e, &seed, SurveyState::Scanned);
            serde_json::json!({
                "body": f.body.label, "regime": e.regime_now, "kind": e.kind,
                "events": e.events.iter().map(|x| serde_json::json!({"t_gyr": x.t_gyr, "to": x.to})).collect::<Vec<_>>(),
                "envelope": digest(&e), "sketch": digest(&sk),
            })
        })
        .collect();
    let gen: Vec<_> = sse_formation::scenario::solar_patch()
        .into_iter()
        .flatten()
        .take(4)
        .map(|h| {
            let run = sse_formation::pipeline::run(h, sse_formation::pipeline::Block::Modern);
            let es = envelopes_of(&run, &s);
            serde_json::json!({
                "system": run.history.manifest.system.to_string(),
                "regimes": es.iter().map(|e| e.regime_now.clone()).collect::<Vec<_>>(),
                "digest": digest(&es),
            })
        })
        .collect();
    serde_json::json!({ "version": crate::SURFACE_VERSION, "fixtures": fx, "generated": gen })
}

/// Golden for S1: tile hashes for several fixture worlds, built alone, at 32
/// samples a tile so the browser harness stays quick.
pub fn atlas_golden() -> serde_json::Value {
    use crate::atlas::{Atlas, TileKey};
    let mut s = settings();
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-atlas-golden");
    let keys = [
        (0, 0, 0, 0),
        (1, 0, 0, 0),
        (2, 0, 0, 0),
        (3, 0, 0, 0),
        (4, 0, 0, 0),
        (5, 0, 0, 0),
        (2, 6, 40, 17),
        (4, 13, 5000, 7000),
    ];
    let out: Vec<_> = fixtures()
        .iter()
        .filter(|f| ["Earth", "Mars", "Titan", "Vesta", "Jupiter", "55 Cancri e"].contains(&f.body.label.as_str()))
        .map(|f| {
            let e = envelope(&s, &f.body);
            let a = Atlas::new(&s, &e, &seed, f.body.age_gyr, vec![]);
            let tiles: Vec<_> = keys
                .iter()
                .map(|&(face, level, x, y)| {
                    let t = a.build(TileKey::new(face, level, x, y).expect("key"));
                    serde_json::json!({"tile": t.key.text(), "octaves": t.octaves, "hash": t.hash})
                })
                .collect();
            serde_json::json!({"body": f.body.label, "atlas": a.address.to_string(), "regime": a.regime, "tiles": tiles})
        })
        .collect();
    serde_json::json!({"version": crate::atlas::ATLAS_VERSION, "worlds": out})
}
