use crate::*;

/// A date from seconds as a double (tests only; dates are whole seconds).
fn d(x: f64) -> Instant {
    Instant::from_seconds_f64(x).unwrap()
}

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
    let mo = moment(&a, d(sun().age_s), &s);
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
    let per_year = moment(&a, Instant::ORIGIN, &s).flares_per_year_above_1e25_j;
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
    let f = flares(&seed(), &a, d(0.0), d(10.0 * JULIAN_YEAR), 1e24, &[], &s).unwrap();
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
    let key = |f: &Flares, lo: f64, hi: f64, e: f64| -> Vec<(Instant, u64)> {
        f.flares
            .iter()
            .filter(|x| x.at_s >= lo && x.at_s <= hi && x.energy_j >= e)
            .map(|x| (x.at, x.energy_j.to_bits()))
            .collect()
    };
    let y = JULIAN_YEAR;
    let long = flares(&seed(), &a, d(0.0), d(2.0 * y), 1e24, &[], &s).unwrap();
    let short = flares(&seed(), &a, d(0.5 * y), d(y), 1e24, &[], &s).unwrap();
    assert_eq!(key(&long, 0.5 * y, y, 1e24), key(&short, 0.5 * y, y, 1e24));
    let big = flares(&seed(), &a, d(0.0), d(2.0 * y), 3e25, &[], &s).unwrap();
    assert_eq!(
        key(&long, 0.0, 2.0 * y, 3e25),
        key(&big, 0.0, 2.0 * y, 3e25)
    );
    assert!(!big.flares.is_empty());
}

/// TP1b: at a star's age a flare's date still falls on any second (before,
/// the date was a double, a multiple of 32 s at 4.6 Gyr), and moving the
/// window's start by seconds keeps the same flares.
#[test]
fn flares_keep_their_seconds_at_a_stars_age() {
    let s = Settings::default();
    let a = activity(&seed(), &ev_lac(), &s).unwrap();
    let age = Instant::seconds(4_610_000_000 * sse_core::time::YEAR_S);
    let to = age.checked_add_seconds(sse_core::time::YEAR_S).unwrap();
    let f = flares(&seed(), &a, age, to, 3e24, &[], &s).unwrap();
    assert!(f.flares.len() > 10, "{}", f.flares.len());
    let off_grid = f.flares.iter().filter(|x| x.at.s % 32 != 0).count();
    assert!(
        off_grid > f.flares.len() / 2,
        "{off_grid} of {}",
        f.flares.len()
    );
    assert!(f.flares.iter().all(|x| x.at >= age && x.at <= to));
    let later = age.checked_add_seconds(7).unwrap();
    let g = flares(&seed(), &a, later, to, 3e24, &[], &s).unwrap();
    let dates = |f: &Flares| -> Vec<Instant> {
        f.flares
            .iter()
            .map(|x| x.at)
            .filter(|t| *t >= later)
            .collect()
    };
    assert_eq!(dates(&f), dates(&g));
}

/// Ejections hit or miss by geometry, and light gives warning before the particles.
#[test]
fn ejections_hit_or_miss_by_geometry() {
    let s = Settings::default();
    let a = activity(&seed(), &sun(), &s).unwrap();
    let f = flares(
        &seed(),
        &a,
        d(0.0),
        d(100.0 * JULIAN_YEAR),
        3e25,
        &[earth()],
        &s,
    )
    .unwrap();
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
                let there = earth().direction(h.arrives);
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
    let f = flares(&seed(), &a, d(0.0), d(30.0 * DAY), 3e24, &[tg], &s).unwrap();
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

fn star(mass: f64, radius: f64, teff: f64) -> Star {
    let lum =
        4.0 * sse_core::math::PI * radius * radius * 5.670_374e-8 * (teff * teff) * (teff * teff);
    Star {
        address: "u:x/sys:0/star:0".into(),
        mass_kg: mass * M_SUN,
        radius_m: radius * R_SUN,
        luminosity_w: lum * R_SUN * R_SUN,
        teff_k: teff,
        age_s: 1e9 * JULIAN_YEAR,
        rotation_period_s: Some(30.0 * DAY),
        companion_period_s: None,
        cycle_period_s: None,
    }
}

/// Surface cells against the Sun's granules (Nordlund et al. 2009: 1 to 2
/// Mm, minutes) and supergranules (Rieutord and Rincon 2010: about 30 Mm,
/// one to two days); a few giant cells across Betelgeuse (Freytag et al.
/// 2002; Chiavassa et al. 2010); none above the granulation boundary (Gray
/// and Nagel 1989); kilometre cells on a cool white dwarf (Tremblay et al. 2013).
#[test]
fn surface_cells_from_temperature_and_gravity() {
    let set = Settings::default();
    let c = surface_convection(&sun(), &set).expect("the Sun boils");
    assert!(
        (1.0e6..=2.0e6).contains(&c.granule_size_m),
        "{}",
        c.granule_size_m
    );
    assert!(
        (240.0..=900.0).contains(&c.granule_lifetime_s),
        "{}",
        c.granule_lifetime_s
    );
    assert!(
        (20e6..=40e6).contains(&c.supergranule_size_m),
        "{}",
        c.supergranule_size_m
    );
    assert!(
        (0.5 * DAY..=2.5 * DAY).contains(&c.supergranule_lifetime_s),
        "{}",
        c.supergranule_lifetime_s
    );
    assert!(
        (2000.0..=10000.0).contains(&c.network_cells),
        "{}",
        c.network_cells
    );
    // Betelgeuse: 18 solar masses, 764 solar radii, 3600 K (Joyce et al. 2020).
    let b = surface_convection(&star(18.0, 764.0, 3600.0), &set).unwrap();
    // The large cells number only a handful over the face and live years;
    // the smaller granules beneath them are tens across and live weeks.
    assert!(b.network_cells < 30.0, "{}", b.network_cells);
    assert!(
        b.supergranule_lifetime_s > JULIAN_YEAR,
        "{}",
        b.supergranule_lifetime_s
    );
    assert!(b.granules_across < 100.0, "{}", b.granules_across);
    assert!(
        b.granule_lifetime_s > 10.0 * DAY,
        "{}",
        b.granule_lifetime_s
    );
    assert!(b.speed_m_s <= b.sound_speed_m_s);
    // A young red dwarf: many small cells, slow churning.
    let m = surface_convection(&ev_lac(), &set).unwrap();
    assert!(m.granule_size_m < 0.3 * c.granule_size_m && m.speed_m_s < c.speed_m_s);
    // Vega-like A star: no surface convection.
    assert!(surface_convection(&star(2.1, 2.4, 9600.0), &set).is_none());
    // A cool white dwarf: granules about a kilometre across.
    let w = surface_convection(&star(0.6, 0.0125, 6000.0), &set).unwrap();
    assert!(
        (0.1e3..=10e3).contains(&w.granule_size_m),
        "{}",
        w.granule_size_m
    );
    // It reaches the activity read-out, with its why line.
    let a = activity(&seed(), &sun(), &set).unwrap();
    assert!(a.convection.is_some());
    assert!(a.why.iter().any(|w| w.contains("granules")));
}

mod prominences {
    use super::*;
    use crate::prominence::*;

    fn ab_dor() -> Star {
        Star {
            address: "u:t/sys:2/star:0".into(),
            mass_kg: 0.86 * M_SUN,
            radius_m: 0.96 * R_SUN,
            luminosity_w: 0.4 * L_SUN,
            teff_k: 5000.0,
            age_s: 5e7 * JULIAN_YEAR,
            rotation_period_s: Some(0.514 * DAY),
            companion_period_s: None,
            cycle_period_s: None,
        }
    }

    fn run(s: &Star, from_d: f64, to_d: f64) -> Prominences {
        let set = Settings::default();
        let a = activity(&seed(), s, &set).unwrap();
        prominences(
            &seed(),
            s,
            &a,
            s.age_s + from_d * DAY,
            s.age_s + to_d * DAY,
            &ProminenceSettings::default(),
            &set,
        )
        .unwrap()
    }

    #[test]
    fn the_sun_holds_a_dozen_cool_clouds_tens_of_thousands_of_km_up() {
        let p = run(&sun(), 0.0, 365.0);
        let c = p.corona.clone().unwrap();
        assert!((0.005..0.02).contains(&c.surface_field_t), "{c:?}");
        assert!((1.3e6..2.2e6).contains(&c.temperature_k), "{c:?}");
        assert!(!c.slingshots, "{c:?}");
        assert!((5.0..30.0).contains(&p.mean_at_once), "{}", p.mean_at_once);
        assert!(!p.prominences.is_empty());
        for q in &p.prominences {
            assert!(q.kind != "slingshot");
            // Absolute: solar prominences are 3 to 150 thousand km high, 10¹⁶ to 10¹⁸ per m³.
            assert!((3e6..1.5e8).contains(&q.height_m), "{q:?}");
            assert!((5e15..1e18).contains(&q.density_m3), "{q:?}");
            assert!((1e9..1e14).contains(&q.mass_kg), "{q:?}");
            assert!((1e-6..1e-3).contains(&q.share_visible), "{q:?}");
            assert_eq!(q.light.lines.len(), 2);
        }
        // About the mean number alive at the window's middle (Poisson spread and the cycle allowed).
        let mid = sun().age_s + 182.0 * DAY;
        let alive = p
            .prominences
            .iter()
            .filter(|q| q.born_s <= mid && q.born_s + q.life_s >= mid)
            .count() as f64;
        assert!(
            alive > 0.1 * p.mean_at_once && alive < 4.0 * p.mean_at_once,
            "{alive} against {}",
            p.mean_at_once
        );
    }

    #[test]
    fn a_fast_spinner_slings_clouds_at_its_corotation_radius() {
        let p = run(&ab_dor(), 0.0, 5.0);
        let c = p.corona.clone().unwrap();
        assert!(c.slingshots, "{c:?}");
        assert!((2.0..4.0).contains(&c.corotation_radii), "{c:?}");
        let sl: Vec<_> = p
            .prominences
            .iter()
            .filter(|q| q.kind == "slingshot")
            .collect();
        assert!(!sl.is_empty());
        // A saturated star's long window is cut, earliest kept, and says how far it is whole.
        let long = run(&ab_dor(), 0.0, 90.0);
        assert_eq!(
            long.prominences.len(),
            ProminenceSettings::default().max_listed
        );
        assert!(long.complete_to_s > ab_dor().age_s + 20.0 * DAY && long.complete_to_s < long.to_s);
        for q in sl {
            assert!(q.eruption.is_some());
            assert!((q.height_m / ab_dor().radius_m - (c.corotation_radii - 1.0)).abs() < 1e-9);
        }
    }

    #[test]
    fn overlapping_windows_agree_and_hot_stars_have_none() {
        let a = run(&sun(), 0.0, 100.0);
        let b = run(&sun(), 50.0, 150.0);
        let key = |q: &Prominence| (q.born_s.to_bits(), q.life_s.to_bits(), q.height_m.to_bits());
        let mid = sun().age_s + 75.0 * DAY;
        let alive = |p: &Prominences| {
            let mut v: Vec<_> = p
                .prominences
                .iter()
                .filter(|q| q.born_s <= mid && q.born_s + q.life_s >= mid)
                .map(key)
                .collect();
            v.sort();
            v
        };
        assert_eq!(alive(&a), alive(&b));
        assert!(!alive(&a).is_empty());
        let mut hot = sun();
        hot.teff_k = 9500.0;
        hot.mass_kg = 2.0 * M_SUN;
        let p = run(&hot, 0.0, 30.0);
        assert!(p.prominences.is_empty() && p.corona.is_none());
    }
}

mod active_regions {
    use super::*;
    use crate::regions::*;

    fn sun_cycle() -> Star {
        Star {
            cycle_period_s: Some(11.0 * JULIAN_YEAR),
            ..sun()
        }
    }

    #[test]
    fn the_suns_regions_cover_its_spots_and_follow_the_butterfly() {
        let s = sun_cycle();
        let set = Settings::default();
        let a = activity(&seed(), &s, &set).unwrap();
        let c = a.cycle.clone().expect("a Sun-like cycle");
        let g = regions(&seed(), &a, s.age_s, s.age_s + 11.0 * JULIAN_YEAR, &set).unwrap();
        // Absolute: a handful to a few dozen at once, lasting days to months.
        assert!((3.0..40.0).contains(&g.mean_at_once), "{}", g.mean_at_once);
        assert!(g.regions.len() > 100, "{}", g.regions.len());
        for r in &g.regions {
            assert!(
                r.lat_deg.abs() <= 35.0 && r.lat_deg.abs() >= 2.0,
                "{}",
                r.lat_deg
            );
            assert!(r.life_s > 2.0 * DAY && r.life_s <= 200.0 * DAY + 1.0);
            // Joy's law: the leader is nearer the equator.
            assert!(
                r.tilt_deg * r.lat_deg < 10.0 * r.lat_deg.abs(),
                "{} at {}",
                r.tilt_deg,
                r.lat_deg
            );
        }
        // Spörer: early in the cycle high, late low.
        let progress = |t: f64| wrap(t / c.period_s + c.phase_offset - 0.75, 1.0);
        let mean = |lo: f64, hi: f64| {
            let v: Vec<f64> = g
                .regions
                .iter()
                .filter(|r| (lo..hi).contains(&progress(r.born_s)))
                .map(|r| r.lat_deg.abs())
                .collect();
            v.iter().sum::<f64>() / v.len().max(1) as f64
        };
        let (early, late) = (mean(0.05, 0.3), mean(0.7, 0.95));
        assert!(early > 20.0 && late < 15.0, "{early} then {late}");
        // Their spot area at any moment is the star's coverage, about.
        let mut sum = 0.0;
        let n = 200;
        for k in 0..n {
            let t = s.age_s + (k as f64 + 0.5) / n as f64 * 11.0 * JULIAN_YEAR;
            sum += g.regions.iter().map(|r| r.share_at(t)).sum::<f64>();
        }
        let mean_cov = sum / n as f64;
        assert!(
            (0.5 * a.spot_coverage..1.6 * a.spot_coverage).contains(&mean_cov),
            "{mean_cov} against {}",
            a.spot_coverage
        );
    }

    #[test]
    fn flares_and_prominences_sit_in_the_regions() {
        let s = ev_lac();
        let set = Settings::default();
        let a = activity(&seed(), &s, &set).unwrap();
        let f = flares(
            &seed(),
            &a,
            d(s.age_s),
            d(s.age_s + 30.0 * DAY),
            1e25,
            &[],
            &set,
        )
        .unwrap();
        let sited: Vec<&Flare> = f.flares.iter().filter(|x| x.site.is_some()).collect();
        assert!(!sited.is_empty() && sited.len() * 10 >= f.flares.len() * 9);
        for x in sited {
            let site = x.site.as_ref().unwrap();
            let alive = alive_at(&seed(), &a, x.at_s, &set).unwrap();
            let g = alive
                .iter()
                .find(|g| g.address == site.region)
                .expect("its region is alive");
            assert!((g.lat_deg - site.lat_deg).abs() < 1e-9);
            // An ejection leaves from over its region.
            if let Some(e) = &x.ejection {
                let lat = m::asin(e.direction[2]) * 180.0 / m::PI;
                assert!((lat - site.lat_deg).abs() < 1e-6, "{lat} {}", site.lat_deg);
            }
        }
        let p = crate::prominence::prominences(
            &seed(),
            &sun_cycle(),
            &activity(&seed(), &sun_cycle(), &set).unwrap(),
            sun_cycle().age_s,
            sun_cycle().age_s + 120.0 * DAY,
            &crate::prominence::ProminenceSettings::default(),
            &set,
        )
        .unwrap();
        let sa = activity(&seed(), &sun_cycle(), &set).unwrap();
        let active: Vec<_> = p
            .prominences
            .iter()
            .filter(|x| x.kind == "active")
            .collect();
        assert!(!active.is_empty());
        for x in &active {
            let alive = alive_at(&seed(), &sa, x.born_s, &set).unwrap();
            assert!(alive.iter().any(|g| Some(&g.address) == x.region.as_ref()));
        }
        assert!(p
            .prominences
            .iter()
            .filter(|x| x.kind == "quiescent")
            .all(|x| x.region.is_some()));
    }

    #[test]
    fn overlapping_windows_and_single_moments_agree() {
        let s = sun_cycle();
        let set = Settings::default();
        let a = activity(&seed(), &s, &set).unwrap();
        let t = s.age_s + 40.0 * DAY;
        let w = regions(&seed(), &a, s.age_s, s.age_s + 90.0 * DAY, &set).unwrap();
        let v = regions(&seed(), &a, t, t + 5.0 * DAY, &set).unwrap();
        let here: Vec<&str> = alive_at(&seed(), &a, t, &set)
            .unwrap()
            .iter()
            .map(|g| g.address.as_str())
            .collect::<Vec<_>>()
            .into_iter()
            .map(|x| {
                w.regions
                    .iter()
                    .find(|g| g.address == x)
                    .map(|g| g.address.as_str())
                    .expect("in the long window")
            })
            .collect();
        assert!(!here.is_empty());
        for x in &here {
            assert!(v.regions.iter().any(|g| g.address == *x));
        }
    }
}
