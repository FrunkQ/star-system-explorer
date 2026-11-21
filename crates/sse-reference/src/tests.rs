use super::*;
use serde_json::json;

fn fixture() -> J {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/packs/reference-five-errors.json"
    ))
    .unwrap();
    serde_json::from_str::<J>(&s).unwrap()["provides"]["reference"].clone()
}

fn good() -> J {
    json!({
        "sources": [{"id": "ciaaw-2021", "text": "CIAAW standard atomic weights 2021"},
                    {"id": "fs09", "text": "Fray and Schmitt 2009", "doi": "10.1016/j.pss.2009.09.011"}],
        "elements": [
            {"id": "h", "z": 1, "atomic_weight": {"v": 1.008, "unit": "g/mol", "src": "ciaaw-2021"}},
            {"id": "o", "z": 8, "atomic_weight": {"v": 15.999, "unit": "g/mol", "src": "ciaaw-2021"}},
            {"id": "c", "z": 6, "atomic_weight": {"v": 12.011, "unit": "g/mol", "src": "ciaaw-2021"}}
        ],
        "substances": [
            {"id": "h2o", "formula": {"h": 2, "o": 1}, "molar_mass": {"v": 18.015, "unit": "g/mol", "src": "ciaaw-2021"}},
            {"id": "h2", "formula": {"h": 2}},
            {"id": "o2", "formula": {"o": 2}},
            {"id": "oh-", "formula": {"o": 1, "h": 1}, "charge": -1},
            {"id": "h+", "formula": {"h": 1}, "charge": 1},
            {"id": "co2", "formula": {"c": 1, "o": 2}, "phases": {
                "triple_point_k": {"v": 216.58, "unit": "K", "src": "fs09"},
                "triple_point_pa": {"v": 518000, "unit": "Pa", "src": "fs09"},
                "latent_heat_sublimation_j_mol": {"v": 26000, "unit": "J/mol", "src": "fs09"},
                "vapour_pressure": {"fit": "clausius-clapeyron", "coeffs": [518000, 216.58, 26000], "valid_k": [150, 216.58], "src": "fs09"}
            }}
        ],
        "mixtures": [{"id": "fizz", "recipe": {"h2o": {"v": 0.95, "src": "ciaaw-2021"}, "co2": {"v": 0.05, "src": "ciaaw-2021"}}}],
        "reactions": [
            {"id": "burn-hydrogen", "terms": [["h2", -2], ["o2", -1], ["h2o", 2]]},
            {"id": "water-splits", "terms": [["h2o", -1], ["h+", 1], ["oh-", 1]]}
        ]
    })
}

#[test]
fn five_mistakes_give_five_messages() {
    let e = Reference::from_json(&[("fixture".into(), fixture())]).unwrap_err();
    assert_eq!(e.len(), 5, "{e:#?}");
    for needle in [
        "unobtainium",
        "disagrees with its formula",
        "vapour curve gives",
        "recipe sums to",
        "does not conserve elements",
    ] {
        assert!(
            e.iter().any(|m| m.contains(needle)),
            "no message about {needle}: {e:#?}"
        );
    }
}

#[test]
fn a_good_sheet_loads_and_works_out_molar_mass() {
    let r = Reference::from_json(&[("base".into(), good())]).unwrap();
    let mm = r.molar_mass_kg_mol("h2o").unwrap();
    assert!((mm - 0.018015).abs() < 1e-9, "{mm}");
    let d = r.describe("substances", "h2o").unwrap();
    assert_eq!(d["packs"], json!(["base"]));
    assert_eq!(d["sources"][0]["id"], json!("ciaaw-2021"));
    assert!(d["worked"]["molar_mass"]["v"].as_f64().is_some());
    assert_eq!(
        r.listing(Some("elements"))["elements"],
        json!(["c", "h", "o"])
    );
}

#[test]
fn an_override_must_say_what_it_replaces() {
    let quiet = json!({"elements": [{"id": "h", "atomic_weight": {"v": 1.0, "unit": "g/mol", "src": "ciaaw-2021"}}]});
    let e = Reference::from_json(&[("base".into(), good()), ("tweak".into(), quiet)]).unwrap_err();
    assert_eq!(e.len(), 1, "{e:#?}");
    assert!(e[0].contains("without saying what it replaces"), "{}", e[0]);

    let said = json!({"elements": [{"id": "h", "atomic_weight": {"v": 1.00794, "unit": "g/mol", "src": "ciaaw-2021", "replaces": "base"}}]});
    let r = Reference::from_json(&[("base".into(), good()), ("tweak".into(), said)]).unwrap();
    let w = r.row("elements", "h").unwrap()["atomic_weight"]["v"]
        .as_f64()
        .unwrap();
    assert_eq!(w, 1.00794);
    // The rest of the row stays.
    assert_eq!(r.row("elements", "h").unwrap()["z"], json!(1));
    assert_eq!(
        r.origin["elements/h"],
        vec!["base".to_string(), "tweak".to_string()]
    );
    // A new row needs nothing.
    let new = json!({"elements": [{"id": "n", "z": 7, "atomic_weight": {"v": 14.007, "unit": "g/mol", "src": "ciaaw-2021"}}]});
    assert!(Reference::from_json(&[("base".into(), good()), ("more".into(), new)]).is_ok());
}

#[test]
fn charge_is_conserved_and_unknown_names_are_caught() {
    let mut g = good();
    g["reactions"] = json!([
        {"id": "lost-charge", "terms": [["h2o", -1], ["h+", 1], ["oh-", 1], ["h+", 0]]},
        {"id": "half", "terms": [["h2o", -1], ["h2", 1], ["o2", 0.5]]},
        {"id": "ghost", "terms": [["unobtainium", -1]]}
    ]);
    g["substances"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "x", "formula": {"zz": 1}}));
    let e = Reference::from_json(&[("base".into(), g)]).unwrap_err();
    assert_eq!(e.len(), 2, "{e:#?}");
    assert!(e.iter().any(|m| m.contains("unobtainium")));
    assert!(e.iter().any(|m| m.contains("element zz")));

    let mut g = good();
    // Elements balance, charge does not: two protons become a hydrogen molecule.
    g["reactions"] = json!([{"id": "charged", "terms": [["h+", -2], ["h2", 1]]}]);
    let e = Reference::from_json(&[("base".into(), g)]).unwrap_err();
    assert_eq!(e.len(), 1, "{e:#?}");
    assert!(e[0].contains("does not conserve charge"), "{}", e[0]);
}

#[test]
fn fray_schmitt_form_is_read() {
    // CO2 from Fray and Schmitt 2009 table: A0..A5 for 40-194.7 K.
    let fit = json!({"fit": "fray-schmitt-2009", "coeffs": [1.476e1, -2.571e3, -7.781e4, 4.325e6, -1.207e8, 1.350e9]});
    let p = vapour_pressure_pa(&fit, 194.7).unwrap();
    // Sublimation point at one atmosphere.
    assert!((p / 101_325.0 - 1.0).abs() < 0.05, "{p}");
}

#[test]
fn the_same_sheet_hashes_the_same_twice() {
    let a =
        serde_json::to_string(&Reference::from_json(&[("base".into(), good())]).unwrap()).unwrap();
    let b =
        serde_json::to_string(&Reference::from_json(&[("base".into(), good())]).unwrap()).unwrap();
    assert_eq!(
        sse_core::hash::ContentHash::of_bytes(a.as_bytes()),
        sse_core::hash::ContentHash::of_bytes(b.as_bytes())
    );
}

/// The shipped sheet (MP2, `packs/base-reference`) loads clean once it is in
/// the repository.
#[test]
fn the_base_sheet_loads_clean() {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packs/base-reference/pack.json"
    );
    let path = std::env::var("SSE_BASE_REFERENCE").unwrap_or_else(|_| p.to_string());
    let Ok(s) = std::fs::read_to_string(&path) else {
        return;
    };
    let m: sse_pack::PackManifest = serde_json::from_str(&s).unwrap();
    let r = Reference::from_packs(&[m]);
    assert!(r.is_ok(), "{:#?}", r.err());
    // Unedited, the sheet's handle is the compiled constants bit for bit,
    // which is what keeps every golden the same (MP1c).
    let h = r.unwrap().handle();
    for id in sse_core::constants::Constants::IDS {
        assert_eq!(
            h.get(id).map(f64::to_bits),
            sse_core::constants::Constants::REAL
                .get(id)
                .map(f64::to_bits),
            "{id}: sheet {:?}, compiled {:?}",
            h.get(id),
            sse_core::constants::Constants::REAL.get(id)
        );
    }
}

fn constants(extra: J) -> J {
    let mut g = json!({
        "sources": [{"id": "codata-2018", "text": "CODATA 2018"}],
        "constants": [
            {"id": "c", "name": "speed of light", "value": {"v": 299792458.0, "unit": "m s-1", "src": "codata-2018"}},
            {"id": "h", "name": "Planck constant", "value": {"v": 6.62607015e-34, "unit": "J s", "src": "codata-2018"}},
            {"id": "k_B", "name": "Boltzmann constant", "value": {"v": 1.380649e-23, "unit": "J K-1", "src": "codata-2018"}},
            {"id": "sigma_SB", "name": "Stefan-Boltzmann constant", "derived_from": "2 pi^5 k_B^4 / (15 h^3 c^2)",
             "value": {"v": 5.670374419e-8, "unit": "W m-2 K-4", "src": "codata-2018"}}
        ]
    });
    if let Some(e) = extra.as_array() {
        g["constants"]
            .as_array_mut()
            .unwrap()
            .extend(e.iter().cloned());
    }
    g
}

#[test]
fn derived_constants_are_worked_out_and_checked() {
    let r = Reference::from_json(&[("base".into(), constants(json!([])))]).unwrap();
    let s = r.constant("sigma_SB").unwrap();
    assert!((s / 5.670374419e-8 - 1.0).abs() < 1e-9, "{s}");
    // A typed copy that disagrees is refused.
    let bad = constants(
        json!([{"id": "two_c", "name": "x", "derived_from": "2 c", "value": {"v": 6e8, "unit": "m s-1", "src": "codata-2018"}}]),
    );
    let e = Reference::from_json(&[("base".into(), bad)]).unwrap_err();
    assert_eq!(e.len(), 1, "{e:#?}");
    assert!(e[0].contains("typed as"), "{}", e[0]);
    // A formula naming an unknown constant is refused.
    let bad = constants(
        json!([{"id": "q", "name": "x", "derived_from": "c / zeta", "value": {"v": 1.0, "unit": "1", "src": "codata-2018"}}]),
    );
    let e = Reference::from_json(&[("base".into(), bad)]).unwrap_err();
    assert!(e[0].contains("zeta"), "{e:#?}");
}

#[test]
fn halving_h_moves_stefan_boltzmann_by_eight() {
    // sigma goes as h^-3: halving h multiplies it by 8.
    let half = json!({"constants": [{"id": "h", "value": {"v": 3.313035075e-34, "unit": "J s", "src": "codata-2018", "replaces": "base"}}]});
    let r = Reference::from_json(&[
        ("base".into(), constants(json!([]))),
        ("what-if".into(), half),
    ])
    .unwrap();
    let s = r.constant("sigma_SB").unwrap();
    assert!((s / (8.0 * 5.670374419e-8) - 1.0).abs() < 1e-9, "{s}");
    // The handle follows the edit too.
    assert_eq!(r.handle().sigma_sb, s);
    // Unedited, the handle holds the typed copy.
    let r = Reference::from_json(&[("base".into(), constants(json!([])))]).unwrap();
    assert_eq!(r.handle().sigma_sb, 5.670374419e-8);
}

#[test]
fn formulas_parse_as_written() {
    let look = |n: &str| match n {
        "a" => Some(2.0),
        "b" => Some(3.0),
        _ => None,
    };
    assert_eq!(expr::eval("a b", look).unwrap(), 6.0);
    assert_eq!(expr::eval("a^b / (a b)", look).unwrap(), 8.0 / 6.0);
    assert_eq!(expr::eval("-a + 1.5e1", look).unwrap(), 13.0);
    assert_eq!(expr::eval("a 648000 / b", look).unwrap(), 432000.0);
    assert!(expr::eval("(a", look).is_err());
    assert_eq!(
        expr::names("2 pi^5 k_B^4 / (15 h^3 c^2)"),
        vec!["c", "h", "k_B"]
    );
}

#[test]
fn checked_by_checks_but_never_works_out() {
    let g = |extra: J| {
        let mut b = constants(json!([
            {"id": "G", "name": "G", "value": {"v": 6.6743e-11, "unit": "m3 kg-1 s-2", "src": "codata-2018"}},
            {"id": "GM_sun", "name": "GM", "value": {"v": 1.3271244e20, "unit": "m3 s-2", "src": "codata-2018"}},
            {"id": "M_sun", "name": "Sun", "checked_by": "GM_sun / G", "value": {"v": 1.988409870698051e30, "unit": "kg", "src": "codata-2018"}}
        ]));
        if !extra.is_null() {
            b["constants"].as_array_mut().unwrap().push(extra);
        }
        b
    };
    assert!(Reference::from_json(&[("base".into(), g(J::Null))]).is_ok());
    let twice = json!({"constants": [{"id": "G", "value": {"v": 1.33486e-10, "unit": "m3 kg-1 s-2", "src": "codata-2018", "replaces": "base"}}]});
    let r =
        Reference::from_json(&[("base".into(), g(J::Null)), ("what-if".into(), twice)]).unwrap();
    // Doubling G keeps the Sun's mass.
    assert_eq!(r.constant("M_sun").unwrap(), 1.988409870698051e30);
    let mut bad = g(J::Null);
    bad["constants"][6]["value"]["v"] = json!(2.0e30);
    let e = Reference::from_json(&[("base".into(), bad)]).unwrap_err();
    assert!(e[0].contains("M_sun"), "{e:#?}");
}
