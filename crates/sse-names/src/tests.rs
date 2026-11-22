//! NM1: the naming engine (naming design section 7, NM1 acceptance).

use crate::make::{name, name_with, siblings, Request};
use crate::scheme::{Library, Scheme};
use crate::text::{check, compose, fold, near};
use serde_json::{json, Value as J};
use sse_core::{Address, ContentHash, Seed};
use std::collections::{BTreeMap, BTreeSet};

/// Vesh, from the design thread's name sheet (NM3d), written as JSON.
pub fn vesh() -> J {
    json!({
        "id": "vesh", "label": "Vesh", "kind": "invented", "script": "latin",
        "about": "A hive-building people's tongue. Guttural, with one click and doubled stops.",
        "sounds": {
            "classes": {
                "C": [["k", 5], ["kh", 4], ["r", 4], ["z", 4], ["g", 3], ["q", 3], ["gh", 2], ["tk", 2], ["x", 1]],
                "V": [["a", 6], ["i", 3], ["u", 3], ["aa", 2]],
                "E": [["k", 4], ["z", 3], ["kh", 3], ["q", 2], ["rr", 2]]
            },
            "shapes": [["CV", 4], ["CVE", 5], ["VE", 1]],
            "length": [2, 3],
            "never": [["aa", "a"], ["a", "aa"], ["x", "x"], ["k", "k"], ["q", "q"], ["rr", "r"], ["kh", "kh"]],
            "stress": "first",
            "say": {"kh": "ch as in loch", "x": "a sharp tongue click", "rr": "a rolled r"},
            "ipa": {"kh": "x", "g": "ɡ", "gh": "ɣ", "r": "ɾ", "rr": "r", "tk": "t͡k", "x": "ǂ", "aa": "aː"}
        },
        "means": {"red": "khaz", "mountain": "rak", "home": "uq", "sky": "xaak", "great": "gaarr", "sea": "zuq"},
        "forms": {
            "planet": [["{word}", 1]],
            "peak": [["{means:colour?}{means:mountain}", 1]],
            "ocean": [["{means:great}{means:sea}", 1]],
            "port": [["{means:sky}{means:home} {word}", 1]],
            "default": [["{word}", 1]]
        }
    })
}

fn catalogue() -> J {
    json!({
        "id": "catalogue", "label": "Catalogue", "kind": "catalogue",
        "forms": {
            "star": [["KS {serial:4}", 1]],
            "planet": [["{parent} {roman}", 1]],
            "moon": [["{parent} {letter}", 1]],
            "galaxy": [["{coord}", 1]],
            "default": [["{parent} {n}", 1]]
        },
        "follow": ["planet", "moon"]
    })
}

fn lib(schemes: Vec<J>) -> Library {
    Library::load(&[("test.names".into(), json!({"schemes": schemes}))]).unwrap()
}

fn addr(s: &str) -> Address {
    s.parse().unwrap()
}

fn req<'a>(
    seed: &'a Seed,
    a: &'a Address,
    scheme: &'a str,
    kind: &'a str,
    facts: &'a BTreeMap<String, String>,
) -> Request<'a> {
    Request {
        seed,
        address: a,
        layer: "common",
        scheme,
        kind,
        index: 0,
        serial: None,
        parent: None,
        system: None,
        facts,
    }
}

fn digest(lib: &Library, scheme: &str, n: u64) -> String {
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    let mut all = String::new();
    for i in 0..n {
        let a = addr(&format!("u:t/sys:{}/body:{}", i / 10, i % 10));
        let r = req(&seed, &a, scheme, "planet", &none);
        all.push_str(&name(lib, &r).unwrap().show(None, None));
        all.push('\n');
    }
    ContentHash::of_bytes(all.as_bytes()).hex()
}

#[test]
fn a_name_is_a_keyed_draw_and_nothing_else() {
    let l = lib(vec![vesh(), catalogue()]);
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    let a = addr("u:t/sys:4/body:2");
    let n1 = name(&l, &req(&seed, &a, "vesh", "planet", &none)).unwrap();
    let n2 = name(&l, &req(&seed, &a, "vesh", "planet", &none)).unwrap();
    assert_eq!(n1, n2);
    // Another address, another seed or another layer: another draw.
    let b = addr("u:t/sys:4/body:3");
    assert_ne!(
        n1.form,
        name(&l, &req(&seed, &b, "vesh", "planet", &none))
            .unwrap()
            .form
    );
    let other = Seed::from_text("other");
    assert_ne!(
        n1.form,
        name(&l, &req(&other, &a, "vesh", "planet", &none))
            .unwrap()
            .form
    );
    // 10,000 names are the same with an unrelated scheme stacked beside,
    // and change when the scheme itself changes.
    let d = digest(&l, "vesh", 10_000);
    let mut more = vec![vesh(), catalogue()];
    more.push(json!({"id": "other", "label": "Other", "forms": {"default": [["Ka", 1]]}}));
    assert_eq!(digest(&lib(more), "vesh", 10_000), d);
    let mut v = vesh();
    v["about"] = json!("Edited.");
    assert_ne!(digest(&lib(vec![v]), "vesh", 200), digest(&l, "vesh", 200));
}

#[test]
fn vesh_names_read_with_their_faces_and_meanings() {
    let l = lib(vec![vesh()]);
    let seed = Seed::from_text("nm1");
    let facts: BTreeMap<String, String> = [("colour".to_string(), "red".to_string())].into();
    let a = addr("u:t/sys:0/body:1/peak:3");
    let n = name(&l, &req(&seed, &a, "vesh", "peak", &facts)).unwrap();
    assert_eq!(n.show(None, None), "Khazrak");
    assert_eq!(n.say, "KHAZ-rak");
    assert_eq!(n.ipa, "/ˈxaz.ɾak/");
    let means: Vec<_> = n.parts.iter().map(|p| p.means.clone().unwrap()).collect();
    assert_eq!(means, vec!["red", "mountain"]);
    assert_eq!(n.hints.get("kh").map(String::as_str), Some("ch as in loch"));
    // Without a colour the optional part drops away.
    let none = BTreeMap::new();
    assert_eq!(
        name(&l, &req(&seed, &a, "vesh", "peak", &none))
            .unwrap()
            .show(None, None),
        "Rak"
    );
    // A port: a fixed form and a drawn word.
    let p = name(&l, &req(&seed, &a, "vesh", "port", &none)).unwrap();
    assert!(
        p.show(None, None).starts_with("Xaakuq "),
        "{}",
        p.show(None, None)
    );
    // Words never hold a forbidden sequence.
    for i in 0..2000 {
        let a = addr(&format!("u:t/sys:{i}/body:0"));
        let w = name(&l, &req(&seed, &a, "vesh", "planet", &none)).unwrap();
        let s = w.show(None, None).to_lowercase();
        assert!(!s.contains("xx"), "{s}");
        assert!(w.ipa.starts_with('/'));
    }
}

#[test]
fn names_follow_their_parent_and_typed_names_stay() {
    let l = lib(vec![catalogue()]);
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    let star = addr("u:t/sys:0/body:0");
    let mut r = req(&seed, &star, "catalogue", "star", &none);
    r.layer = "designation";
    r.serial = Some(102);
    let s = name(&l, &r).unwrap();
    assert_eq!(s.show(None, None), "KS 0102");
    assert!(s.numbered && !s.follows);
    let planet = addr("u:t/sys:0/body:0/body:1");
    let mut r = req(&seed, &planet, "catalogue", "planet", &none);
    r.index = 1;
    let p = name(&l, &r).unwrap();
    assert_eq!(p.form, "{parent} II");
    assert!(p.follows);
    assert_eq!(p.show(Some("HD102"), None), "HD102 II");
    // Rename the star: the planet follows, and its moon follows the planet.
    let moon = addr("u:t/sys:0/body:0/body:1/body:0");
    let m = name(&l, &req(&seed, &moon, "catalogue", "moon", &none)).unwrap();
    let pn = p.show(Some("Mystar"), None);
    assert_eq!(pn, "Mystar II");
    assert_eq!(m.show(Some(&pn), None), "Mystar II b");
    // A typed planet name is plain text; its moon follows the typed name.
    assert_eq!(m.show(Some("Haven"), None), "Haven b");
    // Numbers past the alphabet and big numerals.
    let mut r = req(&seed, &moon, "catalogue", "moon", &none);
    r.index = 25;
    assert_eq!(name(&l, &r).unwrap().show(Some("X"), None), "X aa");
    let mut r = req(&seed, &planet, "catalogue", "planet", &none);
    r.index = 13;
    assert_eq!(name(&l, &r).unwrap().show(Some("X"), None), "X XIV");
    // A coordinate designation from the sky position.
    let facts: BTreeMap<String, String> = [
        ("ra_deg".into(), "83.82".into()),
        ("dec_deg".into(), "-5.39".into()),
    ]
    .into();
    let g = addr("u:t/gal:3");
    assert_eq!(
        name(&l, &req(&seed, &g, "catalogue", "galaxy", &facts))
            .unwrap()
            .show(None, None),
        "J0535-0523"
    );
}

#[test]
fn siblings_never_share_a_name_or_sound_alike() {
    let l = lib(vec![vesh(), catalogue()]);
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    let addrs: Vec<Address> = (0..200)
        .map(|i| addr(&format!("u:t/sys:7/body:{i}")))
        .collect();
    let rs: Vec<Request> = addrs
        .iter()
        .map(|a| req(&seed, a, "vesh", "planet", &none))
        .collect();
    let out = siblings(&l, &rs);
    let names: Vec<String> = out
        .iter()
        .map(|n| fold(&n.as_ref().unwrap().show(None, None)))
        .collect();
    for (i, a) in names.iter().enumerate() {
        for b in &names[..i] {
            assert!(!near(a, b), "{a} and {b}");
        }
    }
    // The same set given in another order names each the same.
    let mut rev: Vec<Request> = rs.clone();
    rev.reverse();
    let back = siblings(&l, &rev);
    for (i, n) in back.iter().enumerate() {
        assert_eq!(n, &out[199 - i]);
    }
    // 200 designations are unique by construction.
    let rs: Vec<Request> = addrs
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let mut r = req(&seed, a, "catalogue", "planet", &none);
            r.index = i as u32;
            r.parent = Some("KS 0001");
            r
        })
        .collect();
    let set: BTreeSet<String> = siblings(&l, &rs)
        .into_iter()
        .map(|n| n.unwrap().show(Some("KS 0001"), None))
        .collect();
    assert_eq!(set.len(), 200);
}

#[test]
fn a_mix_holds_one_tradition_per_system_with_a_few_strays() {
    let a_list = json!({"id": "north", "label": "North", "lists": {"n": ["Ashford", "Kelmere", "Brookton", "Harrowick", "Lindmere", "Oakford"]}, "forms": {"default": [["{list:n}", 1]]}});
    let mix = json!({"id": "nations", "label": "Many nations", "mix": {"of": [["north", 1], ["vesh", 1]], "clump": "sys", "strays_permille": 100}});
    let l = lib(vec![vesh(), a_list, mix]);
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    let mut strays = 0;
    let mut kinds = BTreeSet::new();
    for s in 0..50 {
        let mut by: BTreeMap<String, u32> = BTreeMap::new();
        for b in 0..20 {
            let a = addr(&format!("u:t/sys:{s}/body:{b}"));
            let n = name(&l, &req(&seed, &a, "nations", "planet", &none)).unwrap();
            *by.entry(n.scheme.clone()).or_default() += 1;
        }
        let top = *by.values().max().unwrap();
        strays += 20 - top;
        kinds.insert(by.iter().max_by_key(|x| x.1).unwrap().0.clone());
    }
    // Both traditions lead somewhere, and about a twentieth stray
    // (10 % stray, half of them back into the clump's own choice).
    assert_eq!(kinds.len(), 2);
    assert!((20..=90).contains(&strays), "{strays}");
}

#[test]
fn the_avoid_list_is_honoured_after_the_fold() {
    let s = json!({"id": "bad", "label": "Bad", "lists": {"n": ["Grot", "Gr0t", "Ashford"]}, "forms": {"default": [["{list:n}", 1]]}});
    let l = Library::load(&[("p".into(), json!({"schemes": [s], "avoid": ["grot"]}))]).unwrap();
    assert_eq!(l.avoid_hits("bad"), 2);
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    for i in 0..200 {
        let a = addr(&format!("u:t/sys:{i}"));
        let n = name(&l, &req(&seed, &a, "bad", "planet", &none)).unwrap();
        assert_eq!(n.show(None, None), "Ashford");
    }
    // A scheme whose every draw is avoided gives up, saying why.
    let only = json!({"id": "only", "label": "Only", "lists": {"n": ["Grot"]}, "forms": {"default": [["{list:n}", 1]]}});
    let l = Library::load(&[("p".into(), json!({"schemes": [only], "avoid": ["grot"]}))]).unwrap();
    let a = addr("u:t/sys:1");
    let e = name(&l, &req(&seed, &a, "only", "planet", &none)).unwrap_err();
    assert!(e.contains("keeps its designation only"), "{e}");
    // Taken names move on too.
    let l2 = lib(vec![vesh()]);
    let first = name(&l2, &req(&seed, &a, "vesh", "planet", &none)).unwrap();
    let f = fold(&first.show(None, None));
    let second = name_with(
        &l2,
        &req(&seed, &a, "vesh", "planet", &none),
        &mut |x, _| x == f,
    )
    .unwrap();
    assert_ne!(second.form, first.form);
    assert_eq!(second.tries, 2);
}

#[test]
fn text_is_made_safe_and_look_alikes_fold_together() {
    assert_eq!(compose("e\u{301}toile"), "étoile");
    assert_eq!(compose("u\u{308}\u{304}"), "ǖ");
    assert_eq!(compose("\u{1112}\u{1161}\u{11AB}"), "한");
    assert_eq!(fold("Таrn"), fold("Tarn"));
    assert_eq!(fold("Ōsaka-jō"), "osakajo");
    assert_eq!(fold("Gr0t"), "grot");
    assert!(
        near("kelmere", "kelmare") && near("kelmere", "kelmeres") && !near("kelmere", "ashford")
    );
    for bad in [
        "Ta\u{202E}rn",
        "Ta\u{200B}rn",
        "Ta\u{0007}rn",
        "<b>Tarn</b>",
        "{word}",
        "a\u{301}\u{302}\u{303}",
        " Tarn",
        "\u{E000}",
    ] {
        assert!(check(bad).is_err(), "{bad:?}");
    }
    assert!(check(&"a".repeat(65)).is_err());
    for good in [
        "Tarn",
        "Mount Ka'el",
        "天宫",
        "चन्द्रयान",
        "Hawaiʻi",
        "São Paulo",
        "Ōsaka",
    ] {
        check(good).unwrap_or_else(|e| panic!("{good}: {e}"));
    }
}

fn refused(v: J) -> String {
    Library::load(&[("hostile".into(), json!({"schemes": [v]}))])
        .unwrap_err()
        .join("\n")
}

#[test]
fn hostile_schemes_are_refused_with_a_reason() {
    let base = || json!({"id": "h", "label": "Hostile", "forms": {"default": [["{list:n}", 1]]}, "lists": {"n": ["Ka"]}});
    // A direction override hidden in a row.
    let mut v = base();
    v["lists"]["n"] = json!(["Ka\u{202E}lam"]);
    assert!(refused(v).contains("invisible or control"));
    // Markup.
    let mut v = base();
    v["lists"]["n"] = json!(["<script>"]);
    assert!(refused(v).contains("not a letter"));
    // A name too long.
    let mut v = base();
    v["lists"]["n"] = json!(["a".repeat(80)]);
    assert!(refused(v).contains("longer than the 64"));
    // Too many rows.
    let mut v = base();
    v["lists"]["n"] = J::Array((0..50_001).map(|i| json!(format!("N{i}"))).collect());
    assert!(refused(v).contains("50000 rows"));
    // Too big.
    let mut v = base();
    v["about"] = json!("x".repeat(1 << 20));
    assert!(refused(v).contains("bytes is more than"));
    // Unknown parts, missing lists, words without sounds.
    let mut v = base();
    v["forms"]["default"] = json!([["{eval}", 1]]);
    assert!(refused(v).contains("not a part a form knows"));
    let mut v = base();
    v["forms"]["default"] = json!([["{list:none}", 1]]);
    assert!(refused(v).contains("no list none"));
    let mut v = base();
    v["forms"]["default"] = json!([["{word}", 1]]);
    assert!(refused(v).contains("needs sounds"));
    // A long syllable run, or a shape naming no class.
    let mut v = vesh();
    v["sounds"]["length"] = json!([2, 40]);
    assert!(refused(v).contains("length is 1 to 8"));
    let mut v = vesh();
    v["sounds"]["shapes"] = json!([["CQ", 1]]);
    assert!(refused(v).contains("names no class Q"));
    // A mix of itself, or of nothing.
    let mix = json!({"id": "loop", "label": "Loop", "mix": {"of": [["loop2", 1]]}});
    let mix2 = json!({"id": "loop2", "label": "Loop two", "mix": {"of": [["loop", 1]]}});
    let e = Library::load(&[("p".into(), json!({"schemes": [mix, mix2]}))])
        .unwrap_err()
        .join("\n");
    assert!(e.contains("contains itself"), "{e}");
    let e = refused(json!({"id": "m", "label": "Mix", "mix": {"of": [["gone", 1]]}}));
    assert!(e.contains("names no scheme"), "{e}");
    // Unknown fields are refused, not ignored.
    let mut v = base();
    v["script_code"] = json!("x");
    assert!(refused(v).contains("unknown field"));
}

#[test]
fn the_old_feature_settings_still_load() {
    let p: J = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packs/base-surface/pack.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let naming = find(&p, "naming").expect("base-surface carries naming settings");
    let s = Scheme::from_naming_settings("legacy", naming).unwrap();
    let mut l = Library::default();
    l.schemes.insert("legacy".into(), s);
    let seed = Seed::from_text("nm1");
    let none = BTreeMap::new();
    let a = addr("u:t/sys:0/body:1/feature:3");
    let n = name(&l, &req(&seed, &a, "legacy", "peak", &none)).unwrap();
    let shown = n.show(None, None);
    assert!(shown.starts_with("Mount "), "{shown}");
}

fn find<'a>(v: &'a J, key: &str) -> Option<&'a J> {
    match v {
        J::Object(o) => o.get(key).or_else(|| o.values().find_map(|x| find(x, key))),
        J::Array(a) => a.iter().find_map(|x| find(x, key)),
        _ => None,
    }
}

/// The shipped `packs/base-names` (NM3a), loaded as the kernel loads it.
fn base_names() -> Library {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packs/base-names/pack.json"
    );
    let v: J = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    Library::load(&[(
        "sse.base-names".into(),
        v["provides"]["other"]["names"].clone(),
    )])
    .unwrap()
}

/// NM3a: Sol's bodies get designations as real practice writes them, in
/// both forms, and every one follows a renamed star.
#[test]
fn base_names_designates_sol_as_real_practice_does() {
    let l = base_names();
    let seed = Seed::from_text("nm3a");
    let none = BTreeMap::new();
    let sun = addr("u:t/sys:0/body:0");
    let mut r = req(&seed, &sun, "catalogue", "star", &none);
    r.layer = "designation";
    r.serial = Some(102);
    let star = name(&l, &r).unwrap().show(None, None);
    assert_eq!(star, "SX 000102");
    // A coordinate designation from the sky position.
    let sky: BTreeMap<String, String> = [
        ("ra_deg".into(), "83.82".into()),
        ("dec_deg".into(), "-5.39".into()),
    ]
    .into();
    let r = req(&seed, &sun, "catalogue", "star-sky", &sky);
    assert!(name(&l, &r)
        .unwrap()
        .show(None, None)
        .starts_with("SX J0535"));
    // The eight planets by orbit, then by letter from b.
    let mut roman = vec![];
    let mut letter = vec![];
    for i in 0..8u32 {
        let a = addr(&format!("u:t/sys:0/body:0/body:{i}"));
        for (scheme, out) in [
            ("catalogue", &mut roman),
            ("catalogue-letters", &mut letter),
        ] {
            let mut r = req(&seed, &a, scheme, "planet", &none);
            r.index = i;
            let n = name(&l, &r).unwrap();
            assert!(n.follows, "{scheme}");
            out.push(n.show(Some(&star), None));
        }
    }
    assert_eq!(roman[2], "SX 000102 III");
    assert_eq!(roman[7], "SX 000102 VIII");
    assert_eq!(letter[0], "SX 000102 b");
    assert_eq!(letter[4], "SX 000102 f");
    // Moons: a letter after a numeral, a numeral after a letter (Kepler-1625 b I).
    let io = addr("u:t/sys:0/body:0/body:4/body:0");
    let m = name(&l, &req(&seed, &io, "catalogue", "moon", &none)).unwrap();
    assert_eq!(m.show(Some("SX 000102 V"), None), "SX 000102 V b");
    let m = name(&l, &req(&seed, &io, "catalogue-letters", "moon", &none)).unwrap();
    assert_eq!(m.show(Some("SX 000102 f"), None), "SX 000102 f I");
    // Components of a multiple star.
    let mut r = req(&seed, &sun, "catalogue", "component", &none);
    r.index = 1;
    assert_eq!(
        name(&l, &r).unwrap().show(Some("SX 000102"), None),
        "SX 000102 B"
    );
    // A surface feature takes the IAU's descriptor term.
    let mons = addr("u:t/sys:0/body:0/body:3/feat:0");
    let mut r = req(&seed, &mons, "catalogue", "shield", &none);
    r.index = 0;
    assert_eq!(
        name(&l, &r).unwrap().show(Some("SX 000102 IV"), None),
        "SX 000102 IV Mons 1"
    );
    // Renaming the star carries down.
    assert_eq!(
        name(&l, &req(&seed, &io, "catalogue", "moon", &none))
            .unwrap()
            .show(Some("Sol V"), None),
        "Sol V b"
    );
}

/// NM3a: a 200-body system has no duplicate designation, in either form, and
/// every surface kind the surface crate names has a form.
#[test]
fn base_names_gives_a_200_body_system_no_duplicate() {
    let l = base_names();
    let seed = Seed::from_text("nm3a");
    let none = BTreeMap::new();
    for scheme in ["catalogue", "catalogue-letters"] {
        let addrs: Vec<Address> = (0..200)
            .map(|i| addr(&format!("u:t/sys:0/body:0/body:{i}")))
            .collect();
        let reqs: Vec<Request> = addrs
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let mut r = req(&seed, a, scheme, "planet", &none);
                r.index = i as u32;
                r.parent = Some("SX 000102");
                r
            })
            .collect();
        let names: BTreeSet<String> = siblings(&l, &reqs)
            .into_iter()
            .map(|n| n.unwrap().show(Some("SX 000102"), None))
            .collect();
        assert_eq!(names.len(), 200, "{scheme}");
    }
    for kind in [
        "ocean",
        "sea",
        "lake",
        "continent",
        "island",
        "range",
        "peak",
        "summit",
        "deep",
        "ice-cap",
        "crater",
        "shield",
        "plains",
        "rift",
        "chasma",
        "corona",
        "patera",
        "scarp",
        "settlement",
        "port",
        "mine",
        "base",
    ] {
        let a = addr("u:t/sys:0/body:0/body:3/feat:0");
        let n = name(&l, &req(&seed, &a, "catalogue", kind, &none)).unwrap();
        assert!(n.form.starts_with("{parent} "), "{kind}: {}", n.form);
        assert!(n.form.ends_with(" 1"), "{kind}: {}", n.form);
    }
    // The avoid list loaded, folded.
    assert!(l.avoid.contains("merde") && l.avoid.len() > 40);
}
