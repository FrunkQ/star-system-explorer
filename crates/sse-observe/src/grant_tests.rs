//! Grants as proper events, and tags that grant (the 4X case from V10).

use crate::scenario::{gm, scene, setup, tarn};
use crate::*;
use sse_chronicle::kinds;
use sse_core::Address;

fn view_of(fx: &crate::scenario::Fx, audience: &str, a: &Address) -> Option<String> {
    scene(fx, audience)
        .subjects
        .iter()
        .find(|s| &s.address == a)
        .map(|s| s.view.clone())
}

#[test]
fn grant_sets_takes_back_and_starts_an_audience() {
    let mut fx = setup();
    let moon: Address = "u:kestrel/sys:0/body:3/body:0".parse().unwrap();
    let root = Address::universe("kestrel").unwrap();
    // Vesh starts knowing only what the fog view shows: a starting view on the root.
    grant(
        &mut fx.c,
        &root,
        "vesh",
        Some("rumour"),
        true,
        fx.now,
        &gm(),
    )
    .unwrap();
    assert_eq!(view_of(&fx, "vesh", &tarn()).as_deref(), Some("rumour"));
    // A discovery: Tarn (and its moon, by inheritance) through "scanned".
    let id = grant(
        &mut fx.c,
        &tarn(),
        "vesh",
        Some("scanned"),
        false,
        fx.now,
        &gm(),
    )
    .unwrap();
    assert_eq!(view_of(&fx, "vesh", &tarn()).as_deref(), Some("scanned"));
    assert_eq!(view_of(&fx, "vesh", &moon).as_deref(), Some("scanned"));
    // It is an observe.grant event naming the audience and the view, so history says when.
    let e = fx.c.events().iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.body.kind, kinds::OBSERVE_GRANT);
    assert_eq!(e.body.subject, vec![tarn()]);
    // Taking it back falls back to the starting view; everyone else never changed.
    grant(&mut fx.c, &tarn(), "vesh", None, false, fx.now, &gm()).unwrap();
    assert_eq!(view_of(&fx, "vesh", &tarn()).as_deref(), Some("rumour"));
    assert_eq!(
        scene(&fx, EVERYONE),
        crate::scenario::read::<Scene>("tarn-before.json")
    );
}

#[test]
fn tags_grant_views_per_audience() {
    let mut fx = setup();
    let moon: Address = "u:kestrel/sys:0/body:3/body:0".parse().unwrap();
    let root = Address::universe("kestrel").unwrap();
    for a in ["vesh", "orun"] {
        grant(&mut fx.c, &root, a, Some("rumour"), true, fx.now, &gm()).unwrap();
    }
    // One rule for every empire: whatever carries discovered:<empire> shows scanned to it.
    fx.w.campaign.tag_grants = vec![TagGrant {
        audience: "*".into(),
        tag: "discovered:{audience}".into(),
        view: "scanned".into(),
    }];
    fx.w.tags
        .entry(tarn())
        .or_default()
        .insert("discovered:vesh".into());
    assert_eq!(view_of(&fx, "vesh", &tarn()).as_deref(), Some("scanned"));
    assert_eq!(
        view_of(&fx, "vesh", &moon).as_deref(),
        Some("scanned"),
        "moons follow a tagged planet"
    );
    assert_eq!(
        view_of(&fx, "orun", &tarn()).as_deref(),
        Some("rumour"),
        "Orun has not found it"
    );
    // An explicit grant on the body beats its tag; a tag on the moon beats a grant on its planet.
    grant(
        &mut fx.c,
        &tarn(),
        "vesh",
        Some("rumour"),
        false,
        fx.now,
        &gm(),
    )
    .unwrap();
    assert_eq!(view_of(&fx, "vesh", &tarn()).as_deref(), Some("rumour"));
    fx.w.tags
        .entry(moon.clone())
        .or_default()
        .insert("discovered:vesh".into());
    assert_eq!(view_of(&fx, "vesh", &moon).as_deref(), Some("scanned"));
    // A rule for one audience only, and effective_view without tags ignores rules.
    fx.w.campaign.tag_grants.push(TagGrant {
        audience: "orun".into(),
        tag: "charted".into(),
        view: "scanned".into(),
    });
    fx.w.tags
        .entry(tarn())
        .or_default()
        .insert("charted".into());
    assert_eq!(view_of(&fx, "orun", &tarn()).as_deref(), Some("scanned"));
    let state = fx.c.state_at(fx.now);
    assert_eq!(
        effective_view(&state, &tarn(), "orun", &fx.w.campaign),
        "rumour"
    );
    // A campaign with no tag rules serialises exactly as before (old manifests and goldens hold).
    let plain = Campaign {
        tag_grants: vec![],
        ..fx.w.campaign.clone()
    };
    assert!(!serde_json::to_string(&plain)
        .unwrap()
        .contains("tag_grants"));
}
