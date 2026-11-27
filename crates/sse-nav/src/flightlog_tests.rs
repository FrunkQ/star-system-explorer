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

#[test]
fn a_hyperbola_leaves_the_sun_with_the_energy_its_elements_give() {
    use crate::flightlog::OrbitElements;
    use crate::kepler::dot;
    let mu = sol().mu();
    // Voyager 2 after Neptune (Horizons, as the spaceflight thread wrote it).
    let el: OrbitElements = serde_json::from_value(serde_json::json!({
        "a_m": -601327973767.1986, "e": 6.284206181042654, "i_deg": 78.81679939983958,
        "node_deg": 101.6327895441973, "peri_deg": 130.0648468201176, "m0_deg": 0.0, "epoch_s": 0
    }))
    .unwrap();
    // At periapsis: |a| (e − 1) from the Sun.
    let (r0, _) = el.state(mu, 0.0);
    let q = 601327973767.1986 * (6.284206181042654 - 1.0);
    assert!((norm(r0) - q).abs() < 1.0, "{} against {q}", norm(r0));
    // Thirty years on: still the same orbital energy, μ / 2|a|, and far out.
    let (r, v) = el.state(mu, 30.0 * 365.25 * DAY);
    let energy = dot(v, v) / 2.0 - mu / norm(r);
    let want = mu / (2.0 * 601327973767.1986);
    assert!(
        (energy - want).abs() < 1e-6 * want,
        "{energy} against {want}"
    );
    assert!(norm(r) / AU > 50.0, "{} au", norm(r) / AU);
}

#[test]
fn the_star_answers_to_the_systems_own_address() {
    use crate::flightlog::Ephemeris;
    let sys = sol();
    assert_eq!(
        Ephemeris::body(&sys, "u:sol/sys:sun", 0.0).map(|b| b.0),
        Some([0.0; 3])
    );
    assert!(Ephemeris::body(&sys, "u:sol/sys:sun/body:earth", 0.0).is_some());
    assert!(
        Ephemeris::body(&sys, "u:sol/sys:sun/body:earth/body:moon", 0.0).is_none(),
        "Sol's NAV fixture has no Moon yet"
    );
}

/// Voyager 2 from the real spaceflight pack's log (Horizons flyby times): each
/// arc between flybys is solved from its ends, so rewinding meets Jupiter,
/// Saturn, Uranus and Neptune on their dates, and after Neptune the hyperbola
/// carries it out past 100 au by 2020.
#[test]
fn voyager_2_rewinds_through_four_flybys() {
    let sys = sol();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text =
        std::fs::read_to_string(root.join("tests/fixtures/flightlog/voyager-2.json")).unwrap();
    let l: FlightLog = serde_json::from_str(&text).unwrap();
    let flybys: Vec<(i64, String)> = l
        .events
        .iter()
        .filter_map(|e| match &e.what {
            What::Flyby { body, .. } => Some((e.at_s, body.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(flybys.len(), 4);
    let mut last_r = 1.0;
    for (t, body) in &flybys {
        // A second before: arriving at the planet (the arc ends at its centre).
        let just = l.at(t - 1, &sys);
        assert_eq!(just.kind, "course", "{body}");
        let b = crate::flightlog::Ephemeris::body(&sys, body, *t as f64).unwrap();
        let miss = norm(crate::kepler::sub(just.r_m.unwrap(), b.0));
        assert!(miss < 1.0e6, "missed {body} by {} km", miss / 1e3);
        // Each flyby is further out than the last.
        let r = norm(b.0) / AU;
        assert!(r > last_r, "{body} at {r} au");
        last_r = r;
    }
    // 2020-01-01 (TT): out on the hyperbola, past 100 au (Horizons: about 122 au).
    let t2020 = T0 + 7305 * 86_400;
    let out = l.at(t2020, &sys);
    let d = norm(out.r_m.unwrap()) / AU;
    assert!(d > 100.0 && d < 140.0, "{d} au");
}

/// CM2: a state turned into elements and back is the same state, for an
/// ellipse, a circle, an equatorial orbit, a retrograde one and a hyperbola.
#[test]
fn elements_from_a_state_give_the_state_back() {
    use crate::flightlog::{elements_from_state, OrbitElements};
    let mu = 3.986004418e14;
    let cases = [
        (7.0e6, 0.1, 28.5, 40.0, Some(60.0), Some(10.0)),
        (6.778e6, 0.0, 51.6, 300.0, None, None),
        (4.2164e7, 0.01, 0.0, 0.0, Some(90.0), Some(200.0)),
        (8.0e6, 0.2, 150.0, 10.0, Some(250.0), Some(300.0)),
        (-2.0e7, 1.5, 20.0, 70.0, Some(30.0), Some(15.0)),
    ];
    for (a, e, i, node, peri, m0) in cases {
        let el = OrbitElements {
            a_m: a,
            e,
            i_deg: i,
            node_deg: node,
            peri_deg: peri,
            m0_deg: m0,
            varpi_deg: None,
            mean_longitude_deg: None,
            epoch_s: 0,
        };
        let (r, v) = el.state(mu, 1234.0);
        let back = elements_from_state(r, v, mu, 1234).expect("not parabolic");
        assert!((back.a_m - a).abs() < 1e-6 * a.abs(), "{back:?}");
        assert!((back.e - e).abs() < 1e-9, "{back:?}");
        for dt in [0.0, 600.0, 5000.0] {
            let (r1, v1) = el.state(mu, 1234.0 + dt);
            let (r2, v2) = back.state(mu, dt);
            let d = norm([r1[0] - r2[0], r1[1] - r2[1], r1[2] - r2[2]]);
            let dv = norm([v1[0] - v2[0], v1[1] - v2[1], v1[2] - v2[2]]);
            assert!(d < 1e-3 && dv < 1e-6, "{a} {e} {i}: {d} m, {dv} m/s off");
        }
    }
}
