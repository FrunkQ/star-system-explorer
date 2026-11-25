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
    assert_eq!(cp.envelope, EnvelopeParams::default());
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
