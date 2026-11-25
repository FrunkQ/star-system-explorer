//! ND6: packs carry their own sources, fudge list and reproduction checks;
//! the kernel runs the checks on load and reports them.

use serde_json::{json, Value};
use std::path::PathBuf;

fn pack(p: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    serde_json::from_str(&std::fs::read_to_string(root.join(p)).unwrap()).unwrap()
}
fn send(k: &mut crate::Kernel, v: Value) -> Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}
fn load(packs: Vec<Value>) -> (crate::Kernel, Value) {
    let mut k = crate::Kernel::new();
    let r = send(&mut k, json!({"op": "load_stack", "packs": packs}));
    (k, r)
}
fn base() -> Value {
    pack("packs/base-physics/pack.json")
}

/// A small pack in the shape a paper's pack takes: a flow with a fitted
/// Model number, a declared fudge, two sources and a check against a
/// published number run through the base pack's planet flow.
fn paper(tolerance: f64) -> Value {
    json!({
        "format": "sse.pack/1",
        "id": "test.paper",
        "version": "0.1.0",
        "kind": ["physics"],
        "kernel_range": "^0.1",
        "licence": "CC-BY-4.0",
        "requires": [{"id": "sse.base-physics", "range": "^0.1"}],
        "provides": {
            "quantity_kinds": [
                {"id": "test.length", "dim": [1, 0, 0, 0, 0, 0, 0], "description": "ND6 fixture"},
                {"id": "test.stretched", "dim": [1, 0, 0, 0, 0, 0, 0], "description": "ND6 fixture"}
            ],
            "flows": [{
                "format": "sse.flow/1",
                "id": "test.stretch",
                "version": "1.0.0",
                "hook": "test.stretch",
                "inputs": [{"name": "length", "kind": "test.length"}],
                "nodes": [
                    {"id": "k", "atom": "sse.model.number", "config": {
                        "name": "stretch factor", "value": 1.5, "unit": "1", "range": [1, 2],
                        "grade": "fitted",
                        "source": {"text": "A made-up fit", "url": "https://example.org/fit", "where": "Table 1"}
                    }},
                    {"id": "s", "atom": "sse.math.equation", "config": {
                        "name": "How long it ends up",
                        "formula": "y = k * x",
                        "inputs": {
                            "k": {"kind": "model.number", "unit": "1", "name": "stretch factor"},
                            "x": {"kind": "test.length", "unit": "m", "name": "length before"}
                        },
                        "outputs": {"y": {"kind": "test.stretched", "unit": "m", "name": "length after"}},
                        "source": {"text": "Definition", "url": "https://example.org/def", "where": "eq. 1"},
                        "explain": {"curious": "It gets longer.", "science": "A linear stretch."},
                        "examples": [{"k": 2, "x": 3, "y": 6}]
                    }}
                ],
                "wires": [
                    {"from": "k.value", "to": "s.k"},
                    {"from": "in.length", "to": "s.x"}
                ],
                "publishes": [{"kind": "test.stretched", "from": "s.y"}]
            }]
        },
        "sources": [
            {"id": "facts", "text": "Planetary fact sheet", "url": "https://nssdc.gsfc.nasa.gov/planetary/factsheet/"},
            {"id": "fit", "text": "A made-up fit", "doi": "10.0000/example"}
        ],
        "fudges": [{
            "id": "linear",
            "what": "a straight-line stretch",
            "stands_in_for": "the full curve",
            "replace_with": "a table of measured lengths",
            "flows": ["test.stretch"],
            "source": "fit"
        }],
        "reproduction": [{
            "name": "A world like ours",
            "source": "facts",
            "where": "gravity row",
            "flow": "sse.base.planet-bulk",
            "inputs": {"mass": {"value": 5.97, "unit": "kg"}, "radius": {"value": 6371, "unit": "km"}},
            "expect": {"body.surface_gravity": {"value": 9.8, "unit": "m_s2", "tolerance": tolerance}}
        }]
    })
}

#[test]
fn a_pack_reports_its_sources_fudges_and_reproduction() {
    let mut p = paper(0.01);
    p["reproduction"][0]["inputs"]["mass"]["value"] = json!(5.97e24);
    let (mut k, r) = load(vec![base(), p]);
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(&mut k, json!({"op": "pack_reports", "pack": "test.paper"}));
    assert_eq!(r["type"], "pack_reports", "{r}");
    let rep = &r["reports"][0];
    assert_eq!(rep["pack"], "test.paper");
    assert_eq!(rep["reproduced"], true);
    assert_eq!(rep["sources"].as_array().unwrap().len(), 2);
    // Declared first, then the fitted Model number the kernel found.
    let f = rep["fudges"].as_array().unwrap();
    assert_eq!(f.len(), 2, "{f:?}");
    assert_eq!(f[0]["from"], "declared");
    assert_eq!(f[1]["from"], "model_number");
    assert_eq!(f[1]["id"], "test.stretch/k");
    assert_eq!(f[1]["number"]["grade"], "fitted");
    assert_eq!(f[1]["source"], "https://example.org/fit");
    let row = &rep["reproduction"][0]["rows"][0];
    assert_eq!(row["kind"], "body.surface_gravity");
    assert_eq!(row["published"], 9.8);
    let got = row["got"].as_f64().unwrap();
    assert!((got - 9.81).abs() < 0.02, "{row}");
    assert!(row["off"].as_f64().unwrap() < 0.01, "{row}");
    assert_eq!(row["pass"], true);
    // The whole stack: one report per pack, the base pack's empty.
    let all = send(&mut k, json!({"op": "pack_reports"}));
    let all = all["reports"].as_array().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0]["pack"], "sse.base-physics");
    assert_eq!(all[0]["reproduced"], true);
}

#[test]
fn a_check_outside_its_tolerance_refuses_the_pack_with_both_numbers() {
    let mut p = paper(0.0001);
    p["reproduction"][0]["inputs"]["mass"]["value"] = json!(5.97e24);
    let (_, r) = load(vec![base(), p]);
    assert_eq!(r["type"], "stack_rejected", "{r}");
    let m = r["issues"][0].as_str().unwrap();
    assert!(m.contains("A world like ours"), "{m}");
    assert!(m.contains("facts gives 9.8 m_s2"), "{m}");
    assert!(m.contains("test.paper gives 9.8"), "{m}");
    assert!(m.contains("the pack allows 0.010 %"), "{m}");
}

#[test]
fn a_check_that_cannot_run_says_why() {
    // 5.97 kg: a world far too light, so gravity is far off the published one.
    let (_, r) = load(vec![base(), paper(0.01)]);
    assert_eq!(r["type"], "stack_rejected", "{r}");
    let mut p = paper(0.01);
    p["reproduction"][0]["flow"] = json!("test.nowhere");
    let (_, r) = load(vec![base(), p]);
    let m = r["issues"][0].as_str().unwrap();
    assert!(
        m.contains("could not run (the stack has no flow test.nowhere)"),
        "{m}"
    );
}

#[test]
fn sources_and_fudges_are_checked_with_the_stack() {
    let mut p = paper(0.01);
    p["reproduction"][0]["source"] = json!("nobody");
    p["fudges"][0]["flows"] = json!(["sse.base.planet-bulk"]);
    p["sources"][1]["doi"] = json!("");
    let (_, r) = load(vec![base(), p]);
    assert_eq!(r["type"], "stack_rejected", "{r}");
    let all = r["issues"].to_string();
    assert!(
        all.contains("cites nobody, which is not among the pack's sources"),
        "{all}"
    );
    assert!(
        all.contains("lives in sse.base.planet-bulk, which is not a flow or box of this pack"),
        "{all}"
    );
    assert!(all.contains("source fit has a DOI or an address"), "{all}");
}

#[test]
fn an_unknown_pack_has_no_report() {
    let (mut k, _) = load(vec![base()]);
    let r = send(&mut k, json!({"op": "pack_reports", "pack": "test.paper"}));
    assert_eq!(r["code"], "unknown_pack", "{r}");
}

#[test]
fn a_pack_without_them_hashes_as_before() {
    let b = base();
    let m: sse_pack::PackManifest = serde_json::from_value(b.clone()).unwrap();
    let back = serde_json::to_value(&m).unwrap();
    for k in ["sources", "fudges", "reproduction"] {
        assert!(back.get(k).is_none(), "{k}");
    }
}
