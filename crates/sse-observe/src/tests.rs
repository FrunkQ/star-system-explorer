//! K9 acceptance: filtered scenes for the mockup's Tarn before and after
//! scan match hand-written fixtures; grants inherit; audiences are separate;
//! secrets stay home; honesty marks on the GM scene only.

use crate::scene::GM;
use crate::*;
use sse_chronicle::{kinds, Event, EventBody, Evidence, Payload, UniverseTime};
use sse_core::cbor::Value;
use sse_core::Address;
use std::collections::BTreeSet;

use crate::scenario::{gm, put, read, scene, setup, tarn};

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
        notes: &[],
    };
    assert!(build_scene(&inp, EVERYONE, &"u:kestrel".parse().unwrap())
        .unwrap_err()
        .contains("\"nope\" is not defined"));
}

/// Golden digest; the same function runs in the browser harness.
#[test]
fn golden_digest() {
    let got = crate::scenario::golden();
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

/// J1: record notes in scenes go only to the audiences they are for, wait
/// for their tier, and leave a scene without notes hashing as before.
#[test]
fn record_notes_follow_audience_and_tier() {
    use sse_record::{add_note, Audience, NoteDraft};
    let mut fx = setup();
    let plain = scene(&fx, "vesh").hash();
    let draft =
        |text: &str, audience: Audience, tier: Option<&str>, subjects: Vec<Address>| NoteDraft {
            subjects,
            text: text.into(),
            tags: vec![],
            by: "gm".into(),
            audience,
            tier: tier.map(str::to_string),
            supersedes: None,
            narrative: false,
            arrives: None,
        };
    let actor: Address = "u:kestrel/sys:0/actor:guild".parse().unwrap();
    for d in [
        draft(
            "the crew landed here",
            Audience::Only(vec!["crew".into()]),
            None,
            vec![tarn()],
        ),
        draft(
            "a beacon on the moon",
            Audience::everyone(),
            None,
            vec![tarn()],
        ),
        draft(
            "seen from orbit after a scan",
            Audience::everyone(),
            Some("scanned"),
            vec![tarn()],
        ),
        draft(
            "the guild owns Tarn's port",
            Audience::Only(vec!["vesh".into()]),
            None,
            vec![actor.clone(), tarn()],
        ),
        draft("a plot twist", Audience::gm(), None, vec![tarn()]),
    ] {
        add_note(&mut fx.c, &d, fx.now, &gm()).unwrap();
    }
    let texts = |fx: &crate::scenario::Fx, aud: &str, a: &Address| -> Vec<String> {
        scene(fx, aud)
            .subjects
            .iter()
            .find(|s| &s.address == a)
            .map(|s| s.notes.iter().map(|n| n.text.clone()).collect())
            .unwrap_or_default()
    };
    assert_eq!(
        texts(&fx, "crew", &tarn()),
        vec!["the crew landed here", "a beacon on the moon"]
    );
    assert_eq!(
        texts(&fx, "vesh", &tarn()),
        vec!["a beacon on the moon", "the guild owns Tarn's port"]
    );
    assert_eq!(texts(&fx, GM, &tarn()).len(), 5);
    // Player scenes carry no record event ids; the GM's do.
    let ids = |aud: &str| -> Vec<Option<String>> {
        scene(&fx, aud)
            .subjects
            .iter()
            .flat_map(|s| s.notes.iter().map(|n| n.id.clone()))
            .collect()
    };
    assert!(ids("crew").iter().all(Option::is_none));
    assert!(ids(GM).iter().all(Option::is_some));
    // An actor known only from the record shows to its audience alone.
    assert_eq!(
        texts(&fx, "vesh", &actor),
        vec!["the guild owns Tarn's port"]
    );
    assert!(scene(&fx, "crew")
        .subjects
        .iter()
        .all(|s| s.address != actor));
    let n = scene(&fx, "vesh")
        .subjects
        .into_iter()
        .find(|s| s.address == actor)
        .unwrap();
    assert_eq!(n.notes[0].links, vec![tarn()]);
    // The tiered note appears once the audience's view shows that tier.
    let campaign = fx.w.campaign.clone();
    assert!(!texts(&fx, "crew", &tarn()).contains(&"seen from orbit after a scan".to_string()));
    scan_toggle(&mut fx.c, &tarn(), "crew", &campaign, fx.now, &gm()).unwrap();
    assert!(texts(&fx, "crew", &tarn()).contains(&"seen from orbit after a scan".to_string()));
    // A scene with no notes serialises without the field, so its hash is the old one.
    let fresh = setup();
    assert_eq!(scene(&fresh, "vesh").hash(), plain);
    assert!(!serde_json::to_string(&scene(&fresh, "vesh"))
        .unwrap()
        .contains("notes"));
}
