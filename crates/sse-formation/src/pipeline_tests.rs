//! The formation pipeline: stages chained, heat on every solid world.
use crate::pipeline::*;
use crate::scenario::solar_patch;

fn histories(n: usize) -> Vec<crate::FormationHistory> {
    solar_patch().into_iter().flatten().take(n).collect()
}

#[test]
fn pipeline_runs_every_block_deterministically() {
    for h in histories(6) {
        for block in [Block::Modern, Block::Classic, Block::ClassicCorrected] {
            let a = run(h.clone(), block);
            let b = run(h.clone(), block);
            assert_eq!(
                serde_json::to_string(&a).unwrap(),
                serde_json::to_string(&b).unwrap()
            );
            assert_eq!(a.pipeline, PIPELINE_VERSION);
            for w in a.events.windows(2) {
                assert!(w[0].t_myr <= w[1].t_myr);
            }
            // Every solid world gets a heat budget from the planet-heat flow; giants do not.
            let solid = a
                .worlds
                .iter()
                .filter(|w| w.envelope != crate::early::Envelope::Giant)
                .count();
            assert_eq!(a.heat.len(), solid, "{block:?}");
            for x in &a.heat {
                assert_eq!(x.flow, HEAT_FLOW);
                assert!(x.radiogenic_power_w > 0.0 && x.heat_flow_w_m2 > 0.0);
                assert_eq!(x.active_now, x.age_gyr < x.active_geology_until_gyr);
            }
            if block == Block::Modern {
                assert!(a.growth.is_some() && a.upheaval.is_some() && a.leftovers.is_some());
            } else {
                assert!(a.upheaval.is_none());
                assert!(a.accrete.is_some() || !a.notes.is_empty(), "{:?}", a.notes);
            }
        }
    }
}

/// Bigger worlds stay geologically alive longer, and thorium extends it
/// (Unterborn et al. 2015); Earth-sized worlds make terawatts.
#[test]
fn heat_follows_size_and_inheritance() {
    let mut earthlike = vec![];
    let mut pairs = vec![];
    for h in histories(12) {
        let base = run(h.clone(), Block::Modern);
        let mut rich = h.clone();
        rich.inheritance.r_process_rel *= 2.0;
        let rich = run(rich, Block::Modern);
        for (a, b) in base.heat.iter().zip(&rich.heat) {
            assert_eq!(a.address, b.address);
            assert!(b.active_geology_until_gyr >= a.active_geology_until_gyr);
            if a.active_geology_until_gyr > 0.0 && a.active_geology_until_gyr < 100.0 {
                assert!(b.active_geology_until_gyr > a.active_geology_until_gyr);
            }
        }
        for (w, x) in base
            .worlds
            .iter()
            .filter(|w| w.envelope != crate::early::Envelope::Giant)
            .zip(&base.heat)
        {
            pairs.push((w.mass_earth, x.active_geology_until_gyr));
            if (0.5..2.0).contains(&w.mass_earth) {
                earthlike.push(x.radiogenic_power_w);
            }
        }
    }
    assert!(!earthlike.is_empty(), "no Earth-sized worlds in 12 systems");
    for p in &earthlike {
        assert!((3e12..2e14).contains(p), "Earth-sized power {p}");
    }
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let k = pairs.len() / 4;
    let small: f64 = pairs[..k].iter().map(|p| p.1).sum::<f64>() / k as f64;
    let large: f64 = pairs[pairs.len() - k..].iter().map(|p| p.1).sum::<f64>() / k as f64;
    assert!(large > small, "large {large} small {small}");
}

#[test]
fn golden() {
    crate::golden::check(
        "formation-pipeline.json",
        &crate::scenario::pipeline_golden(),
    );
}

/// ER2: the planet-heat flow is compiled against the blocks in force and
/// kept with the physics in force, so a pack that edits the radiogenic
/// heating block's potassium setting moves every solid world's heat, and the
/// shipped answer is unchanged outside it.
#[test]
fn a_block_edit_in_force_reaches_the_heat_flow() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let read = |p: &str| {
        sse_pack::PackManifest::from_json(&std::fs::read_to_string(format!("{root}/{p}")).unwrap())
            .unwrap()
    };
    let packs = [
        read("packs/base-physics/pack.json"),
        read("tests/fixtures/packs/block-edit-wind-and-heat.json"),
    ];
    let physics = std::sync::Arc::new(sse_physics::Physics::build(
        &packs,
        None,
        std::sync::Arc::default(),
        std::sync::Arc::new(Default::default()),
    ));
    let (h, shipped) = histories(6)
        .into_iter()
        .map(|h| (h.clone(), run(h, Block::Modern)))
        .find(|(_, r)| !r.heat.is_empty())
        .expect("a system with a solid world");
    let edited = sse_physics::with(&physics, physics.constants(), || {
        run(h.clone(), Block::Modern)
    });
    let again = run(h, Block::Modern);
    assert!(!shipped.heat.is_empty());
    for ((a, b), c) in shipped.heat.iter().zip(&edited.heat).zip(&again.heat) {
        assert!(b.radiogenic_power_w > a.radiogenic_power_w, "{a:?} {b:?}");
        assert_eq!(a.radiogenic_power_w, c.radiogenic_power_w);
    }
}
