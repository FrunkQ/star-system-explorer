//! Formulas as data (package ND3, equation block design): the parser, the
//! unit and dimension checks, and the run, shared by the Equation block
//! (`sse-logic`) and the physics atoms whose maths is written as a formula
//! (`sse-atoms`, package ND3b). Moved out of `sse-logic` so the atoms below
//! it can be formulas too.
//!
//!
//! What a pack author writes (design section 2):
//!
//! ```json
//! { "atom": "sse.math.equation",
//!   "config": {
//!     "name": "How far carbon's colours shift when squeezed",
//!     "formula": "shift = alpha * density",
//!     "inputs":  { "density": { "kind": "atmosphere.photosphere_density", "unit": "g/cm3", "valid": [0, 2] },
//!                  "alpha":   { "kind": "model.swan_shift_per_density", "unit": "eV cm3/g" } },
//!     "outputs": { "shift": { "kind": "light.band_shift", "unit": "eV" } },
//!     "source":  { "text": "...", "doi": "...", "where": "eq. 3" },
//!     "explain": { "curious": "...", "science": "..." },
//!     "examples": [ { "density": 0.15, "alpha": 0.2, "shift": 0.03 } ] } }
//! ```
//!
//! The language (design section 3): the block's inputs and nothing else as
//! names; `+ - * / ^`, brackets, unary minus; `exp expm1 ln log10 sqrt cbrt abs
//! min max clamp sin cos tan atan2`; `if <condition> then a else b` with
//! comparisons joined by `and`, `or`, `not`; several lines `name = ...`, a
//! later line using an earlier one; `pi`. Bare numbers are whole numbers
//! from 0 to [`MAX_WHOLE`] and their ratios, because they are part of the
//! mathematics; anything else is a letter with a source (design section 4).
//!
//! Every input is a port, so every letter arrives on a visible wire: from a
//! block that worked it out for this world, a Look up block (a fact of
//! nature) or a Model number block (a number that belongs to the model). A
//! typed constant (`logic.const`) is refused on an Equation's input.
//!
//! On load: names, units and dimensions are checked through the formula,
//! the operations counted (at most [`MAX_OPS`]), and the worked examples
//! run. Running is a walk down the parsed formula through the same pinned
//! maths the number blocks use ([`crate::blocks::number`]), in the order
//! written: no rearranging, no fused multiply-add.
//!
//! A choice evaluates only the side it takes, and a line that is not an
//! output is worked out only when something needs it. So a guard (`if t <=
//! 0 then 0 else ...`) protects the side it guards. Blocks in a flow are
//! all evaluated, so an expansion whose untaken side cannot be worked out
//! (a division by zero there) reports that error where the Equation gives a
//! value; wherever the expansion runs, its results are the same bits.

pub mod atom;
pub use atom::FormulaAtom;

use serde::Deserialize;
use serde_json::Value as J;
use sse_core::quantity::parse_unit;
use sse_core::Dimension;
use sse_flow::{Port, Shape, Val};
use std::collections::{BTreeMap, BTreeSet};

/// The Equation block's id.
pub const ID: &str = "sse.math.equation";
/// Most operations one Equation holds (cost class trivial; design 8).
pub const MAX_OPS: usize = 64;
/// Largest whole number a formula may hold as a bare number.
pub const MAX_WHOLE: u32 = 32;
const FUNCS: [&str; 14] = [
    "exp", "expm1", "ln", "log10", "sqrt", "cbrt", "abs", "min", "max", "clamp", "sin", "cos",
    "tan", "atan2",
];
const WORDS: [&str; 7] = ["if", "then", "else", "and", "or", "not", "pi"];

// ---------------------------------------------------------------- config

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortSpec {
    /// The quantity kind the wire carries.
    pub kind: String,
    /// The unit, as text (`g/cm3`, `W m-2 K-4`, `1`).
    pub unit: String,
    /// The worded name shown beside the symbol ("distance from the star").
    #[serde(default)]
    pub name: Option<String>,
    /// Where the relation holds, in the port's unit.
    #[serde(default)]
    pub valid: Option<(f64, f64)>,
    /// A list (a spectrum on its wavelength grid): the formula runs once
    /// per entry and every output is a list of the same length.
    #[serde(default)]
    pub list: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    pub text: String,
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    /// Where in the source the relation is ("eq. 3", "table 2").
    #[serde(rename = "where", default)]
    pub at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainSpec {
    #[serde(default)]
    pub curious: String,
    #[serde(default)]
    pub science: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquationCfg {
    pub name: String,
    pub formula: String,
    pub inputs: BTreeMap<String, PortSpec>,
    pub outputs: BTreeMap<String, PortSpec>,
    #[serde(default)]
    pub source: Option<SourceSpec>,
    /// A bookkeeping Equation: needs no source (design 7).
    #[serde(default)]
    pub logic: bool,
    #[serde(default)]
    pub explain: ExplainSpec,
    /// Worked examples in the ports' units; `tolerance` (relative, default
    /// 1e-6) for a number quoted from a paper.
    #[serde(default)]
    pub examples: Vec<BTreeMap<String, J>>,
}

// ---------------------------------------------------------------- syntax

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bin {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmp {
    Less,
    AtMost,
    Greater,
    AtLeast,
    Equal,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    /// A bare whole number.
    Whole(u32),
    Pi,
    Input(usize),
    Line(usize),
    Neg(usize),
    Bin(Bin, usize, usize),
    Call(Func, Vec<usize>),
    If(usize, usize, usize),
    Cmp(Cmp, usize, usize),
    And(usize, usize),
    Or(usize, usize),
    Not(usize),
}

/// A parsed, checked Equation: the "short list of steps" of design 8B.
#[derive(Clone, Debug)]
pub struct Parsed {
    pub cfg: EquationCfg,
    /// Input symbols in port order.
    pub inputs: Vec<String>,
    pub in_dims: Vec<Dimension>,
    pub in_scales: Vec<f64>,
    pub outputs: Vec<String>,
    pub out_dims: Vec<Dimension>,
    pub out_scales: Vec<f64>,
    /// Every line in order: (name, root expression, output index if any).
    pub lines: Vec<(String, usize, Option<usize>)>,
    pub line_dims: Vec<Dimension>,
    pub exprs: Vec<Expr>,
    /// The dimension each bare zero takes from its place (for expansion).
    pub zero_dims: BTreeMap<usize, Dimension>,
    pub ops: usize,
    pub list: bool,
}

#[derive(Clone, PartialEq, Debug)]
enum Tok {
    Num(String),
    Id(String),
    Sym(&'static str),
}

fn lex(line: &str, ln: usize) -> Result<Vec<(Tok, usize)>, String> {
    let c: Vec<char> = line.chars().collect();
    let mut out = vec![];
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        let col = i + 1;
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_ascii_digit() || ch == '.' {
            let st = i;
            while i < c.len() && (c[i].is_ascii_alphanumeric() || c[i] == '.') {
                i += 1;
            }
            out.push((Tok::Num(c[st..i].iter().collect()), col));
        } else if ch.is_alphabetic() || ch == '_' {
            let st = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push((Tok::Id(c[st..i].iter().collect()), col));
        } else {
            let two: String = c[i..(i + 2).min(c.len())].iter().collect();
            let sym = match two.as_str() {
                "<=" => Some("<="),
                ">=" => Some(">="),
                "==" => Some("=="),
                _ => None,
            };
            if let Some(s) = sym {
                out.push((Tok::Sym(s), col));
                i += 2;
                continue;
            }
            let s = match ch {
                '+' => "+",
                '-' => "-",
                '*' => "*",
                '/' => "/",
                '^' => "^",
                '(' => "(",
                ')' => ")",
                ',' => ",",
                '<' => "<",
                '>' => ">",
                '=' => "=",
                _ => {
                    return Err(format!(
                        "line {ln}, column {col}: {ch:?} is not part of the formula language"
                    ))
                }
            };
            out.push((Tok::Sym(s), col));
            i += 1;
        }
    }
    Ok(out)
}

struct P<'a> {
    toks: Vec<(Tok, usize)>,
    i: usize,
    ln: usize,
    exprs: &'a mut Vec<Expr>,
    inputs: &'a [String],
    lines: &'a [String],
    used_inputs: &'a mut BTreeSet<usize>,
}

impl P<'_> {
    fn at(&self) -> String {
        match self.toks.get(self.i) {
            Some((_, c)) => format!("line {}, column {}", self.ln, c),
            None => format!("line {}, at its end", self.ln),
        }
    }
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i).map(|t| &t.0)
    }
    fn is(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
            || matches!(self.peek(), Some(Tok::Id(x)) if x == s)
    }
    fn eat(&mut self, s: &str) -> bool {
        if self.is(s) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn want(&mut self, s: &str) -> Result<(), String> {
        if self.eat(s) {
            Ok(())
        } else {
            Err(format!("{}: expected `{s}`", self.at()))
        }
    }
    fn push(&mut self, e: Expr) -> usize {
        self.exprs.push(e);
        self.exprs.len() - 1
    }
    fn expr(&mut self) -> Result<usize, String> {
        if self.eat("if") {
            let c = self.or()?;
            self.want("then")?;
            let a = self.expr()?;
            self.want("else")?;
            let b = self.expr()?;
            return Ok(self.push(Expr::If(c, a, b)));
        }
        self.sum()
    }
    fn sum(&mut self) -> Result<usize, String> {
        let mut a = self.product()?;
        loop {
            let op = if self.eat("+") {
                Bin::Add
            } else if self.eat("-") {
                Bin::Sub
            } else {
                return Ok(a);
            };
            let b = self.product()?;
            a = self.push(Expr::Bin(op, a, b));
        }
    }
    fn product(&mut self) -> Result<usize, String> {
        let mut a = self.unary()?;
        loop {
            let op = if self.eat("*") {
                Bin::Mul
            } else if self.eat("/") {
                Bin::Div
            } else {
                return Ok(a);
            };
            let b = self.unary()?;
            a = self.push(Expr::Bin(op, a, b));
        }
    }
    fn unary(&mut self) -> Result<usize, String> {
        if self.eat("-") {
            let a = self.unary()?;
            return Ok(self.push(Expr::Neg(a)));
        }
        self.power()
    }
    fn power(&mut self) -> Result<usize, String> {
        let a = self.atom()?;
        if self.eat("^") {
            // Right to left: a^b^c is a^(b^c); -a^2 is -(a^2).
            let b = self.unary()?;
            return Ok(self.push(Expr::Bin(Bin::Pow, a, b)));
        }
        Ok(a)
    }
    fn atom(&mut self) -> Result<usize, String> {
        let at = self.at();
        match self.toks.get(self.i).cloned() {
            Some((Tok::Sym("("), _)) => {
                self.i += 1;
                let e = self.expr()?;
                self.want(")")?;
                Ok(e)
            }
            Some((Tok::Num(t), _)) => {
                self.i += 1;
                match t.parse::<u32>() {
                    Ok(n) if n <= MAX_WHOLE => Ok(self.push(Expr::Whole(n))),
                    _ => Err(format!(
                        "{at}: the number {t} is not part of the mathematics (only whole numbers 0 to {MAX_WHOLE} and their ratios are): give this letter a source, a Look up or a Model number block wired in"
                    )),
                }
            }
            Some((Tok::Id(name), _)) => {
                self.i += 1;
                if name == "pi" {
                    return Ok(self.push(Expr::Pi));
                }
                if let Some(f) = FUNCS.iter().position(|f| *f == name) {
                    let op = [
                        Func::Exp,
                        Func::Expm1,
                        Func::Ln,
                        Func::Log10,
                        Func::Sqrt,
                        Func::Cbrt,
                        Func::Abs,
                        Func::Min,
                        Func::Max,
                        Func::Clamp,
                        Func::Sin,
                        Func::Cos,
                        Func::Tan,
                        Func::Atan2,
                    ][f];
                    let arity = match op {
                        Func::Min | Func::Max | Func::Atan2 => 2,
                        Func::Clamp => 3,
                        _ => 1,
                    };
                    self.want("(")?;
                    let mut args = vec![self.expr()?];
                    while self.eat(",") {
                        args.push(self.expr()?);
                    }
                    self.want(")")?;
                    if args.len() != arity {
                        return Err(format!(
                            "{at}: {name} takes {arity} value{}, not {}",
                            if arity == 1 { "" } else { "s" },
                            args.len()
                        ));
                    }
                    return Ok(self.push(Expr::Call(op, args)));
                }
                if WORDS.contains(&name.as_str()) {
                    return Err(format!("{at}: `{name}` is out of place here"));
                }
                if let Some(k) = self.lines.iter().position(|l| *l == name) {
                    return Ok(self.push(Expr::Line(k)));
                }
                if let Some(k) = self.inputs.iter().position(|l| *l == name) {
                    self.used_inputs.insert(k);
                    return Ok(self.push(Expr::Input(k)));
                }
                Err(format!(
                    "{at}: `{name}` has no line into this block: every letter is an input, wired from a block that works it out, a Look up block or a Model number block"
                ))
            }
            _ => Err(format!("{at}: expected a value")),
        }
    }
    fn or(&mut self) -> Result<usize, String> {
        let mut a = self.and()?;
        while self.eat("or") {
            let b = self.and()?;
            a = self.push(Expr::Or(a, b));
        }
        Ok(a)
    }
    fn and(&mut self) -> Result<usize, String> {
        let mut a = self.not()?;
        while self.eat("and") {
            let b = self.not()?;
            a = self.push(Expr::And(a, b));
        }
        Ok(a)
    }
    fn not(&mut self) -> Result<usize, String> {
        if self.eat("not") {
            let a = self.not()?;
            return Ok(self.push(Expr::Not(a)));
        }
        // A comparison, or a bracketed condition.
        let (mark, n) = (self.i, self.exprs.len());
        let used = self.used_inputs.clone();
        match self.compare() {
            Ok(c) => Ok(c),
            Err(e) => {
                self.i = mark;
                self.exprs.truncate(n);
                *self.used_inputs = used;
                if self.eat("(") {
                    let c = self.or()?;
                    self.want(")")?;
                    Ok(c)
                } else {
                    Err(e)
                }
            }
        }
    }
    fn compare(&mut self) -> Result<usize, String> {
        let a = self.sum()?;
        let op = [
            ("<=", Cmp::AtMost),
            (">=", Cmp::AtLeast),
            ("==", Cmp::Equal),
            ("<", Cmp::Less),
            (">", Cmp::Greater),
        ]
        .into_iter()
        .find(|(s, _)| self.is(s))
        .map(|(_, c)| c)
        .ok_or_else(|| {
            format!(
                "{}: a condition compares two values (<, <=, >, >=, ==)",
                self.at()
            )
        })?;
        self.i += 1;
        let b = self.sum()?;
        Ok(self.push(Expr::Cmp(op, a, b)))
    }
}

// ------------------------------------------------------------ dimensions

#[derive(Clone, Copy, PartialEq, Debug)]
enum Dv {
    D(Dimension),
    /// A bare zero: it takes the dimension of its place.
    Zero,
}

struct Dims<'a> {
    exprs: &'a [Expr],
    in_dims: &'a [Dimension],
    line_dims: &'a [Dimension],
    zero: BTreeMap<usize, Dimension>,
}

/// A fixed exponent as written: `n`, `-n`, `n/m`, `-n/m`, `-(n/m)`.
pub fn ratio(exprs: &[Expr], e: usize) -> Option<(i64, i64)> {
    match &exprs[e] {
        Expr::Whole(n) => Some((i64::from(*n), 1)),
        Expr::Neg(a) => ratio(exprs, *a).map(|(p, q)| (-p, q)),
        Expr::Bin(Bin::Div, a, b) => match (&exprs[*a], &exprs[*b]) {
            (Expr::Whole(p), Expr::Whole(q)) if *q > 0 => Some((i64::from(*p), i64::from(*q))),
            (Expr::Neg(x), Expr::Whole(q)) if *q > 0 => match exprs[*x] {
                Expr::Whole(p) => Some((-i64::from(p), i64::from(*q))),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

impl Dims<'_> {
    fn same(
        &mut self,
        a: usize,
        b: usize,
        want: Option<Dimension>,
        what: &str,
    ) -> Result<Dv, String> {
        let da = self.infer(a, want)?;
        let db = self.infer(
            b,
            match da {
                Dv::D(d) => Some(d),
                Dv::Zero => want,
            },
        )?;
        match (da, db) {
            (Dv::D(x), Dv::D(y)) if x != y => Err(format!(
                "{what} [{x}] and [{y}]: the two sides must carry one dimension"
            )),
            (Dv::Zero, Dv::D(y)) => {
                self.infer(a, Some(y))?;
                Ok(Dv::D(y))
            }
            (x, _) => Ok(x),
        }
    }
    fn pure(&mut self, a: usize, what: &str) -> Result<(), String> {
        match self.infer(a, Some(Dimension::NONE))? {
            Dv::D(d) if d != Dimension::NONE => Err(format!(
                "{what} of [{d}]: it takes a pure number (a ratio, an angle in radians)"
            )),
            _ => Ok(()),
        }
    }
    fn num(&mut self, a: usize) -> Result<Dimension, String> {
        Ok(match self.infer(a, None)? {
            Dv::D(d) => d,
            Dv::Zero => {
                self.infer(a, Some(Dimension::NONE))?;
                Dimension::NONE
            }
        })
    }
    fn infer(&mut self, e: usize, want: Option<Dimension>) -> Result<Dv, String> {
        Ok(match self.exprs[e].clone() {
            Expr::Whole(0) => match want {
                Some(d) => {
                    self.zero.insert(e, d);
                    Dv::D(d)
                }
                None => Dv::Zero,
            },
            Expr::Whole(_) | Expr::Pi => Dv::D(Dimension::NONE),
            Expr::Input(i) => Dv::D(self.in_dims[i]),
            Expr::Line(k) => Dv::D(self.line_dims[k]),
            Expr::Neg(a) => self.infer(a, want)?,
            Expr::Bin(Bin::Add | Bin::Sub, a, b) => self.same(a, b, want, "adding")?,
            Expr::Bin(op @ (Bin::Mul | Bin::Div), a, b) => {
                let (x, y) = (self.num(a)?, self.num(b)?);
                Dv::D(if op == Bin::Mul { x.mul(y) } else { x.div(y) })
            }
            Expr::Bin(Bin::Pow, a, b) => {
                self.pure(b, "a power")?;
                let d = self.num(a)?;
                if d == Dimension::NONE {
                    return Ok(Dv::D(d));
                }
                let (p, q) = ratio(self.exprs, b).ok_or_else(|| {
                    format!("[{d}] to a power: a value with a dimension takes a fixed power written as a whole number or a ratio (3, -2, 3/2)")
                })?;
                let (p8, q8) = (i8::try_from(p), i8::try_from(q));
                match (p8, q8) {
                    (Ok(p), Ok(q)) => Dv::D(d.powi(p).root(q).ok_or_else(|| {
                        format!("[{d}] to the power {p}/{q} is not a whole dimension")
                    })?),
                    _ => return Err(format!("the power {p}/{q} is too large")),
                }
            }
            Expr::Call(op, args) => match op {
                Func::Exp
                | Func::Expm1
                | Func::Ln
                | Func::Log10
                | Func::Sin
                | Func::Cos
                | Func::Tan => {
                    self.pure(args[0], op.id().trim_start_matches("math."))?;
                    Dv::D(Dimension::NONE)
                }
                Func::Sqrt => {
                    let d = self.num(args[0])?;
                    Dv::D(d.root(2).ok_or_else(|| {
                        format!("the square root of [{d}] is not a whole dimension")
                    })?)
                }
                Func::Cbrt => {
                    let d = self.num(args[0])?;
                    Dv::D(d.root(3).ok_or_else(|| {
                        format!("the cube root of [{d}] is not a whole dimension")
                    })?)
                }
                Func::Abs => self.infer(args[0], want)?,
                Func::Min | Func::Max => self.same(args[0], args[1], want, "comparing")?,
                Func::Clamp => {
                    let d = self.same(args[0], args[1], want, "clamping")?;
                    let w = match d {
                        Dv::D(d) => Some(d),
                        Dv::Zero => want,
                    };
                    let d2 = self.same(args[1], args[2], w, "clamping")?;
                    if let (Dv::Zero, Dv::D(x)) = (d, d2) {
                        self.same(args[0], args[1], Some(x), "clamping")?;
                    }
                    if d == Dv::Zero {
                        d2
                    } else {
                        d
                    }
                }
                Func::Atan2 => {
                    self.same(args[0], args[1], None, "an angle from")?;
                    Dv::D(Dimension::NONE)
                }
                _ => unreachable!("parser makes only these calls"),
            },
            Expr::If(c, a, b) => {
                self.infer(c, None)?;
                self.same(a, b, want, "a choice between")?
            }
            Expr::Cmp(_, a, b) => {
                if self.same(a, b, None, "comparing")? == Dv::Zero {
                    self.same(a, b, Some(Dimension::NONE), "comparing")?;
                }
                Dv::D(Dimension::NONE)
            }
            Expr::And(a, b) | Expr::Or(a, b) => {
                self.infer(a, None)?;
                self.infer(b, None)?;
                Dv::D(Dimension::NONE)
            }
            Expr::Not(a) => {
                self.infer(a, None)?;
                Dv::D(Dimension::NONE)
            }
        })
    }
}

fn unit_si(u: &str, at: &str) -> Result<(f64, Dimension), String> {
    parse_unit(u).map_err(|e| format!("{at}: {e}"))
}

/// A name a keen amateur reads (design 8A): words, not a symbol or an id.
fn worded(name: &str) -> bool {
    let words: Vec<&str> = name.split_whitespace().collect();
    words.len() >= 2
        && name.chars().any(char::is_alphabetic)
        && !name.contains(['_', '.', '/', '='])
}

impl Parsed {
    /// Parse and check an Equation's config: everything design section 10
    /// says is refused at load is refused here, in words that name the place.
    pub fn new(config: Option<&J>) -> Result<Parsed, String> {
        let j = config.ok_or("an Equation needs its config: name, formula, inputs, outputs")?;
        let cfg: EquationCfg =
            serde_json::from_value(j.clone()).map_err(|e| format!("Equation: {e}"))?;
        if !worded(&cfg.name) {
            return Err(format!(
                "Equation {:?}: its name says what happens in the world, in words (\"Speed needed to escape\"), not a symbol or an id",
                cfg.name
            ));
        }
        let me = format!("Equation {:?}", cfg.name);
        if cfg.explain.curious.trim().is_empty() || cfg.explain.science.trim().is_empty() {
            return Err(format!("{me}: explain needs a curious and a science line"));
        }
        if !cfg.logic {
            let s = cfg.source.as_ref().ok_or_else(|| {
                format!("{me}: a physics Equation carries its source (or is marked `logic` for bookkeeping)")
            })?;
            if s.doi.as_deref().is_none_or(str::is_empty)
                && s.url.as_deref().is_none_or(str::is_empty)
            {
                return Err(format!("{me}: its source needs a DOI or an address"));
            }
            if s.at.as_deref().is_none_or(str::is_empty) {
                return Err(format!(
                    "{me}: its source says where the relation is (\"eq. 3\", \"table 2\")"
                ));
            }
        }
        if cfg.examples.is_empty() {
            return Err(format!("{me}: at least one worked example, run on load"));
        }
        let mut inputs = vec![];
        let (mut in_dims, mut in_scales) = (vec![], vec![]);
        for (k, p) in &cfg.inputs {
            if FUNCS.contains(&k.as_str()) || WORDS.contains(&k.as_str()) || !ident(k) {
                return Err(format!("{me}: {k:?} cannot be a letter"));
            }
            let (s, d) = unit_si(&p.unit, &format!("{me}, input {k}"))?;
            inputs.push(k.clone());
            in_dims.push(d);
            in_scales.push(s);
        }
        let list = cfg.inputs.values().any(|p| p.list);
        let mut outputs = vec![];
        let (mut out_dims, mut out_scales) = (vec![], vec![]);
        for (k, p) in &cfg.outputs {
            if !ident(k) || cfg.inputs.contains_key(k) {
                return Err(format!("{me}: output {k:?} must be a new letter"));
            }
            if p.list {
                return Err(format!(
                    "{me}: output {k}: outputs are lists when an input is, so they take no `list`"
                ));
            }
            let (s, d) = unit_si(&p.unit, &format!("{me}, output {k}"))?;
            outputs.push(k.clone());
            out_dims.push(d);
            out_scales.push(s);
        }
        let mut exprs = vec![];
        let mut lines: Vec<(String, usize, Option<usize>)> = vec![];
        let mut names: Vec<String> = vec![];
        let mut used = BTreeSet::new();
        let mut line_dims = vec![];
        let mut zero = BTreeMap::new();
        for (li, text) in cfg.formula.split(['\n', ';']).enumerate() {
            let ln = li + 1;
            if text.trim().is_empty() {
                continue;
            }
            let toks = lex(text, ln).map_err(|e| format!("{me}: {e}"))?;
            let (Some((Tok::Id(name), _)), Some((Tok::Sym("="), _))) = (toks.first(), toks.get(1))
            else {
                return Err(format!("{me}: line {ln} is `name = formula`"));
            };
            let name = name.clone();
            if cfg.inputs.contains_key(&name) {
                return Err(format!("{me}: line {ln} sets {name}, which is an input"));
            }
            if names.contains(&name) {
                return Err(format!("{me}: line {ln} sets {name} a second time"));
            }
            if FUNCS.contains(&name.as_str()) || WORDS.contains(&name.as_str()) {
                return Err(format!("{me}: line {ln}: {name:?} cannot be a letter"));
            }
            let mut p = P {
                toks: toks[2..].to_vec(),
                i: 0,
                ln,
                exprs: &mut exprs,
                inputs: &inputs,
                lines: &names,
                used_inputs: &mut used,
            };
            let root = p.expr().map_err(|e| format!("{me}: {e}"))?;
            if p.i < p.toks.len() {
                return Err(format!(
                    "{me}: {}: unexpected text after the formula",
                    p.at()
                ));
            }
            let out = outputs.iter().position(|o| *o == name);
            let mut dims = Dims {
                exprs: &exprs,
                in_dims: &in_dims,
                line_dims: &line_dims,
                zero: std::mem::take(&mut zero),
            };
            let want = out.map(|o| out_dims[o]);
            let d = dims
                .infer(root, want)
                .map_err(|e| format!("{me}: line {ln}: {e}"))?;
            zero = dims.zero;
            let d = match d {
                Dv::D(d) => d,
                Dv::Zero => want.unwrap_or(Dimension::NONE),
            };
            if let Some(o) = out {
                if d != out_dims[o] {
                    return Err(format!(
                        "{me}: line {ln}: {name} works out as [{d}] but is declared {:?} [{}]",
                        cfg.outputs[&name].unit, out_dims[o]
                    ));
                }
            }
            names.push(name.clone());
            line_dims.push(d);
            lines.push((name, root, out));
        }
        for o in &outputs {
            if !names.contains(o) {
                return Err(format!("{me}: no line sets the output {o}"));
            }
        }
        for (k, n) in inputs.iter().enumerate() {
            if !used.contains(&k) {
                return Err(format!("{me}: the input {n} is not used in the formula"));
            }
        }
        let ops = exprs
            .iter()
            .filter(|e| {
                !matches!(
                    e,
                    Expr::Whole(_) | Expr::Pi | Expr::Input(_) | Expr::Line(_)
                )
            })
            .count();
        if ops > MAX_OPS {
            return Err(format!(
                "{me}: {ops} operations is more than the {MAX_OPS} an Equation holds: split it into a box"
            ));
        }
        let parsed = Parsed {
            cfg,
            inputs,
            in_dims,
            in_scales,
            outputs,
            out_dims,
            out_scales,
            lines,
            line_dims,
            exprs,
            zero_dims: zero,
            ops,
            list,
        };
        parsed.run_examples()?;
        Ok(parsed)
    }

    pub fn me(&self) -> String {
        format!("Equation {:?}", self.cfg.name)
    }

    fn run_examples(&self) -> Result<(), String> {
        for (k, ex) in self.cfg.examples.iter().enumerate() {
            let at = format!("{}: worked example {}", self.me(), k + 1);
            let tol = ex.get("tolerance").and_then(J::as_f64).unwrap_or(1e-6);
            for key in ex.keys() {
                if key != "tolerance" && !self.inputs.contains(key) && !self.outputs.contains(key) {
                    return Err(format!("{at}: {key:?} is not a letter of this Equation"));
                }
            }
            let mut ins = vec![];
            for (i, n) in self.inputs.iter().enumerate() {
                let v = ex.get(n).ok_or_else(|| format!("{at}: gives no {n}"))?;
                let s = self.in_scales[i];
                ins.push(if self.cfg.inputs[n].list {
                    let a = v.as_array().ok_or_else(|| format!("{at}: {n} is a list"))?;
                    Val::List(
                        a.iter()
                            .map(|x| x.as_f64().map(|x| Val::Number(x * s)))
                            .collect::<Option<_>>()
                            .ok_or_else(|| format!("{at}: {n} holds numbers"))?,
                    )
                } else {
                    Val::Number(v.as_f64().ok_or_else(|| format!("{at}: {n} is a number"))? * s)
                });
            }
            let (got, _) = self.run(&ins).map_err(|e| format!("{at}: {e}"))?;
            for (o, n) in self.outputs.iter().enumerate() {
                let Some(want) = ex.get(n) else { continue };
                let s = self.out_scales[o];
                let pairs: Vec<(f64, f64)> = match (&got[o], want) {
                    (Val::Number(g), w) => vec![(*g / s, w.as_f64().unwrap_or(f64::NAN))],
                    (Val::List(g), J::Array(w)) if g.len() == w.len() => g
                        .iter()
                        .zip(w)
                        .map(|(g, w)| {
                            (
                                g.num().unwrap_or(f64::NAN) / s,
                                w.as_f64().unwrap_or(f64::NAN),
                            )
                        })
                        .collect(),
                    _ => return Err(format!("{at}: {n} should be a list as long as the inputs")),
                };
                for (g, w) in pairs {
                    let ok = if w == 0.0 {
                        g.abs() <= tol
                    } else {
                        (g - w).abs() <= tol * w.abs()
                    };
                    if !ok {
                        return Err(format!(
                            "{at}: works out {n} = {g} {}, the example says {w}",
                            self.cfg.outputs[n].unit
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// The ports this Equation's config gives the node.
    pub fn ports(&self) -> (Vec<Port>, Vec<Port>) {
        let mk = |n: &str, p: &PortSpec, d: Dimension, s: f64, list: bool| Port {
            name: n.into(),
            kind: p.kind.clone(),
            dim: d,
            valid: p.valid.map(|(a, b)| (a * s, b * s)),
            shape: list.then(|| Shape::List(Box::new(Shape::Number))),
            any_dim: false,
        };
        let ins = self
            .inputs
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let p = &self.cfg.inputs[n];
                mk(n, p, self.in_dims[i], self.in_scales[i], p.list)
            })
            .collect();
        let outs = self
            .outputs
            .iter()
            .enumerate()
            .map(|(o, n)| {
                mk(
                    n,
                    &self.cfg.outputs[n],
                    self.out_dims[o],
                    self.out_scales[o],
                    self.list,
                )
            })
            .collect();
        (ins, outs)
    }

    /// Run on wire values (SI): the outputs in port order and the inputs the
    /// result depended on.
    pub fn run(&self, v: &[Val]) -> Result<(Vec<Val>, BTreeSet<usize>), String> {
        let mut used = BTreeSet::new();
        if !self.list {
            let x: Vec<f64> = v
                .iter()
                .map(|x| x.num().ok_or("expected a number"))
                .collect::<Result<_, _>>()?;
            let outs = self.run_one(&x, &mut used)?;
            return Ok((outs.into_iter().map(Val::Number).collect(), used));
        }
        let mut n = None;
        for (i, x) in v.iter().enumerate() {
            if let Val::List(l) = x {
                if l.len() > MAX_LIST {
                    return Err(format!(
                        "{} holds {} entries, more than the {} a list takes",
                        self.inputs[i],
                        l.len(),
                        MAX_LIST
                    ));
                }
                match n {
                    Some(k) if k != l.len() => {
                        return Err(format!(
                            "the lists are not one length ({} holds {}, not {k})",
                            self.inputs[i],
                            l.len()
                        ))
                    }
                    _ => n = Some(l.len()),
                }
            }
        }
        let n = n.unwrap_or(0);
        let mut cols: Vec<Vec<Val>> = vec![Vec::with_capacity(n); self.outputs.len()];
        for k in 0..n {
            let x: Vec<f64> = v
                .iter()
                .map(|x| match x {
                    Val::List(l) => l[k].num(),
                    x => x.num(),
                })
                .collect::<Option<_>>()
                .ok_or("expected numbers")?;
            let outs = self
                .run_one(&x, &mut used)
                .map_err(|e| format!("entry {k}: {e}"))?;
            for (c, o) in cols.iter_mut().zip(outs) {
                c.push(Val::Number(o));
            }
        }
        Ok((cols.into_iter().map(Val::List).collect(), used))
    }

    /// Run on single numbers (SI), the outputs in order: the physics atoms'
    /// path, with no record of which inputs were read.
    pub fn run_numbers(&self, x: &[f64]) -> Result<Vec<f64>, String> {
        let mut used = BTreeSet::new();
        self.run_one(x, &mut used)
    }

    fn run_one(&self, x: &[f64], used: &mut BTreeSet<usize>) -> Result<Vec<f64>, String> {
        let mut memo: Vec<Option<f64>> = vec![None; self.lines.len()];
        let mut outs = vec![0.0; self.outputs.len()];
        for (k, (_, _, o)) in self.lines.iter().enumerate() {
            if let Some(o) = o {
                outs[*o] = self.line(k, x, &mut memo, used)?;
            }
        }
        Ok(outs)
    }

    fn line(
        &self,
        k: usize,
        x: &[f64],
        memo: &mut Vec<Option<f64>>,
        used: &mut BTreeSet<usize>,
    ) -> Result<f64, String> {
        if let Some(v) = memo[k] {
            return Ok(v);
        }
        let v = self
            .num(self.lines[k].1, x, memo, used)
            .map_err(|e| format!("{}: {e}", self.lines[k].0))?;
        memo[k] = Some(v);
        Ok(v)
    }

    fn num(
        &self,
        e: usize,
        x: &[f64],
        memo: &mut Vec<Option<f64>>,
        used: &mut BTreeSet<usize>,
    ) -> Result<f64, String> {
        let go = |e: usize, memo: &mut Vec<Option<f64>>, used: &mut BTreeSet<usize>| {
            self.num(e, x, memo, used)
        };
        Ok(match &self.exprs[e] {
            Expr::Whole(n) => f64::from(*n),
            Expr::Pi => sse_core::math::PI,
            Expr::Input(i) => {
                used.insert(*i);
                x[*i]
            }
            Expr::Line(k) => self.line(*k, x, memo, used)?,
            Expr::Neg(a) => -go(*a, memo, used)?,
            Expr::Bin(op, a, b) => {
                let (p, q) = (go(*a, memo, used)?, go(*b, memo, used)?);
                let op = match op {
                    Bin::Add => Func::Add,
                    Bin::Sub => Func::Subtract,
                    Bin::Mul => Func::Multiply,
                    Bin::Div => Func::Divide,
                    Bin::Pow => Func::Power,
                };
                number(op, &[p, q])?
            }
            Expr::Call(op, args) => {
                let mut v = Vec::with_capacity(args.len());
                for a in args {
                    v.push(go(*a, memo, used)?);
                }
                number(*op, &v)?
            }
            Expr::If(c, a, b) => {
                if self.cond(*c, x, memo, used)? {
                    go(*a, memo, used)?
                } else {
                    go(*b, memo, used)?
                }
            }
            _ => return Err("a condition stands where a value should".into()),
        })
    }

    fn cond(
        &self,
        e: usize,
        x: &[f64],
        memo: &mut Vec<Option<f64>>,
        used: &mut BTreeSet<usize>,
    ) -> Result<bool, String> {
        Ok(match &self.exprs[e] {
            Expr::Cmp(op, a, b) => {
                let (p, q) = (self.num(*a, x, memo, used)?, self.num(*b, x, memo, used)?);
                match op {
                    Cmp::Less => p < q,
                    Cmp::AtMost => p <= q,
                    Cmp::Greater => p > q,
                    Cmp::AtLeast => p >= q,
                    Cmp::Equal => p == q,
                }
            }
            // Read in order, stopping when the answer is known, as the
            // and/or blocks do.
            Expr::And(a, b) => self.cond(*a, x, memo, used)? && self.cond(*b, x, memo, used)?,
            Expr::Or(a, b) => self.cond(*a, x, memo, used)? || self.cond(*b, x, memo, used)?,
            Expr::Not(a) => !self.cond(*a, x, memo, used)?,
            _ => return Err("a value stands where a condition should".into()),
        })
    }

    /// The one canonical text of the formula (design 6): every operation
    /// bracketed, one space around operators, lines joined by newlines.
    pub fn canonical(&self) -> String {
        self.lines
            .iter()
            .map(|(n, r, _)| format!("{n} = {}", self.text(*r, true)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn text(&self, e: usize, top: bool) -> String {
        let t = |e: usize| self.text(e, false);
        let wrap = |s: String| if top { s } else { format!("({s})") };
        match &self.exprs[e] {
            Expr::Whole(n) => n.to_string(),
            Expr::Pi => "pi".into(),
            Expr::Input(i) => self.inputs[*i].clone(),
            Expr::Line(k) => self.lines[*k].0.clone(),
            Expr::Neg(a) => wrap(format!("-{}", t(*a))),
            Expr::Bin(op, a, b) => {
                let s = match op {
                    Bin::Add => "+",
                    Bin::Sub => "-",
                    Bin::Mul => "*",
                    Bin::Div => "/",
                    Bin::Pow => "^",
                };
                wrap(format!("{} {s} {}", t(*a), t(*b)))
            }
            Expr::Call(op, args) => format!(
                "{}({})",
                op.id()
                    .trim_start_matches("math.")
                    .trim_start_matches("logic."),
                args.iter()
                    .map(|a| self.text(*a, true))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expr::If(c, a, b) => wrap(format!(
                "if {} then {} else {}",
                self.text(*c, true),
                self.text(*a, true),
                self.text(*b, true)
            )),
            Expr::Cmp(op, a, b) => {
                let s = match op {
                    Cmp::Less => "<",
                    Cmp::AtMost => "<=",
                    Cmp::Greater => ">",
                    Cmp::AtLeast => ">=",
                    Cmp::Equal => "==",
                };
                wrap(format!("{} {s} {}", t(*a), t(*b)))
            }
            Expr::And(a, b) => wrap(format!("{} and {}", t(*a), t(*b))),
            Expr::Or(a, b) => wrap(format!("{} or {}", t(*a), t(*b))),
            Expr::Not(a) => wrap(format!("not {}", t(*a))),
        }
    }

    /// How many choices the formula makes: the editor marks each one.
    pub fn choices(&self) -> usize {
        self.exprs
            .iter()
            .filter(|e| matches!(e, Expr::If(..)))
            .count()
    }
}

fn ident(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_alphabetic() || f == '_')
        && c.all(|x| x.is_alphanumeric() || x == '_')
}

/// Most entries in a list input (the logic blocks' limit).
pub const MAX_LIST: usize = 4096;

/// The pinned operations a formula uses, with the id of the number block
/// that does the same (so a formula expands into blocks with the same bits).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Func {
    Add,
    Subtract,
    Multiply,
    Divide,
    Min,
    Max,
    Clamp,
    Abs,
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

impl Func {
    pub const ALL: [Func; 19] = [
        Func::Add,
        Func::Subtract,
        Func::Multiply,
        Func::Divide,
        Func::Min,
        Func::Max,
        Func::Clamp,
        Func::Abs,
        Func::Power,
        Func::Sqrt,
        Func::Cbrt,
        Func::Exp,
        Func::Expm1,
        Func::Ln,
        Func::Log10,
        Func::Sin,
        Func::Cos,
        Func::Tan,
        Func::Atan2,
    ];
    /// The number block that does the same.
    pub fn id(self) -> &'static str {
        match self {
            Func::Add => "logic.add",
            Func::Subtract => "logic.subtract",
            Func::Multiply => "logic.multiply",
            Func::Divide => "logic.divide",
            Func::Min => "logic.min",
            Func::Max => "logic.max",
            Func::Clamp => "logic.clamp",
            Func::Abs => "logic.abs",
            Func::Power => "math.power",
            Func::Sqrt => "math.sqrt",
            Func::Cbrt => "math.cbrt",
            Func::Exp => "math.exp",
            Func::Expm1 => "math.expm1",
            Func::Ln => "math.ln",
            Func::Log10 => "math.log10",
            Func::Sin => "math.sin",
            Func::Cos => "math.cos",
            Func::Tan => "math.tan",
            Func::Atan2 => "math.atan2",
        }
    }
}

/// The pinned maths of the number blocks, shared by every formula and
/// every number block so both give the same bits and the same refusals. A
/// result that is not a finite number is an error that names the step.
pub fn number(op: Func, x: &[f64]) -> Result<f64, String> {
    use sse_core::math as m;
    use Func::*;
    let r = match op {
        Add => x[0] + x[1],
        Subtract => x[0] - x[1],
        Multiply => x[0] * x[1],
        Divide if x[1] == 0.0 => return Err("division by zero".into()),
        Divide => x[0] / x[1],
        Min => x[0].min(x[1]),
        Max => x[0].max(x[1]),
        Abs => x[0].abs(),
        Clamp if x[1] > x[2] => return Err(format!("lo {} is above hi {}", x[1], x[2])),
        Clamp => x[0].max(x[1]).min(x[2]),
        Power => m::pow(x[0], x[1]),
        Sqrt if x[0] < 0.0 => return Err(format!("the square root of {} (below zero)", x[0])),
        Sqrt => m::sqrt(x[0]),
        Cbrt => m::cbrt(x[0]),
        Exp => m::exp(x[0]),
        Expm1 => m::expm1(x[0]),
        Ln | Log10 if x[0] <= 0.0 => {
            return Err(format!("the logarithm of {} (not above zero)", x[0]))
        }
        Ln => m::ln(x[0]),
        Log10 => m::log10(x[0]),
        Sin => m::sin(x[0]),
        Cos => m::cos(x[0]),
        Tan => m::tan(x[0]),
        Atan2 => m::atan2(x[0], x[1]),
    };
    if !r.is_finite() {
        return Err(format!(
            "{} gives a number too large to hold (from {x:?})",
            op.id()
        ));
    }
    Ok(r)
}
