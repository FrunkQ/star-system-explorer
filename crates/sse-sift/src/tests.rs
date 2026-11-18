//! K8 acceptance: a rule pack produces tags with hysteresis; classification
//! with working and stickiness; sifter hits carry evidence and measured
//! significance; golden digest.

use crate::*;
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::{Address, ContentHash, Seed};
use sse_pack::PackManifest;
use sse_resolve::{resolve_recorded, ChildRef, Envelope};
use std::collections::BTreeMap;

fn pack() -> PackManifest {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/sift/rules-pack.json"
    ))
    .unwrap();
    PackManifest::from_json(&s).unwrap()
}
fn content() -> SiftContent {
    load(&[pack()]).unwrap()
}
fn facts(kv: &[(&str, Value)]) -> BTreeMap<String, Value> {
    kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}
fn body(n: u64) -> Address {
    "u:k8/sys:0"
        .parse::<Address>()
        .unwrap()
        .child("body", n)
        .unwrap()
}
fn reading(t: i64, temp: f64) -> Subject {
    Subject {
        address: Some(body(1)),
        facts: facts(&[
            ("surface_temp_k", Value::Float(temp)),
            ("pressure_bar", Value::Float(1.2)),
            ("has_ocean", Value::Bool(true)),
            ("o2_bar", Value::Float(0.2)),
            ("ocean_fraction", Value::Float(0.7)),
        ]),
        now_s: t,
        ..Default::default()
    }
}

#[test]
fn rule_pack_loads_and_rejects_duplicates_readably() {
    let c = content();
    assert_eq!(c.rules.rules.len(), 4);
    assert_eq!(c.atlas.classes.len(), 6);
    assert_eq!(c.sifters.len(), 5);
    let mut other = pack();
    other.id = "test.rival".into();
    let errs = load(&[pack(), other]).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e == "test.rival: rule habitable.v1 is already defined by test.sift-rules"),
        "{errs:?}"
    );
}

/// Acceptance: tags with hysteresis. A world whose temperature wobbles
/// around the 310 K edge keeps its tag until it passes the 5 K margin.
#[test]
fn tags_have_hysteresis() {
    let c = content();
    let temps = [
        300.0, 309.0, 311.0, 309.5, 313.0, 314.9, 316.0, 312.0, 309.0,
    ];
    let series: Vec<Subject> = temps
        .iter()
        .enumerate()
        .map(|(i, t)| reading(i as i64 * 1000, *t))
        .collect();
    let (held, changes) = c.rules.track(&series);
    let hab: Vec<(i64, bool)> = changes
        .iter()
        .filter(|c| c.tag == "habitable")
        .map(|c| (c.at_s, c.gained))
        .collect();
    assert_eq!(hab, vec![(0, true), (6000, false), (8000, true)]);
    // Without hysteresis the same series flickers.
    let mut flat = c.rules.clone();
    for r in &mut flat.rules {
        r.stay = Some(r.when.clone());
    }
    let (_, flicker) = flat.track(&series);
    assert!(flicker.iter().filter(|c| c.tag == "habitable").count() >= 5);
    // A rule built on another's tag follows it; evidence names the facts read.
    assert!(held.contains_key("garden"));
    let lost = changes
        .iter()
        .find(|c| c.tag == "habitable" && !c.gained)
        .unwrap();
    assert!(lost
        .evidence
        .iter()
        .any(|r| r.field == "surface_temp_k" && r.value == Value::Float(316.0)));
}

#[test]
fn compare_reads_the_parent() {
    let c = content();
    let mut s = Subject::new(facts(&[("orbit_m", Value::Float(1.1e7))]));
    s.address = Some(body(2));
    s.parent = facts(&[("roche_m", Value::Float(1.0e7))]);
    let tags = c.rules.apply(&s, &BTreeMap::new());
    let t = &tags["roche-danger"];
    assert_eq!(
        t.evidence
            .iter()
            .map(|r| r.field.as_str())
            .collect::<Vec<_>>(),
        vec!["orbit_m", "parent.roche_m"]
    );
    // applies_to: a system never gets body tags.
    s.address = Some("u:k8/sys:0".parse().unwrap());
    assert!(c.rules.apply(&s, &BTreeMap::new()).is_empty());
}

#[test]
fn classification_deepest_fit_with_working() {
    let a = content().atlas;
    let mut s = reading(0, 290.0);
    s.facts.insert("tidally_locked".into(), Value::Bool(true));
    let l = a.label("test", &s, None);
    assert_eq!(l.base.as_deref(), Some("world/temperate/ocean"));
    assert_eq!(l.fit, 1.0);
    assert_eq!(l.modifiers, vec!["locked".to_string()]);
    // A near miss says what would have to change.
    s.facts.insert("ocean_fraction".into(), Value::Float(0.2));
    let l = a.label("test", &s, None);
    assert_eq!(l.base.as_deref(), Some("world/temperate"));
    let ocean = l
        .nearest
        .iter()
        .find(|f| f.class == "world/temperate/ocean")
        .unwrap();
    let lim = ocean.limiting.as_ref().unwrap();
    assert_eq!(lim.axis, "ocean");
    assert!((lim.change_to_core.unwrap() - 0.3).abs() < 1e-12);
    // No class fits: unclassified, with nearest listed.
    let none = a.label("test", &Subject::new(BTreeMap::new()), None);
    assert_eq!(none.base, None);
    assert!(!none.nearest.is_empty());
}

#[test]
fn labels_are_sticky_and_changes_are_dated() {
    let a = content().atlas;
    let mut series: Vec<Subject> = [300.0, 309.0, 312.0, 312.5, 330.0]
        .iter()
        .enumerate()
        .map(|(i, t)| reading(i as i64, *t))
        .collect();
    for s in &mut series {
        s.facts.insert("ocean_fraction".into(), Value::Float(0.0));
    }
    let changes = a.track("test", &series);
    // At 312 K "hot" fits 1.0 but temperate still fits 0.93: kept. At 330 K it flips.
    assert_eq!(
        changes,
        vec![
            (0, None, Some("world/temperate".into())),
            (4, Some("world/temperate".into()), Some("world/hot".into()))
        ]
    );
    let l = a.label("test", &series[2], Some("world/temperate"));
    assert!(l.kept_by_stickiness);
    // Bad atlases are rejected.
    let mut bad = a.clone();
    bad.classes[1].region = vec![Bound::Numeric {
        axis: "temp".into(),
        core: (260.0, 310.0),
        fringe: (270.0, 300.0),
    }];
    assert!(bad
        .validate()
        .unwrap_err()
        .0
        .contains("fringe must enclose"));
}

fn seed() -> Seed {
    Seed::from_text("k8")
}
fn patch() -> Envelope {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/resolve/patch.json"
    ))
    .unwrap();
    serde_json::from_str(&s).unwrap()
}

fn set(
    c: &mut Chronicle,
    kind: &str,
    a: &Address,
    f: BTreeMap<String, Value>,
    at: UniverseTime,
) -> ContentHash {
    let mut s = BTreeMap::new();
    s.insert(a.clone(), f);
    let e = Event::new(EventBody {
        kind: kind.into(),
        subject: vec![a.clone()],
        occurred_at: at,
        activated_at: at,
        recognised_at: c.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: s,
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("test.gm", "edit", "1"),
    })
    .unwrap();
    c.append(e).unwrap()
}

struct Campaign {
    c: Chronicle,
    clouds: BTreeMap<Address, Envelope>,
    now: UniverseTime,
    moon: Address,
    moon_event: ContentHash,
}

fn campaign() -> Campaign {
    let env = patch();
    let mut c = Chronicle::new();
    let pubr = Publisher::new("test.resolve", "patch", "1");
    let k = sse_resolve::children(&seed(), &env).unwrap();
    let mut first_sys = None;
    for i in 0..k.generated {
        let r = resolve_recorded(&mut c, &seed(), &env, &ChildRef::Generated(i), &pubr).unwrap();
        first_sys.get_or_insert((env.address.child("sys", i).unwrap(), r.envelope.unwrap()));
    }
    let (sys, sys_env) = first_sys.unwrap();
    let r0 = resolve_recorded(&mut c, &seed(), &sys_env, &ChildRef::Generated(0), &pubr).unwrap();
    let world = sys.child("body", 0).unwrap();
    assert!(r0.new);
    let now = UniverseTime::seconds(435_000_000_000_000_000);
    set(
        &mut c,
        kinds::AUTHORED_SET,
        &world,
        facts(&[
            ("roche_m", Value::Float(1.8e7)),
            ("day_s", Value::Float(1.0e7)),
            ("year_s", Value::Float(1.01e7)),
        ]),
        UniverseTime::ORIGIN,
    );
    let moon = world.authored_child("body", 1).unwrap();
    let moon_event = set(
        &mut c,
        kinds::AUTHORED_SET,
        &moon,
        facts(&[("orbit_m", Value::Float(2.0e7))]),
        UniverseTime::ORIGIN,
    );
    set(
        &mut c,
        kinds::OBSERVE_GRANT,
        &world,
        facts(&[("observe.view", Value::Text("scanned".into()))]),
        now,
    );
    let mut clouds = BTreeMap::new();
    clouds.insert(env.address.clone(), env);
    Campaign {
        c,
        clouds,
        now,
        moon,
        moon_event,
    }
}

fn run(cp: &Campaign, content: &SiftContent) -> Vec<Hit> {
    let state = cp.c.state_at(cp.now);
    // Tags from the rule pack, then sift.
    let mut tags: BTreeMap<Address, BTreeMap<String, TagState>> = BTreeMap::new();
    for a in state.facts.keys() {
        let s = Subject {
            address: Some(a.clone()),
            facts: state.facts[a].clone(),
            parent: a
                .parent()
                .and_then(|p| state.facts.get(&p).cloned())
                .unwrap_or_default(),
            now_s: cp.now.s,
            ..Default::default()
        };
        let t = content.rules.apply(&s, &BTreeMap::new());
        if !t.is_empty() {
            tags.insert(a.clone(), t);
        }
    }
    let w = World {
        state: &state,
        chronicle: Some(&cp.c),
        tags: &tags,
        clouds: &cp.clouds,
        now: cp.now,
    };
    sift(&content.sifters, &w, &"u:k7".parse().unwrap())
}

/// Acceptance: sifter hits carry evidence (facts, values, the events that
/// fixed them) and a measured significance.
#[test]
fn sifter_hits_carry_evidence() {
    let cp = campaign();
    let hits = run(&cp, &content());
    let by = |id: &str| hits.iter().filter(|h| h.sifter == id).collect::<Vec<_>>();
    let roche = by("moon-in-roche");
    assert_eq!(roche.len(), 1);
    assert_eq!(roche[0].subject, cp.moon);
    let orbit = roche[0]
        .evidence
        .iter()
        .find(|e| e.field == "orbit_m")
        .unwrap();
    assert_eq!(orbit.event, Some(cp.moon_event));
    assert_eq!(orbit.value, Value::Float(2.0e7));
    assert!(roche[0]
        .evidence
        .iter()
        .any(|e| e.field == "roche_m" && e.address == cp.moon.parent().unwrap()));
    assert_eq!(by("day-is-year").len(), 1);
    assert_eq!(by("players-learned").len(), 1);
    assert!(by("players-learned")[0].evidence[0].event.is_some());
    // Rarity is measured from the patch cloud: P(metallicity >= 0.3) under its truncated normal.
    assert!(!by("metal-rich").is_empty() && !by("star-dies-soon").is_empty());
    for h in by("metal-rich") {
        let p = h.significance.rarity_p.unwrap_or_else(|| panic!("{h:?}"));
        // Truncated normal(0, 0.2) on [-1, 0.5]: P(x >= 0.2) = 0.1534.
        assert!((p - 0.1534).abs() < 1e-3, "{p}");
        assert!(h.evidence.iter().all(|e| e.event.is_some()));
    }
    for h in by("star-dies-soon") {
        assert!(h.significance.persistence_s <= 31_557_600e9);
    }
    // Best first, ties by sifter then address; scores in [0, 1].
    assert!(hits
        .windows(2)
        .all(|w| w[0].significance.score >= w[1].significance.score));
    assert!(hits
        .iter()
        .all(|h| (0.0..=1.0).contains(&h.significance.score)));
    // Sifting mutates nothing.
    assert_eq!(cp.c.len(), campaign().c.len());
}

#[test]
fn significance_components() {
    let s = score(3.15576e17, 9_999, 0.0, Some(1e-4));
    assert_eq!(s.components[0], 1.0);
    assert!((s.components[1] - 1.0).abs() < 1e-12);
    assert_eq!(s.components[2], 1.0);
    assert_eq!(s.components[3], 1.0);
    let t = score(0.0, 0, 1.0, None);
    assert_eq!(t.score, 0.0);
}

#[test]
fn golden_digest() {
    let cp = campaign();
    let hits = run(&cp, &content());
    let digest = ContentHash::of(&serde_json::to_string(&hits).unwrap()).unwrap();
    let got = serde_json::json!({
        "hits": hits.len(),
        "hits_digest": digest.hex(),
        "state": cp.c.state_at(cp.now).hash().hex(),
    });
    crate::golden::check("sift.json", &got);
}
