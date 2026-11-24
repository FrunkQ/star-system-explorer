//! ST1 checks: real systems read as they should, and each check's verdict
//! turns where its paper says.

use crate::protocol::*;
use serde_json::{json, Value as J};

const AU: f64 = sse_core::sheet::AU;
const M_SUN: f64 = 1.98840987e30;
const M_EARTH: f64 = 5.9722e24;

fn body(addr: &str, kind: &str, host: Option<&str>, mass: f64, a_au: f64, e: f64, i_deg: f64) -> J {
    let mut f = json!({"body.kind": kind, "body.mass": mass});
    if let Some(h) = host {
        f["orbit.host"] = json!(h);
        f["orbit.semi_major_axis"] = json!(a_au * AU);
        f["orbit.eccentricity"] = json!(e);
        f["orbit.inclination"] = json!(i_deg * sse_core::math::PI / 180.0);
    }
    json!({"address": addr, "facts": f})
}

fn ask(k: &mut crate::Kernel, bodies: Vec<J>, add: Option<Vec<J>>) -> J {
    let mut q = json!({"op": "orbit_stability", "system": "u:t/sys:0", "bodies": bodies});
    if let Some(a) = add {
        q["add"] = json!(a);
    }
    let r = k.handle_json(&q.to_string());
    let v: J = serde_json::from_str(&r).unwrap();
    assert_eq!(v["type"], "orbit_stability_is", "{v}");
    v
}

fn find<'a>(v: &'a J, check: &str, first: &str) -> &'a J {
    v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check"] == check && c["bodies"][0].as_str().unwrap().ends_with(first))
        .unwrap_or_else(|| panic!("no {check} for {first}: {v}"))
}

fn sol(only_inner: bool) -> Vec<J> {
    let s = "u:t/sys:0/star:0";
    let mut v = vec![body(s, "star", None, M_SUN, 0.0, 0.0, 0.0)];
    let p = [
        ("mercury", 0.387, 0.2056, 7.0, 0.3301e24),
        ("venus", 0.723, 0.0068, 3.39, 4.867e24),
        ("earth", 1.0, 0.0167, 0.0, 5.972e24),
        ("mars", 1.524, 0.0934, 1.85, 0.6417e24),
        ("jupiter", 5.203, 0.0484, 1.3, 1898e24),
        ("saturn", 9.537, 0.0539, 2.49, 568e24),
        ("uranus", 19.19, 0.0473, 0.77, 86.8e24),
        ("neptune", 30.07, 0.0086, 1.77, 102e24),
    ];
    for (k, (n, a, e, i, m)) in p.iter().enumerate() {
        if only_inner && k >= 4 {
            break;
        }
        v.push(body(
            &format!("{s}/planet:{k}"),
            "planet",
            Some(s),
            *m,
            *a,
            *e,
            *i,
        ));
        let _ = n;
    }
    v
}

#[test]
fn the_solar_system_lasts_but_has_the_stretch_to_collide() {
    let mut k = crate::Kernel::new();
    let v = ask(&mut k, sol(false), None);
    // Every neighbour pair is far enough apart.
    for c in v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["check"] == "hill")
    {
        assert!(c["level"].as_u64().unwrap() <= 1, "{c}");
        assert!(c["results"]["spacing"].as_f64().unwrap() > 2.0 * sse_core::math::sqrt(3.0));
    }
    // Laskar and Petit 2017 section 5.4.1: the giants' deficit makes the
    // inner system AMD-unstable; the outer pairs are AMD-stable.
    assert_eq!(find(&v, "amd", "planet:0")["level"], 1);
    assert_eq!(find(&v, "amd", "planet:2")["level"], 1);
    assert_eq!(find(&v, "amd", "planet:4")["level"], 0);
    assert_eq!(find(&v, "amd", "planet:6")["level"], 0);
    assert_eq!(v["verdict"], "watch");
    // Split off, the inner four are AMD-stable ("hierarchical AMD-stable").
    let inner = ask(&mut k, sol(true), None);
    for c in inner["checks"].as_array().unwrap() {
        assert_eq!(c["level"], 0, "{c}");
    }
    assert_eq!(inner["verdict"], "lasts");
    // Each check shows its working.
    let c = find(&v, "amd", "planet:1");
    assert_eq!(c["block"], "sse.stability.amd_critical");
    assert!(c["inputs"]["system_amd"].as_f64().unwrap() > 0.0);
    assert!(c["results"]["critical_amd"].as_f64().unwrap() > 0.0);
}

#[test]
fn neighbours_too_close_or_crossing_do_not_last() {
    let mut k = crate::Kernel::new();
    let s = "u:t/sys:0/star:0";
    let star = body(s, "star", None, M_SUN, 0.0, 0.0, 0.0);
    // Two Earths 0.02 au apart: about 1.6 mutual Hill radii.
    let v = ask(
        &mut k,
        vec![
            star.clone(),
            body(
                &format!("{s}/planet:0"),
                "planet",
                Some(s),
                M_EARTH,
                1.0,
                0.0,
                0.0,
            ),
            body(
                &format!("{s}/planet:1"),
                "planet",
                Some(s),
                M_EARTH,
                1.02,
                0.0,
                0.0,
            ),
        ],
        None,
    );
    assert_eq!(find(&v, "hill", "planet:0")["level"], 2);
    assert_eq!(v["verdict"], "unstable");
    // Stretched enough that the orbits cross.
    let v = ask(
        &mut k,
        vec![
            star.clone(),
            body(
                &format!("{s}/planet:0"),
                "planet",
                Some(s),
                M_EARTH,
                1.0,
                0.3,
                0.0,
            ),
            body(
                &format!("{s}/planet:1"),
                "planet",
                Some(s),
                M_EARTH,
                1.2,
                0.0,
                0.0,
            ),
        ],
        None,
    );
    assert_eq!(find(&v, "hill", "planet:0")["level"], 3);
    // Twelve Hill radii apart and round: lasts.
    let v = ask(
        &mut k,
        vec![
            star,
            body(
                &format!("{s}/planet:0"),
                "planet",
                Some(s),
                M_EARTH,
                1.0,
                0.0,
                0.0,
            ),
            body(
                &format!("{s}/planet:1"),
                "planet",
                Some(s),
                M_EARTH,
                1.16,
                0.0,
                0.0,
            ),
        ],
        None,
    );
    assert_eq!(find(&v, "hill", "planet:0")["level"], 0);
}

#[test]
fn moons_stay_well_inside_the_gravity_bubble() {
    let mut k = crate::Kernel::new();
    let s = "u:t/sys:0/star:0";
    let e = format!("{s}/planet:0");
    let base = vec![
        body(s, "star", None, M_SUN, 0.0, 0.0, 0.0),
        body(&e, "planet", Some(s), M_EARTH, 1.0, 0.0167, 0.0),
    ];
    // Earth's Hill radius is about 0.01 au; the Moon at 0.00257 au stays.
    let mut b = base.clone();
    b.push(body(
        &format!("{e}/moon:0"),
        "moon",
        Some(&e),
        7.342e22,
        0.00257,
        0.0549,
        5.1,
    ));
    let v = ask(&mut k, b, None);
    assert_eq!(find(&v, "moon", "moon:0")["level"], 0);
    // At 0.6 of the bubble a forward moon is lost; a backward one stays.
    let mut b = base.clone();
    b.push(body(
        &format!("{e}/moon:0"),
        "moon",
        Some(&e),
        1e20,
        0.006,
        0.0,
        0.0,
    ));
    assert_eq!(find(&ask(&mut k, b, None), "moon", "moon:0")["level"], 2);
    let mut b = base;
    b.push(body(
        &format!("{e}/moon:0"),
        "moon",
        Some(&e),
        1e20,
        0.006,
        0.0,
        170.0,
    ));
    assert_eq!(find(&ask(&mut k, b, None), "moon", "moon:0")["level"], 0);
}

#[test]
fn binaries_and_tilted_companions() {
    let mut k = crate::Kernel::new();
    let a = "u:t/sys:0/star:0";
    let b = format!("{a}/star:1");
    // Alpha Centauri A and B.
    let stars = |tilt: f64, ab: f64, eb: f64| {
        vec![
            body(a, "star", None, 1.079 * M_SUN, 0.0, 0.0, 0.0),
            body(&b, "star", Some(a), 0.909 * M_SUN, ab, eb, tilt),
        ]
    };
    let mut s = stars(0.0, 23.4, 0.52);
    s.push(body(
        &format!("{a}/planet:0"),
        "planet",
        Some(a),
        M_EARTH,
        1.0,
        0.0,
        0.0,
    ));
    s.push(body(
        &format!("{a}/planet:1"),
        "planet",
        Some(a),
        M_EARTH,
        5.0,
        0.0,
        0.0,
    ));
    s.push(body(
        &format!("{b}/planet:0"),
        "planet",
        Some(&b),
        M_EARTH,
        1.0,
        0.0,
        0.0,
    ));
    let v = ask(&mut k, s, None);
    assert_eq!(
        find(&v, "binary_one_star", "sys:0/star:0/planet:0")["level"],
        0
    );
    assert_eq!(
        find(&v, "binary_one_star", "sys:0/star:0/planet:1")["level"],
        2
    );
    let round_b = find(&v, "binary_one_star", "star:1/planet:0");
    assert_eq!(round_b["level"], 0);
    assert_eq!(round_b["around"], b);
    // Holman and Wiegert: about 2.8 au round A.
    let lim = find(&v, "binary_one_star", "sys:0/star:0/planet:0")["results"]["limit"]
        .as_f64()
        .unwrap()
        / AU;
    assert!((2.5..3.0).contains(&lim), "{lim}");
    // A planet round both stars of a close pair (Kepler-16 b) lasts.
    let mut s = stars(0.0, 0.22431, 0.15944);
    s.push(body(
        &format!("{a}/planet:0"),
        "planet",
        Some(a),
        0.333 * 1.898e27,
        0.7048,
        0.0069,
        0.0,
    ));
    assert_eq!(
        find(&ask(&mut k, s, None), "binary_both_stars", "planet:0")["level"],
        0
    );
    // A companion 300 au out, tilted 60 degrees: the planet's orbit swings
    // up to e = 0.76 (Kozai 1962), a note, not a verdict of loss.
    let mut s = stars(60.0, 300.0, 0.2);
    s.push(body(
        &format!("{a}/planet:0"),
        "planet",
        Some(a),
        M_EARTH,
        1.0,
        0.0,
        0.0,
    ));
    let v = ask(&mut k, s, None);
    let c = find(&v, "kozai", "planet:0");
    assert_eq!(c["level"], 1);
    assert!(
        (c["results"]["largest_eccentricity"].as_f64().unwrap() - 0.7638).abs() < 1e-3,
        "{c}"
    );
    assert_eq!(v["verdict"], "watch");
}

#[test]
fn adding_a_body_reads_only_what_it_touches() {
    let mut k = crate::Kernel::new();
    let s = "u:t/sys:0/star:0";
    let new = body(
        &format!("{s}/planet:9"),
        "planet",
        Some(s),
        M_EARTH,
        1.03,
        0.0,
        0.0,
    );
    let v = ask(&mut k, sol(false), Some(vec![new]));
    let checks = v["checks"].as_array().unwrap();
    assert!(!checks.is_empty());
    for c in checks {
        let names = c["bodies"].to_string();
        assert!(names.contains("planet:9") || c["check"] == "amd", "{c}");
    }
    // Squeezed in at 1.03 au it is too close to Earth.
    assert_eq!(find(&v, "hill", "planet:2")["level"], 2);
    assert_eq!(v["verdict"], "unstable");
    // A body with no mass is named, not guessed.
    let mut q = sol(true);
    q.push(json!({"address": format!("{s}/planet:9"), "facts": {"orbit.host": s, "orbit.semi_major_axis": 2.0 * AU}}));
    let v = ask(&mut k, q, None);
    assert!(
        v["unchecked"].to_string().contains("planet:9 body.mass"),
        "{v}"
    );
    let _ = Request::Hello;
}
