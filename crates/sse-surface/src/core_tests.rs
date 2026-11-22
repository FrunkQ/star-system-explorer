//! MF1: core heat histories against what the planets show.

use crate::scenario::{fixtures, settings};

fn core_of(name: &str) -> crate::core::CoreHistory {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .expect("fixture")
        .body;
    crate::envelope(&settings(), &b).core.expect("an iron core")
}

fn ago(c: &crate::core::CoreHistory, kind: &str) -> Vec<f64> {
    let age = c.steps.last().expect("steps").t_gyr;
    c.events
        .iter()
        .filter(|e| e.kind == kind)
        .map(|e| age - e.t_gyr)
        .collect()
}

#[test]
fn cores_are_iron_squeezed_by_their_planets() {
    // Earth 3,480 km, Mars 1,830 km (InSight), Mercury 2,020 km, Moon about 350 km.
    for (w, want, tol) in [
        ("Earth", 3480.0, 0.03),
        ("Mars", 1830.0, 0.05),
        ("Mercury", 2020.0, 0.08),
        ("Moon", 350.0, 0.15),
    ] {
        let c = core_of(w);
        let r = c.core_radius_km;
        assert!((r / want - 1.0).abs() < tol, "{w}: core radius {r:.0} km");
    }
    let e = core_of("Earth");
    assert!(
        (110.0..150.0).contains(&e.cmb_pressure_gpa),
        "{}",
        e.cmb_pressure_gpa
    );
}

#[test]
fn earths_field_never_stops_and_its_inner_core_is_young() {
    let c = core_of("Earth");
    assert!(c.dynamo_now, "{}", c.why_now);
    assert!(ago(&c, "dynamo-death").is_empty(), "{:?}", c.events);
    let started = ago(&c, "dynamo-start");
    assert!(started[0] > 3.5, "{started:?}");
    let born = ago(&c, "inner-core-birth");
    assert_eq!(born.len(), 1, "{:?}", c.events);
    assert!(
        (0.5..1.5).contains(&born[0]),
        "inner core born {} Gyr ago",
        born[0]
    );
    // The core's top stays liquid.
    let now = c.steps.last().expect("steps");
    assert!(
        now.inner_core_km < c.core_radius_km * 0.95,
        "{}",
        now.inner_core_km
    );
}

#[test]
fn mars_field_dies_young() {
    let c = core_of("Mars");
    assert!(!c.dynamo_now);
    let died = ago(&c, "dynamo-death");
    assert_eq!(died.len(), 1, "{:?}", c.events);
    assert!((3.6..4.1).contains(&died[0]), "died {} Gyr ago", died[0]);
    assert!(
        c.steps.iter().all(|s| s.inner_core_km == 0.0),
        "Mars' core stays liquid"
    );
}

#[test]
fn venus_has_no_field_because_its_core_is_not_cooling() {
    let c = core_of("Venus");
    assert!(!c.dynamo_now);
    assert!(c.why_now.contains("not cooling"), "{}", c.why_now);
    let now = c.steps.last().expect("steps");
    assert!(now.q_cmb_w_m2 < now.q_adiabat_w_m2);
    assert!(ago(&c, "dynamo-death")[0] > 0.5);
}

#[test]
fn every_change_has_a_dated_reason() {
    for w in ["Earth", "Mars", "Venus", "Mercury", "Moon"] {
        let c = core_of(w);
        for e in &c.events {
            assert!(e.why.contains("Gyr ago"), "{w}: {}", e.why);
        }
        assert!(c.steps.windows(2).all(|p| p[1].t_gyr > p[0].t_gyr), "{w}");
    }
}
