//! W7b: the circulation as process strengths (review section 8's tests).
use crate::strengths::strengths;
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

fn worlds() -> Vec<Conditions> {
    let f: Fxs =
        serde_json::from_str(include_str!("../../../tests/fixtures/weather/worlds.json")).unwrap();
    f.worlds.into_iter().map(|w| w.conditions).collect()
}

fn world(label: &str) -> Conditions {
    worlds().into_iter().find(|c| c.label == label).unwrap()
}

#[test]
fn solar_system_labels_are_unchanged() {
    let cs = base_settings();
    let want = [
        ("Earth", "earth-like"),
        ("Mars", "thin-air"),
        ("Venus", "slow-rotator"),
        ("Titan", "slow-rotator"),
        ("Moon", "airless"),
        ("Jupiter", "banded-giant"),
        ("Locked temperate world", "tidally-locked"),
        ("Fast-spinning Earth", "fast-rotator"),
    ];
    for (w, r) in want {
        assert_eq!(circulation::circulation(&cs, &world(w)).regime, r, "{w}");
    }
}

#[test]
fn venus_super_rotates_and_earth_does_not() {
    let cs = base_settings();
    let v = strengths(&cs, &world("Venus"));
    let e = strengths(&cs, &world("Earth"));
    assert!(v.super_rotation > 0.9, "Venus {}", v.super_rotation);
    assert!(e.super_rotation < 0.01, "Earth {}", e.super_rotation);
    // Mitchell and Vallis's quoted values, to an order of magnitude.
    assert!((0.01..0.06).contains(&e.thermal_rossby));
    assert!(v.thermal_rossby > 300.0);
    // Earth's storms grow in days, faster than its air cools.
    assert!(e.baroclinic > 0.9);
}

/// Earth's air spun from 1 h to locked: every strength and length moves
/// smoothly; neighbouring spins (5 % apart) never differ by more than 0.05 in
/// a strength, or 6 % in the Hadley edge and the eddy wind (each scales as
/// 1/Ω, so 5 % is the smooth change).
#[test]
fn a_spin_sweep_changes_every_output_smoothly() {
    let cs = base_settings();
    let mut c = world("Earth");
    let year_h = c.year_days * 24.0;
    let mut p = 1.0;
    let mut prev: Option<(strengths::Strengths, circulation::Circulation)> = None;
    while p <= year_h * 1.0001 {
        c.rotation_hours = p.min(year_h);
        let s = strengths(&cs, &c);
        let k = circulation::circulation(&cs, &c);
        if let Some((s0, k0)) = &prev {
            for (a, b, name) in [
                (s0.super_rotation, s.super_rotation, "super-rotation"),
                (s0.baroclinic, s.baroclinic, "baroclinic"),
                (s0.day_night, s.day_night, "day-night"),
                (s0.collapse, s.collapse, "collapse"),
            ] {
                assert!((a - b).abs() < 0.05, "{name} jumps {a} -> {b} at {p:.1} h");
            }
            let h = k.hadley_raw_deg / k0.hadley_raw_deg;
            assert!((0.94..1.06).contains(&h), "Hadley edge jumps at {p:.1} h");
            let r = s.eddy_wind_m_s / s0.eddy_wind_m_s;
            assert!((0.94..1.06).contains(&r), "eddy wind jumps at {p:.1} h");
        }
        prev = Some((s, k));
        p *= 1.05;
    }
    // Locked at the same spin: the same air, no jump.
    let (s0, _) = prev.unwrap();
    c.rotation_hours = year_h;
    c.star_locked = true;
    let s = strengths(&cs, &c);
    assert!((s.day_night - s0.day_night).abs() < 0.05);
    assert!((s.super_rotation - s0.super_rotation).abs() < 1e-9);
}

#[test]
fn a_cold_locked_world_collapses_and_a_warm_one_does_not() {
    let cs = base_settings();
    let mut cold = world("Locked temperate world");
    cold.gases = [("co2".to_string(), 1.0)].into_iter().collect();
    cold.mean_molar_kg = 0.044;
    cold.pressure_bar = 0.01;
    cold.t_surface_k = 175.0;
    cold.flux_w_m2 = 250.0;
    cold.liquid = None;
    let s = strengths(&cs, &cold);
    assert!(
        s.collapse > 0.5,
        "cold {} night {} frost {:?}",
        s.collapse,
        s.night_k,
        s.condense_k
    );
    let warm = world("Locked temperate world");
    let w = strengths(&cs, &warm);
    assert!(w.collapse < 0.05, "warm {}", w.collapse);
    // Thicker air carries more heat to the night side.
    let mut thick = cold.clone();
    thick.pressure_bar = 1.0;
    assert!(strengths(&cs, &thick).day_night < s.day_night);
}

#[test]
fn pluto_stays_collisional_and_the_moon_does_not() {
    let cs = base_settings();
    let mut pluto = world("Mars");
    pluto.label = "Pluto".into();
    pluto.radius_m = 1.188e6;
    pluto.gravity_m_s2 = 0.62;
    pluto.pressure_bar = 1.0e-5;
    pluto.t_surface_k = 40.0;
    pluto.mean_molar_kg = 0.028;
    pluto.gases = [("n2".to_string(), 0.99), ("ch4".to_string(), 0.005)]
        .into_iter()
        .collect();
    let p = strengths(&cs, &pluto);
    assert!(p.collisional > 0.99, "Pluto Kn {}", p.knudsen);
    let m = strengths(&cs, &world("Moon"));
    assert!(m.collisional < 0.01);
    let u = circulation::circulation(&cs, &world("Moon")).prevailing_wind(10.0);
    assert!(u.abs() < 0.05, "Moon wind {u}");
}

/// Tilted worlds: the energy balance puts the warmth where the sun is over a
/// year (poles as warm as the equator near 54°, warmer beyond), and the rain
/// belt chases the overhead sun into the summer hemisphere.
#[test]
fn a_steep_tilt_moves_the_warmth_and_the_rain() {
    let cs = base_settings();
    let mut c = world("Earth");
    let n_of = |k: &Climate| k.lat_deg.len();
    c.obliquity_deg = 54.0;
    let k = orbit(&cs, &c, 0.0);
    let n = n_of(&k);
    assert!(
        (k.t_k[n - 1] - k.t_k[n / 2]).abs() < 8.0,
        "54°: pole {} equator {}",
        k.t_k[n - 1],
        k.t_k[n / 2]
    );
    c.obliquity_deg = 98.0;
    let k = orbit(&cs, &c, 0.0);
    assert!(k.t_k[n - 1] > k.t_k[n / 2] + 10.0);
    assert!(
        k.precip_m_yr[n - 1] > k.precip_m_yr[n / 2],
        "the summer pole rains"
    );
    // In each season the wettest band is in the summer hemisphere.
    for s in &k.seasons {
        let j = (0..n).fold(0, |b, j| if s.precip[j] > s.precip[b] { j } else { b });
        let warm = (0..n).fold(0, |b, j| if s.t_k[j] > s.t_k[b] { j } else { b });
        assert!(
            s.lat_deg[j] * s.lat_deg[warm] >= 0.0,
            "rain at {} sun at {}",
            s.lat_deg[j],
            s.lat_deg[warm]
        );
    }
    // Earth's own rain stays tropical.
    let e = orbit(&cs, &world("Earth"), 0.0);
    assert!(e.precip_m_yr[n / 2] > 2.0 * e.precip_m_yr[n - 1]);
}

/// An eccentric orbit: Mars is nearest the Sun in southern summer (solar
/// longitude 251°), so its south has the hotter summers.
#[test]
fn an_eccentric_orbit_makes_unequal_summers() {
    let cs = base_settings();
    let m = world("Mars");
    let k = orbit(&cs, &m, 0.0);
    let n = k.lat_deg.len();
    assert!(
        k.t_max_k[0] > k.t_max_k[n - 1] + 10.0,
        "south {} north {}",
        k.t_max_k[0],
        k.t_max_k[n - 1]
    );
    // A circular orbit gives the old seasons exactly.
    let mut c = world("Earth");
    c.eccentricity = 0.0;
    c.perihelion_lon_deg = 0.0;
    for i in 0..48 {
        let p = i as f64 / 48.0;
        let (l, f) = ebm::orbit_at(&c, p);
        assert!((l - 360.0 * p).abs() < 1e-9 && (f - 1.0).abs() < 1e-12);
    }
    // Dates keep their seasons: phase 0 is the northern spring equinox, and
    // Earth is nearest the Sun in early January (just after phase 0.78).
    let e = world("Earth");
    assert!(ebm::orbit_at(&e, 0.0).0.abs() < 1e-9);
    let near = (0..400).map(|i| i as f64 / 400.0).fold(0.0, |b: f64, p| {
        if ebm::orbit_at(&e, p).1 > ebm::orbit_at(&e, b).1 {
            p
        } else {
            b
        }
    });
    assert!((0.76..0.82).contains(&near), "{near}");
}

/// A world past a paper's tested range is marked, not refused: Earth is
/// inside Mitchell and Vallis's runs; Venus's Ro_T is far above them.
#[test]
fn worlds_outside_a_papers_range_are_marked() {
    let cs = base_settings();
    let e = strengths(&cs, &world("Earth"));
    assert!(
        !e.outside.iter().any(|x| x.starts_with("super-rotation")),
        "{:?}",
        e.outside
    );
    let v = strengths(&cs, &world("Venus"));
    assert!(
        v.outside.iter().any(|x| x.starts_with("super-rotation")),
        "{:?}",
        v.outside
    );
    assert!(v.why.iter().any(|x| x.contains("extrapolation")));
}
