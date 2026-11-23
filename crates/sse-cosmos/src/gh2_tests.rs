//! GH2: orbits and the four processes after infall (design 7.3 and 10).

use super::*;
use crate::assembly::{mass_today_for, tree, AssemblyParams, Level};
use crate::haloes::{HaloParams, Haloes};
use sse_core::{Address, Seed};

struct World {
    c: Cosmology,
    w: crate::web::Web,
}

fn world() -> World {
    let (c, w) = crate::g3_tests::world();
    World { c, w }
}

fn haloes(w: &World) -> Haloes<'_> {
    Haloes {
        cosmo: &w.c,
        web: &w.w,
        params: HaloParams::default(),
        seed: Seed::from_text("gh2"),
        universe: Address::universe("gh2").unwrap(),
    }
}

fn host_of(w: &World, h: &Halo, op: &OrbitParams) -> (Host, GalaxyParams, Galaxy) {
    let gp = GalaxyParams::default();
    let g = crate::galaxy::evolve(&w.c, &gp, h);
    let host = Host::new(&w.c, &gp, h, &g, 0.5, w.c.age_gyr(), op);
    (host, gp, g)
}

#[test]
fn the_host_table_matches_the_direct_formulas() {
    let w = world();
    let hs = haloes(&w);
    let h = hs.median(1.0e12, "table");
    let op = OrbitParams::default();
    let gp = GalaxyParams::default();
    let g = crate::galaxy::evolve(&w.c, &gp, &h);
    let now = w.c.age_gyr();
    let host = Host::new(&w.c, &gp, &h, &g, now, now, &op);
    let a = w.c.scale_of_time(now).min(1.0);
    let s = Structure::new(&w.c, &gp.structure, &|z| h.mass_at(z), state_at(&g, now), a);
    let mut worst: f64 = 0.0;
    // Between grid points, from 0.3 kpc to the virial radius.
    let mut r = 0.3;
    while r < s.r200_kpc {
        let here = host.at(now, r);
        let v = s.speeds(r).total * kms();
        let pull = v * v / r;
        worst = worst.max((here.pull / pull - 1.0).abs());
        let rho = s.dark_density(r) * (1.0 + s.hot_msun / s.dark_msun);
        worst = worst.max((here.rho / rho - 1.0).abs());
        r *= 1.137;
    }
    // A stated tolerance: the table is within 2 % of the formulas.
    assert!(worst < 0.02, "{worst}");
    // The potential is the pull's integral: dΦ/dr = g.
    let (r1, r2) = (8.0, 8.5);
    let d = (host.at(now, r2).phi - host.at(now, r1).phi) / (r2 - r1);
    let g_mid = host.at(now, 8.25).pull;
    assert!((d / g_mid - 1.0).abs() < 0.02, "{d} {g_mid}");
    // The drag factor table against erf.
    let t = DragTable::new();
    for x in [0.01, 0.3, 0.77, 1.0, 1.9, 3.3, 7.5, 9.0] {
        let f = if x >= DRAG_X_MAX {
            1.0
        } else {
            m::erf(x) - 2.0 * x / m::sqrt(m::PI) * m::exp(-x * x)
        };
        assert!((t.at(x) - f).abs() < 2.0e-4, "{x}: {} {f}", t.at(x));
    }
}

#[test]
fn leapfrog_energy_drifts_less_than_a_thousandth_over_a_hubble_time() {
    let w = world();
    let hs = haloes(&w);
    let h = hs.median(1.0e12, "drift");
    let op = OrbitParams::default();
    let gp = GalaxyParams::default();
    let g = crate::galaxy::evolve(&w.c, &gp, &h);
    let now = w.c.age_gyr();
    // One slice: a potential that does not change with time.
    let host = Host::new(&w.c, &gp, &h, &g, now, now, &op);
    let mut worst: f64 = 0.0;
    for (i, u) in [
        [0.2, 0.3, 0.1, 0.4, 0.6],
        [0.7, 0.9, 0.9, 0.9, 0.1],
        [0.5, 0.5, 0.5, 0.3, 0.5],
    ]
    .iter()
    .enumerate()
    {
        let mut b = infall(
            &w.c,
            &gp,
            &op,
            h.mass_at(0.0),
            &format!("b{i}"),
            1.0e9,
            0.0,
            0.0,
            *u,
            None,
        );
        // Start inside the halo so the orbit stays inside the table.
        for k in 0..3 {
            b.pos_kpc[k] *= 0.5;
        }
        let e0 = energy(&host, 0.0, b.pos_kpc, b.vel_kpc_gyr);
        let f = follow_with(&host, &b, 13.0, &op, false);
        let e1 = energy(&host, 13.0, f.pos_kpc, f.vel_kpc_gyr);
        worst = worst.max(((e1 - e0) / e0).abs());
        eprintln!(
            "drift {i}: {} steps {} dt {}",
            (e1 - e0) / e0,
            f.steps,
            f.step_gyr
        );
        assert!(f.steps > 100, "{}", f.steps);
    }
    assert!(worst < 1.0e-3, "{worst}");
}

/// The Chart tree of a median 10¹² M☉ host, every member followed to now.
fn chart(w: &World, name: &str, op: &OrbitParams) -> Vec<(crate::assembly::Progenitor, Fate)> {
    let hs = haloes(w);
    let h = hs.median(1.0e12, name);
    let (host, gp, _) = host_of(w, &h, op);
    let t = tree(
        &w.c,
        hs.seed,
        &h,
        0.0,
        &AssemblyParams::default(),
        Level::Chart,
    )
    .unwrap();
    let d = sse_core::Draws::new(
        hs.seed,
        h.address.parse().unwrap(),
        sse_core::Stream::new(crate::assembly::FILL_STREAM).unwrap(),
    );
    t.progenitors
        .iter()
        .map(|p| {
            let k = p.draw_base;
            let u = [
                d.unit(k + 2),
                d.unit(k + 3),
                d.unit(k + 4),
                d.unit(k + 5),
                d.unit(k + 6),
            ];
            let ph = hs.median_at(
                mass_today_for(&w.c, p.mass_at_infall_msun, p.z_infall),
                &p.address.parse().unwrap(),
            );
            let pg = crate::galaxy::evolve(&w.c, &gp, &ph);
            let s = state_at(&pg, p.t_infall_gyr);
            let grip = Grip {
                stars_msun: s.stars,
                gas_msun: s.gas,
                disc_kpc: s.disc_kpc,
                gas_scale: gp.structure.gas_scale_factor,
            };
            let b = infall(
                &w.c,
                &gp,
                op,
                h.mass_at(p.z_infall),
                &p.address,
                p.mass_at_infall_msun,
                p.z_infall,
                p.t_infall_gyr,
                u,
                Some(grip),
            );
            (p.clone(), follow(&host, &b, w.c.age_gyr(), op))
        })
        .collect()
}

#[test]
fn the_four_processes_shape_a_satellite_system() {
    let w = world();
    let op = OrbitParams::default();
    let (mut n, mut merged_big, mut big, mut alive_small, mut small) = (0, 0, 0, 0, 0);
    let (mut quenched_small, mut stripped, mut torn) = (0, 0, 0);
    let mut alive = 0;
    let (mut mid, mut mid_forming) = (0, 0);
    let mut delays = vec![];
    for name in ["mw0", "mw1", "mw2", "mw3"] {
        for (p, f) in chart(&w, name, &op) {
            n += 1;
            assert!(f.dark_kept <= 1.0 && f.dark_kept >= 0.0, "{f:?}");
            assert!(!f.labels.is_empty());
            // Stars sit deep in the dark halo, so they go more slowly than
            // the dark matter: with half of it still bound, four in five stars
            // are (GH4's tidal rule; Peñarrubia et al. 2008).
            if f.disrupted_gyr.is_none() {
                assert!(
                    f.stars_kept >= f.dark_kept,
                    "{} {}",
                    f.dark_kept,
                    f.stars_kept
                );
                if f.dark_kept >= 0.5 {
                    assert!(f.stars_kept > 0.8, "{} {}", f.dark_kept, f.stars_kept);
                }
            }
            if f.pericentres > 0 && f.dark_kept < 0.9 {
                stripped += 1;
            }
            if f.labels
                .iter()
                .any(|l| l == "being torn apart" || l == "stream" || l == "shell")
            {
                torn += 1;
            }
            let alive_now = f.merged_gyr.is_none() && f.disrupted_gyr.is_none();
            alive += usize::from(alive_now);
            if alive_now && p.psi > 0.01 {
                mid += 1;
                mid_forming += usize::from(f.quench_gyr.is_none());
            }
            if let Some(q) = f.quench_gyr {
                delays.push(q - p.t_infall_gyr);
            }
            if p.psi > 0.03 {
                big += 1;
                merged_big += usize::from(f.merged_gyr.is_some() || f.disrupted_gyr.is_some());
            } else if p.psi < 3.0e-3 {
                small += 1;
                alive_small += usize::from(alive_now);
                quenched_small += usize::from(alive_now && f.quench_gyr.is_some());
            }
        }
    }
    delays.sort_by(f64::total_cmp);
    eprintln!(
        "big survivors {mid}, still forming {mid_forming}; quench delay median {:.2} Gyr",
        delays[delays.len() / 2]
    );
    eprintln!("n {n} alive {alive} big {big} gone {merged_big} small {small} alive {alive_small} quenched {quenched_small} stripped {stripped} torn {torn}");
    // Sinking: the big ones (over 3 % of the host) mostly reach the centre
    // or are torn apart; the small ones mostly survive.
    assert!(big > 0 && merged_big * 2 > big, "{merged_big} of {big}");
    assert!(alive_small * 2 > small, "{alive_small} of {small}");
    // Stripping: most that came close lost dark matter, and some lost stars.
    assert!(stripped * 2 > n / 2, "{stripped} of {n}");
    assert!(torn > 0);
    // Gas stripping: nearly all small survivors have stopped forming stars
    // (Wetzel, Tollerud and Weisz 2015).
    assert!(
        quenched_small * 10 >= alive_small * 8,
        "{quenched_small} of {alive_small}"
    );
    // ... while the biggest survivors keep going more often.
    assert!(
        mid_forming * alive_small > (alive_small - quenched_small) * mid,
        "{mid_forming} of {mid}"
    );
}

#[test]
fn a_path_is_the_same_on_rerun_and_whatever_time_it_is_read_to() {
    let w = world();
    let op = OrbitParams::default();
    let a = chart(&w, "rerun", &op);
    let b = chart(&w, "rerun", &op);
    assert_eq!(a, b);
    // Read to an earlier time: the same path up to there (cut 2: the step
    // is the body's own, not the reading's).
    let hs = haloes(&w);
    let h = hs.median(1.0e12, "rerun");
    let (host, gp, _) = host_of(&w, &h, &op);
    for (p, f) in a.iter().take(10) {
        let d = sse_core::Draws::new(
            hs.seed,
            h.address.parse().unwrap(),
            sse_core::Stream::new(crate::assembly::FILL_STREAM).unwrap(),
        );
        let k = p.draw_base;
        let u = [
            d.unit(k + 2),
            d.unit(k + 3),
            d.unit(k + 4),
            d.unit(k + 5),
            d.unit(k + 6),
        ];
        let b = infall(
            &w.c,
            &gp,
            &op,
            h.mass_at(p.z_infall),
            &p.address,
            p.mass_at_infall_msun,
            p.z_infall,
            p.t_infall_gyr,
            u,
            None,
        );
        let t_mid = 0.5 * (p.t_infall_gyr + w.c.age_gyr());
        let early = follow(&host, &b, t_mid, &op);
        let late = follow(&host, &b, w.c.age_gyr(), &op);
        assert_eq!(early.step_gyr, f.step_gyr);
        let n = early.track.len() - 1;
        assert_eq!(early.track[..n], late.track[..n], "{}", p.address);
    }
}

/// Survey level timed (design section 10: about 1 s on a laptop after the
/// cuts). Native here; the harness bench gives the WebAssembly figure.
#[test]
#[ignore]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn survey_orbits_timed() {
    let w = world();
    let hs = haloes(&w);
    let op = OrbitParams::default();
    let h = hs.median(1.0e12, "survey");
    let (host, gp, _) = host_of(&w, &h, &op);
    let t = tree(
        &w.c,
        hs.seed,
        &h,
        0.0,
        &AssemblyParams::default(),
        Level::Survey,
    )
    .unwrap();
    let d = sse_core::Draws::new(
        hs.seed,
        h.address.parse().unwrap(),
        sse_core::Stream::new(crate::assembly::FILL_STREAM).unwrap(),
    );
    let start = std::time::Instant::now();
    let mut steps = 0;
    for p in &t.progenitors {
        let k = p.draw_base;
        let u = [
            d.unit(k + 2),
            d.unit(k + 3),
            d.unit(k + 4),
            d.unit(k + 5),
            d.unit(k + 6),
        ];
        let b = infall(
            &w.c,
            &gp,
            &op,
            h.mass_at(p.z_infall),
            &p.address,
            p.mass_at_infall_msun,
            p.z_infall,
            p.t_infall_gyr,
            u,
            None,
        );
        steps += follow(&host, &b, w.c.age_gyr(), &op).steps;
    }
    let s = start.elapsed().as_secs_f64();
    eprintln!(
        "survey: {} bodies, {steps} steps, {s:.3} s, {:.3} us a step",
        t.progenitors.len(),
        1e6 * s / steps as f64
    );
}
