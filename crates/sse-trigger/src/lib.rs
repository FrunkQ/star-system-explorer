//! Triggers, effects, the gate and approval policies (record design
//! section 5, track J4).
//!
//! A **trigger** says when: `on_event` (an event of a kind arrives for a
//! subject), `on_condition` (a condition becomes true, and has stayed true
//! for `hold_s`, with hysteresis so it fires once), `on_time` (every
//! interval) or `on_intent` (an intent of a kind, `intent.<kind>` in the
//! record). An **effect** proposes a change from a closed list: `set_kept`,
//! `add_note`, `grant`, `tag`, `untag`, `send_intent`, `set_pin` (`create`
//! and `retire` come with templates, J7).
//!
//! Effects never touch state directly. Every one passes **one gate**
//! ([`Rules::step`] and [`Rules::decide`]) on the GM's machine, under the
//! rule's approval policy (`auto`, `ask`, `ask_if`; the GM may override it
//! per rule), and becomes an ordinary event whose publisher names the pack,
//! rule and version. Record-side effects cite the firing as their cause;
//! universe-side ones (grants, tags, pins) cite nothing secret, and the GM's
//! record holds `rules.applied` linking the firing to them, so "why did that
//! happen?" always has an answer and no player stream carries a hidden id.
//!
//! The runtime state (which conditions are armed, since when, what fired,
//! what waits for approval) is itself record events (`rules.*`), so the
//! whole thing is a fold over the two chronicles: a run replays
//! bit-identically. Nothing here runs on a player device (design 5.4, 8A).
#![forbid(unsafe_code)]

pub mod intents;
pub mod programs;
pub mod split;
pub mod templates;

use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use sse_chronicle::{
    kinds as ck, Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use sse_record::{Audience, KeptSet, NoteDraft, Registry};
use sse_sift::predicate::{as_num, Predicate, Subject};
use std::collections::{BTreeMap, BTreeSet};

/// Record event kinds written by the gate.
pub mod kinds {
    pub const BOUNDARY: &str = "rules.boundary";
    pub const ARMED: &str = "rules.armed";
    pub const DISARMED: &str = "rules.disarmed";
    pub const FIRED: &str = "rules.fired";
    pub const PROPOSED: &str = "rules.proposed";
    pub const DECIDED: &str = "rules.decided";
    pub const APPLIED: &str = "rules.applied";
    pub const POLICY: &str = "rules.policy";
    pub const LIMIT: &str = "rules.limit";
    /// An intent sent by an effect or a seat: `intent.<kind>`.
    pub const INTENT_PREFIX: &str = "intent.";
}

/// Universe fields: `tag.<name>` = true tags a subject (as the kernel reads it).
pub const TAG_FIELD: &str = "tag.";
/// Passes at one time boundary before the gate stops and reports the chain.
pub const DEFAULT_PASS_LIMIT: u32 = 8;
/// Ticks of one `on_time` rule in one step, at most (a step over a long gap).
pub const MAX_TICKS: i64 = 10_000;

/// When a rule fires.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum When {
    /// An event of this kind (`*` at the end matches a prefix) becomes active
    /// for a subject in scope. `tag` narrows to notes carrying that tag.
    OnEvent {
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
    },
    /// A condition becomes true for a subject in scope and stays true for `hold_s`.
    OnCondition {
        condition: Predicate,
        #[serde(default)]
        hold_s: i64,
    },
    /// Every `every_s` from `from_s`, once per tick, for each subject in scope
    /// (the universe root when the scope names none).
    OnTime {
        every_s: i64,
        #[serde(default)]
        from_s: i64,
    },
    /// An intent of this kind arrives (before any approval of the intent itself).
    OnIntent { kind: String },
}

/// Which subjects a rule watches.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// Only subjects carrying this tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Only these addresses (none: any).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<String>,
    /// Only subjects at or under this address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub under: Option<String>,
    /// Watch pairs: each subject above with each observer carrying this tag.
    /// The condition reads the subject's facts as they are, the observer's as
    /// `observer.<field>`, and `pair.distance_m` from both positions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairs: Option<Pairs>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairs {
    pub observer_tag: String,
    /// The position fields (metres) on both sides, for `pair.distance_m`.
    #[serde(default = "position_fields")]
    pub position: [String; 3],
}

fn position_fields() -> [String; 3] {
    ["pos.x_m".into(), "pos.y_m".into(), "pos.z_m".into()]
}

/// How a firing's effects pass the gate.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "policy")]
pub enum Approval {
    /// Applied at once.
    Auto,
    /// Queued for the GM with its explanation; applied on yes ("umpire assist").
    Ask,
    /// Auto unless the condition holds for the subject (ask before anything is destroyed).
    AskIf { condition: Predicate },
}

/// One proposed change (the closed list of design 5.2).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "effect")]
pub enum Effect {
    /// Set a kept value (`value`), or move a number by `add` (clamped to the
    /// kind's range). The writer is `effect.<rule id>`, which the kind must list.
    SetKept {
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<J>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        add: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Write a note at the subject. `{subject}` and `{observer}` in the text
    /// are replaced by the addresses.
    AddNote {
        text: String,
        #[serde(default)]
        audience: Audience,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tier: Option<String>,
    },
    /// Grant a data view of the subject to an audience: `view`, or the next
    /// rung of `ladder` above the audience's present view there. On a pair
    /// rule, `audience` may be `{observer.<field>}`, the observer's text fact
    /// (for example `{observer.crew.audience}`), so each crew learns for itself.
    Grant {
        audience: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        ladder: Vec<String>,
    },
    Tag {
        tag: String,
    },
    Untag {
        tag: String,
    },
    /// Send an intent (`intent.<kind>` in the record; `on_intent` rules see it).
    SendIntent {
        kind: String,
        #[serde(default)]
        payload: J,
    },
    /// The one bridge to physics (rule 2.2.1): an authored fact on the subject.
    SetPin {
        field: String,
        value: J,
    },
    /// Make an authored object from a template (J7), under the subject:
    /// `<subject>/<kind>:<key>` (the next free number when no key).
    Create {
        template: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Retire the subject, an object made from a template (J7).
    Retire {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

impl Effect {
    fn name(&self) -> &'static str {
        match self {
            Effect::SetKept { .. } => "set_kept",
            Effect::AddNote { .. } => "add_note",
            Effect::Grant { .. } => "grant",
            Effect::Tag { .. } => "tag",
            Effect::Untag { .. } => "untag",
            Effect::SendIntent { .. } => "send_intent",
            Effect::SetPin { .. } => "set_pin",
            Effect::Create { .. } => "create",
            Effect::Retire { .. } => "retire",
        }
    }
    /// The design's default: ask for create, retire and set_pin; auto otherwise.
    fn asks_by_default(&self) -> bool {
        matches!(
            self,
            Effect::Create { .. } | Effect::Retire { .. } | Effect::SetPin { .. }
        )
    }
}

/// A rule as a pack declares it (`provides.other.triggers.rules`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleDoc {
    pub id: String,
    /// Rules run in phase order, then id order (design 5.4).
    #[serde(default)]
    pub phase: i32,
    pub when: When,
    #[serde(default)]
    pub scope: Scope,
    pub effects: Vec<Effect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<Approval>,
    /// Shown with a proposal and in "why?".
    #[serde(default)]
    pub explain: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesPack {
    #[serde(default)]
    rules: Vec<RuleDoc>,
    #[serde(default)]
    pass_limit: Option<u32>,
}

/// A loaded rule and the pack that declares it.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Rule {
    pub pack: String,
    pub version: String,
    #[serde(flatten)]
    pub doc: RuleDoc,
}

/// The stack's rules, in run order.
#[derive(Clone, PartialEq, Debug, Default, Serialize)]
pub struct Rules {
    pub rules: Vec<Rule>,
    pub pass_limit: u32,
}

fn ident_ok(s: &str) -> bool {
    !s.is_empty()
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-.".contains(&c))
}

/// Where each record or universe write lands, for the report.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Written {
    /// `record` or `universe`.
    pub chronicle: String,
    pub event: String,
    pub effect: String,
}

/// One firing.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Firing {
    pub rule: String,
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observer: Option<String>,
    /// The event that caused it (`on_event`, `on_intent`), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// The `rules.fired` event in the record.
    pub fired: String,
    /// `applied`, `proposed` (waits for the GM) or `refused: <why>`.
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub written: Vec<Written>,
}

/// What one time boundary did.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct StepReport {
    pub at_s: i64,
    /// Firings per pass.
    pub passes: Vec<Vec<Firing>>,
    /// Set when the pass limit stopped the boundary: the rules firing in each pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<Vec<Vec<String>>>,
}

/// A proposal waiting for the GM.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub rule: String,
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observer: Option<String>,
    pub at_s: i64,
    pub effects: Vec<Effect>,
    pub explain: String,
}

/// Everything the gate needs to read and write.
pub struct Gate<'a> {
    pub universe: &'a mut Chronicle,
    pub record: &'a mut Chronicle,
    pub registry: &'a Registry,
    pub campaign: Option<&'a sse_observe::Campaign>,
    pub root: &'a Address,
    /// The stack's templates (J7), for `create`.
    pub templates: &'a templates::Templates,
}

fn text(s: &str) -> Value {
    Value::Text(s.into())
}

fn data_text<'a>(e: &'a Event, k: &str) -> Option<&'a str> {
    match &e.body.payload.data {
        Some(Value::Map(m)) => match m.get(k) {
            Some(Value::Text(t)) => Some(t),
            _ => None,
        },
        _ => None,
    }
}

fn data_int(e: &Event, k: &str) -> Option<i64> {
    match &e.body.payload.data {
        Some(Value::Map(m)) => match m.get(k) {
            Some(Value::Int(i)) => i64::try_from(*i).ok(),
            _ => None,
        },
        _ => None,
    }
}

fn json_value(v: &J) -> Result<Value, String> {
    serde_json::from_value::<Value>(v.clone()).map_err(|e| e.to_string())
}

/// The world a rule reads at a time: universe facts (tags taken out of
/// `tag.<name>`) with the record's kept values merged in, per address.
pub fn subjects(
    universe: &Chronicle,
    record: &Chronicle,
    root: &Address,
    at: UniverseTime,
) -> BTreeMap<Address, Subject> {
    let u = universe.state_at(at);
    let r = record.state_at(at);
    let mut facts: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
    let mut tags: BTreeMap<Address, BTreeSet<String>> = BTreeMap::new();
    for (a, f) in &u.facts {
        if !root.contains(a) {
            continue;
        }
        for (k, v) in f {
            if let Some(t) = k.strip_prefix(TAG_FIELD) {
                if *v == Value::Bool(true) {
                    tags.entry(a.clone()).or_default().insert(t.to_string());
                }
            } else {
                facts
                    .entry(a.clone())
                    .or_default()
                    .insert(k.clone(), v.clone());
            }
        }
    }
    for (a, f) in &r.facts {
        if root.contains(a) {
            let m = facts.entry(a.clone()).or_default();
            for (k, v) in f {
                m.insert(k.clone(), v.clone());
            }
        }
    }
    for a in tags.keys() {
        facts.entry(a.clone()).or_default();
    }
    facts
        .iter()
        .map(|(a, f)| {
            (
                a.clone(),
                Subject {
                    address: Some(a.clone()),
                    facts: f.clone(),
                    parent: a
                        .parent()
                        .and_then(|p| facts.get(&p).cloned())
                        .unwrap_or_default(),
                    tags: tags.get(a).cloned().unwrap_or_default(),
                    now_s: at.s,
                },
            )
        })
        .collect()
}

impl Scope {
    fn admits(&self, a: &Address, s: Option<&Subject>) -> bool {
        if !self.addresses.is_empty() && !self.addresses.iter().any(|x| *x == a.to_string()) {
            return false;
        }
        if let Some(u) = &self.under {
            match u.parse::<Address>() {
                Ok(u) if u.contains(a) => {}
                _ => return false,
            }
        }
        if let Some(t) = &self.tag {
            if !s.is_some_and(|s| s.tags.contains(t)) {
                return false;
            }
        }
        true
    }
}

/// A subject (or pair) a rule looks at.
struct Watched {
    subject: Address,
    observer: Option<Address>,
    view: Subject,
}

impl Watched {
    fn key(&self) -> String {
        match &self.observer {
            Some(o) => format!("{}|{o}", self.subject),
            None => self.subject.to_string(),
        }
    }
}

fn watched(scope: &Scope, world: &BTreeMap<Address, Subject>) -> Vec<Watched> {
    let singles: Vec<(&Address, &Subject)> = world
        .iter()
        .filter(|(a, s)| scope.admits(a, Some(s)))
        .collect();
    let Some(p) = &scope.pairs else {
        return singles
            .into_iter()
            .map(|(a, s)| Watched {
                subject: a.clone(),
                observer: None,
                view: s.clone(),
            })
            .collect();
    };
    let mut out = vec![];
    for (oa, os) in world
        .iter()
        .filter(|(_, s)| s.tags.contains(&p.observer_tag))
    {
        for (a, s) in &singles {
            if *a == oa {
                continue;
            }
            let mut v = (*s).clone();
            for (k, x) in &os.facts {
                v.facts.insert(format!("observer.{k}"), x.clone());
            }
            let pos = |s: &Subject| -> Option<[f64; 3]> {
                let g = |k: &str| s.facts.get(k).and_then(as_num);
                Some([g(&p.position[0])?, g(&p.position[1])?, g(&p.position[2])?])
            };
            if let (Some(x), Some(y)) = (pos(s), pos(os)) {
                let d = [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
                let r = sse_core::math::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
                v.facts.insert("pair.distance_m".into(), Value::Float(r));
            }
            out.push(Watched {
                subject: (*a).clone(),
                observer: Some(oa.clone()),
                view: v,
            });
        }
    }
    out
}

/// What the record says about one rule and watched key: armed since when, fired since.
#[derive(Default, Clone)]
struct Arm {
    since: Option<i64>,
    fired: bool,
}

struct Pending {
    rule: usize,
    subject: Address,
    observer: Option<Address>,
    /// (in the record?, id) of the causing event.
    cause: Option<(bool, ContentHash)>,
    tick: Option<i64>,
}

impl Rules {
    /// Templates the rules' `create` effects name that the stack lacks.
    pub fn check_templates(&self, t: &templates::Templates) -> Vec<String> {
        let mut out = vec![];
        for r in &self.rules {
            for e in &r.doc.effects {
                if let Effect::Create { template, .. } = e {
                    if t.get(template).is_none() {
                        out.push(format!(
                            "rule {}: create names template {template}, which no pack declares",
                            r.doc.id
                        ));
                    }
                }
            }
        }
        out
    }

    /// Intent kinds the rules listen for (`on_intent`) or send (`send_intent`): `(rule, kind)`.
    pub fn intent_kinds(&self) -> Vec<(String, String)> {
        let mut out = vec![];
        for r in &self.rules {
            if let When::OnIntent { kind } = &r.doc.when {
                out.push((r.doc.id.clone(), kind.clone()));
            }
            for e in &r.doc.effects {
                if let Effect::SendIntent { kind, .. } = e {
                    out.push((r.doc.id.clone(), kind.clone()));
                }
            }
        }
        out
    }

    /// Read every pack's `provides.other.triggers` (bottom to top). A rule id
    /// declared twice is refused; so are effects the gate cannot carry out.
    pub fn from_packs(packs: &[(String, String, J)]) -> Result<Rules, Vec<String>> {
        let mut rules: Vec<Rule> = vec![];
        let mut problems = vec![];
        let mut limit = DEFAULT_PASS_LIMIT;
        for (pack, version, v) in packs {
            let p: RulesPack = match serde_json::from_value(v.clone()) {
                Ok(p) => p,
                Err(e) => {
                    problems.push(format!("{pack}: triggers: {e}"));
                    continue;
                }
            };
            if let Some(l) = p.pass_limit {
                limit = l.clamp(1, 64);
            }
            for d in p.rules {
                if !ident_ok(&d.id) {
                    problems.push(format!(
                        "{pack}: rule {:?}: ids are lowercase letters, digits and _ - .",
                        d.id
                    ));
                }
                if let Some(r) = rules.iter().find(|r| r.doc.id == d.id) {
                    problems.push(format!(
                        "{pack}: rule {} is already declared by {}",
                        d.id, r.pack
                    ));
                }
                if d.effects.is_empty() {
                    problems.push(format!("{pack}: rule {}: no effects", d.id));
                }
                for e in &d.effects {
                    if let Effect::Grant {
                        view,
                        ladder,
                        audience,
                    } = e
                    {
                        if audience.contains('{') {
                            let ok = observer_field(audience).is_some_and(|f| !f.is_empty());
                            if !ok || d.scope.pairs.is_none() {
                                problems.push(format!(
                                    "{pack}: rule {}: a grant audience is a name, or {{observer.<field>}} on a pair rule",
                                    d.id
                                ));
                            }
                        }
                        if view.is_none() == ladder.is_empty() {
                            problems.push(format!(
                                "{pack}: rule {}: a grant names a view or a ladder, not both or neither",
                                d.id
                            ));
                        }
                    }
                    if let Effect::SetKept { value, add, .. } = e {
                        if value.is_some() == add.is_some() {
                            problems.push(format!(
                                "{pack}: rule {}: set_kept takes a value or an add, not both or neither",
                                d.id
                            ));
                        }
                    }
                }
                match &d.when {
                    When::OnTime { every_s, .. } if *every_s <= 0 => {
                        problems.push(format!("{pack}: rule {}: every_s must be positive", d.id))
                    }
                    When::OnCondition { hold_s, .. } if *hold_s < 0 => problems.push(format!(
                        "{pack}: rule {}: hold_s must not be negative",
                        d.id
                    )),
                    _ => {}
                }
                rules.push(Rule {
                    pack: pack.clone(),
                    version: version.clone(),
                    doc: d,
                });
            }
        }
        if !problems.is_empty() {
            return Err(problems);
        }
        rules.sort_by(|a, b| {
            a.doc
                .phase
                .cmp(&b.doc.phase)
                .then_with(|| a.doc.id.cmp(&b.doc.id))
        });
        Ok(Rules {
            rules,
            pass_limit: limit,
        })
    }

    /// Check the rules against the record's kept kinds and the data views:
    /// every `set_kept` kind exists and lists `effect.<rule>` as a writer;
    /// every granted view exists.
    pub fn check(&self, reg: &Registry, views: &[String]) -> Vec<String> {
        let mut out = vec![];
        for r in &self.rules {
            for e in &r.doc.effects {
                match e {
                    Effect::SetKept { kind, .. } if !kind.starts_with(sse_record::MISC_PREFIX) => {
                        match reg.get(kind) {
                            None => out.push(format!(
                                "{}: rule {}: no kept kind {kind}",
                                r.pack, r.doc.id
                            )),
                            Some(k) => {
                                let w = format!("effect.{}", r.doc.id);
                                if !k.writers.contains(&w) {
                                    out.push(format!(
                                        "{}: rule {}: kept kind {kind} does not list writer {w}",
                                        r.pack, r.doc.id
                                    ));
                                }
                            }
                        }
                    }
                    Effect::Grant { view, ladder, .. } => {
                        for v in view.iter().chain(ladder) {
                            if !views.iter().any(|x| x == v) {
                                out.push(format!(
                                    "{}: rule {}: no data view {v}",
                                    r.pack, r.doc.id
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        out
    }

    fn find(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.doc.id == id)
    }

    /// The GM's policy override for a rule, if any (latest `rules.policy` wins).
    fn policy_override(record: &Chronicle, rule: &str) -> Option<Approval> {
        record
            .events()
            .iter()
            .rev()
            .filter(|e| e.body.kind == kinds::POLICY && data_text(e, "rule") == Some(rule))
            .find_map(|e| data_text(e, "approval").and_then(|t| serde_json::from_str(t).ok()))
    }

    /// The last boundary stepped, if any.
    pub fn last_boundary(record: &Chronicle) -> Option<i64> {
        record
            .events()
            .iter()
            .rev()
            .find(|e| e.body.kind == kinds::BOUNDARY)
            .map(|e| e.body.occurred_at.s)
    }

    fn arms(record: &Chronicle) -> BTreeMap<(String, String), Arm> {
        let mut m: BTreeMap<(String, String), Arm> = BTreeMap::new();
        for e in record.events() {
            let k = &e.body.kind;
            if k != kinds::ARMED && k != kinds::DISARMED && k != kinds::FIRED {
                continue;
            }
            let (Some(r), Some(w)) = (data_text(e, "rule"), data_text(e, "key")) else {
                continue;
            };
            let a = m.entry((r.to_string(), w.to_string())).or_default();
            match k.as_str() {
                kinds::ARMED => {
                    *a = Arm {
                        since: Some(e.body.occurred_at.s),
                        fired: false,
                    }
                }
                kinds::DISARMED => *a = Arm::default(),
                _ => a.fired = true,
            }
        }
        m
    }

    fn write(
        record: &mut Chronicle,
        kind: &str,
        subject: &Address,
        at: UniverseTime,
        causes: Vec<ContentHash>,
        data: BTreeMap<String, Value>,
        publisher: Publisher,
    ) -> Result<ContentHash, String> {
        let e = Event::new(EventBody {
            kind: kind.into(),
            subject: vec![subject.clone()],
            occurred_at: at,
            activated_at: at,
            recognised_at: record.next_stamp(),
            causes,
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Derived,
            publisher,
        })
        .map_err(|e| e.to_string())?;
        record.append(e).map_err(|e| e.to_string())
    }

    fn publisher(r: &Rule) -> Publisher {
        Publisher::new(&r.pack, &r.doc.id, &r.version)
    }

    /// Step to a time boundary: fire what is due, pass each firing's effects
    /// through the gate, and run further passes for what those effects set
    /// off, up to the pass limit (then report the chain and stop).
    pub fn step(&self, g: &mut Gate, at_s: i64) -> Result<StepReport, String> {
        let prev = Self::last_boundary(g.record);
        if let Some(p) = prev {
            if at_s < p {
                return Err(format!(
                    "time boundaries run forwards: the last was {p} s, this is {at_s} s"
                ));
            }
        }
        let at = UniverseTime::seconds(at_s);
        let gm = Publisher::new("sse.rules", "gate", "1");
        let mut data = BTreeMap::new();
        data.insert("at_s".into(), Value::Int(at_s.into()));
        let boundary = Self::write(g.record, kinds::BOUNDARY, g.root, at, vec![], data, gm)?;
        let mut report = StepReport {
            at_s,
            ..Default::default()
        };
        // Events that can set off on_event and on_intent rules in the first pass:
        // active in (prev, at]; later passes see what the pass before wrote.
        let mut window_u: Vec<usize> = (0..g.universe.events().len())
            .filter(|&i| {
                let t = g.universe.events()[i].body.activated_at.s;
                t <= at_s && prev.is_none_or(|p| t > p)
            })
            .collect();
        let mut window_r: Vec<usize> = (0..g.record.events().len())
            .filter(|&i| {
                let e = &g.record.events()[i];
                let t = e.body.activated_at.s;
                e.id != boundary && t <= at_s && prev.is_none_or(|p| t > p)
            })
            .collect();
        let mut chain = vec![];
        for pass in 0..=self.pass_limit {
            let (u0, r0) = (g.universe.events().len(), g.record.events().len());
            let due = self.due(g, at, prev, pass == 0, &window_u, &window_r)?;
            if due.is_empty() {
                break;
            }
            let names: Vec<String> = due
                .iter()
                .map(|d| self.rules[d.rule].doc.id.clone())
                .collect();
            chain.push(names);
            if pass == self.pass_limit {
                let mut data = BTreeMap::new();
                data.insert(
                    "chain".into(),
                    Value::Text(serde_json::to_string(&chain).expect("plain")),
                );
                Self::write(
                    g.record,
                    kinds::LIMIT,
                    g.root,
                    at,
                    vec![],
                    data,
                    Publisher::new("sse.rules", "gate", "1"),
                )?;
                report.limit = Some(chain);
                break;
            }
            let mut fired = vec![];
            for d in due {
                fired.push(self.fire(g, at, pass, d)?);
            }
            report.passes.push(fired);
            window_u = (u0..g.universe.events().len()).collect();
            window_r = (r0..g.record.events().len())
                .filter(|&i| !g.record.events()[i].body.kind.starts_with("rules."))
                .collect();
        }
        Ok(report)
    }

    /// Everything due in one pass, in run order. Arming and disarming are
    /// written as they are seen (bookkeeping, not effects).
    fn due(
        &self,
        g: &mut Gate,
        at: UniverseTime,
        prev: Option<i64>,
        first: bool,
        window_u: &[usize],
        window_r: &[usize],
    ) -> Result<Vec<Pending>, String> {
        let world = subjects(g.universe, g.record, g.root, at);
        let arms = Self::arms(g.record);
        let mut out = vec![];
        for (ri, r) in self.rules.iter().enumerate() {
            match &r.doc.when {
                When::OnEvent { kind, tag } => {
                    let hits = |e: &Event| -> bool {
                        let k = &e.body.kind;
                        let kind_ok = match kind.strip_suffix('*') {
                            Some(p) => k.starts_with(p),
                            None => k == kind,
                        };
                        kind_ok
                            && !k.starts_with("rules.")
                            && tag.as_ref().is_none_or(|t| {
                                sse_record::note_of(e)
                                    .is_some_and(|n| n.tags.iter().any(|x| x == t))
                            })
                    };
                    self.event_hits(&mut out, ri, r, g, &world, window_u, window_r, hits);
                }
                When::OnIntent { kind } => {
                    let want = format!("{}{kind}", kinds::INTENT_PREFIX);
                    self.event_hits(&mut out, ri, r, g, &world, &[], window_r, |e| {
                        e.body.kind == want
                    });
                }
                When::OnTime { every_s, from_s } => {
                    if !first {
                        continue;
                    }
                    // Ticks from_s + k every_s in (prev, at]; with no previous boundary, [from_s, at].
                    let lo = match prev {
                        Some(p) if p >= *from_s => (p - from_s) / every_s + 1,
                        _ => 0,
                    };
                    let hi = if at.s >= *from_s {
                        (at.s - from_s) / every_s
                    } else {
                        -1
                    };
                    if hi - lo + 1 > MAX_TICKS {
                        return Err(format!(
                            "rule {}: {} ticks in one step; step more often",
                            r.doc.id,
                            hi - lo + 1
                        ));
                    }
                    let targets: Vec<Address> = if r.doc.scope == Scope::default() {
                        vec![g.root.clone()]
                    } else {
                        watched(&r.doc.scope, &world)
                            .into_iter()
                            .map(|w| w.subject)
                            .collect()
                    };
                    for k in lo..=hi {
                        for t in &targets {
                            out.push(Pending {
                                rule: ri,
                                subject: t.clone(),
                                observer: None,
                                cause: None,
                                tick: Some(from_s + k * every_s),
                            });
                        }
                    }
                }
                When::OnCondition { condition, hold_s } => {
                    for w in watched(&r.doc.scope, &world) {
                        let key = (r.doc.id.clone(), w.key());
                        let arm = arms.get(&key).cloned().unwrap_or_default();
                        let holds = condition.eval(&w.view, arm.since.is_some()).holds;
                        let mut data = BTreeMap::new();
                        data.insert("rule".into(), text(&r.doc.id));
                        data.insert("key".into(), text(&key.1));
                        let mut since = arm.since;
                        if holds && since.is_none() {
                            Self::write(
                                g.record,
                                kinds::ARMED,
                                &w.subject,
                                at,
                                vec![],
                                data,
                                Self::publisher(r),
                            )?;
                            since = Some(at.s);
                        } else if !holds && since.is_some() {
                            Self::write(
                                g.record,
                                kinds::DISARMED,
                                &w.subject,
                                at,
                                vec![],
                                data,
                                Self::publisher(r),
                            )?;
                            continue;
                        }
                        if let Some(s) = since {
                            if !arm.fired && at.s - s >= *hold_s {
                                out.push(Pending {
                                    rule: ri,
                                    subject: w.subject,
                                    observer: w.observer,
                                    cause: None,
                                    tick: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        // Rules already run in phase and id order; within a rule, by subject, observer, cause.
        out.sort_by(|a, b| {
            a.rule
                .cmp(&b.rule)
                .then_with(|| a.subject.cmp(&b.subject))
                .then_with(|| a.observer.cmp(&b.observer))
                .then_with(|| a.tick.cmp(&b.tick))
                .then_with(|| a.cause.map(|c| c.1).cmp(&b.cause.map(|c| c.1)))
        });
        Ok(out)
    }

    #[allow(clippy::too_many_arguments)]
    fn event_hits(
        &self,
        out: &mut Vec<Pending>,
        ri: usize,
        r: &Rule,
        g: &Gate,
        world: &BTreeMap<Address, Subject>,
        window_u: &[usize],
        window_r: &[usize],
        hits: impl Fn(&Event) -> bool,
    ) {
        let sources = window_u
            .iter()
            .map(|&i| (false, &g.universe.events()[i]))
            .chain(window_r.iter().map(|&i| (true, &g.record.events()[i])));
        for (in_record, e) in sources {
            if !hits(e) {
                continue;
            }
            let Some(s) = e.body.subject.first() else {
                continue;
            };
            if !g.root.contains(s) || !r.doc.scope.admits(s, world.get(s)) {
                continue;
            }
            out.push(Pending {
                rule: ri,
                subject: s.clone(),
                observer: None,
                cause: Some((in_record, e.id)),
                tick: None,
            });
        }
    }

    fn fire(
        &self,
        g: &mut Gate,
        at: UniverseTime,
        pass: u32,
        d: Pending,
    ) -> Result<Firing, String> {
        let r = &self.rules[d.rule];
        let key = match &d.observer {
            Some(o) => format!("{}|{o}", d.subject),
            None => d.subject.to_string(),
        };
        let mut data = BTreeMap::new();
        data.insert("rule".into(), text(&r.doc.id));
        data.insert("key".into(), text(&key));
        data.insert("pass".into(), Value::Int(pass.into()));
        if let Some(o) = &d.observer {
            data.insert("observer".into(), text(&o.to_string()));
        }
        if let Some(t) = d.tick {
            data.insert("tick_s".into(), Value::Int(t.into()));
        }
        if let Some((rec, c)) = d.cause {
            data.insert("cause".into(), text(&c.hex()));
            data.insert(
                "cause_in".into(),
                text(if rec { "record" } else { "universe" }),
            );
        }
        let causes = match d.cause {
            Some((true, c)) => vec![c],
            _ => vec![],
        };
        let fired = Self::write(
            g.record,
            kinds::FIRED,
            &d.subject,
            at,
            causes,
            data,
            Self::publisher(r),
        )?;
        // The intent behind an `on_intent` firing, for `{intent.<field>}` values.
        let intent_payload: Option<J> = match d.cause {
            Some((true, c)) => g
                .record
                .events()
                .iter()
                .find(|e| e.id == c && e.body.kind.starts_with(kinds::INTENT_PREFIX))
                .and_then(|e| data_text(e, "payload"))
                .and_then(|t| serde_json::from_str(t).ok()),
            _ => None,
        };
        // The observer's facts, for `{observer.<field>}` grant audiences.
        let observer_facts = match &d.observer {
            Some(o)
                if r.doc.effects.iter().any(
                    |e| matches!(e, Effect::Grant { audience, .. } if audience.contains('{')),
                ) =>
            {
                subjects(g.universe, g.record, g.root, at)
                    .remove(o)
                    .map(|s| s.facts)
                    .unwrap_or_default()
            }
            _ => BTreeMap::new(),
        };
        let effects: Vec<Effect> = r
            .doc
            .effects
            .iter()
            .map(|e| {
                let e = substitute(e, &d.subject, d.observer.as_ref(), &observer_facts);
                match &intent_payload {
                    Some(p) => with_intent(&e, p),
                    None => e,
                }
            })
            .collect();
        let policy = Self::policy_override(g.record, &r.doc.id)
            .or_else(|| r.doc.approval.clone())
            .unwrap_or_else(|| {
                if effects.iter().any(Effect::asks_by_default) {
                    Approval::Ask
                } else {
                    Approval::Auto
                }
            });
        let ask = match &policy {
            Approval::Auto => false,
            Approval::Ask => true,
            Approval::AskIf { condition } => {
                let world = subjects(g.universe, g.record, g.root, at);
                world
                    .get(&d.subject)
                    .is_some_and(|s| condition.eval(s, false).holds)
            }
        };
        let mut firing = Firing {
            rule: r.doc.id.clone(),
            subject: d.subject.to_string(),
            observer: d.observer.as_ref().map(|o| o.to_string()),
            cause: d.cause.map(|c| c.1.hex()),
            fired: fired.hex(),
            outcome: String::new(),
            written: vec![],
        };
        if ask {
            let mut data = BTreeMap::new();
            data.insert("rule".into(), text(&r.doc.id));
            data.insert(
                "effects".into(),
                Value::Text(serde_json::to_string(&effects).expect("plain")),
            );
            if let Some(o) = &d.observer {
                data.insert("observer".into(), text(&o.to_string()));
            }
            data.insert("explain".into(), text(&r.doc.explain));
            let id = Self::write(
                g.record,
                kinds::PROPOSED,
                &d.subject,
                at,
                vec![fired],
                data,
                Self::publisher(r),
            )?;
            firing.outcome = "proposed".into();
            firing.written.push(Written {
                chronicle: "record".into(),
                event: id.hex(),
                effect: "proposal".into(),
            });
            return Ok(firing);
        }
        match self.apply(g, r, &d.subject, &effects, at, fired) {
            Ok(w) => {
                firing.outcome = "applied".into();
                firing.written = w;
            }
            Err(e) => firing.outcome = format!("refused: {e}"),
        }
        Ok(firing)
    }

    /// Carry out a firing's effects together (all or none), then link them in `rules.applied`.
    fn apply(
        &self,
        g: &mut Gate,
        r: &Rule,
        subject: &Address,
        effects: &[Effect],
        at: UniverseTime,
        because: ContentHash,
    ) -> Result<Vec<Written>, String> {
        // Check first, so a refused effect leaves nothing half-done.
        for e in effects {
            self.precheck(g, r, subject, e, at)?;
        }
        let mut out = vec![];
        for e in effects {
            let (chron, id) = self.apply_one(g, r, subject, e, at, because)?;
            out.push(Written {
                chronicle: chron.into(),
                event: id.hex(),
                effect: e.name().into(),
            });
        }
        let mut data = BTreeMap::new();
        data.insert("rule".into(), text(&r.doc.id));
        data.insert(
            "written".into(),
            Value::Text(serde_json::to_string(&out).expect("plain")),
        );
        Self::write(
            g.record,
            kinds::APPLIED,
            subject,
            at,
            vec![because],
            data,
            Self::publisher(r),
        )?;
        Ok(out)
    }

    fn kept_target(
        g: &Gate,
        r: &Rule,
        subject: &Address,
        e: &Effect,
        at: UniverseTime,
    ) -> Result<Option<Value>, String> {
        let Effect::SetKept {
            kind, value, add, ..
        } = e
        else {
            return Ok(None);
        };
        if let Some(v) = value {
            return Ok(Some(json_value(v)?));
        }
        let add = add.unwrap_or(0.0);
        let now = g
            .record
            .state_at(at)
            .get(subject, kind)
            .and_then(as_num)
            .or_else(|| {
                g.registry
                    .get(kind)
                    .and_then(|k| k.default.as_ref())
                    .and_then(J::as_f64)
            })
            .unwrap_or(0.0);
        let mut v = now + add;
        if let Some((lo, hi)) = g.registry.get(kind).and_then(|k| k.range) {
            v = v.clamp(lo, hi);
        }
        let _ = r;
        Ok(Some(Value::Float(v)))
    }

    fn precheck(
        &self,
        g: &Gate,
        r: &Rule,
        subject: &Address,
        e: &Effect,
        at: UniverseTime,
    ) -> Result<(), String> {
        match e {
            Effect::SetKept { kind, .. } => {
                let v = Self::kept_target(g, r, subject, e, at)?;
                g.registry
                    .check(kind, v.as_ref(), &format!("effect.{}", r.doc.id))
                    .map_err(|x| x.to_string())
            }
            Effect::Grant { .. } if g.campaign.is_none() => {
                Err("the stack defines no data views".into())
            }
            Effect::Grant { audience, .. } if audience.contains('{') => Err(format!(
                "grant audience {audience}: the observer has no such text fact"
            )),
            Effect::SetPin { value, .. } => json_value(value).map(|_| ()),
            Effect::Create { template, key, .. } => {
                let t = g
                    .templates
                    .get(template)
                    .ok_or_else(|| format!("no template {template}"))?;
                templates::next_address(g.universe, t, subject, key.as_deref()).map(|_| ())
            }
            Effect::Retire { .. } => {
                let st = g.universe.state_at(at);
                if st.get(subject, templates::fields::ID).is_none() {
                    Err(format!("{subject} was not made from a template"))
                } else if st.get(subject, templates::fields::RETIRED) == Some(&Value::Bool(true)) {
                    Err(format!("{subject} is already retired"))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn apply_one(
        &self,
        g: &mut Gate,
        r: &Rule,
        subject: &Address,
        e: &Effect,
        at: UniverseTime,
        because: ContentHash,
    ) -> Result<(&'static str, ContentHash), String> {
        let publ = Self::publisher(r);
        let writer = format!("effect.{}", r.doc.id);
        Ok(match e {
            Effect::SetKept { kind, reason, .. } => {
                let v = Self::kept_target(g, r, subject, e, at)?;
                let id = sse_record::set_kept(
                    g.record,
                    g.registry,
                    &KeptSet {
                        address: subject.clone(),
                        kind: kind.clone(),
                        value: v,
                        writer,
                        reason: reason.clone().or_else(|| Some(r.doc.explain.clone())),
                        causes: vec![because],
                    },
                    at,
                    &publ,
                )
                .map_err(|x| x.to_string())?;
                ("record", id)
            }
            Effect::AddNote {
                text: t,
                audience,
                tags,
                tier,
            } => {
                let id = sse_record::add_note(
                    g.record,
                    &NoteDraft {
                        subjects: vec![subject.clone()],
                        text: t.clone(),
                        tags: tags.clone(),
                        by: writer,
                        audience: audience.clone(),
                        tier: tier.clone(),
                        supersedes: None,
                        narrative: false,
                        arrives: None,
                    },
                    at,
                    &publ,
                )
                .map_err(|x| x.to_string())?;
                ("record", id)
            }
            Effect::Grant {
                audience,
                view,
                ladder,
            } => {
                let campaign = g.campaign.ok_or("the stack defines no data views")?;
                let to = match view {
                    Some(v) => v.clone(),
                    None => {
                        let state = g.universe.state_at(at);
                        let now = sse_observe::effective_view(&state, subject, audience, campaign);
                        match ladder.iter().position(|x| *x == now) {
                            Some(i) if i + 1 < ladder.len() => ladder[i + 1].clone(),
                            Some(_) => {
                                return Err(format!("{audience} already has the top rung {now}"))
                            }
                            None => ladder[0].clone(),
                        }
                    }
                };
                let id =
                    sse_observe::grant(g.universe, subject, audience, Some(&to), false, at, &publ)
                        .map_err(|x| x.to_string())?;
                ("universe", id)
            }
            Effect::Tag { tag } | Effect::Untag { tag } => {
                let mut payload = Payload::default();
                let field = format!("{TAG_FIELD}{tag}");
                if matches!(e, Effect::Tag { .. }) {
                    payload
                        .set
                        .insert(subject.clone(), [(field, Value::Bool(true))].into());
                } else {
                    payload.unset.insert(subject.clone(), vec![field]);
                }
                (
                    "universe",
                    authored(g.universe, subject, at, payload, publ)?,
                )
            }
            Effect::SetPin { field, value } => {
                let mut payload = Payload::default();
                payload.set.insert(
                    subject.clone(),
                    [(field.clone(), json_value(value)?)].into(),
                );
                (
                    "universe",
                    authored(g.universe, subject, at, payload, publ)?,
                )
            }
            Effect::SendIntent { kind, payload } => {
                let mut data = BTreeMap::new();
                data.insert("from".into(), text(&writer));
                data.insert(
                    "payload".into(),
                    Value::Text(serde_json::to_string(payload).expect("plain")),
                );
                let id = Self::write(
                    g.record,
                    &format!("{}{kind}", kinds::INTENT_PREFIX),
                    subject,
                    at,
                    vec![because],
                    data,
                    publ,
                )?;
                ("record", id)
            }
            Effect::Create {
                template,
                key,
                name,
            } => {
                let t = g
                    .templates
                    .get(template)
                    .ok_or_else(|| format!("no template {template}"))?
                    .clone();
                let (_, id) = templates::create(
                    g.universe,
                    g.record,
                    g.registry,
                    g.root,
                    &t,
                    subject,
                    key.as_deref(),
                    name.as_deref(),
                    at,
                    vec![because],
                )?;
                ("universe", id)
            }
            Effect::Retire { reason } => {
                let id = templates::retire(
                    g.universe,
                    subject,
                    reason.as_deref().or(Some(r.doc.explain.as_str())),
                    at,
                    &publ,
                )?;
                ("universe", id)
            }
        })
    }

    /// Proposals waiting for the GM, oldest first.
    pub fn pending(record: &Chronicle) -> Vec<Proposal> {
        let decided: BTreeSet<String> = record
            .events()
            .iter()
            .filter(|e| e.body.kind == kinds::DECIDED)
            .filter_map(|e| data_text(e, "proposal").map(String::from))
            .collect();
        record
            .events()
            .iter()
            .filter(|e| e.body.kind == kinds::PROPOSED && !decided.contains(&e.id.hex()))
            .map(|e| Proposal {
                id: e.id.hex(),
                rule: data_text(e, "rule").unwrap_or_default().into(),
                subject: e
                    .body
                    .subject
                    .first()
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
                observer: data_text(e, "observer").map(String::from),
                at_s: e.body.occurred_at.s,
                effects: data_text(e, "effects")
                    .and_then(|t| serde_json::from_str(t).ok())
                    .unwrap_or_default(),
                explain: data_text(e, "explain").unwrap_or_default().into(),
            })
            .collect()
    }

    /// The GM answers a proposal: yes applies its effects (at `at_s`, default
    /// the proposal's own time), no records the refusal. Either way it leaves
    /// the queue.
    pub fn decide(
        &self,
        g: &mut Gate,
        proposal: &str,
        yes: bool,
        at_s: Option<i64>,
    ) -> Result<Vec<Written>, String> {
        let p = Self::pending(g.record)
            .into_iter()
            .find(|p| p.id == proposal)
            .ok_or_else(|| format!("no proposal {proposal} waits for an answer"))?;
        let r = self
            .find(&p.rule)
            .ok_or_else(|| format!("rule {} is not loaded", p.rule))?
            .clone();
        let at_s = at_s.unwrap_or(p.at_s);
        if at_s < p.at_s {
            return Err("an answer cannot come before the proposal".into());
        }
        let at = UniverseTime::seconds(at_s);
        let pid = ContentHash::parse(&p.id).ok_or("bad proposal id")?;
        let subject: Address = p.subject.parse().map_err(|e| format!("{e:?}"))?;
        if yes {
            for e in &p.effects {
                self.precheck(g, &r, &subject, e, at)?;
            }
        }
        let mut data = BTreeMap::new();
        data.insert("proposal".into(), text(&p.id));
        data.insert("rule".into(), text(&p.rule));
        data.insert("yes".into(), Value::Bool(yes));
        let d = Self::write(
            g.record,
            kinds::DECIDED,
            &subject,
            at,
            vec![pid],
            data,
            Publisher::new("gm", "approve", "1"),
        )?;
        if !yes {
            return Ok(vec![]);
        }
        self.apply(g, &r, &subject, &p.effects, at, d)
    }

    /// The GM changes a rule's approval policy without editing the pack.
    pub fn set_policy(
        &self,
        g: &mut Gate,
        rule: &str,
        approval: &Approval,
        at_s: i64,
    ) -> Result<ContentHash, String> {
        self.find(rule).ok_or_else(|| format!("no rule {rule}"))?;
        let mut data = BTreeMap::new();
        data.insert("rule".into(), text(rule));
        data.insert(
            "approval".into(),
            Value::Text(serde_json::to_string(approval).expect("plain")),
        );
        Self::write(
            g.record,
            kinds::POLICY,
            g.root,
            UniverseTime::seconds(at_s),
            vec![],
            data,
            Publisher::new("gm", "policy", "1"),
        )
    }

    /// The rule ids as loaded, with their effective policies.
    pub fn listing(&self, record: &Chronicle) -> Vec<J> {
        self.rules
            .iter()
            .map(|r| {
                let policy = Self::policy_override(record, &r.doc.id);
                serde_json::json!({
                    "id": r.doc.id,
                    "pack": r.pack,
                    "phase": r.doc.phase,
                    "when": r.doc.when,
                    "effects": r.doc.effects.iter().map(Effect::name).collect::<Vec<_>>(),
                    "approval": policy.or_else(|| r.doc.approval.clone()),
                    "explain": r.doc.explain,
                })
            })
            .collect()
    }
}

fn authored(
    universe: &mut Chronicle,
    subject: &Address,
    at: UniverseTime,
    payload: Payload,
    publisher: Publisher,
) -> Result<ContentHash, String> {
    let e = Event::new(EventBody {
        kind: ck::AUTHORED_SET.into(),
        subject: vec![subject.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: universe.next_stamp(),
        causes: vec![],
        payload,
        evidence: Evidence::Authored,
        publisher,
    })
    .map_err(|e| e.to_string())?;
    universe.append(e).map_err(|e| e.to_string())
}

/// A kept value written as `"{intent.<field>}"` takes that field of the
/// intent that fired the rule (an `on_intent` rule), any JSON type.
fn with_intent(e: &Effect, payload: &J) -> Effect {
    match e {
        Effect::SetKept {
            kind,
            value: Some(J::String(v)),
            add,
            reason,
        } => {
            let got = v
                .strip_prefix("{intent.")
                .and_then(|f| f.strip_suffix('}'))
                .and_then(|f| payload.get(f));
            Effect::SetKept {
                kind: kind.clone(),
                value: Some(got.cloned().unwrap_or_else(|| J::String(v.clone()))),
                add: *add,
                reason: reason.clone(),
            }
        }
        other => other.clone(),
    }
}

/// The field named by an `{observer.<field>}` audience.
fn observer_field(a: &str) -> Option<&str> {
    a.strip_prefix("{observer.")?.strip_suffix('}')
}

fn substitute(
    e: &Effect,
    subject: &Address,
    observer: Option<&Address>,
    observer_facts: &BTreeMap<String, Value>,
) -> Effect {
    let fill = |t: &str| {
        let t = t.replace("{subject}", &subject.to_string());
        match observer {
            Some(o) => t.replace("{observer}", &o.to_string()),
            None => t,
        }
    };
    match e {
        Effect::AddNote {
            text,
            audience,
            tags,
            tier,
        } => Effect::AddNote {
            text: fill(text),
            audience: audience.clone(),
            tags: tags.clone(),
            tier: tier.clone(),
        },
        Effect::Create {
            template,
            key,
            name,
        } => Effect::Create {
            template: template.clone(),
            key: key.clone(),
            name: name.as_deref().map(fill),
        },
        Effect::Grant {
            audience,
            view,
            ladder,
        } => Effect::Grant {
            // Left as written when the fact is missing; the precheck refuses it.
            audience: match observer_field(audience).and_then(|f| observer_facts.get(f)) {
                Some(Value::Text(t)) if !t.contains('{') => t.clone(),
                _ => audience.clone(),
            },
            view: view.clone(),
            ladder: ladder.clone(),
        },
        other => other.clone(),
    }
}

/// The record's `rules.*` events, for a GM's "why?" (never sent to a seat).
pub fn trail(record: &Chronicle) -> Vec<J> {
    record
        .events()
        .iter()
        .filter(|e| e.body.kind.starts_with("rules."))
        .map(|e| {
            serde_json::json!({
                "id": e.id.hex(),
                "kind": e.body.kind,
                "at_s": e.body.occurred_at.s,
                "subject": e.body.subject.first().map(|a| a.to_string()),
                "rule": data_text(e, "rule"),
                "pass": data_int(e, "pass"),
                "causes": e.body.causes.iter().map(|c| c.hex()).collect::<Vec<_>>(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod intent_tests;
