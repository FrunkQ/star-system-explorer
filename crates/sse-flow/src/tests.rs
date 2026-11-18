use crate::flow::*;
use crate::toy;
use sse_core::{Quantity, Units};
use std::collections::BTreeMap;

const TOY: &str = include_str!("../../../tests/fixtures/flows/toy-ten.json");

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

fn earth_inputs(u: &Units, star_mass_sun: f64, src: SourceLabel) -> BTreeMap<String, InputValue> {
    let iv = |q: Quantity| InputValue {
        q,
        source: SourceLabel::Resolved,
    };
    let mut m = BTreeMap::new();
    m.insert(
        "star_mass".into(),
        InputValue {
            q: u.q(star_mass_sun, "m_sun").unwrap(),
            source: src,
        },
    );
    m.insert("distance".into(), iv(u.q(1.0, "au").unwrap()));
    m.insert("albedo".into(), iv(Quantity::scalar(0.3)));
    m.insert("planet_mass".into(), iv(u.q(1.0, "m_earth").unwrap()));
    m.insert("planet_radius".into(), iv(u.q(1.0, "r_earth").unwrap()));
    m
}

#[test]
fn ten_atom_flow_evaluates_and_passes_its_golden_cases() {
    let fx = Fx::new();
    let flow = Flow::compile(FlowDoc::from_json(TOY).unwrap(), &fx.cx()).unwrap();
    assert_eq!(flow.node_ids().count(), 10);
    assert_eq!(flow.run_tests(&fx.cx()), Vec::<String>::new());
}

#[test]
fn every_output_is_explained() {
    let fx = Fx::new();
    let flow = Flow::compile(FlowDoc::from_json(TOY).unwrap(), &fx.cx()).unwrap();
    let ev = flow
        .eval(
            &fx.cx(),
            &earth_inputs(&fx.units, 1.0, SourceLabel::Resolved),
        )
        .unwrap();
    let kinds: Vec<String> = flow.published_kinds().map(String::from).collect();
    assert_eq!(kinds.len(), 10);
    for k in &kinds {
        let e = ev
            .explain(k)
            .unwrap_or_else(|| panic!("no explanation for {k}"));
        assert_eq!(e.validity, Validity::InRange, "{k}");
        assert!(
            !e.formula_display.is_empty()
                && !e.lenses.curious.is_empty()
                && e.lenses.engine.contains(&flow.hash.hex()),
            "{k}"
        );
        assert!(!e.inputs.is_empty());
        // every derived input can itself be explained: walk the chain to flow inputs
        for i in &e.inputs {
            if let Some(r) = &i.explanation_ref {
                assert!(ev.explain_port(r).is_some(), "{k} -> {r}");
            }
        }
    }
    let ts = ev.explain("planet.surface_temperature").unwrap();
    assert!(ts.lenses.science.contains("bisection"));
    assert_eq!(
        ts.inputs
            .iter()
            .find(|i| i.port == "flux")
            .unwrap()
            .explanation_ref
            .as_deref(),
        Some("sun.flux")
    );
    assert!(ev.work_units >= 90);
}

#[test]
fn validity_labels_propagate() {
    let fx = Fx::new();
    let flow = Flow::compile(FlowDoc::from_json(TOY).unwrap(), &fx.cx()).unwrap();
    // a 40 solar-mass star is outside the toy mass-luminosity range
    let ev = flow
        .eval(
            &fx.cx(),
            &earth_inputs(&fx.units, 40.0, SourceLabel::Resolved),
        )
        .unwrap();
    assert_eq!(
        ev.validity_of("star.luminosity"),
        Some(Validity::Extrapolated)
    );
    assert_eq!(
        ev.validity_of("planet.equilibrium_temperature"),
        Some(Validity::Extrapolated)
    );
    assert_eq!(
        ev.validity_of("planet.surface_gravity"),
        Some(Validity::InRange)
    );
    let e = ev.explain("star.luminosity").unwrap();
    assert_eq!(e.out_of_range, vec!["mass".to_string()]);
    // an impossible authored input marks everything downstream
    let ev = flow
        .eval(
            &fx.cx(),
            &earth_inputs(&fx.units, 1.0, SourceLabel::AuthoredImpossible),
        )
        .unwrap();
    assert_eq!(
        ev.validity_of("planet.surface_temperature"),
        Some(Validity::ImpossibleInput)
    );
    assert_eq!(
        ev.validity_of("planet.bulk_density"),
        Some(Validity::InRange)
    );
}

fn errors_of(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<String> {
    let fx = Fx::new();
    let mut v: serde_json::Value = serde_json::from_str(TOY).unwrap();
    edit(&mut v);
    let doc: FlowDoc = serde_json::from_value(v).unwrap();
    match Flow::compile(doc, &fx.cx()) {
        Ok(_) => vec![],
        Err(e) => e.iter().map(|x| x.to_string()).collect(),
    }
}

#[test]
fn rejects_cycles() {
    let e = errors_of(|v| {
        // feed teq's output back into lum via a new loop: teff -> rad is a cycle with rad -> teff
        v["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id": "loop", "atom": "toy.radius_from_mass"}));
        let w = v["wires"].as_array_mut().unwrap();
        w.retain(|x| x["to"] != "rad.mass");
        w.push(serde_json::json!({"from": "loop.radius", "to": "teff.radius"}));
        w.retain(|x| x["to"] != "teff.radius" || x["from"] == "loop.radius");
        w.push(serde_json::json!({"from": "lum.luminosity", "to": "loop.mass"}));
    });
    // loop.mass gets a power into a mass port: dimension error too; now make a real cycle
    assert!(e.iter().any(|m| m.contains("dimension mismatch")), "{e:?}");
    let e = errors_of(|v| {
        v["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id": "a", "atom": "toy.surface_gravity"}));
        v["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id": "b", "atom": "toy.bulk_density"}));
        let w = v["wires"].as_array_mut().unwrap();
        w.push(serde_json::json!({"from": "in.planet_mass", "to": "a.mass"}));
        w.push(serde_json::json!({"from": "b.density", "to": "a.radius"}));
        w.push(serde_json::json!({"from": "in.planet_mass", "to": "b.mass"}));
        w.push(serde_json::json!({"from": "a.g", "to": "b.radius"}));
    });
    assert!(
        e.iter().any(|m| m.contains("cycle through nodes a, b")),
        "{e:?}"
    );
}

#[test]
fn rejects_dimension_and_structure_errors() {
    let e = errors_of(|v| {
        let w = v["wires"].as_array_mut().unwrap();
        for x in w.iter_mut() {
            if x["to"] == "sun.distance" {
                x["from"] = "in.star_mass".into();
            }
        }
    });
    assert!(
        e.iter()
            .any(|m| m.contains("dimension mismatch: [kg] into [m]")),
        "{e:?}"
    );

    let e = errors_of(|v| {
        v["wires"]
            .as_array_mut()
            .unwrap()
            .retain(|x| x["to"] != "teq.albedo");
    });
    assert!(
        e.iter().any(|m| m == "teq.albedo: input is not wired"),
        "{e:?}"
    );

    let e = errors_of(|v| {
        v["publishes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"kind": "planet.insolation", "from": "sun.flux"}));
        v["nodes"][0]["params"]["exponent"] = 99.0.into();
        v["nodes"][1]["atom"] = "toy.nope".into();
        v["publishes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"kind": "planet.mass", "from": "teq.teq"}));
    });
    assert!(e.iter().any(|m| m.contains("published twice")), "{e:?}");
    assert!(
        e.iter().any(|m| m.contains("outside allowed range")),
        "{e:?}"
    );
    assert!(
        e.iter().any(|m| m.contains("unknown atom \"toy.nope\"")),
        "{e:?}"
    );
    assert!(
        e.iter()
            .any(|m| m.contains("publish planet.mass: dimension mismatch")),
        "{e:?}"
    );
}

#[test]
fn nan_is_an_error_not_a_value() {
    let fx = Fx::new();
    let flow = Flow::compile(FlowDoc::from_json(TOY).unwrap(), &fx.cx()).unwrap();
    let mut i = earth_inputs(&fx.units, 1.0, SourceLabel::Resolved);
    i.get_mut("planet_radius").unwrap().q.value = 0.0; // g = G m / 0 -> inf
    let e = flow.eval(&fx.cx(), &i).unwrap_err();
    assert!(e.message.contains("not finite"), "{e}");
}

#[test]
fn evaluation_is_bit_stable() {
    let fx = Fx::new();
    let flow = Flow::compile(FlowDoc::from_json(TOY).unwrap(), &fx.cx()).unwrap();
    let ev = flow
        .eval(
            &fx.cx(),
            &earth_inputs(&fx.units, 1.0, SourceLabel::Resolved),
        )
        .unwrap();
    let mut h = sse_core::hash::Hasher::new("toy-flow");
    for k in flow.published_kinds() {
        h.update(k.as_bytes())
            .update(&ev.published(k).unwrap().value.to_bits().to_be_bytes());
    }
    assert_eq!(
        h.finish().hex(),
        "a063fe366d1dc9f6f02a652408cbf7165cd50609dbdf51c7ac455fd5072d1d5a"
    );
    assert_eq!(
        flow.hash.hex(),
        "327fac6b8cf7ab82e697ab3ef06b60635308b9d38cf926f1beb20a50ec675729"
    );
}
