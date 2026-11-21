//! TG1: tag declarations, the load-time check, campaign tags, exclusive
//! groups and the reserved economy names (tags design sections 4 and 7).

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
fn without(mut p: serde_json::Value, tag: &str) -> serde_json::Value {
    let ts = p["provides"]["other"]["tags"]["tags"]
        .as_array_mut()
        .unwrap();
    ts.retain(|t| t["id"] != tag);
    p
}
fn load(k: &mut crate::Kernel, packs: Vec<serde_json::Value>) -> serde_json::Value {
    send(k, serde_json::json!({"op": "load_stack", "packs": packs}))
}

#[test]
fn an_undeclared_tag_is_refused_where_it_is_named() {
    let mut k = crate::Kernel::new();
    let base = pack("tests/fixtures/packs/base-physics.json");
    let rules = pack("tests/fixtures/sift/rules-pack.json");
    let templates = pack("tests/fixtures/packs/rpg-templates.json");
    let triggers = pack("tests/fixtures/packs/rpg-rules.json");
    let views = pack("tests/fixtures/packs/tarn-observe.json");
    let record = pack("tests/fixtures/packs/rpg-record.json");
    let all = vec![
        base.clone(),
        record.clone(),
        rules.clone(),
        templates.clone(),
        triggers.clone(),
        views.clone(),
    ];
    let r = load(&mut k, all);
    assert_eq!(r["type"], "stack_loaded", "{r}");
    // One undeclared tag each in a rule, a template, a trigger and a view.
    let mut views = without(views, "ancient-ruins");
    views["provides"]["other"]["observe"]["views"][0]["tags"] =
        serde_json::json!({"mode": "listed", "list": ["lost-colony"]});
    let r = load(
        &mut k,
        vec![
            base,
            record,
            without(rules, "roche-danger"),
            without(templates, "wreck"),
            without(triggers, "insult"),
            without(views, "lost-colony"),
        ],
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    let issues: Vec<&str> = r["issues"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|i| i.as_str())
        .collect();
    let named: Vec<&&str> = issues.iter().filter(|i| i.contains("names tag")).collect();
    // Four readers, plus the fixture's sifter, which reads the rule's tag too.
    assert_eq!(named.len(), 5, "{issues:#?}");
    for (pack, reader, tag) in [
        ("test.sift-rules", "rule", "roche-danger"),
        ("test.sift-rules", "sifter", "roche-danger"),
        ("test.rpg-templates", "template", "wreck"),
        ("test.rpg-rules", "trigger", "insult"),
        ("test.tarn-observe", "view", "lost-colony"),
    ] {
        assert!(
            named
                .iter()
                .any(|m| m.starts_with(&format!("{pack}: {reader} "))
                    && m.contains(&format!("names tag {tag},"))),
            "{pack} {reader} {tag}: {named:#?}"
        );
    }
}

#[test]
fn the_traveller_pack_loads_clean() {
    let mut k = crate::Kernel::new();
    let r = load(
        &mut k,
        vec![
            pack("packs/base-physics/pack.json"),
            pack("packs/traveller/pack.json"),
        ],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let refs =
        crate::tags::references(&[
            serde_json::from_value(pack("packs/traveller/pack.json")).unwrap()
        ]);
    assert!(refs.len() > 80, "{}", refs.len());
}

#[test]
fn a_freely_typed_tag_joins_the_campaign_and_exclusive_groups_hold() {
    let mut k = crate::Kernel::new();
    let mut extra = pack("tests/fixtures/packs/rpg-rules.json");
    extra["id"] = "test.tag-groups".into();
    extra["provides"]["other"] = serde_json::json!({"tags": {
    "groups": [{"id": "allegiance", "label": "Allegiance", "exclusive": true}],
    "tags": [
        {"id": "allegiance.red", "label": "Red"},
        {"id": "allegiance.blue", "label": "Blue"},
        {"id": "mood.calm", "label": "Calm"}
    ]}});
    let r = load(
        &mut k,
        vec![pack("tests/fixtures/packs/base-physics.json"), extra],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "tags", "seed": "tags"}),
    );
    let set = |k: &mut crate::Kernel, a: &str, tag: &str, v: bool| {
        send(
            k,
            serde_json::json!({"op": "set_authored", "edit": {
            "address": a, "field": format!("tag.{tag}"), "value": v, "occurred_at_s": 0}}),
        )
    };
    // Declared by the stack: nothing joins the campaign.
    let r = set(&mut k, "u:tags/sys:1", "mood.calm", true);
    assert_eq!(r["type"], "authored", "{r}");
    assert!(r.get("declared_tag").is_none(), "{r}");
    // Freely typed: it joins the campaign's own tags, once.
    let r = set(&mut k, "u:tags/sys:1", "smugglers-haven", true);
    assert_eq!(r["declared_tag"], "smugglers-haven", "{r}");
    let r = set(&mut k, "u:tags/sys:2", "smugglers-haven", true);
    assert!(r.get("declared_tag").is_none(), "{r}");
    let c = send(&mut k, serde_json::json!({"op": "campaign_tags"}));
    let tags = c["pack"]["tags"]["tags"].as_array().unwrap();
    assert_eq!(tags.len(), 1, "{c}");
    assert_eq!(tags[0]["id"], "smugglers-haven");
    assert_eq!(tags[0]["source"], "authored");
    assert!(
        tags[0].get("tier").is_none(),
        "GM-only by default (ruling 4)"
    );
    // The campaign's tags travel with the save.
    let ex = send(&mut k, serde_json::json!({"op": "export_chronicle"}));
    let mut k2 = crate::Kernel::new();
    send(
        &mut k2,
        serde_json::json!({"op": "import_chronicle", "export": ex["export"]}),
    );
    let c2 = send(&mut k2, serde_json::json!({"op": "campaign_tags"}));
    assert_eq!(c2["pack"]["tags"]["tags"], c["pack"]["tags"]["tags"]);
    // Exclusive: a second allegiance is refused; the same one again, or
    // after the first is taken off, is fine.
    assert_eq!(
        set(&mut k, "u:tags/sys:1", "allegiance.red", true)["type"],
        "authored"
    );
    let r = set(&mut k, "u:tags/sys:1", "allegiance.blue", true);
    assert_eq!(r["code"], "exclusive_tag", "{r}");
    assert_eq!(
        set(&mut k, "u:tags/sys:1", "allegiance.red", true)["type"],
        "authored"
    );
    assert_eq!(
        set(&mut k, "u:tags/sys:2", "allegiance.blue", true)["type"],
        "authored"
    );
    assert_eq!(
        set(&mut k, "u:tags/sys:1", "allegiance.red", false)["type"],
        "authored"
    );
    assert_eq!(
        set(&mut k, "u:tags/sys:1", "allegiance.blue", true)["type"],
        "authored"
    );
}

#[test]
fn the_economy_names_are_reserved_and_declarations_must_agree() {
    let mut k = crate::Kernel::new();
    let base = pack("tests/fixtures/packs/base-physics.json");
    let mut p = pack("tests/fixtures/packs/rpg-rules.json");
    p["id"] = "test.tag-reserve".into();
    p["provides"]["other"] = serde_json::json!({
        "tags": {"groups": [{"id": "trade", "label": "Trade"}],
                 "tags": [{"id": "trade.luxury", "label": "Luxury"}]},
        "record": {"kept": [{"id": "economy.price", "shape": "number", "range": [0.0, 1.0], "default": 0, "tier": "scanned", "describes": "a price", "writers": ["gm"]}]}
    });
    let r = load(&mut k, vec![base.clone(), p.clone()]);
    assert_eq!(r["type"], "stack_rejected", "{r}");
    let text = r["issues"].to_string();
    assert!(text.contains("tag group trade is reserved"), "{text}");
    assert!(
        text.contains("tag trade.luxury is in group trade"),
        "{text}"
    );
    // The economy pack itself may use them.
    p["id"] = "sse.economy".into();
    let r = load(&mut k, vec![base.clone(), p]);
    assert_eq!(r["type"], "stack_loaded", "{r}");
    // Two packs may share a declaration word for word, not disagree.
    let a = |id: &str, label: &str| {
        let mut q = pack("tests/fixtures/packs/rpg-rules.json");
        q["id"] = id.into();
        q["provides"]["other"] =
            serde_json::json!({"tags": {"tags": [{"id": "ruin", "label": label}]}});
        q
    };
    let r = load(
        &mut k,
        vec![base.clone(), a("test.a", "Ruin"), a("test.b", "Ruin")],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = load(
        &mut k,
        vec![base, a("test.a", "Ruin"), a("test.b", "Ancient ruin")],
    );
    assert!(
        r["issues"]
            .to_string()
            .contains("already declared differently by test.a"),
        "{r}"
    );
}

#[test]
fn a_physics_flow_that_reads_a_tag_is_refused() {
    let mut k = crate::Kernel::new();
    let base = pack("tests/fixtures/packs/base-physics.json");
    let toy = pack("tests/fixtures/packs/base-toy.json");
    assert_eq!(
        load(&mut k, vec![base.clone(), toy.clone()])["type"],
        "stack_loaded"
    );
    for atom in ["world.has_tag", "world.read_kept"] {
        let mut bad = toy.clone();
        bad["provides"]["flows"][0]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"id": "cursed", "atom": atom, "config": {"tag": "cursed", "kind": "curse"}}));
        let r = load(&mut k, vec![base.clone(), bad]);
        assert_eq!(r["type"], "stack_rejected", "{r}");
        let text = r["issues"].to_string();
        assert!(
            text.contains(&format!(
                "sse.base-toy: flow toy.star-and-planet: node cursed reads {atom}"
            )),
            "{text}"
        );
    }
}

#[test]
fn fifty_tags_on_sol_change_no_resolved_fact() {
    let mut k = crate::Kernel::new();
    assert_eq!(
        load(&mut k, vec![pack("packs/base-time/pack.json")])["type"],
        "stack_loaded"
    );
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
    );
    let save = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "import_v3": save}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let sys = r["systems"][0].as_str().unwrap().to_string();
    let read = |k: &mut crate::Kernel| {
        let mut v = send(
            k,
            serde_json::json!({"op": "resolved_system", "system": sys}),
        );
        // The labels themselves are the only thing allowed to differ.
        for f in v["facts"].as_object_mut().unwrap().values_mut() {
            f.as_object_mut()
                .unwrap()
                .retain(|k, _| !k.starts_with("tag."));
        }
        for f in v["evidence"]
            .as_object_mut()
            .into_iter()
            .flat_map(|m| m.values_mut())
        {
            f.as_object_mut()
                .unwrap()
                .retain(|k, _| !k.starts_with("tag."));
        }
        v.as_object_mut().unwrap().remove("state_hash");
        v
    };
    let before = read(&mut k);
    let bodies: Vec<String> = before["facts"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert!(!bodies.is_empty());
    let tag = |k: &mut crate::Kernel, i: usize, on: bool| {
        let r = send(
            k,
            serde_json::json!({"op": "set_authored", "edit": {
            "address": bodies[i % bodies.len()], "field": format!("tag.label-{i}"), "value": on, "occurred_at_s": 0}}),
        );
        assert_eq!(r["type"], "authored", "{r}");
    };
    for i in 0..50 {
        tag(&mut k, i, true);
    }
    assert_eq!(read(&mut k), before, "50 tags on");
    for i in 0..50 {
        tag(&mut k, i, false);
    }
    assert_eq!(read(&mut k), before, "50 tags off");
}

#[test]
fn the_gm_overrules_a_rule_tag_in_scenes_conditions_and_sifters() {
    let mut k = crate::Kernel::new();
    let r = load(
        &mut k,
        vec![
            pack("tests/fixtures/packs/tarn-observe.json"),
            pack("tests/fixtures/sift/rules-pack.json"),
        ],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "kestrel", "seed": "k"}),
    );
    let body = "u:kestrel/sys:0/body:3";
    let set = |k: &mut crate::Kernel, a: &str, f: &str, v: serde_json::Value| {
        let r = send(
            k,
            serde_json::json!({"op": "set_authored", "edit": {
            "address": a, "field": f, "value": v, "occurred_at_s": 0}}),
        );
        assert_eq!(r["type"], "authored", "{r}");
    };
    for (f, v) in [
        ("surface_temp_k", serde_json::json!(290.0)),
        ("pressure_bar", serde_json::json!(1.0)),
        ("has_ocean", serde_json::json!(true)),
        ("o2_bar", serde_json::json!(0.2)),
        ("orbit_m", serde_json::json!(1.0e7)),
    ] {
        set(&mut k, body, f, v);
    }
    set(
        &mut k,
        "u:kestrel/sys:0",
        "roche_m",
        serde_json::json!(2.0e7),
    );
    let scene_tags = |k: &mut crate::Kernel| {
        let s = send(
            k,
            serde_json::json!({"op": "scene", "audience": "gm", "scope": "u:kestrel", "at_s": 0}),
        );
        s["scene"]["subjects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["address"] == body)
            .map(|x| x["tags"].clone())
            .unwrap_or_default()
    };
    let hits = |k: &mut crate::Kernel| {
        let r = send(
            k,
            serde_json::json!({"op": "sift", "sifters": ["moon-in-roche"], "at_s": 0}),
        );
        assert_eq!(r["type"], "sifted", "{r}");
        r["hits"].as_array().unwrap().len()
    };
    // The rules' answer: habitable, garden (which builds on habitable) and
    // inside the Roche zone, so the sifter fires.
    let derived = scene_tags(&mut k);
    assert_eq!(
        derived,
        serde_json::json!(["garden", "habitable", "roche-danger"])
    );
    assert_eq!(hits(&mut k), 1);
    // Held off: gone from the scene, from the rule that reads it (garden)
    // and from the sifter.
    set(&mut k, body, "tag.habitable", serde_json::json!(false));
    set(&mut k, body, "tag.roche-danger", serde_json::json!(false));
    assert_eq!(scene_tags(&mut k), serde_json::json!([]));
    assert_eq!(hits(&mut k), 0);
    // Forced on where no rule holds: shown, with the authored mark.
    set(&mut k, body, "surface_temp_k", serde_json::json!(500.0));
    set(&mut k, body, "tag.habitable", serde_json::json!(true));
    assert_eq!(
        scene_tags(&mut k),
        serde_json::json!(["garden", "habitable"])
    );
    let u = k.universe.as_ref().unwrap();
    let state = u
        .chronicle
        .state_at(sse_chronicle::UniverseTime::seconds(0));
    let a: sse_core::Address = body.parse().unwrap();
    let f = &state.facts[&a];
    let subject = sse_sift::Subject {
        address: Some(a.clone()),
        facts: f.clone(),
        parent: state.facts[&a.parent().unwrap()].clone(),
        ..Default::default()
    };
    let t =
        k.observe
            .rules
            .apply_authored(&subject, &Default::default(), &sse_sift::authored_tags(f));
    assert_eq!(t["habitable"].rule, sse_sift::AUTHORED);
    assert_eq!(t["habitable"].evidence[0].field, "tag.habitable");
    assert_eq!(
        t["garden"].rule, "garden.v1",
        "a rule still builds on a forced tag"
    );
    // Taking the authored facts away gives the rules' answer back.
    set(&mut k, body, "surface_temp_k", serde_json::json!(290.0));
    set(&mut k, body, "tag.habitable", serde_json::Value::Null);
    set(&mut k, body, "tag.roche-danger", serde_json::Value::Null);
    assert_eq!(scene_tags(&mut k), derived);
    assert_eq!(hits(&mut k), 1);
}
