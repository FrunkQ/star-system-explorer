use crate::atlas::{downsample_error_m, Atlas, TileKey};
use crate::plates::*;
use crate::scenario::{fixtures, settings};
use crate::sketch::{hypsometry, Sketch, SurveyState};
use crate::*;
use sse_core::draw::Seed;

fn env_of(s: &SurfaceSettings, name: &str) -> (Body, SurfaceEnvelope) {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body;
    let e = envelope(s, &b);
    (b, e)
}

fn seed() -> Seed {
    Seed::from_text("sse4-plates-tests")
}

fn bimodality(x: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mean = x.iter().sum::<f64>() / n;
    let sd = (x.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n).max(1e-9);
    let sd = sse_core::math::sqrt(sd);
    let p = |v: f64, k: u32| (0..k).fold(1.0, |a, _| a * (v - mean) / sd);
    let m3 = x.iter().map(|v| p(*v, 3)).sum::<f64>() / n;
    let m4 = x.iter().map(|v| p(*v, 4)).sum::<f64>() / n;
    (m3 * m3 + 1.0) / m4
}

#[test]
fn earth_forms_continents_ridges_and_arcs() {
    let s = settings();
    let (b, e) = env_of(&s, "Earth");
    let h = history(&s, &e, &seed(), b.age_gyr);
    let nc = h.cells.len() as f64;
    assert!(h.steps > 100, "{}", h.steps);
    assert!(
        (3..=20).contains(&h.plates.len()),
        "{} plates",
        h.plates.len()
    );
    let cont = h.cells.iter().filter(|c| c.continental).count() as f64 / nc;
    assert!(cont > 0.15 && cont < 0.45, "continental {cont}");
    let at = |k: usize| {
        h.boundaries
            .iter()
            .filter(|x| x[k].as_ref().is_some_and(|r| r.distance_km == 0.0))
            .count()
    };
    assert!(
        at(0) > 100 && at(1) > 100,
        "ridges {} trenches {}",
        at(0),
        at(1)
    );
    let arcs = h
        .cells
        .iter()
        .filter(|c| c.arc_myr.is_some_and(|t| h.epoch_myr - t < 50.0))
        .count();
    let belts = h.cells.iter().filter(|c| !c.orogeny.is_empty()).count();
    assert!(arcs > 50 && belts > 50, "arcs {arcs} belts {belts}");
    let kinds: std::collections::BTreeSet<&str> =
        h.events.iter().map(|x| x.kind.as_str()).collect();
    for k in ["collision", "plate-split", "pole-jump"] {
        assert!(kinds.contains(k), "no {k} event: {kinds:?}");
    }
    // Two-peaked heights: continents and ocean floor.
    let hs = cell_heights(&s, &h, e.gravity_m_s2);
    assert!(
        bimodality(&hs) > 5.0 / 9.0,
        "bimodality {}",
        bimodality(&hs)
    );
    let sl = sea_level(&hs, e.reference_radius_m, b.oceans).unwrap();
    let land = hs.iter().filter(|x| **x > sl).count() as f64 / nc;
    assert!(land > 0.15 && land < 0.45, "land {land}");
    assert!(hs.iter().all(|x| *x < 12_000.0 && *x > -12_000.0));
}

#[test]
fn stagnant_lid_forms_no_plates() {
    let s = settings();
    for name in ["Mars", "Mercury", "Moon"] {
        let (b, e) = env_of(&s, name);
        let h = history(&s, &e, &seed(), b.age_gyr);
        assert_eq!(h.steps, 0, "{name}");
        assert_eq!(h.plates.len(), 1, "{name}");
        assert!(
            h.boundaries.iter().all(|x| x.iter().all(|k| k.is_none())),
            "{name}"
        );
        assert!(
            h.cells
                .iter()
                .all(|c| c.orogeny.is_empty() && c.arc_myr.is_none()),
            "{name}"
        );
        assert!(h.snapshots.is_empty() && h.events.is_empty(), "{name}");
    }
}

#[test]
fn crust_ages_grow_away_from_ridges() {
    let s = settings();
    let (b, e) = env_of(&s, "Earth");
    let h = history(&s, &e, &seed(), b.age_gyr);
    let sp = e.reference_radius_m * (sse_core::math::PI / 2.0) / h.n as f64 / 1000.0;
    let mut near = (0.0, 0.0);
    let mut far = (0.0, 0.0);
    for (c, bd) in h.cells.iter().zip(&h.boundaries) {
        if c.continental {
            continue;
        }
        if let Some(r) = &bd[0] {
            let d = r.distance_km / sp;
            let age = h.epoch_myr - c.born_myr;
            if d <= 1.0 {
                near = (near.0 + age, near.1 + 1.0);
            } else if (4.0..=8.0).contains(&d) {
                far = (far.0 + age, far.1 + 1.0);
            }
        }
    }
    let (near, far) = (near.0 / near.1, far.0 / far.1);
    assert!(
        near < 0.6 * far,
        "near-ridge mean age {near} Myr, 4 to 8 cells out {far} Myr"
    );
    // Symmetric: across each spreading boundary, the two sides are about equally young.
    let g = Grid::new(h.n);
    let mut diffs = vec![];
    for c in 0..h.cells.len() {
        let a = &h.cells[c];
        if a.continental
            || h.boundaries[c][0]
                .as_ref()
                .is_none_or(|r| r.distance_km > 0.0)
        {
            continue;
        }
        for nb in g.neighbours(c) {
            let o = &h.cells[nb];
            if o.plate != a.plate
                && !o.continental
                && h.boundaries[nb][0]
                    .as_ref()
                    .is_some_and(|r| r.distance_km == 0.0)
            {
                diffs.push((a.born_myr - o.born_myr).abs());
            }
        }
    }
    diffs.sort_by(f64::total_cmp);
    let median = diffs[diffs.len() / 2];
    assert!(
        diffs.len() > 50 && median <= 4.0 * s.plates.step_myr,
        "{} pairs, median age difference {median} Myr",
        diffs.len()
    );
}

#[test]
fn replays_bit_identically_and_drifts() {
    let s = settings();
    let (b, e) = env_of(&s, "Earth");
    let full = history(&s, &e, &seed(), b.age_gyr);
    assert_eq!(full, history(&s, &e, &seed(), b.age_gyr));
    // A run to an earlier date is a prefix of the full run.
    let early = history(&s, &e, &seed(), 2.1);
    assert_eq!(full.snapshots[..early.snapshots.len()], early.snapshots[..]);
    let last = early.snapshots.last().unwrap();
    assert_eq!(last.t_myr, early.epoch_myr);
    assert_eq!(
        last.plate,
        early.cells.iter().map(|c| c.plate).collect::<Vec<_>>()
    );
    // Continents drift on the time ruler.
    let first = &full.snapshots[0];
    let end = full.snapshots.last().unwrap();
    let moved = first
        .continental
        .iter()
        .zip(&end.continental)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        moved as f64 > 0.1 * first.continental.len() as f64,
        "moved {moved}"
    );
    assert!(plates_at(&full, 1000.0).unwrap().t_myr <= 1000.0);
}

#[test]
fn the_surveyed_atlas_reads_the_plates() {
    let mut s = settings();
    s.atlas.tile_samples = 32;
    let (b, e) = env_of(&s, "Earth");
    let h = history(&s, &e, &seed(), b.age_gyr);
    let a = Atlas::from_plates(&s, &e, &seed(), &h, b.oceans, vec![]);
    assert!(a.has_plates());
    assert!(a.address.to_string().ends_with("shell:surface@4500"));
    let mut el = vec![];
    let mut land = 0;
    for f in 0..6 {
        let t = a.build(TileKey::new(f, 1, 1, 0).unwrap());
        land += t.cover.iter().filter(|c| **c == 0).count();
        el.extend(t.elevation_dm.iter().map(|x| *x as f64 / 10.0));
    }
    let frac = land as f64 / el.len() as f64;
    assert!(frac > 0.1 && frac < 0.5, "land {frac}");
    assert!(bimodality(&el) > 5.0 / 9.0, "{}", bimodality(&el));
    for parent in [
        TileKey::new(0, 2, 1, 3).unwrap(),
        TileKey::new(3, 9, 200, 100).unwrap(),
    ] {
        let p = a.build(parent);
        for c in parent.children() {
            let err = downsample_error_m(&p, &a.build(c));
            assert!(
                err <= a.tolerance_m(c.level),
                "{parent:?}: {err} over {}",
                a.tolerance_m(c.level)
            );
        }
    }
    // Sketch statistics still hold for the scanned globe of the same world.
    let sk: Sketch = sketch(&s, &e, &seed(), SurveyState::Scanned);
    assert!(hypsometry(&sk).2 > 5.0 / 9.0);
}

#[test]
fn golden_plates() {
    let got = crate::scenario::plates_golden();
    let path = format!(
        "{}/../../tests/golden/surface-plates.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1");
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden surface-plates changed"
    );
}

#[test]
fn a_world_without_plates_keeps_its_sketch_atlas() {
    let s = settings();
    let (b, e) = env_of(&s, "Mars");
    let h = history(&s, &e, &seed(), b.age_gyr);
    let a = Atlas::from_plates(&s, &e, &seed(), &h, b.oceans, vec![]);
    assert!(!a.has_plates());
    assert!(a.address.to_string().contains("shell:sketch@"));
}
