//! The logic atom set (record design 3.2). Small and fixed, shipped in the
//! core, each with explain text. Marked `class: "logic"`: the citation rule
//! for physics atoms does not apply, and the flow editor draws them apart.
//!
//! A node's `config` holds a logic block's settings (the field a read takes,
//! a compare's hysteresis margin, how many inputs an `and` has). Where the
//! config changes the ports (a read's shape, an `and`'s arity), the node's
//! ports follow it; the declaration shows the default.
//!
//! No loops and no recursion. List blocks work over finite lists; a list
//! longer than [`MAX_LIST`] is a reported error, never a silent truncation.

use serde::Deserialize;
use serde_json::Value as J;
use sse_core::cbor::Value as C;
use sse_core::math as m;
use sse_core::quantity::parse_unit;
use sse_core::{Address, Dimension};
use sse_flow::{
    Atom, AtomDecl, AtomError, BlockClass, Call, CostClass, DimRule, Out, Port, Shape, Val,
};
use sse_sift::predicate::{Predicate, Subject};
use std::collections::{BTreeMap, BTreeSet};

/// Longest list a list block takes (sets the cost class: light).
pub const MAX_LIST: usize = 4096;
/// Most inputs an `and`, `or` or `count_true` takes.
pub const MAX_ARITY: usize = 16;
const LETTERS: [&str; MAX_ARITY] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Less,
    AtMost,
    Greater,
    AtLeast,
    Equal,
    Near,
    InRange,
    And,
    Or,
    Not,
    CountTrue,
    Select,
    Switch,
    Lookup,
    FirstMatch,
    Add,
    Subtract,
    Multiply,
    Divide,
    Min,
    Max,
    Clamp,
    Round,
    Floor,
    Ceiling,
    Trunc,
    Abs,
    Count,
    Sum,
    Any,
    All,
    Filter,
    Sort,
    Take,
    Read,
    ReadKept,
    HasFact,
    HasTag,
    Children,
    Now,
    Staying,
    Const,
    Power,
    Sqrt,
    Cbrt,
    Exp,
    Expm1,
    Ln,
    Log10,
    Sin,
    Cos,
    Tan,
    Atan2,
}

/// Every op with its id and explain text (curious, science, formula).
const OPS: &[(Op, &str, &str, &str, &str)] = &[
    (Op::Less, "logic.less", "True when the first value is below the second.", "a < factor x b. Once true, with hysteresis the factor widens by (1 + margin) so the answer does not flicker at the edge.", "a < f b"),
    (Op::AtMost, "logic.at_most", "True when the first value is no more than the second.", "a <= factor x b, widened by (1 + margin) while staying.", "a <= f b"),
    (Op::Greater, "logic.greater", "True when the first value is above the second.", "a > factor x b, widened by (1 - margin) while staying.", "a > f b"),
    (Op::AtLeast, "logic.at_least", "True when the first value is at least the second.", "a >= factor x b, widened by (1 - margin) while staying.", "a >= f b"),
    (Op::Equal, "logic.equal", "True when two values are the same.", "Exact equality of two values of one shape.", "a = b"),
    (Op::Near, "logic.near", "True when two values are close (a day nearly a year long).", "|a - b| <= tolerance x |b|; the tolerance grows by margin while staying.", "|a - b| <= t |b|"),
    (Op::InRange, "logic.in_range", "True when a value sits between two limits.", "min <= x <= max, inclusive; each limit moves out by margin while staying.", "min <= x <= max"),
    (Op::And, "logic.and", "True when every input is true.", "Reads its inputs in order and stops at the first false one; only the inputs it read are evidence.", "a and b and ..."),
    (Op::Or, "logic.or", "True when any input is true.", "Reads its inputs in order and stops at the first true one; only the inputs it read are evidence.", "a or b or ..."),
    (Op::Not, "logic.not", "The opposite of its input.", "Logical negation.", "not a"),
    (Op::CountTrue, "logic.count_true", "How many of its inputs are true.", "The number of true inputs.", "sum [a_i]"),
    (Op::Select, "logic.select", "Picks one of two values by a condition.", "If, then, else. Only the condition and the branch taken are evidence.", "if c then a else b"),
    (Op::Switch, "logic.switch", "Picks a value by a choice.", "One input per listed case, and `otherwise` for the rest. Only the choice and the case taken are evidence.", "case on of ..."),
    (Op::Lookup, "logic.lookup", "Looks a choice up in a table.", "A data table in the node's config, from choice to value, with an optional default.", "table[key]"),
    (Op::FirstMatch, "logic.first_match", "The first thing in a list that meets a condition.", "The condition is the same language as classification rules (K8), evaluated on each record in order.", "first x in items where c(x)"),
    (Op::Add, "logic.add", "Adds two values.", "a + b; both carry one dimension.", "a + b"),
    (Op::Subtract, "logic.subtract", "Takes one value from another.", "a - b; both carry one dimension.", "a - b"),
    (Op::Multiply, "logic.multiply", "Multiplies two values.", "a x b; the dimensions multiply.", "a b"),
    (Op::Divide, "logic.divide", "Divides one value by another.", "a / b; the dimensions divide. Dividing by zero is an error, never a value.", "a / b"),
    (Op::Min, "logic.min", "The smaller of two values.", "min(a, b).", "min(a, b)"),
    (Op::Max, "logic.max", "The larger of two values.", "max(a, b).", "max(a, b)"),
    (Op::Clamp, "logic.clamp", "Keeps a value between two limits.", "min(max(x, lo), hi); lo must not exceed hi.", "clamp(x, lo, hi)"),
    (Op::Round, "logic.round", "Rounds to the nearest whole number.", "Half away from zero, in SI units (pinned maths).", "round(x)"),
    (Op::Floor, "logic.floor", "Rounds down.", "The largest whole number not above x (pinned maths).", "floor(x)"),
    (Op::Ceiling, "logic.ceiling", "Rounds up.", "The smallest whole number not below x (pinned maths).", "ceil(x)"),
    (Op::Trunc, "logic.trunc", "Drops the fraction.", "Rounds toward zero (pinned maths).", "trunc(x)"),
    (Op::Abs, "logic.abs", "The size of a value, without its sign.", "|x|.", "|x|"),
    (Op::Count, "logic.count", "How many things a list holds.", "The length of a finite list.", "len(items)"),
    (Op::Sum, "logic.sum", "Adds up a list of numbers.", "Summed in list order, so the result is the same on every device.", "sum(items)"),
    (Op::Any, "logic.any", "True when any flag in a list is true.", "Empty lists give false.", "any(items)"),
    (Op::All, "logic.all", "True when every flag in a list is true.", "Empty lists give true.", "all(items)"),
    (Op::Filter, "logic.filter", "Keeps the things in a list that meet a condition.", "The condition is the same language as classification rules (K8), evaluated on each record; order is kept.", "[x in items where c(x)]"),
    (Op::Sort, "logic.sort", "Puts a list in order by one field.", "Numbers before text, records without the field last; ties broken by address, then by the original order.", "sort items by key"),
    (Op::Take, "logic.take", "The first N things of a list.", "At most N; a shorter list is returned whole.", "items[..n]"),
    (Op::Read, "world.read", "Reads one fact about the subject (or a named place).", "The fact's value as the node's shape, and whether it is there in that shape. The read is evidence.", "fact(field)"),
    (Op::ReadKept, "world.read_kept", "Reads a value the campaign record keeps.", "A kept value (track J1) on the GM's machine. Physics never reads the record.", "kept(kind)"),
    (Op::HasFact, "world.has_fact", "True when the subject has a fact.", "The fact exists, whatever its value.", "has(field)"),
    (Op::HasTag, "world.has_tag", "True when the subject carries a tag.", "A tag from the rules (K8) or the record.", "tag(t)"),
    (Op::Children, "world.children", "The things directly inside a place.", "Child addresses in address order.", "children(of)"),
    (Op::Now, "world.now", "The time of the reading.", "Universe time, seconds.", "now"),
    (Op::Staying, "world.staying", "True when the condition already held (hysteresis).", "Lets a block use a wider limit for staying in a state once entered.", "staying"),
    (Op::Const, "logic.const", "A fixed value.", "A constant of any shape, written in the node's config; a number may carry a unit.", "k"),
    (Op::Power, "math.power", "Raises a value to a power.", "x^n with sse_core::math::pow. A whole-number or fractional exponent in the config (`5`, `1/4`) keeps the dimension (metres squared from metres); a wired exponent takes pure numbers only.", "x^n"),
    (Op::Sqrt, "math.sqrt", "The square root of a value.", "IEEE correctly rounded; the dimension halves (a negative value is an error).", "sqrt(x)"),
    (Op::Cbrt, "math.cbrt", "The cube root of a value: the side of a cube from its volume.", "sse_core::math::cbrt (pinned maths, not x^(1/3), which differs in the last bits); the dimension thirds; a negative value gives a negative root.", "cbrt(x)"),
    (Op::Exp, "math.exp", "e raised to a value: how growth compounds.", "exp(x) of a pure number (pinned maths).", "exp(x)"),
    (Op::Expm1, "math.expm1", "e raised to a value, less one, kept accurate when the value is small.", "exp(x) - 1 without the loss of digits near zero (pinned maths); the Planck law uses it.", "exp(x) - 1"),
    (Op::Ln, "math.ln", "The natural logarithm: the power e must be raised to.", "ln(x) of a pure number above zero (pinned maths).", "ln(x)"),
    (Op::Log10, "math.log10", "The base-10 logarithm: how many tens multiply to a value.", "log10(x) of a pure number above zero (pinned maths).", "log10(x)"),
    (Op::Sin, "math.sin", "The sine of an angle.", "sin(x), x in radians (pinned maths).", "sin(x)"),
    (Op::Cos, "math.cos", "The cosine of an angle.", "cos(x), x in radians (pinned maths).", "cos(x)"),
    (Op::Tan, "math.tan", "The tangent of an angle.", "tan(x), x in radians (pinned maths).", "tan(x)"),
    (Op::Atan2, "math.atan2", "The angle of a direction, from its two parts.", "atan2(y, x) in radians, -pi to pi; both parts carry one dimension (pinned maths).", "atan2(y, x)"),
];

/// Settings a logic block may take in its node's `config`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cfg {
    inputs: Option<usize>,
    shape: Option<Shape>,
    field: Option<String>,
    address: Option<String>,
    kind: Option<String>,
    tag: Option<String>,
    value: Option<J>,
    factor: Option<f64>,
    margin: Option<f64>,
    tolerance: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    cases: Option<Vec<String>>,
    table: Option<BTreeMap<String, J>>,
    default: Option<J>,
    #[serde(rename = "where")]
    condition: Option<Predicate>,
    key: Option<String>,
    descending: Option<bool>,
    n: Option<usize>,
    unit: Option<String>,
    exponent: Option<String>,
}

impl Op {
    /// Config keys this op takes.
    fn keys(self) -> &'static [&'static str] {
        use Op::*;
        match self {
            Less | AtMost | Greater | AtLeast => &["factor", "margin"],
            Equal => &["shape"],
            Near => &["tolerance", "margin"],
            InRange => &["min", "max", "margin"],
            And | Or | CountTrue => &["inputs"],
            Select => &["shape"],
            Switch => &["cases", "shape"],
            Lookup => &["table", "shape", "default"],
            FirstMatch | Filter => &["where"],
            Sort => &["key", "descending"],
            Take => &["n", "shape"],
            Count => &["shape"],
            Read => &["field", "shape", "address"],
            ReadKept => &["kind", "shape", "address"],
            HasFact => &["field", "address"],
            HasTag => &["tag", "address"],
            Const => &["shape", "value", "unit"],
            Power => &["exponent"],
            _ => &[],
        }
    }
    fn required(self) -> &'static [&'static str] {
        use Op::*;
        match self {
            Near => &["tolerance"],
            Switch => &["cases"],
            Lookup => &["table"],
            FirstMatch | Filter => &["where"],
            Sort => &["key"],
            Take => &["n"],
            Read | HasFact => &["field"],
            ReadKept => &["kind"],
            HasTag => &["tag"],
            Const => &["value"],
            _ => &[],
        }
    }
}

fn port(name: &str, shape: Shape) -> Port {
    Port {
        name: name.into(),
        kind: shape.to_string(),
        dim: Dimension::NONE,
        valid: None,
        shape: (shape != Shape::Number).then_some(shape),
        any_dim: false,
    }
}
/// A number port of any dimension (or a port of another shape).
fn any(name: &str, shape: &Shape) -> Port {
    let mut p = port(name, shape.clone());
    p.any_dim = *shape == Shape::Number;
    p
}
fn flag(name: &str) -> Port {
    port(name, Shape::Flag)
}
fn list(of: Shape) -> Shape {
    Shape::List(Box::new(of))
}

pub struct Block {
    op: Op,
    decl: AtomDecl,
}

impl Block {
    fn new(op: Op, id: &str, curious: &str, science: &str, formula: &str) -> Self {
        let (inputs, outputs) = ports(op, &Cfg::default());
        let dims = match op {
            Op::Multiply => Some(DimRule::Product),
            Op::Divide => Some(DimRule::Quotient),
            Op::Sqrt => Some(DimRule::Power(1, 2)),
            Op::Cbrt => Some(DimRule::Power(1, 3)),
            _ => None,
        };
        let cost = match op {
            Op::FirstMatch
            | Op::Filter
            | Op::Sort
            | Op::Count
            | Op::Sum
            | Op::Any
            | Op::All
            | Op::Take
            | Op::Children => CostClass::Light,
            _ => CostClass::Trivial,
        };
        Block {
            op,
            decl: AtomDecl {
                id: id.into(),
                version: "1.0.0".into(),
                inputs,
                outputs,
                params: vec![],
                cost_class: cost,
                references: vec![],
                explain: sse_flow::decl::ExplainText {
                    curious: curious.into(),
                    science: science.into(),
                    formula: formula.into(),
                },
                class: Some(BlockClass::Logic),
                dims,
            },
        }
    }
    pub fn op(&self) -> Op {
        self.op
    }
}

/// The logic atom set.
pub fn all() -> Vec<Box<dyn Atom>> {
    OPS.iter()
        .map(|(op, id, c, s, f)| Box::new(Block::new(*op, id, c, s, f)) as Box<dyn Atom>)
        .collect()
}

/// The atom id of an op.
pub fn id_of(op: Op) -> &'static str {
    OPS.iter().find(|x| x.0 == op).expect("every op listed").1
}

/// Ports of an op under a config.
fn ports(op: Op, c: &Cfg) -> (Vec<Port>, Vec<Port>) {
    use Op::*;
    let shape = c.shape.clone().unwrap_or(Shape::Number);
    let n = c.inputs.unwrap_or(2);
    let num = Shape::Number;
    match op {
        Less | AtMost | Greater | AtLeast | Near => {
            (vec![any("a", &num), any("b", &num)], vec![flag("holds")])
        }
        Equal => (
            vec![any("a", &shape), any("b", &shape)],
            vec![flag("holds")],
        ),
        InRange => (vec![any("x", &num)], vec![flag("holds")]),
        And | Or => (
            LETTERS.iter().take(n).map(|l| flag(l)).collect(),
            vec![flag("holds")],
        ),
        CountTrue => (
            LETTERS.iter().take(n).map(|l| flag(l)).collect(),
            vec![port("count", num)],
        ),
        Not => (vec![flag("a")], vec![flag("holds")]),
        Select => (
            vec![flag("if"), any("then", &shape), any("else", &shape)],
            vec![any("value", &shape)],
        ),
        Switch => {
            let mut ins = vec![port("on", Shape::Choice)];
            let k = c.cases.as_ref().map_or(0, Vec::len);
            ins.extend((0..k).map(|i| any(&format!("case{i}"), &shape)));
            ins.push(any("otherwise", &shape));
            (ins, vec![any("value", &shape)])
        }
        Lookup => (
            vec![port("key", Shape::Choice)],
            vec![port("value", shape), flag("found")],
        ),
        FirstMatch => (
            vec![port("items", list(Shape::Record))],
            vec![
                port("item", Shape::Record),
                flag("found"),
                port("index", num),
            ],
        ),
        Add | Subtract | Min | Max => (
            vec![any("a", &num), any("b", &num)],
            vec![any("value", &num)],
        ),
        Multiply | Divide => (
            vec![any("a", &num), any("b", &num)],
            vec![any("value", &num)],
        ),
        Clamp => (
            vec![any("x", &num), any("lo", &num), any("hi", &num)],
            vec![any("value", &num)],
        ),
        Round | Floor | Ceiling | Trunc | Abs => (vec![any("x", &num)], vec![any("value", &num)]),
        Count => (vec![port("items", list(shape))], vec![port("count", num)]),
        Sum => (
            vec![port("items", list(num.clone()))],
            vec![port("sum", num)],
        ),
        Any | All => (vec![port("items", list(Shape::Flag))], vec![flag("holds")]),
        Filter | Sort => (
            vec![port("items", list(Shape::Record))],
            vec![port("items", list(Shape::Record))],
        ),
        Take => (
            vec![port("items", list(shape.clone()))],
            vec![port("items", list(shape))],
        ),
        Read | ReadKept => (vec![], vec![port("value", shape), flag("present")]),
        HasFact | HasTag => (vec![], vec![flag("holds")]),
        Children => (
            vec![port("of", Shape::Ref)],
            vec![port("children", list(Shape::Ref))],
        ),
        Now => (vec![], vec![port("now", num)]),
        Staying => (vec![], vec![flag("staying")]),
        Const => {
            let mut p = port("value", shape);
            if let Some(d) = c.unit.as_deref().and_then(|u| parse_unit(u).ok()) {
                p.dim = d.1;
            }
            (vec![], vec![p])
        }
        Power if c.exponent.is_some() => (vec![any("x", &num)], vec![any("value", &num)]),
        Power => (
            vec![port("x", num.clone()), port("n", num.clone())],
            vec![port("value", num)],
        ),
        Sqrt | Cbrt => (vec![any("x", &num)], vec![any("value", &num)]),
        Exp | Expm1 | Ln | Log10 | Sin | Cos | Tan => {
            (vec![port("x", num.clone())], vec![port("value", num)])
        }
        Atan2 => (
            vec![any("y", &num), any("x", &num)],
            vec![port("angle", num)],
        ),
    }
}

/// A small rational exponent written as text (`5`, `-3/2`, `1/4`): the
/// whole numbers and their ratios a formula may hold (equation design 4).
pub fn exponent(text: &str) -> Result<(i8, i8), String> {
    let (a, b) = text.split_once('/').unwrap_or((text, "1"));
    let p: i8 = a
        .trim()
        .parse()
        .map_err(|_| format!("exponent {text:?} is not a whole number or a ratio of two"))?;
    let q: i8 = b
        .trim()
        .parse()
        .map_err(|_| format!("exponent {text:?} is not a whole number or a ratio of two"))?;
    if q <= 0 {
        return Err(format!("exponent {text:?}: the bottom must be above zero"));
    }
    Ok((p, q))
}

/// The pinned maths of the number blocks, shared with the Equation block so
/// a formula and its blocks give the same bits and the same refusals. A
/// result that is not a finite number is an error that names the step.
pub fn number(op: Op, x: &[f64]) -> Result<f64, String> {
    match sse_formula::Func::ALL
        .iter()
        .find(|f| crate::equation::op_of(**f) == op)
    {
        Some(f) => sse_formula::number(*f, x),
        None => Err(format!("{} is not a number block", id_of(op))),
    }
}

fn parse(op: Op, config: Option<&J>) -> Result<Cfg, String> {
    let Some(j) = config else {
        if let Some(k) = op.required().first() {
            return Err(format!("{} needs `{k}` in its config", id_of(op)));
        }
        return Ok(Cfg::default());
    };
    let obj = j
        .as_object()
        .ok_or_else(|| format!("{}: config must be an object", id_of(op)))?;
    for k in obj.keys() {
        if !op.keys().contains(&k.as_str()) {
            return Err(format!(
                "{} takes no `{k}` (it takes {})",
                id_of(op),
                if op.keys().is_empty() {
                    "no config".to_string()
                } else {
                    op.keys().join(", ")
                }
            ));
        }
    }
    for k in op.required() {
        if !obj.contains_key(*k) {
            return Err(format!("{} needs `{k}` in its config", id_of(op)));
        }
    }
    let c: Cfg = serde_json::from_value(j.clone()).map_err(|e| format!("{}: {e}", id_of(op)))?;
    check(op, &c)?;
    Ok(c)
}

fn check(op: Op, c: &Cfg) -> Result<(), String> {
    let id = id_of(op);
    if let Some(n) = c.inputs {
        if !(2..=MAX_ARITY).contains(&n) {
            return Err(format!("{id}: inputs must be 2 to {MAX_ARITY}"));
        }
    }
    for (k, v) in [
        ("factor", c.factor),
        ("margin", c.margin),
        ("tolerance", c.tolerance),
        ("min", c.min),
        ("max", c.max),
    ] {
        if v.is_some_and(|x| !x.is_finite()) {
            return Err(format!("{id}: {k} must be a finite number"));
        }
    }
    if c.margin.is_some_and(|x| x < 0.0) || c.tolerance.is_some_and(|x| x < 0.0) {
        return Err(format!("{id}: margins and tolerances are not negative"));
    }
    if let (Some(a), Some(b)) = (c.min, c.max) {
        if a > b {
            return Err(format!("{id}: min {a} is above max {b}"));
        }
    }
    if let Some(cases) = &c.cases {
        let set: BTreeSet<&String> = cases.iter().collect();
        if cases.is_empty() || set.len() != cases.len() || cases.len() > 64 {
            return Err(format!("{id}: cases must be 1 to 64 distinct choices"));
        }
    }
    let shape = c.shape.clone().unwrap_or(Shape::Number);
    if let Some(t) = &c.table {
        for (k, v) in t {
            Val::from_json(v, &shape).map_err(|e| format!("{id}: table entry {k:?}: {e}"))?;
        }
    }
    if let Some(d) = &c.default {
        Val::from_json(d, &shape).map_err(|e| format!("{id}: default: {e}"))?;
    }
    if op == Op::Const {
        let v = c.value.as_ref().expect("required");
        Val::from_json(v, &shape).map_err(|e| format!("{id}: value: {e}"))?;
    }
    if let Some(a) = &c.address {
        a.parse::<Address>()
            .map_err(|e| format!("{id}: {a:?} is not an address: {e}"))?;
    }
    if let Some(u) = &c.unit {
        if shape != Shape::Number {
            return Err(format!("{id}: only a number carries a unit"));
        }
        parse_unit(u).map_err(|e| format!("{id}: {e}"))?;
    }
    if let Some(e) = &c.exponent {
        exponent(e).map_err(|e| format!("{id}: {e}"))?;
    }
    if c.n.is_some_and(|n| n > MAX_LIST) {
        return Err(format!("{id}: n is at most {MAX_LIST}"));
    }
    Ok(())
}

fn err(op: Op, m: impl std::fmt::Display) -> AtomError {
    AtomError(format!("{}: {m}", id_of(op)))
}

fn nums(op: Op, v: &[Val]) -> Result<Vec<f64>, AtomError> {
    v.iter()
        .map(|x| x.num().ok_or_else(|| err(op, "expected a number")))
        .collect()
}
fn flags(op: Op, v: &[Val]) -> Result<Vec<bool>, AtomError> {
    v.iter()
        .map(|x| x.flag().ok_or_else(|| err(op, "expected a flag")))
        .collect()
}
fn items(op: Op, v: &Val) -> Result<&[Val], AtomError> {
    let l = v.list().ok_or_else(|| err(op, "expected a list"))?;
    if l.len() > MAX_LIST {
        return Err(err(
            op,
            format!(
                "a list of {} is longer than the {MAX_LIST} a list block takes",
                l.len()
            ),
        ));
    }
    Ok(l)
}

/// A record value as a K8 subject (its fields as facts).
pub fn record_subject(v: &Val) -> Option<Subject> {
    let Val::Record(m) = v else { return None };
    let facts = m
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), to_cbor(v)?)))
        .collect();
    Some(Subject::new(facts))
}

/// A wire value as a chronicle value (records are not chronicle values).
pub fn to_cbor(v: &Val) -> Option<C> {
    Some(match v {
        Val::Number(x) => C::Float(*x),
        Val::Flag(b) => C::Bool(*b),
        Val::Choice(t) | Val::Ref(t) | Val::Text(t) => C::Text(t.clone()),
        Val::List(l) => C::Array(l.iter().map(to_cbor).collect::<Option<_>>()?),
        Val::Record(_) => return None,
    })
}

fn sort_key(v: &Val, key: &str) -> (u8, f64, String, String) {
    let Val::Record(m) = v else {
        return (2, 0.0, String::new(), String::new());
    };
    let addr = m
        .get("address")
        .and_then(|a| a.text())
        .unwrap_or("")
        .to_string();
    match m.get(key) {
        Some(Val::Number(x)) => (0, *x, String::new(), addr),
        Some(x) if x.text().is_some() => (1, 0.0, x.text().unwrap_or("").to_string(), addr),
        _ => (2, 0.0, String::new(), addr),
    }
}

impl Atom for Block {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, _inputs: &[f64], _params: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(err(
            self.op,
            "a logic block is evaluated through its typed form",
        ))
    }
    fn ports(&self, config: Option<&J>) -> Option<(Vec<Port>, Vec<Port>)> {
        let c = parse(self.op, config).unwrap_or_default();
        Some(ports(self.op, &c))
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        parse(self.op, config).map(|_| ())
    }
    fn dim_rule(&self, config: Option<&J>) -> Option<DimRule> {
        if self.op != Op::Power {
            return self.decl.dims;
        }
        let c = parse(self.op, config).ok()?;
        let (p, q) = exponent(c.exponent.as_deref()?).ok()?;
        Some(DimRule::Power(p, q))
    }
    fn eval_typed(&self, v: &[Val], _params: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        use Op::*;
        let op = self.op;
        let c = parse(op, cx.config).map_err(AtomError)?;
        let env = cx.env;
        let shape = c.shape.clone().unwrap_or(Shape::Number);
        let one = |x: Val| Ok(Out::values(vec![x]));
        let world = || {
            env.world
                .ok_or_else(|| err(op, "this block reads the world and has none here"))
        };
        let at: Option<Address> = c.address.as_ref().and_then(|a| a.parse().ok());
        let label = |f: &str| match &c.address {
            Some(a) => format!("{a}#{f}"),
            None => f.to_string(),
        };
        match op {
            Less | AtMost | Greater | AtLeast => {
                let x = nums(op, v)?;
                let (factor, margin) = (c.factor.unwrap_or(1.0), c.margin.unwrap_or(0.0));
                let f = if env.staying {
                    match op {
                        Less | AtMost => factor * (1.0 + margin),
                        _ => factor * (1.0 - margin),
                    }
                } else {
                    factor
                };
                let r = f * x[1];
                one(Val::Flag(match op {
                    Less => x[0] < r,
                    AtMost => x[0] <= r,
                    Greater => x[0] > r,
                    _ => x[0] >= r,
                }))
            }
            Equal => one(Val::Flag(v[0] == v[1])),
            Near => {
                let x = nums(op, v)?;
                let t = c.tolerance.unwrap_or(0.0)
                    + if env.staying {
                        c.margin.unwrap_or(0.0)
                    } else {
                        0.0
                    };
                one(Val::Flag((x[0] - x[1]).abs() <= t * x[1].abs()))
            }
            InRange => {
                let x = nums(op, v)?[0];
                let mg = if env.staying {
                    c.margin.unwrap_or(0.0)
                } else {
                    0.0
                };
                one(Val::Flag(
                    c.min.is_none_or(|lo| x >= lo - mg) && c.max.is_none_or(|hi| x <= hi + mg),
                ))
            }
            And | Or => {
                let b = flags(op, v)?;
                let stop = b.iter().position(|x| if op == And { !*x } else { *x });
                let holds = match op {
                    And => stop.is_none(),
                    _ => stop.is_some(),
                };
                let upto = stop.map_or(b.len(), |i| i + 1);
                Ok(Out {
                    values: vec![Val::Flag(holds)],
                    used: Some((0..upto).collect()),
                    ..Default::default()
                })
            }
            Not => one(Val::Flag(!flags(op, v)?[0])),
            CountTrue => one(Val::Number(
                flags(op, v)?.iter().filter(|x| **x).count() as f64
            )),
            Select => {
                let k = if flags(op, &v[..1])?[0] { 1 } else { 2 };
                Ok(Out {
                    values: vec![v[k].clone()],
                    used: Some(vec![0, k]),
                    ..Default::default()
                })
            }
            Switch => {
                let on = v[0].text().ok_or_else(|| err(op, "expected a choice"))?;
                let cases = c.cases.as_deref().unwrap_or_default();
                let k = cases
                    .iter()
                    .position(|x| x == on)
                    .map_or(v.len() - 1, |i| i + 1);
                Ok(Out {
                    values: vec![v[k].clone()],
                    used: Some(vec![0, k]),
                    ..Default::default()
                })
            }
            Lookup => {
                let key = v[0].text().ok_or_else(|| err(op, "expected a choice"))?;
                let t = c.table.as_ref().expect("required");
                let (val, found) = match t.get(key) {
                    Some(j) => (Val::from_json(j, &shape).map_err(|e| err(op, e))?, true),
                    None => match &c.default {
                        Some(d) => (Val::from_json(d, &shape).map_err(|e| err(op, e))?, false),
                        None => (Val::zero(&shape), false),
                    },
                };
                Ok(Out::values(vec![val, Val::Flag(found)]))
            }
            FirstMatch | Filter => {
                let p = c.condition.as_ref().expect("required");
                let l = items(op, &v[0])?;
                let hit = |x: &Val| {
                    record_subject(x).is_some_and(|mut s| {
                        s.now_s = env.now_s;
                        p.eval(&s, env.staying).holds
                    })
                };
                if op == Filter {
                    return one(Val::List(l.iter().filter(|x| hit(x)).cloned().collect()));
                }
                Ok(Out::values(match l.iter().position(hit) {
                    Some(i) => vec![l[i].clone(), Val::Flag(true), Val::Number(i as f64)],
                    None => vec![
                        Val::Record(BTreeMap::new()),
                        Val::Flag(false),
                        Val::Number(-1.0),
                    ],
                }))
            }
            Add | Subtract | Multiply | Divide | Min | Max | Clamp | Sqrt | Cbrt | Exp | Expm1
            | Ln | Log10 | Sin | Cos | Tan | Atan2 => one(Val::Number(
                number(op, &nums(op, v)?).map_err(|e| err(op, e))?,
            )),
            Power => {
                let mut x = nums(op, v)?;
                if let Some(e) = &c.exponent {
                    let (p, q) = exponent(e).map_err(|e| err(op, e))?;
                    x.push(f64::from(p) / f64::from(q));
                }
                one(Val::Number(number(op, &x).map_err(|e| err(op, e))?))
            }
            Round | Floor | Ceiling | Trunc | Abs => {
                let x = nums(op, v)?[0];
                one(Val::Number(match op {
                    Round => m::round(x),
                    Floor => m::floor(x),
                    Ceiling => m::ceil(x),
                    Trunc => m::trunc(x),
                    _ => number(op, &[x]).map_err(|e| err(op, e))?,
                }))
            }
            Count => one(Val::Number(items(op, &v[0])?.len() as f64)),
            Sum => {
                let l = items(op, &v[0])?;
                let mut s = 0.0;
                for x in nums(op, l)? {
                    s += x;
                }
                one(Val::Number(s))
            }
            Any | All => {
                let b = flags(op, items(op, &v[0])?)?;
                one(Val::Flag(if op == Any {
                    b.iter().any(|x| *x)
                } else {
                    b.iter().all(|x| *x)
                }))
            }
            Sort => {
                let key = c.key.as_deref().expect("required");
                let mut l: Vec<(usize, &Val)> = items(op, &v[0])?.iter().enumerate().collect();
                let desc = c.descending.unwrap_or(false);
                l.sort_by(|(i, a), (j, b)| {
                    let (ka, kb) = (sort_key(a, key), sort_key(b, key));
                    let mut o = ka.0.cmp(&kb.0);
                    if o.is_eq() {
                        o = ka.1.total_cmp(&kb.1).then_with(|| ka.2.cmp(&kb.2));
                        if desc {
                            o = o.reverse();
                        }
                    }
                    o.then_with(|| ka.3.cmp(&kb.3)).then_with(|| i.cmp(j))
                });
                one(Val::List(l.into_iter().map(|(_, x)| x.clone()).collect()))
            }
            Take => {
                let l = items(op, &v[0])?;
                let n = c.n.expect("required").min(l.len());
                one(Val::List(l[..n].to_vec()))
            }
            Read | ReadKept => {
                let w = world()?;
                let (name, got) = if op == Read {
                    let f = c.field.as_deref().expect("required");
                    (label(f), w.fact(at.as_ref(), f))
                } else {
                    let k = c.kind.as_deref().expect("required");
                    (label(&format!("kept:{k}")), w.kept(at.as_ref(), k))
                };
                let val = got.as_ref().and_then(|g| Val::from_fact(g, &shape));
                Ok(Out {
                    values: vec![
                        val.clone().unwrap_or_else(|| Val::zero(&shape)),
                        Val::Flag(val.is_some()),
                    ],
                    reads: got.into_iter().map(|g| (name.clone(), g)).collect(),
                    ..Default::default()
                })
            }
            HasFact => {
                let f = c.field.as_deref().expect("required");
                let got = world()?.fact(at.as_ref(), f);
                Ok(Out {
                    values: vec![Val::Flag(got.is_some())],
                    reads: got.into_iter().map(|g| (label(f), g)).collect(),
                    ..Default::default()
                })
            }
            HasTag => {
                let t = c.tag.as_deref().expect("required");
                let holds = world()?.has_tag(at.as_ref(), t);
                Ok(Out {
                    values: vec![Val::Flag(holds)],
                    reads: if holds {
                        vec![(label(&format!("tag:{t}")), C::Bool(true))]
                    } else {
                        vec![]
                    },
                    ..Default::default()
                })
            }
            Children => {
                let a: Address = v[0]
                    .text()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| err(op, "expected an address"))?;
                let mut kids = world()?.children(&a);
                kids.sort();
                one(Val::List(
                    kids.into_iter().map(|k| Val::Ref(k.to_string())).collect(),
                ))
            }
            Now => one(Val::Number(env.now_s as f64)),
            Staying => one(Val::Flag(env.staying)),
            Const => {
                let j = c.value.as_ref().expect("required");
                let v = Val::from_json(j, &shape).map_err(|e| err(op, e))?;
                one(match (v, c.unit.as_deref()) {
                    (Val::Number(x), Some(u)) => {
                        Val::Number(x * parse_unit(u).map_err(|e| err(op, e))?.0)
                    }
                    (v, _) => v,
                })
            }
        }
    }
}
