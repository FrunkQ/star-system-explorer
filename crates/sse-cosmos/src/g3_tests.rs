//! G3 checks: galaxies against published relations; rings and what died here.
//!
//! Tolerances are factors, not percentages: the observed relations themselves
//! differ by these factors between calibrations (Kewley and Ellison 2008 for
//! metallicity; Speagle et al. 2014 for the main sequence), and a regulator
//! model on median haloes is meant to land in the right band, not to fit.

use crate::cosmology::{Cosmology, Params};
use crate::galaxy::{self as gx, Galaxy, GalaxyParams};
use crate::haloes::{dn_dlnm, HaloParams, Haloes};
use crate::web::{Web, WebParams};
use sse_core::math as m;
use sse_core::{Address, Seed};

pub(crate) fn world() -> (Cosmology, Web) {
    let c = Cosmology::new(Params::default());
    let w = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..Default::default()
        },
        &Seed::from_text("g3"),
        &Address::universe("g3").unwrap(),
    )
    .unwrap();
    (c, w)
}

/// Median-halo galaxies on a grid of present-day halo masses (log10 from, to, step).
pub(crate) fn grid(c: &Cosmology, w: &Web, from: f64, to: f64, step: f64) -> Vec<(f64, Galaxy)> {
    let hs = Haloes {
        cosmo: c,
        web: w,
        params: HaloParams::default(),
        seed: Seed::from_text("g3"),
        universe: Address::universe("g3").unwrap(),
    };
    let p = GalaxyParams::default();
    let mut out = vec![];
    let mut lm = from;
    while lm < to + 1.0e-9 {
        let h = hs.median(
            m::pow(10.0, lm),
            &format!("m{}", m::round(lm * 100.0) as i64),
        );
        out.push((lm, gx::evolve(c, &p, &h)));
        lm += step;
    }
    out
}

fn within(x: f64, want: f64, factor: f64) -> bool {
    x > want / factor && x < want * factor
}

#[test]
fn stellar_to_halo_mass_follows_behroozi() {
    // Behroozi et al. 2019 (UniverseMachine), z = 0 median M*/M_h, read off their figure 9.
    let want = [
        (10.5, 0.002),
        (11.0, 0.005),
        (11.5, 0.015),
        (12.0, 0.03),
        (12.5, 0.025),
        (13.0, 0.016),
        (14.0, 0.004),
    ];
    let (c, w) = world();
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: HaloParams::default(),
        seed: Seed::from_text("g3"),
        universe: Address::universe("g3").unwrap(),
    };
    let mut peak = (0.0, 0.0);
    for (lm, r) in want {
        let g = gx::evolve(
            &c,
            &GalaxyParams::default(),
            &hs.median(
                m::pow(10.0, lm),
                &format!("m{}", m::round(lm * 100.0) as i64),
            ),
        );
        let ratio = g.now.stars / g.now.halo;
        println!("M*/Mh 1e{lm}: {ratio:.4}");
        assert!(
            within(ratio, r, 2.5),
            "M*/Mh at 1e{lm}: {ratio:.4} against {r}"
        );
        if ratio > peak.1 {
            peak = (lm, ratio);
        }
    }
    // The efficiency peaks near 1e12 solar masses: feedback below, hot haloes above.
    assert!((11.5..=12.5).contains(&peak.0), "peak at 1e{}", peak.0);
}

#[test]
fn star_forming_main_sequence_and_quenching() {
    let (c, w) = world();
    let gals = grid(&c, &w, 10.5, 14.0, 0.25);
    let mut last_quenched = false;
    for (lm, g) in &gals {
        let n = &g.now;
        let lms = m::log10(n.stars);
        // Green-valley galaxies (sSFR 1e-11 to 10^-10.6 per year, Salim 2014) are leaving the sequence.
        let green = n.sfr / n.stars < m::pow(10.0, -10.6);
        if !n.quenched && !green && (9.7..=11.1).contains(&lms) {
            // Renzini and Peng 2015 (the local ridge line): log SFR = 0.76 log M* - 7.64.
            // Speagle et al. 2014's z = 0 line sits about a factor 2 lower; factor 3 spans both.
            let want = m::pow(10.0, 0.76 * lms - 7.64);
            assert!(
                within(n.sfr, want, 3.0),
                "SFR at M* 1e{lms:.2}: {:.3} against {want:.3}",
                n.sfr
            );
        }
        // Quenching sets in with mass and stays: no star-forming galaxy above a quenched one.
        if last_quenched {
            assert!(n.quenched, "halo 1e{lm} star-forming above a quenched one");
        }
        last_quenched = n.quenched;
        // Specific star formation rates are physical (Hubble time against a starburst).
        if !n.quenched {
            assert!(
                (1.0e-11..1.0e-8).contains(&(n.sfr / n.stars)),
                "sSFR at 1e{lm}"
            );
        }
    }
    let q12 = gals.iter().find(|g| g.0 >= 12.99).unwrap();
    let s11 = gals.iter().find(|g| g.0 >= 11.49).unwrap();
    assert!(q12.1.now.quenched, "1e13 haloes host quenched galaxies");
    assert!(
        !s11.1.now.quenched,
        "3e11 haloes host star-forming galaxies"
    );
}

#[test]
fn mass_metallicity_relation() {
    // Andrews and Martini 2013 (direct method): 12 + log O/H = 8.798 - log(1 + (10^8.901 / M*)^0.64);
    // solar 12 + log O/H = 8.69 and protosolar Z = 0.0142 (Asplund et al. 2009; present-day photosphere 0.0134).
    let (c, w) = world();
    let mut last = 0.0;
    for (lm, g) in grid(&c, &w, 10.5, 12.5, 0.5) {
        let n = &g.now;
        let oh = 8.798 - m::log10(1.0 + m::pow(m::pow(10.0, 8.901) / n.stars, 0.64));
        let want = 0.0142 * m::pow(10.0, oh - 8.69);
        if n.stars > 1.0e8 {
            // Calibrations differ by up to 0.7 dex at low mass (Kewley and Ellison 2008); the
            // direct method used here is the high one, so a factor 3.
            assert!(
                within(n.z_gas, want, 3.0),
                "Z at 1e{lm}: {:.4} against {want:.4}",
                n.z_gas
            );
        }
        if !n.quenched {
            assert!(n.z_gas > last, "metallicity rises with mass");
            last = n.z_gas;
        }
    }
}

/// Cosmic star formation rate density at z: SFR integrated over the halo mass
/// function at z, each galaxy at its own halo mass then (solar masses per year per Mpc^3).
fn sfrd(c: &Cosmology, gals: &[(f64, Galaxy)], z: f64) -> f64 {
    let pts: Vec<(f64, f64)> = gals
        .iter()
        .map(|(_, g)| {
            let i = g
                .history
                .partition_point(|s| s.z > z)
                .min(g.history.len() - 1);
            (m::ln(g.history[i].halo), g.history[i].sfr)
        })
        .collect();
    let f = |q: (f64, f64)| q.1 * dn_dlnm(c, m::exp(q.0), z);
    pts.windows(2)
        .map(|w| 0.5 * (f(w[0]) + f(w[1])) * (w[1].0 - w[0].0))
        .sum()
}

#[test]
fn cosmic_star_formation_history_follows_madau_dickinson() {
    let (c, w) = world();
    let gals = grid(&c, &w, 10.0, 15.0, 0.25);
    let mut peak = (0.0, 0.0);
    for z in [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0] {
        let got = sfrd(&c, &gals, z);
        // Madau and Dickinson 2014, equation 15, Salpeter, times 0.63 for a Chabrier IMF.
        let want = 0.63 * 0.015 * m::pow(1.0 + z, 2.7) / (1.0 + m::pow((1.0 + z) / 2.9, 5.6));
        println!("sfrd z {z}: {:.2}", got / want);
        assert!(
            within(got, want, 2.6),
            "SFRD at z {z}: {got:.4e} against {want:.4e}"
        );
        if got > peak.1 {
            peak = (z, got);
        }
    }
    // Cosmic noon: the universe formed stars fastest between z 1.5 and 4.
    assert!((1.5..=4.0).contains(&peak.0), "peak at z {}", peak.0);
}

#[test]
fn shapes_and_black_holes() {
    let (c, w) = world();
    for (lm, g) in grid(&c, &w, 11.0, 14.0, 1.0) {
        let n = &g.now;
        // Discs of a few kpc in Milky Way haloes (the Milky Way's is 2.6 kpc, Bland-Hawthorn and Gerhard 2016).
        if (lm - 12.0).abs() < 0.01 {
            assert!((1.5..6.0).contains(&n.disc_kpc), "disc {}", n.disc_kpc);
            assert!(
                matches!(g.morphology, gx::Morphology::Disc | gx::Morphology::Barred),
                "{:?}",
                g.morphology
            );
        }
        if lm > 13.5 {
            assert_eq!(g.morphology, gx::Morphology::Elliptical);
        }
        // Black holes stay below about a percent of their galaxy (Kormendy and Ho 2013 give 0.1 to 0.5% of bulges; holes track bulges, not whole galaxies).
        assert!(n.black_hole < 0.01 * n.stars, "black hole at 1e{lm}");
        // Gas, stars and history are finite and ordered in time.
        assert!(g.history.windows(2).all(|w| w[1].t_gyr > w[0].t_gyr));
        assert!(n.gas >= 0.0 && n.stars > 0.0 && n.bulge <= n.stars);
    }
}

#[test]
fn galaxies_are_deterministic() {
    let (c, w) = world();
    let a = grid(&c, &w, 12.0, 12.0, 1.0);
    let b = grid(&c, &w, 12.0, 12.0, 1.0);
    assert_eq!(a[0].1, b[0].1);
}

pub(crate) fn milky_way_like(c: &Cosmology, w: &Web) -> (Galaxy, crate::rings::Disc) {
    let g = grid(c, w, 12.0, 12.0, 1.0).remove(0).1;
    let d = crate::rings::evolve(
        &g,
        &GalaxyParams::default(),
        &crate::rings::RingParams::default(),
        Seed::from_text("g3"),
        &Address::universe("g3").unwrap(),
    );
    (g, d)
}

/// Milky Way kpc for a radius in this galaxy (scaled by disc length, R_d = 2.6 kpc).
fn mw_kpc(g: &Galaxy, r: f64) -> f64 {
    r * 2.6 / g.now.disc_kpc
}

#[test]
fn radial_metallicity_gradient() {
    use crate::rings::FE;
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    // Least-squares slope of present gas [Fe/H] against radius, 4 to 14 kpc (Milky Way scaled).
    let pts: Vec<(f64, f64)> = d
        .rings
        .iter()
        .map(|r| (mw_kpc(&g, r.radius_kpc), r.abundance(FE)))
        .filter(|p| (4.0..=14.0).contains(&p.0))
        .collect();
    let nf = pts.len() as f64;
    let (sx, sy) = (
        pts.iter().map(|p| p.0).sum::<f64>() / nf,
        pts.iter().map(|p| p.1).sum::<f64>() / nf,
    );
    let slope = pts.iter().map(|p| (p.0 - sx) * (p.1 - sy)).sum::<f64>()
        / pts.iter().map(|p| (p.0 - sx) * (p.0 - sx)).sum::<f64>();
    println!("gradient {slope:.4}");
    // Cepheids give -0.06 (Genovali et al. 2014), open clusters -0.05 (Netopil et al. 2016).
    assert!(
        (-0.08..=-0.03).contains(&slope),
        "gradient {slope:.3} dex per kpc"
    );
}

#[test]
fn solar_neighbourhood_history() {
    use crate::rings::*;
    let (c, w) = world();
    let (g, d) = milky_way_like(&c, &w);
    let si = d
        .rings
        .iter()
        .position(|r| mw_kpc(&g, r.outer_kpc) >= 8.2)
        .unwrap();
    let sun = &d.rings[si];
    // The Sun formed 4.57 Gyr ago from gas of [Fe/H] = 0 by definition; within 0.2 dex.
    let k = m::floor((c.p.now_gyr - 4.57 - d.t_start_gyr) / d.step_gyr) as usize;
    let at_birth = d.feh_history[si][k];
    println!("at birth {at_birth:.3}");
    assert!(
        at_birth.abs() < 0.2,
        "[Fe/H] at the Sun's birth {at_birth:.2}"
    );
    // No G-dwarf problem: under 5% of long-lived stars below [Fe/H] = -1 (observed about 1 to 2%).
    let tot: f64 = sun.formed.iter().sum();
    let poor: f64 = sun
        .formed
        .iter()
        .zip(&sun.feh)
        .filter(|(_, f)| **f < -1.0)
        .map(|(s, _)| s)
        .sum();
    println!("poor {:.4} plateau-check", poor / tot);
    assert!(poor / tot < 0.05, "metal-poor share {:.3}", poor / tot);
    // The [alpha/Fe] knee: a plateau of 0.25 to 0.5 for metal-poor stars, near 0 at solar metallicity.
    let plateau: Vec<f64> = sun
        .feh
        .iter()
        .zip(&sun.alpha_fe)
        .filter(|(f, _)| (-2.5..-1.5).contains(*f))
        .map(|(_, a)| *a)
        .collect();
    assert!(
        !plateau.is_empty() && plateau.iter().all(|a| (0.25..0.5).contains(a)),
        "plateau {plateau:?}"
    );
    let solar: Vec<f64> = sun
        .feh
        .iter()
        .zip(&sun.alpha_fe)
        .filter(|(f, _)| f.abs() < 0.1)
        .map(|(_, a)| *a)
        .collect();
    assert!(
        !solar.is_empty() && solar.iter().all(|a| a.abs() < 0.12),
        "solar [a/Fe] {solar:?}"
    );
    // [alpha/Fe] only falls as iron builds up.
    let ordered: Vec<(f64, f64)> = sun
        .feh
        .iter()
        .zip(&sun.alpha_fe)
        .filter(|(f, _)| **f > -3.0)
        .map(|(f, a)| (*f, *a))
        .collect();
    assert!(ordered
        .windows(2)
        .all(|w| w[1].0 >= w[0].0 - 1.0e-9 && w[1].1 <= w[0].1 + 1.0e-9));
    // Every channel has left a trace, and K-40 has decayed below what was made.
    assert!(
        sun.deaths.core_collapse > 1.0e6
            && sun.deaths.type_ia > 1.0e5
            && sun.deaths.neutron_star_mergers > 100
    );
    assert!(sun.abundance(CN) > -0.3 && sun.abundance(S) > -0.3 && sun.abundance(R) > -0.4);
}

#[test]
fn supernova_rates_today() {
    let (c, w) = world();
    let (_, d) = milky_way_like(&c, &w);
    let cc = d.core_collapse_rate() * 100.0;
    let ia = d.ia_rate() * 100.0;
    // Milky Way: core collapse 1.6 +- 0.5 per century (Rozwadowska et al. 2021), type Ia 0.54 +- 0.12 (Li et al. 2011).
    // This galaxy forms stars about 1.7 times as fast as the Milky Way, hence the wider core-collapse band.
    assert!(
        (0.8..4.5).contains(&cc),
        "core collapse {cc:.2} per century"
    );
    assert!((0.27..1.1).contains(&ia), "type Ia {ia:.2} per century");
}

#[test]
fn r_process_is_lumpy() {
    use crate::rings::R;
    // A dwarf's rings see few neutron-star mergers, so neighbouring rings differ by a
    // lot in r-process (story material), while iron is smooth.
    let (c, w) = world();
    let g = grid(&c, &w, 10.5, 10.5, 1.0).remove(0).1;
    let d = crate::rings::evolve(
        &g,
        &GalaxyParams::default(),
        &crate::rings::RingParams::default(),
        Seed::from_text("g3"),
        &Address::universe("g3").unwrap(),
    );
    let spread = |grp: usize| {
        let v: Vec<f64> = d
            .rings
            .iter()
            .filter(|r| r.gas > 1.0e4)
            .map(|r| r.abundance(grp))
            .filter(|a| *a > -8.0)
            .collect();
        v.windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f64::max)
    };
    assert!(
        spread(R) > 2.0 * spread(crate::rings::FE),
        "r {:.2} fe {:.2}",
        spread(R),
        spread(crate::rings::FE)
    );
    // Determinism: the same draws every time.
    let again = crate::rings::evolve(
        &g,
        &GalaxyParams::default(),
        &crate::rings::RingParams::default(),
        Seed::from_text("g3"),
        &Address::universe("g3").unwrap(),
    );
    assert_eq!(d, again);
}
#[test]
fn pack_settings_are_the_defaults_and_golden_holds() {
    let cp = crate::scenario::CosmologyPack::from_packs(&[crate::scenario::base_cosmology_pack()])
        .unwrap();
    assert_eq!(cp.galaxy, GalaxyParams::default());
    assert_eq!(cp.rings, crate::rings::RingParams::default());
    crate::golden::check("galaxy.json", &crate::scenario::galaxy_golden());
}
