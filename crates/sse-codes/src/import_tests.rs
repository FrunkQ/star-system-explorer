use crate::import::*;
use crate::{Codes, Subject};
use sse_core::cbor::Value;
use sse_pack::PackManifest;
use std::collections::BTreeMap;

fn root(f: &str) -> String {
    format!("{}/../../{f}", env!("CARGO_MANIFEST_DIR"))
}

fn codes() -> Codes {
    let p = |f: &str| PackManifest::from_json(&std::fs::read_to_string(root(f)).unwrap()).unwrap();
    Codes::load(&[
        p("packs/base-atlas/pack.json"),
        p("packs/traveller/pack.json"),
    ])
    .unwrap()
}

fn lanternfall(file: &str) -> Import {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let text = std::fs::read_to_string(root(&format!("tests/fixtures/sector/{file}"))).unwrap();
    let f = sse_sector_io::parse_any(&text);
    import(&c, sheet, &f, &"u:kestrel".parse().unwrap()).unwrap()
}

#[test]
fn a_reference_subsector_imports_with_every_impossible_value_marked() {
    let i = lanternfall("lanternfall.tab");
    eprintln!("{}", i.summary.sentence());
    assert_eq!(i.summary.lines, 12);
    assert_eq!(
        i.summary.imported,
        12,
        "{:?}",
        i.worlds
            .iter()
            .filter_map(|w| w.refused.clone())
            .collect::<Vec<_>>()
    );
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    for w in &i.worlds {
        let p = w.plan.as_ref().unwrap();
        // Never altered: the facts written read back to the listed UWP.
        let facts: BTreeMap<String, Value> = w
            .world_edits
            .iter()
            .map(|e| (e.field.clone(), e.value.clone()))
            .collect();
        let uwp = w.raw.split('\t').nth(4).unwrap();
        let back = c.read(sheet, &Subject::new(facts), None);
        assert_eq!(back.line("uwp"), Some(uwp), "{}", w.name);
        // Impossible values are kept and marked, each with its reason.
        for o in &p.objections {
            for f in &o.fields {
                let e = w.world_edits.iter().find(|e| &e.field == f).unwrap();
                assert!(e.impossible && e.reason.is_some(), "{} {f}", w.name);
            }
        }
        assert!(w.system.starts_with("u:kestrel/sys:"), "{}", w.system);
        assert!(w.system_edits.iter().any(|e| e.field == "map.hex"));
        let z = w
            .system_edits
            .iter()
            .find(|e| e.field == "map.z_pc")
            .unwrap();
        assert!(matches!(z.value, Value::Float(v) if v.abs() <= 0.25));
    }
    // Neighbouring hexes sit about a parsec apart.
    let pos = |name: &str| {
        let w = i.worlds.iter().find(|w| w.name == name).unwrap();
        let f = |k: &str| match w.system_edits.iter().find(|e| e.field == k).unwrap().value {
            Value::Float(v) => v,
            _ => panic!(),
        };
        (f("map.x_pc"), f("map.y_pc"))
    };
    // Ashgrove 0912 and Jessamy's Rest 1011: one hex apart.
    let (a, b) = (pos("Ashgrove"), pos("Jessamy's Rest"));
    let d = sse_core::math::sqrt((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1));
    assert!((d - 1.0).abs() < 1e-9, "{d}");
    // The zone is the referee's call and reads back as written.
    let cinder = i.worlds.iter().find(|w| w.name == "Cinder Gate").unwrap();
    let z = cinder
        .world_edits
        .iter()
        .find(|e| e.field == "traveller.zone")
        .unwrap();
    assert_eq!(z.value, Value::Text("A".into()));
    // Bases as flags.
    let ash = i.worlds.iter().find(|w| w.name == "Ashgrove").unwrap();
    assert!(ash
        .world_edits
        .iter()
        .any(|e| e.field == "traveller.bases.N"));
    assert!(ash
        .world_edits
        .iter()
        .any(|e| e.field == "traveller.bases.S"));
    // A brown dwarf primary and a giant are noted.
    let else_ = i.worlds.iter().find(|w| w.name == "Elsewhen").unwrap();
    assert!(!else_.guessed.is_empty(), "{:?}", else_.guessed);
    eprintln!(
        "{}",
        serde_json::to_string_pretty(
            &i.worlds
                .iter()
                .map(|w| (
                    w.name.clone(),
                    w.guessed.clone(),
                    w.differs.clone(),
                    w.plan.as_ref().map(|p| p
                        .objections
                        .iter()
                        .map(|o| o.reason.clone())
                        .collect::<Vec<_>>())
                ))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
}

#[test]
fn every_format_imports_the_same_worlds() {
    let tab = lanternfall("lanternfall.tab");
    for f in ["lanternfall.sec", "lanternfall.txt"] {
        let other = lanternfall(f);
        assert_eq!(other.summary.imported, tab.summary.imported, "{f}");
        for (a, b) in tab.worlds.iter().zip(&other.worlds) {
            assert_eq!(a.system, b.system, "{f}");
            let ea: Vec<_> = a
                .world_edits
                .iter()
                .filter(|e| !e.field.starts_with("provenance"))
                .collect();
            let eb: Vec<_> = b
                .world_edits
                .iter()
                .filter(|e| !e.field.starts_with("provenance"))
                .collect();
            assert_eq!(ea, eb, "{f} {}", a.name);
        }
    }
}

#[test]
fn messy_pastes_and_unreadable_lines_say_what_happened() {
    let i = lanternfall("lanternfall-messy.txt");
    assert!(!i.warnings.is_empty());
    assert!(i.summary.imported >= 11, "{}", i.summary.sentence());
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let f = sse_sector_io::parse_uwp_lines(
        "0101 Nowhere X?00000-0 000\n0102 Odd A867975-9 ??? G2 V\n0103 Thinair X266000-0 000 G2 V",
    );
    let i = import(&c, sheet, &f, &"u:k".parse().unwrap()).unwrap();
    assert_eq!(
        i.worlds[0].refused.as_deref(),
        Some("no readable code line")
    );
    assert!(i.worlds[0].guessed[0].contains("unknown digits"));
    assert!(i.worlds[1].plan.is_some());
    assert!(
        i.worlds[1].guessed.iter().any(|g| g.contains("pbg")),
        "{:?}",
        i.worlds[1].guessed
    );
    // Size 2 with standard air: kept, marked impossible, with the reason.
    let thin = &i.worlds[2];
    let p = thin
        .world_edits
        .iter()
        .find(|e| e.field == "atmosphere.surface_pressure")
        .unwrap();
    assert!(p.impossible && p.reason.as_ref().unwrap().contains("Jeans"));
    assert_eq!(i.summary.impossible, 1);
    assert_eq!(i.summary.refused, 1);
}
