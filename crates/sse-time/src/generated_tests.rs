//! F7: generated systems aged through their long life.
use crate::generated::*;

use crate::scenario::{generated_runs as runs, sunlike, target};

#[test]
fn generated_systems_age_with_one_story() {
    let settings = crate::scenario::settings();
    let mut engulfed = 0;
    let mut oceans_lost = 0;
    for r in runs(10) {
        let t = target(&r);
        let a = long_life(&r, &settings, t).unwrap();
        let b = long_life(&r, &settings, t).unwrap();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert_eq!(a.spec.planets.len(), r.worlds.len());
        for w in a.story.windows(2) {
            assert!(
                w[0].age_myr <= w[1].age_myr,
                "{} then {}",
                w[0].kind,
                w[1].kind
            );
        }
        let ocean: Vec<&str> = a
            .spec
            .planets
            .iter()
            .filter(|p| p.oceans)
            .map(|p| p.address.as_str())
            .collect();
        for e in &a.run.events {
            match e.kind.as_str() {
                "engulfed" => engulfed += 1,
                "oceans_lost" => {
                    oceans_lost += 1;
                    assert!(
                        e.subjects.iter().any(|s| ocean.contains(&s.as_str())),
                        "{e:?}"
                    );
                }
                _ => {}
            }
        }
        // The formation story comes first: nothing from ageing predates now.
        let now = r.history.inheritance.age_gyr * 1000.0;
        assert!(a
            .story
            .iter()
            .filter(|s| s.from == "ageing")
            .all(|s| s.age_myr >= now - 1e-6));
    }
    eprintln!("F7 generated: engulfed {engulfed}, oceans lost {oceans_lost}");
}

#[test]
fn sunlike_systems_lose_their_inner_worlds() {
    let settings = crate::scenario::settings();
    let (mut close, mut swallowed, mut ocean_worlds, mut boiled) = (0, 0, 0, 0);
    for n in 0..12 {
        let r = sunlike(n);
        let a = long_life(&r, &settings, target(&r)).unwrap();
        // Metal-rich suns live longer: only systems whose star reaches its end
        // in the run can have lost worlds to it.
        if !a.run.events.iter().any(|e| e.kind == "planetary_nebula") {
            continue;
        }
        for p in &a.spec.planets {
            let gone = a
                .run
                .events
                .iter()
                .any(|e| e.kind == "engulfed" && e.subjects.contains(&p.address));
            if p.a_au * (1.0 - p.e) < 0.3 {
                close += 1;
                swallowed += usize::from(gone);
            }
            if p.a_au > 2.5 {
                assert!(!gone, "{} at {} au engulfed", p.address, p.a_au);
            }
            if p.oceans && p.a_au < 1.5 {
                ocean_worlds += 1;
                boiled += usize::from(
                    a.run
                        .events
                        .iter()
                        .any(|e| e.kind == "oceans_lost" && e.subjects.contains(&p.address)),
                );
            }
        }
    }
    eprintln!("F7 sun-like: {swallowed} of {close} close worlds engulfed; {boiled} of {ocean_worlds} inner ocean worlds boiled");
    assert!(close > 0 && swallowed == close, "{swallowed} of {close}");
    assert!(
        ocean_worlds == 0 || boiled == ocean_worlds,
        "{boiled} of {ocean_worlds}"
    );
}

#[test]
fn generated_golden() {
    let path = format!(
        "{}/../../tests/golden/time-generated.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&crate::scenario::generated_golden()).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1");
    assert_eq!(
        text,
        want.replace("\r\n", "\n"),
        "golden time-generated.json changed"
    );
}
