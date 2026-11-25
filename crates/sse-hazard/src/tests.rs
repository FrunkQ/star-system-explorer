use super::*;

const SOL: &str = include_str!("../../../packs/base-sol/pack.json");

fn sheet() -> Sheet {
    Sheet::base()
}

/// A body of the Solar System from base-sol, with its belts from the
/// `read_field` fixture.
fn sol(id: &str) -> (Body, Option<Belts>) {
    let s = sheet();
    let v: serde_json::Value = serde_json::from_str(SOL).unwrap();
    let b = v["provides"]["other"]["sol"]["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == id)
        .unwrap();
    let val = |k: &str| b["values"][k]["v"].as_f64();
    let set = sse_field::Settings {
        electron: Some(sse_field::Electron {
            mass_kg: s.constant("m_e").unwrap(),
            charge_c: s.constant("e").unwrap(),
            eps0: s.constant("eps0").unwrap(),
        }),
        ..sse_field::Settings::default()
    };
    let fields = sse_field::solar_system(&set, s.constant("m_p").unwrap()).unwrap();
    let f = fields
        .iter()
        .find(|f| f.read.address == format!("sol/{id}"));
    (
        Body {
            address: format!("sol/{id}"),
            mass_kg: val("body.mass").unwrap(),
            radius_m: val("body.radius").unwrap(),
            luminosity_w: val("star.luminosity").unwrap_or(0.0),
            air_top_m: None,
            bubble_m: f.map(|f| f.read.obstacle.standoff_m),
        },
        f.and_then(|f| f.read.belts.clone()),
    )
}

fn read(id: &str, ship: Option<Ship>, line: Line) -> HazardMap {
    let (body, belts) = sol(id);
    hazard_map(
        &Ask {
            body,
            belts,
            ship,
            line,
        },
        &sheet(),
    )
    .unwrap()
}

fn thin(shield_kg_m2: f64) -> Ship {
    Ship {
        shield_kg_m2,
        ..stock_ship()
    }
}

fn tone_at(map: &HazardMap, hazard: &str, r: f64) -> Tone {
    map.rows
        .iter()
        .find(|row| row.hazard == hazard)
        .and_then(|row| row.bands.iter().find(|b| r >= b.from_m && r < b.to_m))
        .map_or(Tone::Clear, |b| b.tone)
}

const R_SUN: f64 = 6.957e8;
const R_J: f64 = 6.9911e7;
const AU: f64 = 1.495_978_707e11;

/// The Sun (design section 11): a probe flown as Parker Solar Probe is, its
/// sunshade face-on and rated to about 1,644 K (2,500 °F, NASA's figure for
/// the Thermal Protection System), dives to close to its real closest pass,
/// 9.86 Sun radii from the centre, before the heat stops it; and a ship
/// with 30 km/s cannot park closer than about a sixth of Earth's distance and
/// still leave.
#[test]
fn the_sun_heat_and_the_climb() {
    let probe = Ship {
        name: "sunshade probe".into(),
        delta_v_m_s: 30_000.0,
        shield_kg_m2: 100.0,
        skin: None,
        max_k: Some(1644.0),
        absorbed_over_emitted: 1.0,
        faces: Faces::Shield,
        length_m: 3.0,
    };
    let m = read("sun", Some(probe), Line::Equator);
    let dive = m.dive.as_ref().unwrap();
    assert_eq!(dive.set_by, "heat");
    let q = dive.distance_m / R_SUN;
    assert!((8.0..=11.0).contains(&q), "{q}");
    assert_eq!(m.hull_limit_from, "typed");
    let park = m.park.as_ref().unwrap();
    assert_eq!(park.set_by, "climbing_out");
    let p = park.distance_m / AU;
    assert!((0.15..=0.19).contains(&p), "{p}");
    // A star has no belts and no air to fall into.
    assert!(m.quiet.iter().any(|q| q.starts_with("Radiation")));
    assert!(m.quiet.contains(&"Air".to_string()));
    assert!(m.why.iter().any(|w| w.contains("au")));
}

/// The stock ship stands in when none is chosen, and its hull's limit is its
/// skin's melting point from the data sheet (aluminium, 933 K), not a typed one.
#[test]
fn stock_ship_and_the_sheet() {
    let m = read("sun", None, Line::Equator);
    assert!(m.stock_ship);
    assert_eq!(m.hull_limit_from, "aluminium.melting_point_k");
    assert!((m.hull_limit_k - 933.45).abs() < 1e-9);
    assert!(m.why.iter().any(|w| w.contains("stock ship")));
    // A skin the sheet does not know, with nothing typed, is said plainly.
    let (body, belts) = sol("sun");
    let ship = Ship {
        skin: Some("unobtainium".into()),
        ..stock_ship()
    };
    let e = hazard_map(
        &Ask {
            body,
            belts,
            ship: Some(ship),
            line: Line::Equator,
        },
        &sheet(),
    )
    .unwrap_err();
    assert!(e.contains("no melting point"), "{e}");
}

/// Jupiter (design section 11). In the equator's plane the belts are deadly
/// beside Io whatever the shield, and harsh beside Europa and Ganymede behind
/// a heavy one; Callisto is outside them. Each moon sweeps a lane a ship can
/// loiter in. Over the poles the belts are clear: no field line there closes.
#[test]
fn jupiter_belts_lanes_and_poles() {
    let thin_hull = read("jupiter", Some(thin(10.0)), Line::Equator);
    let heavy = read("jupiter", Some(thin(400.0)), Line::Equator);
    let tone = |m: &HazardMap, moon: &str| {
        m.moons
            .iter()
            .find(|x| x.moon == moon)
            .map(|x| x.tone)
            .unwrap()
    };
    assert_eq!(tone(&thin_hull, "sol/io"), Tone::Deadly);
    assert_eq!(tone(&thin_hull, "sol/europa"), Tone::Deadly);
    assert_eq!(tone(&heavy, "sol/io"), Tone::Deadly);
    assert_eq!(tone(&heavy, "sol/europa"), Tone::Harsh);
    assert_eq!(tone(&heavy, "sol/ganymede"), Tone::Harsh);
    assert_eq!(tone(&thin_hull, "sol/callisto"), Tone::Clear);
    // The dose falls outward and with the shield.
    let sv =
        |m: &HazardMap, moon: &str| m.moons.iter().find(|x| x.moon == moon).unwrap().sv_per_day;
    assert!(sv(&thin_hull, "sol/io") > sv(&thin_hull, "sol/europa"));
    assert!(sv(&thin_hull, "sol/europa") > sv(&thin_hull, "sol/ganymede"));
    assert!(sv(&thin_hull, "sol/europa") > 10.0 * sv(&heavy, "sol/europa"));
    // Lanes: a safe band at each swept lane, with the moon's name on the map.
    assert!(
        thin_hull.safe_bands.len() >= 4,
        "{:?}",
        thin_hull.safe_bands
    );
    for moon in ["Io", "Europa", "Ganymede"] {
        assert!(thin_hull
            .marks
            .iter()
            .any(|mk| mk.what.starts_with(&format!("lane swept by {moon}"))));
    }
    let park = thin_hull.park.as_ref().unwrap();
    assert_eq!(park.set_by, "radiation_belts");
    assert!(
        (5.9..=6.1).contains(&(park.distance_m / R_J)),
        "{}",
        park.distance_m / R_J
    );
    // In the equator's plane a thin hull's dive must stay outside the belts.
    let dive = thin_hull.dive.as_ref().unwrap();
    assert_eq!(dive.set_by, "radiation_belts");
    assert!(
        dive.distance_m
            >= 0.99
                * thin_hull
                    .marks
                    .iter()
                    .find(|mk| mk.what.starts_with("belts' outer"))
                    .unwrap()
                    .distance_m
    );

    // Over the poles: the belts' row folds away (no closed line over a pole),
    // and the climb out sets the park.
    let poles = read("jupiter", Some(thin(10.0)), Line::Poles);
    assert!(
        poles.quiet.iter().any(|q| q.starts_with("Radiation")),
        "{:?}",
        poles.rows
    );
    assert_eq!(poles.park.as_ref().unwrap().set_by, "climbing_out");
    // A fall over the pole still crosses the belts on the way in and out,
    // but far more quickly, so it goes far deeper than one in the plane.
    let pd = poles.dive.as_ref().unwrap();
    assert!(
        pd.distance_m < 0.7 * dive.distance_m,
        "{} {}",
        pd.distance_m / R_J,
        dive.distance_m / R_J
    );
    let pass = poles.pass.as_ref().unwrap();
    assert!(pass.dose_sv < 1.0 && pass.hours_in_belts > 0.0, "{pass:?}");
    // The stock ship (10 g/cm²) over the poles goes deeper still.
    let stock = read("jupiter", None, Line::Poles);
    assert!(stock.dive.as_ref().unwrap().distance_m < pd.distance_m);
}

/// Saturn: the belts start at the main rings' edge, said on the map.
#[test]
fn saturn_belts_start_at_the_rings() {
    let m = read("saturn", None, Line::Equator);
    let inner = m
        .marks
        .iter()
        .find(|mk| mk.what.starts_with("belts' inner edge"))
        .unwrap();
    assert!(inner.what.contains("rings"), "{}", inner.what);
    // 2.27 of the field's reference radius (60,268 km), the A ring's edge.
    let x = inner.distance_m / 6.0268e7;
    assert!((x - 2.27).abs() < 0.01, "{x}");
}

/// Earth: the air sets the belts' inner edge and is deadly to stay in; a
/// heavier shield parks closer; the bubble's edge is marked.
#[test]
fn earth_air_belts_and_shield() {
    let thin_hull = read("earth", Some(thin(10.0)), Line::Equator);
    let heavy = read("earth", Some(thin(400.0)), Line::Equator);
    assert!(thin_hull.rows.iter().any(|r| r.hazard == "air"));
    assert!(thin_hull.marks.iter().any(|mk| mk.what == "top of the air"));
    assert_eq!(tone_at(&thin_hull, "air", 1.05 * 6.371e6), Tone::Deadly);
    assert_eq!(
        tone_at(&thin_hull, "radiation_belts", 4.0 * 6.371e6),
        Tone::Deadly
    );
    assert!(thin_hull.marks.iter().any(|mk| mk.what.contains("bubble")));
    let tp = thin_hull.park.as_ref().unwrap();
    let hp = heavy.park.as_ref().unwrap();
    assert!(hp.distance_m < tp.distance_m, "{hp:?} {tp:?}");
    // Climbing out of a low Earth orbit needs about 3.2 km/s: clear for 10.
    assert!(thin_hull.quiet.contains(&"Climbing out".to_string()));
}

/// A world with no field has no belts: Venus's map has no radiation row and
/// nothing stops a stay above its air but the climb.
#[test]
fn no_field_no_belts() {
    let (body, belts) = sol("venus");
    assert!(belts.is_none());
    let m = hazard_map(
        &Ask {
            body: Body {
                air_top_m: Some(6.0518e6 + 2.5e5),
                ..body
            },
            belts,
            ship: None,
            line: Line::Equator,
        },
        &sheet(),
    )
    .unwrap();
    assert!(m.quiet.iter().any(|q| q.starts_with("Radiation")));
    assert_eq!(m.park.as_ref().unwrap().set_by, "air");
    assert!(m.moons.is_empty());
}

/// Same ask, same answer, to the bit.
#[test]
fn the_same_every_time() {
    let a = serde_json::to_string(&read("jupiter", None, Line::Poles)).unwrap();
    let b = serde_json::to_string(&read("jupiter", None, Line::Poles)).unwrap();
    assert_eq!(a, b);
}
