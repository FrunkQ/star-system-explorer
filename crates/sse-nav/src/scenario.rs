//! T3 scenarios: the base-nav pack, the Sol fixture and the golden.

use crate::{plan, Endpoint, NavSettings, NavSystem, Prefer, RouteRequest};
use sse_pack::PackManifest;

pub fn base_nav_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-nav/pack.json"))
        .expect("base-nav pack")
}

pub fn settings() -> NavSettings {
    NavSettings::from_packs(&[base_nav_pack()]).expect("nav settings")
}

/// The Solar System at J2000 (JPL approximate elements), with each body's
/// magnetic moment from the measured-fields table (MF0), so the two cannot drift.
pub fn sol() -> NavSystem {
    let mut sys: NavSystem =
        serde_json::from_str(include_str!("../../../tests/fixtures/nav/sol.json"))
            .expect("sol fixture");
    let fields: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/magnetic/measured-fields.json"
    ))
    .expect("measured fields");
    let rows = fields["bodies"].as_array().cloned().unwrap_or_default();
    for b in &mut sys.bodies {
        b.magnetic_moment_a_m2 = rows
            .iter()
            .find(|r| {
                r["name"]
                    .as_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case(&b.name))
            })
            .and_then(|r| r["moment_a_m2"].as_f64());
    }
    sys
}

/// NAV fixtures by name (for the kernel's `fixture` field).
pub fn fixture(name: &str) -> Option<NavSystem> {
    name.eq_ignore_ascii_case("sol").then(sol)
}

fn digest<T: serde::Serialize>(t: &T) -> String {
    let text = serde_json::to_string(t).expect("serialisable");
    sse_core::ContentHash::of(&text).expect("hash").hex()
}

/// Golden: Earth to Mars with a chemical ship and a torch, and a ship
/// drifting at 1 au with no speed: candidate summaries and plan digests.
pub fn golden() -> serde_json::Value {
    let s = settings();
    let sys = sol();
    let ship = |id: &str| s.ships[id].clone();
    let earth = || Endpoint::Body("Earth".into());
    let mars = || Endpoint::Body("Mars".into());
    let reqs = [
        (
            "chemical",
            ship("chemical-transfer"),
            earth(),
            Some(mars()),
            Prefer::SaveFuel,
        ),
        (
            "torch",
            ship("fusion-torch"),
            earth(),
            Some(mars()),
            Prefer::Fastest,
        ),
        (
            "drift",
            ship("chemical-transfer"),
            Endpoint::State {
                r_m: [sse_core::quantity::consts::AU, 0.0, 0.0],
                v_m_s: [0.0, 0.0, 0.0],
            },
            None,
            Prefer::Balanced,
        ),
    ];
    let out: Vec<_> = reqs
        .into_iter()
        .map(|(name, ship, from, to, prefer)| {
            let req = RouteRequest {
                ship,
                from,
                to,
                depart_s: None,
                window_days: 800.0,
                prefer,
                rescue: None,
            };
            let p = plan(&s, &sys, &req).expect("plans");
            let cands: Vec<_> = p
                .candidates
                .iter()
                .map(|c| {
                    serde_json::json!({"id": c.id, "days": format!("{:.4}", c.days),
                        "dv_m_s": format!("{:.3}", c.dv_m_s), "difficulty": c.difficulty.label})
                })
                .collect();
            let ways: Vec<_> = p
                .drift
                .iter()
                .flat_map(|d| d.ways_out.iter())
                .map(|w| {
                    serde_json::json!({"kind": w.kind, "label": w.label, "feasible": w.feasible,
                    "dv_m_s": format!("{:.3}", w.dv_m_s)})
                })
                .collect();
            serde_json::json!({"case": name, "candidates": cands, "ways_out": ways,
                "refused": p.refused.len(), "plan": digest(&p)})
        })
        .collect();
    serde_json::json!({"version": crate::NAV_VERSION, "cases": out, "descents": descent_golden()})
}

/// An isothermal atmosphere ρ = ρ₀ e^(−h/H) with sound speed √(γ R T / M),
/// for the descent fixtures (no survey needed).
pub fn iso_air(rho0: f64, t_k: f64, molar: f64, gamma: f64, g: f64) -> impl Fn(f64) -> (f64, f64) {
    let r = 8.314_462_618 / molar;
    let h_scale = r * t_k / g;
    let a = sse_core::math::sqrt(gamma * r * t_k);
    move |h: f64| (rho0 * sse_core::math::exp(-h.max(0.0) / h_scale), a)
}

/// World, radius km, g, air (ρ₀, T, M, γ), gas, lander, entry (h, v, angle).
type Case<'a> = (
    &'a str,
    f64,
    f64,
    (f64, f64, f64, f64),
    &'a str,
    &'a str,
    (f64, f64, f64),
);

/// S5's descent fixtures: Earth, Mars, Titan and the Moon.
pub fn descent_golden() -> serde_json::Value {
    use crate::descent::{descend, EntryState, World};
    let s = settings();
    let d = &s.descent;
    let cases: [Case; 4] = [
        (
            "Earth",
            6371.0,
            9.81,
            (1.225, 250.0, 0.029, 1.4),
            "N2-O2",
            "capsule",
            (120_000.0, 7600.0, -1.5),
        ),
        (
            "Mars",
            3389.5,
            3.71,
            (0.020, 210.0, 0.044, 1.29),
            "CO2",
            "mars-lander",
            (150_000.0, 3460.0, -2.0),
        ),
        (
            "Titan",
            2574.7,
            1.352,
            (5.4, 94.0, 0.028, 1.4),
            "N2-CH4",
            "titan-probe",
            (400_000.0, 6000.0, -65.0),
        ),
        (
            "Moon",
            1737.4,
            1.62,
            (0.0, 1.0, 0.029, 1.4),
            "",
            "moon-lander",
            (15_000.0, 1673.0, 0.0),
        ),
    ];
    let out: Vec<_> = cases
        .iter()
        .map(|(name, r_km, g, (rho, t, mm, ga), gas, lander, (h, v, fpa))| {
            let air = iso_air(*rho, *t, *mm, *ga, *g);
            let w = World {
                name: (*name).into(),
                radius_m: r_km * 1000.0,
                gravity_m_s2: *g,
                ground_m: 0.0,
                air: &air,
                sutton_graves_k: d.sutton_graves.get(*gas).copied().unwrap_or(d.sutton_graves_default),
                surface_wind_m_s: 0.0,
            };
            let e = EntryState { altitude_m: *h, speed_m_s: *v, flight_path_deg: *fpa };
            let o = descend(&d.run, &w, &d.landers[*lander], &e);
            let x = &o.outcome;
            serde_json::json!({"world": name, "lander": lander, "result": x.result, "method": x.method,
                "touchdown_m_s": format!("{:.4}", x.touchdown_m_s), "duration_s": format!("{:.2}", x.duration_s),
                "peak_g": format!("{:.4}", x.peak_g), "peak_heat_w_cm2": format!("{:.4}", x.peak_heat_w_cm2),
                "propellant_kg": format!("{:.3}", x.propellant_used_kg),
                "events": o.events.iter().map(|e| e.kind.clone()).collect::<Vec<_>>()})
        })
        .collect();
    serde_json::Value::Array(out)
}
