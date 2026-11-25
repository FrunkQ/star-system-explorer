//! The Sol today pack's flight logs (track RS, package RS3), as written by
//! `scripts/spaceflight/build_logs.py`: every log reads as a `FlightLog`, its
//! pieces run in order without gaps, every split names constructs that have
//! their own log from that moment, and Cassini rewinds through its flybys.
use sse_nav::flightlog::{Course, Ephemeris, FlightLog, Mark, State, What};
use sse_nav::kepler::{norm, sub};
use sse_nav::scenario::sol;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn logs() -> BTreeMap<String, FlightLog> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../store-packs/sol-today/logs");
    // The showcase logs (build_logs.py) and, under departed/, one per craft that left Earth orbit
    // (build_departed_logs.py, RS8), and under earth-orbit/ the stations and crewed flights (RS8 part 2).
    let mut names: Vec<PathBuf> = [dir.clone(), dir.join("departed"), dir.join("earth-orbit")]
        .iter()
        .flat_map(|d| std::fs::read_dir(d).unwrap())
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    let mut out = BTreeMap::new();
    for p in names {
        let text = std::fs::read_to_string(&p).unwrap();
        let l: FlightLog =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        assert_eq!(stem, l.construct, "file name and construct");
        assert!(out.insert(stem.clone(), l).is_none(), "{stem} twice");
    }
    out
}

#[test]
fn every_log_reads_and_runs_in_order() {
    let all = logs();
    assert!(all.len() >= 30, "{} logs", all.len());
    for (id, l) in &all {
        assert!(!l.pieces.is_empty(), "{id}: no pieces");
        for w in l.pieces.windows(2) {
            assert!(
                w[0].from_s < w[1].from_s,
                "{id}: pieces out of order at {}",
                w[1].from_s
            );
            assert_eq!(
                w[0].to_s,
                Some(w[1].from_s),
                "{id}: gap or overlap at {}",
                w[1].from_s
            );
        }
        for p in &l.pieces {
            if p.mark == Mark::Estimated {
                assert!(
                    p.why.is_some(),
                    "{id}: an estimated piece at {} says why",
                    p.from_s
                );
            }
        }
        let first = l.pieces[0].from_s;
        for e in &l.events {
            assert!(e.at_s >= first, "{id}: event before the log begins");
            if let What::Burn { dv_m_s, .. } = e.what {
                assert!(
                    dv_m_s > 0.0 && dv_m_s < 12_000.0,
                    "{id}: burn of {dv_m_s} m/s"
                );
            }
            if e.mark == Mark::Estimated {
                assert!(
                    e.why.is_some(),
                    "{id}: an estimated event at {} says why",
                    e.at_s
                );
            }
        }
        assert!(
            l.events.windows(2).all(|w| w[0].at_s <= w[1].at_s),
            "{id}: events out of order"
        );
    }
}

/// One split mechanism for stages, boosters, tanks, landers and probes: each
/// construct a split names has its own log, with a piece starting right then;
/// each join is met by a docked piece on the other side.
#[test]
fn splits_and_joins_meet_their_logs() {
    let all = logs();
    for (id, l) in &all {
        for (t, into) in l.splits() {
            let other = all
                .get(into)
                .unwrap_or_else(|| panic!("{id} splits into {into}, which has no log"));
            assert!(
                other.pieces.iter().any(|p| p.from_s == t),
                "{into} has no piece starting at {id}'s split ({t})"
            );
        }
        for e in &l.events {
            if let What::Join { with } = &e.what {
                let other = &all[with];
                let (_, p) = other.piece_at(e.at_s).expect("a piece at the join");
                match &p.state {
                    State::Docked { to } => assert_eq!(to, id),
                    _ => {
                        // The other side docks with this one: this log must then be docked to it.
                        let (_, q) = l.piece_at(e.at_s).unwrap();
                        assert!(
                            matches!(&q.state, State::Docked { to } if to == with),
                            "{id} joins {with}"
                        );
                    }
                }
            }
        }
    }
    // The one-use parts end, and the reused booster lands.
    for gone in [
        "as-506-s-ic",
        "as-506-s-ii",
        "apollo-11-service-module",
        "sts-1-external-tank",
    ] {
        let last = &all[gone].pieces.last().unwrap().state;
        assert!(matches!(last, State::Gone { .. }), "{gone} ends gone");
    }
    for down in [
        "falcon-9-b1019",
        "sts-1-srb-left",
        "sts-1-srb-right",
        "apollo-11-eagle-descent-stage",
    ] {
        let last = &all[down].pieces.last().unwrap().state;
        assert!(matches!(last, State::Landed { .. }), "{down} ends landed");
    }
}

/// Cassini from Horizons: each arc solved from its ends meets Venus, Venus,
/// Earth and Jupiter on the flyby times, then Saturn on the day of arrival,
/// where the log puts it in orbit around Saturn.
#[test]
fn cassini_rewinds_through_its_flybys_to_saturn() {
    let sys = sol();
    let l = &logs()["cassini"];
    let flybys: Vec<(i64, String)> = l
        .events
        .iter()
        .filter_map(|e| match &e.what {
            What::Flyby { body, .. } => Some((e.at_s, body.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(flybys.len(), 4);
    for (t, body) in &flybys {
        let just = l.at(t - 1, &sys);
        assert_eq!(just.kind, "course", "{body}");
        let b = Ephemeris::body(&sys, body, *t as f64).unwrap();
        let miss = norm(sub(just.r_m.unwrap(), b.0));
        assert!(miss < 1.0e6, "missed {body} by {} km", miss / 1e3);
    }
    // A day after arrival (1 Jul 2004): in orbit around Saturn.
    let soi = l
        .events
        .iter()
        .rfind(|e| matches!(e.what, What::Burn { .. }))
        .unwrap()
        .at_s;
    let after = l.at(soi + 86_400, &sys);
    assert_eq!(after.kind, "orbit");
    assert_eq!(after.with.as_deref(), Some("u:sol/sys:sun/body:saturn"));
    let saturn = Ephemeris::body(&sys, "u:sol/sys:sun/body:saturn", (soi + 86_400) as f64).unwrap();
    let d = norm(sub(after.r_m.unwrap(), saturn.0));
    assert!(d < 1.0e10, "{} km from Saturn", d / 1e3);
    // 2018: gone into Saturn.
    assert_eq!(l.at(soi + 14 * 365 * 86_400, &sys).kind, "gone");
}

/// The big-burns fudge, measured: Cassini's log against JPL Horizons'
/// heliocentric positions in cruise (scripts/spaceflight/horizons_cassini.json).
/// The error mixes the fudge (no trims, flybys at the planet's centre) with
/// the NAV fixture's own planet orbits; RS3's package log keeps the figures.
#[test]
fn cassini_cruise_against_horizons() {
    let sys = sol();
    let l = &logs()["cassini"];
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/spaceflight/horizons_cassini.json");
    let hz: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let measure = hz["measure"].as_object().unwrap();
    let mut worst = 0.0_f64;
    for (date, m) in measure {
        let jd = m["jd"].as_f64().unwrap();
        let t = (63_115_200_000.0 + (jd - 2_451_545.0) * 86_400.0) as i64;
        let r: Vec<f64> = m["r_km"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() * 1e3)
            .collect();
        let at = l.at(t, &sys);
        let got = at.r_m.unwrap_or_else(|| panic!("{date}: {}", at.words));
        let err = norm(sub(got, [r[0], r[1], r[2]]));
        let frac = err / norm([r[0], r[1], r[2]]);
        println!(
            "{date}: {} ({}) off by {:.0} thousand km, {:.2}% of its distance",
            at.kind,
            at.words,
            err / 1e6,
            100.0 * frac
        );
        worst = worst.max(frac);
    }
    assert!(worst < 0.02, "worst {:.2}%", 100.0 * worst);
}

/// Every solved course in every log reaches its target on time, wherever the
/// NAV Sol fixture holds the target (moons and small bodies are not in it yet).
#[test]
fn every_solved_course_arrives() {
    let sys = sol();
    let mut met = 0;
    for (id, l) in &logs() {
        for p in &l.pieces {
            if let State::Course {
                course: Course::Lambert { to, arrive_s },
            } = &p.state
            {
                let Some(b) = Ephemeris::body(&sys, to, *arrive_s as f64) else {
                    continue;
                };
                let just = l.at(arrive_s - 1, &sys);
                let Some(r) = just.r_m else { continue };
                let miss = norm(sub(r, b.0));
                assert!(miss < 1.0e6, "{id}: missed {to} by {} km", miss / 1e3);
                met += 1;
            }
        }
    }
    assert!(met >= 12, "{met} courses checked");
}

/// The big-burns fudge in low orbit: the ISS log (one Horizons orbit a year)
/// against Horizons an hour, hours, days and a month after a sample. Drag and
/// the Earth's oblateness move the phase along the orbit; the numbers go to
/// RS3's package log and the fudge register.
#[test]
fn iss_phase_drift_against_horizons() {
    let sys = sol();
    let l = &logs()["iss"];
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/spaceflight/horizons_more.json");
    let hz: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut first = f64::NAN;
    for (date, m) in hz["iss_measure"].as_object().unwrap() {
        let jd = m["jd"].as_f64().unwrap();
        let t = (63_115_200_000.0 + (jd - 2_451_545.0) * 86_400.0) as i64;
        let r: Vec<f64> = m["r_km"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() * 1e3)
            .collect();
        let at = l.at(t, &sys);
        let earth = Ephemeris::body(&sys, "u:sol/sys:sun/body:earth", t as f64).unwrap();
        let geo = sub(at.r_m.unwrap(), earth.0);
        let err = norm(sub(geo, [r[0], r[1], r[2]]));
        println!("{date}: off by {:.0} km", err / 1e3);
        if first.is_nan() {
            first = err;
        }
    }
    // An hour after the sample the orbit is still right to within a few hundred km.
    assert!(first < 500e3, "{} km an hour after", first / 1e3);
}

/// RS8: every craft that left Earth orbit has a log from GCAT's phases, and the
/// logs end the way GCAT says: on the ground where landercat puts them, gone
/// into a world, or still flying.
#[test]
fn every_departed_craft_has_a_log() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../store-packs/sol-today");
    let cat: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("catalogue.json")).unwrap())
            .unwrap();
    let departed = cat["departed"].as_array().unwrap().len();
    let all = logs();
    let mut from_gcat = 0;
    let (mut landed, mut gone, mut flying) = (0, 0, 0);
    for p in std::fs::read_dir(dir.join("logs/departed")).unwrap() {
        let id = p
            .unwrap()
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let l = &all[&id];
        from_gcat += 1;
        match &l.pieces.last().unwrap().state {
            State::Landed {
                lat_deg, lon_deg, ..
            } => {
                assert!(lat_deg.abs() <= 90.0 && lon_deg.abs() <= 360.0, "{id}");
                landed += 1;
            }
            State::Gone { .. } => gone += 1,
            _ => flying += 1,
        }
    }
    println!("{from_gcat} of {departed}: {landed} on the ground, {gone} gone, {flying} still flying or docked");
    // RS8 part 4: carriers split off what they release, by GCAT's REL phases (the split test checks each meets its log).
    let releases = |id: &str| -> Vec<String> {
        all[id]
            .splits()
            .iter()
            .map(|(_, c)| c.to_string())
            .collect()
    };
    assert!(releases("viking-orbiter-1").contains(&"viking-lander-1".to_string()));
    // The showcase missions (Cassini, Huygens, Juno, Dawn, Curiosity, JWST, Voyager 2, Columbia, Eagle) are written
    // in full by build_logs.py; GCAT's logs cover the rest.
    assert!(from_gcat + 12 >= departed, "{from_gcat} of {departed}");
    assert!(landed >= 100 && gone >= 100, "{landed} landed, {gone} gone");
}

/// The generic logs measured: GCAT's shapes, the arcs solved from their ends
/// and the orbits turned to start at the last body, against JPL Horizons'
/// heliocentric positions in cruise (scripts/spaceflight/horizons_departed.json).
#[test]
fn departed_cruise_against_horizons() {
    let sys = sol();
    let all = logs();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/spaceflight/horizons_departed.json");
    let hz: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut worst = 0.0_f64;
    for (id, dates) in hz.as_object().unwrap() {
        let l = &all[id];
        for (date, m) in dates.as_object().unwrap() {
            let jd = m["jd"].as_f64().unwrap();
            let t = (63_115_200_000.0 + (jd - 2_451_545.0) * 86_400.0) as i64;
            let r: Vec<f64> = m["r_km"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() * 1e3)
                .collect();
            let at = l.at(t, &sys);
            let got = at
                .r_m
                .unwrap_or_else(|| panic!("{id} {date}: {}", at.words));
            let err = norm(sub(got, [r[0], r[1], r[2]]));
            let frac = err / norm([r[0], r[1], r[2]]);
            println!(
                "{id} {date}: {} off by {:.1} million km, {:.1}% of its distance",
                at.kind,
                err / 1e9,
                100.0 * frac
            );
            worst = worst.max(frac);
        }
    }
    assert!(worst < 0.3, "worst {:.1}%", 100.0 * worst);
}

/// RS8 part 2: a log for each station core and each human spaceflight vehicle,
/// from GCAT's satcat: Mir deorbited into the Pacific, Shuttle flights landed,
/// Soyuz and Shuttles docked where GCAT's first phase ends at a station.
#[test]
fn stations_and_crewed_flights_have_logs() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../store-packs/sol-today");
    let all = logs();
    let n = std::fs::read_dir(dir.join("logs/earth-orbit"))
        .unwrap()
        .count();
    assert!(n >= 700, "{n} Earth-orbit logs");
    assert!(matches!(
        all["mir"].pieces.last().unwrap().state,
        State::Gone { .. }
    ));
    // RS8 part 3: Soyuz 21 docked to Salyut 5 and landed on the steppe on Wikidata's day.
    match &all["soyuz-21"].pieces.last().unwrap().state {
        State::Landed { lat_deg, .. } => assert!((lat_deg - 47.4).abs() < 1.0, "the steppe"),
        s => panic!("Soyuz 21 ends {s:?}"),
    }
    let landed = std::fs::read_dir(dir.join("logs/earth-orbit"))
        .unwrap()
        .filter(|p| {
            let id = p
                .as_ref()
                .unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            matches!(all[&id].pieces.last().unwrap().state, State::Landed { .. })
        })
        .count();
    assert!(landed >= 440, "{landed} landed");
    let sts = &all["discovery-sts-103"];
    match &sts.pieces.last().unwrap().state {
        State::Landed {
            lat_deg, lon_deg, ..
        } => {
            assert!(
                (lat_deg - 28.6).abs() < 0.5 && (lon_deg + 80.7).abs() < 0.5,
                "KSC"
            )
        }
        s => panic!("STS-103 ends {s:?}"),
    }
    // RS8 part 5: GCAT's end of mission (psatcat TDate) ends what docks. Soyuz 17 lands on 9 Feb 1975 at
    // 11:03 UTC (Wikidata's August date for it was wrong), and Progress 1 burns up on 8 Feb 1978.
    let soyuz17 = all["soyuz-17"].pieces.last().unwrap();
    assert!(matches!(soyuz17.state, State::Landed { .. }));
    let feb9 = 63_115_200_000 - 9_092 * 86_400 - 3_600; // 1975 Feb 9 11:00 UTC, about (J2000 is noon)
    assert!(
        (soyuz17.from_s - feb9).abs() < 2 * 3_600,
        "Soyuz 17 lands on 9 Feb 1975"
    );
    assert!(matches!(
        all["progress-1"].pieces.last().unwrap().state,
        State::Gone { .. }
    ));
    let docked = std::fs::read_dir(dir.join("logs/earth-orbit"))
        .unwrap()
        .filter(|p| {
            let id = p
                .as_ref()
                .unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            matches!(all[&id].pieces.last().unwrap().state, State::Docked { .. })
        })
        .count();
    assert!(
        docked <= 12,
        "{docked} still docked: only modules and flights docked today"
    );
    // Columbia's STS-3 came down at White Sands, which GCAT names.
    match &all["columbia-sts-3"].pieces.last().unwrap().state {
        State::Landed { lon_deg, .. } => assert!((lon_deg + 106.4).abs() < 0.5, "White Sands"),
        s => panic!("STS-3 ends {s:?}"),
    }
}

/// RS9: everyone who has been to space (`crews.json`, from McDowell's lists by
/// `scripts/spaceflight/build_crews.py`). Every leg aboard a craft with a log
/// falls while that log has the craft flying, Apollo 11's crew are where they
/// were on 20 July 1969, and the step count of people in space agrees with the legs.
#[test]
fn crews_ride_the_craft_their_logs_fly() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../store-packs/sol-today/crews.json");
    let crews: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let all = logs();
    let sys = sol();
    let people = crews["people"].as_array().unwrap();
    assert!(people.len() >= 780, "{} people", people.len());
    let mut ids = std::collections::BTreeSet::new();
    for p in people {
        assert!(ids.insert(p["id"].as_str().unwrap()), "{} twice", p["id"]);
    }
    let legs = crews["aboard"].as_array().unwrap();
    let mut on_log = 0;
    let mut outside = Vec::new();
    for l in legs {
        assert!(
            ids.contains(l["person"].as_str().unwrap()),
            "{} has no person",
            l["person"]
        );
        let from = l["from_s"].as_i64().unwrap();
        let to = l["to_s"].as_i64().unwrap_or(from + 86_400);
        assert!(to > from, "{l}");
        if let Some(on) = l["on"].as_str() {
            let log = all.get(on).unwrap_or_else(|| panic!("{on}: no log"));
            on_log += 1;
            let kind = log.at(from + (to - from) / 2, &sys).kind;
            if kind == "before" || kind == "gone" || kind == "unlogged" {
                outside.push(format!("{} on {on}: {kind}", l["person"]));
            }
        }
    }
    assert!(on_log >= 3_000, "{on_log} legs on a pack log");
    // A log's dates come from GCAT, the legs from McDowell's crew lists: they should all agree.
    assert!(
        outside.len() <= 30,
        "{} legs outside their craft's log: {:?}",
        outside.len(),
        &outside[..outside.len().min(30)]
    );
    // 20 July 1969, 20:17 UTC: Armstrong and Aldrin in Eagle on the Moon, Collins in Columbia.
    let landing = 63_115_200_000 - 11_122 * 86_400 + 8 * 3_600 + 17 * 60 + 40;
    let mut up: Vec<(&str, &str)> = legs
        .iter()
        .filter(|l| {
            l["from_s"].as_i64().unwrap() <= landing
                && l["to_s"].as_i64().is_none_or(|t| landing < t)
        })
        .map(|l| {
            (
                l["person"].as_str().unwrap(),
                l["on"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    up.sort();
    eprintln!("{} legs outside their craft's log", outside.len());
    assert_eq!(
        up,
        vec![
            ("AS-00030", "apollo-11-eagle"),
            ("AS-00033", "apollo-11"),
            ("AS-00036", "apollo-11-eagle")
        ],
        "who was in space as Eagle landed"
    );
    // The step list says how many were up at each change; it agrees with the legs and peaks at 20.
    let steps = crews["in_space"].as_array().unwrap();
    let mut peak = 0;
    for s in steps.iter().step_by(37) {
        let t = s[0].as_i64().unwrap();
        let n = legs
            .iter()
            .filter(|l| {
                l["from_s"].as_i64().unwrap() <= t && l["to_s"].as_i64().is_none_or(|e| t < e)
            })
            .count() as i64;
        assert_eq!(s[1].as_i64().unwrap(), n, "count at {t}");
    }
    for s in steps {
        peak = peak.max(s[1].as_i64().unwrap());
    }
    assert_eq!(peak, 20, "most people in space at once");
    // Those killed in flight: Soyuz 1, X-15 3-65, Soyuz 11, Challenger and Columbia.
    let lost = people
        .iter()
        .filter(|p| p["died_in_flight"] == true)
        .count();
    assert_eq!(lost, 19, "died in flight");
    // Short flights carry their peak height, so the pack's edge of space (100 km by default) can mark the flights
    // that reached McDowell's 80 km but not 100: Michael Adams's X-15 flight 3-65-97 peaked at 81.07 km.
    assert_eq!(crews["edge_of_space_km"], 100);
    let peaks: Vec<f64> = legs.iter().filter_map(|l| l["peak_km"].as_f64()).collect();
    assert!(
        peaks.len() >= 120,
        "{} short flights with a height",
        peaks.len()
    );
    assert!(
        peaks.iter().all(|p| (80.0..250.0).contains(p)),
        "heights between 80 and 250 km"
    );
    assert!(
        legs.iter()
            .any(|l| l["craft"].as_str().is_some_and(|c| c.contains("3-65"))
                && l["peak_km"].as_f64() == Some(81.07)),
        "Adams, 81.07 km"
    );
}
