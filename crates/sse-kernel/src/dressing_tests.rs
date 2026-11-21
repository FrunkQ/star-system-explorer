//! TG12: dressing rules (tags design section 5.7).

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn pack(p: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
}
fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}
fn load(k: &mut crate::Kernel, packs: Vec<serde_json::Value>) {
    let r = send(k, serde_json::json!({"op": "load_stack", "packs": packs}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
}
fn set(k: &mut crate::Kernel, a: &str, f: &str, v: serde_json::Value) {
    let r = send(
        k,
        serde_json::json!({"op": "set_authored", "edit": {
            "address": a, "field": f, "value": v, "occurred_at_s": 0}}),
    );
    assert_eq!(r["type"], "authored", "{r}");
}
fn body(n: u32) -> String {
    format!("u:dress/sys:0/body:{n}")
}

/// A campaign with four worlds: only body 1 is a giant with a port within 2 AU.
fn campaign(dressing: serde_json::Value) -> crate::Kernel {
    let mut k = crate::Kernel::new();
    load(
        &mut k,
        vec![
            pack("tests/fixtures/packs/base-physics.json"),
            pack("tests/fixtures/packs/tarn-observe.json"),
            dressing,
        ],
    );
    let r = send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "dress", "seed": "tg12"}),
    );
    assert!(r["type"] != "error", "{r}");
    for (n, mass, au, port) in [
        (1, 1.9e27, 1.5, true),
        (2, 1.9e27, 3.0, true),
        (3, 6.0e24, 1.0, true),
        (4, 5.7e26, 1.0, false),
    ] {
        set(&mut k, &body(n), "mass_kg", serde_json::json!(mass));
        set(&mut k, &body(n), "orbit.a_au", serde_json::json!(au));
        if port {
            set(&mut k, &body(n), "tag.port", serde_json::json!(true));
        }
    }
    k
}
fn candidates(k: &mut crate::Kernel) -> Vec<serde_json::Value> {
    let r = send(k, serde_json::json!({"op": "dressing", "scope": "u:dress"}));
    assert_eq!(r["type"], "dressed", "{r}");
    let mut c = r["candidates"].as_array().unwrap().clone();
    c.sort_by_key(|x| x["id"].as_str().unwrap().to_string());
    c
}
fn gm_facts(k: &mut crate::Kernel, a: &str) -> serde_json::Value {
    let r = send(
        k,
        serde_json::json!({"op": "scene", "audience": "gm", "scope": a, "at_s": 0}),
    );
    assert_eq!(r["type"], "scene", "{r}");
    r["scene"]["subjects"]
        .as_array()
        .and_then(|s| s.iter().find(|x| x["address"] == a))
        .map(|x| x["facts"].clone())
        .unwrap_or_default()
}

#[test]
fn a_depot_goes_on_every_giant_with_a_port_within_2_au_and_nowhere_else() {
    let mut k = campaign(pack("tests/fixtures/packs/dressing-toy.json"));
    let all = candidates(&mut k);
    let depots: Vec<&serde_json::Value> = all
        .iter()
        .filter(|c| c["rule"] == "giant-depot.v1")
        .collect();
    assert_eq!(depots.len(), 1, "{all:?}");
    let d = depots[0];
    assert_eq!(d["world"], body(1));
    assert_eq!(d["template"], "fuel-depot");
    assert_eq!(d["label"], "Fuel depot");
    assert_eq!(d["where"], "low-orbit");
    assert_eq!(d["pack"], "test.dressing-toy");
    // Its "Why?" lists the facts read.
    let read: Vec<&str> = d["why"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["field"].as_str().unwrap())
        .collect();
    assert!(
        read.contains(&"mass_kg") && read.contains(&"orbit.a_au"),
        "{read:?}"
    );
    // Beacons roll per world and copy: some, not all twelve.
    let beacons = all.iter().filter(|c| c["rule"] == "beacons.v1").count();
    assert!((1..12).contains(&beacons), "{beacons} beacons");
    assert!(all
        .iter()
        .filter(|c| c["rule"] == "beacons.v1")
        .all(|c| c["why"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["field"] == "chance")));

    // Same seed on another device: the same candidates. Shuffled rules: the same.
    let mut other = campaign(pack("tests/fixtures/packs/dressing-toy.json"));
    assert_eq!(candidates(&mut other), all);
    let mut shuffled = pack("tests/fixtures/packs/dressing-toy.json");
    shuffled["provides"]["other"]["dressing"]["rules"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let mut third = campaign(shuffled);
    assert_eq!(candidates(&mut third), all);
}

#[test]
fn adopted_objects_stay_when_the_pack_goes_and_hold_offs_survive_a_reload() {
    let dressing = pack("tests/fixtures/packs/dressing-toy.json");
    let mut k = campaign(dressing.clone());
    let before: Vec<serde_json::Value> = (1..=4).map(|n| gm_facts(&mut k, &body(n))).collect();
    let depot = candidates(&mut k)
        .into_iter()
        .find(|c| c["rule"] == "giant-depot.v1")
        .unwrap();
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt_dressing", "candidate": depot["id"], "at_s": 0}),
    );
    assert_eq!(r["type"], "created", "{r}");
    let placed = r["address"].as_str().unwrap().to_string();
    assert!(
        placed.starts_with(&format!("{}/station:", body(1))),
        "{placed}"
    );
    // Adopting changes no fact of any world.
    let after: Vec<serde_json::Value> = (1..=4).map(|n| gm_facts(&mut k, &body(n))).collect();
    assert_eq!(before, after);
    // The object carries the template, the rule's tags and the dressing marks.
    let facts = gm_facts(&mut k, &placed);
    let has = |f: &str, v: serde_json::Value| {
        facts
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["field"] == f && x["value"] == v)
    };
    assert!(has("dressing.pack", "test.dressing-toy".into()), "{facts}");
    assert!(has("dressing.rule", "giant-depot.v1".into()));
    assert!(has("dressing.candidate", depot["id"].clone()));
    assert!(has("dressing.faction", "navy".into()));
    assert!(has("station.crew", 40.into()));
    // Adopted: no longer a candidate, and nothing under it is dressed.
    let left = candidates(&mut k);
    assert!(left.iter().all(|c| c["id"] != depot["id"]));
    assert!(left
        .iter()
        .all(|c| c["world"] == body(1) || !c["world"].as_str().unwrap().starts_with(&placed)));
    // Adopting twice is refused.
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt_dressing", "candidate": depot["id"], "at_s": 0}),
    );
    assert_eq!(r["code"], "no_candidate", "{r}");

    // Hold a beacon off; it survives a reload of the stack.
    let beacon = left
        .iter()
        .find(|c| c["rule"] == "beacons.v1")
        .unwrap()
        .clone();
    let r = send(
        &mut k,
        serde_json::json!({"op": "hold_dressing", "candidate": beacon["id"]}),
    );
    assert_eq!(r["type"], "dressing_held", "{r}");
    load(
        &mut k,
        vec![
            pack("tests/fixtures/packs/base-physics.json"),
            pack("tests/fixtures/packs/tarn-observe.json"),
            dressing.clone(),
        ],
    );
    assert!(candidates(&mut k).iter().all(|c| c["id"] != beacon["id"]));
    // Letting it back brings it back.
    send(
        &mut k,
        serde_json::json!({"op": "hold_dressing", "candidate": beacon["id"], "held": false}),
    );
    assert!(candidates(&mut k).iter().any(|c| c["id"] == beacon["id"]));

    // Edit the depot, then remove the pack: the object stays, and the swap
    // lists it as an orphan the GM touched.
    set(&mut k, &placed, "name", serde_json::json!("Halt Depot"));
    load(
        &mut k,
        vec![
            pack("tests/fixtures/packs/base-physics.json"),
            pack("tests/fixtures/packs/tarn-observe.json"),
        ],
    );
    let facts = gm_facts(&mut k, &placed);
    assert!(facts
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["field"] == "name" && x["value"] == "Halt Depot"));
    let r = send(
        &mut k,
        serde_json::json!({"op": "dressing", "scope": "u:dress"}),
    );
    assert_eq!(r["candidates"], serde_json::json!([]));
    assert_eq!(
        r["orphans"],
        serde_json::json!([{"address": placed, "pack": "test.dressing-toy",
            "rule": "giant-depot.v1", "touched": true}])
    );
}

#[test]
fn a_bad_dressing_rule_is_refused_at_load() {
    let mut p = pack("tests/fixtures/packs/dressing-toy.json");
    p["provides"]["other"]["dressing"]["rules"][0]["place"] = "no-such".into();
    p["provides"]["other"]["dressing"]["rules"][1]["chance"] = 2.0.into();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack",
            "packs": [pack("tests/fixtures/packs/base-physics.json"),
            pack("tests/fixtures/packs/tarn-observe.json"), p]}),
    );
    let text = r.to_string();
    assert!(text.contains("places template no-such"), "{text}");
    assert!(text.contains("chance 2 must be between 0 and 1"), "{text}");
}
