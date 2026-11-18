//! C1 checks: the SSE 3 classic scheme against SSE 3's own code, the Solar
//! System, examples, relabel events and replay (classification design 12).

use crate::classify::{Atlas, Bound, Class, ClassKind, Scheme, Scoring};
use crate::predicate::Subject;
use crate::relabel;
use crate::scenario::{classic_atlas, classic_pack, warming_earth};
use sse_chronicle::{Chronicle, Evidence, RunManifest, UniverseTime};
use sse_core::cbor::Value;
use sse_core::Seed;
use std::collections::BTreeMap;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

fn read(p: &str) -> String {
    std::fs::read_to_string(format!("{ROOT}{p}")).unwrap()
}
fn facts(v: &serde_json::Value) -> BTreeMap<String, Value> {
    serde_json::from_value(v.clone()).unwrap()
}

/// SSE 3's classifier reads its feature map directly: same axes, read by
/// their SSE 3 names, with no unit change and no defaults.
fn feature_space(mut a: Atlas) -> Atlas {
    for (id, axis) in a.axes.iter_mut() {
        axis.field = id.clone();
        axis.divide_by = None;
        axis.missing = None;
    }
    a
}

#[test]
fn classic_pack_is_complete_and_valid() {
    let a = classic_atlas();
    assert_eq!(a.classes.len(), 72);
    assert_eq!(
        a.classes
            .iter()
            .filter(|c| c.kind == ClassKind::Modifier)
            .count(),
        7
    );
    let sc = &a.schemes["sse3-classic"];
    assert_eq!(
        (sc.scoring, sc.tolerance, sc.max_classes),
        (Scoring::Classic, 0.15, 4)
    );
    // Every class carries its learn text and its typical values where SSE 3 had them.
    assert!(a.classes.iter().all(|c| !c.learn.summary.is_empty()));
    assert!(a.classes.iter().filter(|c| !c.typical.is_empty()).count() >= 45);
    assert!(a.classes.iter().filter(|c| c.look.image.is_some()).count() >= 60);
}

/// The port against the original: 3,000 bodies scattered around every
/// fingerprint (on edges, outside them, with features missing), classified
/// by SSE 3.1's own classification.ts.
#[test]
fn classic_scoring_matches_sse3_on_every_oracle_body() {
    let a = feature_space(classic_atlas());
    let oracle: serde_json::Value =
        serde_json::from_str(&read("tests/fixtures/classify/v3-oracle.json")).unwrap();
    let close = |js: f64, rs: f64| (js - rs).abs() <= 0.005 + 1e-9;
    let mut n = 0;
    for (i, b) in oracle["bodies"].as_array().unwrap().iter().enumerate() {
        let l = a.label("sse3-classic", &Subject::new(facts(&b["features"])), None);
        let w = l.classic.as_ref().unwrap();
        let want: Vec<String> = serde_json::from_value(b["classes"].clone()).unwrap();
        assert_eq!(w.classes, want, "body {i}: classes");
        assert_eq!(l.base.as_deref(), b["base"].as_str(), "body {i}: base");
        assert_eq!(
            w.fallback,
            b["fallback"].as_bool().unwrap(),
            "body {i}: fallback"
        );
        assert_eq!(
            w.borderline,
            b["borderline"].as_bool().unwrap(),
            "body {i}: borderline"
        );
        let cands = b["candidates"].as_array().unwrap();
        assert_eq!(w.candidates.len(), cands.len(), "body {i}: candidates");
        for (c, j) in w.candidates.iter().zip(cands) {
            assert_eq!(c.class, j[0].as_str().unwrap(), "body {i}: candidate order");
            assert!(
                close(j[1].as_f64().unwrap(), c.fit),
                "body {i}: {} score {} vs {}",
                c.class,
                c.fit,
                j[1]
            );
        }
        let mods = b["modifiers"].as_array().unwrap();
        assert_eq!(w.modifiers.len(), mods.len(), "body {i}: modifiers");
        for ((c, s), j) in w.modifiers.iter().zip(mods) {
            assert_eq!(c, j[0].as_str().unwrap());
            assert!(close(j[1].as_f64().unwrap(), *s));
        }
        n += 1;
    }
    assert_eq!(n, 3000);
}

/// Every Solar System body gets the label SSE 3 gives it, from SSE4 facts
/// (SI quantities under V4 names) through the axis registry.
#[test]
fn solar_system_bodies_get_their_sse3_labels_from_sse4_facts() {
    let a = classic_atlas();
    let sol: serde_json::Value =
        serde_json::from_str(&read("tests/fixtures/classify/solar-system.json")).unwrap();
    for b in sol["bodies"].as_array().unwrap() {
        let l = a.label("sse3-classic", &Subject::new(facts(&b["facts"])), None);
        let want: Vec<String> = serde_json::from_value(b["v3_classes"].clone()).unwrap();
        assert_eq!(l.classic.as_ref().unwrap().classes, want, "{}", b["name"]);
        // Each label answers why (its bands) and what is nearest.
        let w = l.classic.unwrap();
        let win = &w.candidates[0];
        assert_eq!(Some(&win.class), l.base.as_ref());
        assert!(
            !win.axes.is_empty() && win.limiting.is_some(),
            "{}: no working",
            b["name"]
        );
        assert_eq!(l.nearest.len(), w.candidates.len() - 1);
    }
    // Spot checks a reader can verify.
    let get = |n: &str| {
        let b = sol["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["name"] == n)
            .unwrap();
        a.label("sse3-classic", &Subject::new(facts(&b["facts"])), None)
    };
    assert_eq!(get("Earth").base.as_deref(), Some("planet/earth-analogue"));
    assert_eq!(get("Jupiter").modifiers, vec!["planet/ringed".to_string()]);
    assert_eq!(
        get("Europa").base.as_deref(),
        Some("planet/subsurface-ocean")
    );
    let nep = get("Neptune");
    assert_eq!(nep.base.as_deref(), Some("planet/ice-giant"));
    // Nearest carries what would have to change: Neptune's runner-up says how far.
    assert!(nep.nearest[0]
        .axes
        .iter()
        .any(|x| x.change_to_core.is_some()));
}

#[test]
fn class_examples_pass() {
    assert_eq!(classic_atlas().check_examples(), Vec::<String>::new());
}

/// SSE 3's B16 rule: a complete match outranks a partial one whatever the
/// weight says, and weight still decides when both fit (ported from
/// classification.spec.ts, "rigged" fixture).
#[test]
fn clean_match_outranks_heavier_partial_match() {
    let num = |axis: &str, lo: f64, hi: f64| Bound::Numeric {
        axis: axis.into(),
        core: (lo, hi),
        fringe: (lo, hi),
    };
    let class = |id: &str, w: f64, region: Vec<Bound>| {
        Class {
            weight: w,
            ..serde_json::from_value(
                serde_json::json!({"id": id, "scheme": "s", "kind": "base", "region": []}),
            )
            .unwrap()
        }
        .with_region(region)
    };
    let mut a = Atlas::default();
    for x in ["mass_Me", "density", "Teq_K"] {
        a.axes.insert(
            x.into(),
            serde_json::from_value(serde_json::json!({"field": x, "scale": "linear"})).unwrap(),
        );
    }
    a.schemes.insert(
        "s".into(),
        serde_json::from_value::<Scheme>(serde_json::json!({"id": "s", "scoring": "classic"}))
            .unwrap(),
    );
    a.classes = vec![
        class(
            "planet/generic",
            1.0,
            vec![num("mass_Me", 0.5, 2.0), num("density", 3.0, 8.0)],
        ),
        class(
            "planet/fancy",
            3.0,
            vec![
                num("mass_Me", 0.5, 2.0),
                num("density", 3.0, 8.0),
                num("Teq_K", 250.0, 300.0),
            ],
        ),
    ];
    a.validate().unwrap();
    let body = |t: f64| {
        let mut f = BTreeMap::new();
        f.insert("mass_Me".to_string(), Value::Float(1.0));
        f.insert("density".to_string(), Value::Float(5.5));
        f.insert("Teq_K".to_string(), Value::Float(t));
        Subject::new(f)
    };
    let near_miss = a.label("s", &body(310.0), None);
    assert_eq!(near_miss.base.as_deref(), Some("planet/generic"));
    // ...although on score alone the heavier partial match would have won.
    let w = near_miss.classic.unwrap();
    assert!(w.candidates[1].fit > w.candidates[0].fit);
    assert_eq!(
        a.label("s", &body(275.0), None).base.as_deref(),
        Some("planet/fancy")
    );
}

#[test]
fn relabel_events_are_dated_derived_and_replay_from_the_manifest() {
    let atlas = classic_atlas();
    let run = |atlas: &Atlas| {
        let (mut ch, a, times) = warming_earth();
        let authored_only = ch.state_at(UniverseTime::years(10_000_000_000)).hash();
        let rl = relabel::record(&mut ch, atlas, "sse3-classic", &a, &times).unwrap();
        (ch, rl, authored_only)
    };
    let (ch, rl, authored_only) = run(&atlas);
    let story: Vec<(i64, Option<&str>)> = rl.iter().map(|r| (r.at.s, r.to.as_deref())).collect();
    assert!(rl.len() >= 2, "{story:?}");
    assert_eq!(rl[0].from.as_deref(), Some("planet/earth-analogue"));
    assert_eq!(
        rl.last().unwrap().to.as_deref(),
        Some("planet/lava"),
        "{story:?}"
    );
    for r in &rl {
        let e = ch.events().iter().find(|e| e.id == r.event).unwrap();
        assert_eq!(e.body.evidence, Evidence::Derived);
        assert!(e.body.payload.set.is_empty() && e.body.payload.unset.is_empty());
        assert_eq!(
            e.body.causes.len(),
            1,
            "a relabel points at the change that caused it"
        );
    }
    // Labels are never the only record: the facts are the same with or without them.
    assert_eq!(
        ch.state_at(UniverseTime::years(10_000_000_000)).hash(),
        authored_only
    );
    // Replay: the same authored chronicle and the same packs give the same manifest and the same events.
    let manifest = |ch: &Chronicle| {
        let p = classic_pack();
        RunManifest::new(
            Seed::from_text("sol"),
            "0.1.0",
            vec![sse_chronicle::PackRef {
                id: p.id.clone(),
                version: p.version.clone(),
                content_hash: p.content_hash(),
            }],
            ch.authored_hash(),
        )
        .id()
    };
    let (ch2, rl2, _) = run(&classic_atlas());
    assert_eq!(manifest(&ch), manifest(&ch2));
    assert_eq!(rl, rl2);
    assert_eq!(ch.head(), ch2.head());
    // A different atlas is a different answer, recorded as such: same facts, other events.
    let mut other = classic_atlas();
    other.classes.retain(|c| c.id != "planet/desert");
    let (ch3, rl3, _) = run(&other);
    assert_eq!(ch3.authored_hash(), ch.authored_hash());
    assert_ne!(
        rl3.iter().map(|r| r.event).collect::<Vec<_>>(),
        rl.iter().map(|r| r.event).collect::<Vec<_>>()
    );
}

#[test]
fn removing_every_atlas_leaves_facts_unchanged() {
    let (mut ch, a, times) = warming_earth();
    let before = ch.state_at(UniverseTime::years(10_000_000_000));
    let none = crate::pack::load(&[]).unwrap().atlas;
    assert!(relabel::record(&mut ch, &none, "sse3-classic", &a, &times)
        .unwrap()
        .is_empty());
    relabel::record(&mut ch, &classic_atlas(), "sse3-classic", &a, &times).unwrap();
    assert_eq!(ch.state_at(UniverseTime::years(10_000_000_000)), before);
}

#[test]
fn golden() {
    crate::golden::check("classify.json", &crate::scenario::classify_golden());
}
