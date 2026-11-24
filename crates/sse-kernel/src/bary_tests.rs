//! Barycentre always: the Jacobi hierarchy in `orbits_at`.

use crate::orbits::{orbits_at, Placed};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::Address;
use std::collections::BTreeMap;

const AU: f64 = 1.495_978_707e11;
const M_SUN: f64 = 1.988_47e30;

fn a(s: &str) -> Address {
    s.parse().unwrap()
}
fn body(mass: f64, host: Option<&str>, sma: f64) -> BTreeMap<String, Value> {
    let mut r = BTreeMap::new();
    r.insert("body.mass".to_string(), Value::Float(mass));
    if let Some(h) = host {
        r.insert("orbit.host".to_string(), Value::Text(h.into()));
        r.insert("orbit.semi_major_axis".to_string(), Value::Float(sma));
        r.insert("orbit.mean_anomaly_at_epoch".to_string(), Value::Float(0.0));
    }
    r
}
fn get<'a>(p: &'a [Placed], s: &str) -> &'a Placed {
    p.iter().find(|x| x.address == s).unwrap()
}
fn len(v: [f64; 3]) -> f64 {
    m::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
}
fn minus(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

const SUN: &str = "u:t/sys:0/star:0";
const PLUTO: &str = "u:t/sys:0/body:9";
const CHARON: &str = "u:t/sys:0/body:9/body:1";

/// Pluto and Charon go round a point outside Pluto (about 2,100 km from its
/// centre); the period counts both masses.
#[test]
fn pluto_wobbles_round_a_point_outside_itself() {
    let (mp, mc, ac) = (1.303e22, 1.586e21, 1.9591e7);
    let mut f = BTreeMap::new();
    f.insert(a(SUN), body(M_SUN, None, 0.0));
    f.insert(a(PLUTO), body(mp, Some(SUN), 39.48 * AU));
    f.insert(a(CHARON), body(mc, Some(PLUTO), ac));
    let times = [0, 100_000, 300_000];
    let p = orbits_at(&f, &a("u:t/sys:0"), &times, None);
    let (pl, ch) = (get(&p, PLUTO), get(&p, CHARON));
    assert!(pl.placed && ch.placed, "{p:?}");
    let want = 2.0 * m::PI * m::sqrt(ac * ac * ac / (sse_core::sheet::G * (mp + mc)));
    assert!((ch.period_s.unwrap() / want - 1.0).abs() < 1e-12);
    assert!((ch.period_s.unwrap() / 86_400.0 - 6.387).abs() < 0.01);
    let mut offsets = vec![];
    for (x, y) in pl.at.iter().zip(&ch.at) {
        // Charon's distance from Pluto's centre is the orbit's.
        assert!((len(y.r_m) / ac - 1.0).abs() < 1e-9);
        // The pair's barycentre, from Pluto's centre.
        let bary: Vec<f64> = (0..3)
            .map(|i| (mp * x.sys_m[i] + mc * y.sys_m[i]) / (mp + mc))
            .collect();
        let off = len(minus([bary[0], bary[1], bary[2]], x.sys_m));
        assert!((off - ac * mc / (mp + mc)).abs() < 1.0, "{off}");
        offsets.push(off);
        // Charon's orbit is round Pluto's centre (nothing further in).
        assert!(len(y.centre_m) < 1e-6);
    }
    assert!(offsets[0] > 1.188e6, "outside Pluto's radius");
}

/// A binary: the stars go round each other; a planet outside the companion
/// goes round the pair (its centre is the pair's barycentre, mu counts both);
/// a planet inside goes round its own star.
#[test]
fn a_planet_outside_a_companion_orbits_the_pair() {
    let mut f = BTreeMap::new();
    f.insert(a(SUN), body(M_SUN, None, 0.0));
    f.insert(a("u:t/sys:0/star:1"), body(M_SUN, Some(SUN), AU));
    f.insert(a("u:t/sys:0/body:1"), body(0.0, Some(SUN), 0.1 * AU));
    f.insert(a("u:t/sys:0/body:2"), body(0.0, Some(SUN), 5.0 * AU));
    let p = orbits_at(&f, &a("u:t/sys:0"), &[0], None);
    let b = get(&p, "u:t/sys:0/star:1");
    let inner = get(&p, "u:t/sys:0/body:1");
    let outer = get(&p, "u:t/sys:0/body:2");
    // The companion's mutual orbit: 1 au apart, both stars half way out.
    assert!((len(b.at[0].r_m) / AU - 1.0).abs() < 1e-9);
    assert!((len(b.at[0].sys_m) / AU - 0.5).abs() < 1e-9);
    let year2 = 2.0 * m::PI * m::sqrt(AU * AU * AU / (sse_core::sheet::G * 2.0 * M_SUN));
    assert!((b.period_s.unwrap() / year2 - 1.0).abs() < 1e-12);
    // The inner planet goes round its star alone.
    assert!(len(inner.at[0].centre_m) < 1.0);
    // The outer one goes round the pair: its centre is half the separation out.
    assert!((len(outer.at[0].centre_m) / AU - 0.5).abs() < 1e-9);
    let p5 = 2.0 * m::PI * m::sqrt(125.0 * AU * AU * AU / (sse_core::sheet::G * 2.0 * M_SUN));
    assert!((outer.period_s.unwrap() / p5 - 1.0).abs() < 1e-12);
}

/// Hosts that go round each other in a circle place nothing, and say why.
#[test]
fn a_ring_of_hosts_is_named_not_looped() {
    let mut f = BTreeMap::new();
    f.insert(
        a("u:t/sys:0/body:1"),
        body(1e20, Some("u:t/sys:0/body:2"), 1e7),
    );
    f.insert(
        a("u:t/sys:0/body:2"),
        body(1e20, Some("u:t/sys:0/body:1"), 1e7),
    );
    let p = orbits_at(&f, &a("u:t/sys:0"), &[0], None);
    assert!(p.iter().all(|x| !x.placed), "{p:?}");
    assert!(p[0].missing[0].contains("orbiting nothing"), "{p:?}");
}

/// The stability checks read the same hierarchy: worlds either side of a
/// companion star are not neighbours, and a world outside it is judged
/// against both stars' mass. A V3 star (role only) counts as a star.
#[test]
fn stability_reads_the_barycentre_hierarchy() {
    let mut f = BTreeMap::new();
    let mut sun = body(M_SUN, None, 0.0);
    sun.insert("v3.roleHint".into(), Value::Text("star".into()));
    f.insert(a(SUN), sun);
    let mut b = body(0.5 * M_SUN, Some(SUN), 2.0 * AU);
    b.insert("v3.roleHint".into(), Value::Text("star".into()));
    f.insert(a("u:t/sys:0/star:1"), b);
    for (k, au) in [(1, 0.2), (2, 0.3), (3, 9.0), (4, 14.0)] {
        f.insert(
            a(&format!("u:t/sys:0/body:{k}")),
            body(6e24, Some(SUN), au * AU),
        );
    }
    let (checks, _) = crate::stability::checks(&f, &a("u:t/sys:0"));
    let hill: Vec<&crate::stability::Check> = checks.iter().filter(|c| c.check == "hill").collect();
    let pairs: Vec<Vec<String>> = hill.iter().map(|c| c.bodies.clone()).collect();
    assert_eq!(pairs.len(), 2, "{pairs:?}");
    assert!(!pairs
        .iter()
        .any(|p| p.contains(&"u:t/sys:0/body:2".to_string())
            && p.contains(&"u:t/sys:0/body:3".to_string())));
    let outer = hill
        .iter()
        .find(|c| c.bodies[0] == "u:t/sys:0/body:3")
        .unwrap();
    let m = outer.inputs["host_mass"];
    assert!(m > 1.5 * M_SUN && m < 1.5001 * M_SUN, "{m}");
    assert!(checks.iter().any(|c| c.check == "binary_both_stars"));
}

/// Withdrawing a world takes what orbits it too, even a V3 moon that sits
/// beside it in the address tree.
#[test]
fn a_withdrawn_world_takes_its_moons() {
    let mut f = BTreeMap::new();
    f.insert(a(SUN), body(M_SUN, None, 0.0));
    let mut w = body(6e24, Some(SUN), AU);
    w.insert(crate::withdraw::WITHDRAWN.into(), Value::Bool(true));
    f.insert(a("u:t/sys:0/body:1"), w);
    f.insert(
        a("u:t/sys:0/body:2"),
        body(7e22, Some("u:t/sys:0/body:1"), 4e8),
    );
    f.insert(a("u:t/sys:0/body:3"), body(6e24, Some(SUN), 2.0 * AU));
    crate::withdraw::drop_withdrawn(&mut f);
    let left: Vec<String> = f.keys().map(|k| k.to_string()).collect();
    assert_eq!(left, ["u:t/sys:0/body:3", SUN]);
}

/// K13: a hand-written host must be there, not the body itself, and never
/// close a ring; refused by name, so nothing is silently unplaced.
#[test]
fn hosts_are_checked_on_write() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"t","seed":"t"}),
    );
    let batch = |edits: serde_json::Value| {
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":edits})
    };
    // A star and a world in one edit: fine.
    let r = send(
        &mut k,
        batch(serde_json::json!([
            {"address":SUN,"field":"body.mass","value":M_SUN},
            {"address":"u:t/sys:0/body:1","field":"orbit.host","value":SUN}
        ])),
    );
    assert_eq!(r["type"], "authored", "{r}");
    for (edits, says) in [
        (
            serde_json::json!([{"address":"u:t/sys:0/body:2","field":"orbit.host","value":"u:t/sys:0/body:9"}]),
            "is not in the universe",
        ),
        (
            serde_json::json!([{"address":"u:t/sys:0/body:1","field":"orbit.host","value":"u:t/sys:0/body:1"}]),
            "cannot orbit itself",
        ),
        (
            serde_json::json!([{"address":SUN,"field":"orbit.host","value":"u:t/sys:0/body:1"}]),
            "ring",
        ),
    ] {
        let r = send(&mut k, batch(edits));
        assert_eq!(r["code"], "bad_host", "{r}");
        assert!(r["message"].as_str().unwrap().contains(says), "{r}");
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored","edit":{"address":"u:t/sys:0/body:3",
            "field":"orbit.host","value":"u:t/sys:0/body:8","occurred_at_s":0}}),
    );
    assert_eq!(r["code"], "bad_host", "{r}");
}
