//! ND3: the Equation block and the maths blocks (equation block design 10).

use crate::equation::{expand, Parsed, ID};
use serde_json::{json, Value as J};
use sse_core::cbor::Value as C;
use sse_core::math as m;
use sse_core::sheet as s;
use sse_core::{Dimension, QuantityKind, Units};
use sse_flow::flow::{FlowInputDoc, NodeDoc, WireDoc};
use sse_flow::{Context, Env, Flow, FlowDoc, InputValue, Val, World};
use std::collections::BTreeMap;

fn src() -> J {
    json!({"text": "Test source", "url": "https://example.org/paper", "where": "eq. 1"})
}
fn explain() -> J {
    json!({"curious": "A test.", "science": "A test relation."})
}

/// The design's own example (section 2), with a test address for its source.
fn swan() -> J {
    json!({
        "name": "How far carbon's colours shift when squeezed",
        "formula": "shift = alpha * density",
        "inputs": {
            "density": {"kind": "atmosphere.photosphere_density", "unit": "g/cm3", "valid": [0, 2]},
            "alpha": {"kind": "model.swan_shift_per_density", "unit": "eV cm3/g"}
        },
        "outputs": {"shift": {"kind": "light.band_shift", "unit": "eV"}},
        "source": src(),
        "explain": explain(),
        "examples": [{"density": 0.15, "alpha": 0.2, "shift": 0.03}]
    })
}

/// The equilibrium temperature of design 4.1: every letter a line.
fn t_eq() -> J {
    json!({
        "name": "How warm a bare world would be",
        "formula": "T = (f * S * (1 - A) / sigma) ^ (1/4)",
        "inputs": {
            "S": {"kind": "planet.insolation", "unit": "W m-2", "name": "starlight arriving"},
            "A": {"kind": "planet.bond_albedo", "unit": "1", "name": "share reflected"},
            "f": {"kind": "model.number", "unit": "1", "name": "share of the sphere heated"},
            "sigma": {"kind": "constant.sigma", "unit": "W m-2 K-4", "name": "Stefan-Boltzmann constant"}
        },
        "outputs": {"T": {"kind": "planet.equilibrium_temperature", "unit": "K"}},
        "source": src(),
        "explain": explain(),
        "examples": [{"S": 1361, "A": 0.3, "f": 0.25, "sigma": s::SIGMA_SB, "T": 254.578, "tolerance": 1e-5}]
    })
}

fn planck_cfg() -> J {
    json!({
        "name": "Light a warm body gives at each wavelength",
        "formula": "x = h * c / (lambda * k * T)\nl2 = lambda * lambda\nB = if x > xmax then 0 else 2 * h * c * c / (l2 * l2 * lambda) / expm1(x)",
        "inputs": {
            "lambda": {"kind": "light.wavelength", "unit": "m", "list": true, "name": "wavelength"},
            "T": {"kind": "star.effective_temperature", "unit": "K", "name": "temperature"},
            "h": {"kind": "constant.h", "unit": "J s", "name": "Planck constant"},
            "c": {"kind": "constant.c", "unit": "m s-1", "name": "speed of light"},
            "k": {"kind": "constant.k", "unit": "J K-1", "name": "Boltzmann constant"},
            "xmax": {"kind": "model.number", "unit": "1", "name": "largest exponent kept"}
        },
        "outputs": {"B": {"kind": "light.spectral_radiance", "unit": "W m-3"}},
        "source": {"text": "Planck 1901", "doi": "10.1002/andp.19013090310", "where": "eq. 11"},
        "explain": {"curious": "How bright a warm body glows at each colour.", "science": "The Planck law, per unit wavelength and solid angle."},
        "examples": [{"lambda": [5e-7], "T": 5772, "h": s::H, "c": s::C, "k": s::K_B, "xmax": 700, "B": [2.62385e13], "tolerance": 1e-5}]
    })
}

fn refused(mut cfg: J, key: &str, v: J) -> String {
    if v.is_null() {
        cfg.as_object_mut().unwrap().remove(key);
    } else {
        cfg[key] = v;
    }
    Parsed::new(Some(&cfg)).unwrap_err()
}

#[test]
fn the_designs_example_loads_and_its_worked_example_holds() {
    let p = Parsed::new(Some(&swan())).unwrap();
    let (ins, outs) = p.ports();
    assert_eq!(ins[1].name, "density");
    assert_eq!(ins[1].dim, Dimension::DENSITY);
    // The valid range is converted to SI on the way in.
    assert_eq!(ins[1].valid, Some((0.0, 2000.0)));
    assert_eq!(outs[0].dim, Dimension([2, 1, -2, 0, 0, 0, 0]));
    assert_eq!(p.canonical(), "shift = alpha * density");
    assert_eq!(p.ops, 1);
    // A wrong worked example is refused on load.
    let mut bad = swan();
    bad["examples"][0]["shift"] = json!(0.04);
    let e = Parsed::new(Some(&bad)).unwrap_err();
    assert!(e.contains("worked example 1") && e.contains("0.04"), "{e}");
}

#[test]
fn what_is_refused_on_load_is_refused_in_words() {
    let base = t_eq();
    // A unit mismatch: adding a length to a time.
    let mut c = base.clone();
    c["formula"] = json!("T = (f * S * (1 - A) / sigma) ^ (1/4) + S");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("one dimension"), "{e}");
    // An exponential of something with a dimension.
    c["formula"] = json!("T = exp(S) * (f * S * (1 - A) / sigma) ^ (1/4)");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("exp of") && e.contains("pure number"), "{e}");
    // A power with a dimensioned value that is not fixed.
    c["formula"] = json!("T = (f * S * (1 - A) / sigma) ^ A");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("fixed power"), "{e}");
    // An output whose dimension is not the declared one.
    c["formula"] = json!("T = (f * S * (1 - A) / sigma) ^ (1/2)");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("declared"), "{e}");
    // An unknown name, named: a sheet row written inside the formula.
    c["formula"] = json!("T = (f * S * (1 - A) / sigma_SB) ^ (1/4)");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("`sigma_SB` has no line into this block"), "{e}");
    // A constant's value typed in, and a decimal: give it a source.
    c["formula"] = json!("T = (f * S * (1 - A) / 5.670374419e-8) ^ (1/4)");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("give this letter a source"), "{e}");
    c["formula"] = json!("T = (f * S * (1 - A) / sigma) ^ 0.25");
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(
        e.contains("give this letter a source") && e.contains("column"),
        "{e}"
    );
    c["formula"] = json!("T = (f * S * (1 - A) / sigma) ^ (1/4) * 700 / 700");
    assert!(Parsed::new(Some(&c)).unwrap_err().contains("700"));
    // Small whole numbers and their ratios are the mathematics.
    c["formula"] = json!("T = (f * S * (32 - 31 * 1 - A) / sigma) ^ (2/8)");
    c["examples"][0]["tolerance"] = json!(1e-5);
    Parsed::new(Some(&c)).unwrap();
    // More than 64 operations.
    let long = (0..70).map(|_| "A").collect::<Vec<_>>().join(" * ");
    c["formula"] = json!(format!("T = (f * S * (1 - {long}) / sigma) ^ (1/4)"));
    let e = Parsed::new(Some(&c)).unwrap_err();
    assert!(e.contains("more than the 64"), "{e}");
    // A letter that is declared but never used.
    let mut c = base.clone();
    c["inputs"]["g"] = json!({"kind": "model.number", "unit": "1"});
    c["examples"][0]["g"] = json!(1);
    assert!(Parsed::new(Some(&c)).unwrap_err().contains("g is not used"));
    // Names a keen amateur reads, a source with an address and a place.
    assert!(refused(base.clone(), "name", json!("T_eq")).contains("in words"));
    assert!(refused(base.clone(), "source", J::Null).contains("carries its source"));
    assert!(refused(
        base.clone(),
        "source",
        json!({"text": "x", "where": "eq 1"})
    )
    .contains("DOI or an address"));
    assert!(refused(
        base.clone(),
        "source",
        json!({"text": "x", "doi": "10.1/x"})
    )
    .contains("where the relation is"));
    assert!(refused(base.clone(), "examples", json!([])).contains("worked example"));
    // A bookkeeping Equation needs no source.
    let mut c = refused_ok(base.clone());
    c["logic"] = json!(true);
    c.as_object_mut().unwrap().remove("source");
    Parsed::new(Some(&c)).unwrap();
    // Units the reader does not know.
    let mut c = base;
    c["inputs"]["S"]["unit"] = json!("furlong");
    assert!(Parsed::new(Some(&c)).unwrap_err().contains("furlong"));
}

fn refused_ok(c: J) -> J {
    Parsed::new(Some(&c)).unwrap();
    c
}

#[test]
fn bare_zero_takes_its_place_and_choices_take_one_side() {
    let c = json!({
        "name": "Heat that leaks through a lid",
        "formula": "q = if dT <= 0 then 0 else k * dT / d",
        "inputs": {
            "dT": {"kind": "t", "unit": "K"},
            "k": {"kind": "k", "unit": "W m-1 K-1"},
            "d": {"kind": "d", "unit": "m"}
        },
        "outputs": {"q": {"kind": "q", "unit": "W m-2"}},
        "source": src(), "explain": explain(),
        "examples": [{"dT": 10, "k": 2, "d": 0.5, "q": 40}, {"dT": -3, "k": 2, "d": 0, "q": 0}]
    });
    let p = Parsed::new(Some(&c)).unwrap();
    assert_eq!(p.choices(), 1);
    assert_eq!(p.canonical(), "q = if dT <= 0 then 0 else (k * dT) / d");
    // The guarded side is not worked out (d is zero there): only dT is used.
    let (v, used) = p
        .run(&[Val::Number(0.0), Val::Number(-1.0), Val::Number(2.0)])
        .unwrap();
    assert_eq!(v, vec![Val::Number(0.0)]);
    assert_eq!(used.into_iter().collect::<Vec<_>>(), vec![1]);
}

#[test]
fn a_planck_curve_as_an_equation_is_sse_light_to_the_bit() {
    let p = Parsed::new(Some(&planck_cfg())).unwrap();
    let grid: Vec<f64> = (0..400)
        .map(|i| 1e-7 * m::pow(10.0, f64::from(i) / 100.0))
        .collect();
    let lam = Val::List(grid.iter().map(|x| Val::Number(*x)).collect());
    for t in [3.0, 30.0, 300.0, 1500.0, 5772.0, 30000.0, 1e6] {
        // Port order is alphabetical: T, c, h, k, lambda, xmax.
        let ins = [
            Val::Number(t),
            Val::Number(s::C),
            Val::Number(s::H),
            Val::Number(s::K_B),
            lam.clone(),
            Val::Number(700.0),
        ];
        let (out, _) = p.run(&ins).unwrap();
        let Val::List(b) = &out[0] else { panic!() };
        for (l, got) in grid.iter().zip(b) {
            let want = sse_light::planck(*l, t);
            assert_eq!(
                got.num().unwrap().to_bits(),
                want.to_bits(),
                "lambda {l} at {t} K"
            );
        }
    }
}

// ------------------------------------------------------------- in a flow

fn kinds() -> BTreeMap<String, QuantityKind> {
    let mut k = sse_flow::toy::kinds();
    for (id, dim) in [
        ("constant.sigma", Dimension([0, 1, -3, 0, -4, 0, 0])),
        ("pure", Dimension::NONE),
    ] {
        k.insert(
            id.into(),
            QuantityKind {
                id: id.into(),
                dim,
                range: None,
                description: String::new(),
            },
        );
    }
    k
}

fn node(id: &str, atom: &str, config: Option<J>) -> NodeDoc {
    NodeDoc {
        id: id.into(),
        atom: Some(atom.into()),
        solve: None,
        params: BTreeMap::new(),
        config,
        r#box: None,
    }
}
fn wire(a: &str, b: &str) -> WireDoc {
    WireDoc {
        from: a.into(),
        to: b.into(),
    }
}
fn input(name: &str, kind: &str) -> FlowInputDoc {
    FlowInputDoc {
        name: name.into(),
        kind: kind.into(),
        shape: None,
    }
}
fn flow(inputs: Vec<FlowInputDoc>, nodes: Vec<NodeDoc>, wires: Vec<WireDoc>) -> FlowDoc {
    FlowDoc {
        format: "sse.flow/1".into(),
        id: "test.equation".into(),
        version: "1.0.0".into(),
        hook: "test".into(),
        inputs,
        nodes,
        wires,
        publishes: vec![],
        tests: vec![],
    }
}

fn spinner() -> J {
    json!({
        "name": "Share of a fast spinner's sphere that is heated",
        "value": 0.25, "unit": "1", "range": [0.25, 0.6667], "grade": "derived",
        "source": {"text": "Geometry of a sphere lit from one side", "url": "https://example.org/geometry"}
    })
}

fn t_eq_flow() -> FlowDoc {
    flow(
        vec![
            input("S", "planet.insolation"),
            input("A", "planet.bond_albedo"),
            input("sigma", "constant.sigma"),
        ],
        vec![
            node("f", crate::model::ID, Some(spinner())),
            node("teq", ID, Some(t_eq())),
        ],
        vec![
            wire("in.S", "teq.S"),
            wire("in.A", "teq.A"),
            wire("in.sigma", "teq.sigma"),
            wire("f.value", "teq.f"),
        ],
    )
}

fn compile(d: FlowDoc) -> Result<Flow, String> {
    let (reg, k, u) = (crate::registry(), kinds(), Units::standard());
    let cx = Context {
        atoms: &reg,
        kinds: &k,
        units: &u,
    };
    Flow::compile(d, &cx).map_err(|e| {
        e.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn run(
    f: &Flow,
    ins: &[(&str, f64)],
    at: &str,
    env: &Env,
) -> Result<(Val, Vec<(String, C)>), String> {
    let (reg, k, u) = (crate::registry(), kinds(), Units::standard());
    let cx = Context {
        atoms: &reg,
        kinds: &k,
        units: &u,
    };
    let typed: BTreeMap<String, Val> = ins
        .iter()
        .map(|(n, v)| (n.to_string(), Val::Number(*v)))
        .collect();
    let e = f
        .eval_with(&cx, &BTreeMap::<String, InputValue>::new(), &typed, env)
        .map_err(|e| e.to_string())?;
    Ok((
        e.value_of(at).unwrap().clone(),
        e.evidence(at).unwrap_or_default(),
    ))
}

#[test]
fn every_letter_has_a_line_from_a_source() {
    let f = compile(t_eq_flow()).unwrap();
    let (t, _) = run(
        &f,
        &[("S", 1361.0), ("A", 0.3), ("sigma", s::SIGMA_SB)],
        "teq.T",
        &Env::empty(),
    )
    .unwrap();
    let want = m::pow(0.25 * 1361.0 * (1.0 - 0.3) / s::SIGMA_SB, 1.0 / 4.0);
    assert_eq!(t, Val::Number(want));
    // A letter with no wire is refused, naming it.
    let mut d = t_eq_flow();
    d.wires.pop();
    assert!(compile(d)
        .unwrap_err()
        .contains("teq.f: input is not wired"));
    // A typed number on an Equation's input is refused: use a Model number.
    let mut d = t_eq_flow();
    d.nodes[0] = node("f", "logic.const", Some(json!({"value": 0.25})));
    let e = compile(d).unwrap_err();
    assert!(e.contains("f comes from a typed number"), "{e}");
    // A Model number outside its range is refused at load.
    let mut d = t_eq_flow();
    let mut c = spinner();
    c["value"] = json!(0.9);
    d.nodes[0] = node("f", crate::model::ID, Some(c));
    assert!(compile(d)
        .unwrap_err()
        .contains("outside its allowed range"));
    // A Model number shows its grade and source.
    let mut c = spinner();
    c["grade"] = json!("guessed");
    assert!(crate::model::ModelCfg::parse(Some(&c))
        .unwrap_err()
        .contains("grade is one of"));
}

#[test]
fn an_equation_and_its_blocks_give_the_same_bits() {
    let d = t_eq_flow();
    let x = expand(&d).unwrap();
    assert!(x.nodes.iter().all(|n| n.atom.as_deref() != Some(ID)));
    let (f, g) = (compile(d).unwrap(), compile(x.clone()).unwrap());
    // The expansion's output ref: what the Equation's output became.
    let at = expand_ref(&x);
    for (sv, a) in [(1361.0, 0.3), (0.017, 0.99), (4.5e6, 0.0), (1e-3, 0.5)] {
        let ins = [("S", sv), ("A", a), ("sigma", s::SIGMA_SB)];
        let (p, _) = run(&f, &ins, "teq.T", &Env::empty()).unwrap();
        let (q, _) = run(&g, &ins, &at, &Env::empty()).unwrap();
        assert_eq!(p.num().unwrap().to_bits(), q.num().unwrap().to_bits());
    }
}

/// The last node of an expansion carries its output.
fn expand_ref(x: &FlowDoc) -> String {
    let last = x.nodes.last().unwrap();
    let port = if last.atom.as_deref() == Some("math.atan2") {
        "angle"
    } else {
        "value"
    };
    format!("{}.{port}", last.id)
}

struct W(BTreeMap<String, f64>);
impl World for W {
    fn fact(&self, _: Option<&sse_core::Address>, field: &str) -> Option<C> {
        self.0.get(field).map(|v| C::Float(*v))
    }
    fn has_tag(&self, _: Option<&sse_core::Address>, _: &str) -> bool {
        false
    }
}

#[test]
fn formulas_of_every_kind_expand_to_the_same_bits_and_evidence() {
    let read = |id: &str| node(id, "world.read", Some(json!({"field": id})));
    let cfg = |formula: &str| {
        json!({
            "name": "A mixed formula for the expansion test",
            "formula": formula,
            "inputs": {"a": {"kind": "pure", "unit": "1"}, "b": {"kind": "pure", "unit": "1"}, "c": {"kind": "pure", "unit": "1"}},
            "outputs": {"y": {"kind": "pure", "unit": "1"}},
            "logic": true, "explain": explain(),
            "examples": [{"a": 2, "b": 3, "c": 0.5}]
        })
    };
    let formulas = [
        "y = if c > 1 then a else b",
        "y = -a ^ 2 + b ^ c - -c",
        "u = a * b - c\ny = sqrt(abs(u)) + ln(b) * log10(a) / exp(c)",
        "y = min(a, b) - max(a, c) + clamp(c, 0, 1) * atan2(a, -b)",
        "y = sin(a) * cos(b) / (1 + tan(c) ^ 2) + pi * expm1(c / 3)",
        "y = if a >= b and not (c == 1 or c < 0) then a ^ (3/2) else b ^ (-1/2)",
        "y = (a + b) ^ c * (2 / 3) - 1 / 3 * a",
    ];
    let points = [
        (2.0, 3.0, 0.5),
        (0.4, 7.0, 1.0),
        (9.0, 1.5, 2.0),
        (1.0, 1.0, -0.25),
    ];
    for formula in formulas {
        let mut c = cfg(formula);
        // The worked example is this formula's own value at (2, 3, 0.5).
        let p = Parsed::new(Some(&{
            let mut t = c.clone();
            t["examples"] = json!([{"a": 2, "b": 3, "c": 0.5}]);
            t
        }))
        .unwrap();
        let (v, _) = p
            .run(&[Val::Number(2.0), Val::Number(3.0), Val::Number(0.5)])
            .unwrap();
        c["examples"][0]["y"] = json!(v[0].num().unwrap());
        let d = flow(
            vec![],
            vec![read("a"), read("b"), read("c"), node("eq", ID, Some(c))],
            vec![
                wire("a.value", "eq.a"),
                wire("b.value", "eq.b"),
                wire("c.value", "eq.c"),
            ],
        );
        let x = expand(&d).unwrap();
        let at = expand_ref(&x);
        let (f, g) = (compile(d).unwrap(), compile(x).unwrap());
        for (a, b, cc) in points {
            let w = W([("a".into(), a), ("b".into(), b), ("c".into(), cc)]
                .into_iter()
                .collect());
            let env = Env {
                world: Some(&w),
                ..Env::empty()
            };
            let (p, pe) = run(&f, &[], "eq.y", &env).unwrap_or_else(|e| panic!("{formula}: {e}"));
            let (q, qe) =
                run(&g, &[], &at, &env).unwrap_or_else(|e| panic!("{formula} expanded: {e}"));
            assert_eq!(
                p.num().unwrap().to_bits(),
                q.num().unwrap().to_bits(),
                "{formula} at {a}, {b}, {cc}"
            );
            assert_eq!(pe, qe, "{formula}: the same evidence");
        }
    }
    // A choice reads only the side it takes, in both forms.
    let d = flow(
        vec![],
        vec![
            read("a"),
            read("b"),
            read("c"),
            node(
                "eq",
                ID,
                Some({
                    let mut c = cfg("y = if c > 1 then a else b");
                    c["examples"][0]["y"] = json!(3);
                    c
                }),
            ),
        ],
        vec![
            wire("a.value", "eq.a"),
            wire("b.value", "eq.b"),
            wire("c.value", "eq.c"),
        ],
    );
    let w = W([("a".into(), 5.0), ("b".into(), 6.0), ("c".into(), 2.0)]
        .into_iter()
        .collect());
    let env = Env {
        world: Some(&w),
        ..Env::empty()
    };
    let (_, ev) = run(&compile(d).unwrap(), &[], "eq.y", &env).unwrap();
    let fields: Vec<&str> = ev.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(fields, vec!["a", "c"]);
}

#[test]
fn the_maths_blocks_carry_dimensions() {
    // sqrt of an area is a length; a fixed power keeps the dimension.
    let d = flow(
        vec![input("r", "planet.radius")],
        vec![
            node("sq", "math.power", Some(json!({"exponent": "2"}))),
            node("rt", "math.sqrt", None),
            node("bad", "math.power", Some(json!({"exponent": "1/2"}))),
        ],
        vec![
            wire("in.r", "sq.x"),
            wire("sq.value", "rt.x"),
            wire("in.r", "bad.x"),
        ],
    );
    let e = compile(d).unwrap_err();
    assert!(e.contains("not a whole dimension"), "{e}");
    let d = flow(
        vec![input("r", "planet.radius")],
        vec![node("x", "math.exp", None)],
        vec![wire("in.r", "x.x")],
    );
    assert!(compile(d).unwrap_err().contains("dimension mismatch"));
}

#[test]
fn the_unit_reader_knows_the_designs_units() {
    use sse_core::quantity::{parse_unit, si_unit};
    let (sc, d) = parse_unit("g/cm3").unwrap();
    assert_eq!(d, Dimension::DENSITY);
    assert!((sc - 1000.0).abs() < 1e-9);
    let (sc, d) = parse_unit("eV cm3/g").unwrap();
    assert_eq!(d, Dimension([5, 0, -2, 0, 0, 0, 0]));
    assert!((sc / (s::E * 1e-3) - 1.0).abs() < 1e-12);
    assert_eq!(
        parse_unit("m3 kg-1 s-2").unwrap().1,
        Dimension::GRAV_PARAM.div(Dimension::MASS)
    );
    assert_eq!(
        parse_unit("W m^-2 K^-4").unwrap().1,
        Dimension([0, 1, -3, 0, -4, 0, 0])
    );
    assert_eq!(parse_unit("1").unwrap(), (1.0, Dimension::NONE));
    assert_eq!(parse_unit("km/s").unwrap().1, Dimension::VELOCITY);
    assert_eq!(parse_unit("Myr").unwrap().0, 1e6 * s::YR_JULIAN);
    for d in [
        Dimension::GRAV_PARAM,
        Dimension::MAGNETIC_FIELD,
        Dimension::NONE,
    ] {
        assert_eq!(parse_unit(&si_unit(d)).unwrap().1, d);
    }
    assert!(parse_unit("a/b/c").is_err());
    assert!(parse_unit("furlong").is_err());
}
