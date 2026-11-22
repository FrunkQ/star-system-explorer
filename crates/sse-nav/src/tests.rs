use crate::courses::{hohmann_dv, hohmann_time};
use crate::kepler::{lambert, norm, propagate, state, sub, Elements};
use crate::scenario::{golden, settings, sol};
use crate::{plan, Endpoint, NavBody, NavSystem, Prefer, RouteRequest, Ship};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, DAY, GM_SUN, R_SUN};

fn req(ship: Ship, from: Endpoint, to: Option<Endpoint>, prefer: Prefer) -> RouteRequest {
    RouteRequest {
        ship,
        from,
        to,
        depart_s: None,
        window_days: 800.0,
        prefer,
        rescue: None,
    }
}

fn body(name: &str) -> Endpoint {
    Endpoint::Body(name.into())
}

#[test]
fn lambert_arcs_reach_their_targets() {
    let mu = GM_SUN;
    let cases = [
        ([AU, 0.0, 0.0], [0.0, 1.524 * AU, 0.0], 120.0),
        ([AU, 0.0, 0.0], [-1.2 * AU, 0.7 * AU, 0.05 * AU], 200.0),
        (
            [0.7 * AU, 0.2 * AU, 0.0],
            [-5.2 * AU, 1.0 * AU, 0.1 * AU],
            900.0,
        ),
        ([AU, 0.0, 0.0], [0.3 * AU, 0.3 * AU, 0.0], 30.0),
        // The long way round (more than half a turn).
        ([AU, 0.0, 0.0], [0.0, -1.524 * AU, 0.0], 400.0),
    ];
    for (r1, r2, days) in cases {
        let (v1, v2) = lambert(r1, r2, days * DAY, mu).expect("solvable");
        let (r, v) = propagate(r1, v1, mu, days * DAY);
        let miss = norm(sub(r, r2));
        assert!(miss < 1e-6 * norm(r2), "missed by {miss} m");
        assert!(norm(sub(v, v2)) < 1e-6 * norm(v2));
    }
    // Opposite points are degenerate for one revolution in one plane.
    assert!(lambert([AU, 0.0, 0.0], [-2.0 * AU, 0.0, 0.0], 200.0 * DAY, mu).is_none());
}

#[test]
fn propagation_agrees_with_kepler_elements() {
    let el = Elements {
        a_m: 1.524 * AU,
        e: 0.0934,
        i_deg: 1.85,
        node_deg: 49.6,
        varpi_deg: -23.9,
        mean_longitude_deg: -4.55,
    };
    let (r0, v0) = state(&el, GM_SUN, 0.0);
    for days in [1.0, 100.0, 300.0, 687.0, 5000.0] {
        let (ra, va) = state(&el, GM_SUN, days * DAY);
        let (rb, vb) = propagate(r0, v0, GM_SUN, days * DAY);
        assert!(norm(sub(ra, rb)) < 1e-7 * norm(ra), "{days} d");
        assert!(norm(sub(va, vb)) < 1e-7 * norm(va));
    }
    // A hyperbola too: energy is kept.
    let v = [0.0, 50_000.0, 0.0];
    let (r, vv) = propagate([AU, 0.0, 0.0], v, GM_SUN, 300.0 * DAY);
    let e0 = 0.5 * 50_000.0 * 50_000.0 - GM_SUN / AU;
    let e1 = 0.5 * norm(vv) * norm(vv) - GM_SUN / norm(r);
    assert!((e1 - e0).abs() < 1e-8 * e0.abs());
}

/// Circular coplanar Earth and Mars, massless (no escape or capture burns).
fn ideal() -> NavSystem {
    let b = |n: &str, a: f64, l: f64| NavBody {
        address: format!("u:t/sys:0/body:{n}"),
        name: n.into(),
        mass_earth: 0.0,
        radius_m: 6.0e6,
        a_au: a,
        e: 0.0,
        i_deg: 0.0,
        node_deg: 0.0,
        varpi_deg: 0.0,
        mean_longitude_deg: l,
        port: true,
        stock: vec![],
        magnetic_moment_a_m2: None,
    };
    NavSystem {
        name: "ideal".into(),
        star_mass_msun: 1.0,
        star_luminosity_lsun: 1.0,
        star_radius_rsun: 1.0,
        t0_s: 0.0,
        bodies: vec![b("earth", 1.0, 0.0), b("mars", 1.524, 0.0)],
        belts: vec![],
    }
}

#[test]
fn fixture_transfers_match_analytic_solutions() {
    let s = settings();
    let sys = ideal();
    let mu = sys.mu();
    let (r1, r2) = (AU, 1.524 * AU);
    let th = hohmann_time(mu, r1, r2);
    let (d1, d2) = hohmann_dv(mu, r1, r2);
    // Textbook values: 259 days, 2.94 and 2.65 km/s.
    assert!((th / DAY - 258.9).abs() < 0.5, "{}", th / DAY);
    assert!(
        (d1 - 2945.0).abs() < 10.0 && (d2 - 2649.0).abs() < 10.0,
        "{d1} {d2}"
    );
    let mut ship = s.ships["nuclear-thermal"].clone();
    ship.wet_t = 1000.0;
    let p = plan(
        &s,
        &sys,
        &req(ship, body("earth"), Some(body("mars")), Prefer::SaveFuel),
    )
    .unwrap();
    let h = p
        .candidates
        .iter()
        .find(|c| c.id == "hohmann")
        .expect("hohmann");
    assert!((h.dv_m_s - (d1 + d2)).abs() < 1.0);
    assert!((h.days - th / DAY).abs() < 1e-6);
    // The cheapest Lambert arc on the grid is the Hohmann transfer, within the grid's resolution.
    let l = p
        .candidates
        .iter()
        .find(|c| c.id == "lambert-fuel")
        .expect("lambert");
    assert!(
        (l.dv_m_s - (d1 + d2)).abs() < 0.03 * (d1 + d2),
        "{} against {}",
        l.dv_m_s,
        d1 + d2
    );
    assert!(
        (l.days - th / DAY).abs() < 0.1 * th / DAY,
        "{} days",
        l.days
    );
    // The Hohmann arc ends at Mars's orbit.
    let end = *h.track.last().unwrap();
    assert!((norm(end) - r2).abs() < 1e-6 * r2);
}

#[test]
fn torch_flights_match_constant_thrust() {
    let s = settings();
    // A star too light to matter: fixed points 1 au apart.
    let mut sys = ideal();
    sys.star_mass_msun = 1e-20;
    sys.bodies.clear();
    let a_au = [0.0, 0.0, 0.0];
    let from = Endpoint::State {
        r_m: [AU, 0.0, 0.0],
        v_m_s: a_au,
    };
    let to = Endpoint::State {
        r_m: [2.0 * AU, 0.0, 0.0],
        v_m_s: a_au,
    };
    let mut ship = s.ships["fusion-torch"].clone();
    ship.exhaust_km_s = 100_000.0;
    let p = plan(
        &s,
        &sys,
        &req(
            ship.clone(),
            from.clone(),
            Some(to.clone()),
            Prefer::Fastest,
        ),
    )
    .unwrap();
    let a = ship.accel_m_s2();
    let t = 2.0 * m::sqrt(AU / a);
    let c = &p.candidates[0];
    assert_eq!(c.id, "torch-100");
    assert!(
        (c.days * DAY - t).abs() < 1e-6 * t,
        "{} against {}",
        c.days * DAY,
        t
    );
    assert!((c.dv_m_s - a * t).abs() < 1e-6 * a * t);
    // With less fuel it coasts: t = d / v + v / a at v = half the usable Δv.
    ship.exhaust_km_s = 2000.0;
    let p = plan(
        &s,
        &sys,
        &req(ship.clone(), from, Some(to), Prefer::Fastest),
    )
    .unwrap();
    let vc = 0.5 * ship.usable_dv_m_s();
    let t = AU / vc + vc / a;
    let c = p.candidates.iter().find(|c| c.id == "torch-100").unwrap();
    assert!((c.days * DAY - t).abs() < 1e-6 * t);
    assert!(c.label.contains("cruise"));
    // Burn times add up to the trip.
    let total: f64 = c.burns.iter().map(|b| b.duration_s).sum();
    assert!((total - t).abs() < 1e-6 * t);
}

#[test]
fn solar_system_courses_carry_hazards_and_reasons() {
    let s = settings();
    let sys = sol();
    // Earth to Mars, chemical: coasting courses only, months long, with delays.
    let p = plan(
        &s,
        &sys,
        &req(
            s.ships["chemical-transfer"].clone(),
            body("Earth"),
            Some(body("Mars")),
            Prefer::SaveFuel,
        ),
    )
    .unwrap();
    assert!(!p.candidates.is_empty(), "{:?}", p.refused);
    assert!(p.candidates.iter().all(|c| c.kind != "torch"));
    let c = &p.candidates[0];
    assert!(c.days > 100.0 && c.days < 400.0, "{}", c.days);
    assert!(c.dv_m_s < p.usable_dv_m_s);
    assert!(c.dose_msv > 100.0, "months of cosmic rays: {}", c.dose_msv);
    assert!(c.difficulty.reasons.iter().any(|r| r.starts_with("dose")));
    assert!(c.delay_max_s > 60.0 && c.delay_max_s < 1500.0);
    assert!(c.reserve_left > 0.0 && c.reserve_left < 1.0);
    // Sorted by fuel.
    for w in p.candidates.windows(2) {
        assert!(w[0].dv_m_s <= w[1].dv_m_s);
    }
    // A torch to Jupiter: days, its belts, and the Main Belt on the way.
    let p = plan(
        &s,
        &sys,
        &req(
            s.ships["fusion-torch"].clone(),
            body("Earth"),
            Some(body("Jupiter")),
            Prefer::Fastest,
        ),
    )
    .unwrap();
    let c = &p.candidates[0];
    assert_eq!(c.kind, "torch");
    assert!(c.days < 15.0, "{}", c.days);
    assert!(
        c.hazards.iter().any(|h| h.kind == "belts"),
        "{:?}",
        c.hazards
    );
    assert!(c.hazards.iter().any(|h| h.kind == "debris"));
    assert!(c.difficulty.score > 0.0);
    // Toward the Sun: heat.
    let p = plan(
        &s,
        &sys,
        &req(
            s.ships["fusion-torch"].clone(),
            body("Earth"),
            Some(body("Mercury")),
            Prefer::Fastest,
        ),
    )
    .unwrap();
    assert!(p.candidates.iter().any(|c| c.peak_flux_w_m2 > 5000.0));
    // A ship with too little fuel is refused, with why.
    let mut weak = s.ships["chemical-transfer"].clone();
    weak.wet_t = 110.0;
    let p = plan(
        &s,
        &sys,
        &req(weak, body("Earth"), Some(body("Neptune")), Prefer::SaveFuel),
    )
    .unwrap();
    assert!(p.candidates.is_empty());
    assert!(p.refused.iter().any(|r| r.why.contains("needs")));
    // Unknown and same bodies.
    let e = plan(
        &s,
        &sys,
        &req(
            s.ships["fusion-torch"].clone(),
            body("Vulcan"),
            Some(body("Mars")),
            Prefer::Fastest,
        ),
    );
    assert!(e.unwrap_err().contains("Vulcan"));
    let e = plan(
        &s,
        &sys,
        &req(
            s.ships["fusion-torch"].clone(),
            body("Mars"),
            Some(body("Mars")),
            Prefer::Fastest,
        ),
    );
    assert!(e.is_err());
}

#[test]
fn a_ship_not_under_power_gets_ways_out() {
    let s = settings();
    let sys = sol();
    // At rest 1 au from the Sun: it falls in, in the free-fall time.
    let p = plan(
        &s,
        &sys,
        &req(
            s.ships["chemical-transfer"].clone(),
            Endpoint::State {
                r_m: [AU, 0.0, 0.0],
                v_m_s: [0.0, 0.0, 0.0],
            },
            None,
            Prefer::Balanced,
        ),
    )
    .unwrap();
    let d = p.drift.as_ref().unwrap();
    let x = R_SUN / AU;
    let tff =
        m::sqrt(AU * AU * AU / (2.0 * sys.mu())) * (m::acos(m::sqrt(x)) + m::sqrt(x * (1.0 - x)));
    let fall = d.fate_s.unwrap() - d.t_s;
    assert!(d.fate.starts_with("falls into the star"), "{}", d.fate);
    assert!(
        (fall - tff).abs() < 1e-3 * tff,
        "{} against {} days",
        fall / DAY,
        tff / DAY
    );
    assert!((tff / DAY - 64.5).abs() < 0.5);
    let safe = d.ways_out.iter().find(|w| w.kind == "safe-orbit").unwrap();
    assert!((safe.dv_m_s - m::sqrt(sys.mu() / AU)).abs() < 1.0);
    assert!(!safe.feasible, "chemical ship cannot find 30 km/s");
    let rescue: Vec<_> = d.ways_out.iter().filter(|w| w.kind == "rescue").collect();
    assert_eq!(rescue.len(), 2, "Earth and Mars have ports");
    assert!(rescue.iter().any(|w| w.feasible), "{rescue:?}");
    assert!(d.ways_out[0].feasible, "feasible first");
    assert!(d.ways_out.iter().any(|w| w.kind == "drift" && !w.feasible));
    // In Earth's orbit just behind Earth: stays in orbit, meets Earth.
    let (re, ve) = sys.at(sys.body("Earth").unwrap(), sys.t0_s);
    let back = 2.0e9;
    let ahead = crate::kepler::scale(ve, 1.0 / norm(ve));
    let r = sub(re, crate::kepler::scale(ahead, back));
    let v = crate::kepler::add(ve, crate::kepler::scale(ahead, 30.0));
    let p = plan(
        &s,
        &sys,
        &req(
            s.ships["chemical-transfer"].clone(),
            Endpoint::State { r_m: r, v_m_s: v },
            None,
            Prefer::Balanced,
        ),
    )
    .unwrap();
    let d = p.drift.unwrap();
    assert!(d.fate.starts_with("stays in orbit"), "{}", d.fate);
    assert!(d.orbit.bound);
    assert!(
        d.encounters.iter().any(|e| e.name == "Earth"),
        "{:?}",
        d.encounters
    );
    assert!(d.ways_out.iter().any(|w| w.kind == "burn-to" && w.feasible));
    // Asking for ways out from a body is refused.
    assert!(plan(
        &s,
        &sys,
        &req(
            s.ships["chemical-transfer"].clone(),
            body("Earth"),
            None,
            Prefer::Balanced
        )
    )
    .is_err());
}

#[test]
fn settings_refuse_unknown_reasons() {
    let mut s = settings();
    s.difficulty.points.insert("bad-luck".into(), 1.0);
    assert!(s.check().unwrap_err().contains("bad-luck"));
    let mut s = settings();
    s.drift.rescue_ship = "nothing".into();
    assert!(s.check().is_err());
}

#[test]
fn the_same_question_gives_the_same_plan() {
    let s = settings();
    let sys = sol();
    let r = req(
        s.ships["fusion-torch"].clone(),
        body("Mars"),
        Some(body("Saturn")),
        Prefer::Balanced,
    );
    assert_eq!(plan(&s, &sys, &r).unwrap(), plan(&s, &sys, &r).unwrap());
}

#[test]
fn golden_nav() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/nav.json");
    let got = serde_json::to_string_pretty(&golden()).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &got).unwrap();
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1 once");
    assert_eq!(got, want, "nav golden changed");
}

#[test]
fn radiation_belts_follow_the_magnetopause_not_a_mass_gate() {
    let s = settings();
    let sys = sol();
    let rs = &s.radiation;
    let reach = |n: &str| {
        let b = sys.bodies.iter().find(|b| b.name == n).unwrap();
        crate::hazards::belt_reach_radii(b, rs)
    };
    // Earth's magnetopause stands near 10 radii (Chapman and Ferraro).
    let earth = reach("Earth").expect("Earth has belts");
    assert!((8.0..13.0).contains(&earth), "Earth {earth}");
    assert!(reach("Saturn").is_some(), "Saturn has belts");
    assert!(reach("Jupiter").is_some());
    assert!(reach("Uranus").is_some() && reach("Neptune").is_some());
    // Mercury's field barely holds the wind off its surface; Mars and Venus have none.
    assert!(reach("Mercury").is_none(), "Mercury {:?}", reach("Mercury"));
    assert!(reach("Mars").is_none() && reach("Venus").is_none());
}

#[test]
fn belt_dose_scales_with_the_field() {
    let s = settings();
    let sys = sol();
    let st = |n: &str| {
        crate::hazards::belt_strength(
            sys.bodies.iter().find(|b| b.name == n).unwrap(),
            &s.radiation,
        )
    };
    assert!(
        (st("Jupiter") - 1.0).abs() < 0.1,
        "Jupiter {}",
        st("Jupiter")
    );
    assert!(st("Earth") < 0.1 && st("Saturn") < 0.1);
}

#[test]
fn leaving_earth_crosses_its_belts_briefly() {
    let s = settings();
    let sys = sol();
    let r = req(
        s.ships["chemical-transfer"].clone(),
        body("Earth"),
        Some(body("Mars")),
        Prefer::SaveFuel,
    );
    let p = plan(&s, &sys, &r).unwrap();
    let c = &p.candidates[0];
    let h = c
        .hazards
        .iter()
        .find(|h| h.kind == "belts" && h.text.contains("Earth"))
        .expect("Earth's belts listed");
    assert!(h.text.contains(" mSv"), "{}", h.text);
    assert!(
        !c.difficulty
            .reasons
            .iter()
            .any(|r| r.contains("radiation belts")),
        "{:?}",
        c.difficulty.reasons
    );
}
