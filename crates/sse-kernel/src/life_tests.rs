//! L3: life on the chart's own worlds. The catalogued Solar System's worlds
//! and every world of a grown system go through `life_view` by their
//! addresses on the chart.

use crate::sol_tests::open_sol;
use std::path::PathBuf;

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const SOL: &str = "u:home/sys:sol";

fn address(k: &mut crate::Kernel, name: &str) -> String {
    let r = send(k, serde_json::json!({"op":"resolved_system","system":SOL}));
    r["facts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, f)| f["name"] == name)
        .map(|(a, _)| a.clone())
        .unwrap_or_else(|| panic!("no {name}"))
}

fn life(k: &mut crate::Kernel, name: &str) -> serde_json::Value {
    let a = address(k, name);
    let r = send(k, serde_json::json!({"op":"life_view","body":a}));
    assert_eq!(r["type"], "life_viewed", "{name}: {r}");
    assert_eq!(r["source"], a.as_str());
    r
}

fn ids(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|h| h["id"].as_str().unwrap().to_string())
        .collect()
}

fn held(r: &serde_json::Value) -> String {
    r["held"].to_string()
}

/// Catalogued Earth grows its own life from its measured climate, air and
/// ocean: land life under an oxygen air the coupled air box builds, with
/// the history's events and epochs. Nothing about Earth is a fixture here.
#[test]
fn catalogued_earth_grows_land_life_and_its_own_oxygen() {
    let mut k = open_sol(&[]);
    let r = life(&mut k, "Earth");
    let v = &r["view"];
    assert_eq!(v["signatures"]["alive"], true, "{v}");
    let last = v["timeline"].as_array().unwrap().last().unwrap().clone();
    let caps: Vec<&str> = last["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    for c in ["oxygenic", "complex_cells", "land"] {
        assert!(caps.contains(&c), "Earth lacks {c}: {caps:?}");
    }
    // Production within a factor of three of Earth's 105 Gt C a year
    // (Field et al. 1998), and oxygen made by life, not typed in.
    let prod = last["production_gc_yr"].as_f64().unwrap();
    assert!(prod > 3.5e16 && prod < 3.2e17, "production {prod:e}");
    let o2 = last["world"]["o2_fraction"].as_f64().unwrap();
    assert!(o2 > 0.05, "air O2 {o2}");
    assert!(
        v["guilds"].as_array().unwrap().len() >= 8,
        "{}",
        v["guilds"]
    );
    assert!(v["epochs"].as_array().unwrap().len() >= 3);
    assert!(held(&r).contains("held through the whole history"));
    // The same world twice is the same life.
    let again = life(&mut open_sol(&[]), "Earth");
    assert_eq!(r, again);
}

/// Europa: microbes in the ocean under its ice, fed by its tidal heat, and
/// hidden from every remote scan. Venus, the Moon and Mars are lifeless for
/// the reasons their habitats give; a giant has no ground to live on and
/// its methane is no sign of life.
#[test]
fn the_solar_systems_other_worlds_read_their_own_conditions() {
    let mut k = open_sol(&[]);
    let eu = life(&mut k, "Europa");
    assert_eq!(eu["view"]["signatures"]["alive"], true, "{eu}");
    assert!(ids(&eu["view"]["hooks"]).contains(&"hidden_life".to_string()));
    assert!(held(&eu).contains("conductive lid"), "{}", held(&eu));
    assert!(!eu["view"]["guilds"].as_array().unwrap().is_empty());

    for n in ["Venus", "Moon", "Mars", "Jupiter"] {
        let r = life(&mut k, n);
        assert_eq!(r["view"]["signatures"]["ever_alive"], false, "{n}: {r}");
        assert!(r["view"]["guilds"].as_array().unwrap().is_empty(), "{n}");
    }
    let ju = life(&mut k, "Jupiter");
    assert!(held(&ju).contains("no ground"), "{}", held(&ju));
    assert!(
        !ids(&ju["view"]["hooks"]).contains(&"false_positive".to_string()),
        "methane in a hydrogen air is no sign of life: {}",
        ju["view"]["hooks"]
    );
    let ve = life(&mut k, "Venus");
    assert!(held(&ve).contains("no rain"), "{}", held(&ve));
}

/// A grown system: every one of its worlds answers, with a life view or a
/// plain "no habitats", and the answer repeats.
#[test]
fn every_world_of_a_grown_system_answers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let pack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("packs/base-time/pack.json")).unwrap(),
    )
    .unwrap();
    let open = || {
        let mut k = crate::Kernel::new();
        send(
            &mut k,
            serde_json::json!({"op": "load_stack", "packs": [pack.clone()]}),
        );
        send(
            &mut k,
            serde_json::json!({"op": "open_universe", "id": "home", "seed": "home"}),
        );
        let form = serde_json::json!({"seed": "adopt-1", "system": "u:home/sys:5", "star_pins": {"primary_mass": 1.0}});
        let r = send(
            &mut k,
            serde_json::json!({"op": "adopt", "formation": form}),
        );
        assert_eq!(r["type"], "adopted", "{r}");
        k
    };
    let mut k = open();
    let rs = send(
        &mut k,
        serde_json::json!({"op": "resolved_system", "system": "u:home/sys:5"}),
    );
    let worlds: Vec<String> = rs["surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["address"].as_str().unwrap().to_string())
        .collect();
    assert!(worlds.len() >= 3, "{rs}");
    let mut viewed = 0;
    for w in &worlds {
        let r = send(&mut k, serde_json::json!({"op": "life_view", "body": w}));
        match r["type"].as_str() {
            Some("life_viewed") => {
                viewed += 1;
                assert_eq!(r["source"], w.as_str());
                assert!(r["view"]["timeline"].is_array(), "{r}");
            }
            _ => assert_eq!(r["code"], "no_habitats", "{w}: {r}"),
        }
    }
    assert!(viewed >= 1, "no grown world gave a life view");
    let w = &worlds[0];
    let a = send(&mut k, serde_json::json!({"op": "life_view", "body": w}));
    let b = send(
        &mut open(),
        serde_json::json!({"op": "life_view", "body": w}),
    );
    assert_eq!(a, b);
    // A body that is not on the chart has no habitats, said plainly.
    let r = send(
        &mut k,
        serde_json::json!({"op": "life_view", "body": "u:home/sys:5/body:99"}),
    );
    assert_eq!(r["code"], "no_habitats", "{r}");
}

/// LE1 and LE2 on the chart's own worlds: catalogued Earth captures about
/// 130 TW with a surplus; Europa's ocean is alive, barely, with its numbers.
#[test]
fn life_view_carries_the_energy_ledger() {
    let mut k = open_sol(&[]);
    let e = life(&mut k, "Earth");
    let w = &e["view"]["energy"]["world"];
    let c = w["captured_w"].as_f64().unwrap();
    assert!((130e12 / 3.0..130e12 * 3.0).contains(&c), "{w}");
    assert!(c <= w["offered_w"].as_f64().unwrap());
    assert!(w["offered_w"].as_f64().unwrap() <= w["arriving_w"].as_f64().unwrap());
    assert!(w["surplus_w"].as_f64().unwrap() > 0.0, "{w}");
    assert_eq!(w["brought_in_w"], 0.0);
    assert!(e["view"]["energy"]["absorbed_sunlight_w"].is_null());
    let eu = life(&mut k, "Europa");
    let hs = eu["view"]["energy"]["habitats"].as_array().unwrap();
    let living: Vec<_> = hs
        .iter()
        .filter(|h| h["captured_w"].as_f64().unwrap_or(0.0) > 0.0)
        .collect();
    assert!(!living.is_empty(), "{}", eu["view"]["energy"]);
    for h in living {
        assert_eq!(h["barely"], true, "{h}");
        assert!(h["surplus_w"].as_f64().unwrap() < 0.0, "{h}");
    }
}

/// LE3 and LE4 on the chart's own Earth: the guilds share out the captured
/// power, and Earth reads about one times Earth's pace.
#[test]
fn life_view_shares_the_energy_and_reads_earths_pace() {
    let mut k = open_sol(&[]);
    let e = life(&mut k, "Earth");
    let v = &e["view"];
    let guilds = v["guilds"].as_array().unwrap();
    let share: f64 = guilds
        .iter()
        .map(|g| g["share_of_captured"].as_f64().unwrap())
        .sum();
    assert!((share - 1.0).abs() < 1e-9, "{share}");
    let pace = v["speed"]["pace_x_earth"].as_f64().unwrap();
    assert!((0.33..3.0).contains(&pace), "{}", v["speed"]["words"]);
    assert!(v["speed"]["words"]
        .as_str()
        .unwrap()
        .contains("Earth's pace"));
}

/// The catalogued Earth's early ocean keeps the iron its vents bring: today's
/// oxygen is life's product, and the history does not read it before life
/// makes it. Before oxygen-releasing photosynthesis the mats run on that
/// iron, near Canfield et al. 2006's 1e14 to 1e15 g C a year, not on vent
/// hydrogen alone (8e11).
#[test]
fn earths_early_ocean_keeps_its_iron() {
    let mut k = open_sol(&[]);
    let v = life(&mut k, "Earth")["view"].clone();
    let mats = v["timeline"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| {
            let caps = p["capabilities"].as_array().unwrap();
            caps.iter().any(|c| c == "anoxygenic") && !caps.iter().any(|c| c == "oxygenic")
        })
        .map(|p| p["production_gc_yr"].as_f64().unwrap())
        .fold(0.0, f64::max);
    assert!(mats > 1.0e14, "{mats:e}");
}

/// L3-heat: the catalogued Earth's history reads its interior's own heat at
/// each age, from the thermal history, not today's held throughout. The
/// Archean Earth lost about three times the heat it loses now.
#[test]
fn earths_young_interior_runs_hotter() {
    let mut k = open_sol(&[]);
    let a = address(&mut k, "Earth");
    let (r, _) = k.body_reading(&a).unwrap();
    let p = params();
    let e = sse_life::world_life::heat_epochs(&r, p.heat_epoch_step, p.heat_epoch_cap);
    let then = |t: f64| e.iter().rev().find(|(from, _)| *from <= t).unwrap().1;
    let now = r
        .heat_history
        .iter()
        .rev()
        .find(|x| x.at_myr <= r.age_myr)
        .unwrap()
        .heat_flow_w_m2;
    let ratio = then(r.age_myr - 3500.0) / now;
    assert!((2.0..5.0).contains(&ratio), "{e:?} now {now}");
    assert!(e.len() >= 3, "{e:?}");
    let (_, _, held) = k.body_life(&a).unwrap();
    assert!(
        held.iter().any(|h| h.contains("its own history")),
        "{held:?}"
    );
}

/// The history switches its habitats at each heat epoch: a world whose
/// interior dies part way loses its vents from then on, and nothing before.
#[test]
fn a_dying_interior_changes_the_history_from_its_date() {
    let mut k = open_sol(&[]);
    let a = address(&mut k, "Earth");
    let (r, seed) = k.body_reading(&a).unwrap();
    let p = params();
    let run = |cold: bool| {
        let mut w = r.clone();
        let late = if cold { p.vents_above_w_m2 / 4.0 } else { 0.1 };
        w.heat_history = vec![
            sse_life::world_life::HeatSample {
                at_myr: 0.0,
                heat_flow_w_m2: 0.1,
            },
            sse_life::world_life::HeatSample {
                at_myr: 2500.0,
                heat_flow_w_m2: late,
            },
        ];
        let s = sse_surface::scenario::settings();
        sse_life::world_life::life(
            &w,
            &[sse_life::scenario::base_life_pack()],
            &s.climate,
            &seed,
        )
        .unwrap()
        .history
    };
    let (hot, cold) = (run(false), run(true));
    let before = |h: &sse_life::history::LifeHistory| {
        h.points
            .iter()
            .filter(|x| x.at_myr < 2500.0)
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(before(&hot), before(&cold));
    assert_ne!(hot.points, cold.points);
}

fn params() -> sse_life::world_life::FromWorldParams {
    sse_life::world_life::FromWorldParams::from_packs(&[sse_life::scenario::base_life_pack()])
        .unwrap()
}
