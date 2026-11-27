//! W3 tests: giants' bands, storm families and polar regimes.

use super::*;
use crate::scenario::base_weather_pack;
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::Address;

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

fn digest<T: serde::Serialize>(t: &T) -> String {
    let text = serde_json::to_string(t).expect("serialisable");
    sse_core::ContentHash::of(&text).expect("hash").hex()
}

/// ~200 made-up giants from keyed draws on radius, mass, spin, temperature,
/// internal heat and molar mass.
fn random_giants(n: u64) -> Vec<Conditions> {
    let base = fixture("Saturn");
    let d = Draws::new(
        Seed::from_text("w3-random-giants"),
        Address::universe("w3").unwrap(),
        Stream::new("test.giants.v1").unwrap(),
    );
    let logu = |i: u64, lo: f64, hi: f64| lo * m::exp(d.unit(i) * m::ln(hi / lo));
    (0..n)
        .map(|i| {
            let k = 8 * i;
            let r = 6.371e6 * d.uniform(k, 3.0, 15.0);
            let mass = M_EARTH_KG * logu(k + 1, 8.0, 1500.0);
            let mut c = base.clone();
            c.address = Address::universe("w3").unwrap().child("body", i).unwrap();
            c.label = format!("giant {i}");
            c.radius_m = r;
            c.gravity_m_s2 = G * mass / (r * r);
            c.rotation_hours = logu(k + 2, 5.0, 40.0);
            c.t_surface_k = logu(k + 3, 50.0, 1200.0);
            c.internal_flux_w_m2 = logu(k + 4, 0.01, 20.0);
            c.mean_molar_kg = d.uniform(k + 5, 0.0022, 0.0027);
            c.year_days = logu(k + 6, 300.0, 60000.0);
            c
        })
        .collect()
}

#[test]
fn settings_load_and_every_reference_is_linked() {
    let gs = GiantSettings::from_packs(&[base_weather_pack()]).unwrap();
    assert!(gs.references.len() >= 10);
    assert!(gs
        .references
        .iter()
        .all(|r| r.doi.is_some() || r.url.is_some()));
    // The W1 weather settings still load from the same pack.
    crate::scenario::settings();
}

#[test]
fn fixtures_land_in_their_polar_regimes() {
    let gs = base_settings();
    let mut wrong = vec![];
    for f in fixtures() {
        let l = layout(&gs, &f.conditions);
        let north = &l.poles[0];
        if north.regime != f.expect {
            wrong.push(format!(
                "{}: {} not {} ({:?})",
                f.conditions.label, north.regime, f.expect, north.why
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn jupiter_ring_saturn_vortex_with_hexagon_ice_giants_no_polygon() {
    let gs = base_settings();
    let j = layout(&gs, &fixture("Jupiter"));
    // 6 to 7 eastward jets a hemisphere observed (visual polish citation
    // check, section 6): the Rhines width with the eddy speed gives 13.
    assert_eq!(j.jets, 13);
    assert!(j.a_over_ld >= 40.0, "{}", j.a_over_ld);
    for p in &j.poles {
        assert_eq!(p.regime, "cyclone-ring");
        assert!(gs.polar.ring_counts.contains(&p.ring_count));
        assert_eq!(p.polygon_sides, 0);
    }
    let s = layout(&gs, &fixture("Saturn"));
    assert!(s.a_over_ld <= 30.0, "{}", s.a_over_ld);
    assert!(
        (76.0..81.0).contains(&s.polar_jet_lat_deg),
        "Saturn's jet at {} (hexagon about 78°N)",
        s.polar_jet_lat_deg
    );
    assert_eq!(s.poles[0].regime, "single-vortex");
    assert_eq!(s.poles[0].polygon_sides, 6, "{}", s.polygon_why);
    // Uranus: weak internal heat and high Bu, a diffuse cap; Neptune: one
    // vortex, but its jet sits below 70°, so no polygon.
    let u = layout(&gs, &fixture("Uranus"));
    assert!(u.poles.iter().all(|p| p.regime == "diffuse-cap"));
    let n = layout(&gs, &fixture("Neptune"));
    assert!(n.poles.iter().all(|p| p.regime == "single-vortex"));
    for l in [&u, &n] {
        assert!(l.ice && l.jets == 3);
        assert!(l.polar_jet_lat_deg < gs.polygon.min_lat_deg);
        assert!(l.poles.iter().all(|p| p.polygon_sides == 0));
    }
    // Ice giants: retrograde equator, prograde high latitudes.
    let w = |l: &GiantLayout, c: &Conditions, lat: f64| zonal_wind(&gs, l, c.radius_m, lat);
    let nc = fixture("Neptune");
    assert!(w(&n, &nc, 0.0) < -300.0 && w(&n, &nc, 65.0) > 150.0);
    // Gas giants: prograde equator.
    let jc = fixture("Jupiter");
    assert!(w(&j, &jc, 0.0) > 0.0);
}

#[test]
fn jet_profile_reads_the_eddy_speed_and_matches_the_bands() {
    let gs = base_settings();
    for f in fixtures() {
        let c = &f.conditions;
        let l = layout(&gs, c);
        let p = jet_profile(&gs, c, &l);
        let n = 2 * l.jets as usize;
        for v in [
            &p.lat_deg,
            &p.jet_speed_m_s,
            &p.jet_shear_m_s,
            &p.eddy_speed_m_s,
            &p.rhines_scale_m,
            &p.deformation_radius_m,
        ] {
            assert_eq!(v.len(), n, "{}", c.label);
        }
        // One jet's Rhines width, a few bands' worth of latitude near 30°:
        // the bands are as wide as the Rhines width says (within 2, the
        // factor the literature leaves open).
        let k = p.lat_deg.iter().position(|x| *x > 25.0).unwrap_or(n - 1);
        let band_m = (l.band_edges_deg[k + 1] - l.band_edges_deg[k]).to_radians() * c.radius_m;
        let r = p.rhines_scale_m[k] / band_m;
        assert!((0.5..2.0).contains(&r), "{}: Rhines over band {r}", c.label);
        assert!(p.eddy_speed_m_s.iter().all(|u| *u == l.eddy_m_s));
    }
    // Jupiter's Rhines width near 30° with U 17 m/s: about 9,000 km
    // (check file: 9,520 km at U 40 m/s for pi sqrt(U/beta), the same
    // number as pi sqrt(2U/beta) at 20 m/s).
    let jc = fixture("Jupiter");
    let j = layout(&gs, &jc);
    let w = sse_climate::circulation::rhines_width_m(jc.radius_m, jc.spin_rate(), j.eddy_m_s, 30.0);
    assert!((7.0e6..1.0e7).contains(&w), "{w}");
}

#[test]
fn band_edges_match_the_s6_cloud_deck() {
    // sse-surface regimes::cloud_deck, same inputs (its formula, restated).
    let gs = base_settings();
    for f in fixtures() {
        let c = &f.conditions;
        let l = layout(&gs, c);
        let omega = std::f64::consts::TAU / (c.rotation_hours * 3600.0);
        let u = if l.ice { 100.0 } else { 17.0 };
        let jets = m::round(0.5 * m::sqrt(c.radius_m * omega / u)).clamp(1.0, 60.0) as u32;
        let nb = 2 * jets;
        let edges: Vec<f64> = (0..=nb)
            .map(|i| m::asin(-1.0 + 2.0 * i as f64 / nb as f64).to_degrees())
            .collect();
        assert_eq!(l.jets, jets, "{}", c.label);
        assert_eq!(l.band_edges_deg.len(), edges.len());
        for (a, b) in l.band_edges_deg.iter().zip(&edges) {
            assert!((a - b).abs() < 1e-9, "{}: {a} {b}", c.label);
        }
    }
    // The surface pack's wind scales and ice threshold are the same numbers.
    let surf = sse_surface::scenario::settings();
    let cl = &surf.regime_features.clouds;
    assert_eq!(cl.wind_gas_m_s, gs.bands.wind_gas_m_s);
    assert_eq!(cl.wind_ice_m_s, gs.bands.wind_ice_m_s);
    assert_eq!(cl.eddy_gas_m_s, gs.bands.eddy_gas_m_s);
    assert_eq!(cl.eddy_ice_m_s, gs.bands.eddy_ice_m_s);
    assert_eq!(cl.ice_giant_mass_earth, gs.bands.ice_giant_mass_earth);
}

#[test]
fn zones_and_belts_alternate_symmetrically() {
    let gs = base_settings();
    let c = fixture("Jupiter");
    let l = layout(&gs, &c);
    let nj = l.jets as f64;
    for k in 0..l.jets {
        let lat = m::asin((k as f64 + 0.5) / nj) * 180.0 / m::PI;
        let wn = zone_weight(&l, lat);
        let ws = zone_weight(&l, -lat);
        assert_eq!(wn, ws);
        assert!(
            if k % 2 == 0 { wn > 0.99 } else { wn < 0.01 },
            "band {k}: {wn}"
        );
    }
    // Prograde jet on the poleward edge of the first zone, retrograde next.
    let edge = |k: f64| m::asin(k / nj) * 180.0 / m::PI;
    assert!(
        zonal_wind(&gs, &l, c.radius_m, edge(1.0))
            > 0.9 * l.wind_scale_m_s * m::cos(edge(1.0) / DEG)
    );
    assert!(zonal_wind(&gs, &l, c.radius_m, -edge(2.0)) < 0.0);
}

#[test]
fn storms_keep_their_ids_and_drift_with_the_wind() {
    let gs = base_settings();
    for label in ["Jupiter", "Neptune"] {
        let c = fixture(label);
        let l = layout(&gs, &c);
        let (d1, d2) = (4000.0, 4100.0);
        let a = giant_systems(&gs, &c, Date { day: d1 });
        let b = giant_systems(&gs, &c, Date { day: d2 });
        let mut shared = 0;
        for s in &a {
            let Some(t) = b.iter().find(|x| x.id == s.id) else {
                assert!(s.ends < d2, "{} vanished while alive", s.id);
                continue;
            };
            shared += 1;
            assert_eq!(
                (s.born, s.ends, s.lat_deg, &s.track),
                (t.born, t.ends, t.lat_deg, &t.track)
            );
            if s.born > ALWAYS {
                let u = zonal_wind(&gs, &l, c.radius_m, s.lat_deg);
                let want = drift_lon(u, c.radius_m, s.lat_deg, s.lon_deg, d1, d2);
                let diff = (want - t.lon_deg + 540.0).rem_euclid(360.0) - 180.0;
                assert!(diff.abs() < 1e-6, "{}: {} vs {}", s.id, want, t.lon_deg);
                assert!(s.born <= d1 && d2 < s.ends);
            }
        }
        assert!(shared > 0, "{label}: no storm alive on both dates");
        // Every storm returned is alive on the date.
        assert!(a.iter().all(|s| s.born <= d1 && d1 < s.ends));
    }
}

#[test]
fn storm_families_follow_their_conditions() {
    let gs = base_settings();
    let count = |label: &str, kind: &str| {
        let c = fixture(label);
        (0..60)
            .map(|k| {
                giant_systems(
                    &gs,
                    &c,
                    Date {
                        day: 500.0 * k as f64,
                    },
                )
                .iter()
                .filter(|s| s.kind == kind)
                .count()
            })
            .sum::<usize>()
    };
    // Ovals and strings on gas giants, dark spots on ice giants.
    assert!(count("Jupiter", "oval") > 0);
    assert_eq!(count("Jupiter", "dark-spot"), 0);
    assert!(count("Neptune", "dark-spot") > 0);
    assert_eq!(count("Neptune", "oval"), 0);
    // Outbreaks only on the cool gas giant: Saturn, not Jupiter (165 K) or ice giants.
    let sat = fixture("Saturn");
    let mut outbreaks = std::collections::BTreeSet::new();
    for k in 0..(4.0 * sat.year_days / 20.0) as u32 {
        for s in giant_systems(
            &gs,
            &sat,
            Date {
                day: 20.0 * k as f64,
            },
        ) {
            if s.kind == "outbreak" {
                outbreaks.insert(s.id.clone());
                assert!(s.ends - s.born == gs.outbreaks.duration_days);
            }
        }
    }
    assert!(
        !outbreaks.is_empty() && outbreaks.len() <= 5,
        "{outbreaks:?}"
    );
    let jup = fixture("Jupiter");
    for k in 0..400 {
        assert!(giant_systems(
            &gs,
            &jup,
            Date {
                day: 50.0 * k as f64
            }
        )
        .iter()
        .all(|s| s.kind != "outbreak"));
    }
}

#[test]
fn every_system_explains_itself() {
    let gs = base_settings();
    for f in fixtures() {
        for day in [0.0, 2500.0, 7777.0] {
            for s in giant_systems(&gs, &f.conditions, Date { day }) {
                assert!(s.why.contains("Rule"), "{}: {}", s.id, s.why);
                assert!(
                    s.evidence == "rule" || s.evidence == "drawn",
                    "{}: {}",
                    s.id,
                    s.evidence
                );
            }
        }
    }
}

#[test]
fn most_random_giants_show_no_polygon_and_sides_follow_the_relation() {
    let gs = base_settings();
    let pg = &gs.polygon;
    let giants = random_giants(200);
    let (mut with, mut poles) = (0, 0);
    let mut sides_seen = std::collections::BTreeSet::new();
    let mut regs = std::collections::BTreeMap::new();
    for c in &giants {
        let l = layout(&gs, c);
        // The relation, restated from the conditions.
        let omega = c.spin_rate();
        let a = c.radius_m;
        let u = if l.ice {
            gs.bands.wind_ice_m_s
        } else {
            gs.bands.wind_gas_m_s
        };
        let lat = 90.0 - m::cbrt(u * a * a / (2.0 * omega)) / a * 180.0 / m::PI;
        let lat = lat.max(0.0);
        let ld = gs.stratification.brunt_s * c.scale_height_m()
            / (2.0 * omega * m::sin(lat.max(1.0) * m::PI / 180.0));
        let waves = 2.0 * m::PI * a * m::cos(lat * m::PI / 180.0) / (pg.wavelength_ld * ld);
        assert!((waves - l.polygon_waves).abs() < 1e-9 * waves.max(1.0));
        let want = (m::round(waves) as i64).clamp(pg.min_sides as i64, pg.max_sides as i64) as u32;
        for p in &l.poles {
            poles += 1;
            *regs.entry(p.regime.clone()).or_insert(0) += 1;
            let eligible = pg.regimes.contains(&p.regime)
                && lat >= pg.min_lat_deg
                && l.polar_jet_m_s >= pg.min_jet_m_s
                && waves >= pg.min_sides as f64 - 0.5
                && waves < pg.max_sides as f64 + 0.5;
            if eligible {
                with += 1;
                assert_eq!(p.polygon_sides, want, "{}", c.label);
                sides_seen.insert(want);
            } else {
                assert_eq!(p.polygon_sides, 0, "{}", c.label);
            }
        }
    }
    let frac = with as f64 / poles as f64;
    assert_eq!(regs.len(), 3, "every polar regime occurs: {regs:?}");
    assert!(frac < 0.25, "{with} of {poles} poles carry a polygon");
    assert!(with > 0, "the rule never fires on random giants");
    assert!(sides_seen.len() >= 2, "sides seen {sides_seen:?}");
}

#[test]
fn chromophore_is_thicker_over_belts_and_decks_carry_their_size() {
    let gs = base_settings();
    let c = fixture("Jupiter");
    let p = prepare(&gs, &c, Date { day: 0.0 });
    let l = &p.layout;
    let ch = gs.clouds.chromophore.as_ref().unwrap();
    // Sromovsky et al. 2017: 0.05 to 0.76 at 1 µm, radius 0.12 to 0.29 µm.
    let nj = l.jets as f64;
    let at = |x: f64| m::asin(x / nj) * 180.0 / m::PI;
    let read = |lat: f64| {
        let cell = giant_cell(&gs, &c, &p, lat, 0.0);
        let a = cell
            .aerosols
            .iter()
            .find(|a| a.kind == "chromophore")
            .cloned()
            .expect("chromophore");
        (a, cell.layers[2].particle_radius_um)
    };
    let (z, rz) = read(at(0.5)); // zone centre (band 0 from the equator)
    let (b, _) = read(at(1.5)); // belt centre (band 1)
    assert!(b.optical_depth > z.optical_depth);
    for a in [&z, &b] {
        assert!(
            (0.05..=0.76).contains(&a.optical_depth),
            "{}",
            a.optical_depth
        );
        assert!((0.12..=0.29).contains(&a.particle_radius_um));
    }
    assert_eq!(l.layers[2], ch.above);
    assert_eq!(rz, 1.0, "ammonia ice deck radius");
    // Ice giants have no ammonia top deck, so no chromophore.
    let n = fixture("Neptune");
    let pn = prepare(&gs, &n, Date { day: 0.0 });
    let cell = giant_cell(&gs, &n, &pn, 30.0, 0.0);
    assert!(cell.aerosols.iter().all(|a| a.kind != "chromophore"));
}

#[test]
fn cells_show_belts_zones_and_storms() {
    let gs = base_settings();
    let c = fixture("Jupiter");
    let p = (0..40)
        .map(|k| {
            prepare(
                &gs,
                &c,
                Date {
                    day: 250.0 * k as f64,
                },
            )
        })
        .find(|q| q.systems.iter().any(|s| s.kind == "oval"))
        .expect("an oval");
    let l = &p.layout;
    let nj = l.jets as f64;
    let zone = m::asin(2.5 / nj) * 180.0 / m::PI;
    let belt = m::asin(1.5 / nj) * 180.0 / m::PI;
    // Away from any storm: pick the longitude farthest from every system.
    let quiet = |lat: f64| {
        (0..72)
            .map(|k| -180.0 + 5.0 * k as f64)
            .max_by(|x, y| {
                let near = |lon: f64| {
                    p.systems
                        .iter()
                        .map(|s| distance_km(c.radius_m, lat, lon, s.lat_deg, s.lon_deg))
                        .fold(f64::INFINITY, f64::min)
                };
                near(*x).total_cmp(&near(*y))
            })
            .unwrap()
    };
    let z = giant_cell(&gs, &c, &p, zone, quiet(zone));
    let b = giant_cell(&gs, &c, &p, belt, quiet(belt));
    assert!(z.layers[2].optical_depth > b.layers[2].optical_depth);
    assert!(z.layers[2].cloud_cover > b.layers[2].cloud_cover);
    assert_eq!(z.layers[2].condensate, "ammonia ice");
    // An oval brightens its centre.
    let ov = p
        .systems
        .iter()
        .find(|s| s.kind == "oval")
        .expect("an oval");
    let at = giant_cell(&gs, &c, &p, ov.lat_deg, ov.lon_deg);
    let away = giant_cell(&gs, &c, &p, ov.lat_deg, quiet(ov.lat_deg));
    assert!(at.layers[2].optical_depth > away.layers[2].optical_depth);
    // A dark spot darkens its centre.
    let n = fixture("Neptune");
    let pn = (0..40)
        .map(|k| {
            prepare(
                &gs,
                &n,
                Date {
                    day: 300.0 * k as f64,
                },
            )
        })
        .find(|q| q.systems.iter().any(|s| s.kind == "dark-spot"))
        .expect("a dark spot");
    let ds = pn.systems.iter().find(|s| s.kind == "dark-spot").unwrap();
    let at = giant_cell(&gs, &n, &pn, ds.lat_deg, ds.lon_deg);
    let away = giant_cell(&gs, &n, &pn, ds.lat_deg, wrap(ds.lon_deg + 180.0));
    assert!(at.layers[2].optical_depth < 0.5 * away.layers[2].optical_depth);
    assert_eq!(at.layers[2].condensate, "methane ice");
    // Saturn: the jet follows the hexagon, so the wind at a vertex and at a
    // side's midpoint on the jet circle differ.
    let s = fixture("Saturn");
    let ps = prepare(&gs, &s, Date { day: 0.0 });
    let q = &ps.layout.poles[0];
    let lat = ps.layout.polar_jet_lat_deg;
    let vertex = giant_cell(&gs, &s, &ps, lat, q.orient_lon_deg);
    let side = giant_cell(&gs, &s, &ps, lat, wrap(q.orient_lon_deg + 30.0));
    assert!(vertex.layers[2].wind_m_s[0] > side.layers[2].wind_m_s[0]);
}

#[test]
fn tiles_are_deterministic() {
    let gs = base_settings();
    let key = TileKey::new(2, 2, 1, 3).unwrap();
    for f in fixtures() {
        let a = giant_tile(&gs, &f.conditions, Date { day: 1234.5 }, key, 6);
        let b = giant_tile(&gs, &f.conditions, Date { day: 1234.5 }, key, 6);
        assert_eq!(a, b);
        assert_eq!(a.cells.len(), 36);
        assert!(a.cells.iter().all(|x| x.layers.len() == LAYERS.len()
            && x.layers
                .iter()
                .all(|y| (0.0..=1.0).contains(&y.cloud_cover))));
    }
}

/// Golden: each fixture's layout numbers, polar regimes, systems on two
/// dates and one field tile, by digest.
fn golden() -> serde_json::Value {
    let gs = base_settings();
    let key = TileKey::new(0, 1, 0, 0).unwrap();
    let rows: Vec<_> = fixtures()
        .iter()
        .map(|f| {
            let c = &f.conditions;
            let l = layout(&gs, c);
            let s1 = giant_systems(&gs, c, Date { day: 1000.0 });
            let s2 = giant_systems(&gs, c, Date { day: 9000.0 });
            let t = giant_tile(&gs, c, Date { day: 1000.0 }, key, 8);
            let flow = crate::eddies::flow_table(&gs, &crate::scenario::settings().climate, c, 1000.0, 5.0);
            serde_json::json!({
                "world": c.label,
                "jets": l.jets,
                "burger": l.burger,
                "a_over_ld": l.a_over_ld,
                "polar_jet_lat_deg": l.polar_jet_lat_deg,
                "polygon_waves": l.polygon_waves,
                "poles": l.poles.iter().map(|p| serde_json::json!({
                    "north": p.north, "regime": p.regime, "ring": p.ring_count, "polygon": p.polygon_sides,
                })).collect::<Vec<_>>(),
                "systems_day_1000": s1.len(),
                "systems_day_9000": s2.len(),
                "layout": digest(&l),
                "systems": digest(&(s1, s2)),
                "field": digest(&t),
                "flow": digest(&flow),
                "eddies_day_1000_span_5": flow.eddies.len(),
                "eddy_power_w_kg": flow.facts.eddy_power_w_kg,
            })
        })
        .collect();
    serde_json::json!({ "version": crate::WEATHER_VERSION, "giants": rows })
}

#[test]
fn golden_weather_giants() {
    check_golden("weather-giants.json", &golden());
}

#[test]
fn deck_bases_sit_where_their_vapour_saturates() {
    let gs = base_settings();
    let cs = crate::scenario::settings().climate;
    let dh = gs.clouds.heights.as_ref().unwrap();
    for f in fixtures() {
        let c = &f.conditions;
        let l = layout(&gs, c);
        let p = deck_base_bar(dh, &cs, c, &l.layers);
        assert!(p[0] > p[1] && p[1] > p[2], "{}: {p:?}", c.label);
    }
    // Ice giants' methane cloud near 1.2 to 1.5 bar, Saturn's ammonia near
    // 1.5 (deeper than Jupiter's: colder at 1 bar, less ammonia).
    for (w, lo, hi) in [
        ("Saturn", 1.2, 2.0),
        ("Uranus", 1.0, 1.6),
        ("Neptune", 1.1, 1.8),
    ] {
        let c = fixture(w);
        let p = deck_base_bar(dh, &cs, &c, &layout(&gs, &c).layers);
        assert!((lo..hi).contains(&p[2]), "{w}: {}", p[2]);
    }
    // Jupiter's ammonia cloud base near 0.7 bar (Braude et al. 2020: no
    // deeper than 0.7 ± 0.1 bar outside the Great Red Spot).
    let c = fixture("Jupiter");
    let l = layout(&gs, &c);
    let p = deck_base_bar(dh, &cs, &c, &l.layers);
    assert_eq!(l.layers[2], "ammonia ice");
    assert!((0.5..0.9).contains(&p[2]), "{}", p[2]);
    // Heights rise with the decks and the cell carries them.
    let gp = prepare(&gs, &c, Date { day: 0.0 });
    let mut cell = giant_cell(&gs, &c, &gp, 10.0, 0.0);
    set_heights(&gs, &cs, &c, &l, &mut cell);
    assert!(cell.layers[2].base_m > cell.layers[1].base_m);
    assert!(cell.layers[2].top_m > cell.layers[2].base_m);
}

#[test]
fn sheared_ovals_follow_the_steady_patch_relation() {
    let gs = base_settings();
    let c = fixture("Jupiter");
    let l = layout(&gs, &c);
    let a = c.radius_m;
    let mut seen = 0;
    for k in 0..40 {
        let p = prepare(
            &gs,
            &c,
            Date {
                day: 250.0 * k as f64,
            },
        );
        for s in p.systems.iter().filter(|s| s.kind == "oval") {
            seen += 1;
            // The relation, back from the aspect: ζ / ω = λ (λ - 1) / (λ + 1).
            let lam = s.aspect;
            let h = if s.lat_deg >= 0.0 { 1.0 } else { -1.0 };
            let omega = -h * 2.0 * m::exp(0.5) * s.strength / (0.5 * s.size_km * 1000.0);
            let q = zonal_vorticity(&gs, &l, a, s.lat_deg) / omega;
            let back = lam * (lam - 1.0) / (lam + 1.0);
            assert!(
                (back - q).abs() < 1e-9 || (lam - (m::sqrt(2.0) - 1.0)).abs() < 1e-12,
                "{}: λ {lam} gives {back}, ζ/ω {q}",
                s.id
            );
            assert!(lam > 0.0);
        }
    }
    assert!(seen > 0, "no oval in forty dates");
    // Absolute: an anticyclone in anticyclonic shear of a third its own vorticity: λ² - (4/3)λ - 1/3 = 0.
    let want = (4.0 / 3.0 + m::sqrt(16.0 / 9.0 + 4.0 / 3.0)) / 2.0;
    let q = 1.0 / 3.0;
    assert!((want * (want - 1.0) / (want + 1.0) - q).abs() < 1e-12);
    assert!((want - 1.548_583_770_354_863_5).abs() < 1e-9);
}

#[test]
fn an_oval_is_longer_along_the_wind() {
    let gs = base_settings();
    let c = fixture("Jupiter");
    let mut p = prepare(&gs, &c, Date { day: 0.0 });
    // One oval alone, 2.0 times longer east-west, at 20°N.
    p.systems = vec![WeatherSystem {
        id: "test".into(),
        kind: "oval".into(),
        lat_deg: 20.0,
        lon_deg: 0.0,
        track: vec![],
        size_km: 10000.0,
        strength: 100.0,
        born: 0.0,
        ends: 1e9,
        aspect: 2.0,
        count: 0,
        why: String::new(),
        evidence: "rule".into(),
    }];
    let a = c.radius_m;
    let deg = |km: f64| km * 1000.0 / a * 180.0 / m::PI;
    // 5000 km east and 2500 km north are both on the same contour (r = 5000 / (5000 √2) √2... equal elliptical radius).
    let east = giant_cell(
        &gs,
        &c,
        &p,
        20.0,
        deg(5000.0 * m::sqrt(2.0)) / m::cos(20.0 * m::PI / 180.0),
    );
    let north = giant_cell(&gs, &c, &p, 20.0 + deg(5000.0 / m::sqrt(2.0)), 0.0);
    // The storm's own share: the deck with it over the deck without it, at the same place.
    let mut bare = p.clone();
    bare.systems.clear();
    let gain = |x: &Cell| {
        x.layers[2].optical_depth
            - giant_cell(&gs, &c, &bare, x.lat_deg, x.lon_deg).layers[2].optical_depth
    };
    assert!(
        (gain(&east) - gain(&north)).abs() < 0.02,
        "east {} north {}",
        gain(&east),
        gain(&north)
    );
    assert!(gain(&east) > 0.5);
    // And the same distance north is further out than east.
    let n2 = giant_cell(&gs, &c, &p, 20.0 + deg(5000.0 * m::sqrt(2.0)), 0.0);
    assert!(gain(&n2) < 0.5 * gain(&east));
}

#[test]
fn a_backwards_spinner_mirrors_its_winds_and_vortices() {
    let gs = base_settings();
    let fwd = fixture("Jupiter");
    let mut back = fwd.clone();
    back.rotation_hours = -fwd.rotation_hours;
    let (lf, lb) = (layout(&gs, &fwd), layout(&gs, &back));
    assert!(lb.retrograde && !lf.retrograde);
    for lat in [-60.0, -20.0, 0.0, 15.0, 45.0] {
        let (u, v) = (
            zonal_wind(&gs, &lf, fwd.radius_m, lat),
            zonal_wind(&gs, &lb, back.radius_m, lat),
        );
        assert!((u + v).abs() < 1e-9, "{lat}: {u} vs {v}");
    }
    // The north polar vortex's winds go round the other way.
    let pf = prepare(&gs, &fwd, Date { day: 0.0 });
    let pb = prepare(&gs, &back, Date { day: 0.0 });
    let s = pf
        .systems
        .iter()
        .find(|s| s.lat_deg > 80.0 && s.kind == "cyclone")
        .expect("a polar cyclone");
    let lat = s.lat_deg - 0.3 * (0.5 * s.size_km) * 1000.0 / fwd.radius_m * 180.0 / m::PI;
    let cf = giant_cell(&gs, &fwd, &pf, lat, s.lon_deg);
    let cb = giant_cell(&gs, &back, &pb, lat, s.lon_deg);
    let (wf, wb) = (cf.layers[2].wind_m_s[0], cb.layers[2].wind_m_s[0]);
    assert!(wf * wb < 0.0, "{wf} vs {wb}");
}

#[test]
fn a_patch_reads_the_same_cells_closer() {
    let gs = base_settings();
    let c = fixture("Saturn");
    let block = crate::scenario::block();
    let date = Date { day: 0.0 };
    let pt = block.patch(&c, date, 90.0, 0.0, 20000.0, 24, &[]);
    assert_eq!(pt.cells.len(), 24 * 24);
    // Each cell is the giant's own cell at its place.
    let p = prepare(&gs, &c, date);
    for k in [0usize, 100, 300, 575] {
        let x = &pt.cells[k];
        let want = giant_cell(&gs, &c, &p, x.lat_deg, x.lon_deg);
        assert_eq!(x.layers[2].optical_depth, want.layers[2].optical_depth);
    }
    // The hexagon: cells inside its edge have the thinner high deck, cells just outside the thicker one, on a vertex
    // ray and a mid-side ray alike, though the edge sits nearer the pole along the mid-side ray.
    let l = layout(&gs, &c);
    let q = l.poles.iter().find(|q| q.north).unwrap();
    assert_eq!(q.polygon_sides, 6);
    let r_jet = (90.0 - l.polar_jet_lat_deg) / (180.0 / m::PI) * c.radius_m / 1000.0;
    let mid = polygon_edge_km(r_jet, 6, q.orient_lon_deg, q.orient_lon_deg + 30.0);
    assert!((mid - r_jet * m::cos(m::PI / 6.0)).abs() < 1e-6);
    // Directions round the centre: patch_dir at a pole gives the bearing's longitude.
    let d = crate::rules::patch_dir(c.radius_m, 90.0, 0.0, 1000.0, 0.0);
    let (lat, lon) = crate::rules::lat_lon(d);
    assert!((lon - 90.0).abs() < 1e-6 && lat < 90.0 && lat > 89.0);
    let d = crate::rules::patch_dir(c.radius_m, 0.0, 30.0, 0.0, 1000.0);
    let (lat, lon) = crate::rules::lat_lon(d);
    assert!(lat > 0.0 && (lon - 30.0).abs() < 1e-9);
}

/// Largest divergence and largest vorticity of the top deck's wind over a small grid round a place, s⁻¹, by central
/// differences on the sphere: div = (1 / a cos φ) (∂u/∂λ + ∂(v cos φ)/∂φ).
fn div_and_curl(
    gs: &GiantSettings,
    c: &Conditions,
    p: &GiantPrepared,
    lat0: f64,
    lon0: f64,
    half_deg: f64,
) -> (f64, f64) {
    let d = m::PI / 180.0;
    let a = c.radius_m;
    let h = 1e-3;
    let w = |lat: f64, lon: f64| giant_cell(gs, c, p, lat, lon).layers[2].wind_m_s;
    let (mut div, mut curl) = (0.0f64, 0.0f64);
    for i in 0..9 {
        for j in 0..9 {
            let lat = lat0 + half_deg * (i as f64 / 4.0 - 1.0);
            let lon = lon0 + half_deg * (j as f64 / 4.0 - 1.0);
            let cp = m::cos(lat * d);
            let (e, wst) = (w(lat, lon + h), w(lat, lon - h));
            let (n, s) = (w(lat + h, lon), w(lat - h, lon));
            let du_dl = (e[0] - wst[0]) / (2.0 * h * d);
            let dv_dl = (e[1] - wst[1]) / (2.0 * h * d);
            let dvc = (n[1] * m::cos((lat + h) * d) - s[1] * m::cos((lat - h) * d)) / (2.0 * h * d);
            let duc = (n[0] * m::cos((lat + h) * d) - s[0] * m::cos((lat - h) * d)) / (2.0 * h * d);
            div = div.max(((du_dl + dvc) / (a * cp)).abs());
            curl = curl.max(((dv_dl - duc) / (a * cp)).abs());
        }
    }
    (div, curl)
}

#[test]
fn ws1_a_sheared_ovals_winds_pile_nothing_up() {
    let gs = base_settings();
    let c = fixture("Saturn");
    let p = prepare(&gs, &c, Date { day: 0.0 });
    let s = p
        .systems
        .iter()
        .filter(|s| s.kind == "oval" && s.aspect > 1.2)
        .max_by(|x, y| x.aspect.total_cmp(&y.aspect))
        .expect("a sheared oval");
    let half = 0.5 * s.size_km * 1000.0 / c.radius_m * 180.0 / m::PI;
    let (div, curl) = div_and_curl(&gs, &c, &p, s.lat_deg, s.lon_deg, half);
    // Absolute: the vortex turns at about 2e-4 s⁻¹; the round direction inside a stretched distance gave a
    // divergence of 2e-5 s⁻¹, a tenth of that. What is left is the differencing's own error, near 1e-9.
    assert!(curl > 1e-5, "curl {curl:e}");
    assert!(
        div < 1e-8,
        "divergence {div:e} against vorticity {curl:e} (aspect {})",
        s.aspect
    );
}

#[test]
fn ws1_the_hexagons_wind_runs_along_its_sides() {
    let gs = base_settings();
    let c = fixture("Saturn");
    let p = prepare(&gs, &c, Date { day: 0.0 });
    let l = &p.layout;
    let q = l.poles.iter().find(|q| q.north).unwrap();
    assert_eq!(q.polygon_sides, 6);
    let d = m::PI / 180.0;
    let r_jet = (90.0 - l.polar_jet_lat_deg) * d * c.radius_m / 1000.0;
    // A point on the edge a quarter of the way along a side, and the side's direction in the polar plane.
    let lon = q.orient_lon_deg + 15.0;
    let r = polygon_edge_km(r_jet, 6, q.orient_lon_deg, lon);
    let lat = 90.0 - r / (c.radius_m / 1000.0) / d;
    let wind = giant_cell(&gs, &c, &p, lat, lon).layers[2].wind_m_s;
    let v1 = [m::cos(q.orient_lon_deg * d), m::sin(q.orient_lon_deg * d)];
    let v2 = [
        m::cos((q.orient_lon_deg + 60.0) * d),
        m::sin((q.orient_lon_deg + 60.0) * d),
    ];
    let side = [v2[0] - v1[0], v2[1] - v1[1]];
    let (sl, cl) = (m::sin(lon * d), m::cos(lon * d));
    let flow = [wind[0] * -sl + wind[1] * -cl, wind[0] * cl + wind[1] * -sl];
    let cross = (flow[0] * side[1] - flow[1] * side[0])
        / (m::hypot(flow[0], flow[1]) * m::hypot(side[0], side[1]));
    assert!(m::hypot(flow[0], flow[1]) > 50.0, "{wind:?}");
    assert!(
        cross.abs() < m::sin(0.5 * d),
        "the wind crosses the side at {:.2}°",
        m::asin(cross.abs()) / d
    );
    // And it piles nothing up there either, off the corners.
    let (div, curl) = div_and_curl(&gs, &c, &p, lat, lon, 0.5);
    assert!(
        curl > 1e-6 && div < 1e-8,
        "divergence {div:e} against vorticity {curl:e}"
    );
}
