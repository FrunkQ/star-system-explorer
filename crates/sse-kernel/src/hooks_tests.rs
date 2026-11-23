//! TG8 part 1: the base hooks pack's `access.*` labels and the optional
//! intrigue pack.

use super::universe_pack_tests::{kestrel_with, pack, send, sol_with};

/// A data view to read scenes through, and the packs under test.
fn stack(mut packs: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    packs.insert(0, pack("tests/fixtures/packs/tarn-observe.json"));
    packs
}

/// Each subject's tags in a scene for `audience`, by address.
fn tags(k: &mut crate::Kernel, scope: &str, audience: &str) -> Vec<(String, Vec<String>)> {
    let s = send(
        k,
        serde_json::json!({"op": "scene", "audience": audience, "scope": scope, "at_s": 0}),
    );
    assert_eq!(s["type"], "scene", "{s}");
    s["scene"]["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            (
                x["address"].as_str().unwrap().to_string(),
                x["tags"]
                    .as_array()
                    .map(|t| {
                        t.iter()
                            .map(|t| t.as_str().unwrap_or_default().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
            )
        })
        .collect()
}
fn with(t: &[(String, Vec<String>)], tag: &str) -> Vec<String> {
    t.iter()
        .filter(|(_, ts)| ts.iter().any(|x| x == tag))
        .map(|(a, _)| a.clone())
        .collect()
}

#[test]
fn access_labels_follow_the_facts_on_sol_and_kestrel() {
    let (mut k, sys) = sol_with(stack(vec![pack("packs/base-hooks/pack.json")]));
    let t = tags(&mut k, &sys, "gm");
    let giants: Vec<String> = (4..=7).map(|n| format!("{sys}/body:{n}")).collect();
    // Gas skimming on the four giants only; never the Sun.
    assert_eq!(with(&t, "access.gas-skimming"), giants);
    // Aerobraking where the catalogue's surface pressure is at least 100 Pa
    // (Venus, Earth, Mars at 636 Pa) and in the giants' envelopes; not
    // Pluto (1.3 Pa) and not Mercury (no air).
    let body = |n: u32| format!("{sys}/body:{n}");
    let aero = with(&t, "access.aerobraking");
    assert_eq!(aero, (1..=7).map(body).collect::<Vec<_>>(), "{aero:?}");
    assert!(
        !aero.contains(&body(8)) && !aero.contains(&body(0)),
        "{aero:?}"
    );
    // Gravity assists from every planet, not the Moon, Pluto or Ceres.
    let assist = with(&t, "access.gravity-assist");
    assert_eq!(assist.len(), 8, "{assist:?}");
    assert!(
        assist.iter().all(|a| !a.ends_with("body:2/body:0")),
        "{assist:?}"
    );

    // Kestrel, in the chart room's names: Hollow (61 K) has ice to mine, the
    // giant Vault (118 K) does not.
    let mut k = kestrel_with(stack(vec![pack("packs/base-hooks/pack.json")]));
    let t = tags(&mut k, "u:kestrel/sys:1", "gm");
    assert_eq!(with(&t, "access.ice-mining"), ["u:kestrel/sys:1/body:5"]);
    assert_eq!(with(&t, "access.gas-skimming"), ["u:kestrel/sys:1/body:4"]);

    // Every label's "Why?" lists the facts it read.
    let u = k.universe.as_ref().unwrap();
    let state = u
        .chronicle
        .state_at(sse_chronicle::UniverseTime::seconds(0));
    let a: sse_core::Address = "u:kestrel/sys:1/body:5".parse().unwrap();
    let subject = sse_sift::Subject {
        address: Some(a.clone()),
        facts: state.facts[&a].clone(),
        ..Default::default()
    };
    let got =
        k.observe
            .rules
            .apply_authored(&subject, &Default::default(), &Default::default(), None);
    let why: Vec<&str> = got["access.ice-mining"]
        .evidence
        .iter()
        .map(|e| e.field.as_str())
        .collect();
    assert!(
        why.contains(&"surface_k") && why.contains(&"mass_earth"),
        "{why:?}"
    );
}

#[test]
fn intrigue_is_optional_seeded_and_gm_only() {
    // Not in the base hooks pack: without the intrigue pack, no hook at all.
    let (mut k, sys) = sol_with(stack(vec![pack("packs/base-hooks/pack.json")]));
    assert!(tags(&mut k, &sys, "gm")
        .iter()
        .all(|(_, t)| t.iter().all(|x| !x.starts_with("intrigue."))));
    // With every chance at 1 the hooks all land, and the GM sees them; a
    // player sees none (the tags carry no tier: GM-only until disclosed).
    let mut sure = pack("packs/intrigue/pack.json");
    for r in sure["provides"]["other"]["rules"].as_array_mut().unwrap() {
        r["chance"] = 1.0.into();
    }
    let (mut k, sys) = sol_with(stack(vec![pack("packs/base-hooks/pack.json"), sure]));
    let earth = format!("{sys}/body:2");
    let gm = tags(&mut k, &sys, "gm");
    let on_earth = &gm.iter().find(|(a, _)| *a == earth).unwrap().1;
    assert_eq!(
        on_earth
            .iter()
            .filter(|x| x.starts_with("intrigue."))
            .count(),
        4,
        "{on_earth:?}"
    );
    assert!(with(&gm, "intrigue.legend")
        .iter()
        .all(|a| !a.ends_with("star:0")));
    // Even with the fullest view of Earth.
    let view = "survey";
    let r = send(
        &mut k,
        serde_json::json!({"op": "grant", "address": earth, "audience": "players", "view": view, "at_s": 0}),
    );
    assert_eq!(r["type"], "granted", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "scene", "audience": "players", "scope": sys, "at_s": 0}),
    );
    assert_eq!(r["type"], "scene", "{r}");
    assert!(!r["scene"]["subjects"].as_array().unwrap().is_empty());
    assert!(!r.to_string().contains("intrigue."), "{r}");
    // The real odds: the same hooks on a second run (seeded), and few of them.
    let (mut a, _) = sol_with(stack(vec![pack("packs/intrigue/pack.json")]));
    let (mut b, _) = sol_with(stack(vec![pack("packs/intrigue/pack.json")]));
    let ta = tags(&mut a, &sys, "gm");
    assert_eq!(ta, tags(&mut b, &sys, "gm"));
    let hooks: usize = ta
        .iter()
        .map(|(_, t)| t.iter().filter(|x| x.starts_with("intrigue.")).count())
        .sum();
    assert!(hooks < 12, "{hooks} hooks on eleven worlds");
}

#[test]
fn hooks_stack_with_the_traveller_and_universe_packs() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"), pack("packs/base-time/pack.json"),
            pack("packs/base-reference/pack.json"), pack("packs/base-hooks/pack.json"),
            pack("packs/traveller/pack.json"), pack("packs/near-future/pack.json"),
            pack("packs/space-opera/pack.json"), pack("packs/intrigue/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
}

