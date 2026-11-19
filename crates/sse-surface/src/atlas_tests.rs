use crate::atlas::*;
use crate::scenario::{fixtures, settings};
use crate::sketch::SurveyState;
use crate::*;
use sse_core::draw::Seed;

fn small() -> SurfaceSettings {
    let mut s = settings();
    s.atlas.tile_samples = 32;
    s
}

fn atlas_of(s: &SurfaceSettings, name: &str, sites: Vec<Site>) -> Atlas {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body;
    let e = envelope(s, &b);
    Atlas::new(
        s,
        &e,
        &Seed::from_text("sse4-atlas-tests"),
        b.age_gyr,
        sites,
    )
}

#[test]
fn any_tile_builds_alone() {
    let s = small();
    let a = atlas_of(&s, "Earth", vec![]);
    let keys = [
        TileKey::new(0, 0, 0, 0).unwrap(),
        TileKey::new(3, 4, 7, 2).unwrap(),
        TileKey::new(5, 9, 300, 511).unwrap(),
    ];
    let forward: Vec<Tile> = keys.iter().map(|k| a.build(*k)).collect();
    // A fresh atlas, keys in the other order: identical tiles.
    let b = atlas_of(&s, "Earth", vec![]);
    for (k, t) in keys.iter().zip(&forward).rev() {
        assert_eq!(&b.build(*k), t);
    }
    assert_ne!(forward[0].hash, forward[1].hash);
    assert_eq!(
        forward[2].address.to_string(),
        format!("{}/tile:5.9.300.511", a.address)
    );
    assert!(a.address.to_string().ends_with("/shell:surface@4500"));
}

#[test]
fn children_downsample_to_parents() {
    let s = small();
    for name in ["Earth", "Mars", "Europa", "Io", "Vesta"] {
        let a = atlas_of(&s, name, vec![]);
        for parent in [
            TileKey::new(0, 0, 0, 0).unwrap(),
            TileKey::new(2, 3, 5, 1).unwrap(),
            TileKey::new(4, 8, 100, 200).unwrap(),
            TileKey::new(1, 14, 9000, 4000).unwrap(),
        ] {
            let p = a.build(parent);
            for c in parent.children() {
                let ch = a.build(c);
                let err = downsample_error_m(&p, &ch);
                let tol = a.tolerance_m(c.level);
                assert!(
                    err <= tol,
                    "{name} {parent:?} -> {c:?}: error {err} m over tolerance {tol} m"
                );
            }
        }
    }
}

#[test]
fn faces_cover_the_sphere() {
    for f in 0..6u8 {
        for (u, v) in [(0.0, 0.0), (0.9, -0.3), (-0.99, 0.99), (0.25, 0.5)] {
            let d = direction(f, u, v);
            let (g, uu, vv) = face_of(d);
            assert_eq!(g, f);
            assert!((uu - u).abs() < 1e-12 && (vv - v).abs() < 1e-12);
        }
    }
    // Spacing is even to within the tangent warp's ratio across a face.
    let s = small();
    let a = atlas_of(&s, "Earth", vec![]);
    let t = a.build(TileKey::new(0, 0, 0, 0).unwrap());
    assert!(
        t.spacing_m > 250_000.0 && t.spacing_m < 350_000.0,
        "{}",
        t.spacing_m
    );
}

#[test]
fn disclosure_and_the_fact_floor() {
    let s = settings();
    let site = Site {
        label: "Landing zone".into(),
        lat_deg: 10.0,
        lon_deg: 20.0,
        radius_km: 5.0,
    };
    let a = atlas_of(&s, "Earth", vec![site]);
    let floor = a.level_for(s.atlas.fact_floor_m);
    let site_floor = a.level_for(s.atlas.site_floor_m);
    assert!(a.spacing_m(floor) <= 100.0 && a.spacing_m(floor - 1) > 100.0);
    assert!(site_floor > floor);
    let k0 = TileKey::new(0, 0, 0, 0).unwrap();
    assert!(matches!(
        a.tile(k0, SurveyState::Unexamined),
        Err(Refusal::Withheld { .. })
    ));
    assert!(a.max_level(SurveyState::Scanned, &k0) == Some(0));
    assert!(matches!(
        a.tile(TileKey::new(0, 1, 0, 0).unwrap(), SurveyState::Scanned),
        Err(Refusal::Withheld { .. })
    ));
    // The site's tile at the site floor, found from its direction.
    let (la, lo) = (10f64.to_radians(), 20f64.to_radians());
    let d = [
        sse_core::math::cos(la) * sse_core::math::cos(lo),
        sse_core::math::cos(la) * sse_core::math::sin(lo),
        sse_core::math::sin(la),
    ];
    let (f, u, v) = face_of(d);
    let at = |level: u8| {
        let n = (1u64 << level) as f64;
        let x = (((u + 1.0) / 2.0) * n) as u32;
        let y = (((v + 1.0) / 2.0) * n) as u32;
        TileKey::new(f, level, x, y).unwrap()
    };
    let deep = at(site_floor);
    assert_eq!(a.max_level(SurveyState::Landed, &deep), Some(site_floor));
    assert_eq!(a.max_level(SurveyState::Surveyed, &deep), Some(floor));
    assert!(matches!(
        a.tile(deep, SurveyState::Surveyed),
        Err(Refusal::BelowFactFloor { .. })
    ));
    // Away from the site, landing does not sharpen the map.
    let far = TileKey::new(5, site_floor, 0, 0).unwrap();
    assert_eq!(a.max_level(SurveyState::Landed, &far), Some(floor));
    // Below the site floor is decoration everywhere.
    let finer = at(site_floor + 1);
    assert!(matches!(
        a.tile(finer, SurveyState::Landed),
        Err(Refusal::BelowFactFloor { .. })
    ));
}

#[test]
fn level_zero_agrees_with_the_sketch() {
    let s = small();
    let a = atlas_of(&s, "Earth", vec![]);
    let mut land = 0usize;
    let mut n = 0usize;
    for f in 0..6 {
        let t = a.build(TileKey::new(f, 0, 0, 0).unwrap());
        land += t.cover.iter().filter(|c| **c == 0).count();
        n += t.cover.len();
        assert!(t.material.iter().all(|m| (*m as usize) < MATERIALS.len()));
        assert!(t.slope_ddeg.iter().all(|x| *x <= 900));
    }
    let frac = land as f64 / n as f64;
    assert!(frac > 0.15 && frac < 0.5, "land {frac}");
    // A giant has no relief and no detail.
    let j = atlas_of(&s, "Jupiter", vec![]);
    let t = j.build(TileKey::new(2, 3, 1, 1).unwrap());
    assert!(t.elevation_dm.iter().all(|h| *h == 0));
    assert!(t.material.iter().all(|m| MATERIALS[*m as usize] == "cloud"));
}

#[test]
fn the_cache_never_changes_a_tile() {
    let s = small();
    let a = atlas_of(&s, "Mars", vec![]);
    let mut c = TileCache::new(2);
    let k = |x| TileKey::new(1, 3, x, 2).unwrap();
    let t1 = c.get(&a, k(1), SurveyState::Surveyed).unwrap();
    let again = c.get(&a, k(1), SurveyState::Surveyed).unwrap();
    assert_eq!(t1, again);
    assert_eq!((c.hits, c.misses), (1, 1));
    c.get(&a, k(2), SurveyState::Surveyed).unwrap();
    c.get(&a, k(3), SurveyState::Surveyed).unwrap();
    assert_eq!(c.len(), 2);
    assert_eq!(
        c.get(&a, k(1), SurveyState::Surveyed).unwrap(),
        a.build(k(1))
    );
    // Disclosure applies to cached tiles too.
    assert!(c.get(&a, k(2), SurveyState::Scanned).is_err());
}

#[test]
fn epochs_have_their_own_atlas() {
    let s = small();
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == "Earth")
        .unwrap()
        .body;
    let e = envelope(&s, &b);
    let seed = Seed::from_text("sse4-atlas-tests");
    let young = Atlas::new(&s, &e, &seed, 0.05, vec![]);
    let now = Atlas::new(&s, &e, &seed, b.age_gyr, vec![]);
    assert_eq!(young.regime, "magma-ocean");
    assert_eq!(now.regime, "mobile-lid");
    assert_ne!(young.address, now.address);
    let k = TileKey::new(0, 0, 0, 0).unwrap();
    assert!(young
        .build(k)
        .cover
        .iter()
        .all(|c| COVERS[*c as usize] == "magma"));
}

#[test]
fn golden_atlas() {
    let got = crate::scenario::atlas_golden();
    let path = format!(
        "{}/../../tests/golden/surface-atlas.json",
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
        "golden surface-atlas changed"
    );
}
