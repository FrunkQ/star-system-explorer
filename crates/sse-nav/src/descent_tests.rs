use crate::descent::{descend, plan_down, EntryState, World};
use crate::scenario::settings;
use sse_core::math as m;

use crate::scenario::iso_air as iso;

fn world<'a>(
    name: &str,
    radius_km: f64,
    g: f64,
    air: &'a dyn Fn(f64) -> (f64, f64),
    k: f64,
) -> World<'a> {
    World {
        name: name.into(),
        radius_m: radius_km * 1000.0,
        gravity_m_s2: g,
        ground_m: 0.0,
        air,
        sutton_graves_k: k,
        surface_wind_m_s: 0.0,
    }
}

#[test]
fn earth_capsules_land_on_parachutes() {
    let s = settings();
    let d = &s.descent;
    let air = iso(1.225, 250.0, 0.029, 1.4, 9.81);
    let w = world("Earth", 6371.0, 9.81, &air, d.sutton_graves["N2-O2"]);
    let e = EntryState {
        altitude_m: 120_000.0,
        speed_m_s: 7600.0,
        flight_path_deg: -1.5,
    };
    let out = descend(&d.run, &w, &d.landers["capsule"], &e);
    let o = &out.outcome;
    assert_eq!(o.result, "landed", "{o:?}");
    assert_eq!(o.method, "chute");
    assert!(!o.needs_engines);
    assert!(
        o.touchdown_m_s < 10.0 && o.touchdown_m_s > 4.0,
        "{}",
        o.touchdown_m_s
    );
    assert!(o.peak_g > 2.0 && o.peak_g < 10.0, "{}", o.peak_g);
    assert!(
        o.peak_heat_w_cm2 > 20.0 && o.peak_heat_w_cm2 < 400.0,
        "{}",
        o.peak_heat_w_cm2
    );
    assert!(out.events.iter().any(|e| e.kind == "chute"));
    assert!(out.events.windows(2).all(|p| p[0].t_s <= p[1].t_s));
}

#[test]
fn thin_air_on_mars_needs_engines() {
    let s = settings();
    let d = &s.descent;
    let air = iso(0.020, 210.0, 0.044, 1.29, 3.71);
    let w = world("Mars", 3389.5, 3.71, &air, d.sutton_graves["CO2"]);
    // From a low orbit, lift up (a direct hyperbolic entry needs bank
    // guidance, which is not modelled).
    let e = EntryState {
        altitude_m: 150_000.0,
        speed_m_s: 3460.0,
        flight_path_deg: -2.0,
    };
    let mut chute_only = d.landers["mars-lander"].clone();
    chute_only.engine = None;
    let out = descend(&d.run, &w, &chute_only, &e);
    assert!(out.outcome.needs_engines);
    assert!(
        out.outcome.chute_terminal_m_s.unwrap() > 40.0,
        "{:?}",
        out.outcome
    );
    assert_eq!(out.outcome.result, "crash");
    let out = descend(&d.run, &w, &d.landers["mars-lander"], &e);
    assert_eq!(out.outcome.method, "chute-and-engines", "{:?}", out.outcome);
    assert!(
        matches!(out.outcome.result.as_str(), "landed" | "hard-landing"),
        "{:?}",
        out.outcome
    );
    assert!(out.outcome.propellant_used_kg > 0.0);
}

#[test]
fn titan_parachute_descent_is_long() {
    let s = settings();
    let d = &s.descent;
    let air = iso(5.4, 94.0, 0.028, 1.4, 1.352);
    let w = world("Titan", 2574.7, 1.352, &air, d.sutton_graves["N2-CH4"]);
    let e = EntryState {
        altitude_m: 400_000.0,
        speed_m_s: 6000.0,
        flight_path_deg: -65.0,
    };
    let out = descend(&d.run, &w, &d.landers["titan-probe"], &e);
    let o = &out.outcome;
    assert_eq!(o.result, "landed", "{o:?}");
    assert!(!o.needs_engines);
    assert!(o.duration_s > 1.5 * 3600.0, "{} h", o.duration_s / 3600.0);
    assert!(o.touchdown_m_s < 7.0, "{}", o.touchdown_m_s);
}

#[test]
fn airless_worlds_land_on_engines() {
    let s = settings();
    let d = &s.descent;
    let air = |_h: f64| (0.0, 1.0);
    let w = world("Moon", 1737.4, 1.62, &air, d.sutton_graves_default);
    let v_orb = m::sqrt(4.9048e12 / (1737.4e3 + 15_000.0));
    let e = EntryState {
        altitude_m: 15_000.0,
        speed_m_s: v_orb,
        flight_path_deg: 0.0,
    };
    let out = descend(&d.run, &w, &d.landers["moon-lander"], &e);
    let o = &out.outcome;
    assert_eq!(o.result, "landed", "{o:?}");
    assert_eq!(o.method, "engines");
    assert!(o.needs_engines);
    assert!(o.why.iter().any(|w| w.contains("no air")));
    // Δv at least the orbital speed (gravity losses on top).
    let lm = &d.landers["moon-lander"];
    let e = lm.engine.as_ref().unwrap();
    let dv = e.isp_s * crate::G0 * m::ln(lm.mass_kg / (lm.mass_kg - o.propellant_used_kg));
    assert!(dv > v_orb && dv < 1.5 * v_orb, "{dv} against {v_orb}");
}

/// LR1 (landing run-through): a ship with a torch drive and a bare hull (a
/// Traveller far trader: 1 g, 3000 km/s exhaust, a hull good for 1 W/cm²)
/// burns up if it enters Mars's air at orbital speed, so plan_down flies it
/// down on its engines from the top of the air instead, as ship_at_world's
/// powered landing reckons. A capsule that enters well keeps its entry, and a
/// lander too weak to hover keeps its honest verdict.
#[test]
fn a_torch_ship_flies_down_from_orbit() {
    let s = settings();
    let d = &s.descent;
    let air = iso(0.020, 210.0, 0.044, 1.29, 3.71);
    let w = world("Mars", 3389.5, 3.71, &air, d.sutton_graves["CO2"]);
    let e = EntryState {
        altitude_m: 150_000.0,
        speed_m_s: 3460.0,
        flight_path_deg: -2.0,
    };
    let mass = 840_000.0;
    let mut ship = d.landers["mars-lander"].clone();
    ship.name = "Far trader".into();
    ship.mass_kg = mass;
    ship.chute = None;
    ship.heat_limit_w_cm2 = 1.0;
    ship.touchdown_m_s = 1.0;
    ship.jettison_kg = 0.0;
    ship.lift_to_drag = 0.0;
    ship.engine = Some(crate::descent::Engine {
        thrust_n: 1.0 * crate::G0 * mass,
        isp_s: 3.0e6 / crate::G0,
        propellant_kg: 40_000.0,
    });
    let entry = descend(&d.run, &w, &ship, &e);
    assert_eq!(entry.outcome.result, "burned-up", "{:?}", entry.outcome);
    let out = plan_down(&d.run, &w, &ship, &e);
    let o = &out.outcome;
    assert_eq!(o.result, "landed", "{o:?}");
    assert_eq!(o.method, "engines-from-orbit");
    assert!(o.peak_heat_w_cm2 <= 1.0, "{}", o.peak_heat_w_cm2);
    assert!(o.why[0].contains("burned up"), "{:?}", o.why);
    // A lander that enters well keeps the entry.
    let ml = plan_down(&d.run, &w, &d.landers["mars-lander"], &e);
    assert_eq!(ml, descend(&d.run, &w, &d.landers["mars-lander"], &e));
    // Under its weight the drive cannot hold it up: the entry stands.
    let mut weak = ship.clone();
    weak.engine.as_mut().unwrap().thrust_n = 0.3 * crate::G0 * mass;
    assert_eq!(plan_down(&d.run, &w, &weak, &e).outcome.result, "burned-up");
}
