//! LE2: the energy ledger (life-from-energy design section 3).

use crate::budget::*;
use crate::ledger::*;
use crate::scenario::*;

fn all_ids(bp: &BudgetParams) -> Vec<String> {
    bp.metabolisms.iter().map(|m| m.id.clone()).collect()
}

/// captured <= offered <= arriving, wherever both sides are known.
fn ordered(l: &Lines) -> Result<(), String> {
    let tol = |x: f64| x * (1.0 + 1e-9) + 1e-12;
    if let (Some(c), Some(o)) = (l.captured_w, l.offered_w) {
        if c > tol(o) {
            return Err(format!("captured {c:e} > offered {o:e}"));
        }
    }
    if let (Some(o), Some(a)) = (l.offered_w, l.arriving_w) {
        if o > tol(a) {
            return Err(format!("offered {o:e} > arriving {a:e}"));
        }
    }
    Ok(())
}

#[test]
fn petela_matches_its_quoted_values() {
    let sun = petela(288.0, 5800.0);
    assert!((0.92..0.94).contains(&sun), "{sun}");
    let red = petela(288.0, 3000.0);
    assert!((0.86..0.88).contains(&red), "{red}");
    assert_eq!(petela(300.0, 300.0), 0.0);
}

#[test]
fn earth_ledger_sums_and_captures_about_130_tw() {
    let bp = budget_params();
    let g = graph("earth");
    let held = world_budget(&g, &bp, &all_ids(&bp));
    let l = ledger(&g, &bp, Some(&held));
    // Lines sum to the world.
    let s = |f: fn(&Lines) -> Option<f64>| -> f64 {
        l.habitats.iter().filter_map(|h| f(&h.lines)).sum()
    };
    assert_eq!(l.world.captured_w, Some(s(|x| x.captured_w)));
    assert_eq!(l.world.arriving_w, Some(s(|x| x.arriving_w)));
    assert_eq!(l.world.offered_w, Some(s(|x| x.offered_w)));
    assert_eq!(l.world.brought_in_w, 0.0);
    for h in &l.habitats {
        ordered(&h.lines).unwrap_or_else(|e| panic!("{}: {e}", h.kind));
        assert_eq!(h.lines.brought_in_w, 0.0);
    }
    // About 130 TW captured (Field et al. 1998 at about 40 kJ per g C),
    // within a factor of three.
    let c = l.world.captured_w.unwrap();
    assert!((130e12 / 3.0..130e12 * 3.0).contains(&c), "{c:e} W");
    // Life's share of the light arriving is about a thousandth or less.
    let a = l.world.arriving_w.unwrap();
    assert!(c / a < 0.01, "{c:e} of {a:e}");
    // Earth's surface habitats run a surplus.
    assert!(l.world.surplus_w.unwrap() > 0.0);
    assert_eq!(l.world, ledger(&g, &bp, Some(&held)).world, "deterministic");
    // The world comparison with absorbed sunlight waits on the climate.
    assert_eq!(l.absorbed_sunlight_w, None);
    assert!(!l.notes.is_empty());
}

#[test]
fn a_lifeless_world_captures_nothing_and_europa_has_no_surplus() {
    let bp = budget_params();
    let g = archean_earth();
    let l = ledger(&g, &bp, None);
    assert_eq!(l.world.captured_w, Some(0.0));
    assert!(l.world.offered_w.unwrap() > 0.0);
    let g = graph("europa");
    let held = world_budget(&g, &bp, &all_ids(&bp));
    let l = ledger(&g, &bp, Some(&held));
    for h in &l.habitats {
        assert!(h.barely, "{h:?}");
        assert!(h.lines.surplus_w.unwrap() < 0.0, "{h:?}");
    }
}

#[test]
fn ten_thousand_random_worlds_keep_the_order() {
    use sse_core::{Address, Draws, Seed, Stream};
    let bp = budget_params();
    let base = graph("earth");
    let d = Draws::new(
        Seed::from_text("le2"),
        Address::universe("le2").unwrap(),
        Stream::new("life.ledger.test.v1").unwrap(),
    );
    let ids = all_ids(&bp);
    let mut known = 0;
    for i in 0..10_000u64 {
        let u = |k: u64| d.unit(i * 32 + k);
        let lg = |k: u64, lo: f64, hi: f64| sse_core::math::pow(10.0, lo + (hi - lo) * u(k));
        let mut g = base.clone();
        g.habitats.truncate(2);
        for (j, h) in g.habitats.iter_mut().enumerate() {
            let o = 8 * j as u64;
            let p = &mut h.profile;
            let t = 150.0 + 300.0 * u(o);
            p.temperature_k = Some(crate::Range {
                lo: t,
                hi: t + 20.0 * u(o + 1),
            });
            p.energy.light_w_m2 = (u(o + 2) < 0.8).then(|| lg(o + 3, -3.0, 3.5));
            p.energy.light_peak_nm = Some(250.0 + 1500.0 * u(o + 4));
            p.energy.available = vec!["light".into(), "chemical".into()];
            p.area_m2 = (u(o + 5) < 0.9).then(|| lg(o + 6, 0.0, 15.0));
            for key in ["fe2_mol_yr", "h2_mol_yr", "o2_mol_yr", "reductant_mol_yr"] {
                p.materials.insert(key.into(), lg(o + 7, 0.0, 14.0));
            }
        }
        // Life holds a random part of what the pack lists.
        let held_ids: Vec<String> = ids
            .iter()
            .enumerate()
            .filter(|(k, _)| u(20 + *k as u64) < 0.5)
            .map(|(_, x)| x.clone())
            .collect();
        let held = world_budget(&g, &bp, &held_ids);
        let l = ledger(&g, &bp, Some(&held));
        for h in &l.habitats {
            ordered(&h.lines).unwrap_or_else(|e| panic!("{i} {}: {e}", h.kind));
            for v in [h.lines.arriving_w, h.lines.offered_w, h.lines.captured_w]
                .into_iter()
                .flatten()
            {
                assert!(v.is_finite() && v >= 0.0, "{i}: {h:?}");
            }
            if h.lines.arriving_w.is_some() && h.lines.captured_w.is_some() {
                known += 1;
            }
        }
        ordered(&l.world).unwrap_or_else(|e| panic!("{i} world: {e}"));
    }
    assert!(known > 5_000, "only {known} habitats fully known");
}
