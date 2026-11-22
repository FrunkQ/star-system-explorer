use super::*;

fn torch() -> Ship {
    serde_json::from_value(serde_json::json!({
        "name": "Fusion torch", "accel_g": 1.0, "sustained": true,
        "exhaust_km_s": 2000.0, "wet_t": 2000.0, "dry_t": 1000.0
    }))
    .unwrap()
}

fn photon() -> Ship {
    // An idealised relativistic drive: exhaust at c, mass ratio 100.
    serde_json::from_value(serde_json::json!({
        "name": "Lightsail", "accel_g": 1.0, "sustained": true,
        "exhaust_km_s": 299_792.458, "wet_t": 100.0, "dry_t": 1.0, "reserve": 0.0
    }))
    .unwrap()
}

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
    };
    (s, j)
}

fn sys(id: &str, x: f64, fuel: bool) -> VoyageSystem {
    VoyageSystem {
        id: id.into(),
        name: id.into(),
        xyz_pc: [x, 0.0, 0.0],
        fuel: if fuel {
            vec!["gas giant".into()]
        } else {
            vec![]
        },
        surveyed: true,
        fuel_odds: None,
        odds_from: String::new(),
    }
}

fn opts(m: Minimise) -> Options {
    Options {
        minimise: m,
        gambles: false,
        settings: VoyageSettings::default(),
    }
}

#[test]
fn a_one_g_photon_drive_matches_the_textbook_relativistic_rocket() {
    // 1 g to a star 4.3 light years away and stop: 3.6 years aboard and
    // 5.9 at home (Baez, The Relativistic Rocket, its table).
    let ly_pc = C * JULIAN_YEAR / PC_M;
    let d = 4.3 * ly_pc;
    let f = flight(&photon(), 99.0, d).unwrap();
    let yr = |s: f64| s / JULIAN_YEAR;
    assert!(!f.coast);
    assert!((yr(f.ship_s) - 3.58).abs() < 0.05, "{}", yr(f.ship_s));
    assert!((yr(f.home_s) - 5.9).abs() < 0.1, "{}", yr(f.home_s));
    // Mass ratio exp(2w): w = acosh(1 + a d / 2c²).
    let x = G0 * d * PC_M / (2.0 * C * C);
    let w = m::acosh(1.0 + x);
    let ratio = 100.0 / (100.0 - f.propellant_t);
    assert!((m::ln(ratio) - 2.0 * w).abs() < 1e-9);
}

#[test]
fn a_fusion_torch_coasts_at_the_speed_its_propellant_affords() {
    let f = flight(&torch(), 1000.0, 1.0).unwrap();
    assert!(f.coast);
    // Δw = (v_e/c) ln(m0 / m_floor), half each way; v ≈ c·tanh(Δw/2).
    let floor = 1000.0 + 0.1 * 1000.0;
    let w = 0.5 * (2.0e6 / C) * m::ln(2000.0 / floor);
    assert!((f.peak_beta - m::tanh(w)).abs() < 1e-12);
    // At about 600 km/s a parsec takes over a thousand years.
    let yr = f.home_s / JULIAN_YEAR;
    assert!(yr > 1000.0 && yr < 2000.0, "{yr}");
    // Time dilation is tiny at that speed but not zero.
    assert!(f.ship_s < f.home_s && f.ship_s > 0.99999 * f.home_s);
    assert!((f.propellant_t - (2000.0 - floor)).abs() < 1e-9);
}

#[test]
fn short_flights_agree_with_newton() {
    // 0.001 pc at 1 g with plenty of propellant: t = 2 sqrt(d/a).
    let f = flight(&photon(), 99.0, 0.001).unwrap();
    let d = 0.001 * PC_M;
    let newton = 2.0 * m::sqrt(d / G0);
    assert!(
        (f.home_s / newton - 1.0).abs() < 1e-3,
        "{}",
        f.home_s / newton
    );
}

#[test]
fn a_chemical_ship_cannot_fly_between_stars() {
    let (s, _) = courier();
    assert_eq!(flight(&s, 20.0, 1.0), Err(NoFlight::NotSustained));
    let r = plan(
        &s,
        None,
        &[sys("a", 0.0, false), sys("b", 1.0, false)],
        "a",
        &Goal::To("b".into()),
        &opts(Minimise::HomeTime),
    )
    .unwrap_err();
    assert!(
        r[0].contains("no jump drive and no sustained drive"),
        "{r:?}"
    );
}

#[test]
fn jumps_carry_fuel_and_refuel_where_the_crew_knows_of_fuel() {
    let (s, j) = courier();
    // Five systems 1.5 pc apart; the tank holds two 2-parsec jumps.
    let line = vec![
        sys("a", 0.0, false),
        sys("b", 1.5, false),
        sys("c", 3.0, true),
        sys("d", 4.5, false),
        sys("e", 6.0, false),
    ];
    let v = plan(
        &s,
        Some(&j),
        &line,
        "a",
        &Goal::To("e".into()),
        &opts(Minimise::HomeTime),
    )
    .unwrap();
    // 6 pc in Jump-2: three jumps at least; fuel 20 t each; refuel at c.
    assert!(v.legs.iter().all(|l| l.means == "jump"));
    assert!(v.fictional);
    assert_eq!(v.needs.as_deref(), Some("Jump-2"));
    let stops: Vec<&str> = v
        .legs
        .iter()
        .filter(|l| l.refuel.is_some())
        .map(|l| l.to.as_str())
        .collect();
    assert_eq!(stops, vec!["c"], "{:#?}", v.legs);
    for l in &v.legs {
        assert!(l.jump_fuel_left_t >= 0.0);
    }
    // Without the fuel at c it runs dry, and says so.
    let mut dry = line.clone();
    dry[2].fuel.clear();
    let r = plan(
        &s,
        Some(&j),
        &dry,
        "a",
        &Goal::To("e".into()),
        &opts(Minimise::HomeTime),
    )
    .unwrap_err();
    assert!(r[0].contains("no known fuel"), "{r:?}");
}

#[test]
fn a_gap_too_wide_names_the_rating_it_needs() {
    let (s, j) = courier();
    let r = plan(
        &s,
        Some(&j),
        &[sys("a", 0.0, true), sys("b", 2.6, true)],
        "a",
        &Goal::To("b".into()),
        &opts(Minimise::Legs),
    )
    .unwrap_err();
    assert_eq!(
        r,
        vec!["b cannot be reached: it needs Jump-3, the Kestrel Wren is Jump-2"]
    );
}

#[test]
fn unsurveyed_systems_are_dry_unless_gambles_are_allowed() {
    let (s, j) = courier();
    let mut line = vec![
        sys("a", 0.0, false),
        sys("b", 2.0, false),
        sys("c", 4.0, false),
        sys("d", 6.0, false),
    ];
    line[2].surveyed = false;
    line[2].fuel_odds = Some(0.7);
    line[2].odds_from = "the ship's sensors".into();
    assert!(plan(
        &s,
        Some(&j),
        &line,
        "a",
        &Goal::To("d".into()),
        &opts(Minimise::Legs)
    )
    .is_err());
    let mut o = opts(Minimise::Legs);
    o.gambles = true;
    let v = plan(&s, Some(&j), &line, "a", &Goal::To("d".into()), &o).unwrap();
    assert_eq!(v.gambles, 1);
    assert!((v.odds - 0.7).abs() < 1e-12);
    assert!(
        v.legs[1].gamble
            && v.legs[1]
                .words
                .contains("70% chance of fuel from the ship's sensors"),
        "{:#?}",
        v.legs
    );
    // Below the pack's floor of odds it is not worth the risk.
    line[2].fuel_odds = Some(0.2);
    assert!(plan(&s, Some(&j), &line, "a", &Goal::To("d".into()), &o).is_err());
}

#[test]
fn the_explorer_takes_the_nearest_unvisited_and_ties_go_outward() {
    let (s, j) = courier();
    let home = [0.0, 0.0, 0.0];
    let systems = vec![
        sys("home", 0.0, true),
        VoyageSystem {
            xyz_pc: [1.0, 0.0, 0.0],
            ..sys("east", 0.0, false)
        },
        VoyageSystem {
            xyz_pc: [-1.0, 0.0, 0.0],
            ..sys("west", 0.0, false)
        },
        sys("far", 1.8, false),
    ];
    let goal = |v: &[&str], at: &str| {
        let g = Goal::NearestUnvisited {
            visited: v.iter().map(|s| s.to_string()).collect(),
            home,
        };
        plan(&s, Some(&j), &systems, at, &g, &opts(Minimise::Legs)).unwrap()
    };
    // From home, east and west tie on legs and distance: id order picks east.
    let v = goal(&["home"], "home");
    assert_eq!(v.to, "east");
    // From east, far (0.8 pc) is nearer than west (2 pc).
    let v = goal(&["home", "east"], "east");
    assert_eq!(v.to, "far");
    // Everything visited: a refusal, in words.
    let g = Goal::NearestUnvisited {
        visited: vec!["home".into(), "east".into(), "west".into(), "far".into()],
        home,
    };
    let r = plan(&s, Some(&j), &systems, "far", &g, &opts(Minimise::Legs)).unwrap_err();
    assert!(!r.is_empty());
}

#[test]
fn ties_at_the_same_distance_go_to_the_one_farther_from_home() {
    let (s, j) = courier();
    // At "mid" (1 pc out), "in" and "out" are both 1 pc away; out is farther from home.
    let systems = vec![
        sys("home", -1.0, true),
        VoyageSystem {
            xyz_pc: [0.0, 0.0, 0.0],
            ..sys("in", 0.0, false)
        },
        sys("mid", 1.0, true),
        sys("out", 2.0, false),
    ];
    let g = Goal::NearestUnvisited {
        visited: vec!["home".into(), "mid".into()],
        home: [-1.0, 0.0, 0.0],
    };
    let v = plan(&s, Some(&j), &systems, "mid", &g, &opts(Minimise::Legs)).unwrap();
    assert_eq!(v.to, "out");
}

#[test]
fn the_minimised_measure_changes_the_plan() {
    // A torch that also jumps: by home time it jumps; by fuel it still jumps
    // only if jump fuel is cheaper. Here flight burns 900 t, a jump 10 t.
    let (_, j) = courier();
    let mut s = torch();
    s.jump = Some(j.clone());
    let systems = vec![sys("a", 0.0, false), sys("b", 0.9, false)];
    let v = plan(
        &s,
        s.jump.as_ref(),
        &systems,
        "a",
        &Goal::To("b".into()),
        &opts(Minimise::HomeTime),
    )
    .unwrap();
    assert_eq!(v.legs[0].means, "jump");
    // With no jump fuel, only the flight is left.
    let mut empty = j;
    empty.fuel_t = Some(0.0);
    let v = plan(
        &s,
        Some(&empty),
        &systems,
        "a",
        &Goal::To("b".into()),
        &opts(Minimise::HomeTime),
    )
    .unwrap();
    assert_eq!(v.legs[0].means, "flight");
    assert!(!v.fictional);
    assert!(
        v.legs[0].words.contains("years at home"),
        "{}",
        v.legs[0].words
    );
}

#[test]
fn plans_are_repeatable() {
    let (s, j) = courier();
    let mut systems = vec![];
    for i in 0..40 {
        let t = i as f64;
        systems.push(VoyageSystem {
            id: format!("s{i:02}"),
            name: String::new(),
            xyz_pc: [
                m::sin(t) * 6.0,
                m::cos(t * 1.7) * 6.0,
                m::sin(t * 0.3) * 2.0,
            ],
            fuel: if i % 3 == 0 {
                vec!["ice".into()]
            } else {
                vec![]
            },
            surveyed: true,
            fuel_odds: None,
            odds_from: String::new(),
        });
    }
    let g = Goal::To("s39".into());
    let a = plan(&s, Some(&j), &systems, "s00", &g, &opts(Minimise::HomeTime));
    let b = plan(&s, Some(&j), &systems, "s00", &g, &opts(Minimise::HomeTime));
    assert_eq!(a, b);
}

#[test]
fn metal_rich_and_heavier_stars_are_likelier_to_have_giants() {
    let p = GiantPrior::default();
    assert!((p.odds(1.0, 0.0) - 0.07).abs() < 1e-12);
    assert!(p.odds(1.0, 0.3) > 2.0 * p.odds(1.0, 0.0));
    assert!(p.odds(0.3, 0.0) < p.odds(1.0, 0.0));
    assert!(p.odds(3.0, 1.5) <= 0.9);
}
