//! Candidate courses: the textbook Hohmann transfer, Lambert arcs found on a
//! grid of departure and flight times, and torch flights for sustained drives.

use crate::hazards::{assess, Sample};
use crate::kepler::{self, add, norm, scale, sub, V3};
use crate::{Burn, Course, Endpoint, NavBody, NavSettings, NavSystem, Refused, RouteRequest};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, DAY};

/// A course end at a time: position, velocity, and the body if it is one.
pub(crate) fn end_state<'a>(
    sys: &'a NavSystem,
    e: &Endpoint,
    base_t: f64,
    t: f64,
) -> (V3, V3, Option<&'a NavBody>) {
    match e {
        Endpoint::Body(a) => {
            let b = sys.body(a).expect("checked by plan");
            let (r, v) = sys.at(b, t);
            (r, v, Some(b))
        }
        Endpoint::State { r_m, v_m_s } => {
            let (r, v) = kepler::propagate(*r_m, *v_m_s, sys.mu(), t - base_t);
            (r, v, None)
        }
    }
}

/// Burn to leave (or enter) a parking orbit with hyperbolic excess `v_inf`
/// (the Oberth effect: Δv = √(v∞² + 2μ/r) − √(μ/r)); with no body, `v_inf` itself.
pub fn end_burn(s: &NavSettings, b: Option<&NavBody>, v_inf: f64) -> f64 {
    match b {
        Some(b) if b.mass_earth > 0.0 && b.radius_m > 0.0 => {
            let mu = b.mu();
            let r = b.radius_m * s.parking_radii;
            m::sqrt(v_inf * v_inf + 2.0 * mu / r) - m::sqrt(mu / r)
        }
        _ => v_inf,
    }
}

/// Hohmann time between two radii, s.
pub fn hohmann_time(mu: f64, r1: f64, r2: f64) -> f64 {
    let a = 0.5 * (r1 + r2);
    m::PI * m::sqrt(a * a * a / mu)
}

/// Hohmann burns at the two radii (heliocentric), m/s.
pub fn hohmann_dv(mu: f64, r1: f64, r2: f64) -> (f64, f64) {
    let v1 = m::sqrt(mu / r1);
    let v2 = m::sqrt(mu / r2);
    let dv1 = v1 * (m::sqrt(2.0 * r2 / (r1 + r2)) - 1.0);
    let dv2 = v2 * (1.0 - m::sqrt(2.0 * r1 / (r1 + r2)));
    (dv1.abs(), dv2.abs())
}

fn longitude(r: V3) -> f64 {
    m::atan2(r[1], r[0])
}

fn wrap(x: f64) -> f64 {
    let y = m::fmod(x, m::TAU);
    if y > m::PI {
        y - m::TAU
    } else if y <= -m::PI {
        y + m::TAU
    } else {
        y
    }
}

struct Ctx<'a> {
    s: &'a NavSettings,
    sys: &'a NavSystem,
    req: &'a RouteRequest,
    base: f64,
    to: &'a Endpoint,
}

impl Ctx<'_> {
    #[allow(clippy::too_many_arguments)]
    fn course(
        &self,
        id: &str,
        kind: &str,
        label: &str,
        depart: f64,
        arrive: f64,
        dv: f64,
        peak_g: f64,
        burns: Vec<Burn>,
        samples: Vec<Sample>,
        mut why: Vec<String>,
        straight: bool,
    ) -> Course {
        let ship = &self.req.ship;
        let prop = ship.propellant_for(dv);
        let have = ship.propellant();
        let (_, _, fb) = end_state(self.sys, &self.req.from, self.base, depart);
        let (_, _, tb) = end_state(self.sys, self.to, self.base, arrive);
        let burn_time: f64 = burns.iter().map(|b| b.duration_s).sum();
        let a = assess(
            self.s,
            self.sys,
            ship,
            &samples,
            fb,
            tb,
            peak_g,
            dv,
            burn_time / (arrive - depart).max(1.0),
            straight,
        );
        why.extend(a.why);
        Course {
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
            depart_s: depart,
            arrive_s: arrive,
            days: (arrive - depart) / DAY,
            dv_m_s: dv,
            propellant_t: prop,
            reserve_left: if have > 0.0 {
                ((have - prop) / have).max(0.0)
            } else {
                0.0
            },
            peak_g,
            burns,
            dose_msv: a.dose_msv,
            spe_chance: a.spe_chance,
            peak_flux_w_m2: a.peak_flux_w_m2,
            closest_star_au: a.closest_star_m / AU,
            delay_home_s: a.delay_home_s,
            delay_max_s: a.delay_max_s,
            hazards: a.hazards,
            difficulty: a.difficulty,
            why,
            track: samples.iter().map(|x| x.r).collect(),
            // Set by each kind: a torch flight brakes on its drive and arrives at rest.
            arrive_v_inf_m_s: 0.0,
        }
    }

    fn impulsive_burns(&self, dv_dep: f64, dv_arr: f64, tof: f64) -> Vec<Burn> {
        let a = self.req.ship.accel_m_s2().max(1e-9);
        vec![
            Burn {
                at_s: 0.0,
                dv_m_s: dv_dep,
                duration_s: dv_dep / a,
                what: "departure burn".into(),
            },
            Burn {
                at_s: tof,
                dv_m_s: dv_arr,
                duration_s: dv_arr / a,
                what: "arrival burn".into(),
            },
        ]
    }

    fn coast_samples(&self, r: V3, v: V3, depart: f64, tof: f64) -> Vec<Sample> {
        let n = self.s.samples;
        (0..n)
            .map(|k| {
                let tau = tof * k as f64 / (n - 1) as f64;
                let (rr, vv) = kepler::propagate(r, v, self.sys.mu(), tau);
                Sample {
                    t: depart + tau,
                    tau,
                    r: rr,
                    v: vv,
                }
            })
            .collect()
    }
}

/// Every candidate for one origin and destination (over the ship's fuel
/// included: `plan` refuses those), and the ones that cannot be flown.
pub(crate) fn candidates(
    s: &NavSettings,
    sys: &NavSystem,
    req: &RouteRequest,
    t0: f64,
    to: &Endpoint,
) -> Result<(Vec<Course>, Vec<Refused>), String> {
    let c = Ctx {
        s,
        sys,
        req,
        base: t0,
        to,
    };
    let mu = sys.mu();
    let usable = req.ship.usable_dv_m_s();
    let mut out = vec![];
    let mut refused = vec![];
    let (r1, _, fb) = end_state(sys, &req.from, t0, t0);
    let (r2, _, tb) = end_state(sys, to, t0, t0);
    let (n1, n2) = (norm(r1), norm(r2));
    let th = hohmann_time(mu, n1, n2);
    let window = req.window_days.max(0.0) * DAY;

    // The textbook transfer between the two orbits' sizes, at the best phase in the window.
    if let (Some(f), Some(t)) = (fb, tb) {
        let (ra, rb) = (f.a_au * AU, t.a_au * AU);
        let tt = hohmann_time(mu, ra, rb);
        let (d1, d2) = hohmann_dv(mu, ra, rb);
        let dep = end_burn(s, fb, d1);
        let arr = end_burn(s, tb, d2);
        let dv = dep + arr;
        // Target ahead of the origin by π − n₂ t at departure.
        let n_to = m::sqrt(mu / (rb * rb * rb));
        let want = wrap(m::PI - n_to * tt);
        let steps = 720u32;
        let mut best = (f64::INFINITY, t0);
        for k in 0..=steps {
            let td = t0 + window * k as f64 / steps as f64;
            let (pa, _) = sys.at(f, td);
            let (pb, _) = sys.at(t, td);
            let err = wrap(longitude(pb) - longitude(pa) - want).abs();
            if err < best.0 {
                best = (err, td);
            }
        }
        let td = best.1;
        let (pa, va) = sys.at(f, td);
        let vdir = scale(va, 1.0 / norm(va));
        let outward = rb > ra;
        let v_start = add(va, scale(vdir, if outward { d1 } else { -d1 }));
        let samples = c.coast_samples(pa, v_start, td, tt);
        let why = vec![
            format!(
                "Hohmann: half an ellipse touching both orbits ({:.3} and {:.3} au), {:.0} days; burns {:.2} and {:.2} km/s around the star",
                f.a_au, t.a_au, tt / DAY, d1 / 1000.0, d2 / 1000.0
            ),
            format!(
                "leaves when {} leads by {:.0}° (best phase in the window is {:.0}° off)",
                t.name,
                want.to_degrees(),
                best.0.to_degrees()
            ),
            "ideal: assumes circular orbits in one plane; the Lambert courses use the real orbits".into(),
        ];
        let mut course = c.course(
            "hohmann",
            "hohmann",
            "Textbook transfer (Hohmann)",
            td,
            td + tt,
            dv,
            req.ship.accel_g,
            c.impulsive_burns(dep, arr, tt),
            samples,
            why,
            false,
        );
        course.arrive_v_inf_m_s = d2;
        out.push(course);
    }

    // Lambert arcs on a grid.
    let nd = s.departures.max(1);
    let nt = s.tofs.max(2);
    struct Pt {
        td: f64,
        tof: f64,
        dep: f64,
        arr: f64,
        /// Speed over escape at the destination before the arrival burn.
        v_inf: f64,
        r: V3,
        v: V3,
    }
    let eval = |td: f64, tof: f64| -> Option<Pt> {
        if tof <= 0.0 {
            return None;
        }
        let (ra, va, _) = end_state(sys, &req.from, t0, td);
        let (rb, vb, _) = end_state(sys, to, t0, td + tof);
        let (v1, v2) = kepler::lambert(ra, rb, tof, mu)?;
        Some(Pt {
            td,
            tof,
            dep: end_burn(s, fb, norm(sub(v1, va))),
            arr: end_burn(s, tb, norm(sub(v2, vb))),
            v_inf: norm(sub(v2, vb)),
            r: ra,
            v: v1,
        })
    };
    let mut grid: Vec<Pt> = vec![];
    let step_td = if nd > 1 {
        window / (nd - 1) as f64
    } else {
        0.0
    };
    let step_tof = th * (s.tof_fractions[1] - s.tof_fractions[0]) / (nt - 1) as f64;
    for i in 0..nd {
        let td = t0 + step_td * i as f64;
        for j in 0..nt {
            let tof = th * s.tof_fractions[0] + step_tof * j as f64;
            if let Some(p) = eval(td, tof) {
                grid.push(p);
            }
        }
    }
    // Refine the cheapest point: a 5 × 5 search around it, halving the steps.
    if let Some(k) = (0..grid.len())
        .min_by(|&a, &b| (grid[a].dep + grid[a].arr).total_cmp(&(grid[b].dep + grid[b].arr)))
    {
        let (mut td, mut tof) = (grid[k].td, grid[k].tof);
        let mut best = grid[k].dep + grid[k].arr;
        let (mut htd, mut htof) = (0.5 * step_td, 0.5 * step_tof);
        for _ in 0..s.refine {
            let (ctd, ctof) = (td, tof);
            for a in -2i32..=2 {
                for b in -2i32..=2 {
                    let x = (ctd + htd * a as f64).clamp(t0, t0 + window);
                    let y = ctof + htof * b as f64;
                    if let Some(p) = eval(x, y) {
                        let dv = p.dep + p.arr;
                        if dv < best {
                            best = dv;
                            td = x;
                            tof = y;
                        }
                    }
                }
            }
            htd *= 0.5;
            htof *= 0.5;
        }
        if let Some(p) = eval(td, tof) {
            grid.push(p);
        }
    }
    if grid.is_empty() {
        refused.push(Refused {
            kind: "lambert".into(),
            why: "no coasting arc joins the two in the window".into(),
        });
    } else {
        let pick = |score: &dyn Fn(&Pt) -> f64, need_budget: bool| -> Option<usize> {
            let mut best: Option<(f64, usize)> = None;
            for (k, p) in grid.iter().enumerate() {
                if need_budget && p.dep + p.arr > usable {
                    continue;
                }
                let sc = score(p);
                if best.is_none_or(|b| sc < b.0) {
                    best = Some((sc, k));
                }
            }
            best.map(|b| b.1)
        };
        let fuel = pick(&|p| p.dep + p.arr, false).expect("grid not empty");
        let fast = pick(&|p| p.tof, true);
        let bal = pick(&|p| (p.dep + p.arr) * p.tof, true);
        let mut seen: Vec<usize> = vec![];
        let picks = [
            (
                Some(fuel),
                "lambert-fuel",
                "Save fuel: coast on a transfer orbit",
            ),
            (fast, "lambert-fast", "Fastest coast within the fuel"),
            (bal, "lambert-balanced", "Balanced: time against fuel"),
        ];
        for (k, id, label) in picks {
            let Some(k) = k else { continue };
            if seen.contains(&k) {
                continue;
            }
            seen.push(k);
            let p = &grid[k];
            let dv = p.dep + p.arr;
            let why = vec![format!(
                "Lambert arc: leaves {:.0} days into the window, {:.0} days in flight ({:.2} × the Hohmann time); {:.2} km/s to leave, {:.2} km/s to arrive",
                (p.td - t0) / DAY,
                p.tof / DAY,
                p.tof / th,
                p.dep / 1000.0,
                p.arr / 1000.0
            )];
            let samples = c.coast_samples(p.r, p.v, p.td, p.tof);
            let mut course = c.course(
                id,
                "lambert",
                label,
                p.td,
                p.td + p.tof,
                dv,
                req.ship.accel_g,
                c.impulsive_burns(p.dep, p.arr, p.tof),
                samples,
                why,
                false,
            );
            course.arrive_v_inf_m_s = p.v_inf;
            out.push(course);
        }
    }

    // Torch flights.
    let a = req.ship.accel_m_s2();
    if req.ship.sustained && a > 0.0 {
        let (ra, va, _) = end_state(sys, &req.from, t0, t0);
        let ends = end_burn(s, fb, 0.0) + end_burn(s, tb, 0.0);
        let mut last_dv = -1.0;
        for &frac in &s.torch_fractions {
            let budget = usable * frac;
            // Fixed point on the target's motion.
            let mut tt = 2.0 * m::sqrt(norm(sub(r2, r1)) / a);
            let mut res = (0.0, 0.0, 0.0, 0.0, false, [0.0; 3]);
            let mut ok = true;
            for _ in 0..s.target_steps.max(1) {
                let (rb, vb, _) = end_state(sys, to, t0, t0 + tt);
                let d = norm(sub(rb, ra));
                let vrel = norm(sub(vb, va));
                let b = budget - ends - vrel;
                if b <= 0.0 {
                    ok = false;
                    break;
                }
                let vpeak = m::sqrt(a * d);
                let (t_move, dv_move, coast) = if vpeak <= 0.5 * b {
                    (2.0 * m::sqrt(d / a), 2.0 * vpeak, false)
                } else {
                    let vc = 0.5 * b;
                    (d / vc + vc / a, 2.0 * vc, true)
                };
                tt = t_move + vrel / a;
                res = (t_move, dv_move + vrel + ends, d, vrel, coast, rb);
            }
            let (t_move, dv, d, vrel, coast, rb) = res;
            if !ok || dv <= 0.0 {
                refused.push(Refused {
                    kind: format!("torch-{:.0}", frac * 100.0),
                    why: format!(
                        "the matching of speeds and the escape and capture take more than {:.0}% of the fuel",
                        frac * 100.0
                    ),
                });
                continue;
            }
            if (dv - last_dv).abs() < 1e-6 * dv.max(1.0) {
                continue;
            }
            last_dv = dv;
            let u = scale(sub(rb, ra), 1.0 / d.max(1.0));
            let vc = if coast {
                0.5 * (dv - vrel - ends)
            } else {
                m::sqrt(a * d)
            };
            let t_acc = vc / a;
            let n = s.samples;
            let samples: Vec<Sample> = (0..n)
                .map(|k| {
                    let tau = tt * k as f64 / (n - 1) as f64;
                    let x = tau.min(t_move);
                    let (dist, speed) = if x <= t_acc {
                        (0.5 * a * x * x, a * x)
                    } else if x >= t_move - t_acc {
                        let y = t_move - x;
                        (d - 0.5 * a * y * y, a * y)
                    } else {
                        (0.5 * a * t_acc * t_acc + vc * (x - t_acc), vc)
                    };
                    Sample {
                        t: t0 + tau,
                        tau,
                        r: add(ra, scale(u, dist)),
                        v: add(va, scale(u, speed)),
                    }
                })
                .collect();
            let mut burns = vec![Burn {
                at_s: 0.0,
                dv_m_s: vc + ends * 0.5,
                duration_s: t_acc,
                what: "accelerate".into(),
            }];
            if coast {
                burns.push(Burn {
                    at_s: t_acc,
                    dv_m_s: 0.0,
                    duration_s: t_move - 2.0 * t_acc,
                    what: format!("coast at {:.0} km/s", vc / 1000.0),
                });
            }
            burns.push(Burn {
                at_s: t_move - t_acc,
                dv_m_s: vc + ends * 0.5,
                duration_s: t_acc,
                what: "turn over and brake".into(),
            });
            if vrel > 0.0 {
                burns.push(Burn {
                    at_s: t_move,
                    dv_m_s: vrel,
                    duration_s: vrel / a,
                    what: "match the destination's orbital speed".into(),
                });
            }
            let id = format!("torch-{:.0}", frac * 100.0);
            let label = if coast {
                format!("Torch: cruise at {:.0} km/s", vc / 1000.0)
            } else {
                "Torch: burn all the way, turn over at midpoint".into()
            };
            let why = vec![
                format!(
                    "{:.3} au in a straight line at {:.2} g: {} ({:.1} days)",
                    d / AU,
                    req.ship.accel_g,
                    if coast {
                        "accelerate, coast, brake"
                    } else {
                        "t = 2√(d/a)"
                    },
                    t_move / DAY
                ),
                format!(
                    "plus {:.2} km/s to match orbital speeds and {:.2} km/s to leave and enter parking orbits",
                    vrel / 1000.0,
                    ends / 1000.0
                ),
            ];
            out.push(c.course(
                &id,
                "torch",
                &label,
                t0,
                t0 + tt,
                dv,
                req.ship.accel_g,
                burns,
                samples,
                why,
                true,
            ));
        }
    } else if req.ship.sustained {
        refused.push(Refused {
            kind: "torch".into(),
            why: "the drive gives no thrust".into(),
        });
    }

    // Coasting courses take their burns as short kicks. A gentle drive cannot:
    // past the share set in the pack, the burns are most of the flight, and the
    // course is flown as a slow spiral instead (RK5).
    let max_share = s.impulsive_burn_max;
    let mut kept = vec![];
    for c in out {
        let coasting = c.kind == "hohmann" || c.kind == "lambert";
        let burning: f64 = c.burns.iter().map(|b| b.duration_s).sum();
        let flight = (c.arrive_s - c.depart_s).max(1.0);
        if coasting && burning > max_share * flight {
            refused.push(Refused {
                kind: c.id.clone(),
                why: format!(
                    "its burns take {:.0} days of a {:.0}-day flight, so they are not short kicks; flown as a slow spiral instead",
                    burning / DAY,
                    flight / DAY
                ),
            });
        } else {
            kept.push(c);
        }
    }
    let mut out = kept;
    if let Some(c) = spiral(&c, t0, window, max_share) {
        out.push(c);
    }
    Ok((out, refused))
}

/// Speed of a circular parking orbit about a body, m/s (zero with no body).
fn parking_speed(s: &NavSettings, b: Option<&NavBody>) -> f64 {
    match b {
        Some(b) if b.mass_earth > 0.0 && b.radius_m > 0.0 => {
            m::sqrt(b.mu() / (b.radius_m * s.parking_radii))
        }
        _ => 0.0,
    }
}

/// A slow spiral (RK5): months of gentle thrust along the orbit, for a drive
/// too weak for short burns. Out of the parking orbit, across between the two
/// circular orbits, and down into the parking orbit at the far end. Each part
/// costs about the difference of the circular speeds (Edelbaum 1961, coplanar
/// case; escape from a circular orbit by tangential thrust costs about the
/// orbit's own speed). Thrust is constant, so the ship speeds up as it burns
/// off its mass: t = (vₑ/a₀)(1 − e^(−Δv/vₑ)). It leaves when the far world will
/// be alongside where the spiral ends, searched over the window as the Hohmann
/// phase is. Offered only when the short-burn courses would not be short.
fn spiral(c: &Ctx, t0: f64, window: f64, max_share: f64) -> Option<Course> {
    let (s, sys, req) = (c.s, c.sys, c.req);
    let (Endpoint::Body(fa), Endpoint::Body(ta)) = (&req.from, c.to) else {
        return None;
    };
    let (f, t) = (sys.body(fa)?, sys.body(ta)?);
    let ship = &req.ship;
    let a0 = ship.accel_m_s2();
    let ve = ship.exhaust_km_s * 1000.0;
    if a0 <= 0.0 || ve <= 0.0 || f.a_au <= 0.0 || t.a_au <= 0.0 {
        return None;
    }
    let mu = sys.mu();
    let (ra, rb) = (f.a_au * AU, t.a_au * AU);
    // Only when the textbook transfer's own burns would be long.
    let (d1, d2) = hohmann_dv(mu, ra, rb);
    let short = (end_burn(s, Some(f), d1) + end_burn(s, Some(t), d2)) / a0;
    if short <= max_share * hohmann_time(mu, ra, rb) {
        return None;
    }
    let (v1, v2) = (m::sqrt(mu / ra), m::sqrt(mu / rb));
    let (out_dv, in_dv) = (parking_speed(s, Some(f)), parking_speed(s, Some(t)));
    let across = (v1 - v2).abs();
    let dv = out_dv + across + in_dv;
    // Time to have spent `x` of Δv, with the mass going down as it burns.
    let time_for = |x: f64| (ve / a0) * (1.0 - m::exp(-x / ve));
    let t_out = time_for(out_dv);
    let t_across = time_for(out_dv + across) - t_out;
    let t_in = time_for(dv) - t_out - t_across;
    // The angle swept on the way across: dθ = (v/r) dt with r = μ/v², stepped in Δv.
    let steps = 2000u32;
    let mut theta = 0.0;
    let mut track: Vec<(f64, f64, f64)> = vec![(0.0, ra, 0.0)];
    let mut tau = 0.0;
    for k in 0..steps {
        let x0 = across * k as f64 / steps as f64;
        let x1 = across * (k + 1) as f64 / steps as f64;
        let dt = time_for(out_dv + x1) - time_for(out_dv + x0);
        let xm = 0.5 * (x0 + x1);
        let v = if rb > ra { v1 - xm } else { v1 + xm };
        let r = mu / (v * v);
        theta += v / r * dt;
        tau += dt;
        track.push((tau, r, theta));
    }
    // The far world must be alongside where the spiral ends: search the window.
    let lon = |r: V3| m::atan2(r[1], r[0]);
    let n = 720u32;
    let mut best = (f64::INFINITY, t0);
    for k in 0..=n {
        let td = t0 + window * k as f64 / n as f64;
        let (pa, _) = sys.at(f, td + t_out);
        let (pb, _) = sys.at(t, td + t_out + t_across);
        let err = wrap(lon(pb) - lon(pa) - theta).abs();
        if err < best.0 {
            best = (err, td);
        }
    }
    let td = best.1;
    let total = t_out + t_across + t_in;
    let (pa, _) = sys.at(f, td + t_out);
    let start = lon(pa);
    let ns = s.samples.max(2);
    let samples: Vec<Sample> = (0..ns)
        .map(|k| {
            let tau = total * k as f64 / (ns - 1) as f64;
            let (r, v) = if tau <= t_out {
                sys.at(f, td + tau)
            } else if tau >= t_out + t_across {
                sys.at(t, td + tau)
            } else {
                let x = tau - t_out;
                let i = track.partition_point(|p| p.0 < x).min(track.len() - 1);
                let (_, rr, th) = track[i];
                let ang = start + th;
                let sp = m::sqrt(mu / rr);
                (
                    [rr * m::cos(ang), rr * m::sin(ang), 0.0],
                    [-sp * m::sin(ang), sp * m::cos(ang), 0.0],
                )
            };
            Sample {
                t: td + tau,
                tau,
                r,
                v,
            }
        })
        .collect();
    let mut burns = vec![];
    if out_dv > 0.0 {
        burns.push(Burn {
            at_s: 0.0,
            dv_m_s: out_dv,
            duration_s: t_out,
            what: format!("spiral out of orbit about {}", f.name),
        });
    }
    burns.push(Burn {
        at_s: t_out,
        dv_m_s: across,
        duration_s: t_across,
        what: format!(
            "spiral {} to {}'s orbit",
            if rb > ra { "out" } else { "in" },
            t.name
        ),
    });
    if in_dv > 0.0 {
        burns.push(Burn {
            at_s: t_out + t_across,
            dv_m_s: in_dv,
            duration_s: t_in,
            what: format!("spiral down into orbit about {}", t.name),
        });
    }
    let why = vec![
        format!(
            "slow spiral at {:.1e} g: thrusting the whole way, {:.0} days ({:.0} out of orbit about {}, {:.0} across, {:.0} down to {})",
            ship.accel_g, total / DAY, t_out / DAY, f.name, t_across / DAY, t_in / DAY, t.name
        ),
        format!(
            "Δv about the differences of circular speeds (Edelbaum): {:.2} km/s to leave, {:.2} across, {:.2} to arrive",
            out_dv / 1000.0, across / 1000.0, in_dv / 1000.0
        ),
        format!(
            "leaves {:.0} days into the window so {} is alongside at the end ({:.0}° off); orbits taken as circular and in one plane",
            (td - t0) / DAY, t.name, best.0.to_degrees()
        ),
    ];
    Some(c.course(
        "spiral",
        "spiral",
        "Slow spiral: thrust all the way",
        td,
        td + total,
        dv,
        ship.accel_g,
        burns,
        samples,
        why,
        false,
    ))
}
