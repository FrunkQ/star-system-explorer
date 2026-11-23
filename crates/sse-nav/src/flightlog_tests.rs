//! The flight log (`flightlog.rs`): a craft parked at Earth, a course solved
//! from its ends to Mars, an orbit there, and an end; rewinding reads each.
use crate::flightlog::{FlightLog, What};
use crate::kepler::norm;
use crate::scenario::sol;
use crate::NavSystem;
use sse_core::quantity::consts::{AU, DAY};

const T0: i64 = 63_115_200_000; // Sol fixture's own epoch.

/// The first day Mars leads Earth by the Hohmann phase angle (44°), so the solved arc is the short way round.
fn window(sys: &NavSystem) -> i64 {
    let ang = |n: &str, t: i64| {
        let (r, _) = sys.at(sys.body(n).unwrap(), t as f64);
        sse_core::math::atan2(r[1], r[0]).to_degrees()
    };
    (2..800)
        .map(|d| T0 + d * 86_400)
        .find(|&t| {
            let lead = (ang("Mars", t) - ang("Earth", t)).rem_euclid(360.0);
            (lead - 44.0).abs() < 1.0
        })
        .expect("a window within 800 days")
}

fn log(sys: &NavSystem) -> FlightLog {
    let leave = window(sys);
    let arrive = leave + 259 * 86_400;
    let earth_r = sys.body("Earth").unwrap().radius_m;
    let mars_r = sys.body("Mars").unwrap().radius_m;
    serde_json::from_value(serde_json::json!({
        "construct": "u/sol/construct:pathfinder", "name": "Pathfinder",
        "pieces": [
            { "from_s": T0, "state": { "kind": "landed", "body": "Earth", "lat_deg": 28.5, "lon_deg": -80.6 }, "mark": "authored" },
            { "from_s": T0 + 86_400, "state": { "kind": "orbit", "around": "Earth",
                "elements": { "a_m": earth_r + 300e3, "e": 0.0, "epoch_s": T0 + 86_400 } }, "mark": "authored" },
            { "from_s": leave, "state": { "kind": "course", "course": { "kind": "lambert", "to": "Mars", "arrive_s": arrive } }, "mark": "estimated" },
            { "from_s": arrive, "state": { "kind": "orbit", "around": "Mars",
                "elements": { "a_m": mars_r + 400e3, "e": 0.0, "epoch_s": arrive } }, "mark": "authored" },
            { "from_s": arrive + 400 * 86_400, "state": { "kind": "gone", "how": "lost_contact" }, "mark": "authored" }
        ],
        "events": [
            { "at_s": T0 + 86_400, "kind": "launch", "mark": "authored" },
            { "at_s": T0 + 86_400, "kind": "split", "into": ["u/sol/construct:pathfinder-stage-1"], "mark": "authored" },
            { "at_s": leave, "kind": "burn", "dv_m_s": 3600.0, "propellant_t": 12.0, "mark": "estimated" },
            { "at_s": arrive, "kind": "burn", "dv_m_s": 2100.0, "mark": "estimated" },
            { "at_s": arrive + 86_400, "kind": "stop", "type": "refuel", "journey_id": "j1", "fuel_kg": 800.0, "mark": "authored" }
        ]
    }))
    .unwrap()
}

#[test]
fn rewinding_reads_each_piece_of_a_mars_voyage() {
    let sys = sol();
    let l = log(&sys);
    let leave = window(&sys);
    let arrive = leave + 259 * 86_400;
    assert_eq!(l.at(T0 - 1, &sys).kind, "before");
    let pad = l.at(T0 + 3600, &sys);
    assert_eq!(
        (pad.kind.as_str(), pad.with.as_deref()),
        ("landed", Some("Earth"))
    );
    // Parked: within a few thousand km of Earth.
    let parked = l.at(leave - 86_400, &sys);
    let (re, _) = sys.at(sys.body("Earth").unwrap(), (leave - 86_400) as f64);
    let off = norm(crate::kepler::sub(parked.r_m.unwrap(), re));
    assert!(off > 6.5e6 && off < 6.8e6, "{off}");
    // Halfway: between the two orbits, around the star.
    let mid = l.at(leave + 130 * 86_400, &sys);
    let d = norm(mid.r_m.unwrap()) / AU;
    assert_eq!(mid.kind, "course");
    assert!(d > 1.0 && d < 1.67, "{d} au");
    // The solved arc meets Mars on time.
    let just = l.at(arrive - 1, &sys);
    let (rm, _) = sys.at(sys.body("Mars").unwrap(), (arrive - 1) as f64);
    let miss = norm(crate::kepler::sub(just.r_m.unwrap(), rm));
    assert!(miss < 1.0e5, "missed Mars by {} km", miss / 1e3);
    assert_eq!(
        l.at(arrive + DAY as i64, &sys).with.as_deref(),
        Some("Mars")
    );
    let end = l.at(arrive + 500 * 86_400, &sys);
    assert_eq!(
        (end.kind.as_str(), end.words.as_str()),
        ("gone", "contact lost")
    );
}

#[test]
fn events_splits_and_sse3_stops_keep_their_fields() {
    let sys = sol();
    let l = log(&sys);
    assert_eq!(
        l.splits(),
        vec![(T0 + 86_400, "u/sol/construct:pathfinder-stage-1")]
    );
    let leave = window(&sys);
    let burns: Vec<f64> = l
        .events_between(leave, i64::MAX)
        .iter()
        .filter_map(|e| match &e.what {
            What::Burn { dv_m_s, .. } => Some(*dv_m_s),
            _ => None,
        })
        .collect();
    assert_eq!(burns, vec![3600.0, 2100.0]);
    let back: FlightLog = serde_json::from_value(serde_json::to_value(&l).unwrap()).unwrap();
    assert_eq!(back, l);
    let stop = serde_json::to_value(&l.events[4]).unwrap();
    assert_eq!(stop["type"], "refuel");
    assert_eq!(stop["fuel_kg"], 800.0);
    assert_eq!(stop["kind"], "stop");
}
