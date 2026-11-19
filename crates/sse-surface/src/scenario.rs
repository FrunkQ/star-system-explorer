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

/// Golden for S2: the Earth fixture's plate history on a 16-cell-a-face grid,
/// and tiles of its surveyed atlas.
pub fn plates_golden() -> serde_json::Value {
    use crate::atlas::{Atlas, TileKey};
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-plates-golden");
    let out: Vec<_> = fixtures()
        .iter()
        .filter(|f| ["Earth", "Wet super-Earth", "Mars"].contains(&f.body.label.as_str()))
        .map(|f| {
            let e = envelope(&s, &f.body);
            let h = crate::plates::history(&s, &e, &seed, f.body.age_gyr);
            let a = Atlas::from_plates(&s, &e, &seed, &h, f.body.oceans, vec![]);
            let tiles: Vec<_> = [(0, 0, 0, 0), (2, 3, 4, 5), (5, 10, 700, 300)]
                .iter()
                .map(|&(face, level, x, y)| {
                    a.build(TileKey::new(face, level, x, y).expect("key")).hash
                })
                .collect();
            serde_json::json!({
                "body": f.body.label, "steps": h.steps, "plates": h.plates.len(),
                "events": h.events.len(), "snapshots": h.snapshots.len(),
                "history": digest(&h), "tiles": tiles,
            })
        })
        .collect();
    serde_json::json!({"version": crate::plates::PLATES_VERSION, "worlds": out})
}

/// Golden for S3: wearing down on a 16-cell-a-face grid for Earth (plates, wet)
/// and Mars (sketch, wet for its first Gyr), and tiles of the worn atlas.
pub fn wear_golden() -> serde_json::Value {
    use crate::atlas::{Atlas, TileKey};
    use crate::wear::{wear_world, WaterHistory};
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-wear-golden");
    let out: Vec<_> = fixtures()
        .iter()
        .filter(|f| ["Earth", "Mars"].contains(&f.body.label.as_str()))
        .map(|f| {
            let e = envelope(&s, &f.body);
            let h = crate::plates::history(&s, &e, &seed, f.body.age_gyr);
            let water = if f.body.label == "Mars" {
                WaterHistory { wet_from_gyr: 0.0, wet_until_gyr: 1.0, oceans: 0.05, pressure_bar: 0.006 }
            } else {
                WaterHistory::steady(f.body.oceans, f.body.pressure_bar, f.body.age_gyr)
            };
            let w = wear_world(&s, &e, &seed, &h, &water, f.body.magma_end_myr.max(50.0));
            let a = Atlas::from_worn(&s, &e, &seed, &h, &w, vec![]);
            let tiles: Vec<_> = [(0, 0, 0, 0), (3, 5, 9, 20)]
                .iter()
                .map(|&(face, level, x, y)| a.build(TileKey::new(face, level, x, y).expect("key")).hash)
                .collect();
            serde_json::json!({
                "body": f.body.label, "epochs": w.epochs.len(),
                "rivers": w.river.iter().filter(|x| **x).count(), "lakes": w.lake.iter().filter(|x| **x).count(),
                "eroded_m3": w.eroded_m3, "wear": digest(&w), "tiles": tiles,
            })
        })
        .collect();
    serde_json::json!({"version": crate::wear::WEAR_VERSION, "worlds": out})
}
