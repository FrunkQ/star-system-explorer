//! W8 part 1 (matter pack MP3): `sse.volatile.species`, the climate's view
//! of the master data sheet. Frost and cloud read one curve per substance.
use crate::settings::{base_reference_pack, volatile_species};
use crate::*;

fn pack(json: &str) -> sse_pack::PackManifest {
    sse_pack::PackManifest::from_json(json).unwrap()
}

#[test]
fn the_climate_reads_its_substances_off_the_sheet() {
    let cs = base_settings();
    let ids: Vec<&str> = cs.substances.iter().map(|s| s.gas.as_str()).collect();
    for id in ["ar", "ch4", "co", "co2", "h2o", "n2", "nh3"] {
        assert!(ids.contains(&id), "{id} missing from {ids:?}");
    }
    // Molar masses are worked out from the sheet's formulas.
    let w = cs.substances.iter().find(|s| s.gas == "h2o").unwrap();
    assert!((w.molar_kg - 0.018015).abs() < 1e-5, "{}", w.molar_kg);
    // A climate pack may no longer carry its own table.
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-climate/pack.json")).unwrap();
    v["provides"]["other"]["climate"]["substances"] =
        serde_json::to_value(&cs.substances[..1]).unwrap();
    let e = ClimateSettings::from_packs(&[pack(&v.to_string())]);
    assert!(e.is_err_and(|e| e.contains("retired")));
}

#[test]
fn frost_and_cloud_read_the_same_curve() {
    let cs = base_settings();
    for sub in &cs.substances {
        // Below each triple point the frost point inverts the saturation
        // pressure the cloud reads, so the two can never disagree.
        for f in [1e-6, 1e-3, 0.3] {
            let p = f * sub.triple_pa;
            let t = ebm::frost_k(sub, p);
            assert!(t > 0.0 && t < sub.triple_k, "{} {p} -> {t}", sub.gas);
            let back = ebm::p_sat(sub, t);
            assert!(
                (back / p - 1.0).abs() < 1e-6,
                "{}: {p} -> {t} K -> {back}",
                sub.gas
            );
            assert_eq!(strengths::condense_k(sub, p), t);
        }
    }
    // Pluto's 1 Pa of nitrogen freezes onto ice near 36.5 to 37 K (the old
    // two-point curve with the liquid's latent heat gave 33.5 K).
    let n2 = cs.substances.iter().find(|s| s.gas == "n2").unwrap();
    let t = ebm::frost_k(n2, 1.0);
    assert!((36.0..37.5).contains(&t), "{t}");
    // Mars's CO2 at 610 Pa: about 148 K.
    let co2 = cs.substances.iter().find(|s| s.gas == "co2").unwrap();
    let t = ebm::frost_k(co2, 610.0);
    assert!((146.0..150.0).contains(&t), "{t}");
}

#[test]
fn a_new_volatile_on_the_sheet_gets_frost_and_cloud_with_no_code() {
    // A fixture pack adds "xenoice", with a triple point, a Clausius-Clapeyron
    // curve and a colour: nothing else.
    let fx = pack(
        r#"{"format": "sse.pack/1", "id": "test.xenoice", "version": "0.1.0", "kind": ["physics"],
        "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"reference": {
          "sources": [{"id": "made-up", "text": "Fictional, for the W8 test"}],
          "substances": [{"id": "xenoice", "formula": {"xe": 1, "f": 2},
            "names": {"physics": "xenon difluoride", "plain": "xenoice", "story": "xenoice"},
            "phases": {
              "triple_point_k": {"v": 120.0, "unit": "K", "src": "made-up"},
              "triple_point_pa": {"v": 2000.0, "unit": "Pa", "src": "made-up"},
              "vapour_pressure": {"fit": "clausius-clapeyron", "coeffs": [2000.0, 120.0, 30000.0], "valid_k": [40.0, 120.0], "src": "made-up"},
              "density_solid": {"v": 4300.0, "unit": "kg m-3", "src": "made-up"}},
            "optical": {"condensed_colour_rgb": {"v": [0.8, 0.9, 1.0], "unit": "1", "src": "made-up"}}}]}}}"#,
    );
    let stack = [base_reference_pack(), crate::base_pack(), fx];
    let species = volatile_species(&stack).unwrap();
    assert!(species.iter().any(|s| s.gas == "xenoice"));
    let cs = ClimateSettings::from_packs(&stack).unwrap();
    // A cold airless-ish world of it: the climate's ground frost and its
    // cloud both find it.
    let mut c = crate::water_tests::pluto();
    c.label = "Xeno".into();
    c.gases = [("xenoice".to_string(), 1.0)].into_iter().collect();
    c.mean_molar_kg = 0.169;
    c.pressure_bar = 1.0e-3;
    c.t_surface_k = 80.0;
    c.flux_w_m2 = 5.0;
    let st = strengths::strengths(&cs, &c);
    let x = cs.substances.iter().find(|s| s.gas == "xenoice").unwrap();
    assert_eq!(st.condense_k, Some(ebm::frost_k(x, 100.0)));
    let g = glance(&cs, &c, 4500.0);
    assert!(g.condensables.iter().any(|n| n == "xenoice"), "{g:?}");
}
