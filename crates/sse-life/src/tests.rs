//! L0 checks (life design section 14): interface schemas, habitat graphs
//! for the five fixture worlds, hash stability, the empty suite.

use crate::habitat::{Contact, HabitatGraph};
use crate::scenario::{base_rules, graph, world, WORLDS};
use crate::*;
use sse_suite::SuiteAdapter;

fn kinds(g: &HabitatGraph) -> Vec<String> {
    g.habitats
        .iter()
        .map(|h| format!("{}/{}", h.source, h.kind))
        .collect()
}

fn t(s: i64) -> sse_chronicle::UniverseTime {
    sse_chronicle::UniverseTime { s, ns: 0 }
}

/// One sample of every interface value, written out field by field.
fn samples() -> serde_json::Value {
    let g = graph("europa");
    let addr = g.body.clone();
    let profile = g.habitats[0].profile.clone();
    let envelope = LifeEnvelope {
        address: addr.clone(),
        epoch: t(1),
        p_present: Some(0.4),
        origin_time: Some(TimeDistribution {
            quantiles: vec![(0.1, 1.0e16), (0.5, 3.0e16)],
        }),
        energy_budget_w: Some(1.0e9),
        biomass_class: Some("sparse".into()),
        capabilities: [("methanogenesis".to_string(), 0.3)].into_iter().collect(),
        complexity: [("guilds".to_string(), 3.0)].into_iter().collect(),
        fluxes: vec![EnvelopeFlux {
            field: "atmosphere.pp.ch4".into(),
            rate: 1.0e-12,
        }],
        signatures: [("ch4".to_string(), 0.01)].into_iter().collect(),
        uncertainty: "sample".into(),
        evidence: LifeEvidence::PlausibleReconstruction,
        inputs: Some(g.hash()),
    };
    let resolved = ResolvedBiosphere {
        address: addr.clone(),
        guilds: vec![Guild {
            id: "vent-methanogens".into(),
            name: "vent methanogens".into(),
            capabilities: vec!["methanogenesis".into()],
            habitats: [(g.habitats[1].address.clone(), 1.0)].into_iter().collect(),
            biomass_kg: Some(1.0e12),
            since: Some(t(2)),
        }],
        relations: vec![Relation {
            kind: RelationKind::Habitat,
            from: "vent-methanogens".into(),
            to: "sea-floor".into(),
        }],
        evidence: LifeEvidence::PlausibleReconstruction,
    };
    let query = LifeQuery {
        address: addr,
        epoch: t(3),
        capabilities: vec!["photosynthesis".into()],
    };
    let feasibility = Feasibility {
        answer: Some(false),
        reason: "no light reaches the ocean".into(),
    };
    serde_json::json!({
        "profile": profile,
        "envelope": envelope,
        "resolved": resolved,
        "query": query,
        "feasibility": feasibility,
        "rules": base_rules(),
        "graph": g,
    })
}

#[test]
fn interfaces_keep_their_wire_shape() {
    // The L0 shapes are fixed: this file changes only on purpose
    // (`SSE_BLESS=1` rewrites it), and every value reads back unchanged.
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/life/interfaces.json"
    );
    let got = samples();
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string_pretty(&got).unwrap() + "\n").unwrap();
    }
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(v, got, "an L0 interface changed shape");
    fn round<T: serde::de::DeserializeOwned + serde::Serialize>(v: &serde_json::Value) {
        let t: T = serde_json::from_value(v.clone()).expect("reads");
        assert_eq!(&serde_json::to_value(&t).unwrap(), v);
    }
    round::<LifeRequirementProfile>(&v["profile"]);
    round::<LifeEnvelope>(&v["envelope"]);
    round::<ResolvedBiosphere>(&v["resolved"]);
    round::<LifeQuery>(&v["query"]);
    round::<Feasibility>(&v["feasibility"]);
    round::<Vec<HabitatRule>>(&v["rules"]);
    round::<HabitatGraph>(&v["graph"]);
}

#[test]
fn earth_has_sea_land_rock_and_cave_habitats_joined_up() {
    let g = graph("earth");
    let k = kinds(&g);
    for want in [
        "ocean/ocean-sunlit",
        "ocean/ocean-interior",
        "ocean/sea-floor",
        "crust/deep-crust",
        "tropics/land-surface",
        "tropics/soil",
        "arctic/land-surface",
        "movile/cave",
    ] {
        assert!(k.contains(&want.to_string()), "{want} missing from {k:?}");
    }
    // The ice sheet and the atmosphere are not habitats by default.
    assert!(!k
        .iter()
        .any(|x| x.starts_with("antarctica/") || x.starts_with("atmosphere/")));
    let l = |a, b| g.linked(a, b).map(|l| (l.contact, l.certain));
    assert_eq!(
        l(("ocean", "ocean-sunlit"), ("ocean", "ocean-interior")),
        Some((Contact::Vertical, true))
    );
    assert_eq!(
        l(("ocean", "ocean-interior"), ("ocean", "sea-floor")),
        Some((Contact::Vertical, true))
    );
    assert_eq!(
        l(("ocean", "sea-floor"), ("crust", "deep-crust")),
        Some((Contact::Embedded, true))
    );
    assert_eq!(
        l(("tropics", "land-surface"), ("tropics", "soil")),
        Some((Contact::Vertical, true))
    );
    assert_eq!(
        l(("tropics", "soil"), ("crust", "deep-crust")),
        Some((Contact::Vertical, true))
    );
    assert_eq!(
        l(
            ("tropics", "land-surface"),
            ("north-temperate", "land-surface")
        ),
        Some((Contact::Lateral, true))
    );
    // The coast is a guess: the ocean basins have no drawn outline yet.
    assert_eq!(
        l(("ocean", "ocean-sunlit"), ("tropics", "land-surface")),
        Some((Contact::Vertical, false))
    );
    assert!(l(("ocean", "ocean-interior"), ("crust", "deep-crust")).is_none());
    assert!(l(("tropics", "land-surface"), ("arctic", "land-surface")).is_none());
    assert_eq!(
        l(("movile", "cave"), ("north-temperate", "soil")),
        Some((Contact::Embedded, true))
    );
    // Vents: the sea floor can draw on chemical energy from the warm interior.
    let floor = g.get("ocean", "sea-floor").unwrap();
    assert!(floor
        .profile
        .energy
        .available
        .contains(&"chemical".to_string()));
    let sun = &g.get("ocean", "ocean-sunlit").unwrap().profile;
    assert_eq!(sun.energy.light_w_m2, Some(185.0));
    let area = sun.area_m2.unwrap();
    assert!((area / 3.6e14 - 1.0).abs() < 0.01, "ocean area {area}");
}

#[test]
fn mars_has_only_ground_rock_and_caves() {
    let g = graph("mars");
    assert_eq!(
        kinds(&g),
        [
            "crust/deep-crust",
            "surface/land-surface",
            "surface/soil",
            "arsia-cave/cave"
        ]
    );
    let surf = &g.get("surface", "land-surface").unwrap().profile;
    assert_eq!(
        surf.temperature_k,
        Some(Range {
            lo: 150.0,
            hi: 300.0
        })
    );
    assert_eq!(surf.radiation_dose_msv_yr, Some(230.0));
    assert!(g
        .linked(("arsia-cave", "cave"), ("crust", "deep-crust"))
        .is_some());
}

#[test]
fn europa_is_an_ocean_over_rock_with_vents() {
    let g = graph("europa");
    assert_eq!(kinds(&g), ["ocean/sea-floor", "ocean/under-ice-ocean"]);
    assert_eq!(g.links.len(), 1);
    let floor = &g.get("ocean", "sea-floor").unwrap().profile;
    assert!(floor.energy.available.contains(&"chemical".to_string()));
    let ocean = &g.get("ocean", "under-ice-ocean").unwrap().profile;
    assert!(ocean.energy.available.contains(&"chemical".to_string()));
    assert!(!ocean.energy.available.contains(&"light".to_string()));
    // A 100 km deep global ocean under 20 km of ice: about 2.8e18 m3, twice Earth's.
    let v = ocean.volume_m3.unwrap();
    assert!((v / 2.79e18 - 1.0).abs() < 0.01, "volume {v}");
}

#[test]
fn titan_has_methane_seas_and_a_sealed_ocean_on_ice() {
    let g = graph("titan");
    assert_eq!(
        kinds(&g),
        [
            "ocean/under-ice-ocean",
            "kraken/hydrocarbon-lake",
            "ligeia/hydrocarbon-lake"
        ]
    );
    // Its ocean sits on high-pressure ice, not rock: no sea floor, no vents.
    assert!(g.get("ocean", "sea-floor").is_none());
    let ocean = &g.get("ocean", "under-ice-ocean").unwrap().profile;
    assert!(!ocean.energy.available.contains(&"chemical".to_string()));
    assert!(g.links.is_empty(), "{:?}", g.summary());
    let k = &g.get("kraken", "hydrocarbon-lake").unwrap().profile;
    assert_eq!(k.solvent, "methane-ethane");
    assert_eq!(k.temperature_k, Some(Range { lo: 92.0, hi: 92.0 }));
}

#[test]
fn venus_lists_its_clouds_and_its_hot_plains_unconnected() {
    let g = graph("venus");
    assert_eq!(
        kinds(&g),
        ["clouds/cloud-deck", "plains/land-surface", "plains/soil"]
    );
    // The cloud deck floats 48 km up, touching nothing.
    let deck = &g.get("clouds", "cloud-deck").unwrap().address;
    assert!(!g.links.iter().any(|l| &l.a == deck || &l.b == deck));
    let c = &g.get("clouds", "cloud-deck").unwrap().profile;
    assert_eq!(
        c.temperature_k,
        Some(Range {
            lo: 260.0,
            hi: 340.0
        })
    );
    assert_eq!(c.properties.get("sulphuric_acid_fraction"), Some(&0.8));
    // The plains are a candidate place, listed honestly at 737 K.
    let p = &g.get("plains", "land-surface").unwrap().profile;
    assert_eq!(
        p.temperature_k,
        Some(Range {
            lo: 737.0,
            hi: 737.0
        })
    );
    assert!(c.missing.contains(&"light_w_m2".to_string()));
}

#[test]
fn graphs_are_stable_under_hash() {
    for (n, s) in WORLDS {
        let a = graph(n);
        // Same stack written differently (re-serialised, keys reordered).
        let v: serde_json::Value = serde_json::from_str(s).unwrap();
        let again = sse_world::WorldStack::from_json(&serde_json::to_string(&v).unwrap()).unwrap();
        let b = HabitatGraph::derive(&again, &base_rules()).unwrap();
        assert_eq!(a.hash(), b.hash(), "{n}");
        // Changing one number changes the hash.
        let mut w = world(n);
        w.radius_m *= 1.001;
        assert_ne!(
            HabitatGraph::derive(&w, &base_rules()).unwrap().hash(),
            a.hash(),
            "{n}"
        );
    }
}

#[test]
fn broken_stacks_and_duplicate_rules_are_refused() {
    let mut w = world("europa");
    w.shells[1].within = Some("nowhere".into());
    assert!(HabitatGraph::derive(&w, &base_rules()).is_err());
    let p = crate::scenario::base_life_pack();
    let e = crate::habitat::rules(&[p.clone(), p]).unwrap_err();
    assert!(e[0].contains("already defined"), "{e:?}");
}

#[test]
fn authored_habitats_without_a_rule_are_kept() {
    let tarn =
        sse_world::WorldStack::from_json(include_str!("../../../tests/fixtures/worlds/tarn.json"))
            .unwrap();
    let g = HabitatGraph::derive(&tarn, &base_rules()).unwrap();
    let k = kinds(&g);
    // Tarn's biomes match the land rules; its brine ocean the sub-ice rule.
    for want in [
        "forest/land-surface",
        "reef/soil",
        "brine-ocean/under-ice-ocean",
    ] {
        assert!(k.contains(&want.to_string()), "{want} not in {k:?}");
    }
    let mut odd = tarn.clone();
    odd.regions[0].kind = "glowing_pool".into();
    odd.regions[0].provides = vec!["habitat".into()];
    let g = HabitatGraph::derive(&odd, &base_rules()).unwrap();
    let h = g.get("south-pole", "glowing_pool").unwrap();
    assert_eq!(
        (h.rule.as_str(), h.profile.medium.as_str()),
        ("authored", "unknown")
    );
}

#[test]
fn the_empty_suite_claims_nothing() {
    let g = graph("earth");
    let mut s = NoLifeModel::default();
    let t0 = t(0);
    let env = s.envelope(&g.body, t0, &g.profiles());
    assert_eq!(env.evidence, LifeEvidence::NoModel);
    assert_eq!((env.p_present, env.capabilities.len()), (None, 0));
    assert!(env.inputs.is_some());
    let seed = sse_core::Seed::from_text("l0");
    assert_eq!(s.resolve(&env, &seed).unwrap_err().0, NO_MODEL);
    let q = LifeQuery {
        address: g.body.clone(),
        epoch: t0,
        capabilities: vec!["photosynthesis".into()],
    };
    assert_eq!(s.query_feasible(&q, &g.profiles()).answer, None);
    assert_eq!(LifeSuite::explain(&s, &g.body, "life.stage"), NO_MODEL);
    // Through the K10 socket: installed, advances, returns nothing.
    assert_eq!(s.manifest().id, "life");
    assert_eq!(s.advance(t(1)).unwrap(), Default::default());
    assert_eq!(
        SuiteAdapter::explain(&s, &g.body, "x").as_deref(),
        Some(NO_MODEL)
    );
}

#[test]
fn golden() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/life.json");
    let got = crate::scenario::golden();
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, serde_json::to_string_pretty(&got).unwrap() + "\n").unwrap();
    }
    let want: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        got, want,
        "tests/golden/life.json moved: bless only on purpose"
    );
}
