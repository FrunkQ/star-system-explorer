//! E2: metabolisms generated from open gradients (evolution design 5.1,
//! 5.3 and 14).

use crate::chemistry::ThermoTable;
use crate::gradients::GradientParams;
use crate::history::*;
use crate::scenario::*;
use sse_core::Seed;

fn gp() -> GradientParams {
    GradientParams::from_packs(&[base_life_pack()]).expect("life gradients")
}
fn table() -> ThermoTable {
    ThermoTable::from_packs(&[base_life_pack()]).expect("life chemistry")
}
fn run_g(w: &WorldTimeline, hp: &HistoryParams, seed: &str) -> LifeHistory {
    let (gp, t) = (gp(), table());
    let gm = GradientModel { gp: &gp, table: &t };
    history_model(
        w,
        &budget_params(),
        hp,
        Some(&gm),
        Seed::from_text(seed),
        None,
    )
    .unwrap()
}
fn seeds(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("earth-{i}")).collect()
}

fn outline(h: &LifeHistory) -> (bool, bool, bool) {
    let e = h.origin_myr.is_some_and(|t| t < 1000.0);
    let mic = h.first.get("multicellularity").is_none_or(|t| *t >= 2500.0);
    let big = h.first.get("large_bodies").is_some_and(|t| *t >= 3500.0);
    (e, mic, big)
}

#[test]
fn earth_outline_at_least_as_often_as_the_listed_model() {
    let w = earth_timeline();
    let hp = history_params();
    let n = 200;
    let (mut listed, mut grown) = (0, 0);
    let mut oxygenic = vec![];
    for s in seeds(n) {
        let a = outline(&history(&w, &budget_params(), &hp, Seed::from_text(&s)).unwrap());
        listed += (a.0 && a.1 && a.2) as u32;
        let h = run_g(&w, &hp, &s);
        let b = outline(&h);
        grown += (b.0 && b.1 && b.2) as u32;
        if let Some(t) = h.first.get("metabolism:oxygenic") {
            oxygenic.push(*t);
        }
    }
    oxygenic.sort_by(f64::total_cmp);
    println!(
        "Earth outline in {listed} of {n} seeds listed, {grown} grown; oxygenic in {} seeds, median {:.0} Myr",
        oxygenic.len(),
        oxygenic.get(oxygenic.len() / 2).copied().unwrap_or(f64::NAN)
    );
    assert!(grown >= listed, "{grown} < {listed}");
}

fn with_dial(name: &str, v: f64) -> HistoryParams {
    let mut hp = history_params();
    hp.dials.insert(name.into(), v);
    hp
}

#[test]
fn aerobic_respiration_follows_oxygen_on_every_seed_that_oxygenates() {
    let w = earth_timeline();
    let hp = history_params();
    let mut oxygenated = 0;
    for s in seeds(100) {
        let h = run_g(&w, &hp, &s);
        let Some(ox) = h.first.get("metabolism:oxygenic").copied() else {
            // No oxygen maker: nothing that needs oxygen, and no complex cells.
            assert!(!h.has("metabolism:aerobic-respiration"), "{s}");
            assert!(!h.has("complex_cells"), "{s}");
            continue;
        };
        oxygenated += 1;
        let aer = h.first.get("metabolism:aerobic-respiration").copied();
        assert!(
            aer.is_some_and(|t| t > ox),
            "{s}: oxygenic {ox}, aerobic {aer:?}"
        );
        // Complex cells wait for the power of an oxygen-using metabolism.
        if let Some(cc) = h.first.get("complex_cells") {
            assert!(*cc > aer.unwrap(), "{s}");
        }
        // Every metabolism event says what it would add and against what.
        for e in h
            .events
            .iter()
            .filter(|e| e.kind == "capability" && e.subject.starts_with(METABOLISM))
        {
            assert!(e.why.contains(" W to the "), "{}", e.why);
        }
    }
    assert!(oxygenated > 90, "{oxygenated} of 100 oxygenate");
}

#[test]
fn dials_stay_monotonic_on_the_gradient_model() {
    let w = earth_timeline();
    let ss = seeds(30);
    for (dial, cap) in [
        ("oxygenic", "metabolism:oxygenic"),
        ("complex_cells", "large_bodies"),
        ("origin", "chemotrophy"),
    ] {
        let c: Vec<usize> = [0.01, 1.0, 100.0]
            .iter()
            .map(|d| {
                let hp = with_dial(dial, *d);
                ss.iter().filter(|s| run_g(&w, &hp, s).has(cap)).count()
            })
            .collect();
        println!("{dial} dial 0.01, 1, 100: {cap} in {c:?} of {}", ss.len());
        assert!(c[0] <= c[1] && c[1] <= c[2] && c[0] < c[2], "{dial}: {c:?}");
    }
    // Per seed, a bigger oxygenic dial never makes oxygenic photosynthesis later.
    for s in ss.iter().take(15) {
        let t = |d: f64| {
            run_g(&w, &with_dial("oxygenic", d), s)
                .first
                .get("metabolism:oxygenic")
                .copied()
                .unwrap_or(f64::INFINITY)
        };
        assert!(t(10.0) <= t(1.0) && t(1.0) <= t(0.1), "{s}");
    }
}

#[test]
fn the_same_seed_gives_the_same_history_and_the_listed_model_is_untouched() {
    let w = earth_timeline();
    let hp = history_params();
    assert_eq!(run_g(&w, &hp, "earth-7"), run_g(&w, &hp, "earth-7"));
    // The listed model never reads the metabolic block.
    let mut bare = hp.clone();
    bare.metabolic = None;
    let a = history(&w, &budget_params(), &hp, Seed::from_text("earth-7")).unwrap();
    let b = history(&w, &budget_params(), &bare, Seed::from_text("earth-7")).unwrap();
    assert_eq!(a, b);
    // The gradient model refuses to run without it.
    let (gp, t) = (gp(), table());
    let gm = GradientModel { gp: &gp, table: &t };
    assert!(history_model(
        &w,
        &budget_params(),
        &bare,
        Some(&gm),
        Seed::from_text("x"),
        None
    )
    .is_err());
}
