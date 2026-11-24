//! RK5: slow spirals for gentle drives, checked against published design studies.

use crate::scenario::{settings, sol};
use crate::{plan, Endpoint, Prefer, RouteRequest, Ship};

fn gentle(accel_g: f64, exhaust_km_s: f64) -> Ship {
    let mut ship = settings().ships["nuclear-thermal"].clone();
    ship.name = "Gentle".into();
    ship.accel_g = accel_g;
    ship.exhaust_km_s = exhaust_km_s;
    ship.sustained = true;
    ship.reserve = 0.0;
    ship
}

fn to_jupiter(ship: Ship) -> crate::RoutePlan {
    let sys = sol();
    let req = RouteRequest {
        ship,
        from: Endpoint::Body("Earth".into()),
        to: Some(Endpoint::Body("Jupiter".into())),
        depart_s: Some(sys.t0_s),
        window_days: 800.0,
        prefer: Prefer::Fastest,
        rescue: None,
    };
    plan(&settings(), &sys, &req).unwrap()
}

#[test]
fn a_gentle_drive_spirals_instead_of_kicking() {
    // HOPE's nuclear electric Callisto ship (NASA/TM-2003-212691 in the Paper spacecraft pack): 6e-5 g, 8,000 s.
    let p = to_jupiter(gentle(6e-5, 78.5));
    let ids: Vec<&str> = p.candidates.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"spiral"), "{ids:?} {:#?}", p.refused);
    assert!(
        !ids.iter()
            .any(|i| i.starts_with("lambert") || *i == "hohmann"),
        "short kicks are refused: {ids:?}"
    );
    assert!(
        p.refused.iter().any(|r| r.why.contains("not short kicks")),
        "{:#?}",
        p.refused
    );
    let c = p.candidates.iter().find(|c| c.id == "spiral").unwrap();
    // The study flies 832 days, to Callisto's orbit rather than a low one; the spiral should be the same order.
    assert!(
        c.days > 600.0 && c.days < 1200.0,
        "{} days: {:#?}",
        c.days,
        c.why
    );
    assert!(c.burns.iter().all(|b| b.duration_s > 0.0));
    let burning: f64 = c.burns.iter().map(|b| b.duration_s).sum();
    assert!(
        (burning - (c.arrive_s - c.depart_s)).abs() < 1.0,
        "thrust all the way"
    );
    assert_eq!(c.arrive_v_inf_m_s, 0.0);
    // Its track runs from Earth's distance out to Jupiter's.
    let r0 = crate::kepler::norm(c.track[0]) / sse_core::quantity::consts::AU;
    let r1 = crate::kepler::norm(*c.track.last().unwrap()) / sse_core::quantity::consts::AU;
    assert!(r0 < 1.1 && r1 > 4.9, "{r0} {r1}");
}

#[test]
fn a_brisk_drive_keeps_its_short_burns() {
    let p = to_jupiter(settings().ships["nuclear-thermal"].clone());
    assert!(p.candidates.iter().all(|c| c.id != "spiral"));
    assert!(
        !p.refused.iter().any(|r| r.why.contains("not short kicks")),
        "{:#?}",
        p.refused
    );
}

#[test]
fn spirals_are_repeatable() {
    let a = to_jupiter(gentle(1.5e-5, 5170.0));
    let b = to_jupiter(gentle(1.5e-5, 5170.0));
    assert_eq!(a, b);
    let c = a.candidates.iter().find(|c| c.id == "spiral").unwrap();
    // The fission fragment ship's study needs over 2,100 days; more Δv here, as it goes down to a low orbit.
    assert!(c.days > 2100.0, "{}", c.days);
}
