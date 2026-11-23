//! DM2: dark energy's pressure, w = w0 + wa (1 − a), in the expansion and in
//! the growth of structure (galaxy histories design 3.4).

use crate::cosmology::{Cosmology, Params};
use sse_core::math as m;

fn with_w(w0: f64, wa: f64) -> Cosmology {
    Cosmology::new(Params {
        w0,
        wa,
        ..Params::default()
    })
}

/// DESI's evolving dark energy (DR2 with supernovae, arXiv:2503.14738)
/// steps cleanly: growth rises all the way to today.
#[test]
fn desi_like_dark_energy_grows_structure_smoothly() {
    let c = with_w(-0.75, -0.86);
    let mut last = 0.0;
    for k in 1..=100 {
        let a = f64::from(k) / 100.0;
        let d = c.growth(a);
        assert!(d > last, "{a}");
        last = d;
    }
    assert!((c.growth(1.0) - 1.0).abs() < 1e-12);
}

/// The design's check: stepping the growth equation gives today's closed
/// form (Heath 1977) to 0.2 % for a cosmological constant.
#[test]
fn the_stepped_growth_matches_the_closed_form_for_a_constant() {
    let c = Cosmology::new(Params::default());
    assert!(c.is_lambda() && c.growth_table.is_empty());
    for a in [0.05, 0.1, 0.25, 0.5, 0.75, 1.0] {
        let (s, ex) = (c.growth_by_steps(a), c.growth_exact(a));
        assert!((s / ex - 1.0).abs() < 0.002, "{a}: {s} {ex}");
    }
    // And a constant written as w0 = −1, wa = 0 changes nothing at all.
    assert_eq!(c.dark_energy(0.3), 1.0);
    assert_eq!(c.e(0.3), with_w(-1.0, 0.0).e(0.3));
}

/// With w other than −1 the growth rate today follows Linder's growth
/// index, f = Ωm^γ with γ = 0.55 + 0.05 (1 + w(z = 1)) for w(z = 1) above
/// −1 and 0.55 + 0.02 (1 + w(z = 1)) below (Linder 2005,
/// doi:10.1103/PhysRevD.72.043529), to a per cent.
#[test]
fn the_growth_rate_follows_linders_index() {
    // The fit is for w not crossing −1; DESI's (w0, wa) = (−0.75, −0.86)
    // does cross it, and there γ comes out 0.557 against the fit's 0.546.
    for (w0, wa) in [(-0.8, 0.0), (-0.9, 0.0), (-0.9, -0.3), (-1.1, 0.2)] {
        let c = with_w(w0, wa);
        assert!(!c.growth_table.is_empty());
        let h = 1.0e-3;
        let f = (m::ln(c.growth(m::exp(h))) - m::ln(c.growth(m::exp(-h)))) / (2.0 * h);
        let e2 = c.e(1.0) * c.e(1.0);
        let om = c.p.omega_m / e2;
        let w1 = c.w(0.5);
        let gamma = 0.55 + if w1 > -1.0 { 0.05 } else { 0.02 } * (1.0 + w1);
        let want = m::pow(om, gamma);
        assert!((f / want - 1.0).abs() < 0.01, "{w0} {wa}: {f} {want}");
    }
}

/// Dark energy that thins out (w above −1) weighed more in the past, so
/// growth stalled earlier: less structure has grown since half today's
/// size, and the universe is younger at the same Hubble constant.
#[test]
fn thinner_dark_energy_stops_growth_earlier() {
    let l = Cosmology::new(Params::default());
    let q = with_w(-0.8, 0.0);
    assert!(q.growth(0.5) > l.growth(0.5));
    assert!(q.dark_energy(0.5) > 1.0);
    assert!(q.age_gyr() < l.age_gyr());
    // Density scaling for constant w: a^(−3(1 + w)).
    assert!((q.dark_energy(0.5) / m::pow(0.5, -0.6) - 1.0).abs() < 1e-12);
}

/// The web's voids against the measured universal void profile (Hamaus,
/// Sutter and Wandelt 2014, doi:10.1103/PhysRevLett.112.251302): emptied
/// middles (δ near −0.8), density rising through the void's edge, and for
/// small voids a compensating wall just outside it. Stacked around the
/// deepest minima of a slice through the web today.
#[test]
#[allow(clippy::needless_range_loop)]
fn the_webs_voids_have_the_measured_shape() {
    let (c, w) = crate::g3_tests::world();
    let n = 160usize;
    let extent = 320.0;
    let s = w.slice(&c, 1.0, [0.0; 3], extent, n as u32, w.params.octaves);
    let d = |x: usize, y: usize| s.density[y * n + x];
    let px = extent / n as f64;
    // Local minima well below the mean, at least 8 Mpc from the edge and from each other.
    let edge = (12.0 / px) as usize;
    let mut minima: Vec<(f64, usize, usize)> = vec![];
    for y in edge..n - edge {
        for x in edge..n - edge {
            let v = d(x, y);
            if v > 0.4 {
                continue;
            }
            let mut low = true;
            for dy in 0..5usize {
                for dx in 0..5usize {
                    let (xx, yy) = (x + dx - 2, y + dy - 2);
                    if (xx, yy) != (x, y) && d(xx, yy) < v {
                        low = false;
                    }
                }
            }
            if low {
                minima.push((v, x, y));
            }
        }
    }
    minima.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(minima.len() >= 8, "{}", minima.len());
    // Mean profile in rings one sample wide, out to 60 Mpc.
    let rings = (60.0 / px) as usize;
    let mut sum = vec![0.0_f64; rings];
    let mut cnt = vec![0.0_f64; rings];
    for &(_, x0, y0) in minima.iter().take(40) {
        for y in 0..n {
            for x in 0..n {
                let dx = (x as f64 - x0 as f64) * px;
                let dy = (y as f64 - y0 as f64) * px;
                let r = sse_core::math::sqrt(dx * dx + dy * dy);
                let k = (r / px) as usize;
                if k < rings {
                    sum[k] += d(x, y) - 1.0;
                    cnt[k] += 1.0;
                }
            }
        }
    }
    let prof: Vec<f64> = (0..rings).map(|k| sum[k] / cnt[k].max(1.0)).collect();
    // Emptied middle.
    assert!(prof[0] < -0.6, "{prof:?}");
    // The edge: density climbs back to the mean some 10 to 40 Mpc out
    // (Hamaus et al.'s voids are 10 to 70 Mpc/h across), and climbs
    // steadily from the middle to it.
    let k_v = prof.iter().position(|&p| p >= -0.02).expect("an edge");
    let r_v = k_v as f64 * px;
    assert!((10.0..40.0).contains(&r_v), "{r_v} {prof:?}");
    for k in 1..k_v {
        assert!(prof[k] >= prof[k - 1] - 0.03, "{k}: {prof:?}");
    }
    // Half-way, the profile is still well below the mean.
    assert!(prof[k_v / 2] < -0.3, "{prof:?}");
}
