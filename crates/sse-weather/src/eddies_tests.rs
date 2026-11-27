// WS2: a giant's eddies from the heat it sends out (design SSE4-stirred-clouds-design.md 3.2, 3.3).
use super::*;
use crate::giants::{base_settings, fixtures, layout};

fn world(name: &str) -> Conditions {
    fixtures()
        .into_iter()
        .find(|f| f.conditions.label == name)
        .expect("fixture")
        .conditions
}

/// The speed of an eddy as wide as the band holding 45°N.
fn band_speed(name: &str) -> f64 {
    let gs = base_settings();
    let c = world(name);
    let l = layout(&gs, &c);
    let b = bands(&l)
        .into_iter()
        .find(|b| b.north && b.lat_lo_deg <= 45.0 && b.lat_hi_deg > 45.0)
        .unwrap();
    eddy_speed(&c, eddy_power(&gs, &c), b.width_m(c.radius_m))
}

#[test]
fn ws2_eddies_run_at_the_speed_their_heat_gives() {
    // Fitted on Jupiter: 17 m/s at band width (absolute).
    let j = band_speed("Jupiter");
    assert!((j - 17.0).abs() < 0.3, "Jupiter {j}");
    // Saturn follows from its own heat, gravity and band width: about 8 m/s (design section 5).
    let s = band_speed("Saturn");
    assert!((s / 8.0 - 1.0).abs() < 0.1, "Saturn {s}");
    // Every world follows the cube-root law in its heat, gravity and band width, from Jupiter's fit. (The ice
    // giants' three bands are about 21,000 km wide, so their eddies come out at 6 to 7 m/s, not the design's
    // "about 3": still sixty times slower than their 400 m/s jets.)
    let gs = base_settings();
    let fgl = |w: &str| {
        let c = world(w);
        let l = layout(&gs, &c);
        let b = bands(&l)
            .into_iter()
            .find(|b| b.north && b.lat_lo_deg <= 45.0 && b.lat_hi_deg > 45.0)
            .unwrap();
        heat_out_w_m2(&c) * c.gravity_m_s2 * b.width_m(c.radius_m)
    };
    for w in ["Saturn", "Uranus", "Neptune"] {
        let want = j * m::cbrt(fgl(w) / fgl("Jupiter"));
        let u = band_speed(w);
        assert!(
            (u / want - 1.0).abs() < 0.02,
            "{w}: {u} m/s, the law gives {want}"
        );
    }
}

#[test]
fn ws2_an_eddy_against_the_shear_is_gone_in_one_shear_time() {
    let gs = base_settings();
    let c = world("Jupiter");
    let l = layout(&gs, &c);
    let es = eddies_between(&gs, &c, &l, 100.0, 100.0);
    assert!(es.len() > 1000, "{} eddies", es.len());
    let torn: Vec<&Eddy> = es.iter().filter(|e| e.torn).collect();
    let held: Vec<&Eddy> = es.iter().filter(|e| !e.torn).collect();
    assert!(
        !torn.is_empty() && !held.is_empty(),
        "{} torn, {} held",
        torn.len(),
        held.len()
    );
    for e in &torn {
        let zeta = zonal_vorticity(&gs, &l, c.radius_m, e.lat_deg).abs();
        let life_s = (e.ends - e.born) * DAY_S;
        assert!(
            life_s <= 1.0 / zeta * (1.0 + 1e-9),
            "lived {life_s} s against a shear time {}",
            1.0 / zeta
        );
    }
    // A torn eddy has no steady shape (held at the limit, sqrt 2 - 1); a held one has its own.
    for e in &torn {
        assert!((e.aspect - (m::sqrt(2.0) - 1.0)).abs() < 1e-12, "{e:?}");
    }
    for e in &held {
        assert!(e.aspect > m::sqrt(2.0) - 1.0, "{e:?}");
    }
    eprintln!("Jupiter day 100: {} eddies, {} torn", es.len(), torn.len());
}

#[test]
fn ws2_the_same_eddies_whichever_date_asks() {
    let gs = base_settings();
    let c = world("Saturn");
    let l = layout(&gs, &c);
    let span = eddies_between(&gs, &c, &l, 990.0, 1010.0);
    let at = eddies_between(&gs, &c, &l, 1000.0, 1000.0);
    let alive = |e: &&Eddy| e.born <= 1000.0 && e.ends >= 1000.0;
    let a: Vec<&Eddy> = span.iter().filter(alive).collect();
    let b: Vec<&Eddy> = at.iter().filter(alive).collect();
    assert!(!b.is_empty());
    assert_eq!(a, b);
}

#[test]
fn ws2_no_heat_no_eddies() {
    let gs = base_settings();
    let mut c = world("Uranus");
    c.internal_flux_w_m2 = 0.0;
    c.flux_w_m2 = 0.0;
    let l = layout(&gs, &c);
    assert!(eddies_between(&gs, &c, &l, 0.0, 10.0).is_empty());
}

// WS3: plumes from the heat below, and how fast the deck is renewed (design 3.4, 3.5).

#[test]
fn ws3_plumes_alive_average_the_count_their_heat_gives() {
    let gs = base_settings();
    let c = world("Jupiter");
    let l = layout(&gs, &c);
    let (want, _) = plume_count(&gs, &c);
    assert!(want > 10.0, "Jupiter {want} plumes");
    // Over a long run of dates the plumes alive average the block's count.
    let ps = plumes_between(&gs, &c, &l, 0.0, 400.0);
    let mut sum = 0.0;
    let n = 200;
    for k in 0..n {
        let day = 100.0 + k as f64;
        sum += ps.iter().filter(|p| p.born <= day && p.ends > day).count() as f64;
    }
    let mean = sum / n as f64;
    assert!(
        (mean / want - 1.0).abs() < 0.2,
        "{mean} alive on average, the block gives {want}"
    );
    // The same plumes whichever date asks.
    let alive = |p: &&Plume| p.born <= 250.0 && p.ends > 250.0;
    let one = plumes_between(&gs, &c, &l, 250.0, 250.0);
    let at: Vec<&Plume> = one.iter().filter(alive).collect();
    let span: Vec<&Plume> = ps.iter().filter(alive).collect();
    assert!(!at.is_empty());
    assert_eq!(at, span);
}

#[test]
fn ws3_no_heat_from_below_no_plumes() {
    let gs = base_settings();
    let mut c = world("Saturn");
    c.internal_flux_w_m2 = 0.0;
    let l = layout(&gs, &c);
    assert!(plumes_between(&gs, &c, &l, 0.0, 100.0).is_empty());
}

#[test]
fn ws3_a_deck_is_renewed_in_months() {
    let gs = base_settings();
    let cs = &crate::scenario::settings().climate;
    let c = world("Jupiter");
    let l = layout(&gs, &c);
    let d = renewal_s(&gs, cs, &c, &l).expect("Jupiter's deck renews") / DAY_S;
    // Micron ammonia grains falling through a scale height of deck: weeks to a year (unverified).
    assert!(d > 10.0 && d < 1000.0, "{d} days");
    eprintln!("Jupiter's deck renewed in {d:.0} days");
}
