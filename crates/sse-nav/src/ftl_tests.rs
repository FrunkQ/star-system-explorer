//! FTL rules as data: where a leg may begin.

use crate::ftl::*;
use crate::scenario::sol;
use sse_core::math as m;

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    m::sqrt(
        (a[0] - b[0]) * (a[0] - b[0])
            + (a[1] - b[1]) * (a[1] - b[1])
            + (a[2] - b[2]) * (a[2] - b[2]),
    )
}

fn traveller() -> FtlRule {
    serde_json::from_value(serde_json::json!({
        "id": "jump", "name": "Jump drive", "family": "jump",
        "start": [{"kind": "clear_of", "multiple": 100.0, "of": "diameter", "bodies": {"which": "every"}}],
        "end": [{"kind": "clear_of", "multiple": 100.0, "of": "diameter", "bodies": {"which": "every"}}],
        "crew": "awake, a week in jump space", "cost": "fuel by the parsec",
        "duration": {"form": "fixed", "days": 7.0, "spread_days": 1.0},
        "risks": ["misjump on unrefined fuel"], "messages": {"by": "ships"}, "fictional": true
    }))
    .unwrap()
}

#[test]
fn a_hundred_diameters_of_every_world_and_the_star() {
    let sys = sol();
    let map = jump_map(&sys, &traveller().start, sys.t0_s);
    let earth = map
        .keepouts
        .iter()
        .find(|k| k.name == "Earth")
        .expect("Earth's limit");
    assert!(
        (earth.radius_m - 100.0 * 2.0 * 6.371e6).abs() < 1.0e6,
        "{}",
        earth.radius_m
    );
    let sun = &map.keepouts[0];
    assert!(
        (sun.radius_m / 1.496e11 - 0.93).abs() < 0.01,
        "the Sun's limit is about 0.93 au: {}",
        sun.radius_m / 1.496e11
    );
    // A ship parked in low Earth orbit may not jump; one 3 au out, clear of every limit, may.
    let e = sys.body("earth").unwrap();
    let (r, _) = sys.at(e, sys.t0_s);
    assert!(!may_begin(&map, [r[0] + 7.0e6, r[1], r[2]]));
    assert!(may_begin(&map, [0.0, 0.0, 3.0 * 1.496e11 + 1.0e10]));
    assert_eq!(
        map.words,
        vec!["clear of the star and every world by 100 diameters".to_string()]
    );
}

#[test]
fn lagrange_points_sit_where_the_pulls_balance() {
    let sys = sol();
    let places = vec![Place::Lagrange {
        points: vec![1, 4, 5],
        bodies: Which::Massive { min_earth: 1.0 },
        within: None,
    }];
    let map = jump_map(&sys, &places, sys.t0_s);
    let e = sys.body("earth").unwrap();
    let (r, _) = sys.at(e, sys.t0_s);
    let d = dist(r, [0.0; 3]);
    let pts: Vec<_> = map.points.iter().filter(|p| p.name == "Earth").collect();
    assert_eq!(pts.len(), 3);
    // L4 and L5: on Earth's orbit, 60° from Earth, so as far from Earth as from the Sun.
    for p in pts.iter().filter(|p| p.what == "L4" || p.what == "L5") {
        let from_earth = dist(p.at_m, r);
        assert!((from_earth / d - 1.0).abs() < 1e-9, "{}", from_earth / d);
    }
    // L1: Earth's Hill radius sunward, about 1.5 million km.
    let l1 = pts.iter().find(|p| p.what == "L1").unwrap();
    let from_earth = dist(l1.at_m, r);
    assert!((from_earth / 1.5e9 - 1.0).abs() < 0.05, "{from_earth}");
    // Mercury and Mars are under an Earth mass: no points of their own.
    assert!(!map.points.iter().any(|p| p.name == "Mars"));
    assert!(may_begin(&map, l1.at_m));
    assert!(!may_begin(&map, r));
}

#[test]
fn no_places_means_anywhere_and_gates_are_named_for_the_caller() {
    let sys = sol();
    assert!(jump_map(&sys, &[], sys.t0_s).anywhere);
    let map = jump_map(
        &sys,
        &[Place::Gate {
            construct: "ring gate".into(),
            linked: true,
        }],
        sys.t0_s,
    );
    assert_eq!(map.gates, vec!["ring gate".to_string()]);
    assert!(!map.anywhere);
}

#[test]
fn how_long_a_leg_takes_is_its_own_field() {
    assert_eq!(traveller().duration.home_days(3.0, 3.0), 7.0);
    assert_eq!(traveller().aboard_per_home, 1.0);
    assert_eq!(Duration::Instant.home_days(10.0, 1.0), 0.0);
    assert!(
        (Duration::PerParsec {
            days_at_rating_1: 30.0
        }
        .home_days(2.0, 2.0)
            - 30.0)
            .abs()
            < 1e-12
    );
    // Warp 1 is light speed: a parsec takes 3.26 years; the rating cubed at warp 2 makes it 8 times quicker.
    let w = Duration::Formula {
        times_light: 1.0,
        power: 3.0,
    };
    assert!(
        (w.home_days(1.0, 1.0) / 365.25 - 3.2616).abs() < 1e-3,
        "{}",
        w.home_days(1.0, 1.0) / 365.25
    );
    assert!((w.home_days(1.0, 1.0) / w.home_days(1.0, 2.0) - 8.0).abs() < 1e-9);
}

#[test]
fn out_of_the_well_by_gravity_and_at_the_poles() {
    let sys = sol();
    let map = jump_map(
        &sys,
        &[Place::Gravity {
            m_s2: 1.0e-4,
            above: false,
        }],
        sys.t0_s,
    );
    // Near Earth gravity is far above 1e-4 m/s²; at 5 au from the Sun it is about 2.4e-4; at 10 au, 6e-5.
    let e = sys.body("earth").unwrap();
    let (r, _) = sys.at(e, sys.t0_s);
    assert!(!may_begin_in(&map, Some(&sys), r));
    assert!(may_begin_in(&map, Some(&sys), [0.0, 0.0, 10.0 * 1.496e11]));
    let poles = jump_map(
        &sys,
        &[Place::Poles {
            au: 5.0,
            within_au: 0.5,
        }],
        sys.t0_s,
    );
    assert_eq!(poles.points.len(), 2);
    assert!(may_begin(&poles, [0.0, 0.0, -5.2 * 1.496e11]));
}
