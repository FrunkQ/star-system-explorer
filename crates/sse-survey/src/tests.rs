use crate::*;

const AU: f64 = sse_core::quantity::consts::AU;
const PC: f64 = 3.085_677_581e16;
const MSUN: f64 = 1.988_41e30;
const MEARTH: f64 = 5.9722e24;
const MJUP: f64 = 1.898_19e27;

fn sun() -> Star {
    Star {
        address: "u:sol/sys:0/star:0".into(),
        mass_kg: MSUN,
        radius_m: 6.957e8,
        visible_w: 0.437_75 * 3.828e26,
        activity_ppm_hr: 15.0,
        rv_jitter_m_s: 1.0,
        timing_precision_s: None,
    }
}

fn planets() -> Vec<Planet> {
    let deg = sse_core::math::PI / 180.0;
    vec![
        Planet {
            address: "u:sol/sys:0/body:2".into(),
            name: Some("Earth".into()),
            mass_kg: MEARTH,
            radius_m: 6.371e6,
            a_m: AU,
            e: 0.0167,
            i_rad: 0.0,
            node_rad: 0.0,
            phase_rad: 0.3,
            geometric_albedo: 0.434,
        },
        Planet {
            address: "u:sol/sys:0/body:4".into(),
            name: Some("Jupiter".into()),
            mass_kg: MJUP,
            radius_m: 6.9911e7,
            a_m: 5.2026 * AU,
            e: 0.0489,
            i_rad: 1.303 * deg,
            node_rad: 100.464 * deg,
            phase_rad: 2.0,
            geometric_albedo: 0.538,
        },
    ]
}

fn observer(direction: [f64; 3]) -> Observer {
    Observer {
        direction,
        distance_m: 10.0 * PC,
    }
}

fn kepler_like() -> Instrument {
    Instrument {
        name: "space photometer, 0.95 m".into(),
        aperture_m: 0.95,
        efficiency: 0.5,
        photometric_floor_ppm_hr: 10.0,
        rv_precision_m_s: None,
        astrometric_precision_uas: None,
        contrast_floor: None,
        imaging_wavelength_m: 1.6e-6,
        visits_per_day: 0.0,
        duty_cycle: 0.9,
    }
}

fn spectro_astro() -> Instrument {
    Instrument {
        name: "spectrograph and astrometric array".into(),
        aperture_m: 4.0,
        efficiency: 0.5,
        photometric_floor_ppm_hr: 1000.0,
        rv_precision_m_s: Some(1.0),
        astrometric_precision_uas: Some(20.0),
        contrast_floor: None,
        imaging_wavelength_m: 1.6e-6,
        visits_per_day: 0.2,
        duty_cycle: 0.0,
    }
}

fn find(r: &SurveyResult, planet: &str, m: Method) -> Finding {
    r.findings
        .iter()
        .find(|f| f.planet.ends_with(planet) && f.method == m)
        .cloned()
        .unwrap()
}

/// The textbook numbers: Jupiter swings the Sun by 12.5 m/s and 500 µas at
/// 10 pc; Earth by 9 cm/s; Earth's transit is 84 ppm deep and 13 hours long.
#[test]
fn solar_system_signals() {
    let s = Settings::default();
    let o = observer([1.0, 0.0, 0.0]);
    let ps = planets();
    let jw = wobble(&sun(), &ps[1], &o, &spectro_astro(), 12.0 * YEAR, &s).unwrap();
    assert!(
        jw.why.contains("12.4") || jw.why.contains("12.5"),
        "{}",
        jw.why
    );
    let ew = wobble(&sun(), &ps[0], &o, &spectro_astro(), 12.0 * YEAR, &s).unwrap();
    assert!(ew.why.contains("0.089"), "{}", ew.why);
    let ja = position_wobble(&sun(), &ps[1], &o, &spectro_astro(), 12.0 * YEAR, &s).unwrap();
    assert!(
        ja.why.contains("circle 99") || ja.why.contains("circle 100"),
        "{}",
        ja.why
    );
    let et = transit(&sun(), &ps[0], &o, &kepler_like(), 4.0 * YEAR, &s);
    assert!(et.why.contains("dimming it by 84 ppm"), "{}", et.why);
    assert!(et.why.contains("for 13."), "{}", et.why);
}

/// From the ecliptic plane: Earth by transit with four years of a Kepler-like
/// photometer; Jupiter by wobble and position with twelve years of watching.
#[test]
fn from_the_ecliptic() {
    let o = observer([1.0, 0.0, 0.0]);
    let photo = run(&Survey {
        star: sun(),
        planets: planets(),
        observer: o.clone(),
        instrument: kepler_like(),
        watch_s: 4.0 * YEAR,
        settings: None,
    });
    assert_eq!(
        find(&photo, "body:2", Method::Transit).verdict,
        Verdict::Found
    );
    let spec = run(&Survey {
        star: sun(),
        planets: planets(),
        observer: o,
        instrument: spectro_astro(),
        watch_s: 12.0 * YEAR,
        settings: None,
    });
    assert_eq!(
        find(&spec, "body:4", Method::Wobble).verdict,
        Verdict::Found
    );
    assert_eq!(
        find(&spec, "body:4", Method::PositionWobble).verdict,
        Verdict::Found
    );
    assert_eq!(
        find(&spec, "body:2", Method::Wobble).verdict,
        Verdict::Missed
    );
    // Players see the minimum mass from the wobble, never the true mass.
    let w = find(&spec, "body:4", Method::Wobble);
    let msini = w
        .measured
        .iter()
        .find(|x| x.what.starts_with("minimum"))
        .unwrap()
        .value;
    assert!(msini <= MJUP && msini > 0.99 * MJUP);
}

/// One degree above the plane Earth no longer transits (its window is
/// R☉/a = 0.27°), and the result changes as the observer moves.
#[test]
fn earth_transits_only_from_near_its_plane() {
    let s = Settings::default();
    let ps = planets();
    let deg = sse_core::math::PI / 180.0;
    let at = |lat: f64| {
        let d = [
            sse_core::math::cos(lat * deg),
            0.0,
            sse_core::math::sin(lat * deg),
        ];
        transit(&sun(), &ps[0], &observer(d), &kepler_like(), 4.0 * YEAR, &s).verdict
    };
    assert_eq!(at(0.0), Verdict::Found);
    assert_eq!(at(0.2), Verdict::Found);
    assert_eq!(at(1.0), Verdict::Missed);
    assert_eq!(at(30.0), Verdict::Missed);
}

/// Face-on: no transits and no wobble at all, but the position wobble is a
/// full circle and still finds Jupiter.
#[test]
fn face_on_systems_hide_from_transit_and_wobble() {
    let s = Settings::default();
    let ps = planets();
    let o = observer([0.0, 0.0, 1.0]);
    let ins = spectro_astro();
    let mut tilted = ps[1].clone();
    tilted.i_rad = 0.0;
    assert_eq!(
        transit(&sun(), &tilted, &o, &kepler_like(), 12.0 * YEAR, &s).verdict,
        Verdict::Missed
    );
    let w = wobble(&sun(), &tilted, &o, &ins, 12.0 * YEAR, &s).unwrap();
    assert_eq!(w.verdict, Verdict::Missed);
    assert!(w.snr < 1e-6);
    assert_eq!(
        position_wobble(&sun(), &tilted, &o, &ins, 12.0 * YEAR, &s)
            .unwrap()
            .verdict,
        Verdict::Found
    );
}

/// Too short a watch: a strong wobble is only a candidate until a full orbit
/// is seen; a single transit cannot fix a period.
#[test]
fn watch_time_matters() {
    let s = Settings::default();
    let ps = planets();
    let o = observer([1.0, 0.0, 0.0]);
    let w = wobble(&sun(), &ps[1], &o, &spectro_astro(), 6.0 * YEAR, &s).unwrap();
    assert_eq!(w.verdict, Verdict::Candidate);
    let t = transit(&sun(), &ps[0], &o, &kepler_like(), 1.2 * YEAR, &s);
    assert_ne!(t.verdict, Verdict::Found, "{}", t.why);
}

/// Direct imaging: a big telescope with a fine coronagraph sees Jupiter at
/// 10 pc (0.5″ away, 1e-9 contrast); one with 1e-6 contrast does not; a small
/// one cannot separate Earth from the Sun at all at 100 pc.
#[test]
fn direct_imaging_needs_reach_and_contrast() {
    let ps = planets();
    let mut ins = spectro_astro();
    ins.aperture_m = 8.0;
    ins.contrast_floor = Some(1e-10);
    let o = observer([1.0, 0.0, 0.0]);
    assert_eq!(
        direct_imaging(&sun(), &ps[1], &o, &ins).unwrap().verdict,
        Verdict::Found
    );
    ins.contrast_floor = Some(1e-6);
    assert_eq!(
        direct_imaging(&sun(), &ps[1], &o, &ins).unwrap().verdict,
        Verdict::Missed
    );
    let mut small = ins.clone();
    small.aperture_m = 0.3;
    small.contrast_floor = Some(1e-12);
    let far = Observer {
        direction: [1.0, 0.0, 0.0],
        distance_m: 100.0 * PC,
    };
    let f = direct_imaging(&sun(), &ps[0], &far, &small).unwrap();
    assert_eq!(f.verdict, Verdict::Missed);
    assert!(f.why.contains("glare"));
}

/// Pulsar planets (Wolszczan and Frail 1992): a few Earth masses round a
/// pulsar with microsecond timing is easy.
#[test]
fn pulsar_timing_finds_small_planets() {
    let s = Settings::default();
    let mut psr = sun();
    psr.mass_kg = 1.4 * MSUN;
    psr.radius_m = 1e4;
    psr.timing_precision_s = Some(3e-6);
    let p = Planet {
        address: "u:t/sys:0/body:1".into(),
        name: Some("PSR B1257+12 c".into()),
        mass_kg: 4.3 * MEARTH,
        radius_m: 1e7,
        a_m: 0.36 * AU,
        e: 0.0,
        i_rad: 0.9,
        node_rad: 0.0,
        phase_rad: 0.0,
        geometric_albedo: 0.3,
    };
    let f = timing(
        &psr,
        &p,
        &observer([1.0, 0.0, 0.0]),
        &spectro_astro(),
        2.0 * YEAR,
        &s,
    )
    .unwrap();
    assert_eq!(f.verdict, Verdict::Found, "{}", f.why);
}

#[test]
fn golden() {
    let r = run(&Survey {
        star: sun(),
        planets: planets(),
        observer: observer([0.6, 0.0, 0.8]),
        instrument: spectro_astro(),
        watch_s: 12.0 * YEAR,
        settings: None,
    });
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/survey.json"
    );
    let text = serde_json::to_string_pretty(&r).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    let want = std::fs::read_to_string(path).expect("golden exists (bless with SSE_BLESS=1)");
    assert_eq!(text, want);
}
