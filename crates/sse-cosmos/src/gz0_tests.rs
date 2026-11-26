//! GZ0 (grand zoom, design section 6.2 point 4): how far a halo's place in
//! the cell catalogue (`h.position`, read by `halo_cell`, `galaxies_seen`
//! and the sky) is from its place in the web cube (`Web::displaced` at the
//! cube's time). A check with no feature: it measures, so GZ15 knows what it
//! settles, and fails only if the gap is not what the design expects.

use crate::cosmology::Cosmology;
use crate::haloes::Haloes;
use crate::scenario::{base_cosmology_pack, CosmologyPack};
use crate::web::{Cache, Web, WebParams};
use sse_core::math as m;
use sse_core::{Address, Seed};

#[test]
fn a_galaxy_sits_a_few_mpc_apart_in_the_cube_and_the_catalogue() {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap();
    let c = Cosmology::new(cp.params.clone());
    // As the kernel reads it (sse-kernel cosmos.rs `web`).
    let w = Web::new(
        &c,
        WebParams {
            octaves: cp.web.octaves.min(4),
            ..cp.web.clone()
        },
        &Seed::from_text("gz0"),
        &Address::universe("gz0").unwrap(),
    )
    .unwrap();
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: Seed::from_text("gz0"),
        universe: Address::universe("gz0").unwrap(),
    };
    let d = c.growth(1.0);
    let t = w.truncation(d).min(w.params.octaves - 1);
    let bins = hs.bin_table();
    let mut cache = Cache::default();
    let mut jumps = vec![];
    for iz in -2..=2 {
        for iy in -2..=2 {
            for ix in -2..=2 {
                for h in hs.cell_above([ix, iy, iz], &bins, 1.0e11) {
                    let p = w.displaced(&mut cache, h.position, t, d);
                    let s: f64 = (0..3)
                        .map(|k| (p[k] - h.position[k]) * (p[k] - h.position[k]))
                        .sum();
                    jumps.push(m::sqrt(s));
                }
            }
        }
    }
    jumps.sort_by(f64::total_cmp);
    let q = |f: f64| jumps[((jumps.len() - 1) as f64 * f) as usize];
    println!(
        "GZ0 jump over {} haloes above 1e11 Msun: median {:.2} Mpc, 90% {:.2}, max {:.2} (cell {} Mpc)",
        jumps.len(),
        q(0.5),
        q(0.9),
        q(1.0),
        cp.haloes.cell_mpc
    );
    assert!(jumps.len() > 50, "{}", jumps.len());
    // The design expects "several Mpc" at most: linear displacements today
    // are of order 10 Mpc/h in the largest flows.
    assert!(q(0.5) > 0.1 && q(1.0) < 40.0, "{:?}", (q(0.5), q(1.0)));
}
