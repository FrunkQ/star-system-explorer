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
        let w = wear(&s, &e, n, &h0, &water, &cl, 0.0, to, k % 20 == 0);
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

/// On a world whose plates move, wear routes water and lays sediment but
/// keeps the plate history's continents; sediment counts only where it
/// still lies, and what reaches the sea builds shelves, never islands.
#[test]
fn sediment_lies_where_it_was_left_and_shelves_stay_under_water() {
    let s = settings();
    let (b, e, h, w) = earth(&s);
    let h0 = base_heights(&s, &e, &seed(), &h);
    let sl0 = sea_level(&h0, e.reference_radius_m, b.oceans).expect("a sea");
    let nc = h0.len();
    let land0 = (0..nc).filter(|&c| h0[c] >= sl0).count();
    let land = (0..nc).filter(|&c| w.heights[c] >= 0.0).count();
    assert!(
        (land as f64 - land0 as f64).abs() <= 0.02 * land0 as f64,
        "land kept: {land0} -> {land}"
    );
    // No cell that started under the sea is raised above it by sediment.
    let raised = (0..nc)
        .filter(|&c| h0[c] < sl0 && w.heights[c] >= 0.0 && w.sediment_m[c] > 10.0)
        .count();
    assert!(raised <= nc / 1000, "{raised} sea cells built into land");
    // Bare land: a cover of sediment on part of it, crystalline rock elsewhere.
    let covered = (0..nc)
        .filter(|&c| w.heights[c] >= 0.0 && w.sediment_m[c] > 10.0)
        .count() as f64
        / land.max(1) as f64;
    assert!(
        (0.15..0.7).contains(&covered),
        "sediment-covered share of land {covered}"
    );
}

/// W9: rivers carry runoff, glaciers sit where snow outlasts the summer, and
/// dunes lie where the wind moves sand on dry ground, on wet and dry worlds.
#[test]
fn rivers_glaciers_and_dunes_read_the_one_climates_water_and_wind() {
    let s = settings();
    let (_, _, _, w) = earth(&s);
    let g = Grid::new(w.n);
    let lat = |c: usize| sse_core::math::asin(g.dirs[c][2].clamp(-1.0, 1.0)).to_degrees();
    let nc = w.heights.len();
    let land: Vec<usize> = (0..nc).filter(|&c| w.heights[c] >= 0.0).collect();
    let dunes: Vec<f64> = land
        .iter()
        .filter(|&&c| w.dunes[c])
        .map(|&c| lat(c))
        .collect();
    let ice: Vec<f64> = land
        .iter()
        .filter(|&&c| w.glaciated[c])
        .map(|&c| lat(c))
        .collect();
    eprintln!(
        "Earth land {} dunes {} (lat {:?}) glaciated {} rivers {}",
        land.len(),
        dunes.len(),
        dunes
            .iter()
            .map(|x| x.round())
            .collect::<Vec<_>>()
            .get(..dunes.len().min(20)),
        ice.len(),
        w.river.iter().filter(|x| **x).count()
    );
    assert!(!dunes.is_empty(), "Earth's deserts have dunes");
    assert!(
        dunes.iter().all(|l| l.abs() < 45.0),
        "dunes in the dry belts"
    );
    // Glaciers: a larger share of the land poleward, and near the equator
    // only on high mountains (the tropical snow line is near 5 km).
    let share = |lo: f64, hi: f64| {
        let cs: Vec<usize> = land
            .iter()
            .cloned()
            .filter(|&c| (lo..hi).contains(&lat(c).abs()))
            .collect();
        cs.iter().filter(|&&c| w.glaciated[c]).count() as f64 / cs.len().max(1) as f64
    };
    assert!(share(60.0, 90.0) > 0.9 && share(0.0, 15.0) < share(30.0, 45.0));
    let tropical_low = land
        .iter()
        .filter(|&&c| lat(c).abs() < 15.0 && w.glaciated[c])
        .map(|&c| w.heights[c])
        .fold(f64::INFINITY, f64::min);
    assert!(
        tropical_low > 4000.0,
        "tropical glacier at {tropical_low} m"
    );
    assert!(!ice.is_empty());
    assert!(land.iter().any(|&c| w.runoff_m_yr[c] > 0.0));
    // Each land cell's climate label, read off its own numbers.
    for a in ["ice-cap", "rainforest", "temperate"] {
        assert!(land.iter().any(|&c| w.areas[c] == a), "Earth has {a}");
    }
    assert!(land
        .iter()
        .any(|&c| w.areas[c] == "desert" || w.areas[c] == "rainless"));
    assert!((0..nc).all(|c| w.heights[c] >= 0.0 || w.areas[c].is_empty()));
    // Mars, dry for 3.5 Gyr: no runoff now, dunes from the wind alone.
    let (_, _, m) = mars_at(&s, 4.5);
    assert!(m.runoff_m_yr.iter().all(|x| *x == 0.0));
    assert!(m.dunes.iter().any(|x| *x), "Mars has dunes");
}

/// W9 part 2: with an orbit history, the ice edge advances and retreats:
/// cells near it spend part of each cycle under ice and record advances
/// (moraines), cells far inside or outside it record none.
#[test]
fn an_orbit_history_moves_the_ice_edge() {
    use sse_climate::forcing::{MoonPull, OrbitTrack, TrackSample};
    let s = settings();
    let (b, e) = env_of(&s, "Earth");
    let h = history(&s, &e, &seed(), b.age_gyr);
    let water = WaterHistory::steady(b.oceans, b.pressure_bar, b.age_gyr);
    let mut cl = wet(&s, &b, &e, &water, 23.93);
    let s3 = -18.85 / 3600.0 * sse_core::math::PI / 180.0;
    let track = OrbitTrack {
        samples: (0..=200)
            .map(|k| {
                let t = f64::from(k) * 2000.0;
                TrackSample {
                    t_yr: t,
                    e: 0.0167,
                    varpi: 0.0,
                    incl: 0.035,
                    node: s3 * t,
                }
            })
            .collect(),
        source: "one inclination mode".into(),
    };
    let moon = [MoonPull {
        mass_kg: 7.342e22,
        a_m: 3.844e8,
    }];
    cl.ice_ages = Some(crate::climate::ice_ages(
        &s,
        &b,
        &cl,
        &track,
        &moon,
        b.age_gyr * 1000.0,
    ));
    let w = wear_world(&s, &e, &seed(), &h, &water, &cl, b.magma_end_myr);
    let nc = w.heights.len();
    let edge = (0..nc)
        .filter(|&c| w.heights[c] >= 0.0 && w.ice_share[c] > 0.0 && w.ice_share[c] < 1.0)
        .count();
    let moraines = (0..nc)
        .filter(|&c| w.glacial_advances_per_myr[c] > 0.0)
        .count();
    assert!(edge > 0 && moraines > 0, "edge {edge} moraines {moraines}");
    // Advances only where the ice comes and goes.
    assert!((0..nc)
        .all(|c| w.glacial_advances_per_myr[c] == 0.0
            || (w.ice_share[c] > 0.0 && w.ice_share[c] < 1.0)));
}

/// The edges and lengths of a grid on a body of radius `r` (as `wear` builds them).
fn grid_edges(n: u32, r: f64) -> (std::collections::BTreeSet<(usize, usize)>, Vec<f64>) {
    let g = Grid::new(n);
    let mut edges = std::collections::BTreeSet::new();
    for c in 0..g.len() {
        for x in g.neighbours(c) {
            if x != c {
                edges.insert((c.min(x), c.max(x)));
            }
        }
    }
    let lens = edges
        .iter()
        .map(|&(a, b)| {
            let (p, q) = (g.dirs[a], g.dirs[b]);
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            (sse_core::math::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) * r).max(1.0)
        })
        .collect();
    (edges, lens)
}

fn rough(nc: usize, k: &str) -> Vec<f64> {
    let d = Draws::new(
        Seed::from_text(k),
        "u:t/sys:1/body:0".parse().unwrap(),
        Stream::new("surface.test.v1").unwrap(),
    );
    (0..nc as u64).map(|i| 1000.0 * d.normal(i)).collect()
}

/// The implicit solve gives the surface the explicit steps give, where both
/// can run: a grid fine enough that one epoch needs hundreds of explicit
/// steps, on a body sized so the epoch only partly smooths its largest
/// features (a degree-2 bulge decays by about e^-1) while it wipes out the
/// cell-scale roughness. Stated tolerance: the heights differ by under 2% of
/// what the epoch changed (RMS), and the total height is kept to a part in a
/// billion.
#[test]
fn implicit_slopes_match_the_explicit_steps() {
    let n = 48;
    let kappa = 10.0;
    let dt = 1.0e8;
    // kappa dt / R^2 = 1/6: the degree-2 mode's rate l(l+1) kappa / R^2 times dt is 1.
    let r = sse_core::math::sqrt(6.0 * kappa * dt);
    let (edges, lens) = grid_edges(n, r);
    let g = Grid::new(n);
    let noise = rough(g.len(), "implicit-vs-explicit");
    let h0: Vec<f64> = g
        .dirs
        .iter()
        .zip(&noise)
        .map(|(d, e)| 3000.0 * (3.0 * d[2] * d[2] - 1.0) + 1500.0 * d[0] * d[1] + 0.3 * e)
        .collect();
    let lmin = lens.iter().copied().fold(f64::INFINITY, f64::min);
    let sub = sse_core::math::ceil(dt * kappa / (lmin * lmin) / 0.2) as u32;
    assert!(
        sub > EXPLICIT_STEPS_MAX && sub < 5000,
        "{sub} explicit steps"
    );
    let mut he = h0.clone();
    relax_explicit(&mut he, &edges, &lens, kappa, dt, sub, &mut |_| {});
    let mut hi = h0.clone();
    let got = relax_implicit(
        &mut hi,
        &edges,
        &lens,
        kappa * dt,
        IMPLICIT_STEPS,
        &mut |_| {},
    );
    assert_eq!(got.steps, IMPLICIT_STEPS);
    let rms = |v: &mut dyn Iterator<Item = f64>| {
        let (s, k) = v.fold((0.0, 0.0), |(s, k), x| (s + x * x, k + 1.0));
        sse_core::math::sqrt(s / k)
    };
    let changed = rms(&mut he.iter().zip(&h0).map(|(a, b)| a - b));
    let apart = rms(&mut he.iter().zip(&hi).map(|(a, b)| a - b));
    assert!(
        changed > 500.0,
        "the epoch changed the surface: {changed} m"
    );
    assert!(
        apart < 0.02 * changed,
        "implicit {apart} m from explicit, of {changed} m changed"
    );
    let (s0, si): (f64, f64) = (h0.iter().sum(), hi.iter().sum());
    assert!(
        (s0 - si).abs() <= 1e-9 * h0.iter().map(|x| x.abs()).sum::<f64>(),
        "total height kept: {s0} vs {si}"
    );
}

/// Small bodies stay fast: an epoch of slope relaxation on a Phobos-sized
/// grid (11 km, 64 cells a face) costs under 12,000 edge sweeps and under a tenth of the
/// explicit steps it replaces, and the Moon still takes the explicit steps
/// exactly as before.
#[test]
fn slope_relaxation_on_a_small_body_is_cheap() {
    // The surface grid base-surface uses (cells_per_face 64).
    let n = 64;
    let kappa = 10.0;
    let dt = 1.0e8;
    let (edges, lens) = grid_edges(n, 11.0e3);
    let mut h = rough(6 * (n * n) as usize, "phobos-cost");
    let got = relax_slopes(&mut h, &edges, &lens, kappa, dt, &mut |_| {});
    assert_eq!(got.steps, IMPLICIT_STEPS, "the implicit solve");
    let lmin = lens.iter().copied().fold(f64::INFINITY, f64::min);
    let explicit = sse_core::math::ceil(dt * kappa / (lmin * lmin) / 0.2) as u64;
    assert!(
        got.sweeps < 12_000 && got.sweeps * 10 < explicit,
        "{} sweeps for one epoch on an 11 km body, against {explicit} explicit steps",
        got.sweeps
    );
    let (edges, lens) = grid_edges(n, 1.7374e6);
    let mut h = rough(6 * (n * n) as usize, "moon-cost");
    let got = relax_slopes(&mut h, &edges, &lens, kappa, dt, &mut |_| {});
    assert!(
        got.steps <= EXPLICIT_STEPS_MAX,
        "the Moon keeps the explicit steps"
    );
    assert_eq!(got.sweeps, got.steps as u64);
}
