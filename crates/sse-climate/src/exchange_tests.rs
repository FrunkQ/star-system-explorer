//! W8 part 2: `sse.volatile.exchange` (weather review section 8's tests).
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
        .map(|w| w.conditions)
        .find(|c| c.label == label)
        .unwrap()
}

fn pluto_with_ice(kg_m2: f64) -> Conditions {
    let mut p = crate::water_tests::pluto();
    p.ground_ice_kg_m2.insert("n2".into(), kg_m2);
    p
}

fn lat_mean(cl: &Climate, v: &[f64], keep: impl Fn(f64) -> bool) -> f64 {
    let (mut s, mut a) = (0.0, 0.0);
    for (l, x) in cl.lat_deg.iter().zip(v) {
        if keep(l.abs()) {
            let w = sse_core::math::cos(l.to_radians());
            s += w * x;
            a += w;
        }
    }
    s / a
}

/// Mars: the CO2 caps come out of the air and go back, so the pressure
/// swings by about a quarter over the year with two minima, the deeper in
/// southern winter (Viking: 25 to 30 %, Hess et al. 1980; the review's test).
#[test]
fn mars_breathes_its_air_into_its_caps_twice_a_year() {
    let cs = base_settings();
    let cl = orbit(&cs, &world("Mars"), 4500.0);
    let ex = cl.exchange.as_ref().expect("Mars exchanges CO2");
    assert_eq!(ex.gas, "co2");
    assert!(!ex.buffered);
    assert!((0.22..0.33).contains(&ex.p_swing), "swing {}", ex.p_swing);
    // The given mean pressure holds (the air is the inventory).
    let given = 0.95 * 0.006e5;
    assert!(
        (ex.p_mean_pa / given - 1.0).abs() < 0.03,
        "{}",
        ex.p_mean_pa
    );
    let p: Vec<f64> = ex.seasons.iter().map(|s| s.p_pa).collect();
    let n = p.len();
    let minima: Vec<usize> = (0..n)
        .filter(|&i| p[i] < p[(i + n - 1) % n] && p[i] < p[(i + 1) % n])
        .collect();
    assert_eq!(minima.len(), 2, "{p:?}");
    // The deeper minimum in southern winter (solar longitude 90 to 180°,
    // the second quarter or so of a year counted from northern spring).
    let deepest = minima
        .iter()
        .cloned()
        .min_by(|a, b| p[*a].total_cmp(&p[*b]))
        .unwrap();
    let phase = (deepest as f64 + 0.5) / n as f64;
    assert!((0.25..0.6).contains(&phase), "{phase} {p:?}");
    // Caps at both poles in their winters, none in the tropics.
    let most = |j: usize| {
        ex.seasons
            .iter()
            .map(|s| s.ice_kg_m2[j])
            .fold(0.0, f64::max)
    };
    assert!(most(0) > 100.0 && most(cl.lat_deg.len() - 1) > 100.0);
    for (j, l) in cl.lat_deg.iter().enumerate() {
        if l.abs() < 40.0 {
            assert_eq!(most(j), 0.0, "{l}");
        }
    }
    // The frosted ground sits at the frost point: the coldest band's
    // coldest season is within a kelvin of it.
    let tf = ex
        .seasons
        .iter()
        .map(|s| s.frost_k)
        .fold(f64::INFINITY, f64::min);
    let coldest = cl.t_min_k.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(coldest > tf - 1.0, "{coldest} {tf}");
}

/// Pluto: with nitrogen ice on the ground its pressure is set by the ice's
/// temperature, not by how much ice there is, and follows the frost point
/// through the year.
#[test]
fn plutos_pressure_follows_its_ice() {
    let cs = base_settings();
    let a = orbit(&cs, &pluto_with_ice(1000.0), 4500.0);
    let b = orbit(&cs, &pluto_with_ice(3000.0), 4500.0);
    let (ea, eb) = (a.exchange.as_ref().unwrap(), b.exchange.as_ref().unwrap());
    assert!(ea.buffered && eb.buffered);
    assert!(
        (ea.p_mean_pa / eb.p_mean_pa - 1.0).abs() < 0.2,
        "{} {}",
        ea.p_mean_pa,
        eb.p_mean_pa
    );
    let n2 = cs.substances.iter().find(|s| s.gas == "n2").unwrap();
    for s in &ea.seasons {
        let back = ebm::p_sat(n2, s.frost_k);
        assert!((back / s.p_pa - 1.0).abs() < 0.02, "{} {}", back, s.p_pa);
    }
    // A warmer season means a higher pressure (no pressure without ice to
    // feed it: the ground is isothermal at the frost point).
    let mut order: Vec<&crate::exchange::ExchangeSeason> = ea.seasons.iter().collect();
    order.sort_by(|x, y| x.frost_k.total_cmp(&y.frost_k));
    for w in order.windows(2) {
        assert!(w[1].p_pa >= w[0].p_pa);
    }
    // Its ice sublimes and lofts sand where it lies.
    let ss = &cs.saltation;
    let l = (-80..=80)
        .step_by(10)
        .map(|x| saltation::lofting(ss, &a, f64::from(x)))
        .fold(0.0, f64::max);
    assert!(l > 0.2, "{l}");
}

/// Bertrand et al. 2024's test: the same model and world, with only the
/// tilt changed, gives Pluto its low-latitude ice and a Triton-like world
/// its polar caps (above about 54° the poles get more sunlight a year than
/// the equator).
#[test]
fn the_tilt_alone_moves_the_ice_from_equator_to_poles() {
    let cs = base_settings();
    let pluto = orbit(&cs, &pluto_with_ice(1000.0), 4500.0);
    let mut c = pluto_with_ice(1000.0);
    c.obliquity_deg = 30.0;
    let triton = orbit(&cs, &c, 4500.0);
    let ep = pluto.exchange.as_ref().unwrap();
    let et = triton.exchange.as_ref().unwrap();
    let kept = |cl: &Climate, ex: &crate::exchange::Exchange| -> Vec<f64> {
        cl.lat_deg
            .iter()
            .zip(&ex.perennial)
            .filter(|(_, p)| **p)
            .map(|(l, _)| *l)
            .collect()
    };
    let kp = kept(&pluto, ep);
    let kt = kept(&triton, et);
    assert!(
        kp.len() >= 3 && kp.iter().all(|l| l.abs() < 35.0),
        "Pluto {kp:?}"
    );
    assert!(
        kt.len() >= 2 && kt.iter().all(|l| l.abs() > 60.0),
        "Triton {kt:?}"
    );
    let low = |cl: &Climate, ex: &crate::exchange::Exchange| {
        lat_mean(cl, &ex.ice_mean_kg_m2, |l| l < 30.0)
    };
    let high = |cl: &Climate, ex: &crate::exchange::Exchange| {
        lat_mean(cl, &ex.ice_mean_kg_m2, |l| l > 60.0)
    };
    assert!(low(&pluto, ep) > high(&pluto, ep));
    assert!(high(&triton, et) > 10.0 * low(&triton, et).max(1.0));
    // The ice is moved, not made: the inventory is kept.
    for (cl, ex) in [(&pluto, ep), (&triton, et)] {
        let held = lat_mean(cl, &ex.seasons.last().unwrap().ice_kg_m2, |_| true)
            + ex.seasons.last().unwrap().p_pa / 0.62;
        assert!((held / (1000.0 + 0.995) - 1.0).abs() < 0.02, "{held}");
    }
}

#[test]
fn earth_and_venus_exchange_nothing() {
    let cs = base_settings();
    for w in ["Earth", "Venus", "Titan", "Jupiter", "Moon"] {
        assert!(orbit(&cs, &world(w), 4500.0).exchange.is_none(), "{w}");
    }
}
