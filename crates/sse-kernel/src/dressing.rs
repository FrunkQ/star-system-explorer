//! TG12: dressing rules (tags design section 5.7). A universe pack says "a
//! world like this gets one of these": a condition over facts and tags, a
//! template to place, where, how many, and optionally a seeded chance.
//!
//! - **Deterministic.** The condition reads facts and tags. A chance is
//!   drawn from the universe seed, the world's address and the rule id with
//!   the copy number (`<rule>#<n>`), as TG4 draws a rule tag's chance.
//! - **Lazy, stored nowhere.** Candidates are worked out when asked for a
//!   scope. They follow the facts if the world changes.
//! - **Adopted, then the campaign's.** Adopting makes an ordinary authored
//!   object from the template under the world. It carries the pack, rule and
//!   candidate that made it (`dressing.*` facts), so removing the pack
//!   leaves it valid.
//! - **The GM wins.** A held-off candidate never comes back for that world.
//!   The hold-off is a record fact at the world, under `observe.*`, so no
//!   scene shows it.
//! - **Never physics.** Dressing adds objects; it changes no fact.

use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use sse_chronicle::State;
use sse_core::cbor::Value;
use sse_core::{Address, Seed};
use sse_sift::Predicate;
use std::collections::{BTreeMap, BTreeSet};

/// The world field that holds a candidate off (record side): `<prefix><rule>#<n>`.
pub const HELD_PREFIX: &str = "observe.dressing.held.";

/// Facts an adopted object carries.
pub mod fields {
    pub const PACK: &str = "dressing.pack";
    pub const RULE: &str = "dressing.rule";
    pub const CANDIDATE: &str = "dressing.candidate";
    pub const WHERE: &str = "dressing.where";
    pub const FACTION: &str = "dressing.faction";
}

/// One dressing rule (`provides.other.dressing.rules[]`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DressingRule {
    pub id: String,
    /// Condition over the world's facts and tags (and its parent's facts).
    pub when: Predicate,
    /// Template id to place.
    pub place: String,
    /// Where on or around the world: `low-orbit`, `high-orbit`, `surface`,
    /// `L4`, `belt`, ... (a label the chart room draws by).
    #[serde(rename = "where")]
    pub at: String,
    #[serde(default = "one")]
    pub count: u32,
    /// Seeded chance per copy (TG4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f64>,
    /// Extra tags the placed object carries beyond its template's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// The actor that holds it (an actor id in the record).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    /// Address segment kinds the rule applies to (`body`, ...); empty = all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to: Vec<String>,
    pub explain: String,
}
fn one() -> u32 {
    1
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DressingPack {
    #[serde(default)]
    #[allow(dead_code)]
    about: Option<String>,
    rules: Vec<DressingRule>,
}

/// The stack's dressing rules, bottom to top, with the pack each came from.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Dressing {
    pub rules: Vec<(String, DressingRule)>,
}

impl Dressing {
    /// Read every pack's `provides.other.dressing`. A rule id declared twice,
    /// a template the stack does not provide, a count of 0 and odds outside
    /// 0 to 1 are refused.
    pub fn from_packs(
        packs: &[(String, J)],
        templates: &sse_trigger::templates::Templates,
    ) -> Result<Self, Vec<String>> {
        let mut out = Dressing::default();
        let mut problems = vec![];
        let mut ids = BTreeSet::new();
        for (pack, v) in packs {
            let d: DressingPack = match serde_json::from_value(v.clone()) {
                Ok(d) => d,
                Err(e) => {
                    problems.push(format!("{pack}: dressing: {e}"));
                    continue;
                }
            };
            for r in d.rules {
                if !ids.insert(r.id.clone()) {
                    problems.push(format!("{pack}: dressing rule {} defined twice", r.id));
                }
                if templates.get(&r.place).is_none() {
                    problems.push(format!(
                        "{pack}: dressing rule {}: places template {}, which no pack in the stack provides",
                        r.id, r.place
                    ));
                }
                if r.count == 0 {
                    problems.push(format!(
                        "{pack}: dressing rule {}: count must be at least 1",
                        r.id
                    ));
                }
                if let Some(p) = r.chance {
                    if !(0.0..=1.0).contains(&p) {
                        problems.push(format!(
                            "{pack}: dressing rule {}: chance {p} must be between 0 and 1",
                            r.id
                        ));
                    }
                }
                out.rules.push((pack.clone(), r));
            }
        }
        if problems.is_empty() {
            Ok(out)
        } else {
            Err(problems)
        }
    }

    pub fn get(&self, id: &str) -> Option<&(String, DressingRule)> {
        self.rules.iter().find(|(_, r)| r.id == id)
    }
}

fn json(v: &Value) -> J {
    serde_json::to_value(v).unwrap_or(J::Null)
}

/// A candidate's id: `<world>#<rule>#<n>`.
pub fn candidate_id(world: &Address, rule: &str, n: u32) -> String {
    format!("{world}#{rule}#{n}")
}

/// Parse a candidate id back into world, rule and copy number.
pub fn parse_candidate(id: &str) -> Option<(Address, String, u32)> {
    let mut parts = id.rsplitn(3, '#');
    let n = parts.next()?.parse().ok()?;
    let rule = parts.next()?.to_string();
    let world = parts.next()?.parse().ok()?;
    Some((world, rule, n))
}

/// The candidates for every world at or under `scope`: `state` is the
/// universe state with record facts folded in, `tags` each address's tags
/// (rules and authored). Held-off and adopted candidates are left out.
pub fn candidates(
    d: &Dressing,
    templates: &sse_trigger::templates::Templates,
    state: &State,
    tags: &BTreeMap<Address, BTreeSet<String>>,
    seed: &Seed,
    scope: &Address,
    now_s: i64,
) -> Vec<J> {
    // Candidates already adopted, by id.
    let adopted: BTreeSet<String> = state
        .facts
        .values()
        .filter_map(|f| match f.get(fields::CANDIDATE) {
            Some(Value::Text(c)) => Some(c.clone()),
            _ => None,
        })
        .collect();
    let mut out = vec![];
    for (a, f) in state.facts.iter().filter(|(a, _)| scope.contains(a)) {
        // Objects placed by dressing are not themselves dressed.
        if f.contains_key(fields::CANDIDATE) {
            continue;
        }
        let subject = sse_sift::Subject {
            address: Some(a.clone()),
            facts: f.clone(),
            parent: a
                .parent()
                .and_then(|p| state.facts.get(&p).cloned())
                .unwrap_or_default(),
            tags: tags.get(a).cloned().unwrap_or_default(),
            now_s,
        };
        for (pack, r) in &d.rules {
            if !r.applies_to.is_empty() && !r.applies_to.iter().any(|k| k == a.last().kind()) {
                continue;
            }
            let e = r.when.eval(&subject, false);
            if !e.holds {
                continue;
            }
            for n in 0..r.count {
                let copy = format!("{}#{n}", r.id);
                if let Some(p) = r.chance {
                    let rolled = p >= 1.0 || (p > 0.0 && sse_sift::chance_roll(seed, a, &copy) < p);
                    if !rolled {
                        continue;
                    }
                }
                let id = candidate_id(a, &r.id, n);
                let held = matches!(
                    f.get(&format!("{HELD_PREFIX}{copy}")),
                    Some(Value::Bool(true))
                );
                if held || adopted.contains(&id) {
                    continue;
                }
                let label = templates
                    .get(&r.place)
                    .map(|t| t.doc.label.clone())
                    .unwrap_or_default();
                let mut why: Vec<J> = e
                    .read
                    .iter()
                    .map(|x| serde_json::json!({"field": x.field, "value": json(&x.value)}))
                    .collect();
                if let Some(p) = r.chance {
                    why.push(serde_json::json!({"field": "chance", "value": p}));
                }
                let mut c = serde_json::json!({
                    "id": id,
                    "world": a.to_string(),
                    "rule": r.id,
                    "pack": pack,
                    "template": r.place,
                    "label": label,
                    "where": r.at,
                    "explain": r.explain,
                    "why": why,
                });
                if !r.tags.is_empty() {
                    c["tags"] = serde_json::json!(r.tags);
                }
                if let Some(fa) = &r.faction {
                    c["faction"] = serde_json::json!(fa);
                }
                out.push(c);
            }
        }
    }
    out
}

/// Objects dressing placed whose pack or rule is no longer in the stack,
/// not retired: what a pack swap offers to retire. `touched` says whether
/// anything but the creation ever wrote to it (those the GM edited are
/// kept by default).
pub fn orphans(
    d: &Dressing,
    chronicle: &sse_chronicle::Chronicle,
    state: &State,
    scope: &Address,
) -> Vec<J> {
    let mut out = vec![];
    for (a, f) in state.facts.iter().filter(|(a, _)| scope.contains(a)) {
        let (Some(Value::Text(pack)), Some(Value::Text(rule))) =
            (f.get(fields::PACK), f.get(fields::RULE))
        else {
            continue;
        };
        if matches!(f.get("retired"), Some(Value::Bool(true))) {
            continue;
        }
        if d.get(rule).is_some_and(|(p, _)| p == pack) {
            continue;
        }
        let writes = chronicle
            .events()
            .iter()
            .filter(|e| e.body.payload.set.contains_key(a) || e.body.payload.unset.contains_key(a))
            .count();
        out.push(serde_json::json!({
            "address": a.to_string(), "pack": pack, "rule": rule, "touched": writes > 1,
        }));
    }
    out
}

/// The template an adopted candidate is made from: the stack's template
/// plus the rule's tags and the dressing marks, so the object stands alone.
pub fn adopt_template(
    t: &sse_trigger::templates::Template,
    pack: &str,
    r: &DressingRule,
    id: &str,
) -> sse_trigger::templates::Template {
    let mut t = t.clone();
    let f = &mut t.doc.facts;
    f.insert(fields::PACK.into(), J::from(pack));
    f.insert(fields::RULE.into(), J::from(r.id.as_str()));
    f.insert(fields::CANDIDATE.into(), J::from(id));
    f.insert(fields::WHERE.into(), J::from(r.at.as_str()));
    if let Some(fa) = &r.faction {
        f.insert(fields::FACTION.into(), J::from(fa.as_str()));
    }
    for tag in &r.tags {
        if !t.doc.tags.contains(tag) {
            t.doc.tags.push(tag.clone());
        }
    }
    t
}
