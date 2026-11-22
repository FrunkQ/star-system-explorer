//! MP17a: the sheet as the Look up block reads it.

use super::lookup::unit_dimension;
use super::*;
use sse_core::Dimension;
use sse_flow::sheet::Sheet;

fn base() -> Reference {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packs/base-reference/pack.json"
    ))
    .unwrap();
    let v: J = serde_json::from_str(&s).unwrap();
    Reference::from_json(&[(
        "sse.base-reference".into(),
        v["provides"]["reference"].clone(),
    )])
    .expect("the base sheet loads")
}

#[test]
fn units_as_the_sheet_writes_them() {
    assert_eq!(unit_dimension("kg m-3").unwrap(), Dimension::DENSITY);
    assert_eq!(
        unit_dimension("m3 kg-1 s-2").unwrap(),
        Dimension([3, -1, -2, 0, 0, 0, 0])
    );
    assert_eq!(
        unit_dimension("J kg-1").unwrap(),
        Dimension([2, 0, -2, 0, 0, 0, 0])
    );
    assert_eq!(
        unit_dimension("S m-1").unwrap(),
        Dimension([-3, -1, 3, 2, 0, 0, 0])
    );
    assert_eq!(unit_dimension("1").unwrap(), Dimension::NONE);
    assert_eq!(unit_dimension("").unwrap(), Dimension::NONE);
    assert!(unit_dimension("furlong").is_err());
}

#[test]
fn rows_are_named_once_or_by_their_table() {
    let r = base();
    assert_eq!(r.row_of("h2o").unwrap(), "substances/h2o");
    assert_eq!(r.row_of("constants/G").unwrap(), "constants/G");
    // Planck's constant and hydrogen share the id h.
    let e = r.row_of("h").unwrap_err();
    assert!(e.contains("constants/h"), "{e}");
    assert_eq!(r.row_of("elements/h").unwrap(), "elements/h");
    assert!(r.row_of("unobtainium").is_err());
    assert!(r.row_of("spells/h2o").is_err());
}

#[test]
fn a_constant_reads_with_its_source() {
    let r = base();
    let g = r.look_up("constants/G", "", None).unwrap();
    assert!(g.known);
    assert_eq!(g.value, sse_core::quantity::consts::G);
    assert_eq!(g.unit, "m3 kg-1 s-2");
    assert!(g.source.is_some());
    assert_eq!(
        r.dimension(Some("constants/G"), "").unwrap(),
        Dimension([3, -1, -2, 0, 0, 0, 0])
    );
    let c = r.look_up("constants/c", "value", None).unwrap();
    assert_eq!(c.grade.as_deref(), Some("convention"));
}

#[test]
fn water_reads_by_phase_and_by_name() {
    let r = base();
    // The typed copy this replaces: 2.468e6 J/kg (NIST, at 288 K).
    let l = r.look_up("h2o", "latent_vaporisation", None).unwrap();
    assert!(l.known);
    assert_eq!(l.value, 2.468e6);
    assert_eq!(l.property, "phases.latent_vaporisation");
    assert_eq!(l.source.as_ref().unwrap()["id"], "nist-webbook");
    // Ice VI's own density at 1 GPa and 270 K, with its grade and source.
    let d = r.look_up("h2o", "density", Some((1.0e9, 270.0))).unwrap();
    assert_eq!(d.phase.as_deref(), Some("h2o.ice_vi"));
    assert_eq!(d.value, 1310.0);
    assert_eq!(d.grade.as_deref(), Some("measured"));
    assert!(d.in_range);
    assert!(d.source.as_ref().unwrap().get("text").is_some());
    // The molar mass is worked out, never typed.
    let mm = r.look_up("h2o", "molar_mass", None).unwrap();
    assert!((mm.value - 0.018015).abs() < 1e-5, "{}", mm.value);
    assert_eq!(
        r.dimension(Some("h2o"), "density").unwrap(),
        Dimension::DENSITY
    );
}

#[test]
fn a_missing_property_is_an_answer_and_a_fit_needs_a_temperature() {
    let r = base();
    let y = r.look_up("h2o", "yield_strength", None).unwrap();
    assert!(!y.known);
    assert_eq!(y.value, 0.0);
    assert!(y.reason.unwrap().contains("no yield_strength"));
    // The unit still comes from a row that has it, so the block loads.
    assert_eq!(
        r.dimension(Some("h2o"), "yield_strength").unwrap(),
        Dimension::PRESSURE
    );
    assert!(r.dimension(None, "charisma").is_err());
    // Water ice's sublimation line is a fit: no temperature, no value.
    let v = r.look_up("h2o", "vapour_pressure", None).unwrap();
    assert!(!v.known, "{v:?}");
    let v = r
        .look_up("h2o", "vapour_pressure", Some((1.0, 200.0)))
        .unwrap();
    assert!(v.known);
    assert!(
        (0.1..0.2).contains(&v.value),
        "ice at 200 K: about 0.16 Pa, got {}",
        v.value
    );
    assert!(r.look_up("h2o", "density", Some((-1.0, 200.0))).is_err());
}
