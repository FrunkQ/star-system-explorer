//! K10 acceptance: a stub life suite and stub physics exchange fluxes at
//! common boundaries; the coupled run replays bit-identically (re-run,
//! resume from checkpoint, and fold without the suite).

use crate::*;
use sse_chronicle::{Chronicle, Event, UniverseTime};
use sse_core::cbor::Value;
use sse_core::Seed;

use crate::scenario::{coordinator, full_run, num, publisher, start, world, StubLife, GYR};

/// Acceptance, part 1: the suite and physics exchange fluxes. Life raises
/// oxygen, oxygen cools the world in the next physics step.
#[test]
fn stub_suite_exchanges_fluxes() {
    let (c, r) = full_run();
    assert_eq!(r.boundaries, 20);
    assert!(r.violations.is_empty());
    let end = c.state_at(UniverseTime::seconds(5 * GYR));
    let living: Vec<u64> = (0..3)
        .filter(|i| num(end.get(&world(*i), "life.biomass")) > 0.0)
        .collect();
    assert!(!living.is_empty(), "some world came alive");
    for i in &living {
        let w = world(*i);
        let o2 = num(end.get(&w, "o2_frac"));
        assert!(o2 > 0.0, "oxygen came from life");
        let base = num(end.get(&w, "base_temp_k"));
        let t = num(end.get(&w, "temp_k"));
        // Physics read the oxygen at its last step: cooler than without life.
        assert!(t < base + 12.0 * 5.0, "{t}");
        assert!(end.get(&w, "signature.life.biosignature").is_some());
        assert!(end.get(&w, "life.origin_s").is_some());
    }
    // The frozen world never had liquid water: no life, no oxygen.
    assert_eq!(num(end.get(&world(2), "life.biomass")), 0.0);
    assert_eq!(num(end.get(&world(2), "o2_frac")), 0.0);
    // Every boundary event cites the physics step of its boundary and the previous boundary.
    let advances: Vec<&Event> = c
        .events()
        .iter()
        .filter(|e| e.body.kind == "suite.life.advance")
        .collect();
    assert_eq!(advances.len(), 20);
    assert_eq!(advances[1].body.causes.len(), 2);
    assert_eq!(advances[1].body.causes[0], advances[0].id);
}

/// Acceptance, part 2: the coupled run replays bit-identically.
#[test]
fn coupled_run_replays_bit_identically() {
    let (a, ra) = full_run();
    let (b, rb) = full_run();
    assert_eq!(ra, rb);
    assert_eq!(a.head(), b.head());
    // Stop half way, then resume with a fresh coordinator from the chronicle.
    let mut c = start();
    coordinator(DeterminismClass::A, false)
        .run(
            &mut c,
            UniverseTime::ORIGIN,
            UniverseTime::seconds(2 * GYR),
            &publisher(),
        )
        .unwrap();
    let mut k = coordinator(DeterminismClass::A, false);
    k.resume_from(&c).unwrap();
    let rc = k
        .run(
            &mut c,
            UniverseTime::seconds(2 * GYR),
            UniverseTime::seconds(5 * GYR),
            &publisher(),
        )
        .unwrap();
    assert_eq!(
        c.head(),
        a.head(),
        "resume gives the same chronicle as one run"
    );
    assert_eq!(rc.state_hash, ra.state_hash);
    // Class B: a player device folds the recorded results without the suite.
    let mut cb = start();
    let rb = coordinator(DeterminismClass::B, false)
        .run(
            &mut cb,
            UniverseTime::ORIGIN,
            UniverseTime::seconds(5 * GYR),
            &publisher(),
        )
        .unwrap();
    let mut copy = Chronicle::new();
    for e in cb.events() {
        copy.append(e.clone()).unwrap();
    }
    assert_eq!(
        replay_state(&copy, UniverseTime::seconds(5 * GYR)).hash(),
        rb.state_hash
    );
    // Every intermediate boundary reads back the same way.
    for k in 0..=20 {
        let t = UniverseTime::seconds(k * GYR / 4);
        assert_eq!(replay_state(&copy, t).hash(), cb.state_at(t).hash());
    }
}

/// The life suite may change the air; it may not override pressure or
/// temperature. Attempts are dropped and reported, never applied.
#[test]
fn suites_cannot_override_physics() {
    let mut c = start();
    let r = coordinator(DeterminismClass::A, true)
        .run(
            &mut c,
            UniverseTime::ORIGIN,
            UniverseTime::seconds(GYR),
            &publisher(),
        )
        .unwrap();
    let end = c.state_at(UniverseTime::seconds(GYR));
    for i in 0..3 {
        assert_eq!(num(end.get(&world(i), "pressure_pa")), 1.0e5);
        assert_ne!(num(end.get(&world(i), "temp_k")), 999.0);
    }
    assert!(r
        .violations
        .iter()
        .any(|v| v.field == "pressure_pa" && v.reason.contains("physics owns it")));
    assert!(r
        .violations
        .iter()
        .any(|v| v.field == "temp_k" && v.reason.contains("does not own")));
    // And they are on the record.
    let e = c
        .events()
        .iter()
        .find(|e| e.body.kind == "suite.life.advance")
        .unwrap();
    let Some(Value::Map(d)) = &e.body.payload.data else {
        panic!()
    };
    assert!(matches!(&d["violations"], Value::Array(v) if !v.is_empty()));
}

#[test]
fn socket_basics() {
    let mut k = coordinator(DeterminismClass::A, false);
    assert!(k
        .install(Box::new(StubLife::new(DeterminismClass::A, false)))
        .unwrap_err()
        .to_string()
        .contains("installed twice"));
    assert!(Coordinator::new(Seed::from_text("x"), "u:x".parse().unwrap(), 0).is_err());
    // The L0 empty life suite: installed, does nothing, says why.
    let mut k = Coordinator::new(Seed::from_text("x"), "u:k10".parse().unwrap(), GYR).unwrap();
    k.install(Box::new(NullSuite::life())).unwrap();
    let mut c = start();
    let r = k
        .run(
            &mut c,
            UniverseTime::ORIGIN,
            UniverseTime::seconds(2 * GYR),
            &publisher(),
        )
        .unwrap();
    assert_eq!(r.boundaries, 2);
    assert_eq!(
        k.explain("life", &world(0), "life.biomass").as_deref(),
        Some("no life model installed")
    );
}

/// Golden digest; the same function runs in the browser harness.
#[test]
fn golden_digest() {
    let got = crate::scenario::golden();
    let path = format!(
        "{}/../../tests/golden/suite.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1");
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden suite.json changed"
    );
}
