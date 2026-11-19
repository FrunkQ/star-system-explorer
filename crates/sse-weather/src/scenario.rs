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
    RuleBlock::new(settings())
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
