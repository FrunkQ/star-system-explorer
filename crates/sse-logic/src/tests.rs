//! J2 acceptance (record design section 10): truth tables for every logic
//! atom; a K8 rule and the same condition as blocks give identical results
//! and evidence; rolls are keyed on the table secret, independent of
//! evaluation order, and the commitment verifies after reveal; `classify`
//! wired into a flow gives the class atlas's own label and explanation;
//! `place` is the second engine block.

use crate::condition::{to_blocks, SubjectWorld};
use crate::dice;
use crate::engine::{Classify, Place, PlaceEngine};
use serde_json::{json, Value as J};
use sse_core::cbor::Value as C;
use sse_core::{Address, Seed, Units};
use sse_flow::flow::{FlowInputDoc, NodeDoc, WireDoc};
use sse_flow::{
    AtomError, AtomRegistry, Call, Context, Env, Flow, FlowDoc, Out, Shape, Val, World,
};
use sse_sift::predicate::{Predicate, Subject};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

fn n(x: f64) -> Val {
    Val::Number(x)
}
fn f(b: bool) -> Val {
    Val::Flag(b)
}
fn ch(s: &str) -> Val {
    Val::Choice(s.into())
}
fn rec(j: J) -> Val {
    Val::from_json(&j, &Shape::Record).unwrap()
}
fn addr(s: &str) -> Address {
    s.parse().unwrap()
}

/// A world for the truth tables: one subject, its children, kept values.
struct W {
    s: Subject,
    kids: Vec<Address>,
    kept: BTreeMap<String, C>,
}
impl World for W {
    fn fact(&self, a: Option<&Address>, field: &str) -> Option<C> {
        SubjectWorld(&self.s).fact(a, field)
    }
    fn has_tag(&self, a: Option<&Address>, tag: &str) -> bool {
        SubjectWorld(&self.s).has_tag(a, tag)
    }
    fn children(&self, a: &Address) -> Vec<Address> {
        self.kids
            .iter()
            .filter(|k| k.parent().as_ref() == Some(a))
            .cloned()
            .collect()
    }
    fn kept(&self, a: Option<&Address>, kind: &str) -> Option<C> {
        (a.is_none())
            .then(|| self.kept.get(kind).cloned())
            .flatten()
    }
}
fn world() -> W {
    let mut s = Subject::new(
        [
            ("mass".to_string(), C::Float(2.0)),
            ("moons".to_string(), C::Int(3)),
            ("name".to_string(), C::Text("Tarn".into())),
            ("port".to_string(), C::Bool(true)),
        ]
        .into(),
    );
    s.parent = [("mass".to_string(), C::Float(10.0))].into();
    s.tags = ["garden".to_string()].into();
    s.address = Some(addr("u:t/sys:0/body:1"));
    W {
        s,
        kids: vec![
            addr("u:t/sys:0/body:1/body:1"),
            addr("u:t/sys:0/body:1/body:0"),
            addr("u:t/sys:0/body:2"),
        ],
        kept: [("trade.credits".to_string(), C::Float(1500.0))].into(),
    }
}

struct T {
    reg: AtomRegistry,
    seen: RefCell<BTreeSet<String>>,
}
impl T {
    fn new() -> Self {
        T {
            reg: crate::registry(),
            seen: RefCell::new(BTreeSet::new()),
        }
    }
    fn try_run(
        &self,
        id: &str,
        config: Option<J>,
        ins: Vec<Val>,
        env: &Env,
    ) -> Result<Out, AtomError> {
        self.seen.borrow_mut().insert(id.to_string());
        let a = self.reg.get(id).unwrap_or_else(|| panic!("no atom {id}"));
        a.check_config(config.as_ref()).map_err(AtomError)?;
        let (inputs, _) = a
            .ports(config.as_ref())
            .unwrap_or_else(|| (a.decl().inputs.clone(), a.decl().outputs.clone()));
        assert_eq!(inputs.len(), ins.len(), "{id}: inputs");
        for (p, v) in inputs.iter().zip(&ins) {
            assert!(
                v.fits(&p.shape()),
                "{id}.{}: {v:?} is not a {}",
                p.name,
                p.shape()
            );
        }
        let params: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
        let mut c = Call {
            env,
            config: config.as_ref(),
            flow_id: "test.flow",
            node_id: "n",
        };
        a.eval_typed(&ins, &params, &mut c)
    }
    fn run(&self, id: &str, config: Option<J>, ins: Vec<Val>) -> Out {
        self.try_run(id, config, ins, &Env::empty()).unwrap()
    }
    fn holds(&self, id: &str, config: Option<J>, ins: Vec<Val>) -> bool {
        self.run(id, config, ins).values[0].flag().unwrap()
    }
    fn staying(&self, id: &str, config: Option<J>, ins: Vec<Val>) -> bool {
        let env = Env {
            staying: true,
            ..Env::empty()
        };
        self.try_run(id, config, ins, &env).unwrap().values[0]
            .flag()
            .unwrap()
    }
    fn value(&self, id: &str, config: Option<J>, ins: Vec<Val>) -> Val {
        self.run(id, config, ins).values[0].clone()
    }
    fn fails(&self, id: &str, config: Option<J>, ins: Vec<Val>) -> String {
        self.try_run(id, config, ins, &Env::empty()).unwrap_err().0
    }
    fn in_world(&self, id: &str, config: J, ins: Vec<Val>) -> Out {
        let w = world();
        let env = Env {
            world: Some(&w),
            subject: w.s.address.clone(),
            now_s: 1000,
            ..Env::empty()
        };
        self.try_run(id, Some(config), ins, &env).unwrap()
    }
}

#[test]
fn every_block_is_logic_and_explained() {
    let r = crate::registry();
    let ids: Vec<&str> = r.decls().map(|d| d.id.as_str()).collect();
    assert_eq!(ids.len(), 44, "{ids:?}");
    for d in r.decls() {
        assert_eq!(d.class, Some(sse_flow::BlockClass::Logic), "{}", d.id);
        assert!(
            !d.explain.curious.is_empty() && !d.explain.science.is_empty(),
            "{}",
            d.id
        );
        assert!(
            d.references.is_empty(),
            "logic blocks carry no physics citation"
        );
    }
}

#[test]
fn truth_tables() {
    let t = T::new();
    // Compare, with factor and the staying form.
    for (id, rows) in [
        ("logic.less", [true, false, false]),
        ("logic.at_most", [true, true, false]),
        ("logic.greater", [false, false, true]),
        ("logic.at_least", [false, true, true]),
    ] {
        for (a, want) in [1.0, 2.0, 3.0].into_iter().zip(rows) {
            assert_eq!(t.holds(id, None, vec![n(a), n(2.0)]), want, "{id} {a} vs 2");
        }
    }
    assert!(t.holds(
        "logic.less",
        Some(json!({"factor": 2.0})),
        vec![n(3.0), n(2.0)]
    ));
    let m = Some(json!({"margin": 0.1}));
    assert!(!t.holds("logic.less", m.clone(), vec![n(2.1), n(2.0)]));
    assert!(t.staying("logic.less", m.clone(), vec![n(2.1), n(2.0)]));
    assert!(!t.holds("logic.greater", m.clone(), vec![n(1.9), n(2.0)]));
    assert!(t.staying("logic.greater", m, vec![n(1.9), n(2.0)]));
    // Equal, by shape.
    assert!(t.holds("logic.equal", None, vec![n(2.0), n(2.0)]));
    assert!(!t.holds("logic.equal", None, vec![n(2.0), n(2.5)]));
    let c = Some(json!({"shape": "choice"}));
    assert!(t.holds("logic.equal", c.clone(), vec![ch("pirate"), ch("pirate")]));
    assert!(!t.holds("logic.equal", c, vec![ch("pirate"), ch("patrol")]));
    assert!(t.holds(
        "logic.equal",
        Some(json!({"shape": "flag"})),
        vec![f(false), f(false)]
    ));
    // Near.
    let tol = Some(json!({"tolerance": 0.1, "margin": 0.1}));
    assert!(t.holds("logic.near", tol.clone(), vec![n(1.05), n(1.0)]));
    assert!(!t.holds("logic.near", tol.clone(), vec![n(1.15), n(1.0)]));
    assert!(t.staying("logic.near", tol, vec![n(1.15), n(1.0)]));
    // In range: inclusive, open ends, margin.
    let r = Some(json!({"min": 1.0, "max": 2.0, "margin": 0.5}));
    for (x, want, stay) in [
        (0.9, false, true),
        (1.0, true, true),
        (2.0, true, true),
        (2.6, false, false),
    ] {
        assert_eq!(
            t.holds("logic.in_range", r.clone(), vec![n(x)]),
            want,
            "{x}"
        );
        assert_eq!(
            t.staying("logic.in_range", r.clone(), vec![n(x)]),
            stay,
            "{x}"
        );
    }
    assert!(t.holds("logic.in_range", Some(json!({"min": 1.0})), vec![n(1e9)]));
    assert!(t.holds("logic.in_range", None, vec![n(-1e9)]));
    // And, or: every row, with the inputs each one used.
    for (a, b) in [(false, false), (false, true), (true, false), (true, true)] {
        let o = t.run("logic.and", None, vec![f(a), f(b)]);
        assert_eq!(o.values, vec![f(a && b)]);
        assert_eq!(o.used, Some(if a { vec![0, 1] } else { vec![0] }));
        let o = t.run("logic.or", None, vec![f(a), f(b)]);
        assert_eq!(o.values, vec![f(a || b)]);
        assert_eq!(o.used, Some(if a { vec![0] } else { vec![0, 1] }));
        assert_eq!(t.value("logic.not", None, vec![f(a)]), f(!a));
        assert_eq!(
            t.value("logic.count_true", None, vec![f(a), f(b)]),
            n((a as u8 + b as u8) as f64)
        );
    }
    let three = Some(json!({"inputs": 3}));
    let o = t.run("logic.and", three.clone(), vec![f(true), f(false), f(true)]);
    assert_eq!((o.values[0].clone(), o.used), (f(false), Some(vec![0, 1])));
    assert_eq!(
        t.value("logic.count_true", three, vec![f(true), f(false), f(true)]),
        n(2.0)
    );
    // Choose.
    let o = t.run("logic.select", None, vec![f(true), n(1.0), n(2.0)]);
    assert_eq!((o.values[0].clone(), o.used), (n(1.0), Some(vec![0, 1])));
    let o = t.run(
        "logic.select",
        Some(json!({"shape": "choice"})),
        vec![f(false), ch("a"), ch("b")],
    );
    assert_eq!((o.values[0].clone(), o.used), (ch("b"), Some(vec![0, 2])));
    let sw = Some(json!({"cases": ["pirate", "patrol"], "shape": "number"}));
    for (on, want, k) in [("pirate", 1.0, 1), ("patrol", 2.0, 2), ("trader", 9.0, 3)] {
        let o = t.run(
            "logic.switch",
            sw.clone(),
            vec![ch(on), n(1.0), n(2.0), n(9.0)],
        );
        assert_eq!(
            (o.values[0].clone(), o.used),
            (n(want), Some(vec![0, k])),
            "{on}"
        );
    }
    let lk = Some(json!({"table": {"frigate": 2, "cruiser": 5}, "default": 1}));
    assert_eq!(
        t.run("logic.lookup", lk.clone(), vec![ch("cruiser")])
            .values,
        vec![n(5.0), f(true)]
    );
    assert_eq!(
        t.run("logic.lookup", lk, vec![ch("scout")]).values,
        vec![n(1.0), f(false)]
    );
    let lk = Some(json!({"table": {"a": "x"}, "shape": "text"}));
    assert_eq!(
        t.run("logic.lookup", lk, vec![ch("b")]).values,
        vec![Val::Text(String::new()), f(false)]
    );
    // Numbers.
    for (id, want) in [
        ("logic.add", 8.0),
        ("logic.subtract", 4.0),
        ("logic.multiply", 12.0),
        ("logic.divide", 3.0),
        ("logic.min", 2.0),
        ("logic.max", 6.0),
    ] {
        assert_eq!(t.value(id, None, vec![n(6.0), n(2.0)]), n(want), "{id}");
    }
    assert!(t
        .fails("logic.divide", None, vec![n(1.0), n(0.0)])
        .contains("division by zero"));
    assert_eq!(
        t.value("logic.clamp", None, vec![n(5.0), n(0.0), n(3.0)]),
        n(3.0)
    );
    assert_eq!(
        t.value("logic.clamp", None, vec![n(-1.0), n(0.0), n(3.0)]),
        n(0.0)
    );
    assert!(t
        .fails("logic.clamp", None, vec![n(1.0), n(3.0), n(0.0)])
        .contains("above"));
    for (x, r, fl, c, tr, ab) in [
        (2.5, 3.0, 2.0, 3.0, 2.0, 2.5),
        (-2.5, -3.0, -3.0, -2.0, -2.0, 2.5),
        (2.4, 2.0, 2.0, 3.0, 2.0, 2.4),
    ] {
        assert_eq!(t.value("logic.round", None, vec![n(x)]), n(r), "round {x}");
        assert_eq!(t.value("logic.floor", None, vec![n(x)]), n(fl), "floor {x}");
        assert_eq!(
            t.value("logic.ceiling", None, vec![n(x)]),
            n(c),
            "ceiling {x}"
        );
        assert_eq!(t.value("logic.trunc", None, vec![n(x)]), n(tr), "trunc {x}");
        assert_eq!(t.value("logic.abs", None, vec![n(x)]), n(ab), "abs {x}");
    }
    // Lists.
    let nums = Val::List(vec![n(1.0), n(2.0), n(4.5)]);
    assert_eq!(t.value("logic.count", None, vec![nums.clone()]), n(3.0));
    assert_eq!(t.value("logic.sum", None, vec![nums.clone()]), n(7.5));
    assert_eq!(
        t.value("logic.take", Some(json!({"n": 2})), vec![nums.clone()]),
        Val::List(vec![n(1.0), n(2.0)])
    );
    assert_eq!(
        t.value("logic.take", Some(json!({"n": 9})), vec![nums]),
        Val::List(vec![n(1.0), n(2.0), n(4.5)])
    );
    for (bs, any, all) in [
        (vec![], false, true),
        (vec![false], false, false),
        (vec![true, false], true, false),
        (vec![true, true], true, true),
    ] {
        let l = Val::List(bs.iter().map(|b| f(*b)).collect());
        assert_eq!(t.value("logic.any", None, vec![l.clone()]), f(any));
        assert_eq!(t.value("logic.all", None, vec![l]), f(all));
    }
    let ships = Val::List(vec![
        rec(json!({"address": "u:t/ship:c", "name": "Cutter", "guns": 2, "kind": "patrol"})),
        rec(json!({"address": "u:t/ship:a", "name": "Avid", "guns": 6, "kind": "pirate"})),
        rec(json!({"address": "u:t/ship:b", "name": "Brig", "guns": 2, "kind": "pirate"})),
        rec(json!({"address": "u:t/ship:d", "name": "Drifter"})),
    ]);
    let pirates = Some(json!({"where": {"if": "is", "field": "kind", "value": "pirate"}}));
    let names = |v: &Val| -> Vec<String> {
        v.list()
            .unwrap()
            .iter()
            .map(|r| match r {
                Val::Record(m) => m["name"].text().unwrap().to_string(),
                _ => panic!(),
            })
            .collect()
    };
    assert_eq!(
        names(&t.value("logic.filter", pirates.clone(), vec![ships.clone()])),
        ["Avid", "Brig"]
    );
    let o = t.run("logic.first_match", pirates, vec![ships.clone()]);
    assert_eq!(names(&Val::List(vec![o.values[0].clone()])), ["Avid"]);
    assert_eq!(o.values[1..], [f(true), n(1.0)]);
    let none = Some(json!({"where": {"if": "range", "field": "guns", "min": 99}}));
    assert_eq!(
        t.run("logic.first_match", none, vec![ships.clone()]).values[1..],
        [f(false), n(-1.0)]
    );
    // Sort: ties broken by address; records without the key last.
    let by_guns = Some(json!({"key": "guns"}));
    assert_eq!(
        names(&t.value("logic.sort", by_guns, vec![ships.clone()])),
        ["Brig", "Cutter", "Avid", "Drifter"]
    );
    let desc = Some(json!({"key": "guns", "descending": true}));
    assert_eq!(
        names(&t.value("logic.sort", desc, vec![ships.clone()])),
        ["Avid", "Brig", "Cutter", "Drifter"]
    );
    assert_eq!(
        names(&t.value("logic.sort", Some(json!({"key": "name"})), vec![ships])),
        ["Avid", "Brig", "Cutter", "Drifter"]
    );
    let long = Val::List(vec![n(0.0); crate::blocks::MAX_LIST + 1]);
    assert!(t
        .fails("logic.sum", None, vec![long])
        .contains("longer than"));
    // World reads, with their evidence.
    let o = t.in_world("world.read", json!({"field": "mass"}), vec![]);
    assert_eq!(
        (o.values, o.reads),
        (vec![n(2.0), f(true)], vec![("mass".into(), C::Float(2.0))])
    );
    let o = t.in_world("world.read", json!({"field": "parent.mass"}), vec![]);
    assert_eq!(o.values, vec![n(10.0), f(true)]);
    let o = t.in_world("world.read", json!({"field": "moons"}), vec![]);
    assert_eq!(o.values, vec![n(3.0), f(true)]);
    let o = t.in_world(
        "world.read",
        json!({"field": "name", "shape": "text"}),
        vec![],
    );
    assert_eq!(o.values, vec![Val::Text("Tarn".into()), f(true)]);
    // A fact of another shape is not present, but was read (K8 notes it too).
    let o = t.in_world("world.read", json!({"field": "name"}), vec![]);
    assert_eq!((o.values, o.reads.len()), (vec![n(0.0), f(false)], 1));
    let o = t.in_world("world.read", json!({"field": "nothing"}), vec![]);
    assert_eq!((o.values, o.reads.len()), (vec![n(0.0), f(false)], 0));
    let o = t.in_world(
        "world.read",
        json!({"field": "mass", "address": "u:t/sys:0/body:2"}),
        vec![],
    );
    assert_eq!(o.values, vec![n(0.0), f(false)]);
    let o = t.in_world("world.read_kept", json!({"kind": "trade.credits"}), vec![]);
    assert_eq!(o.values, vec![n(1500.0), f(true)]);
    assert_eq!(
        o.reads,
        vec![("kept:trade.credits".into(), C::Float(1500.0))]
    );
    assert_eq!(
        t.in_world("world.has_fact", json!({"field": "port"}), vec![])
            .values,
        vec![f(true)]
    );
    assert_eq!(
        t.in_world("world.has_fact", json!({"field": "rings"}), vec![])
            .values,
        vec![f(false)]
    );
    let o = t.in_world("world.has_tag", json!({"tag": "garden"}), vec![]);
    assert_eq!(
        (o.values, o.reads),
        (vec![f(true)], vec![("tag:garden".into(), C::Bool(true))])
    );
    let o = t.in_world("world.has_tag", json!({"tag": "ruins"}), vec![]);
    assert_eq!((o.values, o.reads.len()), (vec![f(false)], 0));
    let o = t.in_world(
        "world.children",
        json!({}),
        vec![Val::Ref("u:t/sys:0/body:1".into())],
    );
    assert_eq!(
        o.values,
        vec![Val::List(vec![
            Val::Ref("u:t/sys:0/body:1/body:0".into()),
            Val::Ref("u:t/sys:0/body:1/body:1".into())
        ])]
    );
    assert_eq!(
        t.in_world("world.now", json!({}), vec![]).values,
        vec![n(1000.0)]
    );
    assert_eq!(t.value("world.staying", None, vec![]), f(false));
    assert!(t.staying("world.staying", None, vec![]));
    assert!(t
        .try_run(
            "world.read",
            Some(json!({"field": "mass"})),
            vec![],
            &Env::empty()
        )
        .unwrap_err()
        .0
        .contains("has none here"));
    assert_eq!(
        t.value("logic.const", Some(json!({"value": 4.0})), vec![]),
        n(4.0)
    );
    assert_eq!(
        t.value(
            "logic.const",
            Some(json!({"shape": "list<choice>", "value": ["a"]})),
            vec![]
        ),
        Val::List(vec![ch("a")])
    );
    // Configs are checked: unknown keys, missing ones, bad values.
    assert!(t
        .fails(
            "logic.and",
            Some(json!({"inputs": 40})),
            vec![f(true), f(true)]
        )
        .contains("inputs must be"));
    assert!(t
        .fails(
            "logic.less",
            Some(json!({"tolerance": 1})),
            vec![n(1.0), n(1.0)]
        )
        .contains("takes no `tolerance`"));
    assert!(t
        .fails("world.read", None, vec![])
        .contains("needs `field`"));
    assert!(t
        .fails(
            "logic.in_range",
            Some(json!({"min": 3, "max": 1})),
            vec![n(1.0)]
        )
        .contains("above max"));
    assert!(t
        .fails(
            "logic.const",
            Some(json!({"shape": "flag", "value": 3})),
            vec![]
        )
        .contains("expected a flag"));
    // Every logic atom has rows here (the dice have their own test).
    let all: BTreeSet<String> = crate::blocks::all()
        .iter()
        .map(|a| a.decl().id.clone())
        .collect();
    let seen = t.seen.borrow().clone();
    let missed: Vec<&String> = all.difference(&seen).collect();
    assert!(missed.is_empty(), "no truth table for {missed:?}");
}

fn cx_parts() -> (
    AtomRegistry,
    BTreeMap<String, sse_core::QuantityKind>,
    Units,
) {
    (crate::registry(), sse_flow::toy::kinds(), Units::standard())
}
fn node(id: &str, atom: &str, config: Option<J>) -> NodeDoc {
    NodeDoc {
        id: id.into(),
        atom: Some(atom.into()),
        solve: None,
        params: BTreeMap::new(),
        config,
    }
}
fn wire(a: &str, b: &str) -> WireDoc {
    WireDoc {
        from: a.into(),
        to: b.into(),
    }
}
fn doc(id: &str, inputs: Vec<FlowInputDoc>, nodes: Vec<NodeDoc>, wires: Vec<WireDoc>) -> FlowDoc {
    FlowDoc {
        format: "sse.flow/1".into(),
        id: id.into(),
        version: "1.0.0".into(),
        hook: "test".into(),
        inputs,
        nodes,
        wires,
        publishes: vec![],
        tests: vec![],
    }
}
fn errors(d: FlowDoc, reg: &AtomRegistry) -> String {
    let (_, k, u) = cx_parts();
    let cx = Context {
        atoms: reg,
        kinds: &k,
        units: &u,
    };
    Flow::compile(d, &cx)
        .unwrap_err()
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn shapes_and_dimensions_are_checked_at_edit_time() {
    let (reg, _, _) = cx_parts();
    let input = |name: &str, kind: &str| FlowInputDoc {
        name: name.into(),
        kind: kind.into(),
        shape: None,
    };
    // A flag into a number port.
    let e = errors(
        doc(
            "t",
            vec![],
            vec![
                node("k", "world.staying", None),
                node("a", "logic.abs", None),
            ],
            vec![wire("k.staying", "a.x")],
        ),
        &reg,
    );
    assert!(e.contains("shape mismatch: flag into number"), "{e}");
    // A mass plus a length, through any-dimension ports.
    let e = errors(
        doc(
            "t",
            vec![input("m", "star.mass"), input("d", "orbit.semi_major_axis")],
            vec![node("s", "logic.add", None)],
            vec![wire("in.m", "s.a"), wire("in.d", "s.b")],
        ),
        &reg,
    );
    assert!(e.contains("dimension mismatch: inputs carry"), "{e}");
    // A bad config is an edit-time error with the node named.
    let e = errors(
        doc(
            "t",
            vec![],
            vec![node("r", "world.read", Some(json!({"feild": "x"})))],
            vec![],
        ),
        &reg,
    );
    assert!(e.starts_with("r: world.read takes no `feild`"), "{e}");
    // Adding config to no node leaves the document's bytes, so its hash, as before.
    let plain = serde_json::to_string(&node("a", "logic.abs", None)).unwrap();
    assert!(!plain.contains("config"));
}

fn subject_env<'a>(w: &'a SubjectWorld<'a>, s: &Subject, staying: bool) -> Env<'a> {
    Env {
        world: Some(w),
        subject: s.address.clone(),
        now_s: s.now_s,
        staying,
        ..Env::empty()
    }
}

/// Evaluate a condition as blocks: (holds, evidence).
fn as_blocks(p: &Predicate, s: &Subject, staying: bool) -> (bool, Vec<(String, C)>) {
    let (reg, k, u) = cx_parts();
    let cx = Context {
        atoms: &reg,
        kinds: &k,
        units: &u,
    };
    let c = to_blocks(p, "test.condition").unwrap();
    let flow = Flow::compile(c.doc, &cx).unwrap_or_else(|e| panic!("{e:?}"));
    let w = SubjectWorld(s);
    let ev = flow
        .eval_with(
            &cx,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &subject_env(&w, s, staying),
        )
        .unwrap();
    (
        ev.value_of(&c.output).unwrap().flag().unwrap(),
        ev.evidence(&c.output).unwrap(),
    )
}

fn by_k8(p: &Predicate, s: &Subject, staying: bool) -> (bool, Vec<(String, C)>) {
    let e = p.eval(s, staying);
    (
        e.holds,
        e.read.into_iter().map(|r| (r.field, r.value)).collect(),
    )
}

/// Subjects for the rules pack: every mix of present, absent, inside, on
/// the edge and outside for each fact the rules read.
fn grid() -> Vec<Subject> {
    let temps = [
        None,
        Some(C::Float(250.0)),
        Some(C::Float(270.0)),
        Some(C::Int(288)),
        Some(C::Float(312.0)),
    ];
    let press = [
        None,
        Some(C::Float(0.4)),
        Some(C::Float(1.0)),
        Some(C::Float(5.1)),
    ];
    let ocean = [
        None,
        Some(C::Bool(true)),
        Some(C::Bool(false)),
        Some(C::Text("yes".into())),
    ];
    let o2 = [None, Some(C::Float(0.1)), Some(C::Float(0.21))];
    let orbit = [
        None,
        Some((C::Float(1.0e8), C::Float(0.9e8))),
        Some((C::Float(1.25e8), C::Float(1.0e8))),
    ];
    let mut out = Vec::new();
    let mut i = 0u64;
    for t in &temps {
        for p in &press {
            for oc in &ocean {
                for o in &o2 {
                    for ob in &orbit {
                        let mut f = BTreeMap::new();
                        let mut put = |k: &str, v: &Option<C>| {
                            if let Some(v) = v {
                                f.insert(k.to_string(), v.clone());
                            }
                        };
                        put("surface_temp_k", t);
                        put("pressure_bar", p);
                        put("has_ocean", oc);
                        put("o2_bar", o);
                        let mut s = Subject::new(BTreeMap::new());
                        if let Some((a, r)) = ob {
                            f.insert("orbit_m".into(), a.clone());
                            s.parent.insert("roche_m".into(), r.clone());
                        }
                        if i.is_multiple_of(3) {
                            f.insert("port".into(), C::Bool(i.is_multiple_of(2)));
                        }
                        if i % 4 == 1 {
                            s.tags.insert("habitable".into());
                        }
                        s.facts = f;
                        s.address = Some(format!("u:grid/sys:0/body:{i}").parse().unwrap());
                        out.push(s);
                        i += 1;
                    }
                }
            }
        }
    }
    out
}

fn solar() -> Vec<Subject> {
    let p = format!("{ROOT}tests/fixtures/classify/solar-system.json");
    let v: J = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let mut s = Subject::new(serde_json::from_value(b["facts"].clone()).unwrap());
            s.address = Some(format!("u:sol/sys:0/body:{i}").parse().unwrap());
            s.now_s = 1_000_000;
            if i % 2 == 0 {
                s.tags.insert("garden".into());
            }
            s
        })
        .collect()
}

/// Acceptance: a K8 rule and the same condition as blocks give identical
/// results and evidence, entering and staying.
#[test]
fn k8_rules_and_blocks_agree_on_results_and_evidence() {
    let content = sse_sift::pack::load(&[sse_sift::scenario::pack()]).unwrap();
    let mut preds: Vec<(String, Predicate)> = Vec::new();
    for r in &content.rules.rules {
        preds.push((r.id.clone(), r.when.clone()));
        if let Some(s) = &r.stay {
            preds.push((format!("{} (stay)", r.id), s.clone()));
        }
    }
    let hand: Vec<J> = vec![
        json!({"if": "all", "of": []}),
        json!({"if": "any", "of": []}),
        json!({"if": "not", "p": {"if": "has", "field": "pressure_bar"}}),
        json!({"if": "any", "of": [{"if": "tag", "tag": "habitable"}, {"if": "is", "field": "port", "value": true}, {"if": "range", "field": "o2_bar", "max": 0.15}]}),
        json!({"if": "near", "a": "surface_temp_k", "b": "o2_bar", "tolerance": 1000.0}),
        json!({"if": "compare", "left": "o2_bar", "op": "ge", "right": "pressure_bar", "factor": 0.2, "margin": 0.1}),
        json!({"if": "compare", "left": "surface_temp_k", "op": "le", "right": "parent.roche_m", "factor": 1e-5}),
        json!({"if": "compare", "left": "pressure_bar", "op": "gt", "right": "o2_bar", "factor": 5.0, "margin": 0.3}),
        json!({"if": "is", "field": "surface_temp_k", "value": 288}),
        json!({"if": "is", "field": "has_ocean", "value": "yes"}),
        json!({"if": "all", "of": (0..20).map(|i| json!({"if": "range", "field": "surface_temp_k", "min": 200 + i, "max": 400 - i})).collect::<Vec<_>>()}),
        json!({"if": "any", "of": (0..20).map(|i| json!({"if": "range", "field": "surface_temp_k", "min": 300 + 2 * i, "max": 301 + 2 * i})).collect::<Vec<_>>()}),
    ];
    for (i, j) in hand.into_iter().enumerate() {
        preds.push((format!("hand {i}"), serde_json::from_value(j).unwrap()));
    }
    let (mut checked, mut trues, mut with_evidence) = (0, 0, 0);
    for (id, p) in &preds {
        for s in grid() {
            for staying in [false, true] {
                let got = as_blocks(p, &s, staying);
                assert_eq!(
                    got,
                    by_k8(p, &s, staying),
                    "{id} on {:?} staying {staying}",
                    s.address
                );
                checked += 1;
                trues += got.0 as usize;
                with_evidence += !got.1.is_empty() as usize;
            }
        }
    }
    // The fixture worlds, with every predicate kind on real facts.
    let solar_preds: Vec<J> = vec![
        json!({"if": "range", "field": "planet.surface_temperature", "min": 273, "max": 373, "margin": 20}),
        json!({"if": "is", "field": "atm.main", "value": "N2"}),
        json!({"if": "is", "field": "hasBiosphere", "value": 1}),
        json!({"if": "is", "field": "name", "value": true}),
        json!({"if": "compare", "left": "orbit.period", "op": "lt", "right": "orbit.semi_major_axis", "factor": 1e-3, "margin": 0.5}),
        json!({"if": "near", "a": "body.surface_gravity", "b": "body.escape_velocity", "tolerance": 0.999}),
        json!({"if": "within", "field": "orbit.period", "from_s": 0, "to_s": 40_000_000}),
        json!({"if": "within", "field": "orbit.period", "from_s": -2_000_000, "to_s": 9_000_000}),
        json!({"if": "all", "of": [{"if": "tag", "tag": "garden"}, {"if": "has", "field": "hydrosphere.coverage"}, {"if": "not", "p": {"if": "range", "field": "body.mass", "min": 1e26}}]}),
    ];
    for j in solar_preds {
        let p: Predicate = serde_json::from_value(j.clone()).unwrap();
        for s in solar() {
            for staying in [false, true] {
                assert_eq!(
                    as_blocks(&p, &s, staying),
                    by_k8(&p, &s, staying),
                    "{j} on {:?}",
                    s.facts.get("name")
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 10_000, "{checked}");
    // Not vacuous: both answers, and evidence, are common.
    assert!(
        trues > checked / 10 && trues < checked * 9 / 10,
        "{trues} of {checked}"
    );
    assert!(with_evidence > checked / 2, "{with_evidence} of {checked}");
}

fn roll_doc(id: &str, nodes: &[&str]) -> FlowDoc {
    let mut ns: Vec<NodeDoc> = nodes
        .iter()
        .map(|n| NodeDoc {
            params: [
                ("dice".to_string(), sse_flow::flow::ParamValue::Si(10.0)),
                ("sides".to_string(), sse_flow::flow::ParamValue::Si(1000.0)),
            ]
            .into(),
            ..node(n, "chance.roll", None)
        })
        .collect();
    ns.push(node(
        "pick",
        "chance.draw",
        Some(json!({"entries": [{"value": "calm", "weight": 1}, {"value": "storm", "weight": 3}]})),
    ));
    doc(id, vec![], ns, vec![])
}

fn rolls(
    d: FlowDoc,
    secret: Option<Seed>,
    subject: &str,
    counter: u64,
) -> Result<BTreeMap<String, Val>, String> {
    let (reg, k, u) = cx_parts();
    let cx = Context {
        atoms: &reg,
        kinds: &k,
        units: &u,
    };
    let ids: Vec<String> = d.nodes.iter().map(|n| n.id.clone()).collect();
    let flow = Flow::compile(d, &cx).unwrap();
    let env = Env {
        subject: Some(addr(subject)),
        table_secret: secret,
        counter,
        ..Env::empty()
    };
    let ev = flow
        .eval_with(&cx, &BTreeMap::new(), &BTreeMap::new(), &env)
        .map_err(|e| e.to_string())?;
    Ok(ids
        .iter()
        .map(|i| {
            let port = if i == "pick" { "value" } else { "dice" };
            (
                i.clone(),
                ev.value_of(&format!("{i}.{port}")).unwrap().clone(),
            )
        })
        .collect())
}

/// Acceptance: rolls are keyed on the table secret, independent of
/// evaluation order; a device with only the universe seed cannot reproduce
/// them; the commitment verifies after reveal.
#[test]
fn dice_are_keyed_on_the_table_secret() {
    let secret = dice::table_secret(b"entropy from the GM machine");
    let universe = Seed::from_text("kestrel");
    let ship = "u:kestrel/sys:0/ship:@corsair";
    // No table secret: refused, with the reason.
    let e = rolls(roll_doc("sse.pirates", &["a"]), None, ship, 0).unwrap_err();
    assert!(e.contains("GM's machine only"), "{e}");
    let base = rolls(
        roll_doc("sse.pirates", &["a", "b", "c"]),
        Some(secret),
        ship,
        7,
    )
    .unwrap();
    // Same campaign, same turn: same dice, whatever the order of the nodes.
    assert_eq!(
        rolls(
            roll_doc("sse.pirates", &["c", "a", "b"]),
            Some(secret),
            ship,
            7
        )
        .unwrap(),
        base
    );
    assert_eq!(
        rolls(roll_doc("sse.pirates", &["b"]), Some(secret), ship, 7).unwrap()["b"],
        base["b"]
    );
    // Each node, turn and subject has its own dice.
    assert_ne!(base["a"], base["b"]);
    assert_ne!(
        rolls(roll_doc("sse.pirates", &["a"]), Some(secret), ship, 8).unwrap()["a"],
        base["a"]
    );
    assert_ne!(
        rolls(
            roll_doc("sse.pirates", &["a"]),
            Some(secret),
            "u:kestrel/sys:0/ship:@patrol",
            7
        )
        .unwrap()["a"],
        base["a"]
    );
    // The universe seed in the secret's place gives other dice.
    assert_ne!(
        rolls(
            roll_doc("sse.pirates", &["a", "b", "c"]),
            Some(universe),
            ship,
            7
        )
        .unwrap(),
        base
    );
    // The commitment seats hold checks out after the reveal, and only then.
    let sent = dice::commitment(&secret);
    assert!(dice::verify(&secret, &sent));
    assert!(!dice::verify(&universe, &sent));
    assert_ne!(sent, secret.0, "the commitment is not the secret");
    // Dice fall in range and every face turns up; draws follow the weights.
    let mut faces = BTreeSet::new();
    let mut storms = 0;
    for turn in 0..400 {
        let d = roll_doc("sse.weather", &[]);
        let mut d6 = d.clone();
        d6.nodes.push(node("d", "chance.roll", None));
        let (reg, k, u) = cx_parts();
        let cx = Context {
            atoms: &reg,
            kinds: &k,
            units: &u,
        };
        let flow = Flow::compile(d6, &cx).unwrap();
        let env = Env {
            subject: Some(addr(ship)),
            table_secret: Some(secret),
            counter: turn,
            ..Env::empty()
        };
        let ev = flow
            .eval_with(&cx, &BTreeMap::new(), &BTreeMap::new(), &env)
            .unwrap();
        let x = ev.value_of("d.total").unwrap().num().unwrap();
        assert!((1.0..=6.0).contains(&x));
        faces.insert(x as u8);
        if ev.value_of("pick.value").unwrap() == &ch("storm") {
            storms += 1;
        }
    }
    assert_eq!(faces.len(), 6);
    assert!(
        (260..=340).contains(&storms),
        "{storms} storms in 400 (expected about 300)"
    );
    assert_eq!(
        dice::stream_for("Sse.Pirate-Patrol", "Roll 1").as_str(),
        "sse.pirate_patrol.roll_1.v1"
    );
}

fn engine_reg(atlas: &sse_sift::classify::Atlas) -> AtomRegistry {
    let mut r = crate::registry();
    r.insert(Box::new(Classify::new(atlas.clone())));
    r.insert(Box::new(Place::new(PlaceEngine {
        atlas: atlas.clone(),
        model: sse_place::scenario::model(),
        life: Box::new(sse_life::NoLifeModel::default()),
        life_rules: sse_life::scenario::base_rules(),
    })));
    r
}

/// Acceptance: `classify` wired into a flow gives the same label and
/// explanation as the class atlas for the fixture worlds, and a rule can
/// ask "is this world a garden?" with the same block the charts use.
#[test]
fn classify_in_a_flow_is_the_class_atlas() {
    let atlas = sse_sift::scenario::base_atlas();
    let reg = engine_reg(&atlas);
    let (_, k, u) = cx_parts();
    let cx = Context {
        atoms: &reg,
        kinds: &k,
        units: &u,
    };
    let mut bodies = solar();
    let p = format!("{ROOT}tests/fixtures/classify/exoplanets.json");
    let v: J = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    bodies.extend(
        v["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| Subject::new(serde_json::from_value(b["facts"].clone()).unwrap())),
    );
    let mut n_checked = 0;
    for scheme in ["physical", "surface", "air", "habitability"] {
        let d = doc(
            "test.classify",
            vec![],
            vec![
                node("c", "engine.classify", Some(json!({"scheme": scheme}))),
                node(
                    "k",
                    "logic.const",
                    Some(json!({"shape": "choice", "value": "habitable/shirt-sleeve"})),
                ),
                node("e", "logic.equal", Some(json!({"shape": "choice"}))),
            ],
            vec![wire("c.class", "e.a"), wire("k.value", "e.b")],
        );
        let flow = Flow::compile(d, &cx).unwrap_or_else(|e| panic!("{e:?}"));
        for s in &bodies {
            let w = SubjectWorld(s);
            let ev = flow
                .eval_with(
                    &cx,
                    &BTreeMap::new(),
                    &BTreeMap::new(),
                    &subject_env(&w, s, false),
                )
                .unwrap();
            let want = atlas.label(scheme, s, None);
            assert_eq!(
                ev.value_of("c.class").unwrap(),
                &ch(want.base.as_deref().unwrap_or(""))
            );
            assert_eq!(
                ev.value_of("c.classified").unwrap(),
                &f(want.base.is_some())
            );
            assert_eq!(
                ev.value_of("c.modifiers").unwrap(),
                &Val::List(want.modifiers.iter().map(|m| ch(m)).collect())
            );
            assert_eq!(ev.value_of("c.fit").unwrap(), &n(want.fit));
            assert_eq!(
                ev.note_of("c").unwrap(),
                &serde_json::to_value(&want).unwrap()
            );
            assert_eq!(
                ev.value_of("e.holds").unwrap(),
                &f(want.base.as_deref() == Some("habitable/shirt-sleeve"))
            );
            n_checked += 1;
        }
    }
    assert!(n_checked >= 40, "{n_checked}");
    // Earth is the garden the rule asks about.
    let earth = solar()
        .into_iter()
        .find(|s| s.facts.get("name") == Some(&C::Text("Earth".into())))
        .unwrap();
    assert_eq!(
        atlas.label("habitability", &earth, None).base.as_deref(),
        Some("habitable/shirt-sleeve")
    );
    // Stickiness through the `held` input, as the atlas does it.
    let d = doc(
        "test.sticky",
        vec![],
        vec![
            node(
                "h",
                "logic.const",
                Some(json!({"shape": "choice", "value": "surface/temperate-ocean"})),
            ),
            node(
                "c",
                "engine.classify",
                Some(json!({"scheme": "surface", "sticky": true})),
            ),
        ],
        vec![wire("h.value", "c.held")],
    );
    let flow = Flow::compile(d, &cx).unwrap();
    for s in &bodies {
        let w = SubjectWorld(s);
        let ev = flow
            .eval_with(
                &cx,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &subject_env(&w, s, false),
            )
            .unwrap();
        let want = atlas.label("surface", s, Some("surface/temperate-ocean"));
        assert_eq!(
            ev.note_of("c").unwrap(),
            &serde_json::to_value(&want).unwrap()
        );
    }
    // An unknown scheme is an edit-time error.
    let e = errors(
        doc(
            "t",
            vec![],
            vec![node(
                "c",
                "engine.classify",
                Some(json!({"scheme": "vibes"})),
            )],
            vec![],
        ),
        &reg,
    );
    assert!(e.contains("no scheme \"vibes\""), "{e}");
    assert_eq!(
        reg.get("engine.classify").unwrap().decl().class,
        Some(sse_flow::BlockClass::Engine)
    );
}

/// `place` is the class atlas's second port set: wired into a flow it gives
/// the C3 placement exactly, keyed on the universe seed.
#[test]
fn place_in_a_flow_is_reverse_placement() {
    let atlas = sse_sift::scenario::base_atlas();
    let reg = engine_reg(&atlas);
    let (_, k, u) = cx_parts();
    let cx = Context {
        atoms: &reg,
        kinds: &k,
        units: &u,
    };
    let model = sse_place::scenario::model();
    let life = sse_life::NoLifeModel::default();
    let rules = sse_life::scenario::base_rules();
    let placer = sse_place::Placer {
        atlas: &atlas,
        model: &model,
        life: &life,
        life_rules: &rules,
    };
    for (sys, scheme, class, scope) in sse_place::scenario::CASES {
        let mut cfg = json!({"scheme": scheme, "class": class});
        if let Some((a, b)) = scope {
            cfg["scope_au"] = json!([a, b]);
        }
        let d = doc(
            "test.place",
            vec![FlowInputDoc {
                name: "system".into(),
                kind: "system".into(),
                shape: Some(Shape::Record),
            }],
            vec![node("p", "engine.place", Some(cfg))],
            vec![wire("in.system", "p.system")],
        );
        let flow = Flow::compile(d, &cx).unwrap_or_else(|e| panic!("{e:?}"));
        let system = sse_place::scenario::system(sys);
        let typed: BTreeMap<String, Val> = [(
            "system".to_string(),
            rec(serde_json::to_value(&system).unwrap()),
        )]
        .into();
        let env = Env {
            subject: Some(sse_place::scenario::address(sys)),
            seed: Some(sse_place::scenario::seed()),
            ..Env::empty()
        };
        let ev = flow.eval_with(&cx, &BTreeMap::new(), &typed, &env).unwrap();
        let want = sse_place::scenario::run(&placer, sys, scheme, class, scope, false);
        assert_eq!(
            ev.note_of("p").unwrap(),
            &serde_json::to_value(&want).unwrap(),
            "{sys} {class}"
        );
        assert_eq!(ev.value_of("p.recognised").unwrap(), &f(want.recognised));
        assert_eq!(
            ev.value_of("p.label").unwrap(),
            &ch(want.label.as_deref().unwrap_or(""))
        );
        assert_eq!(ev.value_of("p.fit").unwrap(), &n(want.fit));
    }
    let e = errors(
        doc(
            "t",
            vec![],
            vec![node(
                "p",
                "engine.place",
                Some(json!({"scheme": "surface", "class": "unicorn"})),
            )],
            vec![],
        ),
        &reg,
    );
    assert!(e.contains("no class \"unicorn\""), "{e}");
}
