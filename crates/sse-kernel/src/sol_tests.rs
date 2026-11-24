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
fn open_sol(extra: &[(&str, &str, serde_json::Value)]) -> crate::Kernel {
    let packs = crate::contract::load_packs(&root().join("tests/fixtures/packs"));
    let mut k = crate::Kernel::new();
    let sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("packs/base-reference/pack.json")).unwrap(),
    )
    .unwrap();
    let r = send(
        &mut k,
        serde_json::json!({"op":"load_stack","packs":[sheet, packs["base-physics"]]}),
    );
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
    let mut addr: Vec<(String, String)> = vec![];
    let mut n = 0;
    let mut edits = vec![];
    for b in sol["bodies"].as_array().unwrap() {
        let id = b["id"].as_str().unwrap().to_string();
        let a = match b["host"].as_str() {
            None => format!("{SOL}/star:0"),
            Some("sun") => {
                n += 1;
                format!("{SOL}/body:{}", n - 1)
            }
            Some(h) => {
                let ha = &addr.iter().find(|(i, _)| i == h).unwrap().1;
                format!("{ha}/body:0")
            }
        };
        edits.push(serde_json::json!({"address":a,"field":"name","value":b["name"]}));
        if let Some(h) = b["host"].as_str() {
            let ha = addr.iter().find(|(i, _)| i == h).unwrap().1.clone();
            edits.push(serde_json::json!({"address":a,"field":"orbit.host","value":ha}));
        }
        for (f, v) in b["values"].as_object().unwrap() {
            if f == "system.age_gyr" {
                let s = v["v"].as_f64().unwrap() * 1e9 * sse_core::sheet::YR_JULIAN;
                edits.push(serde_json::json!({"address":SOL,"field":"system.age","value":s}));
            } else {
                edits.push(serde_json::json!({"address":a,"field":f,"value":v["v"]}));
            }
        }
        for (_, f, v) in extra.iter().filter(|(i, _, _)| *i == id) {
            edits.push(serde_json::json!({"address":a,"field":f,"value":v}));
        }
        addr.push((id, a));
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
/// metal through its rock (Park et al. 2016 find it only partly separated).
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
/// porphyry copper (about 1.7e11 t above 3.3 km); every element's deposits
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
    // Kesler and Wilkinson 2008: about 1.7e11 t (1.7e14 kg).
    assert!((cu / 1.7e14 - 1.0).abs() < 0.3, "arc copper {cu:.3e} kg");
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
