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
}

#[test]
fn element_origins_name_the_legend_and_sum_to_one() {
    let block = |shares: J, also: J| {
        json!({
            "sources": [{"id": "k2020", "text": "Kobayashi et al. 2020"}],
            "origin_legend": [
                {"id": "massive_stars", "physics": "core collapse", "plain": "exploding massive stars", "story": "giants that blew apart"},
                {"id": "white_dwarfs", "physics": "Type Ia", "plain": "white dwarfs", "story": "dead stars that detonated"}
            ],
            "elements": [{"id": "Fe", "z": 26, "symbol": "Fe",
                "origin": {"shares": shares, "src": "k2020", "also": also, "check": "authored"}}]
        })
    };
    let good = Reference::from_json(&[(
        "p".into(),
        block(
            json!({"massive_stars": 0.5, "white_dwarfs": 0.5}),
            json!([]),
        ),
    )]);
    assert!(good.is_ok(), "{:#?}", good.err());
    let legend = good.unwrap().listing(Some("origin_legend"));
    assert_eq!(
        legend["origin_legend"]["white_dwarfs"]["plain"],
        "white dwarfs"
    );
    let bad = Reference::from_json(&[(
        "p".into(),
        block(
            json!({"massive_stars": 0.5, "neutron_stars": 0.3}),
            json!(["nobody"]),
        ),
    )])
    .unwrap_err();
    let all = bad.join("\n");
    assert!(
        all.contains("neutron_stars is not in origin_legend"),
        "{all}"
    );
    assert!(all.contains("sum to 0.8"), "{all}");
    assert!(all.contains("source nobody"), "{all}");
}
