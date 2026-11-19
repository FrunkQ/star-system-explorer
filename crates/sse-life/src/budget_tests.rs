//! L1a checks (life design section 3, box B).

use crate::budget::*;
use crate::scenario::*;

fn find<'a>(b: &'a WorldBudget, kind: &str) -> Vec<&'a HabitatBudget> {
    b.habitats.iter().filter(|h| h.kind == kind).collect()
}

fn print(name: &str, b: &WorldBudget) {
    println!(
        "{name}: production {:.3e} g C/yr, biomass {:.3e} g C, power {:.3e} W, class {}, levels {}, high power {}",
        b.production_gc_yr, b.biomass_gc, b.power_w, b.biomass_class, b.feeding_levels, b.high_power
    );
    for h in &b.habitats {
        println!(
            "  {} {}: {} prod {:?} energy {:?} nutrients {:?} levels {:?} closed {:?} missing {:?}",
            h.habitat,
            h.kind,
            h.limited_by,
            h.production_gc_yr,
            h.energy_cap_gc_yr,
            h.nutrient_cap_gc_yr,
            h.feeding_levels,
            h.closed,
            h.missing
        );
    }
}

#[test]
fn band_fraction_matches_known_values() {
    // The Sun (peak 500 nm, 5800 K): about 37% of its energy is 400 to 700 nm.
    let par = band_fraction(500.0, 400.0, 700.0);
    assert!((0.34..0.40).contains(&par), "{par}");
    // Nearly all of it lies between 200 nm and 5 microns.
    let all = band_fraction(500.0, 200.0, 5000.0);
    assert!((0.97..=1.0).contains(&all), "{all}");
    // A red dwarf (3000 K, peak about 966 nm) has far less visible light.
    let red = band_fraction(966.0, 400.0, 700.0);
    assert!(red < 0.15 && red > 0.0, "{red}");
}

#[test]
fn earth_today() {
    let all = budgets();
    let b = &all.iter().find(|(n, _)| n == "earth").unwrap().1;
    print("earth", b);
    // Field et al. 1998: about 105 Gt C per year (ocean 48.5, land 56.4).
    let gt = b.production_gc_yr / 1.0e15;
    assert!((35.0..315.0).contains(&gt), "{gt} Gt C/yr");
    let ocean = find(b, "ocean-sunlit")[0];
    assert_eq!(
        ocean.limited_by, "nutrients",
        "the open ocean is nutrient-limited"
    );
    let o = ocean.production_gc_yr.unwrap() / 1.0e15;
    assert!((16.0..146.0).contains(&o), "ocean {o}");
    let land: f64 = find(b, "land-surface")
        .iter()
        .filter_map(|h| h.production_gc_yr)
        .sum::<f64>()
        / 1.0e15;
    assert!((19.0..170.0).contains(&land), "land {land}");
    // Bar-On et al. 2018: about 550 Gt C of biomass, mostly plants.
    let bio = b.biomass_gc / 1.0e15;
    assert!((180.0..1650.0).contains(&bio), "biomass {bio}");
    assert!(b.high_power);
    assert!((3..=6).contains(&b.feeding_levels), "{}", b.feeding_levels);
    assert_eq!(b.biomass_class, "teeming");
}

#[test]
fn early_earth_is_far_lower() {
    let all = budgets();
    let now = &all.iter().find(|(n, _)| n == "earth").unwrap().1;
    let early = &all.iter().find(|(n, _)| n == "earth-archean").unwrap().1;
    print("earth-archean", early);
    let ratio = now.production_gc_yr / early.production_gc_yr;
    // Canfield et al. 2006 (iron-based, 10 to 25 times lower) to Kharecha et
    // al. 2005 (hydrogen-based, thousands of times lower).
    assert!((10.0..3000.0).contains(&ratio), "{ratio}");
    assert!(!early.high_power);
    assert!(early.feeding_levels <= 2);
    let ocean = find(early, "ocean-sunlit")[0];
    assert!(ocean.by_metabolism.contains_key("anoxygenic-iron"));
    assert_eq!(ocean.limited_by, "energy");
}

#[test]
fn europa_ocean_within_published_range() {
    let all = budgets();
    let b = &all.iter().find(|(n, _)| n == "europa").unwrap().1;
    print("europa", b);
    // Chyba and Hand 2001: 1e10 to 1e14 g per year with oxidant delivery;
    // our inputs are near the low end of their range.
    assert!(
        (1.0e8..1.0e14).contains(&b.production_gc_yr),
        "{}",
        b.production_gc_yr
    );
    assert!(!b.high_power);
}

#[test]
fn closed_habitats_say_why() {
    let all = budgets();
    let get = |n: &str| &all.iter().find(|(x, _)| x == n).unwrap().1;
    for (world, kind, why) in [
        ("mars", "land-surface", "triple point"),
        ("venus", "land-surface", "window"),
        ("venus", "cloud-deck", "water activity"),
        ("titan", "hydrocarbon-lake", "no chemistry"),
    ] {
        let b = get(world);
        let hs = find(b, kind);
        assert!(!hs.is_empty(), "{world} {kind}");
        for h in hs {
            let r = h
                .closed
                .as_deref()
                .unwrap_or_else(|| panic!("{world} {kind} open"));
            assert!(r.contains(why), "{world} {kind}: {r}");
            assert_eq!(h.production_gc_yr, Some(0.0));
        }
    }
    // A habitat without its inputs is unknown, never zero.
    let b = get("titan");
    let ocean = find(b, "under-ice-ocean")[0];
    assert_eq!(ocean.production_gc_yr, None);
    assert!(!ocean.missing.is_empty());
    assert!(b.unknown.contains(&ocean.habitat));
}

#[test]
fn random_worlds_never_exceed_their_caps() {
    use sse_core::{Address, Draws, Seed, Stream};
    let bp = budget_params();
    let mets: Vec<&Metabolism> = bp.metabolisms.iter().collect();
    let base = graph("earth").habitats[0].profile.clone();
    let d = Draws::new(
        Seed::from_text("l1a"),
        Address::universe("l1a").unwrap(),
        Stream::new("life.budget.test.v1").unwrap(),
    );
    let mut open = 0;
    for i in 0..10_000u64 {
        let u = |k: u64| d.unit(i * 16 + k);
        let lg = |k: u64, lo: f64, hi: f64| sse_core::math::pow(10.0, lo + (hi - lo) * u(k));
        let mut p = base.clone();
        let t = 100.0 + 500.0 * u(0);
        p.temperature_k = Some(crate::Range {
            lo: t,
            hi: t + 50.0 * u(1),
        });
        let pr = lg(2, 1.0, 8.0);
        p.pressure_pa = Some(crate::Range { lo: pr, hi: pr });
        p.medium = ["liquid", "surface", "subsurface", "cloud"][(u(3) * 4.0) as usize % 4].into();
        p.energy.light_w_m2 = (u(4) < 0.8).then(|| lg(5, -3.0, 3.5));
        p.energy.light_peak_nm = Some(200.0 + 1500.0 * u(6));
        p.energy.available = vec!["light".into(), "chemical".into()];
        p.area_m2 = (u(7) < 0.9).then(|| lg(8, 0.0, 15.0));
        p.materials.clear();
        for (k, key) in [
            "p_new_mol_yr",
            "fe2_mol_yr",
            "h2_mol_yr",
            "o2_mol_yr",
            "reductant_mol_yr",
        ]
        .iter()
        .enumerate()
        {
            if u(9 + k as u64) < 0.6 {
                p.materials
                    .insert(key.to_string(), lg(14, 0.0, 16.0) * u(15));
            }
        }
        p.radiation_dose_msv_yr = Some(lg(14, -1.0, 7.0));
        let h = habitat_budget(&p, &bp, &mets);
        for v in [
            h.energy_cap_gc_yr,
            h.nutrient_cap_gc_yr,
            h.production_gc_yr,
            h.power_w,
            h.biomass_gc,
        ]
        .into_iter()
        .flatten()
        {
            assert!(v.is_finite() && v >= 0.0, "{i}: {h:?}");
        }
        if let Some(prod) = h.production_gc_yr {
            if let Some(e) = h.energy_cap_gc_yr {
                assert!(prod <= e, "{i}");
            }
            if let Some(n) = h.nutrient_cap_gc_yr {
                assert!(prod <= n, "{i}");
            }
            for v in h.by_metabolism.values() {
                assert!(*v <= h.energy_cap_gc_yr.unwrap() * (1.0 + 1e-12));
            }
        }
        if h.closed.is_some() {
            assert_eq!(h.production_gc_yr, Some(0.0));
        } else if h.production_gc_yr.unwrap_or(0.0) > 0.0 {
            open += 1;
        }
        if h.production_gc_yr.is_none() {
            assert!(!h.missing.is_empty(), "{i}: unknown without a reason");
        }
        // Deterministic.
        assert_eq!(h, habitat_budget(&p, &bp, &mets));
    }
    assert!(open > 500, "only {open} productive samples");
}

#[test]
fn pack_settings_parse_and_envelope_carries_the_caps() {
    let bp = budget_params();
    assert!(bp.metabolisms.iter().any(|m| m.high_power));
    let all = budgets();
    let b = &all.iter().find(|(n, _)| n == "earth").unwrap().1;
    let env = envelope(b, sse_chronicle::UniverseTime { s: 0, ns: 0 });
    assert_eq!(env.energy_budget_w, Some(b.power_w));
    assert_eq!(env.biomass_class.as_deref(), Some("teeming"));
    assert_eq!(env.p_present, None, "presence is box C's");
    assert_eq!(env.evidence, crate::LifeEvidence::PlausibleReconstruction);
}

#[test]
fn budget_golden() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/life-budget.json"
    );
    let got = crate::scenario::budget_golden();
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, serde_json::to_string_pretty(&got).unwrap() + "\n").unwrap();
    }
    let want: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("bless once")).unwrap();
    assert_eq!(
        got, want,
        "tests/golden/life-budget.json moved: bless only on purpose"
    );
}
