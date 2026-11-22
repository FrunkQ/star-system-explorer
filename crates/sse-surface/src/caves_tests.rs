use crate::caves::{passages, CaveSurvey};
use crate::scenario::{fixtures, settings};
use crate::survey::{survey_world, Survey, SurveyOptions};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;

fn surveyed(name: &str) -> Survey {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .expect("fixture")
        .body;
    survey_world(
        &settings(),
        &b,
        &Seed::from_text("sse4-caves-tests"),
        b.age_gyr,
        &SurveyOptions::default(),
    )
    .expect("survey")
}

fn caves(sv: &Survey, lat: f64, lon: f64, r_km: f64) -> CaveSurvey {
    let d = Draws::new(
        sv.seed,
        sv.envelope.address.clone(),
        Stream::new("surface.caves.v1").unwrap(),
    );
    passages(&settings().caves, &sv.atlas, &sv.tubes, &d, lat, lon, r_km)
}

fn lat_lon(p: [f64; 3]) -> (f64, f64) {
    (
        m::asin(p[2]).to_degrees(),
        m::atan2(p[1], p[0]).to_degrees(),
    )
}

/// The densest tube field of a world (most passage per km²).
fn densest(sv: &Survey) -> &crate::tubes::TubeField {
    sv.tubes
        .iter()
        .max_by(|a, b| {
            let da = a.length_km / (a.radius_km * a.radius_km);
            let db = b.length_km / (b.radius_km * b.radius_km);
            da.total_cmp(&db)
        })
        .expect("a field")
}

/// Passages are the same world from any site: the same tube found from two
/// overlapping windows is the same tube, and a survey repeats exactly.
#[test]
fn passages_are_one_world_from_any_site() {
    let sv = surveyed("Moon");
    let f = densest(&sv);
    let (la, lo) = lat_lon(f.dir);
    let a = caves(&sv, la, lo, 25.0);
    assert!(!a.passages.is_empty(), "{}", a.why);
    assert_eq!(a, caves(&sv, la, lo, 25.0));
    // A small window centred on any node of a tube, away from the first
    // site, finds that same tube, node for node.
    let mut shared = 0;
    for p in &a.passages {
        let mid = &p.nodes[p.nodes.len() / 2];
        let b = caves(&sv, mid.lat_deg, mid.lon_deg, 2.0);
        if let Some(q) = b.passages.iter().find(|q| q.id == p.id) {
            shared += 1;
            assert_eq!(p.segments, q.segments);
            for (x, y) in p.nodes.iter().zip(&q.nodes) {
                assert_eq!(
                    (x.lat_deg, x.lon_deg, x.ground_m, &x.entrance),
                    (y.lat_deg, y.lon_deg, y.ground_m, &y.entrance)
                );
            }
        }
    }
    assert_eq!(shared, a.passages.len(), "a tube not found again");
}

/// Every passage keeps to its field's envelope: no wider than the widest
/// roof that holds, its floor below the ground, an entrance only where a
/// roof ends, and the fallen share near the envelope's lost roof.
#[test]
fn passages_keep_to_their_envelope() {
    let sv = surveyed("Moon");
    let mut fallen = (0usize, 0usize, 0.0);
    for f in sv.tubes.iter().take(6) {
        let (la, lo) = lat_lon(f.dir);
        let c = caves(&sv, la, lo, 25.0);
        for p in &c.passages {
            let f = sv
                .tubes
                .iter()
                .find(|x| x.id == p.field)
                .expect("its field");
            assert!(p.nodes.len() == p.segments.len() + 1);
            assert!(p.depth_m > p.roof_m && p.roof_m > 0.0);
            assert!(p.cause.contains("lava ran"));
            for s in &p.segments {
                assert!(s.width_m <= f.width_max_m + 1e-9 && s.width_m > 0.0);
                assert!((s.height_m - 0.6 * s.width_m).abs() < 1e-9);
                assert_eq!(s.roofed, s.open.is_none());
                assert!(s.length_m > 0.0 && s.length_m < 500.0);
            }
            for (i, n) in p.nodes.iter().enumerate() {
                assert!(n.floor_m < n.ground_m);
                let before = i.checked_sub(1).map(|x| p.segments[x].roofed);
                let after = p.segments.get(i).map(|s| s.roofed);
                let ends = match (before, after) {
                    (Some(a), Some(b)) => a != b,
                    (None, Some(b)) | (Some(b), None) => !b,
                    _ => false,
                };
                assert_eq!(n.entrance.is_some(), ends, "{} node {i}", p.id);
            }
            fallen.0 += p.segments.iter().filter(|s| !s.roofed).count();
            fallen.1 += p.segments.len();
            fallen.2 += (1.0 - f.roof_kept) * p.segments.len() as f64;
        }
    }
    assert!(fallen.1 > 100, "{} segments", fallen.1);
    let got = fallen.0 as f64 / fallen.1 as f64;
    let want = fallen.2 / fallen.1 as f64;
    assert!(
        (got - want).abs() < 0.15,
        "fallen {got:.2}, envelope {want:.2}"
    );
}

/// Away from any lava field there are no passages, and the survey says why.
#[test]
fn no_lava_no_tubes() {
    let sv = surveyed("Europa");
    let c = caves(&sv, 0.0, 0.0, 10.0);
    assert!(c.passages.is_empty() && c.fields.is_empty());
    assert!(c.why.contains("No lava field"));
}
