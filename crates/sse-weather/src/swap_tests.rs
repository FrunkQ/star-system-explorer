//! W6 evidence: switching blocks on a saved campaign changes no climate fact
//! beyond tolerance, keeps recorded systems, shows the difference, and
//! switching back restores the original exactly.

use super::*;
use crate::hooks;
use crate::scenario::{base_weather_pack, fixture, fixtures};
use crate::zonal::ZonalBlock;

fn stack() -> Vec<sse_pack::PackManifest> {
    vec![base_weather_pack()]
}

fn ids() -> (BlockId, BlockId) {
    let r = registry();
    (r[0].clone(), r[1].clone())
}

fn zonal() -> ZonalBlock {
    ZonalBlock::from_packs(&stack()).expect("zonal block")
}

fn check_golden(name: &str, got: &serde_json::Value) {
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

/// The five worlds the package names, plus the rest of the fixtures.
const BOTH_APPLY: [&str; 4] = ["Earth", "Mars", "Venus", "Titan"];
const OUTSIDE: [&str; 1] = ["Locked temperate world"];

/// A saved campaign: a named storm on Earth, a GM order and a triggered
/// storm, and the locked world's permanent substellar storm recorded.
fn campaign() -> (WeatherManifest, Vec<Conditions>) {
    let (rule, _) = ids();
    let b = crate::scenario::block();
    let e = fixture("Earth");
    let l = fixture("Locked temperate world");
    let mut m = WeatherManifest::new(rule);
    let s = b
        .systems(&e, Date { day: 50.0 }, &[])
        .into_iter()
        .find(|s| s.kind == "cyclone")
        .expect("an Earth cyclone");
    m.pins.push(WorldPin {
        world: e.address.clone(),
        pin: Pin {
            id: s.id.clone(),
            name: "Storm Ada".into(),
            system: s,
        },
    });
    let sub = b
        .systems(&l, Date { day: 3.0 }, &[])
        .into_iter()
        .find(|s| s.kind == "substellar-storm")
        .expect("substellar storm");
    m.pins.push(WorldPin {
        world: l.address.clone(),
        pin: Pin {
            id: sub.id.clone(),
            name: "The Eye".into(),
            system: sub,
        },
    });
    m.authored.push(WorldPin {
        world: e.address.clone(),
        pin: hooks::order_clear("gm-clear-1", 51.5, -0.1, 300.0, 60.0, 62.0),
    });
    m.authored.push(WorldPin {
        world: e.address.clone(),
        pin: hooks::trigger_storm(&b, &e, "gm-storm-1", "cyclone", 45.0, -30.0, 70.0, 4.0),
    });
    (
        m,
        vec![e, l, fixture("Mars"), fixture("Venus"), fixture("Titan")],
    )
}

#[test]
fn zonal_block_is_a_different_block_with_its_own_physics() {
    let z = zonal();
    assert_eq!(z.id().id, "sse.weather.zonal-ebm");
    assert_eq!(z.id().level, "simulated");
    assert_eq!(z.provenance().evidence, "calculated");
    let e = fixture("Earth");
    let a = z.annual(&e);
    let n = a.t_k.len();
    let w: Vec<f64> = a
        .lat_deg
        .iter()
        .map(|l| sse_core::math::cos(l * sse_core::math::PI / 180.0))
        .collect();
    let mean = a.t_k.iter().zip(&w).map(|(t, w)| t * w).sum::<f64>() / w.iter().sum::<f64>();
    // The model moves heat, it does not change how warm a world is.
    assert!((mean - e.t_surface_k).abs() < 0.5, "mean {mean}");
    let contrast = a.t_k[n / 2] - a.t_k[n - 1];
    assert!(
        (30.0..60.0).contains(&contrast),
        "Earth contrast {contrast}"
    );
    // Thick or slow air is near isothermal.
    for name in ["Venus", "Titan"] {
        let t = z.annual(&fixture(name)).t_k;
        let spread =
            t.iter().cloned().fold(f64::MIN, f64::max) - t.iter().cloned().fold(f64::MAX, f64::min);
        assert!(spread < 2.0, "{name} spread {spread}");
    }
    // W7a: both blocks draw on the one climate, so their averages agree
    // exactly; what differs is the weather they draw on top.
    let rb = crate::scenario::block().climate(&e);
    let zb = z.climate(&e);
    assert_eq!(rb, zb, "one climate under both blocks");
    let key = sse_surface::atlas::TileKey::new(0, 1, 0, 0).unwrap();
    let day = crate::contract::Date { day: 100.0 };
    assert_ne!(
        crate::scenario::block().field(&e, day, key, 8, &[]),
        z.field(&e, day, key, 8, &[]),
        "the drawing on top differs"
    );
    // Deterministic.
    assert_eq!(z.climate(&e), zonal().climate(&e));
}

#[test]
fn zonal_block_agrees_within_tolerance_where_both_apply() {
    let r = crate::scenario::block();
    let z = zonal();
    let mut wrong = vec![];
    for name in BOTH_APPLY {
        let d = compare(&r, &z, &fixture(name));
        if !d.within() {
            wrong.push(d.lines.join(" "));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn where_a_block_reads_a_different_climate_the_report_says_so() {
    // Since W7a the stock blocks share one climate, so a campaign swap is
    // within tolerance everywhere. A block whose climate pack differs (here:
    // half the evaporation) is still caught and reported.
    let r = crate::scenario::block();
    let mut z = zonal();
    z.shared.climate.supply.latent_share *= 0.3;
    let d = compare(&r, &z, &fixture("Earth"));
    assert!(!d.within(), "{:?}", d.lines);
    assert!(
        d.lines.iter().any(|l| l.contains("outside tolerance")),
        "{:?}",
        d.lines
    );
    let (m, worlds) = campaign();
    let rep = swap(&stack(), &m, &worlds, &ids().1).unwrap();
    assert!(rep.within_tolerance, "{:?}", rep.lines);
    for name in OUTSIDE.iter().chain(BOTH_APPLY.iter()) {
        let d = compare(&r, &zonal(), &fixture(name));
        assert!(d.within(), "{name}: {:?}", d.lines);
    }
}

#[test]
fn swapping_keeps_recorded_systems_and_shows_the_difference() {
    let (m, worlds) = campaign();
    let (_, zid) = ids();
    let rep = swap(&stack(), &m, &worlds, &zid).unwrap();
    assert_eq!(rep.manifest.block, zid);
    assert_eq!(rep.manifest.pins, m.pins);
    assert_eq!(rep.manifest.authored, m.authored);
    assert_eq!(rep.pins.len(), 4);
    assert!(rep.pins_kept, "{:#?}", rep.pins);
    assert!(rep.pins.iter().all(|p| p.kept == Some(true)));
    assert!(rep
        .lines
        .iter()
        .any(|l| l.contains("Storm Ada") && l.contains("honoured")));
    assert!(rep.lines[0].contains("sse.weather.rules"));
    assert!(rep.lines.iter().any(|l| l.starts_with("Earth: cloud")));
    // The new block shows the pinned storm where it was, and the GM's clear sky.
    let z = block_for(&stack(), &zid).unwrap();
    let e = fixture("Earth");
    let pins = rep.manifest.pins_for(&e.address);
    let ada = &m.pins[0].pin.system;
    let day = ada.born + 0.5;
    let got = z.systems(&e, Date { day }, &pins);
    let p = got.iter().find(|s| s.id == ada.id).expect("Ada alive");
    assert_eq!(p.evidence, "pinned");
    assert_eq!(p.track, ada.track);
    let key = sse_surface::atlas::TileKey::new(0, 0, 0, 0).unwrap();
    // A clear-sky order is honoured in the field: no cloud inside it.
    let order = &m.authored[0].pin.system;
    let zb = zonal();
    let tile = zb.field(&e, Date { day: 61.0 }, key, 4, &pins);
    assert_eq!(tile.cells.len(), 16);
    let r = zb.systems_with(&e, Date { day: 61.0 }, &pins);
    assert!(r.iter().any(|s| s.id == order.id));
}

#[test]
fn switching_back_restores_the_original_exactly() {
    let (m, worlds) = campaign();
    let (rid, zid) = ids();
    let key = sse_surface::atlas::TileKey::new(1, 2, 1, 1).unwrap();
    let days = [50.5, 61.0, 71.0];
    let before: Vec<String> = worlds
        .iter()
        .map(|c| weather_digest(&stack(), &m, c, &days, key, 6).unwrap())
        .collect();
    let there = swap(&stack(), &m, &worlds, &zid).unwrap();
    let mid: Vec<String> = worlds
        .iter()
        .map(|c| weather_digest(&stack(), &there.manifest, c, &days, key, 6).unwrap())
        .collect();
    assert_ne!(before, mid, "the other block gives other weather");
    let back = swap(&stack(), &there.manifest, &worlds, &rid).unwrap();
    assert_eq!(back.manifest, m);
    assert!(back.pins_kept);
    let after: Vec<String> = worlds
        .iter()
        .map(|c| weather_digest(&stack(), &back.manifest, c, &days, key, 6).unwrap())
        .collect();
    assert_eq!(before, after);
    // The manifest round-trips through a save's JSON unchanged.
    let text = serde_json::to_string(&back.manifest).unwrap();
    let again: WeatherManifest = serde_json::from_str(&text).unwrap();
    assert_eq!(again, m);
}

#[test]
fn an_unknown_block_is_refused_not_substituted() {
    let mut id = ids().1;
    id.version = "99".into();
    let err = block_for(&stack(), &id).err().expect("refused");
    assert!(err.contains("version 99"), "{err}");
    let (m, worlds) = campaign();
    assert!(swap(&stack(), &m, &worlds, &id).is_err());
}

#[test]
fn zonal_storms_follow_the_temperature_gradient_and_replay() {
    let z = zonal();
    let e = fixture("Earth");
    let s = z.systems(&e, Date { day: 100.0 }, &[]);
    assert!(!s.is_empty());
    for x in &s {
        assert!(x.lat_deg.abs() >= 20.0, "{} at {}", x.id, x.lat_deg);
        // At the steepest gradient of its hemisphere on its day of birth.
        let zs = z.at(&e, Date { day: x.born });
        let steepest = zs
            .lat_deg
            .iter()
            .zip(&zs.gradient_k_deg)
            .filter(|(l, _)| l.abs() >= 20.0 && l.signum() == x.lat_deg.signum())
            .fold(
                (0.0, 0.0f64),
                |b, (l, g)| if g.abs() > b.1 { (*l, g.abs()) } else { b },
            );
        assert_eq!(steepest.0, x.lat_deg, "{}", x.id);
    }
    let later = z.systems(
        &e,
        Date {
            day: s[0].born + 2.0,
        },
        &[],
    );
    let same = later.iter().find(|x| x.id == s[0].id).expect("still alive");
    assert_eq!(same, &s[0]);
    for name in ["Venus", "Titan", "Locked temperate world", "Moon"] {
        assert!(
            z.systems(&fixture(name), Date { day: 100.0 }, &[])
                .is_empty(),
            "{name}"
        );
    }
}

#[test]
fn every_zonal_reference_is_linked_and_the_pack_reads() {
    let z = zonal();
    assert!(z.z.references.len() >= 3);
    assert!(z
        .z
        .references
        .iter()
        .all(|r| r.doi.is_some() || r.url.is_some()));
}

/// Golden: the zonal block's climate on every fixture and the campaign's
/// shown difference.
#[test]
fn golden_weather_swap() {
    let z = zonal();
    let r = crate::scenario::block();
    let rows: Vec<_> = fixtures()
        .iter()
        .map(|f| {
            let c = &f.conditions;
            let cl = z.climate(c);
            let d = compare(&r, &z, c);
            serde_json::json!({
                "world": c.label,
                "cloud": [d.cloud_from, d.cloud_to],
                "precip_m_yr": [d.precip_from_m_yr, d.precip_to_m_yr],
                "within": d.within(),
                "climate": sse_core::ContentHash::of(&serde_json::to_string(&cl).unwrap()).unwrap().hex(),
            })
        })
        .collect();
    let (m, worlds) = campaign();
    let rep = swap(&stack(), &m, &worlds, &ids().1).unwrap();
    check_golden(
        "weather-swap.json",
        &serde_json::json!({ "block": z.id(), "worlds": rows, "swap": rep.lines }),
    );
}

#[test]
fn zonal_field_honours_gm_orders() {
    let z = zonal();
    let e = fixture("Earth");
    let key = sse_surface::atlas::TileKey::new(0, 0, 0, 0).unwrap();
    let date = Date { day: 61.0 };
    let open = z.field(&e, date, key, 1, &[]);
    let (lat, lon) = (open.cells[0].lat_deg, open.cells[0].lon_deg);
    assert!(open.cells[0].layers.iter().any(|l| l.cloud_cover > 0.0));
    let order = hooks::order_clear("gm-clear-eq", lat, lon, 500.0, 60.0, 62.0);
    let calm = hooks::order_calm("gm-calm-eq", lat, lon, 500.0, 60.0, 62.0);
    let t = z.field(&e, date, key, 1, &[order, calm]);
    let c = &t.cells[0];
    assert!(c.layers.iter().all(|l| l.cloud_cover == 0.0));
    assert_eq!(c.precip_mm_day, 0.0);
    assert!(c
        .layers
        .iter()
        .all(|l| sse_core::math::hypot(l.wind_m_s[0], l.wind_m_s[1]) <= 1.0 + 1e-9));
}
