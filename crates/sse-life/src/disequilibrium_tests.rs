//! E8: available Gibbs energy of a world's air.

use crate::chemistry::ThermoTable;
use crate::disequilibrium::*;
use crate::scenario::base_life_pack;
use std::collections::BTreeMap;

fn table() -> ThermoTable {
    ThermoTable::from_packs(&[base_life_pack()]).expect("life chemistry")
}

fn air(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// Today's air, dry mole fractions (NOAA and IPCC AR6 round values).
fn earth_today() -> BTreeMap<String, f64> {
    air(&[
        ("n2", 0.7808),
        ("o2", 0.2095),
        ("co2", 4.2e-4),
        ("ch4", 1.9e-6),
        ("co", 1.0e-7),
        ("h2", 5.3e-7),
    ])
}

#[test]
fn earth_today_is_held_out_of_balance_by_methane_with_oxygen() {
    let d = disequilibrium(
        &table(),
        &DiseqParams::default(),
        &earth_today(),
        288.15,
        1.013,
        true,
    )
    .unwrap();
    // Krissansen-Totton et al. 2016: 1.51 J/mol for the atmosphere alone
    // (theirs counts water vapour and N2O, so agreement within a factor of
    // two is the check).
    assert!(d.available_j_mol > 0.75 && d.available_j_mol < 3.0, "{d:?}");
    assert_eq!(d.by_reaction[0].0, "methane-oxidation", "{d:?}");
    assert!(d.out_of_balance);
    // At equilibrium the methane is gone (to within the search's
    // tolerance: under a hundred-thousandth of what there was).
    assert!(d.equilibrium["ch4"] < 1.9e-11, "{d:?}");
}

#[test]
fn before_oxygen_a_different_pair_carries_it() {
    // An Archean air: CO2 and CH4 high, H2 from volcanoes, no O2
    // (Catling and Zahnle 2020 round values).
    let archean = air(&[
        ("n2", 0.9),
        ("co2", 0.05),
        ("ch4", 2.0e-3),
        ("h2", 1.0e-3),
        ("co", 1.0e-4),
        ("o2", 1.0e-12),
    ]);
    let d = disequilibrium(
        &table(),
        &DiseqParams::default(),
        &archean,
        288.15,
        1.0,
        true,
    )
    .unwrap();
    assert!(d.out_of_balance, "{d:?}");
    let top = &d.by_reaction[0].0;
    assert!(top != "methane-oxidation", "{d:?}");
    assert!(
        top == "methanogenesis" || top == "water-gas-shift" || top == "co-oxidation",
        "{d:?}"
    );
}

#[test]
fn abiotic_oxygen_without_a_reductant_reads_as_equilibrium() {
    // A water-loss world round a red dwarf: lots of O2 and CO2, nothing that
    // burns. High O2 alone looks like life to a single-gas test; the
    // disequilibrium shows it is not held open by anything.
    let lost = air(&[("o2", 0.5), ("co2", 0.3), ("n2", 0.2)]);
    let d = disequilibrium(&table(), &DiseqParams::default(), &lost, 300.0, 5.0, false).unwrap();
    assert!(!d.out_of_balance, "{d:?}");
    assert!(d.available_j_mol < 1e-6, "{d:?}");
}

#[test]
fn the_answer_does_not_depend_on_the_order_of_the_basis() {
    let t = table();
    let mut p = DiseqParams::default();
    let a = disequilibrium(&t, &p, &earth_today(), 288.15, 1.013, true).unwrap();
    p.reactions.reverse();
    p.extra.reverse();
    let b = disequilibrium(&t, &p, &earth_today(), 288.15, 1.013, true).unwrap();
    assert!(
        (a.available_j_mol - b.available_j_mol).abs() < 1e-5 * a.available_j_mol,
        "{a:?} {b:?}"
    );
}

/// Box I: with the air measured, the red-dwarf water-loss world stops being
/// a candidate (its oxygen is at equilibrium), and Earth stays one.
#[test]
fn the_measured_air_settles_the_false_positive() {
    use crate::scenario::signature_cases;
    use crate::signatures::{signatures, SignatureParams};
    let sp = SignatureParams::default();
    let dp = DiseqParams::default();
    let t = table();
    let case = |name: &str| {
        let (_, b, h, o) = signature_cases().into_iter().find(|c| c.0 == name).unwrap();
        signatures(b.as_ref(), h.as_ref(), &o, &sp)
    };
    let mut fp = case("red-dwarf-desert");
    assert!(fp.candidate, "the single-gas scan is fooled");
    let lost = air(&[("o2", 0.5), ("co2", 0.3), ("n2", 0.2)]);
    let d = disequilibrium(&t, &dp, &lost, 300.0, 5.0, false).unwrap();
    refine(&mut fp, &d, &dp, &sp);
    assert!(!fp.candidate && fp.confidence < 0.5, "{fp:?}");
    assert_eq!(fp.signals[0].id, "disequilibrium");
    let mut earth = case("earth");
    let d = disequilibrium(&t, &dp, &earth_today(), 288.15, 1.013, true).unwrap();
    refine(&mut earth, &d, &dp, &sp);
    assert!(earth.candidate && earth.confidence > 0.9, "{earth:?}");
}
