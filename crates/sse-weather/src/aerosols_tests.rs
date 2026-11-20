use super::*;
use crate::scenario::{base_weather_pack, block, fixture, settings};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fx {
    io: Conditions,
    io_sources: Sources,
    earth_sources: Sources,
}

fn fx() -> Fx {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/weather/aerosols.json"
    ))
    .expect("aerosol fixtures")
}

fn aset() -> AerosolSettings {
    AerosolSettings::from_packs(&[base_weather_pack()]).expect("aerosol settings")
}

fn cell(c: &Conditions, day: f64, lat: f64, lon: f64) -> Cell {
    let b = block();
    let p = b.prepare(c, Date { day });
    b.zonal_cell(c, &p, lat, lon)
}

fn at(c: &Conditions, src: &Sources, day: f64, lat: f64, lon: f64) -> (Cell, AerosolOut) {
    let mut x = cell(c, day, lat, lon);
    let o = imprint_aerosols(&aset(), &settings(), c, Date { day }, src, None, &mut x);
    (x, o)
}

fn tau(x: &Cell, kind: &str) -> f64 {
    x.aerosols
        .iter()
        .filter(|a| a.kind == kind)
        .map(|a| a.optical_depth)
        .sum()
}

fn eruption(id: &str, lat: f64, lon: f64, v_dre: f64, so2_tg: f64) -> Eruption {
    Eruption {
        id: id.into(),
        lat_deg: lat,
        lon_deg: lon,
        day: 100.0,
        duration_days: 5.0,
        vent_height_m: 0.0,
        mer_kg_s: Some(v_dre * 2500.0),
        volume_m3: None,
        so2_kg: so2_tg * 1.0e9,
        vent_speed_m_s: None,
    }
}

#[test]
fn mastin_formula_and_earth_plume_height() {
    let a = aset();
    let want = 2.00 * m::pow(1.0e4, 0.241);
    assert_eq!(mastin_height_km(&a, 1.0e4), want);
    assert!((want - 18.36).abs() < 0.05, "{want}");
    // Mass form 0.304 MER^0.241 agrees.
    let mass = 0.304 * m::pow(2.5e7, 0.241);
    assert!((mastin_height_mer_km(&a, 2.5e7) / mass - 1.0).abs() < 0.002);
    // On Earth the scaling is (nearly) one, so the plume is Mastin's height.
    let e = fixture("Earth");
    let f = atmosphere_factor(&a, &e);
    assert!((f - 1.0).abs() < 0.02, "{f}");
    let src = Sources {
        eruptions: vec![eruption("v", 45.0, 10.0, 1.0e4, 0.0)],
        ..Default::default()
    };
    let sys = plumes(&a, &settings(), &e, Date { day: 101.0 }, &src);
    assert_eq!(sys.len(), 1);
    assert_eq!(sys[0].kind, "plume");
    assert_eq!(sys[0].evidence, "rule");
    assert!(
        (sys[0].strength / want - 1.0).abs() < 0.02,
        "{}",
        sys[0].strength
    );
    // Before and after the eruption there is no plume.
    assert!(plumes(&a, &settings(), &e, Date { day: 99.0 }, &src).is_empty());
    assert!(plumes(&a, &settings(), &e, Date { day: 106.0 }, &src).is_empty());
}

#[test]
fn plumes_are_taller_in_thin_air_and_squat_in_thick() {
    let a = aset();
    let e = atmosphere_factor(&a, &fixture("Earth"));
    let mars = atmosphere_factor(&a, &fixture("Mars"));
    let venus = atmosphere_factor(&a, &fixture("Venus"));
    assert!(mars > 2.0 * e, "Mars {mars}");
    assert!(venus < 0.6 * e, "Venus {venus}");
}

#[test]
fn io_plumes_are_ballistic() {
    let a = aset();
    let f = fx();
    let sys = plumes(&a, &settings(), &f.io, Date { day: 15.0 }, &f.io_sources);
    let h = |id: &str| sys.iter().find(|s| s.id == id).expect(id).strength;
    // v²/2g with g = 1.796: 0.5 km/s about 70 km, 1 km/s about 280 km.
    assert!(
        (h("plume-prometheus") - 69.6).abs() < 0.5,
        "{}",
        h("plume-prometheus")
    );
    assert!(
        (h("plume-pele-1") - 278.4).abs() < 0.5,
        "{}",
        h("plume-pele-1")
    );
    // No wind on Io: an umbrella, ash all round the vent.
    let (east, _) = at(&f.io, &f.io_sources, 15.0, -18.7, -101.0);
    let (west, _) = at(&f.io, &f.io_sources, 15.0, -18.7, -107.0);
    assert!(tau(&east, "ash") > 0.0 && (tau(&east, "ash") / tau(&west, "ash") - 1.0).abs() < 1e-6);
    // No climate veil without air.
    assert_eq!(
        veil_cooling_k(
            &a,
            &settings(),
            &f.io,
            Date { day: 400.0 },
            &f.io_sources,
            0.0
        ),
        0.0
    );
}

#[test]
fn ash_streaks_downwind_and_falls_on_the_ground() {
    let e = fixture("Earth");
    let src = Sources {
        eruptions: vec![eruption("v", 45.0, 10.0, 1.0e4, 0.0)],
        ..Default::default()
    };
    // Westerlies at 45°: downwind is east.
    let w = settings();
    let circ = circulation::circulation(&w, &e);
    assert!(circulation::zonal_wind(&w, &circ, 45.0) > 0.0);
    let (vent, ov) = at(&e, &src, 103.0, 45.0, 10.0);
    let (down, od) = at(&e, &src, 103.0, 45.0, 15.0);
    let (far, _) = at(&e, &src, 103.0, 45.0, 22.0);
    let (up, ou) = at(&e, &src, 103.0, 45.0, 5.0);
    let (side, _) = at(&e, &src, 103.0, 48.0, 15.0);
    assert!(tau(&vent, "ash") > tau(&down, "ash"));
    assert!(
        tau(&down, "ash") > tau(&far, "ash") && tau(&far, "ash") > 0.0,
        "{} {}",
        tau(&down, "ash"),
        tau(&far, "ash")
    );
    assert_eq!(tau(&up, "ash"), 0.0);
    assert!(tau(&side, "ash") < tau(&down, "ash"));
    assert!(ov.ash_fall_mm > od.ash_fall_mm && od.ash_fall_mm > 0.0);
    assert!(ou.ash_fall_mm < 1e-6, "{}", ou.ash_fall_mm);
    assert!(ov.ground_darkening > 0.0 && ov.ground_darkening <= 1.0);
    // Ash colour is linear sRGB.
    let ash = vent.aerosols.iter().find(|x| x.kind == "ash").unwrap();
    assert!(ash.colour.iter().all(|c| (0.0..=1.0).contains(c)));
    // After it ends, the ash in the air clears but the ground stays grey.
    let (later, ol) = at(&e, &src, 160.0, 45.0, 10.0);
    assert_eq!(tau(&later, "ash"), 0.0);
    assert!((ol.ash_fall_mm - at(&e, &src, 105.0, 45.0, 10.0).1.ash_fall_mm).abs() < 1e-9);
}

#[test]
fn a_large_eruption_cools_for_years_peaking_after_a_year() {
    let a = aset();
    let w = settings();
    let e = fixture("Earth");
    // Pinatubo-scale: about 1e5 m³/s for hours, 20 Tg SO₂, tropical.
    let src = Sources {
        eruptions: vec![eruption("p", 15.1, 120.3, 1.0e5, 20.0)],
        ..Default::default()
    };
    let k = |day: f64, lat: f64| veil_cooling_k(&a, &w, &e, Date { day }, &src, lat);
    assert_eq!(k(99.0, 0.0), 0.0);
    let (mut best_t, mut best) = (0.0, 0.0);
    for d in 0..(6 * 365) {
        let v = -k(100.0 + d as f64, 0.0);
        if v > best {
            best = v;
            best_t = d as f64 / 365.25;
        }
    }
    assert!((0.8..1.2).contains(&best_t), "peak at {best_t} years");
    assert!((0.3..0.5).contains(&best), "peak cooling {best} K");
    // After the peak, decay e-folds over about a year.
    let r = k(100.0 + 365.25 * 4.0, 0.0) / k(100.0 + 365.25 * 3.0, 0.0);
    assert!((0.3..0.45).contains(&r), "ratio a year apart {r}");
    // Still felt in years 2 and 3.
    assert!(-k(100.0 + 365.25 * 2.5, 0.0) > 0.3 * best);
    // Tropical: the whole world; high-latitude: mostly its own hemisphere.
    assert_eq!(k(465.0, 60.0), k(465.0, -60.0));
    let north = Sources {
        eruptions: vec![eruption("n", 60.0, 0.0, 1.0e5, 20.0)],
        ..Default::default()
    };
    let kn = |lat: f64| veil_cooling_k(&a, &w, &e, Date { day: 465.0 }, &north, lat);
    assert!((kn(-45.0) / kn(45.0) - 0.3).abs() < 1e-9);
    // A small eruption under the tropopause leaves no veil.
    let small = Sources {
        eruptions: vec![eruption("s", 15.0, 0.0, 10.0, 1.0)],
        ..Default::default()
    };
    assert_eq!(
        veil_cooling_k(&a, &w, &e, Date { day: 465.0 }, &small, 0.0),
        0.0
    );
    // The veil is in the air as a sulphate haze.
    let (c, _) = at(&e, &src, 300.0, -30.0, 0.0);
    assert!(tau(&c, "haze") > 0.05, "{:?}", c.aerosols);
}

#[test]
fn big_impacts_bring_an_impact_winter() {
    let a = aset();
    let w = settings();
    let e = fixture("Earth");
    let imp = |energy: f64| Sources {
        impacts: vec![Impact {
            id: "i".into(),
            lat_deg: 21.0,
            lon_deg: -89.5,
            day: 0.0,
            energy_j: Some(energy),
            diameter_m: None,
            speed_m_s: None,
        }],
        ..Default::default()
    };
    let big = imp(4.0e23);
    assert!(veil_cooling_k(&a, &w, &e, Date { day: 60.0 }, &big, -50.0) < -5.0);
    let (x, _) = at(&e, &big, 60.0, -50.0, 90.0);
    assert!(tau(&x, "smoke") > 1.0);
    assert!(!plumes(&a, &w, &e, Date { day: 0.5 }, &big).is_empty());
    let small = imp(4.2e16);
    assert_eq!(
        veil_cooling_k(&a, &w, &e, Date { day: 60.0 }, &small, 21.0),
        0.0
    );
    // Diameter form: 10 km at 20 km/s is about 3e23 J.
    let d = Impact {
        id: "d".into(),
        lat_deg: 0.0,
        lon_deg: 0.0,
        day: 0.0,
        energy_j: None,
        diameter_m: Some(1.0e4),
        speed_m_s: None,
    };
    let en = impact_energy_j(&a, &d);
    assert!((3.0e23..3.3e23).contains(&en), "{en}");
}

#[test]
fn smog_only_over_industry() {
    let e = fixture("Earth");
    let city = |lat: f64, tech: f64, basin: bool| Sources {
        industry: vec![Industry {
            id: "c".into(),
            lat_deg: lat,
            lon_deg: 0.0,
            tech,
            population: 1.0e7,
            basin,
        }],
        ..Default::default()
    };
    let s = city(30.0, 0.5, false);
    let (over, _) = at(&e, &s, 10.0, 30.0, 0.0);
    let (away, _) = at(&e, &s, 10.0, 30.0, 12.0);
    assert!(tau(&over, "smog") > 0.2, "{:?}", over.aerosols);
    assert_eq!(tau(&away, "smog"), 0.0);
    let (none, _) = at(&e, &Sources::default(), 10.0, 30.0, 0.0);
    assert_eq!(tau(&none, "smog"), 0.0);
    // No smog before industry; trapped more under sinking air and in basins.
    assert_eq!(
        tau(&at(&e, &city(30.0, 0.0, false), 10.0, 30.0, 0.0).0, "smog"),
        0.0
    );
    let hadley = circulation::circulation(&settings(), &e).sinking_deg[0];
    let sink = tau(
        &at(&e, &city(hadley, 0.5, false), 10.0, hadley, 0.0).0,
        "smog",
    );
    let eq = tau(&at(&e, &city(2.0, 0.5, false), 10.0, 2.0, 0.0).0, "smog");
    assert!(sink > 1.5 * eq, "sinking {sink} equator {eq}");
    let basin = tau(&at(&e, &city(2.0, 0.5, true), 10.0, 2.0, 0.0).0, "smog");
    assert!(basin > 1.5 * eq);
    // The Moon has no air to hold it.
    let (moon, _) = at(&fixture("Moon"), &s, 10.0, 30.0, 0.0);
    assert_eq!(tau(&moon, "smog"), 0.0);
}

#[test]
fn no_fire_in_low_oxygen_or_the_wet_season() {
    let a = aset();
    let w = settings();
    let e = fixture("Earth");
    let fire = Fire {
        id: "f".into(),
        lat_deg: 12.0,
        lon_deg: 20.0,
        area_km2: 2000.0,
        day_start: 0.0,
        day_end: 400.0,
    };
    let src = Sources {
        fires: vec![fire.clone()],
        ..Default::default()
    };
    let circ = circulation::circulation(&w, &e);
    // Northern winter is the dry season at 12° N; northern summer is wet.
    let dry = Date {
        day: 0.8 * e.year_days,
    };
    let wet = Date {
        day: 0.3 * e.year_days,
    };
    assert!(fire_burns(&a, &w, &e, &circ, dry, &fire).is_ok());
    assert!(fire_burns(&a, &w, &e, &circ, wet, &fire).is_err());
    let sys = plumes(&a, &w, &e, dry, &src);
    assert_eq!(sys.len(), 1);
    let (x, _) = at(&e, &src, dry.day, 12.0, 20.0);
    assert!(tau(&x, "smoke") > 1.0);
    let mut low = e.clone();
    low.gases.insert("o2".into(), 0.10);
    let err = fire_burns(&a, &w, &low, &circ, dry, &fire).unwrap_err();
    assert!(err.contains("oxygen"), "{err}");
    assert!(plumes(&a, &w, &low, dry, &src).is_empty());
    assert_eq!(tau(&at(&low, &src, dry.day, 12.0, 20.0).0, "smoke"), 0.0);
}

#[test]
fn vog_spray_and_sulphur_haze() {
    let e = fixture("Earth");
    let f = fx();
    // Kilauea's vog: haze downwind (trade winds blow west), none far upwind.
    let src = Sources {
        volcanoes: f.earth_sources.volcanoes.clone(),
        ..Default::default()
    };
    let (down, _) = at(&e, &src, 10.0, 19.4, -157.0);
    let (up, _) = at(&e, &src, 10.0, 19.4, -150.0);
    let pp = prepare(&aset(), &settings(), &e, Date { day: 10.0 }, &src);
    assert!(
        tau(&down, "haze") > tau(&up, "haze"),
        "{:?} {:?} {:?}",
        down.aerosols,
        up.aerosols,
        pp.plumes
    );
    // Spray over the sea in wind, none over land.
    let a = aset();
    let w = settings();
    let mut x = cell(&e, 10.0, 45.0, 0.0);
    imprint_aerosols(
        &a,
        &w,
        &e,
        Date { day: 10.0 },
        &Sources::default(),
        Some(true),
        &mut x,
    );
    assert!(tau(&x, "spray") > 0.0, "{:?}", x.aerosols);
    let mut y = cell(&e, 10.0, 45.0, 0.0);
    imprint_aerosols(
        &a,
        &w,
        &e,
        Date { day: 10.0 },
        &Sources::default(),
        Some(false),
        &mut y,
    );
    assert_eq!(tau(&y, "spray"), 0.0);
    // Titan's photochemical haze is not added twice.
    let t = fixture("Titan");
    let mut z = cell(&t, 10.0, 0.0, 0.0);
    let before = z.aerosols.len();
    imprint_aerosols(
        &a,
        &w,
        &t,
        Date { day: 10.0 },
        &Sources::default(),
        None,
        &mut z,
    );
    assert_eq!(z.aerosols.len(), before);
}

fn golden_value() -> serde_json::Value {
    let a = aset();
    let w = settings();
    let e = fixture("Earth");
    let f = fx();
    let d = Date { day: 101.0 };
    let earth = plumes(&a, &w, &e, d, &f.earth_sources);
    let io = plumes(&a, &w, &f.io, Date { day: 15.0 }, &f.io_sources);
    let cells: Vec<_> = [
        (15.1, 120.3),
        (15.1, 125.0),
        (45.0, 14.0),
        (34.0, -118.2),
        (19.4, -157.0),
    ]
    .iter()
    .map(|(lat, lon)| {
        let (c, o) = at(&e, &f.earth_sources, d.day, *lat, *lon);
        serde_json::json!({ "lat": lat, "lon": lon, "aerosols": c.aerosols, "out": o })
    })
    .collect();
    let cooling: Vec<_> = (0..8)
        .map(|q| {
            let day = 100.0 + 0.5 * 365.25 * q as f64;
            serde_json::json!([
                day,
                veil_cooling_k(&a, &w, &e, Date { day }, &f.earth_sources, 0.0)
            ])
        })
        .collect();
    serde_json::json!({ "earth_plumes": earth, "io_plumes": io, "cells": cells, "cooling_k": cooling })
}

#[test]
fn deterministic_and_golden() {
    let g = golden_value();
    assert_eq!(g, golden_value());
    let path = format!(
        "{}/../../tests/golden/weather-aerosols.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing {path}; run with SSE_BLESS=1"));
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden weather-aerosols.json changed"
    );
}
