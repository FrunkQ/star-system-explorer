//! RK6: the Paper spacecraft pack (`store-packs/paper-spacecraft`, built by
//! `scripts/build-paper-spacecraft.py`) stacks on base-tech, its ships close
//! their power books and fly in Sol, and its drives read "not yet built".
//! Trip times are compared with what each study says (the RK7 idea): the
//! tolerances and the reasons for each difference are in
//! `handover/packages/RK6.md`.

use serde_json::{json, Value as J};
use sse_kernel::Kernel;
use std::path::PathBuf;

fn read(rel: &str) -> J {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn ask(k: &mut Kernel, v: J) -> J {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn pack() -> J {
    read("store-packs/paper-spacecraft/pack.json")
}

fn stacked() -> Kernel {
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        json!({"op": "load_stack", "packs": [read("packs/base-tech/pack.json"), pack()]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    k
}

fn ships(p: &J) -> Vec<(String, J)> {
    p["provides"]["other"]["nav_ships"]["ships"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Every ship has a recipe that cites its study, every engine it mounts is in
/// the pack, and its books close: these studies did their sums.
#[test]
fn every_ship_closes_its_power_books() {
    let p = pack();
    let o = &p["provides"]["other"];
    let mut k = stacked();
    let ss = ships(&p);
    assert_eq!(ss.len(), 8);
    for (id, ship) in &ss {
        let rc = &o["recipes"][id];
        assert_eq!(rc["mark"], "not yet built", "{id}");
        for s in rc["src"].as_array().unwrap() {
            assert!(o["sources"][s.as_str().unwrap()].is_string(), "{id}: {s}");
        }
        for m in ship["engines"].as_array().unwrap() {
            assert!(
                o["engines"][m["engine"].as_str().unwrap()].is_object(),
                "{id}"
            );
        }
        let r = ask(&mut k, json!({"op": "ship_power", "ship": ship}));
        assert_eq!(r["type"], "ship_power_is", "{id}: {r}");
        assert_eq!(r["power"]["balance"]["ok"], true, "{id}: {r}");
        assert_eq!(r["power"]["fictional"], false, "{id}");
        // The fitted engines, not the typed figure, set the drive; the
        // builder wrote the typed one from the same numbers.
        let g = r["power"]["accel_g"].as_f64().unwrap();
        let typed = ship["accel_g"].as_f64().unwrap();
        assert!(
            (g - typed).abs() <= 1e-4 * typed,
            "{id}: {g} against {typed}"
        );
    }
    // Discovery II's two settings are one 4,830 MW jet (Williams 2005, Table 2).
    let jet = |id: &str| {
        let e = &o["engines"][id];
        0.5 * e["thrust_n"].as_f64().unwrap() * e["exhaust_km_s"].as_f64().unwrap() * 1000.0
    };
    for id in ["paper.discovery-ii-jupiter", "paper.discovery-ii-saturn"] {
        assert!((jet(id) - 4.83e9).abs() < 0.005 * 4.83e9, "{id}");
    }
}

/// The open-cycle gas core baseline as printed (5200 s and 50,000 lb from a
/// 5,000 MW reactor) carries more jet than its reactor makes. The pack keeps
/// the figures as written and the balance check says so.
#[test]
fn the_open_cycle_baseline_does_not_close_as_printed() {
    let p = pack();
    let e = &p["provides"]["other"]["engines"]["paper.open-cycle-gas-core"];
    let jet = 0.5 * e["thrust_n"].as_f64().unwrap() * e["exhaust_km_s"].as_f64().unwrap() * 1000.0;
    assert!(jet > 5.0e9 && jet < 6.0e9, "{jet}");
    let mut k = stacked();
    let ship = json!({"name": "Gas core test stage", "accel_g": 0.0, "exhaust_km_s": 1.0,
        "wet_t": 500.0, "dry_t": 200.0,
        "power": [{"id": "core", "kind": "gas core", "electric_w": 5.0e9}],
        "engines": [{"engine": {"name": "open cycle as electric", "thrust_n": e["thrust_n"],
            "exhaust_km_s": e["exhaust_km_s"], "energy_from": "electric", "electric_w": 5.0e9}}]});
    let r = ask(&mut k, json!({"op": "ship_power", "ship": ship}));
    assert_eq!(r["power"]["balance"]["ok"], false, "{r}");
    assert_eq!(r["power"]["fictional"], true);
}

/// Each drive leans on an advance, and for the pack's own drives the GM's ink
/// is "not yet built", for a setting at the top of the ladder. (The DRA 5.0
/// ship leans on base-tech's nuclear thermal rocket, which was ground-tested.)
#[test]
fn every_drive_reads_not_yet_built() {
    let p = pack();
    let o = &p["provides"]["other"];
    let mut k = stacked();
    let r = ask(
        &mut k,
        json!({"op": "open_universe", "id": "ps", "seed": "ps"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    let r = ask(
        &mut k,
        json!({"op": "set_tech", "holder": "u:ps", "level": 6, "at_s": 0}),
    );
    assert_eq!(r["type"], "tech_set", "{r}");
    for (id, ship) in ships(&p) {
        let m = &ship["engines"][0]["engine"];
        let e = &o["engines"][m.as_str().unwrap()];
        let part = if e["energy_from"] == "electric" {
            "electric drive"
        } else {
            "drive"
        };
        let need = e["exhaust_km_s"].as_f64().unwrap() * 0.99;
        let r = ask(
            &mut k,
            json!({"op": "tech_reading", "holder": "u:ps",
                "parts": [{"part": part, "needs": {"exhaust_km_s": need}}]}),
        );
        assert_eq!(r["type"], "tech_availability", "{id}: {r}");
        let leans = &r["reading"]["leans_on"][0];
        let wants: Vec<&str> = o["recipes"][&id]["advances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap())
            .collect();
        println!("{id}: leans on {} ({})", leans["advance"], leans["ink"]);
        if wants
            .iter()
            .all(|a| a.starts_with("paper.") || *a == "fusion-torch")
        {
            assert_eq!(leans["ink"], "not yet built", "{id}: {r}");
        }
    }
}

/// Each ship plans its study's voyage in Sol. Where SSE4 flies the same kind
/// of course as the study (a powered torch flight), the trip times agree
/// within the stated tolerance; the reasons for the gaps are in RK6.md.
#[test]
fn the_studies_voyages_fly_in_sol() {
    let p = pack();
    let o = &p["provides"]["other"];
    let mut k = stacked();
    for (id, ship) in ships(&p) {
        let says = &o["recipes"][&id]["study_says"];
        let to = match says["to"].as_str().unwrap() {
            "Callisto" => "Jupiter",
            "Alpha Centauri" => "Neptune",
            t => t,
        };
        let v = ask(
            &mut k,
            json!({"op": "plan_voyage", "fixture": "sol", "from": "Earth", "to": to, "ship": ship}),
        );
        assert_eq!(v["type"], "voyage_planned", "{id}: {v}");
        let legs = v["voyage"]["voyage"]["legs"].as_array().unwrap();
        assert!(!legs.is_empty(), "{id}");
        // Days on the way: the ship's days less any wait for a launch window.
        let wait: f64 = legs.iter().filter_map(|l| l["wait_days"].as_f64()).sum();
        let days = v["voyage"]["voyage"]["ship_days"].as_f64().unwrap() - wait;
        let course = legs[0]["course"].as_str().unwrap();
        let study = says["trip_days"].as_f64().unwrap();
        println!("{id}: SSE4 {days:.0} days ({course}), the study {study:.0}");
        // The voyage carries the books, because the ship has fitted engines.
        assert!(v["voyage"]["power"].is_object(), "{id}");
        // Torch flights the studies also flew under power the whole way.
        let band = match id.as_str() {
            // HOPE's fusion ship: SSE4 248 days, the study 249.
            "paper.hope-mtf" => Some((0.95, 1.05)),
            // Discovery II: SSE4 flies constant acceleration on a straight
            // line; the study's thrust is constant, so its acceleration grows
            // as 818 t of propellant burns off, and its path is shaped.
            "paper.discovery-ii" => Some((1.0, 1.5)),
            "paper.discovery-ii-saturn" => Some((1.0, 1.3)),
            // DRA 5.0: SSE4 193 days on a Lambert arc, the study 180.
            "paper.dra5-ntr" => Some((0.9, 1.2)),
            // HOPE NEP and the fission fragment ship stop at the Main Belt
            // for fuel, and belt legs do not spiral yet: no band (RK6.md, part 3).
            _ => None,
        };
        if let Some((lo, hi)) = band {
            let want = if id == "paper.dra5-ntr" {
                "lambert"
            } else {
                "torch"
            };
            assert!(
                course == want || (want == "lambert" && course == "hohmann"),
                "{id}: {course}"
            );
            let ratio = days / study;
            assert!(ratio >= lo && ratio <= hi, "{id}: {days} against {study}");
        }
    }
}

/// Longshot between the stars: NAV's interstellar flight (constant proper
/// acceleration with a turnover, or a coast when the propellant runs short)
/// over the report's 4.3 light years, against its "approximately 100 years".
#[test]
fn longshot_reaches_alpha_centauri_in_about_a_century() {
    let p = pack();
    let o = &p["provides"]["other"];
    let ship: sse_nav::Ship =
        serde_json::from_value(o["nav_ships"]["ships"]["paper.longshot"].clone()).unwrap();
    let says = &o["recipes"]["paper.longshot"]["study_says"];
    let d_pc = says["distance_ly"].as_f64().unwrap() * sse_core::sheet::LY / sse_core::sheet::PC;
    let f = sse_nav::voyage::flight(&ship, ship.wet_t - ship.dry_t, d_pc, 0.0).unwrap();
    let years = f.home_s / sse_core::sheet::YR_JULIAN;
    let study = says["trip_days"].as_f64().unwrap() * 86_400.0 / sse_core::sheet::YR_JULIAN;
    println!(
        "Longshot: SSE4 {years:.1} years (coast {}, peak {:.3} c), the study {study:.0}",
        f.coast, f.peak_beta
    );
    // SSE4 flies Longshot as one stage at its starting acceleration: 264 t
    // of pellets on 132 t give about 11,000 km/s in all, so it speeds up and
    // brakes at under 2% of c and takes about three times the study's
    // century. The report drops two empty tanks and then the drive itself
    // (sec. 2.2), which a single-stage record cannot. Staging is next.
    assert!(f.coast);
    assert!(years > study && years < 4.0 * study, "{years}");
}
