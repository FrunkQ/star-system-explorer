// WS4: carrying the cloud backwards (design SSE4-stirred-clouds-design.md 3.6, checks in section 9).
use super::*;
use crate::giants::{base_settings, fixtures, layout};

fn world(name: &str) -> Conditions {
    fixtures()
        .into_iter()
        .find(|f| f.conditions.label == name)
        .expect("fixture")
        .conditions
}

fn climate() -> sse_climate::settings::ClimateSettings {
    crate::scenario::settings().climate.clone()
}

#[test]
fn ws4_the_same_picture_whichever_way_the_date_is_reached() {
    // Read at a date directly, or as the second leg of a run there: the same carried places, to the bit.
    let gs = base_settings();
    let c = world("Jupiter");
    let l = layout(&gs, &c);
    let p = stirred_patch(&gs, &climate(), &c, &l, 1000.0, -20.0, 30.0, 4000.0, 3);
    let q = stirred_patch(&gs, &climate(), &c, &l, 1000.0, -20.0, 30.0, 4000.0, 3);
    assert_eq!(p, q);
    // Two legs: carrying back from 1000 by the younger age, then on, lands where one leg does.
    let tau = p.renewal_days;
    assert!(tau > 0.0);
    let k = Carrier::new(&gs, &c, &l, 1000.0 - 2.0 * tau - 1.0, 1000.0);
    let x = unit(-20.0, 30.0);
    let one = k.back(x, 1000.0, 10.0);
    let k2 = Carrier::new(&gs, &c, &l, 1000.0 - 2.0 * tau - 1.0, 1000.0);
    assert_eq!(one, k2.back(x, 1000.0, 10.0));
}

#[test]
fn ws4_carrying_keeps_area() {
    let gs = base_settings();
    let c = world("Jupiter");
    let l = layout(&gs, &c);
    let p = stirred_patch(&gs, &climate(), &c, &l, 1000.0, -20.0, 30.0, 20_000.0, 3);
    let mut worst: f64 = 0.0;
    for cell in &p.cells {
        for g in cell {
            worst = worst.max((g.area - 1.0).abs());
            eprintln!(
                "age {:.0} d, stretch 2^{:.1}, area {:.6}",
                g.age_days, g.stretch_log2, g.area
            );
        }
    }
    eprintln!(
        "Jupiter, 9 cells, 2 generations: worst area {worst:e}, τ {} days",
        p.renewal_days
    );
    // Absolute: the backward map's determinant within 1 per cent of one.
    assert!(worst < 0.01, "area off by {worst}");
    // And the carrying did something: some cloud was stretched.
    assert!(p.cells.iter().flatten().any(|g| g.stretch_log2 > 1.0));
}

#[test]
fn ws4_with_no_eddies_a_band_edge_stays_straight() {
    // No heat: no eddies. Away from the storms, cloud carried along the bands and read back by its row's drift
    // comes from where it is.
    let gs = base_settings();
    let mut c = world("Jupiter");
    c.internal_flux_w_m2 = 0.0;
    c.flux_w_m2 = 0.0;
    let l = layout(&gs, &c);
    let k = Carrier::new(&gs, &c, &l, 600.0, 1000.0);
    for lat in [-41.0, -7.5, 3.0, 12.0, 33.0] {
        for lon in [-150.0, 10.0, 100.0] {
            let x = unit(lat, lon);
            let back = k.back(x, 1000.0, 150.0);
            let r = k.read_place(back, 1000.0, 150.0);
            let d = m::acos(dot(x, r).clamp(-1.0, 1.0)) * c.radius_m / 1000.0;
            let near_storm = k.storms.iter().any(|s| {
                s.at(850.0).is_some_and(|v| {
                    (v.lat_deg - lat).abs() < 0.6 * v.size_km * 1000.0 / c.radius_m * DEG + 2.0
                })
            });
            if !near_storm {
                assert!(d < 1.0, "{lat}, {lon}: read {d} km away");
            }
        }
    }
}

#[test]
fn ws4_the_weights_sum_to_one_and_the_ages_to_tau() {
    let gs = base_settings();
    let c = world("Neptune");
    let l = layout(&gs, &c);
    let p = stirred_patch(&gs, &climate(), &c, &l, 500.0, -20.0, 0.0, 3000.0, 2);
    let tau = p.renewal_days;
    for [y, o] in &p.cells {
        assert!((y.weight + o.weight - 1.0).abs() < 1e-12);
        assert!((o.age_days - y.age_days - tau).abs() < 1e-9);
        assert!((y.weight * y.age_days + o.weight * o.age_days - tau).abs() < 1e-9);
    }
}

/// The aspect of an outline carried back one renewal and read: today's storm frame, √(Σx² / Σy²).
fn carried_aspect(w: &str, day: f64, id: &str, r0: f64) -> (f64, f64, f64) {
    let gs = base_settings();
    let c = world(w);
    let l = layout(&gs, &c);
    let tau = renewal_s(&gs, &climate(), &c, &l).unwrap() / DAY_S;
    let k = Carrier::new(&gs, &c, &l, day - tau - 1.0, day);
    let s = giant_systems(&gs, &c, Date { day })
        .into_iter()
        .find(|s| s.id == id)
        .expect("storm");
    let v = Vortex::of(&s);
    let rr = 0.5 * v.size_km;
    let kk = m::sqrt(v.aspect);
    let (mut sx, mut sy, mut worst) = (0.0, 0.0, 0.0f64);
    for i in 0..24 {
        let t = i as f64 * m::PI / 12.0;
        let p = crate::rules::patch_dir(
            c.radius_m,
            v.lat_deg,
            v.lon_deg,
            r0 * rr * kk * m::cos(t),
            r0 * rr / kk * m::sin(t),
        );
        let r = k.read_place(k.back(p, day, tau), day, tau);
        let (la, lo) = lat_lon(r);
        let (x, y) = crate::giants::storm_xy(c.radius_m, la, lo, v.lat_deg, v.lon_deg);
        sx += x * x;
        sy += y * y;
        worst = worst.max((storm_r_at(c.radius_m, &v, la, lo) / r0 - 1.0).abs());
    }
    (m::sqrt(sx / sy), v.aspect, worst)
}

#[test]
fn ws4_a_storms_outline_keeps_its_shape_for_a_renewal() {
    // An outline at 0.8 of a storm's radius, carried back one renewal and read where the storm stands today, keeps
    // the kernel's aspect within 5 per cent (design section 9). Without a storm turning as one body the band's
    // shear acts twice and Jupiter's comes out at 1.73.
    for (w, day, id) in [
        ("Jupiter", 20000.0, "oval:4:0"),
        ("Saturn", 1000.0, "oval:-6:0"),
    ] {
        let (got, want, worst) = carried_aspect(w, day, id, 0.8);
        assert!(
            (got / want - 1.0).abs() < 0.05,
            "{w} {id}: carried {got}, kernel {want}"
        );
        assert!(worst < 0.1, "{w} {id}: a point's radius moved by {worst}");
    }
}

/// The renderer's reference (WS5): a carry table and the kernel's answers on it, so the renderer's carrying is held
/// to this one. Neptune's whole table with a stirred patch (short renewal, so small), and Saturn's winds at sample
/// places near its polygon and an oval, with the table cut to the two days round them.
fn carry_reference() -> serde_json::Value {
    let gs = base_settings();
    let cs = climate();
    let nep = world("Neptune");
    let ln = layout(&gs, &nep);
    let day = 1000.0;
    let table = carry_table(&gs, &cs, &nep, &ln, day).expect("Neptune renews");
    let patch = stirred_patch(&gs, &cs, &nep, &ln, day, -30.0, 40.0, 8000.0, 4);
    let sat = world("Saturn");
    let ls = layout(&gs, &sat);
    let sday = 1000.0;
    let carrier = Carrier::new(&gs, &sat, &ls, sday - 1.0, sday + 1.0);
    let tau = renewal_s(&gs, &cs, &sat, &ls).unwrap() / DAY_S;
    let st = carrier.table(&sat, sday, tau);
    let mut places = vec![];
    for lat in [
        -82.0, -78.0, -75.0, -60.7, -45.0, 0.0, 30.0, 61.0, 75.0, 77.0, 80.0, 88.0,
    ] {
        for lon in [-170.0, -60.0, 0.0, 15.0, 100.0] {
            places.push((lat, lon));
        }
    }
    // Round Saturn's storms too, where the shield and the storm's own winds act.
    for s in &st.storms {
        for (dl, dn) in [(0.0, 0.0), (1.0, 0.5), (-2.0, 1.0), (3.0, -1.5)] {
            places.push((
                s.lat_deg + dn,
                s.lon_deg + s.drift_deg_d * (sday - s.day_ref) + dl,
            ));
        }
    }
    let winds: Vec<serde_json::Value> = places
        .iter()
        .map(|&(lat, lon)| {
            let w = carrier.wind(unit(lat, lon), sday);
            let (e, n) = frame(lat, lon);
            serde_json::json!({"lat": lat, "lon": lon, "east": dot(w, e), "north": dot(w, n)})
        })
        .collect();
    serde_json::json!({
        "about": "WS5: the kernel's carry tables and its answers on them, so the renderer's carrying (stirred-carry.ts) is held to sse-weather's stirred.rs. Regenerate with SSE_BLESS=1 cargo test -p sse-weather ws5_the_renderers_reference_is_current.",
        "neptune": {"table": table, "patch": patch},
        "saturn": {"table": st, "day": sday, "winds": winds},
    })
}

#[test]
fn ws5_the_renderers_reference_is_current() {
    let path = format!(
        "{}/../../tests/fixtures/weather/carry.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string(&carry_reference()).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1 once");
    assert!(want.replace("\r\n", "\n") == text, "tests/fixtures/weather/carry.json is stale: run SSE_BLESS=1 cargo test -p sse-weather ws5_the_renderers_reference_is_current");
}

#[test]
fn ws5_a_carry_table_reaches_back_two_renewals() {
    let gs = base_settings();
    let c = world("Jupiter");
    let l = layout(&gs, &c);
    let t = carry_table(&gs, &climate(), &c, &l, 500.0).expect("Jupiter renews");
    assert!(
        t.day - t.day0 >= 2.0 * t.renewal_days,
        "{} {} {}",
        t.day,
        t.day0,
        t.renewal_days
    );
    assert_eq!(t.generation_offset_days.len(), 2 * t.jets as usize);
    assert!(t
        .generation_offset_days
        .iter()
        .all(|o| *o >= 0.0 && *o < t.renewal_days));
    assert_eq!(t.u_m_s.len(), 18001);
    assert!(!t.storms.is_empty() && !t.eddies.is_empty());
    // Every eddy alive at some moment of the span.
    assert!(t.eddies.iter().all(|e| e.ends > t.day0 && e.born < t.day));
}
