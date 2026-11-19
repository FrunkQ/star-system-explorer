//! One condition language (record design 3.3).
//!
//! The classifier's K8 predicate (all, any, not, has, range, is, compare,
//! near, tag, within) and the compare and combine blocks are one thing in
//! two notations. [`to_blocks`] writes any predicate as a flow of logic
//! blocks; evaluated against the same subject, the flow gives the same
//! answer and the same evidence ([`sse_flow::Evaluation::evidence`]) as
//! `Predicate::eval`, staying (hysteresis) or not. The compact document
//! form stays, because Orac writes it well and rule tables read better as
//! tables; the flow editor shows the same condition as blocks.

use crate::blocks::{id_of, Op, MAX_ARITY};
use serde_json::{json, Value as J};
use sse_core::cbor::Value as C;
use sse_core::Address;
use sse_flow::flow::{NodeDoc, WireDoc};
use sse_flow::{FlowDoc, World};
use sse_sift::predicate::{Op as Cmp, Predicate, Subject};
use std::collections::BTreeMap;

/// A K8 subject as the world a condition flow reads: its facts (and
/// `parent.<field>`), its tags. Other addresses read nothing.
pub struct SubjectWorld<'a>(pub &'a Subject);

impl SubjectWorld<'_> {
    fn mine(&self, a: Option<&Address>) -> bool {
        a.is_none() || a == self.0.address.as_ref()
    }
}

impl World for SubjectWorld<'_> {
    fn fact(&self, address: Option<&Address>, field: &str) -> Option<C> {
        self.mine(address)
            .then(|| self.0.get(field).cloned())
            .flatten()
    }
    fn has_tag(&self, address: Option<&Address>, tag: &str) -> bool {
        self.mine(address) && self.0.tags.contains(tag)
    }
    fn facts(&self, address: Option<&Address>) -> BTreeMap<String, C> {
        if self.mine(address) {
            self.0.facts.clone()
        } else {
            BTreeMap::new()
        }
    }
    fn parent_facts(&self, address: Option<&Address>) -> BTreeMap<String, C> {
        if self.mine(address) {
            self.0.parent.clone()
        } else {
            BTreeMap::new()
        }
    }
    fn tags(&self, address: Option<&Address>) -> std::collections::BTreeSet<String> {
        if self.mine(address) {
            self.0.tags.clone()
        } else {
            Default::default()
        }
    }
}

/// A condition written as blocks: the flow, and the output that answers.
#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub doc: FlowDoc,
    /// `node.port` of the flag that answers the condition.
    pub output: String,
}

struct B {
    nodes: Vec<NodeDoc>,
    wires: Vec<WireDoc>,
}

impl B {
    fn node(&mut self, op: Op, config: Option<J>) -> String {
        let id = format!("n{}", self.nodes.len());
        self.nodes.push(NodeDoc {
            id: id.clone(),
            atom: Some(id_of(op).into()),
            solve: None,
            params: BTreeMap::new(),
            config,
            r#box: None,
        });
        id
    }
    fn wire(&mut self, from: &str, to: String) {
        self.wires.push(WireDoc {
            from: from.into(),
            to,
        });
    }
    /// `and` or `or` over refs, in order, nesting past the arity limit so the
    /// short-circuit (and so the evidence) is unchanged.
    fn combine(&mut self, op: Op, refs: Vec<String>) -> String {
        if refs.len() == 1 {
            return refs.into_iter().next().expect("one");
        }
        let refs = if refs.len() > MAX_ARITY {
            let mut head: Vec<String> = refs[..MAX_ARITY - 1].to_vec();
            head.push(self.combine(op, refs[MAX_ARITY - 1..].to_vec()));
            head
        } else {
            refs
        };
        let n = self.node(op, Some(json!({ "inputs": refs.len() })));
        const L: &str = "abcdefghijklmnop";
        for (i, r) in refs.iter().enumerate() {
            self.wire(r, format!("{n}.{}", &L[i..=i]));
        }
        format!("{n}.holds")
    }
    fn read(&mut self, field: &str, shape: &str) -> String {
        self.node(Op::Read, Some(json!({ "field": field, "shape": shape })))
    }
    fn go(&mut self, p: &Predicate) -> Result<String, String> {
        Ok(match p {
            Predicate::All { of } | Predicate::Any { of } => {
                let all = matches!(p, Predicate::All { .. });
                if of.is_empty() {
                    let k = self.node(Op::Const, Some(json!({ "shape": "flag", "value": all })));
                    return Ok(format!("{k}.value"));
                }
                let refs = of.iter().map(|q| self.go(q)).collect::<Result<_, _>>()?;
                self.combine(if all { Op::And } else { Op::Or }, refs)
            }
            Predicate::Not { p } => {
                let r = self.go(p)?;
                let n = self.node(Op::Not, None);
                self.wire(&r, format!("{n}.a"));
                format!("{n}.holds")
            }
            Predicate::Has { field } => {
                let n = self.node(Op::HasFact, Some(json!({ "field": field })));
                format!("{n}.holds")
            }
            Predicate::Range {
                field,
                min,
                max,
                margin,
            } => {
                let r = self.read(field, "number");
                let mut cfg = serde_json::Map::new();
                if let Some(x) = min {
                    cfg.insert("min".into(), json!(x));
                }
                if let Some(x) = max {
                    cfg.insert("max".into(), json!(x));
                }
                if *margin != 0.0 {
                    cfg.insert("margin".into(), json!(margin));
                }
                let n = self.node(Op::InRange, Some(J::Object(cfg)));
                self.wire(&format!("{r}.value"), format!("{n}.x"));
                self.combine(Op::And, vec![format!("{n}.holds"), format!("{r}.present")])
            }
            Predicate::Is { field, value } => {
                let (shape, v) = match value {
                    C::Text(t) => ("text", json!(t)),
                    C::Bool(b) => ("flag", json!(b)),
                    C::Int(i) => ("number", json!(*i as f64)),
                    C::Float(f) => ("number", json!(f)),
                    other => return Err(format!("`is` on {field}: {other:?} has no block form")),
                };
                let r = self.read(field, shape);
                let k = self.node(Op::Const, Some(json!({ "shape": shape, "value": v })));
                let e = self.node(Op::Equal, Some(json!({ "shape": shape })));
                self.wire(&format!("{r}.value"), format!("{e}.a"));
                self.wire(&format!("{k}.value"), format!("{e}.b"));
                self.combine(Op::And, vec![format!("{e}.holds"), format!("{r}.present")])
            }
            Predicate::Compare {
                left,
                op,
                right,
                factor,
                margin,
            } => {
                let (l, r) = (self.read(left, "number"), self.read(right, "number"));
                let o = match op {
                    Cmp::Lt => Op::Less,
                    Cmp::Le => Op::AtMost,
                    Cmp::Gt => Op::Greater,
                    Cmp::Ge => Op::AtLeast,
                };
                let c = self.node(o, Some(json!({ "factor": factor, "margin": margin })));
                self.wire(&format!("{l}.value"), format!("{c}.a"));
                self.wire(&format!("{r}.value"), format!("{c}.b"));
                self.combine(
                    Op::And,
                    vec![
                        format!("{c}.holds"),
                        format!("{l}.present"),
                        format!("{r}.present"),
                    ],
                )
            }
            Predicate::Near { a, b, tolerance } => {
                let (x, y) = (self.read(a, "number"), self.read(b, "number"));
                let c = self.node(Op::Near, Some(json!({ "tolerance": tolerance })));
                self.wire(&format!("{x}.value"), format!("{c}.a"));
                self.wire(&format!("{y}.value"), format!("{c}.b"));
                self.combine(
                    Op::And,
                    vec![
                        format!("{c}.holds"),
                        format!("{x}.present"),
                        format!("{y}.present"),
                    ],
                )
            }
            Predicate::Tag { tag } => {
                let n = self.node(Op::HasTag, Some(json!({ "tag": tag })));
                format!("{n}.holds")
            }
            Predicate::Within {
                field,
                from_s,
                to_s,
            } => {
                // trunc(t) >= now + from and trunc(t) <= now + to (whole
                // seconds, as K8 reads the time as an integer).
                let r = self.read(field, "number");
                let t = self.node(Op::Trunc, None);
                self.wire(&format!("{r}.value"), format!("{t}.x"));
                let now = self.node(Op::Now, None);
                let bound = |b: &mut B, s: i64| {
                    let k = b.node(Op::Const, Some(json!({ "value": s as f64 })));
                    let a = b.node(Op::Add, None);
                    b.wire(&format!("{now}.now"), format!("{a}.a"));
                    b.wire(&format!("{k}.value"), format!("{a}.b"));
                    a
                };
                let (lo, hi) = (bound(self, *from_s), bound(self, *to_s));
                let ge = self.node(Op::AtLeast, None);
                self.wire(&format!("{t}.value"), format!("{ge}.a"));
                self.wire(&format!("{lo}.value"), format!("{ge}.b"));
                let le = self.node(Op::AtMost, None);
                self.wire(&format!("{t}.value"), format!("{le}.a"));
                self.wire(&format!("{hi}.value"), format!("{le}.b"));
                self.combine(
                    Op::And,
                    vec![
                        format!("{ge}.holds"),
                        format!("{le}.holds"),
                        format!("{r}.present"),
                    ],
                )
            }
        })
    }
}

impl Condition {
    /// The condition as a box (J3) with one output, `holds`: a rule that
    /// any flow, box or program can use as one block.
    pub fn into_box(self, id: &str, version: &str) -> sse_flow::BoxDoc {
        sse_flow::BoxDoc {
            format: sse_flow::boxes::BOX_FORMAT.into(),
            id: id.into(),
            version: version.into(),
            inputs: vec![],
            outputs: vec![sse_flow::boxes::BoxOutput {
                name: "holds".into(),
                from: self.output,
            }],
            nodes: self.doc.nodes,
            wires: self.doc.wires,
            promoted_params: vec![],
            explain: Default::default(),
            references: vec![],
            tests: vec![],
            fictional: false,
        }
    }
}

/// Write a K8 predicate as a flow of logic blocks.
pub fn to_blocks(p: &Predicate, id: &str) -> Result<Condition, String> {
    let mut b = B {
        nodes: vec![],
        wires: vec![],
    };
    let output = b.go(p)?;
    Ok(Condition {
        doc: FlowDoc {
            format: "sse.flow/1".into(),
            id: id.into(),
            version: "1.0.0".into(),
            hook: "condition".into(),
            inputs: vec![],
            nodes: b.nodes,
            wires: b.wires,
            publishes: vec![],
            tests: vec![],
        },
        output,
    })
}
