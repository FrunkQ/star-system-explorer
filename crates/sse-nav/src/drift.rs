//! "Ship not under power": where a drifting ship's orbit takes it, what it
//! meets, and its ways out (burn for a body, make the orbit safe, use an
//! encounter, or call for rescue), each with its cost and time.

use crate::courses::{self, end_burn};
use crate::kepler::{self, conic, cross, norm, scale, sub, V3};
use crate::{Endpoint, NavSettings, NavSystem, Prefer, RouteRequest, Ship};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, C, DAY, JULIAN_YEAR};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Orbit {
    pub bound: bool,
    pub e: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a_au: Option<f64>,
    pub periapsis_au: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apoapsis_au: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_days: Option<f64>,
    pub inbound: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Encounter {
    pub body: String,
    pub name: String,
    pub at_s: f64,
    pub distance_km: f64,
    /// Distance in the body's Hill radii.
    pub hill: f64,
    /// Speed relative to the body, m/s.
    pub v_rel_m_s: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WayOut {
    /// burn-to, safe-orbit, encounter, rescue, drift.
    pub kind: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub dv_m_s: f64,
    /// Universe time it resolves (arrival, rescue, the encounter).
    pub at_s: f64,
    pub days: f64,
    pub feasible: bool,
    pub why: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DriftReport {
    pub t_s: f64,
    pub r_au: f64,
    pub speed_km_s: f64,
    pub orbit: Orbit,
    pub fate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fate_s: Option<f64>,
    pub encounters: Vec<Encounter>,
    /// Feasible first, then soonest.
    pub ways_out: Vec<WayOut>,
}

/// Time until the ship first comes in to distance `rr` on its conic, by
/// Kepler's equation (elliptic or hyperbolic); None if it never does.
pub fn time_to_radius(r: V3, v: V3, mu: f64, rr: f64) -> Option<f64> {
    let c = conic(r, v, mu);
    let r0 = norm(r);
    if c.periapsis_m > rr || r0 <= rr {
        return None;
    }
    let rv = kepler::dot(r, v);
    if c.alpha > 1e-30 && c.e > 0.0 {
        let a = 1.0 / c.alpha;
        let n = m::sqrt(mu / (a * a * a));
        let big_e0 = m::atan2(rv / m::sqrt(mu * a), 1.0 - r0 / a);
        let cr = ((1.0 - rr / a) / c.e).clamp(-1.0, 1.0);
        let er = m::acos(cr);
        let target = if big_e0 < 0.0 { -er } else { m::TAU - er };
        let mean = |x: f64| x - c.e * m::sin(x);
        Some((mean(target) - mean(big_e0)) / n)
    } else if c.alpha < -1e-30 && rv < 0.0 {
        let a = 1.0 / c.alpha;
        let n = m::sqrt(mu / (-a * -a * -a));
        let f0 = m::asinh(rv / m::sqrt(-mu * a) / c.e);
        let fr = -m::acosh(((1.0 - rr / a) / c.e).max(1.0));
        let mean = |x: f64| c.e * m::sinh(x) - x;
        Some((mean(fr) - mean(f0)) / n)
    } else {
        None
    }
}

/// First time after `t0` the ship's distance from the star drops below
/// `r_stop` (inward) or rises above it (outward), within `span`.
fn first_crossing(
    sys: &NavSystem,
    r: V3,
    v: V3,
    r_stop: f64,
    inward: bool,
    span: f64,
) -> Option<f64> {
    let steps = 4000u32;
    let mu = sys.mu();
    let hit = |tau: f64| {
        let d = norm(kepler::propagate(r, v, mu, tau).0);
        if inward {
            d < r_stop
        } else {
            d > r_stop
        }
    };
    let mut prev = 0.0;
    for k in 1..=steps {
        let tau = span * k as f64 / steps as f64;
        if hit(tau) {
            let (mut lo, mut hi) = (prev, tau);
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if hit(mid) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            return Some(hi);
        }
        prev = tau;
    }
    None
}

pub fn ways_out(
    s: &NavSettings,
    sys: &NavSystem,
    ship: &Ship,
    rescue: Option<&Ship>,
    r: V3,
    v: V3,
    t0: f64,
) -> Result<DriftReport, String> {
    let mu = sys.mu();
    let c = conic(r, v, mu);
    let rn = norm(r);
    let bound = c.alpha > 0.0 && c.e < 1.0 + 1e-12;
    let orbit = Orbit {
        bound,
        e: c.e,
        a_au: (c.alpha > 0.0).then(|| 1.0 / c.alpha / AU),
        periapsis_au: c.periapsis_m / AU,
        apoapsis_au: c.apoapsis_m.is_finite().then(|| c.apoapsis_m / AU),
        period_days: c.period_s.map(|p| p / DAY),
        inbound: c.inbound,
    };
    let horizon = s.drift.horizon_years * JULIAN_YEAR;
    let span = c.period_s.map(|p| p.min(horizon)).unwrap_or(horizon);
    let r_star = sys.star_radius_m();
    let outer = sys
        .bodies
        .iter()
        .map(|b| b.a_au * (1.0 + b.e) * AU)
        .fold(0.0_f64, f64::max);
    // Fate.
    let (fate, fate_s) = if c.periapsis_m < r_star {
        let t = time_to_radius(r, v, mu, r_star)
            .or_else(|| first_crossing(sys, r, v, r_star, true, span));
        match t {
            Some(t) => (
                format!("falls into the star in {:.1} days", t / DAY),
                Some(t0 + t),
            ),
            None => ("its orbit dips into the star".into(), None),
        }
    } else if !bound {
        let edge = (1.5 * outer).max(rn * 1.5);
        match first_crossing(sys, r, v, edge, false, horizon) {
            Some(t) => (
                format!(
                    "leaves the system: passes {:.1} au in {:.1} years",
                    edge / AU,
                    t / JULIAN_YEAR
                ),
                Some(t0 + t),
            ),
            None => ("leaves the system, slowly".into(), None),
        }
    } else {
        let flux = sys.luminosity_w() / (4.0 * m::PI * c.periapsis_m * c.periapsis_m);
        if flux > ship.flux_limit_w_m2 {
            (
                format!(
                    "stays in orbit, but periapsis at {:.3} au is too hot ({:.0} W/m²)",
                    c.periapsis_m / AU,
                    flux
                ),
                None,
            )
        } else {
            (
                format!(
                    "stays in orbit between {:.2} and {:.2} au",
                    c.periapsis_m / AU,
                    c.apoapsis_m / AU
                ),
                None,
            )
        }
    };
    // Encounters within three Hill radii, soonest first.
    let steps = s.drift.steps.max(16);
    let end = fate_s.map(|f| f - t0).unwrap_or(span).min(span);
    let mut enc = vec![];
    for b in &sys.bodies {
        let hill = b.hill_m(sys.star_mass_msun).max(b.radius_m);
        let d_at = |tau: f64| {
            let (rs, vs) = kepler::propagate(r, v, mu, tau);
            let (rb, vb) = sys.at(b, t0 + tau);
            (norm(sub(rs, rb)), norm(sub(vs, vb)))
        };
        let mut prev = (d_at(0.0).0, 0.0);
        let mut falling = false;
        for k in 1..=steps {
            let tau = end * k as f64 / steps as f64;
            let (d, _) = d_at(tau);
            if d > prev.0 && falling && prev.0 < 3.0 * hill {
                let (dd, vr) = d_at(prev.1);
                enc.push(Encounter {
                    body: b.address.clone(),
                    name: b.name.clone(),
                    at_s: t0 + prev.1,
                    distance_km: dd / 1000.0,
                    hill: dd / hill,
                    v_rel_m_s: vr,
                });
            }
            falling = d < prev.0;
            prev = (d, tau);
        }
    }
    enc.sort_by(|a, b| a.at_s.total_cmp(&b.at_s).then(a.body.cmp(&b.body)));
    enc.truncate(s.drift.encounters as usize);

    let usable = ship.usable_dv_m_s();
    let mut ways = vec![];
    // Keep drifting.
    ways.push(WayOut {
        kind: "drift".into(),
        label: "Keep drifting".into(),
        body: None,
        dv_m_s: 0.0,
        at_s: fate_s.unwrap_or(t0 + span),
        days: (fate_s.unwrap_or(t0 + span) - t0) / DAY,
        feasible: !fate.starts_with("falls") && !fate.contains("too hot"),
        why: vec![fate.clone()],
    });
    // Make the orbit safe: circular at the present distance.
    let h = cross(r, v);
    let dir = if norm(h) > 1e-9 * rn * norm(v).max(1.0) {
        let hn = scale(h, 1.0 / norm(h));
        let t = cross(hn, r);
        scale(t, 1.0 / norm(t))
    } else {
        let t = cross([0.0, 0.0, 1.0], r);
        scale(t, 1.0 / norm(t).max(1e-30))
    };
    let vc = scale(dir, m::sqrt(mu / rn));
    let dv_safe = norm(sub(vc, v));
    ways.push(WayOut {
        kind: "safe-orbit".into(),
        label: format!("Burn into a circular orbit at {:.2} au", rn / AU),
        body: None,
        dv_m_s: dv_safe,
        at_s: t0 + dv_safe / ship.accel_m_s2().max(1e-9),
        days: dv_safe / ship.accel_m_s2().max(1e-9) / DAY,
        feasible: dv_safe <= usable && ship.accel_g > 0.0,
        why: vec![format!(
            "{:.2} km/s turns the drift into a circular orbit, safe from the star; the ship has {:.2} km/s usable",
            dv_safe / 1000.0,
            usable / 1000.0
        )],
    });
    // Burn for each body: the cheapest course from the drifting state.
    let from = Endpoint::State { r_m: r, v_m_s: v };
    for b in &sys.bodies {
        let req = RouteRequest {
            ship: ship.clone(),
            from: from.clone(),
            to: Some(Endpoint::Body(b.address.clone())),
            depart_s: Some(t0),
            window_days: (end / DAY / 4.0).clamp(1.0, 120.0),
            prefer: Prefer::SaveFuel,
            rescue: None,
        };
        let (cands, _) = courses::candidates(s, sys, &req, t0, req.to.as_ref().expect("set"))?;
        let best = cands
            .iter()
            .filter(|x| fate_s.is_none_or(|f| x.depart_s < f))
            .min_by(|a, b| a.dv_m_s.total_cmp(&b.dv_m_s).then(a.id.cmp(&b.id)));
        if let Some(x) = best {
            let late = fate_s.is_some_and(|f| x.depart_s >= f);
            ways.push(WayOut {
                kind: "burn-to".into(),
                label: format!("Burn for {}", b.name),
                body: Some(b.address.clone()),
                dv_m_s: x.dv_m_s,
                at_s: x.arrive_s,
                days: (x.arrive_s - t0) / DAY,
                feasible: x.dv_m_s <= usable && ship.accel_g > 0.0 && !late,
                why: vec![
                    format!(
                        "{} ({}): {:.2} km/s, arriving in {:.0} days",
                        x.label,
                        x.difficulty.label,
                        x.dv_m_s / 1000.0,
                        (x.arrive_s - t0) / DAY
                    ),
                    format!("the ship has {:.2} km/s usable", usable / 1000.0),
                ],
            });
        }
    }
    // Use an encounter: capture at the body, or its pull to change course.
    for e in &enc {
        let b = sys.body(&e.body).expect("listed body");
        let dv = end_burn(s, Some(b), e.v_rel_m_s);
        ways.push(WayOut {
            kind: "encounter".into(),
            label: format!("Coast to the pass by {}", b.name),
            body: Some(b.address.clone()),
            dv_m_s: dv,
            at_s: e.at_s,
            days: (e.at_s - t0) / DAY,
            feasible: dv <= usable && e.hill < 1.0,
            why: vec![format!(
                "passes {:.0} km from {} ({:.2} Hill radii) at {:.2} km/s: capture there needs {:.2} km/s, or its pull can bend the drift for free",
                e.distance_km,
                b.name,
                e.hill,
                e.v_rel_m_s / 1000.0,
                dv / 1000.0
            )],
        });
    }
    // Rescue from each port.
    let template = s
        .ships
        .get(&s.drift.rescue_ship)
        .expect("checked by settings");
    let rescuer = rescue.unwrap_or(template);
    for b in sys.bodies.iter().filter(|b| b.port) {
        let (rb, _) = sys.at(b, t0);
        let delay = norm(sub(r, rb)) / C;
        let req = RouteRequest {
            ship: rescuer.clone(),
            from: Endpoint::Body(b.address.clone()),
            to: Some(from.clone()),
            depart_s: Some(t0 + delay),
            window_days: 30.0,
            prefer: Prefer::Fastest,
            rescue: None,
        };
        let (cands, _) =
            courses::candidates(s, sys, &req, t0 + delay, req.to.as_ref().expect("set"))?;
        let ok: Vec<_> = cands
            .iter()
            .filter(|x| x.dv_m_s <= rescuer.usable_dv_m_s())
            .collect();
        let best = ok
            .iter()
            .min_by(|a, b| a.arrive_s.total_cmp(&b.arrive_s).then(a.id.cmp(&b.id)));
        let (at, why, feasible) = match best {
            Some(x) => {
                let late = fate_s.is_some_and(|f| x.arrive_s > f);
                (
                    x.arrive_s,
                    vec![
                        format!(
                            "a distress call reaches {} in {:.1} minutes",
                            b.name,
                            delay / 60.0
                        ),
                        format!(
                            "{} flies out: {} ({:.1} days)",
                            if rescuer.name.is_empty() {
                                "a rescue ship"
                            } else {
                                &rescuer.name
                            },
                            x.label,
                            x.days
                        ),
                        if late {
                            "too late: the drift ends first".to_string()
                        } else {
                            "in time".to_string()
                        },
                    ],
                    !late,
                )
            }
            None => (
                f64::INFINITY,
                vec![format!(
                    "no course from {} is within the rescue ship's fuel",
                    b.name
                )],
                false,
            ),
        };
        ways.push(WayOut {
            kind: "rescue".into(),
            label: format!("Call for rescue from {}", b.name),
            body: Some(b.address.clone()),
            dv_m_s: 0.0,
            at_s: at,
            days: (at - t0) / DAY,
            feasible,
            why,
        });
    }
    ways.sort_by(|a, b| {
        b.feasible
            .cmp(&a.feasible)
            .then(a.at_s.total_cmp(&b.at_s))
            .then(a.label.cmp(&b.label))
    });
    Ok(DriftReport {
        t_s: t0,
        r_au: rn / AU,
        speed_km_s: norm(v) / 1000.0,
        orbit,
        fate,
        fate_s,
        encounters: enc,
        ways_out: ways,
    })
}
