//! L3-heat: a chart world's history takes the interior's heat at each age.
use crate::world_life::{epochs_by_heat, HeatSample};

fn s(at_myr: f64, heat_flow_w_m2: f64) -> HeatSample {
    HeatSample {
        at_myr,
        heat_flow_w_m2,
    }
}

/// Earth-like: hot young, cooling (W/m² by Myr).
fn cooling() -> Vec<HeatSample> {
    [
        (0.0, 0.40),
        (500.0, 0.30),
        (1000.0, 0.26),
        (1500.0, 0.20),
        (2000.0, 0.17),
        (2500.0, 0.14),
        (3000.0, 0.12),
        (3500.0, 0.10),
        (4000.0, 0.09),
        (4500.0, 0.08),
    ]
    .iter()
    .map(|(a, q)| s(*a, *q))
    .collect()
}

#[test]
fn no_history_holds_todays_heat() {
    assert_eq!(
        epochs_by_heat(&[], 100.0, 4500.0, 0.087, 0.2, 8),
        vec![(100.0, 0.087)]
    );
}

#[test]
fn each_epoch_takes_the_heat_at_its_own_start() {
    let e = epochs_by_heat(&cooling(), 700.0, 4500.0, 0.087, 0.2, 8);
    // The first epoch starts when the world is habitable, at the last
    // sample before it, never a later one.
    assert_eq!(e[0], (700.0, 0.30));
    for (from, q) in &e[1..] {
        let at = cooling().into_iter().find(|x| x.at_myr == *from).unwrap();
        assert_eq!(*q, at.heat_flow_w_m2);
    }
    // Starts rise, and the young heat is several times today's.
    assert!(e.windows(2).all(|w| w[0].0 < w[1].0));
    assert!(e[0].1 > 3.0 * 0.087);
}

#[test]
fn small_changes_merge_into_one_epoch() {
    let e = epochs_by_heat(&cooling(), 0.0, 4600.0, 0.08, 0.2, 8);
    // Neighbours differ by more than a fifth of the epoch's heat.
    for w in e.windows(2) {
        assert!((w[1].1 - w[0].1).abs() > 0.2 * w[0].1, "{e:?}");
    }
    assert!(e.len() < cooling().len(), "{e:?}");
    assert!(e.len() >= 4, "{e:?}");
}

#[test]
fn the_cap_widens_the_step() {
    let mut q = 0.5;
    let many: Vec<HeatSample> = (0..100)
        .map(|i| {
            let x = s(i as f64 * 45.0, q);
            q *= 0.97;
            x
        })
        .collect();
    let e = epochs_by_heat(&many, 0.0, 4500.0, 0.03, 0.05, 3);
    assert!(e.len() <= 3 && e.len() >= 2, "{e:?}");
    assert_eq!(e[0], (0.0, 0.5));
}

#[test]
fn samples_after_today_are_not_read() {
    let mut h = cooling();
    h.push(s(9000.0, 0.01));
    let e = epochs_by_heat(&h, 0.0, 4500.0, 0.08, 0.2, 8);
    assert!(e.iter().all(|(from, _)| *from < 4500.0), "{e:?}");
}
