//! V8 flight log on the record: Voyager 2 written entry by entry and rewound
//! through the API, a split that starts the probe's own log, a join that
//! docks, and a withdrawal.

use crate::Kernel;
use serde_json::{json, Value as J};

fn ask(k: &mut Kernel, v: J) -> J {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

fn open() -> Kernel {
    let mut k = Kernel::new();
    let r = k.handle_json(r#"{"op":"open_universe","id":"sol","seed":"flight logs"}"#);
    assert!(r.contains("universe_opened"), "{r}");
    k
}

const T0: i64 = 63_115_200_000; // Sol fixture's own epoch.
const V2: &str = "u:sol/construct:voyager-2";

#[test]
fn voyager_2_written_to_the_record_rewinds_as_its_fixture_does() {
    let mut k = open();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text =
        std::fs::read_to_string(root.join("tests/fixtures/flightlog/voyager-2.json")).unwrap();
    let f: J = serde_json::from_str(&text).unwrap();
    let r = ask(
        &mut k,
        json!({"op": "log_flight", "construct": V2, "name": f["name"], "about": f["about"], "at_s": 0}),
    );
    assert_eq!(r["type"], "flight_logged", "{r}");
    // Written in an odd order: the log is by time, not by writing.
    let mut pieces: Vec<J> = f["pieces"].as_array().unwrap().clone();
    pieces.reverse();
    for p in pieces {
        let r = ask(
            &mut k,
            json!({"op": "log_flight", "construct": V2, "piece": p, "at_s": 0}),
        );
        assert_eq!(r["type"], "flight_logged", "{r}");
    }
    for e in f["events"].as_array().unwrap() {
        ask(
            &mut k,
            json!({"op": "log_flight", "construct": V2, "event": e, "at_s": 0}),
        );
    }
    let t2020 = T0 + 7305 * 86_400;
    let r = ask(
        &mut k,
        json!({"op": "flight_log", "construct": V2, "where_at": t2020, "fixture": "sol"}),
    );
    assert_eq!(r["type"], "flight_log_is", "{r}");
    assert_eq!(r["log"]["name"], f["name"]);
    assert_eq!(
        r["log"]["pieces"], f["pieces"],
        "the same pieces, in time order"
    );
    assert_eq!(
        r["entries"].as_array().unwrap().len(),
        1 + 7 + 7,
        "{}",
        r["entries"]
    );
    let at = &r["at"];
    let rr: Vec<f64> = at["r_m"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let d = (rr[0] * rr[0] + rr[1] * rr[1] + rr[2] * rr[2]).sqrt() / sse_core::quantity::consts::AU;
    assert!(d > 100.0 && d < 140.0, "{d} au");
}

#[test]
fn a_split_starts_the_new_pieces_log_and_a_join_docks() {
    let mut k = open();
    let ship = "u:sol/construct:discovery";
    let pod = "u:sol/construct:pod";
    let orbit = json!({"from_s": T0, "state": {"kind": "orbit", "around": "Earth",
        "elements": {"a_m": 6.778e6, "e": 0.0, "epoch_s": T0}}, "mark": "authored"});
    ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "piece": orbit, "at_s": 0}),
    );
    let r = ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "event":
        {"at_s": T0 + 3600, "kind": "split", "into": [pod], "mark": "authored"}, "at_s": 10}),
    );
    assert_eq!(r["seeded"].as_array().unwrap().len(), 1, "{r}");
    let p = ask(
        &mut k,
        json!({"op": "flight_log", "construct": pod, "where_at": T0 + 7200, "fixture": "sol"}),
    );
    let first = &p["log"]["pieces"][0];
    assert_eq!(first["from_s"], T0 + 3600);
    assert_eq!(first["state"]["kind"], "orbit");
    assert_eq!(first["why"], format!("split from {ship}"));
    assert_eq!(p["at"]["kind"], "orbit", "{p}");
    assert_eq!(p["at"]["with"], "Earth");
    // The pod comes home: it docks to the ship.
    ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "event":
        {"at_s": T0 + 86_400, "kind": "join", "with": pod, "mark": "authored"}, "at_s": 20}),
    );
    let p = ask(
        &mut k,
        json!({"op": "flight_log", "construct": pod, "where_at": T0 + 90_000, "fixture": "sol"}),
    );
    assert_eq!(p["at"]["kind"], "docked", "{p}");
    assert_eq!(p["at"]["with"], ship);
    // As the record stood before the join (written at 20), the pod was still on its own.
    let before = ask(
        &mut k,
        json!({"op": "flight_log", "construct": pod, "at_s": 15}),
    );
    assert_eq!(before["log"]["pieces"].as_array().unwrap().len(), 1);
}

#[test]
fn a_split_on_a_course_says_so_and_entries_withdraw() {
    let mut k = open();
    let ship = "u:sol/construct:courier";
    let course = json!({"from_s": T0, "state": {"kind": "course", "course": {"kind": "lambert", "to": "Mars", "arrive_s": T0 + 200 * 86_400}}, "mark": "planned"});
    let r = ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "piece": course, "at_s": 0}),
    );
    let piece_id = r["event_id"].as_str().unwrap().to_string();
    let r = ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "event":
        {"at_s": T0 + 86_400, "kind": "split", "into": ["u:sol/construct:probe"], "mark": "planned"}, "at_s": 0}),
    );
    assert!(r["seeded"].as_array().unwrap().is_empty());
    assert!(
        r["notes"][0]
            .as_str()
            .unwrap()
            .contains("u:sol/construct:probe split from u:sol/construct:courier on a course"),
        "{r}"
    );
    let w = ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "withdraw": piece_id, "at_s": 1}),
    );
    assert_eq!(w["type"], "flight_logged", "{w}");
    let l = ask(&mut k, json!({"op": "flight_log", "construct": ship}));
    assert!(l["log"]["pieces"].as_array().unwrap().is_empty());
    assert_eq!(l["log"]["events"].as_array().unwrap().len(), 1);
    // Refusals: a bad piece, two things at once, a stranger's entry, outside the universe.
    let bad = ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "piece": {"from_s": 0}, "at_s": 0}),
    );
    assert_eq!(bad["code"], "bad_flight", "{bad}");
    let two = ask(
        &mut k,
        json!({"op": "log_flight", "construct": ship, "piece": course, "name": "x", "at_s": 0}),
    );
    assert_eq!(two["code"], "bad_flight");
    let stranger = ask(
        &mut k,
        json!({"op": "log_flight", "construct": "u:sol/construct:other", "withdraw": piece_id, "at_s": 0}),
    );
    assert_eq!(stranger["code"], "bad_value", "{stranger}");
    let away = ask(
        &mut k,
        json!({"op": "log_flight", "construct": "u:elsewhere/construct:x", "name": "x", "at_s": 0}),
    );
    assert_eq!(away["code"], "bad_address");
}
