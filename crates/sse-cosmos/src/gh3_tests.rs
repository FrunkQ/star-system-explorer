//! GH3: progenitors feed the host; merger responses and marks (design 7.5, 9.4).

use crate::feed::{evolve_fed, Fed, ResponseParams};
use crate::galaxy::GalaxyParams;
use crate::haloes::{HaloParams, Haloes};
use crate::infall::OrbitParams;
use sse_core::{Address, Seed};

pub(crate) struct World {
    pub c: crate::cosmology::Cosmology,
    pub w: crate::web::Web,
}

pub(crate) fn world() -> World {
    let (c, w) = crate::g3_tests::world();
    World { c, w }
}

pub(crate) fn haloes<'a>(w: &'a World, seed: &str) -> Haloes<'a> {
    Haloes {
        cosmo: &w.c,
        web: &w.w,
        params: HaloParams::default(),
        seed: Seed::from_text(seed),
        universe: Address::universe("gh3").unwrap(),
    }
}

pub(crate) fn fed(w: &World, seed: &str, mass: f64, name: &str) -> Fed {
    let hs = haloes(w, seed);
    let h = hs.median(mass, name);
    evolve_fed(
        &w.c,
        &hs,
        &GalaxyParams::default(),
        &OrbitParams::default(),
        &ResponseParams::default(),
        &h,
        &[],
    )
    .unwrap()
}

#[test]
#[ignore]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn explore() {
    let w = world();
    for mass in [1.0e11, 1.0e12, 1.0e13] {
        for i in 0..4 {
            let t0 = std::time::Instant::now();
            let f = fed(&w, "gh3", mass, &format!("x{i}"));
            let hs = haloes(&w, "gh3");
            let h = hs.median(mass, &format!("x{i}"));
            let u = crate::galaxy::evolve(&w.c, &GalaxyParams::default(), &h);
            let a = f.galaxy.assembled.as_ref().unwrap();
            let n = &f.galaxy.now;
            println!(
                "M {mass:.0e} {i}: {:.2?} stars {:.2e} (unfed {:.2e}) bulge {:.2} exsitu {:.3} halo {:.2e} z {:.2e} thick {:.2} big {:.2} merged {} torn {} of {} marks {} bh {:.2e} (unfed {:.2e}) {:?}",
                t0.elapsed(), n.stars, u.now.stars, n.bulge / n.stars, a.exsitu_msun / n.stars,
                a.stellar_halo_msun, a.stellar_halo_z, a.thick_disc_msun / (n.stars - n.bulge).max(1.0),
                a.biggest_share, a.merged, a.torn_apart, a.members, a.marks.len(), n.black_hole, u.now.black_hole, f.galaxy.morphology
            );
            let mut kinds = std::collections::BTreeMap::<String, (u32, f64)>::new();
            for m in &a.marks {
                let e = kinds.entry(m.kind.clone()).or_default();
                e.0 += 1;
                e.1 = e.1.max(m.strength);
            }
            println!("   {kinds:?}");
        }
    }
}

/// Design 7.5's checks over many Milky-Way-mass hosts: the stars not born
/// here come mostly from a few big progenitors, and their metals rise with
/// the stellar mass of the biggest one (D'Souza and Bell 2018,
/// doi:10.1038/s41550-018-0533-x; Deason, Mao and Wechsler 2016,
/// doi:10.3847/0004-637X/821/1/5).
#[test]
fn the_stellar_halo_follows_its_biggest_progenitor() {
    let w = world();
    let mut rows = vec![];
    for i in 0..16 {
        let f = fed(&w, "halo-check", 1.0e12, &format!("h{i}"));
        let a = f.galaxy.assembled.clone().unwrap();
        if a.exsitu_msun <= 0.0 {
            continue;
        }
        // The biggest contributor's stars at the time it fell in.
        let big = f
            .members
            .iter()
            .find(|mb| mb.progenitor.address == a.biggest)
            .unwrap();
        let st = crate::feed::state_at(&big.galaxy, big.progenitor.t_infall_gyr);
        rows.push((st.stars, a.exsitu_z, a.biggest_share, a.exsitu_msun));
    }
    assert!(rows.len() >= 12, "{}", rows.len());
    let mut shares: Vec<f64> = rows.iter().map(|r| r.2).collect();
    shares.sort_by(f64::total_cmp);
    let median = shares[shares.len() / 2];
    assert!(median > 0.25, "{shares:?}");
    // Rank correlation of halo metals with the biggest one's stars.
    let rank = |k: usize| {
        let mut ix: Vec<usize> = (0..rows.len()).collect();
        ix.sort_by(|&a, &b| {
            let (x, y) = match k {
                0 => (rows[a].0, rows[b].0),
                _ => (rows[a].1, rows[b].1),
            };
            x.total_cmp(&y)
        });
        let mut r = vec![0.0; rows.len()];
        for (pos, &i) in ix.iter().enumerate() {
            r[i] = pos as f64;
        }
        r
    };
    let (ra, rb) = (rank(0), rank(1));
    let n = rows.len() as f64;
    let d2: f64 = ra.iter().zip(&rb).map(|(a, b)| (a - b) * (a - b)).sum();
    let rho = 1.0 - 6.0 * d2 / (n * (n * n - 1.0));
    assert!(rho > 0.4, "{rho} {rows:?}");
}

/// The share of stars not born here rises with mass (Rodriguez-Gomez et
/// al. 2016, doi:10.1093/mnras/stv2786): a few per cent in small hosts, a
/// tenth or so at the Milky Way's mass, most of them in giant ellipticals.
#[test]
fn the_share_of_stars_not_born_here_rises_with_mass() {
    let w = world();
    let share = |mass: f64| {
        let mut s = 0.0;
        for i in 0..6 {
            let f = fed(&w, "exsitu", mass, &format!("e{i}"));
            s += f.galaxy.assembled.unwrap().exsitu_msun / f.galaxy.now.stars;
        }
        s / 6.0
    };
    let (a, b, c) = (share(1.0e11), share(1.0e12), share(1.0e13));
    assert!(a < b && b < c, "{a} {b} {c}");
    assert!(a < 0.1, "{a}");
    assert!((0.02..0.4).contains(&b), "{b}");
    assert!(c > 0.4, "{c}");
}

/// A Gaia-Sausage-like meal (design 7.5): a gas-rich galaxy a fifth of the
/// host's mass reaches the centre 10 Gyr ago. The disc there and then is
/// heated into a thick disc, and its metal-poor gas dilutes the host's, so
/// the stars born after start a second, lower-metal family.
#[test]
fn a_gaia_sausage_like_meal_gives_a_thick_disc_and_two_families() {
    let w = world();
    let hs = haloes(&w, "gse");
    let h = hs.median(1.0e12, "gse");
    let gp = GalaxyParams::default();
    let rp = ResponseParams::default();
    let t = 3.8;
    let merge = crate::feed::Arrival::Merge {
        t_gyr: t,
        from: "gse".into(),
        bound_msun: 1.0e11,
        stars: 7.0e8,
        z_stars: 1.0e-3,
        gas: 3.0e9,
        z_gas: 1.0e-3,
        black_hole: 1.0e5,
        prograde: -0.3,
    };
    let quiet = crate::galaxy::evolve_feeding(&w.c, &gp, &h, &[], Some((&rp, &[])));
    let meal = crate::galaxy::evolve_feeding(&w.c, &gp, &h, &[], Some((&rp, &[merge])));
    let (aq, am) = (
        quiet.assembled.clone().unwrap(),
        meal.assembled.clone().unwrap(),
    );
    let thin = |g: &crate::galaxy::Galaxy| g.now.stars - g.now.bulge;
    assert!(aq.thick_disc_msun == 0.0);
    let thick = am.thick_disc_msun / thin(&meal);
    assert!((0.05..0.6).contains(&thick), "{thick}");
    // The gas metals drop at the meal, then climb again (within about a
    // tenth of a Gyr here, since the meal's gas arrives at once).
    let z_at = |g: &crate::galaxy::Galaxy, t: f64| crate::feed::state_at(g, t).z_gas;
    let before = z_at(&meal, t - 0.05);
    let after = z_at(&meal, t + 0.05);
    assert!(after < 0.8 * before, "{before} {after}");
    assert!(z_at(&quiet, t + 0.05) > z_at(&quiet, t - 0.05));
    assert!(z_at(&meal, t + 0.5) > before);
    let kinds: Vec<&str> = am.marks.iter().map(|m| m.kind.as_str()).collect();
    for k in ["thick_disc", "tilt", "lopsided", "burst_knots", "fresh_gas"] {
        assert!(kinds.contains(&k), "{k}: {kinds:?}");
    }
    // Marks fade by their own times; the thick disc lasts.
    for m in &am.marks {
        assert_eq!(m.strength_at(t - 0.01), 0.0);
        match m.fades_gyr {
            Some(f) => assert!(m.strength_at(t + 5.0 * f) < 0.01 * m.strength),
            None => assert_eq!(m.strength_at(13.8), m.strength),
        }
    }
}

/// With no arrivals and the stand-ins off, the feed path changes nothing
/// but what the stand-ins added; and the same inputs give the same galaxy.
#[test]
fn feeding_is_deterministic_and_closes_its_budget() {
    let w = world();
    let a = fed(&w, "same", 1.0e12, "s");
    let b = fed(&w, "same", 1.0e12, "s");
    assert_eq!(a.galaxy, b.galaxy);
    let gp = GalaxyParams::default();
    let hs = haloes(&w, "same");
    let h = hs.median(1.0e12, "s");
    let unfed = crate::galaxy::evolve(&w.c, &gp, &h);
    // Fed or not, the host ends with about the same stars: what arrives
    // was taken back from its smooth inflow.
    let r = a.galaxy.now.stars / unfed.now.stars;
    assert!((0.6..1.4).contains(&r), "{r}");
}

/// Leading arms are rare (design 9.4): Iye, Tadaki and Fukumoto (2019,
/// arXiv:1910.10926) found all 146 spirals with known spin and tilt
/// trailing. Over drawn disc hosts, at most `seen_share_max` show one now.
#[test]
fn leading_arms_are_rare() {
    let w = world();
    let la = ResponseParams::default().leading_arm;
    let (mut spirals, mut seen, mut ever) = (0u32, 0u32, 0u32);
    for mass in [3.0e11, 1.0e12] {
        for i in 0..12 {
            let f = fed(&w, "lead", mass, &format!("l{i}"));
            use crate::galaxy::Morphology::{Barred, Disc};
            if !matches!(f.galaxy.morphology, Disc | Barred) {
                continue;
            }
            spirals += 1;
            let now = f.galaxy.now.t_gyr;
            let a = f.galaxy.assembled.unwrap();
            let arms = a.marks.iter().filter(|m| m.kind == "leading_arm");
            ever += u32::from(arms.clone().any(|m| m.strength >= la.seen_strength));
            seen += u32::from(arms.clone().any(|m| m.strength_at(now) >= la.seen_strength));
        }
    }
    assert!(spirals >= 20, "{spirals}");
    // The early, crowded universe makes them; now they have wound back.
    assert!(ever > 0, "no leading arm in any history");
    assert!(
        f64::from(seen) <= la.seen_share_max * f64::from(spirals),
        "{seen} of {spirals} spirals lead now"
    );
}

/// An NGC 4622-like pass (Byrd, Freeman and Howard 1993): a companion a
/// hundredth of the host's mass, backwards, within a tenth of the disc's
/// radius, in a heavy halo, leaves a leading arm still seen after a Gyr;
/// the same pass forwards, or into a disc as heavy as its halo, leaves none.
#[test]
fn a_small_backwards_pass_leaves_a_leading_arm() {
    let w = world();
    let hs = haloes(&w, "ngc4622");
    let h = hs.median(1.0e12, "ngc4622");
    let gp = GalaxyParams::default();
    let rp = ResponseParams::default();
    let la = &rp.leading_arm;
    let t = 12.0;
    let pass = |prograde: f64, disc_msun: f64| crate::feed::Arrival::Pass {
        from: "companion".into(),
        pass: crate::infall::Pass {
            t_gyr: t,
            disc: false,
            r_kpc: 1.0,
            big_r_kpc: 1.0,
            speed_kms: 400.0,
            vz_kms: 0.0,
            prograde,
            bound_msun: 1.0e9,
            v_c_kms: 200.0,
            host_inside_msun: 1.0e11,
            disc_msun,
            disc_kpc: 3.0,
        },
    };
    let arm = |a: crate::feed::Arrival| {
        let g = crate::galaxy::evolve_feeding(&w.c, &gp, &h, &[], Some((&rp, &[a])));
        g.assembled
            .unwrap()
            .marks
            .into_iter()
            .find(|m| m.kind == "leading_arm")
    };
    let m = arm(pass(-1.0, 1.0e10)).expect("a leading arm");
    assert!(m.strength >= la.seen_strength, "{}", m.strength);
    assert!(m.strength_at(t + 1.0) >= 0.3 * m.strength, "{m:?}");
    assert!(m.strength_at(t + 4.0) < la.seen_strength, "{m:?}");
    assert!(arm(pass(1.0, 1.0e10)).is_none());
    assert!(arm(pass(-1.0, 5.0e10)).is_none());
}
