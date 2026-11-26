use crate::describe::describe;
use crate::lifeline::Phase;
use crate::spectral::{classify, log_g, FAMILIES};

#[test]
fn the_sun_is_g2_v() {
    let c = classify(5772.0, 1.0, 1.0, Phase::MainSequence);
    assert_eq!(c.code, "G2 V");
    assert_eq!(c.family, "main-sequence");
    assert!((log_g(1.0, 1.0) - 4.438).abs() < 0.002);
    // Catalogued, with no phase: the same.
    assert_eq!(classify(5772.0, 1.0, 1.0, Phase::MainSequence).code, "G2 V");
}

#[test]
fn known_stars_read_their_measured_classes() {
    // (teff, mass, radius, phase, want): measured values, rounded. A measured
    // type and a temperature scale agree to about one subclass (stars of one
    // type spread over a hundred kelvin or so), so the subclass may be one off;
    // the letter and the luminosity class must match.
    let cases = [
        (3050.0, 0.122, 0.154, Phase::MainSequence, "M5 V"), // Proxima, M5.5 V
        (5231.0, 0.909, 0.863, Phase::MainSequence, "K1 V"), // Alpha Cen B, K1 V
        (9900.0, 2.06, 1.71, Phase::MainSequence, "A1 V"),   // Sirius A, A1 V
        (5772.0, 1.0, 1.0, Phase::MainSequence, "G2 V"),     // the Sun
        (3910.0, 1.16, 44.2, Phase::Giant, "K5 III"), // Aldebaran, K5 III (Arcturus is metal-poor, so earlier than its temperature)
        (12100.0, 21.0, 78.9, Phase::Giant, "B8 Ia"), // Rigel, B8 Ia
    ];
    for (t, mass, r, ph, want) in cases {
        let c = classify(t, mass, r, ph);
        let (wl, rest) = want.split_at(1);
        let (ws, wlum) = rest.split_once(' ').unwrap();
        let ws: i32 = ws.parse().unwrap();
        let ok = c.letter == wl
            && c.luminosity == wlum
            && (i32::from(c.subclass.unwrap()) - ws).abs() <= 1;
        assert!(
            ok,
            "{t} K {mass} Msun {r} Rsun: {} not {want}; {}",
            c.code, c.why
        );
    }
    // Betelgeuse: M1-2 Ia-ab; a red supergiant either way.
    let c = classify(3600.0, 18.0, 764.0, Phase::Giant);
    assert_eq!(c.letter, "M");
    assert_eq!(c.family, "supergiant", "{}", c.why);
}

#[test]
fn remnants_and_brown_dwarfs_have_honest_classes() {
    let wd = classify(25200.0, 0.6, 0.0125, Phase::WhiteDwarf);
    assert_eq!(wd.code, "DX 2");
    assert!(!wd.notes.is_empty(), "says why the letter is X");
    // Too cool for lines: DC, and the index stops at 10 above it.
    assert_eq!(
        classify(4000.0, 0.6, 0.0125, Phase::WhiteDwarf).code,
        "DC 10"
    );
    // No phase given, white-dwarf gravity: still a white dwarf.
    assert_eq!(
        classify(25200.0, 0.6, 0.0125, Phase::MainSequence).family,
        "white-dwarf"
    );
    assert_eq!(classify(1e6, 1.4, 1.4e-5, Phase::NeutronStar).code, "NS");
    assert_eq!(classify(0.0, 8.0, 0.0, Phase::BlackHole).code, "BH");
    assert_eq!(classify(0.0, 4.0e6, 0.0, Phase::BlackHole).code, "SMBH");
    let t = classify(1100.0, 0.05, 0.1, Phase::BrownDwarf);
    assert_eq!((t.letter.as_str(), t.family), ("T", "brown-dwarf"));
    assert_eq!(classify(1800.0, 0.07, 0.1, Phase::BrownDwarf).letter, "L");
    assert_eq!(classify(400.0, 0.02, 0.1, Phase::BrownDwarf).letter, "Y");
}

/// A star's class follows it through its whole life: the Sun's life line
/// read at every age gives a class at each, in an order that makes sense.
#[test]
fn a_class_follows_a_star_through_its_life() {
    let mut seen: Vec<&str> = vec![];
    for k in 0..400 {
        let age = 1.0e5 * sse_core::math::pow(1.05, f64::from(k));
        if age > 2.0e10 {
            break;
        }
        let d = describe(1.0, age, 0.0).unwrap();
        let n = &d.now;
        let c = classify(n.teff_k, n.mass_msun, n.radius_rsun, n.phase);
        assert!(FAMILIES.contains(&c.family));
        assert!(!c.code.is_empty() && !c.why.is_empty());
        if seen.last() != Some(&c.family) {
            seen.push(c.family);
        }
        if (4.0e9..5.0e9).contains(&age) {
            assert_eq!(c.letter, "G", "the Sun at {age} yr: {}", c.why);
            assert_eq!(c.luminosity, "V");
        }
    }
    assert_eq!(seen.first(), Some(&"pre-main-sequence"));
    assert!(
        seen.contains(&"main-sequence") && seen.contains(&"giant"),
        "{seen:?}"
    );
    assert_eq!(seen.last(), Some(&"white-dwarf"), "{seen:?}");
}

#[test]
fn every_family_is_reached_by_some_star() {
    let mut got = std::collections::BTreeSet::new();
    for &mass in &[0.03, 0.3, 1.0, 3.0, 15.0, 40.0] {
        for k in 0..300 {
            let age = 1.0e4 * sse_core::math::pow(1.08, f64::from(k));
            if age > 1.4e10 {
                break;
            }
            let n = describe(mass, age, 0.0).unwrap().now;
            got.insert(classify(n.teff_k, n.mass_msun, n.radius_rsun, n.phase).family);
        }
    }
    for f in FAMILIES {
        if f == "bright-giant" {
            continue; // reached or not depending on how the sketch giant phase passes through it
        }
        assert!(got.contains(f), "no star reaches {f}: {got:?}");
    }
}

/// The tables and the life line's samples per family and kind, read by
/// scripts/build-base-atlas.py. `SSE_BLESS=1` rewrites it; then run the
/// script.
#[test]
fn star_classes_golden() {
    let got = serde_json::json!({
        "about": "Star classes (sse_formation::spectral): the temperature scales, the luminosity-class gravity loci, and one life-line sample for each family and kind reached. Written by the test star_classes_golden (SSE_BLESS=1); read by scripts/build-base-atlas.py.",
        "tables": crate::spectral::tables(),
        "samples": crate::spectral::samples(),
    });
    crate::golden::check("star-classes.json", &got);
}
