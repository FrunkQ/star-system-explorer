//! Rule predicates: declarative conditions over one subject's facts (H6.6).
//!
//! A predicate reads facts by name. `parent.<field>` reads the parent's
//! facts. Every evaluation returns the facts it read, so a tag or a sifter
//! hit can always show its evidence.

use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::{BTreeMap, BTreeSet};

/// What a predicate is evaluated against.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Subject {
    pub address: Option<Address>,
    pub facts: BTreeMap<String, Value>,
    pub parent: BTreeMap<String, Value>,
    pub tags: BTreeSet<String>,
    /// Universe time of the reading, seconds (for `within`).
    pub now_s: i64,
}

impl Subject {
    pub fn new(facts: BTreeMap<String, Value>) -> Self {
        Subject {
            facts,
            ..Default::default()
        }
    }
    pub fn get(&self, field: &str) -> Option<&Value> {
        match field.strip_prefix("parent.") {
            Some(f) => self.parent.get(f),
            None => self.facts.get(field),
        }
    }
    pub fn num(&self, field: &str) -> Option<f64> {
        as_num(self.get(field)?)
    }
}

pub fn as_num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Op {
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "if", rename_all = "kebab-case")]
pub enum Predicate {
    All {
        of: Vec<Predicate>,
    },
    Any {
        of: Vec<Predicate>,
    },
    Not {
        p: Box<Predicate>,
    },
    /// The fact exists.
    Has {
        field: String,
    },
    /// Numeric range, inclusive. `margin` widens the range for staying in a
    /// state once entered (hysteresis), in the field's own units.
    Range {
        field: String,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        margin: f64,
    },
    /// Equal to a text, number or boolean.
    Is {
        field: String,
        value: Value,
    },
    /// `left op factor * right`, e.g. orbit < 1.0 * roche_limit.
    Compare {
        left: String,
        op: Op,
        right: String,
        #[serde(default = "one")]
        factor: f64,
        /// Relative hysteresis on the factor once entered.
        #[serde(default)]
        margin: f64,
    },
    /// |a - b| <= tolerance * |b| (a day equal to a year).
    Near {
        a: String,
        b: String,
        tolerance: f64,
    },
    /// The subject carries a tag.
    Tag {
        tag: String,
    },
    /// A time fact (seconds) falls between now + `from_s` and now + `to_s`
    /// ("this star dies within the campaign").
    Within {
        field: String,
        from_s: i64,
        to_s: i64,
    },
}
fn one() -> f64 {
    1.0
}

/// One fact a predicate read.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Read {
    pub field: String,
    pub value: Value,
}

/// Result of an evaluation with its evidence.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Eval {
    pub holds: bool,
    pub read: Vec<Read>,
}

impl Predicate {
    /// Evaluate. `staying` selects the widened (hysteresis) form.
    pub fn eval(&self, s: &Subject, staying: bool) -> Eval {
        let mut read = Vec::new();
        let holds = self.go(s, staying, &mut read);
        read.sort_by(|a, b| a.field.cmp(&b.field));
        read.dedup();
        Eval { holds, read }
    }

    fn go(&self, s: &Subject, staying: bool, read: &mut Vec<Read>) -> bool {
        let note = |f: &str, read: &mut Vec<Read>| {
            if let Some(v) = s.get(f) {
                read.push(Read {
                    field: f.into(),
                    value: v.clone(),
                });
            }
        };
        match self {
            Predicate::All { of } => of.iter().all(|p| p.go(s, staying, read)),
            Predicate::Any { of } => of.iter().any(|p| p.go(s, staying, read)),
            Predicate::Not { p } => !p.go(s, staying, read),
            Predicate::Has { field } => {
                note(field, read);
                s.get(field).is_some()
            }
            Predicate::Range {
                field,
                min,
                max,
                margin,
            } => {
                note(field, read);
                let m = if staying { *margin } else { 0.0 };
                s.num(field).is_some_and(|x| {
                    min.is_none_or(|lo| x >= lo - m) && max.is_none_or(|hi| x <= hi + m)
                })
            }
            Predicate::Is { field, value } => {
                note(field, read);
                match (s.get(field), value) {
                    (Some(a), b) if a == b => true,
                    (Some(a), b) => matches!((as_num(a), as_num(b)), (Some(x), Some(y)) if x == y),
                    _ => false,
                }
            }
            Predicate::Compare {
                left,
                op,
                right,
                factor,
                margin,
            } => {
                note(left, read);
                note(right, read);
                let (Some(l), Some(r)) = (s.num(left), s.num(right)) else {
                    return false;
                };
                let f = if staying {
                    match op {
                        Op::Lt | Op::Le => factor * (1.0 + margin),
                        Op::Gt | Op::Ge => factor * (1.0 - margin),
                    }
                } else {
                    *factor
                };
                let rhs = f * r;
                match op {
                    Op::Lt => l < rhs,
                    Op::Le => l <= rhs,
                    Op::Gt => l > rhs,
                    Op::Ge => l >= rhs,
                }
            }
            Predicate::Near { a, b, tolerance } => {
                note(a, read);
                note(b, read);
                matches!((s.num(a), s.num(b)), (Some(x), Some(y)) if (x - y).abs() <= tolerance * y.abs())
            }
            Predicate::Tag { tag } => {
                if s.tags.contains(tag) {
                    read.push(Read {
                        field: format!("tag:{tag}"),
                        value: Value::Bool(true),
                    });
                }
                s.tags.contains(tag)
            }
            Predicate::Within {
                field,
                from_s,
                to_s,
            } => {
                note(field, read);
                s.num(field).is_some_and(|t| {
                    let t = t as i64;
                    t >= s.now_s.saturating_add(*from_s) && t <= s.now_s.saturating_add(*to_s)
                })
            }
        }
    }
}
