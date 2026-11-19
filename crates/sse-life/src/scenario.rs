//! L0 fixture worlds and golden: habitat graphs for Earth, Mars, Europa,
//! Titan and Venus from `tests/fixtures/worlds/`, with the base-life rules.

use crate::habitat::{self, HabitatGraph, HabitatRule};
use sse_pack::PackManifest;
use sse_world::WorldStack;
use std::collections::BTreeMap;

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
    earth_with(&[("fe2_mol_yr", 1.0e15)])
}

/// Earth's fixture with extra ocean supplies (mol per year).
fn earth_with(extra: &[(&str, f64)]) -> HabitatGraph {
    let (_, s) = WORLDS.iter().find(|(n, _)| *n == "earth").expect("earth");
    let mut v: serde_json::Value = serde_json::from_str(s).expect("earth json");
    for sh in v["shells"].as_array_mut().expect("shells") {
        if sh["key"] == "ocean" {
            for (k, x) in extra {
                sh["properties"][format!("material.{k}")] = serde_json::json!(x);
            }
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

/// Box C, D, E settings from the base life pack.
pub fn history_params() -> crate::history::HistoryParams {
    crate::history::HistoryParams::from_packs(&[base_life_pack()]).expect("life history")
}

/// Earth through time for L1b. The physical states are inputs here (L2
/// couples them to life): no habitats before oceans at 150 Myr; a ferrous
/// ocean with hydrothermal hydrogen (about 3e11 mol per year; Kharecha et
/// al. 2005, Sleep and Bird 2007) throughout; oxygen at 1e-6 of the air
/// before 2.1 Gyr, 1% to 3.9 Gyr, 21% after (Lyons et al. 2014), counted only
/// once life makes oxygen. Catastrophes: Hadean impacts, two Snowball
/// glaciations, the end-Permian and end-Cretaceous extinctions.
pub fn earth_timeline() -> crate::history::WorldTimeline {
    use crate::history::{Catastrophe, Epoch, WorldTimeline};
    let g = earth_with(&[("fe2_mol_yr", 1.0e15), ("h2_mol_yr", 3.0e11)]);
    let ep = |id: &str, from: f64, o2: f64, graph: Option<HabitatGraph>| Epoch {
        id: id.into(),
        from_myr: from,
        graph,
        facts: [("o2_fraction".to_string(), o2)].into_iter().collect(),
    };
    let deep = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let shelter = deep(&["deep-crust", "sea-floor", "ocean-interior", "soil", "cave"]);
    let cat =
        |id: &str, at: f64, kind: &str, sev: f64, refuges: Vec<String>, why: &str| Catastrophe {
            id: id.into(),
            at_myr: at,
            kind: kind.into(),
            severity: sev,
            refuges,
            why: why.into(),
        };
    WorldTimeline {
        body: g.body.clone(),
        end_myr: 4570.0,
        epochs: vec![
            ep("earth.hadean", 0.0, 0.0, None),
            ep("earth.archean", 150.0, 1.0e-6, Some(g.clone())),
            ep("earth.great-oxidation", 2100.0, 0.01, Some(g.clone())),
            ep("earth.neoproterozoic-oxygenation", 3900.0, 0.21, Some(g)),
        ],
        catastrophes: vec![
            cat(
                "earth.impact.hadean",
                300.0,
                "large_impact",
                1.0,
                deep(&["deep-crust", "sea-floor"]),
                "a basin-forming impact boils the surface ocean",
            ),
            cat(
                "earth.glaciation.huronian",
                2170.0,
                "global_glaciation",
                0.5,
                shelter.clone(),
                "the Huronian glaciation freezes the surface",
            ),
            cat(
                "earth.glaciation.sturtian",
                3853.0,
                "global_glaciation",
                0.6,
                shelter.clone(),
                "the Sturtian Snowball Earth",
            ),
            cat(
                "earth.extinction.end-permian",
                4318.0,
                "large_igneous_province",
                0.9,
                shelter.clone(),
                "the Siberian Traps erupt: the end-Permian extinction",
            ),
            cat(
                "earth.extinction.end-cretaceous",
                4504.0,
                "large_impact",
                0.75,
                shelter,
                "the Chicxulub impact: the end-Cretaceous extinction",
            ),
        ],
    }
}

/// Europa through time: today's ocean held for 4.5 Gyr (a constant input).
pub fn europa_timeline() -> crate::history::WorldTimeline {
    let g = graph("europa");
    crate::history::WorldTimeline {
        body: g.body.clone(),
        end_myr: 4500.0,
        epochs: vec![crate::history::Epoch {
            id: "europa.ocean".into(),
            from_myr: 0.0,
            graph: Some(g),
            facts: BTreeMap::new(),
        }],
        catastrophes: vec![],
    }
}

/// L1b golden: Earth's and Europa's histories for three seeds.
pub fn history_golden() -> serde_json::Value {
    let bp = budget_params();
    let hp = history_params();
    let mut out = serde_json::Map::new();
    for (name, w) in [("earth", earth_timeline()), ("europa", europa_timeline())] {
        for seed in ["a", "b", "c"] {
            let h = crate::history::history(&w, &bp, &hp, sse_core::Seed::from_text(seed))
                .expect("history");
            let events: Vec<serde_json::Value> = h
                .events
                .iter()
                .map(|e| serde_json::json!([e.at_myr, e.kind, e.subject]))
                .collect();
            out.insert(
                format!("{name}/{seed}"),
                serde_json::json!({
                    "origin_myr": h.origin_myr,
                    "first": h.first,
                    "events": events,
                    "hash": sse_core::ContentHash::of(&h).expect("hash").hex(),
                }),
            );
        }
    }
    serde_json::Value::Object(out)
}
