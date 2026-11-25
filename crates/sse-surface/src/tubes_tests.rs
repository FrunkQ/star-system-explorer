use crate::scenario::{fixtures, settings};
use crate::survey::{survey_world, SurveyOptions};
use sse_core::draw::Seed;

fn tubes_of(name: &str) -> Vec<crate::tubes::TubeField> {
    let s = settings();
    let b = fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .expect("fixture")
        .body;
    survey_world(
        &s,
        &b,
        &Seed::from_text("sse4-tubes-tests"),
        b.age_gyr,
        &SurveyOptions::default(),
    )
    .expect("survey")
    .tubes
}

/// Tubes form best on gentle slopes, not on steep flanks or flat sheets.
#[test]
fn tubes_form_on_gentle_slopes() {
    let t = crate::tubes::TubeSettings::default();
    let f = |x| crate::tubes::forming_strength(&t, x);
    assert!((f(2.0) - 1.0).abs() < 1e-12);
    assert!(f(0.1) < 0.05 && f(20.0) < 0.05, "{} {}", f(0.1), f(20.0));
    assert!(f(1.0) > 0.7 && f(4.0) > 0.7);
}

/// A roof spans wider as gravity falls; airless worlds keep old roofs only
/// where craters have not cut through; Earth's young hot-spot tubes stand.
#[test]
fn tube_roofs_scale_with_gravity_and_wear_with_time() {
    let moon = tubes_of("Moon");
    let earth = tubes_of("Earth");
    let mars = tubes_of("Mars");
    assert!(!moon.is_empty() && !earth.is_empty() && !mars.is_empty());
    let ratio = moon[0].width_max_m / earth[0].width_max_m;
    assert!(
        (ratio - 9.82 / 1.62).abs() < 0.2,
        "Moon to Earth width {ratio}"
    );
    for x in moon.iter().chain(&mars).chain(&earth) {
        assert!(x.width_m <= x.width_max_m + 1e-9 && x.roof_m > 0.0);
        assert!((0.0..=1.0).contains(&x.roof_kept) && (0.0..=1.0).contains(&x.forming));
        let want = match x.roof_kept {
            k if k >= 0.9 => "intact roof",
            k if k >= 0.5 => "skylights",
            k if k >= 0.1 => "pit chains",
            _ => "sinuous rille",
        };
        assert_eq!(x.stage, want);
        assert!(x.cause.contains("Blair et al. 2017"));
    }
    // Earth's are the young ones, still roofed.
    assert!(
        earth.iter().all(|x| x.roof_kept > 0.9),
        "{:?}",
        earth.iter().map(|x| x.roof_kept).collect::<Vec<_>>()
    );
    // The Moon's oldest maria have lost their roofs to craters; some kept.
    assert!(moon.iter().any(|x| x.stage == "sinuous rille"));
    assert!(moon.iter().any(|x| x.roof_kept >= 0.5));
    // S14a part 3: only bowls deeper than the roof hole it, so the younger
    // maria keep whole roofs (Haruyama et al. 2009: the Marius Hills hole
    // opens into a roofed void in mare some 3.5 Gyr old).
    assert!(moon.iter().any(|x| x.stage == "intact roof"));
    // No lava, no tubes.
    assert!(tubes_of("Europa").is_empty());
    // The same world gives the same tubes.
    assert_eq!(tubes_of("Moon"), moon);
}
