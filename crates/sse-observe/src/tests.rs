//! K9 acceptance: filtered scenes for the mockup's Tarn before and after
//! scan match hand-written fixtures; grants inherit; audiences are separate;
//! secrets stay home; honesty marks on the GM scene only.

use crate::scene::GM;
use crate::*;
use sse_chronicle::{
    kinds, Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime,
};
use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::{BTreeMap, BTreeSet};

#[derive(serde::Deserialize)]
struct World {
    disclosure: Disclosure,
    views: Vec<DataView>,
    campaign: Campaign,
    resolved: BTreeMap<Address, BTreeMap<String, Value>>,
    authored: BTreeMap<Address, BTreeMap<String, Value>>,
    tags: BTreeMap<Address, BTreeSet<String>>,
    grants: BTreeMap<Address, String>,
}

fn read<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let p = format!(
        "{}/../../tests/fixtures/observe/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn gm() -> Publisher {
    Publisher::new("test.gm", "session", "1")
}

fn put(c: &mut Chronicle, kind: &str, ev: Evidence, a: &Address, f: BTreeMap<String, Value>) {
    let e = Event::new(EventBody {
        kind: kind.into(),
        subject: vec![a.clone()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: c.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: [(a.clone(), f)].into(),
            ..Default::default()
        },
        evidence: ev,
        publisher: gm(),
    })
    .unwrap();
    c.append(e).unwrap();
}

struct Fx {
    w: World,
    views: BTreeMap<String, DataView>,
    c: Chronicle,
    now: UniverseTime,
}

fn setup() -> Fx {
    let w: World = read("tarn-world.json");
    let mut c = Chronicle::new();
    for (a, f) in &w.resolved {
        put(&mut c, kinds::RESOLVE, Evidence::Resolved, a, f.clone());
    }
    for (a, f) in &w.authored {
        put(
            &mut c,
            kinds::AUTHORED_SET,
            Evidence::Authored,
            a,
            f.clone(),
        );
    }
    for (a, v) in &w.grants {
        put(
            &mut c,
            kinds::OBSERVE_GRANT,
            Evidence::Authored,
            a,
            [("observe.view".to_string(), Value::Text(v.clone()))].into(),
        );
    }
    let views = w.views.iter().map(|v| (v.id.clone(), v.clone())).collect();
    Fx {
        w,
        views,
        c,
        now: UniverseTime::years(4_610_000_000),
    }
}

fn scene(fx: &Fx, audience: &str) -> Scene {
    let state = fx.c.state_at(fx.now);
    let fictional = BTreeSet::new();
    let inp = SceneInput {
        state: &state,
        chronicle: Some(&fx.c),
        tags: &fx.w.tags,
        fictional: &fictional,
        disclosure: &fx.w.disclosure,
        views: &fx.views,
        campaign: &fx.w.campaign,
    };
    build_scene(&inp, audience, &"u:kestrel/sys:0".parse().unwrap()).unwrap()
}

fn tarn() -> Address {
    "u:kestrel/sys:0/body:3".parse().unwrap()
}

/// Acceptance: Tarn before and after one tap of Scanned, against the
/// hand-written fixtures taken from the PlayerData mockup board.
#[test]
fn tarn_before_and_after_scan_match_fixtures() {
    let mut fx = setup();
    let before = scene(&fx, EVERYONE);
    assert_eq!(
        before,
        read::<Scene>("tarn-before.json"),
        "\n{}",
        serde_json::to_string_pretty(&before).unwrap()
    );
    let (_, view) = scan_toggle(
        &mut fx.c,
        &tarn(),
        EVERYONE,
        &fx.w.campaign.clone(),
        fx.now,
        &gm(),
    )
    .unwrap();
    assert_eq!(view, "scanned");
    let after = scene(&fx, EVERYONE);
    assert_eq!(
        after,
        read::<Scene>("tarn-after.json"),
        "\n{}",
        serde_json::to_string_pretty(&after).unwrap()
    );
    // Only Tarn and its moon changed (the moon followed its planet).
    let d = diff(&before, &after);
    assert_eq!(
        d.upsert
            .iter()
            .map(|s| s.address.to_string())
            .collect::<Vec<_>>(),
        vec!["u:kestrel/sys:0/body:3", "u:kestrel/sys:0/body:3/body:0"]
    );
    assert!(d.remove.is_empty());
    // Tap again to undo.
    let (_, view) = scan_toggle(
        &mut fx.c,
        &tarn(),
        EVERYONE,
        &fx.w.campaign.clone(),
        fx.now,
        &gm(),
    )
    .unwrap();
    assert_eq!(view, "long-range");
    assert_eq!(scene(&fx, EVERYONE), before);
    // "When did the players learn this" is in the history.
    assert_eq!(
        fx.c.events()
            .iter()
            .filter(|e| e.body.kind == kinds::OBSERVE_GRANT)
            .count(),
        3
    );
}

#[test]
fn secrets_never_reach_player_scenes() {
    let mut fx = setup();
    for v in ["long-range", "scanned", "survey", "rumour"] {
        put(
            &mut fx.c,
            kinds::OBSERVE_GRANT,
            Evidence::Authored,
            &tarn(),
            [("observe.view".to_string(), Value::Text(v.into()))].into(),
        );
        let s = scene(&fx, EVERYONE);
        assert_eq!(
            s.subjects
                .iter()
                .find(|x| x.address == tarn())
                .unwrap()
                .view,
            v
        );
        let text = serde_json::to_string(&s).unwrap();
        assert!(
            !text.contains("Pirate cache") && !text.contains("ancient-ruins"),
            "{v}"
        );
        assert!(
            !text.contains("resolve.tier"),
            "internal fields never shown"
        );
    }
    // The survey view adds interiors and every non-secret tag.
    put(
        &mut fx.c,
        kinds::OBSERVE_GRANT,
        Evidence::Authored,
        &tarn(),
        [("observe.view".to_string(), Value::Text("survey".into()))].into(),
    );
    let s = scene(&fx, EVERYONE);
    let t = s.subjects.iter().find(|x| x.address == tarn()).unwrap();
    assert_eq!(t.tags, vec!["garden", "habitable", "hazard", "port"]);
    assert!(t
        .facts
        .iter()
        .any(|f| f.field == "interior.core_fraction" && f.value == Some(Value::Float(0.32))));
    // An authored event made only of secret fields is not sent at all; a mixed one names its secret fields.
    let notes = Event::new(EventBody {
        kind: kinds::AUTHORED_SET.into(),
        subject: vec![tarn()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: fx.c.next_stamp(),
        causes: vec![],
        payload: Payload {
            set: [(
                tarn(),
                [("gm.plot".to_string(), Value::Text("x".into()))].into(),
            )]
            .into(),
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: gm(),
    })
    .unwrap();
    assert!(withheld(&notes, &fx.w.disclosure).1);
    let mixed =
        fx.c.events()
            .iter()
            .find(|e| e.body.kind == kinds::AUTHORED_SET)
            .unwrap();
    let (hidden, all) = withheld(mixed, &fx.w.disclosure);
    assert_eq!(
        (hidden, all),
        (vec![(tarn(), "gm.notes".to_string())], false)
    );
    // An unlisted field defaults to the GM-only rung.
    assert_eq!(fx.w.disclosure.field_tier("brand_new_fact"), "secret");
}

#[test]
fn gm_scene_has_everything_and_honesty_marks() {
    let fx = setup();
    let s = scene(&fx, GM);
    let t = s.subjects.iter().find(|x| x.address == tarn()).unwrap();
    assert_eq!(t.view, "gm");
    assert!(t.tags.contains(&"ancient-ruins".to_string()));
    let mass = t.facts.iter().find(|f| f.field == "mass_kg").unwrap();
    assert_eq!(mass.marks, vec!["authored"]);
    assert!(t.facts.iter().any(|f| f.field == "gm.notes"));
    assert!(t
        .facts
        .iter()
        .find(|f| f.field == "name")
        .unwrap()
        .marks
        .is_empty());
    // Player scenes carry no marks unless their view switches them on.
    assert!(scene(&fx, EVERYONE)
        .subjects
        .iter()
        .flat_map(|s| &s.facts)
        .all(|f| f.marks.is_empty()));
}

#[test]
fn audiences_are_separate_and_own_grants_beat_inherited() {
    let mut fx = setup();
    let campaign = fx.w.campaign.clone();
    scan_toggle(&mut fx.c, &tarn(), "vesh", &campaign, fx.now, &gm()).unwrap();
    assert_eq!(scene(&fx, EVERYONE), read::<Scene>("tarn-before.json"));
    let v = scene(&fx, "vesh");
    assert_eq!(
        v.subjects
            .iter()
            .find(|s| s.address == tarn())
            .unwrap()
            .view,
        "scanned"
    );
    // A moon with its own grant keeps it when its planet is scanned.
    let moon: Address = "u:kestrel/sys:0/body:3/body:0".parse().unwrap();
    put(
        &mut fx.c,
        kinds::OBSERVE_GRANT,
        Evidence::Authored,
        &moon,
        [("observe.view".to_string(), Value::Text("rumour".into()))].into(),
    );
    scan_toggle(&mut fx.c, &tarn(), EVERYONE, &campaign, fx.now, &gm()).unwrap();
    let s = scene(&fx, EVERYONE);
    assert_eq!(
        s.subjects.iter().find(|x| x.address == moon).unwrap().view,
        "rumour"
    );
    // An unknown view is a readable error, not a silent leak.
    put(
        &mut fx.c,
        kinds::OBSERVE_GRANT,
        Evidence::Authored,
        &moon,
        [("observe.view".to_string(), Value::Text("nope".into()))].into(),
    );
    let state = fx.c.state_at(fx.now);
    let fictional = BTreeSet::new();
    let inp = SceneInput {
        state: &state,
        chronicle: None,
        tags: &fx.w.tags,
        fictional: &fictional,
        disclosure: &fx.w.disclosure,
        views: &fx.views,
        campaign: &fx.w.campaign,
    };
    assert!(build_scene(&inp, EVERYONE, &"u:kestrel".parse().unwrap())
        .unwrap_err()
        .contains("\"nope\" is not defined"));
}

#[test]
fn golden_digest() {
    let mut fx = setup();
    let before = scene(&fx, EVERYONE).hash();
    let campaign = fx.w.campaign.clone();
    scan_toggle(&mut fx.c, &tarn(), EVERYONE, &campaign, fx.now, &gm()).unwrap();
    let got = serde_json::json!({
        "tarn_before": before.hex(),
        "tarn_after": scene(&fx, EVERYONE).hash().hex(),
        "gm": scene(&fx, GM).hash().hex(),
    });
    let path = format!(
        "{}/../../tests/golden/observe.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1");
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden observe.json changed"
    );
}
