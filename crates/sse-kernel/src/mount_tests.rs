//! CM1: mounts, release and capture (compound ships design section 4).

use serde_json::{json, Value};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn send(k: &mut crate::Kernel, v: Value) -> Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn kernel() -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        json!({"op":"load_stack","packs":[packs["rpg-templates"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    k
}

/// A construct with a name and facts, made at 100 s.
fn make(k: &mut crate::Kernel, name: &str, facts: Value) -> String {
    let c = send(
        k,
        json!({"op":"create","template":"freighter","under":"u:kestrel/sys:0","name":name,"at_s":100}),
    );
    assert_eq!(c["type"], "created", "{c}");
    let a = c["address"].as_str().unwrap().to_string();
    let edits: Vec<Value> = facts
        .as_object()
        .unwrap()
        .iter()
        .map(|(f, v)| json!({"address":a,"field":f,"value":v}))
        .collect();
    if !edits.is_empty() {
        let r = send(
            k,
            json!({"op":"set_authored_batch","occurred_at_s":100,"edits":edits}),
        );
        assert_eq!(r["type"], "authored", "{r}");
    }
    a
}

fn mount(k: &mut crate::Kernel, a: &str, on: &str, place: &str, more: Value, at: i64) -> Value {
    let mut q = json!({"op":"mount_construct","address":a,"on":on,"place":place,"at_s":at});
    for (f, v) in more.as_object().unwrap() {
        q[f] = v.clone();
    }
    send(k, q)
}

fn release(k: &mut crate::Kernel, a: &str, at: i64) -> Value {
    send(k, json!({"op":"release_construct","address":a,"at_s":at}))
}

/// Apollo in the design's shape (section 4.3): the CSM keeps the mission's
/// name and is the root; the S-IVB is behind it with Eagle in its adapter.
fn apollo() -> (crate::Kernel, String, String, String) {
    let mut k = kernel();
    let csm = make(
        &mut k,
        "Columbia",
        json!({"construct.mass_kg": 28800.0,
        "port.nose.place": "front", "port.nose.size_m": 0.8, "port.nose.lines": ["people","power","air"]}),
    );
    let sivb = make(
        &mut k,
        "S-IVB",
        json!({"construct.mass_kg": 119900.0,
        "part.sla.bay_length_m": 8.5, "part.sla.bay_radius_m": 3.3,
        "part.sla.bay_door_m": 6.6, "part.sla.bay_holds_kg": 16000.0}),
    );
    let eagle = make(
        &mut k,
        "Eagle",
        json!({"construct.mass_kg": 15100.0,
        "construct.length_m": 7.0, "construct.radius_m": 2.1,
        "port.top.size_m": 0.8}),
    );
    let r = mount(&mut k, &sivb, &csm, "behind", json!({"push_m_s": 0.3}), 200);
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert_eq!(r["mount"]["bond"], "fixed", "{r}");
    let r = mount(&mut k, &eagle, &sivb, "inside", json!({}), 200);
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert_eq!(r["mount"]["bay"], "sla", "the only bay is used: {r}");
    assert_eq!(r["mount"]["bond"], "rest", "{r}");
    assert_eq!(r["notes"], json!([]), "{r}");
    (k, csm, sivb, eagle)
}

#[test]
fn a_compound_is_the_tree_of_mounts_with_the_mission_at_its_root() {
    let (mut k, csm, sivb, eagle) = apollo();
    let r = release(&mut k, &eagle, 150);
    assert_eq!(r["code"], "release_refused", "not yet carried: {r}");
    // Mounting from either end gives the same compound, rooted at Columbia.
    let r = mount(&mut k, &csm, &eagle, "front", json!({}), 300);
    assert!(
        r["message"].as_str().unwrap().contains("already carries"),
        "no loops: {r}"
    );
    // Transposition: Columbia releases from the stack and captures Eagle at its nose.
    let r = release(&mut k, &sivb, 1000);
    assert_eq!(r["type"], "construct_released", "{r}");
    assert_eq!(r["mount"]["push_m_s"], 0.3, "{r}");
    assert_eq!(r["compound"]["root"], csm.as_str(), "{r}");
    assert_eq!(r["compound"]["mass_kg"], 28800.0, "{r}");
    let r = release(&mut k, &eagle, 1500);
    assert_eq!(r["type"], "construct_released", "{r}");
    assert_eq!(
        r["notes"][0], "no separation push given; the flight log uses the pack's",
        "{r}"
    );
    let r = mount(
        &mut k,
        &eagle,
        &csm,
        "front",
        json!({"port":"nose","own_port":"top","lines":["people","air","people"]}),
        2000,
    );
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert_eq!(r["mount"]["lines"], json!(["air", "people"]), "{r}");
    assert_eq!(r["compound"]["root"], csm.as_str());
    assert_eq!(r["compound"]["mass_kg"], 28800.0 + 15100.0);
    // A second construct cannot take an occupied port.
    let probe = make(&mut k, "Probe", json!({"construct.mass_kg": 10.0}));
    let r = mount(&mut k, &probe, &csm, "front", json!({"port":"nose"}), 2100);
    assert!(
        r["message"].as_str().unwrap().contains("already holds"),
        "{r}"
    );
}

#[test]
fn release_and_capture_move_nothing() {
    let (mut k, csm, sivb, eagle) = apollo();
    let total = |k: &mut crate::Kernel, at: i64| -> f64 {
        let u = k.universe().ok().unwrap();
        let st = u
            .chronicle
            .state_at(sse_chronicle::UniverseTime::seconds(at));
        [&csm, &sivb, &eagle]
            .iter()
            .map(|a| {
                let a: sse_core::Address = a.parse().unwrap();
                match st.facts[&a].get("construct.mass_kg") {
                    Some(sse_core::cbor::Value::Float(x)) => *x,
                    Some(sse_core::cbor::Value::Int(x)) => *x as f64,
                    other => panic!("{other:?}"),
                }
            })
            .sum()
    };
    let start = total(&mut k, 150);
    assert_eq!(release(&mut k, &sivb, 1000)["type"], "construct_released");
    assert_eq!(release(&mut k, &eagle, 1500)["type"], "construct_released");
    assert_eq!(
        mount(&mut k, &eagle, &csm, "front", json!({}), 2000)["type"],
        "construct_mounted"
    );
    for at in [150, 250, 1000, 1500, 2000, 3000] {
        assert_eq!(total(&mut k, at), start, "at {at} s");
    }
}

#[test]
fn bays_ports_and_places_are_checked_by_name() {
    let (mut k, csm, sivb, _) = apollo();
    let big = make(
        &mut k,
        "Big",
        json!({"construct.mass_kg": 2000.0, "construct.radius_m": 3.4}),
    );
    let heavy = make(&mut k, "Heavy", json!({"construct.mass_kg": 2000.0}));
    let bare = make(&mut k, "Bare", json!({}));
    let roomy = make(
        &mut k,
        "Roomy",
        json!({"part.a.bay_length_m": 5.0, "part.b.bay_length_m": 5.0}),
    );
    for (a, on, place, more, says) in [
        (&big, &sivb, "inside", json!({}), "wide (radius)"),
        (&heavy, &sivb, "inside", json!({}), "holds 16000 kg"),
        (
            &bare,
            &sivb,
            "inside",
            json!({"bay":"hold"}),
            "has no bay \"hold\"; bays: sla",
        ),
        (&bare, &roomy, "inside", json!({}), "name which holds"),
        (&bare, &csm, "behind", json!({}), "already holds"),
        (
            &bare,
            &csm,
            "front",
            json!({"port":"tail"}),
            "no port \"tail\"; ports: nose",
        ),
        (
            &bare,
            &csm,
            "hull",
            json!({"port":"nose"}),
            "is at its front, not its hull",
        ),
        (
            &bare,
            &csm,
            "front",
            json!({"port":"nose","lines":["propellant"]}),
            "passes no propellant",
        ),
        (
            &bare,
            &csm,
            "front",
            json!({"along_m": 2.0}),
            "place a hull or outside mount",
        ),
        (
            &bare,
            &csm,
            "hull",
            json!({"stand_off_m": 2.0}),
            "outside mount",
        ),
        (
            &bare,
            &csm,
            "hull",
            json!({"lines":["water"]}),
            "is not one of",
        ),
        (
            &bare,
            &csm,
            "hull",
            json!({"bay":"sla"}),
            "a bay holds an inside mount",
        ),
        (&bare, &bare, "hull", json!({}), "cannot carry itself"),
    ] {
        let r = mount(&mut k, a, on, place, more, 300);
        assert_eq!(r["code"], "mount_refused", "{place}: {r}");
        assert!(r["message"].as_str().unwrap().contains(says), "{says}: {r}");
    }
    let r = send(
        &mut k,
        json!({"op":"mount_construct","address":bare,"on":csm,"place":"aft","at_s":300}),
    );
    assert_eq!(r["code"], "bad_request", "{r}");
    // Port sizes must agree.
    let wide = make(&mut k, "Wide", json!({"port.p.size_m": 1.3}));
    let r = mount(
        &mut k,
        &wide,
        &csm,
        "front",
        json!({"port":"nose","own_port":"p"}),
        300,
    );
    assert!(r["message"].as_str().unwrap().contains("do not fit"), "{r}");
    // Strap-ons on the hull at stations round it, a sensor held clear on a boom.
    for (i, clock) in [0.0, 120.0, 240.0].iter().enumerate() {
        let b = make(
            &mut k,
            &format!("Booster {i}"),
            json!({"construct.mass_kg": 100.0}),
        );
        let r = mount(
            &mut k,
            &b,
            &sivb,
            "hull",
            json!({"along_m": 10.0, "clock_deg": clock, "bond":"fixed"}),
            300,
        );
        assert_eq!(r["type"], "construct_mounted", "{r}");
    }
    let r = mount(
        &mut k,
        &bare,
        &csm,
        "outside",
        json!({"stand_off_m": 7.5}),
        300,
    );
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert_eq!(r["mount"]["bond"], "latch");
    assert_eq!(r["compound"]["members"].as_array().unwrap().len(), 7, "{r}");
    assert_eq!(r["compound"]["mass_unknown"], json!([bare]), "{r}");
    // Inside a carrier with no bay part: made, with a note.
    let r = mount(&mut k, &heavy, &csm, "inside", json!({}), 300);
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert!(
        r["notes"][0].as_str().unwrap().contains("no bay part"),
        "{r}"
    );
}

#[test]
fn a_dock_reads_as_a_latch_on_the_hull() {
    let mut k = kernel();
    let station = make(&mut k, "Station", json!({"construct.mass_kg": 400000.0}));
    let soyuz = make(&mut k, "Soyuz", json!({"construct.mass_kg": 7200.0}));
    let r = send(
        &mut k,
        json!({"op":"join_constructs","address":station,"with":soyuz,"how":"dock","at_s":200}),
    );
    assert_eq!(r["type"], "constructs_joined", "{r}");
    let r = mount(&mut k, &soyuz, &station, "hull", json!({}), 300);
    assert!(
        r["message"].as_str().unwrap().contains("already docked to"),
        "{r}"
    );
    let r = send(
        &mut k,
        json!({"op":"release_construct","address":soyuz,"from":station,"at_s":400}),
    );
    assert_eq!(r["type"], "construct_released", "{r}");
    assert_eq!(r["mount"]["place"], "hull");
    assert_eq!(r["mount"]["docked"], true);
    // Mounted, it cannot dock; carrying, it cannot be merged away.
    assert_eq!(
        mount(&mut k, &soyuz, &station, "hull", json!({}), 500)["type"],
        "construct_mounted"
    );
    let other = make(&mut k, "Progress", json!({}));
    let r = send(
        &mut k,
        json!({"op":"join_constructs","address":other,"with":soyuz,"how":"dock","at_s":600}),
    );
    assert!(
        r["message"].as_str().unwrap().contains("already mounted"),
        "{r}"
    );
    let r = send(
        &mut k,
        json!({"op":"join_constructs","address":other,"with":station,"how":"merge","at_s":600}),
    );
    assert!(
        r["message"].as_str().unwrap().contains("is carried by"),
        "{r}"
    );
    // Releasing from the wrong carrier is refused by name.
    let r = send(
        &mut k,
        json!({"op":"release_construct","address":soyuz,"from":other,"at_s":700}),
    );
    assert!(r["message"].as_str().unwrap().contains("not u:"), "{r}");
    // Mounts are written in date order.
    let r = release(&mut k, &soyuz, 450);
    assert!(
        r["message"].as_str().unwrap().contains("not carried")
            || r["message"].as_str().unwrap().contains("later"),
        "{r}"
    );
    let r = mount(&mut k, &soyuz, &other, "hull", json!({}), 450);
    assert!(r["message"].as_str().unwrap().contains("later"), "{r}");
}

/// Ruling 14A: a land structure is a construct like any other; a rover
/// leaving its garage is a release, and coming home a capture.
#[test]
fn mounts_work_on_the_ground() {
    let mut k = kernel();
    let base = make(
        &mut k,
        "Tranquility Base",
        json!({"construct.mass_kg": 50000.0,
        "part.garage.bay_length_m": 6.0, "part.garage.bay_radius_m": 2.0}),
    );
    let rover = make(
        &mut k,
        "Rover",
        json!({"construct.mass_kg": 210.0,
        "construct.length_m": 3.1, "construct.radius_m": 1.1}),
    );
    let r = mount(&mut k, &rover, &base, "inside", json!({}), 200);
    assert_eq!(r["type"], "construct_mounted", "{r}");
    assert!(
        r["notes"].as_array().unwrap().is_empty(),
        "sizes known, no capacity given: {r}"
    );
    assert_eq!(release(&mut k, &rover, 300)["type"], "construct_released");
    assert_eq!(
        mount(
            &mut k,
            &rover,
            &base,
            "inside",
            json!({"bay":"garage"}),
            400
        )["type"],
        "construct_mounted"
    );
}
