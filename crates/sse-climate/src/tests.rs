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
fn earth_is_warm_at_the_equator_and_wet_in_the_tropics() {
    let cs = base_settings();
    let e = orbit(&cs, &world("Earth"), 4500.0);
    let n = e.lat_deg.len();
    assert!(e.t_k[n / 2] > e.t_k[0] + 25.0 && e.t_k[n / 2] > e.t_k[n - 1] + 25.0);
    let mean = e.mean_of(&e.precip_m_yr);
    assert!(mean > 0.5 && mean < 2.0, "Earth's mean rain {mean} m/yr");
    // Seasons: the poles swing more than the equator.
    let swing = |j: usize| e.t_max_k[j] - e.t_min_k[j];
    assert!(swing(2) > swing(n / 2) && swing(n - 3) > swing(n / 2));
    assert!(e.t_min_k.iter().zip(&e.t_k).all(|(a, b)| a <= b));
}

#[test]
fn a_backwards_spin_turns_every_wind_and_nothing_else() {
    let cs = base_settings();
    let mut v = world("Venus");
    assert!(v.retrograde(), "the fixture spins backwards");
    let back = orbit(&cs, &v, 4500.0);
    v.rotation_hours = -v.rotation_hours;
    let fwd = orbit(&cs, &v, 4500.0);
    assert!(back.wind_m_s.iter().any(|x| x.abs() > 1e-6));
    for (a, b) in back.wind_m_s.iter().zip(&fwd.wind_m_s) {
        assert_eq!(*a, -*b);
    }
    assert_eq!(back.t_k, fwd.t_k);
    assert_eq!(back.precip_m_yr, fwd.precip_m_yr);
    let (b, f) = (&back.circulation, &fwd.circulation);
    assert_eq!(
        (b.cells, &b.edges_deg, b.spin_sign),
        (f.cells, &f.edges_deg, -f.spin_sign)
    );
    for lat in [5.0, 25.0, 45.0, 70.0] {
        assert_eq!(b.prevailing_wind(lat), -f.prevailing_wind(lat));
        assert_ne!(b.easterly(lat), f.easterly(lat));
    }
    // Venus's solar day: 117 Earth days, because the spin runs against the orbit.
    let day = world("Venus").solar_day_hours() / 24.0;
    assert!((day - 116.75).abs() < 0.5, "{day}");
}

#[test]
fn a_kept_climate_is_reused_and_a_new_epoch_is_not() {
    let cs = base_settings();
    let e = world("Earth");
    let mut cache = Cache::new(4);
    let a = cache.orbit(&cs, &e, 4500.0).clone();
    let b = cache.orbit(&cs, &e, 4500.0).clone();
    assert_eq!(a, b);
    assert_eq!(cache.len(), 1);
    cache.orbit(&cs, &e, 4400.0);
    assert_eq!(cache.len(), 2);
    assert_ne!(a.key, cache.orbit(&cs, &e, 4400.0).key);
    assert_eq!(a, orbit(&cs, &e, 4500.0));
}

#[test]
fn glance_is_scalars_and_matches_the_orbit_cells() {
    let cs = base_settings();
    for w in ["Earth", "Mars", "Titan", "Moon"] {
        let c = world(w);
        let g = glance(&cs, &c, 4500.0);
        let o = orbit(&cs, &c, 4500.0);
        assert_eq!(g.circulation, o.circulation);
        assert!(g.key.contains("|glance|") && o.key.contains("|orbit|"));
    }
    assert!(glance(&cs, &world("Earth"), 0.0)
        .condensables
        .contains(&"water".to_string()));
}

#[test]
fn the_airless_moon_has_no_wind_or_rain() {
    let cs = base_settings();
    let m = orbit(&cs, &world("Moon"), 4500.0);
    assert!(m.wind_m_s.iter().all(|x| *x == 0.0));
    assert!(m.precip_m_yr.iter().all(|x| *x == 0.0));
}

#[test]
fn every_reference_has_a_link() {
    let cs = base_settings();
    assert!(cs
        .references
        .iter()
        .all(|r| r.doi.is_some() || r.url.is_some()));
}

#[test]
fn earth_has_easterly_trades_and_westerlies() {
    let cs = base_settings();
    let e = orbit(&cs, &world("Earth"), 4500.0);
    let c = &e.circulation;
    assert_eq!(c.regime, "earth-like");
    assert!(c.easterly(10.0) && !c.easterly(45.0) && c.easterly(80.0));
    assert!(c.prevailing_wind(15.0) < 0.0 && c.prevailing_wind(45.0) > 0.0);
}

/// W7a: the climate lives in this crate only. No other crate's code (tests
/// may restate a relation to check it) carries its own temperature contrast,
/// circulation cell wind, rain-by-band lapse, Hadley edge or Rhines count.
#[test]
fn no_second_climate_outside_the_crate() {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let banned = [
        "contrast_k",
        "contrast_x",
        "cell_wind_m_s",
        "lapse_k_m:",
        "omega / u",
        "held_hou",
        "fn rhines",
        "fn hadley_edge",
    ];
    let mut found = vec![];
    let mut stack = vec![crates];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).expect("crates dir") {
            let p = e.expect("entry").path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir() {
                if name != "sse-climate" && name != "target" {
                    stack.push(p);
                }
                continue;
            }
            if !name.ends_with(".rs") || name.ends_with("_tests.rs") || name == "tests.rs" {
                continue;
            }
            let text = std::fs::read_to_string(&p).expect("source");
            for (i, line) in text.lines().enumerate() {
                let code = line.trim_start();
                if code.starts_with("//") {
                    continue;
                }
                if let Some(b) = banned.iter().find(|b| code.contains(*b)) {
                    found.push(format!("{}:{}: {b}", p.display(), i + 1));
                }
            }
        }
    }
    assert!(found.is_empty(), "climate code outside sse-climate: {found:#?}");
}
