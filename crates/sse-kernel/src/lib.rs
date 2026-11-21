//! SSE4 kernel: the one message API in front of the substrate and engines
//! (H9.1). The same `Kernel::handle_json` runs natively (CLI, tests) and in
//! the browser worker (sse-wasm), so a contract test can prove both agree.
#![forbid(unsafe_code)]
// Errors are full protocol responses on a cold path; boxing them buys nothing.
#![allow(clippy::result_large_err)]

mod adopt;
mod clock;
pub mod contract;
mod cosmos;
mod descent;
mod dressing;
#[cfg(test)]
mod dressing_tests;
#[cfg(test)]
mod matter_tests;
mod nav;
mod orbits;
pub mod protocol;
mod survey;
mod systems;
mod tags;
#[cfg(test)]
mod tags_tests;
mod weather;

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
    /// The seed as the GM typed it (exports reopen from it).
    seed_text: String,
    chronicle: Chronicle,
    branches: BTreeMap<String, ContentHash>,
    /// The pack stack the universe runs on now (bottom to top).
    packs: Vec<sse_chronicle::PackRef>,
    /// Open look-aheads (T2) by branch id; each is also in `branches`.
    looks: BTreeMap<String, sse_time::branch::LookAhead>,
    /// Formation runs of adopted systems by address (a cache: the
    /// `adopt.formation` event holds the request to run again).
    runs: BTreeMap<String, sse_formation::pipeline::SystemRun>,
    /// The campaign record (J1): notes, kept values, actors. Held on the GM
    /// machine only and never broadcast (record design 8A); its own chronicle,
    /// so record activity never moves the universe state hash.
    record: Chronicle,
    /// Recent Orbit-level atlases (S16) by `sse_surface::detail::orbit_key`:
    /// reuse before computing. A cache only; dropped with the universe.
    orbits: adopt::OrbitCache,
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
    let sift = sse_sift::load(ms)?;
    let rules = sift.rules;
    let sifters = sift.sifters;
    let mut kept = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("record") {
            let j = serde_json::to_value(v).map_err(|e| vec![format!("{}: record: {e}", p.id)])?;
            kept.push((p.id.clone(), j));
        }
    }
    let record = sse_record::Registry::from_packs(&kept)?;
    let mut found = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("triggers") {
            let j =
                serde_json::to_value(v).map_err(|e| vec![format!("{}: triggers: {e}", p.id)])?;
            found.push((p.id.clone(), p.version.to_string(), j));
        }
    }
    let triggers = sse_trigger::Rules::from_packs(&found)?;
    let mut found = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("templates") {
            let j =
                serde_json::to_value(v).map_err(|e| vec![format!("{}: templates: {e}", p.id)])?;
            found.push((p.id.clone(), p.version.to_string(), j));
        }
    }
    let templates = sse_trigger::templates::Templates::from_packs(&found)?;
    let mut issues = templates.check(&record);
    issues.extend(triggers.check_templates(&templates));
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut found = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("intents") {
            let j = serde_json::to_value(v).map_err(|e| vec![format!("{}: intents: {e}", p.id)])?;
            found.push((p.id.clone(), j));
        }
    }
    let intents = sse_trigger::intents::IntentRegistry::from_packs(&found)?;
    let mut problems = vec![];
    for (rule, kind) in triggers.intent_kinds() {
        match intents.get(&kind) {
            Some(k) if k.applied_by == sse_trigger::intents::AppliedBy::Record => {}
            Some(_) => problems.push(format!(
                "rule {rule}: intent {kind} is a built-in kind, applied by the kernel or session, not taken up by rules"
            )),
            None => problems.push(format!(
                "rule {rule}: intent {kind} is not declared (provides.other.intents)"
            )),
        }
    }
    let mut found = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("programs") {
            let j =
                serde_json::to_value(v).map_err(|e| vec![format!("{}: programs: {e}", p.id)])?;
            found.push((p.id.clone(), j));
        }
    }
    let programs = sse_trigger::programs::Programs::from_packs(&found)?;
    for (prog, kind) in programs.sends() {
        if intents.get(&kind).is_none() {
            problems.push(format!(
                "program {prog}: sends intent {kind}, which is not declared"
            ));
        }
    }
    let mut found = Vec::new();
    for p in ms {
        if let Some(v) = p.provides.other.get("dressing") {
            let j =
                serde_json::to_value(v).map_err(|e| vec![format!("{}: dressing: {e}", p.id)])?;
            found.push((p.id.clone(), j));
        }
    }
    let dressing = match dressing::Dressing::from_packs(&found, &templates) {
        Ok(d) => d,
        Err(e) => {
            problems.extend(e);
            dressing::Dressing::default()
        }
    };
    // MP1: the master data sheet, merged bottom to top and checked.
    let reference = match sse_reference::Reference::from_packs(ms) {
        Ok(r) => r,
        Err(e) => {
            problems.extend(e);
            sse_reference::Reference::default()
        }
    };
    if !problems.is_empty() {
        return Err(problems);
    }
    let views: Vec<String> = setup
        .as_ref()
        .map(|s| s.views.iter().map(|v| v.id.clone()).collect())
        .unwrap_or_default();
    let issues = triggers.check(&record, &views);
    if !issues.is_empty() {
        return Err(issues);
    }
    let calendar = orbits::load_calendar(ms)?;
    // TG1: every tag named anywhere in the stack is declared.
    let tags = tags::declarations(ms)?;
    let mut problems = tags::check(&tags::references(ms), &tags);
    problems.extend(tags::reserved_kept(ms));
    if !problems.is_empty() {
        return Err(problems);
    }
    // A declaration's tier is the tag's default disclosure tier.
    let mut setup = setup;
    if let Some(s) = setup.as_mut() {
        for (_, t) in tags.tags.values() {
            if let Some(tier) = &t.tier {
                s.disclosure
                    .tags
                    .entry(t.id.clone())
                    .or_insert_with(|| tier.clone());
            }
        }
    }
    Ok(Observe {
        tags,
        setup,
        rules,
        sifters,
        record,
        triggers,
        calendar,
        intents,
        templates,
        programs,
        dressing,
        reference,
    })
}

/// The stack's tag declarations plus the campaign's own (kept in the
/// record at the universe root, TG1). A pack loaded later that declares a
/// tag the campaign already had wins; the campaign's copy is ignored.
fn campaign_tags(u: &Universe, stack: &tags::TagSet) -> Result<tags::TagSet, Response> {
    let mut set = stack.clone();
    let rec = u.record.state_at(UniverseTime::seconds(i64::MAX));
    let mut block = tags::TagBlock::default();
    for (f, v) in rec.facts.get(&u.root).into_iter().flatten() {
        if let (Some(id), Value::Text(t)) = (f.strip_prefix(tags::CAMPAIGN_FIELD), v) {
            if !set.declared(id) {
                block.tags.push(
                    serde_json::from_str(t).map_err(|e| err("bad_record", format!("{f}: {e}")))?,
                );
            }
        }
    }
    let mut errs = vec![];
    set.add("campaign", block, &mut errs);
    Ok(set)
}

/// Add a freely typed tag to the campaign's own declarations.
fn declare_campaign_tag(u: &mut Universe, tag: &str) -> Result<(), Response> {
    let d = tags::TagDecl {
        id: tag.to_string(),
        label: tag.to_string(),
        about: None,
        group: None,
        source: tags::Source::Authored,
        tier: None,
        colour: None,
    };
    let mut f = BTreeMap::new();
    f.insert(
        format!("{}{tag}", tags::CAMPAIGN_FIELD),
        Value::Text(serde_json::to_string(&d).expect("plain data")),
    );
    let root = u.root.clone();
    let e = clock::timeless_set(
        &u.record,
        &root,
        f,
        Evidence::Authored,
        vec![],
        tags::CAMPAIGN_KIND,
        Publisher::new("gm", "tags", "1"),
    )
    .map_err(|e| err("bad_event", e))?;
    u.record
        .append(e)
        .map_err(|e| err("chronicle", format!("{e}")))?;
    Ok(())
}

/// The disclosure a scene uses: the stack's, plus each kept kind at its own
/// tier and `misc.*` at the GM-only rung, where the stack does not say.
fn with_record(d: &sse_observe::Disclosure, r: &sse_record::Registry) -> sse_observe::Disclosure {
    let mut d = d.clone();
    for (f, t) in r.disclosure_fields(d.ladder.secret()) {
        d.fields.entry(f).or_insert(t);
    }
    d
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
    /// Tag declarations from the stack (`provides.other.tags`, TG1).
    tags: tags::TagSet,
    setup: Option<sse_observe::ObserveSetup>,
    rules: sse_sift::RuleSet,
    /// Sifters (`provides.other.sifters`): the story finder's patterns (U8).
    sifters: Vec<sse_sift::Sifter>,
    /// Kept kinds (`provides.other.record`, J1).
    record: sse_record::Registry,
    /// Triggers and effects (`provides.other.triggers`, J4).
    triggers: sse_trigger::Rules,
    /// The top pack's calendar (`provides.other.calendar`).
    calendar: Option<orbits::Calendar>,
    /// Intent kinds: built in and from packs (`provides.other.intents`, J5).
    intents: sse_trigger::intents::IntentRegistry,
    /// Templates (`provides.other.templates`, J7).
    templates: sse_trigger::templates::Templates,
    /// Programs (`provides.other.programs`, J6).
    programs: sse_trigger::programs::Programs,
    /// Dressing rules (`provides.other.dressing`, TG12).
    dressing: dressing::Dressing,
    /// The master data sheet (`provides.reference`, MP1).
    reference: sse_reference::Reference,
}

pub struct Kernel {
    atoms: sse_flow::AtomRegistry,
    stack: Option<(ResolvedStack, BTreeMap<String, Flow>)>,
    observe: Observe,
    /// Game-system code sheets from the stack (sse-codes).
    codes: sse_codes::Codes,
    universe: Option<Universe>,
    jobs: BTreeMap<u32, Job>,
    next_job: u32,
    /// `provides.other.time` from the stack, if any pack provides it.
    time: Option<sse_time::epoch::TimeSettings>,
    /// Recent `survey_surface` results.
    surveys: survey::Cache,
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

fn arr(v: serde_json::Value) -> Vec<serde_json::Value> {
    match v {
        serde_json::Value::Array(a) => a,
        _ => vec![],
    }
}

fn err(code: &str, message: impl Into<String>) -> Response {
    Response::Error {
        code: code.into(),
        message: message.into(),
    }
}

fn record_err(e: sse_record::RecordError) -> Response {
    use sse_record::RecordError as E;
    let code = match &e {
        E::BadNote(_) | E::UnknownNote(_) => "bad_note",
        E::UnknownKind(_) => "unknown_kind",
        E::WriterRefused { .. } => "writer_refused",
        E::BadValue { .. } => "bad_value",
        E::BadActor(_) => "bad_actor",
        E::Registry(_) => "bad_record",
        E::Chronicle(_) => "chronicle",
    };
    err(code, e.to_string())
}

fn in_universe(root: &Address, a: &str) -> Result<Address, Response> {
    let a: Address = a.parse().map_err(|e| err("bad_address", format!("{e}")))?;
    if !root.contains(&a) {
        return Err(err("bad_address", format!("{a} is not in universe {root}")));
    }
    Ok(a)
}

/// Rebuild a chronicle from an export's `[{id, kind, cbor}]`, checking ids.
fn replay_export(evs: &[serde_json::Value], what: &str) -> Result<Chronicle, Response> {
    let mut chronicle = Chronicle::new();
    for (k, e) in evs.iter().enumerate() {
        let bytes = e["cbor"]
            .as_str()
            .and_then(adopt::unhex)
            .ok_or_else(|| err("bad_export", format!("{what} {k}: missing cbor")))?;
        let body: EventBody = sse_core::cbor::from_canonical(&bytes)
            .map_err(|x| err("bad_export", format!("{what} {k}: {x:?}")))?;
        let ev = Event::new(body).map_err(|x| err("bad_export", format!("{what} {k}: {x}")))?;
        if e["id"].as_str() != Some(ev.id.hex().as_str()) {
            return Err(err(
                "bad_export",
                format!("{what} {k}: id does not match its content"),
            ));
        }
        chronicle
            .append(ev)
            .map_err(|x| err("bad_export", format!("{what} {k}: {x}")))?;
    }
    Ok(chronicle)
}

/// The record's hash: its chronicle head (covers notes, which touch no state).
fn record_hash(u: &Universe) -> String {
    u.record.head().hex()
}

fn latest_hash(c: &Chronicle) -> String {
    c.state_at(UniverseTime::seconds(i64::MAX)).hash().hex()
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
/// so `load_stack` can take `packs/base-physics/pack.json`, plus the logic
/// blocks (J2). Ids never clash: `toy.*`, `sse.*`, `logic.*`, `world.*`,
/// `chance.*`.
fn kernel_atoms() -> sse_flow::AtomRegistry {
    let mut r = sse_flow::toy::registry();
    for a in sse_atoms::atoms() {
        r.insert(a);
    }
    // J2: the logic set (`logic.*`, `world.*`, `chance.*`).
    sse_logic::register(&mut r);
    r
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {
            atoms: kernel_atoms(),
            stack: None,
            observe: Observe::default(),
            codes: sse_codes::Codes::default(),
            universe: None,
            surveys: vec![],
            jobs: BTreeMap::new(),
            next_job: 1,
            time: None,
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

    /// Carry out an applied intent: a built-in becomes its kernel request
    /// (the response is returned), a session kind is left to the session, and
    /// a pack kind goes to the record as `intent.<kind>`.
    fn apply_intent(
        &mut self,
        seat: &str,
        intent: &serde_json::Value,
        at_s: i64,
        causes: Vec<ContentHash>,
    ) -> Result<(String, Option<serde_json::Value>), Response> {
        use sse_trigger::intents::{self as si, AppliedBy};
        let reg = self.observe.intents.clone();
        let k = reg.check(intent).map_err(|e| err("bad_intent", e))?.clone();
        Ok(match k.applied_by {
            AppliedBy::Session => ("session".into(), None),
            AppliedBy::Kernel => {
                let req = si::to_request(intent, seat, at_s).expect("a kernel kind");
                let req: Request =
                    serde_json::from_value(req).map_err(|e| err("bad_intent", e.to_string()))?;
                let r = self.dispatch(req)?;
                (
                    "kernel".into(),
                    Some(serde_json::to_value(&r).expect("plain")),
                )
            }
            AppliedBy::Record => {
                let u = self.universe()?;
                let root = u.root.clone();
                let id = si::record_intent(&reg, &mut u.record, &root, seat, intent, at_s, causes)
                    .map_err(|e| err("chronicle", e))?;
                (
                    "record".into(),
                    Some(serde_json::json!({"event_id": id.hex(), "record_hash": record_hash(u)})),
                )
            }
        })
    }

    /// One tick of every running program (J6), in address order.
    fn programs_step(&mut self, at_s: i64) -> Result<Response, Response> {
        use sse_trigger::programs as sp;
        let ps = self.observe.programs.clone();
        let reg = self.observe.intents.clone();
        let (running, seats) = {
            let u = self.universe()?;
            (
                sp::running(&u.record, at_s),
                sse_trigger::intents::seats(&reg, &u.record),
            )
        };
        let mut report = Vec::new();
        for r in running.values() {
            let a: Address = r.address.parse().expect("record addresses parse");
            let mut line =
                serde_json::json!({"address": r.address, "program": r.program, "from": r.state});
            let Some(p) = ps.get(&r.program) else {
                line["skipped"] = "the program is no longer in the stack".into();
                report.push(line);
                continue;
            };
            let d = &p.doc;
            {
                let u = self.universe()?;
                if let Err(e) = sse_trigger::templates::may_run(
                    &u.chronicle,
                    &a,
                    &r.program,
                    UniverseTime::seconds(at_s),
                ) {
                    line["skipped"] = e.into();
                    report.push(line);
                    continue;
                }
            }
            let Some(seat) = seats.iter().find(|s| s.id == r.seat).cloned() else {
                line["skipped"] = format!("no seat {}", r.seat).into();
                report.push(line);
                continue;
            };
            // Sense: what the seat's audience has been shown, never the GM's truth.
            let root = self.universe()?.root.to_string();
            let scene = match self.dispatch(Request::Scene {
                audience: seat.audience.clone(),
                scope: root,
                data_view: None,
                at_s,
                observe: None,
            })? {
                Response::Scene { scene, .. } => scene,
                other => return Err(other),
            };
            let scene: sse_observe::scene::Scene =
                serde_json::from_value(scene).expect("scenes round-trip");
            let sensed: Vec<(Address, sse_sift::Subject)> = scene
                .subjects
                .iter()
                .filter(|x| x.address != a)
                .map(|x| {
                    let mut f = BTreeMap::new();
                    for fact in &x.facts {
                        if let (Some(v), None) = (&fact.value, &fact.placeholder) {
                            f.insert(fact.field.clone(), v.clone());
                        }
                    }
                    let subj = sse_sift::Subject {
                        address: Some(x.address.clone()),
                        facts: f,
                        tags: x.tags.iter().cloned().collect(),
                        now_s: at_s,
                        ..Default::default()
                    };
                    (x.address.clone(), subj)
                })
                .collect();
            line["sensed"] = sensed.len().into();
            // Think: follow transitions (the program knows itself), at most max_steps.
            let mut state = r.state.clone();
            let mut moves = 0u32;
            let picked = loop {
                let st = d.state(&state).expect("checked at load");
                let picked = st
                    .pick
                    .as_ref()
                    .and_then(|pk| sp::pick(pk, d, &sensed))
                    .cloned();
                if moves >= d.budget.max_steps {
                    break picked;
                }
                let me = {
                    let u = self.universe()?;
                    sse_trigger::subjects(
                        &u.chronicle,
                        &u.record,
                        &u.root,
                        UniverseTime::seconds(at_s),
                    )
                    .remove(&a)
                    .unwrap_or_default()
                };
                let mut me = sp::with_params(me, d);
                me.now_s = at_s;
                me.facts
                    .insert("pick.found".into(), Value::Bool(picked.is_some()));
                match st
                    .transitions
                    .iter()
                    .find(|t| t.when.eval(&me, false).holds)
                {
                    Some(t) => {
                        let to = t.to.clone();
                        let u = self.universe()?;
                        sp::write(
                            &mut u.record,
                            sp::kinds::STATE,
                            &a,
                            at_s,
                            &[("program", &r.program), ("state", &to), ("from", &state)],
                            &r.program,
                        )
                        .map_err(|e| err("chronicle", e))?;
                        state = to;
                        moves += 1;
                    }
                    None => break picked,
                }
            };
            line["to"] = state.clone().into();
            line["moves"] = moves.into();
            if let Some(pk) = &picked {
                line["picked"] = pk.to_string().into();
            }
            // Act: the state's intent, through the seat like any player's.
            let st = d.state(&state).expect("checked at load");
            if let Some(act) = &st.act {
                if st.pick.is_some() && picked.is_none() {
                    line["waiting"] = "nothing sensed fits the pick".into();
                } else {
                    let intent = sp::intent(act, d, &a, picked.as_ref());
                    line["intent"] = intent.clone();
                    let sent = self.dispatch(Request::SendIntent {
                        seat: seat.id.clone(),
                        intent,
                        at_s,
                        check_only: None,
                    })?;
                    if let Response::IntentSent {
                        decision,
                        applied_by,
                        proposal,
                        ..
                    } = sent
                    {
                        line["decision"] = decision;
                        if let Some(b) = applied_by {
                            line["applied_by"] = b.into();
                        }
                        if let Some(p) = proposal {
                            line["proposal"] = p.into();
                        }
                    }
                }
            }
            report.push(line);
        }
        let rules = match self.dispatch(Request::RulesStep { at_s })? {
            Response::RulesStepped { report, .. } => report,
            other => return Err(other),
        };
        let u = self.universe()?;
        Ok(Response::ProgramsStepped {
            programs: report,
            rules,
            record_hash: record_hash(u),
        })
    }

    /// A system's spec as resolved now (with the one clock's shifts).
    fn system_spec(&mut self, a: &Address) -> Result<sse_time::epoch::SystemSpec, Response> {
        let u = self.universe()?;
        if !u.root.contains(a) {
            return Err(err(
                "bad_address",
                format!("{a} is not in universe {}", u.root),
            ));
        }
        let key = a.to_string();
        if !u.runs.contains_key(&key) {
            if let Some(req) = adopt::formation_request(&u.chronicle, a) {
                let (_, run, _) = systems::form(req.into())?;
                u.runs.insert(key.clone(), run);
            }
        }
        let s = sse_surface::scenario::settings();
        let w = adopt::worlds_at(&u.chronicle, a, None, u.runs.get(&key), &s)?;
        w.spec.ok_or_else(|| {
            err(
                "no_star",
                format!("{a} has no star with a mass and an age to set"),
            )
        })
    }

    /// "Make this system this old, today" (top-level design 12.2, 17.1).
    fn set_system_age(
        &mut self,
        a: &Address,
        age_myr: f64,
        composition: Option<&str>,
        causes: Vec<ContentHash>,
        fallback: Option<sse_time::epoch::SystemSpec>,
    ) -> Result<Response, Response> {
        if !age_myr.is_finite() || age_myr < 0.0 {
            return Err(err(
                "bad_age",
                format!("age {age_myr} Myr must be 0 or more"),
            ));
        }
        let keep = match composition.unwrap_or("follow") {
            "follow" => false,
            "keep" => true,
            other => {
                return Err(err(
                    "bad_composition",
                    format!("composition {other:?}: follow or keep"),
                ))
            }
        };
        // The kernel's own reading of the system; a look-ahead's spec when the
        // universe holds no facts for it (a system given whole by the client).
        let spec = match (self.system_spec(a), fallback) {
            (Ok(s), _) => s,
            (Err(_), Some(f)) => f,
            (Err(e), None) => return Err(e),
        };
        let before = spec.age_myr;
        let u = self.universe()?;
        let all = u.chronicle.state_at(UniverseTime::seconds(i64::MAX));
        let shift = clock::age_shift_myr(&all, a) + (age_myr - before);
        let feh_before = sse_core::math::log10(spec.z / adopt::DEFAULT_Z);
        let pinned = all.get(a, clock::ORIGIN_PINNED) == Some(&Value::Bool(true));
        let place = |k: &str| all.get(a, k).and_then(clock::as_f64);
        let origin = if pinned {
            None
        } else {
            Some(cosmos::reconstruct_origin(
                &u.seed,
                &u.root,
                cosmos::OriginAt {
                    age_gyr: age_myr / 1000.0,
                    elapsed_gyr: clock::elapsed_myr(&u.chronicle) / 1000.0,
                    r_now_kpc: place("system.galactic_r_kpc"),
                    phi_now_deg: place("system.galactic_phi_deg"),
                    keep_feh: keep.then_some(feh_before),
                },
            )?)
        };
        let impossible = origin.as_ref().and_then(|o| o.impossible.clone());
        let feh_after = match (&origin, keep, &impossible) {
            (Some(o), false, None) => o.feh,
            _ => feh_before,
        };
        let z_after = adopt::DEFAULT_Z * sse_core::math::pow(10.0, feh_after);
        let mut f = BTreeMap::new();
        f.insert(clock::AGE_SHIFT.to_string(), Value::Float(shift));
        match &impossible {
            Some(why) => {
                f.insert(clock::AGE_IMPOSSIBLE.to_string(), Value::Text(why.clone()));
            }
            None if all.get(a, clock::AGE_IMPOSSIBLE).is_some() => {
                f.insert(clock::AGE_IMPOSSIBLE.to_string(), Value::Null);
            }
            None => {}
        }
        let rederive = !keep && impossible.is_none() && origin.is_some() && feh_after != feh_before;
        if rederive {
            f.insert("star.metallicity_z".to_string(), Value::Float(z_after));
        }
        let e = clock::timeless_set(
            &u.chronicle,
            a,
            f,
            if impossible.is_some() {
                Evidence::AuthoredImpossible
            } else {
                Evidence::Authored
            },
            causes,
            "",
            Publisher::new("gm", "age-system", "1"),
        )
        .map_err(|e| err("bad_event", e))?;
        let id = u
            .chronicle
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        let mut origin_event = None;
        if let (Some(o), None) = (&origin, &impossible) {
            let mut g = BTreeMap::new();
            let fl = |x: f64| Value::Float(x);
            g.insert("origin.label".to_string(), Value::Text(o.label.clone()));
            g.insert("origin.birth_gyr".to_string(), fl(o.birth_gyr));
            g.insert("origin.age_gyr".to_string(), fl(o.age_gyr));
            g.insert("origin.birth_r_kpc".to_string(), fl(o.birth_r_kpc));
            g.insert("origin.birth_phi_deg".to_string(), fl(o.birth_phi_deg));
            g.insert("origin.now_r_kpc".to_string(), fl(o.now_r_kpc));
            g.insert("origin.migrated_kpc".to_string(), fl(o.migrated_kpc));
            g.insert("origin.feh_gas".to_string(), fl(o.feh_gas));
            g.insert("origin.feh".to_string(), fl(o.feh));
            g.insert(
                "origin.composition".to_string(),
                Value::Text(o.composition.clone()),
            );
            g.insert("origin.why".to_string(), Value::Text(o.why.clone()));
            g.insert(
                "origin.unusual".to_string(),
                o.unusual.clone().map_or(Value::Null, Value::Text),
            );
            g.insert(
                "origin.path".to_string(),
                Value::Text(serde_json::to_string(&o.path).expect("plain data")),
            );
            let e = clock::timeless_set(
                &u.chronicle,
                a,
                g,
                Evidence::Resolved,
                vec![id],
                clock::ORIGIN_KIND,
                Publisher::new("sse.base-cosmology", "origin", "1"),
            )
            .map_err(|e| err("bad_event", e))?;
            origin_event = Some(
                u.chronicle
                    .append(e)
                    .map_err(|e| err("chronicle", format!("{e}")))?
                    .hex(),
            );
        }
        Ok(Response::SystemAgeSet {
            system: a.to_string(),
            before_myr: before,
            age_myr,
            shift_myr: shift,
            event_id: id.hex(),
            origin_event,
            origin: origin.map(|o| serde_json::to_value(o).expect("plain data")),
            impossible,
            composition: serde_json::json!({
                "mode": if keep { "keep" } else { "follow" },
                "pinned_origin": pinned,
                "feh_before": feh_before,
                "feh_after": feh_after,
                "z_before": spec.z,
                "z_after": if rederive { z_after } else { spec.z },
                "rederive": rederive,
            }),
            state_hash: u
                .chronicle
                .state_at(UniverseTime::seconds(i64::MAX))
                .hash()
                .hex(),
        })
    }

    /// Universe and record state at `at_s` with every address's tags (rule
    /// tags under the GM's authored ones, TG3 and TG4), as the GM sees it.
    fn tagged_state(
        &self,
        at_s: i64,
    ) -> Result<
        (
            sse_chronicle::State,
            BTreeMap<Address, std::collections::BTreeSet<String>>,
        ),
        Response,
    > {
        let u = self
            .universe
            .as_ref()
            .ok_or_else(|| err("no_universe", "open a universe first"))?;
        let at = UniverseTime::seconds(at_s);
        let mut state = u.chronicle.state_at(at);
        for (a, f) in u.record.state_at(at).facts {
            state.facts.entry(a).or_default().extend(f);
        }
        let mut tags = BTreeMap::new();
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
            let t: std::collections::BTreeSet<String> = self
                .observe
                .rules
                .apply_authored(
                    &subject,
                    &BTreeMap::new(),
                    &sse_sift::authored_tags(f),
                    Some(&u.seed),
                )
                .into_keys()
                .collect();
            if !t.is_empty() {
                tags.insert(a.clone(), t);
            }
        }
        Ok((state, tags))
    }

    /// `read_matter` (MP1): one row of the master data sheet with its
    /// sources and packs, or each table's ids.
    fn read_matter(&self, table: Option<&str>, id: Option<&str>) -> Result<Response, Response> {
        let r = &self.observe.reference;
        let value = match (table, id) {
            (Some(t), Some(i)) => r.describe(t, i).ok_or_else(|| {
                err(
                    "no_row",
                    format!("no row {i} in the master data sheet's {t} table"),
                )
            })?,
            (None, Some(i)) => sse_reference::TABLES
                .iter()
                .find_map(|t| r.describe(t, i))
                .ok_or_else(|| {
                    err(
                        "no_row",
                        format!("no row {i} in any table of the master data sheet"),
                    )
                })?,
            (t, None) => {
                if let Some(t) = t {
                    if !sse_reference::TABLES.contains(&t) && t != "sources" {
                        return Err(err(
                            "no_table",
                            format!(
                                "the master data sheet has no table {t} (tables: {}, sources)",
                                sse_reference::TABLES.join(", ")
                            ),
                        ));
                    }
                }
                r.listing(t)
            }
        };
        Ok(Response::Matter { value })
    }

    /// `dressing` (TG12): the candidates and orphans under `scope`.
    fn dressing(&self, scope: &str, at_s: i64) -> Result<Response, Response> {
        let (state, tags) = self.tagged_state(at_s)?;
        let u = self.universe.as_ref().expect("checked");
        let scope_a = in_universe(&u.root, scope)?;
        Ok(Response::Dressed {
            scope: scope_a.to_string(),
            candidates: dressing::candidates(
                &self.observe.dressing,
                &self.observe.templates,
                &state,
                &tags,
                &u.seed,
                &scope_a,
                at_s,
            ),
            orphans: dressing::orphans(&self.observe.dressing, &u.chronicle, &state, &scope_a),
        })
    }

    /// The live candidate `id`, or why it is not one.
    fn live_candidate(&self, id: &str, at_s: i64) -> Result<(Address, String), Response> {
        let (world, rule, _) = dressing::parse_candidate(id)
            .ok_or_else(|| err("bad_candidate", format!("{id:?} is not a candidate id")))?;
        let (state, tags) = self.tagged_state(at_s)?;
        let u = self.universe.as_ref().expect("checked");
        let found = dressing::candidates(
            &self.observe.dressing,
            &self.observe.templates,
            &state,
            &tags,
            &u.seed,
            &world,
            at_s,
        )
        .into_iter()
        .any(|c| c["id"] == id);
        if !found {
            return Err(err(
                "no_candidate",
                format!("{id} is not a candidate at that time (its rule does not hold, or it was held off or adopted)"),
            ));
        }
        Ok((world, rule))
    }

    /// `adopt_dressing` (TG12): make the candidate an authored object.
    fn adopt_dressing(
        &mut self,
        id: &str,
        name: Option<&str>,
        at_s: i64,
    ) -> Result<Response, Response> {
        let (world, rule) = self.live_candidate(id, at_s)?;
        let (pack, r) = self.observe.dressing.get(&rule).cloned().expect("live");
        let t = self
            .observe
            .templates
            .get(&r.place)
            .cloned()
            .expect("checked at load");
        let t = dressing::adopt_template(&t, &pack, &r, id);
        let reg = self.observe.record.clone();
        let u = self.universe()?;
        let (c, _) = sse_trigger::templates::create(
            &mut u.chronicle,
            &mut u.record,
            &reg,
            &u.root,
            &t,
            &world,
            None,
            name,
            UniverseTime::seconds(at_s),
            vec![],
        )
        .map_err(|e| err("create_refused", e))?;
        Ok(Response::Created {
            address: c.address,
            event_id: c.event,
            kept: c.kept,
            state_hash: latest_hash(&u.chronicle),
            record_hash: record_hash(u),
        })
    }

    /// `hold_dressing` (TG12): hold a candidate off its world, or let it back.
    fn hold_dressing(&mut self, id: &str, held: bool) -> Result<Response, Response> {
        let (world, rule, n) = dressing::parse_candidate(id)
            .ok_or_else(|| err("bad_candidate", format!("{id:?} is not a candidate id")))?;
        if self.observe.dressing.get(&rule).is_none() {
            return Err(err(
                "no_candidate",
                format!("no dressing rule {rule} in the stack"),
            ));
        }
        let u = self.universe()?;
        let world = in_universe(&u.root, &world.to_string())?;
        let mut f = BTreeMap::new();
        f.insert(
            format!("{}{rule}#{n}", dressing::HELD_PREFIX),
            Value::Bool(held),
        );
        let e = clock::timeless_set(
            &u.record,
            &world,
            f,
            Evidence::Authored,
            vec![],
            "dressing.hold",
            Publisher::new("gm", "dressing", "1"),
        )
        .map_err(|e| err("bad_event", e))?;
        let id_hash = u
            .record
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        Ok(Response::DressingHeld {
            candidate: id.to_string(),
            held,
            event_id: id_hash.hex(),
        })
    }

    /// `promote_star` (Y7a): draw a fill star once and store it whole.
    fn promote_star(&mut self, address: &str) -> Result<Response, Response> {
        let u = self.universe()?;
        let a = in_universe(&u.root, address)?;
        let state = u.chronicle.state_at(UniverseTime::seconds(i64::MAX));
        let hash = |c: &Chronicle| c.state_at(UniverseTime::seconds(i64::MAX)).hash().hex();
        if let Some(Value::Text(t)) = state
            .facts
            .get(&a)
            .and_then(|f| f.get(cosmos::STAR_DESCRIPTION))
        {
            let description: serde_json::Value =
                serde_json::from_str(t).map_err(|e| err("bad_record", format!("{e}")))?;
            return Ok(Response::StarPromoted {
                address: a.to_string(),
                already: true,
                event_id: None,
                description,
                state_hash: hash(&u.chronicle),
            });
        }
        let drawn = cosmos::with_stars(&u.seed, &u.root, None, |st, _| {
            let s = st
                .draw_one(&a)
                .map_err(|e| err("bad_address", e))?
                .ok_or_else(|| err("not_found", format!("no star system drawn at {a}")))?;
            Ok(st.promote(&s))
        })?;
        let text = serde_json::to_string(&drawn).expect("plain data");
        let mut f = BTreeMap::new();
        f.insert(cosmos::STAR_DESCRIPTION.to_string(), Value::Text(text));
        // Searchable copies; the description is what is read back.
        f.insert("star.mass_msun".to_string(), Value::Float(drawn.mass_msun));
        f.insert("star.birth_gyr".to_string(), Value::Float(drawn.birth_gyr));
        f.insert("star.feh".to_string(), Value::Float(drawn.feh));
        f.insert(
            "star.companions".to_string(),
            Value::Int(drawn.companions.len() as i128),
        );
        let e = clock::timeless_set(
            &u.chronicle,
            &a,
            f,
            Evidence::Resolved,
            vec![],
            cosmos::STAR_PIN_KIND,
            Publisher::new("sse.base-cosmology", "stars", "1"),
        )
        .map_err(|e| err("bad_event", e))?;
        let id = u
            .chronicle
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        Ok(Response::StarPromoted {
            address: a.to_string(),
            already: false,
            event_id: Some(id.hex()),
            description: serde_json::to_value(&drawn).expect("plain data"),
            state_hash: hash(&u.chronicle),
        })
    }

    /// The `sift` op: the stack's sifters over the campaign, or over one
    /// audience's scene.
    /// `classify_preview`: label the campaign's bodies with a candidate
    /// stack's atlas, loading and writing nothing.
    fn classify_preview(
        &mut self,
        packs: Vec<serde_json::Value>,
        scheme: String,
        scope: Option<String>,
        at_s: Option<i64>,
    ) -> Result<Response, Response> {
        let mut ms = Vec::new();
        for (i, p) in packs.iter().enumerate() {
            ms.push(
                serde_json::from_value::<sse_pack::PackManifest>(p.clone())
                    .map_err(|e| err("bad_pack", format!("pack {i}: {e}")))?,
            );
        }
        let atlas = sse_sift::load(&ms)
            .map_err(|es| err("bad_stack", es.join("; ")))?
            .atlas;
        if !atlas.schemes.contains_key(&scheme) && !atlas.classes.iter().any(|c| c.scheme == scheme)
        {
            return Err(err(
                "unknown_scheme",
                format!("no pack in the candidate stack provides scheme {scheme}"),
            ));
        }
        let u = self.universe()?;
        let scope = match scope {
            Some(a) => in_universe(&u.root, &a)?,
            None => u.root.clone(),
        };
        let now = UniverseTime::seconds(at_s.unwrap_or(i64::MAX));
        let mut state = u.chronicle.state_at(now);
        for (a, f) in u.record.state_at(now).facts {
            state.facts.entry(a).or_default().extend(f);
        }
        let mut bodies = vec![];
        let mut unclassified = 0;
        for (a, f) in &state.facts {
            if !scope.contains(a) || !(f.contains_key("body.kind") || f.contains_key("body.mass")) {
                continue;
            }
            let subject = sse_sift::Subject {
                address: Some(a.clone()),
                facts: f.clone(),
                parent: a
                    .parent()
                    .and_then(|p| state.facts.get(&p).cloned())
                    .unwrap_or_default(),
                now_s: now.s,
                ..Default::default()
            };
            let l = atlas.label(&scheme, &subject, None);
            if l.base.is_none() {
                unclassified += 1;
            }
            bodies.push(serde_json::json!({
                "address": a.to_string(),
                "base": l.base,
                "fit": l.fit,
                "modifiers": l.modifiers,
            }));
        }
        let failing_examples = atlas
            .check_examples()
            .into_iter()
            .filter(|m| {
                atlas
                    .classes
                    .iter()
                    .any(|c| c.scheme == scheme && m.starts_with(&format!("{}: ", c.id)))
            })
            .collect();
        Ok(Response::ClassifyPreviewed {
            scheme,
            atlas_hash: atlas.content_hash().hex(),
            at_s,
            bodies,
            unclassified,
            failing_examples,
        })
    }

    fn sift(
        &mut self,
        ids: Option<Vec<serde_json::Value>>,
        audience: Option<String>,
        scope: Option<String>,
        at_s: i64,
    ) -> Result<Response, Response> {
        use sse_sift::TagState;
        // Names pick stack sifters (none named: all of them); objects are inline.
        let mut named = Vec::new();
        let mut inline = Vec::new();
        for v in ids.unwrap_or_default() {
            match v {
                serde_json::Value::String(id) => {
                    let s = self
                        .observe
                        .sifters
                        .iter()
                        .find(|s| s.id == id)
                        .ok_or_else(|| {
                            err(
                                "unknown_sifter",
                                format!("no pack in the stack provides sifter {id}"),
                            )
                        })?;
                    named.push(s.clone());
                }
                v => inline.push(
                    serde_json::from_value::<sse_sift::Sifter>(v)
                        .map_err(|e| err("bad_sifter", format!("sifter: {e}")))?,
                ),
            }
        }
        let sifters: Vec<sse_sift::Sifter> = if named.is_empty() {
            self.observe.sifters.iter().cloned().chain(inline).collect()
        } else {
            named.into_iter().chain(inline).collect()
        };
        let root = self
            .universe
            .as_ref()
            .ok_or_else(|| err("no_universe", "open a universe first"))?
            .root
            .clone();
        let scope = match scope {
            None => root.clone(),
            Some(s) => in_universe(&root, &s)?,
        };
        let now = UniverseTime::seconds(at_s);
        let tag_state = |tag: &str, rule: &str| TagState {
            tag: tag.to_string(),
            rule: rule.to_string(),
            evidence: Vec::new(),
        };
        let gm = audience
            .as_deref()
            .is_none_or(|a| a == sse_observe::scene::GM);
        let mut skipped = Vec::new();
        let sifters_run: Vec<String> = sifters.iter().map(|s| s.id.clone()).collect();
        let hits = if gm {
            let u = self.universe.as_ref().expect("checked");
            let mut state = u.chronicle.state_at(now);
            for (a, f) in u.record.state_at(now).facts {
                state.facts.entry(a).or_default().extend(f);
            }
            // Tags as the scene sees them: rule tags (with their evidence) plus
            // the GM's `tag.*` facts, with the fact itself as evidence.
            let mut tags: BTreeMap<Address, BTreeMap<String, TagState>> = BTreeMap::new();
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
                // TG3: the GM's `tag.x = true` forces a tag on (with the
                // authored mark), `false` holds a rule's tag off.
                let t = self.observe.rules.apply_authored(
                    &subject,
                    &BTreeMap::new(),
                    &sse_sift::authored_tags(f),
                    Some(&u.seed),
                );
                if !t.is_empty() {
                    tags.insert(a.clone(), t);
                }
            }
            let clouds = BTreeMap::new();
            let w = sse_sift::World {
                state: &state,
                chronicle: Some(&u.chronicle),
                tags: &tags,
                clouds: &clouds,
                now,
            };
            sse_sift::sift(&sifters, &w, &scope)
        } else {
            let audience = audience.clone().expect("not the GM");
            let scene = match self.dispatch(Request::Scene {
                audience: audience.clone(),
                scope: scope.to_string(),
                data_view: None,
                at_s,
                observe: None,
            })? {
                Response::Scene { scene, .. } => scene,
                other => return Err(other),
            };
            let scene: sse_observe::scene::Scene =
                serde_json::from_value(scene).expect("scenes round-trip");
            let mut state = sse_chronicle::State::default();
            let mut tags: BTreeMap<Address, BTreeMap<String, TagState>> = BTreeMap::new();
            for sub in &scene.subjects {
                let f = state.facts.entry(sub.address.clone()).or_default();
                for x in &sub.facts {
                    if let (Some(v), None) = (&x.value, &x.placeholder) {
                        f.insert(x.field.clone(), v.clone());
                    }
                }
                for t in &sub.tags {
                    tags.entry(sub.address.clone())
                        .or_default()
                        .insert(t.clone(), tag_state(t, "shown"));
                }
            }
            let run: Vec<sse_sift::Sifter> = sifters
                .into_iter()
                .filter(|s| {
                    let facts = matches!(s.pattern, sse_sift::Pattern::Facts { .. });
                    if !facts {
                        skipped.push(s.id.clone());
                    }
                    facts
                })
                .collect();
            let clouds = BTreeMap::new();
            let w = sse_sift::World {
                state: &state,
                chronicle: None,
                tags: &tags,
                clouds: &clouds,
                now,
            };
            sse_sift::sift(&run, &w, &scope)
        };
        Ok(Response::Sifted {
            scope: scope.to_string(),
            sifters: sifters_run
                .into_iter()
                .filter(|id| !skipped.contains(id))
                .collect(),
            hits: serde_json::to_value(hits).expect("hits are plain data"),
            audience: if gm { None } else { audience },
            skipped,
        })
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
                        // TG2: physics flows never read tags or kept values.
                        let mut problems: Vec<String> = tags::physics_wall(&stack);
                        // Boxes (J3): each one's own golden cases, then flows
                        // compiled with the stack's boxes inlined.
                        for (id, b) in &stack.boxes {
                            for m in sse_flow::boxes::run_box_tests(b, &cx, &stack.boxes) {
                                problems
                                    .push(format!("{} (box {id}): test {m}", stack.box_packs[id]));
                            }
                        }
                        for (id, af) in &stack.flows {
                            match Flow::compile_with(af.doc.clone(), &cx, &stack.boxes) {
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
                        let codes = match sse_codes::Codes::load(&ms) {
                            Ok(c) => c,
                            Err(problems) => {
                                let issues: Vec<sse_pack::StackIssue> = problems
                                    .iter()
                                    .map(|m| sse_pack::StackIssue {
                                        pack: "codes".into(),
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
                        self.codes = codes;
                        self.time = sse_time::epoch::TimeSettings::from_packs(&ms).ok();
                        r
                    }
                }
            }
            Request::OpenUniverse { id, seed } => {
                let root =
                    Address::universe(&id).map_err(|e| err("bad_address", format!("{e}")))?;
                let seed_text = seed.clone();
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
                    seed_text,
                    chronicle,
                    branches: BTreeMap::new(),
                    packs,
                    looks: BTreeMap::new(),
                    runs: BTreeMap::new(),
                    record: Chronicle::new(),
                    orbits: vec![],
                };
                let manifest_id = u.manifest().id().hex();
                self.universe = Some(u);
                Response::UniverseOpened {
                    root: root.to_string(),
                    manifest_id,
                }
            }
            Request::SetAuthored { edit } => {
                let stack_tags = self.observe.tags.clone();
                let u = self.universe()?;
                let e = Self::edit_event(u, &edit, u.chronicle.next_stamp())?;
                // TG1: a tag set on: exclusive groups hold, and a name no pack
                // declares joins the campaign's own tags.
                let mut declare = None;
                if let (Some(tag), serde_json::Value::Bool(true)) =
                    (edit.field.strip_prefix("tag."), &edit.value)
                {
                    let set = campaign_tags(u, &stack_tags)?;
                    let a = in_universe(&u.root, &edit.address)?;
                    let state = u
                        .chronicle
                        .state_at(UniverseTime::seconds(edit.occurred_at_s));
                    let have: Vec<String> = state
                        .facts
                        .get(&a)
                        .into_iter()
                        .flatten()
                        .filter(|(_, v)| **v == Value::Bool(true))
                        .filter_map(|(f, _)| f.strip_prefix("tag.").map(str::to_string))
                        .collect();
                    if let Some(m) = set.exclusive_clash(tag, &have) {
                        return Err(err("exclusive_tag", format!("{a}: {m}")));
                    }
                    if !set.declared(tag) {
                        declare = Some(tag.to_string());
                    }
                }
                let affected = e.body.subject.iter().map(|a| a.to_string()).collect();
                let id = u
                    .chronicle
                    .append(e)
                    .map_err(|e| err("chronicle", format!("{e}")))?;
                if let Some(t) = &declare {
                    declare_campaign_tag(u, t)?;
                }
                Response::Authored {
                    event_id: id.hex(),
                    affected,
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                    declared_tag: declare,
                }
            }
            Request::CampaignTags => {
                let stack_tags = self.observe.tags.clone();
                let u = self.universe()?;
                let set = campaign_tags(u, &stack_tags)?;
                Response::CampaignTagsListed {
                    pack: serde_json::json!({
                        "id": "campaign",
                        "tags": set.block_of("campaign"),
                    }),
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
                    declared_tag: None,
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
                u.looks.remove(&branch);
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
                    aged_event: None,
                    aged_at_myr: None,
                    campaign_now_s: None,
                    system_age: None,
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
                u.looks.remove(&branch);
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
                // TG3: authored true forces a tag, authored false holds a
                // rule's tag off; both come out of the facts.
                let mut tags: BTreeMap<Address, std::collections::BTreeSet<String>> =
                    BTreeMap::new();
                let mut authored: BTreeMap<Address, BTreeMap<String, bool>> = BTreeMap::new();
                for (a, f) in state.facts.iter_mut() {
                    let mine = sse_sift::authored_tags(f);
                    f.retain(|k, _| !k.starts_with(TAG_FIELD));
                    if !mine.is_empty() {
                        authored.insert(a.clone(), mine);
                    }
                }
                let none = BTreeMap::new();
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
                        .apply_authored(
                            &subject,
                            &BTreeMap::new(),
                            authored.get(a).unwrap_or(&none),
                            Some(&u.seed),
                        )
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
                let disclosure = with_record(&setup.disclosure, &self.observe.record);
                let notes = sse_record::notes_at(u.record.events(), UniverseTime::seconds(at_s));
                // Kept values and actors live in the record; the GM machine
                // folds them in here and filters them like any fact.
                for (a, f) in u.record.state_at(UniverseTime::seconds(at_s)).facts {
                    state.facts.entry(a).or_default().extend(f);
                }
                let inp = sse_observe::SceneInput {
                    state: &state,
                    chronicle: Some(&u.chronicle),
                    tags: &tags,
                    fictional: &fictional,
                    disclosure: &disclosure,
                    views: &views,
                    campaign: &setup.campaign,
                    notes: &notes,
                };
                let scene =
                    sse_observe::build_scene_through(&inp, &audience, &scope, data_view.as_deref())
                        .map_err(|e| err("bad_data_view", e))?;
                Response::Scene {
                    scene_hash: scene.hash().hex(),
                    scene: serde_json::to_value(&scene).expect("scenes are plain data"),
                }
            }
            Request::ImportV3 {
                save,
                universe_id,
                reconstruct,
            } => systems::import_v3(&save, universe_id.as_deref(), reconstruct)?,
            Request::RunFormation {
                seed,
                system,
                block,
                birth,
                now_gyr,
                envelope,
                inherit_pins,
                star_pins,
                pins,
            } => systems::run_formation(systems::FormArgs {
                seed,
                system,
                block,
                birth,
                now_gyr,
                envelope,
                inherit_pins,
                star_pins,
                pins,
            })?,
            Request::ReadWorldStack {
                body,
                fixture,
                stack,
            } => systems::read_world_stack(body.as_deref(), fixture.as_deref(), stack)?,
            Request::Weather {
                want,
                world,
                conditions,
                day,
                tile,
                n,
                lat,
                lon,
                pins,
                sources,
                block,
            } => weather::weather(weather::Ask {
                want,
                world,
                conditions,
                day: day.unwrap_or(0.0),
                tile,
                n,
                lat: lat.unwrap_or(0.0),
                lon: lon.unwrap_or(0.0),
                pins: pins.unwrap_or_default(),
                sources,
                block,
            })?,
            Request::ReadLight { fixture, bodies } => {
                systems::read_light(fixture.as_deref(), bodies)?
            }
            Request::ReadNebulae {
                at_s,
                seed,
                deaths,
                planetary,
                remnants,
                glows,
            } => systems::read_nebulae(at_s, seed.as_deref(), deaths, planetary, remnants, glows)?,
            Request::SurveyPlanets { survey } => systems::survey_planets(survey)?,
            Request::ReadSpectrum { targets, observer } => {
                systems::read_spectrum(targets, observer)?
            }
            Request::ReadAirOptics {
                airs,
                wavelengths_m,
            } => systems::read_air_optics(airs, wavelengths_m)?,
            Request::Detect {
                targets,
                sensors,
                radar,
            } => systems::detect(targets, sensors, radar)?,
            Request::ReadActivity {
                stars,
                seed,
                at_s,
                window,
            } => systems::read_activity(stars, seed.as_deref(), at_s, window)?,
            Request::Adopt {
                import_v3,
                formation,
                occurred_at_s,
            } => {
                let u = self.universe()?;
                let a = adopt::adopt(&u.chronicle, &u.root, import_v3, formation, occurred_at_s)?;
                let mut events = vec![];
                for e in a.events {
                    events.push(
                        u.chronicle
                            .append(e)
                            .map_err(|e| err("chronicle", format!("{e}")))?
                            .hex(),
                    );
                }
                if let Some((sys, run)) = a.run {
                    u.runs.insert(sys, run);
                }
                Response::Adopted {
                    events,
                    systems: a.systems,
                    facts: a.facts,
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                }
            }
            Request::ResolvedSystem {
                system,
                at_s,
                tiles,
            } => {
                let a: Address = system
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let u = self.universe()?;
                if !u.root.contains(&a) {
                    return Err(err(
                        "bad_address",
                        format!("{a} is not in universe {}", u.root),
                    ));
                }
                if !u.runs.contains_key(&system) {
                    if let Some(req) = adopt::formation_request(&u.chronicle, &a) {
                        let (_, run, _) = systems::form(req.into())?;
                        u.runs.insert(system.clone(), run);
                    }
                }
                let r = adopt::resolved_system(
                    &u.chronicle,
                    &u.seed,
                    &a,
                    at_s,
                    tiles.as_deref().unwrap_or(&[]),
                    u.runs.get(&system),
                    &mut u.orbits,
                )?;
                Response::SystemResolved {
                    system,
                    source: r.source,
                    at_s: r.at_s,
                    facts: r.facts,
                    evidence: r.evidence,
                    spec: r.spec,
                    surfaces: r.surfaces,
                    tiles: r.tiles,
                    refused: r.refused,
                    clouds: r.clouds,
                }
            }
            Request::PlanRoute {
                fixture,
                nav,
                system,
                at_s,
                ports,
                ship,
                from,
                to,
                depart_s,
                window_days,
                prefer,
                rescue,
            } => {
                let ports = ports.unwrap_or_default();
                let ask = |source| nav::Ask {
                    source,
                    ship: &ship,
                    from: &from,
                    to: to.as_ref(),
                    depart_s,
                    window_days,
                    prefer: prefer.as_deref(),
                    rescue: rescue.as_ref(),
                };
                let plan = match (fixture.as_deref(), nav.as_ref(), system) {
                    (Some(f), None, None) => nav::plan_route(ask(nav::Source::Fixture(f)))?,
                    (None, Some(n), None) => nav::plan_route(ask(nav::Source::Inline(n)))?,
                    (None, None, Some(system)) => {
                        let a: Address = system
                            .parse()
                            .map_err(|e| err("bad_address", format!("{e}")))?;
                        let u = self
                            .universe
                            .as_mut()
                            .ok_or_else(|| err("no_universe", "open a universe first"))?;
                        if !u.root.contains(&a) {
                            return Err(err(
                                "bad_address",
                                format!("{a} is not in universe {}", u.root),
                            ));
                        }
                        if !u.runs.contains_key(&system) {
                            if let Some(req) = adopt::formation_request(&u.chronicle, &a) {
                                let (_, run, _) = systems::form(req.into())?;
                                u.runs.insert(system.clone(), run);
                            }
                        }
                        nav::plan_route(ask(nav::Source::System {
                            ch: &u.chronicle,
                            system: &a,
                            at_s,
                            run: u.runs.get(&system),
                            ports: &ports,
                        }))?
                    }
                    _ => {
                        return Err(err(
                            "bad_request",
                            "give one of a fixture, a nav system or a universe system",
                        ))
                    }
                };
                Response::RoutePlanned { plan }
            }
            Request::GalaxyChart {
                cell,
                halo,
                mass_msun,
                at_gyr,
                place_kpc,
                place_phi,
                edits,
            } => {
                let u = self.universe()?;
                let edits: Vec<sse_cosmos::galaxy::Edit> = edits
                    .unwrap_or_default()
                    .into_iter()
                    .map(serde_json::from_value)
                    .collect::<Result<_, _>>()
                    .map_err(|e| err("bad_edit", format!("history edit: {e}")))?;
                let chart = cosmos::galaxy_chart(cosmos::GalaxyAsk {
                    cell,
                    edits: &edits,
                    seed: &u.seed,
                    root: &u.root,
                    halo: halo.as_deref(),
                    mass_msun,
                    at_gyr,
                    place_kpc,
                    place_phi,
                })?;
                Response::GalaxyCharted { chart }
            }
            Request::WindowView {
                fixture,
                nav,
                at_s,
                from,
                at_au,
                sensor,
            } => Response::WindowViewed {
                view: nav::window_view(
                    fixture.as_deref(),
                    nav.as_ref(),
                    at_s,
                    from.as_deref(),
                    at_au,
                    sensor.as_deref(),
                )?,
            },
            Request::WebSlice {
                z,
                t_gyr,
                centre,
                extent_mpc,
                n,
            } => {
                let u = self.universe()?;
                let web = cosmos::web_slice(cosmos::SliceAsk {
                    seed: &u.seed,
                    root: &u.root,
                    z,
                    t_gyr,
                    centre: centre.unwrap_or([0.0; 3]),
                    extent_mpc,
                    n,
                })?;
                Response::WebSliced { web }
            }
            Request::PlanDescent {
                system,
                body,
                fixture,
                at_s,
                options,
                quick,
                lander,
                lat_deg,
                lon_deg,
                entry,
            } => {
                let quick = quick.unwrap_or(false);
                let ask = |from| descent::Ask {
                    from,
                    options: options.as_ref(),
                    quick,
                    lander: &lander,
                    lat_deg,
                    lon_deg,
                    entry: entry.as_ref(),
                };
                let descent = match (fixture, system, body) {
                    (Some(f), None, None) => {
                        descent::plan_descent(ask(survey::From::Fixture(&f)), &mut self.surveys)?
                    }
                    (None, Some(system), Some(body)) => {
                        let a: Address = system
                            .parse()
                            .map_err(|e| err("bad_address", format!("{e}")))?;
                        let u = self
                            .universe
                            .as_mut()
                            .ok_or_else(|| err("no_universe", "open a universe first"))?;
                        if !u.root.contains(&a) {
                            return Err(err(
                                "bad_address",
                                format!("{a} is not in universe {}", u.root),
                            ));
                        }
                        if !u.runs.contains_key(&system) {
                            if let Some(req) = adopt::formation_request(&u.chronicle, &a) {
                                let (_, run, _) = systems::form(req.into())?;
                                u.runs.insert(system.clone(), run);
                            }
                        }
                        descent::plan_descent(
                            ask(survey::From::System {
                                ch: &u.chronicle,
                                seed: &u.seed,
                                system: &a,
                                body: &body,
                                at_s,
                                run: u.runs.get(&system),
                            }),
                            &mut self.surveys,
                        )?
                    }
                    _ => {
                        return Err(err(
                            "bad_request",
                            "give a fixture, or a system and one of its bodies",
                        ))
                    }
                };
                Response::DescentPlanned { descent }
            }
            Request::SurveySurface {
                system,
                body,
                fixture,
                at_s,
                options,
                tiles,
                quick,
                limit,
            } => {
                let tiles = tiles.unwrap_or_default();
                let quick = quick.unwrap_or(false);
                let limit = limit.unwrap_or(200) as usize;
                let survey = match (fixture, system, body) {
                    (Some(f), None, None) => survey::survey_surface(
                        survey::Request {
                            from: survey::From::Fixture(&f),
                            options: options.as_ref(),
                            tiles: &tiles,
                            quick,
                            limit,
                        },
                        &mut self.surveys,
                    )?,
                    (None, Some(system), Some(body)) => {
                        let a: Address = system
                            .parse()
                            .map_err(|e| err("bad_address", format!("{e}")))?;
                        let u = self
                            .universe
                            .as_mut()
                            .ok_or_else(|| err("no_universe", "open a universe first"))?;
                        if !u.root.contains(&a) {
                            return Err(err(
                                "bad_address",
                                format!("{a} is not in universe {}", u.root),
                            ));
                        }
                        if !u.runs.contains_key(&system) {
                            if let Some(req) = adopt::formation_request(&u.chronicle, &a) {
                                let (_, run, _) = systems::form(req.into())?;
                                u.runs.insert(system.clone(), run);
                            }
                        }
                        survey::survey_surface(
                            survey::Request {
                                from: survey::From::System {
                                    ch: &u.chronicle,
                                    seed: &u.seed,
                                    system: &a,
                                    body: &body,
                                    at_s,
                                    run: u.runs.get(&system),
                                },
                                options: options.as_ref(),
                                tiles: &tiles,
                                quick,
                                limit,
                            },
                            &mut self.surveys,
                        )?
                    }
                    _ => {
                        return Err(err(
                            "bad_request",
                            "give a fixture, or a system and one of its bodies",
                        ))
                    }
                };
                Response::SurfaceSurveyed { survey }
            }
            Request::LifeView { body, fixture } => {
                let (source, view) = adopt::life_view(body.as_deref(), fixture.as_deref())?;
                Response::LifeViewed { source, view }
            }
            Request::ExportChronicle => {
                let u = self.universe()?;
                let events: Vec<serde_json::Value> = u
                    .chronicle
                    .events()
                    .iter()
                    .map(|e| {
                        let bytes = sse_core::cbor::to_canonical(&e.body).expect("event bodies encode");
                        serde_json::json!({"id": e.id.hex(), "kind": e.body.kind, "cbor": adopt::hex(&bytes)})
                    })
                    .collect();
                let record: Vec<serde_json::Value> = u
                    .record
                    .events()
                    .iter()
                    .map(|e| {
                        let bytes = sse_core::cbor::to_canonical(&e.body).expect("event bodies encode");
                        serde_json::json!({"id": e.id.hex(), "kind": e.body.kind, "cbor": adopt::hex(&bytes)})
                    })
                    .collect();
                let manifest_id = u.manifest().id().hex();
                Response::ChronicleExported {
                    export: serde_json::json!({
                        "format": adopt::EXPORT_FORMAT,
                        "kernel_version": KERNEL_VERSION,
                        "root": u.root.to_string(),
                        "seed": u.seed_text,
                        "manifest_id": manifest_id,
                        "packs": u.packs,
                        "head": u.chronicle.head().hex(),
                        "events": events,
                        "record": record,
                        "record_head": u.record.head().hex(),
                    }),
                }
            }
            Request::ImportChronicle { export } => {
                let ex = &export;
                if ex["format"] != adopt::EXPORT_FORMAT {
                    return Err(err(
                        "bad_export",
                        format!("format must be {}", adopt::EXPORT_FORMAT),
                    ));
                }
                let text = |k: &str| -> Result<String, Response> {
                    ex[k]
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| err("bad_export", format!("missing {k}")))
                };
                let root: Address = text("root")?
                    .parse()
                    .map_err(|e| err("bad_export", format!("root: {e}")))?;
                let seed_text = text("seed")?;
                let evs = ex["events"]
                    .as_array()
                    .ok_or_else(|| err("bad_export", "missing events"))?;
                let chronicle = replay_export(evs, "event")?;
                // A GM save carries the record; older exports have none.
                let empty = vec![];
                let rec = ex["record"].as_array().unwrap_or(&empty);
                let record = replay_export(rec, "record event")?;
                if !rec.is_empty()
                    && ex["record_head"].as_str() != Some(record.head().hex().as_str())
                {
                    return Err(err("bad_export", "record head does not match the record"));
                }
                if ex["head"].as_str() != Some(chronicle.head().hex().as_str()) {
                    return Err(err("bad_export", "head does not match the events"));
                }
                let packs: Vec<sse_chronicle::PackRef> =
                    serde_json::from_value(ex["packs"].clone())
                        .map_err(|x| err("bad_export", format!("packs: {x}")))?;
                let stack_matches = self
                    .stack
                    .as_ref()
                    .map(|(s, _)| pack_refs(s) == packs)
                    .unwrap_or(packs.is_empty());
                let u = Universe {
                    root: root.clone(),
                    seed: Seed::from_text(&seed_text),
                    seed_text,
                    chronicle,
                    branches: BTreeMap::new(),
                    packs,
                    looks: BTreeMap::new(),
                    runs: BTreeMap::new(),
                    record,
                    orbits: vec![],
                };
                let r = Response::ChronicleImported {
                    root: root.to_string(),
                    events: evs.len() as u32,
                    head: u.chronicle.head().hex(),
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                    manifest_id: u.manifest().id().hex(),
                    stack_matches,
                };
                self.universe = Some(u);
                r
            }
            Request::AddNote {
                address,
                links,
                text,
                tags,
                by,
                audience,
                tier,
                supersedes,
                narrative,
                at_s,
                arrives_at_s,
            } => {
                let u = self.universe()?;
                let mut subjects = vec![in_universe(&u.root, &address)?];
                for l in links.unwrap_or_default() {
                    subjects.push(in_universe(&u.root, &l)?);
                }
                let audience: sse_record::Audience = match audience {
                    None => sse_record::Audience::gm(),
                    Some(v) => serde_json::from_value(v).map_err(|e| {
                        err(
                            "bad_note",
                            format!("audience: \"everyone\", \"gm\" or a list ({e})"),
                        )
                    })?,
                };
                let supersedes = match supersedes {
                    None => None,
                    Some(h) => Some(ContentHash::parse(&h).ok_or_else(|| {
                        err("bad_note", format!("supersedes {h:?} is not an event id"))
                    })?),
                };
                let by = by.unwrap_or_else(|| "gm".into());
                let d = sse_record::NoteDraft {
                    subjects,
                    text,
                    tags: tags.unwrap_or_default(),
                    by: by.clone(),
                    audience,
                    tier,
                    supersedes,
                    narrative: narrative.unwrap_or(false),
                    arrives: arrives_at_s.map(UniverseTime::seconds),
                };
                let id = sse_record::add_note(
                    &mut u.record,
                    &d,
                    UniverseTime::seconds(at_s),
                    &Publisher::new(&by, "note", "1"),
                )
                .map_err(record_err)?;
                let note = u
                    .record
                    .events()
                    .iter()
                    .rev()
                    .find(|e| e.id == id)
                    .and_then(sse_record::note_of)
                    .expect("just written");
                Response::NoteAdded {
                    note_id: id.hex(),
                    audience: serde_json::to_value(&note.audience).expect("plain"),
                    state_hash: latest_hash(&u.chronicle),
                    record_hash: record_hash(u),
                }
            }
            Request::SetKept {
                address,
                kind,
                value,
                writer,
                reason,
                at_s,
            } => {
                let reg = self.observe.record.clone();
                let u = self.universe()?;
                let a = in_universe(&u.root, &address)?;
                let value = match value {
                    serde_json::Value::Null => None,
                    v => Some(json_to_value(&v).map_err(|e| err("bad_value", e))?),
                };
                let at = UniverseTime::seconds(at_s);
                let previous = u.record.state_at(at).get(&a, &kind).cloned();
                let id = sse_record::set_kept(
                    &mut u.record,
                    &reg,
                    &sse_record::KeptSet {
                        address: a,
                        kind,
                        value,
                        writer: writer.unwrap_or_else(|| "gm".into()),
                        reason,
                        causes: vec![],
                    },
                    at,
                    &Publisher::new("gm", "set_kept", "1"),
                )
                .map_err(record_err)?;
                Response::KeptSet {
                    change_id: id.hex(),
                    previous: previous.map_or(serde_json::Value::Null, |v| {
                        serde_json::to_value(&v).expect("plain")
                    }),
                    state_hash: latest_hash(&u.chronicle),
                    record_hash: record_hash(u),
                }
            }
            Request::AddActor {
                id,
                parent,
                name,
                kind,
                at_s,
            } => {
                let u = self.universe()?;
                let parent = match parent {
                    None => None,
                    Some(p) => Some(in_universe(&u.root, &p)?),
                };
                let root = u.root.clone();
                let a = sse_record::add_actor(
                    &mut u.record,
                    &root,
                    parent.as_ref(),
                    &id,
                    &name,
                    &kind,
                    UniverseTime::seconds(at_s),
                    &Publisher::new("gm", "add_actor", "1"),
                )
                .map_err(record_err)?;
                Response::ActorAdded {
                    address: a.to_string(),
                    state_hash: latest_hash(&u.chronicle),
                    record_hash: record_hash(u),
                }
            }
            Request::Disclosures { audience, at_s } => {
                if audience == sse_observe::scene::GM {
                    return Err(err(
                        "bad_audience",
                        "the GM reads the record itself (op record)",
                    ));
                }
                let root = self.universe()?.root.to_string();
                let scene = match self.dispatch(Request::Scene {
                    audience: audience.clone(),
                    scope: root,
                    data_view: None,
                    at_s,
                    observe: None,
                })? {
                    Response::Scene { scene, .. } => scene,
                    other => return Err(other),
                };
                let u = self.universe()?;
                let rstate = u.record.state_at(UniverseTime::seconds(at_s));
                let mut items = Vec::new();
                // One stream id per note, the same at every subject it names.
                let mut ids: BTreeMap<String, String> = BTreeMap::new();
                for sub in scene["subjects"].as_array().into_iter().flatten() {
                    let a: Address = sub["address"]
                        .as_str()
                        .unwrap_or("")
                        .parse()
                        .expect("scene addresses parse");
                    let rf = rstate.facts.get(&a);
                    let kept: Vec<serde_json::Value> = sub["facts"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|f| {
                            f["placeholder"].is_null()
                                && rf.is_some_and(|m| {
                                    m.contains_key(f["field"].as_str().unwrap_or(""))
                                })
                        })
                        .map(|f| {
                            let mut f = f.clone();
                            if let Some(o) = f.as_object_mut() {
                                o.remove("marks");
                            }
                            f
                        })
                        .collect();
                    let notes: Vec<serde_json::Value> = sub["notes"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|x| {
                            let mut at: Vec<String> = x["links"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(|l| l.as_str().map(str::to_string))
                                .collect();
                            at.push(a.to_string());
                            at.sort();
                            let key =
                                serde_json::json!([at, x["at_s"], x["by"], x["text"], x["tags"]])
                                    .to_string();
                            let next = format!("n{}", ids.len() + 1);
                            let id = ids.entry(key).or_insert(next).clone();
                            let mut x = x.clone();
                            x["id"] = serde_json::json!(id);
                            x
                        })
                        .collect();
                    if kept.is_empty() && notes.is_empty() {
                        continue;
                    }
                    let mut item = serde_json::json!({"subject": a.to_string()});
                    if !notes.is_empty() {
                        item["notes"] = serde_json::json!(notes);
                    }
                    if !kept.is_empty() {
                        item["kept"] = serde_json::json!(kept);
                    }
                    items.push(item);
                }
                let stream_hash = ContentHash::of(&items)
                    .map_err(|e| err("internal", format!("{e:?}")))?
                    .hex();
                Response::Disclosed {
                    audience,
                    at_s,
                    items,
                    stream_hash,
                }
            }
            Request::Record { address, at_s } => {
                let reg = self.observe.record.clone();
                let u = self.universe()?;
                let a = in_universe(&u.root, &address)?;
                let at = UniverseTime::seconds(at_s.unwrap_or(i64::MAX));
                let notes = sse_record::notes_at(u.record.events(), at)
                    .into_iter()
                    .filter(|n| n.about(&a))
                    .map(|n| serde_json::to_value(&n).expect("plain"))
                    .collect();
                let kept = sse_record::kept_at(&u.record, &reg, &a, at)
                    .into_iter()
                    .map(|k| serde_json::to_value(&k).expect("plain"))
                    .collect();
                let actors = sse_record::actors(&u.record, &a, at)
                    .into_iter()
                    .map(|(x, n, k)| serde_json::json!({"address": x.to_string(), "name": n, "kind": k}))
                    .collect();
                Response::RecordRead {
                    address: a.to_string(),
                    notes,
                    kept,
                    actors,
                }
            }
            Request::Systems => {
                let u = self.universe()?;
                Response::SystemsListed {
                    systems: adopt::systems_of(&u.chronicle, &u.root),
                }
            }
            Request::Atoms { ids } => {
                let port = |p: &sse_flow::Port| {
                    let mut j = serde_json::json!({
                        "name": p.name, "kind": p.kind, "dim": p.dim.to_string(), "valid": p.valid,
                    });
                    // Logic ports only, so physics atoms list as before.
                    if let Some(s) = &p.shape {
                        j["shape"] = serde_json::json!(s.to_string());
                    }
                    if p.any_dim {
                        j["any_dim"] = serde_json::json!(true);
                    }
                    j
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
                    if let Some(c) = d.class {
                        let last = atoms.last_mut().expect("just pushed");
                        last["class"] = serde_json::json!(c);
                        if let Some(r) = d.dims {
                            last["dims"] = serde_json::json!(r);
                        }
                    }
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
            Request::LookAhead {
                system,
                target_myr,
                settings,
            } => {
                use sse_time::branch as tb;
                let settings: sse_time::epoch::TimeSettings = match settings {
                    Some(v) => serde_json::from_value(v)
                        .map_err(|e| err("bad_settings", format!("time settings: {e}")))?,
                    None => self.time.clone().ok_or_else(|| {
                        err(
                            "no_time_settings",
                            "no pack in the stack provides time settings",
                        )
                    })?,
                };
                let spec: sse_time::epoch::SystemSpec = serde_json::from_value(system)
                    .map_err(|e| err("bad_system", format!("system: {e}")))?;
                let u = self.universe()?;
                let a: Address = spec
                    .system
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                if !u.root.contains(&a) {
                    return Err(err(
                        "bad_address",
                        format!("{a} is not in universe {}", u.root),
                    ));
                }
                if !target_myr.is_finite() || target_myr <= spec.age_myr {
                    return Err(err(
                        "bad_target",
                        format!(
                            "target {target_myr} Myr must be after the star's age now ({} Myr)",
                            spec.age_myr
                        ),
                    ));
                }
                let horizon = spec.age_myr + settings.horizon_myr;
                let req = tb::Request::new(spec, settings, target_myr);
                let look = tb::look_ahead(&mut u.chronicle, req, &mut |_| true)
                    .map_err(|e| err("look_ahead", e))?
                    .ok_or_else(|| err("look_ahead", "cancelled"))?;
                let sp = &look.request.system;
                let year = |age: f64| sp.birth_year + sse_core::math::round(age * 1.0e6) as i64;
                let events = look
                    .run
                    .events
                    .iter()
                    .map(|e| FateEvent {
                        kind: e.kind.clone(),
                        age_myr: e.age_myr,
                        year: year(e.age_myr),
                        subjects: e.subjects.clone(),
                        why: e.why.clone(),
                        level: e.level.clone(),
                        plausible_future: e.plausible_future,
                        numbers: e.numbers.clone(),
                    })
                    .collect();
                let snapshots = look
                    .run
                    .snapshots
                    .iter()
                    .map(|s| FateSnapshot {
                        age_myr: s.age_myr,
                        year: year(s.age_myr),
                        star: FateStar {
                            phase: s.star.phase.clone(),
                            luminosity: s.star.luminosity,
                            radius_rsun: s.star.radius_rsun,
                            teff: s.star.teff,
                            mass: s.star.mass,
                            colour: s.star.colour.clone(),
                            rgb: s.star.rgb,
                        },
                        planets: s
                            .planets
                            .iter()
                            .map(|p| FatePlanet {
                                name: p.name.clone(),
                                a_au: p.a_au,
                                e: p.e,
                                i_deg: p.i_deg,
                                engulfed: p.engulfed,
                            })
                            .collect(),
                    })
                    .collect();
                let id = look.branch.hex();
                let label = u
                    .chronicle
                    .branch_info(&look.branch)
                    .map(|b| b.label.clone())
                    .unwrap_or_default();
                let r = Response::LookedAhead {
                    branch: id.clone(),
                    label,
                    system: sp.system.clone(),
                    now_myr: sp.age_myr,
                    target_myr,
                    birth_year: sp.birth_year,
                    horizon_myr: horizon,
                    events,
                    snapshots,
                    work_units: u32::try_from(look.run.work).unwrap_or(u32::MAX),
                };
                u.branches.insert(id.clone(), look.branch);
                u.looks.insert(id, look);
                r
            }
            Request::CommitLookAhead {
                branch,
                accept,
                composition,
                move_now_myr,
                write_back,
            } => {
                let mode = match accept.as_deref() {
                    Some(m @ ("campaign_now" | "system_age")) => Some(m.to_string()),
                    Some(other) => {
                        return Err(err(
                            "bad_accept",
                            format!("accept {other:?}: campaign_now or system_age"),
                        ))
                    }
                    None => move_now_myr.map(|_| "system_age".to_string()),
                };
                let u = self.universe()?;
                let mut look = u.looks.remove(&branch).ok_or_else(|| {
                    err(
                        "unknown_branch",
                        format!("{branch} is not an open look-ahead"),
                    )
                })?;
                u.branches.remove(&branch);
                let sp = look.request.system.clone();
                let target = move_now_myr.unwrap_or(look.request.target_myr);
                if mode.is_some() && (!target.is_finite() || target < sp.age_myr) {
                    return Err(err(
                        "bad_target",
                        format!(
                            "move to {target} Myr: before the star's age now ({} Myr)",
                            sp.age_myr
                        ),
                    ));
                }
                let delta = target - sp.age_myr;
                if mode.as_deref() == Some("system_age") {
                    // Run the same future again from a birth moved earlier, so
                    // its events fall before today; the first branch goes.
                    u.chronicle
                        .discard(&look.branch)
                        .map_err(|e| err("chronicle", format!("{e}")))?;
                    let mut sp2 = sp.clone();
                    sp2.birth_year -= sse_core::math::round(delta * 1.0e6) as i64;
                    let req = sse_time::branch::Request::new(
                        sp2,
                        look.request.settings.clone(),
                        look.request.target_myr,
                    );
                    look = sse_time::branch::look_ahead(&mut u.chronicle, req, &mut |_| true)
                        .map_err(|e| err("look_ahead", e))?
                        .ok_or_else(|| err("look_ahead", "cancelled"))?;
                }
                let id = sse_time::branch::commit(&mut u.chronicle, &look, None)
                    .map_err(|e| err("chronicle", e))?;
                let mut aged = None;
                if write_back.unwrap_or(false) {
                    if let Some((e, age)) =
                        adopt::aged_event(&u.chronicle, &look, mode.as_ref().map(|_| target), id)?
                    {
                        let eid = u
                            .chronicle
                            .append(e)
                            .map_err(|e| err("chronicle", format!("{e}")))?;
                        aged = Some((eid.hex(), age));
                    }
                }
                let mut campaign_now_s = None;
                let mut system_age = None;
                match mode.as_deref() {
                    Some("campaign_now") => {
                        let root = u.root.clone();
                        let last = match clock::campaign_span_s(&u.chronicle) {
                            Some((_, last)) => last,
                            None => {
                                // The clock starts here: its first reading is the epoch.
                                let e = orbits::campaign_now_event(&u.chronicle, &root, 0)?;
                                u.chronicle
                                    .append(e)
                                    .map_err(|e| err("chronicle", format!("{e}")))?;
                                0
                            }
                        };
                        // Whole years, as look-ahead dates are.
                        let at = last + sse_core::math::round(delta * 1.0e6) as i64 * 31_557_600;
                        let e = orbits::campaign_now_event(&u.chronicle, &root, at)?;
                        u.chronicle
                            .append(e)
                            .map_err(|e| err("chronicle", format!("{e}")))?;
                        campaign_now_s = Some(at);
                    }
                    Some(_) => {
                        let a: Address = sp
                            .system
                            .parse()
                            .map_err(|e| err("bad_address", format!("{e}")))?;
                        let r = self.set_system_age(
                            &a,
                            target,
                            composition.as_deref(),
                            vec![id],
                            Some(sp.clone()),
                        )?;
                        system_age = Some(serde_json::to_value(&r).expect("plain data"));
                    }
                    None => {}
                }
                let u = self.universe()?;
                Response::Committed {
                    aged_event: aged.as_ref().map(|a| a.0.clone()),
                    aged_at_myr: aged.map(|a| a.1),
                    event_id: id.hex(),
                    state_hash: u
                        .chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex(),
                    campaign_now_s,
                    system_age,
                }
            }
            Request::SetSystemAge {
                system,
                age_myr,
                composition,
            } => {
                let a: Address = system
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                self.set_system_age(&a, age_myr, composition.as_deref(), vec![], None)?
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
            Request::RulesStep { at_s } => {
                let (rules, reg) = (self.observe.triggers.clone(), self.observe.record.clone());
                let tpl = self.observe.templates.clone();
                let campaign = self.observe.setup.as_ref().map(|s| s.campaign.clone());
                let u = self.universe()?;
                let root = u.root.clone();
                let report = rules
                    .step(
                        &mut sse_trigger::Gate {
                            universe: &mut u.chronicle,
                            record: &mut u.record,
                            registry: &reg,
                            campaign: campaign.as_ref(),
                            root: &root,
                            templates: &tpl,
                        },
                        at_s,
                    )
                    .map_err(|e| err("rules", e))?;
                Response::RulesStepped {
                    report: serde_json::to_value(&report).expect("plain"),
                    state_hash: latest_hash(&u.chronicle),
                    record_hash: record_hash(u),
                }
            }
            Request::RulesPending => {
                let u = self.universe()?;
                Response::RulesPending {
                    proposals: sse_trigger::Rules::pending(&u.record)
                        .iter()
                        .map(|p| serde_json::to_value(p).expect("plain"))
                        .collect(),
                }
            }
            Request::RulesDecide {
                proposal,
                approve,
                at_s,
            } => {
                let (rules, reg) = (self.observe.triggers.clone(), self.observe.record.clone());
                let tpl = self.observe.templates.clone();
                let campaign = self.observe.setup.as_ref().map(|s| s.campaign.clone());
                let u = self.universe()?;
                let root = u.root.clone();
                let written = rules
                    .decide(
                        &mut sse_trigger::Gate {
                            universe: &mut u.chronicle,
                            record: &mut u.record,
                            registry: &reg,
                            campaign: campaign.as_ref(),
                            root: &root,
                            templates: &tpl,
                        },
                        &proposal,
                        approve,
                        at_s,
                    )
                    .map_err(|e| err("rules", e))?;
                Response::RulesDecided {
                    written: written
                        .iter()
                        .map(|w| serde_json::to_value(w).expect("plain"))
                        .collect(),
                    state_hash: latest_hash(&u.chronicle),
                    record_hash: record_hash(u),
                }
            }
            Request::RulesPolicy {
                rule,
                approval,
                at_s,
            } => {
                let approval: sse_trigger::Approval = serde_json::from_value(approval)
                    .map_err(|e| err("rules", format!("approval: {e}")))?;
                let (rules, reg) = (self.observe.triggers.clone(), self.observe.record.clone());
                let tpl = self.observe.templates.clone();
                let u = self.universe()?;
                let root = u.root.clone();
                let id = rules
                    .set_policy(
                        &mut sse_trigger::Gate {
                            universe: &mut u.chronicle,
                            record: &mut u.record,
                            registry: &reg,
                            campaign: None,
                            root: &root,
                            templates: &tpl,
                        },
                        &rule,
                        &approval,
                        at_s,
                    )
                    .map_err(|e| err("rules", e))?;
                Response::RulesPolicySet {
                    event_id: id.hex(),
                    record_hash: record_hash(u),
                }
            }
            Request::Rules => {
                let rules = self.observe.triggers.clone();
                let u = self.universe()?;
                Response::RulesListed {
                    rules: rules.listing(&u.record),
                    trail: sse_trigger::trail(&u.record),
                }
            }
            Request::Intents => {
                let reg = self.observe.intents.clone();
                let kinds = serde_json::to_value(&reg.kinds).expect("plain");
                let (seats, pending) = match self.universe.as_ref() {
                    Some(u) => (
                        serde_json::to_value(sse_trigger::intents::seats(&reg, &u.record))
                            .expect("plain"),
                        serde_json::to_value(sse_trigger::intents::pending(&u.record))
                            .expect("plain"),
                    ),
                    None => (serde_json::json!([]), serde_json::json!([])),
                };
                Response::IntentsListed {
                    kinds: arr(kinds),
                    seats: arr(seats),
                    pending: arr(pending),
                }
            }
            Request::SetSeats { seats, at_s } => {
                let reg = self.observe.intents.clone();
                let seats: Vec<sse_trigger::intents::Seat> =
                    serde_json::from_value(serde_json::Value::Array(seats))
                        .map_err(|e| err("bad_seats", e.to_string()))?;
                let u = self.universe()?;
                let id =
                    sse_trigger::intents::set_seats(&reg, &mut u.record, &u.root, &seats, at_s)
                        .map_err(|e| err("bad_seats", e))?;
                Response::SeatsSet {
                    event_id: id.hex(),
                    record_hash: record_hash(u),
                }
            }
            Request::SendIntent {
                seat,
                intent,
                at_s,
                check_only,
            } => {
                use sse_trigger::intents as si;
                let reg = self.observe.intents.clone();
                let u = self.universe()?;
                let s = si::seats(&reg, &u.record)
                    .into_iter()
                    .find(|x| x.id == seat)
                    .ok_or_else(|| err("unknown_seat", format!("no seat {seat}")))?;
                let valid = si::validity(&reg, &intent, &u.chronicle, &u.record, &u.root, at_s);
                let decision = si::decide(&reg, &s, &intent, valid);
                if check_only.unwrap_or(false) {
                    return Ok(Response::IntentSent {
                        decision: serde_json::to_value(&decision).expect("plain"),
                        applied_by: None,
                        proposal: None,
                        applied: None,
                    });
                }
                match decision {
                    si::Decision::Refuse { .. } => Response::IntentSent {
                        decision: serde_json::to_value(&decision).expect("plain"),
                        applied_by: None,
                        proposal: None,
                        applied: None,
                    },
                    si::Decision::AskGm => {
                        let root = u.root.clone();
                        let id = si::propose(&reg, &mut u.record, &root, &s, &intent, at_s)
                            .map_err(|e| err("chronicle", e))?;
                        Response::IntentSent {
                            decision: serde_json::to_value(&decision).expect("plain"),
                            applied_by: None,
                            proposal: Some(id.hex()),
                            applied: None,
                        }
                    }
                    si::Decision::Apply => {
                        let (by, applied) = self.apply_intent(&s.id, &intent, at_s, vec![])?;
                        Response::IntentSent {
                            decision: serde_json::to_value(&decision).expect("plain"),
                            applied_by: Some(by),
                            proposal: None,
                            applied,
                        }
                    }
                }
            }
            Request::IntentDecide {
                proposal,
                approve,
                at_s,
            } => {
                let u = self.universe()?;
                let at = at_s.unwrap_or(i64::MIN);
                let (p, id) = sse_trigger::intents::answer(&mut u.record, &proposal, approve, at)
                    .map_err(|e| err("unknown_proposal", e))?;
                let at = at_s.unwrap_or(p.at_s);
                let (applied_by, applied) = if approve {
                    let (by, a) = self.apply_intent(&p.seat, &p.intent, at, vec![id])?;
                    (Some(by), a)
                } else {
                    (None, None)
                };
                Response::IntentDecided {
                    event_id: id.hex(),
                    seat: p.seat,
                    intent: p.intent,
                    applied_by,
                    applied,
                }
            }
            Request::Templates => Response::TemplatesListed {
                templates: arr(
                    serde_json::to_value(&self.observe.templates.templates).expect("plain")
                ),
            },
            Request::Create {
                template,
                under,
                key,
                name,
                at_s,
            } => {
                let t = self
                    .observe
                    .templates
                    .get(&template)
                    .ok_or_else(|| err("unknown_template", format!("no template {template}")))?
                    .clone();
                let reg = self.observe.record.clone();
                let u = self.universe()?;
                let under = in_universe(&u.root, &under)?;
                let (c, _) = sse_trigger::templates::create(
                    &mut u.chronicle,
                    &mut u.record,
                    &reg,
                    &u.root,
                    &t,
                    &under,
                    key.as_deref(),
                    name.as_deref(),
                    UniverseTime::seconds(at_s),
                    vec![],
                )
                .map_err(|e| err("create_refused", e))?;
                Response::Created {
                    address: c.address,
                    event_id: c.event,
                    kept: c.kept,
                    state_hash: latest_hash(&u.chronicle),
                    record_hash: record_hash(u),
                }
            }
            Request::Retire {
                address,
                reason,
                at_s,
            } => {
                let u = self.universe()?;
                let a = in_universe(&u.root, &address)?;
                let id = sse_trigger::templates::retire(
                    &mut u.chronicle,
                    &a,
                    reason.as_deref(),
                    UniverseTime::seconds(at_s),
                    &Publisher::new("gm", "retire", "1"),
                )
                .map_err(|e| err("retire_refused", e))?;
                Response::Retired {
                    event_id: id.hex(),
                    state_hash: latest_hash(&u.chronicle),
                }
            }
            Request::Sift {
                sifters,
                audience,
                scope,
                at_s,
            } => self.sift(sifters, audience, scope, at_s.unwrap_or(i64::MAX))?,
            Request::ClassifyPreview {
                packs,
                scheme,
                scope,
                at_s,
            } => self.classify_preview(packs, scheme, scope, at_s)?,
            Request::Programs { at_s } => {
                let ps = self.observe.programs.clone();
                let u = self.universe()?;
                let running = sse_trigger::programs::running(&u.record, at_s.unwrap_or(i64::MAX));
                Response::ProgramsListed {
                    programs: ps
                        .programs
                        .iter()
                        .map(|p| serde_json::to_value(p).expect("plain"))
                        .collect(),
                    limit: ps.limit as u32,
                    running: running
                        .values()
                        .map(|r| serde_json::to_value(r).expect("plain"))
                        .collect(),
                }
            }
            Request::AttachProgram {
                address,
                program,
                seat,
                at_s,
            } => {
                use sse_trigger::programs as sp;
                let ps = self.observe.programs.clone();
                let reg = self.observe.intents.clone();
                let u = self.universe()?;
                let a = in_universe(&u.root, &address)?;
                let p = ps.get(&program).ok_or_else(|| {
                    err(
                        "unknown_program",
                        format!("no pack in the stack provides program {program}"),
                    )
                })?;
                sse_trigger::templates::may_run(
                    &u.chronicle,
                    &a,
                    &program,
                    UniverseTime::seconds(at_s),
                )
                .map_err(|e| err("program_refused", e))?;
                let s = sse_trigger::intents::seats(&reg, &u.record)
                    .into_iter()
                    .find(|x| x.id == seat)
                    .ok_or_else(|| err("unknown_seat", format!("no seat {seat}")))?;
                if s.program.as_deref() != Some(program.as_str()) {
                    return Err(err(
                        "program_refused",
                        format!("seat {seat} is not a program seat for {program}"),
                    ));
                }
                let running = sp::running(&u.record, at_s);
                if running.contains_key(&a.to_string()) {
                    return Err(err(
                        "program_refused",
                        format!("{a} already runs a program"),
                    ));
                }
                if running.len() >= ps.limit {
                    return Err(err(
                        "program_limit",
                        format!("the campaign allows {} running programs", ps.limit),
                    ));
                }
                let id = sp::write(
                    &mut u.record,
                    sp::kinds::ATTACHED,
                    &a,
                    at_s,
                    &[
                        ("program", &program),
                        ("seat", &seat),
                        ("state", &p.doc.start),
                    ],
                    &program,
                )
                .map_err(|e| err("chronicle", e))?;
                Response::ProgramAttached {
                    event_id: id.hex(),
                    record_hash: record_hash(u),
                }
            }
            Request::DetachProgram { address, at_s } => {
                use sse_trigger::programs as sp;
                let u = self.universe()?;
                let a = in_universe(&u.root, &address)?;
                let r = sp::running(&u.record, at_s)
                    .remove(&a.to_string())
                    .ok_or_else(|| err("program_refused", format!("{a} runs no program")))?;
                let id = sp::write(
                    &mut u.record,
                    sp::kinds::DETACHED,
                    &a,
                    at_s,
                    &[("program", &r.program)],
                    &r.program,
                )
                .map_err(|e| err("chronicle", e))?;
                Response::ProgramDetached {
                    event_id: id.hex(),
                    record_hash: record_hash(u),
                }
            }
            Request::ProgramsStep { at_s } => self.programs_step(at_s)?,
            Request::OrbitsAt {
                system,
                at_s,
                bodies,
            } => {
                if at_s.is_empty() || at_s.len() > orbits::MAX_TIMES {
                    return Err(err(
                        "bad_request",
                        format!("at_s needs 1 to {} times", orbits::MAX_TIMES),
                    ));
                }
                let a: Address = system
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let (facts, seed) = match &bodies {
                    Some(b) => (
                        orbits::inline_facts(b)?,
                        self.universe.as_ref().map(|u| u.seed),
                    ),
                    None => {
                        let u = self.universe()?;
                        if !u.root.contains(&a) {
                            return Err(err(
                                "bad_address",
                                format!("{a} is not in universe {}", u.root),
                            ));
                        }
                        let st = u.chronicle.state_at(UniverseTime::seconds(i64::MAX));
                        let f: BTreeMap<Address, BTreeMap<String, Value>> = st
                            .facts
                            .iter()
                            .filter(|(x, _)| a.contains(x))
                            .map(|(x, r)| (x.clone(), r.clone()))
                            .collect();
                        (f, Some(u.seed))
                    }
                };
                let placed = orbits::orbits_at(&facts, &a, &at_s, seed.as_ref());
                Response::OrbitsPlaced {
                    system,
                    bodies: placed
                        .iter()
                        .map(|p| serde_json::to_value(p).expect("plain data"))
                        .collect(),
                }
            }
            Request::SetCampaignNow { at_s } => {
                let u = self.universe()?;
                let e = orbits::campaign_now_event(&u.chronicle, &u.root, at_s)?;
                let id = u
                    .chronicle
                    .append(e)
                    .map_err(|e| err("chronicle", format!("{e}")))?;
                Response::CampaignNowIs {
                    at_s: Some(at_s),
                    event_id: Some(id.hex()),
                    state_hash: Some(
                        u.chronicle
                            .state_at(UniverseTime::seconds(i64::MAX))
                            .hash()
                            .hex(),
                    ),
                }
            }
            Request::CampaignNow => {
                let u = self.universe()?;
                let (at_s, event_id) = orbits::campaign_now(&u.chronicle, &u.root);
                Response::CampaignNowIs {
                    at_s,
                    event_id,
                    state_hash: None,
                }
            }
            Request::Calendar => Response::CalendarIs {
                calendar: self
                    .observe
                    .calendar
                    .as_ref()
                    .map(|c| serde_json::to_value(c).expect("plain data")),
            },
            Request::Codes { scope, sheet, at_s } => {
                let scope: Address = scope
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let sheet = match &sheet {
                    Some(id) => self.codes.sheet(id).ok_or_else(|| {
                        err(
                            "unknown_sheet",
                            format!("no code sheet {id:?} in the stack"),
                        )
                    })?,
                    None => self.codes.sheets.first().ok_or_else(|| {
                        err(
                            "no_code_sheets",
                            "the pack stack defines no code sheets (provides.other.code_sheets)",
                        )
                    })?,
                };
                let u = self
                    .universe
                    .as_ref()
                    .ok_or_else(|| err("no_universe", "open a universe first"))?;
                let readings = codes_at(&self.codes, sheet, u, &scope, at_s);
                Response::Codes {
                    sheet: sheet.id.clone(),
                    readings,
                }
            }
            Request::ImportSector {
                text,
                format,
                sheet,
                apply,
                grow,
                at_s,
            } => {
                use sse_sector_io::Format;
                if grow && !apply {
                    return Err(err("bad_request", "grow needs apply"));
                }
                let file = match format.as_deref() {
                    None => sse_sector_io::parse_any(&text),
                    Some("t5_tab") => sse_sector_io::parse_as(Format::T5Tab, &text),
                    Some("sec") => sse_sector_io::parse_as(Format::Sec, &text),
                    Some("uwp_lines") => sse_sector_io::parse_as(Format::UwpLines, &text),
                    Some(f) => {
                        return Err(err(
                            "bad_format",
                            format!("format {f:?}: use t5_tab, sec or uwp_lines"),
                        ))
                    }
                };
                let sheet = match &sheet {
                    Some(id) => self.codes.sheet(id).ok_or_else(|| {
                        err(
                            "unknown_sheet",
                            format!("no code sheet {id:?} in the stack"),
                        )
                    })?,
                    None => self
                        .codes
                        .sheets
                        .iter()
                        .find(|s| s.import.is_some())
                        .ok_or_else(|| {
                            err(
                                "no_code_sheets",
                                "no code sheet in the stack reads sector files",
                            )
                        })?,
                };
                let codes = &self.codes;
                let u = self
                    .universe
                    .as_mut()
                    .ok_or_else(|| err("no_universe", "open a universe first"))?;
                let imp = sse_codes::import::import(codes, sheet, &file, &u.root)
                    .map_err(|e| err("bad_import", e))?;
                let mut events = Vec::new();
                let mut imp = imp;
                if apply {
                    let t = UniverseTime::seconds(at_s);
                    for w in imp.worlds.iter_mut().filter(|w| w.plan.is_some()) {
                        let sys: Address = w.system.parse().expect("import addresses parse");
                        if grow {
                            let plan = w.plan.clone().expect("filtered");
                            let g = sse_codes::plan::grow(plan, sys.clone(), 6, 2)
                                .map_err(|e| err("formation_failed", e))?;
                            let req: FormationRequest = serde_json::from_value(serde_json::json!({
                                "seed": g.rerun.seed,
                                "system": sys.to_string(),
                                "star_pins": g.rerun.star_pins,
                                "pins": g.rerun.pins,
                            }))
                            .expect("a formation request");
                            let a =
                                adopt::adopt(&u.chronicle, &u.root, None, Some(req), Some(at_s))?;
                            for e in a.events {
                                events.push(
                                    u.chronicle
                                        .append(e)
                                        .map_err(|e| err("chronicle", format!("{e}")))?
                                        .hex(),
                                );
                            }
                            if let Some((s, run)) = a.run {
                                u.runs.insert(s, run);
                            }
                            w.main_world = g.main_world.to_string();
                            w.guessed.extend(g.unmet.iter().cloned());
                        }
                        let main: Address = w.main_world.parse().expect("import addresses parse");
                        for impossible in [false, true] {
                            let mut set: BTreeMap<Address, BTreeMap<String, Value>> =
                                BTreeMap::new();
                            for (a, list) in [(&sys, &w.system_edits), (&main, &w.world_edits)] {
                                for e in list.iter().filter(|e| e.impossible == impossible) {
                                    set.entry(a.clone())
                                        .or_default()
                                        .insert(e.field.clone(), e.value.clone());
                                }
                            }
                            if set.is_empty() {
                                continue;
                            }
                            let e = Event::new(EventBody {
                                kind: kinds::AUTHORED_SET.into(),
                                subject: set.keys().cloned().collect(),
                                occurred_at: t,
                                activated_at: t,
                                recognised_at: u.chronicle.next_stamp(),
                                causes: vec![],
                                payload: Payload {
                                    set,
                                    ..Default::default()
                                },
                                evidence: if impossible {
                                    Evidence::AuthoredImpossible
                                } else {
                                    Evidence::Authored
                                },
                                publisher: Publisher::new("gm", "import", "1"),
                            })
                            .map_err(|e| err("bad_event", format!("{e}")))?;
                            let id = u
                                .chronicle
                                .append(e)
                                .map_err(|e| err("chronicle", format!("{e}")))?;
                            events.push(id.hex());
                        }
                    }
                }
                let state_hash = apply.then(|| {
                    u.chronicle
                        .state_at(UniverseTime::seconds(i64::MAX))
                        .hash()
                        .hex()
                });
                Response::SectorImport {
                    summary: imp.summary.sentence(),
                    import: serde_json::to_value(&imp).expect("imports are plain data"),
                    events,
                    state_hash,
                }
            }
            Request::ExportSector {
                scope,
                format,
                sheet,
                at_s,
            } => {
                use sse_sector_io::Format;
                let fmt = match format.as_str() {
                    "t5_tab" => Format::T5Tab,
                    "sec" => Format::Sec,
                    "uwp_lines" => Format::UwpLines,
                    f => {
                        return Err(err(
                            "bad_format",
                            format!("format {f:?}: use t5_tab, sec or uwp_lines"),
                        ))
                    }
                };
                let scope: Address = scope
                    .parse()
                    .map_err(|e| err("bad_address", format!("{e}")))?;
                let sheet = match &sheet {
                    Some(id) => self.codes.sheet(id).ok_or_else(|| {
                        err(
                            "unknown_sheet",
                            format!("no code sheet {id:?} in the stack"),
                        )
                    })?,
                    None => self
                        .codes
                        .sheets
                        .iter()
                        .find(|s| s.import.is_some())
                        .ok_or_else(|| {
                            err(
                                "no_code_sheets",
                                "no code sheet in the stack writes sector files",
                            )
                        })?,
                };
                let hints = sheet.import.clone().expect("chosen for its import hints");
                let u = self
                    .universe
                    .as_ref()
                    .ok_or_else(|| err("no_universe", "open a universe first"))?;
                let (file, skipped) = export_at(&self.codes, sheet, &hints, u, &scope, at_s);
                Response::SectorExport {
                    text: sse_sector_io::write(fmt, &file),
                    written: file.worlds.len() as u32,
                    skipped,
                }
            }
            Request::HaloCell {
                cell,
                min_mass_msun,
                limit,
            } => {
                let u = self.universe()?;
                let cell = cosmos::halo_cell(cosmos::CellAsk {
                    seed: &u.seed,
                    root: &u.root,
                    cell: cell.unwrap_or([0, 0, 0]),
                    min_mass: min_mass_msun.unwrap_or(1.0e10),
                    limit: limit.unwrap_or(300) as usize,
                })?;
                Response::HaloCelled { cell }
            }
            Request::StarCell { cell, t_myr } => {
                let u = self.universe()?;
                let pins =
                    cosmos::star_pins(&u.chronicle.state_at(UniverseTime::seconds(i64::MAX)))?;
                let elapsed = clock::elapsed_myr(&u.chronicle);
                let cell = cosmos::with_stars(&u.seed, &u.root, None, |st, now| {
                    cosmos::star_cell(st, cell, t_myr.unwrap_or(now + elapsed), &pins)
                })?;
                Response::StarCelled { cell }
            }
            Request::PromoteStar { address } => self.promote_star(&address)?,
            Request::ReadMatter { table, id } => {
                self.read_matter(table.as_deref(), id.as_deref())?
            }
            Request::Dressing { scope, at_s } => self.dressing(&scope, at_s.unwrap_or(i64::MAX))?,
            Request::AdoptDressing {
                candidate,
                name,
                at_s,
            } => self.adopt_dressing(&candidate, name.as_deref(), at_s)?,
            Request::HoldDressing { candidate, held } => {
                self.hold_dressing(&candidate, held.unwrap_or(true))?
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

/// The system a body belongs to: its nearest `sys` ancestor (itself if it is
/// one), or its parent when there is none.
fn system_of(a: &Address) -> Option<Address> {
    let mut at = Some(a.clone());
    while let Some(x) = at {
        if x.last().kind() == "sys" {
            return Some(x);
        }
        at = x.parent();
    }
    a.parent()
}

/// `codes`: a reading per body at or under `scope`, each against the bodies
/// of its own system. Authored tags (`tag.*` facts) become subject tags, as
/// in scenes.
fn codes_at(
    codes: &sse_codes::Codes,
    sheet: &sse_codes::CodeSheet,
    u: &Universe,
    scope: &Address,
    at_s: i64,
) -> Vec<serde_json::Value> {
    let state = u.chronicle.state_at(UniverseTime::seconds(at_s));
    let subject = |a: &Address, f: &BTreeMap<String, Value>| {
        let mut facts = BTreeMap::new();
        let mut tags = std::collections::BTreeSet::new();
        for (k, v) in f {
            match k.strip_prefix(TAG_FIELD) {
                Some(t) if *v == Value::Bool(true) => {
                    tags.insert(t.to_string());
                }
                Some(_) => {}
                None => {
                    facts.insert(k.clone(), v.clone());
                }
            }
        }
        sse_sift::Subject {
            address: Some(a.clone()),
            facts,
            parent: a
                .parent()
                .and_then(|p| state.facts.get(&p).cloned())
                .unwrap_or_default(),
            tags,
            now_s: at_s,
        }
    };
    let mut systems: BTreeMap<Address, Vec<sse_sift::Subject>> = BTreeMap::new();
    for (a, f) in &state.facts {
        if let Some(s) = system_of(a) {
            if a != &s {
                systems.entry(s).or_default().push(subject(a, f));
            }
        }
    }
    let mut out = Vec::new();
    for (a, f) in &state.facts {
        if !scope.contains(a) || a.last().kind() == "sys" {
            continue;
        }
        let sys = system_of(a).and_then(|s| systems.get(&s));
        let r = codes.read(sheet, &subject(a, f), sys.map(Vec::as_slice));
        out.push(serde_json::json!({"address": a.to_string(), "reading": r}));
    }
    out
}

/// `export_sector`: one world line per mapped system under `scope`. The
/// file takes its sector name and abbreviation from the first system that
/// has them (a scope spanning two sectors writes one file).
fn export_at(
    codes: &sse_codes::Codes,
    sheet: &sse_codes::CodeSheet,
    hints: &sse_codes::import::ImportHints,
    u: &Universe,
    scope: &Address,
    at_s: i64,
) -> (sse_sector_io::SectorFile, Vec<String>) {
    let state = u.chronicle.state_at(UniverseTime::seconds(at_s));
    let mut file = sse_sector_io::SectorFile::default();
    let mut skipped = Vec::new();
    for (sys, sf) in state
        .facts
        .iter()
        .filter(|(a, f)| scope.contains(a) && a.last().kind() == "sys" && f.contains_key("map.hex"))
    {
        let mut bodies: Vec<sse_sift::Subject> = Vec::new();
        let mut main: Option<usize> = None;
        for (a, f) in state
            .facts
            .iter()
            .filter(|(a, _)| sys.contains(a) && *a != sys)
        {
            let mut facts = BTreeMap::new();
            let mut tags = std::collections::BTreeSet::new();
            for (k, v) in f {
                match k.strip_prefix(TAG_FIELD) {
                    Some(t) if *v == Value::Bool(true) => {
                        tags.insert(t.to_string());
                    }
                    Some(_) => {}
                    None => {
                        facts.insert(k.clone(), v.clone());
                    }
                }
            }
            if main.is_none()
                && (facts.contains_key("provenance.code") || tags.contains("main_world"))
            {
                main = Some(bodies.len());
            }
            bodies.push(sse_sift::Subject {
                address: Some(a.clone()),
                parent: a
                    .parent()
                    .and_then(|p| state.facts.get(&p).cloned())
                    .unwrap_or_default(),
                facts,
                tags,
                now_s: at_s,
            });
        }
        let text = |k: &str| match sf.get(k) {
            Some(Value::Text(t)) => Some(t.clone()),
            _ => None,
        };
        file.name = file.name.take().or_else(|| text("map.sector"));
        file.abbreviation = file.abbreviation.take().or_else(|| text("map.sector_abbr"));
        let Some(m) = main else {
            skipped.push(sys.to_string());
            continue;
        };
        let r = codes.read(sheet, &bodies[m], Some(&bodies));
        file.worlds.push(sse_codes::import::world_line(
            hints,
            &r,
            &bodies[m].facts,
            sf,
        ));
    }
    file.worlds.sort_by(|a, b| a.hex.cmp(&b.hex));
    (file, skipped)
}
