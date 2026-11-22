//! E1: the gradient catalogue (evolution design sections 3 and 14).

use crate::budget::{HabitatBudget, WorldBudget};
use crate::chemistry::ThermoTable;
use crate::gradients::*;
use crate::habitat::HabitatGraph;
use crate::scenario::*;

fn gp() -> GradientParams {
    GradientParams::from_packs(&[base_life_pack()]).expect("life gradients")
}
fn table() -> ThermoTable {
    ThermoTable::from_packs(&[base_life_pack()]).expect("life chemistry")
}
fn run(g: &HabitatGraph) -> (Vec<Catalogue>, WorldBudget) {
    world_catalogue(g, &budget_params(), &gp(), &table())
}

fn show(name: &str, cats: &[Catalogue], b: &WorldBudget) {
    println!(
        "{name}: production {:.3e} g C/yr, biomass {:.3e}, levels {}, high power {}, unknown {}",
        b.production_gc_yr,
        b.biomass_gc,
        b.feeding_levels,
        b.high_power,
        b.unknown.len()
    );
    for (c, h) in cats.iter().zip(&b.habitats) {
        println!(
            "  {} {}: {} prod {:?} secondary {:?} closed {:?} missing {:?}",
            c.kind,
            h.habitat,
            h.limited_by,
            h.production_gc_yr,
            h.secondary_gc_yr,
            h.closed,
            h.missing
        );
        for g in &c.gradients {
            println!(
                "    {} {} open {} dg {:?} power {:?} prod {:?} why {:?}",
                g.kind, g.id, g.open, g.dg_kj, g.power_w, g.production_gc_yr, g.why
            );
        }
    }
}

/// The listed metabolisms an open gradient on this world regenerates, most powerful first.
fn regenerated(cats: &[Catalogue]) -> Vec<String> {
    let mut v: Vec<(f64, String)> = cats
        .iter()
        .flat_map(|c| c.gradients.iter())
        .filter(|g| g.open && g.production_gc_yr.is_some_and(|x| x > 0.0))
        .filter_map(|g| g.matches.clone().map(|m| (g.power_w.unwrap_or(0.0), m)))
        .collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut out = vec![];
    for (_, m) in v {
        if !out.contains(&m) {
            out.push(m);
        }
    }
    out
}

fn kind<'a>(b: &'a WorldBudget, k: &str) -> Vec<&'a HabitatBudget> {
    b.habitats.iter().filter(|h| h.kind == k).collect()
}

#[test]
fn earth_today_meets_the_fidelity_contract() {
    let (cats, b) = run(&graph("earth"));
    show("earth", &cats, &b);
    assert_eq!(
        regenerated(&cats).first().map(String::as_str),
        Some("oxygenic")
    );
    // The same published ranges L1a is checked against (Field et al. 1998;
    // Bar-On et al. 2018).
    let gt = b.production_gc_yr / 1.0e15;
    assert!((35.0..315.0).contains(&gt), "{gt} Gt C/yr");
    let bio = b.biomass_gc / 1.0e15;
    assert!((180.0..1650.0).contains(&bio), "biomass {bio}");
    assert!(b.high_power);
    assert!((3..=6).contains(&b.feeding_levels));
    // Heterotrophs: soils and the ocean interior are no longer unknown.
    for k in ["soil", "ocean-interior"] {
        for h in kind(&b, k) {
            assert!(h.production_gc_yr.is_some(), "{k}: {h:?}");
            assert!(h.secondary_gc_yr.is_some_and(|x| x > 0.0), "{k}: {h:?}");
        }
    }
    // Primary production only: heterotrophs do not make new carbon.
    let listed = crate::budget::world_budget(&graph("earth"), &budget_params(), &all_metabolisms());
    let ratio = b.production_gc_yr / listed.production_gc_yr;
    assert!((0.5..2.0).contains(&ratio), "{ratio}");
}

#[test]
fn early_earth_regenerates_the_anaerobic_metabolisms() {
    let (cats, b) = run(&archean_earth());
    show("archean", &cats, &b);
    let r = regenerated(&cats);
    assert!(r.contains(&"anoxygenic-iron".to_string()), "{r:?}");
    assert!(!b.high_power || r.contains(&"oxygenic".to_string()));
    // With a hydrogen supply too (Kharecha et al. 2005, about 1e12 to 1e13
    // mol per year of volcanic H2), methanogens and hydrogen photosynthesis appear.
    let (cats, _) = run(&archean_with_hydrogen(5.0e12));
    let r = regenerated(&cats);
    for m in ["anoxygenic-hydrogen", "methanogen"] {
        assert!(r.contains(&m.to_string()), "{m}: {r:?}");
    }
}

#[test]
fn europa_counts_its_supply_once() {
    let g = graph("europa");
    let (cats, b) = run(&g);
    show("europa", &cats, &b);
    let r = regenerated(&cats);
    assert_eq!(
        r.first().map(String::as_str),
        Some("aerobic-chemolithotroph"),
        "{r:?}"
    );
    // Chyba and Hand 2001: 1e10 to 1e14 g per year with oxidant delivery; our
    // inputs sit near the low end. L1a counted the one ocean's supply in
    // both habitats; here it is shared, so the total is half.
    let listed = crate::budget::world_budget(&g, &budget_params(), &all_metabolisms());
    assert!(
        (1.0e8..1.0e14).contains(&b.production_gc_yr),
        "{}",
        b.production_gc_yr
    );
    assert!(
        b.production_gc_yr < listed.production_gc_yr * 0.75,
        "{} vs {}",
        b.production_gc_yr,
        listed.production_gc_yr
    );
    assert!(!b.high_power);
}

#[test]
fn closed_and_spent_gradients_say_why() {
    for (world, k, why) in [
        ("mars", "land-surface", "triple point"),
        ("venus", "cloud-deck", "water activity"),
        ("titan", "hydrocarbon-lake", "no chemistry"),
    ] {
        let (_, b) = run(&graph(world));
        for h in kind(&b, k) {
            assert!(
                h.closed.as_deref().unwrap_or("").contains(why),
                "{world} {k}: {h:?}"
            );
        }
    }
    // Iron oxidation at the modern ocean's pH: oxygen rusts the iron first.
    let (cats, _) = run(&archean_earth());
    let g = cats
        .iter()
        .flat_map(|c| c.gradients.iter())
        .find(|g| g.id == "iron-oxidation-aq")
        .expect("iron oxidation listed");
    assert_eq!(g.abiotic, "spent", "{g:?}");
    assert!(!g.open);
}

#[test]
fn random_habitats_never_break_a_cap() {
    use sse_core::{Address, Draws, Seed, Stream};
    let (bp, gp, t) = (budget_params(), gp(), table());
    let base = graph("earth").habitats[0].profile.clone();
    let d = Draws::new(
        Seed::from_text("e1"),
        Address::universe("e1").unwrap(),
        Stream::new("life.gradients.test.v1").unwrap(),
    );
    let keys = [
        "p_new_mol_yr",
        "fe2_mol_yr",
        "h2_mol_yr",
        "o2_mol_yr",
        "reductant_mol_yr",
        "ch4_mol_yr",
        "h2s_mol_yr",
    ];
    let mut open = 0;
    for i in 0..10_000u64 {
        let u = |k: u64| d.unit(i * 32 + k);
        let lg = |k: u64, lo: f64, hi: f64| sse_core::math::pow(10.0, lo + (hi - lo) * u(k));
        let mut p = base.clone();
        let temp = 100.0 + 500.0 * u(0);
        p.temperature_k = Some(crate::Range {
            lo: temp,
            hi: temp + 50.0 * u(1),
        });
        let pr = lg(2, 1.0, 8.0);
        p.pressure_pa = Some(crate::Range { lo: pr, hi: pr });
        p.medium = ["liquid", "surface", "subsurface", "cloud"][(u(3) * 4.0) as usize % 4].into();
        p.energy.light_w_m2 = (u(4) < 0.8).then(|| lg(5, -3.0, 3.5));
        p.energy.light_peak_nm = Some(200.0 + 1500.0 * u(6));
        p.energy.available = if u(7) < 0.5 {
            vec!["light".into(), "chemical".into()]
        } else {
            vec!["oxidant".into(), "chemical".into()]
        };
        p.area_m2 = (u(8) < 0.9).then(|| lg(9, 0.0, 15.0));
        p.properties.insert("ph".into(), 2.0 + 10.0 * u(10));
        p.materials.clear();
        let mut sup = std::collections::BTreeMap::new();
        for (k, key) in keys.iter().enumerate() {
            if u(11 + k as u64) < 0.6 {
                let v = lg(20 + k as u64, 0.0, 16.0);
                p.materials.insert(key.to_string(), v);
                if let Some(s) = gp.supplies.get(*key) {
                    *sup.entry(s.clone()).or_insert(0.0) += v;
                }
            }
        }
        let organic = if u(30) < 0.5 { lg(31, 0.0, 15.0) } else { 0.0 };
        p.radiation_dose_msv_yr = Some(lg(19, -1.0, 7.0));
        let (c, h) = habitat_catalogue(&p, &bp, &gp, &t, &sup, organic);
        assert!(c.gradients.len() <= gp.cap);
        for v in [
            h.energy_cap_gc_yr,
            h.nutrient_cap_gc_yr,
            h.production_gc_yr,
            h.power_w,
            h.biomass_gc,
            h.secondary_gc_yr,
        ]
        .into_iter()
        .flatten()
        {
            assert!(v.is_finite() && v >= 0.0, "{i}: {h:?}");
        }
        if let (Some(prod), Some(e)) = (h.production_gc_yr, h.energy_cap_gc_yr) {
            assert!(prod <= e * (1.0 + 1e-12), "{i}");
        }
        if let (Some(prod), Some(n)) = (h.production_gc_yr, h.nutrient_cap_gc_yr) {
            assert!(prod <= n * (1.0 + 1e-12), "{i}");
        }
        // No gradient spends more of a species than the habitat has.
        let mut used: std::collections::BTreeMap<String, f64> = Default::default();
        for g in c.gradients.iter().filter(|g| g.open && g.kind != "light") {
            let r = t.reaction(&g.id).unwrap();
            for (s, nu) in r.terms.iter().filter(|(_, nu)| *nu < 0.0) {
                *used.entry(s.clone()).or_insert(0.0) += g.supply_mol_yr.unwrap() * -nu;
            }
            // Biomass built never beats the energy released.
            let built_j = g.production_gc_yr.unwrap() * budget_params().chemistry[0].build_j_per_g;
            assert!(
                built_j <= g.power_w.unwrap() * 3.155_76e7 * (1.0 + 1e-9),
                "{i}: {g:?}"
            );
            if !g.why.is_empty() {
                panic!("{i}: open with reasons {g:?}");
            }
            open += 1;
        }
        for (s, n) in used {
            let have = if s == gp.organic {
                organic
            } else {
                sup.get(&s).copied().unwrap_or(f64::INFINITY)
            };
            assert!(n <= have * (1.0 + 1e-9), "{i}: {s} {n} of {have}");
        }
    }
    assert!(open > 1000, "{open}");
}

#[test]
fn the_model_switch_picks_the_budget() {
    let g = graph("earth");
    let mut bp = budget_params();
    assert_eq!(bp.life_model, "listed/1");
    let listed = world_budget_by_model(&g, &bp, &gp(), &table(), &all_metabolisms());
    assert_eq!(
        listed,
        crate::budget::world_budget(&g, &bp, &all_metabolisms())
    );
    bp.life_model = MODEL.into();
    let grown = world_budget_by_model(&g, &bp, &gp(), &table(), &all_metabolisms());
    assert_eq!(grown, run(&g).1);
    assert_ne!(grown, listed);
}
