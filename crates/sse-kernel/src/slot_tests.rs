//! GZ13: a star slot for every system. Standing rule (grand zoom 6.5): no
//! system without a star slot, no star slot without a cell, no cell
//! without a galaxy, no galaxy without a halo in the web.

use crate::Kernel;
use serde_json::{json, Value};

fn send(k: &mut Kernel, v: Value) -> Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn open(seed: &str) -> Kernel {
    let mut k = Kernel::new();
    let r = send(
        &mut k,
        json!({"op": "open_universe", "id": seed, "seed": seed}),
    );
    assert!(r["type"] != "error", "{r}");
    k
}

const SOL: &str = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");

/// A starmap of copies of the synthetic Sol, one per entry, each at its map
/// position (or none), at 10 px per light year.
fn map(positions: &[Option<[f64; 3]>]) -> String {
    let mut v: Value = serde_json::from_str(SOL).unwrap();
    v["scale"] = json!({"unit": "ly", "pixelsPerUnit": 10.0, "showScaleBar": true});
    let one = v["systems"][0].clone();
    let list: Vec<Value> = positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut s = one.clone();
            s["id"] = json!(format!("sn-{i}"));
            match p {
                Some(p) => s["position"] = json!({"x": p[0], "y": p[1], "z": p[2]}),
                None => {
                    s.as_object_mut().unwrap().remove("position");
                }
            }
            s
        })
        .collect();
    v["systems"] = json!(list);
    v.to_string()
}

fn sol(position: Option<[f64; 3]>) -> String {
    map(&[position])
}

fn adopt(k: &mut Kernel, save: &str) -> Value {
    let r = send(k, json!({"op": "adopt", "import_v3": save}));
    assert_eq!(r["type"], "adopted", "{r}");
    r
}

#[test]
fn an_imported_system_is_given_a_drawn_star_of_home_at_adoption() {
    let mut k = open("gz13");
    let r = adopt(&mut k, &sol(Some([0.0, 0.0, 0.0])));
    let sys = r["systems"][0].as_str().unwrap().to_string();
    let p = &r["places"][0];
    assert_eq!(p["system"], sys, "{r}");
    assert_eq!(p["by"], "map", "{p}");
    let star = p["star"].as_str().unwrap().to_string();
    let home = send(&mut k, json!({"op": "home_galaxy"}))["home"]["halo"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(star.starts_with(&format!("{home}/cell:")), "{star}");
    assert!(star.contains("/sys:f"), "{star}");
    // At the Sun's place (map origin): within the 10 pc cell or its neighbours.
    assert!(p["distance_pc"].as_f64().unwrap() < 25.0, "{p}");

    let s = send(&mut k, json!({"op": "star_slot", "system": sys}))["slot"].clone();
    assert_eq!(s["star"], star.as_str(), "{s}");
    assert_eq!(s["in_home"], true);
    // The authored Sun stands in the slot: one solar mass, 4.57 Gyr.
    let cell = send(&mut k, json!({"op": "star_cell", "cell": s["cell"]}));
    let rows = cell["cell"]["systems"].as_array().unwrap();
    let mine: Vec<&Value> = rows
        .iter()
        .filter(|x| x["address"] == star.as_str())
        .collect();
    assert_eq!(mine.len(), 1, "counted once");
    assert_eq!(mine[0]["pinned"], true);
    assert!(
        (mine[0]["mass_msun"].as_f64().unwrap() - 1.0).abs() < 0.01,
        "{}",
        mine[0]
    );
    assert!(s["authored_over"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x == "mass_msun"));
    assert!(s["drawn"]["mass_msun"].is_f64());
    let r_kpc = s["r_kpc"].as_f64().unwrap();
    assert!(r_kpc > 1.0 && r_kpc < 30.0, "{r_kpc}");
    // The slot star is the system's place, not a second system of the
    // campaign: the systems list names the adopted system only.
    let listed = send(&mut k, json!({"op": "systems"}));
    let names: Vec<&str> = listed["systems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["system"].as_str().unwrap())
        .collect();
    assert!(names.contains(&sys.as_str()), "{listed}");
    assert!(!names.contains(&star.as_str()), "{listed}");
}

/// A map position is laid out from the Sun's place: 300 ly on the map
/// lands about 92 pc away, and two systems keep their spacing.
#[test]
fn a_map_position_keeps_the_maps_layout() {
    let mut k = open("gz13");
    let a = adopt(
        &mut k,
        &map(&[Some([0.0, 0.0, 0.0]), Some([3000.0, 0.0, 0.0])]),
    );
    let sa = a["systems"][0].as_str().unwrap();
    let sb = a["systems"][1].as_str().unwrap();
    assert_ne!(a["places"][0]["star"], a["places"][1]["star"]);
    let xa = send(&mut k, json!({"op": "star_slot", "system": sa}))["slot"]["xyz_pc"].clone();
    let xb = send(&mut k, json!({"op": "star_slot", "system": sb}))["slot"]["xyz_pc"].clone();
    let d: f64 = (0..3)
        .map(|i| {
            let x = xa[i].as_f64().unwrap() - xb[i].as_f64().unwrap();
            x * x
        })
        .sum::<f64>();
    let d = sse_core::math::sqrt(d);
    // 3000 px at 10 px per ly = 300 ly = 92 pc, to within a cell or two.
    assert!((d - 92.0).abs() < 30.0, "{d}");
}

/// With no map position, the nearest drawn star that fits the system's own
/// star is taken: mass within 10 %, age within 0.5 Gyr, single.
#[test]
fn with_no_map_position_the_nearest_fitting_star_is_taken() {
    let mut k = open("gz13");
    let r = adopt(&mut k, &map(&[None, None]));
    let p = &r["places"][0];
    assert_eq!(p["by"], "fit", "{p}");
    assert_eq!(p["fits"], true, "{p}");
    let sys = r["systems"][0].as_str().unwrap();
    let s = send(&mut k, json!({"op": "star_slot", "system": sys}))["slot"].clone();
    let drawn = &s["drawn"];
    let m = drawn["mass_msun"].as_f64().unwrap();
    assert!((0.9..=1.1).contains(&m), "{drawn}");
    assert_eq!(drawn["companions"].as_array().unwrap().len(), 0, "{drawn}");
    // The same star again in the same map never takes the same slot.
    assert_eq!(r["places"][1]["fits"], true, "{r}");
    assert_ne!(r["places"][1]["star"], p["star"]);
}

/// The GM moves a slot; the old star goes back to the fill, a pinned star
/// cannot be taken, and the origin reads the new place.
#[test]
fn the_gm_can_move_a_slot() {
    let mut k = open("gz13");
    let r = adopt(
        &mut k,
        &map(&[Some([0.0, 0.0, 0.0]), Some([0.0, 0.0, 0.0])]),
    );
    let sys = r["systems"][0].as_str().unwrap().to_string();
    let old = r["places"][0]["star"].as_str().unwrap().to_string();
    let slot = send(&mut k, json!({"op": "star_slot", "system": sys}))["slot"].clone();
    let cell = send(&mut k, json!({"op": "star_cell", "cell": slot["cell"]}));
    let other = cell["cell"]["systems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["pinned"] == false)
        .unwrap()["address"]
        .as_str()
        .unwrap()
        .to_string();
    let m = send(
        &mut k,
        json!({"op": "set_star_slot", "system": sys, "star": other}),
    );
    assert_eq!(m["type"], "star_slot_set", "{m}");
    assert_eq!(m["slot"]["star"], other.as_str());
    assert_eq!(m["slot"]["by"], "gm");
    let cell = send(&mut k, json!({"op": "star_cell", "cell": slot["cell"]}));
    let rows = cell["cell"]["systems"].as_array().unwrap();
    let back = rows.iter().find(|x| x["address"] == old.as_str());
    assert!(back.is_none_or(|x| x["pinned"] == false), "{back:?}");
    // A second system cannot take a star already pinned.
    let sys2 = r["systems"][1].as_str().unwrap();
    let no = send(
        &mut k,
        json!({"op": "set_star_slot", "system": sys2, "star": other}),
    );
    assert_eq!(no["type"], "error", "{no}");
    assert_eq!(no["code"], "taken", "{no}");
}

/// The site's own event history: supernovae, passes and arm crossings on
/// the slot star's path, the same every time.
#[test]
fn a_slot_has_a_site_history() {
    let mut k = open("gz13");
    let r = adopt(&mut k, &sol(Some([0.0, 0.0, 0.0])));
    let sys = r["systems"][0].as_str().unwrap().to_string();
    let a = send(&mut k, json!({"op": "site_history", "system": sys}));
    assert_eq!(a["type"], "site_history", "{a}");
    let h = &a["history"];
    let span = h["to_myr"].as_f64().unwrap() - h["from_myr"].as_f64().unwrap();
    assert!(
        (span - 4570.0).abs() < 50.0,
        "the star's whole life: {span}"
    );
    println!("GZ13 site history counts {}", h["counts"]);
    assert!(h["hits"].as_array().unwrap().len() > 10, "{}", h["counts"]);
    assert!(
        h["counts"]["core_collapse"].as_u64().unwrap_or(0) > 0,
        "{}",
        h["counts"]
    );
    let b = send(&mut k, json!({"op": "site_history", "system": sys}));
    assert_eq!(a, b);
}

/// Standing rule: every adopted system has a slot in a cell of a galaxy
/// that is a halo in the web, and the origin is read from the slot, not
/// the Sun's radius.
#[test]
fn no_adopted_system_is_without_a_place() {
    let mut k = open("gz13");
    let r = adopt(
        &mut k,
        &map(&[Some([0.0, 0.0, 0.0]), None, Some([-500.0, 800.0, 40.0])]),
    );
    for sys in r["systems"].as_array().unwrap() {
        let sys = sys.as_str().unwrap();
        let s = send(&mut k, json!({"op": "star_slot", "system": sys}))["slot"].clone();
        let star = s["star"].as_str().unwrap_or_else(|| panic!("{s}"));
        assert!(star.contains("/halo:") && star.contains("/cell:"), "{star}");
        assert!(!star.contains("halo:milky-way-like"));
        // Making it older reconstructs its origin from the slot's radius.
        let aged = send(
            &mut k,
            json!({"op": "set_system_age", "system": sys, "age_myr": 6000.0}),
        );
        assert!(aged["type"] != "error", "{aged}");
    }
}
