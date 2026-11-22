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
fn the_ground_and_the_sky_read_one_climate() {
    // W7a: the rain that wore the ground, the climate cover read, and both
    // weather blocks' averages are the same numbers, to the last bit.
    let ss = sse_surface::detail::orbit_settings(&sse_surface::scenario::settings());
    let zonal =
        crate::zonal::ZonalBlock::from_packs(&[crate::scenario::base_weather_pack()]).unwrap();
    let b = block();
    for (name, rot) in [
        ("Earth", 23.93),
        ("Mars", 24.62),
        ("Venus", -5832.5),
        ("Titan", 382.7),
    ] {
        let body = sse_surface::scenario::fixtures()
            .into_iter()
            .find(|f| f.body.label == name)
            .unwrap()
            .body;
        let opts = sse_surface::survey::SurveyOptions {
            rotation_hours: Some(rot),
            ..Default::default()
        };
        let seed = sse_core::draw::Seed::from_text("one-climate");
        let sv = sse_surface::survey::survey_world(&ss, &body, &seed, body.age_gyr, &opts).unwrap();
        let ground = sv.climate.as_ref().expect("a solid world has a climate");
        let cover = &sv.atlas.layers().expect("layers").climate.world;
        assert_eq!(ground, cover, "{name}: wear and cover read one climate");
        let x = opts.extra(Some(rot));
        let (c, _) = crate::conditions::from_surface(&ss, &body, &sv.envelope, &x);
        assert_eq!(c, ground.climate.conditions, "{name}: one set of facts");
        for blk in [&b as &dyn WeatherBlock, &zonal] {
            let cl = blk.climate(&c);
            let rain: Vec<f64> = cl.bands.iter().map(|x| x.precip_m_yr).collect();
            assert_eq!(
                rain, ground.climate.precip_m_yr,
                "{name}: the sky's rain is the ground's"
            );
        }
        let p = b.prepare(&c, Date { day: 100.0 });
        assert_eq!(p.core.t_k, ground.climate.t_k, "{name}: one temperature");
    }
}

#[test]
fn a_backwards_spin_turns_the_weather_winds() {
    let b = block();
    let mut v = fixture("Earth");
    let fwd = b.climate(&v);
    v.rotation_hours = -v.rotation_hours;
    let back = b.climate(&v);
    for (f, k) in fwd.bands.iter().zip(&back.bands) {
        assert_eq!(f.wind_m_s, -k.wind_m_s);
        assert_eq!(f.precip_m_yr, k.precip_m_yr);
    }
    assert!(fwd.bands.iter().any(|x| x.wind_m_s.abs() > 1.0));
}

#[test]
fn averages_keep_sunlight_at_the_ground() {
    let cl = block().climate(&fixture("Earth"));
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
    // Methane rain falls somewhere on Titan (W7b: the rain belt follows the
    // season, so at the equinox it is wettest toward the poles, as Cassini
    // found the lakes).
    let rains = (0..6u8).any(|f| {
        let k = TileKey::new(f, 0, 0, 0).unwrap();
        b.field(&fixture("Titan"), Date { day: 0.0 }, k, 4, &[])
            .cells
            .iter()
            .any(|c| c.precip_kind.contains("methane"))
    });
    assert!(rains, "methane rain somewhere on Titan");
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

// ------------------------------------------------------------------ W5

#[test]
fn one_storm_seen_from_orbit_met_on_descent_and_heard_on_the_ground() {
    use crate::hooks::{column, sound, trigger_storm};
    let b = block();
    let e = fixture("Earth");
    let pin = trigger_storm(
        &b,
        &e,
        "gm-storm-1",
        "tropical-cyclone",
        18.0,
        -50.0,
        30.0,
        4.0,
    );
    let pins = [pin];
    let day = Date { day: 31.0 };
    let sys = b.systems(&e, day, &pins);
    let s = sys
        .iter()
        .find(|s| s.id == "gm-storm-1")
        .expect("storm drawn with pins");
    assert_eq!(s.evidence, "authored");
    let (la, lo) = crate::formations::position_at(s, day.day);
    // From orbit: the field cell over it is thick cloud.
    let d = crate::formations::dir(la, lo);
    let (face, u, v) = sse_surface::atlas::face_of(d);
    let lvl = 6u8;
    let n = 1u32 << lvl;
    let x = (((u + 1.0) / 2.0) * n as f64) as u32;
    let y = (((v + 1.0) / 2.0) * n as f64) as u32;
    let key = TileKey::new(face, lvl, x.min(n - 1), y.min(n - 1)).unwrap();
    let tile = b.field(&e, day, key, 2, &pins);
    assert!(
        tile.cells.iter().all(|c| c.layers[1].cloud_cover > 0.9),
        "{:?}",
        tile.cells[0].layers
    );
    // On descent: hazards.
    let col = column(&b, &e, day, la, lo, &pins, None);
    assert!(col.hazard >= 2, "{:#?}", col.levels);
    assert_eq!(col.levels.len(), 3);
    // On the ground: wind and thunder.
    let snd = sound(&b, &e, day, la, lo, &pins, None);
    assert!(
        snd.thunder_per_hour > 1.0 && snd.precip_mm_day > 10.0,
        "{snd:?}"
    );
    // A fresh block from the packs (regeneration) still honours the pin.
    let again = block().systems(&e, day, &pins);
    assert!(again.iter().any(|x| x == s));
}

#[test]
fn gm_orders_clear_skies_and_calm() {
    use crate::hooks::{order_calm, order_clear, trigger_storm};
    let b = block();
    let e = fixture("Earth");
    let storm = trigger_storm(&b, &e, "gm-storm-2", "cyclone", 50.0, 10.0, 0.0, 3.0);
    let clear = order_clear("gm-clear-1", 50.0, 10.0, 500.0, 0.0, 2.0);
    let calm = order_calm("gm-calm-1", 50.0, 10.0, 500.0, 0.0, 2.0);
    let r = b.report(
        &e,
        Date { day: 0.1 },
        50.0,
        10.0,
        &[storm.clone(), clear, calm],
        None,
    );
    assert_eq!(r.cloud_cover, 0.0);
    assert!(r.wind_km_h < 10.0, "{}", r.wind_km_h);
    // Before the order starts, the storm's cloud is there.
    let before = b.report(&e, Date { day: 0.1 }, 50.0, 10.0, &[storm], None);
    assert!(before.cloud_cover > 0.5);
}

#[test]
fn airless_worlds_are_silent_and_have_no_column() {
    let b = block();
    let m = fixture("Moon");
    let s = crate::hooks::sound(&b, &m, Date { day: 0.0 }, 0.0, 0.0, &[], None);
    assert!(s.silent && s.wind_m_s == 0.0);
    let c = crate::hooks::column(&b, &m, Date { day: 0.0 }, 0.0, 0.0, &[], None);
    assert!(c.levels.is_empty() && c.hazard == 0);
}

#[test]
fn venus_descent_meets_acid_cloud() {
    let b = block();
    let c = crate::hooks::column(
        &b,
        &fixture("Venus"),
        Date { day: 0.0 },
        10.0,
        0.0,
        &[],
        None,
    );
    assert!(
        c.levels.iter().any(|l| l.line.contains("corrosive")),
        "{:#?}",
        c.levels
    );
}
