//! OV3 checks (territory overlays design 11): slots and dates, inherit down,
//! roll up and divided, the reach rule met halfway, outlines, whose map, and
//! the same answers in any order of writing.

use super::holdings::*;
use super::*;

fn addr(s: &str) -> Address {
    s.parse().unwrap()
}
fn t(y: i64) -> UniverseTime {
    UniverseTime::years(y)
}
fn pubr() -> Publisher {
    Publisher::new("gm", "record", "1")
}

const CROWN: &str = "u:a/actor:crown";
const LEAGUE: &str = "u:a/actor:league";
const VESH: &str = "u:a/actor:vesh";

fn draft(place: &str, key: &str, status: Status) -> HoldDraft {
    HoldDraft {
        place: addr(place),
        layer: "so.powers".into(),
        key: key.into(),
        status,
        strength: None,
        extent: None,
        by: None,
        tier: None,
        audience: Audience::everyone(),
        writer: "gm".into(),
        reason: None,
        source: None,
        withdraw: false,
    }
}

fn write(ch: &mut Chronicle, d: &HoldDraft, y: i64) {
    hold(ch, d, t(y), &pubr()).unwrap();
}

fn live(ch: &Chronicle, y: i64) -> Vec<Holding> {
    holdings_at(ch.events(), "so.powers", t(y))
}

#[test]
fn a_holding_on_a_system_answers_for_its_moons() {
    let mut ch = Chronicle::new();
    write(&mut ch, &draft("u:a/sys:7", CROWN, Status::Holds), 2291);
    let l = live(&ch, 2300);
    let moon = addr("u:a/sys:7/body:3/body:1");
    let a = who_holds(&moon, &l, &View::gm(), &LayerRules::default(), &NoWorld).unwrap();
    assert_eq!(a.key, CROWN);
    assert_eq!(a.via, Via::Inherited);
    assert_eq!(a.from, Some(addr("u:a/sys:7")));
    assert!(a.why.contains("contains"), "{}", a.why);
    // Before the date, nothing.
    assert!(who_holds(
        &moon,
        &live(&ch, 2290),
        &View::gm(),
        &LayerRules::default(),
        &NoWorld
    )
    .is_none());
    // A more specific holding wins inside its own place.
    write(
        &mut ch,
        &draft("u:a/sys:7/belt:1/station:2", LEAGUE, Status::Holds),
        2295,
    );
    let l = live(&ch, 2300);
    let st = addr("u:a/sys:7/belt:1/station:2");
    let a = who_holds(&st, &l, &View::gm(), &LayerRules::default(), &NoWorld).unwrap();
    assert_eq!((a.key.as_str(), a.via), (LEAGUE, Via::Held));
    let a = who_holds(&moon, &l, &View::gm(), &LayerRules::default(), &NoWorld).unwrap();
    assert_eq!(a.key, CROWN);
}

#[test]
fn a_later_holding_replaces_and_history_keeps_dates() {
    let mut ch = Chronicle::new();
    write(&mut ch, &draft("u:a/sys:7", CROWN, Status::Holds), 2291);
    write(&mut ch, &draft("u:a/sys:7", LEAGUE, Status::Holds), 2310);
    let l = live(&ch, 2320);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].key, LEAGUE);
    let h = history(ch.events(), "so.powers", t(2320));
    assert_eq!(h.len(), 2);
    assert_eq!(h[0].until_s, Some(t(2310).s));
    // At an earlier date the Crown still holds it, with no end yet.
    let l = live(&ch, 2300);
    assert_eq!(l[0].key, CROWN);
    assert_eq!(l[0].until_s, None);
    // Withdrawn: bare paper.
    let mut w = draft("u:a/sys:7", LEAGUE, Status::Holds);
    w.withdraw = true;
    write(&mut ch, &w, 2330);
    assert!(live(&ch, 2340).is_empty());
}

#[test]
fn a_divided_world_rolls_up_divided() {
    let mut ch = Chronicle::new();
    let w = "u:a/sys:7/body:3";
    for (i, k) in [CROWN, LEAGUE, VESH, CROWN, LEAGUE].iter().enumerate() {
        write(
            &mut ch,
            &draft(&format!("{w}/region:{i}"), k, Status::Holds),
            2291,
        );
    }
    let l = live(&ch, 2300);
    let a = who_holds(&addr(w), &l, &View::gm(), &LayerRules::default(), &NoWorld).unwrap();
    assert_eq!(a.via, Via::RolledUp);
    assert_eq!(a.divided.len(), 3, "{a:?}");
    assert_eq!(a.divided[0].key, CROWN);
    assert!((a.divided[0].share - 0.4).abs() < 1e-12);
    assert!(a.why.starts_with("divided"), "{}", a.why);
    // The system above rolls up divided too.
    let s = who_holds(
        &addr("u:a/sys:7"),
        &l,
        &View::gm(),
        &LayerRules::default(),
        &NoWorld,
    )
    .unwrap();
    assert_eq!(s.divided.len(), 3);
}

struct Weights(BTreeMap<Address, f64>);
impl World for Weights {
    fn position(&self, _: &Address) -> Option<[f64; 3]> {
        None
    }
    fn weight(&self, a: &Address, kind: &str) -> Option<f64> {
        (kind == "population")
            .then(|| self.0.get(a).copied())
            .flatten()
    }
}

#[test]
fn rolling_up_weighs_by_population() {
    let mut ch = Chronicle::new();
    let w = "u:a/sys:7/body:3";
    write(&mut ch, &draft(w, CROWN, Status::Holds), 2291);
    write(
        &mut ch,
        &draft(&format!("{w}/region:1"), LEAGUE, Status::Holds),
        2291,
    );
    let l = live(&ch, 2300);
    let world = Weights([(addr(w), 10.0e9), (addr(&format!("{w}/region:1")), 2.0e9)].into());
    let rules = LayerRules {
        weigh_by: Some("population".into()),
        ..Default::default()
    };
    let a = who_holds(&addr(w), &l, &View::gm(), &rules, &world).unwrap();
    assert_eq!(a.key, CROWN);
    assert_eq!(a.via, Via::RolledUp);
    assert!(a.divided.is_empty());
    assert!(
        a.why.contains("80 %") && a.why.contains("by population"),
        "{}",
        a.why
    );
    // With most of the people in the League's region, the world is divided.
    let world = Weights([(addr(w), 10.0e9), (addr(&format!("{w}/region:1")), 5.0e9)].into());
    let a = who_holds(&addr(w), &l, &View::gm(), &rules, &world).unwrap();
    assert_eq!(a.divided.len(), 2);
}

struct Line;
impl World for Line {
    fn position(&self, a: &Address) -> Option<[f64; 3]> {
        // Systems on a line, one parsec apart by their index.
        let s = a.segments().iter().find(|s| s.kind() == "sys")?;
        let x: f64 = s.key().parse().ok()?;
        Some([x * sse_core::sheet::PC, 0.0, 0.0])
    }
    fn weight(&self, _: &Address, _: &str) -> Option<f64> {
        None
    }
}

#[test]
fn between_stars_borders_run_midway_and_the_void_stays_bare() {
    let mut ch = Chronicle::new();
    write(&mut ch, &draft("u:a/sys:0", CROWN, Status::Holds), 2291);
    write(&mut ch, &draft("u:a/sys:6", LEAGUE, Status::Holds), 2291);
    let l = live(&ch, 2300);
    let rules = LayerRules {
        reach_m: Some(4.0 * sse_core::sheet::PC),
        ..Default::default()
    };
    let at = |i: u32| {
        who_holds(
            &addr(&format!("u:a/sys:{i}")),
            &l,
            &View::gm(),
            &rules,
            &Line,
        )
    };
    assert_eq!(at(2).unwrap().key, CROWN);
    assert_eq!(at(2).unwrap().via, Via::Rule);
    assert_eq!(at(4).unwrap().key, LEAGUE);
    assert!(
        at(2).unwrap().why.contains("2.0 pc"),
        "{}",
        at(2).unwrap().why
    );
    // Midway, the tie goes to the first by address, the same on every device.
    assert_eq!(at(3).unwrap().key, CROWN);
    // Beyond every reach: unclaimed.
    assert!(at(11).is_none());
    // A holding's own reach beats the layer's.
    let mut d = draft("u:a/sys:20", VESH, Status::Holds);
    d.extent = Some(Extent::Reach {
        reach: Distance {
            value: 1.0,
            unit: "pc".into(),
        },
    });
    write(&mut ch, &d, 2295);
    let l = live(&ch, 2300);
    let at = |i: u32| {
        who_holds(
            &addr(&format!("u:a/sys:{i}")),
            &l,
            &View::gm(),
            &rules,
            &Line,
        )
    };
    assert_eq!(at(21).unwrap().key, VESH);
    assert!(at(22).is_none());
}

#[test]
fn outlines_answer_points_on_a_body() {
    let mut ch = Chronicle::new();
    let body = "u:a/sys:7/body:3";
    write(&mut ch, &draft(body, CROWN, Status::Holds), 2291);
    // A drawn country: League holds a box of longitude 10..20, latitude 0..10.
    let mut d = draft(body, LEAGUE, Status::Holds);
    d.extent = Some(Extent::Outline {
        outline: vec![[10.0, 0.0], [20.0, 0.0], [20.0, 10.0], [10.0, 10.0]],
        frame: "body-latlon".into(),
    });
    // A drawn holding shares the place's holds slot, so it goes on a region.
    d.place = addr(&format!("{body}/region:1"));
    write(&mut ch, &d, 2295);
    let l = live(&ch, 2300);
    let on = addr(&format!("{body}/region:1"));
    let p = |x: f64, y: f64| {
        who_holds_point(
            &on,
            "body-latlon",
            [x, y],
            &l,
            &View::gm(),
            &LayerRules::default(),
            &NoWorld,
        )
    };
    assert_eq!(p(15.0, 5.0).unwrap().key, LEAGUE);
    // Outside the outline the region takes the world's holder.
    assert_eq!(p(25.0, 5.0).unwrap().key, CROWN);
    // A bad frame is refused.
    let mut bad = draft(body, VESH, Status::Holds);
    bad.extent = Some(Extent::Outline {
        outline: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        frame: "screen".into(),
    });
    assert!(hold(&mut ch, &bad, t(2300), &pubr()).is_err());
}

#[test]
fn whose_map_as_it_is_as_a_power_draws_it_and_as_players_know_it() {
    let mut ch = Chronicle::new();
    write(&mut ch, &draft("u:a/sys:7", CROWN, Status::Holds), 2291);
    let mut c = draft("u:a/sys:7", LEAGUE, Status::Claims);
    c.by = Some(LEAGUE.into());
    write(&mut ch, &c, 2292);
    let mut secret = draft("u:a/sys:8", VESH, Status::Holds);
    secret.audience = Audience::gm();
    write(&mut ch, &secret, 2293);
    let l = live(&ch, 2300);
    let r = LayerRules::default();
    let gm = who_holds(&addr("u:a/sys:7"), &l, &View::gm(), &r, &NoWorld).unwrap();
    assert_eq!(gm.key, CROWN);
    assert_eq!(gm.also.len(), 1);
    assert_eq!(gm.also[0].status, Status::Claims);
    let league = View {
        audience: "gm".into(),
        according_to: Some(LEAGUE.into()),
    };
    assert_eq!(
        who_holds(&addr("u:a/sys:7"), &l, &league, &r, &NoWorld)
            .unwrap()
            .key,
        LEAGUE
    );
    let players = View {
        audience: "crew".into(),
        according_to: None,
    };
    assert!(who_holds(&addr("u:a/sys:8"), &l, &players, &r, &NoWorld).is_none());
    assert!(who_holds(&addr("u:a/sys:8"), &l, &View::gm(), &r, &NoWorld).is_some());
    // holdings_in lists the bounds and every place held inside it.
    let all = holdings_in(&addr("u:a"), &l, &View::gm(), &r, &NoWorld);
    let places: Vec<String> = all.iter().map(|a| a.place.to_string()).collect();
    assert_eq!(places, ["u:a", "u:a/sys:7", "u:a/sys:8"]);
    assert_eq!(all[0].divided.len(), 2);
    let seen = holdings_in(&addr("u:a"), &l, &players, &r, &NoWorld);
    assert_eq!(
        seen.len(),
        2,
        "the secret holding leaves no trace: {seen:?}"
    );
}

#[test]
fn the_same_answers_in_any_order_of_writing() {
    let writes = [
        ("u:a/sys:7", CROWN, 2291),
        ("u:a/sys:7/body:3/region:1", LEAGUE, 2295),
        ("u:a/sys:9", VESH, 2300),
        ("u:a/sys:7", LEAGUE, 2305),
    ];
    let run = |order: &[usize]| {
        let mut ch = Chronicle::new();
        for &i in order {
            let (p, k, y) = writes[i];
            write(&mut ch, &draft(p, k, Status::Holds), y);
        }
        let l = live(&ch, 2310);
        serde_json::to_string(&holdings_in(
            &addr("u:a"),
            &l,
            &View::gm(),
            &LayerRules::default(),
            &NoWorld,
        ))
        .unwrap()
    };
    let a = run(&[0, 1, 2, 3]);
    assert_eq!(a, run(&[3, 2, 1, 0]));
    assert_eq!(a, run(&[2, 0, 3, 1]));
}

#[test]
fn holdings_never_touch_the_folded_state() {
    let mut ch = Chronicle::new();
    let before = ch.state_at(t(3000));
    write(&mut ch, &draft("u:a/sys:7", CROWN, Status::Holds), 2291);
    let after = ch.state_at(t(3000));
    assert_eq!(before.facts, after.facts);
    let mut bad = draft("u:a/sys:7", CROWN, Status::Influences);
    bad.strength = Some(1.5);
    assert!(hold(&mut ch, &bad, t(2300), &pubr()).is_err());
}
