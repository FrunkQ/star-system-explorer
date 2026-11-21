//! G0 acceptance: system generation runs from an envelope fixture, and the
//! default patch's numbers stand against the published values they cite.

use crate::patch::{feh_ism, GYR_S};
use crate::*;
use sse_chronicle::Chronicle;
use sse_core::cbor::Value;
use sse_resolve::{resolve_all, resolve_recorded, ChildRef, Envelope};

fn model() -> PatchModel {
    PatchModel::milky_way_like()
}
fn births(env: &Envelope) -> Vec<SystemBirth> {
    let seed = scenario::seed();
    resolve_all(&seed, env)
        .unwrap()
        .iter()
        .map(|r| SystemBirth::from_facts(&r.address, &r.facts).unwrap())
        .collect()
}
/// Resolve a big patch's systems (each cohort in one pass) for statistics.
fn big(site: &Site) -> (Conditions, Vec<SystemBirth>) {
    let m = model();
    let seed = scenario::seed();
    let (env, cond) = patch_envelope(&m, "u:g0/patch:6:big".parse().unwrap(), site);
    let mut out = Vec::new();
    for r in resolve_all(&seed, &env).unwrap() {
        out.extend(births(&cohort_envelope(&m, &cond, &r).unwrap()));
    }
    (cond, out)
}
fn mean_sd(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mu = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / (n - 1.0);
    (mu, var.sqrt())
}

#[test]
fn pack_model_loads_and_rejects_bad_data() {
    let m = model();
    assert_eq!(m.id, "milky-way-like.v1");
    let mut bad = m.clone();
    bad.cohorts[0].population = "bulge".into();
    assert_eq!(
        bad.validate().unwrap_err(),
        "cohort names unknown population bulge"
    );
    let mut bad = m.clone();
    bad.imf = sse_resolve::Distribution::Fixed { value: 1.0 };
    assert!(bad.validate().unwrap_err().starts_with("imf must be"));
}

/// The envelope fixture is what the code builds; F0 can start from the file.
#[test]
fn envelope_fixture_is_current() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/cosmos/solar-patch.json"
    );
    let got = serde_json::to_string_pretty(&scenario::fixture()).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
        std::fs::write(path, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(path).expect("fixture; run with SSE_BLESS=1");
    assert_eq!(
        got.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "solar-patch fixture changed"
    );
}

/// Acceptance (galaxy design 12, G0): system generation from the envelope
/// fixture alone, with the generic resolver: same systems as the cosmos
/// code, budgets close exactly, and any resolve order gives the same state.
#[test]
fn systems_generate_from_the_envelope_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/cosmos/solar-patch.json"
    );
    let fx: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let patch: Envelope = serde_json::from_value(fx["patch"].clone()).unwrap();
    let cohorts: Vec<Envelope> = serde_json::from_value(fx["cohorts"].clone()).unwrap();
    patch.validate().unwrap();
    let seed = scenario::seed();

    // Budget closure, patch to cohorts to systems.
    let total = patch.budgets["birth_mass"].amount;
    let cohort_res = resolve_all(&seed, &patch).unwrap();
    let int = |r: &sse_resolve::Resolved| match r.facts["budget.birth_mass"] {
        Value::Int(v) => v as u64,
        _ => panic!(),
    };
    assert_eq!(cohort_res.iter().map(int).sum::<u64>(), total);
    let mut from_fixture = Vec::new();
    for (r, env) in cohort_res.iter().zip(&cohorts) {
        env.validate().unwrap();
        assert_eq!(env.address, r.address);
        assert_eq!(env.budgets["birth_mass"].amount, int(r));
        let systems = resolve_all(&seed, env).unwrap();
        let split = sse_resolve::children(&seed, env).unwrap().splits["birth_mass"].clone();
        assert_eq!(
            systems.iter().map(int).sum::<u64>() + split.residue,
            int(r),
            "{}",
            env.address
        );
        from_fixture.extend(
            systems
                .iter()
                .map(|s| SystemBirth::from_facts(&s.address, &s.facts).unwrap()),
        );
    }

    // Same systems as form_patch, recorded in the chronicle.
    let m = model();
    let mut c = Chronicle::new();
    let f = form_patch(&mut c, &seed, &m, scenario::address(), &Site::solar(&m)).unwrap();
    let formed: Vec<SystemBirth> = f
        .systems
        .iter()
        .map(|(a, r)| SystemBirth::from_facts(a, &r.facts).unwrap())
        .collect();
    assert_eq!(formed, from_fixture);
    assert!((250..380).contains(&formed.len()), "{}", formed.len());

    // Order independence: systems straight from their cohorts in a scrambled
    // order (cohorts unrecorded), then the cohorts: same state.
    let pubr = publisher();
    let mut refs: Vec<(usize, u64)> = Vec::new();
    for (k, env) in cohorts.iter().enumerate() {
        for i in 0..sse_resolve::children(&seed, env).unwrap().generated {
            refs.push((k, i));
        }
    }
    let n = refs.len();
    let mut c2 = Chronicle::new();
    for j in 0..n {
        let (k, i) = refs[(j * 7919) % n];
        resolve_recorded(&mut c2, &seed, &cohorts[k], &ChildRef::Generated(i), &pubr).unwrap();
    }
    for k in (0..cohorts.len() as u64).rev() {
        resolve_recorded(&mut c2, &seed, &patch, &ChildRef::Generated(k), &pubr).unwrap();
    }
    assert_eq!(c2.len(), c.len());
    assert_eq!(
        c2.state_at(patch.epoch).hash(),
        c.state_at(patch.epoch).hash()
    );
}

/// The solar neighbourhood's systems against the values the pack cites.
#[test]
fn solar_neighbourhood_statistics() {
    let m = model();
    let mut site = Site::solar(&m);
    site.size_pc = 60.0; // about 16,000 systems
    let (cond, b) = big(&site);
    assert!(b.len() > 14_000, "{}", b.len());
    let now = site.epoch_gyr();

    // Metallicity distribution of the neighbourhood. Geneva-Copenhagen
    // survey: mean -0.14 (Nordstrom et al. 2004; Holmberg et al. 2009) to
    // -0.06 after recalibration (Casagrande et al. 2011), spread about
    // 0.2 dex. This volume sample counts old thick-disc and halo stars in
    // full, which a magnitude-limited F and G dwarf survey underweights, so
    // its spread runs wider (0.26).
    let feh: Vec<f64> = b.iter().map(|s| s.feh).collect();
    let (mu, sd) = mean_sd(&feh);
    assert!((-0.16..=-0.04).contains(&mu), "mean [Fe/H] {mu}");
    assert!((0.18..=0.28).contains(&sd), "spread {sd}");
    // Few metal-poor stars (no G-dwarf excess): under 3% below -0.8.
    let poor = feh.iter().filter(|x| **x < -0.8).count() as f64 / feh.len() as f64;
    assert!(poor < 0.03, "{poor}");

    // Two disc sequences: thick disc old and alpha-rich, thin disc not.
    let pick = |p: &str| b.iter().filter(|s| s.population == p).collect::<Vec<_>>();
    let thin = pick("thin");
    let thick = pick("thick");
    let a_thin = mean_sd(&thin.iter().map(|s| s.alpha_fe).collect::<Vec<_>>()).0;
    let a_thick = mean_sd(&thick.iter().map(|s| s.alpha_fe).collect::<Vec<_>>()).0;
    assert!(a_thick - a_thin > 0.1, "{a_thick} {a_thin}");
    assert!(thick.iter().all(|s| s.age_gyr(now) >= 8.0 - 1e-9));
    assert!(thin.iter().all(|s| s.age_gyr(now) <= 8.0 + 1e-9));
    // Thick to thin near the midplane about 0.12 (Juric et al. 2008).
    let ratio = thick.len() as f64 / thin.len() as f64;
    assert!((0.09..=0.16).contains(&ratio), "{ratio}");

    // Mass function: the share of systems above 1 solar mass matches the
    // Kroupa IMF's (masses are shares of the cohort budget, so this checks
    // the partition carries the IMF through).
    let want = 1.0 - m.imf.cdf(1.0).unwrap();
    let got = b.iter().filter(|s| s.mass_msun > 1.0).count() as f64 / b.len() as f64;
    let se = (want * (1.0 - want) / b.len() as f64).sqrt();
    assert!((got - want).abs() < 4.0 * se, "{got} {want}");
    let mass: f64 = b.iter().map(|s| s.mass_msun).sum();
    assert!((mass / cond.expected_systems / cond.mean_birth_mass_msun - 1.0).abs() < 0.01);

    // Birth environment: C/O rises with [Fe/H]; Al-26 mostly below the Sun's.
    let rich: Vec<f64> = b.iter().filter(|s| s.feh > 0.1).map(|s| s.c_o).collect();
    let poor_co: Vec<f64> = b.iter().filter(|s| s.feh < -0.4).map(|s| s.c_o).collect();
    assert!(mean_sd(&rich).0 > mean_sd(&poor_co).0);
    let low_al = b.iter().filter(|s| s.al26_rel < 1.0).count() as f64 / b.len() as f64;
    assert!(low_al > 0.75, "{low_al}");
    // Halo stars: metal-poor and lumpier in r-process.
    let halo = pick("halo");
    assert!(!halo.is_empty());
    assert!(mean_sd(&halo.iter().map(|s| s.feh).collect::<Vec<_>>()).0 < -1.0);
}

/// Rates and hazards against the values the pack cites.
#[test]
fn rates_and_hazards() {
    let m = model();
    let c = conditions(&m, &Site::solar(&m));
    // About one core-collapse supernova per 100 solar masses formed (Kroupa).
    assert!(
        (0.008..=0.013).contains(&c.cc_sn_per_msun),
        "{}",
        c.cc_sn_per_msun
    );
    // Galaxy-wide: 1.65 solar masses a year gives 1.8 per century,
    // against 1.63 +- 0.46 (Rozwadowska et al. 2021).
    let per_century = m.sfr.galaxy_msun_per_yr * 100.0 * c.cc_sn_per_msun;
    assert!(
        (1.63 - 0.46..=1.63 + 0.46).contains(&per_century),
        "{per_century}"
    );
    // Stars passing within 1 pc: 19.7 per Myr (Bailer-Jones et al. 2018);
    // this counts systems of stars only (no brown dwarfs), so it may run low,
    // but within a factor of two.
    assert!(
        (19.7 / 2.0..=19.7 * 2.0).contains(&c.encounters_per_myr),
        "{}",
        c.encounters_per_myr
    );
    // Metals have been enough for rocky planets here for most of the
    // galaxy's life.
    let rocky = c.rocky_metals_for_gyr.unwrap();
    assert!((11.0..=13.0).contains(&rocky), "{rocky}");
    assert!(c.sterilising_sn_per_gyr > 0.0 && c.sterilising_sn_per_gyr < 2.0);
    assert!(c.ia_sn_gyr_pc3 > 0.0 && c.ia_sn_gyr_pc3 < c.cc_sn_gyr_pc3);
}

/// Where and when: the gradient, the thick disc above the plane, the outer
/// disc enriched late, and a young galaxy with no thin disc.
#[test]
fn conditions_follow_place_and_time() {
    let m = model();
    let now = m.now_gyr;
    let slope = (feh_ism(&m, 14.0, now) - feh_ism(&m, 4.0, now)) / 10.0;
    assert!((slope + 0.06).abs() < 1e-12, "{slope}");
    let at = |r: f64, z: f64, t: f64| {
        conditions(
            &m,
            &Site {
                r_kpc: r,
                z_pc: z,
                size_pc: 16.0,
                epoch: patch::gyr_time(t),
            },
        )
    };
    let high = at(8.2, 1500.0, now);
    assert!(high.density_pc3["thick"] > high.density_pc3["thin"]);
    let (inner, outer) = (at(4.0, 0.0, now), at(16.0, 0.0, now));
    assert!(inner.rocky_metals_for_gyr.unwrap() > outer.rocky_metals_for_gyr.unwrap());
    assert!(inner.systems_per_pc3 > outer.systems_per_pc3);
    // Far out, the ISM has not yet reached the rocky floor.
    let edge = at(40.0, 0.0, now);
    assert_eq!(edge.rocky_metals_for_gyr, None);
    // 4 Gyr after the origin: halo and part of the thick disc only.
    let young = at(8.2, 0.0, 4.0);
    assert!(!young.density_pc3.contains_key("thin"));
    assert!(young.feh_ism < at(8.2, 0.0, now).feh_ism);
    assert!(young.expected_systems < at(8.2, 0.0, now).expected_systems / 5.0);
    let env = patch_envelope(&m, "u:g0/patch:9:young".parse().unwrap(), &young.site).0;
    env.validate().unwrap();
    let c = sse_resolve::resolve_all(&scenario::seed(), &env).unwrap();
    let cenv = cohort_envelope(&m, &young, &c[1]).unwrap();
    for s in births(&cenv) {
        assert!(s.birth_gyr <= 4.0 && s.birth_gyr >= 1.8, "{}", s.birth_gyr);
    }
    let _ = GYR_S;
}

/// Golden digest; the same function runs in the browser harness.
#[test]
fn golden_digest() {
    crate::golden::check("cosmos.json", &crate::scenario::golden());
}

/// Dying and massive stars near the Sun against what is counted there.
#[test]
fn glows_near_the_sun() {
    let m = model();
    let site = Site::solar(&m);
    let c = conditions(&m, &site);
    let v = site.volume_pc3();
    // White dwarf birth rate near the Sun: about 1e-12 per pc^3 per year
    // (Liebert, Bergeron and Holberg 2005); within a factor of two.
    let per_yr = c.wd_birth_gyr_pc3 * 1e-9;
    assert!((0.5e-12..=2e-12).contains(&per_yr), "{per_yr}");
    // A solar-type star sheds most of its mass: envelopes of 0.5 to 3 solar masses.
    assert!(
        (0.5..=3.0).contains(&c.pn_envelope_msun),
        "{}",
        c.pn_envelope_msun
    );
    // Planetary nebulae near the Sun: about 25 per kpc^3 (Frew 2008);
    // within a factor of three.
    let pn_kpc3 = c.expected_planetary_nebulae / v * 1e9;
    assert!((25.0 / 3.0..=75.0).contains(&pn_kpc3), "{pn_kpc3}");
    // Lifetime implied: tens of thousands of years (observed 10 to 50 kyr).
    let life_yr = c.expected_planetary_nebulae / v / per_yr;
    assert!((1e4..=5e4).contains(&life_yr), "{life_yr}");
    // Remnants and ionising stars are rare in a small box; present and positive.
    assert!(c.expected_cc_remnants > c.expected_ia_remnants);
    assert!(c.ionising_stars_pc3 > 1e-8 && c.ionising_stars_pc3 < 1e-6);
    // A box with no young stars has no ionising stars and no new remnants.
    let mut old = m.clone();
    old.cohorts.retain(|k| k.to_gyr < m.now_gyr - 1.0);
    let c_old = conditions(&old, &Site::solar(&old));
    assert_eq!(c_old.ionising_stars_pc3, 0.0);
    assert_eq!(c_old.expected_cc_remnants, 0.0);
    assert!(c_old.expected_planetary_nebulae > 0.0);
}
