//! Flow documents, validation, deterministic scheduling, evaluation with solve
//! nodes, and explanation bundles (H6.2, H6.5).

use crate::decl::{AtomDecl, AtomRegistry, Call, DimRule, Env, Port};
use crate::value::{Shape, Val};
use serde::{Deserialize, Serialize};
use sse_core::{ContentHash, Dimension, Quantity, QuantityKind, Units};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const FLOW_FORMAT: &str = "sse.flow/1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Si(f64),
    InUnit { value: f64, unit: String },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SolveSpec {
    /// Residual atom: one output named `residual`; solved for `residual = 0`.
    pub atom: String,
    /// The residual atom input that is the unknown.
    pub unknown: String,
    /// Bracket in SI; the residual must change sign across it.
    pub bracket: (f64, f64),
    /// Fixed iteration count of bisection (the deterministic stopping rule).
    #[serde(default = "default_iter")]
    pub iterations: u32,
}
fn default_iter() -> u32 {
    80
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NodeDoc {
    pub id: String,
    #[serde(default)]
    pub atom: Option<String>,
    #[serde(default)]
    pub solve: Option<SolveSpec>,
    #[serde(default)]
    pub params: BTreeMap<String, ParamValue>,
    /// A logic or engine block's settings (a field to read, a scheme, a
    /// table). Left out when absent, so existing flow hashes are unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// A box (J3): `id@range`, a flow of blocks used as one block. Inlined
    /// before compiling ([`Flow::compile_with`]). Left out when absent.
    #[serde(rename = "box", default, skip_serializing_if = "Option::is_none")]
    pub r#box: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FlowInputDoc {
    pub name: String,
    /// The quantity kind (numbers); for other shapes, a free label.
    pub kind: String,
    /// What the input carries (J2); absent means a number of `kind`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<Shape>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WireDoc {
    pub from: String,
    pub to: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PublishDoc {
    pub kind: String,
    pub from: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Expect {
    pub value: f64,
    #[serde(default)]
    pub unit: Option<String>,
    /// Absolute tolerance in the same unit.
    pub tol: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GoldenCase {
    pub name: String,
    pub inputs: BTreeMap<String, ParamValue>,
    pub expect: BTreeMap<String, Expect>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FlowDoc {
    pub format: String,
    pub id: String,
    pub version: String,
    pub hook: String,
    #[serde(default)]
    pub inputs: Vec<FlowInputDoc>,
    pub nodes: Vec<NodeDoc>,
    #[serde(default)]
    pub wires: Vec<WireDoc>,
    #[serde(default)]
    pub publishes: Vec<PublishDoc>,
    #[serde(default)]
    pub tests: Vec<GoldenCase>,
}

impl FlowDoc {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("flow document: {e}"))
    }
    /// Content hash: SHA-256 of the canonical CBOR of the parsed document.
    pub fn content_hash(&self) -> ContentHash {
        ContentHash::of(self).expect("flow docs hold no NaN")
    }
}

/// One validation problem, readable on its own.
#[derive(Clone, PartialEq, Debug)]
pub struct FlowError {
    pub at: String,
    pub message: String,
}
impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.at, self.message)
    }
}

/// What validation and evaluation need to know.
pub struct Context<'a> {
    pub atoms: &'a AtomRegistry,
    pub kinds: &'a BTreeMap<String, QuantityKind>,
    pub units: &'a Units,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Source {
    Input(usize),
    Node(usize, usize),
}

#[derive(Clone, Debug)]
struct Compiled {
    atom: String,
    decl: AtomDecl,
    /// For solve nodes: index of the unknown among the residual atom's inputs.
    solve: Option<(usize, SolveSpec)>,
    /// Node-level input ports (for solve: residual inputs minus the unknown).
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    params: Vec<f64>,
    sources: Vec<Source>,
    config: Option<serde_json::Value>,
    /// A logic or engine block: evaluated through `eval_typed`.
    typed: bool,
}

/// A validated flow, ready to evaluate.
#[derive(Clone, Debug)]
pub struct Flow {
    pub doc: FlowDoc,
    pub hash: ContentHash,
    nodes: Vec<Compiled>,
    input_dims: Vec<Dimension>,
    input_shapes: Vec<Shape>,
    order: Vec<usize>,
    published: BTreeMap<String, (usize, usize)>,
    /// Each node output's dimension (any-dimension logic outputs resolved).
    pub(crate) out_dims: Vec<Vec<Dimension>>,
    /// The boxes this flow was inlined from (J3), outermost first.
    pub boxes: Vec<crate::boxes::BoxUse>,
}

pub(crate) fn split_ref(r: &str) -> Option<(&str, &str)> {
    r.split_once('.')
}

pub(crate) fn param_si(units: &Units, v: &ParamValue, dim: Dimension) -> Result<f64, String> {
    match v {
        ParamValue::Si(x) => Ok(*x),
        ParamValue::InUnit { value, unit } => {
            let q = units.q(*value, unit).map_err(|e| e.to_string())?;
            q.expect(dim, "value").map_err(|e| e.to_string())
        }
    }
}

impl Flow {
    /// Validate a document. Returns every problem found, not just the first.
    pub fn compile(doc: FlowDoc, cx: &Context) -> Result<Flow, Vec<FlowError>> {
        let mut errs = Vec::new();
        let mut err = |at: &str, m: String| {
            errs.push(FlowError {
                at: at.into(),
                message: m,
            })
        };
        if doc.format != FLOW_FORMAT {
            err(
                "format",
                format!("expected {FLOW_FORMAT:?}, found {:?}", doc.format),
            );
        }
        // flow inputs
        let mut input_dims = Vec::new();
        let mut input_shapes = Vec::new();
        let mut input_ix = BTreeMap::new();
        for (i, fi) in doc.inputs.iter().enumerate() {
            let shape = fi.shape.clone().unwrap_or(Shape::Number);
            input_shapes.push(shape.clone());
            match cx.kinds.get(&fi.kind) {
                _ if shape != Shape::Number => input_dims.push(Dimension::NONE),
                Some(k) => input_dims.push(k.dim),
                None => {
                    err(
                        &format!("in.{}", fi.name),
                        format!("unknown quantity kind {:?}", fi.kind),
                    );
                    input_dims.push(Dimension::NONE);
                }
            }
            if input_ix.insert(fi.name.clone(), i).is_some() {
                err(&format!("in.{}", fi.name), "duplicate flow input".into());
            }
        }
        // nodes
        let mut node_ix: BTreeMap<String, usize> = BTreeMap::new();
        let mut nodes: Vec<Option<Compiled>> = Vec::new();
        for n in &doc.nodes {
            if n.id == "in" || n.id.is_empty() || n.id.contains('.') {
                err(
                    &n.id,
                    "node id must be non-empty, contain no '.', and not be \"in\"".into(),
                );
            }
            if node_ix.insert(n.id.clone(), nodes.len()).is_some() {
                err(&n.id, "duplicate node id".into());
            }
            if let Some(b) = &n.r#box {
                err(
                    &n.id,
                    format!(
                        "uses box {b}; boxes are inlined first (compile with the stack's boxes)"
                    ),
                );
                nodes.push(None);
                continue;
            }
            let (atom_id, solve) = match (&n.atom, &n.solve) {
                (Some(a), None) => (a.clone(), None),
                (None, Some(s)) => (s.atom.clone(), Some(s.clone())),
                _ => {
                    err(&n.id, "a node has exactly one of `atom` or `solve`".into());
                    nodes.push(None);
                    continue;
                }
            };
            let Some(atom) = cx.atoms.get(&atom_id) else {
                err(&n.id, format!("unknown atom {atom_id:?}"));
                nodes.push(None);
                continue;
            };
            let decl = atom.decl().clone();
            let typed = decl.class.is_some();
            if let Err(e) = atom.check_config(n.config.as_ref()) {
                err(&n.id, e);
            }
            if typed && solve.is_some() {
                err(&n.id, "logic and engine blocks cannot be solved for".into());
            }
            let (inputs, outputs, solve) = if let Some(s) = solve {
                let Some((ui, up)) = decl.input(&s.unknown) else {
                    err(
                        &n.id,
                        format!("solve unknown {:?} is not an input of {atom_id}", s.unknown),
                    );
                    nodes.push(None);
                    continue;
                };
                if decl.outputs.len() != 1 || decl.outputs[0].name != "residual" {
                    err(
                        &n.id,
                        format!("{atom_id} is not a residual atom (one output named `residual`)"),
                    );
                }
                if s.bracket.0.partial_cmp(&s.bracket.1) != Some(std::cmp::Ordering::Less)
                    || s.iterations == 0
                    || s.iterations > 2000
                {
                    err(
                        &n.id,
                        "solve needs bracket lo < hi and 1..=2000 iterations".into(),
                    );
                }
                let ins: Vec<Port> = decl
                    .inputs
                    .iter()
                    .filter(|p| p.name != s.unknown)
                    .cloned()
                    .collect();
                (ins, vec![up.clone()], Some((ui, s)))
            } else {
                let (i, o) = atom
                    .ports(n.config.as_ref())
                    .unwrap_or_else(|| (decl.inputs.clone(), decl.outputs.clone()));
                (i, o, None)
            };
            let mut params: Vec<f64> = decl.params.iter().map(|p| p.default).collect();
            for (k, v) in &n.params {
                match decl.param(k) {
                    None => err(
                        &format!("{}.{k}", n.id),
                        format!("{atom_id} has no parameter {k:?}"),
                    ),
                    Some((i, p)) => match param_si(cx.units, v, p.dim) {
                        Err(e) => err(&format!("{}.{k}", n.id), e),
                        Ok(x) => {
                            if !x.is_finite() {
                                err(
                                    &format!("{}.{k}", n.id),
                                    format!("value {x} is not a finite number"),
                                );
                            } else if let Some((lo, hi)) =
                                p.range.filter(|(lo, hi)| x < *lo || x > *hi)
                            {
                                err(
                                    &format!("{}.{k}", n.id),
                                    format!("value {x} is outside the allowed range {lo} to {hi}"),
                                );
                            }
                            params[i] = x;
                        }
                    },
                }
            }
            let n_in = inputs.len();
            nodes.push(Some(Compiled {
                atom: atom_id,
                decl,
                solve,
                inputs,
                outputs,
                params,
                sources: vec![Source::Input(usize::MAX); n_in],
                config: n.config.clone(),
                typed,
            }));
        }
        // wires
        let mut wired: BTreeSet<(usize, usize)> = BTreeSet::new();
        // (source, its dimension if known before scheduling, its shape)
        let resolve_out = |r: &str,
                           nodes: &Vec<Option<Compiled>>|
         -> Result<(Source, Option<Dimension>, Shape), String> {
            let (a, b) = split_ref(r)
                .ok_or_else(|| format!("bad reference {r:?} (want node.port or in.name)"))?;
            if a == "in" {
                let i = *input_ix
                    .get(b)
                    .ok_or_else(|| format!("no flow input {b:?}"))?;
                return Ok((
                    Source::Input(i),
                    Some(input_dims[i]),
                    input_shapes[i].clone(),
                ));
            }
            let ni = *node_ix.get(a).ok_or_else(|| format!("no node {a:?}"))?;
            let c = nodes[ni]
                .as_ref()
                .ok_or_else(|| format!("node {a:?} is invalid"))?;
            let (oi, p) = c
                .outputs
                .iter()
                .enumerate()
                .find(|(_, p)| p.name == b)
                .ok_or_else(|| format!("node {a:?} has no output {b:?}"))?;
            Ok((
                Source::Node(ni, oi),
                (!p.any_dim).then_some(p.dim),
                p.shape(),
            ))
        };
        // Dimension checks that wait for scheduling: (node, input, wire label).
        let mut deferred: Vec<(usize, usize, String)> = Vec::new();
        for w in &doc.wires {
            let at = format!("wire {} -> {}", w.from, w.to);
            let from = match resolve_out(&w.from, &nodes) {
                Ok(f) => f,
                Err(e) => {
                    err(&at, e);
                    continue;
                }
            };
            let Some((tn, tp)) = split_ref(&w.to) else {
                err(&at, format!("bad target {:?}", w.to));
                continue;
            };
            let Some(&ni) = node_ix.get(tn) else {
                err(&at, format!("no node {tn:?}"));
                continue;
            };
            let Some(c) = nodes[ni].as_mut() else {
                continue;
            };
            let Some((pi, port)) = c.inputs.iter().enumerate().find(|(_, p)| p.name == tp) else {
                err(&at, format!("node {tn:?} has no input {tp:?}"));
                continue;
            };
            if port.shape() != from.2 {
                err(
                    &at,
                    format!(
                        "shape mismatch: {} into {} ({}.{})",
                        from.2,
                        port.shape(),
                        tn,
                        port.name
                    ),
                );
            } else if from.2 == Shape::Number {
                match from.1 {
                    Some(d) if !port.any_dim => {
                        if port.dim != d {
                            err(
                                &at,
                                format!(
                                    "dimension mismatch: [{}] into [{}] ({}.{})",
                                    d, port.dim, tn, port.name
                                ),
                            );
                        }
                    }
                    _ => deferred.push((ni, pi, at.clone())),
                }
            }
            if !wired.insert((ni, pi)) {
                err(&at, format!("input {tn}.{tp} is wired twice"));
            }
            c.sources[pi] = from.0;
        }
        for (ni, c) in nodes.iter().enumerate() {
            if let Some(c) = c {
                for (pi, p) in c.inputs.iter().enumerate() {
                    if !wired.contains(&(ni, pi)) {
                        err(
                            &format!("{}.{}", doc.nodes[ni].id, p.name),
                            "input is not wired".into(),
                        );
                    }
                }
            }
        }
        // publishes: each kind once, dims agree with the kind
        let mut published = BTreeMap::new();
        let mut deferred_pub: Vec<(usize, usize, String, Dimension)> = Vec::new();
        for p in &doc.publishes {
            let at = format!("publish {}", p.kind);
            match (resolve_out(&p.from, &nodes), cx.kinds.get(&p.kind)) {
                (Err(e), _) => err(&at, e),
                (_, None) => err(&at, format!("unknown quantity kind {:?}", p.kind)),
                (Ok((Source::Input(_), _, _)), _) => err(
                    &at,
                    "a flow input cannot be published; publish a node output".into(),
                ),
                (Ok((Source::Node(_, _), _, sh)), _) if sh != Shape::Number => err(
                    &at,
                    format!("only numbers are published as quantities; this output is a {sh}"),
                ),
                (Ok((Source::Node(n, o), d, _)), Some(k)) => {
                    match d {
                        Some(d) if k.dim != d => err(
                            &at,
                            format!("dimension mismatch: output [{d}] published as [{}]", k.dim),
                        ),
                        Some(_) => {}
                        None => deferred_pub.push((n, o, at.clone(), k.dim)),
                    }
                    if published.insert(p.kind.clone(), (n, o)).is_some() {
                        err(
                            &at,
                            "published twice in this flow (one publisher per quantity)".into(),
                        );
                    }
                }
            }
        }
        // schedule: Kahn, ready set ordered by node id => deterministic
        let ids: Vec<&str> = doc.nodes.iter().map(|n| n.id.as_str()).collect();
        let mut indeg = vec![0usize; nodes.len()];
        let mut users: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); nodes.len()];
        for (ni, c) in nodes.iter().enumerate() {
            if let Some(c) = c {
                let deps: BTreeSet<usize> = c
                    .sources
                    .iter()
                    .filter_map(|s| {
                        if let Source::Node(d, _) = s {
                            Some(*d)
                        } else {
                            None
                        }
                    })
                    .collect();
                indeg[ni] = deps.len();
                for d in deps {
                    users[d].insert(ni);
                }
            }
        }
        let mut ready: BTreeSet<(&str, usize)> = (0..nodes.len())
            .filter(|i| indeg[*i] == 0)
            .map(|i| (ids[i], i))
            .collect();
        let mut order = Vec::new();
        while let Some(first) = ready.pop_first() {
            order.push(first.1);
            for &u in &users[first.1] {
                indeg[u] -= 1;
                if indeg[u] == 0 {
                    ready.insert((ids[u], u));
                }
            }
        }
        if order.len() < nodes.len() {
            let stuck: Vec<&str> = (0..nodes.len())
                .filter(|i| indeg[*i] > 0)
                .map(|i| ids[i])
                .collect();
            err("graph", format!("cycle through nodes {}", stuck.join(", ")));
        }
        // Dimensions through any-dimension logic blocks, in schedule order.
        let mut out_dims: Vec<Vec<Dimension>> = nodes
            .iter()
            .map(|c| {
                c.as_ref()
                    .map_or(vec![], |c| c.outputs.iter().map(|p| p.dim).collect())
            })
            .collect();
        if order.len() == nodes.len() {
            for &ni in &order {
                let Some(c) = nodes[ni].as_ref() else {
                    continue;
                };
                let src_dim = |s: &Source, od: &Vec<Vec<Dimension>>| match *s {
                    Source::Input(i) if i < input_dims.len() => input_dims[i],
                    Source::Node(d, o) => od[d].get(o).copied().unwrap_or(Dimension::NONE),
                    _ => Dimension::NONE,
                };
                for (n2, pi, at) in deferred.iter().filter(|(n2, _, _)| *n2 == ni) {
                    let p = &c.inputs[*pi];
                    let d = src_dim(&c.sources[*pi], &out_dims);
                    if !p.any_dim && p.dim != d {
                        err(
                            at,
                            format!(
                                "dimension mismatch: [{}] into [{}] ({}.{})",
                                d, p.dim, doc.nodes[*n2].id, p.name
                            ),
                        );
                    }
                }
                let any: Vec<Dimension> = c
                    .inputs
                    .iter()
                    .zip(&c.sources)
                    .filter(|(p, _)| p.any_dim && p.shape() == Shape::Number)
                    .map(|(_, s)| src_dim(s, &out_dims))
                    .collect();
                let rule = c.decl.dims.unwrap_or(DimRule::Same);
                let od = match rule {
                    DimRule::Same => {
                        if let Some(d) = any.iter().find(|d| **d != any[0]) {
                            err(
                                &doc.nodes[ni].id,
                                format!("dimension mismatch: inputs carry [{}] and [{d}]", any[0]),
                            );
                        }
                        any.first().copied().unwrap_or(Dimension::NONE)
                    }
                    DimRule::Product | DimRule::Quotient if any.len() == 2 => {
                        if rule == DimRule::Product {
                            any[0].mul(any[1])
                        } else {
                            any[0].div(any[1])
                        }
                    }
                    _ => Dimension::NONE,
                };
                for (o, p) in c.outputs.iter().enumerate() {
                    if p.any_dim {
                        out_dims[ni][o] = od;
                    }
                }
            }
            for (n, o, at, want) in &deferred_pub {
                let d = out_dims[*n][*o];
                if d != *want {
                    err(
                        at,
                        format!("dimension mismatch: output [{d}] published as [{want}]"),
                    );
                }
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        let hash = doc.content_hash();
        Ok(Flow {
            doc,
            hash,
            nodes: nodes.into_iter().map(Option::unwrap).collect(),
            input_dims,
            input_shapes,
            order,
            published,
            out_dims,
            boxes: vec![],
        })
    }

    /// The dimension and shape of a node output (`node.port`) or flow input.
    pub fn port_of(&self, r: &str) -> Option<(Dimension, Shape)> {
        let (a, b) = split_ref(r)?;
        if a == "in" {
            let i = self.doc.inputs.iter().position(|x| x.name == b)?;
            return Some((self.input_dims[i], self.input_shapes[i].clone()));
        }
        let n = self.doc.nodes.iter().position(|x| x.id == a)?;
        let o = self.nodes[n].outputs.iter().position(|p| p.name == b)?;
        Some((self.out_dims[n][o], self.nodes[n].outputs[o].shape()))
    }
    /// Where the physics of everything a flow input feeds holds: the
    /// intersection of the validity ranges of the ports it is wired to.
    /// `None` when unbounded; an empty intersection is `(lo, hi)` with lo > hi.
    pub fn input_validity(&self, name: &str) -> Option<(f64, f64)> {
        let i = self.doc.inputs.iter().position(|x| x.name == name)?;
        let mut out: Option<(f64, f64)> = None;
        for c in &self.nodes {
            for (p, s) in c.inputs.iter().zip(&c.sources) {
                if *s == Source::Input(i) {
                    if let Some((lo, hi)) = p.valid {
                        out = Some(match out {
                            None => (lo, hi),
                            Some((a, b)) => (a.max(lo), b.min(hi)),
                        });
                    }
                }
            }
        }
        out
    }
    /// The declarations of the atoms in this flow, in node order.
    pub fn decls(&self) -> impl Iterator<Item = &AtomDecl> {
        self.nodes.iter().map(|c| &c.decl)
    }
    pub fn node_ids(&self) -> impl Iterator<Item = &str> {
        self.doc.nodes.iter().map(|n| n.id.as_str())
    }
    pub fn published_kinds(&self) -> impl Iterator<Item = &str> {
        self.published.keys().map(String::as_str)
    }
}

/// Where an input value came from (H6.5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceLabel {
    Catalogued,
    Resolved,
    Derived,
    Authored,
    AuthoredImpossible,
}

/// Validity of a value, ordered from best to worst.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Validity {
    InRange,
    Extrapolated,
    ImpossibleInput,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct InputValue {
    pub q: Quantity,
    pub source: SourceLabel,
}

#[derive(Clone, PartialEq, Debug)]
struct NodeOut {
    values: Vec<Val>,
    validity: Validity,
    /// Inputs that fell outside the atom's validity range.
    out_of_range: Vec<String>,
    /// Typed blocks: inputs the result depended on (None: all), facts read,
    /// an engine block's explanation.
    used: Option<Vec<usize>>,
    reads: Vec<(String, sse_core::cbor::Value)>,
    note: Option<serde_json::Value>,
}

/// Result of evaluating a flow.
#[derive(Clone, Debug)]
pub struct Evaluation<'f> {
    flow: &'f Flow,
    inputs: Vec<InputValue>,
    in_vals: Vec<Val>,
    outs: Vec<NodeOut>,
    /// Deterministic work done: one unit per atom call (solve: per iteration).
    pub work_units: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvalError {
    pub node: String,
    pub message: String,
}
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.node, self.message)
    }
}

fn check_finite(vals: &[f64], node: &str, what: &str) -> Result<(), EvalError> {
    if let Some(i) = vals.iter().position(|v| !v.is_finite()) {
        return Err(EvalError {
            node: node.into(),
            message: format!(
                "{what} {i} is not finite (NaN or infinite values are errors, never values)"
            ),
        });
    }
    Ok(())
}

impl Flow {
    /// Evaluate with inputs by flow-input name.
    pub fn eval(
        &self,
        cx: &Context,
        inputs: &BTreeMap<String, InputValue>,
    ) -> Result<Evaluation<'_>, EvalError> {
        self.eval_with(cx, inputs, &BTreeMap::new(), &Env::empty())
    }

    /// Evaluate with numeric inputs, typed inputs (flags, choices, refs,
    /// text, lists, records) and an environment for logic blocks.
    pub fn eval_with(
        &self,
        cx: &Context,
        inputs: &BTreeMap<String, InputValue>,
        typed_inputs: &BTreeMap<String, Val>,
        env: &Env,
    ) -> Result<Evaluation<'_>, EvalError> {
        let mut ins = Vec::new();
        let mut in_vals = Vec::new();
        for (i, fi) in self.doc.inputs.iter().enumerate() {
            let shape = &self.input_shapes[i];
            if *shape != Shape::Number {
                let v = typed_inputs.get(&fi.name).ok_or_else(|| EvalError {
                    node: format!("in.{}", fi.name),
                    message: "missing input".into(),
                })?;
                if !v.fits(shape) {
                    return Err(EvalError {
                        node: format!("in.{}", fi.name),
                        message: format!("expected a {shape} value"),
                    });
                }
                ins.push(InputValue {
                    q: Quantity::si(0.0, Dimension::NONE),
                    source: SourceLabel::Authored,
                });
                in_vals.push(v.clone());
                continue;
            }
            let v = match (inputs.get(&fi.name), typed_inputs.get(&fi.name)) {
                (Some(v), _) => v.clone(),
                (None, Some(Val::Number(x))) => InputValue {
                    q: Quantity::si(*x, self.input_dims[i]),
                    source: SourceLabel::Authored,
                },
                _ => {
                    return Err(EvalError {
                        node: format!("in.{}", fi.name),
                        message: "missing input".into(),
                    })
                }
            };
            v.q.expect(self.input_dims[i], &fi.name)
                .map_err(|e| EvalError {
                    node: format!("in.{}", fi.name),
                    message: e.to_string(),
                })?;
            check_finite(&[v.q.value], &format!("in.{}", fi.name), "input")?;
            in_vals.push(Val::Number(v.q.value));
            ins.push(v);
        }
        let mut outs: Vec<Option<NodeOut>> = vec![None; self.nodes.len()];
        let mut work = 0u64;
        for &ni in &self.order {
            let c = &self.nodes[ni];
            let id = &self.doc.nodes[ni].id;
            let atom = cx.atoms.get(&c.atom).expect("validated");
            let mut vals: Vec<Val> = Vec::with_capacity(c.inputs.len());
            let mut validity = Validity::InRange;
            for s in &c.sources {
                match *s {
                    Source::Input(i) => {
                        vals.push(in_vals[i].clone());
                        if ins[i].source == SourceLabel::AuthoredImpossible {
                            validity = Validity::ImpossibleInput;
                        }
                    }
                    Source::Node(d, o) => {
                        let up = outs[d].as_ref().expect("scheduled before use");
                        vals.push(up.values[o].clone());
                        validity = validity.max(up.validity);
                    }
                }
            }
            let mut out_of_range = Vec::new();
            for (p, v) in c.inputs.iter().zip(&vals) {
                if let Val::Number(v) = v {
                    if p.valid.is_some_and(|(lo, hi)| *v < lo || *v > hi) {
                        out_of_range.push(p.name.clone());
                    }
                }
            }
            let mut extra = crate::decl::Out::default();
            let values: Vec<Val> = if c.typed {
                work += 1;
                let mut call = Call {
                    env,
                    config: c.config.as_ref(),
                    flow_id: &self.doc.id,
                    node_id: id,
                };
                let o = atom
                    .eval_typed(&vals, &c.params, &mut call)
                    .map_err(|e| EvalError {
                        node: id.clone(),
                        message: e.0,
                    })?;
                if o.values.len() != c.outputs.len() {
                    return Err(EvalError {
                        node: id.clone(),
                        message: "atom returned the wrong number of outputs".into(),
                    });
                }
                for (p, v) in c.outputs.iter().zip(&o.values) {
                    if !v.fits(&p.shape()) {
                        return Err(EvalError {
                            node: id.clone(),
                            message: format!(
                                "output {} is not a {} (NaN or infinite values are errors, never values)",
                                p.name,
                                p.shape()
                            ),
                        });
                    }
                }
                let values = o.values.clone();
                extra = o;
                values
            } else {
                let xs: Vec<f64> = vals
                    .iter()
                    .map(|v| v.num().expect("validated: physics atoms take numbers"))
                    .collect();
                let values = match &c.solve {
                    None => {
                        work += 1;
                        atom.eval(&xs, &c.params).map_err(|e| EvalError {
                            node: id.clone(),
                            message: e.0,
                        })?
                    }
                    Some((ui, s)) => {
                        let f = |x: f64| -> Result<f64, EvalError> {
                            let mut full = xs.clone();
                            full.insert(*ui, x);
                            let r = atom.eval(&full, &c.params).map_err(|e| EvalError {
                                node: id.clone(),
                                message: e.0,
                            })?;
                            check_finite(&r, id, "residual")?;
                            Ok(r[0])
                        };
                        let (mut lo, mut hi) = s.bracket;
                        let (flo, fhi) = (f(lo)?, f(hi)?);
                        work += 2;
                        if (flo > 0.0) == (fhi > 0.0) && flo != 0.0 && fhi != 0.0 {
                            return Err(EvalError {
                                node: id.clone(),
                                message: format!(
                                    "residual does not change sign over bracket [{lo}, {hi}]"
                                ),
                            });
                        }
                        let lo_neg = flo < 0.0;
                        for _ in 0..s.iterations {
                            let mid = 0.5 * (lo + hi);
                            let fm = f(mid)?;
                            work += 1;
                            if (fm < 0.0) == lo_neg {
                                lo = mid;
                            } else {
                                hi = mid;
                            }
                        }
                        vec![0.5 * (lo + hi)]
                    }
                };
                check_finite(&values, id, "output")?;
                if values.len() != c.outputs.len() {
                    return Err(EvalError {
                        node: id.clone(),
                        message: "atom returned the wrong number of outputs".into(),
                    });
                }
                values.into_iter().map(Val::Number).collect()
            };
            for (p, v) in c.outputs.iter().zip(&values) {
                if let Val::Number(v) = v {
                    if p.valid.is_some_and(|(lo, hi)| *v < lo || *v > hi) {
                        out_of_range.push(p.name.clone());
                    }
                }
            }
            if !out_of_range.is_empty() {
                validity = validity.max(Validity::Extrapolated);
            }
            outs[ni] = Some(NodeOut {
                values,
                validity,
                out_of_range,
                used: extra.used,
                reads: extra.reads,
                note: extra.note,
            });
        }
        Ok(Evaluation {
            flow: self,
            inputs: ins,
            in_vals,
            outs: outs.into_iter().map(Option::unwrap).collect(),
            work_units: work,
        })
    }

    /// Run the document's own golden cases. Returns failures as readable lines.
    pub fn run_tests(&self, cx: &Context) -> Vec<String> {
        let mut fails = Vec::new();
        for t in &self.doc.tests {
            let mut inputs = BTreeMap::new();
            for (i, fi) in self.doc.inputs.iter().enumerate() {
                match t
                    .inputs
                    .get(&fi.name)
                    .map(|v| param_si(cx.units, v, self.input_dims[i]))
                {
                    Some(Ok(x)) => {
                        inputs.insert(
                            fi.name.clone(),
                            InputValue {
                                q: Quantity::si(x, self.input_dims[i]),
                                source: SourceLabel::Authored,
                            },
                        );
                    }
                    Some(Err(e)) => fails.push(format!("{}: input {}: {e}", t.name, fi.name)),
                    None => fails.push(format!("{}: input {} missing", t.name, fi.name)),
                }
            }
            let ev = match self.eval(cx, &inputs) {
                Ok(e) => e,
                Err(e) => {
                    fails.push(format!("{}: {e}", t.name));
                    continue;
                }
            };
            for (kind, ex) in &t.expect {
                let Some(q) = ev.published(kind) else {
                    fails.push(format!("{}: {kind} is not published", t.name));
                    continue;
                };
                let got = match &ex.unit {
                    Some(u) => cx.units.to(q, u).map_err(|e| e.to_string()),
                    None => Ok(q.value),
                };
                match got {
                    Ok(g) if (g - ex.value).abs() <= ex.tol => {}
                    Ok(g) => fails.push(format!(
                        "{}: {kind} = {g}, expected {} ± {}",
                        t.name, ex.value, ex.tol
                    )),
                    Err(e) => fails.push(format!("{}: {kind}: {e}", t.name)),
                }
            }
        }
        fails
    }
}

/// One input of an explained value (H6.5).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExplainInput {
    pub port: String,
    pub quantity: String,
    pub value: Quantity,
    pub source: String,
    /// `node.port` of the upstream value to explain next, if derived here.
    pub explanation_ref: Option<String>,
    /// A value that is not a number (flag, choice, text, list), as JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shown: Option<serde_json::Value>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Lenses {
    pub curious: String,
    pub science: String,
    pub engine: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExplainPublisher {
    pub flow: String,
    pub flow_version: String,
    pub flow_hash: ContentHash,
    pub node: String,
    pub atom: String,
    pub atom_version: String,
}

/// The explanation bundle every derived value can return (H6.5).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Explanation {
    pub subject: String,
    pub value: Quantity,
    /// A value that is not a number, as JSON (then `value` is a bare 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shown: Option<serde_json::Value>,
    /// An engine block's own explanation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<serde_json::Value>,
    pub publisher: ExplainPublisher,
    pub inputs: Vec<ExplainInput>,
    pub params: BTreeMap<String, f64>,
    pub formula_display: String,
    pub validity: Validity,
    pub out_of_range: Vec<String>,
    pub lenses: Lenses,
    pub references: Vec<crate::decl::Reference>,
}

impl Evaluation<'_> {
    pub fn published(&self, kind: &str) -> Option<Quantity> {
        let (n, o) = *self.flow.published.get(kind)?;
        Some(Quantity::si(
            self.outs[n].values[o].num()?,
            self.flow.out_dims[n][o],
        ))
    }
    /// Any node output (`node.port`) or flow input (`in.name`) as a value.
    pub fn value_of(&self, r: &str) -> Option<&Val> {
        let (a, b) = split_ref(r)?;
        if a == "in" {
            let i = self.flow.doc.inputs.iter().position(|x| x.name == b)?;
            return self.in_vals.get(i);
        }
        let n = self.flow.doc.nodes.iter().position(|x| x.id == a)?;
        let o = self.flow.nodes[n]
            .outputs
            .iter()
            .position(|p| p.name == b)?;
        self.outs[n].values.get(o)
    }
    /// An engine block's own explanation, by node id.
    pub fn note_of(&self, node: &str) -> Option<&serde_json::Value> {
        let n = self.flow.doc.nodes.iter().position(|x| x.id == node)?;
        self.outs[n].note.as_ref()
    }
    /// The world facts a node output rests on: the reads of every block it
    /// depended on, following only the inputs each block used (an `and`
    /// stops at its first false input), sorted by field. The same evidence a
    /// K8 predicate gives for the same condition.
    pub fn evidence(&self, r: &str) -> Option<Vec<(String, sse_core::cbor::Value)>> {
        let (a, _) = split_ref(r)?;
        let n = self.flow.doc.nodes.iter().position(|x| x.id == a)?;
        let mut seen = BTreeSet::new();
        let mut stack = vec![n];
        let mut out = Vec::new();
        while let Some(k) = stack.pop() {
            if !seen.insert(k) {
                continue;
            }
            let o = &self.outs[k];
            out.extend(o.reads.iter().cloned());
            let c = &self.flow.nodes[k];
            let used: Vec<usize> = match &o.used {
                Some(u) => u.clone(),
                None => (0..c.sources.len()).collect(),
            };
            for i in used {
                if let Some(Source::Node(d, _)) = c.sources.get(i) {
                    stack.push(*d);
                }
            }
        }
        out.sort_by(|x, y| x.0.cmp(&y.0));
        out.dedup();
        Some(out)
    }
    /// Every atom a published kind rests on, its own included, sorted (MP1c:
    /// a value is ESTIMATED when one of them is a rescaled table).
    pub fn upstream_atoms(&self, kind: &str) -> Vec<String> {
        let Some(&(n, _)) = self.flow.published.get(kind) else {
            return vec![];
        };
        let mut seen = BTreeSet::new();
        let mut stack = vec![n];
        let mut out = BTreeSet::new();
        while let Some(k) = stack.pop() {
            if !seen.insert(k) {
                continue;
            }
            let d = &self.flow.doc.nodes[k];
            if let Some(a) = d.atom.as_ref().or(d.solve.as_ref().map(|s| &s.atom)) {
                out.insert(a.clone());
            }
            for s in &self.flow.nodes[k].sources {
                if let Source::Node(d, _) = s {
                    stack.push(*d);
                }
            }
        }
        out.into_iter().collect()
    }
    pub fn validity_of(&self, kind: &str) -> Option<Validity> {
        let (n, _) = *self.flow.published.get(kind)?;
        Some(self.outs[n].validity)
    }
    /// Explain a published quantity kind.
    pub fn explain(&self, kind: &str) -> Option<Explanation> {
        let (n, o) = *self.flow.published.get(kind)?;
        Some(self.explain_node(n, o, kind))
    }
    /// Explain any node output by `node.port`.
    pub fn explain_port(&self, r: &str) -> Option<Explanation> {
        let (a, b) = split_ref(r)?;
        let n = self.flow.doc.nodes.iter().position(|x| x.id == a)?;
        let o = self.flow.nodes[n]
            .outputs
            .iter()
            .position(|p| p.name == b)?;
        Some(self.explain_node(n, o, r))
    }

    fn explain_node(&self, n: usize, o: usize, subject: &str) -> Explanation {
        let f = self.flow;
        let c = &f.nodes[n];
        let out = &self.outs[n];
        let node_id = &f.doc.nodes[n].id;
        let shown = |v: &Val| match v {
            Val::Number(_) => None,
            other => Some(serde_json::to_value(other).expect("plain values")),
        };
        let inputs: Vec<ExplainInput> = c
            .inputs
            .iter()
            .zip(&c.sources)
            .map(|(p, s)| match *s {
                Source::Input(i) => ExplainInput {
                    port: p.name.clone(),
                    quantity: f.doc.inputs[i].kind.clone(),
                    value: self.inputs[i].q,
                    source: serde_json::to_value(self.inputs[i].source)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    explanation_ref: None,
                    shown: shown(&self.in_vals[i]),
                },
                Source::Node(d, k) => ExplainInput {
                    port: p.name.clone(),
                    quantity: f.nodes[d].outputs[k].kind.clone(),
                    value: Quantity::si(
                        self.outs[d].values[k].num().unwrap_or(0.0),
                        f.out_dims[d][k],
                    ),
                    source: "derived".into(),
                    explanation_ref: Some(format!(
                        "{}.{}",
                        f.doc.nodes[d].id, f.nodes[d].outputs[k].name
                    )),
                    shown: shown(&self.outs[d].values[k]),
                },
            })
            .collect();
        let params: BTreeMap<String, f64> = c
            .decl
            .params
            .iter()
            .zip(&c.params)
            .map(|(p, v)| (p.name.clone(), *v))
            .collect();
        let solve_note = c
            .solve
            .as_ref()
            .map(|(_, s)| {
                format!(
                    " Solved for {} by bisection over [{}, {}] with {} fixed iterations.",
                    s.unknown, s.bracket.0, s.bracket.1, s.iterations
                )
            })
            .unwrap_or_default();
        let validity_note = match out.validity {
            Validity::InRange => String::new(),
            Validity::Extrapolated => format!(
                " Outside the range where this holds ({}); treat as extrapolated.",
                out.out_of_range.join(", ")
            ),
            Validity::ImpossibleInput => {
                " Depends on an input the GM marked physically impossible.".into()
            }
        };
        let engine = format!(
            "flow {}@{} ({}) node {} atom {}@{}; inputs {}; params {}; validity {:?}",
            f.doc.id,
            f.doc.version,
            f.hash.hex(),
            node_id,
            c.decl.id,
            c.decl.version,
            inputs
                .iter()
                .map(|i| format!("{}={:e}", i.port, i.value.value))
                .collect::<Vec<_>>()
                .join(", "),
            params
                .iter()
                .map(|(k, v)| format!("{k}={v:e}"))
                .collect::<Vec<_>>()
                .join(", "),
            out.validity
        );
        Explanation {
            subject: subject.into(),
            value: Quantity::si(out.values[o].num().unwrap_or(0.0), f.out_dims[n][o]),
            shown: shown(&out.values[o]),
            note: out.note.clone(),
            publisher: ExplainPublisher {
                flow: f.doc.id.clone(),
                flow_version: f.doc.version.clone(),
                flow_hash: f.hash,
                node: node_id.clone(),
                atom: c.decl.id.clone(),
                atom_version: c.decl.version.clone(),
            },
            inputs,
            params,
            formula_display: c.decl.explain.formula.clone(),
            validity: out.validity,
            out_of_range: out.out_of_range.clone(),
            lenses: Lenses {
                curious: c.decl.explain.curious.clone() + &validity_note,
                science: format!(
                    "{} {}{}{}",
                    c.decl.explain.science, c.decl.explain.formula, solve_note, validity_note
                )
                .trim()
                .to_string(),
                engine,
            },
            references: c.decl.references.clone(),
        }
    }
}
