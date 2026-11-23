//! Links between places (`links.rs`): gates make their own legs, charted
//! lanes mark the ship's own, one-way links go one way, and a setting that
//! travels only along links keeps to them.
use crate::links::{counted_words, system_of, Link, Links};
use crate::voyage::{
    plan, plan_with_links, Goal, JumpDrive, Minimise, Options, VoyageSettings, VoyageSystem,
};
use crate::Ship;

fn courier() -> (Ship, JumpDrive) {
    let s: Ship = serde_json::from_value(serde_json::json!({
        "name": "Kestrel Wren", "accel_g": 0.3, "sustained": false,
        "exhaust_km_s": 4.4, "wet_t": 120.0, "dry_t": 100.0
    }))
    .unwrap();
    let j = JumpDrive {
        rating_pc: 2.0,
        fuel_t_per_pc: 10.0,
        tank_t: 40.0,
        fuel_t: None,
        days_per_jump: 7.0,
        whole_parsecs: true,
        rule: "test".into(),
        fuel: "hydrogen".into(),
        limit_diameters: None,
    };
    (s, j)
}

fn sys(id: &str, x: f64) -> VoyageSystem {
    VoyageSystem {
        id: id.into(),
        name: id.into(),
        xyz_pc: [x, 0.0, 0.0],
        fuel: vec!["skim".into()],
        surveyed: true,
        fuel_odds: None,
        odds_from: String::new(),
        star: None,
    }
}

fn systems() -> Vec<VoyageSystem> {
    vec![
        sys("u/a", 0.0),
        sys("u/b", 1.0),
        sys("u/c", 2.0),
        sys("u/d", 10.0),
    ]
}

fn opts() -> Options {
    Options {
        minimise: Minimise::HomeTime,
        gambles: false,
        settings: VoyageSettings::default(),
    }
}

fn link(v: serde_json::Value) -> Link {
    serde_json::from_value(v).unwrap()
}

fn gate(one_way: bool) -> Link {
    link(serde_json::json!({
        "id": "gate-ad", "name": "the Long Gate", "kind": "gate", "one_way": one_way,
        "a": { "at": "u/a" },
        "b": { "at": "u/d/body:earth/site:plaza" },
        "rule": { "id": "gate", "name": "gate transit", "family": "gate",
                  "duration": { "form": "fixed", "days": 1.0 }, "messages": { "by": "ships" }, "fictional": true },
        "counted": { "setting.gates": 1.0 }
    }))
}

#[test]
fn a_gate_makes_its_own_leg_where_the_drive_cannot_reach() {
    let (s, j) = courier();
    let sy = systems();
    let to = Goal::To("u/d".into());
    assert!(
        plan(&s, Some(&j), &sy, "u/a", &to, &opts()).is_err(),
        "8 pc is past Jump-2"
    );
    let links = Links {
        links: vec![gate(false)],
        only: false,
    };
    let v = plan_with_links(&s, Some(&j), &sy, "u/a", &to, &opts(), &links).unwrap();
    assert_eq!(v.legs.len(), 1);
    let l = &v.legs[0];
    assert_eq!(
        (l.means.as_str(), l.link.as_deref()),
        ("link", Some("gate-ad"))
    );
    assert_eq!((l.home_days, l.ship_days, l.jump_fuel_t), (1.0, 1.0, 0.0));
    assert!(l.fictional && v.fictional);
    assert!(
        l.words
            .contains("the Long Gate (gate transit), 1 gates · 10.0 pc measured"),
        "{}",
        l.words
    );
    // Two-way by default.
    let back = plan_with_links(
        &s,
        Some(&j),
        &sy,
        "u/d",
        &Goal::To("u/a".into()),
        &opts(),
        &links,
    )
    .unwrap();
    assert_eq!(back.legs[0].link.as_deref(), Some("gate-ad"));
}

#[test]
fn a_one_way_link_goes_one_way() {
    let (s, j) = courier();
    let sy = systems();
    let links = Links {
        links: vec![gate(true)],
        only: false,
    };
    assert!(plan_with_links(
        &s,
        Some(&j),
        &sy,
        "u/a",
        &Goal::To("u/d".into()),
        &opts(),
        &links
    )
    .is_ok());
    assert!(plan_with_links(
        &s,
        Some(&j),
        &sy,
        "u/d",
        &Goal::To("u/a".into()),
        &opts(),
        &links
    )
    .is_err());
}

#[test]
fn charted_lanes_mark_the_ships_own_legs_and_can_be_required() {
    let (s, j) = courier();
    let sy = systems();
    let lane = |id: &str, a: &str, b: &str| {
        link(
            serde_json::json!({ "id": id, "kind": "trade lane", "a": { "at": a }, "b": { "at": b } }),
        )
    };
    let lanes = vec![lane("ab", "u/a", "u/b"), lane("bc", "u/b", "u/c")];
    let to = Goal::To("u/c".into());
    // Free to go: one 2 pc jump straight there, off the lanes.
    let free = plan_with_links(
        &s,
        Some(&j),
        &sy,
        "u/a",
        &to,
        &opts(),
        &Links {
            links: lanes.clone(),
            only: false,
        },
    )
    .unwrap();
    assert_eq!(free.legs.len(), 1);
    assert_eq!(free.legs[0].link, None);
    // Held to the lanes: two jumps, each along its lane, flown with the ship's own drive.
    let held = plan_with_links(
        &s,
        Some(&j),
        &sy,
        "u/a",
        &to,
        &opts(),
        &Links {
            links: lanes,
            only: true,
        },
    )
    .unwrap();
    assert_eq!(
        held.legs
            .iter()
            .map(|l| l.link.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("ab"), Some("bc")]
    );
    assert!(
        held.legs
            .iter()
            .all(|l| l.means == "jump" && l.words.contains("along a charted lane")),
        "{:?}",
        held.legs
    );
    // Nothing joins D, so a lanes-only setting says why.
    let why = plan_with_links(
        &s,
        Some(&j),
        &sy,
        "u/a",
        &Goal::To("u/d".into()),
        &opts(),
        &Links {
            links: vec![],
            only: true,
        },
    )
    .unwrap_err();
    assert!(
        why[0].starts_with("every leg must run along a link"),
        "{why:?}"
    );
}

#[test]
fn an_end_inside_a_system_belongs_to_that_system() {
    let sy = systems();
    assert_eq!(system_of(&sy, "u/d/body:earth/site:plaza"), Some(3));
    assert_eq!(system_of(&sy, "u/d"), Some(3));
    assert_eq!(system_of(&sy, "u/dx"), None);
    let mut l = gate(false);
    l.counted.insert("traveller.jumps".into(), 2.0);
    assert_eq!(
        counted_words(&l, 7.31),
        "1 gates · 2 jumps · 7.3 pc measured"
    );
}
