use super::*;
use sse_core::cbor::Value;
use std::collections::BTreeMap;

fn root(p: &str) -> String {
    format!("{}/../../{p}", env!("CARGO_MANIFEST_DIR"))
}

fn pack(p: &str) -> PackManifest {
    PackManifest::from_json(&std::fs::read_to_string(root(p)).unwrap()).unwrap()
}

fn codes() -> Codes {
    Codes::load(&[
        pack("packs/base-atlas/pack.json"),
        pack("packs/traveller/pack.json"),
    ])
    .unwrap()
}

fn json_facts(v: &serde_json::Value) -> BTreeMap<String, Value> {
    serde_json::from_value(v.clone()).unwrap()
}

/// The Solar System fixture bodies as subjects, by name.
fn solar() -> Vec<(String, Subject)> {
    let d: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root("tests/fixtures/classify/solar-system.json")).unwrap(),
    )
    .unwrap();
    d["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            let mut s = Subject::new(json_facts(&b["facts"]));
            s.address = Some(b["address"].as_str().unwrap().parse().unwrap());
            (b["name"].as_str().unwrap().to_string(), s)
        })
        .collect()
}

fn body(name: &str) -> Subject {
    solar().into_iter().find(|(n, _)| n == name).unwrap().1
}

fn uwp(c: &Codes, s: &Subject) -> String {
    let sheet = c.sheet("traveller.world").unwrap();
    c.read(sheet, s, None).line("uwp").unwrap().to_string()
}

#[test]
fn pack_loads_beside_the_base_atlas_and_every_scheme_is_clean() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    c.check(sheet).unwrap();
    for g in &sheet.segments {
        if let Source::Classify { scheme, .. } = &g.source {
            assert_eq!(
                c.sift.atlas.overlaps(scheme),
                Vec::<(String, String)>::new(),
                "{scheme}"
            );
        }
    }
    // Stacked twice is a readable error, never a silent override.
    let twice = Codes::load(&[
        pack("packs/traveller/pack.json"),
        pack("packs/traveller/pack.json"),
    ]);
    assert!(twice.is_err());
}

#[test]
fn solar_system_bodies_read_as_expected() {
    let c = codes();
    let want = [
        ("Mercury", "X300000-0"),
        ("Venus", "X8C0000-0"),
        ("Earth", "X867900-0"),
        ("Moon", "X200000-0"),
        ("Mars", "X410000-0"),
        ("Ceres", "X100000-0"),
        ("Jupiter", "XFA0000-0"),
        ("Titan", "X3A0000-0"),
        ("Io", "X200000-0"),
        ("Pluto", "X100000-0"),
    ];
    for (n, w) in want {
        assert_eq!(uwp(&c, &body(n)), w, "{n}");
    }
}

#[test]
fn every_digit_answers_why() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let r = c.read(sheet, &body("Earth"), None);
    let atm = r.segment("atmosphere").unwrap();
    let Why::Classified { label, defaulted } = &atm.why else {
        panic!("{:?}", atm.why)
    };
    assert_eq!(label.base.as_deref(), Some("traveller.atmosphere/06"));
    // Only the authored-kind axis stood in ("no unusual kind set").
    assert_eq!(defaulted, &vec!["atmosphere.traveller_kind".to_string()]);
    // The oxygen flag it read was set by a rule, whose evidence is the air itself.
    let ox = r
        .prepared
        .iter()
        .find(|t| t.tag == "traveller.prep.oxygen")
        .unwrap();
    assert!(ox.evidence.iter().any(|e| e.field == "atmosphere.pp.o2"));
    // The social digits were absent and read as 0: the reading says so.
    let gov = r.segment("government").unwrap();
    let Why::Classified { defaulted, .. } = &gov.why else {
        panic!()
    };
    assert_eq!(defaulted, &vec!["society.government".to_string()]);
    // Each segment is classified (no unknown marks) on a full world.
    for s in r
        .segments
        .iter()
        .filter(|s| !matches!(s.why, Why::NoSystem { .. }))
    {
        assert_ne!(s.symbol, UNKNOWN, "{}", s.id);
    }
    // Near misses come with the change that would reach them.
    assert!(!label.nearest.is_empty());
}

fn earth_with_society() -> Subject {
    let mut s = body("Earth");
    for (k, v) in [
        ("society.government", Value::Int(7)),
        ("society.law_level", Value::Int(5)),
        ("society.tech_level", Value::Int(9)),
        ("society.starport", Value::Text("A".into())),
    ] {
        s.facts.insert(k.into(), v);
    }
    s
}

#[test]
fn trade_codes_and_hints_come_from_the_digits() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let r = c.read(sheet, &earth_with_society(), None);
    assert_eq!(r.line("uwp"), Some("A867975-9"));
    assert_eq!(r.group("trade").unwrap().codes(), vec!["Ga", "Hi", "Pr"]);
    // Government 7 (balkanised) hints at an amber zone; the zone itself is the referee's.
    assert_eq!(r.group("zone_hint").unwrap().codes(), vec!["amber"]);
    assert!(r.group("zone").unwrap().codes().is_empty());
    // The trade code's evidence is the digits, and the digits' own why is their label.
    let ga = &r.group("trade").unwrap().tags[0].1;
    let fields: Vec<&str> = ga.evidence.iter().map(|e| e.field.as_str()).collect();
    assert_eq!(
        fields,
        vec![
            "traveller.uwp.atmosphere",
            "traveller.uwp.hydrographics",
            "traveller.uwp.size"
        ]
    );
    // Physics-only codes: Mercury is hot and airless; the Moon is a satellite.
    let m = c.read(sheet, &body("Mercury"), None);
    assert_eq!(m.group("trade").unwrap().codes(), vec!["Ba", "Ho", "Va"]);
    let moon = c.read(sheet, &body("Moon"), None);
    assert_eq!(moon.group("trade").unwrap().codes(), vec!["Ba", "Sa", "Va"]);
    let venus = c.read(sheet, &body("Venus"), None);
    // Insidious air and no water: a hellworld.
    assert!(venus.group("trade").unwrap().codes().contains(&"He"));
}

#[test]
fn pbg_counts_the_system() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let mut sys: Vec<Subject> = solar().into_iter().map(|(_, s)| s).collect();
    let belt = Subject {
        address: Some("u:sol/sys:0/belt:0".parse().unwrap()),
        ..Default::default()
    };
    sys.push(belt.clone());
    let r = c.read(sheet, &body("Earth"), Some(&sys));
    // 8.1 billion: multiplier 8; one belt; Jupiter, Saturn, Uranus and Neptune.
    assert_eq!(r.line("pbg"), Some("814"));
    let Why::Counted { members, .. } = &r.segment("gas_giants").unwrap().why else {
        panic!()
    };
    assert_eq!(members.len(), 4);
    // A belt as the main world reads as size 0, with the asteroid code.
    let b = c.read(sheet, &belt, Some(&sys));
    assert_eq!(b.line("uwp"), Some("X000000-0"));
    assert!(b.group("trade").unwrap().codes().contains(&"As"));
}

#[test]
fn authored_codes_win_and_keep_the_physics_beside_them() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let mut s = earth_with_society();
    s.facts
        .insert("traveller.code.size".into(), Value::Text("3".into()));
    s.facts
        .insert("traveller.zone".into(), Value::Text("A".into()));
    let r = c.read(sheet, &s, None);
    assert_eq!(r.line("uwp"), Some("A367975-9"));
    let size = r.segment("size").unwrap();
    assert!(size.disagrees());
    assert_eq!(
        size.why,
        Why::Authored {
            field: "traveller.code.size".into(),
            physics: Some("8".into())
        }
    );
    // The groups read the code as it stands: size 3 rules out Garden.
    assert!(!r.group("trade").unwrap().codes().contains(&"Ga"));
    assert_eq!(r.group("zone").unwrap().codes(), vec!["A"]);
}

/// Reference profiles: classic examples from published subsectors' public
/// data lines (codes only), and corners of every table.
const REFERENCE: &[&str] = &[
    "A867975-9",
    "A788899-C",
    "B565776-A",
    "C4207B9-A",
    "E5A0000-5",
    "X000000-0",
    "A9D7AA9-F",
    "B300588-C",
    "C7C9547-8",
    "D200000-0",
    "E8B5634-7",
    "A6A2400-D",
    "BAE6362-8",
    "C2008A8-C",
    "X420000-0",
    "B8C4733-9",
    "A5146B8-B",
    "CFA0644-9",
    "A000989-E",
    "EA7A553-6",
    "C111111-1",
    "B9E5869-A",
    "AAAA5FJ-Z",
    "BF87745-C",
    "C344300-7",
    "D7B1588-7",
];

#[test]
fn codes_round_trip_through_a_world() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let base = body("Earth").facts;
    for line in REFERENCE {
        let p = pins(&c, sheet, "uwp", line).unwrap();
        assert!(p.iter().all(|s| !s.unknown), "{line}");
        let world = Subject::new(typical_world(sheet, &p, &base));
        assert_eq!(&uwp(&c, &world), line, "{line}");
    }
    // Every symbol of every classified segment, one at a time, over a base code.
    for g in &sheet.segments {
        let Source::Classify { symbols, .. } = &g.source else {
            continue;
        };
        for y in symbols {
            let mut cur = parse(sheet, "uwp", "A867975-9").unwrap();
            for (seg, sym) in cur.iter_mut() {
                if *seg == g.id {
                    *sym = y.symbol.clone();
                }
            }
            let line: String = {
                let segs: Vec<SegmentReading> = cur
                    .iter()
                    .map(|(id, s)| SegmentReading {
                        id: id.clone(),
                        symbol: s.clone(),
                        value: None,
                        why: Why::NoSystem { tag: String::new() },
                    })
                    .collect();
                fill(&sheet.layouts["uwp"], &segs)
            };
            let p = pins(&c, sheet, "uwp", &line).unwrap();
            let world = Subject::new(typical_world(sheet, &p, &base));
            assert_eq!(uwp(&c, &world), line, "{} {}", g.id, y.symbol);
        }
    }
}

#[test]
fn worlds_round_trip_through_their_code() {
    // World to profile to world keeps every digit.
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let mut worlds: Vec<Subject> = solar().into_iter().map(|(_, s)| s).collect();
    worlds.push(earth_with_society());
    for w in worlds {
        let a = uwp(&c, &w);
        let p = pins(&c, sheet, "uwp", &a).unwrap();
        let back = Subject::new(typical_world(sheet, &p, &BTreeMap::new()));
        assert_eq!(uwp(&c, &back), a);
    }
    // The population multiplier comes back too.
    let r = c.read(sheet, &body("Earth"), Some(&[]));
    let mut segs = parse(sheet, "uwp", r.line("uwp").unwrap()).unwrap();
    segs.extend(parse(sheet, "pbg", r.line("pbg").unwrap()).unwrap());
    let line = r.line("uwp").unwrap();
    let mut p = pins(&c, sheet, "uwp", line).unwrap();
    p.extend(pins(&c, sheet, "pbg", r.line("pbg").unwrap()).unwrap());
    let back = Subject::new(typical_world(sheet, &p, &BTreeMap::new()));
    assert_eq!(back.num("settlements.population"), Some(8e9));
    assert_eq!(c.read(sheet, &back, Some(&[])).line("pbg"), Some("800"));
}

#[test]
fn pins_carry_ranges_in_fact_units() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let p = pins(&c, sheet, "uwp", "A867975-9").unwrap();
    let size = p.iter().find(|s| s.segment == "size").unwrap();
    let Constraint::Range { field, min, max } = &size.classes[0].constraints[0] else {
        panic!()
    };
    assert_eq!(field, "body.radius");
    // Size 8: 12,000 to 13,600 km across, so radius 6,000 to 6,800 km.
    assert!(
        (min - 6.0e6).abs() < 1.0 && (max - 6.8e6).abs() < 1.0,
        "{min} {max}"
    );
    assert!(pins(&c, sheet, "uwp", "A86797-9").is_err());
    assert!(pins(&c, sheet, "uwp", "A867975+9").is_err());
    let odd = pins(&c, sheet, "uwp", "A8?7975-9").unwrap();
    assert!(odd.iter().any(|s| s.unknown));
}

#[test]
fn ehex_skips_i_and_o() {
    assert_eq!(ehex(17), Some('H'));
    assert_eq!(ehex(18), Some('J'));
    assert_eq!(ehex(22), Some('N'));
    assert_eq!(ehex(23), Some('P'));
    assert_eq!(ehex(33), Some('Z'));
    assert_eq!(ehex(34), None);
    assert_eq!(ehex_value('j'), Some(18));
    assert_eq!(ehex_value('I'), None);
    assert_eq!(leading(8.1e9, 9), 8);
    assert_eq!(leading(0.0, 0), 0);
    assert_eq!(leading(1e15, 15), 1);
}

#[test]
fn golden() {
    let c = codes();
    let sheet = c.sheet("traveller.world").unwrap();
    let sys: Vec<Subject> = solar().into_iter().map(|(_, s)| s).collect();
    let mut out = serde_json::Map::new();
    out.insert(
        "pack".into(),
        serde_json::json!(pack("packs/traveller/pack.json").content_hash().to_string()),
    );
    for (n, s) in solar() {
        let r = c.read(sheet, &s, Some(&sys));
        let trade = r.group("trade").unwrap().codes().join(" ");
        out.insert(n, serde_json::json!([r.line("uwp"), r.line("pbg"), trade]));
    }
    // X3b: one coded world grown into a system (same seed, same system on every device).
    let p = crate::plan::plan(
        &c,
        sheet,
        &[("uwp", "A867975-9"), ("pbg", "812")],
        &["G2 V"],
        &BTreeMap::new(),
        "golden:0101",
    )
    .unwrap();
    let g = crate::plan::grow(p, "u:traveller/sys:0".parse().unwrap(), 12, 2).unwrap();
    out.insert(
        "grown".into(),
        serde_json::json!({
            "attempt": g.attempt, "giants": g.giants, "belts": g.belts,
            "main_world": g.main_world.to_string(), "worlds": g.run.worlds.len(),
            "history": g.run.history.hash().hex(),
        }),
    );
    let got = serde_json::Value::Object(out);
    let path = root("tests/golden/traveller-codes.json");
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing {path}; run with SSE_BLESS=1"));
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden traveller-codes changed"
    );
}
