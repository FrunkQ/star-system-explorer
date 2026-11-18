//! K10 acceptance: a stub life suite and stub physics exchange fluxes at
//! common boundaries; the coupled run replays bit-identically (re-run,
//! resume from checkpoint, and fold without the suite).

use crate::*;
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, State, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, Dimension, Draws, Seed, Stream};
use std::collections::BTreeMap;

const GYR: i64 = 31_557_600_000_000_000;

fn world(n: u64) -> Address {
    "u:k10/sys:0"
        .parse::<Address>()
        .unwrap()
        .child("body", n)
        .unwrap()
}
fn num(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Float(f)) => *f,
        Some(Value::Int(i)) => *i as f64,
        other => panic!("{other:?}"),
    }
}

/// Physics: the star brightens; oxygen (from life) cools the world a little.
struct StubPhysics;
impl Physics for StubPhysics {
    fn step(
        &mut self,
        s: &State,
        scope: &Address,
        _from: UniverseTime,
        to: UniverseTime,
    ) -> BTreeMap<Address, BTreeMap<String, Value>> {
        let mut out = BTreeMap::new();
        for (a, f) in s.facts.iter().filter(|(a, _)| scope.contains(a)) {
            if let Some(Value::Float(base)) = f.get("base_temp_k") {
                let o2 = f.get("o2_frac").map_or(0.0, |v| num(Some(v)));
                let t = base + 12.0 * (to.s as f64 / GYR as f64) - 30.0 * o2;
                out.insert(a.clone(), [("temp_k".to_string(), Value::Float(t))].into());
            }
        }
        out
    }
}

/// Life: logistic biomass that needs liquid water, an origin drawn once per
/// world, an oxygen flux back to the air. `rogue` also tries to set pressure.
#[derive(Default)]
struct StubLife {
    manifest: Option<SuiteManifest>,
    pending: Vec<InputDataset>,
    seed: Option<Seed>,
    /// Internal state: boundaries advanced (checkpointed).
    steps: i64,
    rogue: bool,
}
impl StubLife {
    fn new(class: DeterminismClass, rogue: bool) -> Self {
        StubLife {
            manifest: Some(SuiteManifest {
                id: "life".into(),
                version: "0.1.0".into(),
                determinism_class: class,
                requirement_profile: vec![
                    Requirement {
                        field: "temp_k".into(),
                        dim: Dimension::TEMPERATURE,
                        required: true,
                        description: "surface temperature".into(),
                    },
                    Requirement {
                        field: "o2_frac".into(),
                        dim: Dimension::NONE,
                        required: true,
                        description: "oxygen fraction of the air".into(),
                    },
                ],
                owns: vec!["life.biomass".into(), "life.origin_s".into()],
                may_flux: vec!["o2_frac".into()],
            }),
            rogue,
            ..Default::default()
        }
    }
}
impl SuiteAdapter for StubLife {
    fn manifest(&self) -> &SuiteManifest {
        self.manifest.as_ref().unwrap()
    }
    fn provide(&mut self, seed: &Seed, input: &InputDataset) -> Result<(), SuiteError> {
        self.seed = Some(*seed);
        self.pending.push(input.clone());
        Ok(())
    }
    fn advance(&mut self, to: UniverseTime) -> Result<AdvanceOutput, SuiteError> {
        self.steps += 1;
        let mut out = AdvanceOutput::default();
        for inp in std::mem::take(&mut self.pending) {
            let dt = (inp.to.s - inp.from.s) as f64 / GYR as f64;
            let temp = num(inp.values.get("temp_k"));
            let mut b = inp.own.get("life.biomass").map_or(0.0, |v| num(Some(v)));
            let wet = (273.0..373.0).contains(&temp);
            if b == 0.0 && wet {
                let d = Draws::new(
                    self.seed.unwrap(),
                    inp.address.clone(),
                    Stream::new("life.origin.v1").unwrap(),
                );
                if d.unit(inp.to.s as u64) < 0.5 {
                    b = 0.01;
                    out.events.push(SuiteEvent {
                        kind: "origin".into(),
                        subject: inp.address.clone(),
                        set: [("life.origin_s".to_string(), Value::Int(to.s as i128))].into(),
                    });
                }
            } else if b > 0.0 {
                let r = if wet { 2.0 } else { -1.0 };
                b = (b + r * b * (1.0 - b) * dt).clamp(0.0, 1.0);
            }
            out.own.insert(
                inp.address.clone(),
                [("life.biomass".to_string(), Value::Float(b))].into(),
            );
            if b > 0.0 {
                out.fluxes.push(Flux {
                    address: inp.address.clone(),
                    field: "o2_frac".into(),
                    delta: 0.05 * b * dt,
                });
                out.signatures.insert(
                    inp.address.clone(),
                    [("biosignature".to_string(), Value::Float(b * 0.8))].into(),
                );
            }
            if self.rogue {
                out.fluxes.push(Flux {
                    address: inp.address.clone(),
                    field: "pressure_pa".into(),
                    delta: 1.0e5,
                });
                out.own.insert(
                    inp.address.clone(),
                    [
                        ("life.biomass".to_string(), Value::Float(b)),
                        ("temp_k".to_string(), Value::Float(999.0)),
                    ]
                    .into(),
                );
            }
        }
        Ok(out)
    }
    fn checkpoint(&self) -> Value {
        Value::Int(self.steps as i128)
    }
    fn restore(&mut self, s: &Value) -> Result<(), SuiteError> {
        match s {
            Value::Int(i) => {
                self.steps = *i as i64;
                Ok(())
            }
            _ => Err(SuiteError("bad checkpoint".into())),
        }
    }
}

fn start() -> Chronicle {
    let mut c = Chronicle::new();
    let mut set = BTreeMap::new();
    for (i, base) in [(0u64, 262.0), (1, 285.0), (2, 140.0)] {
        set.insert(
            world(i),
            [
                ("base_temp_k".to_string(), Value::Float(base)),
                ("o2_frac".to_string(), Value::Float(0.0)),
                ("pressure_pa".to_string(), Value::Float(1.0e5)),
            ]
            .into(),
        );
    }
    let e = Event::new(EventBody {
        kind: kinds::RESOLVE.into(),
        subject: set.keys().cloned().collect(),
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: c.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            ..Default::default()
        },
        evidence: Evidence::Resolved,
        publisher: Publisher::new("test", "seed", "1"),
    })
    .unwrap();
    c.append(e).unwrap();
    c
}

fn coordinator(class: DeterminismClass, rogue: bool) -> Coordinator {
    let mut k =
        Coordinator::new(Seed::from_text("k10"), "u:k10".parse().unwrap(), GYR / 4).unwrap();
    k.set_physics(Box::new(StubPhysics));
    k.install(Box::new(StubLife::new(class, rogue))).unwrap();
    k
}

fn publisher() -> Publisher {
    Publisher::new("sse.kernel", "coordinator", "1")
}

fn full_run() -> (Chronicle, RunReport) {
    let mut c = start();
    let r = coordinator(DeterminismClass::A, false)
        .run(
            &mut c,
            UniverseTime::ORIGIN,
            UniverseTime::seconds(5 * GYR),
            &publisher(),
        )
        .unwrap();
    (c, r)
}

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

#[test]
fn golden_digest() {
    let (c, r) = full_run();
    let got =
        serde_json::json!({ "events": c.len(), "head": r.head.hex(), "state": r.state_hash.hex() });
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
