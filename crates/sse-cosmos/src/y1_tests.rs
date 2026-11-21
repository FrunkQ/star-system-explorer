//! Y1 checks: the count tree (night sky design section 4.2, table row Y1).

use crate::envelope::{EnvelopeParams, GalaxyEnvelope, GreenParams};
use crate::g3_tests::{milky_way_like, world};
use crate::star_tree::TreeWalk;
use crate::stars::*;
use sse_core::{Address, Seed};

macro_rules! setup {
    ($st:ident, $now:ident, $sun:ident) => {
        let (c, w) = world();
        let (g, d) = milky_way_like(&c, &w);
        let env = GalaxyEnvelope {
            galaxy: &g,
            disc: &d,
            params: EnvelopeParams::default(),
            seed: Seed::from_text("g4"),
            address: Address::universe("g4").unwrap(),
        };
        let $st = Stars {
            env: &env,
            green: GreenParams::default(),
            params: StarParams::default(),
            seed: Seed::from_text("g5"),
            galaxy: Address::universe("g5").unwrap(),
        };
        let $now = $st.frame_of(g.now.t_gyr * 1000.0);
        let $sun = [env.sun_radius_kpc() * 1000.0, 0.0, 0.0];
    };
}

/// Per-cell counts are Poisson about G5's envelope mean: chi-squared over
/// 4,000 cells, and the totals agree.
#[test]
fn cell_counts_are_poisson_about_the_envelope() {
    setup!(st, now, sun);
    let walk = TreeWalk::new(&st, now);
    let classes = st.params.class_edges_lsun.len() + 1;
    let c0 = st.cell_at(st.from_grid(sun, now as f64), now as f64);
    let (mut chi2, mut n_sum, mut mu_sum, mut cells) = (0.0, 0.0, 0.0, 0usize);
    // A slab through the plane, so the counts run from crowded to sparse.
    for dx in 0..20 {
        for dy in 0..10 {
            for dz in -10..10 {
                let cell = [c0[0] + dx, c0[1] + dy, c0[2] + dz];
                let n: u64 = (0..classes)
                    .map(|k| walk.cell_count(cell, k).unwrap())
                    .sum();
                let mu = st.expected(cell, now);
                chi2 += (n as f64 - mu) * (n as f64 - mu) / mu;
                n_sum += n as f64;
                mu_sum += mu;
                cells += 1;
            }
        }
    }
    let nf = cells as f64;
    let sd = sse_core::math::sqrt(2.0 * nf);
    println!(
        "{cells} cells: chi2 {chi2:.0} (expect {nf} ± {sd:.0}), counts {n_sum} against {mu_sum:.0}"
    );
    assert!(
        (chi2 - nf).abs() < 5.0 * sd,
        "chi2 {chi2} over {cells} cells"
    );
    assert!(
        (n_sum - mu_sum).abs() < 5.0 * sse_core::math::sqrt(mu_sum),
        "{n_sum} {mu_sum}"
    );
}

/// A bright star a sky walk finds 1 kpc away is the very star the cell draw
/// gives there, and of the class the walk asked for.
#[test]
fn a_bright_star_found_from_afar_is_the_cell_draws_star() {
    setup!(st, now, sun);
    let bright = st.params.class_edges_lsun.len() - 1;
    let walk = TreeWalk::new(&st, now);
    let found = walk.cells_within(bright, sun, 1000.0).unwrap();
    assert!(
        !found.is_empty(),
        "no stars brighter than 100 L☉ within 1 kpc"
    );
    let t0 = now as f64 * st.params.frame_myr;
    for (cell, n) in found.iter().take(5) {
        let only = TreeWalk::new(&st, now).draw(*cell, Some(bright)).unwrap();
        assert_eq!(only.len() as u64, *n);
        let all = st.cell(*cell, now, &[]).unwrap();
        for s in &only {
            assert!(all.contains(s), "the sky's star is the cell's star");
            let l = st.at(s, t0).state.luminosity_lsun;
            assert_eq!(st.params.class_of(l), bright, "L {l}");
        }
    }
}

/// The walk reads the envelope in proportion to the stars it finds, not to
/// the volume: doubling the radius multiplies the volume by eight, and the
/// reads per star found stay within the eight children read per level.
#[test]
fn a_sky_walk_costs_what_it_finds() {
    setup!(st, now, sun);
    let brightest = st.params.class_edges_lsun.len();
    let mut rows = vec![];
    for r in [250.0, 500.0] {
        let walk = TreeWalk::new(&st, now);
        let found: u64 = walk
            .cells_within(brightest, sun, r)
            .unwrap()
            .iter()
            .map(|x| x.1)
            .sum();
        let cells = 4.19 * r * r * r / 1000.0;
        rows.push((r, found, walk.reads(), cells));
    }
    println!("{rows:?}");
    for (_, found, reads, cells) in &rows {
        // Eight children read per level walked, per star, plus the roots.
        let bound = 8 * (st.params.tree_depth as usize + 1) * (*found as usize + 1) + 600;
        assert!(*reads <= bound, "{reads} reads for {found} stars");
        assert!(
            (*reads as f64) < *cells,
            "{reads} reads against {cells} cells"
        );
    }
    // Eight times the volume, about eight times the stars and the reads.
    let (a, b) = (&rows[0], &rows[1]);
    let per_star = |r: &(f64, u64, usize, f64)| r.2 as f64 / r.1.max(1) as f64;
    assert!(per_star(b) < 1.5 * per_star(a), "{rows:?}");
}

/// Cells drawn in any order, in one walk or many, give the same stars.
#[test]
fn the_tree_is_order_independent() {
    setup!(st, now, sun);
    let c0 = st.cell_at(st.from_grid(sun, now as f64), now as f64);
    let cells = [
        c0,
        [c0[0] + 70, c0[1] - 3, c0[2] + 1],
        [c0[0] - 64, c0[1], c0[2] - 5],
    ];
    let a = TreeWalk::new(&st, now);
    let one: Vec<_> = cells.iter().map(|c| a.draw(*c, None).unwrap()).collect();
    let b = TreeWalk::new(&st, now);
    let mut other: Vec<_> = cells
        .iter()
        .rev()
        .map(|c| b.draw(*c, None).unwrap())
        .collect();
    other.reverse();
    assert_eq!(one, other);
    for (c, s) in cells.iter().zip(&one) {
        let mut s = s.clone();
        s.sort_by(|a, b| a.address.cmp(&b.address));
        assert_eq!(st.cell(*c, now, &[]).unwrap(), s);
    }
}
