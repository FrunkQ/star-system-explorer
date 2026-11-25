//! V9w light from haloes: the web cube's galaxies.

use crate::cosmology::Cosmology;
use crate::haloes::Haloes;
use crate::scenario::{base_cosmology_pack, CosmologyPack};
use crate::web::{Environment, Web, WebParams};
use crate::web_light::{cube_galaxies, MedianGalaxies};
use sse_core::{Address, Seed};

fn world() -> (Cosmology, CosmologyPack, Web) {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap();
    let c = Cosmology::new(cp.params.clone());
    // As the kernel reads it (sse-kernel cosmos.rs `web`).
    let w = Web::new(
        &c,
        WebParams {
            octaves: cp.web.octaves.min(4),
            ..cp.web.clone()
        },
        &Seed::from_text("web-light"),
        &Address::universe("web-light").unwrap(),
    )
    .unwrap();
    (c, cp, w)
}

fn haloes<'a>(c: &'a Cosmology, cp: &CosmologyPack, w: &'a Web) -> Haloes<'a> {
    Haloes {
        cosmo: c,
        web: w,
        params: cp.haloes.clone(),
        seed: Seed::from_text("web-light"),
        universe: Address::universe("web-light").unwrap(),
    }
}

#[test]
fn the_haloes_above_a_floor_are_the_cells_own() {
    let (c, cp, w) = world();
    let hs = haloes(&c, &cp, &w);
    let bins = hs.bin_table();
    for idx in [[0, 0, 0], [3, -2, 1], [-5, 4, 7]] {
        let full = hs.cell(idx);
        for floor in [cp.haloes.min_mass, 1.0e11, 1.0e12] {
            let light = hs.cell_above(idx, &bins, floor);
            let want: Vec<_> = full.haloes.iter().filter(|h| h.mass >= floor).collect();
            assert_eq!(light.len(), want.len(), "cell {idx:?} floor {floor:e}");
            for (a, b) in light.iter().zip(want) {
                // Bit for bit: mass, place, growth and spin; only the mergers and neighbour are left out.
                assert_eq!(a.address, b.address);
                assert_eq!(a.mass.to_bits(), b.mass.to_bits());
                assert_eq!(a.position.map(f64::to_bits), b.position.map(f64::to_bits));
                assert_eq!(a.alpha.to_bits(), b.alpha.to_bits());
                assert_eq!(a.beta.to_bits(), b.beta.to_bits());
                assert_eq!(a.spin.to_bits(), b.spin.to_bits());
                assert!(a.mergers.is_empty() && a.neighbour.is_none());
            }
        }
    }
}

#[test]
fn galaxies_light_the_web_not_the_voids() {
    let (c, cp, w) = world();
    let hs = haloes(&c, &cp, &w);
    let median = MedianGalaxies::new(&hs, &cp.galaxy, cp.web_light.floor_msun, cp.haloes.max_mass);
    let (centre, extent, n) = ([0.0, 0.0, 0.0], 100.0, 32);
    let upto = w.params.octaves - 1;
    let gals = cube_galaxies(&hs, &median, &cp.web_light, 1.0, centre, extent, upto);
    println!("{} galaxies", gals.len());
    // About 0.01 haloes above 1e11 per cubic Mpc today: thousands in a 100 Mpc cube.
    assert!(
        gals.len() > 2_000 && gals.len() < 40_000,
        "{} galaxies",
        gals.len()
    );
    assert!(gals.iter().all(|g| g.halo_msun >= cp.web_light.floor_msun));
    assert!(
        gals.windows(2).all(|p| p[0].halo_msun >= p[1].halo_msun),
        "biggest first"
    );
    // Starlight: a Milky Way halo holds a few percent of its mass in stars; the biggest have quenched.
    let mw: Vec<_> = gals
        .iter()
        .filter(|g| (8.0e11..1.5e12).contains(&g.halo_msun))
        .collect();
    assert!(!mw.is_empty());
    for g in &mw {
        let r = g.stars_msun / g.halo_msun;
        assert!((0.005..0.08).contains(&r), "M*/Mh {r}");
    }
    let big = &gals[0];
    assert!(big.halo_msun > 1.0e13 && big.quenched, "{big:?}");
    // Where they sit: the environment of the nearest parcel of the same cube.
    let v = w.volume(&c, 1.0, centre, extent, n, upto);
    let mut in_void = 0usize;
    let mut stars_void = 0.0;
    let mut stars_all = 0.0;
    for g in &gals {
        let mut best = (f64::INFINITY, Environment::Void);
        for (i, e) in v.environment.iter().enumerate() {
            let d: f64 = (0..3)
                .map(|k| {
                    let q = v.position[3 * i + k] - g.position[k];
                    q * q
                })
                .sum();
            if d < best.0 {
                best = (d, *e);
            }
        }
        stars_all += g.stars_msun;
        if best.1 == Environment::Void {
            in_void += 1;
            stars_void += g.stars_msun;
        }
    }
    // Void galaxies form later, so they are younger: more star formation for their stars than the rest.
    let ssfr = |void: bool| {
        let (mut sf, mut st) = (0.0, 0.0);
        for g in &gals {
            let mut best = (f64::INFINITY, Environment::Void);
            for (i, e) in v.environment.iter().enumerate() {
                let d: f64 = (0..3)
                    .map(|k| {
                        let q = v.position[3 * i + k] - g.position[k];
                        q * q
                    })
                    .sum();
                if d < best.0 {
                    best = (d, *e);
                }
            }
            if (best.1 == Environment::Void) == void && g.halo_msun < 3.0e12 {
                sf += g.sfr_msun_yr;
                st += g.stars_msun;
            }
        }
        sf / st
    };
    let (sv, sw) = (ssfr(true), ssfr(false));
    println!("specific star formation: voids {sv:.3e} per year, the web {sw:.3e}");
    assert!(sv > sw, "void galaxies are younger ({sv} against {sw})");
    let share = in_void as f64 / gals.len() as f64;
    let light = stars_void / stars_all;
    println!("galaxies in voids {share:.2}, their share of the stars {light:.2}");
    // Voids fill 77% of space; their galaxies are far fewer and smaller.
    assert!(share < 0.5, "galaxies in voids {share}");
    assert!(light < share, "void galaxies are smaller");
}

#[test]
fn the_young_web_has_fewer_and_smaller_galaxies() {
    let (c, cp, w) = world();
    let hs = haloes(&c, &cp, &w);
    let median = MedianGalaxies::new(&hs, &cp.galaxy, cp.web_light.floor_msun, cp.haloes.max_mass);
    let upto = w.params.octaves - 1;
    let now = cube_galaxies(&hs, &median, &cp.web_light, 1.0, [0.0; 3], 60.0, upto);
    let then = cube_galaxies(&hs, &median, &cp.web_light, 0.25, [0.0; 3], 60.0, upto);
    let stars = |g: &[crate::web_light::WebGalaxy]| g.iter().map(|g| g.stars_msun).sum::<f64>();
    println!("now {} galaxies, z = 3 {}", now.len(), then.len());
    assert!(then.len() < now.len() / 2);
    assert!(stars(&then) < stars(&now) / 3.0);
    assert!(then.iter().all(|g| !g.quenched || g.halo_msun > 1.0e12));
}
