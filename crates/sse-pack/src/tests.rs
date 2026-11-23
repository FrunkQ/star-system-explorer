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

const BOXES: &str = include_str!("../../../tests/fixtures/boxes/toy-boxes.json");
const BOXED: &str = include_str!("../../../tests/fixtures/boxes/toy-boxed.json");

fn boxed_base() -> PackManifest {
    let mut p = pack("sse.base-toy", "0.1.0", json!({"kind": ["base-physics"]}));
    p.provides.quantity_kinds = sse_flow::toy::kinds().into_values().collect();
    p.provides.flows = vec![FlowDoc::from_json(BOXED).unwrap()];
    p.provides.boxes = serde_json::from_str(BOXES).unwrap();
    p
}

fn rival_bulk(version: &str, replaces: bool) -> PackManifest {
    let mut b: Vec<sse_flow::BoxDoc> = serde_json::from_str(BOXES).unwrap();
    let mut bulk = b.remove(2);
    assert_eq!(bulk.id, "toy.bulk");
    bulk.version = version.into();
    bulk.fictional = true;
    bulk.explain.curious = "Bulk properties under the Acme rules.".into();
    let mut p = pack(
        "acme.bulk",
        "1.0.0",
        json!({"requires": [{"id": "sse.base-toy", "range": "^0.1"}]}),
    );
    if replaces {
        p.replaces = vec![Replacement {
            flow: "toy.bulk".into(),
            pack: "sse.base-toy".into(),
            range: "^0.1".into(),
        }];
    }
    p.provides.boxes = vec![bulk];
    p
}

/// J3: boxes come from packs; replacing one is explicit, as for flows, and
/// a fictional box marks everything downstream.
#[test]
fn a_pack_replaces_a_box_explicitly() {
    let s = resolve(&[boxed_base()], KERNEL).unwrap();
    assert_eq!(s.boxes.len(), 3);
    assert_eq!(s.box_packs["toy.bulk"], "sse.base-toy");
    assert!(s.fictional_kinds.is_empty());
    // A pack without boxes serialises without the field: its hash is unchanged.
    assert!(!serde_json::to_string(&base()).unwrap().contains("boxes"));
    // Providing the same box again without `replaces` is refused.
    let e = report(&resolve(&[boxed_base(), rival_bulk("1.1.0", false)], KERNEL).unwrap_err());
    assert!(
        e.contains("provides box toy.bulk already provided by sse.base-toy; add `replaces`"),
        "{e}"
    );
    // With `replaces`, the rival's box is the one the flow uses.
    let s = resolve(&[boxed_base(), rival_bulk("1.1.0", true)], KERNEL).unwrap();
    assert_eq!(s.box_packs["toy.bulk"], "acme.bulk");
    assert_eq!(s.boxes["toy.bulk"].version, "1.1.0");
    let atoms = sse_flow::toy::registry();
    let cx = sse_flow::Context {
        atoms: &atoms,
        kinds: &s.kinds,
        units: &s.units,
    };
    let f = sse_flow::Flow::compile_with(
        s.flows["toy.star-and-planet-boxed"].doc.clone(),
        &cx,
        &s.boxes,
    )
    .unwrap();
    let bulk = f.boxes.iter().find(|u| u.id == "toy.bulk").unwrap();
    assert_eq!((bulk.version.as_str(), bulk.fictional), ("1.1.0", true));
    // Its fictional flag reaches what the flow publishes.
    assert!(s.is_fictional("planet.surface_gravity"));
    // A replacement written against another version of the base is refused.
    let mut wrong = rival_bulk("1.1.0", true);
    wrong.replaces[0].range = "^3".into();
    let e = report(&resolve(&[boxed_base(), wrong], KERNEL).unwrap_err());
    assert!(
        e.contains("replaces toy.bulk@^3, but the stack has sse.base-toy 0.1.0"),
        "{e}"
    );
    // A box version outside what the flow asks for is a compile error naming both.
    let s = resolve(&[boxed_base(), rival_bulk("2.0.0", true)], KERNEL).unwrap();
    let cx = sse_flow::Context {
        atoms: &atoms,
        kinds: &s.kinds,
        units: &s.units,
    };
    let e = sse_flow::Flow::compile_with(
        s.flows["toy.star-and-planet-boxed"].doc.clone(),
        &cx,
        &s.boxes,
    )
    .unwrap_err();
    assert!(
        e.iter()
            .any(|x| x.message == "wants box toy.bulk@^1.0, but the stack has toy.bulk 2.0.0"),
        "{e:?}"
    );
}

#[test]
fn the_catalogued_solar_system_pack_loads() {
    // packs/base-sol (scripts/build-base-sol.py) is a manifest the kernel
    // accepts, and every Sol body names a source the pack carries.
    let m = PackManifest::from_json(include_str!("../../../packs/base-sol/pack.json")).unwrap();
    assert_eq!(m.id, "sse.base-sol");
    let doc: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-sol/pack.json")).unwrap();
    let sol = &doc["provides"]["other"]["sol"];
    let srcs: Vec<&str> = sol["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    let bodies = sol["bodies"].as_array().unwrap();
    assert_eq!(bodies.len(), 12);
    for b in bodies {
        for (_, v) in b["values"].as_object().unwrap() {
            assert!(srcs.contains(&v["src"].as_str().unwrap()), "{b}");
        }
    }
}

#[test]
fn the_future_tech_base_pack_loads() {
    // packs/base-tech (scripts/build-base-tech.py, FT1) is a manifest the
    // kernel accepts: ten limits, every mark cites a source the pack carries,
    // and every ceiling names its law.
    let text = include_str!("../../../packs/base-tech/pack.json");
    let m = PackManifest::from_json(text).unwrap();
    assert_eq!(m.id, "sse.base-tech");
    let doc: serde_json::Value = serde_json::from_str(text).unwrap();
    let tech = &doc["provides"]["other"]["tech"];
    assert_eq!(tech["limits"].as_array().unwrap().len(), 10);
    for f in tech["figures"].as_array().unwrap() {
        for mk in f["marks"].as_array().unwrap() {
            let src = mk["src"].as_str().unwrap();
            assert!(tech["sources"].get(src).is_some(), "{src}");
            if mk["mark"] == "ceiling" {
                assert!(mk["law"].is_string(), "{}", f["id"]);
            }
        }
    }
    assert_eq!(tech["ladder"]["size_per_level_short"], 10.0);
}
