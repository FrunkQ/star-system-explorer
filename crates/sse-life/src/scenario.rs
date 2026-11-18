//! L0 fixture worlds and golden: habitat graphs for Earth, Mars, Europa,
//! Titan and Venus from `tests/fixtures/worlds/`, with the base-life rules.

use crate::habitat::{self, HabitatGraph, HabitatRule};
use sse_pack::PackManifest;
use sse_world::WorldStack;

pub const WORLDS: [(&str, &str); 5] = [
    (
        "earth",
        include_str!("../../../tests/fixtures/worlds/earth.json"),
    ),
    (
        "mars",
        include_str!("../../../tests/fixtures/worlds/mars.json"),
    ),
    (
        "europa",
        include_str!("../../../tests/fixtures/worlds/europa.json"),
    ),
    (
        "titan",
        include_str!("../../../tests/fixtures/worlds/titan.json"),
    ),
    (
        "venus",
        include_str!("../../../tests/fixtures/worlds/venus.json"),
    ),
];

pub fn base_life_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-life/pack.json"))
        .expect("base life pack")
}
pub fn base_rules() -> Vec<HabitatRule> {
    habitat::rules(&[base_life_pack()]).expect("base life rules")
}
pub fn world(name: &str) -> WorldStack {
    let (_, s) = WORLDS
        .iter()
        .find(|(n, _)| *n == name)
        .expect("fixture world");
    WorldStack::from_json(s).expect("world stack")
}
pub fn graph(name: &str) -> HabitatGraph {
    HabitatGraph::derive(&world(name), &base_rules()).expect("habitat graph")
}

pub fn golden() -> serde_json::Value {
    let worlds: serde_json::Map<String, serde_json::Value> = WORLDS
        .iter()
        .map(|(n, _)| {
            let g = graph(n);
            (
                n.to_string(),
                serde_json::json!({ "hash": g.hash().hex(), "summary": g.summary() }),
            )
        })
        .collect();
    serde_json::json!({ "worlds": worlds })
}
