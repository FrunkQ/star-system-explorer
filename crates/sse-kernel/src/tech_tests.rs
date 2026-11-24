//! FT2 checks (future tech design 5 to 8): a level set on the setting and
//! inherited, exceptions, "Develop it" as one dated event, maturity acting on
//! the figure on both sides of a part's home level (5.2), and fictional by
//! ceiling, with fuel-specific ceilings.

use crate::Kernel;
use std::path::PathBuf;

fn ask(k: &mut Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn base_tech() -> serde_json::Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packs/base-tech/pack.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

const YR: i64 = 31_557_600;

fn open() -> Kernel {
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [base_tech()]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "ft", "seed": "ft"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    for (id, parent) in [("vesh", None), ("third-fleet", Some("u:ft/actor:vesh"))] {
        let mut q = serde_json::json!({"op": "add_actor", "id": id, "name": id, "kind": "polity", "at_s": 0});
        if let Some(p) = parent {
            q["parent"] = p.into();
        }
        let r = ask(&mut k, q);
        assert_eq!(r["type"], "actor_added", "{r}");
    }
    // The setting is "Sol today", level 4.
    let r = ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": "u:ft", "level": 4, "at_s": 0}),
    );
    assert_eq!(r["type"], "tech_set", "{r}");
    k
}

fn reading(
    k: &mut Kernel,
    who: &str,
    needs: serde_json::Value,
    at_s: Option<i64>,
) -> serde_json::Value {
    let mut q = serde_json::json!({"op": "tech_reading", "holder": who,
        "parts": [{"part": "structure", "needs": needs}]});
    if let Some(t) = at_s {
        q["at_s"] = t.into();
    }
    let r = ask(k, q);
    assert_eq!(r["type"], "tech_availability", "{r}");
    r["reading"].clone()
}

#[test]
fn a_level_on_the_setting_is_inherited_and_exceptions_are_dated() {
    let mut k = open();
    let fleet = "u:ft/actor:vesh/actor:third-fleet";
    let t = ask(
        &mut k,
        serde_json::json!({"op": "tech_of", "holder": fleet}),
    );
    assert_eq!(t["type"], "tech_read", "{t}");
    assert_eq!(t["tech"]["level"][0], 4);
    assert_eq!(t["tech"]["level"][1]["by"], "u:ft");
    let holds = t["tech"]["holds"].to_string();
    assert!(
        holds.contains("pbo-fibre") && !holds.contains("fusion-torch"),
        "{holds}"
    );
    // The Vesh never took up fission; the fleet inherits that.
    ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": "u:ft/actor:vesh", "lose": ["fission-reactor"], "at_s": 0}),
    );
    let t = ask(
        &mut k,
        serde_json::json!({"op": "tech_of", "holder": fleet}),
    );
    assert_eq!(
        t["tech"]["lost"]["fission-reactor"]["by"],
        "u:ft/actor:vesh"
    );
    assert!(!t["tech"]["holds"].to_string().contains("fission-reactor"));
    // Unknown advances and levels off the ladder are refused.
    let r = ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": "u:ft/actor:vesh", "gain": ["warp"], "at_s": 0}),
    );
    assert_eq!(r["code"], "bad_tech", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": "u:ft/actor:vesh", "level": 9, "at_s": 0}),
    );
    assert_eq!(r["code"], "bad_tech", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": "u:ft/sys:1", "level": 3, "at_s": 0}),
    );
    assert_eq!(r["code"], "bad_tech", "{r}");
}

#[test]
fn the_availability_reading_never_says_no() {
    let mut k = open();
    let vesh = "u:ft/actor:vesh";
    // PBO fibre is theirs at level 4.
    let r = reading(
        &mut k,
        vesh,
        serde_json::json!({"strength_for_weight": 3.0e6}),
        None,
    );
    assert_eq!(r["reading"], "can build", "{r}");
    assert_eq!(r["needs"][0]["advance"], "pbo-fibre");
    assert_eq!(r["leans_on"][0]["ink"], "built");
    // A beanstalk on Earth needs nanotube cable, two levels short. A
    // material loses 5 times per level (47 to 1.9 MN m/kg), which is worse
    // than the PBO fibre they hold, so the first cable is floored at PBO.
    let r = reading(
        &mut k,
        vesh,
        serde_json::json!({"strength_for_weight": 3.0e7}),
        None,
    );
    assert_eq!(r["reading"], "short", "{r}");
    let n = &r["needs"][0];
    assert_eq!(n["levels_short"], 2);
    assert_eq!(n["maturity"], "worse_by");
    assert_eq!(n["figure_now"], 3_718_000.0);
    assert_eq!(n["floored_at"], "PBO fibre");
    let size = n["size_factor"].as_f64().unwrap();
    assert!((size - 47_015_000.0 / 3_718_000.0).abs() < 1e-9, "{size}");
    assert_eq!(r["part_sizes"]["structure"], size);
    assert_eq!(r["leans_on"][0]["ink"], "not yet built");
    assert_eq!(r["leans_on"][0]["family"], "material");
    assert!(
        n["words"]
            .as_str()
            .unwrap()
            .starts_with("2 levels short: Woven carbon cable, no better yet than their PBO fibre"),
        "{r}"
    );
    // Develop it: one dated event; before it the reading is as it was.
    let r = ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": vesh, "gain": ["nanotube-cable"],
        "reason": "the Vesh gain woven carbon cable", "at_s": 2310 * YR}),
    );
    assert_eq!(r["type"], "tech_set", "{r}");
    let after = reading(
        &mut k,
        vesh,
        serde_json::json!({"strength_for_weight": 3.0e7}),
        Some(2311 * YR),
    );
    assert_eq!(after["reading"], "can build", "{after}");
    let before = reading(
        &mut k,
        vesh,
        serde_json::json!({"strength_for_weight": 3.0e7}),
        Some(2309 * YR),
    );
    assert_eq!(before["reading"], "short");
    // A ringworld floor: past the bond ceiling, fictional by arithmetic, and
    // past anything on the ladder, in numbers.
    let r = reading(
        &mut k,
        vesh,
        serde_json::json!({"strength_for_weight": 1.5e12}),
        None,
    );
    assert_eq!(r["reading"], "past known", "{r}");
    assert!(r["fictional"].as_bool().unwrap());
    assert!(
        r["needs"][0]["fictional"]
            .as_str()
            .unwrap()
            .contains("chemical bond energy"),
        "{r}"
    );
    let times = r["needs"][0]["times_their_best"].as_f64().unwrap();
    assert!((times - 1.5e12 / 47_015_000.0).abs() < 1.0, "{times}");
}

#[test]
fn maturity_acts_on_the_figure() {
    let mut k = open();
    let vesh = "u:ft/actor:vesh";
    let drive = |k: &mut Kernel, needs: serde_json::Value, fuel: Option<&str>| {
        let mut p = serde_json::json!({"part": "drive", "needs": needs});
        if let Some(f) = fuel {
            p["fuel"] = f.into();
        }
        let r = ask(
            k,
            serde_json::json!({"op": "tech_reading", "holder": vesh, "parts": [p]}),
        );
        assert_eq!(r["type"], "tech_availability", "{r}");
        r["reading"].clone()
    };
    // A fusion torch one level short: an engine is 7 times worse per level,
    // still far past their nuclear thermal rocket, so no floor; its fuel's
    // energy is physics and never moves.
    let r = drive(
        &mut k,
        serde_json::json!({"exhaust_km_s": 300.0, "fuel_j_per_kg": 3.0e14}),
        None,
    );
    let ex = r["needs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["figure"] == "exhaust_km_s")
        .unwrap()
        .clone();
    assert_eq!(ex["reading"], "short", "{r}");
    assert_eq!(ex["maturity"], "worse_by");
    assert!((ex["figure_now"].as_f64().unwrap() - 347.499 / 7.0).abs() < 1e-9);
    assert!((ex["size_factor"].as_f64().unwrap() - 7.0).abs() < 1e-9);
    assert!(ex.get("floored_at").is_none(), "{ex}");
    let fuel = r["needs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["figure"] == "fuel_j_per_kg")
        .unwrap()
        .clone();
    assert_eq!(fuel["maturity"], "fixed", "{fuel}");
    assert_eq!(fuel["size_factor"], 1.0);
    // 6 km/s is past the hydrogen-oxygen ceiling, but that ceiling is for
    // that fuel only: a nuclear thermal rocket gives it, nothing fictional.
    let r = drive(&mut k, serde_json::json!({"exhaust_km_s": 6.0}), None);
    assert_eq!(r["reading"], "can build", "{r}");
    assert_eq!(r["needs"][0]["advance"], "nuclear-thermal");
    assert_eq!(r["fictional"], false, "{r}");
    // The same need from a hydrogen-oxygen engine breaks its chemistry.
    let r = drive(
        &mut k,
        serde_json::json!({"exhaust_km_s": 6.0}),
        Some("hydrogen-oxygen"),
    );
    assert_eq!(r["fictional"], true, "{r}");
}

#[test]
fn parts_shrink_beyond_their_home_level() {
    let mut k = open();
    let vesh = "u:ft/actor:vesh";
    ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": vesh, "level": 6, "at_s": 0}),
    );
    // The hydrogen-oxygen rocket's home is level 3. Three beyond, its exhaust
    // would be 1.5 cubed better, but it stops at the ceiling for its own fuel
    // (5.18 km/s), not at the nuclear rocket's mark; the part shrinks to match.
    let r = ask(
        &mut k,
        serde_json::json!({"op": "tech_reading", "holder": vesh,
        "parts": [{"part": "drive", "needs": {"exhaust_km_s": 4.0}}]}),
    );
    let n = &r["reading"]["needs"][0];
    assert_eq!(n["advance"], "chemical-rocket", "{r}");
    assert_eq!(n["maturity"], "improved");
    assert_eq!(n["figure_now"], 5.18);
    assert!((n["size_factor"].as_f64().unwrap() - 4.565 / 5.18).abs() < 1e-9);
    // Parts read through an unknown figure are refused, not guessed.
    let r = ask(
        &mut k,
        serde_json::json!({"op": "tech_reading", "holder": vesh,
        "parts": [{"part": "drive", "needs": {"warp_factor": 9.0}}]}),
    );
    assert_eq!(r["code"], "bad_tech", "{r}");
}

#[test]
fn a_fictional_device_keeps_the_size_rule_and_says_where() {
    let mut k = Kernel::new();
    let setting = serde_json::json!({"format": "sse.pack/1", "id": "t.torch", "version": "0.1.0",
        "kind": ["universe"], "kernel_range": "^0.1", "licence": "Apache-2.0",
        "provides": {"other": {"tech": {"advances": [{"id": "reactionless", "name": "Reactionless drive",
            "level": 7, "sets": {"exhaust_km_s": 100000.0}, "part": "drive", "mark": "fictional",
            "where": {"momentum": "the nearest large mass"}}]}}}});
    let r = ask(
        &mut k,
        serde_json::json!({"op": "load_stack", "packs": [base_tech(), setting]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    ask(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "ft", "seed": "ft"}),
    );
    ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": "u:ft", "level": 4, "at_s": 0}),
    );
    let r = reading(
        &mut k,
        "u:ft",
        serde_json::json!({"exhaust_km_s": 50000.0}),
        None,
    );
    let n = &r["needs"][0];
    assert_eq!(n["advance"], "reactionless", "{r}");
    assert_eq!(n["family"], "fictional");
    assert_eq!(n["maturity"], "size");
    assert_eq!(n["size_factor"], 1000.0);
    assert_eq!(
        r["leans_on"][0]["where"]["momentum"],
        "the nearest large mass"
    );
}

#[test]
fn ceilings_for_one_fuel_and_electric_drives() {
    // RK2: hydrogen-oxygen's exhaust ceiling holds for that fuel only, so a
    // drive needing an ion thruster's exhaust is not fictional; a "drive"
    // need still reads the chemical rocket, not the electric drives that
    // meet the same exhaust speed but also need a power plant.
    let mut k = open();
    let vesh = "u:ft/actor:vesh";
    ask(
        &mut k,
        serde_json::json!({"op": "set_tech", "holder": vesh, "level": 3, "at_s": 0}),
    );
    let r = ask(
        &mut k,
        serde_json::json!({"op": "tech_reading", "holder": vesh,
        "parts": [{"part": "electric drive", "needs": {"exhaust_km_s": 30.0}}]}),
    );
    let n = &r["reading"]["needs"][0];
    assert!(n.get("fictional").is_none(), "{r}");
    assert_eq!(n["advance"], "ion-thruster", "{r}");
    let r = ask(
        &mut k,
        serde_json::json!({"op": "tech_reading", "holder": vesh,
        "parts": [{"part": "drive", "needs": {"exhaust_km_s": 4.0}}]}),
    );
    assert_eq!(
        r["reading"]["needs"][0]["advance"], "chemical-rocket",
        "{r}"
    );
    // Past light speed is still fictional.
    let r = ask(
        &mut k,
        serde_json::json!({"op": "tech_reading", "holder": vesh,
        "parts": [{"part": "drive", "needs": {"exhaust_km_s": 4.0e5}}]}),
    );
    assert!(r["reading"]["needs"][0]["fictional"].is_string(), "{r}");
}
