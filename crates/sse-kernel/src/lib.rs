//! SSE4 kernel: the one message API in front of the substrate and engines
//! (H9.1). The same `Kernel::handle_json` runs natively (CLI, tests) and in
//! the browser worker (sse-wasm), so a contract test can prove both agree.
#![forbid(unsafe_code)]
// Errors are full protocol responses on a cold path; boxing them buys nothing.
#![allow(clippy::result_large_err)]

pub mod contract;
pub mod protocol;

use protocol::*;
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, RunManifest, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::hash::Hasher;
use sse_core::{Address, ContentHash, Draws, Quantity, Seed, Stream};
use sse_flow::{Context, Flow, InputValue, SourceLabel};
use sse_pack::ResolvedStack;
use std::collections::BTreeMap;

pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

struct Universe {
    root: Address,
    seed: Seed,
    chronicle: Chronicle,
    branches: BTreeMap<String, ContentHash>,
    /// The pack stack the universe runs on now (bottom to top).
    packs: Vec<sse_chronicle::PackRef>,
}

/// Event kind for a pack stack change in an open universe.
pub const STACK_ADOPT: &str = "stack.adopt";

fn pack_refs(s: &ResolvedStack) -> Vec<sse_chronicle::PackRef> {
    s.packs
        .iter()
        .map(|(id, v, h)| sse_chronicle::PackRef {
            id: id.clone(),
            version: v.clone(),
            content_hash: *h,
        })
        .collect()
}

/// Data views, disclosure and tag rules from a pack stack.
fn load_observe(ms: &[sse_pack::PackManifest]) -> Result<Observe, Vec<String>> {
    let mut found = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("observe") {
            let j = serde_json::to_value(v).map_err(|e| vec![format!("{}: observe: {e}", p.id)])?;
            found.push((p.id.clone(), j));
        }
    }
    let setup = sse_observe::ObserveSetup::from_stack(&found).map_err(|e| vec![e])?;
    let rules = sse_sift::load(ms)?.rules;
    Ok(Observe { setup, rules })
}

/// Field prefix for an authored tag: `tag.<name>` = true puts the tag on the
/// subject (false or absent: not tagged). Taken out of the facts before a
/// scene is built, so it never shows as a fact.
pub const TAG_FIELD: &str = "tag.";

impl Universe {
    fn manifest(&self) -> RunManifest {
        RunManifest::new(
            self.seed,
            KERNEL_VERSION,
            self.packs.clone(),
            self.chronicle.authored_hash(),
        )
    }

    /// Record that the universe now runs on `stack`: an authored event at the
    /// root (the GM's choice), whose facts name the stack hash and every pack
    /// by id, version and content hash. Re-adopting the same stack records
    /// nothing.
    fn adopt(&mut self, stack: &ResolvedStack) -> Result<Option<StackAdopted>, Response> {
        let packs = pack_refs(stack);
        if packs == self.packs {
            return Ok(None);
        }
        let mut f = BTreeMap::new();
        f.insert("stack.hash".to_string(), Value::Text(stack.hash.hex()));
        f.insert(
            "stack.packs".to_string(),
            Value::Array(
                packs
                    .iter()
                    .map(|p| {
                        Value::Text(format!("{}@{}#{}", p.id, p.version, p.content_hash.hex()))
                    })
                    .collect(),
            ),
        );
        let mut set = BTreeMap::new();
        set.insert(self.root.clone(), f);
        let e = Event::new(EventBody {
            kind: STACK_ADOPT.into(),
            subject: vec![self.root.clone()],
            occurred_at: UniverseTime::ORIGIN,
            activated_at: UniverseTime::ORIGIN,
            recognised_at: self.chronicle.next_stamp(),
            causes: vec![],
            payload: Payload {
                set,
                ..Default::default()
            },
            evidence: Evidence::Authored,
            publisher: Publisher::new("gm", "stack", "1"),
        })
        .map_err(|e| err("bad_event", format!("{e}")))?;
        let id = self
            .chronicle
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        self.packs = packs;
        Ok(Some(StackAdopted {
            event_id: id.hex(),
            state_hash: self
                .chronicle
                .state_at(UniverseTime::seconds(i64::MAX))
                .hash()
                .hex(),
            manifest_id: self.manifest().id().hex(),
        }))
    }
}

struct Job {
    total: u32,
    done: u32,
    hasher: Hasher,
}

/// What the stack says about observation: data views and disclosure
/// (`provides.other.observe`) and the rule pack's tags (`provides.other.rules`).
#[derive(Default)]
struct Observe {
    setup: Option<sse_observe::ObserveSetup>,
    rules: sse_sift::RuleSet,
}

pub struct Kernel {
    atoms: sse_flow::AtomRegistry,
    stack: Option<(ResolvedStack, BTreeMap<String, Flow>)>,
    observe: Observe,
    universe: Option<Universe>,
    jobs: BTreeMap<u32, Job>,
    next_job: u32,
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

fn err(code: &str, message: impl Into<String>) -> Response {
    Response::Error {
        code: code.into(),
        message: message.into(),
    }
}

fn json_to_value(v: &serde_json::Value) -> Result<Value, String> {
    serde_json::from_value::<Value>(v.clone()).map_err(|e| e.to_string())
}

fn source_label(s: &Option<String>) -> Result<SourceLabel, String> {
    Ok(match s.as_deref().unwrap_or("authored") {
        "catalogued" => SourceLabel::Catalogued,
        "resolved" => SourceLabel::Resolved,
        "derived" => SourceLabel::Derived,
        "authored" => SourceLabel::Authored,
        "authored-impossible" => SourceLabel::AuthoredImpossible,
        other => return Err(format!("unknown source label {other:?}")),
    })
}

/// One benchmark work unit: a fixed 12-step Kepler solve.
fn bench_unit(i: u32) -> f64 {
    use sse_core::math as m;
    let e = (i % 97) as f64 / 100.0;
    let mean = (i % 360) as f64 * (m::PI / 180.0);
    let mut ea = if e < 0.8 { mean } else { m::PI };
    for _ in 0..12 {
        ea -= (ea - e * m::sin(ea) - mean) / (1.0 - e * m::cos(ea));
    }
    ea
}

/// The toy atoms (contract fixtures) plus the real library (`sse-atoms`, P1),
/// so `load_stack` can take `packs/base-physics/pack.json`. Ids never clash:
/// `toy.*` against `sse.*`.
fn kernel_atoms() -> sse_flow::AtomRegistry {
    let mut r = sse_flow::toy::registry();
    for a in sse_atoms::atoms() {
        r.insert(a);
    }
    r
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {
            atoms: kernel_atoms(),
            stack: None,
            observe: Observe::default(),
            universe: None,
            jobs: BTreeMap::new(),
            next_job: 1,
        }
    }

    /// Handle one JSON request; always returns one JSON response.
    pub fn handle_json(&mut self, req: &str) -> String {
        let resp = match serde_json::from_str::<Request>(req) {
            Ok(r) => self.handle(r),
            Err(e) => err("bad_request", e.to_string()),
        };
        serde_json::to_string(&resp).expect("responses serialise")
    }

    pub fn handle(&mut self, req: Request) -> Response {
        match self.dispatch(req) {
            Ok(r) => r,
            Err(r) => r,
        }
    }

    fn universe(&mut self) -> Result<&mut Universe, Response> {
        self.universe
            .as_mut()
            .ok_or_else(|| err("no_universe", "open a universe first"))
    }

    fn edit_event(
        u: &Universe,
        edit: &AuthoredEdit,
        stamp: sse_chronicle::SessionStamp,
    ) -> Result<Event, Response> {
        let a: Address = edit
            .address
            .parse()
            .map_err(|e| err("bad_address", format!("{e}")))?;
        if !u.root.contains(&a) {
            return Err(err(
                "bad_address",
                format!("{a} is not in universe {}", u.root),
            ));
        }
        let v = json_to_value(&edit.value).map_err(|e| err("bad_value", e))?;
        let mut f = BTreeMap::new();
        f.insert(edit.field.clone(), v);
        let mut set = BTreeMap::new();
        set.insert(a.clone(), f);
        let t = UniverseTime::seconds(edit.occurred_at_s);
        Event::new(EventBody {
            kind: kinds::AUTHORED_SET.into(),
            subject: vec![a],
            occurred_at: t,
            activated_at: t,
            recognised_at: stamp,
            causes: vec![],
            payload: Payload {
                set,
                ..Default::default()
            },
            evidence: if edit.impossible {
                Evidence::AuthoredImpossible
            } else {
                Evidence::Authored
            },
            publisher: Publisher::new("gm", "edit", "1"),
        })
        .map_err(|e| err("bad_event", format!("{e}")))
    }

    fn flow_inputs(
        &self,
        flow: &Flow,
        inputs: &BTreeMap<String, InputSpec>,
        units: &sse_core::Units,
    ) -> Result<BTreeMap<String, InputValue>, Response> {
        let mut out = BTreeMap::new();
        let kinds = &self.stack.as_ref().unwrap().0.kinds;
        for fi in &flow.doc.inputs {
            let spec = inputs
                .get(&fi.name)
                .ok_or_else(|| err("missing_input", format!("input {} is required", fi.name)))?;
            let dim = kinds[&fi.kind].dim;
            let q = match &spec.unit {
                Some(u) => units
                    .q(spec.value, u)
                    .map_err(|e| err("bad_unit", e.to_string()))?,
                None => Quantity::si(spec.value, dim),
            };
            q.expect(dim, &fi.name)
                .map_err(|e| err("dimension", e.to_string()))?;
            let source = source_label(&spec.source).map_err(|e| err("bad_source", e))?;
            out.insert(fi.name.clone(), InputValue { q, source });
        }
        Ok(out)
    }

    fn dispatch(&mut self, req: Request) -> Result<Response, Response> {
        Ok(match req {
            Request::Hello => Response::Hello {
                kernel_version: KERNEL_VERSION.into(),
                protocol: PROTOCOL.into(),
                harness_version: sse_core::harness::HARNESS_VERSION.into(),
                draw_algorithm: sse_core::draw::DRAW_ALGORITHM.into(),
                ops: ops(),
                tiers: tiers(),
            },
            Request::LoadStack { packs } => {
                let mut ms = Vec::new();
                for (i, p) in packs.iter().enumerate() {
                    ms.push(
                        serde_json::from_value::<sse_pack::PackManifest>(p.clone())
                            .map_err(|e| err("bad_pack", format!("pack {i}: {e}")))?,
                    );
                }
                match sse_pack::resolve(&ms, KERNEL_VERSION) {
                    Err(issues) => Response::StackRejected {
                        report: sse_pack::report(&issues),
                        issues: issues.iter().map(|i| i.to_string()).collect(),
                    },
                    Ok(stack) => {
                        let cx = Context {
                            atoms: &self.atoms,
                            kinds: &stack.kinds,
                            units: &stack.units,
                        };
                        let mut flows = BTreeMap::new();
                        let mut problems = Vec::new();
                        for (id, af) in &stack.flows {
                            match Flow::compile(af.doc.clone(), &cx) {
                                Ok(f) => {
                                    flows.insert(id.clone(), f);
                                }
                                Err(es) => problems.extend(
                                    es.into_iter()
                                        .map(|e| format!("{} (flow {id}): {e}", af.pack)),
                                ),
                            }
                        }
                        // Each flow's own golden cases (H5.5) run here too, so an
                        // edited flow whose cases no longer hold is refused with
                        // the failing case named, not loaded to drift silently.
                        if problems.is_empty() {
                            for (id, f) in &flows {
                                for m in f.run_tests(&cx) {
                                    problems.push(format!(
                                        "{} (flow {id}): test {m}",
                                        stack.flows[id].pack
                                    ));
                                }
                            }
                        }
                        if !problems.is_empty() {
                            let issues: Vec<sse_pack::StackIssue> = problems
                                .iter()
                                .map(|m| sse_pack::StackIssue {
                                    pack: "flow".into(),
                                    message: m.clone(),
                                })
                                .collect();
                            return Ok(Response::StackRejected {
                                report: sse_pack::report(&issues),
                                issues: problems,
                            });
                        }
                        let observe = match load_observe(&ms) {
                            Ok(o) => o,
                            Err(problems) => {
                                let issues: Vec<sse_pack::StackIssue> = problems
                                    .iter()
                                    .map(|m| sse_pack::StackIssue {
                                        pack: "observe".into(),
                                        message: m.clone(),
                                    })
                                    .collect();
                                return Ok(Response::StackRejected {
                                    report: sse_pack::report(&issues),
                                    issues: problems,
                                });
                            }
                        };
                        let adopted = match self.universe.as_mut() {
                            Some(u) => u.adopt(&stack)?,
                            None => None,
                        };
                        let r = Response::StackLoaded {
                            stack_hash: stack.hash.hex(),
                            flows: flows.keys().cloned().collect(),
                            publishers: stack.publishers.len() as u32,
                            fictional_kinds: stack.fictional_kinds.iter().cloned().collect(),
                            adopted,
                        };
                        self.stack = Some((stack, flows));
                        self.observe = observe;
                        r
                    }
                }
            }
            Request::OpenUniverse { id, seed } => {
                let root =
                    Address::universe(&id).map_err(|e| err("bad_address", format!("{e}")))?;
                let seed = Seed::from_text(&seed);
                let chronicle = Chronicle::new();
                let packs = self
                    .stack
                    .as_ref()
                    .map(|(s, _)| pack_refs(s))
                    .unwrap_or_default();
                let u = Universe {
                    root: root.clone(),
                    seed,
                    chronicle,
                    branches: BTreeMap::new(),
                    packs,
                };
                let manifest_id = u.manifest().id().hex();
                self.universe = Some(u);
                Response::UniverseOpened {
                    root: root.to_string(),
                    manifest_id,
                }
            }
            Request::SetAuthored { edit } => {
                let u = self.universe()?;
                let e = Self::edit_event(u, &edit, u.chronicle.next_stamp())?;
                let affected = e.body.subject.iter().map(|a| a.to_string()).collect();
                let id = u
                    .chronicle
                    .append(e)
                    .map_err(|e| err("chronicle", format!("{e}")))?;
                Response::Authored {
                    event_id: id.hex(),
                    affected,
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                }
            }
            Request::History { address } => {
                let a: Address = address
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let u = self.universe()?;
                let events = u
                    .chronicle
                    .events()
                    .iter()
                    .filter(|e| e.body.subject.iter().any(|s| a.contains(s)))
                    .map(|e| EventSummary {
                        id: e.id.hex(),
                        kind: e.body.kind.clone(),
                        occurred_at_s: e.body.occurred_at.s,
                        evidence: serde_json::to_value(e.body.evidence)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string(),
                        subject: e.body.subject.iter().map(|s| s.to_string()).collect(),
                    })
                    .collect();
                Response::History { events }
            }
            Request::StateHash { scope, at_s } => {
                let u = self.universe()?;
                let st = u.chronicle.state_at(UniverseTime::seconds(at_s));
                let hash = match scope {
                    None => st.hash(),
                    Some(s) => {
                        st.hash_scope(&s.parse().map_err(|e| err("bad_address", format!("{e}")))?)
                    }
                };
                Response::StateHash { hash: hash.hex() }
            }
            Request::Derive { flow, inputs } => {
                let (stack, flows) = self
                    .stack
                    .as_ref()
                    .ok_or_else(|| err("no_stack", "load a pack stack first"))?;
                let f = flows
                    .get(&flow)
                    .ok_or_else(|| err("unknown_flow", format!("no flow {flow:?} in the stack")))?;
                let ins = self.flow_inputs(f, &inputs, &stack.units)?;
                let cx = Context {
                    atoms: &self.atoms,
                    kinds: &stack.kinds,
                    units: &stack.units,
                };
                let ev = f.eval(&cx, &ins).map_err(|e| err("eval", e.to_string()))?;
                let pack = &stack.flows[&flow].pack;
                let values = f
                    .published_kinds()
                    .map(|k| {
                        let q = ev.published(k).unwrap();
                        let v = DerivedValue {
                            value: q.value,
                            dim: q.dim.to_string(),
                            validity: serde_json::to_value(ev.validity_of(k).unwrap())
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string(),
                            fictional: stack.is_fictional(k),
                            publisher: format!("{pack}/{}@{}", f.doc.id, f.doc.version),
                        };
                        (k.to_string(), v)
                    })
                    .collect();
                Response::Derived {
                    values,
                    work_units: ev.work_units as u32,
                }
            }
            Request::Explain { flow, inputs, kind } => {
                let (stack, flows) = self
                    .stack
                    .as_ref()
                    .ok_or_else(|| err("no_stack", "load a pack stack first"))?;
                let f = flows
                    .get(&flow)
                    .ok_or_else(|| err("unknown_flow", format!("no flow {flow:?} in the stack")))?;
                let ins = self.flow_inputs(f, &inputs, &stack.units)?;
                let cx = Context {
                    atoms: &self.atoms,
                    kinds: &stack.kinds,
                    units: &stack.units,
                };
                let ev = f.eval(&cx, &ins).map_err(|e| err("eval", e.to_string()))?;
                let x = ev
                    .explain(&kind)
                    .or_else(|| ev.explain_port(&kind))
                    .ok_or_else(|| {
                        err(
                            "unknown_kind",
                            format!("{kind:?} is not published by {flow} (or a node.port)"),
                        )
                    })?;
                Response::Explanation {
                    explanation: serde_json::to_value(&x).unwrap(),
                    fictional: stack.is_fictional(&kind),
                }
            }
            Request::Draw {
                address,
                stream,
                counter,
            } => {
                let u = self.universe()?;
                let a: Address = address
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let s = Stream::new(&stream).map_err(|e| err("bad_stream", e.to_string()))?;
                let d = Draws::new(u.seed, a, s);
                Response::Drawn {
                    hex: format!("{:016x}", d.u64(counter as u64)),
                    unit: d.unit(counter as u64),
                }
            }
            Request::Branch { label } => {
                let u = self.universe()?;
                let id = u.chronicle.branch(&label);
                u.branches.insert(id.hex(), id);
                Response::BranchOpened { branch: id.hex() }
            }
            Request::BranchSet { branch, edit } => {
                let u = self.universe()?;
                let b = *u
                    .branches
                    .get(&branch)
                    .ok_or_else(|| err("unknown_branch", branch.clone()))?;
                let info = u.chronicle.branch_info(&b).unwrap();
                let stamp = info
                    .events
                    .last()
                    .map(|e| e.body.recognised_at.0 + 1)
                    .unwrap_or(u.chronicle.next_stamp().0);
                let e = Self::edit_event(u, &edit, sse_chronicle::SessionStamp(stamp))?;
                let affected = e.body.subject.iter().map(|a| a.to_string()).collect();
                let id = u
                    .chronicle
                    .branch_append(&b, e)
                    .map_err(|e| err("chronicle", format!("{e}")))?;
                let st = u
                    .chronicle
                    .branch_state_at(&b, UniverseTime::seconds(i64::MAX))
                    .unwrap();
                Response::Authored {
                    event_id: id.hex(),
                    affected,
                    state_hash: st.hash().hex(),
                }
            }
            Request::BranchStateHash { branch, at_s } => {
                let u = self.universe()?;
                let b = *u
                    .branches
                    .get(&branch)
                    .ok_or_else(|| err("unknown_branch", branch.clone()))?;
                Response::StateHash {
                    hash: u
                        .chronicle
                        .branch_state_at(&b, UniverseTime::seconds(at_s))
                        .unwrap()
                        .hash()
                        .hex(),
                }
            }
            Request::Commit { branch, at_s } => {
                let u = self.universe()?;
                let b = u
                    .branches
                    .remove(&branch)
                    .ok_or_else(|| err("unknown_branch", branch.clone()))?;
                let root = u.root.clone();
                let id = u
                    .chronicle
                    .commit(
                        &b,
                        UniverseTime::seconds(at_s),
                        vec![root],
                        Publisher::new("gm", "commit", "1"),
                    )
                    .map_err(|e| err("chronicle", format!("{e}")))?;
                Response::Committed {
                    event_id: id.hex(),
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                }
            }
            Request::Discard { branch } => {
                let u = self.universe()?;
                let b = u
                    .branches
                    .remove(&branch)
                    .ok_or_else(|| err("unknown_branch", branch.clone()))?;
                u.chronicle
                    .discard(&b)
                    .map_err(|e| err("chronicle", format!("{e}")))?;
                Response::Discarded { branch }
            }
            Request::StartJob { job } => {
                let JobSpec::Benchmark { units } = job;
                if units == 0 {
                    return Err(err("bad_job", "a job needs at least one unit"));
                }
                let id = self.next_job;
                self.next_job += 1;
                self.jobs.insert(
                    id,
                    Job {
                        total: units,
                        done: 0,
                        hasher: Hasher::new("sse.kernel.benchmark.v1"),
                    },
                );
                let cost = if units <= 1_000 {
                    Cost::Trivial
                } else if units <= 1_000_000 {
                    Cost::Light
                } else if units <= 100_000_000 {
                    Cost::Medium
                } else {
                    Cost::Heavy
                };
                Response::JobStarted {
                    job: id,
                    total_units: units,
                    cost,
                }
            }
            Request::StepJob { job, budget } => {
                let j = self.jobs.get_mut(&job).ok_or_else(|| {
                    err(
                        "unknown_job",
                        format!("no job {job} (finished or cancelled)"),
                    )
                })?;
                let end = j.total.min(j.done.saturating_add(budget.max(1)));
                for i in j.done..end {
                    j.hasher.update(&bench_unit(i).to_bits().to_be_bytes());
                }
                j.done = end;
                if j.done < j.total {
                    Response::JobProgress {
                        job,
                        done_units: j.done,
                        total_units: j.total,
                    }
                } else {
                    let j = self.jobs.remove(&job).unwrap();
                    Response::JobDone {
                        job,
                        done_units: j.done,
                        result: serde_json::json!({ "digest": j.hasher.finish().hex() }),
                    }
                }
            }
            Request::CancelJob { job } => {
                self.jobs
                    .remove(&job)
                    .ok_or_else(|| err("unknown_job", format!("no job {job}")))?;
                Response::JobCancelled { job }
            }
            Request::Scene {
                audience,
                scope,
                data_view,
                at_s,
                observe,
            } => {
                let scope: Address = scope
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let draft = match observe {
                    None => None,
                    Some(v) => Some(
                        sse_observe::ObserveSetup::from_stack(&[("draft".into(), v)])
                            .map_err(|e| err("bad_observe", e))?
                            .ok_or_else(|| err("bad_observe", "the draft setup is empty"))?,
                    ),
                };
                let setup = draft
                    .as_ref()
                    .or(self.observe.setup.as_ref())
                    .ok_or_else(|| {
                        err(
                            "no_data_views",
                            "the pack stack defines no data views (provides.other.observe)",
                        )
                    })?;
                let u = self
                    .universe
                    .as_ref()
                    .ok_or_else(|| err("no_universe", "open a universe first"))?;
                let mut state = u.chronicle.state_at(UniverseTime::seconds(at_s));
                // Authored tags out of the facts; rule tags from the stack.
                let mut tags: BTreeMap<Address, std::collections::BTreeSet<String>> =
                    BTreeMap::new();
                for (a, f) in state.facts.iter_mut() {
                    let keys: Vec<String> = f
                        .keys()
                        .filter(|k| k.starts_with(TAG_FIELD))
                        .cloned()
                        .collect();
                    for k in keys {
                        if f.remove(&k) == Some(Value::Bool(true)) {
                            tags.entry(a.clone())
                                .or_default()
                                .insert(k[TAG_FIELD.len()..].to_string());
                        }
                    }
                }
                for (a, f) in &state.facts {
                    let subject = sse_sift::Subject {
                        address: Some(a.clone()),
                        facts: f.clone(),
                        parent: a
                            .parent()
                            .and_then(|p| state.facts.get(&p).cloned())
                            .unwrap_or_default(),
                        now_s: at_s,
                        ..Default::default()
                    };
                    for t in self
                        .observe
                        .rules
                        .apply(&subject, &BTreeMap::new())
                        .into_keys()
                    {
                        tags.entry(a.clone()).or_default().insert(t);
                    }
                }
                state.facts.retain(|_, f| !f.is_empty());
                let fictional = self
                    .stack
                    .as_ref()
                    .map(|(s, _)| s.fictional_kinds.iter().cloned().collect())
                    .unwrap_or_default();
                let views = setup.views_by_id();
                let inp = sse_observe::SceneInput {
                    state: &state,
                    chronicle: Some(&u.chronicle),
                    tags: &tags,
                    fictional: &fictional,
                    disclosure: &setup.disclosure,
                    views: &views,
                    campaign: &setup.campaign,
                };
                let scene =
                    sse_observe::build_scene_through(&inp, &audience, &scope, data_view.as_deref())
                        .map_err(|e| err("bad_data_view", e))?;
                Response::Scene {
                    scene_hash: scene.hash().hex(),
                    scene: serde_json::to_value(&scene).expect("scenes are plain data"),
                }
            }
            Request::Atoms { ids } => {
                let port = |p: &sse_flow::Port| {
                    serde_json::json!({
                        "name": p.name, "kind": p.kind, "dim": p.dim.to_string(), "valid": p.valid,
                    })
                };
                let mut atoms = Vec::new();
                let mut missing = Vec::new();
                let want: Option<std::collections::BTreeSet<&str>> =
                    ids.as_ref().map(|v| v.iter().map(String::as_str).collect());
                let mut decls: Vec<&sse_flow::AtomDecl> = self.atoms.decls().collect();
                decls.sort_by(|a, b| a.id.cmp(&b.id));
                for d in decls {
                    if want.as_ref().is_some_and(|w| !w.contains(d.id.as_str())) {
                        continue;
                    }
                    atoms.push(serde_json::json!({
                        "id": d.id,
                        "version": d.version,
                        "inputs": d.inputs.iter().map(port).collect::<Vec<_>>(),
                        "outputs": d.outputs.iter().map(port).collect::<Vec<_>>(),
                        "params": d.params.iter().map(|p| serde_json::json!({
                            "name": p.name, "dim": p.dim.to_string(), "default": p.default,
                            "range": p.range, "description": p.description,
                        })).collect::<Vec<_>>(),
                        "cost_class": d.cost_class,
                        "references": d.references,
                        "explain": d.explain,
                    }));
                }
                if let Some(w) = &want {
                    for id in w {
                        if !atoms.iter().any(|a| a["id"] == *id) {
                            missing.push(*id);
                        }
                    }
                }
                if !missing.is_empty() {
                    return Err(err(
                        "unknown_atom",
                        format!("no atom {}", missing.join(", ")),
                    ));
                }
                Response::Atoms { atoms }
            }
            Request::ScanToggle {
                address,
                audience,
                at_s,
            } => {
                let a: Address = address
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let campaign = self
                    .observe
                    .setup
                    .as_ref()
                    .ok_or_else(|| {
                        err(
                            "no_data_views",
                            "the pack stack defines no data views (provides.other.observe)",
                        )
                    })?
                    .campaign
                    .clone();
                let u = self.universe()?;
                if !u.root.contains(&a) {
                    return Err(err(
                        "bad_address",
                        format!("{a} is not in universe {}", u.root),
                    ));
                }
                let (id, view) = sse_observe::scan_toggle(
                    &mut u.chronicle,
                    &a,
                    &audience,
                    &campaign,
                    UniverseTime::seconds(at_s),
                    &Publisher::new("gm", "scan", "1"),
                )
                .map_err(|e| err("chronicle", format!("{e}")))?;
                Response::ScanToggled {
                    event_id: id.hex(),
                    view,
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                }
            }
            Request::Grant {
                address,
                audience,
                view,
                default,
                at_s,
            } => {
                let a: Address = address
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let setup = self.observe.setup.as_ref().ok_or_else(|| {
                    err(
                        "no_data_views",
                        "the pack stack defines no data views (provides.other.observe)",
                    )
                })?;
                if let Some(v) = &view {
                    if !setup.views.iter().any(|d| &d.id == v) {
                        return Err(err("unknown_view", format!("no data view {v:?}")));
                    }
                }
                let campaign = setup.campaign.clone();
                let default = default.unwrap_or(false);
                let u = self.universe()?;
                if !u.root.contains(&a) {
                    return Err(err(
                        "bad_address",
                        format!("{a} is not in universe {}", u.root),
                    ));
                }
                if default && a != u.root {
                    return Err(err(
                        "bad_address",
                        format!(
                            "a starting view is granted on the universe root {}, not {a}",
                            u.root
                        ),
                    ));
                }
                let at = UniverseTime::seconds(at_s);
                let id = sse_observe::grant(
                    &mut u.chronicle,
                    &a,
                    &audience,
                    view.as_deref(),
                    default,
                    at,
                    &Publisher::new("gm", "grant", "1"),
                )
                .map_err(|e| err("chronicle", format!("{e}")))?;
                let state = u.chronicle.state_at(UniverseTime::seconds(i64::MAX));
                Response::Granted {
                    event_id: id.hex(),
                    view: sse_observe::effective_view(&state, &a, &audience, &campaign),
                    state_hash: state.hash().hex(),
                }
            }
            Request::Manifest => {
                let m = self.universe()?.manifest();
                Response::Manifest {
                    manifest: serde_json::to_value(&m).expect("manifest is plain data"),
                    manifest_id: m.id().hex(),
                }
            }
        })
    }
}

#[cfg(test)]
mod tests;
