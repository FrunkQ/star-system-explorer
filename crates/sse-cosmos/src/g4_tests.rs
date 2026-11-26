//! G4 checks: envelopes from a galaxy, continuity, hazards, and the green band.

use crate::envelope::{EnvelopeParams, GalaxyEnvelope, GreenParams, Place};
use crate::g3_tests::{milky_way_like, world};
use crate::galaxy::Galaxy;
use crate::rings::Disc;
use sse_core::{Address, Seed};

fn env<'a>(g: &'a Galaxy, d: &'a Disc) -> GalaxyEnvelope<'a> {
    GalaxyEnvelope {
        galaxy: g,
        disc: d,
        params: EnvelopeParams::default(),
        seed: Seed::from_text("g4"),
        address: Address::universe("g4").unwrap(),
        remains: Default::default(),
    }
}

#[test]
fn envelope_is_continuous_across_ring_edges_and_patch_faces() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let gp = GreenParams::default();
    let close = |a: f64, b: f64, rel: f64| (a - b).abs() <= rel * a.abs().max(b.abs()).max(1.0e-12);
    // Either side of every ring edge (and of every ring centre, where the interpolation changes pieces).
    let mut probes = vec![];
    for r in &d.rings {
        probes.push(r.inner_kpc);
        probes.push(r.radius_kpc);
    }
    for t in [6.0, 13.8] {
        for &r in probes.iter().filter(|r| **r > 0.5) {
            let a = e.local(
                Place {
                    r_kpc: r - 1.0e-7,
                    phi: 1.0,
                    z_pc: 30.0,
                    t_gyr: t,
                },
                &gp,
            );
            let b = e.local(
                Place {
                    r_kpc: r + 1.0e-7,
                    phi: 1.0,
                    z_pc: 30.0,
                    t_gyr: t,
                },
                &gp,
            );
            assert!(
                (a.feh - b.feh).abs() < 0.01,
                "[Fe/H] jumps at {r} kpc, t {t}"
            );
            for (x, y, what) in [
                (a.gas_pc3, b.gas_pc3, "gas"),
                (a.stars_pc3, b.stars_pc3, "stars"),
                (a.sfr_gyr_pc3, b.sfr_gyr_pc3, "sfr"),
                (a.sterilising_per_gyr, b.sterilising_per_gyr, "supernovae"),
            ] {
                assert!(close(x, y, 0.01), "{what} jumps at {r} kpc, t {t}: {x} {y}");
            }
        }
    }
    // Across texture lattice faces (every 50 pc) and in time between steps.
    for k in 0..40 {
        let r = 8.0 + f64::from(k) * 0.05;
        let a = e.local(
            Place {
                r_kpc: r - 1.0e-5,
                phi: 0.2,
                z_pc: 0.0,
                t_gyr: 13.0,
            },
            &gp,
        );
        let b = e.local(
            Place {
                r_kpc: r + 1.0e-5,
                phi: 0.2,
                z_pc: 0.0,
                t_gyr: 13.0,
            },
            &gp,
        );
        assert!(close(a.texture, b.texture, 0.001), "texture jumps at {r}");
        let p = Place {
            r_kpc: r,
            phi: 0.2,
            z_pc: 0.0,
            t_gyr: 10.0 + f64::from(k) * 0.0127,
        };
        let q = Place {
            t_gyr: p.t_gyr + 1.0e-4,
            ..p
        };
        assert!(
            close(e.local(p, &gp).gas_pc3, e.local(q, &gp).gas_pc3, 0.01),
            "gas jumps in time at {}",
            p.t_gyr
        );
    }
}

#[test]
fn solar_neighbourhood_now() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let gp = GreenParams::default();
    // Azimuthal mean at the Sun's radius and height (arms average out over phi).
    let n = 64;
    let mut sum = [0.0f64; 6];
    for k in 0..n {
        let l = e.local(
            Place {
                r_kpc: e.sun_radius_kpc(),
                phi: sse_core::math::TAU * f64::from(k) / f64::from(n),
                z_pc: 20.0,
                t_gyr: 13.8,
            },
            &gp,
        );
        for (s, v) in sum.iter_mut().zip([
            l.systems_pc3,
            l.gas_pc3,
            l.sterilising_per_gyr,
            l.grb_per_gyr,
            l.encounters_per_myr,
            l.c_o,
        ]) {
            *s += v / f64::from(n);
        }
    }
    let [systems, gas, steril, grb, enc, c_o] = sum;
    println!("systems {systems:.3} gas {gas:.3} sterilising {steril:.2} grb {grb:.2} encounters {enc:.3} C/O {c_o:.2}");
    // Local stellar density 0.04 systems per pc^3 for stars (Reid et al. 2002; about 0.1 with brown dwarfs and remnants).
    assert!((0.03..0.3).contains(&systems), "systems per pc^3 {systems}");
    // Midplane gas about 0.05 solar masses per pc^3 (McKee, Parravano and Hollenbach 2015 give 0.04 to 0.05).
    assert!((0.015..0.15).contains(&gas), "gas {gas}");
    // Supernovae within 10 pc: about 1 to 3 per Gyr (Gehrels et al. 2003, scaled from 8 pc).
    assert!((0.3..6.0).contains(&steril), "sterilising {steril}");
    // Lethal gamma-ray bursts: Piran and Jimenez 2014 give about 0.5 to 1 per Gyr at the Sun now.
    assert!((0.1..2.0).contains(&grb), "bursts {grb}");
    // Passages within 0.1 pc: a few per Myr at most (Bailer-Jones et al. 2018 find about 20 within 1 pc per Myr).
    assert!((0.02..1.0).contains(&enc), "encounters {enc}");
    assert!((0.35..0.8).contains(&c_o), "C/O {c_o}");
    // In an arm, more gas and star formation than between arms.
    let arm = (0..n)
        .map(|k| {
            e.local(
                Place {
                    r_kpc: 8.0,
                    phi: sse_core::math::TAU * f64::from(k) / f64::from(n),
                    z_pc: 0.0,
                    t_gyr: 13.8,
                },
                &gp,
            )
        })
        .fold((f64::MAX, 0.0f64), |(lo, hi), l| {
            (lo.min(l.arm), hi.max(l.arm))
        });
    assert!(arm.0 < 0.6 && arm.1 > 1.4, "arm contrast {arm:?}");
}

#[test]
fn green_band_and_its_drift() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let gp = GreenParams::default();
    let s = g.now.disc_kpc / 2.6; // Milky Way kpc
    let now = e.green_band(13.8, &gp, true, 0.25).expect("a band now");
    let (a, b) = (now.0 / s, now.1 / s);
    println!("band now {a:.2} to {b:.2} kpc (Milky Way scaled)");
    assert!(a < 8.2 && b > 8.2, "the Sun's radius is in the band");
    assert!((3.0..8.0).contains(&a), "inner edge {a}");
    assert!(b > 12.0, "outer edge {b}");
    // Earlier the band is narrower and its outer edge further in: it grows outward with time.
    let then = e
        .green_band(13.8 - 5.0, &gp, true, 0.25)
        .expect("a band 5 Gyr ago");
    println!("band 5 Gyr ago {:.2} to {:.2}", then.0 / s, then.1 / s);
    assert!(now.1 > then.1 + 2.0 * s, "outer edge moves out");
    assert!(now.1 - now.0 > then.1 - then.0, "band widens");
    // Complex life nowhere before the universe was 6 Gyr old (Lineweaver et al. 2004: the zone emerges about 8 Gyr ago).
    for t in [3.0, 4.0, 5.0, 6.0] {
        assert!(
            e.green_band(t, &gp, true, 0.25).is_none(),
            "complex band at {t} Gyr"
        );
    }
    // Simple-life conditions come first, near the Sun's radius.
    let first = (0..40)
        .map(|k| 2.0 + 0.25 * f64::from(k))
        .find(|t| e.green_band(*t, &gp, false, 0.25).is_some())
        .expect("habitable at some time");
    let fb = e.green_band(first, &gp, false, 0.25).unwrap();
    println!(
        "habitable first at {first} Gyr, {:.2} to {:.2}",
        fb.0 / s,
        fb.1 / s
    );
    assert!(
        (3.0..8.0).contains(&first),
        "habitable first at {first} Gyr"
    );
    // A pack can redefine habitable: with no metallicity ceiling the band reaches further in.
    let loose = GreenParams {
        feh_max: 9.0,
        encounters_max_per_myr: 9.0,
        ..gp.clone()
    };
    assert!(e.green_band(13.8, &loose, true, 0.25).unwrap().0 < now.0);
}

#[test]
fn outcome_first_finds_green_places() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let gp = GreenParams::default();
    let found = e.find_green(13.8, &gp, 5);
    assert_eq!(found.len(), 5);
    assert!(found.iter().all(|(_, gr)| gr.complex));
    assert!(found
        .windows(2)
        .all(|w| w[0].0.sterilising_per_gyr <= w[1].0.sterilising_per_gyr));
    assert_eq!(found, e.find_green(13.8, &gp, 5), "deterministic");
}

#[test]
fn a_galaxy_feeds_the_patch_machinery() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let base = crate::PatchModel::milky_way_like();
    let pm = e.patch_model(&base);
    pm.validate().unwrap();
    // The fitted clock reproduces the Sun's ring at its birth and now.
    let r0 = pm.sun_radius_kpc;
    let born = crate::patch::feh_ism(&pm, r0, pm.now_gyr - 4.57);
    let want = e.surface(r0, pm.now_gyr - 4.57).feh;
    assert!((born - want).abs() < 0.02, "{born} against {want}");
    println!(
        "gradient {:.3} tau {:.2} feh now {:.2}",
        pm.enrichment.gradient_dex_per_kpc, pm.enrichment.tau_gyr, pm.enrichment.feh_now_at_sun
    );
    let mut ch = sse_chronicle::Chronicle::new();
    let site = crate::Site {
        r_kpc: r0,
        z_pc: 20.0,
        size_pc: 8.0,
        epoch: crate::patch::gyr_time(pm.now_gyr),
    };
    let f = crate::form_patch(
        &mut ch,
        &Seed::from_text("g4"),
        &pm,
        "u:g4/patch:solar".parse().unwrap(),
        &site,
    )
    .unwrap();
    assert!(!f.systems.is_empty());
    assert!((f.conditions.feh_ism - pm.enrichment.feh_now_at_sun).abs() < 1.0e-6);
}

#[test]
fn pack_settings_are_the_defaults_and_golden_holds() {
    let cp = crate::scenario::CosmologyPack::from_packs(&[crate::scenario::base_cosmology_pack()])
        .unwrap();
    // The pack's settings are the defaults, plus the home galaxy's measured arms.
    let mut generic = cp.envelope.clone();
    generic.measured = Default::default();
    assert_eq!(generic, EnvelopeParams::default());
    assert_eq!(cp.envelope.measured.arms.len(), 7);
    assert_eq!(cp.green, GreenParams::default());
    crate::golden::check("envelope.json", &crate::scenario::envelope_golden());
}

/// Arms trail (design 9.4, Alex 24 Nov): the disc turns towards +phi (the
/// stars' orbits advance phi), so walking outward along a crest the arm
/// falls back in phi. All 146 spirals with known spin and tilt in Iye,
/// Tadaki and Fukumoto (2019, arXiv:1910.10926) trail.
#[test]
fn arms_trail_the_disc_turning() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let p = &e.params;
    let rd = g.now.disc_kpc.max(0.1);
    let tan_pitch = sse_core::math::tan(p.pitch_deg * sse_core::math::PI / 180.0);
    let t = 13.8;
    // The crest's phi at a radius: the azimuth (of 720) where the arm term peaks,
    // searched within half an arm spacing of the inner crest.
    let crest = |r: f64, near: f64| {
        let span = sse_core::math::PI / f64::from(p.arms.max(1));
        (0..=720)
            .map(|i| near - span + 2.0 * span * f64::from(i) / 720.0)
            .max_by(|a, b| {
                let at = |phi: f64| {
                    e.arm(&Place {
                        r_kpc: r,
                        phi,
                        z_pc: 0.0,
                        t_gyr: t,
                    })
                };
                at(*a).total_cmp(&at(*b))
            })
            .unwrap()
    };
    let inner = crest(rd, 0.0);
    // Out by a radius that winds the crest by 0.3 rad.
    let r_out = rd * sse_core::math::exp(0.3 * tan_pitch);
    let outer = crest(r_out, inner);
    assert!(
        (outer - (inner - 0.3)).abs() < 0.02,
        "crest at {inner} inside, {outer} outside: the arm should fall back by 0.3 rad"
    );
}

fn measured_env<'a>(g: &'a Galaxy, d: &'a Disc) -> GalaxyEnvelope<'a> {
    let cp = crate::scenario::CosmologyPack::from_packs(&[crate::scenario::base_cosmology_pack()])
        .unwrap();
    let mut e = env(g, d);
    e.params = cp.envelope;
    e.params.measured.frame_phi_rad = 0.7;
    e.params.measured.prepare();
    e
}

/// The home galaxy's arms are measured (Reid et al. 2019, ApJ 885, 131,
/// Table 2): each arm's crest passes through its kink at R_kink (scaled to the
/// galaxy's own Sun radius) and beta_kink from the home's direction, and the
/// Sun now sits in the Local Arm.
#[test]
fn the_home_galaxy_has_its_measured_arms() {
    use sse_core::math as m;
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = measured_env(&g, &d);
    let now = g.now.t_gyr;
    let k = e.params.measured.r0_kpc / e.sun_radius_kpc();
    let at = |r: f64, phi: f64, t: f64| {
        e.arm(&Place {
            r_kpc: r,
            phi,
            z_pc: 0.0,
            t_gyr: t,
        })
    };
    for a in &e.params.measured.arms {
        let r = a.r_kink_kpc / k;
        let phi = 0.7 + a.beta_kink_deg * m::PI / 180.0;
        // On the crest the boost is a local peak along the ring.
        let here = at(r, phi, now);
        let dphi = 3.0 * a.width_kpc / k / r;
        assert!(
            here > at(r, phi + dphi, now) && here > at(r, phi - dphi, now),
            "{}: {here} not a crest",
            a.name
        );
        assert!(here > 1.0, "{}: {here}", a.name);
    }
    // The Sun (beta 0 at its radius) is inside the Local Arm's width.
    let sun = at(e.sun_radius_kpc(), 0.7, now);
    assert!(sun > 1.5, "the Sun's arm boost {sun}");
    // The pattern turns: 100 Myr on, the same place has moved off the arm's
    // crest by the pattern speed (the arms keep their shape).
    let omega_p = e.params.pattern_kms_kpc * 1.0227;
    let later = at(e.sun_radius_kpc(), 0.7 + omega_p * 0.1, now + 0.1);
    assert!((later - sun).abs() < 1e-9, "{later} {sun}");
    // The crests the chart draws pass through the kinks too.
    let crests = e.arm_crests(now, 20.0);
    assert_eq!(crests.len(), 7);
    assert!(crests.iter().any(|(n, _)| n == "Local"));
}

/// The measured arms move a ring's gas between arm and interarm but keep its
/// total: the boost averages to about 1 round every ring.
#[test]
fn measured_arms_keep_each_rings_total() {
    use sse_core::math as m;
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = measured_env(&g, &d);
    let now = g.now.t_gyr;
    for r in [3.0, 4.5, 6.0, 8.2, 10.0, 13.0] {
        let n = 2880;
        let mean: f64 = (0..n)
            .map(|i| {
                e.arm(&Place {
                    r_kpc: r,
                    phi: 2.0 * m::PI * f64::from(i) / f64::from(n),
                    z_pc: 0.0,
                    t_gyr: now,
                })
            })
            .sum::<f64>()
            / f64::from(n);
        assert!((mean - 1.0).abs() < 0.12, "ring {r} kpc averages {mean}");
    }
}

/// A bar sweeps its gas into the nucleus (Local Arm package): once the home
/// galaxy is bar-unstable, the rings between the nucleus and the bar's end
/// hold almost no gas, the nucleus holds more than the disc beyond, and no
/// place there counts as green for new systems, while the disc beyond the
/// bar keeps its gas and its green verdict.
#[test]
fn a_bar_sweeps_its_gas_into_the_nucleus() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let gp = GreenParams::default();
    let rp = crate::rings::RingParams::default();
    let rd = g.now.disc_kpc;
    let tb = g.bar_from_gyr.expect("the Milky-Way-like galaxy is barred");
    assert!(tb < 13.0, "bar from {tb}");
    let now = 13.8;
    let swept = e.surface(0.6 * rd, now).gas;
    let beyond = e.surface(2.0 * rd, now).gas;
    let nucleus = e.surface(0.11 * rd, now).gas;
    // Measured: the Milky Way's gas between about 0.3 and 3 kpc is a
    // hundredth or less of the disc's beyond (absolute floor: under 0.1
    // solar masses per pc², where the disc holds several).
    assert!(swept < 0.1 && beyond > 3.0, "swept {swept} beyond {beyond}");
    assert!(nucleus > beyond, "nucleus {nucleus} beyond {beyond}");
    for r in d
        .rings
        .iter()
        .filter(|r| r.radius_kpc > rp.bar_nucleus_rd * rd)
    {
        if r.radius_kpc < rp.bar_rd * rd {
            assert!(
                r.swept_from_gyr.is_some_and(|t| t >= tb - 0.06),
                "{}",
                r.radius_kpc
            );
        } else {
            assert!(r.swept_from_gyr.is_none(), "{}", r.radius_kpc);
        }
    }
    let at = |r: f64, t: f64| {
        e.local(
            Place {
                r_kpc: r,
                phi: 0.0,
                z_pc: 0.0,
                t_gyr: t,
            },
            &gp,
        )
    };
    let inside = e.green(&at(0.6 * rd, now), &gp);
    assert!(!inside.gas_kept && !inside.habitable && !inside.complex);
    // Before the bar the same place kept its gas.
    assert!(e.green(&at(0.6 * rd, tb - 0.5), &gp).gas_kept);
    assert!(e.green(&at(e.sun_radius_kpc(), now), &gp).gas_kept);
}

/// Each star is counted once: mergers move a share of the disc's stars into
/// the bulge and the rings give the same share up, so the rings' stars and
/// the bulge's together are the galaxy's, to the part that fell in from
/// outside (the bulge's ex-situ stars, which the rings never held).
#[test]
fn the_bulge_and_the_disc_count_each_star_once() {
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let e = env(&g, &d);
    let rings: f64 = (0..d.rings.len())
        .map(|i| d.states[i].last().map_or(0.0, |s| s.stars))
        .sum();
    let joined: f64 = g.bulge_joins.iter().map(|b| b.1).sum();
    let burst: f64 = g.bulge_bursts.iter().map(|b| b.1).sum();
    assert!(g.now.bulge > 0.0 && (joined + burst - g.now.bulge).abs() < 1.0e-6 * g.now.bulge);
    // The rings gave up what mergers moved: their stars are fewer than they
    // formed, by no more than the share every move took (a star born before
    // every move lost the most).
    assert!(!g.disc_moves.is_empty());
    let formed: f64 = d.rings.iter().map(|r| r.stars()).sum();
    let all_moves: f64 = g.disc_moves.iter().map(|(_, f)| 1.0 - f).product();
    println!("rings {rings:.3e} formed {formed:.3e} kept at least {all_moves:.3}");
    assert!(rings < formed * (1.0 - 1.0e-9) && rings >= formed * all_moves * (1.0 - 1.0e-9));
    // The bulge's births as they stood at a time hold only what had joined by then.
    let half = g.bulge_births_at(8.0).iter().map(|b| b.1).sum::<f64>();
    assert!(
        (half - g.bulge_at(8.0)).abs() < 0.1 * g.bulge_at(8.0).max(1.0),
        "{half} {}",
        g.bulge_at(8.0)
    );
    // Its density is the Hernquist sphere's: the mass inside a large radius is nearly all of it.
    let a = g.bulge_a_kpc(13.8);
    assert!(a > 0.05 && a < 3.0, "bulge scale {a}");
    let rho = e.bulge_pc3(Place {
        r_kpc: a,
        phi: 0.0,
        z_pc: 0.0,
        t_gyr: 13.8,
    });
    let want = g.now.bulge * a * 1000.0
        / (2.0
            * sse_core::math::PI
            * (a * 1000.0)
            * ((2.0 * a * 1000.0) * (2.0 * a * 1000.0) * (2.0 * a * 1000.0)));
    assert!(
        (rho - want).abs() < 1.0e-9 * want.max(1.0e-12),
        "{rho} {want}"
    );
}
