//! Nucleus checks: black holes against published relations, quasars at
//! cosmic noon, dormant holes today, unified-model views, mergers and
//! tidal disruptions.

use crate::g3_tests::world;
use crate::galaxy::{self as gx, GalaxyParams};
use crate::haloes::{HaloParams, Haloes};
use crate::nucleus::*;
use sse_core::math as m;
use sse_core::{Address, Seed};

fn nuclei(lms: &[f64]) -> Vec<(f64, crate::haloes::Halo, gx::Galaxy, Nucleus)> {
    let (c, w) = world();
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: HaloParams::default(),
        seed: Seed::from_text("g3"),
        universe: Address::universe("g3").unwrap(),
    };
    let gp = GalaxyParams::default();
    let np = NucleusParams::default();
    lms.iter()
        .map(|&lm| {
            let h = hs.median(
                m::pow(10.0, lm),
                &format!("m{}", m::round(lm * 100.0) as i64),
            );
            let g = gx::evolve(&c, &gp, &h);
            let n = nucleus(&c, &gp, &np, &h, &g, &[], Seed::from_text("g3"));
            (lm, h, g, n)
        })
        .collect()
}

#[test]
fn black_holes_quasars_and_quiet_nuclei() {
    let np = NucleusParams::default();
    let all = nuclei(&[11.0, 11.5, 12.0, 12.5, 13.0, 13.5, 14.0]);
    for (lm, _, g, n) in &all {
        let now = &n.now;
        let peak = n.history.iter().map(|p| p.luminosity_w).fold(0.0, f64::max);
        let quasar_t: Vec<f64> = n
            .history
            .iter()
            .filter(|p| p.class == NucleusClass::Quasar)
            .map(|p| p.t_gyr)
            .collect();
        let ep_q = n
            .episodes
            .iter()
            .filter(|e| e.class == NucleusClass::Quasar)
            .count();
        eprintln!(
            "1e{lm}: hole {:.2e} (bulge {:.2e}), now {:?} lambda {:.1e} L {:.1e} W, peak {:.1e} W, quasar points {} (from {:?}), quasar episodes {}, {} mergers, seed {}, TDE {:.1e}/yr, radio {}",
            now.mass_msun, g.now.bulge, now.class, now.eddington_ratio, now.luminosity_w, peak,
            quasar_t.len(), quasar_t.first(), ep_q, n.mergers.len(), n.seed.channel, n.tde_rate_per_yr, n.radio_loud
        );
        // Mass is the reservoir's; the light never exceeds Eddington in steady feeding.
        assert!(now.mass_msun > 0.0 && now.eddington_ratio <= 1.0 + 1e-9);
        // Today's nuclei are quiet: no quasars at z = 0 in median haloes.
        assert_ne!(now.class, NucleusClass::Quasar, "1e{lm}");
        assert!(n.history.windows(2).all(|w| w[1].t_gyr > w[0].t_gyr));
        assert!(n
            .history
            .iter()
            .all(|p| p.luminosity_w.is_finite() && p.luminosity_w >= 0.0));
        // Big galaxies' holes follow their bulges (Kormendy and Ho 2013: 0.1 to 0.5%),
        // through the holes merging galaxies bring.
        if *lm >= 13.0 {
            let r = now.mass_msun / g.now.bulge;
            assert!((5.0e-4..1.0e-2).contains(&r), "1e{lm}: hole/bulge {r:.1e}");
        }
        // Above the Hills mass, Sun-like stars are swallowed whole.
        if now.mass_msun > np.hills_mass_msun {
            assert_eq!(n.tde_rate_per_yr, 0.0);
        }
    }
    // Big haloes had quasars, and their quasar era is the young universe (cosmic noon or earlier).
    let big = &all.iter().find(|r| r.0 >= 13.0).unwrap().3;
    let q: Vec<f64> = big
        .episodes
        .iter()
        .filter(|e| e.class == NucleusClass::Quasar)
        .map(|e| e.start_gyr)
        .chain(
            big.history
                .iter()
                .filter(|p| p.class == NucleusClass::Quasar)
                .map(|p| p.t_gyr),
        )
        .collect();
    assert!(!q.is_empty(), "a group-scale halo had a quasar phase");
    assert!(q.iter().any(|t| *t < 6.0), "{q:?}");
    // Milky-Way-scale holes are millions of suns (Sgr A* 4.3e6) and a TDE every 1e4 to 1e5 years.
    let mw = &all.iter().find(|r| (r.0 - 12.0).abs() < 0.01).unwrap().3;
    assert!(
        (1.0e6..1.0e8).contains(&mw.now.mass_msun),
        "{}",
        mw.now.mass_msun
    );
    assert!(
        (1.0e-5..1.0e-3).contains(&mw.tde_rate_per_yr),
        "{}",
        mw.tde_rate_per_yr
    );
}

#[test]
fn light_views_and_events() {
    let np = NucleusParams::default();
    // Eddington: a 1e8 hole at the Eddington rate shines at 1.26e39 W (1.26e46 erg/s).
    let mdot_edd = L_EDD_PER_MSUN * 1.0e8
        / (np.radiative_efficiency * 299_792_458.0_f64 * 299_792_458.0)
        / 1.988_47e30
        * 31_557_600.0;
    let (l, lam, t) = np.light(1.0e8, mdot_edd);
    assert!((lam - 1.0).abs() < 1e-9 && (l - 1.257e39).abs() < 1e33);
    assert_eq!(np.class(l, lam), NucleusClass::Quasar);
    // Disc peaks in the far ultraviolet for 1e8 Msun (about 1e5 K), hotter for smaller holes.
    assert!((5.0e4..5.0e5).contains(&t), "{t}");
    let (_, _, t6) = np.light(1.0e6, mdot_edd / 100.0);
    assert!(t6 > t);
    // Radiatively inefficient below 1%: luminosity falls as the rate squared.
    let (l1, _, _) = np.light(1.0e8, mdot_edd * 1.0e-3);
    let (l2, _, _) = np.light(1.0e8, mdot_edd * 2.0e-3);
    assert!((l2 / l1 - 4.0).abs() < 1e-9);
    // Salpeter time: about 50 Myr at 10%.
    assert!((np.salpeter_gyr() - 0.05).abs() < 0.001);

    let all = nuclei(&[13.0]);
    let (_, _, _, n) = &all[0];
    let active = n
        .history
        .iter()
        .find(|p| matches!(p.class, NucleusClass::Quasar | NucleusClass::Seyfert))
        .expect("an active point");
    // Unified model: along the axis, type 1 (or a blazar if radio loud); edge on, type 2.
    let a = n.jet_axis;
    let along = seen_from(n, &np, active, a);
    assert!(along == "type_1" || along == "blazar", "{along}");
    let side = if a[2].abs() < 0.9 {
        [a[1], -a[0], 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    assert_eq!(seen_from(n, &np, active, side), "type_2");
    assert!((m::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2]) - 1.0).abs() < 1e-12);
    // Mergers carry chirp masses below both holes' total.
    for h in &n.mergers {
        assert!(h.chirp_msun < h.primary_msun + h.secondary_msun);
    }
    // Tidal disruptions: windows agree with their union; deterministic.
    let mw = &nuclei(&[12.0])[0].3;
    let s = Seed::from_text("g3");
    let whole = tidal_disruptions(mw, &np, s, 13.0, 13.2);
    let mut parts = tidal_disruptions(mw, &np, s, 13.0, 13.1);
    parts.extend(tidal_disruptions(mw, &np, s, 13.1, 13.2));
    assert_eq!(whole, parts);
    let expected = mw.tde_rate_per_yr * 2.0e8;
    eprintln!(
        "MW-like TDEs in 200 Myr: {} (expected about {expected:.0})",
        whole.len()
    );
    assert!((whole.len() as f64 - expected).abs() < 5.0 * m::sqrt(expected) + 3.0);
    // Facts are body facts.
    let f = facts(mw, &mw.now);
    assert!(f.contains_key("body.mass") && f.contains_key("body.radius"));
    assert_eq!(mw.address, format!("{}/nucleus:0", mw.halo));
    // Same inputs, same nucleus.
    assert_eq!(nuclei(&[12.0])[0].3, *mw);
}

#[test]
fn pack_holds_the_defaults_and_golden() {
    let cp = crate::scenario::CosmologyPack::from_packs(&[crate::scenario::base_cosmology_pack()])
        .unwrap();
    assert_eq!(cp.nucleus, NucleusParams::default());
    crate::golden::check("nucleus.json", &crate::scenario::nucleus_golden());
}
