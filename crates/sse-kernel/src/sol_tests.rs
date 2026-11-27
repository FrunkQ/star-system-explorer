//! The catalogued Solar System through the surface chain: Sol enters from
//! `packs/base-sol` as one catalogued batch per body, the way the start
//! sheet's door sends it, and its worlds resolve from those facts.

use sse_core::math as m;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const SOL: &str = "u:home/sys:sol";

/// The base-sol bodies at the start sheet's addresses (Sun star:0, bodies
/// body:N in sheet order, moons under their host), values as the pack has
/// them, plus `extra` facts by body id.
pub(crate) fn open_sol(extra: &[(&str, &str, serde_json::Value)]) -> crate::Kernel {
    open_sol_with(&[], extra)
}

/// `open_sol` with more packs on top of the sheet and base-physics.
fn open_sol_with(
    more: &[serde_json::Value],
    extra: &[(&str, &str, serde_json::Value)],
) -> crate::Kernel {
    let mut k = crate::Kernel::new();
    let sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-reference/pack.json")).unwrap(),
    )
    .unwrap();
    let physics: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-physics/pack.json")).unwrap(),
    )
    .unwrap();
    let mut stack = vec![sheet, physics];
    stack.extend(more.iter().cloned());
    let r = send(&mut k, serde_json::json!({"op":"load_stack","packs":stack}));
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"home","seed":"home"}),
    );
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-sol/pack.json")).unwrap(),
    )
    .unwrap();
    let sol = &pack["provides"]["other"]["sol"];
    // The app's own way in (`reference::sol_batch`, as `ui-contract/src/sol.ts`
    // builds it): angles printed in degrees become radians. Writing the
    // sheet's values raw gave Earth a tilt of 23.44 radians (CS1).
    let (addr, mut edits) = crate::reference::sol_batch(sol, SOL);
    for (id, a) in &addr {
        for (_, f, v) in extra.iter().filter(|(i, _, _)| i == id) {
            edits.push(serde_json::json!({"address":a,"field":f,"value":v}));
        }
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "evidence":"catalogued","source":"packs/base-sol","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    k
}

fn world(k: &mut crate::Kernel, name: &str) -> serde_json::Value {
    let r = send(k, serde_json::json!({"op":"resolved_system","system":SOL}));
    assert_eq!(r["type"], "system_resolved", "{r}");
    let addr = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == name)
        .map(|(a, _)| a.clone())
        .unwrap_or_else(|| panic!("no {name}"));
    r["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["address"] == addr.as_str())
        .cloned()
        .unwrap_or_else(|| panic!("no surface for {name}: {r}"))
}

/// base-sol as it ships: measured air, temperature and water reach the
/// chain, which then gives the planets we know. Plates and a field on Earth;
/// a still lid and no field on Venus and Mars.
#[test]
fn sol_from_its_catalogue_alone() {
    let mut k = open_sol(&[]);
    let e = world(&mut k, "Earth");
    assert_eq!(e["t_surface_now_k"], 288.0, "{e}");
    assert_eq!(e["regime_now"], "mobile-lid", "{e}");
    assert_eq!(e["core"]["dynamo_now"], true, "{e}");
    let v = world(&mut k, "Venus");
    assert_eq!(v["t_surface_now_k"], 737.0, "{v}");
    assert_ne!(v["regime_now"], "mobile-lid", "{v}");
    assert_eq!(v["core"]["dynamo_now"], false, "{v}");
    let m = world(&mut k, "Mars");
    assert_eq!(m["t_surface_now_k"], 214.0, "{m}");
    assert_eq!(m["regime_now"], "stagnant-lid", "{m}");
    assert_eq!(m["core"]["dynamo_now"], false, "{m}");
}

/// A world with no catalogued temperature (the Moon: NSSDCA prints only a
/// range) sits at its equilibrium temperature from its own Bond albedo.
#[test]
fn a_world_without_a_measured_mean_uses_its_own_albedo() {
    let mut k = open_sol(&[]);
    let e = world(&mut k, "Moon");
    let (teff, r_sun, a) = (5772.0_f64, 695_700e3_f64, 149.598e9_f64);
    let t_eq = teff * m::sqrt(r_sun / (2.0 * a)) * m::pow(1.0 - 0.11, 0.25);
    let t = e["t_surface_now_k"].as_f64().unwrap();
    assert!((t - t_eq).abs() < 0.01, "{t} against {t_eq}");
}

/// Facts typed over the catalogue win the same way.
#[test]
fn typed_air_and_temperature_win_over_estimates() {
    let mut k = open_sol(&[
        ("moon", "planet.surface_temperature", 250.0.into()),
        ("moon", "atmosphere.surface_pressure", 101_400.0.into()),
    ]);
    let e = world(&mut k, "Moon");
    assert_eq!(e["t_surface_now_k"], 250.0, "{e}");
}

/// A measurement belongs to the world it was made on: Mars moved to 1 AU by
/// the GM keeps its thin air's warming but answers to the brighter Sun.
#[test]
fn a_moved_world_keeps_its_air_but_not_its_old_temperature() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let mars = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == "Mars")
        .map(|(a, _)| a.clone())
        .unwrap();
    let au = sse_core::sheet::AU;
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "edits":[{"address":mars,"field":"orbit.semi_major_axis","value":au}]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let m = world(&mut k, "Mars");
    let t = m["t_surface_now_k"].as_f64().unwrap();
    let expect = 214.0 * m::sqrt(227.956e9 / au);
    assert!((t - expect).abs() < 0.01, "{t} against {expect}");
    // Typed by the GM, a temperature stands as typed, wherever the world is.
    send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "edits":[{"address":mars,"field":"planet.surface_temperature","value":230.0}]}),
    );
    assert_eq!(world(&mut k, "Mars")["t_surface_now_k"], 230.0);
}

fn composition(k: &mut crate::Kernel, name: &str, level: &str) -> serde_json::Value {
    let r = send(k, serde_json::json!({"op":"resolved_system","system":SOL}));
    let a = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == name)
        .map(|(a, _)| a.clone())
        .unwrap();
    let c = send(
        k,
        serde_json::json!({"op":"read_composition","system":SOL,"body":a,"level":level}),
    );
    assert_eq!(c["type"], "composition_read", "{c}");
    c["bodies"][0].clone()
}

fn share(c: &serde_json::Value, id: &str) -> f64 {
    c["elements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == id)
        .map(|e| e["mass_fraction"].as_f64().unwrap())
        .unwrap_or(0.0)
}

/// MP6: Earth's bulk from its catalogue alone, against McDonough and Sun
/// 1995 (table 4, bulk Earth: Fe 32.0, O 29.7, Si 16.1, Mg 15.4 wt%).
/// Rock's elements within 15%. Iron within 30%: its metal still carries
/// CI's sulfur, which Earth lost to volatility (MP14 brings that).
#[test]
fn earths_elements_from_its_catalogue() {
    let mut k = open_sol(&[]);
    let c = composition(&mut k, "Earth", "orbit");
    for (id, want, tol) in [
        ("o", 0.297, 0.15),
        ("si", 0.161, 0.15),
        ("mg", 0.154, 0.15),
        ("fe", 0.320, 0.30),
    ] {
        let got = share(&c, id);
        assert!(
            ((got - want) / want).abs() <= tol,
            "{id}: {got} against {want}"
        );
    }
    let total: f64 = c["elements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kg"].as_f64().unwrap())
        .sum();
    let mass = c["bulk"]["mass_kg"].as_f64().unwrap();
    assert!(
        (total - mass).abs() / mass < 0.01,
        "the elements make the body: {total} of {mass}"
    );
    assert_eq!(c["separation_strength"], 1.0, "{c}");
}

/// MP6: the same world at both levels. Every element's total at Orbit is
/// the sum over its reservoirs at Survey. Earth's core holds most of its
/// gold; its crust holds far more of its potassium than its share of the
/// rock; its sea holds hydrogen and its air nitrogen.
#[test]
fn orbit_totals_are_survey_reservoirs_summed() {
    let mut k = open_sol(&[]);
    let orbit = composition(&mut k, "Earth", "orbit");
    let survey = composition(&mut k, "Earth", "survey");
    let at = |c: &serde_json::Value, id: &str| -> serde_json::Value {
        c["elements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("no {id}"))
    };
    for e in orbit["elements"].as_array().unwrap() {
        let id = e["id"].as_str().unwrap();
        let kg = e["kg"].as_f64().unwrap();
        let r = &at(&survey, id)["reservoirs"];
        let sum: f64 = ["core_kg", "mantle_kg", "crust_kg", "sea_kg", "air_kg"]
            .iter()
            .map(|k| r[k].as_f64().unwrap())
            .sum();
        assert!(
            (sum - kg).abs() <= 1e-9 * kg.max(1e-30),
            "{id}: {sum} against {kg}"
        );
        assert!(e.get("reservoirs").is_none(), "Orbit gives totals only");
    }
    let au = at(&survey, "au");
    let share = au["reservoirs"]["core_kg"].as_f64().unwrap() / au["kg"].as_f64().unwrap();
    assert!(share > 0.9, "Earth's core holds most of its gold: {share}");
    let k40 = at(&survey, "k");
    let crust = k40["reservoirs"]["crust_kg"].as_f64().unwrap() / k40["kg"].as_f64().unwrap();
    let fc = survey["crust_fraction"].as_f64().unwrap();
    assert!(
        crust > 10.0 * fc,
        "potassium rises into the crust: {crust} with a crust of {fc}"
    );
    let w = at(&survey, "h");
    assert!(
        w["reservoirs"]["sea_kg"].as_f64().unwrap() > 0.0,
        "the sea holds hydrogen"
    );
    let n = at(&survey, "n");
    assert!(
        n["reservoirs"]["air_kg"].as_f64().unwrap() > 0.0,
        "the air holds nitrogen"
    );
}

/// A small world barely melted, so it barely separated: Ceres keeps its
/// metal through its rock. Park et al. 2016 find it only partly separated,
/// rock from volatiles (ice and salts); its metal is our own reading.
#[test]
fn small_worlds_barely_separate() {
    let mut k = open_sol(&[]);
    let c = composition(&mut k, "Ceres", "survey");
    let s = c["separation_strength"].as_f64().unwrap();
    assert!(s < 0.1, "{s}");
    let bad = send(
        &mut k,
        serde_json::json!({"op":"read_composition","system":SOL,"level":"site"}),
    );
    assert_eq!(bad["code"], "bad_level", "{bad}");
}

/// MP8: Earth's deposits hold real kilograms drawn from its crust. The ore
/// share is calibrated so that its arc copper is Kesler and Wilkinson 2008's
/// porphyry copper (about 1.7e11 t, at all crustal depths); every element's deposits
/// and the rest add up to the crust `read_composition` gives at Survey.
#[test]
fn earths_deposits_hold_its_porphyry_copper() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let earth = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == "Earth")
        .map(|(a, _)| a.clone())
        .unwrap();
    let s = send(
        &mut k,
        serde_json::json!({"op":"survey_surface","system":SOL,"body":earth,"limit":0}),
    );
    assert_eq!(s["type"], "surface_surveyed", "{s}");
    let h = &s["survey"]["holdings"];
    let kind = |k: &str| {
        h["kinds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["kind"] == k)
            .cloned()
            .unwrap()
    };
    let held = |x: &serde_json::Value, id: &str| {
        x["elements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .map(|e| e["kg"].as_f64().unwrap())
            .unwrap_or(0.0)
    };
    let arc = kind("copper-arc");
    let cu = held(&arc, "cu");
    // Kesler and Wilkinson 2008: about 1.7e11 t (1.7e14 kg). Within 35%:
    // with Earth's measured core share (32.5%, SOL4) the mantle, and so the
    // arc's copper, is a touch smaller than with the old 32% stand-in.
    assert!((cu / 1.7e14 - 1.0).abs() < 0.35, "arc copper {cu:.3e} kg");
    assert!(held(&kind("gold-orogenic"), "au") > 0.0, "{h}");
    let c = composition(&mut k, "Earth", "survey");
    for x in h["crust"].as_array().unwrap() {
        let id = x["id"].as_str().unwrap();
        let crust = x["crust_kg"].as_f64().unwrap();
        let (a, b) = (
            x["in_deposits_kg"].as_f64().unwrap(),
            x["disseminated_kg"].as_f64().unwrap(),
        );
        assert!(a >= 0.0 && b >= 0.0, "{x}");
        assert!(((a + b) - crust).abs() <= 1e-9 * crust, "{x}");
        let want = c["elements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .map(|e| e["reservoirs"]["crust_kg"].as_f64().unwrap())
            .unwrap();
        assert!(
            (want - crust).abs() <= 1e-9 * want,
            "{id}: {want} against {crust}"
        );
    }
}

/// TG11 (MP8b): resource inventories read off the makeup, with no dice. Two
/// asteroids added beside Sol from makeup facts: a metal-rich one holds more
/// iron per kilogram than a carbonaceous one. A pack's kind for an element
/// nobody wrote a block for (iridium) gets an inventory; a made good gets
/// none; an undeclared element id works on its own.
#[test]
fn inventories_follow_the_makeup() {
    let pack = serde_json::json!({
        "format": "sse.pack/1", "id": "test.resources", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"resources": {"kinds": [
            {"id": "iron", "name": "Iron", "matter": "fe", "reach": "whole"},
            {"id": "iridium", "name": "Iridium", "matter": "ir"},
            {"id": "water-ice", "name": "Water ice", "matter": "h2o", "reach": "whole"},
            {"id": "packaged-food", "name": "Packaged food"}
        ]}}}
    });
    let mut k = open_sol_with(&[pack], &[]);
    let rock = |n: u32, name: &str, makeup: [f64; 4]| {
        let a = format!("{SOL}/body:{n}");
        let mut e = vec![
            serde_json::json!({"address":a,"field":"name","value":name}),
            serde_json::json!({"address":a,"field":"body.mass","value":2.0e19}),
            serde_json::json!({"address":a,"field":"body.radius","value":1.2e5}),
            serde_json::json!({"address":a,"field":"orbit.semi_major_axis","value":4.2e11}),
            serde_json::json!({"address":a,"field":"orbit.host","value":format!("{SOL}/star:0")}),
        ];
        for (f, v) in ["makeup.metal", "makeup.rock", "makeup.carbon", "makeup.ice"]
            .iter()
            .zip(makeup)
        {
            e.push(serde_json::json!({"address":a,"field":f,"value":v}));
        }
        (a, e)
    };
    let (metal_a, mut edits) = rock(40, "Metal rock", [0.7, 0.3, 0.0, 0.0]);
    let (carb_a, more) = rock(41, "Carbon rock", [0.05, 0.6, 0.05, 0.3]);
    edits.extend(more);
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
            "evidence":"authored","source":"test","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let inv = |k: &mut crate::Kernel, body: &str, kinds: serde_json::Value| {
        let r = send(
            k,
            serde_json::json!({"op":"read_inventory","system":SOL,"body":body,"kinds":kinds}),
        );
        assert_eq!(r["type"], "inventory_read", "{r}");
        r["bodies"][0].clone()
    };
    let of = |b: &serde_json::Value, id: &str| {
        b["kinds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == id)
            .cloned()
            .unwrap()
    };
    let all = serde_json::json!(["iron", "iridium", "water-ice", "packaged-food", "ni"]);
    let m = inv(&mut k, &metal_a, all.clone());
    let c = inv(&mut k, &carb_a, all.clone());
    let fe = |b: &serde_json::Value| of(b, "iron")["per_kg"].as_f64().unwrap();
    assert!(
        fe(&m) > 3.0 * fe(&c),
        "iron per kg {} against {}",
        fe(&m),
        fe(&c)
    );
    assert!(of(&m, "iridium")["kg"].as_f64().unwrap() > 0.0, "{m}");
    assert!(
        of(&c, "water-ice")["kg"].as_f64().unwrap() > of(&m, "water-ice")["kg"].as_f64().unwrap()
    );
    assert_eq!(of(&m, "packaged-food")["kg"].as_f64().unwrap(), 0.0);
    assert!(of(&m, "packaged-food")["note"].is_string());
    assert!(
        of(&m, "ni")["kg"].as_f64().unwrap() > 0.0,
        "an undeclared element id is a kind"
    );
    assert_eq!(m["source"], "the body's makeup facts");
    // No dice: the same read gives the same amounts.
    assert_eq!(inv(&mut k, &metal_a, all), m);
    // Every declared kind when none is named; an unknown kind is refused.
    let r = send(
        &mut k,
        serde_json::json!({"op":"read_inventory","system":SOL,"body":metal_a}),
    );
    assert_eq!(r["bodies"][0]["kinds"].as_array().unwrap().len(), 4, "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"read_inventory","system":SOL,"kinds":["unobtainium"]}),
    );
    assert_eq!(r["code"], "unknown_resource", "{r}");
}

/// X3j (record-logic design 2.4): a resource moves only by `transfer`, which
/// conserves it. Mining a rock raises its `extracted` and the ship's `stock`;
/// what a place holds is natural - extracted + stock; a place cannot give
/// more than it holds, and a made good cannot appear from nothing.
#[test]
fn transfers_conserve_and_mining_draws_down_a_rock() {
    let pack = serde_json::json!({
        "format": "sse.pack/1", "id": "test.mining", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"resources": {"kinds": [
            {"id": "iron", "name": "Iron", "matter": "fe", "reach": "crust"},
            {"id": "packaged-food", "name": "Packaged food"}
        ]}}}
    });
    let mut k = open_sol_with(&[pack], &[]);
    let rock = format!("{SOL}/body:40");
    let mut edits = vec![];
    for (f, v) in [
        ("name", serde_json::json!("Metal rock")),
        ("body.mass", 2.0e19.into()),
        ("body.radius", 1.2e5.into()),
        ("orbit.semi_major_axis", 4.2e11.into()),
        ("orbit.host", format!("{SOL}/star:0").into()),
        ("makeup.metal", 0.7.into()),
        ("makeup.rock", 0.3.into()),
        ("makeup.carbon", 0.0.into()),
        ("makeup.ice", 0.0.into()),
    ] {
        edits.push(serde_json::json!({"address": rock, "field": f, "value": v}));
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
        "evidence":"authored","source":"test","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let ship = format!("{SOL}/ship:7");
    let station = format!("{SOL}/station:2");
    let iron = |k: &mut crate::Kernel| {
        let r = send(
            k,
            serde_json::json!({"op":"read_inventory","system":SOL,"body":rock,"kinds":["iron"]}),
        );
        assert_eq!(r["type"], "inventory_read", "{r}");
        r["bodies"][0]["kinds"][0].clone()
    };
    let before = iron(&mut k);
    let natural = before["kg"].as_f64().unwrap();
    assert!(natural > 1e15, "{before}");
    assert_eq!(before["held_kg"].as_f64().unwrap(), natural);
    assert_eq!(before["extracted_kg"].as_f64().unwrap(), 0.0);
    let move_ = |k: &mut crate::Kernel, from: &str, to: &str, kind: &str, kg: f64, at_s: i64| {
        send(
            k,
            serde_json::json!({"op":"transfer","kind":kind,"from":from,"to":to,"kg":kg,"at_s":at_s}),
        )
    };
    // Mine 5 kt into the ship.
    let r = move_(&mut k, &rock, &ship, "iron", 5.0e6, 100);
    assert_eq!(r["type"], "transferred", "{r}");
    assert_eq!(r["from"]["extracted_kg"].as_f64().unwrap(), 5.0e6, "{r}");
    assert_eq!(r["to"]["stock_kg"].as_f64().unwrap(), 5.0e6, "{r}");
    assert_eq!(
        r["to"]["natural_kg"].as_f64().unwrap(),
        0.0,
        "a ship holds only stock"
    );
    let after = iron(&mut k);
    assert_eq!(after["extracted_kg"].as_f64().unwrap(), 5.0e6, "{after}");
    assert_eq!(after["held_kg"].as_f64().unwrap(), natural - 5.0e6);
    // Earlier reads are unchanged: the record is dated.
    let r = send(
        &mut k,
        serde_json::json!({"op":"read_inventory","system":SOL,"body":rock,"kinds":["iron"],"at_s":50}),
    );
    assert_eq!(
        r["bodies"][0]["kinds"][0]["held_kg"].as_f64().unwrap(),
        natural,
        "{r}"
    );
    // Unload 2 kt at the station; the ship cannot give more than it holds.
    let r = move_(&mut k, &ship, &station, "iron", 2.0e6, 200);
    assert_eq!(r["from"]["held_kg"].as_f64().unwrap(), 3.0e6, "{r}");
    assert_eq!(r["to"]["held_kg"].as_f64().unwrap(), 2.0e6, "{r}");
    let r = move_(&mut k, &ship, &station, "iron", 3.5e6, 300);
    assert_eq!(r["code"], "not_enough", "{r}");
    // Putting ore back on the rock adds stock there, so the books still balance.
    let r = move_(&mut k, &ship, &rock, "iron", 1.0e6, 400);
    assert_eq!(r["to"]["stock_kg"].as_f64().unwrap(), 1.0e6, "{r}");
    assert_eq!(r["to"]["held_kg"].as_f64().unwrap(), natural - 4.0e6, "{r}");
    // Nothing from nothing: food nobody has stocked cannot be moved.
    let r = move_(&mut k, &station, &ship, "packaged-food", 1.0, 500);
    assert_eq!(r["code"], "not_enough", "{r}");
    // The GM stocks it by hand (a kept value), then it moves.
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_kept","address":station,"kind":"resource.packaged-food.stock","value":800.0,"at_s":600}),
    );
    assert_eq!(r["type"], "kept_set", "{r}");
    let r = move_(&mut k, &station, &ship, "packaged-food", 300.0, 700);
    assert_eq!(r["to"]["held_kg"].as_f64().unwrap(), 300.0, "{r}");
    assert_eq!(r["from"]["held_kg"].as_f64().unwrap(), 500.0, "{r}");
    // Bad asks are refused in words.
    assert_eq!(
        move_(&mut k, &ship, &ship, "iron", 1.0, 800)["code"],
        "bad_address"
    );
    assert_eq!(
        move_(&mut k, &ship, &station, "iron", -1.0, 800)["code"],
        "bad_value"
    );
    assert_eq!(
        move_(&mut k, &ship, &station, "unobtainium", 1.0, 800)["code"],
        "unknown_resource"
    );
}

/// X3k (TG8 part 2): the base resources pack's labels read off held
/// amounts on Sol: the four giants carry hydrogen and helium-3 to skim,
/// Earth has water at the surface and Venus none, and an authored iron rock
/// is metal-rich where Earth is not. (The catalogue planets all read the
/// same iron share today, about 23%, so Mercury is not yet metal-rich: the
/// composition stand-in, not the label.)
#[test]
fn resource_labels_read_off_sol() {
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../packs/base-resources/pack.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut k = open_sol_with(&[pack], &[]);
    let rock = format!("{SOL}/body:40");
    let mut edits = vec![];
    for (f, v) in [
        ("name", serde_json::json!("Iron rock")),
        ("body.mass", 2.0e19.into()),
        ("body.radius", 1.2e5.into()),
        ("orbit.semi_major_axis", 4.2e11.into()),
        ("orbit.host", format!("{SOL}/star:0").into()),
        ("makeup.metal", 0.8.into()),
        ("makeup.rock", 0.2.into()),
        ("makeup.carbon", 0.0.into()),
        ("makeup.ice", 0.0.into()),
    ] {
        edits.push(serde_json::json!({"address": rock, "field": f, "value": v}));
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
        "evidence":"authored","source":"test","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"read_inventory","system":SOL}),
    );
    assert_eq!(r["type"], "inventory_read", "{r}");
    let labels_at = |a: &str| -> Vec<String> {
        r["bodies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["body"] == a)
            .map(|b| {
                b["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|t| t.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    let labels = |n: u32| labels_at(&format!("{SOL}/body:{n}"));
    assert!(
        labels_at(&rock).contains(&"resource.metal-rich".to_string()),
        "{:?}",
        labels_at(&rock)
    );
    assert!(
        !labels(2).contains(&"resource.metal-rich".to_string()),
        "Earth {:?}",
        labels(2)
    );
    assert!(
        labels(2).contains(&"resource.water".to_string()),
        "Earth {:?}",
        labels(2)
    );
    assert!(
        !labels(1).contains(&"resource.water".to_string()),
        "Venus {:?}",
        labels(1)
    );
    for n in 4..=7 {
        let l = labels(n);
        assert!(
            l.contains(&"resource.hydrogen".to_string()),
            "giant {n} {l:?}"
        );
        assert!(
            l.contains(&"resource.helium-3".to_string()),
            "giant {n} {l:?}"
        );
    }
    for n in 0..=3 {
        assert!(
            !labels(n).contains(&"resource.helium-3".to_string()),
            "rocky {n}"
        );
    }
}

/// X3k: `mine` moves ore at the ship's own rate, capped by what is left;
/// the rock's label goes when it is mined out, by itself.
#[test]
fn mining_out_a_rock_takes_its_label() {
    let pack = serde_json::json!({
        "format": "sse.pack/1", "id": "test.mine", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {
            "resources": {"kinds": [
                {"id": "ice", "name": "Ice", "matter": "h2o", "reach": "whole",
                 "label": {"tag": "test.icy", "min_kg": 1.0}}
            ]},
            "tags": {"groups": [{"id": "test", "label": "Test", "applies_to": ["body"]}],
                     "tags": [{"id": "test.icy", "label": "icy", "group": "test", "source": "derived"}]}
        }}
    });
    let mut k = open_sol_with(&[pack], &[]);
    let rock = format!("{SOL}/body:41");
    let ship = format!("{SOL}/ship:3");
    let mut edits = vec![];
    for (f, v) in [
        ("name", serde_json::json!("Ice rock")),
        ("body.mass", 1.0e12.into()),
        ("body.radius", 700.0.into()),
        ("orbit.semi_major_axis", 4.2e11.into()),
        ("orbit.host", format!("{SOL}/star:0").into()),
        ("makeup.metal", 0.05.into()),
        ("makeup.rock", 0.45.into()),
        ("makeup.carbon", 0.0.into()),
        ("makeup.ice", 0.5.into()),
    ] {
        edits.push(serde_json::json!({"address": rock, "field": f, "value": v}));
    }
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,
        "evidence":"authored","source":"test","edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let read = |k: &mut crate::Kernel, at_s: i64| {
        let r = send(
            k,
            serde_json::json!({"op":"read_inventory","system":SOL,"body":rock,"at_s":at_s}),
        );
        assert_eq!(r["type"], "inventory_read", "{r}");
        r["bodies"][0].clone()
    };
    let b = read(&mut k, 10);
    let held = b["kinds"][0]["held_kg"].as_f64().unwrap();
    assert!(held > 0.0, "{b}");
    assert_eq!(b["labels"], serde_json::json!(["test.icy"]), "{b}");
    // No rate, no mining.
    let r = send(
        &mut k,
        serde_json::json!({"op":"mine","ship":ship,"body":rock,"kind":"ice","hours":1.0,"at_s":20}),
    );
    assert_eq!(r["code"], "no_mining_rate", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"evidence":"authored","source":"test",
        "edits":[{"address": ship, "field": "mining.rate_kg_h", "value": held / 4.0}]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    // One hour takes a quarter; the label stays.
    let r = send(
        &mut k,
        serde_json::json!({"op":"mine","ship":ship,"body":rock,"kind":"ice","hours":1.0,"at_s":3600}),
    );
    assert_eq!(r["type"], "transferred", "{r}");
    assert!(
        (r["kg"].as_f64().unwrap() - held / 4.0).abs() <= held * 1e-12,
        "{r}"
    );
    assert_eq!(
        read(&mut k, 3600)["labels"],
        serde_json::json!(["test.icy"])
    );
    // Ten more hours take only what is left, and the label goes.
    let r = send(
        &mut k,
        serde_json::json!({"op":"mine","ship":ship,"body":rock,"kind":"ice","hours":10.0,"at_s":7200}),
    );
    assert_eq!(r["type"], "transferred", "{r}");
    assert_eq!(r["from"]["held_kg"].as_f64().unwrap(), 0.0, "{r}");
    assert!(
        (r["to"]["held_kg"].as_f64().unwrap() - held).abs() <= held * 1e-12,
        "{r}"
    );
    let b = read(&mut k, 7200);
    assert_eq!(b["labels"], serde_json::json!([]), "{b}");
    assert_eq!(b["kinds"][0]["label"]["earned"], false, "{b}");
    // Earlier, it still had it; and an empty rock cannot be mined again.
    assert_eq!(read(&mut k, 10)["labels"], serde_json::json!(["test.icy"]));
    let r = send(
        &mut k,
        serde_json::json!({"op":"mine","ship":ship,"body":rock,"kind":"ice","hours":1.0,"at_s":9000}),
    );
    assert_eq!(r["code"], "not_enough", "{r}");
}

/// MP8c: a giant is mostly an envelope of hydrogen and helium, at the birth
/// cloud's shares, around the mass-metallicity relation's heavy elements
/// (Thorngren et al. 2016): Jupiter about 14% heavy, Neptune far more.
/// Hydrogen and helium-3 skim from Jupiter's envelope; Earth has none.
#[test]
fn giants_are_mostly_envelope() {
    let pack = serde_json::json!({
        "format": "sse.pack/1", "id": "test.gas", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"resources": {"kinds": [
            {"id": "hydrogen", "matter": "h", "reach": "envelope"},
            {"id": "helium-3", "matter": "he3", "reach": "envelope"}
        ]}}}
    });
    let mut k = open_sol_with(&[pack], &[]);
    let jup = composition(&mut k, "Jupiter", "survey");
    let nep = composition(&mut k, "Neptune", "survey");
    let heavy = |c: &serde_json::Value| 1.0 - share(c, "h") - share(c, "he");
    let (hj, hn) = (heavy(&jup), heavy(&nep));
    assert!(hj > 0.08 && hj < 0.25, "Jupiter heavy share {hj}");
    assert!(hn > 2.0 * hj, "Neptune {hn} against Jupiter {hj}");
    // Hydrogen about 0.71 to helium's 0.27 by mass in the envelope.
    let ratio = share(&jup, "h") / share(&jup, "he");
    assert!(ratio > 2.2 && ratio < 3.2, "H/He {ratio}");
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let addr = |name: &str| {
        r["facts"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, f)| f["name"] == name)
            .map(|(a, _)| a.clone())
            .unwrap()
    };
    let inv = |k: &mut crate::Kernel, name: &str| {
        let r = send(
            k,
            serde_json::json!({"op":"read_inventory","system":SOL,"body":addr(name)}),
        );
        assert_eq!(r["type"], "inventory_read", "{r}");
        let kinds = r["bodies"][0]["kinds"].clone();
        let get = |id: &str| {
            kinds
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .cloned()
                .unwrap()
        };
        (get("hydrogen"), get("helium-3"))
    };
    let (jh, j3) = inv(&mut k, "Jupiter");
    let (eh, _) = inv(&mut k, "Earth");
    assert!(jh["kg"].as_f64().unwrap() > 1.0e26, "{jh}");
    assert_eq!(eh["kg"].as_f64().unwrap(), 0.0, "{eh}");
    // Helium-3: the envelope's helium at the birth cloud's isotope share
    // (Lodders 2021: 1.66e-4 of helium atoms), by mass.
    let he = share(&jup, "he") * jup["bulk"]["mass_kg"].as_f64().unwrap();
    let he3 = j3["kg"].as_f64().unwrap();
    let want = he * 1.66e-4 * 3.016 / 4.0026;
    assert!(
        (he3 / want - 1.0).abs() < 0.02,
        "helium-3 {he3:.3e} against {want:.3e}"
    );
}

/// `reference_tiles`: Mars, the Moon and the rest show their own faces
/// without a campaign, the campaign is left as it was, and an unknown id
/// says which bodies there are.
#[test]
fn reference_bodies_have_surfaces_without_a_campaign() {
    let mut k = open_sol(&[]);
    let before = send(&mut k, serde_json::json!({"op":"state_hash"}));
    let tiles: Vec<_> = (0..6)
        .map(|face| serde_json::json!({"face":face,"level":0,"x":0,"y":0}))
        .collect();
    for id in ["mercury", "venus", "moon", "mars"] {
        let r = send(
            &mut k,
            serde_json::json!({"op":"reference_tiles","body":id,"tiles":tiles}),
        );
        assert_eq!(r["type"], "reference_tiles", "{r}");
        assert_eq!(r["tiles"].as_array().unwrap().len(), 6, "{id}: {r}");
        assert!(
            r["address"]
                .as_str()
                .unwrap()
                .starts_with("u:reference/sys:sol/"),
            "{r}"
        );
    }
    let moon = send(
        &mut k,
        serde_json::json!({"op":"reference_tiles","body":"moon","tiles":tiles}),
    );
    assert_eq!(moon["address"], "u:reference/sys:sol/body:2/body:0");
    // The same tiles as the opened Solar System's own Mars.
    let mars = send(
        &mut k,
        serde_json::json!({"op":"reference_tiles","body":"mars","tiles":tiles}),
    );
    assert_eq!(send(&mut k, serde_json::json!({"op":"state_hash"})), before);
    // With no campaign open at all, it still answers.
    let mut bare = crate::Kernel::new();
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-reference/pack.json")).unwrap(),
    )
    .unwrap();
    send(
        &mut bare,
        serde_json::json!({"op":"load_stack","packs":[sheet, packs["base-physics"].clone()]}),
    );
    let r = send(
        &mut bare,
        serde_json::json!({"op":"reference_tiles","body":"venus","tiles":tiles}),
    );
    assert_eq!(r["tiles"].as_array().unwrap().len(), 6, "{r}");
    // The reference Mars is one world, whatever campaign is open (its own
    // seed, not the campaign's).
    let m2 = send(
        &mut bare,
        serde_json::json!({"op":"reference_tiles","body":"mars","tiles":tiles}),
    );
    assert_eq!(m2["tiles"], mars["tiles"]);
    let r = send(
        &mut bare,
        serde_json::json!({"op":"reference_tiles","body":"vulcan","tiles":tiles}),
    );
    assert_eq!(r["code"], "no_reference_body", "{r}");
    assert!(r["message"].as_str().unwrap().contains("mars"), "{r}");
}

/// Every Solar System world publishes its moment of inertia factor: the
/// rocky ones from core and mantle, the giants as polytropes less a core.
#[test]
fn worlds_publish_their_moment_of_inertia_factor() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let c = |name: &str| -> f64 {
        r["facts"]
            .as_object()
            .unwrap()
            .values()
            .find(|f| f["name"] == name)
            .and_then(|f| f["body.moment_of_inertia_factor"].as_f64())
            .unwrap_or_else(|| panic!("no C for {name}"))
    };
    let (earth, mars, jupiter, saturn) = (c("Earth"), c("Mars"), c("Jupiter"), c("Saturn"));
    assert!(earth > 0.3 && earth < 0.37, "Earth {earth}");
    assert!(mars > 0.3 && mars < 0.4, "Mars {mars}");
    assert!(jupiter > 0.22 && jupiter < 0.27, "Jupiter {jupiter}");
    assert!(saturn > 0.19 && saturn < jupiter, "Saturn {saturn}");
    eprintln!("C: Earth {earth:.4} Mars {mars:.4} Moon {:.4} Jupiter {jupiter:.4} Saturn {saturn:.4} Uranus {:.4}", c("Moon"), c("Uranus"));
}

/// A catalogued world's measured temperature is today's: its time-lapse
/// follows the young Sun's fainter light back (the faint young Sun), and
/// the frames no longer say the history waits.
#[test]
fn catalogued_earth_feels_the_faint_young_sun() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"surface_frames","system":SOL,"body":"u:home/sys:sol/body:2"}),
    );
    assert_eq!(r["type"], "frames_made");
    let frames = r["frames"]["frames"]["frames"].as_array().expect("frames");
    let mean = |f: &serde_json::Value| {
        let t = f["temp_k"].as_array().unwrap();
        t.iter().map(|x| x.as_f64().unwrap()).sum::<f64>() / t.len() as f64
    };
    let (first, last) = (mean(&frames[0]), mean(frames.last().unwrap()));
    assert!(
        first < last - 10.0,
        "young Earth {first:.1} K, today {last:.1} K"
    );
    let pending = r.to_string();
    assert!(!pending.contains("this world's climate is a fixed value"));
}

/// MF2 part 2: the giants make their fields in their metallic hydrogen.
/// Jupiter's equatorial field is about 420 μT (Connerney et al. 2018) and
/// Saturn's about 21 μT; the ice giants are weaker still. Within a factor
/// of a few, as the scaling law promises.
#[test]
fn giants_make_their_fields_in_their_deep_hydrogen() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let fact = |name: &str, key: &str| -> f64 {
        r["facts"]
            .as_object()
            .unwrap()
            .values()
            .find(|f| f["name"] == name)
            .and_then(|f| f[key].as_f64())
            .unwrap_or_else(|| panic!("no {key} for {name}"))
    };
    for n in ["Jupiter", "Saturn", "Uranus", "Neptune"] {
        eprintln!(
            "MF2 {n}: B_eq {:.1} μT, moment {:.2e}, top {:.2} R, Rm {:.2e}, flow {:.2e}",
            fact(n, "body.surface_magnetic_field") * 1e6,
            fact(n, "body.magnetic_moment"),
            fact(n, "interior.dynamo_radius") / fact(n, "body.radius"),
            fact(n, "interior.magnetic_reynolds"),
            fact(n, "interior.dynamo_flow_speed"),
        );
    }
    let bj = fact("Jupiter", "body.surface_magnetic_field") * 1e6;
    assert!((140.0..1300.0).contains(&bj), "Jupiter {bj} μT");
    let bs = fact("Saturn", "body.surface_magnetic_field") * 1e6;
    assert!(bs > 3.0 && bs < bj, "Saturn {bs} μT");
    let top = fact("Jupiter", "interior.dynamo_radius") / fact("Jupiter", "body.radius");
    assert!(
        (0.7..0.95).contains(&top),
        "Jupiter's dynamo top at {top} R"
    );
    // MF3 part 1: every magnetised world has a drawn tilt, marked drawn; a
    // world with no field has none.
    for f in r["facts"].as_object().unwrap().values() {
        let on = f["body.magnetic_moment"].as_f64().is_some_and(|x| x > 0.0);
        if on {
            let tilt = f["field.tilt"].as_f64().unwrap();
            assert!((0.0..=90.0).contains(&tilt), "{tilt}");
            assert_eq!(f["field.tilt_grade"], "drawn");
        } else {
            assert!(f.get("field.tilt").is_none(), "{f}");
        }
    }
    let again = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    assert_eq!(again["facts"], r["facts"]);
}

/// SOL4: catalogued rocky worlds bring their measured interiors. Mercury is
/// metal-rich (its core is about 70% of its mass), the Moon nearly
/// coreless, and the measured moment of inertia factors win over the
/// two-layer estimate.
#[test]
fn catalogued_worlds_bring_their_measured_interiors() {
    let mut k = open_sol(&[]);
    let fe = |k: &mut crate::Kernel, n: &str| share(&composition(k, n, "orbit"), "fe");
    let (mercury, earth, moon, mars) = (
        fe(&mut k, "Mercury"),
        fe(&mut k, "Earth"),
        fe(&mut k, "Moon"),
        fe(&mut k, "Mars"),
    );
    // Mercury's bulk is about 60 to 70% iron; the core alloy's light
    // elements (CI's sulfur, MP14) hold it near half here, against Earth's 0.3.
    assert!(mercury > 0.45, "Mercury's iron {mercury}");
    assert!(moon < 0.15, "the Moon's iron {moon}");
    assert!(
        mercury > earth && earth > mars && mars > moon,
        "{mercury} {earth} {mars} {moon}"
    );
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let c = |name: &str| {
        r["facts"]
            .as_object()
            .unwrap()
            .values()
            .find(|f| f["name"] == name)
            .and_then(|f| f["body.moment_of_inertia_factor"].as_f64())
            .unwrap()
    };
    assert_eq!(c("Earth"), 0.3307);
    assert_eq!(c("Moon"), 0.3932);
}

/// SOL5: a catalogued world's climate breathes its measured air, not the
/// default mix (Mars and Venus are mostly CO2, with no free oxygen).
#[test]
fn catalogued_worlds_breathe_their_measured_air() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    for name in ["Mars", "Venus", "Earth"] {
        let addr = r["facts"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, f)| f["name"] == name)
            .map(|(a, _)| a.clone())
            .unwrap();
        let s = send(
            &mut k,
            serde_json::json!({"op":"survey_surface","system":SOL,"body":addr,"limit":0}),
        );
        assert_eq!(s["type"], "surface_surveyed", "{s}");
        let assumed = &s["survey"]["climate"]["assumed"];
        assert!(assumed.is_array(), "{name}: no climate in {s}");
        let assumed = assumed.to_string();
        assert!(
            !assumed.contains("default mix"),
            "{name} climate assumed {assumed}"
        );
    }
    let mars = r["facts"]
        .as_object()
        .unwrap()
        .values()
        .find(|f| f["name"] == "Mars")
        .unwrap();
    let gases = crate::adopt::catalogued_gases(None);
    assert!(gases.is_none());
    assert!(
        mars["atmosphere.composition.co2"].as_f64().unwrap() > 0.9,
        "{mars}"
    );
}

/// S14a part 3: the catalogued Moon's younger maria keep whole tube roofs
/// (Haruyama et al. 2009), while its oldest have opened into rilles.
#[test]
fn the_moons_younger_maria_keep_roofed_tubes() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let moon = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == "Moon")
        .map(|(a, _)| a.clone())
        .unwrap();
    let s = send(
        &mut k,
        serde_json::json!({"op":"survey_surface","system":SOL,"body":moon,"limit":0}),
    );
    assert_eq!(s["type"], "surface_surveyed", "{s}");
    let tubes = s["survey"]["tubes"].as_array().expect("tubes");
    let stages: Vec<&str> = tubes.iter().filter_map(|t| t["stage"].as_str()).collect();
    assert!(stages.contains(&"intact roof"), "{stages:?}");
}

/// SW1: the airless Moon's bare ground weathers dark and red with exposure
/// age, while young craters stay fresher; Earth's air stops it, and
/// Mercury, closer to the Sun, weathers faster.
#[test]
fn airless_ground_weathers_with_exposure_age() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let addr = |name: &str| {
        r["facts"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, f)| f["name"] == name)
            .map(|(a, _)| a.clone())
            .unwrap()
    };
    let tiles: Vec<serde_json::Value> = (0..6)
        .map(|f| serde_json::json!({"face":f,"level":1,"x":0,"y":0}))
        .collect();
    let survey = |k: &mut crate::Kernel, name: &str| {
        let s = send(
            k,
            serde_json::json!({"op":"survey_surface","system":SOL,"body":addr(name),"limit":0,"tiles":tiles}),
        );
        assert_eq!(s["type"], "surface_surveyed", "{s}");
        s
    };
    let moon = survey(&mut k, "Moon");
    let w = &moon["survey"]["weathering"];
    assert_eq!(w["active"], true, "{w}");
    assert_eq!(w["wind_reaches_ground"], true, "{w}");
    let f = w["mature_factor_rgb"].as_array().unwrap();
    assert!(f[0].as_f64() > f[2].as_f64(), "mature soil is redder: {w}");
    let mut all: Vec<u8> = vec![];
    for t in moon["survey"]["tiles"].as_array().unwrap() {
        all.extend(
            t["maturity"]
                .as_array()
                .expect("maturity")
                .iter()
                .map(|x| x.as_u64().unwrap() as u8),
        );
    }
    let mean = all.iter().map(|x| *x as f64).sum::<f64>() / all.len() as f64;
    let least = *all.iter().min().unwrap();
    let moon_tau = w["tau_myr"].as_f64().unwrap();
    assert!(mean > 200.0, "old lunar ground is mature: {mean}");
    assert!(least < 200, "young craters stay fresher: {least}");
    let earth = survey(&mut k, "Earth");
    assert!(
        earth["survey"]["weathering"].is_null(),
        "air stops weathering"
    );
    for t in earth["survey"]["tiles"].as_array().unwrap() {
        assert!(t.get("maturity").is_none(), "air stops weathering");
    }
    let mercury = survey(&mut k, "Mercury");
    let w = &mercury["survey"]["weathering"];
    assert_eq!(w["active"], true, "{w}");
    // Closer in, the wind is stronger, so Mercury's soil matures faster.
    assert!(w["tau_myr"].as_f64().unwrap() < moon_tau, "{w}");
}

/// The giants carry their measured 1 bar temperatures, so Saturn's top deck
/// is ammonia ice (and takes the chromophore), not the ice giants' methane.
#[test]
fn saturns_top_deck_is_ammonia_from_its_one_bar_temperature() {
    let mut k = open_sol(&[]);
    let s = world(&mut k, "Saturn");
    assert_eq!(s["t_surface_now_k"], 134.0, "{s:.300}");
    let addr = s["address"].as_str().unwrap().to_string();
    let w = send(
        &mut k,
        serde_json::json!({"op":"weather","want":"field","system":SOL,"body":addr}),
    );
    let text = w.to_string();
    let c = send(
        &mut k,
        serde_json::json!({"op":"weather","want":"climate","system":SOL,"body":s["address"]}),
    );
    // Its own catalogued air, hydrogen and helium, not the default mix.
    let gases = &c["data"]["core"]["conditions"]["gases"];
    assert_eq!(gases["h2"], 0.963, "{gases}");
    assert!(gases.get("n2").is_none(), "{gases}");
    assert!(text.contains("ammonia ice"), "{text:.600}");
    assert!(text.contains("chromophore"), "{text:.600}");
    assert!(!text.contains("methane ice"), "{text:.600}");
    let u = world(&mut k, "Uranus");
    let w = send(
        &mut k,
        serde_json::json!({"op":"weather","want":"field","system":SOL,"body":u["address"]}),
    );
    assert!(w.to_string().contains("methane ice"), "{w:.600}");
}

/// IG2: a catalogued moon that carries only its mass and radius is as icy
/// to the regime rules as to its ice ground (one door, read off its
/// density), so Europa's thin shell cracks and renews its ground (young,
/// bright ice) while Callisto's thick lid keeps old dark ground. Titan
/// carries its measured air (Huygens), so it is a hydrocarbon world, not
/// airless. Ceres's open ice sublimes away, so its ice regime is rock.
#[test]
fn icy_moons_take_their_regime_from_their_ice() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let tiles: Vec<serde_json::Value> = (0..6)
        .map(|f| serde_json::json!({"face":f,"level":0,"x":0,"y":0}))
        .collect();
    let mut look = |name: &str| {
        let a = r["facts"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, f)| f["name"] == name)
            .map(|(a, _)| a.clone())
            .unwrap();
        let s = send(
            &mut k,
            serde_json::json!({"op":"survey_surface","system":SOL,"body":a,"limit":0,"tiles":tiles}),
        );
        assert_eq!(s["type"], "surface_surveyed", "{s}");
        let sv = &s["survey"];
        let (mut al, mut age, mut ice, mut n) = (0.0, 0.0, 0.0, 0.0);
        for t in sv["tiles"].as_array().unwrap() {
            for (i, x) in t["albedo"].as_array().unwrap().iter().enumerate() {
                al += x.as_f64().unwrap() / 1000.0;
                age += t["surface_age_myr"][i].as_f64().unwrap();
                ice += f64::from(u8::from(t["material"][i] == 3));
                n += 1.0;
            }
        }
        let regime = sv["envelope"]["regime_now"].as_str().unwrap().to_string();
        (regime, al / n, age / n, ice / n)
    };
    let europa = look("Europa");
    let callisto = look("Callisto");
    assert_eq!(europa.0, "ice-shell", "{europa:?}");
    assert_eq!(callisto.0, "cold-ice-lid", "{callisto:?}");
    // Europa's craters say its surface is tens of millions of years old;
    // the whole age (4567 Myr) is the fault this fixes.
    assert!(europa.2 < 500.0, "Europa's ground is young: {europa:?}");
    assert!(callisto.2 > 4000.0, "Callisto's lid is old: {callisto:?}");
    assert!(europa.1 > 0.5, "young ice is bright: {europa:?}");
    assert!(europa.1 > callisto.1 + 0.3, "{europa:?} {callisto:?}");
    let titan = look("Titan");
    assert_eq!(titan.0, "hydrocarbon", "{titan:?}");
    let ceres = look("Ceres");
    assert_eq!(ceres.0, "cold-ice-lid", "{ceres:?}");
    assert_eq!(ceres.3, 0.0, "Ceres's open ice is lost: {ceres:?}");
    let facts = r["facts"].as_object().unwrap();
    let t = facts.values().find(|f| f["name"] == "Titan").unwrap();
    assert!(
        (t["atmosphere.surface_pressure"].as_f64().unwrap() - 146_700.0).abs() < 1.0,
        "{t}"
    );
}

/// WS2: the weather op's flow table for a giant of the catalogued Sol: its jets, storms and eddies, the eddies fed
/// by the heat it sends out; a rocky world has none.
#[test]
fn a_giants_flow_table_carries_its_eddies() {
    let mut k = open_sol(&[]);
    let j = world(&mut k, "Jupiter");
    let w = send(
        &mut k,
        serde_json::json!({"op":"weather","want":"flow","system":SOL,"body":j["address"],"day":10.0,"span_days":2.0}),
    );
    let t = &w["data"];
    assert_eq!(t["day0"], 10.0, "{w:.400}");
    assert_eq!(t["day1"], 12.0);
    assert_eq!(t["jet_lat_deg"].as_array().unwrap().len(), 721);
    let eddies = t["eddies"].as_array().unwrap();
    assert!(eddies.len() > 500, "{} eddies", eddies.len());
    assert!(t["facts"]["eddy_power_w_kg"].as_f64().unwrap() > 0.0);
    let earth = world(&mut k, "Earth");
    let e = send(
        &mut k,
        serde_json::json!({"op":"weather","want":"flow","system":SOL,"body":earth["address"]}),
    );
    assert_eq!(e["code"], "not_a_giant", "{e:.300}");
}

/// CS1: catalogued worlds take the Sun's measured light at their distance
/// from it (a moon's is its planet's) and their own measured Bond albedo
/// (NSSDCA: Earth 0.294, Moon 0.11), never a sun-like guess or the pack's
/// default.
#[test]
fn catalogued_worlds_get_their_own_sunlight_and_albedo() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    for (name, albedo) in [("Earth", 0.294), ("Moon", 0.11)] {
        let addr = r["facts"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, f)| f["name"] == name)
            .map(|(a, _)| a.clone())
            .unwrap();
        let s = send(
            &mut k,
            serde_json::json!({"op":"survey_surface","system":SOL,"body":addr,"limit":0}),
        );
        let c = &s["survey"]["climate"];
        let assumed = c["assumed"].to_string();
        assert!(!assumed.contains("sun-like"), "{name}: {assumed}");
        assert!(!assumed.contains("Bond albedo"), "{name}: {assumed}");
        // About 340 W/m² over the sphere at 1 AU (1361 / 4); the Moon's was
        // its orbit round the Earth before HV2c, about 10^8 times too much.
        let arriving = c["sunlight"]["arriving_w_m2"].as_f64().unwrap();
        assert!((330.0..350.0).contains(&arriving), "{name}: {arriving}");
        let bond = c["sunlight"]["bond_albedo"].as_f64().unwrap();
        assert!((bond - albedo).abs() < 0.03, "{name}: albedo {bond}");
    }
}

/// The Sol loader turns the sheet's degrees into radians, as the app does:
/// Earth's tilt is 23.44° (0.409 rad), so its equator is warmer than its
/// poles. Written raw, the tilt was 23.44 rad and Earth's climate ran
/// upside down (equator 279 K, poles 305 K).
#[test]
fn sol_loads_angles_in_radians_and_earths_equator_is_warmest() {
    let mut k = open_sol(&[]);
    let r = send(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":SOL}),
    );
    let (addr, earth) = r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == "Earth")
        .map(|(a, f)| (a.clone(), f.clone()))
        .unwrap();
    let tilt = earth["body.obliquity"].as_f64().unwrap();
    assert!(
        (tilt - 23.44 * sse_core::math::PI / 180.0).abs() < 1e-9,
        "{tilt}"
    );
    let s = send(
        &mut k,
        serde_json::json!({"op":"survey_surface","system":SOL,"body":addr,"limit":0}),
    );
    let t: Vec<f64> = s["survey"]["climate"]["t_k"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let equator = t[t.len() / 2];
    // The year and sphere average of the sunlight is S/4, about 340 W/m²
    // (CERES era); the wrong tilt made it 386 (moved here from CS1's
    // sunlight test, which needs this loader).
    let arriving = s["survey"]["climate"]["sunlight"]["arriving_w_m2"]
        .as_f64()
        .unwrap();
    assert!((330.0..350.0).contains(&arriving), "{arriving}");
    // Measured: about 300 K at the equator, 228 to 255 K at the poles.
    assert!(equator > t[0] + 30.0, "{t:?}");
    assert!(equator > t[t.len() - 1] + 30.0, "{t:?}");
}
