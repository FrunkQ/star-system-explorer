//! Hazards along a course and its difficulty with reasons.
//!
//! Radiation: cruise dose from galactic cosmic rays scaled by days and extra
//! shielding (Zeitlin et al. 2013, Science 340, 1080, doi:10.1126/science.1235989:
//! 1.84 mSv/day measured in cruise to Mars); solar particle events by time
//! spent close to the star, scaled by (r / 1 au)⁻² and luminosity; giants'
//! radiation belts on close passes. Heat: the star's flux L / 4πr². Gravity:
//! grazes and the star's pull against the thrust. Debris: time in belts.
//! Delays: one-way light time to the origin.

use crate::kepler::{norm, sub, V3};
use crate::{Difficulty, Hazard, NavBody, NavSettings, NavSystem, Ship, G0};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, C, DAY};

/// One point along a course: universe time, time since departure, state.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sample {
    pub t: f64,
    pub tau: f64,
    pub r: V3,
    pub v: V3,
}

pub struct Assessment {
    pub dose_msv: f64,
    pub spe_chance: f64,
    pub peak_flux_w_m2: f64,
    pub closest_star_m: f64,
    pub delay_home_s: f64,
    pub delay_max_s: f64,
    pub hazards: Vec<Hazard>,
    pub difficulty: Difficulty,
    pub why: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn assess(
    s: &NavSettings,
    sys: &NavSystem,
    ship: &Ship,
    samples: &[Sample],
    from: Option<&NavBody>,
    to: Option<&NavBody>,
    peak_g: f64,
    dv: f64,
    burn_fraction: f64,
    straight: bool,
) -> Assessment {
    let rs = &s.radiation;
    let hz = &s.hazards;
    let mut hazards = vec![];
    let mut reasons: Vec<(String, String)> = vec![];
    let n = samples.len();
    let days = if n > 1 {
        (samples[n - 1].tau - samples[0].tau) / DAY
    } else {
        0.0
    };
    // Cosmic rays.
    let extra = (ship.shield_g_cm2 - rs.gcr_ref_shield_g_cm2).max(-rs.gcr_ref_shield_g_cm2);
    let gcr_rate = rs.gcr_msv_day * m::exp(-extra / rs.gcr_atten_g_cm2);
    let mut dose = gcr_rate * days;
    // Solar storms and heat, by distance from the star.
    let lum = sys.star_luminosity_lsun.max(1e-6);
    let mut spe_expect = 0.0;
    let mut spe_dose = 0.0;
    let mut peak_flux: f64 = 0.0;
    let mut closest = f64::INFINITY;
    let mut closest_at = 0.0;
    for w in samples.windows(2) {
        let dt = (w[1].tau - w[0].tau) / DAY;
        let r = 0.5 * (norm(w[0].r) + norm(w[1].r));
        let k = lum * (AU / r) * (AU / r);
        spe_expect += rs.spe_per_year_1au / 365.25 * k * dt;
        spe_dose += rs.spe_per_year_1au / 365.25
            * k
            * dt
            * rs.spe_msv_1au
            * k
            * m::exp(-extra / rs.gcr_atten_g_cm2);
    }
    for x in samples {
        let r = norm(x.r);
        let f = sys.luminosity_w() / (4.0 * m::PI * r * r);
        peak_flux = peak_flux.max(f);
        if r < closest {
            closest = r;
            closest_at = x.tau;
        }
    }
    let spe_chance = 1.0 - m::exp(-spe_expect);
    dose += spe_dose;
    // Belts, grazes, the star.
    let r_star = sys.star_radius_m();
    if closest < hz.star_radii * r_star {
        reasons.push((
            "star".into(),
            format!("passes {:.1} stellar radii from the star", closest / r_star),
        ));
        hazards.push(Hazard {
            kind: "star".into(),
            text: format!("passes {:.3} au from the star", closest / AU),
            at_s: Some(closest_at),
        });
    }
    let end_skip = (n / 16).max(1);
    for b in &sys.bodies {
        let mut best = (f64::INFINITY, 0.0);
        for (k, x) in samples.iter().enumerate() {
            let at_end = (Some(b) == from && k < end_skip) || (Some(b) == to && k + end_skip >= n);
            if at_end {
                continue;
            }
            let (rb, _) = sys.at(b, x.t);
            let d = norm(sub(x.r, rb));
            if d < best.0 {
                best = (d, x.tau);
            }
        }
        let ends = Some(b) == from || Some(b) == to;
        if best.0 < hz.graze_radii * b.radius_m && !ends {
            reasons.push((
                "graze".into(),
                format!("passes {:.0} km from {}", best.0 / 1000.0, b.name),
            ));
            hazards.push(Hazard {
                kind: "graze".into(),
                text: format!(
                    "passes {:.1} radii from {}: its pull bends the course, so it needs a correction",
                    best.0 / b.radius_m,
                    b.name
                ),
                at_s: Some(best.1),
            });
        }
        let belts = b.mass_earth >= rs.belt_min_mass_earth;
        let in_belts = best.0 < rs.belt_radii * b.radius_m || (ends && belts);
        if belts && in_belts {
            // Rough time inside: the belt diameter at the local speed, or a day at an end.
            let hours = if ends {
                24.0
            } else {
                let v = samples
                    .iter()
                    .map(|x| norm(x.v))
                    .fold(0.0_f64, f64::max)
                    .max(1.0);
                2.0 * rs.belt_radii * b.radius_m / v / 3600.0
            };
            let d = rs.belt_msv_hour * hours * m::exp(-extra / rs.gcr_atten_g_cm2);
            dose += d;
            reasons.push((
                "belts".into(),
                format!("{:.0} mSv in {}'s radiation belts", d, b.name),
            ));
            hazards.push(Hazard {
                kind: "belts".into(),
                text: format!(
                    "about {:.0} hours in {}'s radiation belts, {:.0} mSv behind this shielding",
                    hours, b.name, d
                ),
                at_s: Some(best.1),
            });
        }
    }
    // Debris belts.
    for belt in &sys.belts {
        let mut inside = 0.0;
        let mut first = None;
        for w in samples.windows(2) {
            let r = norm(w[0].r) / AU;
            let z = w[0].r[2].abs() / AU;
            if r >= belt.a_in_au && r <= belt.a_out_au && z < 0.2 * r {
                inside += w[1].tau - w[0].tau;
                first.get_or_insert(w[0].tau);
            }
        }
        if inside > 0.0 {
            reasons.push((
                "debris".into(),
                format!("{:.0} days in the {}", inside / DAY, belt.name),
            ));
            hazards.push(Hazard {
                kind: "debris".into(),
                text: format!(
                    "{:.0} days crossing the {} ({:.1} to {:.1} au): watch for debris",
                    inside / DAY,
                    belt.name,
                    belt.a_in_au,
                    belt.a_out_au
                ),
                at_s: first,
            });
        }
    }
    // The star's pull against the thrust on a straight course.
    let mut why = vec![];
    if straight {
        let a = ship.accel_m_s2().max(1e-9);
        let worst = samples
            .iter()
            .map(|x| {
                let r = norm(x.r);
                sys.mu() / (r * r) / a
            })
            .fold(0.0_f64, f64::max);
        if worst > hz.gravity_fraction {
            reasons.push((
                "gravity-ignored".into(),
                format!(
                    "the star's pull reaches {:.0}% of the thrust: the straight course is approximate",
                    worst * 100.0
                ),
            ));
        } else {
            why.push(format!(
                "the star's pull stays under {:.1}% of the thrust, so a straight course is close",
                worst * 100.0
            ));
        }
    }
    // Delays.
    let home = |t: f64| from.map(|b| sys.at(b, t).0);
    let mut delay_max: f64 = 0.0;
    let mut delay_home = 0.0;
    if let Some(first) = samples.first() {
        let origin0 = home(first.t).unwrap_or(first.r);
        for x in samples {
            let o = home(x.t).unwrap_or(origin0);
            let d = norm(sub(x.r, o)) / C;
            delay_max = delay_max.max(d);
            delay_home = d;
        }
    }
    // Fuel, g, dose, heat, length, burns.
    let usable = ship.usable_dv_m_s().max(1e-9);
    let used = dv / usable;
    if used > 0.9 {
        reasons.push((
            "fuel-tight".into(),
            format!("uses {:.0}% of the usable fuel", used * 100.0),
        ));
    } else if used > 0.6 {
        reasons.push((
            "fuel-close".into(),
            format!("uses {:.0}% of the usable fuel", used * 100.0),
        ));
    }
    if peak_g > ship.g_limit {
        reasons.push((
            "over-g".into(),
            format!("{:.1} g is over the crew's {:.1} g", peak_g, ship.g_limit),
        ));
    }
    let dl = s.difficulty.dose_limit_msv;
    if dose > 2.0 * dl {
        reasons.push((
            "dose-high".into(),
            format!("{:.0} mSv, over twice the trip limit of {:.0}", dose, dl),
        ));
    } else if dose > dl {
        reasons.push((
            "dose".into(),
            format!("{:.0} mSv, over the trip limit of {:.0}", dose, dl),
        ));
    }
    if peak_flux > ship.flux_limit_w_m2 {
        reasons.push((
            "heat".into(),
            format!(
                "{:.0} W/m² from the star, over the ship's {:.0}",
                peak_flux, ship.flux_limit_w_m2
            ),
        ));
        hazards.push(Hazard {
            kind: "heat".into(),
            text: format!(
                "peak sunlight {:.0} W/m² ({:.1} × Earth's)",
                peak_flux,
                peak_flux / 1361.0
            ),
            at_s: Some(closest_at),
        });
    }
    let [long, very] = s.difficulty.long_trip_days;
    if days > very {
        reasons.push(("very-long-trip".into(), format!("{:.0} days", days)));
    } else if days > long {
        reasons.push(("long-trip".into(), format!("{:.0} days", days)));
    }
    if !straight && burn_fraction > hz.long_burn_fraction {
        reasons.push((
            "long-burns".into(),
            format!(
                "burns take {:.0}% of the trip at {:.2} g: finite-burn losses",
                burn_fraction * 100.0,
                peak_g
            ),
        ));
    }
    if spe_chance > 0.05 {
        hazards.push(Hazard {
            kind: "solar-storm".into(),
            text: format!(
                "{:.0}% chance of a large solar particle event on the way",
                spe_chance * 100.0
            ),
            at_s: None,
        });
    }
    let score: f64 = reasons
        .iter()
        .map(|(k, _)| s.difficulty.points.get(k).copied().unwrap_or(0.0))
        .sum();
    let label = s
        .difficulty
        .bands
        .iter()
        .rev()
        .find(|(min, _)| score >= *min)
        .map(|(_, l)| l.clone())
        .unwrap_or_else(|| "routine".into());
    let _ = G0;
    Assessment {
        dose_msv: dose,
        spe_chance,
        peak_flux_w_m2: peak_flux,
        closest_star_m: closest,
        delay_home_s: delay_home,
        delay_max_s: delay_max,
        hazards,
        difficulty: Difficulty {
            score,
            label,
            reasons: reasons
                .into_iter()
                .map(|(k, t)| format!("{k}: {t}"))
                .collect(),
        },
        why,
    }
}
