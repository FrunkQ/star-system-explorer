//! DM3: the matter ledger of a web cell (galaxy histories design 4.1).

use crate::galaxy::GalaxyParams;
use crate::haloes::{HaloParams, Haloes};
use crate::ledger::{cell, Ledger, LedgerParams, Reservoirs};
use crate::web::Environment;
use sse_core::{Address, Seed};

fn ledgers(n: i64) -> Vec<Ledger> {
    let (c, w) = crate::g3_tests::world();
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: HaloParams::default(),
        seed: Seed::from_text("dm3"),
        universe: Address::universe("dm3").unwrap(),
    };
    let lp = LedgerParams::default();
    let res = Reservoirs::new(&hs, &GalaxyParams::default(), lp.galaxy_low_msun, 1.0e15);
    let mut out = vec![];
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                out.push(cell(&hs, &res, &lp, [i * 3, j * 3, k * 3]));
            }
        }
    }
    out
}

/// The parts add up to the cell's matter, and ordinary matter over all
/// matter is the universe's baryon share, to rounding.
#[test]
fn the_ledger_closes_in_every_cell() {
    for l in ledgers(3) {
        let d = &l.dark;
        let dark = d.catalogued_haloes + d.haloes_below_floor + d.never_collapsed;
        let total = dark + l.ordinary.total();
        assert!((total / l.matter_msun - 1.0).abs() < 1e-9, "{l:?}");
        assert!((l.baryon_share / l.f_b - 1.0).abs() < 1e-9, "{l:?}");
        let o = &l.ordinary;
        for v in [
            o.stars,
            o.cold_gas,
            o.hot_gas,
            o.thrown_out,
            o.black_holes,
            o.between_galaxies,
        ] {
            assert!(v >= 0.0, "{l:?}");
        }
        let shares: f64 = l.environments.iter().map(|s| s.volume).sum();
        assert!((shares - 1.0).abs() < 1e-12);
    }
}

/// The census of ordinary matter today: stars are under a tenth, and most
/// is thin gas outside galaxies (Fukugita and Peebles 2004,
/// doi:10.1086/425155; Macquart et al. 2020, doi:10.1038/s41586-020-2300-2,
/// who found the missing part between galaxies with fast radio bursts).
#[test]
fn most_ordinary_matter_is_thin_gas_and_stars_are_under_a_tenth() {
    let ls = ledgers(3);
    let mut sum = crate::ledger::Ordinary::default();
    for l in &ls {
        let o = &l.ordinary;
        sum.stars += o.stars;
        sum.cold_gas += o.cold_gas;
        sum.hot_gas += o.hot_gas;
        sum.thrown_out += o.thrown_out;
        sum.black_holes += o.black_holes;
        sum.between_galaxies += o.between_galaxies;
    }
    let t = sum.total();
    let stars = sum.stars / t;
    let thin = (sum.hot_gas + sum.thrown_out + sum.between_galaxies) / t;
    assert!((0.02..0.1).contains(&stars), "{sum:?}");
    assert!(thin > 0.8, "{sum:?}");
    assert!(sum.black_holes / t < 1.0e-3, "{sum:?}");
}

/// Matter gathers into filaments and knots and drains from voids: their
/// share of matter is above and below their share of volume. (The shares
/// themselves depend on how the web is classified: Libeskind et al. 2018,
/// doi:10.1093/mnras/stx1976, compare twelve methods; the T-web at a zero
/// threshold used here gives fewer voids than Cautun et al. 2014's NEXUS+.)
#[test]
fn matter_gathers_into_filaments_and_drains_from_voids() {
    let ls = ledgers(3);
    let mut vol = [0.0; 4];
    let mut mat = [0.0; 4];
    for l in &ls {
        for s in &l.environments {
            vol[s.environment as usize] += s.volume * l.matter_msun / l.matter_msun;
            mat[s.environment as usize] += s.matter;
        }
    }
    let f = Environment::Filament as usize;
    let k = Environment::Knot as usize;
    let v = Environment::Void as usize;
    assert!(mat[f] + mat[k] > vol[f] + vol[k], "{vol:?} {mat:?}");
    assert!(mat[v] < vol[v], "{vol:?} {mat:?}");
}

#[test]
#[ignore]
fn probe() {
    let ls = ledgers(3);
    let (mut st, mut cg, mut hg, mut th, mut bh, mut bw, mut dc, mut db, mut dn) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let mut vol = [0.0; 4];
    let mut mat = [0.0; 4];
    for l in &ls {
        let o = &l.ordinary;
        st += o.stars;
        cg += o.cold_gas;
        hg += o.hot_gas;
        th += o.thrown_out;
        bh += o.black_holes;
        bw += o.between_galaxies;
        dc += l.dark.catalogued_haloes;
        db += l.dark.haloes_below_floor;
        dn += l.dark.never_collapsed;
        for s in &l.environments {
            vol[s.environment as usize] += s.volume / ls.len() as f64;
            mat[s.environment as usize] += s.matter / ls.len() as f64;
        }
    }
    let t = st + cg + hg + th + bh + bw;
    let d = dc + db + dn;
    println!(
        "ordinary: stars {:.3} cold {:.3} hot {:.3} thrown {:.3} bh {:.1e} between {:.3}",
        st / t,
        cg / t,
        hg / t,
        th / t,
        bh / t,
        bw / t
    );
    println!(
        "dark: catalogued {:.3} below {:.3} never {:.3}",
        dc / d,
        db / d,
        dn / d
    );
    println!("env vol {vol:?} mat {mat:?}");
}
