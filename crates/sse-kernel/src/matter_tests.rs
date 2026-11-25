//! MP1: the master data sheet loads with the stack and `read_matter` reads it.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn pack(p: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
}
fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}
fn load(k: &mut crate::Kernel, top: &str) -> serde_json::Value {
    send(
        k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"), pack(top)]}),
    )
}

#[test]
fn a_sheet_with_five_mistakes_is_refused_with_five_messages() {
    let mut k = crate::Kernel::new();
    let r = load(&mut k, "tests/fixtures/packs/reference-five-errors.json");
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert_eq!(r["issues"].as_array().unwrap().len(), 5, "{r}");
}

#[test]
fn read_matter_gives_a_row_with_its_sources_and_packs() {
    let mut k = crate::Kernel::new();
    let a = load(&mut k, "tests/fixtures/packs/reference-toy.json");
    assert_eq!(a["type"], "stack_loaded", "{a}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "substances", "id": "h2o"}),
    );
    assert_eq!(r["type"], "matter", "{r}");
    let v = &r["value"];
    assert_eq!(v["packs"], serde_json::json!(["test.reference-toy"]));
    let mm = v["worked"]["molar_mass"]["v"].as_f64().unwrap();
    assert!((mm - 0.018015).abs() < 1e-9, "{mm}");
    // By id alone, and a listing.
    let r = send(&mut k, serde_json::json!({"op": "read_matter", "id": "o"}));
    assert_eq!(r["value"]["table"], "elements", "{r}");
    let r = send(&mut k, serde_json::json!({"op": "read_matter"}));
    assert_eq!(
        r["value"]["substances"],
        serde_json::json!(["h2", "h2o", "o2"]),
        "{r}"
    );
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "spells"}),
    );
    assert_eq!(r["type"], "error", "{r}");
    // The same stack hashes the same twice.
    let mut k2 = crate::Kernel::new();
    let b = load(&mut k2, "tests/fixtures/packs/reference-toy.json");
    assert_eq!(a["stack_hash"], b["stack_hash"]);
}

fn derive(k: &mut crate::Kernel, flow: &str, inputs: serde_json::Value) -> serde_json::Value {
    let r = send(
        k,
        serde_json::json!({"op": "derive", "flow": flow, "inputs": inputs}),
    );
    assert_eq!(r["type"], "derived", "{r}");
    r["values"].clone()
}

/// MP1c: with G doubled, the blocks that follow G give the right answer for
/// the new value; with the real sheet they give exactly today's numbers.
#[test]
fn doubling_g_shortens_the_year_by_root_two_and_doubles_gravity() {
    let orbit = serde_json::json!({
        "host_mass": {"value": 1, "unit": "m_sun"}, "body_mass": {"value": 1, "unit": "m_earth"},
        "semi_major_axis": {"value": 1, "unit": "au"}, "eccentricity": {"value": 0.0167}});
    let bulk = serde_json::json!({"mass": {"value": 1, "unit": "m_earth"}, "radius": {"value": 6371, "unit": "km"}});
    let base = || pack("packs/base-physics/pack.json");
    let mut real = crate::Kernel::new();
    let r = send(
        &mut real,
        serde_json::json!({"op": "load_stack", "packs": [base()]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let mut strong = crate::Kernel::new();
    let r = send(
        &mut strong,
        serde_json::json!({"op": "load_stack", "packs": [
        base(), pack("tests/fixtures/packs/reference-strong-gravity.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");

    let v = |vals: &serde_json::Value, q: &str| {
        vals[q]["value"]
            .as_f64()
            .unwrap_or_else(|| panic!("{q}: {vals}"))
    };
    let (a, b) = (
        derive(&mut real, "sse.base.orbit-basics", orbit.clone()),
        derive(&mut strong, "sse.base.orbit-basics", orbit),
    );
    let ratio = v(&b, "orbit.period") / v(&a, "orbit.period");
    assert!(
        (ratio - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12,
        "{ratio}"
    );
    let (a, b) = (
        derive(&mut real, "sse.base.planet-bulk", bulk.clone()),
        derive(&mut strong, "sse.base.planet-bulk", bulk),
    );
    let ratio = v(&b, "body.surface_gravity") / v(&a, "body.surface_gravity");
    assert!((ratio - 2.0).abs() < 1e-12, "{ratio}");
    // Nothing leaks: after the strong-gravity call, the real kernel still
    // gives the real year.
    assert_eq!(sse_core::constants::g(), sse_core::quantity::consts::G);
}

/// MP1c: the reach report puts every block in exactly one group, and every
/// declared block is a real atom.
#[test]
fn the_reach_report_lists_every_block_once() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-physics/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "G", "value": 1.33486e-10}),
    );
    assert_eq!(r["type"], "reach", "{r}");
    let mut seen = std::collections::BTreeSet::new();
    for g in ["follows", "estimated", "cannot_follow"] {
        for b in r[g].as_array().unwrap() {
            assert!(
                seen.insert(b["block"].as_str().unwrap().to_string()),
                "{b} twice"
            );
        }
    }
    let mut all: std::collections::BTreeSet<String> =
        k.atoms.decls().map(|d| d.id.clone()).collect();
    // DM1: the cosmos chain's stages are listed beside the atoms.
    all.extend(sse_atoms::reach::STAGES.iter().map(|(s, _)| s.to_string()));
    assert_eq!(seen, all);
    for (a, _) in sse_atoms::reach::DECLARED {
        assert!(all.contains(*a), "declared block {a} is not an atom");
    }
    let period = r["follows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["block"] == "sse.orbit.period")
        .unwrap();
    assert_eq!(period["uses_it"], true);
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "charm"}),
    );
    assert_eq!(r["type"], "error", "{r}");
}

/// MP1c (design 7A.6): a unit added on the sheet can be typed like any
/// built-in one.
#[test]
fn a_unit_from_the_sheet_can_be_typed() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
        pack("packs/base-physics/pack.json"), pack("tests/fixtures/packs/reference-toy.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let orbit = |a: serde_json::Value| {
        serde_json::json!({
        "host_mass": {"value": 1, "unit": "m_sun"}, "body_mass": {"value": 1, "unit": "m_earth"},
        "semi_major_axis": a, "eccentricity": {"value": 0.0}})
    };
    let by_league = derive(
        &mut k,
        "sse.base.orbit-basics",
        orbit(serde_json::json!({"value": 1.0e6, "unit": "league"})),
    );
    let by_m = derive(
        &mut k,
        "sse.base.orbit-basics",
        orbit(serde_json::json!({"value": 4.828032e9, "unit": "m"})),
    );
    assert_eq!(
        by_league["orbit.period"]["value"],
        by_m["orbit.period"]["value"]
    );
}

/// The shipped sheet (or the one `SSE_BASE_REFERENCE` names) loads in a
/// stack, its units included, and does not move a constant.
#[test]
fn the_base_sheet_loads_in_a_stack() {
    let path = std::env::var("SSE_BASE_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root().join("packs/base-reference/pack.json"));
    let Ok(s) = std::fs::read_to_string(&path) else {
        return;
    };
    let sheet: serde_json::Value = serde_json::from_str(&s).unwrap();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"), sheet]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let g = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "g"}),
    );
    assert_eq!(g["in_force"], g["real"], "{g}");
}

/// MP17: `phase_at` through the kernel, reading water's map (MP18).
#[test]
fn phase_at_answers_through_the_kernel() {
    let path = root().join("packs/base-reference/pack.json");
    let sheet: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"), sheet]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let at = |k: &mut crate::Kernel, p: f64, t: f64| {
        send(
            k,
            serde_json::json!({"op": "phase_at", "substance": "h2o", "p_pa": p, "t_k": t}),
        )
    };
    let r = at(&mut k, 101_325.0, 300.0);
    assert_eq!(r["type"], "matter", "{r}");
    assert_eq!(r["value"]["phase"], "h2o.liquid", "{r}");
    assert_eq!(r["value"]["from"], "table", "{r}");
    assert_eq!(r["value"]["grade"], "measured", "{r}");
    assert_eq!(at(&mut k, 101_325.0, 250.0)["value"]["phase"], "h2o.ice_ih");
    // Ice VI under Ganymede's ocean, by the same question.
    assert_eq!(at(&mut k, 1.0e9, 270.0)["value"]["phase"], "h2o.ice_vi");
    let r = send(
        &mut k,
        serde_json::json!({"op": "phase_at", "substance": "unobtainium", "p_pa": 1.0, "t_k": 300.0}),
    );
    assert_eq!(r["type"], "error", "{r}");
}

/// MP1c part 2: with twice G the Sun is 16 times as bright and twice as
/// wide by stellar homology, its lifetime a sixteenth, and each of those
/// values carries the ESTIMATED law. The real kernel stamps nothing. (At
/// the real age of 4.57 Gyr such a Sun would be gone: 0.625 Gyr of life.)
#[test]
fn doubling_g_makes_the_sun_estimated_sixteen_times_brighter() {
    let sun = serde_json::json!({"mass": {"value": 1, "unit": "m_sun"}, "age": {"value": 0.1, "unit": "gyr"}});
    let base = || pack("packs/base-physics/pack.json");
    let mut real = crate::Kernel::new();
    send(
        &mut real,
        serde_json::json!({"op": "load_stack", "packs": [base()]}),
    );
    let mut strong = crate::Kernel::new();
    send(
        &mut strong,
        serde_json::json!({"op": "load_stack", "packs": [
        base(), pack("tests/fixtures/packs/reference-strong-gravity.json")]}),
    );
    let a = derive(&mut real, "sse.base.star-main-sequence", sun.clone());
    let b = derive(&mut strong, "sse.base.star-main-sequence", sun);
    let ratio = |q: &str| b[q]["value"].as_f64().unwrap() / a[q]["value"].as_f64().unwrap();
    assert!((ratio("star.luminosity") - 16.0).abs() < 1e-9, "{b}");
    assert!((ratio("star.radius") - 2.0).abs() < 1e-12, "{b}");
    assert!(
        (ratio("star.main_sequence_lifetime") - 1.0 / 16.0).abs() < 1e-12,
        "{b}"
    );
    // T_eff ∝ (L / R²)^¼ = (16 / 4)^¼ = √2.
    assert!(
        (ratio("star.effective_temperature") - std::f64::consts::SQRT_2).abs() < 1e-9,
        "{b}"
    );
    for q in [
        "star.luminosity",
        "star.radius",
        "star.effective_temperature",
        "star.main_sequence_lifetime",
    ] {
        let e = b[q]["estimated"]
            .as_array()
            .unwrap_or_else(|| panic!("{q}: {b}"));
        assert!(
            e.iter().any(|l| l.as_str().unwrap().contains("Adams 2008")),
            "{q}: {b}"
        );
        assert!(a[q].get("estimated").is_none(), "{q}: {a}");
    }
}

fn lookup_kernel(edit: bool) -> crate::Kernel {
    let mut packs = vec![
        pack("packs/base-physics/pack.json"),
        pack("packs/base-reference/pack.json"),
        pack("tests/fixtures/packs/lookup-flows.json"),
    ];
    if edit {
        packs.push(pack("tests/fixtures/packs/lookup-edit.json"));
    }
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": packs}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    k
}

/// MP17a: the Look up block reads the sheet. Water's latent heat is the
/// sheet's value; the density at 1 GPa and 270 K is ice VI's own; a row
/// without the property answers known = false; doubling the latent heat in
/// a pack moves only the flow that reads it; "used by" lists the flows that
/// wire a Look up block to the row.
#[test]
fn the_look_up_block_reads_the_sheet() {
    let mut k = lookup_kernel(false);
    let m = serde_json::json!({"mass": {"value": 1.0}});
    let heat = derive(&mut k, "test.water-boil-heat", m.clone());
    assert_eq!(heat["test.latent"]["value"], 2.468e6, "{heat}");
    let at = |k: &mut crate::Kernel, p: f64, t: f64| {
        derive(
            k,
            "test.water-density-at",
            serde_json::json!({"p": {"value": p}, "t": {"value": t}}),
        )
    };
    let ice6 = at(&mut k, 1.0e9, 270.0);
    let d6 = ice6["test.density"]["value"].as_f64().unwrap();
    assert!(
        (1250.0..1400.0).contains(&d6),
        "ice VI is about 1.3 g/cm3: {ice6}"
    );
    let liquid = at(&mut k, 101_325.0, 300.0)["test.density"]["value"]
        .as_f64()
        .unwrap();
    assert!((990.0..1010.0).contains(&liquid), "{liquid}");
    let none = derive(&mut k, "test.iron-density", m.clone());
    assert_eq!(none["test.strength"]["value"], 0.0, "{none}");

    // Change the sheet: only the flow that reads the changed value moves.
    let mut e = lookup_kernel(true);
    assert_eq!(
        derive(&mut e, "test.water-boil-heat", m.clone())["test.latent"]["value"],
        4.936e6
    );
    assert_eq!(at(&mut e, 1.0e9, 270.0), ice6);
    assert_eq!(derive(&mut e, "test.iron-density", m), none);

    // Used by, from the wiring.
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "substances", "id": "h2o"}),
    );
    let used: Vec<&str> = r["value"]["used_by"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["flow"].as_str().unwrap())
        .collect();
    assert_eq!(
        used,
        [
            "test.iron-density",
            "test.water-boil-heat",
            "test.water-density-at"
        ],
        "{r}"
    );
    let r = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "substances", "id": "co2"}),
    );
    assert_eq!(r["value"]["used_by"], serde_json::json!([]), "{r}");
}

/// A Look up block naming a row the sheet lacks, or with no sheet in the
/// stack, refuses to load, saying why.
#[test]
fn a_look_up_block_with_no_row_or_no_sheet_is_refused() {
    let mut flows = pack("tests/fixtures/packs/lookup-flows.json");
    flows["provides"]["flows"][0]["nodes"][0]["config"]["what"] = "unobtainium".into();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"),
            pack("packs/base-reference/pack.json"), flows]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(r.to_string().contains("unobtainium"), "{r}");
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"),
            pack("tests/fixtures/packs/lookup-flows.json")]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(r.to_string().contains("master data sheet"), "{r}");
}

/// DM1: the Cosmos group on the sheet. Unedited, the cosmos chain gives
/// exactly what it gave before the parameters moved to the sheet; with twice
/// the dark matter it follows (expansion, ages, epochs), the reach report
/// says so, and nothing leaks into the next call.
#[test]
fn more_dark_matter_follows_through_the_cosmos_chain() {
    let chart = |packs: Vec<serde_json::Value>| {
        let mut k = crate::Kernel::new();
        if !packs.is_empty() {
            let r = send(
                &mut k,
                serde_json::json!({"op": "load_stack", "packs": packs}),
            );
            assert_eq!(r["type"], "stack_loaded", "{r}");
        }
        let r = send(
            &mut k,
            serde_json::json!({"op": "open_universe", "id": "g3", "seed": "g3"}),
        );
        assert_eq!(r["type"], "universe_opened", "{r}");
        let c = send(
            &mut k,
            serde_json::json!({"op": "galaxy_chart", "halo": "milky-way-like"}),
        );
        assert_eq!(c["type"], "galaxy_charted", "{c}");
        (k, c)
    };
    let base = || {
        vec![
            pack("packs/base-physics/pack.json"),
            pack("packs/base-reference/pack.json"),
        ]
    };
    let (_, none) = chart(vec![]);
    let (mut real, sheet) = chart(base());
    assert_eq!(none, sheet, "an unedited sheet changes nothing");
    let mut more = base();
    more.push(pack("tests/fixtures/packs/reference-more-dark-matter.json"));
    let (mut k, dark) = chart(more);
    assert_ne!(dark["chart"]["epochs"], sheet["chart"]["epochs"]);
    assert_ne!(dark["chart"]["history"], sheet["chart"]["history"]);

    let r = send(
        &mut k,
        serde_json::json!({"op": "read_reach", "constant": "Omega_c"}),
    );
    assert_eq!(r["type"], "reach", "{r}");
    assert_eq!(r["real"], 0.2607228);
    assert_eq!(r["in_force"], 0.5214456);
    let find = |g: &str, b: &str| {
        r[g].as_array()
            .unwrap()
            .iter()
            .find(|x| x["block"] == b)
            .cloned()
            .unwrap_or_else(|| panic!("{b} not in {g}: {r}"))
    };
    // The expansion reads Omega_m, which rests on Omega_c.
    assert_eq!(find("follows", "sse.cosmos.expansion")["uses_it"], true);
    assert_eq!(find("follows", "sse.cosmos.baryon_share")["uses_it"], true);
    assert!(find("estimated", "sse.cosmos.major_mergers")["law"]
        .as_str()
        .unwrap()
        .contains("Fakhouri"));
    find("cannot_follow", "sse.cosmos.galaxy_regulator");
    // The worked-out rows move with it on the sheet.
    let m = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "constants", "id": "Omega_m"}),
    );
    let om = m["value"]["worked"]["value"]["v"].as_f64().unwrap();
    assert!(
        (om - (0.04897 + 0.5214456 + 0.0014072)).abs() < 1e-12,
        "{m}"
    );
    // The real kernel still reads Planck's dark matter, and the handle is back.
    let r = send(
        &mut real,
        serde_json::json!({"op": "read_reach", "constant": "Omega_c"}),
    );
    assert_eq!(r["in_force"], r["real"], "{r}");
    assert_eq!(
        sse_core::constants::cosmos(),
        sse_core::constants::Cosmos::REAL
    );
}

/// DM1: dark matter and dark energy carry the "theoretical" mark with the
/// line that says what would change the picture, and the Cosmos page's
/// notes (the particle route) come with the listing.
#[test]
fn dark_matter_is_marked_theoretical_on_the_sheet() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"), pack("packs/base-reference/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    for id in [
        "Omega_c",
        "Omega_Lambda",
        "w0",
        "wa",
        "m_wdm",
        "sigma_dm",
        "m_fdm",
    ] {
        let m = send(
            &mut k,
            serde_json::json!({"op": "read_matter", "table": "constants", "id": id}),
        );
        let row = &m["value"]["row"];
        assert_eq!(row["mark"], "theoretical", "{id}: {m}");
        assert!(row["mark_note"].as_str().unwrap().len() > 40, "{id}");
        assert_eq!(row["group"], "cosmos");
    }
    // A group note's sources read by id (MP10 asks for their text and DOI).
    let s = send(
        &mut k,
        serde_json::json!({"op": "read_matter", "table": "sources", "id": "clowe2006"}),
    );
    assert_eq!(s["value"]["row"]["doi"], "10.1086/508162", "{s}");
    for id in ["H0", "Omega_b", "sigma8", "T_cmb"] {
        let m = send(
            &mut k,
            serde_json::json!({"op": "read_matter", "table": "constants", "id": id}),
        );
        assert!(m["value"]["row"]["mark"].is_null(), "{id} is measured: {m}");
    }
    let l = send(&mut k, serde_json::json!({"op": "read_matter"}));
    let cosmos = &l["value"]["groups"]["cosmos"];
    assert_eq!(cosmos["name"], "Cosmos", "{l}");
    assert!(cosmos["notes"][0]["text"]
        .as_str()
        .unwrap()
        .contains("particle route"));
}

/// DM6: with G drifting (fictional fixture), a year's length at each date
/// follows Kepler's factor P ∝ G^(-1/2), read from the stack's sheet by
/// date; today's year is the real one, and the stack without the law gives
/// the same handle at every date.
#[test]
fn a_drifting_g_changes_the_year_by_keplers_factor_at_each_date() {
    let sheet = |extra: Option<&str>| {
        let mut packs = vec![serde_json::from_value::<sse_pack::PackManifest>(pack(
            "packs/base-reference/pack.json",
        ))
        .unwrap()];
        if let Some(p) = extra {
            packs.push(serde_json::from_value(pack(p)).unwrap());
        }
        sse_reference::Reference::from_packs(&packs).unwrap()
    };
    let atoms = sse_atoms::atoms();
    let period = atoms
        .iter()
        .find(|a| a.decl().id == "sse.orbit.period")
        .unwrap();
    let year = |r: &sse_reference::Reference, t: f64| {
        sse_core::constants::with(r.handle_on(t), || {
            period
                .eval(&[1.495_978_707e11, 1.988_409_870_698_051e30], &[])
                .unwrap()[0]
        })
    };
    let real = sheet(None);
    let drift = sheet(Some("tests/fixtures/packs/reference-drifting-g.json"));
    let now = year(&real, 13.8);
    assert_eq!(year(&drift, 13.8), now);
    assert_eq!(year(&real, 12.8), now, "no law, no history");
    for (t, g_ratio) in [(12.8, 1.1), (13.3, 1.05), (8.8, 1.5)] {
        let ratio = year(&drift, t) / now;
        assert!(
            (ratio - 1.0 / sse_core::math::sqrt(g_ratio)).abs() < 1e-12,
            "{t}: {ratio}"
        );
    }
}

/// ND3: an Equation reads G through a visible Look up block, so a what-if
/// that changes G on the sheet changes its result with no edit to the pack;
/// with the real sheet it gives the formula's own value.
#[test]
fn an_equation_follows_a_looked_up_constant() {
    let speed = |edit: bool| {
        let mut packs = vec![
            pack("packs/base-physics/pack.json"),
            pack("packs/base-reference/pack.json"),
            pack("tests/fixtures/packs/equation-flows.json"),
        ];
        if edit {
            let mut g = pack("tests/fixtures/packs/reference-strong-gravity.json");
            g["provides"]["reference"]["constants"][0]["value"]["replaces"] =
                "sse.base-reference".into();
            packs.push(g);
        }
        let mut k = crate::Kernel::new();
        let r = send(
            &mut k,
            serde_json::json!({"op": "load_stack", "packs": packs}),
        );
        assert_eq!(r["type"], "stack_loaded", "{r}");
        let v = derive(
            &mut k,
            "test.escape-speed",
            serde_json::json!({"mass": {"value": 1, "unit": "m_earth"}, "radius": {"value": 6371, "unit": "km"}}),
        );
        v["test.escape_speed"]["value"].as_f64().unwrap()
    };
    let (real, strong) = (speed(false), speed(true));
    let g = sse_core::quantity::consts::G;
    let want = sse_core::math::sqrt(2.0 * g * (sse_core::quantity::consts::GM_EARTH / g) / 6.371e6);
    assert_eq!(real, want);
    assert!(
        (strong / real - std::f64::consts::SQRT_2).abs() < 1e-12,
        "{strong} / {real}"
    );
    // An Equation with a letter typed in its formula, or a missing source, is
    // refused when the stack loads.
    let mut p = pack("tests/fixtures/packs/equation-flows.json");
    p["provides"]["flows"][0]["nodes"][1]["config"]["formula"] =
        "v = sqrt(2 * G * M / 6371000)".into();
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [
            pack("packs/base-physics/pack.json"),
            pack("packs/base-reference/pack.json"), p]}),
    );
    assert_eq!(r["type"], "stack_rejected", "{r}");
    assert!(r.to_string().contains("give this letter a source"), "{r}");
}

/// DM6 in the kernel: a block asked at a date reads a constant with a
/// history at that date. With G drifting (10 % stronger a billion years
/// ago), a year asked a billion years back is shorter by Kepler's factor;
/// asked now, or on a stack with no law at any date, it is today's year.
#[test]
fn a_block_asked_at_a_date_reads_g_at_that_date() {
    let orbit = serde_json::json!({
        "host_mass": {"value": 1, "unit": "m_sun"}, "body_mass": {"value": 1, "unit": "m_earth"},
        "semi_major_axis": {"value": 1, "unit": "au"}, "eccentricity": {"value": 0.0167}});
    let base = || pack("packs/base-physics/pack.json");
    let kernel = |extra: bool| {
        let mut k = crate::Kernel::new();
        let mut packs = vec![pack("packs/base-reference/pack.json"), base()];
        if extra {
            packs.push(pack("tests/fixtures/packs/reference-drifting-g.json"));
        }
        let r = send(
            &mut k,
            serde_json::json!({"op": "load_stack", "packs": packs}),
        );
        assert_eq!(r["type"], "stack_loaded", "{r}");
        k
    };
    let year = |k: &mut crate::Kernel, at_s: Option<i64>| {
        let mut q =
            serde_json::json!({"op": "derive", "flow": "sse.base.orbit-basics", "inputs": orbit});
        if let Some(s) = at_s {
            q["at_s"] = s.into();
        }
        let r = send(k, q);
        assert_eq!(r["type"], "derived", "{r}");
        r["values"]["orbit.period"]["value"].as_f64().unwrap()
    };
    let gyr = (1.0e9 * crate::clock::JULIAN_YEAR_S) as i64;
    let mut real = kernel(false);
    let mut drift = kernel(true);
    let now = year(&mut real, None);
    assert_eq!(year(&mut real, Some(-gyr)), now, "no law, no history");
    assert_eq!(year(&mut drift, None), now, "today's G is the real one");
    let back = year(&mut drift, Some(-gyr)) / now;
    assert!(
        (back - 1.0 / sse_core::math::sqrt(1.1)).abs() < 1e-9,
        "{back}"
    );
    // On a campaign clock: a billion years before its first now.
    send(
        &mut drift,
        serde_json::json!({"op": "open_universe", "id": "dated", "seed": "dated"}),
    );
    let start = 5_000_000;
    let r = send(
        &mut drift,
        serde_json::json!({"op": "set_campaign_now", "at_s": start}),
    );
    assert_ne!(r["type"], "error", "{r}");
    let back = year(&mut drift, Some(start - gyr)) / now;
    assert!(
        (back - 1.0 / sse_core::math::sqrt(1.1)).abs() < 1e-9,
        "{back}"
    );
    assert_eq!(year(&mut drift, Some(start)), now);
    // And nothing leaks out of the dated call.
    assert_eq!(sse_core::constants::g(), sse_core::quantity::consts::G);
}
