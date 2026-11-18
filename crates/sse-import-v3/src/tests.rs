use super::*;
use std::path::PathBuf;

fn fixture() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/v3/synthetic-sol.json");
    std::fs::read_to_string(p).unwrap()
}

fn imported() -> Import {
    import_json(&fixture(), "v3-synthetic-sol").unwrap()
}

fn fact<'a>(im: &'a Import, a: &str, f: &str) -> &'a Fact {
    let addr: Address = a.parse().unwrap();
    im.facts
        .get(&addr)
        .and_then(|m| m.get(f))
        .unwrap_or_else(|| panic!("no {f} at {a}"))
}

#[test]
fn addresses_and_counts() {
    let im = imported();
    let c = &im.report.counts;
    assert_eq!(
        (c.systems, c.bodies, c.constructs, c.barycentres),
        (1, 3, 1, 0)
    );
    assert_eq!(im.report.source_kind, "starmap");
    assert_eq!(im.report.bundle_format, Some(1));
    assert_eq!(im.report.v3_app_version.as_deref(), Some("3.1.0"));
    assert_eq!(
        fact(&im, "u:v3-synthetic-sol/sys:@1/body:@2", "parent").value,
        "u:v3-synthetic-sol/sys:@1/body:@1"
    );
    assert_eq!(
        fact(&im, "u:v3-synthetic-sol/sys:@1/body:@3", "orbit.host").value,
        "u:v3-synthetic-sol/sys:@1/body:@2"
    );
    assert_eq!(
        fact(&im, "u:v3-synthetic-sol/sys:@1/craft:@1", "v3.name").value,
        "Station"
    );
}

#[test]
fn authored_quantities_arrive_in_si() {
    let im = imported();
    let e = "u:v3-synthetic-sol/sys:@1/body:@2";
    assert_eq!(fact(&im, e, "body.mass").value.as_f64(), Some(5.9722e24));
    assert_eq!(fact(&im, e, "body.radius").value.as_f64(), Some(6.371e6));
    assert_eq!(
        fact(&im, e, "orbit.semi_major_axis").value.as_f64(),
        Some(AU)
    );
    assert_eq!(
        fact(&im, e, "orbit.eccentricity").value.as_f64(),
        Some(0.0167)
    );
    let tilt = fact(&im, e, "body.axial_tilt").value.as_f64().unwrap();
    assert!((tilt - 23.44 * m::PI / 180.0).abs() < 1e-15);
    assert_eq!(
        fact(
            &im,
            "u:v3-synthetic-sol/sys:@1/body:@1",
            "star.effective_temperature"
        )
        .value
        .as_f64(),
        Some(5772.0)
    );
    let age = fact(&im, "u:v3-synthetic-sol/sys:@1", "system.age")
        .value
        .as_f64()
        .unwrap();
    assert!((age / (1e9 * 365.25 * 86400.0) - 4.57).abs() < 1e-12);
}

#[test]
fn derived_values_are_recomputed_not_imported() {
    let im = imported();
    let e: Address = "u:v3-synthetic-sol/sys:@1/body:@2".parse().unwrap();
    assert!(!im.facts[&e].contains_key("v3.densityGcm3"));
    let d = |field: &str, name: &str| {
        im.report
            .differences
            .iter()
            .find(|d| d.field == field && d.name == name)
            .unwrap_or_else(|| panic!("no difference for {name} {field}"))
            .clone()
    };
    assert!(d("densityGcm3", "Earth").within);
    assert!(d("calculatedGravity_ms2", "Earth").within);
    assert!(d("orbit.hostMu", "Earth").within);
    assert!(d("orbit.n_rad_per_s", "Earth").within);
    // The Moon's V3 density was an override: V4's physics disagrees, and says so.
    let moon = d("densityGcm3", "Moon");
    assert!(!moon.within, "{moon:?}");
    assert!((moon.v4 - 3.344).abs() < 0.01);
    assert_eq!(im.report.differences_outside_tolerance(), 1);
    // Seen but not yet comparable by first-wave atoms: named, with counts.
    assert_eq!(
        im.report.derived_not_compared.get("equilibriumTempK"),
        Some(&1)
    );
    assert_eq!(im.report.derived_not_compared.get("albedo"), Some(&1));
}

#[test]
fn honesty_marks_tags_and_gm_notes() {
    let im = imported();
    let moon = "u:v3-synthetic-sol/sys:@1/body:@3";
    assert_eq!(
        fact(&im, moon, "v3.overrides").evidence,
        Evidence::AuthoredImpossible
    );
    assert_eq!(im.report.honesty_marks, vec![moon.to_string()]);
    let e = "u:v3-synthetic-sol/sys:@1/body:@2";
    let tags = fact(&im, e, "v3.tags").value.as_array().unwrap().clone();
    let keys: Vec<&str> = tags.iter().map(|t| t["key"].as_str().unwrap()).collect();
    assert_eq!(
        keys,
        vec!["capital", "ocean"],
        "physics tags dropped, manual and overriding kept"
    );
    assert_eq!(tags[0]["secret"], true);
    assert_eq!(fact(&im, e, "v3.gmNotes").value, "the cult hides here");
}

#[test]
fn nothing_is_lost_silently() {
    let im = imported();
    let r = &im.report;
    assert_eq!(r.unclassified.get("aFieldV3NeverHad"), Some(&1));
    assert_eq!(
        fact(&im, "u:v3-synthetic-sol", "v3.aFieldV3NeverHad").value,
        42
    );
    assert_eq!(r.presentation.get("image"), Some(&1));
    assert_eq!(r.presentation.get("mapGrid"), Some(&1));
    assert_eq!(r.dropped.get("placementHealCount"), Some(&1));
    assert_eq!(r.dropped.get("generationEngine"), Some(&1));
    assert_eq!(r.dropped.get("system.rulePackId"), Some(&1));
    assert_eq!(r.dropped.get("orbit.seed"), Some(&1));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn import_is_deterministic_and_goes_into_a_chronicle() {
    let a = imported();
    let b = imported();
    assert_eq!(a.report.state_hash, b.report.state_hash);
    assert_eq!(a.report.to_json(), b.report.to_json());
    let events = a.events();
    assert!(events
        .iter()
        .any(|e| e.body.evidence == Evidence::AuthoredImpossible));
    assert_eq!(a.chronicle().events().len(), events.len());
    // Editing one authored value changes the state hash.
    let changed = fixture().replace("\"massKg\": 7.346e22", "\"massKg\": 7.35e22");
    let c = import_json(&changed, "v3-synthetic-sol").unwrap();
    assert_ne!(a.report.state_hash, c.report.state_hash);
}

#[test]
fn a_bare_system_file_imports_too() {
    let v: Json = serde_json::from_str(&fixture()).unwrap();
    let sys = v["systems"][0]["system"].to_string();
    let im = import_json(&sys, "v3-sol-only").unwrap();
    assert_eq!(im.report.source_kind, "system");
    assert_eq!(im.report.counts.bodies, 3);
}

#[test]
fn refuses_what_is_not_a_v3_save() {
    assert!(import_json("[1,2]", "x").is_err());
    assert!(import_json("{\"hello\": 1}", "x").is_err());
    assert!(import_json("{\"systems\": 3}", "x").is_err());
    assert!(import_json("{\"systems\": [], \"bundleFormat\": \"one\"}", "x").is_err());
}

#[test]
fn colours_true_colour_is_derived_and_tag_categories_are_kept() {
    let mut v: Json = serde_json::from_str(&fixture()).unwrap();
    v["coiCategories"] = serde_json::json!([{
        "id": "owner", "label": "Faction", "color": "#aa3300", "textColor": "#ffffff",
        "single": true, "enabled": true, "required": true,
        "tags": [{"key": "owner/empire", "label": "Empire"}]
    }]);
    let earth = v["systems"][0]["system"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["name"] == "Earth")
        .unwrap();
    earth["apparentColorHex"] = "#4a6fa5".into();
    let im = import_json(&v.to_string(), "v3-synthetic-sol").unwrap();
    let r = &im.report;
    // The GM's tag categories, with their colours, arrive as authored campaign facts.
    let cats = fact(&im, "u:v3-synthetic-sol", "v3.coiCategories");
    assert_eq!(cats.evidence, Evidence::Authored);
    assert_eq!(cats.value[0]["color"], "#aa3300");
    assert_eq!(cats.value[0]["textColor"], "#ffffff");
    assert!(!r.unclassified.contains_key("coiCategories"));
    // V3's true colour is derived: V4 recomputes it, so it is not a fact and not presentation.
    let e: Address = "u:v3-synthetic-sol/sys:@1/body:@2".parse().unwrap();
    assert!(!im.facts[&e].contains_key("v3.apparentColorHex"));
    assert!(!r.presentation.contains_key("apparentColorHex"));
    assert_eq!(r.derived_not_compared.get("apparentColorHex"), Some(&1));
}
