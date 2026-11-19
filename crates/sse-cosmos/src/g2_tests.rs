//! G2 checks: the mass function, bias, counts over a sample volume, growth, spin, mergers.

use crate::cosmology::{quad, Cosmology, Params};
use crate::haloes::{self as hl, HaloParams, Haloes};
use crate::web::{Web, WebParams};
use sse_core::math as m;
use sse_core::{Address, Seed};

fn setup(min_mass: f64) -> (Cosmology, Web, HaloParams) {
    let c = Cosmology::new(Params::default());
    let w = Web::new(
        &c,
        WebParams::default(),
        &Seed::from_text("g2"),
        &Address::universe("g2").unwrap(),
    )
    .unwrap();
    (
        c,
        w,
        HaloParams {
            min_mass,
            ..Default::default()
        },
    )
}

#[test]
fn sheth_tormen_normalisation_and_bias() {
    // All mass is in haloes of some mass: the integral of f(nu) dln nu is 1;
    // the mass-weighted mean bias is 1 (Sheth and Tormen 1999).
    let lo = m::ln(1.0e-10);
    let hi = m::ln(8.0);
    let f = quad(&|l| hl::st_f(m::exp(l)), lo, hi, 256);
    let b = quad(
        &|l| hl::st_f(m::exp(l)) * hl::st_bias(m::exp(l)),
        lo,
        hi,
        256,
    );
    println!("integral f {f:.4}, mean bias {b:.4}");
    assert!((f - 1.0).abs() < 0.01);
    assert!((b - 1.0).abs() < 0.02);
    // Rare haloes are strongly biased; common ones are not.
    assert!(hl::st_bias(3.0) > 3.0 && hl::st_bias(0.5) < 1.0);
}

#[test]
fn counts_over_a_sample_volume_follow_sheth_tormen() {
    let (c, w, p) = setup(1.0e11);
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: p.clone(),
        seed: Seed::from_text("g2"),
        universe: Address::universe("g2").unwrap(),
    };
    let mut cells = vec![];
    for x in 0..4 {
        for y in 0..4 {
            for z in 0..4 {
                cells.push(hs.cell([x, y, z]));
            }
        }
    }
    let volume = 64.0 * p.cell_mpc * p.cell_mpc * p.cell_mpc;
    // Drawn against unconditional expected, per decade.
    for dec in [11, 12, 13] {
        let n: u64 = cells
            .iter()
            .flat_map(|c| c.bins.iter())
            .filter(|b| b.0 >= f64::from(dec) && b.0 < f64::from(dec + 1))
            .map(|b| b.2)
            .sum();
        let expect = quad(
            &|l| hl::dn_dlnm(&c, m::exp(l), 0.0),
            m::ln(10.0) * f64::from(dec),
            m::ln(10.0) * f64::from(dec + 1),
            4,
        ) * volume;
        let sig = m::sqrt(expect);
        println!(
            "1e{dec} to 1e{}: drawn {n}, Sheth-Tormen {expect:.1} (+- {sig:.1} Poisson)",
            dec + 1
        );
        assert!(
            (n as f64 - expect).abs() < 4.0 * sig + 0.15 * expect,
            "decade {dec}"
        );
    }
    // Budget closure and bias: dense cells hold more of their mass in big haloes.
    for cl in &cells {
        assert!(cl.halo_mass <= cl.mass);
        assert_eq!(
            cl.haloes.len() as u64,
            cl.bins.iter().map(|b| b.2).sum::<u64>()
        );
    }
    let mut by: Vec<(f64, f64)> = cells.iter().map(|c| (c.delta_l, c.halo_mass)).collect();
    by.sort_by(|a, b| a.0.total_cmp(&b.0));
    let low: f64 = by[..16].iter().map(|x| x.1).sum();
    let high: f64 = by[48..].iter().map(|x| x.1).sum();
    println!("halo mass in the 16 densest cells {high:.3e}, the 16 emptiest {low:.3e}");
    assert!(high > 2.0 * low);
    // Spin: median 0.035.
    let mut spins: Vec<f64> = cells
        .iter()
        .flat_map(|c| c.haloes.iter().map(|h| h.spin))
        .collect();
    spins.sort_by(|a, b| a.total_cmp(b));
    let med = spins[spins.len() / 2];
    println!("{} haloes, spin median {med:.4}", spins.len());
    assert!((med / 0.035 - 1.0).abs() < 0.1);
    // Mergers per halo against the Fakhouri, Ma and Boylan-Kolchin integral.
    let hs_all: Vec<&hl::Halo> = cells
        .iter()
        .flat_map(|c| c.haloes.iter())
        .filter(|h| h.mass > 1.0e12)
        .collect();
    let drawn: usize = hs_all.iter().map(|h| h.mergers.len()).sum();
    let expect: f64 = hs_all
        .iter()
        .map(|h| {
            quad(
                &|z| {
                    if h.mass_at(z) < p.min_mass * 0.1 {
                        0.0
                    } else {
                        hl::major_rate(h.mass_at(z), p.major_ratio, z)
                    }
                },
                0.0,
                p.merger_z_max,
                12,
            )
        })
        .sum();
    println!(
        "major mergers since z = 6 for {} haloes above 1e12: drawn {drawn}, expected {expect:.1}",
        hs_all.len()
    );
    assert!((drawn as f64 - expect).abs() < 4.0 * m::sqrt(expect) + 0.1 * expect);
}

#[test]
fn growth_histories() {
    let (c, _, _) = setup(1.0e10);
    let z_half = |mass: f64| {
        let (a, b) = hl::correa(&c, mass, 0.0);
        let h = hl::Halo {
            address: String::new(),
            mass,
            position: [0.0; 3],
            alpha: a,
            beta: b,
            z_half: 0.0,
            spin: 0.0,
            mergers: vec![],
            neighbour: None,
        };
        hl::half_mass_z(&h)
    };
    let (z11, z12, z15) = (z_half(1.0e11), z_half(1.0e12), z_half(1.0e15));
    println!("half-mass redshift: 1e11 {z11:.2}, 1e12 {z12:.2}, 1e15 {z15:.2}");
    // Milky-Way haloes assemble half their mass near z = 1 to 1.5, clusters near z = 0.5 (Wechsler et al. 2002; Correa et al. 2015 fig. 4).
    assert!(z12 > 0.8 && z12 < 2.0);
    assert!(z15 > 0.2 && z15 < 0.9);
    assert!(z11 > z12 && z12 > z15, "small haloes form first");
    // Denser surroundings: earlier.
    let (a, b) = hl::correa(&c, 1.0e12, -0.3);
    let h = hl::Halo {
        address: String::new(),
        mass: 1.0e12,
        position: [0.0; 3],
        alpha: a,
        beta: b,
        z_half: 0.0,
        spin: 0.0,
        mergers: vec![],
        neighbour: None,
    };
    assert!(hl::half_mass_z(&h) > z12);
}

#[test]
fn cells_are_order_independent_and_record_mergers() {
    let (c, w, p) = setup(1.0e11);
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: p,
        seed: Seed::from_text("g2"),
        universe: Address::universe("g2").unwrap(),
    };
    let a = hs.cell([2, -1, 5]);
    let _ = hs.cell([0, 0, 0]);
    let b = hs.cell([2, -1, 5]);
    assert_eq!(a, b);
    assert_ne!(a, hs.cell([2, -1, 6]));
    let mut ch = sse_chronicle::Chronicle::new();
    let n: usize = a.haloes.iter().map(|h| h.mergers.len()).sum();
    let ids = hs.record(&mut ch, &a).unwrap();
    assert_eq!(ids.len(), n);
    for e in ch.events() {
        assert_eq!(e.body.kind, "halo.merger");
    }
    // Neighbours: always a larger halo, closer than the search radius.
    for h in &a.haloes {
        if let Some(nb) = &h.neighbour {
            let other = a.haloes.iter().find(|x| x.address == nb.address).unwrap();
            assert!(other.mass > h.mass && nb.distance_mpc < 3.0);
            assert_eq!(nb.bound, nb.approach_gyr.is_some());
        }
    }
}

#[test]
fn g2_golden() {
    crate::golden::check("haloes.json", &crate::scenario::haloes_golden());
}
