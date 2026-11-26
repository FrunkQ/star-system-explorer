//! XO1 part 2: the gravity formula blocks against real objects (design section 11).

use crate::bend::b_crit;
use sse_core::math as m;
use sse_flow::Atom;

/// Run a gravity formula block from the base pack, inputs in port order.
fn block(id: &str, inputs: &[f64]) -> Vec<f64> {
    let a = sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .unwrap_or_else(|| panic!("no block {id}"));
    a.eval(inputs, &[])
        .unwrap_or_else(|e| panic!("{id}: {}", e.0))
}

/// Seconds of arc in a radian, and millionths of them.
const ARCSEC: f64 = 180.0 * 3600.0 / m::PI;
const MICROARCSEC: f64 = ARCSEC * 1e6;

fn rg_of_suns(n: f64) -> f64 {
    let m_sun = sse_core::sheet::GM_SUN / sse_core::sheet::G;
    block("sse.gravity.gravitational_radius", &[n * m_sun])[0]
}

// ---- the gravity blocks against real objects (design section 11) ----

#[test]
fn starlight_grazing_the_sun_is_bent_by_1_75_arcseconds() {
    let rg = rg_of_suns(1.0);
    let a = block("sse.gravity.weak_bend", &[rg, sse_core::sheet::R_SUN])[0];
    assert!((a * ARCSEC - 1.75).abs() < 0.01, "{}", a * ARCSEC);
}

#[test]
fn m87_and_the_milky_way_centre_show_their_measured_shadows() {
    let pc = sse_core::sheet::PC;
    // M87*: 6.5 thousand million Suns at 16.8 Mpc; EHT ring 42 ± 3 µas,
    // shadow (smaller than the ring) about 40.
    let th = block(
        "sse.gravity.shadow_angle",
        &[rg_of_suns(6.5e9), 16.8e6 * pc],
    )[0];
    let d = 2.0 * th * MICROARCSEC;
    assert!((d - 39.7).abs() < 0.5, "M87 {d}");
    // Sgr A*: 4.15 million Suns at 8.2 kpc; about 50 µas (EHT 2022 ring 51.8 ± 2.3).
    let th = block(
        "sse.gravity.shadow_angle",
        &[rg_of_suns(4.15e6), 8.2e3 * pc],
    )[0];
    let d = 2.0 * th * MICROARCSEC;
    assert!((d - 51.9).abs() < 0.5, "Sgr A* {d}");
    // From far away the shadow's edge is the table's critical ray.
    let th = block("sse.gravity.shadow_angle", &[1.0, 1e9])[0];
    assert!((th * 1e9 / b_crit() - 1.0).abs() < 1e-8);
}

#[test]
fn last_stable_orbit_by_spin() {
    let isco = |a| block("sse.gravity.last_stable_orbit", &[1.0, a])[0];
    assert_eq!(isco(0.0), 6.0);
    assert!((isco(0.998) - 1.237).abs() < 1e-3, "{}", isco(0.998));
    assert!((isco(-1.0) - 9.0).abs() < 1e-12);
    assert!((isco(1.0) - 1.0).abs() < 1e-12);
    // the photon orbit and the horizon sit inside it, at every spin
    for a in [-1.0, -0.5, 0.0, 0.5, 0.9, 0.998] {
        let ph = block("sse.gravity.photon_orbit", &[1.0, a])[0];
        let hz = block("sse.gravity.horizon", &[1.0, a])[0];
        assert!(hz <= ph && ph <= isco(a) + 1e-12, "a {a}: {hz} {ph}");
    }
    assert_eq!(block("sse.gravity.photon_orbit", &[1.0, 0.0])[0], 3.0);
    assert_eq!(block("sse.gravity.horizon", &[1.0, 0.0])[0], 2.0);
}

#[test]
fn a_clock_held_at_three_schwarzschild_radii_runs_at_0_816() {
    let out = block("sse.gravity.clock_rate", &[1.0, 6.0]);
    assert!((out[0] - 0.816).abs() < 5e-4, "{}", out[0]);
    // no circular orbit inside the photon orbit, no hovering inside the horizon
    assert_eq!(block("sse.gravity.clock_rate", &[1.0, 2.5])[1], 0.0);
    assert_eq!(block("sse.gravity.clock_rate", &[1.0, 1.5])[0], 0.0);
}

#[test]
fn a_small_hole_stretches_hard_and_a_big_one_gently() {
    let m_sun = sse_core::sheet::GM_SUN / sse_core::sheet::G;
    // 10 Suns, 1,000 km: over two metres, thousands of Earth gravities.
    let k = block("sse.gravity.tide", &[10.0 * m_sun, 1e6])[0];
    let g_two_metres = 2.0 * k / sse_core::sheet::G0;
    assert!(g_two_metres > 500.0, "{g_two_metres}");
    // 4 million Suns, at its own horizon: under a hundredth of a gravity.
    let rh = 2.0 * rg_of_suns(4.15e6);
    let k = block("sse.gravity.tide", &[4.15e6 * m_sun, rh])[0];
    assert!(2.0 * k / sse_core::sheet::G0 < 0.01);
}
