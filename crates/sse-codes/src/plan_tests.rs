use crate::plan::*;
use crate::{Codes, Subject};
use sse_core::cbor::Value;
use sse_pack::PackManifest;
use std::collections::BTreeMap;

fn codes() -> Codes {
    let p = |f: &str| {
        PackManifest::from_json(
            &std::fs::read_to_string(format!("{}/../../{f}", env!("CARGO_MANIFEST_DIR"))).unwrap(),
        )
        .unwrap()
    };
    Codes::load(&[
        p("packs/base-atlas/pack.json"),
        p("packs/traveller/pack.json"),
    ])
    .unwrap()
}

fn plan_for(c: &Codes, uwp: &str, pbg: &str, stars: &[&str]) -> Plan {
    let sheet = c.sheet("traveller.world").unwrap();
    plan(
        c,
        sheet,
        &[("uwp", uwp), ("pbg", pbg)],
        stars,
        &BTreeMap::new(),
        "test:0101",
    )
    .unwrap()
}

fn facts(p: &Plan) -> BTreeMap<String, Value> {
    p.edits
        .iter()
        .map(|e| (e.field.clone(), e.value.clone()))
        .collect()
}

#[test]
fn stars_read_from_stellar_data() {
    let g2 = star("G2 V").unwrap();
    assert!((g2.mass - 1.016).abs() < 1e-9, "{}", g2.mass);
    assert!(g2.note.is_none());
    assert!((star("M0 V").unwrap().mass - 0.57).abs() < 1e-12);
    assert!(star("K2 III").unwrap().note.unwrap().contains("III"));
    assert!(star("D").unwrap().note.is_some());
    assert!(star("Q7 V").is_none());
    // Earth's orbit for the Sun: albedo 0.3 gives 255 K at 1 au.
    assert!((orbit_for(254.9, 1.0) - 1.0).abs() < 0.01);
}

#[test]
fn an_earthlike_code_is_placed_like_earth_with_no_marks() {
    let c = codes();
    let p = plan_for(&c, "A867975-9", "814", &["G2 V"]);
    assert!(p.objections.is_empty(), "{:?}", p.objections);
    assert!(p.pin.a_au > 0.9 && p.pin.a_au < 1.15, "{}", p.pin.a_au);
    assert!(
        p.pin.mass_earth > 0.9 && p.pin.mass_earth < 1.1,
        "{}",
        p.pin.mass_earth
    );
    assert_eq!((p.want_giants, p.want_belts), (Some(4), Some(1)));
    assert!(p.edits.iter().all(|e| !e.impossible));
    let tag = p
        .edits
        .iter()
        .find(|e| e.field == "provenance.generator")
        .unwrap();
    assert_eq!(tag.value, Value::Text("traveller-generated".into()));
    // The facts written read back to the same codes, population multiplier too.
    let sheet = c.sheet("traveller.world").unwrap();
    let r = c.read(sheet, &Subject::new(facts(&p)), Some(&[]));
    assert_eq!(r.line("uwp"), Some("A867975-9"));
    assert_eq!(&r.line("pbg").unwrap()[..1], "8");
}

#[test]
fn impossible_codes_are_kept_and_marked_never_altered() {
    let c = codes();
    // Size 2 (3,200 km) with standard air.
    let p = plan_for(&c, "X266000-0", "000", &["G2 V"]);
    assert_eq!(p.objections.len(), 1);
    assert!(
        p.objections[0].reason.contains("Jeans parameter"),
        "{}",
        p.objections[0].reason
    );
    let pr = p
        .edits
        .iter()
        .find(|e| e.field == "atmosphere.surface_pressure")
        .unwrap();
    assert!(pr.impossible);
    assert_eq!(
        pr.value,
        Value::Float(1.0e5),
        "the code wins: 1 bar is kept"
    );
    let sheet = c.sheet("traveller.world").unwrap();
    assert_eq!(
        c.read(sheet, &Subject::new(facts(&p)), None).line("uwp"),
        Some("X266000-0")
    );
    // Liquid under vacuum.
    let v = plan_for(&c, "X507000-0", "000", &["G2 V"]);
    assert!(v
        .objections
        .iter()
        .any(|o| o.fields == vec!["surface.liquid_fraction".to_string()]));
    // And a sound world has none.
    assert!(plan_for(&c, "X766000-0", "000", &["G2 V"])
        .objections
        .is_empty());
}

#[test]
fn make_physically_sound_proposes_the_smallest_change() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let (line, what) = sound(&c, sheet, "uwp", "X266000-0").unwrap().unwrap();
    let diff = line
        .chars()
        .zip("X266000-0".chars())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(diff, 1, "{line}");
    assert!(plan_for(&c, &line, "000", &["G2 V"]).objections.is_empty());
    assert!(what.contains(" to "), "{what}");
    assert_eq!(
        sound(&c, sheet, "uwp", "A867975-9").unwrap().unwrap().0,
        "A867975-9"
    );
}

#[test]
fn growing_honours_the_main_world_and_is_reproducible() {
    let c = codes();
    let p = plan_for(&c, "A867975-9", "812", &["G2 V"]);
    let sys: sse_core::Address = "u:traveller/sys:0".parse().unwrap();
    let a = grow(p.clone(), sys.clone(), 12, 2).unwrap();
    let b = grow(p, sys, 12, 2).unwrap();
    assert_eq!(
        serde_json::to_string(&a.run).unwrap(),
        serde_json::to_string(&b.run).unwrap()
    );
    assert_eq!(a.main_world, b.main_world);
    let w = a
        .run
        .protoplanets
        .iter()
        .find(|w| w.address == a.main_world)
        .unwrap();
    assert!((w.a_au - a.plan.pin.a_au).abs() < 1e-9);
    // Counts: met, or said plainly.
    if a.giants != 2 {
        assert!(
            a.unmet.iter().any(|u| u.contains("gas giants")),
            "{:?}",
            a.unmet
        );
    }
    if a.belts != 1 {
        assert!(a.unmet.iter().any(|u| u.contains("belts")), "{:?}", a.unmet);
    }
    eprintln!(
        "grown: attempt {}, giants {}, belts {}, unmet {:?}",
        a.attempt, a.giants, a.belts, a.unmet
    );
}
