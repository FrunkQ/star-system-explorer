//! V3 barycentre nodes mapped onto the hierarchy on adoption (BC1 next).

use sse_core::math as m;

const AU_M: f64 = 1.495_978_707e11;
const M_SUN: f64 = 1.988_47e30;

fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn star(id: &str, parent: &str, mass_sun: f64, a_au: f64, m0: f64) -> serde_json::Value {
    serde_json::json!({
        "id": id, "name": id, "parentId": parent, "kind": "body", "roleHint": "star",
        "massKg": mass_sun * M_SUN, "radiusKm": 600000.0, "temperatureK": 5000.0,
        "orbit": {"hostId": parent, "t0": 0, "elements":
            {"a_AU": a_au, "e": 0.1, "i_deg": 3.0, "Omega_deg": 10.0, "omega_deg": 20.0, "M0_rad": m0}}
    })
}

fn world(id: &str, host: &str, mass: f64, a_au: f64) -> serde_json::Value {
    serde_json::json!({
        "id": id, "name": id, "parentId": host, "kind": "body", "roleHint": "planet",
        "massKg": mass, "radiusKm": 6000.0,
        "orbit": {"hostId": host, "t0": 0, "elements":
            {"a_AU": a_au, "e": 0.0, "i_deg": 0.0, "Omega_deg": 0.0, "omega_deg": 0.0, "M0_rad": 0.0}}
    })
}

/// A V3 binary as V3 writes it (`setupStars.ts`): two stars round a massless
/// root barycentre, each at its share of the separation, half a turn apart;
/// a circumbinary planet round the barycentre; a planet round the companion;
/// and a V3 double planet (`barycenterReconcile.ts`): a pair round its own
/// barycentre, which orbits star A.
fn save() -> String {
    let (ma, mb, sep) = (1.0, 0.5, 20.0);
    let (big, small, pair_sep_au) = (6.0e24, 1.5e24, 0.002);
    let mut nodes = vec![
        serde_json::json!({"id": "ab", "name": "AB Barycentre", "parentId": null,
            "kind": "barycenter", "memberIds": ["a", "b"], "effectiveMassKg": (ma + mb) * M_SUN}),
        star("a", "ab", ma, sep * mb / (ma + mb), 0.0),
        star("b", "ab", mb, sep * ma / (ma + mb), m::PI),
        world("p", "ab", 1.0e25, 80.0),
        world("q", "b", 1.0e24, 1.0),
        serde_json::json!({"id": "dd", "name": "Double", "parentId": "a", "kind": "barycenter",
            "memberIds": ["big", "small"], "effectiveMassKg": big + small,
            "orbit": {"hostId": "a", "t0": 0, "elements":
                {"a_AU": 2.0, "e": 0.0, "i_deg": 0.0, "Omega_deg": 0.0, "omega_deg": 0.0, "M0_rad": 1.0}}}),
        world("big", "dd", big, pair_sep_au * small / (big + small)),
        world("small", "dd", small, pair_sep_au * big / (big + small)),
        world("moon", "small", 1.0e20, 0.0002),
    ];
    if let Some(o) = nodes[7]["orbit"]["elements"].as_object_mut() {
        o.insert("M0_rad".into(), serde_json::json!(m::PI));
    }
    serde_json::json!({
        "bundleFormat": 1, "id": "bin", "name": "Binary", "appVersion": "3.1.0",
        "systems": [{"id": "s", "name": "Binary", "position": {"x": 0, "y": 0, "z": 0},
            "system": {"id": "sys-bin", "name": "Binary", "seed": "x", "epochT0": 0, "nodes": nodes}}]
    })
    .to_string()
}

fn adopted() -> (crate::Kernel, String, serde_json::Value) {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "bin", "seed": "bin"}),
    );
    assert!(r["type"] != "error", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "adopt", "import_v3": save()}),
    );
    assert_eq!(r["type"], "adopted", "{r}");
    let sys = r["systems"][0].as_str().unwrap().to_string();
    (k, sys, r)
}

fn placed<'a>(o: &'a serde_json::Value, a: &str) -> &'a serde_json::Value {
    o["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["address"] == a)
        .unwrap_or_else(|| panic!("{a} not in {o}"))
}

fn period(o: &serde_json::Value, a: &str) -> f64 {
    let b = placed(o, a);
    assert_eq!(b["placed"], true, "{b}");
    b["period_s"].as_f64().unwrap()
}

fn kepler(a_m: f64, mass: f64) -> f64 {
    2.0 * m::PI * m::sqrt(a_m * a_m * a_m / (sse_core::sheet::G * mass))
}

#[test]
fn a_v3_binary_becomes_the_companion_orbit() {
    let (mut k, sys, _) = adopted();
    let at = |n: u32| format!("{sys}/body:@{n}");
    let (a, b, p, q) = (at(1), at(2), at(3), at(4));
    let (big, small, moon) = (at(5), at(6), at(7));
    let v = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": sys}),
    );
    let facts = &v["facts"];
    let fact = |x: &str, f: &str| facts[x][f].clone();
    // No barycentre objects: the bary rows are gone, and nothing names them.
    assert!(
        !v.to_string().contains("/bary:"),
        "a barycentre address is left: {v}"
    );
    // A orbits nothing (the pair was the root); B orbits A at the whole
    // separation, keeping its own shape and phase.
    assert!(fact(&a, "orbit.host").is_null(), "{}", facts[&a]);
    assert_eq!(fact(&b, "orbit.host"), a.as_str());
    let sep = fact(&b, "orbit.semi_major_axis").as_f64().unwrap();
    assert!((sep / (20.0 * AU_M) - 1.0).abs() < 1e-12, "{sep}");
    assert_eq!(fact(&b, "orbit.eccentricity").as_f64(), Some(0.1));
    assert!((fact(&b, "orbit.mean_anomaly_at_epoch").as_f64().unwrap() - m::PI).abs() < 1e-12);
    // Worlds round the barycentre now orbit A with their elements; a world
    // round B stays round B; the double planet's heavy member takes the
    // pair's orbit round A, its light one orbits it, its moon stays put.
    assert_eq!(fact(&p, "orbit.host"), a.as_str());
    assert_eq!(fact(&q, "orbit.host"), b.as_str());
    assert_eq!(fact(&big, "orbit.host"), a.as_str());
    assert!(
        (fact(&big, "orbit.semi_major_axis").as_f64().unwrap() / (2.0 * AU_M) - 1.0).abs() < 1e-12
    );
    assert_eq!(
        fact(&big, "orbit.mean_anomaly_at_epoch").as_f64(),
        Some(1.0)
    );
    assert_eq!(fact(&small, "orbit.host"), big.as_str());
    assert!(
        (fact(&small, "orbit.semi_major_axis").as_f64().unwrap() / (0.002 * AU_M) - 1.0).abs()
            < 1e-12
    );
    assert_eq!(fact(&moon, "orbit.host"), small.as_str());
    assert_eq!(fact(&small, "parent"), big.as_str());

    // The hierarchy places them all, with the mutual periods V3 meant.
    let o = send(
        &mut k,
        serde_json::json!({"op": "orbits_at", "system": sys, "at_s": [0, 86400]}),
    );
    assert_eq!(o["type"], "orbits_placed", "{o}");
    // B's period counts everything further in (the double planet round A) and
    // its own world (Jacobi).
    let pb = period(&o, &b);
    assert!(
        (pb / kepler(20.0 * AU_M, 1.5 * M_SUN + 8.5e24 + 1.0e20) - 1.0).abs() < 1e-9,
        "{pb}"
    );
    // The circumbinary world counts both stars and all inside it.
    let pp = period(&o, &p);
    assert!(
        (pp / kepler(80.0 * AU_M, 1.5 * M_SUN + 8.5e24 + 1.0e20 + 1.0e25) - 1.0).abs() < 1e-9,
        "{pp}"
    );
    let ps = period(&o, &small);
    assert!(
        (ps / kepler(0.002 * AU_M, 7.5e24 + 1.0e20) - 1.0).abs() < 1e-9,
        "{ps}"
    );
    for x in [&q, &big, &moon] {
        period(&o, x);
    }
}

#[test]
fn the_report_says_how_each_pair_was_mapped() {
    let mut im = sse_import_v3::import_json(&save(), "bin").unwrap();
    let said = crate::adopt::rehome_barycentres(&mut im.facts);
    assert_eq!(said.len(), 2, "{said:?}");
    assert!(
        said[0].contains("now orbit") && said[0].contains("body:@1"),
        "{said:?}"
    );
    // A save with no barycentre is left exactly as it was.
    let sol = include_str!("../../../tests/fixtures/v3/synthetic-sol.json");
    let mut im = sse_import_v3::import_json(sol, "sol").unwrap();
    let before = im.facts.clone();
    assert!(crate::adopt::rehome_barycentres(&mut im.facts).is_empty());
    assert_eq!(before, im.facts);
}

/// A hierarchical triple as V3 builds it (`generateFromConfig.ts`): an inner
/// pair's barycentre is itself a member of the outer one. The inner pair's
/// heavy star stands for it: the third star orbits it at the outer
/// separation, the inner companion at the inner one.
#[test]
fn a_v3_triple_nests_onto_one_primary() {
    let (ma, mb, mc) = (1.1, 0.9, 0.12);
    let (inner, outer) = (23.0, 8700.0);
    let (mab, total) = (ma + mb, ma + mb + mc);
    let mut ab = serde_json::json!({"id": "ab", "name": "AB", "parentId": "o", "kind": "barycenter",
        "memberIds": ["a", "b"],
        "orbit": {"hostId": "o", "t0": 0, "elements":
            {"a_AU": outer * mc / total, "e": 0.0, "i_deg": 0.0, "Omega_deg": 0.0, "omega_deg": 0.0, "M0_rad": 0.0}}});
    ab["effectiveMassKg"] = serde_json::json!(mab * M_SUN);
    let nodes = vec![
        serde_json::json!({"id": "o", "name": "O", "parentId": null, "kind": "barycenter",
            "memberIds": ["ab", "c"]}),
        ab,
        star("a", "ab", ma, inner * mb / mab, 0.0),
        star("b", "ab", mb, inner * ma / mab, m::PI),
        star("c", "o", mc, outer * mab / total, m::PI),
    ];
    let save = serde_json::json!({
        "bundleFormat": 1, "id": "tri", "name": "Triple", "appVersion": "3.1.0",
        "systems": [{"id": "s", "name": "Triple", "position": {"x": 0, "y": 0, "z": 0},
            "system": {"id": "sys-tri", "name": "Triple", "seed": "x", "epochT0": 0, "nodes": nodes}}]
    })
    .to_string();
    let mut im = sse_import_v3::import_json(&save, "tri").unwrap();
    let said = crate::adopt::rehome_barycentres(&mut im.facts);
    assert_eq!(said.len(), 2, "{said:?}");
    let get = |a: &str, f: &str| {
        let a: sse_core::Address = a.parse().unwrap();
        im.facts
            .get(&a)
            .and_then(|r| r.get(f))
            .map(|x| x.value.clone())
    };
    let (a, b, c) = (
        "u:tri/sys:@1/body:@1",
        "u:tri/sys:@1/body:@2",
        "u:tri/sys:@1/body:@3",
    );
    assert_eq!(get(a, "orbit.host"), None);
    assert_eq!(get(b, "orbit.host"), Some(a.into()));
    assert_eq!(get(c, "orbit.host"), Some(a.into()));
    let sep = |x: &str| get(x, "orbit.semi_major_axis").unwrap().as_f64().unwrap() / AU_M;
    assert!((sep(b) / inner - 1.0).abs() < 1e-12, "{}", sep(b));
    assert!((sep(c) / outer - 1.0).abs() < 1e-12, "{}", sep(c));
    assert!(im.facts.keys().all(|k| k.last().kind() != "bary"));
    // The pairs' names went with A.
    assert_eq!(get(a, "v3.pair.bary:@1.name"), Some("O".into()));
    assert_eq!(get(a, "v3.pair.bary:@2.name"), Some("AB".into()));
}
