//! W9 part 2: the spin axis under the orbit's wobble, and ice ages.
use crate::forcing::*;
use crate::*;
use serde::Deserialize;
use sse_time::secular::{Planet, Secular};

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
        .map(|w| w.conditions)
        .find(|c| c.label == label)
        .unwrap()
}

const D: f64 = std::f64::consts::PI / 180.0;

/// Venus to Saturn at J2000 (mean elements on the ecliptic, Standish's JPL
/// table): mass (Sun = 1), a, e, ϖ, i, Ω.
fn solar_system() -> Vec<Planet> {
    [
        (2.4478e-6, 0.72333, 0.00677, 131.53, 3.3947, 76.68),
        (3.0035e-6, 1.00000, 0.01671, 102.94, 0.00005, 348.74),
        (3.2272e-7, 1.52368, 0.09340, 336.04, 1.8506, 49.58),
        (9.5479e-4, 5.20260, 0.04849, 14.75, 1.3033, 100.46),
        (2.8588e-4, 9.55491, 0.05551, 92.43, 2.4889, 113.67),
    ]
    .iter()
    .map(|&(mass, a, e, w, i, n)| Planet {
        mass,
        a,
        e,
        varpi: w * D,
        i: i * D,
        node: n * D,
    })
    .collect()
}

fn earth_track(span_yr: f64, every_yr: f64) -> OrbitTrack {
    let ps = solar_system();
    let sec = Secular::new(1.0, &ps);
    let n = (span_yr / every_yr) as usize;
    let samples = (0..=n)
        .map(|k| {
            let t = k as f64 * every_yr;
            let p = sec.advance(&ps, t)[1];
            TrackSample {
                t_yr: t,
                e: p.e,
                varpi: p.varpi,
                incl: p.i,
                node: p.node,
            }
        })
        .collect();
    OrbitTrack {
        samples,
        source: "T1 secular (Venus to Saturn)".into(),
    }
}

fn earth_spin(span_yr: f64) -> Vec<SpinSample> {
    let cs = base_settings();
    let c = world("Earth");
    let moon = MoonPull {
        mass_kg: 7.342e22,
        a_m: 3.844e8,
    };
    let kb = precession_parts(&cs, &c, star_kg(1.0), 1.0, &[moon]);
    spin_history(&earth_track(span_yr, 500.0), 23.44, 283.0, kb, 50.0, 500.0)
}

#[test]
fn earths_precession_constant_comes_out_of_the_formula() {
    let cs = base_settings();
    let c = world("Earth");
    let moon = MoonPull {
        mass_kg: 7.342e22,
        a_m: 3.844e8,
    };
    let (k, beta) = precession_parts(&cs, &c, star_kg(1.0), 1.0, &[moon]);
    let e: f64 = 0.0167;
    let psi = k * (sse_core::math::pow(1.0 - e * e, -1.5) + beta) * sse_core::math::cos(23.44 * D)
        / D
        * 3600.0;
    // Zeebe 2022 Table 1: 50.38 arcsec a year.
    assert!((psi - 50.38).abs() < 1.5, "{psi} arcsec/yr");
    // Without the Moon the Sun alone gives about a third.
    let (k2, b2) = precession_parts(&cs, &c, star_kg(1.0), 1.0, &[]);
    assert_eq!(b2, 0.0);
    assert!(k2 == k);
}

#[test]
fn earths_tilt_swings_on_a_41_kyr_beat() {
    let spin = earth_spin(1.0e6);
    let ob: Vec<f64> = spin.iter().map(|s| s.obliquity_deg).collect();
    let (lo, hi) = ob
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| {
            (a.min(*x), b.max(*x))
        });
    // About 22.1° to 24.5° (Laskar 2004); secular theory to 5 planets: close.
    assert!(lo > 21.0 && hi < 25.5 && hi - lo > 1.0, "{lo}..{hi}");
    // Peaks a million years apart: about 1e6 / 41e3 ≈ 24.
    let peaks = ob
        .windows(3)
        .filter(|w| w[1] > w[0] && w[1] >= w[2])
        .count();
    assert!((20..=28).contains(&peaks), "{peaks} tilt peaks in 1 Myr");
    // Climatic precession: perihelion against the equinox goes round in
    // about 21 kyr, so about 45 to 50 turns a million years.
    let turns = spin
        .windows(2)
        .filter(|w| w[1].perihelion_lon_deg < w[0].perihelion_lon_deg - 180.0)
        .count();
    assert!((40..=55).contains(&turns), "{turns} precession turns");
}

#[test]
fn a_lone_world_has_no_cycles() {
    let cs = base_settings();
    let c = world("Earth");
    let kb = precession_parts(&cs, &c, star_kg(1.0), 1.0, &[]);
    let spin = spin_history(
        &OrbitTrack::fixed(0.0167, 2.0e5),
        23.44,
        283.0,
        kb,
        100.0,
        2000.0,
    );
    for s in &spin {
        assert!(
            (s.obliquity_deg - 23.44).abs() < 1e-6,
            "{}",
            s.obliquity_deg
        );
    }
}

/// The review's test: an obliquity-cycle fixture writes the expected number
/// of glacial advances. Earth on a circular orbit whose plane wobbles in one
/// mode (Earth's s3, −18.85″ a year, 2°): the tilt alone cycles, at
/// 2π / (Ψ + s). A place on the edge of the ice: one advance per tilt cycle;
/// a place deep under ice or far from it: none.
#[test]
fn the_tilt_cycle_writes_one_glacial_advance_per_cycle() {
    let cs = base_settings();
    let mut c = world("Earth");
    c.eccentricity = 0.0;
    let cl = orbit(&cs, &c, 4500.0);
    let s3 = -18.85 / 3600.0 * D;
    let samples = (0..=800)
        .map(|k| {
            let t = k as f64 * 500.0;
            TrackSample {
                t_yr: t,
                e: 0.0,
                varpi: 0.0,
                incl: 2.0 * D,
                node: s3 * t,
            }
        })
        .collect();
    let track = OrbitTrack {
        samples,
        source: "one inclination mode".into(),
    };
    let moon = MoonPull {
        mass_kg: 7.342e22,
        a_m: 3.844e8,
    };
    let (k, beta) = precession_parts(&cs, &c, star_kg(1.0), 1.0, &[moon]);
    let psi = k * (1.0 + beta) * sse_core::math::cos(23.44 * D);
    let period = 2.0 * std::f64::consts::PI / (psi + s3);
    let spin = spin_history(&track, 23.44, 283.0, (k, beta), 50.0, 1000.0);
    let model = ice_age_model(&cs, &cl, 4500.0, spin, "T1 secular");
    let (lo, hi) = model.obliquity_range();
    // Find the latitude in the north where the ice balance at sea level is
    // zero today: the edge of the ice.
    let bal = |l: f64| water::water_at(&cs, &cl, 6.5, l, 0.0).ice_balance_m_yr;
    let (mut a, mut b) = (45.0, 89.0);
    assert!(bal(a) < 0.0 && bal(b) > 0.0);
    for _ in 0..50 {
        let mid = 0.5 * (a + b);
        if bal(mid) > 0.0 {
            b = mid
        } else {
            a = mid
        }
    }
    let edge = model.at(&cs, &cl, 6.5, b, 0.0);
    // Lower tilt, cooler summers: more ice.
    assert!(
        water::water_at(&cs, &model.probe, 6.5, b, 0.0).ice_balance_m_yr > bal(b),
        "edge at {b:.1}°"
    );
    let cycles = 4.0e5 / period;
    assert!((38.0e3..45.0e3).contains(&period), "tilt period {period}");
    assert!(
        (edge.advances as f64 - cycles).abs() <= 1.0,
        "{} advances in 400 kyr (tilt {lo:.2}..{hi:.2}), expected about {cycles:.0}",
        edge.advances
    );
    assert!(
        edge.glaciated_share > 0.1 && edge.glaciated_share < 0.9,
        "{}",
        edge.glaciated_share
    );
    // The dated onsets are the advances counted, in time order, about one
    // tilt period apart.
    let on = model.onsets_yr(&cs, &cl, 6.5, b, 0.0);
    assert_eq!(on.len() as u32, edge.advances);
    for w in on.windows(2) {
        assert!(w[1] > w[0] && (w[1] - w[0]) > 0.5 * period, "{on:?}");
    }
    let deep = model.at(&cs, &cl, 6.5, 85.0, 3000.0);
    assert_eq!((deep.advances, deep.glaciated_share), (0, 1.0));
    assert!(model.onsets_yr(&cs, &cl, 6.5, 85.0, 3000.0).is_empty());
    let tropics = model.at(&cs, &cl, 6.5, 0.0, 0.0);
    assert_eq!((tropics.advances, tropics.glaciated_share), (0, 0.0));
}

/// The real Solar System: tilt and precession both pace Earth's ice.
#[test]
fn earths_ice_edge_follows_tilt_and_precession() {
    let cs = base_settings();
    let c = world("Earth");
    let cl = orbit(&cs, &c, 4500.0);
    let model = ice_age_model(&cs, &cl, 4500.0, earth_spin(4.0e5), "T1 secular");
    let bal = |l: f64| water::water_at(&cs, &cl, 6.5, l, 0.0).ice_balance_m_yr;
    let (mut a, mut b) = (45.0, 89.0);
    for _ in 0..50 {
        let mid = 0.5 * (a + b);
        if bal(mid) > 0.0 {
            b = mid
        } else {
            a = mid
        }
    }
    let edge = model.at(&cs, &cl, 6.5, b, 0.0);
    // Between one per tilt cycle (about 10) and one per precession cycle
    // and its harmonics (about 19 to 40).
    assert!((8..=40).contains(&edge.advances), "{}", edge.advances);
}
