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

/// Archean Earth with volcanic hydrogen as well (mol per year; Kharecha et
/// al. 2005 put it near 1e12 to 1e13) and the Archean ocean's trace of
/// sulfate (under 2.5 micromolar; Crowe et al. 2014).
pub fn archean_with_hydrogen(h2_mol_yr: f64) -> HabitatGraph {
    let (_, s) = WORLDS.iter().find(|(n, _)| *n == "earth").expect("earth");
    let mut v: serde_json::Value = serde_json::from_str(s).expect("earth json");
    for sh in v["shells"].as_array_mut().expect("shells") {
        if sh["key"] == "ocean" {
            sh["properties"]["material.fe2_mol_yr"] = serde_json::json!(1.0e15);
            sh["properties"]["material.h2_mol_yr"] = serde_json::json!(h2_mol_yr);
            sh["properties"]["activity.so4-2"] = serde_json::json!(1.0e-6);
        }
    }
    let w = WorldStack::from_json(&v.to_string()).expect("archean world");
    HabitatGraph::derive(&w, &base_rules()).expect("archean graph")
}

/// Earth's fixture with extra ocean supplies (mol per year).
pub(crate) fn earth_with(extra: &[(&str, f64)]) -> HabitatGraph {
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
        graph_while: vec![],
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
            graph_while: vec![],
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

/// Fixture physics for trait derivation (Williams, NASA planetary fact sheets).
pub fn physics(name: &str) -> crate::biosphere::WorldPhysics {
    let (g, air, day, tilt, locked) = match name {
        "earth" | "earth-archean" => (sse_core::sheet::G0, Some(1.225), 24.0, 23.4, false),
        "mars" => (3.71, Some(0.020), 24.6, 25.2, false),
        "europa" => (1.315, None, 85.2, 0.1, true),
        "titan" => (1.352, Some(5.4), 382.7, 0.3, true),
        "venus" => (8.87, Some(65.0), 2802.0, 2.6, false),
        _ => (sse_core::sheet::G0, Some(1.225), 24.0, 23.4, false),
    };
    crate::biosphere::WorldPhysics {
        surface_gravity_m_s2: g,
        air_density_kg_m3: air,
        star_teff_k: sse_core::sheet::T_SUN,
        day_hours: day,
        obliquity_deg: tilt,
        tidally_locked: locked,
    }
}

/// Every capability in the base pack (Earth today).
pub fn all_capabilities() -> Vec<String> {
    history_params()
        .capabilities
        .iter()
        .map(|c| c.id.clone())
        .collect()
}

/// Earth today and Archean Earth, resolved.
pub fn biospheres() -> Vec<(String, crate::biosphere::Biosphere)> {
    let bp = budget_params();
    let hp = history_params();
    let first: BTreeMap<String, f64> = [
        ("chemotrophy", 700.0),
        ("anoxygenic", 1100.0),
        ("oxygenic", 1700.0),
        ("complex_cells", 2800.0),
        ("multicellularity", 3600.0),
        ("large_bodies", 3990.0),
        ("land", 4100.0),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), *v))
    .collect();
    let today = crate::biosphere::resolve(
        &graph("earth"),
        &bp,
        &hp,
        &all_capabilities(),
        &first,
        Some(66.0),
        &physics("earth"),
        sse_core::Seed::from_text("earth"),
        9_230_000_000 - 4_570_000_000,
    )
    .expect("earth biosphere");
    let early_caps: Vec<String> = ["chemotrophy", "dormancy", "anoxygenic"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let early = crate::biosphere::resolve(
        &earth_with(&[("fe2_mol_yr", 1.0e15), ("h2_mol_yr", 3.0e11)]),
        &bp,
        &hp,
        &early_caps,
        &first,
        None,
        &physics("earth-archean"),
        sse_core::Seed::from_text("earth"),
        9_230_000_000 - 4_570_000_000,
    )
    .expect("archean biosphere");
    vec![("earth".into(), today), ("earth-archean".into(), early)]
}

/// L1c golden: guilds, shares and traits of both biospheres.
pub fn biosphere_golden() -> serde_json::Value {
    let mut out = serde_json::Map::new();
    for (n, b) in biospheres() {
        out.insert(
            n,
            serde_json::json!({
                "guilds": b.guilds.iter().map(|g| serde_json::json!([g.id, g.share, g.intake_gc_yr, g.plain_name])).collect::<Vec<_>>(),
                "traits": b.traits,
                "diversity": b.diversity,
                "relations": b.resolved.relations.len(),
                "hash": sse_core::ContentHash::of(&b).expect("hash").hex(),
            }),
        );
    }
    serde_json::Value::Object(out)
}

/// Europa's ocean biosphere: chemotrophs and dormancy under 20 km of ice.
pub fn europa_biosphere() -> crate::biosphere::Biosphere {
    let caps: Vec<String> = ["chemotrophy", "dormancy"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let first: BTreeMap<String, f64> = [("chemotrophy", 900.0), ("dormancy", 2600.0)]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
    crate::biosphere::resolve(
        &graph("europa"),
        &budget_params(),
        &history_params(),
        &caps,
        &first,
        None,
        &physics("europa"),
        sse_core::Seed::from_text("europa"),
        9_230_000_000 - 4_570_000_000,
    )
    .expect("europa biosphere")
}

/// What an observer could measure on the L1d fixture worlds.
pub fn observables(name: &str) -> crate::signatures::Observables {
    use crate::signatures::Observables;
    let base = Observables {
        star_teff_k: sse_core::sheet::T_SUN,
        atmosphere_bar: Some(1.0),
        abiotic_o2: 0.0,
        abiotic_ch4_ppm: 0.01,
        ice_shell_km: None,
        cloud_cover: 0.6,
        land_fraction: 0.29,
        obliquity_deg: 23.4,
        recycling_gyr: Some(0.8),
        erosion_gyr: None,
        age_myr: 4570.0,
        reducing_air: false,
    };
    match name {
        "earth-archean" => Observables {
            age_myr: 2500.0,
            land_fraction: 0.1,
            ..base
        },
        "europa" => Observables {
            atmosphere_bar: None,
            abiotic_ch4_ppm: 0.0,
            ice_shell_km: Some(20.0),
            cloud_cover: 0.0,
            land_fraction: 0.0,
            obliquity_deg: 0.1,
            recycling_gyr: Some(0.05),
            ..base
        },
        // The scripted false positive: a dry world round a red dwarf whose
        // oceans were lost in the star's bright youth, leaving oxygen.
        "red-dwarf-desert" => Observables {
            star_teff_k: 3200.0,
            abiotic_o2: 0.1,
            abiotic_ch4_ppm: 0.0,
            cloud_cover: 0.1,
            land_fraction: 1.0,
            obliquity_deg: 5.0,
            recycling_gyr: None,
            erosion_gyr: Some(2.0),
            age_myr: 6000.0,
            ..base
        },
        _ => base,
    }
}

/// The L1d cases: Earth today, Archean Earth, Europa (the scripted false
/// negative), a red-dwarf desert (the scripted false positive) and a dead
/// Earth (life's history without a living biosphere).
#[allow(clippy::type_complexity)]
pub fn signature_cases() -> Vec<(
    String,
    Option<crate::biosphere::Biosphere>,
    Option<crate::history::LifeHistory>,
    crate::signatures::Observables,
)> {
    let bios = biospheres();
    let earth_h = crate::history::history(
        &earth_timeline(),
        &budget_params(),
        &history_params(),
        sse_core::Seed::from_text("a"),
    )
    .expect("earth history");
    vec![
        (
            "earth".into(),
            Some(bios[0].1.clone()),
            Some(earth_h.clone()),
            observables("earth"),
        ),
        (
            "earth-archean".into(),
            Some(bios[1].1.clone()),
            None,
            observables("earth-archean"),
        ),
        (
            "europa".into(),
            Some(europa_biosphere()),
            None,
            observables("europa"),
        ),
        (
            "red-dwarf-desert".into(),
            None,
            None,
            observables("red-dwarf-desert"),
        ),
        (
            "dead-earth".into(),
            None,
            Some(earth_h),
            observables("earth"),
        ),
    ]
}

/// L1d golden: signatures, hooks and epoch names of each case.
pub fn signatures_golden() -> serde_json::Value {
    let p = crate::signatures::SignatureParams::default();
    let sp = crate::sift::SiftParams::default();
    let mut out = serde_json::Map::new();
    for (n, b, h, o) in signature_cases() {
        let s = crate::signatures::signatures(b.as_ref(), h.as_ref(), &o, &p);
        let inp = crate::sift::HookInputs {
            habitable_until_myr: None,
            o2_fraction: s.o2,
            now_myr: o.age_myr,
        };
        let v = crate::sift::view(&n, b.as_ref(), h.as_ref(), &s, &inp, &sp);
        out.insert(
            n,
            serde_json::json!({
                "facts": crate::signatures::facts(&s),
                "hooks": v.hooks.iter().map(|x| x.id.clone()).collect::<Vec<_>>(),
                "epochs": v.epochs.iter().map(|e| serde_json::json!([e.name, e.from_myr, e.to_myr, e.major])).collect::<Vec<_>>(),
                "hash": sse_core::ContentHash::of(&v).expect("hash").hex(),
            }),
        );
    }
    serde_json::Value::Object(out)
}

/// L2 fixtures: Earth with no dated oxygen (the air box decides), and a
/// small, reduced, volcanically busy world on the same habitats.
pub fn coupled_manifest(name: &str) -> crate::feedback::CoupledManifest {
    use crate::feedback::{AirParams, CoupledManifest, MANIFEST_FORMAT};
    let mut world = earth_timeline();
    for e in &mut world.epochs {
        e.facts.insert("o2_fraction".into(), 0.0);
    }
    let air = match name {
        "reduced" => AirParams {
            air_mol: 5.0e19,
            area_rel: 0.3,
            reductant_mol_yr: 3.0e13,
            ..AirParams::default()
        },
        _ => AirParams::default(),
    };
    CoupledManifest {
        format: MANIFEST_FORMAT.into(),
        world,
        budget: budget_params(),
        history: history_params(),
        air,
        seed: "a".into(),
    }
}

/// L2 golden: the coupled runs of both fixtures, sampled every 500 Myr.
pub fn feedback_golden() -> serde_json::Value {
    let mut out = serde_json::Map::new();
    for n in ["earth", "reduced"] {
        let man = coupled_manifest(n);
        let r = crate::feedback::coupled(&man).expect("coupled run");
        let samples: Vec<_> = r
            .air
            .series
            .iter()
            .filter(|p| (p.at_myr % 500.0) == 0.0)
            .map(|p| serde_json::json!([p.at_myr, p.o2_fraction, p.ch4_ppm, p.co2_rel]))
            .collect();
        out.insert(
            n.into(),
            serde_json::json!({
                "oxygenated_myr": r.oxygenated_myr,
                "first": r.history.first,
                "air": samples,
                "manifest_hash": r.manifest_hash,
                "hash": sse_core::ContentHash::of(&r).expect("hash").hex(),
            }),
        );
    }
    serde_json::Value::Object(out)
}
