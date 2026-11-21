use crate::atlas::{downsample_error_m, Atlas, TileKey};
use crate::plates::*;
use crate::scenario::{fixtures, settings};
use crate::wear::*;
use crate::*;
use sse_core::draw::{Draws, Seed, Stream};

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
    Seed::from_text("sse4-wear-tests")
}

fn wet(
    s: &SurfaceSettings,
    b: &Body,
    e: &SurfaceEnvelope,
    water: &WaterHistory,
    rot: f64,
) -> crate::climate::WorldClimate {
    let x = sse_climate::Extra {
        rotation_hours: Some(rot),
        ..Default::default()
    };
    crate::climate::wet_climate(s, b, e, water, &x, b.age_gyr * 1000.0)
}

fn earth(s: &SurfaceSettings) -> (Body, SurfaceEnvelope, PlateHistory, Wear) {
    let (b, e) = env_of(s, "Earth");
    let h = history(s, &e, &seed(), b.age_gyr);
    let water = WaterHistory::steady(b.oceans, b.pressure_bar, b.age_gyr);
    let cl = wet(s, &b, &e, &water, 23.93);
    let w = wear_world(s, &e, &seed(), &h, &water, &cl, b.magma_end_myr);
    (b, e, h, w)
}

fn mars_at(s: &SurfaceSettings, t_gyr: f64) -> (SurfaceEnvelope, PlateHistory, Wear) {
    let (b, e) = env_of(s, "Mars");
    let h = history(s, &e, &seed(), t_gyr);
    let water = WaterHistory {
        wet_from_gyr: 0.0,
        wet_until_gyr: 1.0,
        oceans: 0.05,
        pressure_bar: 0.006,
    };
    let cl = wet(s, &b, &e, &water, 24.62);
    let w = wear_world(s, &e, &seed(), &h, &water, &cl, 50.0);
    (e, h, w)
}

#[test]
fn sediment_balance_closes() {
    let s = settings();
    let (_, _, _, w) = earth(&s);
    assert!(w.eroded_m3 > 0.0);
    assert!(
        (w.eroded_m3 - w.deposited_m3).abs() <= 1e-9 * w.eroded_m3,
        "{} {}",
        w.eroded_m3,
        w.deposited_m3
    );
    let (_, _, w) = mars_at(&s, 4.5);
    assert!((w.eroded_m3 - w.deposited_m3).abs() <= 1e-9 * w.eroded_m3);
    // Dry after 1 Gyr: nothing is eroded by water later.
    assert!(w
        .epochs
        .iter()
        .filter(|x| !x.wet)
        .all(|x| x.eroded_m3 == 0.0));
}

#[test]
fn drainage_is_dendritic_and_follows_hacks_law() {
    let s = settings();
    let (_, e, _, w) = earth(&s);
    let nc = w.heights.len();
    let cell = 4.0 * sse_core::math::PI * e.reference_radius_m * e.reference_radius_m / nc as f64;
    let (mut sx, mut sy, mut sxx, mut sxy, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for c in 0..nc {
        if w.river[c] && w.area_m2[c] > 10.0 * cell && w.stream_m[c] > 0.0 {
            let (x, y) = (
                sse_core::math::ln(w.area_m2[c]),
                sse_core::math::ln(w.stream_m[c]),
            );
            sx += x;
            sy += y;
            sxx += x * x;
            sxy += x * y;
            n += 1.0;
        }
    }
    let hack = (n * sxy - sx * sy) / (n * sxx - sx * sx);
    assert!(
        n > 50.0 && hack > 0.45 && hack < 0.8,
        "Hack exponent {hack} over {n} cells"
    );
    // Branching: river cells fed by two or more river cells.
    let mut donors = vec![0u32; nc];
    for c in 0..nc {
        let r = w.receiver[c] as usize;
        if r != c && w.river[c] {
            donors[r] += 1;
        }
    }
    let confluences = (0..nc).filter(|&c| w.river[c] && donors[c] >= 2).count();
    assert!(confluences > 10, "{confluences} confluences");
    assert!(w.lake.iter().any(|x| *x));
}

#[test]
fn no_river_runs_uphill_in_a_thousand_worlds() {
    let mut s = settings();
    s.wear.epoch_myr = 100.0;
    let (b, e) = env_of(&s, "Earth");
    let n = 8;
    let nc = 6 * n * n;
    for k in 0..1000u64 {
        let d = Draws::new(
            Seed::from_text(&format!("uphill-{k}")),
            e.address.clone(),
            Stream::new("surface.test.v1").unwrap(),
        );
        let h0: Vec<f64> = (0..nc as u64)
            .map(|i| 2000.0 * d.normal(i) + 500.0 * d.normal(nc as u64 + i))
            .collect();
        let oceans = [0.0, 0.3, 1.0, 3.0][(k % 4) as usize];
        let water = WaterHistory::steady(oceans, 1.0, 1.0);
        let cl = wet(&s, &b, &e, &water, 23.93);
        // Every tenth world is also worn for 300 Myr.
        let to = if k % 10 == 0 { 300.0 } else { 0.0 };
        let w = wear(&s, &e, n, &h0, &water, &cl, 0.0, to);
        let h = &w.heights;
        for c in 0..nc as usize {
            // Downhill on the true surface wherever neither end is in a filled basin.
            let r = w.receiver[c] as usize;
            if r != c && !w.pit[c] && !w.pit[r] {
                assert!(
                    h[r] <= h[c] + 1e-6,
                    "world {k}: {c} -> {r} uphill {} -> {}",
                    h[c],
                    h[r]
                );
            }
            // Every path ends at an outlet, without cycles.
            let mut x = c;
            let mut steps = 0;
            while w.receiver[x] as usize != x {
                x = w.receiver[x] as usize;
                steps += 1;
                assert!(steps <= nc as usize, "world {k}: cycle from {c}");
            }
            if w.sea_level_raw_m.is_some() {
                assert!(h[x] < 1e-6, "world {k}: outlet {x} above sea level");
            }
        }
    }
}

#[test]
fn an_old_dried_world_keeps_softened_valleys() {
    let s = settings();
    let (_, _, young) = mars_at(&s, 1.0);
    let (_, _, old) = mars_at(&s, 4.5);
    let g = Grid::new(young.n);
    // Steepness around the valleys: mean height step to the four neighbours.
    let steep = |w: &Wear, cells: &[usize]| {
        cells
            .iter()
            .map(|&c| {
                g.neighbours(c)
                    .iter()
                    .map(|&x| (w.heights[x] - w.heights[c]).abs())
                    .sum::<f64>()
                    / 4.0
            })
            .sum::<f64>()
            / cells.len() as f64
    };
    let valleys: Vec<usize> = (0..young.heights.len())
        .filter(|&c| young.incision_m[c] > 100.0 && young.river[c])
        .collect();
    assert!(valleys.len() > 20, "{} valleys", valleys.len());
    let (s_young, s_old) = (steep(&young, &valleys), steep(&old, &valleys));
    assert!(s_old < s_young, "softened: {s_young} -> {s_old}");
    // Kept: the valleys are still cut into the old surface.
    let kept = valleys
        .iter()
        .filter(|&&c| old.incision_m[c] > 50.0)
        .count();
    assert!(
        kept * 2 >= valleys.len(),
        "{kept} of {} valleys kept",
        valleys.len()
    );
    assert!(old.incision_m.iter().filter(|x| **x > 100.0).count() >= valleys.len() / 2);
    assert!(old.river.iter().all(|x| !x), "no rivers on a dry world");
}

#[test]
fn the_worn_atlas_has_channels() {
    let mut s = settings();
    s.atlas.tile_samples = 32;
    let (_, e, h, w) = earth(&s);
    let a = Atlas::from_worn(&s, &e, &seed(), &h, &w, vec![]);
    let mut channels = 0;
    for f in 0..6 {
        let t = a.build(TileKey::new(f, 1, 0, 1).unwrap());
        assert_eq!(t.flow.len(), t.elevation_dm.len());
        assert!(!t.pending.iter().any(|p| p.contains("S3")));
        channels += t.flow.iter().filter(|x| **x > 0).count();
    }
    assert!(channels > 0);
    let parent = TileKey::new(2, 4, 3, 9).unwrap();
    let p = a.build(parent);
    for c in parent.children() {
        let err = downsample_error_m(&p, &a.build(c));
        assert!(err <= a.tolerance_m(c.level), "{err}");
    }
    assert_eq!(a.build(parent), p);
}

#[test]
fn golden_wear() {
    let got = crate::scenario::wear_golden();
    let path = format!(
        "{}/../../tests/golden/surface-wear.json",
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
        "golden surface-wear changed"
    );
}
