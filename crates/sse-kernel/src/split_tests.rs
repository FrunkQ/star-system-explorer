//! RS4, construct half: split and join of constructs at a dated event.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

/// The facts at `a` at `at_s`, as field -> value.
fn facts(k: &mut crate::Kernel, a: &str, at_s: i64) -> serde_json::Map<String, serde_json::Value> {
    use sse_core::cbor::Value;
    let u = k.universe().ok().unwrap();
    let st = u
        .chronicle
        .state_at(sse_chronicle::UniverseTime::seconds(at_s));
    let a: sse_core::Address = a.parse().unwrap();
    st.facts
        .get(&a)
        .map(|row| {
            row.iter()
                .filter_map(|(k, v)| {
                    let j = match v {
                        Value::Float(x) => serde_json::json!(x),
                        Value::Int(i) => serde_json::json!(*i as f64),
                        Value::Text(t) => serde_json::json!(t),
                        Value::Bool(b) => serde_json::json!(b),
                        _ => return None,
                    };
                    Some((k.clone(), j))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A stack in the Apollo shape: Columbia with its service module and Eagle
/// (two parts), made from a template, with a mass.
fn apollo() -> (crate::Kernel, String) {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[packs["rpg-templates"]]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"kestrel","seed":"k"}),
    );
    let c = send(
        &mut k,
        serde_json::json!({"op":"create","template":"freighter","under":"u:kestrel/sys:0",
            "name":"Columbia","at_s":100}),
    );
    assert_eq!(c["type"], "created", "{c}");
    let ship = c["address"].as_str().unwrap().to_string();
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":100,"edits":[
            {"address":ship,"field":"construct.mass_kg","value":45000.0},
            {"address":ship,"field":"part.eagle.mass_kg","value":15000.0},
            {"address":ship,"field":"part.eagle.label","value":"Lunar module"},
            {"address":ship,"field":"part.sm.mass_kg","value":24000.0}
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    (k, ship)
}

#[test]
fn a_split_moves_the_part_and_its_mass_at_its_date() {
    let (mut k, ship) = apollo();
    let r = send(
        &mut k,
        serde_json::json!({"op":"split_construct","address":ship,"key":"eagle",
            "name":"Eagle","parts":["eagle"],"at_s":500}),
    );
    assert_eq!(r["type"], "construct_split", "{r}");
    let eagle = r["address"].as_str().unwrap().to_string();
    assert_eq!(eagle, "u:kestrel/sys:0/ship:eagle");
    let e = facts(&mut k, &eagle, 600);
    assert_eq!(e["name"], "Eagle");
    assert_eq!(e["part.eagle.label"], "Lunar module");
    assert_eq!(e["construct.mass_kg"], 15000.0);
    assert_eq!(e["split.from"], ship.as_str());
    let c = facts(&mut k, &ship, 600);
    assert_eq!(c["construct.mass_kg"], 30000.0);
    assert!(!c.contains_key("part.eagle.mass_kg"), "{c:?}");
    assert_eq!(c["part.sm.mass_kg"], 24000.0);
    // Before the date nothing has moved.
    let before = facts(&mut k, &ship, 400);
    assert_eq!(before["construct.mass_kg"], 45000.0);
    assert!(facts(&mut k, &eagle, 400).is_empty());
    // Unknown part, too much mass, a taken key: refused by name.
    for (ask, says) in [
        (
            serde_json::json!({"parts":["lm"]}),
            "no part \"lm\"; parts: sm",
        ),
        (serde_json::json!({"mass_kg": 1.0e6}), "the split takes"),
        (
            serde_json::json!({"key":"eagle","mass_kg": 1.0}),
            "already exists",
        ),
        (serde_json::json!({}), "name parts to move"),
    ] {
        let mut q = serde_json::json!({"op":"split_construct","address":ship,"at_s":700});
        for (f, v) in ask.as_object().unwrap() {
            q[f] = v.clone();
        }
        let r = send(&mut k, q);
        assert_eq!(r["code"], "split_refused", "{r}");
        assert!(r["message"].as_str().unwrap().contains(says), "{r}");
    }
    // A split-off piece can be retired like a made one.
    let r = send(
        &mut k,
        serde_json::json!({"op":"retire","address":eagle,"reason":"descent stage left on the Moon","at_s":900}),
    );
    assert_eq!(r["type"], "retired", "{r}");
}

#[test]
fn dock_undock_and_merge() {
    let (mut k, ship) = apollo();
    let r = send(
        &mut k,
        serde_json::json!({"op":"split_construct","address":ship,"parts":["eagle"],"at_s":500}),
    );
    let eagle = r["address"].as_str().unwrap().to_string();
    assert_eq!(facts(&mut k, &eagle, 600)["name"], "Columbia eagle");
    let join = |k: &mut crate::Kernel, how: &str, at: i64| {
        send(
            k,
            serde_json::json!({"op":"join_constructs","address":ship,"with":eagle,"how":how,"at_s":at}),
        )
    };
    let r = join(&mut k, "dock", 600);
    assert_eq!(r["type"], "constructs_joined", "{r}");
    assert_eq!(r["how"], "dock");
    assert_eq!(facts(&mut k, &eagle, 650)["docked.to"], ship.as_str());
    let r = join(&mut k, "dock", 660);
    assert!(
        r["message"].as_str().unwrap().contains("already docked"),
        "{r}"
    );
    let r = join(&mut k, "undock", 700);
    assert_eq!(r["type"], "constructs_joined", "{r}");
    assert!(!facts(&mut k, &eagle, 750).contains_key("docked.to"));
    let r = join(&mut k, "undock", 760);
    assert!(
        r["message"].as_str().unwrap().contains("is not docked"),
        "{r}"
    );
    // Merge: the part and mass come back; the other is retired, naming where it went.
    let r = join(&mut k, "merge", 800);
    assert_eq!(r["type"], "constructs_joined", "{r}");
    let c = facts(&mut k, &ship, 850);
    assert_eq!(c["construct.mass_kg"], 45000.0);
    assert_eq!(c["part.eagle.mass_kg"], 15000.0);
    let e = facts(&mut k, &eagle, 850);
    assert_eq!(e["retired"], true);
    assert_eq!(e["joined.into"], ship.as_str());
    let r = join(&mut k, "dock", 900);
    assert!(r["message"].as_str().unwrap().contains("retired"), "{r}");
    // History keeps it all.
    let h = send(&mut k, serde_json::json!({"op":"history","address":ship}));
    let kinds: Vec<&str> = h["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["kind"].as_str())
        .collect();
    assert!(
        kinds.contains(&"construct.split") && kinds.contains(&"construct.join"),
        "{h}"
    );
}
