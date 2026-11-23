//! A ship at a world (aerocapture, entry, landing, take-off).

use crate::entry::*;
use crate::scenario::{settings, sol};
use crate::Ship;

fn capsule_ship() -> Ship {
    // An Orion-sized crewed ship with a blunt ablative shield.
    serde_json::from_value(serde_json::json!({
        "name": "Capsule", "accel_g": 0.5, "exhaust_km_s": 3.1,
        "wet_t": 10.0, "dry_t": 9.0, "crew": 4,
        "shield": {"nose_radius_m": 6.0, "max_heat_flux_w_m2": 4.0e6,
                   "max_heat_load_j_m2": 4.0e8, "kind": "ablative"},
        "aero": {"ref_area_m2": 19.6, "cd": 1.3, "l_over_d": 0.3},
        "landing": {"legs": false, "max_touchdown_m_s": 9.0,
                    "chute": {"area_m2": 3000.0, "cd": 0.8, "max_mach": 0.8, "max_q_pa": 12000.0, "fill_s": 12.0}}
    }))
    .unwrap()
}

fn at(ship: &Ship, name: &str) -> AtWorld {
    let s = settings();
    let sys = sol();
    let b = sys.body(name).unwrap();
    let (v_inf, why) = hohmann_arrival(&sys, b);
    at_world(&s.entry, &s.descent, ship, b, b.air.as_ref(), v_inf, why)
}

fn check<'a>(w: &'a AtWorld, id: &str) -> &'a Check {
    w.checks.iter().find(|c| c.id == id).unwrap()
}

#[test]
fn a_capsule_aerocaptures_at_earth_in_a_real_corridor() {
    let w = at(&capsule_ship(), "Earth");
    let c = check(&w, "aerocapture");
    let f = &c.figures;
    // Arriving from Venus at about 2.5 km/s over escape.
    assert!((2000.0..3500.0).contains(&w.v_inf_m_s), "{}", w.v_inf_m_s);
    let h = f["pass_height_m"];
    assert!((40_000.0..140_000.0).contains(&h), "{h}");
    assert!(f["corridor_low_m"] < h && h < f["corridor_high_m"]);
    assert!(f["propellant_saved_t"] > 0.0);
    assert!(c
        .dangers
        .iter()
        .all(|d| d.chance.p >= 0.0 && d.chance.p <= 1.0));
    assert!(c.verdict != "not-here");
}

#[test]
fn peak_g_of_a_pass_does_not_depend_on_the_ship_shape() {
    let mut a = capsule_ship();
    let w1 = at(&a, "Mars");
    a.aero.as_mut().unwrap().ref_area_m2 *= 4.0;
    let w2 = at(&a, "Mars");
    let (g1, g2) = (
        check(&w1, "aerocapture").figures["peak_g"],
        check(&w2, "aerocapture").figures["peak_g"],
    );
    assert!((g1 - g2).abs() < 0.02 * g1, "{g1} {g2}");
    // A draggier ship passes higher.
    assert!(
        check(&w2, "aerocapture").figures["pass_height_m"]
            > check(&w1, "aerocapture").figures["pass_height_m"]
    );
}

#[test]
fn giants_have_no_ground_and_airless_worlds_no_aerobraking() {
    let ship = capsule_ship();
    let j = at(&ship, "Jupiter");
    assert_eq!(check(&j, "entry").verdict, "not-here");
    assert_eq!(check(&j, "take-off").verdict, "not-here");
    assert_ne!(check(&j, "aerocapture").verdict, "not-here");
    let m = at(&ship, "Mercury");
    assert_eq!(check(&m, "aerocapture").verdict, "not-here");
    assert_ne!(check(&m, "take-off").verdict, "not-here");
}

#[test]
fn a_weak_drive_is_shown_as_a_danger_not_refused() {
    let s = settings();
    let chem = s.ships["chemical-transfer"].clone();
    let w = at(&chem, "Earth");
    let t = check(&w, "take-off");
    let thrust = t
        .dangers
        .iter()
        .find(|d| d.id == "take-off.thrust")
        .unwrap();
    assert_eq!(thrust.level, "danger");
    assert!(thrust.chance.p > 0.99);
    assert!(!t.would_pass.is_empty());
    // The same ship at Mercury's lower gravity is still under one g of
    // weight: 0.3 g of thrust against 0.38 g.
    let m = at(&chem, "Mercury");
    assert_eq!(check(&m, "take-off").verdict, "dangerous");
}

#[test]
fn ships_without_shape_numbers_are_guessed_and_say_so() {
    let s = settings();
    let w = at(&s.ships["fusion-torch"], "Mars");
    let c = check(&w, "aerocapture");
    assert!(c.why.iter().any(|y| y.contains("no shape numbers")));
}

#[test]
fn the_entry_reuses_the_descent_and_lands_a_capsule_on_earth() {
    let w = at(&capsule_ship(), "Earth");
    let e = check(&w, "entry");
    assert!(
        e.figures["peak_g"] > 1.0 && e.figures["peak_g"] < 15.0,
        "{:?}",
        e.figures
    );
    assert!(e.why.iter().any(|y| y.starts_with("descent result")));
    assert!(e.figures["touchdown_m_s"] < 12.0, "{:?}", e.figures);
}

#[test]
fn brief_tolerance_falls_with_time_but_not_under_the_hours_limit() {
    let s = settings().entry;
    assert!(brief_g(&s, 1.0, 3.0) > brief_g(&s, 100.0, 3.0));
    assert!(brief_g(&s, 1.0e5, 3.0) >= 3.0);
    let mid = brief_g(&s, 3.0, 3.0);
    assert!(mid < 25.0 && mid > 15.0, "{mid}");
}

#[test]
fn ship_fields_are_optional_and_round_trip() {
    let s = settings();
    let t = &s.ships["fusion-torch"];
    let j = serde_json::to_value(t).unwrap();
    assert!(j.get("shield").is_none());
    let c = capsule_ship();
    let back: Ship = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    assert_eq!(back, c);
}
