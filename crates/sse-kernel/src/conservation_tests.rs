//! CM4: the conservation check for compounds (compound ships design 5.4),
//! absolute as the house rule asks (engine map PHY-34).
//!
//! A long deterministic history of captures, releases, splits, docks,
//! undocks and merges over a fleet is written through the kernel's own ops.
//! Then, at every event's second, the fleet's mass is checked against the
//! known start: to the kilogram, as a number of kilograms and not a ratio.
//! No construct ever holds less than nothing, and reading each compound from
//! its root counts every construct exactly once.
//!
//! Propellant joins the sum when it can cross a mount (CM3: burns drawing
//! tanks, transfers with both ends); until then a construct's propellant is
//! inside its `construct.mass_kg` and moves only by a split or a merge.

use crate::Kernel;
use serde_json::{json, Value as J};
use sse_core::cbor::Value;

const START_KG: f64 = 3_000_000.0;
const FLEET: usize = 8;
const STEPS: u64 = 600;

fn ask(k: &mut Kernel, v: J) -> J {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

/// A small deterministic stream of numbers (a 64-bit LCG, Knuth's MMIX constants).
struct Draw(u64);
impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[(self.next() % xs.len() as u64) as usize]
    }
}

fn mass(row: &std::collections::BTreeMap<String, Value>) -> Option<f64> {
    match row.get("construct.mass_kg") {
        Some(Value::Float(x)) => Some(*x),
        Some(Value::Int(i)) => Some(*i as f64),
        _ => None,
    }
}

/// The fleet's history, written through the ops; returns the kernel, the
/// event seconds, and how many of each op were made (not refused).
fn history(seed: u64) -> (Kernel, Vec<i64>, std::collections::BTreeMap<String, u32>) {
    let mut k = Kernel::new();
    let r = ask(
        &mut k,
        json!({"op":"open_universe","id":"cm4","seed":"conservation"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    let mut d = Draw(seed);
    // Masses that sum to the start exactly: seven drawn, the last takes the rest.
    let mut ships: Vec<String> = (0..FLEET)
        .map(|i| format!("u:cm4/sys:0/ship:s{i}"))
        .collect();
    let mut edits = vec![];
    let mut left = START_KG;
    for (i, s) in ships.iter().enumerate() {
        let m = if i + 1 == FLEET {
            left
        } else {
            (50_000 + d.next() % 400_000) as f64
        };
        left -= m;
        edits.push(json!({"address":s,"field":"construct.mass_kg","value":m}));
        for p in 0..3 {
            edits.push(json!({"address":s,"field":format!("part.p{p}.mass_kg"),"value":(1000 + d.next() % 9000) as f64}));
        }
        edits.push(json!({"address":s,"field":"port.a.size_m","value":1.0}));
        edits.push(json!({"address":s,"field":"port.b.size_m","value":1.0}));
        edits.push(json!({"address":s,"field":"part.bay.bay_holds_kg","value":200_000.0}));
    }
    let r = ask(
        &mut k,
        json!({"op":"set_authored_batch","occurred_at_s":0,"edits":edits}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    let places = ["front", "behind", "inside", "hull", "outside"];
    let mut times = vec![0];
    let mut docks: Vec<(String, String)> = vec![];
    let mut made: std::collections::BTreeMap<String, u32> = Default::default();
    for step in 1..=STEPS {
        let t = step as i64 * 60;
        let a = d.pick(&ships).clone();
        let b = d.pick(&ships).clone();
        let q = match d.next() % 7 {
            0 | 1 => {
                let place = *d.pick(&places);
                let mut q =
                    json!({"op":"mount_construct","address":a,"on":b,"place":place,"at_s":t});
                match place {
                    "front" | "behind" => q["port"] = (*d.pick(&["a", "b"])).into(),
                    "hull" => q["clock_deg"] = ((d.next() % 360) as f64).into(),
                    "outside" => q["stand_off_m"] = 5.0.into(),
                    _ => {}
                }
                q
            }
            2 | 3 => json!({"op":"release_construct","address":a,"at_s":t}),
            4 => {
                let mut q = json!({"op":"split_construct","address":a,"at_s":t});
                if d.next() % 2 == 0 {
                    q["parts"] = json!([format!("p{}", d.next() % 3)]);
                } else {
                    q["mass_kg"] = ((d.next() % 20_000) as f64 + 0.25).into();
                }
                q
            }
            5 if !docks.is_empty() && d.next() % 2 == 0 => {
                let (h, w): (String, String) = d.pick(&docks).clone();
                json!({"op":"join_constructs","address":h,"with":w,"how":"undock","at_s":t})
            }
            5 => json!({"op":"join_constructs","address":b,"with":a,"how":"dock","at_s":t}),
            _ => json!({"op":"join_constructs","address":b,"with":a,"how":"merge","at_s":t}),
        };
        let r = ask(&mut k, q.clone());
        let ty = r["type"].as_str().unwrap_or("").to_string();
        if ty == "error" {
            continue;
        }
        *made
            .entry(
                q["op"].as_str().unwrap().to_string()
                    + &q["how"]
                        .as_str()
                        .map(|h| format!(":{h}"))
                        .unwrap_or_default(),
            )
            .or_default() += 1;
        if q["how"] == "dock" {
            docks.push((
                q["address"].as_str().unwrap().into(),
                q["with"].as_str().unwrap().into(),
            ));
        }
        if ty == "construct_split" {
            ships.push(r["address"].as_str().unwrap().to_string());
        }
        times.push(t);
    }
    (k, times, made)
}

/// The check itself, at one second: the fleet's mass to the kilogram, no
/// construct below zero, and each construct counted once across compounds.
fn check(k: &mut Kernel, t: i64) -> Result<(), String> {
    let u = k.universe().ok().unwrap();
    let st = u
        .chronicle
        .state_at(sse_chronicle::UniverseTime::seconds(t));
    let live: Vec<_> = st
        .facts
        .iter()
        .filter(|(_, row)| row.get("retired") != Some(&Value::Bool(true)))
        .filter(|(_, row)| mass(row).is_some())
        .collect();
    let mut total = 0.0;
    for (a, row) in &live {
        let m = mass(row).unwrap();
        if m < 0.0 {
            return Err(format!("{a} holds {m} kg at {t} s"));
        }
        total += m;
    }
    if (total - START_KG).abs() >= 1.0 {
        return Err(format!(
            "at {t} s the fleet holds {total} kg; it started with {START_KG} kg ({} kg {})",
            (total - START_KG).abs(),
            if total > START_KG { "made" } else { "lost" }
        ));
    }
    // Read from every root, the compounds hold every construct once.
    let mut seen = 0.0;
    let mut members = 0;
    for (a, row) in &live {
        if sse_trigger::mount::mount_of(row).is_none() {
            let c = sse_trigger::mount::compound(&st, a);
            members += c.members.len();
            seen += c.mass_kg().0;
        }
    }
    if members != live.len() || (seen - START_KG).abs() >= 1.0 {
        return Err(format!(
            "at {t} s the compounds hold {members} constructs and {seen} kg; the fleet is {} and {START_KG} kg",
            live.len()
        ));
    }
    Ok(())
}

#[test]
fn a_compound_history_neither_makes_nor_loses_a_kilogram() {
    for seed in [1u64, 27, 1969] {
        let (mut k, times, made) = history(seed);
        // The history really exercises every rule, not just refusals.
        for op in [
            "mount_construct",
            "release_construct",
            "split_construct",
            "join_constructs:dock",
            "join_constructs:undock",
            "join_constructs:merge",
        ] {
            assert!(
                made.get(op).copied().unwrap_or(0) >= 3,
                "seed {seed}: {made:?}"
            );
        }
        for t in &times {
            if let Err(e) = check(&mut k, *t) {
                panic!("seed {seed}: {e}");
            }
        }
        // And between events: half a minute after each.
        for t in &times {
            check(&mut k, t + 30).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        }
    }
}
