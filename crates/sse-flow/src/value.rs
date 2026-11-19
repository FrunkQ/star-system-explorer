//! Typed values on wires (track J2, record design 3.1).
//!
//! The flow engine carried only numbers. Logic blocks also carry flags,
//! choices, references to addresses, text, finite lists and records. A port
//! with no declared shape is a `number`, so every existing atom and flow is
//! unchanged, and numbers still travel the original fast path.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// The shape a port carries. Written as text: `number`, `flag`, `choice`,
/// `ref`, `text`, `record`, `list<number>`, `list<list<ref>>`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Shape {
    Number,
    Flag,
    Choice,
    Ref,
    Text,
    Record,
    List(Box<Shape>),
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Number => f.write_str("number"),
            Shape::Flag => f.write_str("flag"),
            Shape::Choice => f.write_str("choice"),
            Shape::Ref => f.write_str("ref"),
            Shape::Text => f.write_str("text"),
            Shape::Record => f.write_str("record"),
            Shape::List(s) => write!(f, "list<{s}>"),
        }
    }
}

impl FromStr for Shape {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "number" => Shape::Number,
            "flag" => Shape::Flag,
            "choice" => Shape::Choice,
            "ref" => Shape::Ref,
            "text" => Shape::Text,
            "record" => Shape::Record,
            _ => match s.strip_prefix("list<").and_then(|r| r.strip_suffix('>')) {
                Some(inner) => Shape::List(Box::new(inner.parse()?)),
                None => return Err(format!("unknown shape {s:?}")),
            },
        })
    }
}

impl Serialize for Shape {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Shape {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A value on a wire.
#[derive(Clone, PartialEq, Debug)]
pub enum Val {
    Number(f64),
    Flag(bool),
    Choice(String),
    Ref(String),
    Text(String),
    List(Vec<Val>),
    Record(BTreeMap<String, Val>),
}

impl Val {
    pub fn num(&self) -> Option<f64> {
        match self {
            Val::Number(x) => Some(*x),
            _ => None,
        }
    }
    pub fn flag(&self) -> Option<bool> {
        match self {
            Val::Flag(b) => Some(*b),
            _ => None,
        }
    }
    pub fn text(&self) -> Option<&str> {
        match self {
            Val::Choice(s) | Val::Ref(s) | Val::Text(s) => Some(s),
            _ => None,
        }
    }
    pub fn list(&self) -> Option<&[Val]> {
        match self {
            Val::List(v) => Some(v),
            _ => None,
        }
    }
    /// Does this value fit `shape` (lists checked element by element)?
    pub fn fits(&self, shape: &Shape) -> bool {
        match (self, shape) {
            (Val::Number(x), Shape::Number) => x.is_finite(),
            (Val::Flag(_), Shape::Flag)
            | (Val::Choice(_), Shape::Choice)
            | (Val::Ref(_), Shape::Ref)
            | (Val::Text(_), Shape::Text)
            | (Val::Record(_), Shape::Record) => true,
            (Val::List(v), Shape::List(s)) => v.iter().all(|x| x.fits(s)),
            _ => false,
        }
    }
    /// The plain default of a shape: 0, false, empty text, empty list.
    pub fn zero(shape: &Shape) -> Val {
        match shape {
            Shape::Number => Val::Number(0.0),
            Shape::Flag => Val::Flag(false),
            Shape::Choice => Val::Choice(String::new()),
            Shape::Ref => Val::Ref(String::new()),
            Shape::Text => Val::Text(String::new()),
            Shape::Record => Val::Record(BTreeMap::new()),
            Shape::List(_) => Val::List(vec![]),
        }
    }
    /// Read a JSON value as `shape`.
    pub fn from_json(v: &serde_json::Value, shape: &Shape) -> Result<Val, String> {
        use serde_json::Value as J;
        Ok(match (shape, v) {
            (Shape::Number, J::Number(n)) => {
                Val::Number(n.as_f64().ok_or_else(|| format!("{n} is not a number"))?)
            }
            (Shape::Flag, J::Bool(b)) => Val::Flag(*b),
            (Shape::Choice, J::String(s)) => Val::Choice(s.clone()),
            (Shape::Ref, J::String(s)) => {
                s.parse::<sse_core::Address>()
                    .map_err(|e| format!("{s:?} is not an address: {e}"))?;
                Val::Ref(s.clone())
            }
            (Shape::Text, J::String(s)) => Val::Text(s.clone()),
            (Shape::List(inner), J::Array(a)) => Val::List(
                a.iter()
                    .map(|x| Val::from_json(x, inner))
                    .collect::<Result<_, _>>()?,
            ),
            (Shape::Record, J::Object(m)) => Val::Record(
                m.iter()
                    .map(|(k, x)| Ok((k.clone(), Val::guess(x)?)))
                    .collect::<Result<_, String>>()?,
            ),
            (s, v) => return Err(format!("expected a {s} value, found {v}")),
        })
    }
    /// A JSON value without a declared shape (record fields): numbers,
    /// flags, text, lists and records.
    pub fn guess(v: &serde_json::Value) -> Result<Val, String> {
        use serde_json::Value as J;
        Ok(match v {
            J::Number(n) => Val::Number(n.as_f64().ok_or("not a number")?),
            J::Bool(b) => Val::Flag(*b),
            J::String(s) => Val::Text(s.clone()),
            J::Array(a) => Val::List(a.iter().map(Val::guess).collect::<Result<_, _>>()?),
            J::Object(m) => Val::Record(
                m.iter()
                    .map(|(k, x)| Ok((k.clone(), Val::guess(x)?)))
                    .collect::<Result<_, String>>()?,
            ),
            J::Null => return Err("null is not a value".into()),
        })
    }
    /// A chronicle fact as a value of `shape`, if it is one.
    pub fn from_fact(v: &sse_core::cbor::Value, shape: &Shape) -> Option<Val> {
        use sse_core::cbor::Value as C;
        Some(match (shape, v) {
            (Shape::Number, C::Float(f)) => Val::Number(*f),
            (Shape::Number, C::Int(i)) => Val::Number(*i as f64),
            (Shape::Flag, C::Bool(b)) => Val::Flag(*b),
            (Shape::Choice, C::Text(t)) => Val::Choice(t.clone()),
            (Shape::Ref, C::Text(t)) => Val::Ref(t.clone()),
            (Shape::Text, C::Text(t)) => Val::Text(t.clone()),
            (Shape::List(inner), C::Array(a)) => Val::List(
                a.iter()
                    .map(|x| Val::from_fact(x, inner))
                    .collect::<Option<_>>()?,
            ),
            _ => return None,
        })
    }
}

impl Serialize for Val {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Val::Number(x) => s.serialize_f64(*x),
            Val::Flag(b) => s.serialize_bool(*b),
            Val::Choice(t) | Val::Ref(t) | Val::Text(t) => s.serialize_str(t),
            Val::List(v) => v.serialize(s),
            Val::Record(m) => m.serialize(s),
        }
    }
}
