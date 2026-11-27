//! SB2: small bodies through the kernel.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const SYS: &str = "u:small/sys:1";

fn open() -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-reference/pack.json")).unwrap(),
    )
    .unwrap();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[sheet, packs["base-physics"].clone()]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"small","seed":"small"}),
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":"kernel-form-9","system":SYS,"star_pins":{"primary_mass":1.0}}}),
    );
    assert_ne!(r["type"], "error", "{r}");
    k
}

fn clouds(k: &mut crate::Kernel) -> Vec<serde_json::Value> {
    let r = send(k, serde_json::json!({"op":"resolved_system","system":SYS}));
    r["clouds"].as_array().cloned().unwrap_or_default()
}

#[test]
fn a_belt_member_resolves_the_same_every_time() {
    let mut k = open();
    let cs = clouds(&mut k);
    assert!(!cs.is_empty(), "the formed system has clouds");
    let cloud = cs[0]["address"].as_str().unwrap().to_string();
    let member = format!("{cloud}/obj:40");
    let a = send(
        &mut k,
        serde_json::json!({"op":"small_body","system":SYS,"member":member}),
    );
    assert_eq!(a["type"], "small_body_is", "{a}");
    let b = send(
        &mut k,
        serde_json::json!({"op":"small_body","system":SYS,"cloud":cloud,"rank":40}),
    );
    assert_eq!(a, b);
    let body = &a["body"];
    assert_eq!(body["address"], member);
    for f in [
        "mass_kg",
        "radius_m",
        "makeup.metal",
        "makeup.rock",
        "makeup.carbon",
        "makeup.ice",
    ] {
        assert!(body["facts"][f].is_number(), "{f}");
    }
    assert!(body["history"].as_array().is_some_and(|h| !h.is_empty()));
    let bad = send(
        &mut k,
        serde_json::json!({"op":"small_body","system":SYS,"member":format!("{cloud}/obj:0")}),
    );
    assert_eq!(bad["type"], "error");
}

#[test]
fn notables_list_resolves_and_made_bodies_follow_their_pins() {
    let mut k = open();
    let n = send(&mut k, serde_json::json!({"op":"small_body","system":SYS}));
    assert_eq!(n["type"], "small_body_is", "{n}");
    for b in n["notables"].as_array().unwrap() {
        assert_eq!(b["structure"]["kind"], "stripped-core");
        assert!(b["why"].as_str().unwrap().contains("iron core"));
    }
    let addr = format!("{SYS}/small:gm-1");
    let c = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"contact-binary","diameter_km":1.0,"a_au":1.5}),
    );
    assert_eq!(c["type"], "small_body_is", "{c}");
    assert_eq!(c["body"]["pair"]["kind"], "contact");
    assert_eq!(c["body"]["figure"]["lobes"].as_array().unwrap().len(), 2);
    assert_eq!(c["body"]["address"], addr);
    let d = c["body"]["diameter_km"].as_f64().unwrap();
    assert!(d > 0.7 && d < 1.4, "{d}");
    let p = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"binary-pair","mass_kg":1e12,"a_au":1.5}),
    );
    assert_eq!(p["body"]["pair"]["kind"], "separated", "{p}");
    let comet = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"comet-nucleus","diameter_km":4.0,"a_au":3.5,"e":0.7}),
    );
    assert_eq!(comet["type"], "small_body_is", "{comet}");
    // SB4: a comet nucleus at perihelion (3.5 x 0.3 = 1.05 AU) is active, water leading.
    let act = &comet["body"]["activity"];
    assert_eq!(act["active"], true, "{act}");
    assert_eq!(act["dominant"], "h2o", "{act}");
    assert!(act["coma_km"].as_f64().unwrap() > 1e4);
    let bad = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":addr,"kind":"dragon","diameter_km":1.0,"a_au":1.0}),
    );
    assert_eq!(bad["type"], "error");
}

#[test]
fn the_designer_pins_reach_the_body_and_say_what_they_take() {
    let mut k = open();
    let r = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":format!("{SYS}/small:d1"),"kind":"contact-binary","diameter_km":2.0,"a_au":2.5,"structure":"rubble-pile","spin_hours":1.0,"size_ratio":0.7}),
    );
    assert_eq!(r["type"], "small_body_is", "{r}");
    let b = &r["body"];
    assert_eq!(b["structure"]["kind"], "rubble-pile");
    assert_eq!(b["pair"]["kind"], "contact");
    assert!((b["spin_hours"].as_f64().unwrap() - 1.0).abs() < 1e-9);
    let takes = b["takes"]
        .as_array()
        .expect("an implausible spin says what it takes");
    assert!(takes.iter().any(|t| t["about"] == "spin"), "{takes:?}");
    // Bad makeup is a request error, not a refusal of the design.
    let bad = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":SYS,"address":format!("{SYS}/small:d2"),"kind":"asteroid","diameter_km":2.0,"a_au":2.5,"makeup":[1.0]}),
    );
    assert_eq!(bad["type"], "error", "{bad}");
}

#[test]
fn a_body_can_be_designed_where_no_formation_ran() {
    let mut k = open();
    let sys = "u:small/sys:2";
    let r = send(
        &mut k,
        serde_json::json!({"op":"make_small_body","system":sys,"address":format!("{sys}/small:c1"),"kind":"comet-nucleus","diameter_km":5.0,"a_au":3.0,"e":0.6}),
    );
    assert_eq!(r["type"], "small_body_is", "{r}");
    assert_eq!(r["body"]["made_from"], "designed");
    assert!(r["body"]["makeup"]["ice"].as_f64().unwrap() > 0.3);
    assert!(
        r["body"]["activity"].is_object(),
        "an icy nucleus says how active it is"
    );
}

#[test]
fn small_shape_gives_any_small_body_its_lump() {
    let mut k = open();
    let sys = "u:small/sys:2";
    let ask = |k: &mut crate::Kernel, a: &str| {
        send(
            k,
            serde_json::json!({"op":"small_shape","system":sys,"address":format!("{sys}/body:{a}"),"diameter_km":22.5,"mass_kg":1.07e16,"spin_hours":7.65}),
        )
    };
    let p = ask(&mut k, "phobos");
    assert_eq!(p["type"], "small_body_is", "{p}");
    let b = &p["body"];
    assert_eq!(b["round"], false);
    assert_eq!(
        b["pair"]["kind"], "single",
        "a known body is never made a pair"
    );
    assert!(b["figure"]["relief"].as_f64().unwrap() > 0.01);
    let ax = b["figure"]["axes_m"].as_array().unwrap();
    assert!(
        ax[2].as_f64().unwrap() / ax[0].as_f64().unwrap() < 0.97,
        "{ax:?}"
    );
    // Phobos's density (1,860) reads as part ice, part rock.
    assert!(b["makeup"]["ice"].as_f64().unwrap() > 0.1);
    // The same address is the same lump; another is another.
    assert_eq!(ask(&mut k, "phobos"), p);
    assert_ne!(ask(&mut k, "deimos")["body"]["figure"], b["figure"]);
    let big = send(
        &mut k,
        serde_json::json!({"op":"small_shape","system":sys,"address":format!("{sys}/body:dwarf"),"diameter_km":1200.0,"ice":0.5}),
    );
    assert_eq!(big["body"]["round"], true, "{big}");
}

/// A formation seed whose system has a ringed giant (found by search: kernel-form-81, cloud 2).
fn open_ringed() -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["base-physics"].clone()]}),
    );
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"small","seed":"small"}),
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":"kernel-form-81","system":SYS,"star_pins":{"primary_mass":1.0}}}),
    );
    assert_ne!(r["type"], "error", "{r}");
    k
}

#[test]
fn a_formed_ring_is_one_ring_with_zones_hung_on_its_world() {
    let mut k = open_ringed();
    let cs = clouds(&mut k);
    let ring = cs.iter().find(|c| c["kind"] == "ring").expect("a ring");
    let world = ring["around_address"]
        .as_str()
        .expect("the ring names its world by address");
    assert!(world.starts_with(SYS), "{world}");
    let cloud = ring["address"].as_str().unwrap();
    let a = send(
        &mut k,
        serde_json::json!({"op":"ring_structure","system":SYS,"cloud":cloud,"samples":256}),
    );
    assert_eq!(a["type"], "ring_structure_is", "{a}");
    let r = &a["ring"];
    let zones = r["zones"].as_array().unwrap();
    assert!(zones.len() >= 2, "{r}");
    // The zones hold the cloud's mass, and the profile is the asked size.
    let sum: f64 = zones.iter().map(|z| z["mass_kg"].as_f64().unwrap()).sum();
    assert!((sum / r["mass_kg"].as_f64().unwrap() - 1.0).abs() < 1e-9);
    assert_eq!(r["profile"]["tau"].as_array().unwrap().len(), 256);
    // The same ask is the same ring; another variant is another ring with the same marks.
    let b = send(
        &mut k,
        serde_json::json!({"op":"ring_structure","system":SYS,"cloud":cloud,"samples":256}),
    );
    assert_eq!(a, b);
    let c = send(
        &mut k,
        serde_json::json!({"op":"ring_structure","system":SYS,"cloud":cloud,"samples":256,"variant":1}),
    );
    assert_eq!(a["ring"]["marks"], c["ring"]["marks"]);
    assert_ne!(a["ring"]["profile"], c["ring"]["profile"]);
}

#[test]
fn a_ring_from_its_stated_inputs_answers_and_a_bad_one_says_why() {
    let mut k = open();
    let ask = serde_json::json!({
        "key": "saturn", "host": {"mass_kg": 5.683e26, "radius_m": 6.0268e7},
        "sculptors": [{"name": "Pan", "a_m": 1.33584e8, "mass_kg": 4.95e15}],
        "ring": {"inner_m": 1.2234e8, "outer_m": 1.3678e8, "zones": [{"name": "A", "inner_m": 1.2234e8, "outer_m": 1.3678e8, "tau": 0.7, "albedo": 0.5}]}
    });
    let a = send(
        &mut k,
        serde_json::json!({"op":"ring_structure","ask":ask,"samples":64}),
    );
    assert_eq!(a["type"], "ring_structure_is", "{a}");
    assert!(a["ring"]["marks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["kind"] == "gap"));
    let bad = send(
        &mut k,
        serde_json::json!({"op":"ring_structure","ask":{"key":"x","host":{"mass_kg":0.0,"radius_m":1.0},"ring":{"inner_m":1.0,"outer_m":2.0}}}),
    );
    assert_eq!(bad["type"], "error", "{bad}");
    let neither = send(&mut k, serde_json::json!({"op":"ring_structure"}));
    assert_eq!(neither["code"], "bad_request", "{neither}");
}
