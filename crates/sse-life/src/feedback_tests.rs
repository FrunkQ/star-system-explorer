//! L2 checks: oxygenation from the balance, not by date; a reduced world
//! stays anoxic; replay from the manifest; conservation at the boundary.

use crate::feedback::{coupled, sustained_rise, AirParams, AirPoint, CoupledManifest, CoupledRun};
use crate::scenario::coupled_manifest;

const SEEDS: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

fn run(name: &str, seed: &str) -> CoupledRun {
    let mut man = coupled_manifest(name);
    man.seed = seed.into();
    coupled(&man).unwrap()
}

#[test]
fn earth_oxygenates_through_the_balance_not_by_date() {
    let mut early = vec![];
    let mut complex = 0;
    for s in SEEDS {
        let r = run("earth", s);
        let oxygenic = r.history.first["oxygenic"];
        let oxy = r
            .oxygenated_myr
            .unwrap_or_else(|| panic!("seed {s} never oxygenated"));
        // Never before the oxygen makers exist.
        assert!(oxy >= oxygenic, "seed {s}: {oxy} before {oxygenic}");
        // Early oxygenic photosynthesis does not oxygenate the air at once:
        // the reducing sinks hold it down until the interior cools.
        if oxygenic < 1500.0 {
            assert!(oxy - oxygenic > 700.0, "seed {s}: {oxy} vs {oxygenic}");
            early.push(oxy);
        }
        if let Some(c) = r.history.first.get("complex_cells") {
            assert!(*c >= oxygenic);
            complex += 1;
        }
        let end = r.air.series.last().unwrap();
        assert!(
            (0.1..0.6).contains(&end.o2_fraction),
            "seed {s}: {}",
            end.o2_fraction
        );
        assert!(end.ch4_ppm < 5.0, "seed {s}: {}", end.ch4_ppm);
    }
    // With the same sinks, the same balance point, whatever the date life
    // learned the trick.
    assert!(early.len() >= 4);
    let (lo, hi) = early
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
    assert!(hi - lo <= 50.0, "{early:?}");
    assert!(complex >= 6, "{complex}");
}

#[test]
fn a_small_reduced_world_stays_anoxic() {
    for s in SEEDS {
        let r = run("reduced", s);
        assert_eq!(r.oxygenated_myr, None, "seed {s}");
        let max = r
            .air
            .series
            .iter()
            .map(|p| p.o2_fraction)
            .fold(0.0, f64::max);
        assert!(max < 1.0e-4, "seed {s}: {max}");
        assert!(!r.history.first.contains_key("complex_cells"));
        // Methane builds up instead.
        assert!(r.air.series.last().unwrap().ch4_ppm > 100.0);
    }
}

#[test]
fn replay_from_the_manifest_is_identical() {
    let man = coupled_manifest("earth");
    let text = serde_json::to_string(&man).unwrap();
    let back: CoupledManifest = serde_json::from_str(&text).unwrap();
    let (a, b) = (coupled(&man).unwrap(), coupled(&back).unwrap());
    assert_eq!(a, b);
    assert_eq!(a.manifest_hash, b.manifest_hash);
    let h = |r: &CoupledRun| sse_core::ContentHash::of(r).unwrap().hex();
    assert_eq!(h(&a), h(&b));
    assert_ne!(h(&a), h(&run("earth", "b")));
    let mut bad = man.clone();
    bad.format = "other".into();
    assert!(coupled(&bad).is_err());
}

#[test]
fn exchanged_oxygen_is_conserved() {
    for n in ["earth", "reduced"] {
        let r = run(n, "a");
        let t = &r.air.totals;
        let through = t.burial + t.escape + t.reductants + t.weathering;
        assert!(
            r.air.worst_residual < 1.0e-12 * through,
            "{n}: {}",
            r.air.worst_residual
        );
        assert!(t.residual().abs() < 1.0e-12 * through, "{n}");
        // What the air holds is what the ledger says moved in.
        let held = r.air.o2_mol;
        assert!(
            (t.change - held).abs() <= 1.0e-9 * held.max(1.0),
            "{n}: {} vs {held}",
            t.change
        );
        // Escape never exceeds the reducing demand it cancels.
        assert!(t.escape <= t.reductants + t.reductant_unmet);
        assert_eq!(t.carbon_buried, t.burial);
    }
}

#[test]
fn the_box_answers_life_at_every_step() {
    let r = run("earth", "a");
    assert_eq!(r.air.series.len() as f64 * 5.0, 4570.0);
    // History points carry the world facts the box returned.
    let p = r.history.points.last().unwrap();
    assert!(p.world.contains_key("o2_fraction") && p.world.contains_key("ch4_ppm"));
    // Land life draws CO2 down.
    if r.history.first.contains_key("land") {
        assert!(r.air.series.last().unwrap().co2_rel < 1.0);
    }
}

#[test]
fn brief_whiffs_of_oxygen_do_not_count() {
    let pt = |t: f64, f: f64| AirPoint {
        at_myr: t,
        o2_fraction: f,
        ch4_ppm: 0.0,
        co2_rel: 1.0,
        reductant_mol_yr: 0.0,
        burial_mol_yr: 0.0,
        escape_mol_yr: 0.0,
        fire_factor: 1.0,
    };
    let s: Vec<_> = (0..100)
        .map(|i| {
            let t = i as f64 * 10.0;
            pt(
                t,
                if (100.0..150.0).contains(&t) || t >= 400.0 {
                    0.02
                } else {
                    0.0
                },
            )
        })
        .collect();
    assert_eq!(sustained_rise(&s, 0.01, 200.0), Some(400.0));
    assert_eq!(sustained_rise(&s[..45], 0.01, 200.0), Some(400.0));
    assert_eq!(sustained_rise(&s[..30], 0.01, 200.0), None);
}

#[test]
fn golden() {
    let g = crate::scenario::feedback_golden();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/life-feedback.json"
    );
    let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn pack_matches_the_defaults() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-life/pack.json")).unwrap();
    let mut p: AirParams =
        serde_json::from_value(pack["provides"]["other"]["life_feedback"].clone()).unwrap();
    assert!(p.sources.len() >= 4);
    p.sources.clear();
    assert_eq!(p, AirParams::default());
}

fn run_with(name: &str, seed: &str, f: impl Fn(&mut AirParams)) -> CoupledRun {
    let mut man = coupled_manifest(name);
    man.seed = seed.into();
    f(&mut man.air);
    coupled(&man).unwrap()
}

/// E3 (evolution design 7.4): competing oxygen sinks hold Earth between 15
/// and 30 percent on every seed; switching fire or the aerobic decomposers
/// off lets oxygen run high again; the reduced world stays anoxic.
#[test]
fn competing_sinks_hold_oxygen_near_today() {
    let mut rows = vec![];
    for s in SEEDS {
        let r = run("earth", s);
        let end = r.air.series.last().unwrap();
        let land = r.air.totals.burial_land > 0.0;
        let nofire = run_with("earth", s, |p| p.fire_k = 0.0);
        let nodecomp = run_with("earth", s, |p| p.decomposer_h = 0.0);
        let bare = run_with("earth", s, |p| {
            p.fire_k = 0.0;
            p.decomposer_h = 0.0;
            p.photorespiration = 0.0;
        });
        let o = |r: &CoupledRun| r.air.series.last().unwrap().o2_fraction;
        rows.push((
            s,
            land,
            o(&r),
            o(&nofire),
            o(&nodecomp),
            o(&bare),
            end.fire_factor,
            r.oxygenated_myr,
        ));
        eprintln!("{:?}", rows.last().unwrap());
        assert!(
            (0.15..=0.30).contains(&o(&r)),
            "seed {s}: O2 today {}",
            o(&r)
        );
        assert!(
            o(&nofire) >= o(&r) - 1e-9,
            "seed {s}: fire off never lowers O2"
        );
        assert!(
            o(&nodecomp) >= o(&r) - 1e-9,
            "seed {s}: decomposers off never lower O2"
        );
        assert!(o(&bare) >= o(&nofire) - 1e-9);
        assert!(
            r.air.worst_residual < 1.0e6,
            "seed {s}: {}",
            r.air.worst_residual
        );
    }
    for r in &rows {
        eprintln!("seed {} land {} O2 {:.3}  no fire {:.3}  no decomposers {:.3}  bare {:.3}  fire factor {:.2}  oxygenated {:?}", r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7);
    }
    // Where land life arrives, the sinks matter: the bare box runs high.
    let landed: Vec<_> = rows.iter().filter(|r| r.1).collect();
    assert!(!landed.is_empty());
    assert!(landed.iter().any(|r| r.5 > 0.35), "{landed:?}");
    for s in SEEDS {
        let r = run("reduced", s);
        assert_eq!(r.oxygenated_myr, None, "seed {s}");
    }
}
