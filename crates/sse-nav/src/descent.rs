//! S5: the descent scene's physics. A vehicle comes down from an entry
//! interface (or from orbit on an airless world): ballistic entry with
//! stagnation-point heating, a parachute where the air allows, and a powered
//! final descent where it must. The answer says whether parachutes work in
//! this air or engines must do it all, and gives a timed sequence of events.
//!
//! Planar equations of motion over a spherical world (no lift), integrated by
//! RK4 at a fixed step (deterministic). Heating: Sutton and Graves 1971,
//! q = k √(ρ / r_n) v³ (NASA TR R-376, https://ntrs.nasa.gov/citations/19720003329).
//! Drag: D = ½ ρ v² C_d A. Terminal speed under a canopy √(2 m g / (ρ C_d A)).

use serde::{Deserialize, Serialize};
use sse_core::math as m;

use crate::G0;

/// Guidance time constant for the powered descent, s.
const GUIDE_TAU_S: f64 = 2.0;
/// Share of the spare thrust (thrust less weight) the braking profile plans
/// to use, keeping the rest for steering and errors.
const BRAKE_SHARE: f64 = 0.8;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Parachute {
    pub area_m2: f64,
    pub cd: f64,
    /// Deploy only below this Mach number and dynamic pressure.
    pub max_mach: f64,
    pub max_q_pa: f64,
    /// Fill time: the canopy's drag area grows as (t / fill)² until it is full
    /// (Knacke 1992, Parachute Recovery Systems Design Manual, ch. 5).
    #[serde(default = "fill_s")]
    pub fill_s: f64,
}

fn fill_s() -> f64 {
    4.0
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Engine {
    pub thrust_n: f64,
    pub isp_s: f64,
    pub propellant_kg: f64,
}

/// A lander (stock templates in the pack).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Lander {
    #[serde(default)]
    pub name: String,
    pub mass_kg: f64,
    pub area_m2: f64,
    pub cd: f64,
    pub nose_radius_m: f64,
    #[serde(default)]
    pub chute: Option<Parachute>,
    #[serde(default)]
    pub engine: Option<Engine>,
    /// Heat shield limit, W/cm² at the stagnation point.
    #[serde(default = "heat")]
    pub heat_limit_w_cm2: f64,
    #[serde(default = "g_limit")]
    pub g_limit: f64,
    /// Fastest safe touchdown, m/s.
    #[serde(default = "touch")]
    pub touchdown_m_s: f64,
    /// Lift-to-drag ratio of the entry body, flown at a fixed bank angle
    /// (MSL flew about 0.24, Apollo about 0.3; a sphere 0).
    #[serde(default)]
    pub lift_to_drag: f64,
    /// Bank angle, degrees (0 is full lift up).
    #[serde(default)]
    pub bank_deg: f64,
    /// Mass dropped when the engines light under a parachute or after
    /// entry: heat shield and back shell.
    #[serde(default)]
    pub jettison_kg: f64,
}
fn heat() -> f64 {
    500.0
}
fn g_limit() -> f64 {
    15.0
}
fn touch() -> f64 {
    10.0
}

/// The world's air and gravity as the descent reads them.
pub struct World<'a> {
    pub name: String,
    pub radius_m: f64,
    pub gravity_m_s2: f64,
    /// Ground height at the site above the reference radius, m.
    pub ground_m: f64,
    /// Density (kg/m³) and speed of sound (m/s) at height above the reference radius.
    pub air: &'a dyn Fn(f64) -> (f64, f64),
    /// Sutton–Graves constant for this gas, kg^½ m⁻¹.
    pub sutton_graves_k: f64,
    pub surface_wind_m_s: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EntryState {
    /// Above the reference radius, m.
    pub altitude_m: f64,
    pub speed_m_s: f64,
    /// Below the horizon is negative.
    pub flight_path_deg: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DescentSettings {
    pub step_s: f64,
    pub max_steps: u32,
    /// Ignite when the stopping distance at full thrust, times this margin, reaches the ground.
    pub burn_margin: f64,
    /// Telemetry lines every this many seconds (and at every event).
    pub telemetry_s: f64,
    /// Entry interface height for air thinner than this many kg/m³ is the top of the air.
    pub top_density: f64,
}

/// Pack settings for descents (`nav.descent`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DescentPack {
    #[serde(flatten)]
    pub run: DescentSettings,
    /// Sutton–Graves k by gas name, kg^½ m⁻¹ (default for unknown gases).
    pub sutton_graves: std::collections::BTreeMap<String, f64>,
    pub sutton_graves_default: f64,
    /// Site hazards.
    pub slope_limit_deg: f64,
    pub roughness_limit_m: f64,
    /// Site radius looked at, m.
    pub site_radius_m: f64,
    /// Default flight-path angle at the top of the air, degrees (a deorbit
    /// burn from a low orbit gives about −1 to −3).
    pub entry_fpa_deg: f64,
    pub landers: std::collections::BTreeMap<String, Lander>,
}

/// One landing hazard. Every hazard block (the site, the descent, and later
/// weather, quakes and stellar flares) adds entries of this one shape, so the
/// list grows without changing its readers.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Hazard {
    /// slope, roughness, liquid, heating, deceleration, touchdown, ...
    pub kind: String,
    /// ok, caution or danger.
    pub level: String,
    /// The block that raised it: site, descent, ...
    pub source: String,
    pub why: String,
}

/// Level for a value against a limit: danger over it, caution over 75% of it.
pub fn level(value: f64, limit: f64) -> &'static str {
    if value > limit {
        "danger"
    } else if value > 0.75 * limit {
        "caution"
    } else {
        "ok"
    }
}

/// The descent's own hazards.
pub fn hazards(v: &Lander, o: &DescentOutcome) -> Vec<Hazard> {
    let h = |kind: &str, value: f64, limit: f64, why: String| Hazard {
        kind: kind.into(),
        level: level(value, limit).into(),
        source: "descent".into(),
        why,
    };
    vec![
        h(
            "heating",
            o.peak_heat_w_cm2,
            v.heat_limit_w_cm2,
            format!(
                "peak heating {:.0} W/cm² against the shield's {:.0}",
                o.peak_heat_w_cm2, v.heat_limit_w_cm2
            ),
        ),
        h(
            "deceleration",
            o.peak_g,
            v.g_limit,
            format!(
                "peak {:.1} g against a limit of {:.0} g",
                o.peak_g, v.g_limit
            ),
        ),
        h(
            "touchdown",
            o.touchdown_m_s,
            v.touchdown_m_s,
            format!(
                "touchdown at {:.1} m/s, safe up to {:.0}",
                o.touchdown_m_s, v.touchdown_m_s
            ),
        ),
    ]
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DescentEvent {
    pub t_s: f64,
    pub kind: String,
    pub text: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Telemetry {
    pub t_s: f64,
    pub altitude_m: f64,
    pub speed_m_s: f64,
    pub vertical_m_s: f64,
    pub downrange_km: f64,
    pub decel_g: f64,
    pub heat_w_cm2: f64,
    pub phase: String,
    pub thrust: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DescentOutcome {
    /// landed, hard-landing, crash, burned-up, out-of-fuel.
    pub result: String,
    pub touchdown_m_s: f64,
    pub duration_s: f64,
    pub peak_g: f64,
    pub peak_heat_w_cm2: f64,
    pub heat_load_j_cm2: f64,
    pub propellant_used_kg: f64,
    pub downrange_km: f64,
    /// Parachute alone at the ground: the terminal speed, m/s (None without air or chute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chute_terminal_m_s: Option<f64>,
    /// chute, engines, chute-and-engines, aerobrake-only.
    pub method: String,
    pub needs_engines: bool,
    pub why: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Descent {
    pub outcome: DescentOutcome,
    pub events: Vec<DescentEvent>,
    pub telemetry: Vec<Telemetry>,
}

/// Fly the descent.
pub fn descend(s: &DescentSettings, w: &World, v: &Lander, entry: &EntryState) -> Descent {
    let r0 = w.radius_m;
    let g_at = |h: f64| w.gravity_m_s2 * (r0 / (r0 + h)) * (r0 / (r0 + h));
    let mut h = entry.altitude_m;
    let mut speed = entry.speed_m_s;
    let mut gamma = entry.flight_path_deg.to_radians();
    let mut x = 0.0;
    let mut t = 0.0;
    let mut mass = v.mass_kg;
    let fuel0 = v.engine.as_ref().map(|e| e.propellant_kg).unwrap_or(0.0);
    let mut fuel = fuel0;
    let mut chute_open = false;
    let mut chute_t = 0.0;
    let mut burning = false;
    let mut chute_cut = false;
    let mut peak_drag: f64 = 0.0;
    let mut drag_peaked = false;
    let air_at_ground = (w.air)(w.ground_m).0 > 0.0;
    let mut said_dry = false;
    let mut events = vec![DescentEvent {
        t_s: 0.0,
        kind: "entry".into(),
        text: format!(
            "entry interface at {:.0} km, {:.2} km/s, {:.1}° below the horizon",
            h / 1000.0,
            speed / 1000.0,
            -entry.flight_path_deg
        ),
    }];
    let mut tel = vec![];
    let mut peak_g: f64 = 0.0;
    let mut peak_q: f64 = 0.0;
    let mut load = 0.0;
    let mut next_tel = 0.0;
    let mut result = String::new();
    let mut said_peak_heat = false;
    let mut last_q = 0.0;
    let ground = w.ground_m;
    let (cd_a_body, rn) = (v.cd * v.area_m2, v.nose_radius_m.max(0.01));
    for _ in 0..s.max_steps {
        let (rho, a_snd) = (w.air)(h.max(ground));
        let mach = speed / a_snd.max(1e-6);
        let qdyn = 0.5 * rho * speed * speed;
        // Parachute.
        if let Some(c) = &v.chute {
            if !chute_open
                && !burning
                && rho > 0.0
                && mach < c.max_mach
                && qdyn < c.max_q_pa
                && speed > v.touchdown_m_s * 0.5
                && h - ground > 50.0
            {
                chute_open = true;
                chute_t = t;
                events.push(DescentEvent {
                    t_s: t,
                    kind: "chute".into(),
                    text: format!(
                        "parachute out at {:.1} km, Mach {:.2}, {:.0} m/s",
                        (h - ground) / 1000.0,
                        mach,
                        speed
                    ),
                });
            }
        }
        let cd_a = cd_a_body
            + match (&v.chute, chute_open) {
                (Some(c), true) if !chute_cut => {
                    let u = ((t - chute_t) / c.fill_s.max(1e-3)).min(1.0);
                    c.cd * c.area_m2 * u * u
                }
                _ => 0.0,
            };
        // Engines. They light once the air no longer does most of the braking
        // (drag below full thrust) and the stopping distance, at full thrust
        // plus drag less gravity, reaches the ground. Then a two-axis guidance
        // follows a descent-rate profile and spends what is left on killing
        // the sideways speed (Apollo-style powered descent, simplified).
        let mut a_along = 0.0;
        let mut a_normal = 0.0;
        let mut a_cmd = 0.0;
        if let Some(e) = &v.engine {
            let a_max = e.thrust_n / mass;
            let g = g_at(h);
            let drag_now = 0.5 * rho * speed * speed * cd_a / mass;
            let z = h - ground;
            if drag_now > peak_drag {
                peak_drag = drag_now;
            } else if peak_drag > 0.05 * g && drag_now < 0.9 * peak_drag {
                drag_peaked = true;
            }
            if fuel > 0.0 && !burning {
                let net = BRAKE_SHARE * (a_max - g) + drag_now;
                let stop = if net > 0.0 {
                    speed * speed / (2.0 * net) * s.burn_margin
                } else {
                    f64::INFINITY
                };
                // In air, wait for the aerodynamic braking to peak, and for
                // the parachute when there is one (unless the ground is close).
                // The backstop for a parachute that has not opened yet uses
                // the vertical speed only, so it fires near the ground.
                let down = (-speed * m::sin(gamma)).max(0.0);
                let net_v = BRAKE_SHARE * (a_max - g);
                let stop_v = if net_v > 0.0 {
                    down * down / (2.0 * net_v) * s.burn_margin
                } else {
                    f64::INFINITY
                };
                // Under a parachute the canopy kills the sideways speed, so
                // the engines wait for the vertical stopping distance.
                let chuted = v.chute.is_some() && air_at_ground;
                let ready = !air_at_ground || (drag_peaked && drag_now < a_max);
                let due = if chuted { z <= stop_v } else { z <= stop };
                if ready && due && speed > 0.5 * v.touchdown_m_s {
                    burning = true;
                    if v.jettison_kg > 0.0 {
                        mass = (mass - v.jettison_kg).max(fuel + 1.0);
                    }
                    events.push(DescentEvent {
                        t_s: t,
                        kind: "burn".into(),
                        text: format!(
                            "engines lit at {:.2} km, {:.0} m/s ({:.0} m/s down)",
                            z / 1000.0,
                            speed,
                            -speed * m::sin(gamma)
                        ),
                    });
                    if chute_open && !chute_cut {
                        chute_cut = true;
                        events.push(DescentEvent {
                            t_s: t,
                            kind: "chute-released".into(),
                            text: "parachute and back shell released".into(),
                        });
                    }
                }
            }
            if burning && fuel > 0.0 {
                let (u, w) = (speed * m::cos(gamma), speed * m::sin(gamma));
                let r = r0 + h;
                let g_eff = (g - u * u / r).max(0.0);
                // Descent-rate target: fall no faster than a braking profile at
                // most of the spare thrust allows, and never slower than half the
                // touchdown speed.
                let spare = (a_max - g).max(0.1);
                // Leave time to kill the sideways speed first.
                let t_side = u.abs() / (0.8 * a_max);
                let w_side = z.max(0.0) / t_side.max(1e-6) + 0.5 * v.touchdown_m_s;
                let w_brake = m::sqrt(
                    v.touchdown_m_s * v.touchdown_m_s * 0.25
                        + 2.0 * BRAKE_SHARE * spare * z.max(0.0),
                );
                // The braking profile w² = w₀² + 2·share·spare·z needs share·spare of steady
                // deceleration; feed it forward so the loop does not lag.
                let (w_des, ff) = if w_brake <= w_side {
                    (-w_brake, BRAKE_SHARE * spare)
                } else {
                    (-w_side, 0.0)
                };
                let drag_z = -drag_now * m::sin(gamma);
                let az = (g_eff - drag_z + ff + (w_des - w) / GUIDE_TAU_S).clamp(0.0, a_max);
                let room = m::sqrt((a_max * a_max - az * az).max(0.0));
                let drag_x = drag_now * m::cos(gamma);
                let ax = -(u / GUIDE_TAU_S - drag_x).max(0.0).min(room);
                a_along = ax * m::cos(gamma) + az * m::sin(gamma);
                a_normal = -ax * m::sin(gamma) + az * m::cos(gamma);
                a_cmd = m::sqrt(ax * ax + az * az);
            } else if burning && fuel <= 0.0 && !said_dry {
                said_dry = true;
                events.push(DescentEvent {
                    t_s: t,
                    kind: "out-of-fuel".into(),
                    text: format!("propellant gone at {:.0} m", z),
                });
            }
        }
        let thrust = a_cmd * mass;
        // RK4 on (h, speed, gamma, x).
        let isp = v.engine.as_ref().map(|e| e.isp_s).unwrap_or(1.0);
        let lift_k = if burning {
            0.0
        } else {
            v.lift_to_drag * m::cos(v.bank_deg.to_radians())
        };
        let deriv = |h: f64, sp: f64, ga: f64| -> (f64, f64, f64, f64) {
            let (rho, _) = (w.air)(h.max(ground));
            let drag = 0.5 * rho * sp * sp * cd_a / mass;
            let g = g_at(h);
            let r = r0 + h;
            let acc = -drag + a_along - g * m::sin(ga);
            let dga = if sp > 1.0 {
                (sp / r - g / sp) * m::cos(ga) + a_normal / sp + lift_k * drag / sp
            } else {
                0.0
            };
            (sp * m::sin(ga), acc, dga, sp * m::cos(ga) * r0 / r)
        };
        let dt = s.step_s;
        let k1 = deriv(h, speed, gamma);
        let k2 = deriv(
            h + 0.5 * dt * k1.0,
            speed + 0.5 * dt * k1.1,
            gamma + 0.5 * dt * k1.2,
        );
        let k3 = deriv(
            h + 0.5 * dt * k2.0,
            speed + 0.5 * dt * k2.1,
            gamma + 0.5 * dt * k2.2,
        );
        let k4 = deriv(h + dt * k3.0, speed + dt * k3.1, gamma + dt * k3.2);
        let nh = h + dt / 6.0 * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0);
        let mut nsp = speed + dt / 6.0 * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1);
        let ng = gamma + dt / 6.0 * (k1.2 + 2.0 * k2.2 + 2.0 * k3.2 + k4.2);
        x += dt / 6.0 * (k1.3 + 2.0 * k2.3 + 2.0 * k3.3 + k4.3);
        if nsp < 0.0 {
            // Thrust overshoot: hover straight down.
            nsp = 0.0;
        }
        if thrust > 0.0 {
            let used = thrust / (isp * G0) * dt;
            let used = used.min(fuel.max(0.0));
            fuel -= used;
            mass -= used;
        }
        let drag_acc = 0.5 * rho * speed * speed * cd_a / mass;
        let decel = (drag_acc + thrust / mass) / G0;
        peak_g = peak_g.max(decel);
        let q = w.sutton_graves_k * m::sqrt(rho / rn) * speed * speed * speed / 1.0e4;
        if q < last_q && !said_peak_heat && q > 1.0 {
            said_peak_heat = true;
            events.push(DescentEvent {
                t_s: t,
                kind: "peak-heating".into(),
                text: format!("peak heating {:.0} W/cm² at {:.1} km", last_q, h / 1000.0),
            });
        }
        last_q = q;
        peak_q = peak_q.max(q);
        load += q * dt;
        if t >= next_tel {
            tel.push(Telemetry {
                t_s: t,
                altitude_m: h - ground,
                speed_m_s: speed,
                vertical_m_s: speed * m::sin(gamma),
                downrange_km: x / 1000.0,
                decel_g: decel,
                heat_w_cm2: q,
                phase: if burning {
                    "powered"
                } else if chute_open {
                    "parachute"
                } else {
                    "entry"
                }
                .into(),
                thrust: thrust / v.engine.as_ref().map(|e| e.thrust_n).unwrap_or(1.0),
            });
            next_tel += s.telemetry_s;
        }
        h = nh;
        speed = nsp;
        gamma = ng.clamp(-m::PI / 2.0, m::PI / 2.0);
        t += dt;
        if h <= ground {
            let vt = speed;
            result = if vt <= v.touchdown_m_s {
                "landed"
            } else if vt <= 3.0 * v.touchdown_m_s {
                "hard-landing"
            } else {
                "crash"
            }
            .into();
            events.push(DescentEvent {
                t_s: t,
                kind: "touchdown".into(),
                text: format!("touchdown at {:.1} m/s: {}", vt, result.replace('-', " ")),
            });
            break;
        }
        if burning && speed <= v.touchdown_m_s && h - ground < 0.5 {
            result = "landed".into();
            events.push(DescentEvent {
                t_s: t,
                kind: "touchdown".into(),
                text: format!("touchdown at {:.1} m/s on engines", speed),
            });
            break;
        }
    }
    if result.is_empty() {
        result = "still-descending".into();
    }
    if peak_q > v.heat_limit_w_cm2 && result != "still-descending" {
        events.push(DescentEvent {
            t_s: 0.0,
            kind: "heat-shield".into(),
            text: format!(
                "heating peaked at {:.0} W/cm², over the shield's {:.0}",
                peak_q, v.heat_limit_w_cm2
            ),
        });
        result = "burned-up".into();
    }
    // Parachute alone at the ground.
    let (rho_g, _) = (w.air)(ground);
    let chute_terminal = v.chute.as_ref().filter(|_| rho_g > 0.0).map(|c| {
        m::sqrt(2.0 * v.mass_kg * w.gravity_m_s2 / (rho_g * (c.cd * c.area_m2 + cd_a_body)))
    });
    let air_terminal =
        (rho_g > 0.0).then(|| m::sqrt(2.0 * v.mass_kg * w.gravity_m_s2 / (rho_g * cd_a_body)));
    let needs_engines = chute_terminal
        .or(air_terminal)
        .is_none_or(|vt| vt > v.touchdown_m_s);
    let method = match (chute_open, burning || fuel < fuel0) {
        (true, true) => "chute-and-engines",
        (true, false) => "chute",
        (false, true) => "engines",
        (false, false) => "aerobrake-only",
    };
    let mut why = vec![];
    match (chute_terminal, rho_g > 0.0) {
        (Some(vt), _) => why.push(format!(
            "under the parachute alone it would reach the ground at {:.1} m/s (safe is {:.0})",
            vt, v.touchdown_m_s
        )),
        (None, true) => why.push(format!(
            "with no parachute the air alone slows it to {:.0} m/s",
            air_terminal.unwrap_or(0.0)
        )),
        (None, false) => why.push("no air: engines must do it all".into()),
    }
    if needs_engines {
        why.push("so engines must finish the landing".into());
    }
    if peak_g > v.g_limit {
        why.push(format!(
            "deceleration peaks at {:.1} g, over the limit of {:.0}",
            peak_g, v.g_limit
        ));
    }
    if w.surface_wind_m_s > 0.0 {
        why.push(format!("surface wind {:.0} m/s", w.surface_wind_m_s));
    }
    events.sort_by(|a, b| a.t_s.total_cmp(&b.t_s));
    Descent {
        outcome: DescentOutcome {
            result,
            touchdown_m_s: speed,
            duration_s: t,
            peak_g,
            peak_heat_w_cm2: peak_q,
            heat_load_j_cm2: load,
            propellant_used_kg: fuel0 - fuel.max(0.0),
            downrange_km: x / 1000.0,
            chute_terminal_m_s: chute_terminal,
            method: method.into(),
            needs_engines,
            why,
        },
        events,
        telemetry: tel,
    }
}
