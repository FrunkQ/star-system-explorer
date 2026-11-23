//! Sifters and significance (H6.6): declarative patterns over current facts
//! and the chronicle that surface things worth a GM's attention. Every hit
//! carries its evidence and a significance score from persistence, reach,
//! reversibility and rarity; rarity is measured from the cloud's own
//! distributions, never asserted.

use crate::predicate::{Predicate, Read, Subject};
use crate::rules::TagState;
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, State, UniverseTime};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::{Address, ContentHash};
use sse_resolve::query::{criterion_probability, Criterion};
use sse_resolve::Envelope;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "match", rename_all = "kebab-case")]
pub enum Pattern {
    /// Current facts of a subject satisfy a predicate.
    Facts { when: Predicate },
    /// An event of this kind affected the subject (optionally setting `field`).
    Event {
        kind: String,
        #[serde(default)]
        field: Option<String>,
    },
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// How long the situation lasts, in seconds.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum Persistence {
    /// A set duration: `s` seconds plus `yr` Julian years (the sheet's
    /// `yr_julian`), so a sifter never types a year in seconds.
    Fixed {
        #[serde(default)]
        s: f64,
        #[serde(default, skip_serializing_if = "is_zero")]
        yr: f64,
    },
    /// A fact holding a duration in seconds.
    Field { field: String },
    /// Until the subject's life line ends (`life.ends_s` - now).
    LifeRemaining,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum Reach {
    Fixed {
        n: u64,
    },
    /// The subject and everything resolved under it.
    Descendants,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SignificanceSpec {
    pub persistence: Persistence,
    pub reach: Reach,
    /// 0 = permanent, 1 = trivially undone.
    pub reversibility: f64,
    /// Criteria whose probability under the parent's cloud measures rarity.
    #[serde(default)]
    pub rarity: Vec<Criterion>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sifter {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
    pub pattern: Pattern,
    pub significance: SignificanceSpec,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub address: Address,
    pub field: String,
    pub value: Value,
    /// The event that fixed this fact or matched the pattern, if known.
    #[serde(default)]
    pub event: Option<ContentHash>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Significance {
    pub score: f64,
    pub persistence_s: f64,
    pub reach: u64,
    pub reversibility: f64,
    /// Probability of the rare condition under the parent's cloud, if measured.
    pub rarity_p: Option<f64>,
    /// Normalised components in [0, 1]: persistence, reach, irreversibility, rarity.
    pub components: [f64; 4],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Hit {
    pub sifter: String,
    pub label: String,
    pub subject: Address,
    pub evidence: Vec<EvidenceItem>,
    pub significance: Significance,
}

/// What sifting reads. All read-only.
pub struct World<'a> {
    pub state: &'a State,
    pub chronicle: Option<&'a Chronicle>,
    /// Tags per address with the rule evidence behind them (from `RuleSet::apply`).
    pub tags: &'a BTreeMap<Address, BTreeMap<String, TagState>>,
    /// Clouds by address, for rarity (a subject's parent cloud).
    pub clouds: &'a BTreeMap<Address, Envelope>,
    pub now: UniverseTime,
}

const YEAR: f64 = sse_core::sheet::YR_JULIAN;

/// The four components and the score. Each component saturates at a stated
/// scale: persistence at 10 Gyr, reach at 10,000 addresses, rarity at one in
/// 10,000. The score is their mean.
pub fn score(
    persistence_s: f64,
    reach: u64,
    reversibility: f64,
    rarity_p: Option<f64>,
) -> Significance {
    let p = (m::log10(1.0 + persistence_s.max(0.0) / YEAR) / 10.0).clamp(0.0, 1.0);
    let r = (m::log10(1.0 + reach as f64) / 4.0).clamp(0.0, 1.0);
    let v = (1.0 - reversibility).clamp(0.0, 1.0);
    let x = rarity_p.map_or(0.0, |q| {
        if q <= 0.0 {
            1.0
        } else {
            (-m::log10(q) / 4.0).clamp(0.0, 1.0)
        }
    });
    Significance {
        score: (p + r + v + x) / 4.0,
        persistence_s,
        reach,
        reversibility,
        rarity_p,
        components: [p, r, v, x],
    }
}

fn kind_ok(s: &Sifter, a: &Address) -> bool {
    s.applies_to.is_empty() || s.applies_to.iter().any(|k| k == a.last().kind())
}

fn source_event(c: Option<&Chronicle>, a: &Address, field: &str) -> Option<ContentHash> {
    c?.events()
        .iter()
        .rev()
        .find(|e| {
            e.body
                .payload
                .set
                .get(a)
                .is_some_and(|f| f.contains_key(field))
        })
        .map(|e| e.id)
}

fn subject_of(w: &World, a: &Address) -> Subject {
    Subject {
        address: Some(a.clone()),
        facts: w.state.facts.get(a).cloned().unwrap_or_default(),
        parent: a
            .parent()
            .and_then(|p| w.state.facts.get(&p).cloned())
            .unwrap_or_default(),
        tags: w
            .tags
            .get(a)
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default(),
        now_s: w.now.s,
    }
}

fn significance(spec: &SignificanceSpec, w: &World, subj: &Subject, a: &Address) -> Significance {
    let persistence = match &spec.persistence {
        Persistence::Fixed { s, yr } => *s + *yr * sse_core::sheet::YR_JULIAN,
        Persistence::Field { field } => subj.num(field).unwrap_or(0.0),
        Persistence::LifeRemaining => subj
            .num("life.ends_s")
            .map_or(0.0, |e| (e - w.now.s as f64).max(0.0)),
    };
    let reach = match spec.reach {
        Reach::Fixed { n } => n,
        Reach::Descendants => w.state.facts.keys().filter(|k| a.contains(k)).count() as u64,
    };
    let rarity = if spec.rarity.is_empty() {
        None
    } else {
        a.parent().and_then(|p| w.clouds.get(&p)).map(|env| {
            spec.rarity
                .iter()
                .map(|c| criterion_probability(env, c).0)
                .product()
        })
    };
    score(persistence, reach, spec.reversibility, rarity)
}

/// Run sifters over every address at or under `scope`. Hits come best first
/// (ties: sifter id, then address), so the order is the same everywhere.
pub fn sift(sifters: &[Sifter], w: &World, scope: &Address) -> Vec<Hit> {
    let mut hits = Vec::new();
    for s in sifters {
        match &s.pattern {
            Pattern::Facts { when } => {
                for a in w
                    .state
                    .facts
                    .keys()
                    .filter(|a| scope.contains(a) && kind_ok(s, a))
                {
                    let subj = subject_of(w, a);
                    let e = when.eval(&subj, false);
                    if !e.holds {
                        continue;
                    }
                    // A tag stands for the facts its rule read: show those.
                    let mut reads: Vec<Read> = Vec::new();
                    for r in e.read {
                        match r
                            .field
                            .strip_prefix("tag:")
                            .and_then(|t| w.tags.get(a)?.get(t))
                        {
                            Some(ts) => reads.extend(ts.evidence.iter().cloned()),
                            None => reads.push(r),
                        }
                    }
                    let evidence = reads
                        .into_iter()
                        .map(|r| {
                            let (addr, field) = match r.field.strip_prefix("parent.") {
                                Some(f) => (a.parent().unwrap_or_else(|| a.clone()), f.to_string()),
                                None => (a.clone(), r.field.clone()),
                            };
                            let event = source_event(w.chronicle, &addr, &field);
                            EvidenceItem {
                                address: addr,
                                field,
                                value: r.value,
                                event,
                            }
                        })
                        .collect();
                    hits.push(Hit {
                        sifter: s.id.clone(),
                        label: s.label.clone(),
                        subject: a.clone(),
                        evidence,
                        significance: significance(&s.significance, w, &subj, a),
                    });
                }
            }
            Pattern::Event { kind, field } => {
                let Some(c) = w.chronicle else { continue };
                let mut per: BTreeMap<Address, Vec<EvidenceItem>> = BTreeMap::new();
                for e in c
                    .events()
                    .iter()
                    .filter(|e| &e.body.kind == kind && e.body.activated_at <= w.now)
                {
                    for a in e
                        .body
                        .subject
                        .iter()
                        .filter(|a| scope.contains(a) && kind_ok(s, a))
                    {
                        let set = e.body.payload.set.get(a);
                        let items: Vec<EvidenceItem> = match field {
                            Some(f) => match set.and_then(|m| m.get(f)) {
                                Some(v) => vec![EvidenceItem {
                                    address: a.clone(),
                                    field: f.clone(),
                                    value: v.clone(),
                                    event: Some(e.id),
                                }],
                                None => continue,
                            },
                            None => vec![EvidenceItem {
                                address: a.clone(),
                                field: "event".into(),
                                value: Value::Text(e.body.kind.clone()),
                                event: Some(e.id),
                            }],
                        };
                        per.entry(a.clone()).or_default().extend(items);
                    }
                }
                for (a, evidence) in per {
                    let subj = subject_of(w, &a);
                    hits.push(Hit {
                        sifter: s.id.clone(),
                        label: s.label.clone(),
                        subject: a.clone(),
                        evidence,
                        significance: significance(&s.significance, w, &subj, &a),
                    });
                }
            }
        }
    }
    hits.sort_by(|x, y| {
        y.significance
            .score
            .total_cmp(&x.significance.score)
            .then(x.sifter.cmp(&y.sifter))
            .then(x.subject.cmp(&y.subject))
    });
    hits
}
