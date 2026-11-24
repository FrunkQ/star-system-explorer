//! ATMOS: advice on a world's air.

use crate::atmos::{advise, AdviceParams};
use sse_core::cbor::Value;
use sse_life::chemistry::ThermoTable;
use sse_life::disequilibrium::DiseqParams;
use std::collections::BTreeMap;

fn table() -> ThermoTable {
    ThermoTable::from_packs(&[sse_life::scenario::base_life_pack()]).unwrap()
}

fn row(pairs: &[(&str, f64)]) -> BTreeMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), Value::Float(*v)))
        .collect()
}

/// Earth today as catalogued (base-sol, NSSDCA), with no formation run.
fn earth() -> BTreeMap<String, Value> {
    let mut r = row(&[
        ("body.mass", 5.972e24),
        ("body.radius", 6.371e6),
        ("interior.core_mass_fraction", 0.325),
        ("planet.surface_temperature", 288.0),
        ("atmosphere.surface_pressure", 101400.0),
        ("body.surface_water_oceans", 1.0),
        ("atmosphere.composition.n2", 0.7808),
        ("atmosphere.composition.o2", 0.2095),
        ("atmosphere.composition.ar", 0.00934),
        ("atmosphere.composition.co2", 0.00042),
        ("atmosphere.composition.ch4", 1.94e-6),
        ("atmosphere.composition.h2", 5.5e-7),
    ]);
    r.insert("body.oceans".into(), Value::Bool(true));
    r
}

fn advice(r: &BTreeMap<String, Value>) -> serde_json::Value {
    advise(
        "sse:u/test/sys:sol/body:earth",
        r,
        &table(),
        &DiseqParams::default(),
        &AdviceParams::default(),
    )
    .unwrap()
}

#[test]
fn earths_air_is_told_what_keeps_it_apart() {
    let a = advice(&earth());
    println!("{}", serde_json::to_string_pretty(&a).unwrap());
    assert_eq!(a["redox_from"], "core");
    // Methane with oxygen: the pair that carries the signal, lasting about
    // nine years (Prather, Holmes and Hsu 2012).
    let clashes = a["clashes"].as_array().unwrap();
    let ch4 = clashes
        .iter()
        .find(|c| c["sink"] == "methane-oxidation")
        .expect("methane with oxygen");
    let yr = ch4["time_s"].as_f64().unwrap() / crate::clock::JULIAN_YEAR_S;
    assert!((8.0..10.0).contains(&yr), "{yr}");
    assert!(ch4["pair"].as_array().unwrap().len() == 2);
    // Oxygen does not come from the magma ocean, and the carbon is far more
    // oxidised than the mantle's gas: told, not refused.
    let differs = a["differs"].as_array().unwrap();
    assert!(differs.iter().any(|d| d["gas"] == "o2"), "{differs:?}");
    assert!(
        differs
            .iter()
            .any(|d| d["gas"] == "co" && d["why"].as_str().unwrap().contains("more oxidised")),
        "{differs:?}"
    );
    // Hydrogen leaves Earth in tens of thousands of years.
    let esc = a["escapes"][0]["time_s"].as_f64().unwrap() / crate::clock::JULIAN_YEAR_S;
    assert!((3.0e3..3.0e5).contains(&esc), "{esc}");
    // The suggested batch: nitrogen and argon kept, oxygen dropped (it
    // clashes with what the rock gives off), water to the seas, hydrogen
    // to space, and the shares sum to one.
    let edits = a["suggest"]["edits"].as_array().unwrap();
    assert_eq!(a["suggest"]["op"], "set_authored_batch");
    let val = |g: &str| {
        edits
            .iter()
            .find(|e| e["field"] == format!("atmosphere.composition.{g}"))
            .map(|e| e["value"].as_f64().unwrap())
    };
    assert_eq!(val("o2"), Some(0.0));
    assert!(val("n2").unwrap() > 0.5);
    assert!(val("h2o").is_none() && val("h2") == Some(0.0));
    let sum: f64 = edits
        .iter()
        .filter(|e| {
            e["field"]
                .as_str()
                .unwrap()
                .starts_with("atmosphere.composition.")
        })
        .map(|e| e["value"].as_f64().unwrap())
        .sum();
    assert!((sum - 1.0).abs() < 1e-12, "{sum}");
    // Argon has no formation energy on the stack: left out, and said so.
    assert!(a["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n.as_str().unwrap().contains("ar")));
    let blocks: Vec<&str> = a["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["block"].as_str().unwrap())
        .collect();
    assert_eq!(
        blocks,
        [
            "sse.interior.redox_iw",
            "sse.atmosphere.outgassed_mix",
            "sse.life.disequilibrium"
        ]
    );
}

/// The mantle's redox sets the mix (Alex, 23 Nov): a big core leaves a
/// reduced mantle that breathes hydrogen and carbon monoxide; a small one
/// an oxidised mantle that breathes water and carbon dioxide.
#[test]
fn a_reduced_mantle_breathes_hydrogen_an_oxidised_one_water() {
    let expect = |cmf: f64| {
        let a = advice(&row(&[("interior.core_mass_fraction", cmf)]));
        let e = |g: &str| a["expected"][g].as_f64().unwrap_or(0.0);
        (
            a["redox_iw"].as_f64().unwrap(),
            e("h2") / e("h2o"),
            e("co") / e("co2"),
        )
    };
    let (r_small, h_small, c_small) = expect(0.1);
    let (r_big, h_big, c_big) = expect(0.6);
    assert!(r_big < r_small, "{r_big} {r_small}");
    assert!(h_big > 10.0 * h_small, "{h_big} {h_small}");
    assert!(c_big > 10.0 * c_small, "{c_big} {c_small}");
}

/// Formation's own figures win, and give the expected pressure.
#[test]
fn a_formation_run_sets_the_redox_and_the_pressure() {
    let mut r = row(&[
        ("body.mass", 5.972e24),
        ("body.radius", 6.371e6),
        ("interior.redox_iw", 0.5),
        ("early.surface_water", 1.4e21),
        ("early.co2_partial_pressure", 6.0e6),
    ]);
    r.insert("atmosphere.composition.co2".into(), Value::Float(0.96));
    r.insert("atmosphere.composition.n2".into(), Value::Float(0.035));
    let a = advice(&r);
    assert_eq!(a["redox_from"], "formation");
    // Water's weight as steam plus the CO2: a few hundred bar.
    let pa = a["pressure_pa"].as_f64().unwrap();
    assert!((1.0e7..1.0e8).contains(&pa), "{pa}");
    assert!(a["expected"]["h2o"].as_f64().unwrap() > a["expected"]["h2"].as_f64().unwrap());
    let edits = a["suggest"]["edits"].as_array().unwrap();
    assert!(edits
        .iter()
        .any(|e| e["field"] == "atmosphere.surface_pressure"));
}

/// With nothing known about the mantle, nothing is expected, and an air of
/// gases that go together has no clash.
#[test]
fn nothing_known_nothing_expected() {
    let a = advice(&row(&[
        ("atmosphere.composition.n2", 0.95),
        ("atmosphere.composition.co2", 0.05),
    ]));
    assert_eq!(a["redox_from"], "unknown");
    assert!(a["expected"].as_object().unwrap().is_empty());
    assert!(a["clashes"].as_array().unwrap().is_empty());
}
