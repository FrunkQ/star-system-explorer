//! Boxes: blocks made of blocks (track J3, record design section 4).
//!
//! A box is a flow of blocks with its own ports, published under `id@version`
//! in a pack. From outside it looks exactly like an atom: ports, params,
//! validity, cost class, references, explain ([`box_decl`]). A node uses one
//! with `box: "id@range"`.
//!
//! - **Compiled by inlining.** A flow using boxes evaluates as the flat flow
//!   it stands for: inner nodes take ids `<box node>/<inner node>`, so the
//!   results and explain traces are those of the hand-wired twin, and the
//!   flow hash is the twin's hash.
//! - Boxes nest, at most [`MAX_DEPTH`] deep, and no box contains itself at
//!   any depth. Both are readable errors naming the chain.
//! - A box has its own golden cases ([`run_box_tests`]), keyed by output name.
//! - Replacing a box is a pack's explicit `replaces`, as for flows (H7).

use crate::decl::{AtomDecl, BlockClass, CostClass, ExplainText, Param, Port, Reference};
use crate::flow::{
    param_si, split_ref, Context, Expect, FlowDoc, FlowError, FlowInputDoc, InputValue, NodeDoc,
    ParamValue, PublishDoc, SourceLabel, WireDoc,
};
use crate::Flow;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sse_core::Quantity;
use std::collections::{BTreeMap, BTreeSet};

pub const BOX_FORMAT: &str = "sse.box/1";
/// Deepest nesting of boxes in boxes.
pub const MAX_DEPTH: usize = 8;

/// A box output: its name and the inner `node.port` (or `in.name`) it shows.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BoxOutput {
    pub name: String,
    pub from: String,
}

/// An inner node's parameter shown on the box, so each use can set it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Promoted {
    pub name: String,
    pub node: String,
    pub param: String,
}

/// A box's golden case: inputs by input name, expectations by output name.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BoxCase {
    pub name: String,
    pub inputs: BTreeMap<String, ParamValue>,
    pub expect: BTreeMap<String, Expect>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BoxDoc {
    pub format: String,
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub inputs: Vec<FlowInputDoc>,
    pub outputs: Vec<BoxOutput>,
    pub nodes: Vec<NodeDoc>,
    #[serde(default)]
    pub wires: Vec<WireDoc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub promoted_params: Vec<Promoted>,
    #[serde(default)]
    pub explain: ExplainText,
    /// The box's own citations, beside those of what is inside.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<BoxCase>,
    /// Invented physics: everything downstream is labelled fictional.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fictional: bool,
}

impl BoxDoc {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("box document: {e}"))
    }
    pub fn content_hash(&self) -> sse_core::ContentHash {
        sse_core::ContentHash::of(self).expect("box docs hold no NaN")
    }
}

/// Where boxes are found (the resolved stack's active boxes).
pub trait BoxSource {
    fn get_box(&self, id: &str) -> Option<&BoxDoc>;
}
impl BoxSource for BTreeMap<String, BoxDoc> {
    fn get_box(&self, id: &str) -> Option<&BoxDoc> {
        self.get(id)
    }
}
/// No boxes at all.
pub struct NoBoxes;
impl BoxSource for NoBoxes {
    fn get_box(&self, _: &str) -> Option<&BoxDoc> {
        None
    }
}

/// One use of a box in an inlined flow.
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct BoxUse {
    /// The prefix its inner nodes carry (`star` or `star/photo`).
    pub node: String,
    pub id: String,
    pub version: String,
    pub depth: usize,
    pub fictional: bool,
}

/// A flow with its boxes inlined.
#[derive(Clone, PartialEq, Debug)]
pub struct Inlined {
    pub doc: FlowDoc,
    pub uses: Vec<BoxUse>,
}

/// `id@range` (the range defaults to any version).
pub fn parse_ref(r: &str) -> Result<(String, VersionReq), String> {
    let (id, range) = r.split_once('@').unwrap_or((r, "*"));
    let req = VersionReq::parse(range).map_err(|e| format!("box {r:?}: range {range:?}: {e}"))?;
    if id.is_empty() {
        return Err(format!("box {r:?} has no id"));
    }
    Ok((id.to_string(), req))
}

/// A flattened body (a flow's or a box's): nodes and wires with final ids,
/// where its own inputs are consumed, and its output refs resolved.
#[derive(Default)]
struct Body {
    nodes: Vec<NodeDoc>,
    wires: Vec<WireDoc>,
    /// Body input name -> final `node.port` targets.
    feeds: BTreeMap<String, Vec<String>>,
    /// The requested refs, final: a prefixed `node.port`, or `in.<name>` for
    /// a body input passed straight through.
    outs: Vec<Option<String>>,
}

struct Expanded {
    id: String,
    inputs: BTreeSet<String>,
    feeds: BTreeMap<String, Vec<String>>,
    /// Declared outputs; `None` where an error inside was already reported.
    outs: BTreeMap<String, Option<String>>,
}

struct Inliner<'a> {
    boxes: &'a dyn BoxSource,
    errs: Vec<FlowError>,
    uses: Vec<BoxUse>,
}

impl Inliner<'_> {
    fn err(&mut self, at: &str, m: String) {
        self.errs.push(FlowError {
            at: at.into(),
            message: m,
        });
    }

    /// Expand box node `n` (already named `at`), or `None` after an error.
    fn expand(
        &mut self,
        at: &str,
        n: &NodeDoc,
        r: &str,
        chain: &mut Vec<String>,
    ) -> Option<(Body, Expanded)> {
        if n.atom.is_some() || n.solve.is_some() || n.config.is_some() {
            self.err(at, "a box node has only `box` and `params`".into());
        }
        let (id, req) = match parse_ref(r) {
            Ok(x) => x,
            Err(e) => {
                self.err(at, e);
                return None;
            }
        };
        if let Some(from) = chain.iter().position(|x| *x == id) {
            let mut c = chain[from..].to_vec();
            c.push(id.clone());
            self.err(
                at,
                format!(
                    "box {id} contains itself: {} (a box may not contain itself at any depth)",
                    c.join(" > ")
                ),
            );
            return None;
        }
        if chain.len() + 1 > MAX_DEPTH {
            let mut c = chain.clone();
            c.push(id.clone());
            self.err(
                at,
                format!(
                    "boxes nest {} deep, past the limit of {MAX_DEPTH}: {}",
                    c.len(),
                    c.join(" > ")
                ),
            );
            return None;
        }
        let Some(b) = self.boxes.get_box(&id) else {
            self.err(at, format!("no box {id} in the stack"));
            return None;
        };
        let b = b.clone();
        match Version::parse(&b.version) {
            Ok(v) if req.matches(&v) => {}
            Ok(v) => {
                self.err(at, format!("wants box {r}, but the stack has {id} {v}"));
                return None;
            }
            Err(e) => {
                self.err(at, format!("box {id} version {:?}: {e}", b.version));
                return None;
            }
        }
        if b.format != BOX_FORMAT {
            self.err(at, format!("box {id}: format must be {BOX_FORMAT:?}"));
        }
        self.uses.push(BoxUse {
            node: at.to_string(),
            id: id.clone(),
            version: b.version.clone(),
            depth: chain.len() + 1,
            fictional: b.fictional,
        });
        // Promoted parameters set on the box node go to their inner node.
        let mut inner = b.nodes.clone();
        for (k, v) in &n.params {
            match b.promoted_params.iter().find(|p| p.name == *k) {
                None => self.err(
                    &format!("{at}.{k}"),
                    format!("box {id} has no parameter {k:?}"),
                ),
                Some(p) => match inner.iter_mut().find(|x| x.id == p.node) {
                    None => self.err(
                        &format!("{at}.{k}"),
                        format!(
                            "box {id} promotes {k} from node {:?}, which it does not have",
                            p.node
                        ),
                    ),
                    Some(x) => {
                        x.params.insert(p.param.clone(), v.clone());
                    }
                },
            }
        }
        let refs: Vec<String> = b.outputs.iter().map(|o| o.from.clone()).collect();
        chain.push(id.clone());
        let body = self.body(at, &inner, &b.wires, &refs, chain);
        chain.pop();
        let inputs: BTreeSet<String> = b.inputs.iter().map(|i| i.name.clone()).collect();
        for k in body.feeds.keys() {
            if !inputs.contains(k) {
                self.err(
                    at,
                    format!("box {id} reads in.{k}, which is not one of its inputs"),
                );
            }
        }
        let outs = b
            .outputs
            .iter()
            .zip(&body.outs)
            .map(|(o, r)| (o.name.clone(), r.clone()))
            .collect();
        let e = Expanded {
            id,
            inputs,
            feeds: body.feeds.clone(),
            outs,
        };
        Some((body, e))
    }

    /// Flatten one body under `prefix`; `refs` are the refs to resolve
    /// (a box's outputs, a flow's publishes).
    fn body(
        &mut self,
        prefix: &str,
        nodes: &[NodeDoc],
        wires: &[WireDoc],
        refs: &[String],
        chain: &mut Vec<String>,
    ) -> Body {
        let pre = |id: &str| {
            if prefix.is_empty() {
                id.to_string()
            } else {
                format!("{prefix}/{id}")
            }
        };
        let mut out = Body::default();
        let mut boxes: BTreeMap<String, Expanded> = BTreeMap::new();
        let mut broken: BTreeSet<String> = BTreeSet::new();
        for n in nodes {
            let at = pre(&n.id);
            if n.id.contains('/') {
                self.err(
                    &at,
                    "a node id cannot contain '/' (it marks a box's inside)".into(),
                );
            }
            match &n.r#box {
                None => {
                    let mut m = n.clone();
                    m.id = at;
                    out.nodes.push(m);
                }
                Some(r) => match self.expand(&at, n, r, chain) {
                    Some((b, e)) => {
                        out.nodes.extend(b.nodes);
                        out.wires.extend(b.wires);
                        boxes.insert(n.id.clone(), e);
                    }
                    None => {
                        broken.insert(n.id.clone());
                    }
                },
            }
        }
        // Sources wired into each box's inputs.
        let mut into: BTreeMap<(String, String), String> = BTreeMap::new();
        for w in wires {
            if let Some((t, p)) = split_ref(&w.to) {
                if boxes.contains_key(t) {
                    into.insert((t.to_string(), p.to_string()), w.from.clone());
                }
            }
        }
        let hops = boxes.len() + 1;
        // A ref of this body in final form.
        let resolve = |r: &str, il: &mut Self| -> Option<String> {
            let mut r = r.to_string();
            for _ in 0..=hops {
                let Some((a, b)) = split_ref(&r) else {
                    il.err(&r, "bad reference (want node.port or in.name)".into());
                    return None;
                };
                if a == "in" {
                    return Some(r);
                }
                if broken.contains(a) {
                    return None;
                }
                let Some(e) = boxes.get(a) else {
                    return Some(format!("{}.{b}", pre(a)));
                };
                let src = match e.outs.get(b) {
                    None => {
                        il.err(&pre(a), format!("box {} has no output {b:?}", e.id));
                        return None;
                    }
                    Some(None) => return None,
                    Some(Some(s)) => s,
                };
                match split_ref(src) {
                    Some(("in", x)) => match into.get(&(a.to_string(), x.to_string())) {
                        Some(up) => r = up.clone(),
                        None => {
                            il.err(&format!("{}.{x}", pre(a)), "box input is not wired".into());
                            return None;
                        }
                    },
                    _ => return Some(src.clone()),
                }
            }
            il.err(
                "graph",
                "boxes pass values straight through in a cycle".into(),
            );
            None
        };
        for w in wires {
            let at = format!("wire {} -> {}", w.from, w.to);
            let Some((t, p)) = split_ref(&w.to) else {
                self.err(&at, format!("bad target {:?}", w.to));
                continue;
            };
            if broken.contains(t) {
                continue;
            }
            let targets: Vec<String> = match boxes.get(t) {
                Some(e) if !e.inputs.contains(p) => {
                    self.err(&at, format!("box {} has no input {p:?}", e.id));
                    continue;
                }
                Some(e) => e.feeds.get(p).cloned().unwrap_or_default(),
                None => vec![format!("{}.{p}", pre(t))],
            };
            if targets.is_empty() {
                continue;
            }
            let Some(src) = resolve(&w.from, self) else {
                continue;
            };
            for to in targets {
                match split_ref(&src) {
                    Some(("in", x)) => out.feeds.entry(x.to_string()).or_default().push(to),
                    _ => out.wires.push(WireDoc {
                        from: src.clone(),
                        to,
                    }),
                }
            }
        }
        for (k, e) in &boxes {
            for x in e.feeds.keys() {
                if !into.contains_key(&(k.clone(), x.clone())) {
                    self.err(
                        &format!("{}.{x}", pre(k)),
                        format!("box input is not wired (box {})", e.id),
                    );
                }
            }
        }
        out.outs = refs.iter().map(|r| resolve(r, self)).collect();
        out
    }
}

/// Inline every box in a flow. A flow without boxes comes back unchanged
/// (same document, same hash).
pub fn inline(doc: &FlowDoc, boxes: &dyn BoxSource) -> Result<Inlined, Vec<FlowError>> {
    if doc.nodes.iter().all(|n| n.r#box.is_none()) {
        return Ok(Inlined {
            doc: doc.clone(),
            uses: vec![],
        });
    }
    let mut il = Inliner {
        boxes,
        errs: vec![],
        uses: vec![],
    };
    let refs: Vec<String> = doc.publishes.iter().map(|p| p.from.clone()).collect();
    let body = il.body("", &doc.nodes, &doc.wires, &refs, &mut Vec::new());
    if !il.errs.is_empty() {
        return Err(il.errs);
    }
    Ok(Inlined {
        doc: flat(doc, body, &doc.publishes),
        uses: il.uses,
    })
}

/// The flat document: the body's nodes and wires, flow inputs wired back in
/// (inner wires first in document order, then each input's fan-out).
fn flat(doc: &FlowDoc, body: Body, publishes: &[PublishDoc]) -> FlowDoc {
    let mut wires = body.wires;
    for (x, ts) in body.feeds {
        for t in ts {
            wires.push(WireDoc {
                from: format!("in.{x}"),
                to: t,
            });
        }
    }
    FlowDoc {
        nodes: body.nodes,
        wires,
        publishes: publishes
            .iter()
            .zip(body.outs)
            .map(|(p, r)| PublishDoc {
                kind: p.kind.clone(),
                from: r.expect("resolved, or an error was reported"),
            })
            .collect(),
        ..doc.clone()
    }
}

impl Flow {
    /// Inline the flow's boxes from `boxes`, then validate it.
    pub fn compile_with(
        doc: FlowDoc,
        cx: &Context,
        boxes: &dyn BoxSource,
    ) -> Result<Flow, Vec<FlowError>> {
        let il = inline(&doc, boxes)?;
        let mut f = Flow::compile(il.doc, cx)?;
        f.boxes = il.uses;
        Ok(f)
    }
}

/// A box compiled on its own: the flow of its inside and where its outputs
/// come from.
pub struct CompiledBox {
    pub flow: Flow,
    /// Output name -> `node.port` in `flow` (or `in.name`).
    pub outputs: BTreeMap<String, String>,
}

/// Compile a box's inside (its inner boxes from `boxes`).
pub fn compile_box(
    b: &BoxDoc,
    cx: &Context,
    boxes: &dyn BoxSource,
) -> Result<CompiledBox, Vec<FlowError>> {
    let mut il = Inliner {
        boxes,
        errs: vec![],
        uses: vec![],
    };
    if b.format != BOX_FORMAT {
        il.err(&b.id, format!("format must be {BOX_FORMAT:?}"));
    }
    let refs: Vec<String> = b.outputs.iter().map(|o| o.from.clone()).collect();
    let body = il.body("", &b.nodes, &b.wires, &refs, &mut vec![b.id.clone()]);
    if !il.errs.is_empty() {
        return Err(il.errs);
    }
    let outputs = b
        .outputs
        .iter()
        .zip(&body.outs)
        .map(|(o, r)| (o.name.clone(), r.clone().expect("resolved")))
        .collect();
    let doc = FlowDoc {
        format: crate::flow::FLOW_FORMAT.into(),
        id: b.id.clone(),
        version: b.version.clone(),
        hook: "box".into(),
        inputs: b.inputs.clone(),
        nodes: vec![],
        wires: vec![],
        publishes: vec![],
        tests: vec![],
    };
    let doc = flat(&doc, body, &[]);
    let mut flow = Flow::compile(doc, cx)?;
    flow.boxes = il.uses;
    Ok(CompiledBox { flow, outputs })
}

/// A box's face: the declaration it shows to anything that takes an atom.
/// Validity is the intersection of what is inside, cost class the maximum,
/// references the union; it is fictional if anything inside is.
pub fn box_decl(
    b: &BoxDoc,
    cx: &Context,
    boxes: &dyn BoxSource,
) -> Result<AtomDecl, Vec<FlowError>> {
    let c = compile_box(b, cx, boxes)?;
    let mut inputs = Vec::new();
    for i in &b.inputs {
        let (dim, shape) = c
            .flow
            .port_of(&format!("in.{}", i.name))
            .expect("compiled input");
        inputs.push(Port {
            name: i.name.clone(),
            kind: i.kind.clone(),
            dim,
            valid: c.flow.input_validity(&i.name),
            shape: (shape != crate::Shape::Number).then_some(shape),
            any_dim: false,
        });
    }
    let mut outputs = Vec::new();
    for o in &b.outputs {
        let r = &c.outputs[&o.name];
        let (dim, shape) = c.flow.port_of(r).ok_or_else(|| {
            vec![FlowError {
                at: b.id.clone(),
                message: format!("output {} reads {r}, which is not there", o.name),
            }]
        })?;
        outputs.push(Port {
            name: o.name.clone(),
            kind: o.name.clone(),
            dim,
            valid: None,
            shape: (shape != crate::Shape::Number).then_some(shape),
            any_dim: false,
        });
    }
    let mut params = Vec::new();
    for p in &b.promoted_params {
        let atom = b
            .nodes
            .iter()
            .find(|n| n.id == p.node)
            .and_then(|n| n.atom.as_ref().or(n.solve.as_ref().map(|s| &s.atom)))
            .and_then(|a| cx.atoms.get(a));
        let Some((_, d)) = atom.and_then(|a| a.decl().param(&p.param).map(|(i, d)| (i, d.clone())))
        else {
            return Err(vec![FlowError {
                at: format!("{}.{}", b.id, p.name),
                message: format!(
                    "promotes {}.{}, which is not an atom parameter",
                    p.node, p.param
                ),
            }]);
        };
        params.push(Param {
            name: p.name.clone(),
            ..d
        });
    }
    let decls: Vec<&AtomDecl> = c.flow.decls().collect();
    let cost = decls
        .iter()
        .map(|d| d.cost_class)
        .max()
        .unwrap_or(CostClass::Trivial);
    let mut references = b.references.clone();
    for d in &decls {
        for r in &d.references {
            if !references.contains(r) {
                references.push(r.clone());
            }
        }
    }
    let class = if decls.iter().all(|d| d.class.is_some()) && !decls.is_empty() {
        Some(BlockClass::Logic)
    } else {
        None
    };
    Ok(AtomDecl {
        id: b.id.clone(),
        version: b.version.clone(),
        inputs,
        outputs,
        params,
        cost_class: cost,
        references,
        explain: b.explain.clone(),
        class,
        dims: None,
    })
}

/// Run a box's own golden cases. Failures come back as readable lines.
pub fn run_box_tests(b: &BoxDoc, cx: &Context, boxes: &dyn BoxSource) -> Vec<String> {
    let c = match compile_box(b, cx, boxes) {
        Ok(c) => c,
        Err(e) => return e.iter().map(|x| format!("{}: {x}", b.id)).collect(),
    };
    let mut fails = Vec::new();
    for t in &b.tests {
        let mut inputs = BTreeMap::new();
        for i in &b.inputs {
            let (dim, _) = c
                .flow
                .port_of(&format!("in.{}", i.name))
                .expect("compiled input");
            match t.inputs.get(&i.name).map(|v| param_si(cx.units, v, dim)) {
                Some(Ok(x)) => {
                    inputs.insert(
                        i.name.clone(),
                        InputValue {
                            q: Quantity::si(x, dim),
                            source: SourceLabel::Authored,
                        },
                    );
                }
                Some(Err(e)) => fails.push(format!("{}: {}: input {}: {e}", b.id, t.name, i.name)),
                None => fails.push(format!("{}: {}: input {} missing", b.id, t.name, i.name)),
            }
        }
        let ev = match c.flow.eval(cx, &inputs) {
            Ok(e) => e,
            Err(e) => {
                fails.push(format!("{}: {}: {e}", b.id, t.name));
                continue;
            }
        };
        for (name, ex) in &t.expect {
            let Some(r) = c.outputs.get(name) else {
                fails.push(format!("{}: {}: no output {name}", b.id, t.name));
                continue;
            };
            let (dim, _) = c.flow.port_of(r).expect("compiled output");
            let Some(v) = ev.value_of(r).and_then(|v| v.num()) else {
                fails.push(format!("{}: {}: {name} is not a number", b.id, t.name));
                continue;
            };
            let got = match &ex.unit {
                Some(u) => cx
                    .units
                    .to(Quantity::si(v, dim), u)
                    .map_err(|e| e.to_string()),
                None => Ok(v),
            };
            match got {
                Ok(g) if (g - ex.value).abs() <= ex.tol => {}
                Ok(g) => fails.push(format!(
                    "{}: {}: {name} = {g}, expected {} ± {}",
                    b.id, t.name, ex.value, ex.tol
                )),
                Err(e) => fails.push(format!("{}: {}: {name}: {e}", b.id, t.name)),
            }
        }
    }
    fails
}
