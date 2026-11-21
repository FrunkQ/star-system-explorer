use crate::detail::*;
use crate::scenario::{fixtures, settings};
use crate::survey::{survey_world, SurveyOptions};
use crate::*;
use sse_core::draw::Seed;

fn body(name: &str) -> Body {
    fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body
}

fn seed() -> Seed {
    Seed::from_text("sse4-detail-tests")
}

#[test]
fn the_orbit_globe_is_the_surveyed_world() {
    // S16a's check: a scanned world's continents are the surveyed world's
    // continents. Land and sea from the Orbit atlas against the full survey
    // (wear, impacts, regime landforms and cover on top), on a 48-a-face grid.
    let s = settings();
    let b = body("Earth");
    let e = envelope(&s, &b);
    let orbit = orbit_atlas(&s, &b, &seed(), b.age_gyr).unwrap();
    assert!(orbit.has_plates());
    assert!(
        orbit.address.to_string().contains("shell:orbit@"),
        "{}",
        orbit.address
    );
    let sv = survey_world(&s, &b, &seed(), b.age_gyr, &SurveyOptions::default()).unwrap();
    assert!(sv.atlas.address.to_string().contains("shell:surface@"));
    // The six level-0 tiles: the globe the Body view shows.
    let land = |a: &atlas::Atlas| -> Vec<bool> {
        (0..6u8)
            .flat_map(|f| {
                a.build(atlas::TileKey::new(f, 0, 0, 0).unwrap())
                    .elevation_dm
                    .into_iter()
                    .map(|h| h > 0)
            })
            .collect()
    };
    let agree = |x: &[bool], y: &[bool]| {
        x.iter().zip(y).filter(|(a, b)| a == b).count() as f64 / x.len() as f64
    };
    let surveyed = land(&sv.atlas);
    let same = agree(&land(&orbit), &surveyed);
    assert!(same >= 0.9, "land and sea agree on {same:.3}");
    // The old cheap globe (the regime sketch) was a different world.
    let sketch = atlas::Atlas::new(&s, &e, &seed(), b.age_gyr, vec![]);
    let old = agree(&land(&sketch), &surveyed);
    assert!(old < same, "sketch {old:.3} against orbit {same:.3}");
    eprintln!("orbit {same:.3}, old sketch {old:.3}");
}

#[test]
fn a_world_without_plates_is_one_world_at_every_level() {
    // No plates ever moved: the regime sketch is the height source at both
    // levels, worn on each level's grid, so the heights must still match.
    let s = settings();
    for name in ["Mars", "Venus", "Moon"] {
        let b = body(name);
        let orbit = orbit_atlas(&s, &b, &seed(), b.age_gyr).unwrap();
        let sv = survey_world(&s, &b, &seed(), b.age_gyr, &SurveyOptions::default()).unwrap();
        let hs = |a: &atlas::Atlas| -> Vec<f64> {
            (0..6u8)
                .flat_map(|f| {
                    a.build(atlas::TileKey::new(f, 0, 0, 0).unwrap())
                        .elevation_dm
                        .into_iter()
                        .map(|h| h as f64)
                })
                .collect()
        };
        let (x, y) = (hs(&orbit), hs(&sv.atlas));
        let n = x.len() as f64;
        let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
        let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for (a, b) in x.iter().zip(&y) {
            sxy += (a - mx) * (b - my);
            sxx += (a - mx) * (a - mx);
            syy += (b - my) * (b - my);
        }
        let r = sxy / sse_core::math::sqrt(sxx * syy).max(1e-12);
        assert!(
            r > 0.8,
            "{name}: Orbit and surveyed heights correlate at {r:.3}"
        );
    }
}

#[test]
fn orbit_atlases_reuse_by_key() {
    let s = settings();
    let b = body("Earth");
    let e = envelope(&s, &b);
    let k = orbit_key(&e, &seed(), b.age_gyr);
    assert_eq!(k, orbit_key(&e, &seed(), b.age_gyr));
    assert_ne!(k, orbit_key(&e, &seed(), b.age_gyr - 0.1));
    assert_ne!(k, orbit_key(&e, &Seed::from_text("other"), b.age_gyr));
    let a = orbit_atlas(&s, &b, &seed(), b.age_gyr).unwrap();
    let c = orbit_atlas(&s, &b, &seed(), b.age_gyr).unwrap();
    let key = atlas::TileKey::new(2, 0, 0, 0).unwrap();
    assert_eq!(a.build(key), c.build(key));
}

#[test]
fn levels_say_what_they_resolve() {
    assert!(Level::Glance < Level::Orbit && Level::Survey < Level::Site);
    assert!(Level::Orbit.badge().contains("not been surveyed"));
    assert!(Level::Orbit
        .below_resolution()
        .contains("Survey to see its history"));
}
