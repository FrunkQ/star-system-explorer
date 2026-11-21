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

/// Golden: impacts and volcanism on four fixtures (counts, digests, two
/// tile hashes each from the cratered atlas).
pub fn impacts_golden() -> serde_json::Value {
    use crate::atlas::{Atlas, TileKey};
    use crate::impacts::{impacts, ImpactHistory, ImpactInputs};
    use crate::wear::WaterHistory;
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-impacts-golden");
    let out: Vec<_> = fixtures()
        .iter()
        .filter(|f| ["Moon", "Mars", "Venus", "Ganymede"].contains(&f.body.label.as_str()))
        .map(|f| {
            let e = envelope(&s, &f.body);
            let h = crate::plates::history(&s, &e, &seed, f.body.age_gyr);
            let water = WaterHistory::steady(f.body.oceans, f.body.pressure_bar, f.body.age_gyr);
            let hist = ImpactHistory::lunar();
            let inp = ImpactInputs {
                history: &hist,
                water: &water,
                pressure_bar: f.body.pressure_bar,
                plates: Some(&h),
            };
            let (feat, field) = impacts(&s, &e, &seed, &inp, f.body.age_gyr).expect("known mechanisms");
            let a = Atlas::new(&s, &e, &seed, f.body.age_gyr, vec![]).with_impacts(feat.clone(), field);
            let tiles: Vec<_> = [(0, 0, 0, 0), (3, 5, 9, 20)]
                .iter()
                .map(|&(face, level, x, y)| a.build(TileKey::new(face, level, x, y).expect("key")).hash)
                .collect();
            serde_json::json!({
                "body": f.body.label, "listed_d_km": feat.listed_d_km,
                "craters": feat.craters.len(), "plains": feat.plains.len(), "shields": feat.shields.len(),
                "erased": feat.erased, "features": digest(&feat), "tiles": tiles,
            })
        })
        .collect();
    serde_json::json!({"version": crate::impacts::IMPACTS_VERSION, "worlds": out})
}

/// Golden: S6 regime landforms (counts, digests, two tile hashes each).
pub fn regimes_golden() -> serde_json::Value {
    use crate::atlas::{Atlas, TileKey};
    use crate::regimes::{regime_surface, RegimeInputs};
    let mut s = settings();
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-regimes-golden");
    let out: Vec<_> = fixtures()
        .iter()
        .filter(|f| {
            ["Europa", "Io", "Venus", "Mercury", "Titan", "Jupiter"]
                .contains(&f.body.label.as_str())
        })
        .map(|f| {
            let e = envelope(&s, &f.body);
            let rot = (f.body.label == "Jupiter").then_some(9.93);
            let inp = RegimeInputs {
                rotation_hours: rot,
                mass_earth: f.body.mass_earth,
                impacts: None,
                craters: None,
            };
            let rs = regime_surface(&s, &e, &seed, &inp, f.body.age_gyr);
            let a = Atlas::new(&s, &e, &seed, f.body.age_gyr, vec![]).with_regime(rs.clone());
            let tiles: Vec<_> = [(0, 0, 0, 0), (3, 5, 9, 20)]
                .iter()
                .map(|&(face, level, x, y)| {
                    a.build(TileKey::new(face, level, x, y).expect("key")).hash
                })
                .collect();
            serde_json::json!({
                "body": f.body.label, "regime": rs.regime, "landforms": rs.landforms.len(),
                "overturns": rs.overturns.len(), "jets": rs.clouds.as_ref().map(|c| c.jets),
                "surface": digest(&rs), "tiles": tiles,
            })
        })
        .collect();
    serde_json::json!({"version": crate::regimes::REGIMES_VERSION, "worlds": out})
}

/// Golden: S7 on Earth (worn plates, settled, a GM range and pin, life) and
/// the Moon (craters, polar ice): layer digests, register counts and names,
/// tile hashes; and the Venus aerostat band.
pub fn cover_golden() -> serde_json::Value {
    use crate::atlas::Atlas;
    use crate::cover::{aerostat_band, layers, Air, CoverInputs, Reshape};
    use crate::impacts::{impacts, ImpactHistory, ImpactInputs};
    use crate::named::register;
    use crate::wear::{wear_world, WaterHistory};
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-cover-golden");
    let body = |n: &str| {
        fixtures()
            .into_iter()
            .find(|f| f.body.label == n)
            .expect("fixture")
            .body
    };
    let keys = [(0u8, 0u8, 0u32, 0u32), (4, 2, 1, 2), (3, 5, 9, 20)];
    let mut out = vec![];
    {
        let b = body("Earth");
        let e = envelope(&s, &b);
        let h = crate::plates::history(&s, &e, &seed, b.age_gyr);
        let w = wear_world(
            &s,
            &e,
            &seed,
            &h,
            &WaterHistory::steady(b.oceans, b.pressure_bar, b.age_gyr),
            b.magma_end_myr,
        );
        let a = Atlas::from_worn(&s, &e, &seed, &h, &w, vec![]);
        let epoch = a.epoch_myr as f64;
        let inp = CoverInputs {
            air: Air::guess(&s.cover, &e, b.pressure_bar, true),
            water: true,
            rotation_hours: Some(23.93),
            obliquity_deg: None,
            life: 1.0,
            plates: Some(&h),
            reshape: vec![
                Reshape::RaiseRange {
                    id: "gm:range".into(),
                    from: [10.0, 20.0],
                    to: [10.0, 40.0],
                    height_m: 5000.0,
                    width_km: 150.0,
                    at_myr: epoch - 1.0,
                },
                Reshape::Pin {
                    id: "gm:pin".into(),
                    lat_deg: 30.0,
                    lon_deg: 30.0,
                    height_m: 1200.0,
                    radius_km: 20.0,
                    at_myr: epoch,
                },
            ],
            settle: vec!["settlement".into(), "port".into(), "mine".into()],
        };
        let ly = layers(&s, &e, &seed, &a, &inp);
        let a = a.with_layers(ly);
        let reg = register(&s.features, &e, &seed, &a, &[], vec![]).expect("register");
        out.push(world_json(&a, &reg, &keys));
    }
    {
        let b = body("Moon");
        let e = envelope(&s, &b);
        let water = WaterHistory::steady(0.0, 0.0, b.age_gyr);
        let hist = ImpactHistory::lunar();
        let ii = ImpactInputs {
            history: &hist,
            water: &water,
            pressure_bar: 0.0,
            plates: None,
        };
        let (feat, field) = impacts(&s, &e, &seed, &ii, b.age_gyr).expect("known mechanisms");
        let a = Atlas::new(&s, &e, &seed, b.age_gyr, vec![]).with_impacts(feat, field);
        let inp = CoverInputs {
            air: Air::guess(&s.cover, &e, 0.0, false),
            water: false,
            rotation_hours: Some(655.7),
            obliquity_deg: None,
            life: 0.0,
            plates: None,
            reshape: vec![],
            settle: vec!["base".into()],
        };
        let ly = layers(&s, &e, &seed, &a, &inp);
        let a = a.with_layers(ly);
        let reg = register(&s.features, &e, &seed, &a, &[], vec![]).expect("register");
        out.push(world_json(&a, &reg, &keys));
    }
    let v = body("Venus");
    let ve = envelope(&s, &v);
    let band = aerostat_band(
        &s.cover,
        &ve,
        &Air::guess(&s.cover, &ve, v.pressure_bar, false),
        None,
    )
    .expect("band");
    serde_json::json!({
        "version": crate::cover::COVER_VERSION,
        "worlds": out,
        "venus_aerostat": {"low_km": band.low_km, "high_km": band.high_km, "p_bar": band.p_bar, "t_k": band.t_k, "circuit_days": band.circuit_days},
    })
}

fn world_json(
    a: &crate::atlas::Atlas,
    reg: &crate::named::Register,
    keys: &[(u8, u8, u32, u32)],
) -> serde_json::Value {
    use crate::atlas::TileKey;
    let ly = a.layers().expect("layers");
    let mut kinds = std::collections::BTreeMap::new();
    for f in &reg.features {
        *kinds.entry(f.kind.clone()).or_insert(0u32) += 1;
    }
    let tiles: Vec<_> = keys
        .iter()
        .map(|&(f, l, x, y)| a.build(TileKey::new(f, l, x, y).expect("key")).hash)
        .collect();
    serde_json::json!({
        "address": ly.address, "cells": ly.climate.cells, "contrast_k": ly.climate.contrast_k,
        "deposits": ly.deposits.len(), "built": ly.built.len(), "applied": ly.applied.len(),
        "layers": digest(ly), "features": kinds, "register": digest(reg),
        "first_names": reg.features.iter().take(5).map(|f| f.original_name.clone()).collect::<Vec<_>>(),
        "tiles": tiles,
    })
}
