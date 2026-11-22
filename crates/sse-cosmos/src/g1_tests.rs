//! G1 checks: closed forms against tabulated values, the spectrum, the web.

use crate::cosmology::{self as cm, Cosmology, Params};
use crate::epochs::{self, EpochParams};
use crate::web::{Cache, Environment, Web, WebParams};
use sse_core::math as m;
use sse_core::{Address, Seed};

fn planck() -> Cosmology {
    Cosmology::new(Params::default())
}

#[test]
fn expansion_and_time() {
    let c = planck();
    // Planck 2018 table 2 (last column): age 13.787 Gyr; z* = 1089.80, age there about 0.37 Myr.
    let age = c.age_gyr();
    println!(
        "age {age:.4} Gyr, Omega_r {:.3e}, Omega_L {:.4}",
        c.omega_r, c.omega_l
    );
    assert!((age / 13.787 - 1.0).abs() < 0.002);
    let zr = epochs::recombination_z(&c);
    let tr = c.time_of_scale(1.0 / (1.0 + zr)) * 1.0e9;
    println!("z* {zr:.1}, t* {:.0} yr", tr);
    assert!((zr - 1089.9).abs() < 3.0);
    assert!(tr > 3.5e5 && tr < 3.95e5);
    // The closed form (no radiation) agrees to 0.5 percent below z = 10.
    for z in [0.0, 1.0, 3.0, 10.0] {
        let a = 1.0 / (1.0 + z);
        let closed = cm::time_of_scale_closed(a, c.p.h0, c.p.omega_m, 1.0 - c.p.omega_m);
        let full = c.time_of_scale(a);
        println!("z {z}: closed {closed:.5} with radiation {full:.5} Gyr");
        assert!(
            (closed / full - 1.0).abs() < 5.0e-3,
            "z {z}: {closed} {full}"
        );
        let back = cm::scale_of_time_closed(closed, c.p.h0, c.p.omega_m, 1.0 - c.p.omega_m);
        assert!((back / a - 1.0).abs() < 1.0e-12);
        assert!((c.scale_of_time(full) / a - 1.0).abs() < 1.0e-9);
    }
    // Matter era: t proportional to a^1.5 well after equality.
    let r = c.time_of_scale(0.02) / c.time_of_scale(0.01);
    assert!((r / m::pow(2.0, 1.5) - 1.0).abs() < 0.02);
}

#[test]
fn growth_factor() {
    let c = planck();
    // Carroll, Press and Turner 1992 is good to about 1 percent against the exact integral.
    for a in [0.1, 0.25, 0.5, 0.75, 1.0] {
        let (g, ex) = (c.growth(a), c.growth_exact(a));
        println!("a {a}: CPT {g:.4} exact {ex:.4}");
        assert!((g / ex - 1.0).abs() < 0.015);
    }
    // Early on D grows like a.
    assert!((c.growth(0.01) / c.growth(0.02) - 0.5).abs() < 0.01);
}

#[test]
fn spectrum() {
    let c = planck();
    // Normalised to sigma8.
    assert!((m::sqrt(c.sigma2_r(8.0 / c.h)) / c.p.sigma8 - 1.0).abs() < 1e-12);
    // Transfer: 1 on large scales, falling as ln k / k^2 on small ones.
    assert!((c.transfer(1.0e-5) - 1.0).abs() < 1.0e-3);
    let r = c.transfer(100.0) / c.transfer(10.0);
    assert!(r > 0.01 * 1.2 && r < 0.01 * 2.0, "{r}");
    // sigma(M) falls with mass; its slope near 1e14 solar masses is about -0.3
    // (the cluster-scale slope of LCDM), and a Milky-Way halo (1e12) has sigma near 2.
    let s12 = c.sigma_m(1.0e12);
    let slope = (m::ln(c.sigma_m(2.0e14)) - m::ln(c.sigma_m(5.0e13))) / m::ln(4.0);
    println!(
        "sigma(1e12) {s12:.3}, slope at 1e14 {slope:.3}, sigma(1e6) {:.2}",
        c.sigma_m(1.0e6)
    );
    assert!(s12 > 1.5 && s12 < 2.6);
    assert!(slope < -0.2 && slope > -0.45);
    let mut prev = f64::INFINITY;
    for (_, s) in &c.sigma_table {
        assert!(*s < prev);
        prev = *s;
    }
}

#[test]
fn epochs_in_order_and_where_they_belong() {
    let c = planck();
    let ep = epochs::epochs(&c, &EpochParams::default());
    for e in &ep {
        println!(
            "{:>22} 1e{:6.2} yr z {:?} {}",
            e.id,
            e.log10_yr,
            e.z.map(|z| (z * 100.0).round() / 100.0),
            if e.speculative { "(speculative)" } else { "" }
        );
    }
    let get = |id: &str| ep.iter().find(|e| e.id == id).unwrap();
    // First stars: 3-sigma 1e6 solar-mass minihaloes near z = 20 to 30 (Tegmark et al. 1997; Barkana and Loeb 2001 fig. 6).
    let zf = get("first_stars").z.unwrap();
    assert!(zf > 15.0 && zf < 35.0, "{zf}");
    // Reionisation half way: Planck 2018 table 2 7.82 +- 0.71 (with BAO, the column our parameters use; 7.67 +- 0.73 without); complete by about z = 5.5 to 6.5 (quasar spectra).
    let zh = get("reionisation").z.unwrap();
    assert!(zh > 7.0 && zh < 8.4, "{zh}");
    let ze = get("reionisation_end").z.unwrap();
    assert!(ze > 5.3 && ze < 6.8, "{ze}");
    let zp = get("peak_star_formation").z.unwrap();
    assert!((zp - 1.86).abs() < 0.05, "{zp}");
    assert!((get("now").log10_yr - m::log10(13.8e9)).abs() < 1e-9);
    for w in ep.windows(2) {
        assert!(w[0].log10_yr <= w[1].log10_yr);
    }
    assert!(ep
        .iter()
        .filter(|e| e.speculative)
        .all(|e| e.log10_yr > 11.0));
    // On the chronicle: everything up to now, nothing beyond 292 Gyr.
    let mut ch = sse_chronicle::Chronicle::new();
    let ids = epochs::record(&mut ch, &Address::universe("g1").unwrap(), &ep).unwrap();
    assert_eq!(ids.len(), 7);
}

fn web(octaves: u32) -> (Cosmology, Web) {
    let c = planck();
    let w = Web::new(
        &c,
        WebParams {
            octaves,
            ..Default::default()
        },
        &Seed::from_text("g1"),
        &Address::universe("g1").unwrap(),
    )
    .unwrap();
    (c, w)
}

/// Deterministic sample points: a scrambled lattice over a 640 Mpc box.
fn points(n: u32) -> Vec<[f64; 3]> {
    (0..n)
        .map(|i| {
            let f = |k: u32, p: f64| {
                m::fmod(f64::from(i) * p + f64::from(k) * 0.123, 1.0) * 640.0 - 320.0
            };
            [
                f(1, 0.618_033_988_7),
                f(2, 0.754_877_666_2),
                f(3, 0.569_840_290_9),
            ]
        })
        .collect()
}

#[test]
fn web_variance_matches_the_spectrum_at_three_scales() {
    let (_, w) = web(6);
    let pts = points(2000);
    let mut cache = Cache::default();
    for j in [1_u32, 3, 5] {
        let v: Vec<f64> = pts.iter().map(|p| w.delta0(&mut cache, *p, j)).collect();
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        let var = v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / v.len() as f64;
        println!(
            "octaves to {j} (spacing {:.1} Mpc): variance {var:.4}, target sigma^2 {:.4}",
            w.spacing(j),
            w.variance[j as usize]
        );
        assert!(
            (var / w.variance[j as usize] - 1.0).abs() < 0.12,
            "scale {j}"
        );
        assert!(mean.abs() < 0.1 * m::sqrt(var));
    }
    // Two-point: correlation falls with separation at a fixed scale.
    let corr = |d: f64| {
        let a: Vec<f64> = pts
            .iter()
            .map(|p| w.delta0(&mut Cache::default(), *p, 2))
            .collect();
        let b: Vec<f64> = pts
            .iter()
            .map(|p| w.delta0(&mut Cache::default(), [p[0] + d, p[1], p[2]], 2))
            .collect();
        let n = a.len() as f64;
        let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
        let cov = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x - ma) * (y - mb))
            .sum::<f64>()
            / n;
        let va = a.iter().map(|x| (x - ma) * (x - ma)).sum::<f64>() / n;
        cov / va
    };
    let (c1, c2, c3) = (corr(2.0), corr(10.0), corr(40.0));
    println!("correlation at 2, 10, 40 Mpc: {c1:.3} {c2:.3} {c3:.3}");
    assert!(c1 > c2 && c2 > c3 && c1 > 0.8 && c3.abs() < 0.3);
}

#[test]
fn t_web_fractions_match_a_gaussian_field() {
    // Doroshkevich 1970: for a Gaussian field and threshold 0, voids and knots
    // are about 8 percent each, sheets and filaments about 42 percent each.
    let (_, w) = web(4);
    let pts = points(2000);
    let mut cache = Cache::default();
    let mut counts = [0usize; 4];
    for p in &pts {
        let e = w.environment(&mut cache, *p, 3, 1.0);
        counts[e as usize] += 1;
    }
    let f: Vec<f64> = counts
        .iter()
        .map(|c| *c as f64 / pts.len() as f64)
        .collect();
    println!(
        "void {:.3} sheet {:.3} filament {:.3} knot {:.3}",
        f[0], f[1], f[2], f[3]
    );
    assert!((f[0] - 0.08).abs() < 0.05 && (f[3] - 0.08).abs() < 0.05);
    assert!((f[1] - 0.42).abs() < 0.08 && (f[2] - 0.42).abs() < 0.08);
    assert_eq!(Environment::from_count(2), Environment::Filament);
}

#[test]
fn foam_grows_and_is_deterministic() {
    let (c, w) = web(5);
    let early = w.slice(&c, 0.1, [0.0; 3], 200.0, 16, 4);
    let now = w.slice(&c, 1.0, [0.0; 3], 200.0, 16, 4);
    let spread = |s: &crate::web::Slice| {
        let n = s.density.len() as f64;
        let mean = s.density.iter().sum::<f64>() / n;
        m::sqrt(
            s.density
                .iter()
                .map(|x| (x - mean) * (x - mean))
                .sum::<f64>()
                / n,
        )
    };
    println!(
        "density spread at a = 0.1: {:.3}, now: {:.3}",
        spread(&early),
        spread(&now)
    );
    assert!(spread(&now) > 3.0 * spread(&early));
    // Identical when read again, and in another order.
    let again = w.slice(&c, 1.0, [0.0; 3], 200.0, 16, 4);
    assert_eq!(now, again);
    let (_, w2) = web(5);
    let mut cache = Cache::default();
    let p = [37.5, -12.25, 8.0];
    let a = w2.delta0(&mut cache, p, 4);
    let mut cache2 = Cache::default();
    let _ = w2.delta0(&mut cache2, [500.0, 500.0, 500.0], 4);
    assert_eq!(a, w2.delta0(&mut cache2, p, 4));
}

#[test]
fn cosmology_atoms() {
    let atoms = cm::atoms();
    let get = |id: &str| atoms.iter().find(|a| a.decl().id == id).unwrap();
    let p = [67.66, 0.3111, 0.6889];
    for a in &atoms {
        assert!(!a.decl().references.is_empty());
    }
    let yr = cm::YEAR_S * 1.0e9;
    let t = get("sse.cosmo.time_of_scale").eval(&[1.0], &p).unwrap()[0] / yr;
    assert!((t - 13.79).abs() < 0.01, "{t}");
    let a = get("sse.cosmo.scale_of_time").eval(&[t * yr], &p).unwrap()[0];
    assert!((a - 1.0).abs() < 1e-12);
    let c = planck();
    let d = get("sse.cosmo.growth").eval(&[0.5], &p).unwrap()[0];
    assert!((d / c.growth(0.5) - 1.0).abs() < 1e-3);
    let h = get("sse.cosmo.hubble_rate").eval(&[1.0], &p).unwrap()[0];
    assert!((h * cm::MPC_M / 1000.0 - 67.66).abs() < 1e-9);
    assert!(get("sse.cosmo.time_of_scale").eval(&[-1.0], &p).is_err());
}

#[test]
fn g1_golden() {
    crate::golden::check("cosmology.json", &crate::scenario::cosmology_golden());
}

/// DM1: the measured parameters come from the sheet's Cosmos group through
/// the handle; a pack that still sets one is told where it went, and an
/// edited handle reaches the cosmology.
#[test]
fn cosmology_reads_the_cosmos_group() {
    use crate::scenario::{base_cosmology_pack, CosmologyPack};
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap();
    assert_eq!(cp.params, Params::default());
    let mut j: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-cosmology/pack.json")).unwrap();
    j["provides"]["other"]["cosmology"]["params"]["omega_m"] = serde_json::json!(0.3);
    let old = sse_pack::PackManifest::from_json(&j.to_string()).unwrap();
    let e = CosmologyPack::from_packs(&[old]).unwrap_err();
    assert!(e.contains("Omega_m") && e.contains("Cosmos group"), "{e}");
    let mut c = sse_core::constants::Constants::REAL;
    c.cosmos.omega_m *= 2.0;
    let edited = sse_core::constants::with(c, || {
        CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap()
    });
    assert_eq!(edited.params.omega_m, 0.6222);
    assert!(Cosmology::new(edited.params).age_gyr() < planck().age_gyr());
}
