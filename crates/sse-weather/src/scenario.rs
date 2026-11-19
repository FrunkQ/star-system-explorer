//! Weather scenarios: the base-weather pack, fixture worlds and goldens.

use crate::{Conditions, Date, RuleBlock, WeatherBlock, WeatherSettings};
use serde::Deserialize;
use sse_pack::PackManifest;
use sse_surface::atlas::TileKey;

pub fn base_weather_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-weather/pack.json"))
        .expect("base-weather pack")
}

pub fn settings() -> WeatherSettings {
    WeatherSettings::from_packs(&[base_weather_pack()]).expect("weather settings")
}

pub fn block() -> RuleBlock {
    RuleBlock::from_packs(&[base_weather_pack()]).expect("weather block")
}

#[derive(Deserialize)]
pub struct Fixture {
    pub expect: String,
    pub conditions: Conditions,
}

#[derive(Deserialize)]
struct Fixtures {
    worlds: Vec<Fixture>,
}

pub fn fixtures() -> Vec<Fixture> {
    let f: Fixtures =
        serde_json::from_str(include_str!("../../../tests/fixtures/weather/worlds.json"))
            .expect("weather fixtures");
    f.worlds
}

pub fn fixture(label: &str) -> Conditions {
    fixtures()
        .into_iter()
        .find(|f| f.conditions.label == label)
        .unwrap_or_else(|| panic!("no weather fixture {label}"))
        .conditions
}

fn digest<T: serde::Serialize>(t: &T) -> String {
    let text = serde_json::to_string(t).expect("serialisable");
    sse_core::ContentHash::of(&text).expect("hash").hex()
}

/// Golden: every fixture's regime, climate digest and one field tile digest.
pub fn golden() -> serde_json::Value {
    let b = block();
    let key = TileKey::new(0, 1, 0, 0).expect("tile");
    let rows: Vec<_> = fixtures()
        .iter()
        .map(|f| {
            let c = &f.conditions;
            let cl = b.climate(c);
            let t = b.field(c, Date { day: 100.0 }, key, 8, &[]);
            serde_json::json!({
                "world": c.label, "regime": cl.regime, "condensate": cl.condensate,
                "climate": digest(&cl), "field": digest(&t),
            })
        })
        .collect();
    serde_json::json!({ "version": crate::WEATHER_VERSION, "block": b.id(), "worlds": rows })
}

/// Golden for W2: Earth's systems on two dates, a field tile and a report;
/// identical on every device.
pub fn storms_golden() -> serde_json::Value {
    let b = block();
    let e = fixture("Earth");
    let dates = [Date { day: 100.0 }, Date { day: 250.5 }];
    let sys: Vec<_> = dates
        .iter()
        .map(|d| {
            let s = b.systems(&e, *d, &[]);
            serde_json::json!({
                "day": d.day, "count": s.len(),
                "ids": s.iter().map(|x| x.id.clone()).collect::<Vec<_>>(), "digest": digest(&s),
            })
        })
        .collect();
    let key = TileKey::new(1, 2, 1, 1).expect("tile");
    let tile = b.field(&e, dates[1], key, 8, &[]);
    let r = b.report(&e, dates[1], 15.0, -60.0, &[], None);
    serde_json::json!({ "systems": sys, "tile": digest(&tile), "report": r.lines })
}
