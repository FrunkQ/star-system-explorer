//! `sse.math.equation`, the Equation block (package ND3, equation block
//! design): one block whose formula is written as text in the pack, and
//! [`expand`], the same formula as maths blocks with the same bits.
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
//! names; `+ - * / ^`, brackets, unary minus; `exp expm1 ln log10 sqrt abs
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

use crate::blocks::{self, Op};
use serde_json::{json, Value as J};
use sse_core::quantity::si_unit;
use sse_core::Dimension;
use sse_flow::flow::{FlowDoc, NodeDoc, WireDoc};
use sse_flow::{Atom, AtomDecl, AtomError, BlockClass, Call, CostClass, Out, Port, Val};
pub use sse_formula::*;
use std::collections::{BTreeMap, BTreeSet};

/// The logic block for a formula's operation.
pub fn op_of(f: Func) -> Op {
    match f {
        Func::Add => Op::Add,
        Func::Subtract => Op::Subtract,
        Func::Multiply => Op::Multiply,
        Func::Divide => Op::Divide,
        Func::Min => Op::Min,
        Func::Max => Op::Max,
        Func::Clamp => Op::Clamp,
        Func::Abs => Op::Abs,
        Func::Power => Op::Power,
        Func::Sqrt => Op::Sqrt,
        Func::Cbrt => Op::Cbrt,
        Func::Exp => Op::Exp,
        Func::Expm1 => Op::Expm1,
        Func::Ln => Op::Ln,
        Func::Log10 => Op::Log10,
        Func::Sin => Op::Sin,
        Func::Cos => Op::Cos,
        Func::Tan => Op::Tan,
        Func::Atan2 => Op::Atan2,
    }
}

// -------------------------------------------------------------- the atom

pub struct Equation {
    decl: AtomDecl,
}

impl Default for Equation {
    fn default() -> Self {
        Self::new()
    }
}

impl Equation {
    pub fn new() -> Self {
        Equation {
            decl: AtomDecl {
                id: ID.into(),
                version: "1.0.0".into(),
                inputs: vec![],
                outputs: vec![],
                params: vec![],
                cost_class: CostClass::Light,
                references: vec![],
                explain: sse_flow::decl::ExplainText {
                    curious: "Works out a relation written as a formula, with every letter on a line from where it comes.".into(),
                    science: "One formula as pack data: +, -, *, /, ^, exp, ln, log10, sqrt, trigonometry and choices, unit-checked on load, run through the pinned maths in the order written. Every symbol is an input wired from a block, a Look up or a Model number; the same formula expands into maths blocks with identical bits.".into(),
                    formula: String::new(),
                },
                class: Some(BlockClass::Engine),
                dims: None,
            },
        }
    }
}

impl Atom for Equation {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(format!(
            "{ID} is evaluated through its typed form"
        )))
    }
    fn ports(&self, config: Option<&J>) -> Option<(Vec<Port>, Vec<Port>)> {
        Parsed::new(config).ok().map(|p| p.ports())
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        Parsed::new(config).map(|_| ())
    }
    fn check_wiring(&self, config: Option<&J>, sources: &[Option<&str>]) -> Result<(), String> {
        let Ok(p) = Parsed::new(config) else {
            return Ok(());
        };
        for (n, s) in p.inputs.iter().zip(sources) {
            if *s == Some(blocks::id_of(Op::Const)) {
                return Err(format!(
                    "{}: {n} comes from a typed number: a constant of nature comes from a Look up block, a number of the model from a Model number block with its source",
                    p.me()
                ));
            }
        }
        Ok(())
    }
    fn eval_typed(&self, v: &[Val], _: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        // Parsed again per call for now; design 8B's step list is cached at
        // pack load once a pack stack holds Equations (ND3b).
        let p = Parsed::new(cx.config).map_err(AtomError)?;
        let (values, used) = p
            .run(v)
            .map_err(|e| AtomError(format!("{}: {e}", p.me())))?;
        let letters: Vec<J> = p
            .inputs
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let spec = &p.cfg.inputs[n];
                json!({
                    "symbol": n,
                    "name": spec.name.clone().unwrap_or_else(|| spec.kind.clone()),
                    "value": match &v[i] { Val::Number(x) => json!(x), Val::List(l) => json!(format!("a list of {}", l.len())), _ => J::Null },
                    "unit": si_unit(p.in_dims[i]),
                })
            })
            .collect();
        Ok(Out {
            values,
            used: Some(used.into_iter().collect()),
            note: Some(json!({
                "block": ID,
                "name": p.cfg.name,
                "formula": p.cfg.formula,
                "canonical": p.canonical(),
                "letters": letters,
                "choices": p.choices(),
                "source": p.cfg.source.as_ref().map(|s| json!({"text": s.text, "doi": s.doi, "url": s.url, "where": s.at})),
                "logic": p.cfg.logic,
            })),
            ..Default::default()
        })
    }
}

// ------------------------------------------------------------- expansion

struct Ex<'a> {
    p: &'a Parsed,
    id: &'a str,
    ins: &'a [String],
    nodes: Vec<NodeDoc>,
    wires: Vec<WireDoc>,
    lines: Vec<Option<String>>,
}

fn node(id: String, atom: &str, config: Option<J>) -> NodeDoc {
    NodeDoc {
        id,
        atom: Some(atom.into()),
        solve: None,
        params: BTreeMap::new(),
        config,
        r#box: None,
    }
}

impl Ex<'_> {
    fn add(&mut self, atom: &str, config: Option<J>, ins: &[(&str, String)], out: &str) -> String {
        let id = format!("{}__{}", self.id, self.nodes.len());
        for (port, from) in ins {
            self.wires.push(WireDoc {
                from: from.clone(),
                to: format!("{id}.{port}"),
            });
        }
        self.nodes.push(node(id.clone(), atom, config));
        format!("{id}.{out}")
    }
    fn val(&mut self, e: usize) -> String {
        let p = self.p;
        match &p.exprs[e] {
            Expr::Whole(n) => {
                let mut c = json!({ "value": n });
                if let Some(d) = p.zero_dims.get(&e).filter(|d| **d != Dimension::NONE) {
                    c["unit"] = json!(si_unit(*d));
                }
                self.add(blocks::id_of(Op::Const), Some(c), &[], "value")
            }
            Expr::Pi => self.add(
                blocks::id_of(Op::Const),
                Some(json!({ "value": sse_core::math::PI })),
                &[],
                "value",
            ),
            Expr::Input(i) => self.ins[*i].clone(),
            Expr::Line(k) => self.line(*k),
            Expr::Neg(a) => {
                // -x is x times -1: exact negation, the sign of a zero too.
                let x = self.val(*a);
                let m1 = self.add(
                    blocks::id_of(Op::Const),
                    Some(json!({ "value": -1 })),
                    &[],
                    "value",
                );
                self.add(
                    blocks::id_of(Op::Multiply),
                    None,
                    &[("a", x), ("b", m1)],
                    "value",
                )
            }
            Expr::Bin(Bin::Pow, a, b) => {
                if let Some((n, d)) = ratio(&p.exprs, *b) {
                    let x = self.val(*a);
                    let text = if d == 1 {
                        n.to_string()
                    } else {
                        format!("{n}/{d}")
                    };
                    return self.add(
                        blocks::id_of(Op::Power),
                        Some(json!({ "exponent": text })),
                        &[("x", x)],
                        "value",
                    );
                }
                let (x, n) = (self.val(*a), self.val(*b));
                self.add(
                    blocks::id_of(Op::Power),
                    None,
                    &[("x", x), ("n", n)],
                    "value",
                )
            }
            Expr::Bin(op, a, b) => {
                let (x, y) = (self.val(*a), self.val(*b));
                let op = match op {
                    Bin::Add => Op::Add,
                    Bin::Sub => Op::Subtract,
                    Bin::Mul => Op::Multiply,
                    _ => Op::Divide,
                };
                self.add(blocks::id_of(op), None, &[("a", x), ("b", y)], "value")
            }
            Expr::Call(f, args) => {
                let op = &op_of(*f);
                let v: Vec<String> = args.iter().map(|a| self.val(*a)).collect();
                let names: &[&str] = match op {
                    Op::Min | Op::Max => &["a", "b"],
                    Op::Clamp => &["x", "lo", "hi"],
                    Op::Atan2 => &["y", "x"],
                    _ => &["x"],
                };
                let ins: Vec<(&str, String)> = names.iter().copied().zip(v).collect();
                let out = if *op == Op::Atan2 { "angle" } else { "value" };
                self.add(blocks::id_of(*op), None, &ins, out)
            }
            Expr::If(c, a, b) => {
                let c = self.cond(*c);
                let (x, y) = (self.val(*a), self.val(*b));
                self.add(
                    blocks::id_of(Op::Select),
                    None,
                    &[("if", c), ("then", x), ("else", y)],
                    "value",
                )
            }
            _ => unreachable!("checked on load"),
        }
    }
    fn cond(&mut self, e: usize) -> String {
        match &self.p.exprs[e] {
            Expr::Cmp(op, a, b) => {
                let (x, y) = (self.val(*a), self.val(*b));
                let op = match op {
                    Cmp::Less => Op::Less,
                    Cmp::AtMost => Op::AtMost,
                    Cmp::Greater => Op::Greater,
                    Cmp::AtLeast => Op::AtLeast,
                    Cmp::Equal => Op::Equal,
                };
                self.add(blocks::id_of(op), None, &[("a", x), ("b", y)], "holds")
            }
            Expr::And(a, b) | Expr::Or(a, b) => {
                let op = if matches!(self.p.exprs[e], Expr::And(..)) {
                    Op::And
                } else {
                    Op::Or
                };
                let (x, y) = (self.cond(*a), self.cond(*b));
                self.add(blocks::id_of(op), None, &[("a", x), ("b", y)], "holds")
            }
            Expr::Not(a) => {
                let x = self.cond(*a);
                self.add(blocks::id_of(Op::Not), None, &[("a", x)], "holds")
            }
            _ => unreachable!("checked on load"),
        }
    }
    fn line(&mut self, k: usize) -> String {
        if let Some(r) = &self.lines[k] {
            return r.clone();
        }
        let r = self.val(self.p.lines[k].1);
        self.lines[k] = Some(r.clone());
        r
    }
}

/// The same flow with every Equation replaced by maths blocks, wired as the
/// formula is written (design 6). An Equation over lists is expanded per
/// entry: give it single numbers to expand.
pub fn expand(doc: &FlowDoc) -> Result<FlowDoc, String> {
    let mut out = doc.clone();
    out.nodes.clear();
    out.wires.clear();
    // node.port of an Equation output -> the ref that now carries it.
    let mut moved: BTreeMap<String, String> = BTreeMap::new();
    let wired = |to: &str| {
        doc.wires
            .iter()
            .find(|w| w.to == to)
            .map(|w| w.from.clone())
    };
    for n in &doc.nodes {
        if n.atom.as_deref() != Some(ID) {
            out.nodes.push(n.clone());
            continue;
        }
        let p = Parsed::new(n.config.as_ref())?;
        if p.list {
            return Err(format!(
                "{}: expand an Equation over lists one entry at a time",
                p.me()
            ));
        }
        let ins: Vec<String> = p
            .inputs
            .iter()
            .map(|i| {
                wired(&format!("{}.{i}", n.id)).ok_or_else(|| format!("{}.{i} is not wired", n.id))
            })
            .collect::<Result<_, _>>()?;
        let mut ex = Ex {
            p: &p,
            id: &n.id,
            ins: &ins,
            nodes: vec![],
            wires: vec![],
            lines: vec![None; p.lines.len()],
        };
        for (k, (name, _, o)) in p.lines.iter().enumerate() {
            if o.is_some() {
                let r = ex.line(k);
                moved.insert(format!("{}.{name}", n.id), r);
            }
        }
        out.nodes.extend(ex.nodes);
        out.wires.extend(ex.wires);
    }
    // An expansion's inputs may themselves be moved outputs.
    let follow = |r: &str| {
        let mut r = r.to_string();
        while let Some(x) = moved.get(&r) {
            r = x.clone();
        }
        r
    };
    for w in out.wires.iter_mut() {
        w.from = follow(&w.from);
    }
    let eq_inputs: BTreeSet<&str> = doc
        .nodes
        .iter()
        .filter(|n| n.atom.as_deref() == Some(ID))
        .map(|n| n.id.as_str())
        .collect();
    for w in &doc.wires {
        let into_eq =
            w.to.split_once('.')
                .is_some_and(|(n, _)| eq_inputs.contains(n));
        if !into_eq {
            out.wires.push(WireDoc {
                from: follow(&w.from),
                to: w.to.clone(),
            });
        }
    }
    for p in out.publishes.iter_mut() {
        p.from = follow(&p.from);
    }
    Ok(out)
}

/// A tidy text of a parsed formula for the face of the block.
pub fn canonical(config: &J) -> Result<String, String> {
    Parsed::new(Some(config)).map(|p| p.canonical())
}
