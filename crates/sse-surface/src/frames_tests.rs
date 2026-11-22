use crate::frames::surface_frames;
use crate::scenario::{fixtures, settings};
use crate::survey::SurveyOptions;
use sse_core::draw::Seed;

fn frames_of(name: &str) -> crate::frames::Frames {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .expect("fixture")
        .body;
    surface_frames(
        &settings(),
        &b,
        &Seed::from_text("sse4-frames-tests"),
        b.age_gyr,
        &SurveyOptions::default(),
        false,
        None,
    )
    .expect("frames")
}

/// Earth's frames run from its first plates to now on one grid, every
/// field a value per cell, the dates in order, and the plates moving.
#[test]
fn earth_frames_span_its_history() {
    let f = frames_of("Earth");
    let nc = f.grid.cells as usize;
    assert_eq!(nc, 6 * (f.grid.n * f.grid.n) as usize);
    assert_eq!(f.dirs.len(), 3 * nc);
    assert!(f.frames.len() >= 10, "{} frames", f.frames.len());
    for w in f.frames.windows(2) {
        assert!(w[0].t_myr < w[1].t_myr);
    }
    let last = f.frames.last().unwrap();
    assert!((last.t_myr - f.epoch_myr).abs() < 1e-6);
    for fr in &f.frames {
        for v in [
            &fr.elevation_m,
            &fr.vent,
            &fr.arc,
            &fr.ridge,
            &fr.ice_m,
            &fr.temp_k,
            &fr.rain_m_yr,
            &fr.runoff_m_yr,
        ] {
            assert_eq!(v.len(), nc);
            assert!(v.iter().all(|x| x.is_finite()));
        }
        assert_eq!(fr.plate.len(), nc);
        for v in [&fr.vent, &fr.arc, &fr.ridge] {
            assert!(v.iter().all(|v| (0.0..=1.0).contains(v)));
        }
        assert!(fr.ice_m.iter().all(|v| *v >= 0.0));
        assert!(!fr.why.is_empty());
    }
    let first = &f.frames[0];
    let moved = first
        .plate
        .iter()
        .zip(&last.plate)
        .filter(|(a, b)| a != b)
        .count();
    assert!(moved > nc / 20, "plates moved under {moved} cells");
    // Today: some land, some sea, polar ice, warm tropics.
    let land = last.elevation_m.iter().filter(|z| **z > 0.0).count();
    assert!(land > nc / 20 && land < nc / 2, "land {land} of {nc}");
    assert!(last.ice_m.iter().any(|x| *x > 500.0), "no ice sheet");
    let tmax = last.temp_k.iter().cloned().fold(0.0f32, f32::max);
    assert!(tmax > 290.0, "warmest {tmax}");
    // The Earth fixture's climate is a measured value, so its temperature
    // history is listed as pending, not invented.
    assert!(f.pending.iter().any(|p| p.contains("temperature history")));
    eprintln!(
        "{} frames, {} events; {}",
        f.frames.len(),
        f.events.len(),
        last.why[0]
    );
    assert!(f.events.windows(2).all(|w| w[0].t_myr <= w[1].t_myr));
    assert!(!f.pending.is_empty());
    assert_eq!(f, frames_of("Earth"));
}

/// A world whose plates never moved still gets frames, on one surface.
#[test]
fn still_worlds_get_frames() {
    let f = frames_of("Moon");
    assert!(f.frames.len() > 5);
    assert!(f.why.iter().any(|w| w.contains("never moved")));
    assert!(f
        .events
        .iter()
        .any(|e| e.kind == "impact" && e.diameter_km.is_some()));
    assert!(f.events.iter().any(|e| e.kind == "flood-lava"));
    let a = &f.frames[0].elevation_m;
    assert!(f.frames.iter().all(|fr| &fr.elevation_m == a));
}

/// A frame limit keeps evenly spaced frames and always the last.
#[test]
fn frame_limit_keeps_the_last() {
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == "Mars")
        .expect("fixture")
        .body;
    let at = |mx| {
        surface_frames(
            &settings(),
            &b,
            &Seed::from_text("sse4-frames-tests"),
            b.age_gyr,
            &SurveyOptions::default(),
            false,
            mx,
        )
        .expect("frames")
    };
    let all = at(None);
    let few = at(Some(3));
    assert_eq!(few.frames.len(), 3.min(all.frames.len()));
    // The same last date and ground (vents decay over the spacing, so
    // they differ with it).
    let (a, b) = (few.frames.last().unwrap(), all.frames.last().unwrap());
    assert_eq!(
        (a.t_myr, &a.elevation_m, &a.temp_k),
        (b.t_myr, &b.elevation_m, &b.temp_k)
    );
    assert_eq!(at(Some(1)).frames.len(), 1);
}
