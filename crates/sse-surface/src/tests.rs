use crate::scenario::{fixtures, golden, settings};
use crate::sketch::{hemispheric_split, hypsometry};
use crate::*;
use sse_core::draw::Seed;

fn check_golden(name: &str, got: &serde_json::Value) {
    let path = format!("{}/../../tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = serde_json::to_string_pretty(got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing {path}; run with SSE_BLESS=1"));
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden {name} changed"
    );
}

#[test]
fn fixture_worlds_are_classed_correctly() {
    let s = settings();
    let mut wrong = vec![];
    for f in fixtures() {
        let e = envelope(&s, &f.body);
        if e.regime_now != f.expect {
            wrong.push(format!(
                "{}: got {}, want {}",
                f.body.label, e.regime_now, f.expect
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn regime_changes_have_dated_causes() {
    let s = settings();
    let earth = fixtures()
        .into_iter()
        .find(|f| f.body.label == "Earth")
        .unwrap()
        .body;
    let e = envelope(&s, &earth);
    // Magma ocean, then plates from the moment it solidified.
    assert_eq!(e.timeline.first().unwrap().regime, "magma-ocean");
    assert_eq!(e.events.len(), 1, "{:#?}", e.events);
    assert_eq!(e.events[0].to, "mobile-lid");
    assert!((e.events[0].t_gyr - 0.1).abs() < 1e-12);
    assert!(
        e.events[0].cause.contains("magma ocean solidified"),
        "{}",
        e.events[0].cause
    );
    // Spans tile the life without gaps.
    for w in e.timeline.windows(2) {
        assert_eq!(w[0].to_gyr, w[1].from_gyr);
    }
    assert_eq!(e.timeline.last().unwrap().to_gyr, earth.age_gyr);
    // Earth ten billion years on: radiogenic heat falls below the critical
    // flow and the plates stop, at a bisected date with the reason given.
    let mut old = earth.clone();
    old.age_gyr = 14.0;
    let e = envelope(&s, &old);
    let last = e.events.last().unwrap();
    assert_eq!(
        (last.from.as_str(), last.to.as_str()),
        ("mobile-lid", "stagnant-lid")
    );
    assert!(last.t_gyr > 8.0 && last.t_gyr < 13.0, "{}", last.t_gyr);
    assert!(last.cause.contains("critical value"), "{}", last.cause);
    for ev in &e.events {
        let rule = s.rules.iter().find(|r| r.regime == ev.to).unwrap();
        assert!(ev.cause.contains(&rule.because));
    }
}

#[test]
fn envelopes_are_pure() {
    let s = settings();
    for f in fixtures() {
        assert_eq!(envelope(&s, &f.body), envelope(&s, &f.body));
    }
}

#[test]
fn sketch_globes_are_of_their_kind() {
    let s = settings();
    let seed = Seed::from_text("sse4-surface-tests");
    let by = |name: &str| {
        let b = fixtures()
            .into_iter()
            .find(|f| f.body.label == name)
            .unwrap()
            .body;
        let e = envelope(&s, &b);
        sketch(&s, &e, &seed, SurveyState::Scanned)
    };
    let earth = by("Earth");
    let venus = by("Venus");
    let mars = by("Mars");
    let (_, _, b_earth) = hypsometry(&earth);
    let (_, sd_venus, b_venus) = hypsometry(&venus);
    let (_, sd_mars, _) = hypsometry(&mars);
    // Earth-like: two peaks (continents and ocean floor), about 30% land.
    assert!(b_earth > 5.0 / 9.0, "earth bimodality {b_earth}");
    let land = earth.liquid.iter().filter(|l| !**l).count() as f64 / earth.liquid.len() as f64;
    assert!(land > 0.15 && land < 0.5, "land {land}");
    // Venus-like: one narrow peak.
    assert!(b_venus < 5.0 / 9.0, "venus bimodality {b_venus}");
    assert!(sd_venus < sd_mars);
    // Mars-like: a hemispheric split of kilometres.
    let split = hemispheric_split(&mars);
    assert!(split > 1500.0, "mars split {split}");
    assert!(hemispheric_split(&venus) < split / 3.0);
    // Giant: no heights, albedo bands.
    let jup = by("Jupiter");
    assert!(jup.height_m.iter().all(|h| *h == 0));
    assert!(jup.albedo.iter().max() > jup.albedo.iter().min());
    // Same call, same globe; different worlds, different globes.
    assert_eq!(by("Earth"), earth);
    assert_ne!(earth.albedo, venus.albedo);
}

#[test]
fn disclosure_sharpens_the_sketch() {
    let s = settings();
    let seed = Seed::from_text("sse4-surface-tests");
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == "Earth")
        .unwrap()
        .body;
    let e = envelope(&s, &b);
    let un = sketch(&s, &e, &seed, SurveyState::Unexamined);
    let sc = sketch(&s, &e, &seed, SurveyState::Scanned);
    assert!(un.height_m.is_empty() && !un.albedo.is_empty());
    assert!(sc.height_m.len() == (sc.nlat * sc.nlon) as usize);
    assert!(sc.nlat > un.nlat);
}

#[test]
fn generated_worlds_get_envelopes() {
    let s = settings();
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    let mut n = 0;
    for h in sse_formation::scenario::solar_patch()
        .into_iter()
        .flatten()
        .take(12)
    {
        let run = sse_formation::pipeline::run(h, sse_formation::pipeline::Block::Modern);
        let bodies = bodies_of(&run, &s);
        for (b, e) in bodies.iter().zip(envelopes_of(&run, &s)) {
            n += 1;
            *seen.entry(e.regime_now.clone()).or_default() += 1;
            assert!(!e.timeline.is_empty());
            assert!(e
                .events
                .iter()
                .all(|x| !x.cause.is_empty() && x.t_gyr >= 0.0 && x.t_gyr <= b.age_gyr));
            if b.deep_envelope {
                assert_eq!(e.regime_now, "cloud-deck");
            }
            if !b.deep_envelope
                && b.mass_earth < 0.3
                && b.water_fraction < 0.05
                && e.t_surface_now_k < 1400.0
            {
                assert!(
                    ["stagnant-lid", "small-body", "heat-pipe", "carbon-rich"]
                        .contains(&e.regime_now.as_str()),
                    "{} {} M {}",
                    b.label,
                    e.regime_now,
                    b.mass_earth
                );
            }
        }
    }
    assert!(n > 20, "{n} worlds");
    assert!(seen.len() >= 3, "{seen:?}");
}

#[test]
fn tides_lock_close_worlds() {
    let s = settings();
    let mut b = fixtures()
        .into_iter()
        .find(|f| f.body.label == "Earth")
        .unwrap()
        .body;
    let o = Orbit {
        a_au: 1.0,
        e: 0.0,
        host_msun: 1.0,
    };
    assert!(lock_time_gyr(&b, &o, &s.tide) > 100.0);
    let close = Orbit {
        a_au: 0.05,
        ..o.clone()
    };
    assert!(lock_time_gyr(&b, &close, &s.tide) < 1.0);
    b.orbit = Some(Orbit { e: 0.1, ..close });
    let e = envelope(&s, &b);
    assert!(e.locked);
    assert!(e.tidal_w_m2 > 0.0);
}

#[test]
fn settings_reject_bad_packs() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-surface/pack.json")).unwrap();
    v["provides"]["other"]["surface"]["rules"][0]["when"] =
        serde_json::json!({"no_such_quantity": {"min": 1}});
    let p = sse_pack::PackManifest::from_json(&v.to_string()).unwrap();
    assert!(SurfaceSettings::from_packs(&[p]).is_err());
}

#[test]
fn golden_surface() {
    check_golden("surface.json", &golden());
}
