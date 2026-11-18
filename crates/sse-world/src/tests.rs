use super::*;
use std::path::PathBuf;

fn tarn() -> WorldStack {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/worlds/tarn.json");
    WorldStack::from_json(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// The depth strip of the Strata mockup ("SSE4 mockups", artboard LookStrata),
/// line for line: depth, text, where. One deliberate difference: the mockup
/// writes the ocean's range as "−3 to −11" to fit its column; the ladder adds
/// the unit.
const STRATA: [(&str, &str, &str); 9] = [
    ("+400 km", "Tarn High, station", "orbit"),
    ("+52 km", "Highwater, aerostat", "storm belt"),
    ("0", "Biomes, 3 cities, Ridge 4 volcano", "surface"),
    (
        "\u{2212}0.4 km",
        "Undercroft, lava-tube city",
        "equator shelf",
    ),
    ("\u{2212}3 km", "Polar ice shell", "south pole"),
    (
        "\u{2212}3 to \u{2212}11 km",
        "Sub-ice brine ocean",
        "south pole",
    ),
    ("\u{2212}9 km", "Magma chamber", "under Ridge 4"),
    ("\u{2212}35 km", "Crust ends, 9 plates", "shell"),
    ("\u{2212}2,900 km", "Liquid core, dynamo on", "shell"),
];

#[test]
fn tarn_reproduces_the_strata_mockup_stack() {
    let t = tarn();
    assert!(t.validate().is_empty(), "{:?}", t.validate());
    let got: Vec<(String, String, String)> = t
        .ladder()
        .into_iter()
        .map(|l| (l.depth, l.text, l.place))
        .collect();
    let want: Vec<(String, String, String)> = STRATA
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn the_brine_ocean_stays_liquid_by_the_atoms() {
    // Mockup: "Stays liquid because salt lowers its freezing point and the ice
    // traps heat from below." 12 percent brine at 266 K.
    let (notes, problems) = tarn().physics_notes();
    assert!(problems.is_empty(), "{problems:?}");
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("brine-ocean: freezes at 265.1 K") && n.ends_with("liquid")),
        "{notes:?}"
    );
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("polar-ice:") && n.contains("0.031 W/m²")),
        "{notes:?}"
    );
    // Make it colder than its freezing point and the stack says so.
    let mut cold = tarn();
    let s = cold
        .shells
        .iter_mut()
        .find(|s| s.key == "brine-ocean")
        .unwrap();
    s.properties.insert("temperature_k".into(), 260.0);
    let (_, problems) = cold.physics_notes();
    assert_eq!(problems.len(), 1);
    assert!(problems[0].message.contains("frozen"));
}

#[test]
fn every_object_has_a_stable_address() {
    let t = tarn();
    assert_eq!(
        t.address_of("brine-ocean").unwrap().to_string(),
        "u:mockup-kestrel/sys:@1/body:@3/shell:@brine-ocean"
    );
    assert_eq!(
        t.address_of("south-pole").unwrap().to_string(),
        "u:mockup-kestrel/sys:@1/body:@3/region:@south-pole"
    );
    assert_eq!(
        t.address_of("ridge-4-chamber").unwrap().to_string(),
        "u:mockup-kestrel/sys:@1/body:@3/feature:@ridge-4-chamber"
    );
    assert!(t.address_of("nowhere").is_none());
}

#[test]
fn validation_reports_every_problem() {
    let mut t = tarn();
    t.shells.push(Shell {
        key: "crust".into(),
        kind: "crust".into(),
        name: "Second crust".into(),
        top_m: -10.0,
        bottom_m: -20.0,
        within: Some("atlantis".into()),
        properties: BTreeMap::new(),
        provides: vec![],
    });
    t.shells
        .iter_mut()
        .find(|s| s.key == "mantle")
        .unwrap()
        .top_m = -30_000.0;
    t.features
        .iter_mut()
        .find(|f| f.key == "highwater")
        .unwrap()
        .altitude_m = 70_000.0;
    t.features
        .iter_mut()
        .find(|f| f.key == "ridge-4-chamber")
        .unwrap()
        .altitude_m = 100.0;
    t.features.push(Feature {
        key: "lost".into(),
        kind: "city".into(),
        name: Some("Lost".into()),
        container: Some("atlantis".into()),
        altitude_m: 0.0,
        lat_deg: Some(91.0),
        lon_deg: None,
        properties: BTreeMap::new(),
        provides: vec![],
    });
    let issues: Vec<String> = t.validate().iter().map(|i| i.to_string()).collect();
    let joined = issues.join("\n");
    for want in [
        "crust: key used twice",
        "crust: within unknown region \"atlantis\"",
        "mantle: overlaps crust",
        "highwater: altitude 70000 m is outside region storm-belt",
        "ridge-4-chamber: is in ridge-4 but not beneath it",
        "lost: in unknown container \"atlantis\"",
        "lost: latitude 91 is off the globe",
    ] {
        assert!(joined.contains(want), "missing {want:?} in\n{joined}");
    }
}

#[test]
fn depth_formatting() {
    assert_eq!(format_km(0.0), "0");
    assert_eq!(format_km(400_000.0), "+400");
    assert_eq!(format_km(-400.0), "\u{2212}0.4");
    assert_eq!(format_km(-2_900_000.0), "\u{2212}2,900");
    assert_eq!(format_km(-1_234_567_000.0), "\u{2212}1,234,567");
}

#[test]
fn round_trips_through_json() {
    let t = tarn();
    let back = WorldStack::from_json(&serde_json::to_string(&t).unwrap()).unwrap();
    assert_eq!(t, back);
}
