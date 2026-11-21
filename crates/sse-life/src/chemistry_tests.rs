//! E0: chemistries as data, the thermodynamic table and reaction energetics
//! (evolution design section 14).

use crate::budget::{world_budget, BudgetParams};
use crate::chemistry::*;
use crate::scenario::*;
use std::collections::BTreeMap;

fn table() -> ThermoTable {
    ThermoTable::from_packs(&[base_life_pack()]).expect("life chemistry")
}

fn at_published(t: &ThermoTable, id: &str) -> Energetics {
    let r = t.reaction(id).expect("reaction");
    t.energetics(r, 298.15, &t.published_activities(r))
        .expect("energetics")
}

#[test]
fn every_reference_reaction_balances_and_matches_its_published_value() {
    let t = table();
    assert!(t.reactions.len() >= 12, "{}", t.reactions.len());
    for r in &t.reactions {
        t.balance(r).unwrap();
        let src = r.source.as_deref().expect("every reaction is sourced");
        assert!(t.sources.contains_key(src), "{}: {src}", r.id);
        let want = r.published_kj.expect("published value") * 1000.0;
        let got = at_published(&t, &r.id).gibbs_j;
        let off = ((got - want) / want).abs();
        assert!(
            off <= 0.05,
            "{}: {got} J vs published {want} J ({off:.3})",
            r.id
        );
    }
    // A reaction that does not balance is refused when the pack loads.
    let mut bad = t.clone();
    bad.reactions[0].terms[0].1 = -2.0;
    assert!(bad.check().unwrap_err().contains("does not balance"));
}

#[test]
fn the_redox_tower_comes_out_in_order() {
    // Per electron from hydrogen at pH 7: oxygen, then nitrate, then
    // sulfate, then carbon dioxide (Thauer et al. 1977's tower).
    let t = table();
    let e = |id: &str| at_published(&t, id).per_electron_j.expect("redox");
    let ladder = [
        e("knallgas"),
        e("denitrification"),
        e("sulfate-reduction"),
        e("methanogenesis-bicarbonate"),
    ];
    for w in ladder.windows(2) {
        assert!(w[0] < w[1], "{ladder:?}");
    }
    // All four still clear the minimum quantum per turnover only at high
    // concentrations for the last two: a fact E1's first test will use.
    assert!(ladder[3] > -20.0e3, "{ladder:?}");
}

#[test]
fn local_conditions_move_the_energy() {
    let t = table();
    let r = t.reaction("methanogenesis").unwrap();
    let cool = t.energetics(r, 298.15, &BTreeMap::new()).unwrap();
    let hot = t.energetics(r, 358.15, &BTreeMap::new()).unwrap();
    // Five gases in, one out: entropy falls, so warming costs energy.
    assert!(hot.gibbs_j > cool.gibbs_j, "{hot:?} {cool:?}");
    // Scarce hydrogen (1e-5 bar, as in sediments) nearly shuts it off.
    let lean: BTreeMap<String, f64> = [("h2".to_string(), 1.0e-5)].into_iter().collect();
    let starved = t.energetics(r, 298.15, &lean).unwrap();
    assert!(starved.gibbs_j > -20.0e3, "{starved:?}");
    assert!(starved.gibbs_j < cool.gibbs_j + 115.0e3);
    // A missing activity is an error, not a guess.
    let zero: BTreeMap<String, f64> = [("h2".to_string(), 0.0)].into_iter().collect();
    assert!(t.energetics(r, 298.15, &zero).is_err());
}

fn legacy(bp: &BudgetParams) -> BudgetParams {
    let c = &bp.chemistry[0];
    let mut old = bp.clone();
    old.chemistry = vec![];
    old.chemistries = vec![c.solvent.clone()];
    old.window_k = Some(c.window_k);
    old.water_triple_pa = c.liquid_min_pa;
    old.min_water_activity = c.min_activity;
    old
}

#[test]
fn carbon_water_closures_are_unchanged() {
    let bp = budget_params();
    assert_eq!(bp.chemistry.len(), 1);
    let c = &bp.chemistry[0];
    assert_eq!(
        (c.id.as_str(), c.evidence.as_str()),
        ("carbon-water", "observed")
    );
    let old = legacy(&bp);
    let all = all_metabolisms();
    for (n, _) in WORLDS {
        let g = graph(n);
        assert_eq!(
            world_budget(&g, &bp, &all),
            world_budget(&g, &old, &all),
            "{n}"
        );
    }
    let g = archean_earth();
    assert_eq!(
        world_budget(&g, &bp, &archean_metabolisms()),
        world_budget(&g, &old, &archean_metabolisms())
    );
    // Each closure test, one at a time, with the same words as before.
    let base = graph("earth").habitats[0].profile.clone();
    let mut cases = vec![];
    let mut p = base.clone();
    p.solvent = "ammonia".into();
    cases.push(p);
    let mut p = base.clone();
    p.temperature_k = Some(crate::Range {
        lo: 400.0,
        hi: 420.0,
    });
    cases.push(p);
    let mut p = base.clone();
    p.pressure_pa = Some(crate::Range {
        lo: 100.0,
        hi: 200.0,
    });
    cases.push(p);
    let mut p = base.clone();
    p.properties.insert("water_activity".into(), 0.3);
    cases.push(p);
    let mut p = base.clone();
    p.radiation_dose_msv_yr = Some(1.0e8);
    cases.push(p);
    let want = [
        "no chemistry for ammonia in this pack stack",
        "400 to 420 K is outside the 253 to 395 K window",
        "at 200 Pa water cannot be liquid (triple point 612 Pa)",
        "water activity 0.3 is below the 0.585 limit",
        "a dose of 100000000 mSv/yr is above the 1000000 limit",
    ];
    for (p, w) in cases.iter().zip(want) {
        let now = crate::budget::closed_reason(p, &bp);
        assert_eq!(now.as_deref(), Some(w));
        assert_eq!(now, crate::budget::closed_reason(p, &old));
    }
}

#[test]
fn no_code_names_a_solvent() {
    for (name, src) in [
        ("budget.rs", include_str!("budget.rs")),
        ("biosphere.rs", include_str!("biosphere.rs")),
        ("history.rs", include_str!("history.rs")),
        ("feedback.rs", include_str!("feedback.rs")),
    ] {
        assert!(!src.contains("== \"water\""), "{name}");
        assert!(!src.contains("\"water\".into()"), "{name}");
    }
}

#[test]
fn a_chemistry_needs_an_honest_tag() {
    let mut c = budget_params().chemistry[0].clone();
    c.check().unwrap();
    c.evidence = "certain".into();
    assert!(c.check().unwrap_err().contains("evidence"));
}
