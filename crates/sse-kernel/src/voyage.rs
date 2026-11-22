//! `plan_voyage` (V8 part 3): NAV's voyage finder over the campaign's star
//! cells. The systems come from the cells around the start (and the
//! destination), as `star_cell` draws them at the campaign's now; what the
//! crew knows of fuel and which systems it has surveyed come with the ask
//! (the autopilot program fills them from its seat's audience, J6 part 2).

use crate::err;
use crate::protocol::Response;
use sse_cosmos::stars::Stars;
use sse_nav::voyage::{self, Goal, Minimise, Options, VoyageSystem};
use std::collections::BTreeMap;

pub(crate) struct Ask<'a> {
    pub from: &'a str,
    pub to: Option<&'a str>,
    pub home: Option<&'a str>,
    pub ship: &'a serde_json::Value,
    pub minimise: Option<&'a str>,
    pub visited: &'a [String],
    pub known_fuel: &'a BTreeMap<String, Vec<String>>,
    pub sensed_fuel: &'a BTreeMap<String, f64>,
    pub surveyed: &'a [String],
    pub gambles: bool,
}

pub(crate) fn minimise(m: Option<&str>) -> Result<Minimise, Response> {
    Ok(match m.unwrap_or("home_time") {
        "home_time" => Minimise::HomeTime,
        "ship_time" => Minimise::ShipTime,
        "fuel" => Minimise::Fuel,
        "legs" => Minimise::Legs,
        other => {
            return Err(bad(format!(
                "minimise is home_time, ship_time, fuel or legs, not {other}"
            )))
        }
    })
}

/// Cells searched at most (each 10 pc, tens of systems).
const MAX_CELLS: usize = 125;

fn bad(e: impl std::fmt::Display) -> Response {
    err("bad_request", e.to_string())
}

/// The cell a star system address was drawn in (`.../cell:x_y_z/sys:...`).
fn cell_of(address: &str) -> Option<[i64; 3]> {
    let (head, _) = address.rsplit_once("/sys:")?;
    let (_, key) = head.rsplit_once("/cell:")?;
    let n: Vec<i64> = key.split('_').filter_map(|x| x.parse().ok()).collect();
    (n.len() == 3).then(|| [n[0], n[1], n[2]])
}

/// A short name for a drawn system: its slot and cell ("k3 in 988.0.0").
fn short(address: &str) -> String {
    match (address.rsplit_once("/sys:"), cell_of(address)) {
        (Some((_, sys)), Some(c)) => {
            let slot = sys.rsplit_once('k').map(|x| x.1).unwrap_or(sys);
            format!("k{slot} in {}.{}.{}", c[0], c[1], c[2])
        }
        _ => address.to_string(),
    }
}

pub(crate) fn plan_voyage(
    st: &Stars,
    t_myr: f64,
    pins: &[sse_cosmos::stars::StarSystem],
    a: Ask,
) -> Result<serde_json::Value, Response> {
    let s = sse_nav::scenario::settings();
    let ship: sse_nav::Ship = match a.ship {
        serde_json::Value::String(id) => s.ships.get(id).cloned().ok_or_else(|| {
            err(
                "unknown_ship",
                format!(
                    "no ship template {id}; templates: {}",
                    s.ships.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            )
        })?,
        other => serde_json::from_value(other.clone()).map_err(|e| bad(format!("ship: {e}")))?,
    };
    let minimise = minimise(a.minimise)?;
    let c0 = cell_of(a.from).ok_or_else(|| bad(format!("{}: not a star system", a.from)))?;
    let c1 = match a.to {
        Some(t) => cell_of(t).ok_or_else(|| bad(format!("{t}: not a star system")))?,
        None => c0,
    };
    let lo = [0, 1, 2].map(|i| c0[i].min(c1[i]) - 1);
    let hi = [0, 1, 2].map(|i| c0[i].max(c1[i]) + 1);
    let count: i64 = (0..3).map(|i| hi[i] - lo[i] + 1).product();
    if count as usize > MAX_CELLS {
        return Err(err(
            "voyage_too_far",
            format!(
                "the voyage spans {count} star cells, more than the {MAX_CELLS} one plan searches; plan it in parts"
            ),
        ));
    }
    let frame = st.frame_of(t_myr);
    // Positions at the asked time. Frames are 0.1 Myr (Y1), so a system
    // stays within one cell of the cell that drew it, and the box of cells
    // one wider than the voyage on every side holds every true neighbour.
    let mut systems: Vec<VoyageSystem> = vec![];
    for x in lo[0]..=hi[0] {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                for sys in st.cell([x, y, z], frame, pins).map_err(bad)? {
                    let id = sys.address.to_string();
                    let at = st.at(&sys, t_myr);
                    let (odds, from) = match a.sensed_fuel.get(&id) {
                        Some(&o) => (o.clamp(0.0, 1.0), "the ship's sensors"),
                        None => (
                            s.voyage.giant_prior.odds(sys.mass_msun, sys.feh),
                            "its star's mass and metallicity",
                        ),
                    };
                    systems.push(VoyageSystem {
                        fuel_odds: Some(odds),
                        odds_from: from.to_string(),
                        name: short(&id),
                        fuel: a.known_fuel.get(&id).cloned().unwrap_or_default(),
                        surveyed: a.surveyed.contains(&id) || a.known_fuel.contains_key(&id),
                        xyz_pc: at.xyz_pc,
                        star: (at.state.luminosity_lsun > 0.0 && at.state.teff_k > 0.0).then(
                            || {
                                let t = 5772.0 / at.state.teff_k;
                                sse_nav::voyage::StarWell {
                                    mass_msun: at.state.mass_msun,
                                    // R = √L (T☉/T)², Stefan-Boltzmann.
                                    radius_rsun: sse_core::math::sqrt(at.state.luminosity_lsun)
                                        * t
                                        * t,
                                    luminosity_lsun: at.state.luminosity_lsun,
                                }
                            },
                        ),
                        id,
                    });
                }
            }
        }
    }
    let find = |id: &str| systems.iter().position(|s| s.id == id);
    let home = match a.home {
        Some(h) => find(h)
            .map(|i| systems[i].xyz_pc)
            .ok_or_else(|| bad(format!("{h}: home is not in the cells searched")))?,
        None => match find(a.from) {
            Some(i) => systems[i].xyz_pc,
            None => {
                return Err(err(
                    "unknown_system",
                    format!("{}: no such system in its cell now", a.from),
                ))
            }
        },
    };
    let goal = match a.to {
        Some(t) => Goal::To(t.to_string()),
        None => Goal::NearestUnvisited {
            visited: a.visited.to_vec(),
            home,
        },
    };
    let o = Options {
        minimise,
        gambles: a.gambles,
        settings: s.voyage.clone(),
    };
    let result = voyage::plan(&ship, ship.jump.as_ref(), &systems, a.from, &goal, &o);
    let means = |ship: &sse_nav::Ship| {
        let mut m = vec![];
        if let Some(j) = &ship.jump {
            m.push(serde_json::json!({
                "means": "jump",
                "words": format!("{}, {} t of {} in a {} t tank, {} days a jump{}", j.rating_words(), j.fuel(), j.fuel, j.tank_t, j.days_per_jump,
                    j.limit_diameters.map(|n| format!(", from {n} star diameters out")).unwrap_or_default()),
                "rule": j.rule,
                "fictional": true,
            }));
        }
        if ship.sustained {
            m.push(serde_json::json!({
                "means": "flight",
                "words": format!("{} g sustained, exhaust {} km/s, {} t of {}", ship.accel_g.min(ship.g_limit), ship.exhaust_km_s, ship.propellant(),
                    if ship.fuels.is_empty() { "propellant".to_string() } else { ship.fuels.join(" and ") }),
                "fictional": false,
            }));
        }
        m
    };
    // The systems near the route, for the console's small map.
    let near: Vec<serde_json::Value> = {
        let pts: Vec<[f64; 3]> = match &result {
            Ok(v) => std::iter::once(v.from.as_str())
                .chain(v.legs.iter().map(|l| l.to.as_str()))
                .filter_map(find)
                .map(|i| systems[i].xyz_pc)
                .collect(),
            Err(_) => vec![home],
        };
        let r = 3.0;
        let mut rows: Vec<(f64, usize)> = systems
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let d = pts
                    .iter()
                    .map(|p| {
                        let (x, y, z) =
                            (s.xyz_pc[0] - p[0], s.xyz_pc[1] - p[1], s.xyz_pc[2] - p[2]);
                        sse_core::math::sqrt(x * x + y * y + z * z)
                    })
                    .fold(f64::INFINITY, f64::min);
                (d <= r).then_some((d, i))
            })
            .collect();
        rows.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        rows.truncate(400);
        rows.sort_by_key(|r| r.1);
        rows.iter()
            .map(|&(_, i)| {
                let s = &systems[i];
                serde_json::json!({
                    "address": s.id, "name": s.name, "xyz_pc": s.xyz_pc,
                    "fuel": s.fuel, "surveyed": s.surveyed,
                    "fuel_odds": s.fuel_odds, "odds_from": s.odds_from,
                    "visited": a.visited.contains(&s.id),
                })
            })
            .collect()
    };
    let base = serde_json::json!({
        "ship": ship.name,
        "means": means(&ship),
        "searched": systems.len(),
        "cells": count,
        "t_myr": t_myr,
        "systems": near,
        "fuels": sse_nav::refuel::needs(&ship),
        "refuel": ship.refuel,
        "model": [
            "systems: the campaign's star cells around the start and the destination at the campaign's now (Y7a)",
            "jump: the ship's jump drive, a pack rule (fictional): its rating, fuel per parsec and days per jump",
            "flight: constant proper acceleration with turnover, special relativity and the relativistic rocket equation; a coast at the best speed the propellant allows (Baez, The Relativistic Rocket)",
            "fuel: carried leg to leg; a stop refuels only where the crew knows of a source the ship can use (a scoop for a giant, an ice processor for ice, docking for a port) that gives every fuel it burns; exotic fuels only at ports that stock them; unsurveyed systems are dry unless gambles are allowed, the ship can skim, and the odds clear the pack's floor",
            "odds: the ship's sensors where it has sensed a system, else giant-planet odds from the star's mass and metallicity (Johnson et al. 2010, PASP 122, 905; doi:10.1086/655775)",
            "positions: the systems at the asked time; frames are 0.1 Myr, so each stays within one cell of the cell that drew it",
            "wells: a flight pays the Δv to climb out of the start star's well from its temperate orbit (√L au) and to settle into the destination's, (√2 − 1)√(GM/r) each, by the rocket equation; a jump limit (a pack rule) adds the flight out to it and in from it",
            "left out: the stars' own motion",
        ],
    });
    let mut out = base;
    match result {
        Ok(v) => {
            out["voyage"] = serde_json::to_value(v).expect("plain data");
            out["refusals"] = serde_json::json!([]);
        }
        Err(r) => {
            out["voyage"] = serde_json::Value::Null;
            out["refusals"] = serde_json::to_value(r).expect("plain data");
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::Kernel;

    fn ask(k: &mut Kernel, req: &str) -> serde_json::Value {
        serde_json::from_str(&k.handle_json(req)).unwrap()
    }

    #[test]
    fn a_jump_courier_finds_the_nearest_unvisited_star_and_a_route_to_another() {
        let mut k = Kernel::new();
        let r = k.handle_json(r#"{"op":"open_universe","id":"v8","seed":"v8"}"#);
        assert!(r.contains("universe_opened"), "{r}");
        let cell = ask(&mut k, r#"{"op":"star_cell"}"#);
        let systems = cell["cell"]["systems"].as_array().unwrap();
        assert!(systems.len() > 5, "{}", systems.len());
        let from = systems[0]["address"].as_str().unwrap().to_string();
        let req = serde_json::json!({
            "op": "plan_voyage", "from": from, "ship": "jump-courier",
            "visited": [from], "minimise": "legs"
        });
        let v = ask(&mut k, &req.to_string());
        let p = &v["voyage"]["voyage"];
        assert_eq!(v["type"], "voyage_planned", "{v}");
        assert!(p.is_object(), "{v}");
        assert_eq!(p["legs"].as_array().unwrap().len(), 1, "{p}");
        assert!(p["legs"][0]["distance_pc"].as_f64().unwrap() <= 2.0);
        // Repeatable.
        assert_eq!(v, ask(&mut k, &req.to_string()));
        // The antimatter probe flies, slower than light, and ages less aboard.
        let to = p["to"].as_str().unwrap();
        let req = serde_json::json!({
            "op": "plan_voyage", "from": from, "to": to, "ship": "relativistic-probe"
        });
        let v = ask(&mut k, &req.to_string());
        let leg = &v["voyage"]["voyage"]["legs"][0];
        assert_eq!(leg["means"], "flight", "{v}");
        assert!(leg["ship_days"].as_f64() < leg["home_days"].as_f64());
        // A chemical ship refuses, in words.
        let req = serde_json::json!({
            "op": "plan_voyage", "from": from, "to": to, "ship": "chemical-transfer"
        });
        let v = ask(&mut k, &req.to_string());
        assert!(v["voyage"]["voyage"].is_null());
        assert!(v["voyage"]["refusals"][0]
            .as_str()
            .unwrap()
            .contains("cannot cross between stars"));
    }

    #[test]
    fn in_a_system_the_same_finder_plans_transfers_with_windows() {
        let mut k = Kernel::new();
        let v = ask(
            &mut k,
            r#"{"op":"plan_voyage","fixture":"sol","from":"Earth","to":"Mars","ship":"nuclear-thermal","minimise":"fuel"}"#,
        );
        assert_eq!(v["type"], "voyage_planned", "{v}");
        assert_eq!(v["voyage"]["scale"], "system");
        let leg = &v["voyage"]["voyage"]["legs"][0];
        assert_eq!(leg["means"], "course", "{v}");
        assert!(leg["wait_days"].as_f64().is_some());
        let places = v["voyage"]["places"].as_array().unwrap();
        assert!(places.iter().any(|p| p["address"] == "belt:Main Belt"));
        // Nearest unvisited from Earth, by a torch.
        let v = ask(
            &mut k,
            r#"{"op":"plan_voyage","fixture":"sol","from":"Earth","ship":"fusion-torch"}"#,
        );
        assert!(v["voyage"]["voyage"]["to"].is_string(), "{v}");
    }
}
