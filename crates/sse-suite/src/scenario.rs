//! The golden scenario and the stub suites used to prove the socket: stub
//! physics (a brightening star, oxygen cooling) and a stub life suite
//! (logistic biomass, an origin per world, an oxygen flux). Used by the
//! tests and by the browser harness.

use crate::*;
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, State, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, Dimension, Draws, Seed, Stream};
use std::collections::BTreeMap;

pub const GYR: i64 = 31_557_600_000_000_000;

pub fn world(n: u64) -> Address {
    "u:k10/sys:0"
        .parse::<Address>()
        .unwrap()
        .child("body", n)
        .unwrap()
}
pub fn num(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Float(f)) => *f,
        Some(Value::Int(i)) => *i as f64,
        other => panic!("{other:?}"),
    }
}

/// Physics: the star brightens; oxygen (from life) cools the world a little.
pub struct StubPhysics;
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
pub struct StubLife {
    manifest: Option<SuiteManifest>,
    pending: Vec<InputDataset>,
    seed: Option<Seed>,
    /// Internal state: boundaries advanced (checkpointed).
    steps: i64,
    rogue: bool,
}
impl StubLife {
    pub fn new(class: DeterminismClass, rogue: bool) -> Self {
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

pub fn start() -> Chronicle {
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

pub fn coordinator(class: DeterminismClass, rogue: bool) -> Coordinator {
    let mut k =
        Coordinator::new(Seed::from_text("k10"), "u:k10".parse().unwrap(), GYR / 4).unwrap();
    k.set_physics(Box::new(StubPhysics));
    k.install(Box::new(StubLife::new(class, rogue))).unwrap();
    k
}

pub fn publisher() -> Publisher {
    Publisher::new("sse.kernel", "coordinator", "1")
}

pub fn full_run() -> (Chronicle, RunReport) {
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

pub fn golden() -> serde_json::Value {
    let (c, r) = full_run();
    serde_json::json!({ "events": c.len(), "head": r.head.hex(), "state": r.state_hash.hex() })
}
