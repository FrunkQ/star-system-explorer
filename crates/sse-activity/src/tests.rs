use crate::*;

fn seed() -> Seed {
    Seed::from_text("activity-tests")
}

fn sun() -> Star {
    Star {
        address: "u:sol/sys:0/star:0".into(),
        mass_kg: M_SUN,
        radius_m: R_SUN,
        luminosity_w: L_SUN,
        teff_k: 5772.0,
        age_s: 4.57e9 * JULIAN_YEAR,
        rotation_period_s: Some(26.09 * DAY),
        companion_period_s: None,
        cycle_period_s: None,
    }
}

/// EV Lacertae: a young, fast-spinning M3.5 red dwarf (4.38-day spin).
fn ev_lac() -> Star {
    Star {
        address: "u:t/sys:1/star:0".into(),
        mass_kg: 0.32 * M_SUN,
        radius_m: 0.35 * R_SUN,
        luminosity_w: 0.013 * L_SUN,
        teff_k: 3300.0,
        age_s: 3e8 * JULIAN_YEAR,
        rotation_period_s: Some(4.38 * DAY),
        companion_period_s: None,
        cycle_period_s: None,
    }
}

fn earth() -> Target {
    Target {
        address: "u:sol/sys:0/body:2".into(),
        distance_m: AU,
        period_s: Some(JULIAN_YEAR),
        phase_rad: 0.0,
    }
}

/// The Sun: moderate, X-rays near the observed 10^-6.24 of its power
/// (1e20 to 5e20 W in soft X-rays over a cycle), flares of 10³² erg or more
/// about once a year, superflares about once a century.
#[test]
fn the_sun() {
    let s = Settings::default();
    let a = activity(&seed(), &sun(), &s).unwrap();
    assert_eq!(a.regime, "moderate");
    let log_rx = m::log10(a.xray_fraction);
    assert!((log_rx + 6.24).abs() < 0.1, "{log_rx}");
    assert!(a.xray_w > 1e20 && a.xray_w < 5e20, "{}", a.xray_w);
    let c = a.cycle.as_ref().unwrap();
    assert_eq!(c.branch, "inactive");
    // The inactive branch gives 6.1 years; the source itself says the Sun is an outlier.
    assert!((c.period_s / JULIAN_YEAR - 6.1).abs() < 0.3);
    let mo = moment(&a, sun().age_s, &s);
    let mean_rate = act::flare_rate_above(a.xray_w, 1e25, act::FLARE_K, 2.0) * JULIAN_YEAR;
    assert!(mean_rate > 0.1 && mean_rate < 10.0, "{mean_rate}");
    assert!(mo.flares_per_year_above_1e27_j > 2e-3 && mo.flares_per_year_above_1e27_j < 2e-2);
    assert!(a.spot_coverage > 0.002 && a.spot_coverage < 0.005);
    // An authored cycle wins.
    let mut s11 = sun();
    s11.cycle_period_s = Some(11.0 * JULIAN_YEAR);
    assert_eq!(
        activity(&seed(), &s11, &s).unwrap().cycle.unwrap().branch,
        "authored"
    );
    // Spin from age alone lands on the Sun's own spin.
    let mut aged = sun();
    aged.rotation_period_s = None;
    let p = activity(&seed(), &aged, &s).unwrap().rotation_period_s / DAY;
    assert!((p - 26.09).abs() < 0.01);
}

/// A young red dwarf is saturated: X-rays at 10^-3.13 of its power, no clean
/// cycle, and hundreds of flares a year.
#[test]
fn young_red_dwarf_is_saturated() {
    let s = Settings::default();
    let a = activity(&seed(), &ev_lac(), &s).unwrap();
    assert_eq!(a.regime, "saturated");
    assert!((m::log10(a.xray_fraction) + 3.13).abs() < 1e-9);
    assert!(a.cycle.is_none());
    let per_year = moment(&a, 0.0, &s).flares_per_year_above_1e25_j;
    assert!(per_year > 5.0, "{per_year}");
}

/// The young Sun: saturated at 20 million years, active but past saturation by 100 million.
#[test]
fn the_young_sun_spins_down() {
    let s = Settings::default();
    let mut y = sun();
    y.rotation_period_s = None;
    y.age_s = 2e7 * JULIAN_YEAR;
    assert_eq!(activity(&seed(), &y, &s).unwrap().regime, "saturated");
    y.age_s = 1e8 * JULIAN_YEAR;
    let a = activity(&seed(), &y, &s).unwrap();
    assert_ne!(a.regime, "saturated");
    assert!(a.xray_fraction > 1e-5);
}

/// Conditions, not kinds: hot stars, compact remnants and tidally spun stars.
#[test]
fn conditions_decide() {
    let s = Settings::default();
    let mut vega = sun();
    vega.teff_k = 9600.0;
    vega.mass_kg = 2.1 * M_SUN;
    assert_eq!(activity(&seed(), &vega, &s).unwrap().regime, "no-corona");
    let mut rigel = vega.clone();
    rigel.teff_k = 12_100.0;
    let a = activity(&seed(), &rigel, &s).unwrap();
    assert_eq!(a.regime, "hot-wind");
    assert!(!a.flares);
    let mut wd = sun();
    wd.radius_m = 0.012 * R_SUN;
    assert_eq!(activity(&seed(), &wd, &s).unwrap().regime, "compact");
    let mut old_binary = sun();
    old_binary.age_s = 8e9 * JULIAN_YEAR;
    old_binary.rotation_period_s = None;
    old_binary.companion_period_s = Some(2.0 * DAY);
    let a = activity(&seed(), &old_binary, &s).unwrap();
    assert_eq!(a.rotation_from, "tides");
    assert!(a.xray_fraction > 1e-4, "{}", a.xray_fraction);
}

/// Flare energies follow the power law: each decade about a tenth as many as the one below.
#[test]
fn flare_energies_follow_the_power_law() {
    let s = Settings::default();
    let a = activity(&seed(), &ev_lac(), &s).unwrap();
    let f = flares(&seed(), &a, 0.0, 10.0 * JULIAN_YEAR, 1e24, &[], &s).unwrap();
    let count = |lo: f64, hi: f64| {
        f.flares
            .iter()
            .filter(|x| x.energy_j >= lo && x.energy_j < hi)
            .count() as f64
    };
    let (n24, n25, n26) = (count(1e24, 1e25), count(1e25, 1e26), count(1e26, 1e27));
    assert!(n24 > 1000.0, "{n24}");
    assert!((n24 / n25 - 10.0).abs() < 2.5, "{n24} {n25}");
    assert!(n26 > 3.0 && n26 < 40.0, "{n26}");
    // Expected count matches what was drawn to a few percent.
    let expected = act::flare_rate_above(a.xray_w, 1e24, act::FLARE_K, 2.0) * 10.0 * JULIAN_YEAR;
    let total = f.flares.len() as f64;
    assert!(
        (total / expected - 1.0).abs() < 0.08,
        "{total} vs {expected}"
    );
    // Light: the peak power integrates to the white-light flash plus the X-ray share.
    let fl = &f.flares[0];
    let tab = sse_light::table::LightTable::base();
    let p = sse_light::publish(&fl.light, &tab);
    assert!(
        (p.total_w / (fl.peak_w * 1.1) - 1.0).abs() < 0.02,
        "{} {}",
        p.total_w,
        fl.peak_w
    );
}

/// Same flares whatever window or lowest energy is asked for.
#[test]
fn flares_are_keyed_not_streamed() {
    let s = Settings::default();
    let a = activity(&seed(), &ev_lac(), &s).unwrap();
    let key = |f: &Flares, lo: f64, hi: f64, e: f64| -> Vec<(u64, u64)> {
        f.flares
            .iter()
            .filter(|x| x.at_s >= lo && x.at_s <= hi && x.energy_j >= e)
            .map(|x| (x.at_s.to_bits(), x.energy_j.to_bits()))
            .collect()
    };
    let y = JULIAN_YEAR;
    let long = flares(&seed(), &a, 0.0, 2.0 * y, 1e24, &[], &s).unwrap();
    let short = flares(&seed(), &a, 0.5 * y, y, 1e24, &[], &s).unwrap();
    assert_eq!(key(&long, 0.5 * y, y, 1e24), key(&short, 0.5 * y, y, 1e24));
    let big = flares(&seed(), &a, 0.0, 2.0 * y, 3e25, &[], &s).unwrap();
    assert_eq!(
        key(&long, 0.0, 2.0 * y, 3e25),
        key(&big, 0.0, 2.0 * y, 3e25)
    );
    assert!(!big.flares.is_empty());
}

/// Ejections hit or miss by geometry, and light gives warning before the particles.
#[test]
fn ejections_hit_or_miss_by_geometry() {
    let s = Settings::default();
    let a = activity(&seed(), &sun(), &s).unwrap();
    let f = flares(&seed(), &a, 0.0, 100.0 * JULIAN_YEAR, 3e25, &[earth()], &s).unwrap();
    let ej: Vec<&Ejection> = f
        .flares
        .iter()
        .filter_map(|x| x.ejection.as_ref())
        .collect();
    assert!(ej.len() > 10, "{}", ej.len());
    let hits: usize = ej.iter().map(|e| e.hits.len()).sum();
    let misses: usize = ej.iter().map(|e| e.misses.len()).sum();
    assert!(hits > 0 && misses > hits, "{hits} {misses}");
    for fl in &f.flares {
        if let Some(e) = &fl.ejection {
            for h in &e.hits {
                let there = earth().direction(h.arrives_s);
                assert!(dot(e.direction, there) >= m::cos(e.half_width_rad));
                let days = h.warning_s / DAY;
                assert!(days > 0.3 && days < 5.0, "{days}");
            }
        }
    }
}

#[test]
fn golden() {
    let s = Settings::default();
    let a = activity(&seed(), &ev_lac(), &s).unwrap();
    let tg = Target {
        address: "u:t/sys:1/body:1".into(),
        distance_m: 0.05 * AU,
        period_s: Some(5.0 * DAY),
        phase_rad: 1.0,
    };
    let f = flares(&seed(), &a, 0.0, 30.0 * DAY, 3e24, &[tg], &s).unwrap();
    let text = serde_json::to_string_pretty(&(a, f)).unwrap() + "\n";
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/activity.json"
    );
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    let want = std::fs::read_to_string(path).expect("golden exists (bless with SSE_BLESS=1)");
    assert_eq!(text, want);
}
