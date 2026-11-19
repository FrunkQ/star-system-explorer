//! J4 checks: the sensor ladder, ask until approved, policies, replay,
//! the pass limit with its chain, and load refusals.

use super::*;
use serde_json::json;

fn root() -> Address {
    "u:t".parse().unwrap()
}
fn body() -> Address {
    "u:t/sys:0/body:3".parse().unwrap()
}
fn ship() -> Address {
    "u:t/ship:calla".parse().unwrap()
}

fn set(ch: &mut Chronicle, a: &Address, at: i64, fields: &[(&str, Value)]) {
    let mut p = Payload::default();
    p.set.insert(
        a.clone(),
        fields
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    );
    authored(
        ch,
        a,
        UniverseTime::seconds(at),
        p,
        Publisher::new("gm", "edit", "1"),
    )
    .unwrap();
}

fn campaign() -> sse_observe::Campaign {
    serde_json::from_value(
        json!({"default_view": "long-range", "before": "long-range", "after": "scanned"}),
    )
    .unwrap()
}

fn registry() -> Registry {
    Registry::from_packs(&[(
        "test.rpg".into(),
        json!({"kept": [
            {"id": "politics.standing", "shape": "number", "range": [-10.0, 10.0], "default": 0, "tier": "secret", "writers": ["gm", "effect.consequence"], "describes": "how an actor regards the party"}
        ]}),
    )])
    .unwrap()
}

fn rules(v: J) -> Rules {
    Rules::from_packs(&[("test.rules".into(), "1.0.0".into(), v)]).unwrap()
}

fn ladder() -> J {
    json!({"rules": [{
        "id": "sensor-ladder",
        "when": {"on_condition": {
            "condition": {"if": "compare", "left": "pair.distance_m", "op": "le", "right": "observer.sensor.range_m"},
            "hold_s": 3600
        }},
        "scope": {"under": "u:t/sys:0", "pairs": {"observer_tag": "ship"}},
        "effects": [{"effect": "grant", "audience": "crew", "ladder": ["long-range", "scanned", "survey"]}],
        "explain": "a ship with the right sensor in range long enough learns more"
    }]})
}

struct World {
    universe: Chronicle,
    record: Chronicle,
}

fn world() -> World {
    let mut u = Chronicle::new();
    set(
        &mut u,
        &body(),
        0,
        &[
            ("name", text("Tarn")),
            ("pos.x_m", Value::Float(0.0)),
            ("pos.y_m", Value::Float(0.0)),
            ("pos.z_m", Value::Float(0.0)),
        ],
    );
    set(
        &mut u,
        &ship(),
        0,
        &[
            ("tag.ship", Value::Bool(true)),
            ("sensor.range_m", Value::Float(1.0e9)),
            ("pos.x_m", Value::Float(5.0e9)),
            ("pos.y_m", Value::Float(0.0)),
            ("pos.z_m", Value::Float(0.0)),
        ],
    );
    // The ship closes in at 1000 s, leaves at 7000 s, returns at 8000 s.
    for (t, x) in [(1000, 5.0e8), (7000, 4.0e9), (8000, 2.0e8)] {
        set(&mut u, &ship(), t, &[("pos.x_m", Value::Float(x))]);
    }
    World {
        universe: u,
        record: Chronicle::new(),
    }
}

fn gate<'a>(
    w: &'a mut World,
    reg: &'a Registry,
    c: &'a sse_observe::Campaign,
    r: &'a Address,
) -> Gate<'a> {
    Gate {
        universe: &mut w.universe,
        record: &mut w.record,
        registry: reg,
        campaign: Some(c),
        root: r,
        templates: &NO_TEMPLATES,
    }
}

static NO_TEMPLATES: templates::Templates = templates::Templates {
    templates: Vec::new(),
};

fn view(w: &World, at: i64) -> String {
    sse_observe::effective_view(
        &w.universe.state_at(UniverseTime::seconds(at)),
        &body(),
        "crew",
        &campaign(),
    )
}

#[test]
fn j4_sensor_ladder_moves_a_body_up_at_the_right_range_and_time() {
    let rs = rules(ladder());
    let (reg, c, r) = (registry(), campaign(), root());
    let mut w = world();
    let mut log = vec![];
    for t in [
        0, 1000, 2000, 4599, 4600, 6000, 7000, 8000, 11_599, 11_600, 20_000,
    ] {
        let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), t).unwrap();
        let fired: Vec<&Firing> = rep.passes.iter().flatten().collect();
        log.push((t, fired.len(), view(&w, t)));
        for f in &fired {
            assert_eq!(f.outcome, "applied", "{f:?}");
            assert_eq!(f.observer.as_deref(), Some("u:t/ship:calla"));
        }
    }
    println!("{log:?}");
    let at = |t: i64| log.iter().find(|x| x.0 == t).unwrap();
    assert_eq!(at(0).2, "long-range", "out of range");
    assert_eq!(at(2000).1, 0, "in range, not long enough");
    assert_eq!(at(4599).1, 0);
    assert_eq!(
        (at(4600).1, at(4600).2.as_str()),
        (1, "scanned"),
        "an hour in range: one rung up"
    );
    assert_eq!(at(6000).1, 0, "fires once per stretch");
    assert_eq!(at(7000).2, "scanned", "leaving keeps what was learnt");
    assert_eq!(at(11_599).1, 0);
    assert_eq!(
        (at(11_600).1, at(11_600).2.as_str()),
        (1, "survey"),
        "back in range for an hour: the next rung"
    );
    assert_eq!(at(20_000).1, 0);
    // Every rung move is a universe grant event published by the rule, citing nothing secret.
    let grants: Vec<&Event> = w
        .universe
        .events()
        .iter()
        .filter(|e| e.body.kind == ck::OBSERVE_GRANT)
        .collect();
    assert_eq!(grants.len(), 2);
    for g in &grants {
        assert_eq!(g.body.publisher.item, "sensor-ladder");
        assert!(g.body.causes.is_empty());
    }
    // The record explains each one: fired, then applied, citing the firing.
    let trail = trail(&w.record);
    assert!(trail.iter().filter(|e| e["kind"] == kinds::APPLIED).count() == 2);
    assert!(trail.iter().filter(|e| e["kind"] == kinds::ARMED).count() == 2);
    assert!(
        trail
            .iter()
            .filter(|e| e["kind"] == kinds::DISARMED)
            .count()
            == 1
    );
    // A step back in time is refused.
    assert!(rs.step(&mut gate(&mut w, &reg, &c, &r), 100).is_err());
}

fn consequence(approval: J) -> J {
    json!({"rules": [{
        "id": "consequence",
        "when": {"on_event": {"kind": "record.note", "tag": "insult"}},
        "effects": [
            {"effect": "set_kept", "kind": "politics.standing", "add": -1.0, "reason": "insulted"},
            {"effect": "add_note", "text": "the Vesh remember", "audience": "gm"}
        ],
        "approval": approval,
        "explain": "an insult costs standing"
    }]})
}

fn insult(w: &mut World, at: i64) -> ContentHash {
    sse_record::add_note(
        &mut w.record,
        &NoteDraft {
            subjects: vec![body()],
            text: "the crew peed on the statue".into(),
            tags: vec!["insult".into()],
            by: "crew-seat".into(),
            audience: Audience::Only(vec!["crew".into()]),
            tier: None,
            supersedes: None,
            narrative: false,
            arrives: None,
        },
        UniverseTime::seconds(at),
        &Publisher::new("crew-seat", "note", "1"),
    )
    .unwrap()
}

fn standing(w: &World, at: i64) -> Option<f64> {
    w.record
        .state_at(UniverseTime::seconds(at))
        .get(&body(), "politics.standing")
        .and_then(as_num)
}

#[test]
fn j4_an_ask_effect_changes_nothing_until_approved() {
    let rs = rules(consequence(json!({"policy": "ask"})));
    let (reg, c, r) = (registry(), campaign(), root());
    let mut w = world();
    rs.step(&mut gate(&mut w, &reg, &c, &r), 0).unwrap();
    insult(&mut w, 50);
    let u_before = w.universe.state_at(UniverseTime::seconds(i64::MAX)).hash();
    let notes_before =
        sse_record::notes_at(w.record.events(), UniverseTime::seconds(i64::MAX)).len();
    let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), 100).unwrap();
    assert_eq!(rep.passes[0][0].outcome, "proposed");
    assert!(rep.passes[0][0].cause.is_some());
    // Nothing changed: universe, kept values, notes.
    assert_eq!(
        w.universe.state_at(UniverseTime::seconds(i64::MAX)).hash(),
        u_before
    );
    assert_eq!(standing(&w, 1000), None);
    assert_eq!(
        sse_record::notes_at(w.record.events(), UniverseTime::seconds(i64::MAX)).len(),
        notes_before
    );
    let p = Rules::pending(&w.record);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].explain, "an insult costs standing");
    assert_eq!(p[0].effects.len(), 2);
    // Stepping on does not apply it either, nor fire again for the same note.
    rs.step(&mut gate(&mut w, &reg, &c, &r), 200).unwrap();
    assert_eq!(standing(&w, 1000), None);
    assert_eq!(Rules::pending(&w.record).len(), 1);
    // Yes applies both effects together, citing the decision.
    let written = rs
        .decide(&mut gate(&mut w, &reg, &c, &r), &p[0].id, true, Some(300))
        .unwrap();
    assert_eq!(written.len(), 2);
    assert_eq!(standing(&w, 1000), Some(-1.0));
    assert_eq!(standing(&w, 250), None, "applied at the answer's time");
    assert!(Rules::pending(&w.record).is_empty());
    assert!(
        rs.decide(&mut gate(&mut w, &reg, &c, &r), &p[0].id, true, None)
            .is_err(),
        "answered once"
    );
    // A second insult, refused: nothing changes.
    insult(&mut w, 400);
    rs.step(&mut gate(&mut w, &reg, &c, &r), 500).unwrap();
    let q = Rules::pending(&w.record);
    assert_eq!(q.len(), 1);
    assert!(rs
        .decide(&mut gate(&mut w, &reg, &c, &r), &q[0].id, false, None)
        .unwrap()
        .is_empty());
    assert_eq!(standing(&w, 1000), Some(-1.0));
    // The GM turns the rule to auto without editing the pack: the next insult applies at once.
    rs.set_policy(
        &mut gate(&mut w, &reg, &c, &r),
        "consequence",
        &Approval::Auto,
        500,
    )
    .unwrap();
    insult(&mut w, 600);
    let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), 700).unwrap();
    assert_eq!(rep.passes[0][0].outcome, "applied");
    assert_eq!(standing(&w, 1000), Some(-2.0));
    // ask_if: auto unless the condition holds (here: standing already at -2 or below).
    rs.set_policy(
        &mut gate(&mut w, &reg, &c, &r),
        "consequence",
        &serde_json::from_value(json!({"policy": "ask_if", "condition": {"if": "range", "field": "politics.standing", "max": -2.0}})).unwrap(),
        700,
    )
    .unwrap();
    insult(&mut w, 800);
    let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), 900).unwrap();
    assert_eq!(rep.passes[0][0].outcome, "proposed");
    // A clamped range: standing never leaves [-10, 10].
    rs.set_policy(
        &mut gate(&mut w, &reg, &c, &r),
        "consequence",
        &Approval::Auto,
        900,
    )
    .unwrap();
    for k in 0..12 {
        insult(&mut w, 1000 + k * 10);
    }
    rs.step(&mut gate(&mut w, &reg, &c, &r), 2000).unwrap();
    assert_eq!(standing(&w, 3000), Some(-10.0));
}

#[test]
fn j4_default_policies_and_load_refusals() {
    // set_pin asks by default; tags and notes are auto.
    let rs = rules(json!({"rules": [
        {"id": "pin", "when": {"on_time": {"every_s": 100}}, "scope": {"addresses": ["u:t/sys:0/body:3"]},
         "effects": [{"effect": "set_pin", "field": "atmosphere.note", "value": "smoky"}]},
        {"id": "tagger", "when": {"on_time": {"every_s": 100}}, "scope": {"addresses": ["u:t/sys:0/body:3"]},
         "effects": [{"effect": "tag", "tag": "visited"}]}
    ]}));
    let (reg, c, r) = (registry(), campaign(), root());
    let mut w = world();
    let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), 100).unwrap();
    let out: Vec<(&str, &str)> = rep.passes[0]
        .iter()
        .map(|f| (f.rule.as_str(), f.outcome.as_str()))
        .collect();
    // Ticks at 0 and 100 (no previous boundary: from from_s).
    assert_eq!(
        out,
        vec![
            ("pin", "proposed"),
            ("pin", "proposed"),
            ("tagger", "applied"),
            ("tagger", "applied")
        ]
    );
    let s = subjects(&w.universe, &w.record, &r, UniverseTime::seconds(100));
    assert!(s[&body()].tags.contains("visited"));
    assert!(!s[&body()].facts.contains_key("atmosphere.note"));
    // Refusals at load.
    let bad = |v: J| Rules::from_packs(&[("p".into(), "1".into(), v)]).unwrap_err();
    assert!(bad(json!({"rules": [{"id": "Bad", "when": {"on_time": {"every_s": 1}}, "effects": [{"effect": "tag", "tag": "x"}]}]}))[0].contains("ids are lowercase"));
    assert!(bad(json!({"rules": [{"id": "a", "when": {"on_time": {"every_s": 0}}, "effects": [{"effect": "tag", "tag": "x"}]}]}))[0].contains("every_s"));
    // J7: create loads, and a missing template is named against the stack's.
    let made = Rules::from_packs(&[("p".into(), "1".into(), json!({"rules": [{"id": "a", "when": {"on_time": {"every_s": 1}}, "effects": [{"effect": "create", "template": "ship"}]}]}))]).unwrap();
    assert!(made.check_templates(&NO_TEMPLATES)[0].contains("names template ship"));
    assert!(bad(
        json!({"rules": [{"id": "a", "when": {"on_time": {"every_s": 1}}, "effects": []}]})
    )[0]
    .contains("no effects"));
    let two = Rules::from_packs(&[
        ("p".into(), "1".into(), json!({"rules": [{"id": "a", "when": {"on_time": {"every_s": 1}}, "effects": [{"effect": "tag", "tag": "x"}]}]})),
        ("q".into(), "1".into(), json!({"rules": [{"id": "a", "when": {"on_time": {"every_s": 1}}, "effects": [{"effect": "tag", "tag": "x"}]}]})),
    ])
    .unwrap_err();
    assert!(two[0].contains("already declared by p"));
    // Checks against the stack: kinds, writers, views.
    let rs = rules(json!({"rules": [
        {"id": "nope", "when": {"on_time": {"every_s": 1}}, "effects": [{"effect": "set_kept", "kind": "politics.standing", "add": 1.0}]},
        {"id": "ghost", "when": {"on_time": {"every_s": 1}}, "effects": [{"effect": "set_kept", "kind": "trade.ghost", "add": 1.0}, {"effect": "grant", "audience": "crew", "view": "x-ray"}]}
    ]}));
    let issues = rs.check(&reg, &["long-range".into(), "scanned".into()]);
    println!("{issues:?}");
    assert_eq!(issues.len(), 3);
    assert!(issues
        .iter()
        .any(|i| i.contains("does not list writer effect.nope")));
    assert!(issues
        .iter()
        .any(|i| i.contains("no kept kind trade.ghost")));
    assert!(issues.iter().any(|i| i.contains("no data view x-ray")));
}

#[test]
fn j4_the_pass_limit_is_reported_with_its_chain() {
    let rs = rules(json!({"rules": [
        {"id": "start", "when": {"on_time": {"every_s": 1000}}, "effects": [{"effect": "send_intent", "kind": "ping"}]},
        {"id": "ping", "when": {"on_intent": {"kind": "ping"}}, "effects": [{"effect": "send_intent", "kind": "pong"}]},
        {"id": "pong", "when": {"on_intent": {"kind": "pong"}}, "effects": [{"effect": "send_intent", "kind": "ping"}]}
    ], "pass_limit": 5}));
    let (reg, c, r) = (registry(), campaign(), root());
    let mut w = world();
    let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), 0).unwrap();
    let chain = rep.limit.expect("limit hit");
    println!("{chain:?}");
    assert_eq!(chain.len(), 6);
    assert_eq!(chain[0], vec!["start"]);
    assert_eq!(chain[1], vec!["ping"]);
    assert_eq!(chain[2], vec!["pong"]);
    assert_eq!(rep.passes.len(), 5, "the limit pass is reported, not run");
    assert!(w
        .record
        .events()
        .iter()
        .any(|e| e.body.kind == kinds::LIMIT));
    // A chain that ends on its own is not cut.
    let rs = rules(json!({"rules": [
        {"id": "start", "when": {"on_time": {"every_s": 1000}}, "effects": [{"effect": "send_intent", "kind": "ping"}]},
        {"id": "ping", "when": {"on_intent": {"kind": "ping"}}, "effects": [{"effect": "add_note", "text": "pinged {subject}"}]}
    ]}));
    let mut w = world();
    let rep = rs.step(&mut gate(&mut w, &reg, &c, &r), 0).unwrap();
    assert!(rep.limit.is_none());
    assert_eq!(rep.passes.len(), 2);
    let n = sse_record::notes_at(w.record.events(), UniverseTime::seconds(10));
    assert_eq!(n[0].text, "pinged u:t");
}

/// The whole script: the same inputs and boundaries give the same chronicles, byte for byte.
fn script() -> (ContentHash, ContentHash, Vec<StepReport>) {
    let mut both = ladder();
    let c2 = consequence(json!({"policy": "auto"}));
    both["rules"]
        .as_array_mut()
        .unwrap()
        .extend(c2["rules"].as_array().unwrap().iter().cloned());
    let rs = rules(both);
    let (reg, c, r) = (registry(), campaign(), root());
    let mut w = world();
    let mut reps = vec![];
    for t in [0, 1000, 4600, 7000, 8000, 11_600] {
        if t == 7000 {
            insult(&mut w, 6500);
        }
        reps.push(rs.step(&mut gate(&mut w, &reg, &c, &r), t).unwrap());
    }
    (w.universe.head(), w.record.head(), reps)
}

#[test]
fn j4_a_run_replays_bit_identically() {
    let a = script();
    let b = script();
    assert_eq!(a, b);
    let firings: usize = a.2.iter().map(|r| r.passes.iter().flatten().count()).sum();
    assert_eq!(firings, 3, "two ladder steps and one consequence");
}
