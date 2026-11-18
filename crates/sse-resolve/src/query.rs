//! QUERY: feasibility (H6.4). Read-only until the user commits a pick.
//!
//! - Location-first: "what could be here?" An outlook of the cloud, from the
//!   envelope alone, without resolving anything.
//! - Outcome-first: "where could I find this?" Score clouds for a wish, or
//!   walk a cloud's children, within a work budget. A search that finds
//!   nothing reports "not found within budget", never "impossible".

use crate::dist::Sample;
use crate::envelope::{CountSpec, Envelope, EnvelopeError, FieldSpec};
use crate::partition::budget_splits;
use crate::resolve::{children, resolve, ChildRef};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash, Seed};
use std::collections::BTreeMap;

/// One condition on a child's fact: a numeric range or a category.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Criterion {
    pub field: String,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub is: Option<String>,
}

/// What the user wants to find.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Wish {
    pub label: String,
    pub criteria: Vec<Criterion>,
    /// Smallest expected number of matching children worth reporting.
    #[serde(default)]
    pub min_expected: f64,
}

impl Wish {
    pub fn hash(&self) -> ContentHash {
        ContentHash::of(self).expect("wish holds no NaN")
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FieldOutlook {
    Numeric {
        p05: f64,
        p50: f64,
        p95: f64,
        mean: Option<f64>,
    },
    Category {
        probabilities: Vec<(String, f64)>,
    },
    Inherited {
        value: Value,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BudgetOutlook {
    pub total: u64,
    pub pinned: u64,
    pub deficit: u64,
    /// Expected amount per generated child (minor units).
    pub mean_per_child: f64,
}

/// Location-first answer.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Outlook {
    pub address: Address,
    pub tier: String,
    /// Expected generated children (pins excluded).
    pub expected_generated: f64,
    pub pins: usize,
    pub budgets: BTreeMap<String, BudgetOutlook>,
    pub fields: BTreeMap<String, FieldOutlook>,
}

fn expected_count(env: &Envelope) -> f64 {
    let mean = match &env.count {
        CountSpec::Fixed { n } => *n as f64,
        CountSpec::Poisson { mean } => *mean,
        CountSpec::Uniform { lo, hi } => (*lo + *hi) as f64 / 2.0,
    };
    if env.pins_count_toward {
        (mean - env.pins.len() as f64).max(0.0)
    } else {
        mean
    }
}

/// "What could be here?" from the envelope alone. Mutates nothing, draws nothing.
pub fn location_first(env: &Envelope) -> Result<Outlook, EnvelopeError> {
    env.validate()?;
    let n = expected_count(env);
    let budgets = budget_splits(env, 1)
        .into_iter()
        .map(|(id, s)| {
            (
                id,
                BudgetOutlook {
                    total: s.total,
                    pinned: s.pinned,
                    deficit: s.deficit,
                    mean_per_child: if n > 0.0 { s.remainder as f64 / n } else { 0.0 },
                },
            )
        })
        .collect();
    let num = |s: Sample| match s {
        Sample::Num(x) => x,
        Sample::Int(k) => k as f64,
        Sample::Cat(_) => f64::NAN,
    };
    let fields = env
        .fields
        .iter()
        .map(|(name, f)| {
            let o = match f {
                FieldSpec::Inherit { field } => FieldOutlook::Inherited {
                    value: env.facts.get(field).cloned().unwrap_or(Value::Null),
                },
                FieldSpec::Draw { dist, .. } => match dist {
                    crate::Distribution::Choice { options } => FieldOutlook::Category {
                        probabilities: options
                            .iter()
                            .map(|(l, _)| (l.clone(), dist.prob_is(l).unwrap_or(0.0)))
                            .collect(),
                    },
                    _ => FieldOutlook::Numeric {
                        p05: num(dist.quantile(0.05)),
                        p50: num(dist.quantile(0.5)),
                        p95: num(dist.quantile(0.95)),
                        mean: dist.mean(),
                    },
                },
            };
            (name.clone(), o)
        })
        .collect();
    Ok(Outlook {
        address: env.address.clone(),
        tier: env.tier.clone(),
        expected_generated: n,
        pins: env.pins.len(),
        budgets,
        fields,
    })
}

pub fn value_matches(v: &Value, c: &Criterion) -> bool {
    match v {
        Value::Text(t) => c.min.is_none() && c.max.is_none() && c.is.as_deref() == Some(t.as_str()),
        Value::Float(_) | Value::Int(_) => {
            let x = match v {
                Value::Float(f) => *f,
                Value::Int(i) => *i as f64,
                _ => unreachable!(),
            };
            c.is.is_none() && c.min.is_none_or(|lo| x >= lo) && c.max.is_none_or(|hi| x <= hi)
        }
        Value::Bool(b) => c.is.as_deref() == Some(if *b { "true" } else { "false" }),
        _ => false,
    }
}

/// Probability that one generated child of `env` meets `c` (used for rarity by
/// sifters), with the reason
/// when it cannot.
pub fn criterion_probability(env: &Envelope, c: &Criterion) -> (f64, Option<String>) {
    match env.fields.get(&c.field) {
        None => (
            0.0,
            Some(format!("this cloud says nothing about {}", c.field)),
        ),
        Some(FieldSpec::Inherit { field }) => match env.facts.get(field) {
            Some(v) if value_matches(v, c) => (1.0, None),
            Some(_) => (
                0.0,
                Some(format!("{} is inherited and does not match", c.field)),
            ),
            None => (
                0.0,
                Some(format!(
                    "{} is inherited but the parent has no {field}",
                    c.field
                )),
            ),
        },
        Some(FieldSpec::Draw { dist, .. }) => {
            let p = if let Some(label) = &c.is {
                dist.prob_is(label)
            } else {
                dist.prob_between(
                    c.min.unwrap_or(f64::NEG_INFINITY),
                    c.max.unwrap_or(f64::INFINITY),
                )
            };
            match p {
                Some(p) if p > 0.0 => (p, None),
                Some(_) => (
                    0.0,
                    Some(format!("{} cannot fall in the asked range here", c.field)),
                ),
                None => (
                    0.0,
                    Some(format!(
                        "{} is of the wrong type for this criterion",
                        c.field
                    )),
                ),
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub address: Address,
    /// Probability one child matches every criterion (fields drawn independently).
    pub probability: f64,
    pub expected_matches: f64,
    /// Why a criterion fails here, if one does.
    pub reasons: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum QueryOutcome {
    Found {
        candidates: Vec<Candidate>,
        searched: usize,
        of: usize,
        work: u64,
    },
    /// Never "impossible": only that nothing turned up in what was searched.
    NotFoundWithinBudget {
        searched: usize,
        of: usize,
        work: u64,
        nearest: Vec<Candidate>,
    },
}

/// "Where could I find this?" Scores clouds, best first (ties by address).
/// Clouds are taken in address order, whatever order the caller gives, so
/// the answer depends only on the set and the budget. Each cloud costs
/// `1 + criteria` work units.
pub fn outcome_first(
    clouds: &[Envelope],
    wish: &Wish,
    work_budget: u64,
    max_results: usize,
) -> QueryOutcome {
    let mut order: Vec<&Envelope> = clouds.iter().collect();
    order.sort_by(|a, b| a.address.cmp(&b.address));
    let per = 1 + wish.criteria.len() as u64;
    let mut work = 0u64;
    let mut scored = Vec::new();
    let mut searched = 0;
    for env in order {
        if work + per > work_budget {
            break;
        }
        work += per;
        searched += 1;
        if env.validate().is_err() {
            continue;
        }
        let mut p = 1.0;
        let mut reasons = Vec::new();
        for c in &wish.criteria {
            let (pc, why) = criterion_probability(env, c);
            p *= pc;
            reasons.extend(why);
        }
        scored.push(Candidate {
            address: env.address.clone(),
            probability: p,
            expected_matches: p * expected_count(env),
            reasons,
        });
    }
    let rank = |v: &mut Vec<Candidate>| {
        v.sort_by(|a, b| {
            b.expected_matches
                .total_cmp(&a.expected_matches)
                .then(b.probability.total_cmp(&a.probability))
                .then(a.address.cmp(&b.address))
        })
    };
    let (mut hits, mut rest): (Vec<_>, Vec<_>) = scored
        .into_iter()
        .partition(|c| c.expected_matches > 0.0 && c.expected_matches >= wish.min_expected);
    rank(&mut hits);
    rank(&mut rest);
    if hits.is_empty() {
        rest.truncate(3);
        QueryOutcome::NotFoundWithinBudget {
            searched,
            of: clouds.len(),
            work,
            nearest: rest,
        }
    } else {
        hits.truncate(max_results);
        QueryOutcome::Found {
            candidates: hits,
            searched,
            of: clouds.len(),
            work,
        }
    }
}

/// Result of walking a cloud's real children.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChildSearch {
    pub hits: Vec<(Address, BTreeMap<String, Value>)>,
    /// Index to resume from if the budget ran out; `None` when every child was looked at.
    pub resume_at: Option<u64>,
    pub generated: u64,
    pub work: u64,
}

impl ChildSearch {
    pub fn not_found_within_budget(&self) -> bool {
        self.hits.is_empty() && self.resume_at.is_some()
    }
}

/// Walk generated children from `start` in index order, resolving each
/// without recording it, until `max_hits` match or the budget runs out.
/// These are the real children: the same facts a later resolve records.
pub fn search_children(
    seed: &Seed,
    env: &Envelope,
    wish: &Wish,
    start: u64,
    work_budget: u64,
    max_hits: usize,
) -> Result<ChildSearch, EnvelopeError> {
    let kids = children(seed, env)?;
    let mut work = 0u64;
    let mut hits = Vec::new();
    let mut i = start;
    while i < kids.generated {
        if hits.len() >= max_hits || work >= work_budget {
            break;
        }
        let r = resolve(seed, env, &ChildRef::Generated(i))?;
        work += r.work;
        let ok = wish
            .criteria
            .iter()
            .all(|c| r.facts.get(&c.field).is_some_and(|v| value_matches(v, c)));
        if ok {
            hits.push((r.address, r.facts));
        }
        i += 1;
    }
    Ok(ChildSearch {
        hits,
        resume_at: (i < kids.generated).then_some(i),
        generated: kids.generated,
        work,
    })
}

pub const QUERY_PICK: &str = "query.pick";

/// Commit a pick: records that `address` was *selected for* `wish`, so a
/// hand-picked paradise is never mistaken for a typical system (H6.4).
pub fn commit_pick(
    chronicle: &mut Chronicle,
    address: &Address,
    wish: &Wish,
    at: UniverseTime,
    publisher: &Publisher,
) -> Result<ContentHash, crate::resolve::RecordError> {
    let mut fields = BTreeMap::new();
    fields.insert("query.selected_for".into(), Value::Text(wish.label.clone()));
    fields.insert("query.wish".into(), Value::Text(wish.hash().hex()));
    let mut set = BTreeMap::new();
    set.insert(address.clone(), fields);
    let causes = crate::resolve::recorded(chronicle, address)
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let ev = Event::new(EventBody {
        kind: QUERY_PICK.into(),
        subject: vec![address.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: chronicle.next_stamp(),
        causes,
        payload: Payload {
            set,
            data: Some(sse_core::cbor::to_value(wish).expect("wish encodes")),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: publisher.clone(),
    })?;
    Ok(chronicle.append(ev)?)
}
