//! J3 acceptance (record design section 10): a boxed flow equals its
//! hand-inlined twin value for value and trace for trace; a self-containing
//! box is rejected with a readable report; the depth limit is reported; a
//! box's face is an atom's (validity, cost, references, params) and its own
//! golden cases run. Replacing a box through a pack is tested in sse-pack.

use crate::boxes::{box_decl, inline, run_box_tests, BoxDoc, MAX_DEPTH};
use crate::flow::*;
use crate::{toy, Flow};
use sse_core::{Quantity, Units};
use std::collections::{BTreeMap, BTreeSet};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
const TOY: &str = include_str!("../../../tests/fixtures/flows/toy-ten.json");

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{ROOT}{p}")).unwrap()
}
fn boxes() -> BTreeMap<String, BoxDoc> {
    let v: Vec<BoxDoc> =
        serde_json::from_str(&read("tests/fixtures/boxes/toy-boxes.json")).unwrap();
    v.into_iter().map(|b| (b.id.clone(), b)).collect()
}
fn boxed() -> FlowDoc {
    FlowDoc::from_json(&read("tests/fixtures/boxes/toy-boxed.json")).unwrap()
}

struct Fx {
    atoms: crate::AtomRegistry,
    kinds: BTreeMap<String, sse_core::QuantityKind>,
    units: Units,
}
impl Fx {
    fn new() -> Self {
        Fx {
            atoms: toy::registry(),
            kinds: toy::kinds(),
            units: Units::standard(),
        }
    }
    fn cx(&self) -> Context<'_> {
        Context {
            atoms: &self.atoms,
            kinds: &self.kinds,
            units: &self.units,
        }
    }
}

/// The toy flow wired by hand with the ids inlining gives its boxed nodes.
fn twin() -> FlowDoc {
    let mut d = FlowDoc::from_json(TOY).unwrap();
    d.id = "toy.star-and-planet-boxed".into();
    let rename: BTreeMap<&str, &str> = [
        ("lum", "star/lum"),
        ("rad", "star/rad"),
        ("teff", "star/photo/teff"),
        ("grav", "bulk/grav"),
        ("esc", "bulk/esc"),
        ("dens", "bulk/dens"),
    ]
    .into();
    let r = |s: &str| -> String {
        let (a, b) = s.split_once('.').unwrap();
        format!("{}.{b}", rename.get(a).copied().unwrap_or(a))
    };
    for n in &mut d.nodes {
        if let Some(x) = rename.get(n.id.as_str()) {
            n.id = x.to_string();
        }
    }
    for w in &mut d.wires {
        w.from = r(&w.from);
        w.to = r(&w.to);
    }
    for p in &mut d.publishes {
        p.from = r(&p.from);
    }
    d
}

fn sun_earth(u: &Units) -> BTreeMap<String, InputValue> {
    let q = |v: f64, unit: &str| InputValue {
        q: u.q(v, unit).unwrap(),
        source: SourceLabel::Authored,
    };
    [
        ("star_mass", q(1.0, "m_sun")),
        ("distance", q(1.0, "au")),
        (
            "albedo",
            InputValue {
                q: Quantity::si(0.3, sse_core::Dimension::NONE),
                source: SourceLabel::Authored,
            },
        ),
        ("planet_mass", q(1.0, "m_earth")),
        ("planet_radius", q(1.0, "r_earth")),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// Acceptance: the boxed flow and its hand-wired twin are the same flat
/// flow, value for value and trace for trace.
#[test]
fn a_boxed_flow_equals_its_hand_inlined_twin() {
    let fx = Fx::new();
    let cx = fx.cx();
    let b = Flow::compile_with(boxed(), &cx, &boxes()).unwrap_or_else(|e| panic!("{e:?}"));
    let t = Flow::compile(twin(), &cx).unwrap();
    let set = |f: &Flow| -> (BTreeSet<String>, BTreeSet<(String, String)>) {
        (
            f.doc
                .nodes
                .iter()
                .map(|n| serde_json::to_string(n).unwrap())
                .collect(),
            f.doc
                .wires
                .iter()
                .map(|w| (w.from.clone(), w.to.clone()))
                .collect(),
        )
    };
    assert_eq!(set(&b), set(&t), "the same nodes, params and wires");
    assert_eq!(b.doc.publishes, t.doc.publishes);
    let mut steps = 0;
    for (mass, dist) in [(1.0, 1.0), (0.5, 0.3), (2.0, 4.0), (25.0, 1.0)] {
        let mut ins = sun_earth(&fx.units);
        ins.get_mut("star_mass").unwrap().q = fx.units.q(mass, "m_sun").unwrap();
        ins.get_mut("distance").unwrap().q = fx.units.q(dist, "au").unwrap();
        let (eb, et) = (b.eval(&cx, &ins).unwrap(), t.eval(&cx, &ins).unwrap());
        for p in &t.doc.publishes {
            assert_eq!(eb.published(&p.kind), et.published(&p.kind), "{}", p.kind);
            assert_eq!(
                eb.validity_of(&p.kind),
                et.validity_of(&p.kind),
                "{}",
                p.kind
            );
            // Traces are equal but for the flow hash, which is taken over
            // the document with its wires in their written order.
            let tr = |e: Option<Explanation>, h: &str| {
                let mut e = e.unwrap();
                e.lenses.engine = e.lenses.engine.replace(h, "HASH");
                e.publisher.flow_hash = sse_core::ContentHash::of_bytes(b"");
                e
            };
            assert_eq!(
                tr(eb.explain(&p.kind), &b.hash.hex()),
                tr(et.explain(&p.kind), &t.hash.hex()),
                "trace of {}",
                p.kind
            );
            // And every step of the trace upstream.
            let mut todo: Vec<String> = eb
                .explain(&p.kind)
                .unwrap()
                .inputs
                .iter()
                .filter_map(|i| i.explanation_ref.clone())
                .collect();
            while let Some(r) = todo.pop() {
                let (x, y) = (eb.explain_port(&r), et.explain_port(&r));
                todo.extend(
                    x.as_ref()
                        .unwrap()
                        .inputs
                        .iter()
                        .filter_map(|i| i.explanation_ref.clone()),
                );
                assert_eq!(tr(x, &b.hash.hex()), tr(y, &t.hash.hex()), "trace step {r}");
                steps += 1;
            }
        }
        assert_eq!(eb.work_units, et.work_units);
    }
    assert!(steps > 0);
    // The flow's own golden case passes boxed.
    assert!(b.run_tests(&cx).is_empty(), "{:?}", b.run_tests(&cx));
    // Explain shows the boxes collapsed, with a way in.
    let uses: Vec<(String, String, usize)> = b
        .boxes
        .iter()
        .map(|u| (u.node.clone(), u.id.clone(), u.depth))
        .collect();
    assert_eq!(
        uses,
        [
            ("star".into(), "toy.star".into(), 1),
            ("star/photo".into(), "toy.photosphere".into(), 2),
            ("bulk".into(), "toy.bulk".into(), 1)
        ]
    );
    // A flow with no boxes inlines to itself: same document, same hash.
    let plain = FlowDoc::from_json(TOY).unwrap();
    let il = inline(&plain, &boxes()).unwrap();
    assert_eq!(il.doc.content_hash(), plain.content_hash());
    assert!(!serde_json::to_string(&plain).unwrap().contains("\"box\""));
    // A promoted parameter set on the box reaches its inner node.
    let mut hot = boxed();
    hot.nodes[0]
        .params
        .insert("exponent".into(), ParamValue::Si(4.0));
    let h = Flow::compile_with(hot, &cx, &boxes()).unwrap();
    let ins = sun_earth(&fx.units);
    let mut ins2 = ins.clone();
    ins2.get_mut("star_mass").unwrap().q = fx.units.q(2.0, "m_sun").unwrap();
    let l = h
        .eval(&cx, &ins2)
        .unwrap()
        .published("star.luminosity")
        .unwrap()
        .value;
    let want = sse_core::quantity::consts::L_SUN * sse_core::math::pow(2.0, 4.0);
    assert!((l / want - 1.0).abs() < 1e-12, "{l} vs {want}");
}

fn mk(id: &str, inner: Option<&str>) -> BoxDoc {
    let node = match inner {
        Some(b) => format!(r#"{{ "id": "x", "box": "{b}" }}"#),
        None => r#"{ "id": "x", "atom": "toy.radius_from_mass" }"#.into(),
    };
    let out = if inner.is_some() { "x.out" } else { "x.radius" };
    BoxDoc::from_json(&format!(
        r#"{{ "format": "sse.box/1", "id": "{id}", "version": "1.0.0",
            "inputs": [{{ "name": "mass", "kind": "star.mass" }}],
            "outputs": [{{ "name": "out", "from": "{out}" }}],
            "nodes": [{node}],
            "wires": [{{ "from": "in.mass", "to": "x.mass" }}] }}"#
    ))
    .unwrap()
}
fn user(b: &str) -> FlowDoc {
    FlowDoc::from_json(&format!(
        r#"{{ "format": "sse.flow/1", "id": "t.user", "version": "1.0.0", "hook": "t",
            "inputs": [{{ "name": "m", "kind": "star.mass" }}],
            "nodes": [{{ "id": "b", "box": "{b}" }}],
            "wires": [{{ "from": "in.m", "to": "b.mass" }}],
            "publishes": [{{ "kind": "star.radius", "from": "b.out" }}] }}"#
    ))
    .unwrap()
}
fn errs(d: FlowDoc, lib: &BTreeMap<String, BoxDoc>) -> String {
    let fx = Fx::new();
    Flow::compile_with(d, &fx.cx(), lib)
        .unwrap_err()
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn self_containing_boxes_and_the_depth_limit_are_reported() {
    let lib: BTreeMap<String, BoxDoc> = [
        mk("t.a", Some("t.b@^1")),
        mk("t.b", Some("t.c@^1")),
        mk("t.c", Some("t.a@^1")),
        mk("t.self", Some("t.self")),
    ]
    .into_iter()
    .map(|b| (b.id.clone(), b))
    .collect();
    let e = errs(user("t.a@^1"), &lib);
    assert!(
        e.contains("box t.a contains itself: t.a > t.b > t.c > t.a"),
        "{e}"
    );
    assert!(e.starts_with("b/x/x/x: "), "{e}");
    assert_eq!(e.lines().count(), 1, "one problem, reported once: {e}");
    let e = errs(user("t.self"), &lib);
    assert!(
        e.contains("box t.self contains itself: t.self > t.self"),
        "{e}"
    );
    // A chain exactly MAX_DEPTH deep is fine; one more is reported.
    let chain = |n: usize| -> BTreeMap<String, BoxDoc> {
        (1..=n)
            .map(|i| {
                let inner = (i < n).then(|| format!("t.d{}", i + 1));
                mk(&format!("t.d{i}"), inner.as_deref())
            })
            .map(|b| (b.id.clone(), b))
            .collect()
    };
    let fx = Fx::new();
    let ok = Flow::compile_with(user("t.d1"), &fx.cx(), &chain(MAX_DEPTH)).unwrap();
    assert_eq!(ok.boxes.iter().map(|u| u.depth).max(), Some(MAX_DEPTH));
    let e = errs(user("t.d1"), &chain(MAX_DEPTH + 1));
    assert!(
        e.contains("boxes nest 9 deep, past the limit of 8: t.d1 > t.d2 > t.d3 > t.d4 > t.d5 > t.d6 > t.d7 > t.d8 > t.d9"),
        "{e}"
    );
    // Other readable errors.
    assert!(errs(user("t.missing@^1"), &lib).contains("no box t.missing in the stack"));
    assert!(errs(user("t.a@^2"), &lib).contains("wants box t.a@^2, but the stack has t.a 1.0.0"));
    let mut d = user("t.d1");
    d.nodes[0].params.insert("nope".into(), ParamValue::Si(1.0));
    assert!(errs(d, &chain(2)).contains("b.nope: box t.d1 has no parameter \"nope\""));
    let mut d = user("t.d1");
    d.wires.clear();
    assert!(errs(d, &chain(2)).contains("b.mass: box input is not wired"));
    let mut d = user("t.d1");
    d.wires[0].to = "b.weight".into();
    assert!(errs(d, &chain(2)).contains("box t.d1 has no input \"weight\""));
    // A box node reaching plain compile is refused, not half-built.
    let fx = Fx::new();
    let e = Flow::compile(user("t.d1"), &fx.cx()).unwrap_err();
    assert!(e[0].message.contains("boxes are inlined first"), "{e:?}");
}

#[test]
fn a_box_looks_like_an_atom_from_outside() {
    let fx = Fx::new();
    let cx = fx.cx();
    let lib = boxes();
    let d = box_decl(&lib["toy.star"], &cx, &lib).unwrap_or_else(|e| panic!("{e:?}"));
    let names = |ps: &[crate::Port]| ps.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
    assert_eq!(names(&d.inputs), ["mass"]);
    assert_eq!(names(&d.outputs), ["luminosity", "radius", "teff", "mass"]);
    assert_eq!(d.inputs[0].dim, sse_core::Dimension::MASS);
    assert_eq!(d.outputs[0].dim, sse_core::Dimension::POWER);
    assert_eq!(d.outputs[2].dim, sse_core::Dimension::TEMPERATURE);
    // Validity: the intersection of the inner ports the input feeds.
    let inner = |id: &str| fx.atoms.get(id).unwrap().decl().inputs[0].valid;
    let (a, b) = (
        inner("toy.luminosity_from_mass").unwrap(),
        inner("toy.radius_from_mass").unwrap(),
    );
    assert_eq!(d.inputs[0].valid, Some((a.0.max(b.0), a.1.min(b.1))));
    // Params promoted with the inner declaration; cost the maximum; references the union.
    assert_eq!(d.params.len(), 1);
    assert_eq!(
        (
            d.params[0].name.as_str(),
            d.params[0].default,
            d.params[0].range
        ),
        ("exponent", 3.5, Some((1.0, 6.0)))
    );
    let inner_refs: BTreeSet<String> = [
        "toy.luminosity_from_mass",
        "toy.radius_from_mass",
        "toy.effective_temperature",
    ]
    .iter()
    .flat_map(|i| {
        fx.atoms
            .get(i)
            .unwrap()
            .decl()
            .references
            .iter()
            .map(|r| r.text.clone())
    })
    .collect();
    let got: BTreeSet<String> = d.references.iter().map(|r| r.text.clone()).collect();
    assert_eq!(got, inner_refs);
    let max = [
        "toy.luminosity_from_mass",
        "toy.radius_from_mass",
        "toy.effective_temperature",
    ]
    .iter()
    .map(|i| fx.atoms.get(i).unwrap().decl().cost_class)
    .max();
    assert_eq!(Some(d.cost_class), max);
    assert_eq!(
        d.explain.curious,
        "A main-sequence star from its mass alone."
    );
    assert_eq!(d.class, None, "physics inside: a physics block");
    // Golden cases: the box's own pass; a wrong one is reported by name.
    assert!(run_box_tests(&lib["toy.star"], &cx, &lib).is_empty());
    let mut bad = lib["toy.star"].clone();
    bad.tests[0].expect.get_mut("teff").unwrap().value = 6000.0;
    let f = run_box_tests(&bad, &cx, &lib);
    assert_eq!(f.len(), 1);
    assert!(f[0].starts_with("toy.star: the Sun: teff = 577"), "{f:?}");
}
