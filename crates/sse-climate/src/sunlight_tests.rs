//! HV2: the sunlight budget a climate publishes for the energy flow view.

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

#[test]
fn earth_reflects_about_three_tenths_and_the_books_close() {
    let cs = base_settings();
    let c = orbit(&cs, &world("Earth"), 4500.0);
    let s = c.sunlight();
    // Measured (CERES era): about 340 W/m² arriving, Bond albedo about 0.29,
    // about 240 absorbed. Guarded as limits, not tuned to.
    assert!(
        (330.0..350.0).contains(&s.arriving_w_m2),
        "arriving {}",
        s.arriving_w_m2
    );
    assert!(
        (0.2..0.4).contains(&s.bond_albedo),
        "albedo {}",
        s.bond_albedo
    );
    // Absolute: the books close exactly.
    assert!((s.arriving_w_m2 - s.absorbed_w_m2 - s.reflected_w_m2).abs() < 1e-9);
    assert!((s.emitted_w_m2 - s.absorbed_w_m2 - s.internal_w_m2).abs() < 1e-9);
}
