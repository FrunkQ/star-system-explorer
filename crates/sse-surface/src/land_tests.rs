//! W7c part 2: land and coasts by latitude from the height map.

use crate::climate::land_strips;
use crate::plates::Grid;
use crate::scenario::{fixtures, settings};
use crate::survey::{survey_world, SurveyOptions};
use sse_core::draw::Seed;

#[test]
fn a_half_land_world_has_half_land_and_two_coasts_in_every_strip() {
    let g = Grid::new(48);
    // Land east of the 0° meridian, sea west of it.
    let elev: Vec<f64> = g
        .dirs
        .iter()
        .map(|d| if d[1] > 0.0 { 100.0 } else { -100.0 })
        .collect();
    let (land, coasts) = land_strips(&g, &elev, 18);
    for i in 2..16 {
        assert!((land[i] - 0.5).abs() < 0.06, "strip {i}: land {}", land[i]);
        assert!(
            (coasts[i] - 2.0).abs() < 0.4,
            "strip {i}: coasts {}",
            coasts[i]
        );
    }
}

#[test]
fn a_polar_cap_world_has_no_coasts_at_the_equator() {
    let g = Grid::new(48);
    let elev: Vec<f64> = g
        .dirs
        .iter()
        .map(|d| if d[2] > 0.8 { 100.0 } else { -100.0 })
        .collect();
    let (land, coasts) = land_strips(&g, &elev, 18);
    assert!(land[9] == 0.0 && coasts[9] == 0.0);
    assert!(land[17] == 1.0);
}

#[test]
fn a_surveyed_earth_runs_its_own_land_and_sea() {
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    let seed = Seed::from_text("sse4-land-tests");
    let f = fixtures()
        .into_iter()
        .find(|f| f.body.label == "Earth")
        .unwrap();
    let sv = survey_world(
        &s,
        &f.body,
        &seed,
        f.body.age_gyr,
        &SurveyOptions::default(),
    )
    .unwrap();
    let cl = &sv.climate.as_ref().expect("a climate").climate;
    let c = &cl.conditions;
    assert_eq!(c.land_by_lat.len(), s.climate.bands as usize);
    assert_eq!(c.coasts_by_lat.len(), s.climate.bands as usize);
    // The climate's sea is the map's sea.
    let mean_land = c.land_by_lat.iter().sum::<f64>() / c.land_by_lat.len() as f64;
    assert!(
        c.ocean_fraction > 0.3 && c.ocean_fraction < 0.95,
        "{}",
        c.ocean_fraction
    );
    assert!(mean_land > 0.0 && c.coasts_by_lat.iter().any(|k| *k > 0.0));
    assert_eq!(cl.land.len(), cl.lat_deg.len());
}
