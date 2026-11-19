use crate::atlas::{downsample_error_m, face_of, Atlas, TileKey, COVERS};
use crate::cover::*;
use crate::impacts::{impacts, ImpactHistory, ImpactInputs};
use crate::named::{register, AuthoredName, Rename};
use crate::plates::*;
use crate::scenario::{fixtures, settings};
use crate::wear::*;
use crate::*;
use sse_core::draw::Seed;
use sse_core::math as m;

fn seed() -> Seed {
    Seed::from_text("sse4-cover-tests")
}

fn quick() -> SurfaceSettings {
    let mut s = settings();
    s.plates.cells_per_face = 16;
    s.atlas.tile_samples = 32;
    s
}

fn body(name: &str) -> Body {
    fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body
}

fn inputs<'a>(
    s: &SurfaceSettings,
    b: &Body,
    e: &SurfaceEnvelope,
    plates: Option<&'a PlateHistory>,
    rot: Option<f64>,
) -> CoverInputs<'a> {
    CoverInputs {
        air: Air::guess(&s.cover, e, b.pressure_bar, b.oceans > 0.0),
        water: b.oceans > 0.0 || b.water_fraction > 0.0,
        rotation_hours: rot,
        obliquity_deg: None,
        life: 0.0,
        plates,
        reshape: vec![],
        settle: vec![],
    }
}

/// Earth through plates and wearing down, its atlas, and the inputs.
fn earth(s: &SurfaceSettings) -> (Body, SurfaceEnvelope, PlateHistory, Atlas) {
    let b = body("Earth");
    let e = envelope(s, &b);
    let h = history(s, &e, &seed(), b.age_gyr);
    let w = wear_world(
        s,
        &e,
        &seed(),
        &h,
        &WaterHistory::steady(b.oceans, b.pressure_bar, b.age_gyr),
        b.magma_end_myr,
    );
    let a = Atlas::from_worn(s, &e, &seed(), &h, &w, vec![]);
    (b, e, h, a)
}

fn earth_layered(
    s: &SurfaceSettings,
    f: impl FnOnce(&mut CoverInputs),
) -> (SurfaceEnvelope, Atlas) {
    let (b, e, h, a) = earth(s);
    let mut inp = inputs(s, &b, &e, Some(&h), Some(23.93));
    f(&mut inp);
    let ly = layers(s, &e, &seed(), &a, &inp);
    (e.clone(), a.with_layers(ly))
}

fn moon(s: &SurfaceSettings) -> (Body, SurfaceEnvelope, Atlas) {
    let b = body("Moon");
    let e = envelope(s, &b);
    let water = WaterHistory::steady(0.0, 0.0, b.age_gyr);
    let hist = ImpactHistory::lunar();
    let inp = ImpactInputs {
        history: &hist,
        water: &water,
        pressure_bar: 0.0,
        plates: None,
    };
    let (feat, field) = impacts(s, &e, &seed(), &inp, b.age_gyr).unwrap();
    let a = Atlas::new(s, &e, &seed(), b.age_gyr, vec![]).with_impacts(feat, field);
    (b, e, a)
}

#[test]
fn climate_follows_air_rotation_and_height() {
    let s = quick();
    let cs = &s.cover;
    let clim = |name: &str, rot: f64| {
        let b = body(name);
        let e = envelope(&s, &b);
        let a = Atlas::new(&s, &e, &seed(), b.age_gyr, vec![]);
        let inp = inputs(&s, &b, &e, None, Some(rot));
        layers(&s, &e, &seed(), &a, &inp).climate
    };
    let earth = clim("Earth", 23.93);
    assert_eq!(earth.cells, 3, "Earth has three cells a hemisphere");
    let eq = earth.t_at(0.0, 0.0);
    let pole = earth.t_at(m::PI / 2.0, 0.0);
    assert!(
        (eq - pole - 42.0).abs() < 4.0,
        "Earth equator {eq} pole {pole}"
    );
    assert!((earth.lapse_k_km - 6.5).abs() < 0.3, "{}", earth.lapse_k_km);
    // Equatorial snow line: where the mean falls to the land-ice threshold.
    let snow_km = (eq - cs.land_ice_k) / earth.lapse_k_km;
    assert!((4.5..6.5).contains(&snow_km), "snow line {snow_km} km");
    let venus = clim("Venus", 5832.5);
    assert_eq!(
        venus.cells, 1,
        "Venus turns too slowly for more than one cell"
    );
    assert!(venus.contrast_k < 15.0, "{}", venus.contrast_k);
    let mars = clim("Mars", 24.62);
    assert!(mars.contrast_k > 80.0, "{}", mars.contrast_k);
    assert_eq!(mars.lapse_k_km, 0.0, "air too thin for a lapse here");
    let titan = clim("Titan", 382.7);
    assert!(titan.contrast_k < 5.0, "{}", titan.contrast_k);
    assert_eq!(titan.cells, 1);
    assert_eq!(
        titan.liquid.as_deref(),
        Some("hydrocarbon"),
        "Titan's seas are hydrocarbon"
    );
    // Bands: wet at the equator, dry at 30°, wet at 60°, dry at the poles.
    let w = |d: f64| earth.band_wetness(d.to_radians());
    assert!(w(0.0) > 0.99 && w(30.0) < 0.01 && w(60.0) > 0.99 && w(90.0) < 0.01);
    assert!(earth.easterly(10f64.to_radians()) && !earth.easterly(45f64.to_radians()));
}

#[test]
fn ice_where_it_is_cold_enough() {
    let s = quick();
    let (_, a) = earth_layered(&s, |_| {});
    let ly = a.layers().unwrap();
    // Poles icy, the equator not, on the level-0 globe.
    let mut polar = (0usize, 0usize);
    let mut tropic = (0usize, 0usize);
    for f in 0..6u8 {
        let t = a.build(TileKey::new(f, 0, 0, 0).unwrap());
        let n = t.samples as usize;
        for j in 0..n {
            for i in 0..n {
                let u = -1.0 + 2.0 * (i as f64 + 0.5) / n as f64;
                let v = -1.0 + 2.0 * (j as f64 + 0.5) / n as f64;
                let d = crate::atlas::direction(f, u, v);
                let lat = m::asin(d[2]).to_degrees().abs();
                let ice = COVERS[t.cover[j * n + i] as usize] == "ice";
                let h = t.elevation_dm[j * n + i] as f64 / 10.0;
                if lat > 80.0 {
                    polar.0 += 1;
                    polar.1 += ice as usize;
                }
                if lat < 20.0 && h < 3000.0 {
                    tropic.0 += 1;
                    tropic.1 += ice as usize;
                }
                if ice {
                    let wet = t.wetness[j * n + i] as f64 / 255.0;
                    let temp = ly.climate.t_at(m::asin(d[2]), h);
                    let limit = if h < 0.0 { ly.sea_ice_k } else { ly.land_ice_k };
                    assert!(
                        temp < limit + 0.5 * ly.ice_dry_k * (1.0 - wet) + 1.0,
                        "ice at {temp} K"
                    );
                }
            }
        }
    }
    assert!(polar.1 * 10 >= polar.0 * 9, "polar ice {polar:?}");
    assert_eq!(tropic.1, 0, "no ice on low tropical ground {tropic:?}");
    // A dry world keeps ice only in its cold trap; an airless one without water has none.
    let mars = body("Mars");
    let e = envelope(&s, &mars);
    let am = Atlas::new(&s, &e, &seed(), mars.age_gyr, vec![]);
    let mut inp = inputs(&s, &mars, &e, None, Some(24.62));
    inp.water = true;
    let am = am.clone().with_layers(layers(&s, &e, &seed(), &am, &inp));
    let t = am.build(TileKey::new(4, 0, 0, 0).unwrap());
    assert!(
        t.cover.iter().any(|c| COVERS[*c as usize] == "ice"),
        "Mars north cap"
    );
    let t = am.build(TileKey::new(0, 0, 0, 0).unwrap());
    assert!(
        t.cover.iter().all(|c| COVERS[*c as usize] != "ice"),
        "no ice at Mars's equator"
    );
}

#[test]
fn cover_agrees_between_globe_and_ground() {
    let s = quick();
    let (_, a) = earth_layered(&s, |_| {});
    for &(f, x, y) in &[(0u8, 0u32, 0u32), (4, 0, 0), (5, 1, 0)] {
        let parent = a.build(TileKey::new(f, 1, x, y).unwrap());
        let n = parent.samples as usize;
        let (mut same, mut total) = (0usize, 0usize);
        let mut worst_al: f64 = 0.0;
        for child in TileKey::new(f, 1, x, y).unwrap().children() {
            let c = a.build(child);
            assert!(
                downsample_error_m(&parent, &c) <= a.tolerance_m(2),
                "heights"
            );
            let half = n / 2;
            let (ox, oy) = ((child.x % 2) as usize * half, (child.y % 2) as usize * half);
            for j in 0..half {
                for i in 0..half {
                    let k = |a: usize, b: usize| (2 * j + b) * n + 2 * i + a;
                    let pc = parent.cover[(oy + j) * n + ox + i];
                    let kids = [k(0, 0), k(1, 0), k(0, 1), k(1, 1)];
                    total += 1;
                    if kids.iter().any(|q| c.cover[*q] == pc) {
                        same += 1;
                    }
                    let al: f64 = kids.iter().map(|q| c.albedo[*q] as f64).sum::<f64>() / 4000.0;
                    let pa = parent.albedo[(oy + j) * n + ox + i] as f64 / 1000.0;
                    worst_al = worst_al.max((al - pa).abs());
                }
            }
        }
        assert!(
            same * 100 >= total * 95,
            "cover agrees on {same} of {total}"
        );
        assert!(worst_al <= 0.6, "albedo {worst_al}");
        assert_eq!(parent.colour.len(), n * n);
    }
}

#[test]
fn wetness_falls_inland_and_behind_mountains() {
    let s = quick();
    let (_, a) = earth_layered(&s, |_| {});
    let ly = a.layers().unwrap();
    let nlat = (180.0 / ly.grid_deg).round() as usize;
    let nlon = 2 * nlat;
    let step = ly.grid_deg;
    let mut coast = (0.0, 0usize);
    let mut shadow = (0.0, 0usize);
    let mut open = (0.0, 0usize);
    for i in 0..nlat {
        let la = -90.0 + (i as f64 + 0.5) * step;
        let band = ly.climate.band_wetness(la.to_radians());
        if band < 0.5 {
            continue;
        }
        let dir: i64 = if ly.climate.easterly(la.to_radians()) {
            1
        } else {
            -1
        };
        for j in 0..nlon {
            let at = |jj: i64| {
                let lo = -180.0 + ((jj.rem_euclid(nlon as i64)) as f64 + 0.5) * step;
                a.height(
                    [
                        m::cos(la.to_radians()) * m::cos(lo.to_radians()),
                        m::cos(la.to_radians()) * m::sin(lo.to_radians()),
                        m::sin(la.to_radians()),
                    ],
                    0,
                )
            };
            let (h, p) = at(j as i64);
            if p.liquid {
                continue;
            }
            let w = ly.wet[i * nlon + j] as f64 / 255.0 / band;
            let (_, up1) = at(j as i64 + dir);
            let up_high = (1..4)
                .map(|k| at(j as i64 + dir * k).0)
                .fold(f64::MIN, f64::max);
            if up1.liquid {
                coast.0 += w;
                coast.1 += 1;
            } else if up_high > h + 2000.0 {
                shadow.0 += w;
                shadow.1 += 1;
            } else {
                open.0 += w;
                open.1 += 1;
            }
        }
    }
    let mean = |x: (f64, usize)| x.0 / x.1.max(1) as f64;
    assert!(coast.1 > 0 && open.1 > 0, "{coast:?} {open:?}");
    assert!(
        mean(coast) > mean(open),
        "coast {} inland {}",
        mean(coast),
        mean(open)
    );
    if shadow.1 > 0 {
        assert!(
            mean(shadow) < mean(open),
            "shadow {} open {}",
            mean(shadow),
            mean(open)
        );
    }
}

#[test]
fn deposits_follow_the_history() {
    let s = quick();
    let (b, e, h, a) = earth(&s);
    let mut inp = inputs(&s, &b, &e, Some(&h), Some(23.93));
    let lifeless = layers(&s, &e, &seed(), &a, &inp);
    inp.life = 1.0;
    let living = layers(&s, &e, &seed(), &a, &inp);
    let count = |l: &Layers, k: &str| {
        let code = DEPOSITS.iter().position(|x| *x == k).unwrap() as u8;
        l.cell_deposit.iter().filter(|c| **c == code).count()
    };
    assert!(count(&lifeless, "copper-arc") > 0, "arcs leave copper");
    assert_eq!(
        count(&lifeless, "carbon-buried"),
        0,
        "no life, no buried carbon"
    );
    assert!(
        count(&living, "carbon-buried") > 0,
        "life in thick sediment"
    );
    for (ci, c) in h.cells.iter().enumerate() {
        if lifeless.cell_deposit[ci] == 1 {
            assert!(c.continental && c.arc_myr.is_some());
        }
    }
    // The Moon: ice only in polar craters that never see the Sun.
    let (mb, me, ma) = moon(&s);
    let ml = layers(
        &s,
        &me,
        &seed(),
        &ma,
        &inputs(&s, &mb, &me, None, Some(655.7)),
    );
    let ice: Vec<_> = ml
        .deposits
        .iter()
        .filter(|d| d.kind == "ice-shadowed")
        .collect();
    assert!(!ice.is_empty(), "lunar polar ice");
    assert!(
        ice.iter().all(|d| d.lat_deg.abs() > 55.0),
        "{:?}",
        ice.iter().map(|d| d.lat_deg).collect::<Vec<_>>()
    );
    assert!(ml.deposits.iter().any(|d| d.kind == "nickel-impact"));
    let ma = ma.clone().with_layers(ml.clone());
    let d = &ice[0];
    let t = ma.build({
        let (f, u, v) = face_of(d.dir);
        let n = 1u32 << 3;
        TileKey::new(
            f,
            3,
            ((u + 1.0) * 0.5 * n as f64) as u32,
            ((v + 1.0) * 0.5 * n as f64) as u32,
        )
        .unwrap()
    });
    assert!(t.tags.contains(&d.id), "deposit tagged in its tile");
    assert!(ml
        .explain(d.lat_deg, d.lon_deg, 0.0, me.reference_radius_m / 1000.0)
        .iter()
        .any(|x| x.contains("never sees the Sun")));
}

#[test]
fn aerostat_band_matches_the_air_profile() {
    let s = quick();
    let cs = &s.cover;
    let band = |name: &str| {
        let b = body(name);
        let e = envelope(&s, &b);
        let air = Air::guess(cs, &e, b.pressure_bar, b.oceans > 0.0);
        (aerostat_band(cs, &e, &air, None), air, e)
    };
    let (v, air, e) = band("Venus");
    let v = v.expect("Venus has a cloud-city band");
    // Venus International Reference Atmosphere (Seiff et al. 1985): about 1 bar
    // and 350 K at 50 km, 0.5 bar and 300 K at 55 km.
    assert!((49.0..54.0).contains(&v.low_km), "{v:?}");
    assert!((53.0..57.5).contains(&v.high_km), "{v:?}");
    for (z, p, t) in [
        (v.low_km, v.p_bar[0], v.t_k[0]),
        (v.high_km, v.p_bar[1], v.t_k[1]),
    ] {
        let (tt, pp) = air.profile(cs, e.gravity_m_s2, z * 1000.0);
        assert!((tt - t).abs() < 1e-9 && (pp - p).abs() < 1e-12);
        assert!(p >= cs.aerostat.p_bar[0] - 1e-9 && p <= cs.aerostat.p_bar[1] + 1e-9);
        assert!(t >= cs.aerostat.t_k[0] - 1e-9 && t <= cs.aerostat.t_k[1] + 1e-9);
    }
    // Just outside the band the profile fails the rule.
    let (to, po) = air.profile(cs, e.gravity_m_s2, (v.high_km + 0.2) * 1000.0);
    assert!(po < cs.aerostat.p_bar[0] || to < cs.aerostat.t_k[0]);
    assert!((5.0..10.0).contains(&v.circuit_days), "{}", v.circuit_days);
    let (earth, _, _) = band("Earth");
    let earth = earth.unwrap();
    assert_eq!(earth.low_km, 0.0);
    assert!((1.5..3.5).contains(&earth.high_km), "{earth:?}");
    assert!(band("Mars").0.is_err(), "too thin");
    let (j, _, _) = band("Jupiter");
    let msg = j.expect_err("no shirt-sleeve band on Jupiter");
    assert!(msg.contains("hotter only deeper"), "{msg}");
}

#[test]
fn gm_edits_are_dated_events_and_tagged_when_impossible() {
    let s = quick();
    let (e0, plain) = earth_layered(&s, |_| {});
    let epoch = plain.epoch_myr as f64;
    let edits = vec![
        Reshape::RaiseRange {
            id: "gm:range".into(),
            from: [10.0, 20.0],
            to: [10.0, 40.0],
            height_m: 5000.0,
            width_km: 150.0,
            at_myr: epoch - 1.0,
        },
        Reshape::RaiseRange {
            id: "gm:tower".into(),
            from: [-20.0, 100.0],
            to: [-20.0, 110.0],
            height_m: 40_000.0,
            width_km: 100.0,
            at_myr: epoch - 1.0,
        },
        Reshape::RaiseRange {
            id: "gm:later".into(),
            from: [40.0, 0.0],
            to: [40.0, 10.0],
            height_m: 3000.0,
            width_km: 100.0,
            at_myr: epoch + 10.0,
        },
        Reshape::DropCrater {
            id: "gm:crater".into(),
            lat_deg: 0.0,
            lon_deg: -60.0,
            d_km: 300.0,
            at_myr: epoch - 1.0,
        },
        Reshape::DropCrater {
            id: "gm:shatter".into(),
            lat_deg: 0.0,
            lon_deg: 180.0,
            d_km: 13_000.0,
            at_myr: epoch - 1.0,
        },
        Reshape::FloodBasin {
            id: "gm:flood".into(),
            lat_deg: 10.0,
            lon_deg: 30.0,
            radius_km: 800.0,
            level_m: 2000.0,
            at_myr: epoch - 1.0,
        },
    ];
    let (_, a) = earth_layered(&s, |i| i.reshape = edits.clone());
    let ly = a.layers().unwrap();
    let ap = |id: &str| ly.applied.iter().find(|x| x.id == id).unwrap();
    assert!(ap("gm:range").impossible.is_none() && ap("gm:range").shown);
    assert!(ap("gm:tower")
        .impossible
        .as_deref()
        .unwrap()
        .contains("cannot stand"));
    assert!(!ap("gm:later").shown, "dated after the epoch");
    assert!(ap("gm:crater").impossible.is_none());
    assert!(ap("gm:shatter").impossible.is_some());
    assert!(ap("gm:flood").impossible.is_none());
    assert!(ly.landforms.iter().all(|l| l.id != "gm:later"));
    // The range raises the tiles it crosses; the crater digs; the flood covers.
    let mid = [
        m::cos(10f64.to_radians()) * m::cos(30f64.to_radians()),
        m::cos(10f64.to_radians()) * m::sin(30f64.to_radians()),
        m::sin(10f64.to_radians()),
    ];
    let lvl = 5;
    let rise = a.height_at(mid, lvl) - plain.height_at(mid, lvl);
    let gm = ly.landforms.iter().find(|l| l.id == "gm:range").unwrap();
    let direct = crate::regimes::landform_height(gm, mid, a.radius_m / 1000.0).0;
    assert!(
        rise > 4000.0,
        "rise {rise} direct {direct}, range {gm:?}, min {}",
        a.min_feature_km(lvl)
    );
    let cr = [
        m::cos((-60f64).to_radians()),
        m::sin((-60f64).to_radians()),
        0.0,
    ];
    assert!(a.height_at(cr, lvl) < plain.height_at(cr, lvl) - 500.0);
    let (f, u, v) = face_of(mid);
    let n = 1u32 << lvl;
    let key = TileKey::new(
        f,
        lvl,
        ((u + 1.0) * 0.5 * n as f64) as u32,
        ((v + 1.0) * 0.5 * n as f64) as u32,
    )
    .unwrap();
    let t = a.build(key);
    assert!(t.tags.iter().any(|x| x == "gm:range") && t.tags.iter().any(|x| x == "gm:flood"));
    // Mars cannot hold a lake.
    let mars = body("Mars");
    let me = envelope(&s, &mars);
    let am = Atlas::new(&s, &me, &seed(), mars.age_gyr, vec![]);
    let mut inp = inputs(&s, &mars, &me, None, Some(24.62));
    inp.reshape = vec![Reshape::FloodBasin {
        id: "gm:lake".into(),
        lat_deg: 0.0,
        lon_deg: 0.0,
        radius_km: 100.0,
        level_m: 0.0,
        at_myr: 0.0,
    }];
    let ml = layers(&s, &me, &seed(), &am, &inp);
    assert!(ml.applied[0].impossible.as_deref().unwrap().contains("611"));
    // Tolerance still holds with the GM's relief.
    for l in 1..=lvl {
        let child = key;
        let child =
            TileKey::new(child.face, l, child.x >> (lvl - l), child.y >> (lvl - l)).unwrap();
        let p = a.build(child.parent().unwrap());
        let c = a.build(child);
        assert!(downsample_error_m(&p, &c) <= a.tolerance_m(l), "level {l}");
    }
    let _ = e0;
}

#[test]
fn the_terrain_honours_pins() {
    let s = quick();
    let (_, plain) = earth_layered(&s, |_| {});
    let floor = plain.level_for(s.atlas.fact_floor_m);
    // A sample centre at the floor level.
    let n = 1u64 << floor;
    let total = 32.0 * n as f64;
    let (f, gi, gj) = (0u8, 32 * (n / 2) + 7, 32 * (n / 2) + 11);
    let d = crate::atlas::direction(
        f,
        -1.0 + 2.0 * (gi as f64 + 0.5) / total,
        -1.0 + 2.0 * (gj as f64 + 0.5) / total,
    );
    let (la, lo) = (
        m::asin(d[2]).to_degrees(),
        m::atan2(d[1], d[0]).to_degrees(),
    );
    let before = plain.height_at(d, floor);
    let target = before + 750.0;
    let epoch = plain.epoch_myr as f64;
    let (_, a) = earth_layered(&s, |i| {
        i.reshape = vec![
            Reshape::Pin {
                id: "gm:pin".into(),
                lat_deg: la,
                lon_deg: lo,
                height_m: target,
                radius_km: 3.0,
                at_myr: epoch,
            },
            Reshape::Pin {
                id: "gm:deep".into(),
                lat_deg: -30.0,
                lon_deg: 0.0,
                height_m: -60_000.0,
                radius_km: 50.0,
                at_myr: epoch,
            },
        ]
    });
    let key = TileKey::new(f, floor, (gi / 32) as u32, (gj / 32) as u32).unwrap();
    let t = a.build(key);
    let h = t.elevation_dm[(gj % 32) as usize * 32 + (gi % 32) as usize] as f64 / 10.0;
    assert!((h - target).abs() <= 0.15, "pinned {target}, tile says {h}");
    let ly = a.layers().unwrap();
    let ap = |id: &str| ly.applied.iter().find(|x| x.id == id).unwrap();
    assert!(ap("gm:pin").impossible.is_none(), "{:?}", ap("gm:pin"));
    assert!(ap("gm:deep").impossible.is_some());
    let p = a.build(key.parent().unwrap());
    assert!(downsample_error_m(&p, &t) <= a.tolerance_m(floor));
}

#[test]
fn the_built_layer_follows_its_rules() {
    let s = quick();
    let (_, a) = earth_layered(&s, |i| i.settle = vec!["settlement".into(), "port".into()]);
    let ly = a.layers().unwrap();
    let rk = a.radius_m / 1000.0;
    let st: Vec<_> = ly.built.iter().filter(|b| b.kind == "settlement").collect();
    let kind = s
        .cover
        .built
        .iter()
        .find(|k| k.kind == "settlement")
        .unwrap();
    assert!(
        !st.is_empty() && st.len() as u32 <= kind.max,
        "{}",
        st.len()
    );
    for (i, x) in st.iter().enumerate() {
        let t = ly
            .climate
            .t_at(x.lat_deg.to_radians(), a.height(x.dir, 0).0);
        assert!((240.0..325.0).contains(&t), "{} at {t} K", x.id);
        for y in &st[i + 1..] {
            let ang = m::acos(
                (x.dir[0] * y.dir[0] + x.dir[1] * y.dir[1] + x.dir[2] * y.dir[2]).clamp(-1.0, 1.0),
            );
            assert!(ang * rk >= kind.min_spacing_km, "{} {}", x.id, y.id);
        }
        assert!(x.why.iter().any(|w| w.contains("water")));
    }
    assert!(ly.built.iter().any(|b| b.kind == "port"));
    // A settlement shows in its tile at a level fine enough for its footprint.
    let x = st[0];
    let lvl = 8;
    let (f, u, v) = face_of(x.dir);
    let n = 1u32 << lvl;
    let t = a.build(
        TileKey::new(
            f,
            lvl,
            ((u + 1.0) * 0.5 * n as f64) as u32,
            ((v + 1.0) * 0.5 * n as f64) as u32,
        )
        .unwrap(),
    );
    assert!(t.tags.contains(&x.id));
    assert!(t.structure.contains(&1), "settlement footprint");
    // An unsettled world builds nothing; a base needs no air.
    let (_, bare) = earth_layered(&s, |_| {});
    assert!(bare.layers().unwrap().built.is_empty());
    let (mb, me, ma) = moon(&s);
    let mut inp = inputs(&s, &mb, &me, None, Some(655.7));
    inp.settle = vec!["base".into()];
    let ml = layers(&s, &me, &seed(), &ma, &inp);
    assert!(!ml.built.is_empty() && ml.built.iter().all(|b| b.kind == "base"));
}

#[test]
fn features_are_named_and_renames_change_only_names() {
    let s = quick();
    let (e, a) = earth_layered(&s, |_| {});
    let reg = register(&s.features, &e, &seed(), &a, &[], vec![]).unwrap();
    let kinds = |k: &str| reg.features.iter().filter(|f| f.kind == k).count();
    assert!(
        kinds("ocean") >= 1,
        "{:?}",
        reg.features.iter().map(|f| &f.kind).collect::<Vec<_>>()
    );
    assert!(
        kinds("continent") >= 1 && kinds("range") >= 1 && kinds("peak") >= 1 && kinds("deep") == 1
    );
    let mut names: Vec<_> = reg
        .features
        .iter()
        .map(|f| f.original_name.clone())
        .collect();
    assert!(names.iter().all(|n| !n.trim().is_empty()));
    names.sort();
    names.dedup();
    assert_eq!(
        names.len(),
        reg.features.len(),
        "names are unique on a world"
    );
    assert!(reg.features.iter().all(|f| f.address.contains("/feature:")));
    let ocean = reg
        .features
        .iter()
        .find(|f| f.kind == "ocean")
        .unwrap()
        .clone();
    assert!(ocean.original_name.ends_with(" Ocean"));
    // Renames are dated, per audience, restorable; nothing else changes.
    let tile_before = a.build(TileKey::new(0, 1, 0, 0).unwrap()).hash;
    let mut r2 = register(
        &s.features,
        &e,
        &seed(),
        &a,
        &[AuthoredName {
            id: "gm:pass".into(),
            kind: "pass".into(),
            lat_deg: 45.0,
            lon_deg: 7.0,
            name: "Wolf Pass".into(),
            at_myr: 4000.0,
        }],
        vec![
            Rename {
                feature: ocean.id.clone(),
                name: "Great Sea".into(),
                at_myr: 4000.0,
                audience: None,
                by: "gm".into(),
                restore: false,
            },
            Rename {
                feature: ocean.id.clone(),
                name: "Mother Water".into(),
                at_myr: 4100.0,
                audience: Some("seat:natives".into()),
                by: "gm".into(),
                restore: false,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        r2.name_of(&ocean.id, None, 3999.0).unwrap(),
        ocean.original_name
    );
    assert_eq!(r2.name_of(&ocean.id, None, 4050.0).unwrap(), "Great Sea");
    assert_eq!(
        r2.name_of(&ocean.id, Some("seat:natives"), 4200.0).unwrap(),
        "Mother Water"
    );
    assert_eq!(
        r2.name_of(&ocean.id, Some("seat:others"), 4200.0).unwrap(),
        "Great Sea"
    );
    r2.rename(Rename {
        feature: ocean.id.clone(),
        name: String::new(),
        at_myr: 4300.0,
        audience: None,
        by: "gm".into(),
        restore: true,
    })
    .unwrap();
    assert_eq!(
        r2.name_of(&ocean.id, None, 4400.0).unwrap(),
        ocean.original_name
    );
    assert_eq!(r2.names(&ocean.id).len(), 4);
    assert!(r2.names(&ocean.id)[0].original);
    assert_eq!(r2.get(&ocean.id).unwrap().address, ocean.address);
    assert_eq!(r2.name_of("gm:pass", None, 4000.0).unwrap(), "Wolf Pass");
    assert!(r2
        .rename(Rename {
            feature: "nowhere".into(),
            name: "X".into(),
            at_myr: 0.0,
            audience: None,
            by: "gm".into(),
            restore: false
        })
        .is_err());
    assert_eq!(a.build(TileKey::new(0, 1, 0, 0).unwrap()).hash, tile_before);
    // Same world, same names.
    let again = register(&s.features, &e, &seed(), &a, &[], vec![]).unwrap();
    assert_eq!(again, reg);
}

#[test]
fn sifters_pick_out_the_remarkable() {
    let s = quick();
    let (e, a) = earth_layered(&s, |_| {});
    let reg = register(&s.features, &e, &seed(), &a, &[], vec![]).unwrap();
    let has = |what: &str| {
        reg.features
            .iter()
            .any(|f| f.remarkable.iter().any(|r| r == what))
    };
    assert!(has("highest point") && has("deepest point") && has("largest body of liquid"));
    let (_, me, ma) = moon(&s);
    let mr = register(&s.features, &me, &seed(), &ma, &[], vec![]).unwrap();
    let big = mr
        .features
        .iter()
        .find(|f| f.remarkable.iter().any(|r| r == "largest crater"))
        .unwrap();
    assert!(mr
        .features
        .iter()
        .filter(|f| f.kind == "crater")
        .all(|f| f.extent_km <= big.extent_km));
    assert!(big.original_name.ends_with(" Crater"));
}

#[test]
fn the_same_world_gives_the_same_layers() {
    let s = quick();
    let (_, a) = earth_layered(&s, |i| i.settle = vec!["settlement".into()]);
    let (_, b) = earth_layered(&s, |i| i.settle = vec!["settlement".into()]);
    assert_eq!(a.layers(), b.layers());
    let k = TileKey::new(2, 2, 1, 3).unwrap();
    assert_eq!(a.build(k), b.build(k));
}

#[test]
fn golden_cover() {
    let got = crate::scenario::cover_golden();
    let path = format!(
        "{}/../../tests/golden/surface-cover.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1");
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden surface-cover changed"
    );
}
