//! CO1 evidence: one rule for every world gives the Solar System's Trojans
//! (which planets keep long-lived swarms, and Neptune's against Jupiter's).
use crate::coorbital::*;

/// Sun's mass in Earth masses, and the planets (orbit size AU, mass in Earth
/// masses), round figures for a test only.
const SUN: f64 = 332_946.0;
const PLANETS: [(&str, f64, f64, bool); 8] = [
    ("mercury", 0.387, 0.0553, false),
    ("venus", 0.723, 0.815, false),
    ("earth", 1.0, 1.0, false),
    ("mars", 1.524, 0.107, false),
    ("jupiter", 5.203, 317.8, true),
    ("saturn", 9.537, 95.16, true),
    ("uranus", 19.19, 14.54, true),
    ("neptune", 30.07, 17.15, true),
];

fn sol() -> Vec<Orbit<'static>> {
    PLANETS
        .iter()
        .map(|&(id, a_au, mass, giant)| Orbit {
            id,
            a_au,
            mass,
            giant,
        })
        .collect()
}

fn index(id: &str) -> usize {
    PLANETS.iter().position(|p| p.0 == id).unwrap()
}

#[test]
fn every_planet_of_ours_can_hold_trojans_but_a_heavy_companion_cannot() {
    for w in sol() {
        assert!(zone(w.mass, SUN).stable, "{}", w.id);
    }
    // A brown dwarf a tenth of its star's mass: its L4 and L5 do not hold.
    assert!(!zone(0.1, 1.0).stable);
    // The limit sits at 27 mu (1 - mu) = 1, about 1 part in 26.
    assert!(zone(0.0384, 1.0 - 0.0384).stable);
    assert!(!zone(0.0386, 1.0 - 0.0386).stable);
}

#[test]
fn the_swarm_is_as_wide_as_the_tadpole_zone() {
    // Dermott and Murray 1981: (8 mu / 3)^(1/2); Jupiter's about 0.05.
    let j = zone(317.8, SUN);
    assert!((j.width - 0.0505).abs() < 0.001, "{}", j.width);
    assert!(zone(17.15, SUN).width < j.width / 4.0);
}

#[test]
fn neighbours_decide_who_keeps_them() {
    let s = sol();
    let kept: Vec<&str> = (0..s.len())
        .filter(|&i| disturbed(&s, i).is_none())
        .map(|i| s[i].id)
        .collect();
    // Jupiter and Neptune hold long-lived swarms, Saturn and Uranus do not
    // (Lykawka and Horner 2010); Mars keeps primordial ones (de la Fuente
    // Marcos et al. 2024); Venus's and Mercury's do not last.
    assert_eq!(kept, ["earth", "mars", "jupiter", "neptune"]);
    let sat = disturbed(&s, index("saturn")).unwrap();
    assert_eq!((sat.by.as_str(), sat.p, sat.q), ("jupiter", 5, 2));
    let ura = disturbed(&s, index("uranus")).unwrap();
    assert_eq!((ura.by.as_str(), ura.p, ura.q), ("neptune", 2, 1));
}

#[test]
fn the_world_nearest_the_belt_catches_most() {
    // A belt beyond Neptune, as in the Nice migration: the four catch
    // shares inside Lykawka and Horner 2010's ranges (Saturn just above).
    let s = sol();
    let catch = |id: &str| {
        let i = index(id);
        capture_share(zone(s[i].mass, SUN).width, giants_between(&s, i, 40.0))
    };
    let (j, sa, u, n) = (
        catch("jupiter"),
        catch("saturn"),
        catch("uranus"),
        catch("neptune"),
    );
    assert!((1e-4..1e-3).contains(&n), "neptune {n}");
    assert!((1e-5..1e-4).contains(&u), "uranus {u}");
    assert!((1e-6..1.5e-5).contains(&sa), "saturn {sa}");
    assert!((1e-6..1e-5).contains(&j), "jupiter {j}");
    // Rocky worlds deep inside four giants catch next to nothing.
    assert!(catch("earth") < 1e-7);
}

#[test]
fn neptune_keeps_several_times_jupiters_swarm() {
    // Schwamb et al. 2026: Neptune's long-lived Trojans are 4 to 10 times
    // Jupiter's; Nesvorny et al. 2013: Jupiter keeps about 7e-7 of the disc.
    let s = sol();
    let kept = |id: &str| {
        let i = index(id);
        let z = zone(s[i].mass, SUN);
        capture_share(z.width, giants_between(&s, i, 40.0))
            * long_lived_share(z.mu, disturbed(&s, i).is_some())
    };
    let (j, n) = (kept("jupiter"), kept("neptune"));
    assert!((6e-7..8e-7).contains(&j), "jupiter keeps {j}");
    assert!(
        (4.0..10.0).contains(&(n / j)),
        "neptune / jupiter {}",
        n / j
    );
    assert_eq!(kept("saturn"), 0.0);
    assert_eq!(kept("uranus"), 0.0);
}

#[test]
fn resonances_are_found_in_lowest_terms() {
    assert_eq!(nearest_resonance(1.5).0, 3);
    assert_eq!(nearest_resonance(1.5).1, 2);
    assert_eq!(
        (nearest_resonance(2.484).0, nearest_resonance(2.484).1),
        (5, 2)
    );
}
