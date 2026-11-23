//! GH4: remains as places (design 7.6).

use crate::envelope::{EnvelopeParams, GalaxyEnvelope, GreenParams, Place};
use crate::feed::{with_remains, Fed};
use crate::galaxy::GalaxyParams;
use crate::gh3_tests::{fed, world, World};
use crate::infall::OrbitParams;
use crate::remains::RemainsParams;
use crate::rings::RingParams;
use crate::stars::{StarParams, Stars};
use sse_core::math as m;
use sse_core::{Address, Seed};

pub(crate) fn fed_remains(w: &World, seed: &str, mass: f64, name: &str) -> Fed {
    let mut f = fed(w, seed, mass, name);
    with_remains(
        &mut f,
        &GalaxyParams::default(),
        &OrbitParams::default(),
        &RingParams::default(),
        &RemainsParams::default(),
        Seed::from_text(seed),
    );
    f
}

fn disc_of(f: &Fed) -> crate::rings::Disc {
    crate::rings::evolve(
        &f.galaxy,
        &GalaxyParams::default(),
        &RingParams::default(),
        Seed::from_text("gh4"),
        &Address::universe("gh4").unwrap(),
    )
}

fn env<'a>(f: &'a Fed, d: &'a crate::rings::Disc) -> GalaxyEnvelope<'a> {
    GalaxyEnvelope {
        galaxy: &f.galaxy,
        disc: d,
        params: EnvelopeParams::default(),
        seed: Seed::from_text("gh4"),
        address: Address::universe("gh4").unwrap(),
        remains: Default::default(),
    }
}

#[test]
#[ignore]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn explore() {
    let w = world();
    for i in 0..4 {
        let t0 = std::time::Instant::now();
        let f = fed(&w, "gh3", 1.0e12, &format!("x{i}"));
        let t1 = t0.elapsed();
        let f = {
            let mut f = f;
            let t = std::time::Instant::now();
            with_remains(
                &mut f,
                &GalaxyParams::default(),
                &OrbitParams::default(),
                &RingParams::default(),
                &RemainsParams::default(),
                Seed::from_text("gh3"),
            );
            println!("fed {t1:.2?}, remains {:.2?}", t.elapsed());
            f
        };
        let a = f.galaxy.assembled.as_ref().unwrap();
        let now = f.galaxy.now.t_gyr;
        let d = disc_of(&f);
        let e = env(&f, &d);
        let lost: f64 = a.components.iter().map(|c| c.lost_by(now)).sum();
        let (op, rp) = a.shapes_with.as_ref().unwrap();
        let grid = crate::remains::total_on_grid(&a.components, op, rp, now, 300.0, 60);
        println!(
            "host {i}: halo {:.2e} components {} lost {:.2e} on grid {:.2e}",
            a.stellar_halo_msun,
            a.components.len(),
            lost,
            grid
        );
        for c in &a.components {
            println!(
                "   {} stars {:.2e} peri {:.1} apo {:.1} P {:.2} s {:.3} births {} torn {:?} merged {:?}",
                c.from, c.stars_msun, c.peri_kpc, c.apo_kpc, c.period_gyr, c.spread, c.births.len(), c.torn_gyr, c.merged_gyr
            );
        }
        let sun = e.sun_radius_kpc();
        for z in [0.0, 1000.0, 3000.0] {
            let p = Place {
                r_kpc: sun,
                phi: 0.0,
                z_pc: z,
                t_gyr: now,
            };
            let l = e.local(p, &GreenParams::default());
            println!(
                "   at the Sun's radius, z {z} pc: disc stars {:.3e}, remains {:.3e} per pc3, share {:.4}",
                l.stars_pc3 - l.remains_pc3,
                l.remains_pc3,
                l.remains_pc3 / l.stars_pc3
            );
        }
        let st = Stars {
            env: &e,
            green: GreenParams::default(),
            params: StarParams::default(),
            seed: Seed::from_text("g5"),
            galaxy: Address::universe("g5").unwrap(),
        };
        let frame = st.frame_of(now * 1000.0);
        {
            let t = std::time::Instant::now();
            let mut acc = 0.0;
            for q in 0..10000 {
                acc += st.remains_density_at(
                    [sun * 1000.0 + q as f64, 0.0, 0.0],
                    frame as f64 * st.params.frame_myr,
                );
            }
            println!("   10000 density reads {:.2?} ({acc:.2e})", t.elapsed());
            let t = std::time::Instant::now();
            for q in 0..100 {
                let _ = st.remains_cdf_at(
                    [sun * 1000.0 + q as f64, 0.0, 0.0],
                    frame as f64 * st.params.frame_myr,
                );
            }
            println!("   100 birth mixes {:.2?}", t.elapsed());
        }
        let c = st.params.cell_pc;
        let t = std::time::Instant::now();
        let (mut disc, mut halo) = (0, 0);
        let k = m::floor(sun * 1000.0 / c) as i64;
        for x in 0..4 {
            for y in 0..4 {
                for z in -2..2 {
                    for s in st.cell([k + x, y, z], frame, &[]).unwrap() {
                        if s.born_in.is_some() {
                            halo += 1;
                        } else {
                            disc += 1;
                        }
                    }
                }
            }
        }
        println!(
            "   64 cells near the Sun: {disc} disc, {halo} halo, {:.2?}",
            t.elapsed()
        );
        let plain = fed(&w, "gh3", 1.0e12, &format!("x{i}"));
        let d2 = disc_of(&plain);
        let e2 = env(&plain, &d2);
        let st2 = Stars {
            env: &e2,
            green: GreenParams::default(),
            params: StarParams::default(),
            seed: Seed::from_text("g5"),
            galaxy: Address::universe("g5").unwrap(),
        };
        let t = std::time::Instant::now();
        let mut n2 = 0;
        for x in 0..4 {
            for y in 0..4 {
                for z in -2..2 {
                    n2 += st2.cell([k + x, y, z], frame, &[]).unwrap().len();
                }
            }
        }
        println!("   without remains: {n2} disc, {:.2?}", t.elapsed());
    }
}

fn spearman(a: &[f64], b: &[f64]) -> f64 {
    let rank = |v: &[f64]| {
        let mut idx: Vec<usize> = (0..v.len()).collect();
        idx.sort_by(|i, j| v[*i].total_cmp(&v[*j]));
        let mut r = vec![0.0; v.len()];
        for (k, i) in idx.into_iter().enumerate() {
            r[i] = k as f64;
        }
        r
    };
    let (ra, rb) = (rank(a), rank(b));
    let n = a.len() as f64;
    let d2: f64 = ra.iter().zip(&rb).map(|(x, y)| (x - y) * (x - y)).sum();
    1.0 - 6.0 * d2 / (n * (n * n - 1.0))
}

struct Host {
    fed: Fed,
    disc: crate::rings::Disc,
}

fn hosts(n: usize) -> Vec<Host> {
    let w = world();
    (0..n)
        .map(|i| {
            let fed = fed_remains(&w, "gh3", 1.0e12, &format!("x{i}"));
            let disc = disc_of(&fed);
            Host { fed, disc }
        })
        .collect()
}

/// Every star a member lost is in one of its components, and each shape
/// holds the mass it was given (budget closure).
#[test]
fn remains_close_their_budget() {
    let w = world();
    let f = fed_remains(&w, "gh3", 1.0e12, "x0");
    let a = f.galaxy.assembled.as_ref().unwrap();
    let now = f.galaxy.now.t_gyr;
    let lost: f64 = a.components.iter().map(|c| c.lost_by(now)).sum();
    // Members whose loss is under the floor are left out: a few thousand suns.
    assert!(
        (lost - a.stellar_halo_msun).abs() < 1.0e-3 * a.stellar_halo_msun,
        "{lost} {}",
        a.stellar_halo_msun
    );
    let (op, rp) = a.shapes_with.as_ref().unwrap();
    let big = a
        .components
        .iter()
        .max_by(|x, y| x.stars_msun.total_cmp(&y.stars_msun))
        .unwrap();
    let one = std::slice::from_ref(big);
    let half = 2.0 * big.apo_kpc.max(5.0);
    let on_grid = crate::remains::total_on_grid(one, op, rp, now, half, 72);
    let want = big.lost_by(now);
    assert!(
        (on_grid / want - 1.0).abs() < 0.15,
        "{on_grid:.3e} on the grid against {want:.3e}"
    );
}

/// Design 7.6's checks over twelve Milky-Way-mass hosts: the stellar halo's
/// mass and metals rise with the biggest thing it swallowed (Harmsen et al.
/// 2017, doi:10.1093/mnras/stw2992; D'Souza and Bell 2018,
/// doi:10.1093/mnras/stx3081), one or two progenitors supply most of it
/// (Deason, Mao and Wechsler 2016, doi:10.3847/0004-637X/821/1/5), it thins
/// faster beyond the biggest one's farthest reach (Deason et al. 2018,
/// doi:10.3847/2041-8213/aad0ee), and near the Sun a few stars in a
/// thousand belong to it.
#[test]
fn the_stellar_halo_is_made_of_what_was_swallowed() {
    let hs = hosts(12);
    let (mut mass, mut metals, mut biggest, mut top2, mut share) =
        (vec![], vec![], vec![], vec![], vec![]);
    let mut steeper = 0;
    for h in &hs {
        let a = h.fed.galaxy.assembled.as_ref().unwrap();
        let now = h.fed.galaxy.now.t_gyr;
        let mut s: Vec<f64> = a.components.iter().map(|c| c.stars_msun).collect();
        s.sort_by(|x, y| y.total_cmp(x));
        mass.push(a.stellar_halo_msun);
        metals.push(a.stellar_halo_z);
        biggest.push(s[0]);
        top2.push((s[0] + s.get(1).copied().unwrap_or(0.0)) / a.stellar_halo_msun);
        let e = env(&h.fed, &h.disc);
        let l = e.local(
            Place {
                r_kpc: e.sun_radius_kpc(),
                phi: 0.0,
                z_pc: 0.0,
                t_gyr: now,
            },
            &GreenParams::default(),
        );
        share.push(l.remains_pc3 / l.stars_pc3);
        // Spherically averaged density inside and beyond the biggest one's
        // farthest reach.
        let big = a
            .components
            .iter()
            .max_by(|x, y| x.stars_msun.total_cmp(&y.stars_msun))
            .unwrap();
        let apo = big.apo_kpc;
        let shell = |r: f64| {
            let n = 64;
            (0..n)
                .map(|k| {
                    let mu = -1.0 + (2.0 * k as f64 + 1.0) / n as f64;
                    let phi = 2.4 * k as f64;
                    let q = m::sqrt(1.0 - mu * mu);
                    e.remains_pc3(Place {
                        r_kpc: r * q,
                        phi,
                        z_pc: 1000.0 * r * mu,
                        t_gyr: now,
                    })
                })
                .sum::<f64>()
                / n as f64
        };
        let (d1, d2, d3) = (shell(0.5 * apo), shell(apo), shell(2.0 * apo));
        let inner = m::ln(d2 / d1) / m::ln(2.0);
        let outer = m::ln(d3 / d2) / m::ln(2.0);
        if outer < inner {
            steeper += 1;
        }
    }
    let mut t2 = top2.clone();
    t2.sort_by(|a, b| a.total_cmp(b));
    let rho_mass = spearman(&mass, &biggest);
    let rho_z = spearman(&metals, &biggest);
    let rho_share = spearman(&share, &mass);
    assert!(rho_mass > 0.6, "halo mass against the biggest: {rho_mass}");
    assert!(rho_z > 0.3, "halo metals against the biggest: {rho_z}");
    assert!(t2[6] > 0.6, "one or two supply most: {top2:?}");
    assert!(
        steeper >= 9,
        "thinner beyond the biggest one's reach in {steeper} of 12"
    );
    assert!(
        rho_share > 0.6,
        "the share near the Sun follows the halo: {rho_share}"
    );
    for (s, mh) in share.iter().zip(&mass) {
        if *mh > 1.0e9 {
            assert!(*s > 3.0e-4 && *s < 3.0e-2, "{s} at a halo of {mh:.2e}");
        }
    }
}

/// A halo star says where it was born, carries that galaxy's metals, and is
/// the same star when it is asked for again by its address.
#[test]
fn a_halo_star_keeps_where_it_was_born() {
    let w = world();
    // x3: a host with a heavy stellar halo (about 6e9 M☉).
    let f = fed_remains(&w, "gh3", 1.0e12, "x3");
    let d = disc_of(&f);
    let e = env(&f, &d);
    let st = Stars {
        env: &e,
        green: GreenParams::default(),
        params: StarParams::default(),
        seed: Seed::from_text("g5"),
        galaxy: Address::universe("g5").unwrap(),
    };
    let now = f.galaxy.now.t_gyr;
    let frame = st.frame_of(now * 1000.0);
    let k = m::floor(e.sun_radius_kpc() * 1000.0 / st.params.cell_pc) as i64;
    let (mut halo, mut disc) = (vec![], vec![]);
    for x in 0..4 {
        for y in 0..4 {
            for z in -2..2 {
                for s in st.cell([k + x, y, z], frame, &[]).unwrap() {
                    if s.born_in.is_some() {
                        halo.push(s);
                    } else {
                        disc.push(s);
                    }
                }
            }
        }
    }
    assert!(!halo.is_empty(), "no halo star among {}", disc.len());
    let mean =
        |v: &[crate::stars::StarSystem]| v.iter().map(|s| s.feh).sum::<f64>() / v.len() as f64;
    assert!(mean(&halo) < mean(&disc), "{} {}", mean(&halo), mean(&disc));
    let b = st.birth(&halo[0]);
    assert_eq!(b.population, "stellar halo");
    let born = halo[0].born_in.as_ref().unwrap();
    assert!(
        born.says.starts_with("born in ") && born.says.contains("fell in"),
        "{}",
        born.says
    );
    assert!(halo[0].address.to_string().contains("/sys:f"));
    for s in &halo {
        let again = st.draw_one(&s.address).unwrap().unwrap();
        assert_eq!(&again, s);
    }
    // A promoted halo star keeps its place and its slot: the cell gives it
    // back with the same neighbours.
    let pin = st.promote(&halo[0]);
    let with = st
        .cell(halo[0].cell, frame, std::slice::from_ref(&pin))
        .unwrap();
    let without = st.cell(halo[0].cell, frame, &[]).unwrap();
    assert_eq!(with.len(), without.len());
    assert!(with.iter().any(|s| s.pinned && s.address == pin.address));
}

/// A star too old, or too poor in metals, for any birthplace in the disc
/// fits one born in a galaxy this one swallowed.
#[test]
fn origin_reaches_across_galaxies() {
    let w = world();
    let f = fed_remains(&w, "gh3", 1.0e12, "x3");
    let d = disc_of(&f);
    let e = env(&f, &d);
    let now = f.galaxy.now.t_gyr;
    let ask = crate::origin::OriginAsk {
        now_gyr: now,
        age_gyr: 12.0,
        r_now_kpc: e.sun_radius_kpc(),
        phi_now_deg: 0.0,
        keep_feh: Some(-2.5),
    };
    let mut halo = vec![];
    for age in [9.0, 10.0, 11.0, 12.0, 12.5] {
        for feh in [-2.5, -2.0, -1.5, -1.2, -0.9, -0.6] {
            let o = crate::origin::reconstruct(
                &e,
                &crate::origin::OriginParams::default(),
                &crate::origin::OriginAsk {
                    age_gyr: age,
                    keep_feh: Some(feh),
                    ..ask.clone()
                },
            );
            if o.born_in.is_some() {
                assert!(o.why.contains("stellar halo"), "{}", o.why);
                halo.push((age, feh));
            }
        }
    }
    assert!(!halo.is_empty());
    // A star that fits the disc stays a disc star.
    let ask = crate::origin::OriginAsk {
        age_gyr: 4.6,
        keep_feh: None,
        ..ask
    };
    let o = crate::origin::reconstruct(&e, &crate::origin::OriginParams::default(), &ask);
    assert!(o.born_in.is_none(), "{}", o.why);
}
