//! The catalogued starter systems (packs/base-starters): Alpha Centauri, a
//! nested pair, and Kepler-16, a world round both stars. They enter the way
//! the start sheet's door enters Sol, and the barycentre hierarchy gives the
//! measured periods back (editing design 10A: the fixtures that prove pairs).

use sse_core::math as m;
use std::path::PathBuf;

const DAY: f64 = 86_400.0;
const YR: f64 = 365.25 * DAY;

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

/// Opens `key` from base-starters at `sys`: stars at star:N in sheet order
/// (the first is the one with no host), other bodies at body:N under the
/// system when their host is a star, else under their host. Degrees become
/// radians; the catalogue's kind goes in as body.kind.
fn open(key: &str, sys: &str) -> (crate::Kernel, Vec<(String, String)>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("packs/base-starters/pack.json")).unwrap(),
    )
    .unwrap();
    let s = &pack["provides"]["other"][key];
    let mut k = crate::Kernel::new();
    let u = sys.split('/').next().unwrap().trim_start_matches("u:");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":u,"seed":u}),
    );
    let (mut stars, mut bodies) = (0, 0);
    let mut addr: Vec<(String, String)> = vec![];
    let mut kinds: Vec<(String, String)> = vec![];
    let mut edits = vec![];
    for b in s["bodies"].as_array().unwrap() {
        let id = b["id"].as_str().unwrap().to_string();
        let kind = b["kind"].as_str().unwrap().to_string();
        let host = b["host"].as_str();
        let a = if kind == "star" {
            stars += 1;
            format!("{sys}/star:{}", stars - 1)
        } else {
            let h = host.unwrap();
            let hk = &kinds.iter().find(|(i, _)| i == h).unwrap().1;
            if hk == "star" {
                bodies += 1;
                format!("{sys}/body:{}", bodies - 1)
            } else {
                let ha = &addr.iter().find(|(i, _)| i == h).unwrap().1;
                format!("{ha}/body:0")
            }
        };
        edits.push(serde_json::json!({"address":a,"field":"name","value":b["name"]}));
        edits.push(serde_json::json!({"address":a,"field":"body.kind","value":kind}));
        if let Some(h) = host {
            let ha = &addr.iter().find(|(i, _)| i == h).unwrap().1;
            edits.push(serde_json::json!({"address":a,"field":"orbit.host","value":ha}));
        }
        for (f, v) in b["values"].as_object().unwrap() {
            let x = v["v"].as_f64().unwrap();
            let x = if v["unit"] == "deg" {
                x * m::PI / 180.0
            } else {
                x
            };
            edits.push(serde_json::json!({"address":a,"field":f,"value":x}));
        }
        addr.push((id.clone(), a));
        kinds.push((id, kind));
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "evidence":"catalogued","source":"packs/base-starters","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    (k, addr)
}

fn period_of(k: &mut crate::Kernel, sys: &str, a: &str) -> f64 {
    let r = send(
        k,
        serde_json::json!({"op":"orbits_at","system":sys,"at_s":[0]}),
    );
    let b = r["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["address"] == a)
        .unwrap_or_else(|| panic!("{a}: {r}"))
        .clone();
    assert_eq!(b["placed"], true, "{b}");
    b["period_s"].as_f64().unwrap()
}

fn at(addr: &[(String, String)], id: &str) -> String {
    addr.iter().find(|(i, _)| i == id).unwrap().1.clone()
}

/// Alpha Centauri: B round A in the measured 79.93 years (both masses);
/// Proxima round the pair (all three masses) in about 550 thousand years,
/// inside the paper's 547 (+66, -40); the system lasts.
#[test]
fn alpha_centauri_is_a_nested_pair() {
    let sys = "u:acen/sys:0";
    let (mut k, addr) = open("alpha_centauri", sys);
    let pb = period_of(&mut k, sys, &at(&addr, "alpha-cen-b")) / YR;
    assert!((pb - 79.929).abs() < 0.05, "B: {pb} yr");
    let pp = period_of(&mut k, sys, &at(&addr, "proxima")) / YR;
    assert!((507e3..613e3).contains(&pp), "Proxima: {pp} yr");
    let r = send(
        &mut k,
        serde_json::json!({"op":"orbit_stability","system":sys,"at_s":0}),
    );
    assert_eq!(r["verdict"], "lasts", "{r}");
    // Remove B, leaving Proxima round A: the preview says so and writes
    // nothing.
    let r = send(
        &mut k,
        serde_json::json!({"op":"preview_tree","system":sys,"at_s":10,"edits":[
            {"op":"withdraw","address":at(&addr, "alpha-cen-b"),"at_s":10,"others":"leave"}
        ]}),
    );
    assert_eq!(r["type"], "tree_previewed", "{r}");
    let after = r["after"]["bodies"].as_array().unwrap();
    assert!(
        !after
            .iter()
            .any(|b| b["address"] == at(&addr, "alpha-cen-b")),
        "{r}"
    );
    let prox = after
        .iter()
        .find(|b| b["address"] == at(&addr, "proxima"))
        .unwrap();
    assert_eq!(prox["host"], at(&addr, "alpha-cen-a"), "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"orbits_at","system":sys,"at_s":[20]}),
    );
    assert!(
        r["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["address"] == at(&addr, "alpha-cen-b")),
        "the preview wrote nothing: {r}"
    );
}

/// Kepler-16: the pair in 41.08 days, and the planet round both stars in
/// the measured 228.8 days (period from the pair's mass, not star A's
/// alone, which would give 242); the system lasts.
#[test]
fn kepler_16_has_a_world_round_both_stars() {
    let sys = "u:k16/sys:0";
    let (mut k, addr) = open("kepler_16", sys);
    let pb = period_of(&mut k, sys, &at(&addr, "kepler-16-b-star")) / DAY;
    assert!((pb - 41.0792).abs() < 0.02, "pair: {pb} d");
    let pp = period_of(&mut k, sys, &at(&addr, "kepler-16-b")) / DAY;
    assert!((pp - 228.776).abs() < 0.5, "planet: {pp} d");
    assert!(at(&addr, "kepler-16-b").ends_with("/body:0"));
    let r = send(
        &mut k,
        serde_json::json!({"op":"orbit_stability","system":sys,"at_s":0}),
    );
    assert_eq!(r["verdict"], "lasts", "{r}");
    // Outside the nearest lasting orbit round both stars (Holman and
    // Wiegert: 0.646 au for this pair), so read as round the pair.
    assert!(
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["check"] == "binary_both_stars"),
        "{r}"
    );
}
