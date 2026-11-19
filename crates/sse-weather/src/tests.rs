use crate::circulation::{circulation, condensation};
use crate::contract::Pin;
use crate::scenario::{block, fixture, fixtures, golden, settings};
use crate::*;
use sse_surface::atlas::TileKey;

fn check_golden(name: &str, got: &serde_json::Value) {
    let path = format!("{}/../../tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = serde_json::to_string_pretty(got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing {path}; run with SSE_BLESS=1"));
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden {name} changed"
    );
}

#[test]
fn fixtures_land_in_expected_regimes() {
    let s = settings();
    let mut wrong = vec![];
    for f in fixtures() {
        let c = circulation(&s, &f.conditions);
        if c.regime != f.expect {
            wrong.push(format!(
                "{}: {} not {} ({:?})",
                f.conditions.label, c.regime, f.expect, c.why
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn earth_has_three_cells_and_belts_in_the_right_places() {
    let s = settings();
    let e = fixture("Earth");
    let c = circulation(&s, &e);
    assert_eq!(c.cells, 3, "{:?}", c.why);
    assert!(
        (25.0..40.0).contains(&c.hadley_edge_deg),
        "{}",
        c.hadley_edge_deg
    );
    assert_eq!(c.storm_tracks_deg.len(), 1);
    assert!(
        (40.0..60.0).contains(&c.storm_tracks_deg[0]),
        "{:?}",
        c.storm_tracks_deg
    );
    let cl = block().climate(&e);
    assert_eq!(cl.condensate, "water");
    assert!(
        (0.8..1.2).contains(&cl.precip_mean_m_yr),
        "{}",
        cl.precip_mean_m_yr
    );
    assert!(
        (0.55..0.75).contains(&cl.cloud_cover_mean),
        "{}",
        cl.cloud_cover_mean
    );
    // Wet equator, dry subtropics, wetter storm track, dry poles.
    let at = |lat: f64| {
        cl.bands
            .iter()
            .min_by(|a, b| (a.lat_deg - lat).abs().total_cmp(&(b.lat_deg - lat).abs()))
            .unwrap()
            .precip_m_yr
    };
    assert!(
        at(2.5) > 2.0 * at(27.5),
        "equator {} subtropics {}",
        at(2.5),
        at(27.5)
    );
    assert!(
        at(47.5) > at(27.5),
        "storm track {} subtropics {}",
        at(47.5),
        at(27.5)
    );
    assert!(at(87.5) < at(47.5));
    // Trades blow from the east, westerlies in mid-latitudes.
    let wind = |lat: f64| crate::circulation::zonal_wind(&s, &c, lat);
    assert!(wind(15.0) < 0.0 && wind(45.0) > 0.0);
}

#[test]
fn averages_keep_the_erosion_terms_form() {
    // S3's rain term reads `rain_cells`; Earth's must equal the surface pack's value.
    let surf = sse_surface::scenario::settings();
    let cl = block().climate(&fixture("Earth"));
    assert_eq!(cl.rain_cells, surf.wear.rain_cells);
    assert!((0.6..0.75).contains(&cl.surface_sun_fraction));
}

#[test]
fn venus_titan_mars_get_their_skies() {
    let b = block();
    let v = b.climate(&fixture("Venus"));
    assert_eq!(v.condensate, "sulphuric acid");
    assert!(v.cloud_cover_mean > 0.99 && v.precip_mean_m_yr == 0.0);
    let t = b.climate(&fixture("Titan"));
    assert_eq!(t.condensate, "methane");
    assert!(
        (0.001..0.1).contains(&t.precip_mean_m_yr),
        "{}",
        t.precip_mean_m_yr
    );
    let key = TileKey::new(4, 0, 0, 0).unwrap();
    let tile = b.field(&fixture("Titan"), Date { day: 0.0 }, key, 2, &[]);
    assert!(tile.cells[0].aerosols.iter().any(|a| a.kind == "haze"));
    assert!(tile.cells[0].precip_kind.contains("methane"));
    let m = b.climate(&fixture("Mars"));
    assert_eq!(m.precip_mean_m_yr, 0.0);
    assert!(m.cloud_cover_mean <= 0.15);
    let mc = condensation(&settings(), &fixture("Mars"));
    assert!(mc.is_some(), "thin water ice cloud on Mars");
    assert_eq!(b.climate(&fixture("Moon")).cloud_cover_mean, 0.0);
}

#[test]
fn fields_are_deterministic_and_cover_the_tile() {
    let b = block();
    let e = fixture("Earth");
    let key = TileKey::new(2, 2, 1, 3).unwrap();
    let a = b.field(&e, Date { day: 42.5 }, key, 6, &[]);
    let c = b.field(&e, Date { day: 42.5 }, key, 6, &[]);
    assert_eq!(a, c);
    assert_eq!(a.cells.len(), 36);
    assert!(a.cells.iter().all(|x| x.layers.len() == LAYERS.len()));
}

#[test]
fn seasons_move_the_rain_belt() {
    let b = block();
    let e = fixture("Earth");
    let june = b.prepare(
        &e,
        Date {
            day: 0.25 * e.year_days,
        },
    );
    let dec = b.prepare(
        &e,
        Date {
            day: 0.75 * e.year_days,
        },
    );
    assert!(
        june.itcz_deg > 4.0 && dec.itcz_deg < -4.0,
        "{} {}",
        june.itcz_deg,
        dec.itcz_deg
    );
}

#[test]
fn every_reference_is_linked() {
    let s = settings();
    assert!(s.references.len() >= 10);
    assert!(s
        .references
        .iter()
        .all(|r| r.doi.is_some() || r.url.is_some()));
}

#[test]
fn golden_weather() {
    check_golden("weather.json", &golden());
}

// ------------------------------------------------------------------ W2

fn year_of_systems(c: &Conditions, days: i64) -> Vec<WeatherSystem> {
    let b = block();
    let mut seen = std::collections::BTreeMap::new();
    for d in 0..days {
        for s in b.systems(
            c,
            Date {
                day: d as f64 + 0.5,
            },
            &[],
        ) {
            seen.entry(s.id.clone()).or_insert(s);
        }
    }
    seen.into_values().collect()
}

#[test]
fn earth_storms_fall_in_the_right_latitudes() {
    let e = fixture("Earth");
    let all = year_of_systems(&e, 365);
    let tc: Vec<_> = all
        .iter()
        .filter(|s| s.kind == "tropical-cyclone")
        .collect();
    let cy: Vec<_> = all.iter().filter(|s| s.kind == "cyclone").collect();
    assert!(
        (40..140).contains(&tc.len()),
        "tropical cyclones a year: {}",
        tc.len()
    );
    assert!(tc.iter().all(|s| (5.0..35.0).contains(&s.lat_deg.abs())));
    let mean = cy.iter().map(|s| s.lat_deg.abs()).sum::<f64>() / cy.len() as f64;
    assert!((40.0..60.0).contains(&mean), "storm track mean {mean}");
    assert!(cy.len() > 200);
}

#[test]
fn no_tropical_cyclones_without_warm_ocean() {
    let mut dry = fixture("Earth");
    dry.liquid = None;
    dry.ocean_fraction = 0.0;
    let mut cold = fixture("Earth");
    cold.t_surface_k = 265.0;
    for c in [
        dry,
        cold,
        fixture("Mars"),
        fixture("Venus"),
        fixture("Titan"),
    ] {
        let n = year_of_systems(&c, 200)
            .iter()
            .filter(|s| s.kind == "tropical-cyclone")
            .count();
        assert_eq!(n, 0, "{}", c.label);
    }
}

#[test]
fn a_storm_is_the_same_whichever_day_it_is_asked_from() {
    let b = block();
    let e = fixture("Earth");
    let a = b.systems(&e, Date { day: 120.3 }, &[]);
    let later = b.systems(&e, Date { day: 121.1 }, &[]);
    let mut shared = 0;
    for s in &a {
        if let Some(t) = later.iter().find(|t| t.id == s.id) {
            assert_eq!(s, t);
            shared += 1;
        }
    }
    assert!(shared > 3);
}

struct Ridge;
impl crate::formations::Terrain for Ridge {
    fn ground(&self, d: [f64; 3]) -> (f64, bool) {
        let (_, lon) = crate::rules::lat_lon(d);
        let h = 3000.0 * (1.0 - (lon.abs() / 4.0)).max(0.0);
        (h, lon.abs() > 30.0)
    }
}

#[test]
fn mountains_bank_cloud_windward_and_dry_the_lee() {
    let b = block();
    let e = fixture("Earth");
    let p = b.prepare(&e, Date { day: 10.0 });
    // Westerlies at 45°: the west slope is windward.
    let w = b.cell_at(&e, &p, Date { day: 10.0 }, &[], Some(&Ridge), 45.0, -2.0);
    let l = b.cell_at(&e, &p, Date { day: 10.0 }, &[], Some(&Ridge), 45.0, 2.0);
    assert!(
        w.precip_mm_day > 2.0 * l.precip_mm_day,
        "{} {}",
        w.precip_mm_day,
        l.precip_mm_day
    );
    assert!(w.layers[0].cloud_cover > l.layers[0].cloud_cover);
}

#[test]
fn locked_worlds_cloud_the_day_side() {
    let b = block();
    let c = fixture("Locked temperate world");
    let p = b.prepare(&c, Date { day: 1.0 });
    let sys = b.systems_at(&c, &p, Date { day: 1.0 }, &[]);
    assert!(sys.iter().any(|s| s.kind == "substellar-storm"));
    let day = b.cell_at(&c, &p, Date { day: 1.0 }, &sys, None, 0.0, 0.0);
    let night = b.cell_at(&c, &p, Date { day: 1.0 }, &sys, None, 0.0, 180.0);
    assert!(day.layers[1].cloud_cover > 0.9 && night.layers[1].cloud_cover < 0.3);
}

#[test]
fn pins_replace_drawn_systems_and_outlive_regeneration() {
    let b = block();
    let e = fixture("Earth");
    let a = b.systems(&e, Date { day: 50.0 }, &[]);
    let mut s = a.iter().find(|s| s.kind == "cyclone").unwrap().clone();
    s.ends += 3.0;
    let pin = Pin {
        id: s.id.clone(),
        name: "Storm Ada".into(),
        system: s.clone(),
    };
    let got = b.systems(&e, Date { day: s.ends - 1.0 }, &[pin]);
    let p = got
        .iter()
        .find(|x| x.id == s.id)
        .expect("pinned storm alive");
    assert_eq!(p.evidence, "pinned");
    assert_eq!(got.iter().filter(|x| x.id == s.id).count(), 1);
}

#[test]
fn reports_read_plainly() {
    let r = block().report(&fixture("Earth"), Date { day: 200.0 }, 45.0, 0.0, &[], None);
    assert!(r.lines[0].starts_with("Wind "));
    assert!(r.lines.iter().any(|l| l.starts_with("Visibility")));
    let m = block().report(&fixture("Moon"), Date { day: 200.0 }, 0.0, 0.0, &[], None);
    assert_eq!(m.wind_km_h, 0.0);
}

#[test]
fn golden_weather_storms() {
    check_golden("weather-storms.json", &crate::scenario::storms_golden());
}
