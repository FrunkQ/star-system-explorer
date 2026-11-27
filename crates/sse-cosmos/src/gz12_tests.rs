//! GZ12: home is chosen from the web's own catalogue.

use crate::cosmology::Cosmology;
use crate::haloes::Haloes;
use crate::home::{cell_of, choose, fits, halo_at, shell, HomeWish};
use crate::scenario::{base_cosmology_pack, CosmologyPack};
use crate::web::{Web, WebParams};
use sse_core::{Address, Seed};

fn with_haloes<R>(seed: &str, f: impl FnOnce(&Haloes, &CosmologyPack, &Cosmology) -> R) -> R {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap();
    let c = Cosmology::new(cp.params.clone());
    // As the kernel reads it (sse-kernel cosmos.rs `web`).
    let w = Web::new(
        &c,
        WebParams {
            octaves: cp.web.octaves.min(4),
            ..cp.web.clone()
        },
        &Seed::from_text(seed),
        &Address::universe(seed).unwrap(),
    )
    .unwrap();
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: Seed::from_text(seed),
        universe: Address::universe(seed).unwrap(),
    };
    f(&hs, &cp, &c)
}

#[test]
fn shells_hold_the_right_cells_nearest_first() {
    assert_eq!(shell(0), vec![[0, 0, 0]]);
    assert_eq!(shell(1).len(), 26);
    assert_eq!(shell(2).len(), 98);
    assert_eq!(shell(1)[0], [-1, 0, 0]);
    assert_eq!(*shell(1).last().unwrap(), [1, 1, 1]);
    assert_eq!(cell_of("u:a/cell:-1.0.12/halo:3"), Some([-1, 0, 12]));
    assert_eq!(cell_of("u:a/halo:milky-way-like"), None);
}

/// The default wish finds a fitting home in every one of a handful of
/// universes, inside the cells searched; the home is catalogued (it is one
/// of its cell's haloes, read back by address) and meets every wish.
#[test]
fn every_universe_gets_a_catalogued_home_that_fits() {
    for seed in ["g3", "gz12", "sse4", "alpha", "beta", "gamma"] {
        with_haloes(seed, |hs, cp, c| {
            let h = choose(hs, &cp.galaxy, &cp.home, c.age_gyr()).unwrap();
            println!(
                "GZ12 {seed}: {} in cell {:?}, {:.2e} Msun, fits {}, {} cells read: {:?}",
                h.halo.address, h.cell, h.halo.mass, h.fits_wish, h.cells_read, h.why
            );
            assert!(h.fits_wish, "{seed}: no fitting home in the searched cells");
            let (n, _) = halo_at(hs, &h.halo.neighbour.as_ref().unwrap().address).unwrap();
            assert!(n.mass <= 3.0 * h.halo.mass, "{} {}", n.mass, h.halo.mass);
            assert!(fits(
                &h.halo,
                &cp.home,
                c.age_gyr(),
                hs.params.major_ratio,
                &|x| crate::home::disc_kpc_of(hs, &cp.galaxy, x),
                Some(n.mass)
            )
            .is_some());
            let (back, idx) = halo_at(hs, &h.halo.address).unwrap();
            assert_eq!(back, h.halo);
            assert_eq!(idx, h.cell);
            // Never the median stand-in.
            assert!(!h.halo.address.ends_with("/halo:milky-way-like"));
        });
    }
}

/// Pure: the same seed gives the same home, and a wish nobody can meet is
/// answered honestly (nearest in mass, marked), never refused.
#[test]
fn the_choice_is_repeatable_and_honest_when_nothing_fits() {
    with_haloes("gz12", |hs, cp, c| {
        let a = choose(hs, &cp.galaxy, &cp.home, c.age_gyr()).unwrap();
        let b = choose(hs, &cp.galaxy, &cp.home, c.age_gyr()).unwrap();
        assert_eq!(a, b);
        let never = HomeWish {
            quiet_gyr: 100.0,
            search_shells: 0,
            ..HomeWish::default()
        };
        let n = choose(hs, &cp.galaxy, &never, c.age_gyr()).unwrap();
        assert!(!n.fits_wish);
        assert_eq!(n.cells_read, 1);
        assert_eq!(n.cell, [0, 0, 0]);
        assert!(
            n.halo.mass >= 5.0e11 && n.halo.mass <= 2.0e12,
            "{}",
            n.halo.mass
        );
        let bad = HomeWish {
            neighbour: "friendly".into(),
            ..HomeWish::default()
        };
        assert!(choose(hs, &cp.galaxy, &bad, c.age_gyr()).is_err());
        assert!(halo_at(hs, "u:other/cell:0.0.0/halo:0").is_err());
        assert!(halo_at(hs, "u:gz12/cell:0.0.0/halo:99999").is_err());
    });
}

/// Found by GZ12: an early-forming catalogued host (g3's home) has
/// progenitors that fell in at z above 12, whose "mass today" came out NaN
/// and wiped the host's fed history (no stars at all). Every progenitor now
/// gets a finite mass that reproduces its infall mass, and the host's
/// history stays finite and holds a galaxy's worth of stars.
#[test]
fn an_early_forming_home_grows_a_whole_galaxy() {
    with_haloes("g3", |hs, cp, c| {
        let home = choose(hs, &cp.galaxy, &cp.home, c.age_gyr()).unwrap().halo;
        let fed = crate::feed::evolve_fed(
            c,
            hs,
            &cp.galaxy,
            &cp.sky.satellites.orbits,
            &cp.sky.satellites.responses,
            &home,
            &[],
        )
        .unwrap();
        let mut short = 0;
        for mb in &fed.members {
            let p = &mb.progenitor;
            let m0 = crate::assembly::mass_today_for(c, p.mass_at_infall_msun, p.z_infall);
            assert!(
                m0.is_finite() && m0 >= p.mass_at_infall_msun,
                "{} {m0}",
                p.address
            );
            // Never more than it had; exactly it wherever a median halo
            // can grow that big that early (else the nearest such halo).
            let back = mb.halo.mass_at(p.z_infall);
            assert!(back <= p.mass_at_infall_msun * 1.001, "{}", p.address);
            if back < p.mass_at_infall_msun * 0.999 {
                short += 1;
            }
        }
        println!(
            "GZ12 g3 home: {short} of {} progenitors fell in bigger than any median halo then",
            fed.members.len()
        );
        assert!(short * 10 < fed.members.len(), "{short}");
        for st in &fed.galaxy.history {
            assert!(st.stars.is_finite() && st.gas.is_finite(), "t {}", st.t_gyr);
        }
        let now = &fed.galaxy.now;
        // A halo near 1e12 suns holds some 1e10 to 1e11 stars (abundance matching).
        assert!(now.stars > 1.0e10 && now.stars < 1.0e11, "{}", now.stars);
    });
}
