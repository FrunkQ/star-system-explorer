//! `plan_route`: NAV (T3) through the kernel. The system comes from a NAV
//! fixture (`sol`), an inline `nav` system, or an adopted system in the open
//! universe, read at `at_s` like `resolved_system` (planets from its
//! SystemSpec, radii from the surface bodies, the star from its facts).

use crate::adopt::worlds_at;
use crate::err;
use crate::protocol::Response;
use sse_chronicle::Chronicle;
use sse_core::cbor::Value;
use sse_core::quantity::consts::{JULIAN_YEAR, L_SUN, R_EARTH, R_SUN, STEFAN_BOLTZMANN};
use sse_core::{math as m, Address};
use sse_formation::pipeline::SystemRun;
use sse_nav::{Belt, NavBody, NavSettings, NavSystem, RouteRequest, Ship};

pub(crate) enum Source<'a> {
    Fixture(&'a str),
    Inline(&'a serde_json::Value),
    System {
        ch: &'a Chronicle,
        system: &'a Address,
        at_s: Option<i64>,
        run: Option<&'a SystemRun>,
        ports: &'a [String],
    },
}

fn bad(e: impl std::fmt::Display) -> Response {
    err("bad_request", e.to_string())
}

/// A ship given as a template id or in full.
fn ship(s: &NavSettings, v: &serde_json::Value) -> Result<Ship, Response> {
    match v {
        serde_json::Value::String(id) => s.ships.get(id).cloned().ok_or_else(|| {
            err(
                "unknown_ship",
                format!(
                    "no ship template {id}; templates: {}",
                    s.ships.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            )
        }),
        other => serde_json::from_value(other.clone()).map_err(|e| bad(format!("ship: {e}"))),
    }
}

fn num(v: Option<&Value>) -> Option<f64> {
    match v? {
        Value::Float(x) => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// An adopted system as NAV reads it.
fn from_system(
    ch: &Chronicle,
    system: &Address,
    at_s: Option<i64>,
    run: Option<&SystemRun>,
    ports: &[String],
) -> Result<(NavSystem, Option<f64>), Response> {
    let ss = sse_surface::scenario::settings();
    let w = worlds_at(ch, system, at_s, run, &ss)?;
    let spec = w.spec.ok_or_else(|| {
        err(
            "no_orbits",
            format!("{system} has no star mass and age, so no orbits to fly"),
        )
    })?;
    let star = w
        .facts
        .get(&spec.star_address.parse::<Address>().map_err(bad)?);
    let lum_w = star.and_then(|r| num(r.get("star.luminosity")));
    let teff = star.and_then(|r| num(r.get("star.effective_temperature")));
    let lum = lum_w
        .map(|l| l / L_SUN)
        .unwrap_or(m::pow(spec.star_mass, 4.0));
    let radius = match (lum_w, teff) {
        (Some(l), Some(t)) if t > 0.0 => {
            m::sqrt(l / (4.0 * m::PI * STEFAN_BOLTZMANN * t * t * t * t)) / R_SUN
        }
        _ => m::pow(spec.star_mass, 0.8),
    };
    let bodies = spec
        .planets
        .iter()
        .map(|p| {
            let rad = w
                .bodies
                .iter()
                .find(|b| b.address.to_string() == p.address)
                .map(|b| b.radius_earth * R_EARTH)
                // Mass-radius (Chen and Kipping 2017, rocky branch) when there is no surface body.
                .unwrap_or(R_EARTH * m::pow(p.mass_me.max(1e-6), 0.28));
            NavBody {
                address: p.address.clone(),
                name: p.name.clone(),
                mass_earth: p.mass_me,
                radius_m: rad,
                a_au: p.a_au,
                e: p.e,
                i_deg: p.i_deg,
                node_deg: p.node_deg,
                varpi_deg: p.varpi_deg,
                mean_longitude_deg: p.mean_longitude_deg,
                port: ports.contains(&p.address),
            }
        })
        .collect();
    let now_s = (spec.birth_year as f64 + spec.age_myr * 1.0e6) * JULIAN_YEAR;
    let t_read = (w.at_s != i64::MAX).then_some(w.at_s as f64);
    Ok((
        NavSystem {
            name: spec.name.clone(),
            star_mass_msun: spec.star_mass,
            star_luminosity_lsun: lum,
            star_radius_rsun: radius,
            t0_s: now_s,
            bodies,
            belts: spec
                .debris
                .iter()
                .map(|d| Belt {
                    name: d.name.clone(),
                    a_in_au: d.a_in_au,
                    a_out_au: d.a_out_au,
                })
                .collect(),
        },
        t_read,
    ))
}

pub(crate) struct Ask<'a> {
    pub source: Source<'a>,
    pub ship: &'a serde_json::Value,
    pub from: &'a serde_json::Value,
    pub to: Option<&'a serde_json::Value>,
    pub depart_s: Option<f64>,
    pub window_days: Option<f64>,
    pub prefer: Option<&'a str>,
    pub rescue: Option<&'a serde_json::Value>,
}

/// The NAV system a source names, and the time it was read at (a system's).
pub(crate) fn nav_system(source: Source) -> Result<(NavSystem, Option<f64>), Response> {
    Ok(match source {
        Source::Fixture(name) => (
            sse_nav::scenario::fixture(name).ok_or_else(|| {
                err(
                    "unknown_fixture",
                    format!("no NAV fixture {name}; fixtures: sol"),
                )
            })?,
            None,
        ),
        Source::Inline(v) => (
            serde_json::from_value(v.clone()).map_err(|e| bad(format!("nav: {e}")))?,
            None,
        ),
        Source::System {
            ch,
            system,
            at_s,
            run,
            ports,
        } => from_system(ch, system, at_s, run, ports)?,
    })
}

pub(crate) fn plan_route(a: Ask) -> Result<serde_json::Value, Response> {
    let s = sse_nav::scenario::settings();
    let (sys, t_read) = nav_system(a.source)?;
    let mut req = serde_json::json!({
        "ship": ship(&s, a.ship)?,
        "from": a.from,
        "window_days": a.window_days.unwrap_or(365.0),
    });
    if let Some(t) = a.to {
        req["to"] = t.clone();
    }
    if let Some(d) = a.depart_s.or(t_read) {
        req["depart_s"] = d.into();
    }
    if let Some(p) = a.prefer {
        req["prefer"] = p.into();
    }
    if let Some(r) = a.rescue {
        req["rescue"] = serde_json::to_value(ship(&s, r)?).expect("plain data");
    }
    let req: RouteRequest = serde_json::from_value(req).map_err(|e| bad(format!("route: {e}")))?;
    let plan = sse_nav::plan(&s, &sys, &req).map_err(|e| err("route_refused", e))?;
    Ok(serde_json::to_value(plan).expect("plain data"))
}

/// `window_view`: what a sensor sees from beside a body or a point, in a fixture or inline system.
pub(crate) fn window_view(
    fixture: Option<&str>,
    nav: Option<&serde_json::Value>,
    at_s: Option<f64>,
    from: Option<&str>,
    at_au: Option<[f64; 3]>,
    sensor: Option<&str>,
) -> Result<serde_json::Value, Response> {
    let s = sse_nav::scenario::settings();
    let sys: NavSystem = match (fixture, nav) {
        (Some(name), None) => sse_nav::scenario::fixture(name).ok_or_else(|| {
            err(
                "unknown_fixture",
                format!("no NAV fixture {name}; fixtures: sol"),
            )
        })?,
        (None, Some(v)) => {
            serde_json::from_value(v.clone()).map_err(|e| bad(format!("nav: {e}")))?
        }
        _ => return Err(bad("give a fixture or a nav system")),
    };
    let vp = match (from, at_au) {
        (Some(b), None) => sse_nav::sky::Viewpoint::Body(b.into()),
        (None, Some(p)) => sse_nav::sky::Viewpoint::Point([
            p[0] * sse_core::quantity::consts::AU,
            p[1] * sse_core::quantity::consts::AU,
            p[2] * sse_core::quantity::consts::AU,
        ]),
        _ => return Err(bad("give a body to look from or a point at_au, not both")),
    };
    let v = sse_nav::sky::window(
        &s.window,
        &sys,
        &vp,
        at_s.unwrap_or(sys.t0_s),
        sensor.unwrap_or("eye"),
    )
    .map_err(|e| err("window_refused", e))?;
    Ok(serde_json::to_value(v).expect("plain data"))
}

/// `plan_voyage` inside one system (V8 part 3b): the voyage finder over the
/// system's bodies and belts, legs priced by NAV's courses.
pub(crate) struct VoyageAsk<'a> {
    pub ship: &'a serde_json::Value,
    pub from: &'a str,
    pub to: Option<&'a str>,
    pub depart_s: Option<f64>,
    pub minimise: sse_nav::voyage::Minimise,
    pub visited: &'a [String],
}

pub(crate) fn plan_voyage(source: Source, a: VoyageAsk) -> Result<serde_json::Value, Response> {
    use sse_nav::voyage::{Goal, Options};
    let s = sse_nav::scenario::settings();
    let (sys, t_read) = nav_system(source)?;
    let ship = ship(&s, a.ship)?;
    let t0 = a.depart_s.or(t_read).unwrap_or(sys.t0_s);
    let places = sse_nav::insystem::places(&sys, &s.voyage.in_system);
    let home = sys
        .body(a.from)
        .map(|b| [b.a_au / sse_nav::insystem::AU_PER_PC, 0.0, 0.0])
        .unwrap_or([0.0; 3]);
    let goal = match a.to {
        Some(t) => Goal::To(t.to_string()),
        None => Goal::NearestUnvisited {
            visited: a
                .visited
                .iter()
                .cloned()
                .chain([a.from.to_string()])
                .collect(),
            home,
        },
    };
    let o = Options {
        minimise: a.minimise,
        gambles: false,
        settings: s.voyage.clone(),
    };
    let result = sse_nav::insystem::plan_in_system(&s, &sys, &ship, a.from, &goal, t0, &o);
    let rows: Vec<serde_json::Value> = places
        .iter()
        .map(|p| {
            serde_json::json!({
                "address": p.id, "name": p.name,
                "a_au": p.xyz_pc[0] * sse_nav::insystem::AU_PER_PC,
                "fuel": p.fuel,
                "belt": p.id.starts_with("belt:"),
                "visited": a.visited.contains(&p.id) || a.visited.contains(&p.name),
            })
        })
        .collect();
    let (voyage, refusals) = match result {
        Ok((v, _)) => (serde_json::to_value(v).expect("plain data"), vec![]),
        Err(r) => (serde_json::Value::Null, r),
    };
    Ok(serde_json::json!({
        "scale": "system",
        "system": sys.name,
        "ship": ship.name,
        "depart_s": t0,
        "places": rows,
        "voyage": voyage,
        "refusals": refusals,
        "model": [
            "places: the system's bodies and belts; fuel by the pack's rules (a giant to skim, ice beyond the snow line at 2.7 au times the square root of the star's luminosity, Hayashi 1981, a port)",
            "body to body: NAV's courses (Hohmann and Lambert transfers with their launch windows, torch courses), the quickest and the cheapest of each kept",
            "belts: Hohmann transfers or torch flights to the belt's middle radius, met anywhere on the ring with no window",
            "the same search as between the stars: fuel carried, refuel where fuel is known, the chosen measure minimised",
        ],
    }))
}
