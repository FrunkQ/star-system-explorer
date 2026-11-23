//! OV3 through the kernel: a pack's area layer, holdings written and read,
//! the reach rule between drawn star systems, and the same answers on two
//! kernels.

use crate::Kernel;

fn ask(k: &mut Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn layer_pack(reach_pc: f64) -> serde_json::Value {
    serde_json::json!({
        "format": "sse.pack/1", "id": "test.powers", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"overlays": [
            {"id": "so.powers", "label": "Powers of the Reach", "areas": {
                "by": {"relation": "holds"}, "answers": {"from": "actors"},
                "level": "auto",
                "fill": {"between_stars": {"reach": {"value": reach_pc, "unit": "pc"}},
                         "in_system": "hill", "on_surface": "regions"},
                "weigh_by": "count", "look": "sse.look.atlas", "tier": "obvious"}},
            {"id": "test.grid", "label": "A grid", "frame": "galaxy-plane-pc", "grid": "sse.view.hex-map"}
        ]}}
    })
}

fn open(reach_pc: f64) -> Kernel {
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [layer_pack(reach_pc)]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "ov", "seed": "ov"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    k
}

/// Two systems of the Sun's cell, their places in pc, and a third that lies
/// between them or near the first.
fn three(k: &mut Kernel) -> Vec<(String, [f64; 3])> {
    let c = ask(k, serde_json::json!({"op": "star_cell"}));
    assert_eq!(c["type"], "star_celled", "{c}");
    c["cell"]["systems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let x = &s["xyz_pc"];
            (
                s["address"].as_str().unwrap().to_string(),
                [
                    x[0].as_f64().unwrap(),
                    x[1].as_f64().unwrap(),
                    x[2].as_f64().unwrap(),
                ],
            )
        })
        .collect()
}

fn d(a: [f64; 3], b: [f64; 3]) -> f64 {
    let e = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    sse_core::math::sqrt(e[0] * e[0] + e[1] * e[1] + e[2] * e[2])
}

fn hold(k: &mut Kernel, place: &str, key: &str, at_s: i64) -> serde_json::Value {
    ask(
        k,
        serde_json::json!({"op": "hold", "place": place, "layer": "so.powers", "key": key,
                           "status": "holds", "audience": "everyone", "at_s": at_s}),
    )
}

#[test]
fn holdings_through_the_kernel() {
    let mut k = open(4.0);
    let sys = three(&mut k);
    assert!(sys.len() > 3);
    let (a, pa) = sys[0].clone();
    // The nearest other system to the first.
    let (b, pb) = sys[1..]
        .iter()
        .min_by(|x, y| d(x.1, pa).total_cmp(&d(y.1, pa)))
        .cloned()
        .unwrap();
    let r = hold(&mut k, &a, "u:ov/actor:crown", 0);
    assert_eq!(r["type"], "held", "{r}");
    assert_eq!(r["holding"]["status"], "holds");
    // A moon of the held system is the Crown's.
    let moon = format!("{a}/body:3/body:1");
    let w = ask(
        &mut k,
        serde_json::json!({"op": "who_holds", "place": moon, "layer": "so.powers"}),
    );
    assert_eq!(w["type"], "who_held", "{w}");
    assert_eq!(w["answer"]["key"], "u:ov/actor:crown");
    assert_eq!(w["answer"]["via"], "inherited");
    // The neighbour, if within the reach, takes the Crown by the reach rule.
    let w = ask(
        &mut k,
        serde_json::json!({"op": "who_holds", "place": b, "layer": "so.powers"}),
    );
    if d(pa, pb) <= 4.0 {
        assert_eq!(w["answer"]["via"], "rule", "{w}");
        assert!(
            w["answer"]["why"].as_str().unwrap().contains("pc away"),
            "{w}"
        );
    } else {
        assert!(w["answer"].is_null(), "{w}");
    }
    // A League holding on the neighbour: the border runs between them.
    hold(&mut k, &b, "u:ov/actor:league", 10);
    let w = ask(
        &mut k,
        serde_json::json!({"op": "who_holds", "place": b, "layer": "so.powers"}),
    );
    assert_eq!(w["answer"]["key"], "u:ov/actor:league");
    assert_eq!(w["answer"]["via"], "held");
    // Before the League's date, the Crown's (or nobody's).
    let w0 = ask(
        &mut k,
        serde_json::json!({"op": "who_holds", "place": b, "layer": "so.powers", "at_s": 5}),
    );
    assert_ne!(w0["answer"]["key"], "u:ov/actor:league", "{w0}");
    // Every holding in the universe, with history.
    let l = ask(
        &mut k,
        serde_json::json!({"op": "holdings_in", "bounds": "u:ov", "layer": "so.powers", "history": true}),
    );
    assert_eq!(l["type"], "holdings_listed", "{l}");
    let hs = l["holdings"].as_array().unwrap();
    assert_eq!(hs.len(), 3, "{l}");
    assert_eq!(hs[0]["place"], "u:ov");
    assert_eq!(hs[0]["divided"].as_array().unwrap().len(), 2, "{l}");
    assert_eq!(l["changes"].as_array().unwrap().len(), 2);
    // The same answers on a second kernel, written in the other order.
    let mut k2 = open(4.0);
    hold(&mut k2, &b, "u:ov/actor:league", 10);
    hold(&mut k2, &a, "u:ov/actor:crown", 0);
    let l2 = ask(
        &mut k2,
        serde_json::json!({"op": "holdings_in", "bounds": "u:ov", "layer": "so.powers"}),
    );
    assert_eq!(l["holdings"], l2["holdings"]);
}

#[test]
fn layers_are_declared_or_the_gms_own() {
    let mut k = open(4.0);
    let a = three(&mut k)[0].0.clone();
    let r = ask(
        &mut k,
        serde_json::json!({"op": "hold", "place": a, "layer": "nope", "key": "x", "status": "holds", "at_s": 0}),
    );
    assert_eq!(r["code"], "unknown_layer", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "hold", "place": a, "layer": "misc.trade", "key": "guild", "status": "influences",
                           "strength": 0.4, "at_s": 0}),
    );
    assert_eq!(r["type"], "held", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "hold", "place": a, "layer": "so.powers", "key": "x", "status": "rules", "at_s": 0}),
    );
    assert_eq!(r["code"], "bad_holding", "{r}");
    // A bad reach in a pack is refused at load.
    let mut bad = layer_pack(4.0);
    bad["provides"]["other"]["overlays"][0]["areas"]["fill"]["between_stars"]["reach"]["unit"] =
        "kg".into();
    let r = ask(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [bad]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(r["issues"].to_string().contains("not a length"), "{r}");
}

#[test]
fn players_never_see_a_secret_holding() {
    let mut k = open(4.0);
    let a = three(&mut k)[0].0.clone();
    let r = ask(
        &mut k,
        serde_json::json!({"op": "hold", "place": a, "layer": "so.powers", "key": "u:ov/actor:vesh",
                           "status": "holds", "audience": "gm", "at_s": 0}),
    );
    assert_eq!(r["type"], "held", "{r}");
    let w = ask(
        &mut k,
        serde_json::json!({"op": "who_holds", "place": a, "layer": "so.powers", "audience": "crew"}),
    );
    assert!(w["answer"].is_null(), "{w}");
    let l = ask(
        &mut k,
        serde_json::json!({"op": "holdings_in", "bounds": "u:ov", "layer": "so.powers", "audience": "crew", "history": true}),
    );
    assert!(!l.to_string().contains("vesh"), "{l}");
}
