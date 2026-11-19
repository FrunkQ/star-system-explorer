//! L1b checks (life design sections 4 to 6 and 14).

use crate::history::*;
use crate::scenario::*;
use sse_core::Seed;

fn run(w: &WorldTimeline, hp: &HistoryParams, seed: &str) -> LifeHistory {
    history(w, &budget_params(), hp, Seed::from_text(seed)).unwrap()
}

fn seeds(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("earth-{i}")).collect()
}

fn with_dial(name: &str, v: f64) -> HistoryParams {
    let mut hp = history_params();
    hp.dials.insert(name.into(), v);
    hp
}

#[test]
fn earth_outline_in_most_seeds() {
    let w = earth_timeline();
    let hp = history_params();
    let n = 200;
    let (mut early, mut microbial, mut late_big, mut all) = (0, 0, 0, 0);
    for s in seeds(n) {
        let h = run(&w, &hp, &s);
        let e = h.origin_myr.is_some_and(|t| t < 1000.0);
        let mic = h.first.get("multicellularity").is_none_or(|t| *t >= 2500.0);
        let big = h.first.get("large_bodies").is_some_and(|t| *t >= 3500.0);
        early += e as u32;
        microbial += mic as u32;
        late_big += big as u32;
        all += (e && mic && big) as u32;
    }
    println!("of {n}: life before 1 Gyr {early}, microbial for 2.5 Gyr {microbial}, large bodies after 3.5 Gyr {late_big}, all three {all}");
    let most = n as u32 / 2;
    assert!(early > most && microbial > most && late_big > most && all > most);
}

#[test]
fn dials_move_outcomes_monotonically() {
    let w = earth_timeline();
    let ss = seeds(60);
    // Origin: per seed, a bigger dial never makes origin later.
    let runs: Vec<Vec<LifeHistory>> = [0.1, 1.0, 10.0]
        .iter()
        .map(|d| {
            ss.iter()
                .map(|s| run(&w, &with_dial("origin", *d), s))
                .collect()
        })
        .collect();
    for i in 0..ss.len() {
        let t = |k: usize| runs[k][i].origin_myr.unwrap_or(f64::INFINITY);
        assert!(
            t(2) <= t(1) && t(1) <= t(0),
            "{}: {} {} {}",
            ss[i],
            t(0),
            t(1),
            t(2)
        );
    }
    let count = |rs: &Vec<LifeHistory>, cap: &str| rs.iter().filter(|h| h.has(cap)).count();
    let o: Vec<usize> = runs.iter().map(|r| count(r, "chemotrophy")).collect();
    assert!(o[0] <= o[1] && o[1] <= o[2], "{o:?}");
    // Hard steps: more worlds reach the step and what follows it.
    for (dial, cap) in [("oxygenic", "oxygenic"), ("complex_cells", "large_bodies")] {
        let c: Vec<usize> = [0.1, 1.0, 10.0]
            .iter()
            .map(|d| {
                let hp = with_dial(dial, *d);
                ss.iter().filter(|s| run(&w, &hp, s).has(cap)).count()
            })
            .collect();
        println!("{dial} dial 0.1, 1, 10: {cap} in {c:?} of {}", ss.len());
        assert!(c[0] <= c[1] && c[1] <= c[2] && c[0] < c[2], "{dial}: {c:?}");
    }
}

#[test]
fn every_event_has_a_cause() {
    let w = earth_timeline();
    let hp = history_params();
    let inputs: Vec<String> = w
        .epochs
        .iter()
        .map(|e| format!("epoch:{}", e.id))
        .chain(hp.settings.iter().map(|s| format!("setting:{}", s.id)))
        .chain(w.catastrophes.iter().map(|c| c.id.clone()))
        .collect();
    for s in seeds(20) {
        let h = run(&w, &hp, &s);
        assert!(!h.events.is_empty());
        for (i, e) in h.events.iter().enumerate() {
            assert!(!e.causes.is_empty(), "{s} {e:?}");
            assert!(!e.why.is_empty());
            for c in &e.causes {
                let earlier = h.events[..i].iter().any(|x| &x.id == c);
                assert!(
                    earlier || inputs.contains(c),
                    "{s}: {} has unknown cause {c}",
                    e.id
                );
            }
        }
        // Life with deep refuges is never wiped out on Earth.
        assert!(h.events.iter().all(|e| e.kind != "sterilised"));
        // Dials are stamped on what depends on them.
        let ox = h.events.iter().find(|e| e.subject == "oxygenic");
        if let Some(ox) = ox {
            assert_eq!(ox.dials, vec!["oxygenic".to_string()]);
        }
    }
}

#[test]
fn catastrophes_cost_diversity_and_can_sterilise() {
    let hp = history_params();
    let mut w = earth_timeline();
    let h = run(&w, &hp, "earth-0");
    let kinds: Vec<&str> = h.events.iter().map(|e| e.kind.as_str()).collect();
    assert!(
        kinds.contains(&"catastrophe") && kinds.contains(&"recovery"),
        "{kinds:?}"
    );
    // Recovery follows its catastrophe.
    for e in h.events.iter().filter(|e| e.kind == "recovery") {
        let c = h.events.iter().find(|x| x.id == e.causes[0]).unwrap();
        assert!(e.at_myr > c.at_myr && e.at_myr - c.at_myr < 100.0);
    }
    // A total event with no refuge sterilises the world; life may start again.
    w.catastrophes.push(crate::history::Catastrophe {
        id: "test.total".into(),
        at_myr: 1500.0,
        kind: "test".into(),
        severity: 1.0,
        refuges: vec![],
        why: "a test event that kills every habitat".into(),
    });
    let mut restarted = 0;
    for s in seeds(20) {
        let h = run(&w, &hp, &s);
        if h.origin_myr.is_some_and(|t| t < 1500.0) {
            assert!(h.events.iter().any(|e| e.kind == "sterilised"), "{s}");
            let after = h
                .events
                .iter()
                .filter(|e| e.kind == "origin" && e.at_myr > 1500.0)
                .count();
            restarted += after;
            let alive = h.points.iter().find(|p| p.at_myr == 1550.0).unwrap();
            assert!(alive.capabilities.iter().all(|c| c == "chemotrophy") || after > 0);
        }
    }
    assert!(restarted > 0, "a second origin in some seeds");
}

#[test]
fn a_cold_thin_biosphere_is_slow() {
    let hp = history_params();
    let earth = run(&earth_timeline(), &hp, "earth-0");
    let last = |h: &LifeHistory| {
        let n = h.points.len();
        (h.points[n - 1].opportunity - h.points[n - 3].opportunity)
            / (h.points[n - 1].at_myr - h.points[n - 3].at_myr)
    };
    let mut slowest = f64::INFINITY;
    for s in seeds(10) {
        let e = run(&europa_timeline(), &hp, &s);
        if e.origin_myr.is_some_and(|t| t < 4000.0) {
            let r = last(&earth) / last(&e);
            println!(
                "{s}: Europa accumulates opportunity {r:.0} times more slowly than Earth today"
            );
            slowest = slowest.min(r);
            assert!(e
                .first
                .keys()
                .all(|c| c == "chemotrophy" || c == "dormancy"));
        }
    }
    assert!(slowest.is_finite() && slowest > 1000.0, "{slowest}");
}

#[test]
fn deterministic_and_recorded_with_causes() {
    let w = earth_timeline();
    let hp = history_params();
    let a = run(&w, &hp, "earth-3");
    assert_eq!(a, run(&w, &hp, "earth-3"));
    assert_ne!(a, run(&w, &hp, "earth-4"));
    let mut ch = sse_chronicle::Chronicle::new();
    let ids = record(&mut ch, &a, 0).unwrap();
    assert_eq!(ids.len(), a.events.len());
    let linked = ch
        .events()
        .iter()
        .filter(|e| !e.body.causes.is_empty())
        .count();
    let internal = a
        .events
        .iter()
        .filter(|e| e.causes.iter().any(|c| c.starts_with("life:")))
        .count();
    assert_eq!(linked, internal);
}

#[test]
fn history_golden() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/life-history.json"
    );
    let got = crate::scenario::history_golden();
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, serde_json::to_string_pretty(&got).unwrap() + "\n").unwrap();
    }
    let want: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("bless once")).unwrap();
    assert_eq!(
        got, want,
        "tests/golden/life-history.json moved: bless only on purpose"
    );
}
