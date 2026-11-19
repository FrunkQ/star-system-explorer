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

/// The Solar System at J2000 (JPL approximate elements).
pub fn sol() -> NavSystem {
    serde_json::from_str(include_str!("../../../tests/fixtures/nav/sol.json")).expect("sol fixture")
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
    serde_json::json!({"version": crate::NAV_VERSION, "cases": out})
}
