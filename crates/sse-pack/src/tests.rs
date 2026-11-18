use super::*;
use serde_json::json;

const TOY: &str = include_str!("../../../tests/fixtures/flows/toy-ten.json");
const KERNEL: &str = "0.1.0";

fn pack(id: &str, version: &str, extra: serde_json::Value) -> PackManifest {
    let mut v = json!({
        "format": "sse.pack/1", "id": id, "version": version, "kind": ["physics"],
        "kernel_range": "^0.1", "licence": "Apache-2.0"
    });
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

fn base() -> PackManifest {
    let mut p = pack("sse.base-toy", "0.1.0", json!({"kind": ["base-physics"]}));
    p.provides.quantity_kinds = sse_flow::toy::kinds().into_values().collect();
    p.provides.flows = vec![FlowDoc::from_json(TOY).unwrap()];
    p
}

/// A one-node flow publishing `kind` at `hook` from the toy equilibrium atom.
fn teq_flow(id: &str, hook: &str, input_kind: &str, publish: &str) -> serde_json::Value {
    json!({
        "format": "sse.flow/1", "id": id, "version": "0.1.0", "hook": hook,
        "inputs": [{"name": "flux", "kind": input_kind}, {"name": "albedo", "kind": "planet.bond_albedo"}],
        "nodes": [{"id": "t", "atom": "toy.equilibrium_temperature"}],
        "wires": [{"from": "in.flux", "to": "t.flux"}, {"from": "in.albedo", "to": "t.albedo"}],
        "publishes": [{"kind": publish, "from": "t.teq"}]
    })
}

#[test]
fn base_stack_resolves_with_one_publisher_each() {
    let s = resolve(&[base()], KERNEL).unwrap();
    assert_eq!(s.publishers.len(), 10);
    assert_eq!(
        s.publisher_of("system.toy", "planet.equilibrium_temperature")
            .unwrap()
            .pack,
        "sse.base-toy"
    );
    assert!(s.fictional_kinds.is_empty());
    let again = resolve(&[base()], KERNEL).unwrap();
    assert_eq!(s.hash, again.hash);
}

#[test]
fn conflicting_packs_are_rejected_with_a_readable_report() {
    let rival = pack(
        "acme.hot-planets",
        "1.0.0",
        json!({
            "requires": [{"id": "sse.base-toy", "range": "^0.1"}],
            "provides": {"flows": [teq_flow("acme.teq", "system.toy", "planet.insolation", "planet.equilibrium_temperature")]}
        }),
    );
    let e = resolve(&[base(), rival], KERNEL).unwrap_err();
    let r = report(&e);
    assert!(r.contains("only one publisher is allowed"), "{r}");
    assert!(
        r.contains("planet.equilibrium_temperature at hook system.toy"),
        "{r}"
    );
    assert!(r.starts_with("Pack stack rejected: 1 problem\n"), "{r}");
}

#[test]
fn declared_override_works() {
    let mut toy_v: serde_json::Value = serde_json::from_str(TOY).unwrap();
    toy_v["id"] = "acme.star-and-planet-hot".into();
    toy_v["nodes"][9]["params"]["tau"] = 3.0.into(); // thicker greenhouse
    let over = pack(
        "acme.thick-air",
        "1.0.0",
        json!({
            "requires": [{"id": "sse.base-toy", "range": "^0.1"}],
            "replaces": [{"flow": "toy.star-and-planet", "pack": "sse.base-toy", "range": "^0.1"}],
            "provides": {"flows": [toy_v]}
        }),
    );
    let s = resolve(&[base(), over.clone()], KERNEL).unwrap();
    let p = s
        .publisher_of("system.toy", "planet.surface_temperature")
        .unwrap();
    assert_eq!(
        (p.pack.as_str(), p.flow.as_str()),
        ("acme.thick-air", "acme.star-and-planet-hot")
    );
    assert!(!s.flows.contains_key("toy.star-and-planet"));

    // an override written for a different version of the base is refused
    let mut wrong = over;
    wrong.replaces[0].range = "^2".into();
    let e = report(&resolve(&[base(), wrong], KERNEL).unwrap_err());
    assert!(
        e.contains("replaces toy.star-and-planet@^2, but the stack has sse.base-toy 0.1.0"),
        "{e}"
    );
}

#[test]
fn fictional_label_propagates_downstream() {
    let mut force = pack(
        "acme.force",
        "1.0.0",
        json!({
            "fictional": true,
            "requires": [{"id": "sse.base-toy", "range": "^0.1"}],
            "provides": {"flows": [teq_flow("acme.force-heat", "planet.force", "planet.insolation", "planet.force_temperature")]}
        }),
    );
    force.provides.quantity_kinds.push(QuantityKind {
        id: "planet.force_temperature".into(),
        dim: sse_core::Dimension::TEMPERATURE,
        range: None,
        description: "how hot the Force runs here".into(),
    });
    // a real-physics pack that consumes the fictional kind
    let mut consumer_flow = teq_flow(
        "acme.glow",
        "planet.glow",
        "planet.insolation",
        "planet.glow_temperature",
    );
    consumer_flow["inputs"][0]["kind"] = "planet.force_temperature".into();
    let mut glow = pack(
        "acme.glow",
        "1.0.0",
        json!({"provides": {"flows": [consumer_flow]}}),
    );
    glow.provides.quantity_kinds.push(QuantityKind {
        id: "planet.glow_temperature".into(),
        dim: sse_core::Dimension::TEMPERATURE,
        range: None,
        description: String::new(),
    });
    let s = resolve(&[base(), force, glow], KERNEL).unwrap();
    assert!(s.is_fictional("planet.force_temperature"));
    assert!(
        s.is_fictional("planet.glow_temperature"),
        "fictional input taints the consumer"
    );
    assert!(!s.is_fictional("planet.equilibrium_temperature"));
}

#[test]
fn structural_problems_are_all_reported() {
    let mut bad = pack("Bad Id", "one", json!({}));
    bad.kernel_range = "^9".into();
    let mut redefine = pack(
        "acme.redefine",
        "1.0.0",
        json!({
            "kernel_range": "^9",
            "requires": [{"id": "acme.missing", "range": "^1"}],
            "provides": {"units": [{"id": "au", "symbol": "au", "dim": [1,0,0,0,0,0,0], "scale": 1.5e11}]}
        }),
    );
    redefine.provides.quantity_kinds.push(QuantityKind {
        id: "star.mass".into(),
        dim: sse_core::Dimension::LENGTH,
        range: None,
        description: String::new(),
    });
    let dup = base();
    let e = report(&resolve(&[base(), bad, redefine, dup], KERNEL).unwrap_err());
    for want in [
        "namespaced lowercase",
        "is not semver",
        "needs kernel ^9, this kernel is 0.1.0",
        "requires acme.missing ^1, which is not lower in the stack",
        "redefines unit au",
        "redefines quantity kind star.mass as [m]",
        "appears twice in the stack",
        "already provided by sse.base-toy; add `replaces`",
    ] {
        assert!(e.contains(want), "missing {want:?} in\n{e}");
    }
}

#[test]
fn every_flow_in_the_resolved_stack_compiles() {
    let s = resolve(&[base()], KERNEL).unwrap();
    let atoms = sse_flow::toy::registry();
    let cx = sse_flow::Context {
        atoms: &atoms,
        kinds: &s.kinds,
        units: &s.units,
    };
    for f in s.flows.values() {
        sse_flow::Flow::compile(f.doc.clone(), &cx).unwrap();
    }
}
