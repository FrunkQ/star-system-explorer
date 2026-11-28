//! An epoch's habitat graph follows the history's own air (`graph_while`):
//! an ocean keeps the iron its vents bring while the air is still without
//! oxygen, whatever the air holds at the end.

use crate::history::*;
use crate::scenario::*;
use sse_core::Seed;

/// Today's Earth (vent hydrogen, an oxic ocean without iron) held for the
/// whole history, with or without the anoxic ocean (the vents' iron too)
/// for while the air has no oxygen, as a chart world is built.
fn held_earth(o2: f64, anoxic_variant: bool) -> WorldTimeline {
    let today = earth_with(&[("h2_mol_yr", 3.0e11)]);
    WorldTimeline {
        body: today.body.clone(),
        end_myr: 4500.0,
        epochs: vec![Epoch {
            id: "earth.today-held".into(),
            from_myr: 0.0,
            graph: Some(today),
            facts: [("o2_fraction".to_string(), o2)].into_iter().collect(),
            graph_while: if anoxic_variant {
                vec![GraphWhile {
                    fact: "o2_fraction".into(),
                    below: 1.0e-5,
                    graph: earth_with(&[("h2_mol_yr", 3.0e11), ("fe2_mol_yr", 1.0e15)]),
                    why: "an ocean without oxygen keeps the iron its vents bring".into(),
                }]
            } else {
                vec![]
            },
        }],
        catastrophes: vec![],
    }
}

fn run(w: &WorldTimeline, s: &str) -> LifeHistory {
    history(w, &budget_params(), &history_params(), Seed::from_text(s)).unwrap()
}

/// Production while life has anoxygenic photosynthesis and not yet the
/// oxygenic kind, g C a year (the largest such point).
fn mats(h: &LifeHistory) -> Option<f64> {
    h.points
        .iter()
        .filter(|p| {
            p.capabilities.iter().any(|c| c == "anoxygenic")
                && !p.capabilities.iter().any(|c| c == "oxygenic")
        })
        .map(|p| p.production_gc_yr)
        .reduce(f64::max)
}

#[test]
fn an_anoxic_air_lets_the_ocean_keep_its_iron() {
    let mut n = 0;
    for i in 0..10 {
        let s = format!("earth-{i}");
        let (Some(with), Some(without)) = (
            mats(&run(&held_earth(0.0, true), &s)),
            mats(&run(&held_earth(0.0, false), &s)),
        ) else {
            continue;
        };
        // Canfield et al. 2006: Archean iron photosynthesis near 1e15 g C a
        // year; without the iron the mats live on vent hydrogen alone.
        assert!(with > 1.0e14, "{s}: {with:e}");
        assert!(with > 100.0 * without, "{s}: {with:e} vs {without:e}");
        n += 1;
    }
    assert!(n > 5, "{n}");
}

#[test]
fn oxygen_in_the_air_closes_the_variant() {
    // The air already holds oxygen (an epoch fact here, the air box in a
    // coupled run): the ocean is today's, and the variant changes nothing.
    for i in 0..5 {
        let s = format!("earth-{i}");
        assert_eq!(
            run(&held_earth(0.21, true), &s),
            run(&held_earth(0.21, false), &s),
            "{s}"
        );
    }
}
