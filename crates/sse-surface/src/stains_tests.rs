use super::*;
use crate::envelope;
use crate::scenario::{fixtures, settings};
use std::collections::BTreeMap;

fn world(name: &str) -> (Body, SurfaceEnvelope) {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body;
    let e = envelope(&settings(), &b);
    (b, e)
}

fn air(g: &[(&str, f64)]) -> BTreeMap<String, f64> {
    g.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn of(name: &str, gases: &[(&str, f64)], p_bar: f64, flux: f64) -> Vec<Stain> {
    let (b, e) = world(name);
    let g = air(gases);
    stains(
        &b,
        &e,
        &StainInputs {
            gases: &g,
            pressure_bar: p_bar,
            flux_w_m2: flux,
            exchange: None,
            lat_deg: &[],
            gases_assumed: false,
        },
    )
}

fn share(s: &[Stain], kind: &str) -> f64 {
    s.iter()
        .find(|x| x.kind == kind)
        .map(|x| x.share)
        .unwrap_or(0.0)
}

#[test]
fn mars_rusts_under_its_thin_carbon_dioxide_and_the_moon_does_not() {
    // Mars: 6 mbar, 95% CO2 (NSSDCA), 586 W/m².
    let mars = of(
        "Mars",
        &[("co2", 0.95), ("n2", 0.028), ("ar", 0.02)],
        0.006,
        586.0,
    );
    assert!(share(&mars, "ferric-oxide") > 0.9, "{mars:#?}");
    assert_eq!(mars[0].material_id, "mgs-1");
    let moon = of("Moon", &[], 0.0, 1361.0);
    assert_eq!(share(&moon, "ferric-oxide"), 0.0, "{moon:#?}");
    assert_eq!(share(&moon, "tholin"), 0.0);
    let mercury = of("Mercury", &[], 0.0, 9100.0);
    assert!(mercury.is_empty(), "{mercury:#?}");
}

#[test]
fn methane_under_starlight_makes_tholin_more_where_there_is_more() {
    // Titan: 1.5 bar, 5% CH4 near the ground, 15 W/m²: thoroughly coated.
    let titan = of("Titan", &[("n2", 0.95), ("ch4", 0.05)], 1.5, 15.0);
    assert!(share(&titan, "tholin") > 0.99, "{titan:#?}");
    assert_eq!(
        share(&titan, "ferric-oxide"),
        0.0,
        "methane keeps it reduced"
    );
    // Triton: 14 µbar, 0.02% CH4, 1.5 W/m²: a pink tinge, much less than Titan.
    let triton = of("Triton", &[("n2", 0.9998), ("ch4", 2e-4)], 1.4e-5, 1.5);
    let t = share(&triton, "tholin");
    assert!(t > 0.1 && t < 0.8, "{t}");
    // Earth's trace methane in oxidising air makes no haze (Trainer et al. 2006).
    let earth = of(
        "Earth",
        &[("n2", 0.78), ("o2", 0.21), ("co2", 4.2e-4), ("ch4", 1.9e-6)],
        1.0,
        1361.0,
    );
    assert_eq!(share(&earth, "tholin"), 0.0, "{earth:#?}");
    // No methane, no tholin.
    assert_eq!(
        share(&of("Triton", &[("n2", 1.0)], 1.4e-5, 1.5), "tholin"),
        0.0
    );
}

#[test]
fn a_hot_dry_world_is_painted_with_sulphur() {
    let io = of("Io", &[("so2", 1.0)], 1e-9, 50.0);
    assert!(share(&io, "sulphur") > 0.5, "{io:#?}");
    let moon = of("Moon", &[], 0.0, 1361.0);
    assert_eq!(share(&moon, "sulphur"), 0.0);
}

#[test]
fn stains_grow_smoothly_with_their_dose() {
    let mut last = 0.0;
    for p in [1e-5, 1e-4, 1e-3, 1e-2] {
        let s = share(&of("Mars", &[("co2", 0.95)], p, 586.0), "ferric-oxide");
        assert!(s >= last, "{p}: {s} < {last}");
        last = s;
    }
}

#[test]
fn a_guessed_air_stains_nothing() {
    let (b, e) = world("Mars");
    let g = air(&[("co2", 0.95)]);
    let s = stains(
        &b,
        &e,
        &StainInputs {
            gases: &g,
            pressure_bar: 0.006,
            flux_w_m2: 586.0,
            exchange: None,
            lat_deg: &[],
            gases_assumed: true,
        },
    );
    assert_eq!(share(&s, "ferric-oxide"), 0.0, "{s:#?}");
}
