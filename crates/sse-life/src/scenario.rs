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

/// Box B settings from the base life pack.
pub fn budget_params() -> crate::budget::BudgetParams {
    crate::budget::BudgetParams::from_packs(&[base_life_pack()]).expect("life budget")
}

/// Every metabolism in the base pack.
pub fn all_metabolisms() -> Vec<String> {
    budget_params()
        .metabolisms
        .iter()
        .map(|m| m.id.clone())
        .collect()
}

/// Metabolisms before oxygenic photosynthesis (Archean Earth).
pub fn archean_metabolisms() -> Vec<String> {
    ["anoxygenic-iron", "anoxygenic-hydrogen", "methanogen"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Earth about 3.5 Gyr ago for box B: today's fixture with a ferrous-iron
/// ocean (hydrothermal Fe2+ about 1e15 mol per year, enough for Canfield et
/// al. 2006's 1.7e14 to 5e14 mol C per year at 4 Fe per C).
pub fn archean_earth() -> HabitatGraph {
    let (_, s) = WORLDS.iter().find(|(n, _)| *n == "earth").expect("earth");
    let mut v: serde_json::Value = serde_json::from_str(s).expect("earth json");
    for sh in v["shells"].as_array_mut().expect("shells") {
        if sh["key"] == "ocean" {
            sh["properties"]["material.fe2_mol_yr"] = serde_json::json!(1.0e15);
        }
    }
    let w = WorldStack::from_json(&v.to_string()).expect("archean world");
    HabitatGraph::derive(&w, &base_rules()).expect("archean graph")
}

/// The fixture worlds' budgets: (name, budget) for every world with all the
/// base metabolisms, plus Archean Earth with the early ones.
pub fn budgets() -> Vec<(String, crate::budget::WorldBudget)> {
    let bp = budget_params();
    let all = all_metabolisms();
    let mut out: Vec<(String, crate::budget::WorldBudget)> = WORLDS
        .iter()
        .map(|(n, _)| {
            (
                n.to_string(),
                crate::budget::world_budget(&graph(n), &bp, &all),
            )
        })
        .collect();
    out.push((
        "earth-archean".into(),
        crate::budget::world_budget(&archean_earth(), &bp, &archean_metabolisms()),
    ));
    out
}

/// L1a golden: each world's totals and every habitat's verdict.
pub fn budget_golden() -> serde_json::Value {
    let worlds: serde_json::Map<String, serde_json::Value> = budgets()
        .into_iter()
        .map(|(n, b)| {
            let habitats: Vec<serde_json::Value> = b
                .habitats
                .iter()
                .map(|h| {
                    serde_json::json!([
                        h.habitat.to_string(),
                        h.limited_by,
                        h.production_gc_yr,
                        h.feeding_levels,
                        h.closed
                    ])
                })
                .collect();
            let hash = sse_core::ContentHash::of(&b).expect("hash").hex();
            (
                n,
                serde_json::json!({
                    "production_gc_yr": b.production_gc_yr,
                    "biomass_gc": b.biomass_gc,
                    "power_w": b.power_w,
                    "class": b.biomass_class,
                    "feeding_levels": b.feeding_levels,
                    "high_power": b.high_power,
                    "habitats": habitats,
                    "hash": hash,
                }),
            )
        })
        .collect();
    serde_json::json!({ "worlds": worlds })
}
