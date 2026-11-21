//! E5, functional traits (evolution design section 6).

use crate::traits::*;
use sse_core::draw::{Draws, Stream};
use sse_core::{Address, Seed};

fn earthlike() -> Conditions {
    Conditions {
        temperature_k: 288.0,
        swing_k: 15.0,
        water_balance: 0.2,
        gravity_m_s2: 9.81,
        nutrient_rel: 1.0,
        disturbance_rel: 1.0,
        opportunity: 1.0,
    }
}

fn habitat(n: u32) -> Address {
    format!("u:traits/sys:0/body:{n}/habitat:@land-surface")
        .parse()
        .unwrap()
}

fn settle(p: &TraitParams, gs: &mut [Guild], c: &Conditions) {
    for i in 0..200 {
        step(p, gs, c, i as f64 * 0.1, 0.1);
    }
}

/// Across 10,000 random habitats and abrupt changes, no trait ever leaves
/// the limits physics sets for it.
#[test]
fn limits_never_broken_across_10000_worlds() {
    let p = TraitParams {
        candidates: 3,
        ..TraitParams::default()
    };
    let seed = Seed::from_text("e5-limits");
    let draws = Draws::new(seed, habitat(0), Stream::new("test.e5.worlds.v1").unwrap());
    let u = |i: u64| draws.unit(i);
    let world = |i: u64| Conditions {
        temperature_k: 200.0 + 250.0 * u(i * 8),
        swing_k: 80.0 * u(i * 8 + 1),
        water_balance: -1.0 + 2.0 * u(i * 8 + 2),
        gravity_m_s2: 0.5 + 30.0 * u(i * 8 + 3),
        nutrient_rel: 5.0 * u(i * 8 + 4),
        disturbance_rel: 5.0 * u(i * 8 + 5),
        opportunity: sse_core::math::pow(10.0, -3.0 + 5.0 * u(i * 8 + 6)),
    };
    for w in 0..10_000u64 {
        let c0 = world(w);
        let mut gs = seed_guilds(&p, &c0, &habitat(w as u32), &seed).unwrap();
        for k in 0..4u64 {
            let c = world(w + 20_000 + k * 10_000);
            step(&p, &mut gs, &c, k as f64, 1.0 + k as f64);
            // Lost guilds keep the values they died with, as a record.
            for g in gs.iter().filter(|g| g.alive) {
                for t in &p.traits {
                    let (lo, hi) = p.limits(t, &c);
                    let v = g.traits[&t.id];
                    assert!(
                        v >= lo && v <= hi,
                        "world {w} {}: {v} outside {lo}..{hi}",
                        t.id
                    );
                }
                assert!(g.share >= 0.0 && g.share <= 1.0);
            }
        }
    }
}

/// The same 40 K warming, spread over shorter and shorter times: the faster
/// it comes, the more guilds are lost, and a slow enough change loses none.
#[test]
fn extinctions_scale_with_the_speed_of_change() {
    let p = TraitParams::default();
    let seed = Seed::from_text("e5-speed");
    let lost_over = |span_myr: f64| -> usize {
        let mut lost = 0;
        for (n, opp) in [0.01, 0.03, 0.1, 0.3, 1.0, 3.0, 10.0]
            .into_iter()
            .enumerate()
        {
            let mut c = Conditions {
                opportunity: opp,
                ..earthlike()
            };
            let mut gs = seed_guilds(&p, &c, &habitat(n as u32), &seed).unwrap();
            // Settle for 100 relaxation times first.
            let tau = 0.5 / opp;
            for i in 0..100 {
                step(&p, &mut gs, &c, i as f64, tau);
            }
            let start = gs.iter().filter(|g| g.alive).count();
            let steps = 200;
            for i in 0..steps {
                c.temperature_k = 288.0 + 40.0 * (i + 1) as f64 / steps as f64;
                step(&p, &mut gs, &c, i as f64, span_myr / steps as f64);
            }
            lost += start - gs.iter().filter(|g| g.alive).count();
        }
        lost
    };
    let spans = [0.01, 0.1, 1.0, 10.0, 100.0, 1000.0];
    let lost: Vec<usize> = spans.iter().map(|s| lost_over(*s)).collect();
    for w in lost.windows(2) {
        assert!(w[0] >= w[1], "{lost:?}");
    }
    assert!(lost[0] > lost[3], "{lost:?}");
    assert_eq!(*lost.last().unwrap(), 0, "{lost:?}");
}

/// Listing guilds in another order changes nothing.
#[test]
fn order_independent() {
    let p = TraitParams::default();
    let seed = Seed::from_text("e5-order");
    let c = earthlike();
    let mut a = seed_guilds(&p, &c, &habitat(1), &seed).unwrap();
    let mut b = a.clone();
    b.reverse();
    let hot = Conditions {
        temperature_k: 320.0,
        ..c.clone()
    };
    let ea = step(&p, &mut a, &hot, 1.0, 0.05);
    let eb = step(&p, &mut b, &hot, 1.0, 0.05);
    b.reverse();
    assert_eq!(a, b);
    let mut ea_ids: Vec<_> = ea.iter().map(|e| e.guild.clone()).collect();
    let mut eb_ids: Vec<_> = eb.iter().map(|e| e.guild.clone()).collect();
    ea_ids.sort();
    eb_ids.sort();
    assert_eq!(ea_ids, eb_ids);
}

/// For a habitat that stays put, the cheap envelope and the stepped history
/// agree: the survivors' mean traits sit at the optima.
#[test]
fn envelope_agrees_with_stepped_history() {
    let p = TraitParams::default();
    let seed = Seed::from_text("e5-envelope");
    for (n, c) in [
        earthlike(),
        Conditions {
            temperature_k: 300.0,
            water_balance: -0.6,
            gravity_m_s2: 3.7,
            ..earthlike()
        },
        Conditions {
            temperature_k: 270.0,
            swing_k: 40.0,
            disturbance_rel: 3.0,
            ..earthlike()
        },
    ]
    .into_iter()
    .enumerate()
    {
        let mut gs = seed_guilds(&p, &c, &habitat(n as u32), &seed).unwrap();
        settle(&p, &mut gs, &c);
        let mean = mean_traits(&gs);
        let env = p.envelope(&c);
        for t in &p.traits {
            let tol = if t.optimum == "stature" {
                t.tolerance * p.stature_limit(&c)
            } else {
                t.tolerance
            };
            assert!(
                (mean[&t.id] - env[&t.id]).abs() < 0.01 * tol,
                "{}: stepped {} envelope {}",
                t.id,
                mean[&t.id],
                env[&t.id]
            );
        }
    }
    // The hydraulic limit: lower gravity, taller possible trees.
    let mars = Conditions {
        gravity_m_s2: 3.71,
        water_balance: 1.0,
        ..earthlike()
    };
    assert!(p.envelope(&mars)["stature_m"] > 300.0);
}

/// A shift the guilds can just follow: strain climbs past the rescue mark,
/// then eases, and the rescue is recorded.
#[test]
fn a_survivable_shift_is_recorded_as_rescue() {
    let p = TraitParams::default();
    let seed = Seed::from_text("e5-rescue");
    let c = earthlike();
    let mut gs = seed_guilds(&p, &c, &habitat(2), &seed).unwrap();
    settle(&p, &mut gs, &c);
    let warm = Conditions {
        temperature_k: 313.0, // 2.5 tolerances away
        ..c
    };
    let mut events = vec![];
    for i in 0..40 {
        events.extend(step(&p, &mut gs, &warm, 100.0 + i as f64 * 0.01, 0.01));
    }
    assert!(events.iter().any(|e| e.kind == "rescued"), "{events:?}");
    assert!(events.iter().all(|e| e.kind != "lost"));
}
