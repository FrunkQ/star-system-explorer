use crate::regimes::*;
use crate::scenario::{fixtures, settings};
use crate::*;
use sse_core::draw::Seed;

fn body(name: &str) -> Body {
    fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body
}

fn surface_with(
    s: &SurfaceSettings,
    b: &Body,
    rot: Option<f64>,
    impacts: Option<&crate::impacts::Features>,
) -> (RegimeSurface, f64) {
    let e = envelope(s, b);
    let inp = RegimeInputs {
        rotation_hours: rot,
        mass_earth: b.mass_earth,
        impacts,
    };
    (
        regime_surface(
            s,
            &e,
            &Seed::from_text("sse4-regimes-tests"),
            &inp,
            b.age_gyr,
        ),
        e.reference_radius_m / 1000.0,
    )
}

fn surface_of(name: &str, rot: Option<f64>) -> (RegimeSurface, f64) {
    surface_with(&settings(), &body(name), rot, None)
}

fn count(r: &RegimeSurface, kind: &str) -> usize {
    r.landforms.iter().filter(|l| l.kind == kind).count()
}

fn mean_age_myr(r: &RegimeSurface, kind: &str) -> f64 {
    let v: Vec<f64> = r
        .landforms
        .iter()
        .filter(|l| l.kind == kind)
        .map(|l| r.epoch_myr - l.formed_myr)
        .collect();
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

#[test]
fn ice_shells_crack_young_and_old() {
    let (eu, er) = surface_of("Europa", None);
    let (ga, _) = surface_of("Ganymede", None);
    // Europa: ridges and bands, all young (its surface renews in about 60 Myr), with chaos.
    assert!(count(&eu, "double-ridge") > 100 && count(&eu, "band") > 10);
    assert!(
        mean_age_myr(&eu, "double-ridge") < 100.0,
        "{}",
        mean_age_myr(&eu, "double-ridge")
    );
    let chaos = eu.cover_fraction("chaos", er, 48);
    eprintln!("Europa chaos covers {:.0}%", 100.0 * chaos);
    assert!(chaos > 0.1 && chaos < 0.4, "{chaos}");
    // Lineae trend about 45° from east: the mean |cos(azimuth)| is near cos 45°.
    let cosines: Vec<f64> = eu
        .landforms
        .iter()
        .filter(|l| l.kind == "double-ridge")
        .map(|l| {
            let e = [-l.dir[1], l.dir[0], 0.0];
            let n = sse_core::math::sqrt(e[0] * e[0] + e[1] * e[1]).max(1e-12);
            let ax = l.axis.unwrap();
            ((ax[0] * e[0] + ax[1] * e[1]) / n).abs()
        })
        .collect();
    let mean = cosines.iter().sum::<f64>() / cosines.len() as f64;
    assert!(mean > 0.6 && mean < 0.8, "mean azimuth cosine {mean}");
    // Ganymede: the same grammar, but cold now, so its bands are old and chaos is rare.
    assert!(
        mean_age_myr(&ga, "double-ridge") > 500.0,
        "{}",
        mean_age_myr(&ga, "double-ridge")
    );
    assert!(count(&ga, "chaos") * 10 < count(&eu, "chaos"));
    // Low relief: an ice shell stands within a few hundred metres.
    assert!(eu.landforms.iter().all(|l| l.height_m.abs() <= 300.0));
}

#[test]
fn io_vents_and_raises_blocks() {
    let (io, _) = surface_of("Io", None);
    let (p, mm) = (count(&io, "patera"), count(&io, "massif"));
    eprintln!("Io: {p} paterae, {mm} mountains");
    // About 417 paterae and 135 mountains are known.
    assert!(p > 300 && p < 550, "{p}");
    assert!(mm > 90 && mm < 190, "{mm}");
    let tallest = io
        .landforms
        .iter()
        .filter(|l| l.kind == "massif")
        .map(|l| l.height_m)
        .fold(0.0, f64::max);
    assert!(tallest > 8000.0 && tallest <= 17_000.0, "{tallest}");
    // Everything is young: the surface renews in about a Myr.
    assert!(mean_age_myr(&io, "patera") < 5.0);
}

#[test]
fn venus_has_coronae_rifts_and_overturns() {
    let (v, _) = surface_of("Venus", None);
    let c = count(&v, "corona");
    eprintln!(
        "Venus: {c} coronae, {} chasmata, {} overturns",
        count(&v, "chasma"),
        v.overturns.len()
    );
    assert!(c > 350 && c < 700, "{c}");
    assert!(count(&v, "chasma") > 5);
    // Median corona about 250 km across.
    let mut d: Vec<f64> = v
        .landforms
        .iter()
        .filter(|l| l.kind == "corona")
        .map(|l| l.size_km)
        .collect();
    d.sort_by(|a, b| a.total_cmp(b));
    let med = d[d.len() / 2];
    assert!(med > 150.0 && med < 400.0, "{med}");
    assert!(!v.overturns.is_empty());
    assert!(v.overturns.windows(2).all(|w| w[0].t_myr <= w[1].t_myr));
    assert!(v
        .overturns
        .iter()
        .all(|o| o.text.contains("Global overturn")));
}

#[test]
fn shrinking_lid_worlds_get_scarps_and_mars_a_rift() {
    let (me, _) = surface_of("Mercury", None);
    let (mo, _) = surface_of("Moon", None);
    assert!(count(&me, "scarp") > 100);
    // Contraction scarps are taller on the bigger world.
    let tall = |r: &RegimeSurface| {
        r.landforms
            .iter()
            .filter(|l| l.kind == "scarp")
            .map(|l| l.height_m)
            .fold(0.0, f64::max)
    };
    assert!(tall(&me) > tall(&mo));
    // Mars: a rift beside its tallest S4 shield, dated and explained.
    let s = settings();
    let b = body("Mars");
    let e = envelope(&s, &b);
    let seed = Seed::from_text("sse4-regimes-tests");
    let w = crate::wear::WaterHistory::steady(0.0, b.pressure_bar, b.age_gyr);
    let hist = crate::impacts::ImpactHistory::lunar();
    let inp = crate::impacts::ImpactInputs {
        history: &hist,
        water: &w,
        pressure_bar: b.pressure_bar,
        plates: None,
    };
    let (f, _) = crate::impacts::impacts(&s, &e, &seed, &inp, b.age_gyr).unwrap();
    let (r, _) = surface_with(&s, &b, None, Some(&f));
    let rift = r
        .landforms
        .iter()
        .find(|l| l.kind == "rift")
        .expect("a rift beside the tallest shield");
    assert!(rift.cause.contains("shield:") && rift.height_m < -3000.0);
    assert!(rift.size_km > 1000.0);
}

#[test]
fn titan_dunes_lie_low_and_lakes_lie_north() {
    let (t, r) = surface_of("Titan", None);
    let dunes = t.cover_fraction("dune-field", r, 48);
    let lakes = t.cover_fraction("lake", r, 96);
    eprintln!(
        "Titan dunes {:.1}%, lakes {:.2}%",
        100.0 * dunes,
        100.0 * lakes
    );
    assert!(dunes > 0.07 && dunes < 0.25, "{dunes}");
    assert!(lakes > 0.0005 && lakes < 0.03, "{lakes}");
    for l in &t.landforms {
        match l.kind.as_str() {
            "dune-field" => assert!(l.lat_deg.abs() <= 30.0 + 1e-9),
            "lake" => assert!(l.lat_deg.abs() >= 55.0 - 1e-9),
            _ => {}
        }
    }
    let north = t
        .landforms
        .iter()
        .filter(|l| l.kind == "lake" && l.lat_deg > 0.0)
        .count();
    assert!(north * 2 > count(&t, "lake"));
}

#[test]
fn giants_band_by_rotation() {
    let (j, _) = surface_of("Jupiter", Some(9.93));
    let c = j.clouds.as_ref().unwrap();
    eprintln!("{}", c.cause);
    assert!(c.jets >= 7 && c.jets <= 14, "{}", c.jets);
    assert_eq!(c.band_edges_deg.len(), 2 * c.jets as usize + 1);
    assert!(c.chemistry.contains("ammonia"));
    // Spin it slower: fewer, wider bands.
    let (slow, _) = surface_of("Jupiter", Some(40.0));
    assert!(slow.clouds.unwrap().jets < c.jets);
    // An ice giant with fast winds has a few broad bands (Neptune: about 3 jets).
    let mut nep = body("Jupiter");
    nep.mass_earth = 17.1;
    nep.radius_earth = 3.88;
    let (n, _) = surface_with(&settings(), &nep, Some(16.1), None);
    let nc = n.clouds.unwrap();
    assert!(nc.jets >= 2 && nc.jets <= 4, "{}", nc.jets);
    // Zones and belts differ in brightness and height.
    let (a0, h0) = c.at([1.0, 0.0, 0.0], 69_911.0);
    let lat = 0.5 * (c.band_edges_deg[c.jets as usize + 1] + c.band_edges_deg[c.jets as usize + 2]);
    let v = [
        sse_core::math::cos(lat.to_radians()),
        0.0,
        sse_core::math::sin(lat.to_radians()),
    ];
    let (a1, h1) = c.at(v, 69_911.0);
    assert!(
        (a0 - a1).abs() > 0.05 && (h0 - h1).abs() > 1000.0,
        "{a0} {a1} {h0} {h1}"
    );
}

#[test]
fn the_sketch_regimes_hold_their_kind() {
    use crate::atlas::{Atlas, TileKey, COVERS};
    let s = {
        let mut s = settings();
        s.atlas.tile_samples = 32;
        s
    };
    let seed = Seed::from_text("sse4-regimes-tests");
    let tiles = |name: &str| {
        let b = body(name);
        let e = envelope(&s, &b);
        let a = Atlas::new(&s, &e, &seed, b.age_gyr, vec![]);
        (0..6)
            .map(|f| a.build(TileKey::new(f, 0, 0, 0).unwrap()))
            .collect::<Vec<_>>()
    };
    // Ocean world: no land anywhere.
    for t in tiles("Deep ocean world") {
        assert!(t.cover.iter().all(|c| COVERS[*c as usize] == "water"));
        assert!(t.elevation_dm.iter().all(|h| *h < 0));
    }
    // Lava world: the day side molten, the night side crust.
    let lava = tiles("55 Cancri e");
    assert!(lava[0].cover.iter().all(|c| COVERS[*c as usize] == "magma"));
    assert!(lava[1].cover.iter().all(|c| COVERS[*c as usize] != "magma"));
    // Small body: not round. (Real Vesta spans about 0.24 of its radius; the S0
    // sketch's lumps give more, a known limit in S6.md.)
    let v = tiles("Vesta");
    let (lo, hi) = v
        .iter()
        .flat_map(|t| t.elevation_dm.iter())
        .fold((i32::MAX, i32::MIN), |a, h| (a.0.min(*h), a.1.max(*h)));
    let r = body("Vesta").radius_earth * 6371.0e3;
    let spread = (hi - lo) as f64 / 10.0 / r;
    assert!(spread > 0.05 && spread < 1.0, "{spread}");
    for n in ["Vesta", "Deep ocean world", "55 Cancri e"] {
        assert!(!surface_of(n, None).0.notes.is_empty());
    }
}

#[test]
fn every_landform_answers_why() {
    for n in ["Europa", "Io", "Venus", "Mercury", "Titan", "Carbon world"] {
        let (r, rk) = surface_of(n, None);
        for l in r.landforms.iter().take(40) {
            assert!(
                l.cause.contains("Gyr ago") && l.cause.contains("Heat flow"),
                "{}",
                l.cause
            );
            let why = r.explain(l.lat_deg, l.lon_deg, rk);
            assert!(
                why.iter().any(|x| x.starts_with(&format!("{}:", l.id))),
                "{n}: {}",
                l.id
            );
        }
    }
    assert!(surface_of("Carbon world", None).0.speculative);
}

fn regime_atlas(name: &str, rot: Option<f64>) -> crate::atlas::Atlas {
    let mut s = settings();
    s.atlas.tile_samples = 32;
    let b = body(name);
    let e = envelope(&s, &b);
    let seed = Seed::from_text("sse4-regimes-tests");
    let inp = RegimeInputs {
        rotation_hours: rot,
        mass_earth: b.mass_earth,
        impacts: None,
    };
    let rs = regime_surface(&s, &e, &seed, &inp, b.age_gyr);
    crate::atlas::Atlas::new(&s, &e, &seed, b.age_gyr, vec![]).with_regime(rs)
}

#[test]
fn landforms_refine_within_tolerance() {
    use crate::atlas::{downsample_error_m, TileKey};
    for (name, rot) in [
        ("Europa", None),
        ("Io", None),
        ("Venus", None),
        ("Titan", None),
        ("Jupiter", Some(9.93)),
    ] {
        let a = regime_atlas(name, rot);
        let rs = a.regime_surface().unwrap();
        // Chain down over a landform (or the equator for a giant).
        let dir = rs
            .landforms
            .iter()
            .max_by(|x, y| x.width_km.total_cmp(&y.width_km))
            .map(|l| l.dir)
            .unwrap_or([1.0, 0.0, 0.0]);
        let (f, u, v) = crate::atlas::face_of(dir);
        let floor = a.level_for(a.settings().fact_floor_m).min(11);
        let mut worst: f64 = 0.0;
        let mut tagged = false;
        for level in 0..floor {
            let n = (1u64 << level) as f64;
            let key = TileKey::new(
                f,
                level,
                (((u + 1.0) / 2.0) * n) as u32,
                (((v + 1.0) / 2.0) * n) as u32,
            )
            .unwrap();
            let p = a.build(key);
            tagged |= !p.tags.is_empty();
            for ch in key.children() {
                let t = a.build(ch);
                let err = downsample_error_m(&p, &t);
                let tol = a.tolerance_m(ch.level);
                worst = worst.max(err / tol);
                assert!(err <= tol, "{name} {key:?} -> {ch:?}: {err} m over {tol} m");
            }
        }
        assert!(tagged, "{name}: no tags");
        eprintln!("{name}: worst {:.0}% of tolerance", 100.0 * worst);
    }
}

#[test]
fn titan_tiles_show_lakes_and_dunes() {
    use crate::atlas::{TileKey, COVERS, MATERIALS};
    let a = regime_atlas("Titan", None);
    let rs = a.regime_surface().unwrap();
    let lake = rs
        .landforms
        .iter()
        .filter(|l| l.kind == "lake")
        .max_by(|x, y| x.size_km.total_cmp(&y.size_km))
        .unwrap();
    let (f, u, v) = crate::atlas::face_of(lake.dir);
    let level = 4u8;
    let n = (1u64 << level) as f64;
    let t = a.build(
        TileKey::new(
            f,
            level,
            (((u + 1.0) / 2.0) * n) as u32,
            (((v + 1.0) / 2.0) * n) as u32,
        )
        .unwrap(),
    );
    assert!(t.cover.iter().any(|c| COVERS[*c as usize] == "hydrocarbon"));
    let dune = rs
        .landforms
        .iter()
        .find(|l| l.kind == "dune-field")
        .unwrap();
    let (f, u, v) = crate::atlas::face_of(dune.dir);
    let t = a.build(
        TileKey::new(
            f,
            level,
            (((u + 1.0) / 2.0) * n) as u32,
            (((v + 1.0) / 2.0) * n) as u32,
        )
        .unwrap(),
    );
    assert!(t
        .material
        .iter()
        .any(|m| MATERIALS[*m as usize] == "organic-sediment"));
}

#[test]
fn golden_regimes() {
    let got = crate::scenario::regimes_golden();
    let path = format!(
        "{}/../../tests/golden/surface-regimes.json",
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
        "golden surface-regimes changed"
    );
}
