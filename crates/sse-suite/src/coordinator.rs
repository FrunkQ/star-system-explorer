//! The coordinator at common time boundaries (H11).

use crate::socket::{DeterminismClass, InputDataset, Physics, SuiteAdapter};
use serde::{Deserialize, Serialize};
use sse_chronicle::{
    kinds, Chronicle, ChronicleError, Event, EventBody, Evidence, Payload, Publisher, State,
    UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash, Seed};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Something a suite tried to do outside its declared rights. Dropped and
/// reported, never applied, never silent.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Violation {
    pub suite: String,
    pub at_s: i64,
    pub address: Address,
    pub field: String,
    pub reason: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RunReport {
    pub boundaries: u64,
    pub events: u64,
    pub violations: Vec<Violation>,
    pub head: ContentHash,
    pub state_hash: ContentHash,
}

#[derive(Debug)]
pub enum CoordinatorError {
    DuplicateSuite(String),
    BadSuiteId(String),
    BadStep,
    Suite { suite: String, message: String },
    Chronicle(ChronicleError),
}
impl fmt::Display for CoordinatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoordinatorError::DuplicateSuite(s) => write!(f, "suite {s} is installed twice"),
            CoordinatorError::BadSuiteId(s) => {
                write!(f, "suite id {s:?} must be lowercase letters, digits and _")
            }
            CoordinatorError::BadStep => write!(f, "boundary step must be positive"),
            CoordinatorError::Suite { suite, message } => write!(f, "suite {suite}: {message}"),
            CoordinatorError::Chronicle(e) => write!(f, "{e}"),
        }
    }
}
impl From<ChronicleError> for CoordinatorError {
    fn from(e: ChronicleError) -> Self {
        CoordinatorError::Chronicle(e)
    }
}
impl From<sse_chronicle::EventError> for CoordinatorError {
    fn from(e: sse_chronicle::EventError) -> Self {
        CoordinatorError::Chronicle(e.into())
    }
}

pub struct Coordinator {
    pub seed: Seed,
    pub scope: Address,
    /// Seconds between common boundaries.
    pub step_s: i64,
    suites: BTreeMap<String, Box<dyn SuiteAdapter>>,
    physics: Option<Box<dyn Physics>>,
    last: BTreeMap<String, ContentHash>,
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)] // one place that builds coordinator events
fn event(
    c: &Chronicle,
    kind: String,
    subject: Vec<Address>,
    at: UniverseTime,
    causes: Vec<ContentHash>,
    set: BTreeMap<Address, BTreeMap<String, Value>>,
    data: Option<Value>,
    publisher: &Publisher,
) -> Result<Event, sse_chronicle::EventError> {
    Event::new(EventBody {
        kind,
        subject,
        occurred_at: at,
        activated_at: at,
        recognised_at: c.next_stamp(),
        causes,
        payload: Payload {
            set,
            data,
            ..Default::default()
        },
        evidence: Evidence::Derived,
        publisher: publisher.clone(),
    })
}

impl Coordinator {
    pub fn new(seed: Seed, scope: Address, step_s: i64) -> Result<Self, CoordinatorError> {
        if step_s <= 0 {
            return Err(CoordinatorError::BadStep);
        }
        Ok(Coordinator {
            seed,
            scope,
            step_s,
            suites: BTreeMap::new(),
            physics: None,
            last: BTreeMap::new(),
        })
    }

    pub fn install(&mut self, s: Box<dyn SuiteAdapter>) -> Result<(), CoordinatorError> {
        let id = s.manifest().id.clone();
        let ok = !id.is_empty()
            && id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_');
        if !ok {
            return Err(CoordinatorError::BadSuiteId(id));
        }
        if self.suites.contains_key(&id) {
            return Err(CoordinatorError::DuplicateSuite(id));
        }
        self.suites.insert(id, s);
        Ok(())
    }

    pub fn set_physics(&mut self, p: Box<dyn Physics>) {
        self.physics = Some(p);
    }

    pub fn explain(&self, suite: &str, a: &Address, field: &str) -> Option<String> {
        self.suites.get(suite)?.explain(a, field)
    }

    /// Pick up after an earlier run: each suite is restored from the
    /// checkpoint in its last recorded boundary event.
    pub fn resume_from(&mut self, c: &Chronicle) -> Result<(), CoordinatorError> {
        for (id, s) in self.suites.iter_mut() {
            let kind = format!("suite.{id}.advance");
            if let Some(e) = c.events().iter().rev().find(|e| e.body.kind == kind) {
                if let Some(Value::Map(d)) = &e.body.payload.data {
                    if let Some(cp) = d.get("checkpoint") {
                        s.restore(cp).map_err(|x| CoordinatorError::Suite {
                            suite: id.clone(),
                            message: x.0,
                        })?;
                    }
                }
                self.last.insert(id.clone(), e.id);
            }
        }
        Ok(())
    }

    /// Run boundaries from `from` (exclusive) to `to` (inclusive), every
    /// `step_s` seconds, the last one landing exactly on `to`.
    pub fn run(
        &mut self,
        c: &mut Chronicle,
        from: UniverseTime,
        to: UniverseTime,
        publisher: &Publisher,
    ) -> Result<RunReport, CoordinatorError> {
        let start_len = c.len();
        let mut violations = Vec::new();
        let mut t0 = from;
        let mut n = 0u64;
        while t0 < to {
            let t1 = UniverseTime::seconds(t0.s.saturating_add(self.step_s).min(to.s));
            self.boundary(c, t0, t1, publisher, &mut violations)?;
            t0 = t1;
            n += 1;
        }
        Ok(RunReport {
            boundaries: n,
            events: c.len() - start_len,
            violations,
            head: c.head(),
            state_hash: c.state_at(to).hash(),
        })
    }

    fn boundary(
        &mut self,
        c: &mut Chronicle,
        t0: UniverseTime,
        t1: UniverseTime,
        publisher: &Publisher,
        violations: &mut Vec<Violation>,
    ) -> Result<(), CoordinatorError> {
        // 1. Physics first: it owns physical truth for the interval.
        let mut physics_event = None;
        if let Some(p) = self.physics.as_mut() {
            let sets = p.step(&c.state_at(t0), &self.scope, t0, t1);
            if !sets.is_empty() {
                let subject: Vec<Address> = sets.keys().cloned().collect();
                let e = event(
                    c,
                    kinds::EVOLVE_TRANSITION.into(),
                    subject,
                    t1,
                    vec![],
                    sets,
                    None,
                    publisher,
                )?;
                physics_event = Some(c.append(e)?);
            }
        }
        // 2. Each suite, in id order, sees the same physical state at t1 plus
        //    what suites before it committed this boundary.
        let ids: Vec<String> = self.suites.keys().cloned().collect();
        for id in ids {
            let state = c.state_at(t1);
            let suite = self.suites.get_mut(&id).expect("installed");
            let m = suite.manifest().clone();
            let mut inputs = Vec::new();
            for a in state.facts.keys().filter(|a| self.scope.contains(a)) {
                let facts = &state.facts[a];
                let mut values = BTreeMap::new();
                let mut ok = true;
                for r in &m.requirement_profile {
                    match facts.get(&r.field) {
                        Some(v) => {
                            values.insert(r.field.clone(), v.clone());
                        }
                        None if r.required => ok = false,
                        None => {}
                    }
                }
                if !ok || m.requirement_profile.is_empty() {
                    continue;
                }
                let own = m
                    .owns
                    .iter()
                    .filter_map(|f| facts.get(f).map(|v| (f.clone(), v.clone())))
                    .collect();
                let input = InputDataset {
                    address: a.clone(),
                    from: t0,
                    to: t1,
                    values,
                    own,
                };
                inputs.push(Value::Text(input.hash().hex()));
                suite
                    .provide(&self.seed, &input)
                    .map_err(|x| CoordinatorError::Suite {
                        suite: id.clone(),
                        message: x.0,
                    })?;
            }
            let out = suite.advance(t1).map_err(|x| CoordinatorError::Suite {
                suite: id.clone(),
                message: x.0,
            })?;
            // 3. Check rights, apply fluxes to the physical state.
            let owns: BTreeSet<&String> = m.owns.iter().collect();
            let may: BTreeSet<&String> = m.may_flux.iter().collect();
            let mut set: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
            let bad = |a: &Address, f: &str, why: &str, v: &mut Vec<Violation>| {
                v.push(Violation {
                    suite: id.clone(),
                    at_s: t1.s,
                    address: a.clone(),
                    field: f.into(),
                    reason: why.into(),
                })
            };
            for (a, fields) in &out.own {
                for (f, v) in fields {
                    if !self.scope.contains(a) {
                        bad(a, f, "outside the coordinator's scope", violations);
                    } else if owns.contains(f) {
                        set.entry(a.clone())
                            .or_default()
                            .insert(f.clone(), v.clone());
                    } else {
                        bad(a, f, "the suite does not own this field", violations);
                    }
                }
            }
            for fx in &out.fluxes {
                if !may.contains(&fx.field) {
                    bad(
                        &fx.address,
                        &fx.field,
                        "fluxes into this field are not allowed (physics owns it)",
                        violations,
                    );
                    continue;
                }
                if !self.scope.contains(&fx.address) || !fx.delta.is_finite() {
                    bad(
                        &fx.address,
                        &fx.field,
                        "flux outside scope or not finite",
                        violations,
                    );
                    continue;
                }
                let current = set
                    .get(&fx.address)
                    .and_then(|m| m.get(&fx.field))
                    .or_else(|| state.get(&fx.address, &fx.field))
                    .and_then(num);
                match current {
                    Some(x) => {
                        set.entry(fx.address.clone())
                            .or_default()
                            .insert(fx.field.clone(), Value::Float(x + fx.delta));
                    }
                    None => bad(
                        &fx.address,
                        &fx.field,
                        "flux into a missing or non-numeric field",
                        violations,
                    ),
                }
            }
            for (a, sig) in &out.signatures {
                for (k, v) in sig {
                    set.entry(a.clone())
                        .or_default()
                        .insert(format!("signature.{id}.{k}"), v.clone());
                }
            }
            // 4. Commit: one boundary event per suite, then its own events.
            let mut causes: Vec<ContentHash> = self.last.get(&id).into_iter().copied().collect();
            causes.extend(physics_event);
            let mut subject: Vec<Address> = set.keys().cloned().collect();
            if subject.is_empty() {
                subject.push(self.scope.clone());
            }
            let mine: Vec<Value> = violations
                .iter()
                .filter(|v| v.suite == id && v.at_s == t1.s)
                .map(|v| Value::Text(format!("{} {}: {}", v.address, v.field, v.reason)))
                .collect();
            let class = match m.determinism_class {
                DeterminismClass::A => "A",
                DeterminismClass::B => "B",
            };
            let data = Value::Map(
                [
                    ("class".to_string(), Value::Text(class.into())),
                    ("inputs".to_string(), Value::Array(inputs)),
                    ("checkpoint".to_string(), suite.checkpoint()),
                    ("violations".to_string(), Value::Array(mine)),
                    ("suite_version".to_string(), Value::Text(m.version.clone())),
                ]
                .into(),
            );
            let e = event(
                c,
                format!("suite.{id}.advance"),
                subject,
                t1,
                causes,
                set,
                Some(data),
                publisher,
            )?;
            let adv = c.append(e)?;
            self.last.insert(id.clone(), adv);
            for se in out.events {
                if !self.scope.contains(&se.subject) {
                    bad(
                        &se.subject,
                        &se.kind,
                        "event outside the coordinator's scope",
                        violations,
                    );
                    continue;
                }
                let mut s2 = BTreeMap::new();
                if !se.set.is_empty() {
                    let (ok, rest): (BTreeMap<_, _>, BTreeMap<_, _>) =
                        se.set.into_iter().partition(|(f, _)| owns.contains(f));
                    for f in rest.keys() {
                        bad(
                            &se.subject,
                            f,
                            "the suite does not own this field",
                            violations,
                        );
                    }
                    if !ok.is_empty() {
                        s2.insert(se.subject.clone(), ok);
                    }
                }
                let e = event(
                    c,
                    format!("suite.{id}.{}", se.kind),
                    vec![se.subject],
                    t1,
                    vec![adv],
                    s2,
                    None,
                    publisher,
                )?;
                c.append(e)?;
            }
        }
        Ok(())
    }
}

/// State at `at` from the chronicle alone, with no suite installed: what a
/// player device does for class B suites, and the replay check for class A.
pub fn replay_state(c: &Chronicle, at: UniverseTime) -> State {
    c.state_at(at)
}
