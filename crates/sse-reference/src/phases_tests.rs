//! MP17: many phases per substance, their load checks and `phase_at`.

use super::phases::*;
use super::*;
use serde_json::json;

/// Water's sublimation line (Feistel and Wagner 2007, as on the sheet).
fn sub_fit() -> J {
    json!({"fit": "feistel-wagner-2007", "coeffs": [273.16, 611.657, 20.9969665107897, 3.72437478271362,
        -13.9205483215524, 29.6988765013566, -40.1972392635944, 29.7880481050215, -9.13050963547721],
        "valid_k": [50.0, 273.16], "src": "fw07"})
}

/// Ice Ih's melting line, IAPWS R14-08 (Wagner et al. 2011, doi:10.1063/1.3657937).
fn ih_melt() -> J {
    json!({"form": "iapws_melt", "tn": 273.16, "pn": 611.657, "log": false,
        "terms": [[0.119539337e7, 3.0], [0.808183159e5, 25.75], [0.333826860e4, 103.75]],
        "valid_k": [251.165, 273.16]})
}

const LP: (f64, f64, usize) = (0.0, 10.0, 101);
const TK: (f64, f64, usize) = (100.0, 400.0, 151);

/// A toy map in four stable phases, classified from the same lines the
/// boundaries give: vapour under the sublimation and boiling lines, ice Ih
/// left of its melting line, ice VII above 2.2 GPa, liquid elsewhere.
fn build_cells(ph: &Map<String, J>) -> String {
    let mut s = String::new();
    for i in 0..LP.2 {
        let lp = LP.0 + (LP.1 - LP.0) * i as f64 / (LP.2 - 1) as f64;
        let p = sse_core::math::pow(10.0, lp);
        for j in 0..TK.2 {
            let t = TK.0 + (TK.1 - TK.0) * j as f64 / (TK.2 - 1) as f64;
            let boil =
                611.657 * sse_core::math::exp(-45_000.0 / 8.314_462_618 * (1.0 / t - 1.0 / 273.16));
            let melt = if t >= 251.165 {
                iapws_melt_pa(
                    273.16,
                    611.657,
                    false,
                    &[
                        (0.119539337e7, 3.0),
                        (0.808183159e5, 25.75),
                        (0.333826860e4, 103.75),
                    ],
                    t.min(273.16),
                )
                .unwrap()
            } else {
                2.099e8
            };
            let c = if p > 2.2e9 {
                '3'
            } else if t < 273.16 {
                let ps = sublimation_pa(ph, t).unwrap();
                if p < ps {
                    '2'
                } else if p < melt {
                    '0'
                } else {
                    '1'
                }
            } else if p < boil {
                '2'
            } else {
                '1'
            };
            s.push(c);
        }
    }
    s
}

fn water(edit: impl FnOnce(&mut Map<String, J>)) -> J {
    let mut ph = json!({
        "triple_point_k": {"v": 273.16, "unit": "K", "src": "fw07"},
        "triple_point_pa": {"v": 611.657, "unit": "Pa", "src": "fw07"},
        "vapour_pressure": sub_fit(),
        "forms": [
            {"id": "h2o.ice_ih", "name": {"physics": "ice Ih", "plain": "ordinary ice"}, "kind": "solid",
             "evidence": {"grade": "measured", "src": "fw07"},
             "props": {"density": {"v": 917.0, "unit": "kg m-3", "src": "fw07"}}},
            {"id": "h2o.liquid", "name": {"physics": "liquid water"}, "kind": "liquid",
             "evidence": {"grade": "measured", "src": "fw07"}},
            {"id": "h2o.vapour", "name": {"physics": "water vapour"}, "kind": "gas",
             "evidence": {"grade": "measured", "src": "fw07"}},
            {"id": "h2o.ice_vii", "name": {"physics": "ice VII"}, "kind": "solid",
             "evidence": {"grade": "computed", "p_pa": [2.2e9, 6e10], "src": "iapws11"}},
            {"id": "h2o.ice_ic", "name": {"physics": "ice Ic"}, "kind": "solid", "stability": "metastable",
             "evidence": {"grade": "measured", "src": "fw07"}}
        ],
        "boundaries": [
            {"between": ["h2o.ice_ih", "h2o.liquid"], "curve": ih_melt(), "src": "iapws11"},
            {"between": ["h2o.ice_ih", "h2o.vapour"], "curve": {"form": "vapour"}, "src": "fw07"}
        ],
        "meets": [
            {"phases": ["h2o.ice_ih", "h2o.liquid", "h2o.vapour"], "t_k": 273.16, "p_pa": 611.657, "src": "fw07"}
        ]
    });
    let o = ph.as_object_mut().unwrap();
    let cells = build_cells(o);
    o.insert(
        "table".into(),
        json!({"log10_p_pa": {"lo": LP.0, "hi": LP.1, "n": LP.2}, "t_k": {"lo": TK.0, "hi": TK.1, "n": TK.2},
               "phases": ["h2o.ice_ih", "h2o.liquid", "h2o.vapour", "h2o.ice_vii"], "cells": cells}),
    );
    edit(o);
    json!({
        "sources": [{"id": "fw07", "text": "Feistel and Wagner 2007"},
                    {"id": "iapws11", "text": "Wagner, Riethmann, Feistel and Harvey 2011", "doi": "10.1063/1.3657937"},
                    {"id": "ciaaw-2021", "text": "CIAAW 2021"}],
        "elements": [
            {"id": "h", "z": 1, "atomic_weight": {"v": 1.008, "unit": "g/mol", "src": "ciaaw-2021"}},
            {"id": "o", "z": 8, "atomic_weight": {"v": 15.999, "unit": "g/mol", "src": "ciaaw-2021"}}
        ],
        "substances": [{"id": "h2o", "formula": {"h": 2, "o": 1}, "phases": ph}]
    })
}

fn load(v: J) -> Result<Reference, Vec<String>> {
    Reference::from_json(&[("test".into(), v)])
}

#[test]
fn a_water_map_loads_and_answers() {
    let r = load(water(|_| {})).expect("loads");
    let at = |p: f64, t: f64| r.phase_at("h2o", p, t).unwrap().phase;
    assert_eq!(at(1.013e5, 250.0), "h2o.ice_ih");
    assert_eq!(at(1.013e5, 300.0), "h2o.liquid");
    assert_eq!(at(1.0e3, 300.0), "h2o.vapour");
    assert_eq!(at(10.0, 240.0), "h2o.vapour");
    assert_eq!(at(10.0, 200.0), "h2o.ice_ih");
    // Ih melts under pressure: liquid at 260 K above its melting line
    // (about 140 MPa there).
    assert_eq!(at(3.0e8, 260.0), "h2o.liquid");
    assert_eq!(at(1.0e10, 300.0), "h2o.ice_vii");
    let a = r.phase_at("h2o", 1.0e10, 300.0).unwrap();
    assert_eq!(a.from, "table");
    // A computed phase hands back its grade and the paper's link.
    assert_eq!(a.grade.as_deref(), Some("computed"));
    assert_eq!(a.source.as_ref().unwrap()["doi"], "10.1063/1.3657937");
    assert_eq!(a.form.unwrap()["kind"], "solid");
    // Outside the table, the familiar reading answers.
    assert_eq!(r.phase_at("h2o", 1.0e11, 300.0).unwrap().from, "familiar");
}

#[test]
fn the_iapws_melting_line_reads_its_published_points() {
    // IAPWS R14-08 table 3: ice Ih melts at 208.566 MPa at 251.165 K.
    let terms = [
        (0.119539337e7, 3.0),
        (0.808183159e5, 25.75),
        (0.333826860e4, 103.75),
    ];
    let p = iapws_melt_pa(273.16, 611.657, false, &terms, 251.165).unwrap();
    assert!((p / 208.566e6 - 1.0).abs() < 1e-4, "{p}");
    let p0 = iapws_melt_pa(273.16, 611.657, false, &terms, 273.16).unwrap();
    assert!((p0 - 611.657).abs() < 1e-9);
}

#[test]
fn each_broken_rule_is_named() {
    type Edit = Box<dyn FnOnce(&mut Map<String, J>)>;
    let cases: Vec<(&str, Edit)> = vec![
        (
            "not in phases.forms",
            Box::new(|o| {
                o["boundaries"][0]["between"][1] = json!("h2o.ice_xx");
            }),
        ),
        (
            "differs from triple_point_k",
            Box::new(|o| {
                o["meets"][0]["t_k"] = json!(273.5);
            }),
        ),
        (
            "misses the triple point",
            Box::new(|o| {
                o["meets"][0]["p_pa"] = json!(6.0e4);
                o["triple_point_pa"]["v"] = json!(6.0e4);
            }),
        ),
        (
            "written twice",
            Box::new(|o| {
                let m = o["meets"][0].clone();
                o["meets"].as_array_mut().unwrap().push(m);
            }),
        ),
        (
            "which is metastable",
            Box::new(|o| {
                o["table"]["phases"][3] = json!("h2o.ice_ic");
            }),
        ),
        (
            "cells, not",
            Box::new(|o| {
                o["table"]["cells"] = json!("0123");
            }),
        ),
        (
            "kind",
            Box::new(|o| {
                o["forms"][1]["kind"] = json!("goo");
            }),
        ),
        (
            "evidence grade",
            Box::new(|o| {
                o["forms"][1]["evidence"]["grade"] = json!("guessed");
            }),
        ),
        (
            "not in sources",
            Box::new(|o| {
                o["forms"][0]["props"]["density"]["src"] = json!("nobody");
            }),
        ),
        (
            "other phases beside boundary",
            Box::new(|o| {
                // Swap ice and liquid in the table: the melting line no longer
                // has its two phases either side.
                let c: String = o["table"]["cells"]
                    .as_str()
                    .unwrap()
                    .chars()
                    .map(|c| match c {
                        '0' => '3',
                        c => c,
                    })
                    .collect();
                o["table"]["cells"] = json!(c);
            }),
        ),
    ];
    for (want, edit) in cases {
        let e = load(water(edit)).expect_err(want);
        assert!(e.iter().any(|x| x.contains(want)), "{want}: {e:?}");
    }
}

#[test]
fn a_row_with_one_phase_still_loads_and_answers() {
    let v = water(|o| {
        o.remove("boundaries");
        o.remove("meets");
        o.remove("table");
        o["forms"] = json!([{"id": "h2o.ice_ih", "kind": "solid", "evidence": {"grade": "measured", "src": "fw07"}}]);
    });
    let r = load(v).expect("loads");
    let a = r.phase_at("h2o", 1.0e5, 250.0).unwrap();
    assert_eq!(
        (a.phase.as_str(), a.from.as_str()),
        ("h2o.solid", "familiar")
    );
}

/// The base sheet has no phase tables yet: water answers from its triple
/// point, sublimation line and latent heat, and nothing else changes.
#[test]
fn the_base_sheet_answers_from_the_familiar_three() {
    let s = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packs/base-reference/pack.json"
    ))
    .unwrap();
    let v: J = serde_json::from_str(&s).unwrap();
    let r = load(v["provides"]["reference"].clone()).expect("the base sheet loads");
    let at = |p: f64, t: f64| r.phase_at("h2o", p, t).unwrap().phase;
    assert_eq!(at(1.013e5, 250.0), "h2o.solid");
    assert_eq!(at(1.013e5, 300.0), "h2o.liquid");
    assert_eq!(at(100.0, 300.0), "h2o.vapour");
    assert_eq!(at(1.0e-7, 150.0), "h2o.vapour");
    assert_eq!(at(1.0e-3, 150.0), "h2o.solid");
    // Water boils near 373 K at one atmosphere on the familiar reading.
    assert_eq!(at(1.013e5, 365.0), "h2o.liquid");
    assert_eq!(at(1.013e5, 380.0), "h2o.vapour");
    assert!(r.phase_at("h2o", -1.0, 300.0).is_err());
    assert!(r.phase_at("unobtainium", 1.0, 300.0).is_err());
}

fn to_rle(cells: &str, n_t: usize) -> String {
    let chars: Vec<char> = cells.chars().collect();
    let mut rows = vec![];
    for row in chars.chunks(n_t) {
        let mut runs: Vec<(char, u32)> = vec![];
        for c in row {
            match runs.last_mut() {
                Some(r) if r.0 == *c => r.1 += 1,
                _ => runs.push((*c, 1)),
            }
        }
        let text: Vec<String> = runs
            .iter()
            .map(|(c, n)| {
                let mut d = String::new();
                let mut v = *n;
                while v > 0 {
                    d.insert(0, char::from_digit(v % 36, 36).unwrap());
                    v /= 36;
                }
                format!("{c}{d}")
            })
            .collect();
        rows.push(text.join(" "));
    }
    rows.join("/")
}

/// The run-length table answers as the plain one does, everywhere.
#[test]
fn a_run_length_table_reads_as_the_plain_one() {
    let plain = load(water(|_| {})).unwrap();
    let rle = load(water(|o| {
        let cells = o["table"]["cells"].as_str().unwrap().to_string();
        let t = o["table"].as_object_mut().unwrap();
        t.remove("cells");
        t.insert("rle".into(), json!(to_rle(&cells, TK.2)));
    }))
    .unwrap();
    for i in 0..60 {
        for j in 0..40 {
            let p = sse_core::math::pow(10.0, 0.17 * i as f64);
            let t = 100.0 + 7.5 * j as f64;
            assert_eq!(
                plain.phase_at("h2o", p, t).unwrap().phase,
                rle.phase_at("h2o", p, t).unwrap().phase,
                "{p} {t}"
            );
        }
    }
    // A row that does not fill its temperatures is refused.
    let e = load(water(|o| {
        let t = o["table"].as_object_mut().unwrap();
        t.remove("cells");
        t.insert("rle".into(), json!("02"));
    }))
    .unwrap_err();
    assert!(e.iter().any(|x| x.contains("nodes, not")), "{e:?}");
}

/// A log temperature axis, as the real water map uses (10 K to 10,000 K).
#[test]
fn a_log_temperature_axis_reads() {
    let r = load(water(|o| {
        let n = 31usize;
        // Liquid above 1,000 K, ice below, whatever the pressure.
        let row: String = (0..n)
            .map(|j| {
                if 1.0 + 3.0 * j as f64 / (n - 1) as f64 > 3.0 {
                    '1'
                } else {
                    '0'
                }
            })
            .collect();
        o.remove("boundaries");
        o["table"] = json!({"log10_p_pa": {"lo": 0.0, "hi": 10.0, "n": 2},
            "log10_t_k": {"lo": 1.0, "hi": 4.0, "n": n},
            "phases": ["h2o.ice_ih", "h2o.liquid"], "cells": format!("{row}{row}")});
    }))
    .unwrap();
    assert_eq!(r.phase_at("h2o", 1e5, 20.0).unwrap().phase, "h2o.ice_ih");
    assert_eq!(r.phase_at("h2o", 1e5, 5000.0).unwrap().phase, "h2o.liquid");
}

/// Water's real map, once the sheet holds it (MP18): points from the IAPWS
/// melting-line release and the SeaFreeze phase diagram (Journaux et al.
/// 2020, doi:10.1029/2019JE006176).
#[test]
fn water_on_the_sheet_reads_at_published_points() {
    let path = std::env::var("SSE_BASE_REFERENCE").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../packs/base-reference/pack.json"
        )
        .into()
    });
    let v: J = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let r = load(v["provides"]["reference"].clone()).expect("the sheet loads");
    if !r.phase_tables.contains_key("h2o") {
        return;
    }
    let at = |p: f64, t: f64| r.phase_at("h2o", p, t).unwrap();
    for (p, t, want) in [
        (1.0e5, 250.0, "h2o.ice_ih"),
        (1.0e5, 300.0, "h2o.liquid"),
        (1.0e5, 400.0, "h2o.vapour"),
        (1.0e-7, 150.0, "h2o.vapour"),
        (1.0, 150.0, "h2o.ice_ih"),
        (2.5e8, 250.0, "h2o.ice_iii"),
        (3.0e8, 230.0, "h2o.ice_ii"),
        (5.0e8, 250.0, "h2o.ice_v"),
        (1.0e9, 270.0, "h2o.ice_vi"),
        (1.0e9, 330.0, "h2o.liquid"),
        (1.0e10, 300.0, "h2o.ice_vii"),
    ] {
        let a = at(p, t);
        assert_eq!(a.phase, want, "{p} Pa, {t} K: {a:?}");
        assert_eq!(a.from, "table");
    }
    // Deep in an ice giant: a conducting form, with the paper behind it
    // (superionic ice was seen in laser-shock runs; its edges are computed).
    let deep = at(2.0e11, 4000.0);
    let kind = deep.form.as_ref().unwrap()["kind"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        kind == "superionic solid" || kind == "ionic fluid",
        "{deep:?}"
    );
    assert!(
        matches!(deep.grade.as_deref(), Some("measured" | "computed")),
        "{deep:?}"
    );
    let s = deep.source.unwrap();
    assert!(s.get("doi").is_some() || s.get("url").is_some());
}
