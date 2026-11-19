//! F7: close binaries by rule table.
use crate::binary::*;

fn settings() -> crate::star::StarSettings {
    crate::scenario::settings().star
}

fn side(addr: &str, mass: f64) -> (String, f64) {
    (addr.into(), mass)
}

fn pair(m1: f64, m2: f64, a_au: f64, e: f64) -> Option<Interaction> {
    let rules = BinaryRules::default();
    pair_contact(
        side("u:t/sys:0/star:0", m1),
        side("u:t/sys:0/star:1", m2),
        a_au,
        e,
        0.02,
        &settings(),
        &rules,
    )
}

#[test]
fn roche_lobe_matches_eggleton() {
    // Eggleton 1983: r_L/a = 0.3789 at q = 1, about 0.2 at q = 0.1 (his table, 1 percent).
    assert!((roche_fraction(1.0) - 0.3789).abs() < 0.001);
    assert!((roche_fraction(0.1) - 0.2068).abs() < 0.002);
    assert!(roche_fraction(10.0) > 0.57 && roche_fraction(10.0) < 0.58);
}

/// A Sun-like star with a 0.8 solar-mass companion 1 au away fills its lobe
/// on the giant branch; the mass ratio is above the giant critical value, so a
/// common envelope leaves a white dwarf and the companion a few solar radii
/// apart: a post-common-envelope binary with a period of days (observed
/// periods 0.1 to 10 days, Nebot Gomez-Moran et al. 2011).
#[test]
fn giant_donor_makes_a_common_envelope() {
    let i = pair(1.0, 0.8, 1.0, 0.0).expect("contact");
    assert_eq!(i.outcome, "common-envelope", "{}", i.event.why);
    assert_eq!(i.donor, "u:t/sys:0/star:0");
    let a = i.a_after_au.unwrap();
    let p_days = 365.25 * sse_core::math::sqrt(a * a * a / (i.event.numbers["core_mass"] + 0.8));
    assert!(
        (0.1..10.0).contains(&p_days),
        "post-CE period {p_days} days"
    );
    assert!(i.event.why.contains("common envelope"));
}

/// Wide pairs never touch.
#[test]
fn wide_pairs_do_not_interact() {
    assert!(pair(1.0, 0.8, 100.0, 0.0).is_none());
    assert!(pair(0.3, 0.2, 0.5, 0.0).is_none(), "red dwarfs never swell");
}

/// Algol-like: a 2 solar-mass star with a 1.5 solar-mass companion 0.05 au
/// away fills its lobe as it leaves the main sequence; q = 1.33 is below the
/// critical 3 to 4, so the transfer is stable (HTP02).
#[test]
fn close_pair_with_similar_masses_transfers_stably() {
    let i = pair(2.0, 1.5, 0.05, 0.0).expect("contact");
    assert_eq!(i.donor, "u:t/sys:0/star:0");
    assert_eq!(i.outcome, "stable-mass-transfer", "{}", i.event.why);
    let a = i.a_after_au.unwrap();
    assert!(a.is_finite() && a > 0.0);
}

/// Contact on the main sequence with a lopsided pair: merger.
#[test]
fn lopsided_close_pair_merges() {
    let i = pair(1.5, 0.15, 0.012, 0.0).expect("contact");
    assert_eq!(i.outcome, "merger", "{}", i.event.why);
}

#[test]
fn deterministic() {
    let a = serde_json::to_string(&pair(1.0, 0.8, 1.0, 0.1)).unwrap();
    assert_eq!(a, serde_json::to_string(&pair(1.0, 0.8, 1.0, 0.1)).unwrap());
}

/// Over the G0 solar patch: every close pair is checked, outcomes counted.
#[test]
fn patch_binaries() {
    let rules = BinaryRules::default();
    let star = settings();
    let mut pairs = 0;
    let mut outcomes = std::collections::BTreeMap::<String, usize>::new();
    for h in sse_formation::scenario::solar_patch().into_iter().flatten() {
        pairs += h.stars.iter().filter(|s| s.orbit.is_some()).count();
        let a = interactions(&h, &star, &rules);
        assert_eq!(a, interactions(&h, &star, &rules));
        for i in a {
            assert!(i.age_myr > 0.0 && i.event.why.len() > 40);
            *outcomes.entry(i.outcome).or_default() += 1;
        }
    }
    eprintln!("F7 binaries: {pairs} pairs, {outcomes:?}");
    assert!(pairs > 0);
}
