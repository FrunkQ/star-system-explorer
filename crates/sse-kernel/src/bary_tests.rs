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

/// A withdrawn world takes what orbits it too (withdraw's `take`, "remove
/// them too"), even a V3 moon that sits beside it in the address tree.
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
    let batch = |edits: serde_json::Value| serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":edits});
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

/// K13 `rehost`: keep the motion (the Moon set loose round the Sun stays
/// where it was and is bound), keep it tidy (a round orbit at its present
/// distance), or refuse by name when the motion is not bound to the new host.
#[test]
fn rehost_keeps_motion_or_tidies() {
    let mut k = crate::Kernel::new();
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"t","seed":"t"}),
    );
    let (earth, moon, rock) = (
        "u:t/sys:0/body:3",
        "u:t/sys:0/body:3/body:0",
        "u:t/sys:0/body:7",
    );
    let e = |a: &str, f: &str, v: serde_json::Value| serde_json::json!({"address":a,"field":f,"value":v});
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            e(SUN, "body.mass", M_SUN.into()),
            e(earth, "body.mass", 5.9722e24.into()), e(earth, "orbit.host", SUN.into()),
            e(earth, "orbit.semi_major_axis", 1.496e11.into()),
            e(moon, "body.mass", 7.346e22.into()), e(moon, "orbit.host", earth.into()),
            e(moon, "orbit.semi_major_axis", 3.844e8.into()),
            e(rock, "body.mass", 1e15.into()), e(rock, "orbit.host", SUN.into()),
            e(rock, "orbit.semi_major_axis", 3.0e11.into()),
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let at = 1_000_000_i64;
    let where_ = |k: &mut crate::Kernel, a: &str| -> [f64; 3] {
        let r = send(
            k,
            serde_json::json!({"op":"orbits_at","system":"u:t/sys:0","at_s":[at]}),
        );
        let b = r["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["address"] == a)
            .unwrap_or_else(|| panic!("{r}"))
            .clone();
        let v = &b["at"][0]["sys_m"];
        [
            v[0].as_f64().unwrap(),
            v[1].as_f64().unwrap(),
            v[2].as_f64().unwrap(),
        ]
    };
    let before = where_(&mut k, moon);
    // Not bound to a pebble.
    let r = send(
        &mut k,
        serde_json::json!({"op":"rehost","address":moon,"to":rock,"keep":"motion","at_s":at}),
    );
    assert_eq!(r["code"], "rehost_refused", "{r}");
    assert!(
        r["message"].as_str().unwrap().contains("would escape"),
        "{r}"
    );
    // Nor round its own moon.
    let r = send(
        &mut k,
        serde_json::json!({"op":"rehost","address":earth,"to":moon,"at_s":at}),
    );
    assert!(r["message"].as_str().unwrap().contains("orbits it"), "{r}");
    // Set loose round the Sun, moving as it was: bound, and where it was.
    let r = send(
        &mut k,
        serde_json::json!({"op":"rehost","address":moon,"to":SUN,"keep":"motion","at_s":at}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let after = where_(&mut k, moon);
    assert!(len(minus(after, before)) < 1.0e6, "{before:?} {after:?}");
    // Tidy: a round orbit round the rock at its present distance.
    let r = send(
        &mut k,
        serde_json::json!({"op":"rehost","address":earth,"to":rock,"at_s":at}),
    );
    assert_eq!(r["type"], "authored", "{r}");
}

/// K13 withdraw's three choices: leave what went round it round the body's
/// host (the default), take it too, or hand it to a named body; one event.
/// A body nothing goes round, or one that goes round nothing, is simply
/// withdrawn by default.
#[test]
fn withdraw_takes_leaves_or_hands_on() {
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let (earth, moon, mars) = (
        "u:t/sys:0/body:3",
        "u:t/sys:0/body:3/body:0",
        "u:t/sys:0/body:4",
    );
    let setup = || {
        let mut k = crate::Kernel::new();
        send(
            &mut k,
            serde_json::json!({"op":"open_universe","id":"t","seed":"t"}),
        );
        let e = |a: &str, f: &str, v: serde_json::Value| serde_json::json!({"address":a,"field":f,"value":v});
        let r = send(
            &mut k,
            serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
                e(SUN, "body.mass", M_SUN.into()),
                e(earth, "body.mass", 5.9722e24.into()), e(earth, "orbit.host", SUN.into()),
                e(earth, "orbit.semi_major_axis", 1.496e11.into()),
                e(moon, "body.mass", 7.346e22.into()), e(moon, "orbit.host", earth.into()),
                e(moon, "orbit.semi_major_axis", 3.844e8.into()),
                e(mars, "body.mass", 6.4e23.into()), e(mars, "orbit.host", SUN.into()),
                e(mars, "orbit.semi_major_axis", 2.28e11.into()),
            ]}),
        );
        assert_eq!(r["type"], "authored", "{r}");
        k
    };
    let host_after = |k: &mut crate::Kernel| -> Option<String> {
        let r = send(
            k,
            serde_json::json!({"op":"orbits_at","system":"u:t/sys:0","at_s":[200]}),
        );
        r["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["address"] == moon)
            .map(|b| b["parent"].as_str().unwrap().to_string())
    };
    let gone = |k: &mut crate::Kernel| {
        let u = k.universe().ok().unwrap();
        let mut st = u
            .chronicle
            .state_at(sse_chronicle::UniverseTime::seconds(200));
        crate::withdraw::drop_withdrawn(&mut st.facts);
        !st.facts.contains_key(&a(moon))
    };
    let mut k = setup();
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":earth,"at_s":100,"others":"take"}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    assert!(gone(&mut k));
    // With no choice named, what went round it stays round its host,
    // moving as it was (Alex 24 Nov 2026).
    let mut k = setup();
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":earth,"at_s":100}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    assert_eq!(
        r["affected"].as_array().unwrap().len(),
        2,
        "one event for both: {r}"
    );
    assert!(!gone(&mut k));
    assert_eq!(host_after(&mut k).as_deref(), Some(SUN));
    let mut k = setup();
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":earth,"at_s":100,"others":"hand","to":mars,"keep":"tidy"}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    assert_eq!(host_after(&mut k).as_deref(), Some(mars));
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":mars,"at_s":150,"others":"hand"}),
    );
    assert_eq!(r["code"], "bad_request", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":mars,"at_s":150}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"withdraw","address":SUN,"at_s":160}),
    );
    assert_eq!(r["type"], "authored", "a star goes round nothing: {r}");
}

/// K13 `preview_tree`: the tree before and after edits, nothing written;
/// Pluto and Charon read as a pair, the Earth and the Moon do not.
#[test]
fn preview_tree_shows_before_and_after_and_writes_nothing() {
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let (earth, moon) = ("u:t/sys:0/body:3", "u:t/sys:0/body:3/body:0");
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"t","seed":"t"}),
    );
    let e = |a: &str, f: &str, v: serde_json::Value| serde_json::json!({"address":a,"field":f,"value":v});
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            e(SUN, "body.mass", M_SUN.into()),
            e(earth, "body.mass", 5.9722e24.into()), e(earth, "orbit.host", SUN.into()),
            e(earth, "body.radius", 6.371e6.into()),
            e(earth, "orbit.semi_major_axis", 1.496e11.into()),
            e(moon, "body.mass", 7.346e22.into()), e(moon, "orbit.host", earth.into()),
            e(moon, "orbit.semi_major_axis", 3.844e8.into()),
            e(PLUTO, "body.mass", 1.303e22.into()), e(PLUTO, "orbit.host", SUN.into()),
            e(PLUTO, "body.radius", 1.188e6.into()),
            e(PLUTO, "orbit.semi_major_axis", (39.48 * AU).into()),
            e(CHARON, "body.mass", 1.586e21.into()), e(CHARON, "orbit.host", PLUTO.into()),
            e(CHARON, "orbit.semi_major_axis", 1.9591e7.into()),
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let row = |t: &serde_json::Value, a: &str| -> serde_json::Value {
        t["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["address"] == a)
            .unwrap_or_else(|| panic!("{a} in {t}"))
            .clone()
    };
    let r = send(
        &mut k,
        serde_json::json!({"op":"preview_tree","system":"u:t/sys:0","at_s":100,"edits":[
            {"op":"rehost","address":moon,"to":SUN,"keep":"motion","at_s":100}
        ]}),
    );
    assert_eq!(r["type"], "tree_previewed", "{r}");
    assert_eq!(row(&r["before"], moon)["host"], earth);
    assert_eq!(row(&r["after"], moon)["host"], SUN);
    assert_eq!(row(&r["before"], CHARON)["pair"], true, "{r}");
    assert_eq!(row(&r["before"], moon)["pair"], false, "{r}");
    assert!(
        r["changed"].as_array().unwrap().iter().any(|c| c == moon),
        "{r}"
    );
    // Nothing was written.
    let r = send(
        &mut k,
        serde_json::json!({"op":"preview_tree","system":"u:t/sys:0","at_s":100,"edits":[]}),
    );
    assert_eq!(row(&r["after"], moon)["host"], earth, "{r}");
    // A refused edit names itself, and leaves the universe as it was.
    let r = send(
        &mut k,
        serde_json::json!({"op":"preview_tree","system":"u:t/sys:0","at_s":100,"edits":[
            {"op":"rehost","address":earth,"to":moon,"at_s":100}
        ]}),
    );
    assert!(r["message"].as_str().unwrap().starts_with("edit 0:"), "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"preview_tree","system":"u:t/sys:0","at_s":100,"edits":[
            {"op":"open_universe","id":"x","seed":"x"}
        ]}),
    );
    assert_eq!(r["code"], "bad_request", "{r}");
}

/// ED1's exact answer on a pause: `resolved_system` with `branch` reads a
/// what-if's held value; the main line is untouched until a commit.
#[test]
fn resolved_system_reads_a_what_if_branch() {
    let send = |k: &mut crate::Kernel, v: serde_json::Value| -> serde_json::Value {
        serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
    };
    let earth = "u:t/sys:0/body:3";
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"t","seed":"t"}),
    );
    let e = |a: &str, f: &str, v: serde_json::Value| serde_json::json!({"address":a,"field":f,"value":v});
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            e(SUN, "body.mass", M_SUN.into()),
            e(earth, "body.mass", 5.9722e24.into()), e(earth, "orbit.host", SUN.into()),
            e(earth, "orbit.semi_major_axis", 1.496e11.into()),
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let mass = |r: &serde_json::Value| r["facts"][earth]["body.mass"].clone();
    let r = send(&mut k, serde_json::json!({"op":"branch","label":"held"}));
    let b = r["branch"].as_str().unwrap().to_string();
    let r = send(
        &mut k,
        serde_json::json!({"op":"branch_set","branch":b,"edit":{"address":earth,"field":"body.mass","value":1.2e25,"occurred_at_s":0}}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":"u:t/sys:0","branch":b}),
    );
    assert_eq!(r["type"], "system_resolved", "{r}");
    assert_eq!(mass(&r), 1.2e25, "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":"u:t/sys:0"}),
    );
    assert_eq!(mass(&r), 5.9722e24, "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":"u:t/sys:0","branch":"nope"}),
    );
    assert_eq!(r["code"], "unknown_branch", "{r}");
}
