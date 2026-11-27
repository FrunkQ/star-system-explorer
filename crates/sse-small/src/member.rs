//! Resolving one member of a cloud into a definite body.

use crate::figure as fg;
use crate::{Event, Figure, Host, Lobe, Makeup, Member, Orbit, Pair, Settings, Structure};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};
use sse_formation::leftovers::{Cloud, Leftovers, Spread, Tilt};
use sse_formation::violent::{Upheaval, UpheavalEvent};
use std::collections::BTreeMap;

const AU_M: f64 = sse_core::sheet::AU;
const G: f64 = sse_core::sheet::G;
const M_SUN: f64 = sse_core::sheet::M_SUN;

/// Draw counters, fixed so a draw never shifts when another is added.
mod at {
    pub const DIFF: u64 = 0;
    pub const LAYER: u64 = 1;
    pub const SCATTER: u64 = 2; // 2..6
    pub const RUBBLE: u64 = 10;
    pub const DISRUPTED: u64 = 11;
    pub const SPIN_FLAT: u64 = 12;
    pub const SPIN_PICK: u64 = 13;
    pub const SPIN_MAX: u64 = 14; // 14..17
    pub const SPIN_RESPREAD: u64 = 17;
    pub const PAIR: u64 = 20;
    pub const RATIO: u64 = 21;
    pub const SEPARATION: u64 = 22;
    pub const ELONGATION: u64 = 30;
    pub const FLATTENING: u64 = 31;
    pub const RELIEF: u64 = 32;
    pub const LOBE: u64 = 33; // 33..37
}

/// A body the system's formation made, which exists before anyone looks.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Notable {
    /// The cloud address and rank to resolve it with.
    pub cloud: String,
    pub rank: u64,
    pub why: String,
    pub t_myr: f64,
}

/// What the violent phase did to make a debris cloud: the impact event and
/// the world it came from.
struct ImpactOrigin {
    t_myr: f64,
    outcome: String,
    target: String,
    impactor: String,
}

fn impact_origin(cloud: &Cloud, u: &Upheaval) -> Option<ImpactOrigin> {
    let belt = u.belts.iter().find(|b| b.id == cloud.source)?;
    let world = belt.origin.strip_prefix("impact:")?;
    let earth = cloud.mass_earth();
    // Several impacts may involve one world: the event whose debris matches
    // this belt's mass is the one that made it.
    u.events
        .iter()
        .filter_map(|e| match e {
            UpheavalEvent::Impact {
                t_myr,
                target,
                impactor,
                outcome,
                debris_earth,
                ..
            } if target == world || impactor == world => Some((
                (debris_earth - earth).abs(),
                ImpactOrigin {
                    t_myr: *t_myr,
                    outcome: outcome.clone(),
                    target: target.clone(),
                    impactor: impactor.clone(),
                },
            )),
            _ => None,
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|x| x.1)
}

/// Bodies formation made that exist before anyone looks: the bared core of
/// every world the violent phase shattered is the largest member of that
/// impact's debris.
pub fn notables(u: &Upheaval, l: &Leftovers) -> Vec<Notable> {
    l.clouds
        .iter()
        .filter(|c| c.kind == "debris-belt")
        .filter_map(|c| {
            let o = impact_origin(c, u)?;
            (o.outcome == "shattered").then(|| Notable {
                cloud: c.address.to_string(),
                rank: 1,
                why: format!(
                    "the iron core of {}, laid bare when {} shattered it",
                    o.target, o.impactor
                ),
                t_myr: o.t_myr,
            })
        })
        .collect()
}

fn mean_makeup(c: &Cloud) -> Makeup {
    let s = c.solid.map(|x| x as f64);
    Makeup::from_array([s[0], s[1], s[2], s[3] + s[4] + s[5]])
}

fn mean_e(s: &Spread) -> f64 {
    match s {
        Spread::Rayleigh { sigma } => sigma * m::sqrt(m::PI / 2.0),
        Spread::Uniform { lo, hi } => 0.5 * (lo + hi),
        Spread::Thermal => 2.0 / 3.0,
    }
}

fn mean_i_rad(t: &Tilt) -> f64 {
    match t {
        Tilt::Rayleigh { sigma_deg } => sigma_deg * m::sqrt(m::PI / 2.0) * m::PI / 180.0,
        Tilt::Isotropic => m::PI / 2.0,
    }
}

/// Encounter speed in a disc or shell, m/s: v_kepler sqrt(e^2 + sin^2 i).
fn encounter_speed(a_au: f64, e: f64, i_rad: f64, mass_msun: f64) -> f64 {
    let vk = m::sqrt(G * M_SUN * mass_msun / (a_au * AU_M));
    let si = m::sin(i_rad.min(m::PI / 2.0));
    vk * m::sqrt(e * e + si * si)
}

/// How much more often a body in this cloud is hit than one in the main
/// belt: number density times encounter speed, against the main belt's.
fn collision_factor(c: &Cloud, host: &Host, s: &Settings) -> f64 {
    let cs = &s.collisions;
    let a_mid = 0.5 * (c.a_min_au + c.a_max_au);
    let width = (c.a_max_au - c.a_min_au).max(0.05 * a_mid);
    let vol = |a_mid: f64, width: f64, i: f64, shell: bool, lo: f64, hi: f64| {
        if shell {
            4.0 / 3.0 * m::PI * (hi * hi * hi - lo * lo * lo)
        } else {
            2.0 * m::PI * a_mid * width * 2.0 * a_mid * m::sin(i.max(0.01))
        }
    };
    let shell = matches!(c.i, Tilt::Isotropic);
    let v_here = vol(
        a_mid,
        width,
        mean_i_rad(&c.i),
        shell,
        c.a_min_au,
        c.a_max_au,
    );
    let [lo, hi] = cs.main_belt_a_au;
    let mb_mid = 0.5 * (lo + hi);
    let i_mb = cs.main_belt_i_deg * m::PI / 180.0;
    let v_mb = vol(mb_mid, hi - lo, i_mb, false, lo, hi);
    let n_here = c.over_1km / v_here;
    let n_mb = cs.main_belt_over_1km / v_mb;
    let u_here = encounter_speed(a_mid, mean_e(&c.e), mean_i_rad(&c.i), host.mass_msun);
    let u_mb = encounter_speed(mb_mid, cs.main_belt_e, i_mb, 1.0);
    (n_here * u_here) / (n_mb * u_mb)
}

fn lerp(r: [f64; 2], u: f64) -> f64 {
    r[0] + (r[1] - r[0]) * u
}

/// A truncated exponential: a waiting time with mean `tau`, at most `cap`.
fn truncated_exp(u: f64, tau: f64, cap: f64) -> f64 {
    let f = 1.0 - m::exp(-cap / tau);
    -tau * m::ln(1.0 - u * f)
}

fn fmt_myr(t: f64) -> String {
    if t >= 1000.0 {
        format!("{:.1} billion years", t / 1000.0)
    } else if t >= 1.0 {
        format!("{:.0} million years", t)
    } else {
        format!("{:.0} thousand years", t * 1000.0)
    }
}

/// Resolve the `rank`-th largest member of `cloud`. A pure function of the
/// seed, the cloud, the host, the rank and the settings. `star_au` is the
/// distance from the star for clouds that orbit a planet (rings, irregular
/// moons); `upheaval` names the impact behind a debris cloud.
pub fn resolve(
    seed: &Seed,
    cloud: &Cloud,
    rank: u64,
    host: &Host,
    star_au: Option<f64>,
    upheaval: Option<&Upheaval>,
    s: &Settings,
) -> Member {
    resolve_pinned(
        seed,
        cloud,
        rank,
        host,
        star_au,
        upheaval,
        s,
        &Pins::default(),
    )
}

/// What a GM fixed by hand on a body they added (the Add list, the
/// small-body designer, RW16a): the pair kind ("single", "contact" or
/// "separated"), the eccentricity, and the handful of things that drive the
/// look. Each is optional; whatever is left out still comes from the field,
/// so with nothing pinned the answer is the field's own. A pin is never
/// refused for being implausible: [`crate::takes`] says what it would take.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Pins {
    #[serde(default)]
    pub pair: Option<String>,
    #[serde(default)]
    pub e: Option<f64>,
    /// Mass shares of metal, rock, carbon and ice (normalised).
    #[serde(default)]
    pub makeup: Option<Makeup>,
    /// "rubble-pile", "monolith" or "relaxed" (pulled round by its own gravity).
    #[serde(default)]
    pub structure: Option<String>,
    /// One turn, hours.
    #[serde(default)]
    pub spin_hours: Option<f64>,
    /// How battered, 0 (the smoothest of its kind) to 1 (the roughest): where
    /// its relief sits in the pack's range for its structure.
    #[serde(default)]
    pub battered: Option<f64>,
    /// A pair's secondary radius over the primary's.
    #[serde(default)]
    pub size_ratio: Option<f64>,
}

/// [`resolve`] with pins.
#[allow(clippy::too_many_arguments)]
pub fn resolve_pinned(
    seed: &Seed,
    cloud: &Cloud,
    rank: u64,
    host: &Host,
    star_au: Option<f64>,
    upheaval: Option<&Upheaval>,
    s: &Settings,
    pins: &Pins,
) -> Member {
    let mut body = cloud.resolve(seed, rank);
    if let Some(e) = pins.e {
        body.e = e.clamp(0.0, 0.99);
    }
    let d = Draws::new(
        *seed,
        body.address.clone(),
        Stream::new("small.member.v1").expect("valid"),
    );
    let ring = cloud.kind == "ring";
    let r_star_au = star_au.unwrap_or(body.a_au).max(1e-3);
    let mut history: Vec<Event> = vec![];
    let origin = upheaval.and_then(|u| impact_origin(cloud, u));

    // 1. Makeup: around the cloud's mean, or one layer of a differentiated parent.
    let mean = mean_makeup(cloud);
    let mass_kg = body.mass_kg;
    let mut layer: Option<usize> = None;
    let core = origin.as_ref().is_some_and(|o| o.outcome == "shattered") && rank == 1;
    let makeup = if core {
        let metal_kg = mean.metal * cloud_mass_kg(cloud) * s.makeup.core_keeps;
        let share = (metal_kg / mass_kg).min(0.98);
        let rest = 1.0 - share;
        let other = mean.rock + mean.carbon + mean.ice;
        let k = if other > 0.0 { rest / other } else { 0.0 };
        Makeup::from_array([share, mean.rock * k, mean.carbon * k, mean.ice * k])
    } else {
        let diff = s
            .makeup
            .differentiated_share
            .get(&cloud.kind)
            .copied()
            .unwrap_or(0.0);
        if !ring && d.unit(at::DIFF) < diff {
            // Which layer of the parent it came from, weighted by the layers' masses.
            let w = mean.as_array();
            let u = d.unit(at::LAYER) * w.iter().sum::<f64>();
            let mut acc = 0.0;
            let mut pick = 1;
            for (g, x) in w.iter().enumerate() {
                acc += x;
                if u < acc {
                    pick = g;
                    break;
                }
            }
            layer = Some(pick);
            let p = s.makeup.layer_purity;
            let mut a = w.map(|x| (1.0 - p) * x);
            a[pick] += p;
            Makeup::from_array(a)
        } else {
            let w = mean.as_array();
            let mut a = [0.0; 4];
            for g in 0..4 {
                a[g] = w[g] * m::exp(s.makeup.scatter_ln * d.normal(at::SCATTER + g as u64));
            }
            Makeup::from_array(a)
        }
    };
    let makeup = pins
        .makeup
        .map(|mk| Makeup::from_array(mk.as_array()))
        .unwrap_or(makeup);
    let grain = {
        let rho = s.makeup.grain_density_kg_m3;
        let inv: f64 = makeup
            .as_array()
            .iter()
            .zip(rho.iter())
            .map(|(x, r)| x / r)
            .sum();
        1.0 / inv
    };

    // 2. Temperature and what survives on the surface.
    let temperature_k = 278.6 * m::pow(host.luminosity_lsun.max(1e-6), 0.25) / m::sqrt(r_star_au);
    let ice_keeps = 1.0 / (1.0 + m::exp((temperature_k - s.ice.survives_below_k) / 10.0));
    let surface_ice = makeup.ice * ice_keeps;

    // 3. Collisions: disrupted within the system's age, held by gravity.
    let d_guess_km = body.diameter_km;
    let r_m = 0.5 * d_guess_km * 1000.0;
    let t0 = m::exp(
        (1.0 - makeup.metal) * m::ln(s.collisions.t0_rock_myr)
            + makeup.metal * m::ln(s.collisions.t0_metal_myr),
    );
    let factor = if ring {
        0.0
    } else {
        collision_factor(cloud, host, s).max(1e-12)
    };
    let t_disrupt = t0 * m::sqrt(r_m.max(1e-3)) / factor.max(1e-30);
    let p_disrupt = if ring {
        0.0
    } else {
        1.0 - m::exp(-host.age_myr / t_disrupt)
    };
    let dt = s.collisions.gravity_transition_km;
    let gravity = d_guess_km * d_guess_km / (d_guess_km * d_guess_km + dt * dt);
    let relaxed_strength = relaxed_strength_of(d_guess_km, &makeup, s);
    let rubble_strength = if core {
        0.0
    } else {
        p_disrupt * gravity * (1.0 - relaxed_strength)
    };
    let pinned = pins
        .structure
        .as_deref()
        .filter(|k| matches!(*k, "rubble-pile" | "monolith" | "relaxed"));
    let rubble = match pinned {
        Some(k) => k == "rubble-pile",
        None => d.unit(at::RUBBLE) < rubble_strength,
    };
    let disrupted_myr_ago = (rubble && !ring)
        .then(|| truncated_exp(d.unit(at::DISRUPTED), t_disrupt, host.age_myr.max(1e-3)));
    let (kind_s, porosity) = if let Some(k) = pinned {
        match k {
            "rubble-pile" => ("rubble-pile", s.collisions.porosity_rubble),
            "relaxed" => ("relaxed", 0.0),
            _ => ("monolith", s.collisions.porosity_monolith),
        }
    } else if ring {
        ("ring-aggregate", s.collisions.porosity_rubble)
    } else if core {
        ("stripped-core", s.collisions.porosity_monolith)
    } else if relaxed_strength > 0.5 {
        ("relaxed", 0.0)
    } else if rubble {
        ("rubble-pile", s.collisions.porosity_rubble)
    } else {
        ("monolith", s.collisions.porosity_monolith)
    };
    let rho = grain * (1.0 - porosity);
    let diameter_km = m::cbrt(6.0 * mass_kg / (m::PI * rho)) / 1000.0;
    let radius_m = 0.5 * diameter_km * 1000.0;

    // 4. Sunlight's spin torque (YORP) and the spin.
    let t_yorp = s.yorp.t_myr_at_1km_1au * diameter_km * diameter_km * r_star_au * r_star_au
        / host.luminosity_lsun.max(1e-6);
    let yorp = if ring {
        0.0
    } else {
        1.0 - m::exp(-host.age_myr / t_yorp)
    };
    let barrier_hours = fg::barrier_hours(rho.max(100.0));
    let f_barrier = 24.0 / barrier_hours; // turns per day
    let f_flat = 24.0 / s.yorp.flat_below_hours.max(barrier_hours);
    let mut f = if d.unit(at::SPIN_PICK) < yorp {
        f_flat * d.unit(at::SPIN_FLAT).max(1e-3)
    } else {
        let sig = s.spin.maxwell_mean_per_day / (2.0 * m::sqrt(2.0 / m::PI));
        let (a, b, c) = (
            d.normal(at::SPIN_MAX),
            d.normal(at::SPIN_MAX + 1),
            d.normal(at::SPIN_MAX + 2),
        );
        (sig * m::sqrt(a * a + b * b + c * c)).max(1e-3)
    };
    let f_cap = match kind_s {
        "relaxed" => 24.0 / fg::relaxed_fastest_hours(rho.max(100.0)),
        "monolith" | "stripped-core" => {
            // Strength holds small monoliths beyond the barrier.
            let fast = 24.0 / s.spin.monolith_fastest_hours;
            f_barrier + (fast - f_barrier) * (1.0 - gravity)
        }
        _ => f_barrier * 0.99,
    };
    if f > f_cap {
        f = f_cap * (0.3 + 0.69 * d.unit(at::SPIN_RESPREAD));
    }
    if let Some(h) = pins.spin_hours.filter(|h| *h > 0.0 && h.is_finite()) {
        f = 24.0 / h;
    }
    let spin_hours = 24.0 / f;

    // 5. Pairs: YORP fission of rubble, gentle mergers in a cold disc, impact moons.
    let ps = &s.pairs;
    let v_disp = encounter_speed(
        0.5 * (cloud.a_min_au + cloud.a_max_au),
        mean_e(&cloud.e),
        mean_i_rad(&cloud.i),
        host.mass_msun,
    ) / 1000.0;
    let cold = if ring || star_au.is_some() {
        0.0
    } else {
        1.0 / (1.0 + m::pow(v_disp / ps.cold_speed_km_s, 4.0))
    };
    let primordial = cold * (1.0 - p_disrupt);
    let rubble_share = if rubble { 1.0 } else { rubble_strength };
    let big = m::pow(diameter_km, 4.0) / (m::pow(diameter_km, 4.0) + m::pow(ps.large_moon_km, 4.0));
    let mut odds = [
        (
            "contact",
            "yorp-fission",
            yorp * rubble_share * ps.yorp_contact,
        ),
        (
            "separated",
            "yorp-fission",
            yorp * rubble_share * ps.yorp_separated,
        ),
        ("contact", "gentle-merger", primordial * ps.cold_contact),
        ("separated", "gentle-merger", primordial * ps.cold_separated),
        ("separated", "impact-moon", big * ps.large_moon_share),
    ];
    if ring || core {
        for o in odds.iter_mut() {
            o.2 = 0.0;
        }
    }
    let sum: f64 = odds.iter().map(|o| o.2).sum();
    if sum > 0.95 {
        for o in odds.iter_mut() {
            o.2 *= 0.95 / sum;
        }
    }
    let u = d.unit(at::PAIR);
    let mut acc = 0.0;
    let mut chosen: Option<(&str, &str)> = None;
    if pins.pair.as_deref() == Some("single") {
        // Pinned single: no pair.
    } else if let Some(want) = pins
        .pair
        .as_deref()
        .filter(|w| *w == "contact" || *w == "separated")
    {
        // Pinned: the mechanism is still the field's, weighted by its odds.
        let sum: f64 = odds.iter().filter(|o| o.0 == want).map(|o| o.2).sum();
        if sum > 0.0 {
            for o in odds.iter().filter(|o| o.0 == want) {
                acc += o.2;
                if u * sum < acc {
                    chosen = Some((o.0, o.1));
                    break;
                }
            }
        }
        if chosen.is_none() {
            let how = if cold > 0.5 {
                "gentle-merger"
            } else if want == "separated" && big > 0.5 {
                "impact-moon"
            } else {
                "yorp-fission"
            };
            chosen = Some((
                if want == "contact" {
                    "contact"
                } else {
                    "separated"
                },
                how,
            ));
        }
    } else {
        for o in odds {
            acc += o.2;
            if u < acc {
                chosen = Some((o.0, o.1));
                break;
            }
        }
    }
    let ur = d.unit(at::RATIO);
    let pair = match chosen {
        None => Pair {
            kind: "single".into(),
            origin: None,
            size_ratio: 0.0,
            separation_m: 0.0,
            secondary_mass_kg: 0.0,
        },
        Some((k, o)) => {
            let range = match (k, o) {
                ("contact", "gentle-merger") => ps.cold_secondary,
                ("contact", _) => ps.contact_secondary,
                ("separated", "yorp-fission") => ps.yorp_secondary,
                ("separated", "gentle-merger") => ps.cold_secondary,
                _ => [0.05, 0.2],
            };
            let ratio = pins
                .size_ratio
                .filter(|r| r.is_finite())
                .map(|r| r.clamp(0.01, 1.0))
                .unwrap_or_else(|| lerp(range, ur));
            let r3 = ratio * ratio * ratio;
            let r1 = radius_m / m::cbrt(1.0 + r3);
            let sep = if k == "contact" {
                0.0
            } else {
                let rr = match o {
                    "yorp-fission" => ps.yorp_separation_primary_radii,
                    "gentle-merger" => ps.cold_separation_primary_radii,
                    _ => ps.moon_separation_primary_radii,
                };
                // Log-uniform across the range.
                r1 * rr[0] * m::pow(rr[1] / rr[0], d.unit(at::SEPARATION))
            };
            Pair {
                kind: k.into(),
                origin: Some(o.into()),
                size_ratio: ratio,
                separation_m: sep,
                secondary_mass_kg: mass_kg * r3 / (1.0 + r3),
            }
        }
    };

    // 6. Figure.
    let fs = &s.figure;
    let q = fg::q_of(spin_hours, rho.max(100.0));
    let (eq_b, eq_c, eq_name) = fg::relaxed(q);
    let near_barrier =
        kind_s == "rubble-pile" && spin_hours < barrier_hours / fs.top_shape_share_of_barrier;
    let (ir_b, ir_c, ir_name) = if near_barrier {
        (0.97, 0.88, "top-shape")
    } else {
        let b = lerp(fs.elongation_b_over_a, d.unit(at::ELONGATION));
        let cb = lerp(fs.flattening_c_over_b, d.unit(at::FLATTENING));
        (b, b * cb, "irregular")
    };
    // A pinned structure is taken whole: pinned lumpy keeps no part of the
    // fluid figure, pinned relaxed keeps all of it.
    let shape_relaxed = match pinned {
        Some("relaxed") => 1.0,
        Some(_) => 0.0,
        None => relaxed_strength,
    };
    let w = if kind_s == "relaxed" {
        1.0
    } else {
        shape_relaxed
    };
    let ba = w * eq_b + (1.0 - w) * ir_b;
    let ca = w * eq_c + (1.0 - w) * ir_c;
    let u_relief = pins
        .battered
        .filter(|b| b.is_finite())
        .map(|b| b.clamp(0.0, 1.0))
        .unwrap_or_else(|| d.unit(at::RELIEF));
    let relief = match kind_s {
        "rubble-pile" | "ring-aggregate" => lerp(fs.relief_rubble, u_relief),
        "relaxed" => 0.0,
        _ => lerp(fs.relief_monolith, u_relief),
    } * (1.0 - shape_relaxed);
    let ellipsoid = |r: f64, ba: f64, ca: f64| {
        let a = r / m::cbrt(ba * ca);
        [a, a * ba, a * ca]
    };
    let (fig_kind, axes, lobes) = if pair.kind == "contact" {
        let ratio = pair.size_ratio;
        let r1 = radius_m / m::cbrt(1.0 + ratio * ratio * ratio);
        let r2 = r1 * ratio;
        let flat = pair.origin.as_deref() == Some("gentle-merger");
        let lobe = |r: f64, i: u64| {
            let b = 0.8 + 0.2 * d.unit(at::LOBE + i);
            let c = if flat {
                fs.cold_lobe_c_over_a * (0.9 + 0.2 * d.unit(at::LOBE + 2 + i))
            } else {
                b * (0.7 + 0.25 * d.unit(at::LOBE + 2 + i))
            };
            ellipsoid(r, b, c)
        };
        let (l1, l2) = (lobe(r1, 0), lobe(r2, 1));
        let gap = (l1[0] + l2[0]) * (1.0 - fs.neck_overlap);
        let (m1, m2) = (1.0, ratio * ratio * ratio);
        let x1 = -gap * m2 / (m1 + m2);
        let x2 = gap * m1 / (m1 + m2);
        let env = [
            0.5 * (x2 + l2[0] - (x1 - l1[0])),
            l1[1].max(l2[1]),
            l1[2].max(l2[2]),
        ];
        (
            "two-lobed",
            env,
            vec![
                Lobe {
                    axes_m: l1,
                    offset_m: x1,
                },
                Lobe {
                    axes_m: l2,
                    offset_m: x2,
                },
            ],
        )
    } else {
        let r_primary = if pair.kind == "separated" {
            radius_m / m::cbrt(1.0 + m::pow(pair.size_ratio, 3.0))
        } else {
            radius_m
        };
        let name = if w > 0.5 { eq_name } else { ir_name };
        (name, ellipsoid(r_primary, ba, ca), vec![])
    };
    let figure = Figure {
        kind: fig_kind.into(),
        axes_m: axes,
        lobes,
        relief,
        q,
    };

    // 7. Space weathering since the surface was last made new.
    let surface_age = disrupted_myr_ago.unwrap_or(host.age_myr);
    let weathering = if ring {
        0.0
    } else {
        1.0 - m::exp(-surface_age / (s.weathering.t_myr_at_1au * r_star_au * r_star_au))
    };

    // 8. The type, read off the result.
    let ts = &s.types;
    let q_peri_au = if star_au.is_some() {
        r_star_au
    } else {
        body.a_au * (1.0 - body.e)
    };
    let t_peri =
        278.6 * m::pow(host.luminosity_lsun.max(1e-6), 0.25) / m::sqrt(q_peri_au.max(1e-3));
    let (label, why) = if ring {
        (
            "ring",
            format!(
                "a {} ring particle",
                if makeup.ice > 0.5 {
                    "water-ice"
                } else {
                    "rocky"
                }
            ),
        )
    } else if makeup.metal >= ts.metal_min {
        ("M", format!("{:.0}% metal by mass", 100.0 * makeup.metal))
    } else if layer == Some(1) {
        (
            "V",
            "rock from the crust or mantle of a melted, layered parent".to_string(),
        )
    } else if makeup.ice >= ts.icy_min
        && t_peri > s.ice.survives_below_k
        && temperature_k < s.ice.survives_below_k
    {
        (
            "comet",
            format!(
                "ice-rich ({:.0}%) and its closest approach warms it past {:.0} K",
                100.0 * makeup.ice,
                s.ice.survives_below_k
            ),
        )
    } else if surface_ice >= ts.icy_min {
        if temperature_k < s.ice.reddened_below_k {
            (
                "D",
                format!(
                    "ice-rich and colder than {:.0} K, its surface reddened by radiation",
                    s.ice.reddened_below_k
                ),
            )
        } else {
            ("P", "ice-rich under a dark, carbon-rich crust".to_string())
        }
    } else if makeup.carbon + makeup.ice * (1.0 - ice_keeps) >= ts.carbon_min {
        (
            "C",
            "carbon-rich, its ice long since turned to clays".to_string(),
        )
    } else if weathering < ts.fresh_below {
        (
            "Q",
            "stony, with a fresh surface the solar wind has not yet darkened".to_string(),
        )
    } else {
        ("S", "stony, weathered by the solar wind".to_string())
    };
    let albedo = {
        let a = s.albedo(label);
        if label == "S" || label == "Q" {
            s.albedo("Q") + (s.albedo("S") - s.albedo("Q")) * weathering
        } else {
            a
        }
    };

    // 9. The story.
    let name = |k: &str| match k {
        "asteroid-belt" => "the asteroid belt",
        "outer-belt" => "the outer belt",
        "belt" => "a belt between the giants",
        "debris-belt" => "the debris of an impact",
        "scattered-disc" => "the scattered disc",
        "oort" => "the Oort cloud",
        "trojans" => "a Trojan swarm",
        "irregular-moons" => "a swarm of captured moons",
        "ring" => "a ring",
        _ => "a belt",
    };
    let born = match (&origin, core, layer) {
        (Some(o), true, _) => format!(
            "The iron core of {}, laid bare when {} shattered it {} after the system formed.",
            o.target,
            o.impactor,
            fmt_myr(o.t_myr)
        ),
        (Some(o), false, _) => format!(
            "A piece of {}, thrown off in its {} with {} {} after the system formed.",
            if o.outcome == "hit-and-run" { &o.impactor } else { &o.target },
            match o.outcome.as_str() {
                "hit-and-run" => "grazing hit-and-run",
                "shattered" => "shattering collision",
                _ => "erosive collision",
            },
            if o.outcome == "hit-and-run" { &o.target } else { &o.impactor },
            fmt_myr(o.t_myr)
        ),
        (None, _, Some(g)) => format!(
            "A fragment of the {} of a parent body that grew large and early enough to melt and layer, then broke up, in {}.",
            ["iron core", "rocky mantle", "carbon-rich crust", "icy shell"][g],
            name(&cloud.kind)
        ),
        (None, _, None) => format!("Formed in {} from the material the disc left there.", name(&cloud.kind)),
    };
    history.push(Event {
        myr_ago: origin.as_ref().map(|o| (host.age_myr - o.t_myr).max(0.0)),
        says: born,
    });
    if let Some(t) = disrupted_myr_ago {
        history.push(Event {
            myr_ago: Some(t),
            says: format!(
                "Shattered by a collision about {} ago; the pieces fell back together as a loose rubble pile.",
                fmt_myr(t)
            ),
        });
    } else if kind_s == "monolith" && gravity < 0.5 {
        history.push(Event {
            myr_ago: None,
            says:
                "Small enough that its own strength, not gravity, holds it: a single solid shard."
                    .into(),
        });
    } else if kind_s == "relaxed" {
        history.push(Event {
            myr_ago: None,
            says: "Big enough that its own gravity pulled it round.".into(),
        });
    }
    if yorp > 0.5 && !ring {
        history.push(Event {
            myr_ago: None,
            says: format!(
                "Sunlight's gentle torque (the YORP effect) has had time to set its spin: one turn every {:.1} hours.",
                spin_hours
            ),
        });
    }
    if let Some(o) = &pair.origin {
        let says = match (pair.kind.as_str(), o.as_str()) {
            ("contact", "yorp-fission") => "Spun up by sunlight until it split; the two pieces settled back against each other into a contact binary.",
            ("separated", "yorp-fission") => "Spun up by sunlight until it shed a piece, which now circles it as a small moon.",
            ("contact", "gentle-merger") => "Two bodies that formed side by side in the cold disc and touched at walking pace, like Arrokoth.",
            ("separated", "gentle-merger") => "Formed as a pair in the cold disc and never parted.",
            _ => "A collision threw up the moon that circles it.",
        };
        history.push(Event {
            myr_ago: None,
            says: says.into(),
        });
    }
    if label == "Q" {
        history.push(Event {
            myr_ago: None,
            says: "Its surface was made new recently enough to look unweathered.".into(),
        });
    }

    let mut facts = BTreeMap::new();
    facts.insert("mass_kg".into(), mass_kg);
    facts.insert("radius_m".into(), radius_m);
    facts.insert("density_kg_m3".into(), rho);
    facts.insert("makeup.metal".into(), makeup.metal);
    facts.insert("makeup.rock".into(), makeup.rock);
    facts.insert("makeup.carbon".into(), makeup.carbon);
    facts.insert("makeup.ice".into(), makeup.ice);
    facts.insert("makeup.gas".into(), 0.0);
    facts.insert("spin_hours".into(), spin_hours);
    facts.insert("albedo".into(), albedo);

    Member {
        address: body.address.to_string(),
        cloud: cloud.address.to_string(),
        kind: cloud.kind.clone(),
        rank,
        orbit: Orbit {
            a_au: body.a_au,
            e: body.e,
            i_deg: body.i_deg,
            node_deg: body.node_deg,
            peri_deg: body.peri_deg,
            mean_anomaly_deg: body.mean_anomaly_deg,
            swarm: body.swarm,
        },
        mass_kg,
        diameter_km,
        bulk_density_kg_m3: rho,
        makeup,
        temperature_k,
        type_label: label.into(),
        type_why: why,
        structure: Structure {
            kind: kind_s.into(),
            rubble_strength,
            relaxed_strength,
            disrupted_myr_ago,
            porosity,
        },
        pair,
        spin_hours,
        barrier_hours,
        yorp_strength: yorp,
        figure,
        albedo,
        weathering,
        history,
        facts,
    }
}

fn cloud_mass_kg(c: &Cloud) -> f64 {
    c.mass_earth() * sse_core::sheet::M_EARTH
}

/// The member at an address of the form `<cloud>/obj:<k>` (Cloud::resolve's own child address).
pub fn rank_of(address: &Address) -> Option<u64> {
    address
        .to_string()
        .rsplit_once("obj:")
        .and_then(|(_, k)| k.parse().ok())
}

/// A one-member field for a body a GM adds by hand: the makeup and orbit
/// spread of `template` (the system's own cloud nearest `a_au`, so the body
/// is made of what is really there), at `a_au`, with exactly `mass_kg`.
/// Rank 1 of the returned cloud is the body.
pub fn made_field(
    template: &Cloud,
    address: Address,
    kind: &str,
    a_au: f64,
    mass_kg: f64,
) -> Cloud {
    let mut c = template.clone();
    c.address = address;
    c.kind = kind.to_string();
    c.a_min_au = a_au;
    c.a_max_au = a_au;
    c.a_log = false;
    c.n_top = 1.0;
    c.norm = cloud_mass_kg(&c) / mass_kg.max(1.0);
    c.largest_km = 0.0;
    c
}

/// Mass of a body of diameter `d_km` made of `cloud`'s mean makeup at a
/// monolith's porosity (the resolve then works out its own structure).
pub fn mass_for_diameter(cloud: &Cloud, d_km: f64, s: &Settings) -> f64 {
    let mk = mean_makeup(cloud).as_array();
    let rho = s.makeup.grain_density_kg_m3;
    let inv: f64 = mk.iter().zip(rho.iter()).map(|(x, r)| x / r).sum();
    let bulk = (1.0 - s.collisions.porosity_monolith) / inv;
    m::PI / 6.0 * bulk * m::pow(d_km * 1000.0, 3.0)
}

/// The cloud nearest a distance from the star (by log distance to its range),
/// among those of the given kinds (all kinds when empty).
pub fn nearest_cloud<'a>(clouds: &'a [Cloud], a_au: f64, kinds: &[&str]) -> Option<&'a Cloud> {
    let dist = |c: &Cloud| {
        let la = m::ln(a_au.max(1e-6));
        let (lo, hi) = (m::ln(c.a_min_au.max(1e-6)), m::ln(c.a_max_au.max(1e-6)));
        if la < lo {
            lo - la
        } else if la > hi {
            la - hi
        } else {
            0.0
        }
    };
    clouds
        .iter()
        .filter(|c| c.around.is_none() && (kinds.is_empty() || kinds.contains(&c.kind.as_str())))
        .min_by(|a, b| dist(a).total_cmp(&dist(b)))
}

/// The diameter at which a body of this makeup pulls itself round, km (the
/// pack's rock and ice figures, carbon counted half icy).
pub fn round_km(makeup: &Makeup, s: &Settings) -> f64 {
    s.figure.round_km_rock
        + (s.figure.round_km_icy - s.figure.round_km_rock) * (makeup.ice + makeup.carbon * 0.5)
}

/// How strongly (0 to 1) a body of diameter `d_km` has relaxed to a fluid figure.
pub fn relaxed_strength_of(d_km: f64, makeup: &Makeup, s: &Settings) -> f64 {
    1.0 / (1.0 + m::pow(round_km(makeup, s) / d_km.max(1e-6), 4.0))
}

/// A one-member field for a body designed where the system has no cloud to
/// make it from (no formation record, as for the Solar System as measured):
/// the given makeup, at `a_au`, eccentricity `e`, a thin disc's tilt, with
/// exactly `mass_kg`. Rank 1 of the returned cloud is the body (fudge
/// SB-designed-field: its collision history is a main belt's).
pub fn designed_field(
    address: Address,
    kind: &str,
    makeup: &Makeup,
    a_au: f64,
    e: f64,
    mass_kg: f64,
) -> Cloud {
    let mk = Makeup::from_array(makeup.as_array());
    // Integer budget units in the shares' proportions (the field's mass is
    // set by `norm`, so only the ratios matter).
    let unit = |x: f64| m::round(x * 1.0e9) as u64;
    let solid = [
        unit(mk.metal),
        unit(mk.rock),
        unit(mk.carbon),
        unit(mk.ice),
        0,
        0,
    ];
    let solid = if solid.iter().sum::<u64>() == 0 {
        [0, 1_000_000_000, 0, 0, 0, 0]
    } else {
        solid
    };
    let mut c = Cloud {
        address,
        kind: kind.to_string(),
        around: None,
        source: "designed".into(),
        a_min_au: a_au,
        a_max_au: a_au,
        a_log: false,
        e: Spread::Uniform {
            lo: e.clamp(0.0, 0.99),
            hi: e.clamp(0.0, 0.99),
        },
        i: Tilt::Rayleigh { sigma_deg: 5.0 },
        solid,
        water_from: vec![],
        largest_km: 0.0,
        over_1km: 1.0e6,
        n_top: 1.0,
        norm: 1.0,
    };
    c.norm = cloud_mass_kg(&c) / mass_kg.max(1.0);
    c
}

/// Mass of a body of diameter `d_km` and the given makeup at a monolith's porosity.
pub fn mass_for_makeup(makeup: &Makeup, d_km: f64, s: &Settings) -> f64 {
    let mk = Makeup::from_array(makeup.as_array()).as_array();
    let rho = s.makeup.grain_density_kg_m3;
    let inv: f64 = mk.iter().zip(rho.iter()).map(|(x, r)| x / r).sum();
    let bulk = (1.0 - s.collisions.porosity_monolith) / inv;
    m::PI / 6.0 * bulk * m::pow(d_km * 1000.0, 3.0)
}

/// The makeup a measured bulk density implies, at a monolith's porosity: rock
/// and ice mixed to give the grain density, or rock and metal when it is
/// denser than rock (fudge SB-density-makeup: two-part mixes only).
pub fn makeup_for_density(bulk_kg_m3: f64, s: &Settings) -> Makeup {
    let [metal, rock, _, ice] = s.makeup.grain_density_kg_m3;
    let grain = bulk_kg_m3 / (1.0 - s.collisions.porosity_monolith).max(1e-6);
    // 1 / grain = x / rho_a + (1 - x) / rho_b, solved for x.
    let share =
        |ra: f64, rb: f64| ((1.0 / grain - 1.0 / rb) / (1.0 / ra - 1.0 / rb)).clamp(0.0, 1.0);
    if grain >= rock {
        let x = share(metal, rock);
        Makeup::from_array([x, 1.0 - x, 0.0, 0.0])
    } else {
        let x = share(ice, rock);
        Makeup::from_array([0.0, 1.0 - x, 0.0, x])
    }
}
