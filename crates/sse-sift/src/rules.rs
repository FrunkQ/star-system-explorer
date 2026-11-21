//! Rules: predicates that put tags on subjects, with hysteresis so tags do
//! not flicker while a world ages (H6.6, classification design 5).

use crate::predicate::{Predicate, Read, Subject};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub tag: String,
    /// Condition to gain the tag.
    pub when: Predicate,
    /// Condition to keep it once held. Default: `when` read with its margins
    /// widened, so a subject on the edge does not flicker.
    #[serde(default)]
    pub stay: Option<Predicate>,
    /// Address segment kinds this rule applies to (`body`, `sys`); empty = all.
    #[serde(default)]
    pub applies_to: Vec<String>,
}

/// A tag a rule set, with why.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TagState {
    pub tag: String,
    pub rule: String,
    pub evidence: Vec<Read>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TagChange {
    pub at_s: i64,
    pub tag: String,
    pub rule: String,
    pub gained: bool,
    pub evidence: Vec<Read>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuleError(pub String);
impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The rule name a GM-forced tag carries: the authored mark.
pub const AUTHORED: &str = "authored";

/// The GM's authored tags among a subject's facts: `tag.<name>` true or false.
pub fn authored_tags(facts: &BTreeMap<String, sse_core::cbor::Value>) -> BTreeMap<String, bool> {
    facts
        .iter()
        .filter_map(|(k, v)| match (k.strip_prefix("tag."), v) {
            (Some(t), sse_core::cbor::Value::Bool(b)) => Some((t.to_string(), *b)),
            _ => None,
        })
        .collect()
}

impl RuleSet {
    pub fn validate(&self) -> Result<(), RuleError> {
        let mut ids = BTreeSet::new();
        for r in &self.rules {
            if !ids.insert(&r.id) {
                return Err(RuleError(format!("rule {} defined twice", r.id)));
            }
            if r.tag.is_empty() {
                return Err(RuleError(format!("rule {} sets no tag", r.id)));
            }
        }
        Ok(())
    }

    fn applies(r: &Rule, s: &Subject) -> bool {
        r.applies_to.is_empty()
            || s.address
                .as_ref()
                .is_some_and(|a| r.applies_to.iter().any(|k| k == a.last().kind()))
    }

    /// Tags for `s`, given the tags it held before (`held`: tag -> rule).
    /// A held tag stays while its rule's stay condition holds; a new one needs
    /// the full `when`. Several rules may set one tag: any of them suffices.
    /// Rules see the tags already decided earlier in rule order, so a rule can
    /// build on another's tag.
    pub fn apply(
        &self,
        s: &Subject,
        held: &BTreeMap<String, TagState>,
    ) -> BTreeMap<String, TagState> {
        self.apply_authored(s, held, &BTreeMap::new())
    }

    /// As [`apply`](Self::apply), under the GM's authored tags (tags design
    /// section 4, TG3): `true` forces a tag on and `false` holds it off,
    /// whatever its rules say, and rules that read it see the authored
    /// answer. A forced tag carries the authored mark (rule `authored`, the
    /// fact as its evidence) in place of the rule's. Taking the authored
    /// fact away gives the rule's answer back, with its evidence.
    pub fn apply_authored(
        &self,
        s: &Subject,
        held: &BTreeMap<String, TagState>,
        authored: &BTreeMap<String, bool>,
    ) -> BTreeMap<String, TagState> {
        let mut out: BTreeMap<String, TagState> = BTreeMap::new();
        let mut subj = s.clone();
        for (t, on) in authored {
            if *on {
                subj.tags.insert(t.clone());
            } else {
                subj.tags.remove(t);
            }
        }
        for r in self.rules.iter().filter(|r| Self::applies(r, s)) {
            if out.contains_key(&r.tag) || authored.contains_key(&r.tag) {
                continue;
            }
            let was = held.get(&r.tag).is_some_and(|h| h.rule == r.id);
            let e = match (&r.stay, was) {
                (Some(stay), true) => stay.eval(&subj, false),
                (None, true) => r.when.eval(&subj, true),
                (_, false) => r.when.eval(&subj, false),
            };
            if e.holds {
                subj.tags.insert(r.tag.clone());
                out.insert(
                    r.tag.clone(),
                    TagState {
                        tag: r.tag.clone(),
                        rule: r.id.clone(),
                        evidence: e.read,
                    },
                );
            }
        }
        for (t, on) in authored.iter().filter(|(_, on)| **on) {
            out.insert(
                t.clone(),
                TagState {
                    tag: t.clone(),
                    rule: AUTHORED.into(),
                    evidence: vec![Read {
                        field: format!("tag.{t}"),
                        value: sse_core::cbor::Value::Bool(*on),
                    }],
                },
            );
        }
        out
    }

    /// Run over a time series of readings for one subject (as an ageing run
    /// produces), returning the tag changes with their dates and evidence.
    pub fn track(&self, series: &[Subject]) -> (BTreeMap<String, TagState>, Vec<TagChange>) {
        let mut held = BTreeMap::new();
        let mut changes = Vec::new();
        for s in series {
            let next = self.apply(s, &held);
            for (t, st) in &next {
                if !held.contains_key(t) {
                    changes.push(TagChange {
                        at_s: s.now_s,
                        tag: t.clone(),
                        rule: st.rule.clone(),
                        gained: true,
                        evidence: st.evidence.clone(),
                    });
                }
            }
            for (t, st) in &held {
                if !next.contains_key(t) {
                    let ev = self
                        .rules
                        .iter()
                        .find(|r| r.id == st.rule)
                        .map(|r| r.when.eval(s, true).read)
                        .unwrap_or_default();
                    changes.push(TagChange {
                        at_s: s.now_s,
                        tag: t.clone(),
                        rule: st.rule.clone(),
                        gained: false,
                        evidence: ev,
                    });
                }
            }
            held = next;
        }
        (held, changes)
    }
}
