//! J6: programs, small state machines that drive a program seat (record
//! design section 6). A program belongs to an object made from a template
//! that lists it (J7). Each tick it senses through its seat's audience,
//! picks a target among what it has been shown, and acts by sending an
//! intent through the same checks as a player (J5). Its state lives in the
//! record, so a run replays and a look-ahead branch runs it forward.
//!
//! This module holds the documents, the registry and the record fold; the
//! kernel does the sensing (a scene) and the sending (`send_intent`).

use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash};
use sse_sift::{Predicate, Subject};
use std::collections::BTreeMap;

pub mod kinds {
    pub const ATTACHED: &str = "programs.attached";
    pub const DETACHED: &str = "programs.detached";
    pub const STATE: &str = "programs.state";
}

/// Running programs a campaign allows when no pack says otherwise.
pub const DEFAULT_LIMIT: usize = 16;
/// State changes one program may make in one tick when its doc says nothing.
pub const DEFAULT_MAX_STEPS: u32 = 4;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Max,
    Min,
}

/// Choose one sensed subject: those for which `where` holds, best by `by`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pick {
    #[serde(rename = "where")]
    pub when: Predicate,
    pub by: String,
    #[serde(default = "max")]
    pub order: Order,
}
fn max() -> Order {
    Order::Max
}

/// The intent a state sends: `kind` and a payload in which the strings
/// `{self}`, `{pick}` and `{param.<name>}` are filled in.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Act {
    pub kind: String,
    #[serde(default)]
    pub payload: serde_json::Map<String, J>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub when: Predicate,
    pub to: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<Pick>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act: Option<Act>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    #[serde(default = "steps")]
    pub max_steps: u32,
}
fn steps() -> u32 {
    DEFAULT_MAX_STEPS
}
impl Default for Budget {
    fn default() -> Self {
        Budget { max_steps: steps() }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramDoc {
    pub id: String,
    #[serde(default)]
    pub label: String,
    /// Choices and thresholds, plain controls for the GM (`param.<name>`).
    #[serde(default)]
    pub params: BTreeMap<String, J>,
    pub start: String,
    pub states: Vec<State>,
    #[serde(default)]
    pub budget: Budget,
    #[serde(default)]
    pub explain: String,
}

impl ProgramDoc {
    pub fn state(&self, id: &str) -> Option<&State> {
        self.states.iter().find(|s| s.id == id)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Program {
    #[serde(flatten)]
    pub doc: ProgramDoc,
    pub pack: String,
}

/// Every program of the stack, and the campaign's limit on running ones.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Programs {
    pub programs: Vec<Program>,
    pub limit: usize,
}

impl Default for Programs {
    fn default() -> Self {
        Programs {
            programs: vec![],
            limit: DEFAULT_LIMIT,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackPrograms {
    #[serde(default)]
    programs: Vec<ProgramDoc>,
    #[serde(default)]
    limit: Option<usize>,
}

impl Programs {
    /// `provides.other.programs` of each pack, bottom to top: `{programs, limit?}`.
    /// The top pack's limit wins.
    pub fn from_packs(found: &[(String, J)]) -> Result<Programs, Vec<String>> {
        let mut out = Programs::default();
        let mut problems = vec![];
        for (pack, v) in found {
            let p: PackPrograms = match serde_json::from_value(v.clone()) {
                Ok(p) => p,
                Err(e) => {
                    problems.push(format!("{pack}: programs: {e}"));
                    continue;
                }
            };
            if let Some(l) = p.limit {
                out.limit = l;
            }
            for d in p.programs {
                problems.extend(check(pack, &d));
                if out.get(&d.id).is_some() {
                    problems.push(format!("{pack}: program {} is declared twice", d.id));
                }
                out.programs.push(Program {
                    doc: d,
                    pack: pack.clone(),
                });
            }
        }
        if problems.is_empty() {
            Ok(out)
        } else {
            Err(problems)
        }
    }

    pub fn get(&self, id: &str) -> Option<&Program> {
        self.programs.iter().find(|p| p.doc.id == id)
    }

    /// The intent kinds the programs send, for checking against the registry.
    pub fn sends(&self) -> Vec<(String, String)> {
        let mut out = vec![];
        for p in &self.programs {
            for s in &p.doc.states {
                if let Some(a) = &s.act {
                    out.push((p.doc.id.clone(), a.kind.clone()));
                }
            }
        }
        out
    }
}

fn check(pack: &str, d: &ProgramDoc) -> Vec<String> {
    let mut out = vec![];
    let ident = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-".contains(&c))
    };
    if !ident(&d.id) {
        out.push(format!(
            "{pack}: program id {:?} must be a lower-case name",
            d.id
        ));
    }
    if d.states.is_empty() {
        out.push(format!("{pack}: program {} has no states", d.id));
    }
    for (i, s) in d.states.iter().enumerate() {
        if d.states[..i].iter().any(|x| x.id == s.id) {
            out.push(format!("{pack}: program {}: state {} twice", d.id, s.id));
        }
        for t in &s.transitions {
            if d.state(&t.to).is_none() {
                out.push(format!(
                    "{pack}: program {}: state {} goes to unknown state {}",
                    d.id, s.id, t.to
                ));
            }
        }
        if let Some(a) = &s.act {
            let uses_pick = a.payload.values().any(|v| v.as_str() == Some("{pick}"));
            if uses_pick && s.pick.is_none() {
                out.push(format!(
                    "{pack}: program {}: state {} sends {{pick}} but picks nothing",
                    d.id, s.id
                ));
            }
            for v in a.payload.values() {
                if let Some(p) = v
                    .as_str()
                    .and_then(|t| t.strip_prefix("{param."))
                    .and_then(|t| t.strip_suffix('}'))
                {
                    if !d.params.contains_key(p) {
                        out.push(format!(
                            "{pack}: program {}: state {} names unknown param {p}",
                            d.id, s.id
                        ));
                    }
                }
            }
        }
    }
    if d.state(&d.start).is_none() {
        out.push(format!(
            "{pack}: program {}: start state {} is not declared",
            d.id, d.start
        ));
    }
    if d.budget.max_steps == 0 {
        out.push(format!(
            "{pack}: program {}: max_steps must be at least 1",
            d.id
        ));
    }
    out
}

/// A running program, as the record folds it.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Running {
    pub address: String,
    pub program: String,
    pub seat: String,
    pub state: String,
    pub since_s: i64,
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

/// The programs running at `at_s`, by address.
pub fn running(record: &Chronicle, at_s: i64) -> BTreeMap<String, Running> {
    let mut out: BTreeMap<String, Running> = BTreeMap::new();
    for e in record.events() {
        if e.body.occurred_at.s > at_s {
            continue;
        }
        let Some(a) = e.body.subject.first().map(|a| a.to_string()) else {
            continue;
        };
        match e.body.kind.as_str() {
            kinds::ATTACHED => {
                out.insert(
                    a.clone(),
                    Running {
                        address: a,
                        program: data_text(e, "program").unwrap_or("").into(),
                        seat: data_text(e, "seat").unwrap_or("").into(),
                        state: data_text(e, "state").unwrap_or("").into(),
                        since_s: e.body.occurred_at.s,
                    },
                );
            }
            kinds::DETACHED => {
                out.remove(&a);
            }
            kinds::STATE => {
                if let Some(r) = out.get_mut(&a) {
                    r.state = data_text(e, "state").unwrap_or("").into();
                    r.since_s = e.body.occurred_at.s;
                }
            }
            _ => {}
        }
    }
    out
}

/// Write a program event on the record.
pub fn write(
    record: &mut Chronicle,
    kind: &str,
    subject: &Address,
    at_s: i64,
    data: &[(&str, &str)],
    program: &str,
) -> Result<ContentHash, String> {
    let at = UniverseTime::seconds(at_s);
    let e = Event::new(EventBody {
        kind: kind.into(),
        subject: vec![subject.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: record.next_stamp(),
        causes: vec![],
        payload: Payload {
            data: Some(Value::Map(
                data.iter().map(|(k, v)| (k.to_string(), text(v))).collect(),
            )),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("program", program, "1"),
    })
    .map_err(|e| e.to_string())?;
    record.append(e).map_err(|e| e.to_string())
}

/// A JSON value as a fact.
pub fn fact(v: &J) -> Option<Value> {
    serde_json::from_value(v.clone()).ok()
}

/// The params as `param.<name>` facts, added to a subject.
pub fn with_params(mut s: Subject, d: &ProgramDoc) -> Subject {
    for (k, v) in &d.params {
        if let Some(x) = fact(v) {
            s.facts.insert(format!("param.{k}"), x);
        }
    }
    s
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// The best sensed subject for a pick: `where` holds and `by` is a number.
/// Ties go to the first in sensed order (scenes list addresses in order),
/// so every replay picks the same.
pub fn pick<'a>(p: &Pick, d: &ProgramDoc, sensed: &'a [(Address, Subject)]) -> Option<&'a Address> {
    let mut best: Option<(&Address, f64)> = None;
    for (a, s) in sensed {
        let s = with_params(s.clone(), d);
        if !p.when.eval(&s, false).holds {
            continue;
        }
        let Some(x) = s.facts.get(&p.by).and_then(num) else {
            continue;
        };
        let better = match (&best, &p.order) {
            (None, _) => true,
            (Some((_, b)), Order::Max) => x > *b,
            (Some((_, b)), Order::Min) => x < *b,
        };
        if better {
            best = Some((a, x));
        }
    }
    best.map(|b| b.0)
}

/// The intent a state sends, with `{self}`, `{pick}` and `{param.<name>}` filled in.
pub fn intent(act: &Act, d: &ProgramDoc, me: &Address, picked: Option<&Address>) -> J {
    let mut o = serde_json::Map::new();
    o.insert("kind".into(), J::String(act.kind.clone()));
    for (k, v) in &act.payload {
        let filled = match v.as_str() {
            Some("{self}") => J::String(me.to_string()),
            Some("{pick}") => picked.map_or(J::Null, |a| J::String(a.to_string())),
            Some(t) => match t.strip_prefix("{param.").and_then(|t| t.strip_suffix('}')) {
                Some(p) => d.params.get(p).cloned().unwrap_or(J::Null),
                None => v.clone(),
            },
            None => v.clone(),
        };
        o.insert(k.clone(), filled);
    }
    J::Object(o)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(v: J) -> Result<Programs, Vec<String>> {
        Programs::from_packs(&[("test.p".into(), v)])
    }

    #[test]
    fn load_refusals_and_picks() {
        let ok = json!({"programs": [{"id": "a", "params": {"g": 0.5}, "start": "s", "states": [
            {"id": "s", "pick": {"where": {"if": "compare", "left": "ore", "op": "ge", "right": "param.g"}, "by": "ore"},
             "act": {"kind": "go", "payload": {"to": "{pick}", "x": "{param.g}"}},
             "transitions": [{"when": {"if": "has", "field": "z"}, "to": "s"}]}]}]});
        let p = doc(ok.clone()).unwrap();
        assert_eq!(p.limit, DEFAULT_LIMIT);
        let bad = |f: &dyn Fn(&mut J), why: &str| {
            let mut v = ok.clone();
            f(&mut v);
            let e = doc(v).unwrap_err().join("; ");
            assert!(e.contains(why), "{e}");
        };
        bad(
            &|v| v["programs"][0]["start"] = json!("nope"),
            "start state nope",
        );
        bad(
            &|v| v["programs"][0]["states"][0]["transitions"][0]["to"] = json!("t"),
            "unknown state t",
        );
        bad(
            &|v| {
                v["programs"][0]["states"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("pick");
            },
            "picks nothing",
        );
        bad(
            &|v| v["programs"][0]["states"][0]["act"]["payload"]["x"] = json!("{param.h}"),
            "unknown param h",
        );
        bad(&|v| v["programs"][0]["id"] = json!("Big"), "lower-case");
        bad(&|v| v["programs"][0]["colour"] = json!(1), "unknown field");
        // Picks read only the sensed facts, best first, ties to the lower address.
        let d = &p.programs[0].doc;
        let s = |a: &str, ore: Option<f64>| -> (Address, Subject) {
            let mut f = BTreeMap::new();
            if let Some(x) = ore {
                f.insert("ore".to_string(), Value::Float(x));
            }
            (
                a.parse().unwrap(),
                Subject {
                    facts: f,
                    ..Default::default()
                },
            )
        };
        let sensed = [
            s("u:t/b:3", Some(0.6)),
            s("u:t/b:1", None),
            s("u:t/b:2", Some(0.6)),
            s("u:t/b:4", Some(0.4)),
        ];
        let pk = d.states[0].pick.as_ref().unwrap();
        assert_eq!(
            pick(pk, d, &sensed).unwrap().to_string(),
            "u:t/b:3",
            "first of the tie in sensed order"
        );
        let i = intent(
            d.states[0].act.as_ref().unwrap(),
            d,
            &"u:t/d:1".parse().unwrap(),
            Some(&"u:t/b:3".parse().unwrap()),
        );
        assert_eq!(i, json!({"kind": "go", "to": "u:t/b:3", "x": 0.5}));
    }
}
