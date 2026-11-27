//! F5 evidence: reservoirs conserve the budgets; a resolved object is a pure
//! function of the seed and its address.
use crate::growth_tests::one;
use crate::leftovers::*;
use crate::scenario::system;
use crate::violent::upheave;
use sse_core::Seed;

fn seed() -> Seed {
    Seed::from_text("f3-population")
}

fn run(n: u64, mass: f64, feh: f64) -> Leftovers {
    let (disc, g) = one(n, mass, feh);
    let u = upheave(&seed(), &system(n), &disc, &g);
    leftovers(&seed(), &system(n), &disc, &u)
}

fn water(w: &crate::violent::WaterFrom) -> u64 {
    w.iter().map(|x| x.1).sum()
}

#[test]
fn reservoirs_conserve_the_budgets() {
    for n in 0..300 {
        let l = run(n, 1.0, if n % 2 == 0 { 0.0 } else { 0.3 });
        assert!(l.budget.closes(), "system {n}: {:?}", l.budget);
        for w in &l.worlds {
            assert_eq!(w.ledger.solid(), w.solid, "system {n}: {}", w.id);
            let mut by = vec![];
            w.ledger.water_by_band(&mut by);
            assert_eq!(by.iter().map(|x| x.1).sum::<i128>(), w.solid[3] as i128);
        }
        for c in &l.clouds {
            assert_eq!(water(&c.water_from), c.solid[3], "system {n}: {}", c.kind);
            let total: u64 = c.solid.iter().sum();
            for k in [0u64, 1, 10, 1000] {
                let (parts, rest) = c.split(k);
                assert_eq!(parts.iter().sum::<u64>() + rest, total);
                assert!(parts.windows(2).all(|p| p[0] >= p[1]));
            }
        }
        for mo in &l.moons {
            assert_eq!(water(&mo.water_from), mo.solid[3]);
        }
    }
}

#[test]
fn a_resolved_object_is_the_same_whenever_it_is_asked_for() {
    let s = seed();
    let mut checked = 0;
    for n in 0..60 {
        let l = run(n, 1.0, 0.3);
        for c in &l.clouds {
            let forward: Vec<SmallBody> = (1..=20).map(|k| c.resolve(&s, k)).collect();
            let backward: Vec<SmallBody> = (1..=20).rev().map(|k| c.resolve(&s, k)).collect();
            for (k, b) in forward.iter().enumerate() {
                assert_eq!(*b, backward[19 - k]);
                assert_eq!(b.rank, k as u64 + 1);
                assert!(b.a_au >= c.a_min_au && b.a_au <= c.a_max_au);
                assert!((0.0..1.0).contains(&b.e));
            }
            assert!(forward
                .windows(2)
                .all(|p| p[0].diameter_km >= p[1].diameter_km));
            assert!(forward[0].diameter_km <= SIZE_CAP_KM * 1.000_001);
            assert_ne!(forward[0].address, forward[1].address);
            // Regenerating the whole system gives the same cloud and so the same object.
            let again = run(n, 1.0, 0.3);
            let c2 = again
                .clouds
                .iter()
                .find(|x| x.address == c.address)
                .unwrap();
            assert_eq!(c2.resolve(&s, 7), c.resolve(&s, 7));
            checked += 1;
        }
    }
    assert!(checked > 50, "only {checked} clouds");
}

#[test]
fn population_of_reservoirs() {
    let (mut belt, mut outer, mut oort, mut trojans, mut giants, mut rings) = (0, 0, 0, 0, 0, 0);
    let mut moons_per_giant = vec![];
    for n in 0..1000 {
        let l = run(n, 1.0, 0.0);
        let has = |k: &str| l.clouds.iter().any(|c| c.kind == k);
        belt += has("asteroid-belt") as u32;
        outer += has("outer-belt") as u32;
        oort += has("oort") as u32;
        trojans += has("trojans") as u32;
        for w in l.worlds.iter().filter(|w| w.is_giant()) {
            giants += 1;
            let ms: Vec<&Moon> = l
                .moons
                .iter()
                .filter(|mo| mo.parent == w.id && mo.origin == "regular")
                .collect();
            moons_per_giant.push(ms.len());
            let share = ms
                .iter()
                .map(|mo| mo.solid.iter().sum::<u64>())
                .sum::<u64>() as f64
                / (w.solid.iter().sum::<u64>() + w.gas) as f64;
            assert!(
                (0.5e-4..2.0e-4).contains(&share),
                "satellite share {share} of {}",
                w.id
            );
            if l.clouds
                .iter()
                .any(|c| c.kind == "ring" && c.around.as_ref() == Some(&w.id))
            {
                rings += 1;
            }
        }
    }
    eprintln!(
        "F5 population (1000 Sun-like): asteroid belts {belt}, outer belts {outer}, Oort clouds {oort}, \
         Trojan swarms {trojans}; giants {giants}, ringed {rings}; regular moons per giant {:.2}",
        moons_per_giant.iter().sum::<usize>() as f64 / moons_per_giant.len().max(1) as f64
    );
    assert!(moons_per_giant.iter().all(|n| (2..=5).contains(n)));
    assert!(oort > 0 && trojans > 0 && outer > 0);
    // Every Oort cloud comes from a stirred belt, and only giants stir belts.
    assert!(oort <= giants);
}

#[test]
fn golden() {
    crate::golden::check(
        "formation-leftovers.json",
        &crate::scenario::leftovers_golden(),
    );
}

/// CO1 diagnostics: who keeps Trojans across many Sun-like systems.
#[test]
#[ignore]
fn coorbital_census() {
    use crate::violent::UpheavalEvent;
    let (mut giants_kept, mut giants_lost, mut rocky_kept, mut rocky_caught) = (0, 0, 0, 0);
    let mut disturbed = 0;
    let mut ratios = vec![];
    for n in 0..300 {
        let (disc, g) = one(n, 1.0, 0.0);
        let u = upheave(&seed(), &system(n), &disc, &g);
        let mut kept_by = vec![];
        for e in &u.events {
            if let UpheavalEvent::TrojansThinned {
                world,
                kept_earth,
                lost_earth,
                disturbed_by,
                resonance,
                ..
            } = e
            {
                let w = u.worlds.iter().find(|w| &w.id == world);
                let giant = w.is_some_and(|w| w.is_giant());
                if disturbed_by.is_some() {
                    disturbed += 1;
                }
                if giant {
                    if *kept_earth > 0.0 {
                        giants_kept += 1
                    } else {
                        giants_lost += 1
                    }
                } else {
                    rocky_caught += 1;
                    if *kept_earth > 0.0 {
                        rocky_kept += 1
                    }
                }
                if n < 12 {
                    eprintln!("sys {n} {world} giant {giant} a {:.2} m {:.1} kept {kept_earth:.2e} lost {lost_earth:.2e} by {disturbed_by:?} {resonance:?}",
                        w.map_or(0.0, |w| w.a_au), w.map_or(0.0, |w| w.mass_earth()));
                }
                if giant && *kept_earth > 0.0 {
                    kept_by.push((w.unwrap().a_au, *kept_earth));
                }
            }
        }
        if kept_by.len() >= 2 {
            let outer = kept_by
                .iter()
                .cloned()
                .fold((0.0, 0.0), |a, b| if b.0 > a.0 { b } else { a });
            let inner = kept_by
                .iter()
                .cloned()
                .fold((1e9, 0.0), |a, b| if b.0 < a.0 { b } else { a });
            ratios.push(outer.1 / inner.1);
        }
    }
    eprintln!("giants kept {giants_kept} lost {giants_lost}; rocky caught {rocky_caught} kept {rocky_kept}; disturbed {disturbed}; outer/inner kept ratios {ratios:?}");
}
