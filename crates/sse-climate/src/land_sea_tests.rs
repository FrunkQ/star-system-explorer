//! W7c part 1: land and sea columns in each band.

use crate::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fx {
    conditions: Conditions,
}
#[derive(Deserialize)]
struct Fxs {
    worlds: Vec<Fx>,
}

fn world(label: &str) -> Conditions {
    let f: Fxs =
        serde_json::from_str(include_str!("../../../tests/fixtures/weather/worlds.json")).unwrap();
    f.worlds
        .into_iter()
        .find(|w| w.conditions.label == label)
        .unwrap()
        .conditions
}

/// Earth's land fraction in 10° strips, south to north (approximate,
/// read off zonal land-fraction charts; a test input, not physics).
const EARTH_LAND: [f64; 18] = [
    1.0, 0.75, 0.10, 0.02, 0.03, 0.11, 0.23, 0.22, 0.24, 0.23, 0.26, 0.37, 0.43, 0.53, 0.57, 0.71,
    0.33, 0.07,
];

fn earth() -> Conditions {
    let mut e = world("Earth");
    e.land_by_lat = EARTH_LAND.to_vec();
    e
}

fn range(r: (f64, f64)) -> f64 {
    r.1 - r.0
}

#[test]
fn interiors_swing_more_than_seas_and_the_north_more_than_the_south() {
    let cs = base_settings();
    let c = orbit(&cs, &earth(), 4500.0);
    assert_eq!(c.land.len(), c.lat_deg.len());
    for lat in [42.5, 52.5, 62.5] {
        let (l, s) = (range(c.land_range_at(lat)), range(c.sea_range_at(lat)));
        assert!(l > 2.5 * s, "at {lat}°: land {l:.1} K, sea {s:.1} K");
    }
    // Observed (zonal means of land and of sea near 50° N): land about 30 to
    // 35 K, sea about 8 to 10 K.
    let (l, s) = (range(c.land_range_at(52.5)), range(c.sea_range_at(52.5)));
    assert!(
        (20.0..45.0).contains(&l) && (4.0..15.0).contains(&s),
        "land {l:.1}, sea {s:.1}"
    );
    // The land-rich north's bands swing far more than the sea-girt south's.
    let band = |lat: f64| c.lerp(&c.t_max_k, lat) - c.lerp(&c.t_min_k, lat);
    assert!(
        band(50.0) > 2.0 * band(-50.0),
        "{} vs {}",
        band(50.0),
        band(-50.0)
    );
    assert!(c
        .why
        .iter()
        .any(|w| w.contains("Land and sea warm at their own pace")));
}

#[test]
fn the_band_is_its_land_and_its_sea() {
    let cs = base_settings();
    let c = orbit(&cs, &earth(), 4500.0);
    for s in &c.seasons {
        for j in 0..c.lat_deg.len() {
            let f = c.land[j];
            let mix = f * s.t_land_k[j] + (1.0 - f) * s.t_sea_k[j];
            assert!((mix - s.t_k[j]).abs() < 1e-9, "band {j}");
        }
    }
    // The model moves heat; it does not change how warm the world is.
    let mean = c.mean_of(&c.t_k);
    assert!((mean - c.conditions.t_surface_k).abs() < 1.0, "{mean}");
}

#[test]
fn more_coasts_couple_land_and_sea_more_tightly() {
    let cs = base_settings();
    let mut e = earth();
    let two = orbit(&cs, &e, 4500.0);
    e.coasts_by_lat = vec![6.0; 18];
    let six = orbit(&cs, &e, 4500.0);
    let gap = |c: &Climate| range(c.land_range_at(52.5)) - range(c.sea_range_at(52.5));
    assert!(gap(&six) < gap(&two), "{} vs {}", gap(&six), gap(&two));
}

#[test]
fn strips_are_resampled_onto_the_bands() {
    let cs = base_settings();
    let a = orbit(&cs, &earth(), 4500.0);
    let mut e = world("Earth");
    e.land_by_lat = EARTH_LAND.iter().flat_map(|x| [*x, *x]).collect();
    let b = orbit(&cs, &e, 4500.0);
    assert_eq!(a.land, b.land);
    assert_eq!(a.t_k, b.t_k);
}

#[test]
fn worlds_without_both_keep_one_column() {
    let cs = base_settings();
    // No surface liquid: one column, as before W7c.
    for label in ["Mars", "Venus", "Moon"] {
        let c = orbit(&cs, &world(label), 4500.0);
        assert!(c.land.is_empty() && c.annual.t_land_k.is_empty(), "{label}");
        assert_eq!(c.t_land_at(30.0), c.t_at(30.0));
    }
    // A water world: every band all sea, the land column tied to it.
    let mut w = world("Earth");
    w.land_by_lat = vec![0.0; 18];
    let c = orbit(&cs, &w, 4500.0);
    for s in &c.seasons {
        assert_eq!(s.t_land_k, s.t_sea_k);
    }
    assert!(range(c.sea_range_at(50.0)) < 15.0);
}
