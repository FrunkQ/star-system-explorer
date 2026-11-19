//! Epoch stepping (handoff 6.3): quiet stretches advance analytically, and
//! transitions open detailed windows.
//!
//! Each step: the star from its life line and wind table; orbits widen as
//! the star loses mass (a M constant, the adiabatic invariant for slow mass
//! loss: Jeans 1924; Hadjidemetriou 1963); a giant's tides drag close
//! planets in (Zahn 1977 equilibrium tide in the form of Villaver and Livio
//! 2009, eq. 2, with f = (tau / 2P)^2 capped at 1: Verbunt and Phinney
//! 1995); eccentricities and inclinations follow Laplace-Lagrange secular
//! theory, its frequencies scaled with the star's mass; and named
//! transitions are dated, each with its reason.

use crate::kepler::Elements;
use crate::nbody::{Order, Window};
use crate::secular::{Planet, Secular};
use crate::star::{blackbody_rgb, colour_name, AgeingStar, StarSettings};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_formation::lifeline::Phase;
use std::collections::BTreeMap;

const AU_RSUN: f64 = 149_597_870_700.0 / 6.957e8;
const ME_MSUN: f64 = 3.986_004e14 / 1.327_124_4e20;
const DEG: f64 = m::PI / 180.0;

/// Pack settings (`provides.other.time`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TimeSettings {
    pub star: StarSettings,
    pub main_sequence_step_myr: f64,
    pub subgiant_steps: u32,
    pub giant_steps: u32,
    pub tide_substeps: u32,
    pub white_dwarf_steps: u32,
    /// A pair closer than this many mutual Hill radii opens a detailed window.
    pub window_hill: f64,
    pub window_years: f64,
    pub window_steps_per_orbit: f64,
    /// A window rewrites the orbits only when some semi-major axis changed by
    /// more than this fraction.
    pub window_rewrite_da: f64,
    /// Beyond this many Myr after the start, futures are "one plausible future".
    pub horizon_myr: f64,
    /// Insolation (Earth units) at which an ocean world's greenhouse runs away.
    pub runaway_greenhouse_flux: f64,
    /// White dwarf colour bands: lower Teff bound and name, hottest first.
    pub colours: Vec<(f64, String)>,
    pub pollution_delay_myr: f64,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlanetSpec {
    pub name: String,
    pub address: String,
    pub mass_me: f64,
    pub a_au: f64,
    pub e: f64,
    pub i_deg: f64,
    pub node_deg: f64,
    pub varpi_deg: f64,
    pub mean_longitude_deg: f64,
    /// Has surface oceans that a runaway greenhouse would boil away.
    #[serde(default)]
    pub oceans: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DebrisSpec {
    pub name: String,
    pub address: String,
    pub a_in_au: f64,
    pub a_out_au: f64,
    pub mass_me: f64,
}

/// The system to age.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SystemSpec {
    pub name: String,
    pub system: String,
    pub star_address: String,
    pub star_mass: f64,
    pub z: f64,
    /// The star's age now, Myr.
    pub age_myr: f64,
    /// Universe time of the star's birth, years.
    pub birth_year: i64,
    pub planets: Vec<PlanetSpec>,
    #[serde(default)]
    pub debris: Option<DebrisSpec>,
    #[serde(default)]
    pub sources: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlanetNow {
    pub name: String,
    pub a_au: f64,
    pub e: f64,
    pub i_deg: f64,
    /// Engulfed by the star (no longer evolves).
    pub engulfed: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StarNow {
    pub phase: String,
    pub luminosity: f64,
    pub radius_rsun: f64,
    pub teff: f64,
    pub mass: f64,
    pub colour: String,
    pub rgb: [u8; 3],
}

/// The system at one step.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub age_myr: f64,
    pub star: StarNow,
    pub planets: Vec<PlanetNow>,
}

/// A dated transition with its reason.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TimeEvent {
    pub age_myr: f64,
    pub kind: String,
    /// Addresses it concerns.
    pub subjects: Vec<String>,
    /// Plain answer to "why?".
    pub why: String,
    /// Fidelity of the model behind it (sketch, standard).
    pub level: String,
    /// Beyond the predictability horizon: one plausible future.
    pub plausible_future: bool,
    pub numbers: BTreeMap<String, f64>,
}

/// What a detailed window found.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WindowReport {
    pub age_myr: f64,
    pub reason: String,
    pub years: f64,
    pub steps: u64,
    pub energy_error: f64,
    /// Largest change of any planet's semi-major axis across the window
    /// (fraction, Jacobi osculating elements), and whose.
    pub max_da: f64,
    pub max_da_planet: String,
    /// The window's orbits replaced the mean orbits (a real change was found).
    pub written_back: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AgeingRun {
    pub snapshots: Vec<Snapshot>,
    pub events: Vec<TimeEvent>,
    pub windows: Vec<WindowReport>,
    /// Deterministic work units (steps, window steps), for progress.
    pub work: u64,
}

struct Live {
    spec: PlanetSpec,
    p: Planet,
    mean: f64,
    engulfed: bool,
    boiled: bool,
}

pub struct Ageing<'a> {
    pub settings: &'a TimeSettings,
    pub spec: &'a SystemSpec,
    pub star: AgeingStar,
}

impl TimeSettings {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("time"))
            .ok_or("no pack provides time settings")?;
        serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("time settings: {e}"))
    }
}

impl<'a> Ageing<'a> {
    pub fn new(settings: &'a TimeSettings, spec: &'a SystemSpec) -> Result<Self, String> {
        Ok(Ageing {
            settings,
            spec,
            star: AgeingStar::new(spec.star_mass, spec.z, settings.star.clone())?,
        })
    }

    fn star_now(&self, t: f64) -> StarNow {
        let s = self.star.state_at(t);
        StarNow {
            phase: s.star.phase.label().into(),
            luminosity: s.star.luminosity,
            radius_rsun: s.star.radius,
            teff: s.star.teff,
            mass: s.mass,
            colour: colour_name(&self.settings.colours, s.star.teff),
            rgb: blackbody_rgb(s.star.teff),
        }
    }

    /// Step times from `t0` to `t1` (ages, Myr): even steps on the main
    /// sequence, the giant phase on the star's own clock, white dwarf
    /// cooling in log steps, every phase boundary included.
    fn schedule(&self, t0: f64, t1: f64) -> Vec<f64> {
        let s = self.settings;
        let l = &self.star.line;
        let mut ts = vec![t0];
        let mut push = |t: f64| {
            if t > t0 && t < t1 && ts.last().is_some_and(|x| t > *x) {
                ts.push(t);
            }
        };
        let mut t = t0;
        while t + s.main_sequence_step_myr < l.t_tms {
            t += s.main_sequence_step_myr;
            push(t);
        }
        push(l.t_tms);
        for k in 1..=s.subgiant_steps {
            push(l.t_tms + (l.t_bgb - l.t_tms) * f64::from(k) / f64::from(s.subgiant_steps));
        }
        // Giant phase: steps even in log luminosity, through the star's clock.
        for k in 1..s.giant_steps {
            let u = f64::from(k) / f64::from(s.giant_steps);
            push(self.star.age_at_giant_fraction(u));
        }
        // The tip itself, where the star is largest.
        push(
            self.star
                .age_at_giant_fraction(1.0 - 1.0e-9)
                .min(l.t_end - 1.0e-6),
        );
        push(l.t_end);
        let span = (t1 - l.t_end).max(1.0);
        for k in 0..=s.white_dwarf_steps {
            push(l.t_end + m::pow(span, f64::from(k) / f64::from(s.white_dwarf_steps)));
        }
        ts.push(t1);
        ts
    }

    /// Age the system from now to `target_myr` (star age).
    pub fn run(&self, target_myr: f64) -> AgeingRun {
        self.run_with(target_myr, &mut |_| true)
            .expect("an uncancelled run finishes")
    }

    /// Epoch steps the run will take (for progress: each step is one work
    /// unit, each window step one more).
    pub fn planned_steps(&self, target_myr: f64) -> u64 {
        self.schedule(self.spec.age_myr, target_myr).len() as u64 - 1
    }

    /// As `run`, reporting the work done so far (deterministic work units,
    /// never wall clock) after every epoch step; `go` returning false
    /// cancels the run, which then returns `None` and leaves nothing behind.
    pub fn run_with(&self, target_myr: f64, go: &mut dyn FnMut(u64) -> bool) -> Option<AgeingRun> {
        let s = self.settings;
        let t0 = self.spec.age_myr;
        let horizon = t0 + s.horizon_myr;
        let mut live: Vec<Live> = self
            .spec
            .planets
            .iter()
            .map(|p| Live {
                spec: p.clone(),
                p: Planet {
                    mass: p.mass_me * ME_MSUN,
                    a: p.a_au,
                    e: p.e,
                    varpi: p.varpi_deg * DEG,
                    i: p.i_deg * DEG,
                    node: p.node_deg * DEG,
                },
                mean: (p.mean_longitude_deg - p.varpi_deg) * DEG,
                engulfed: false,
                boiled: false,
            })
            .collect();
        let mut run = AgeingRun {
            snapshots: vec![],
            events: vec![],
            windows: vec![],
            work: 0,
        };
        let m0 = self.star.mass_at(t0);
        let ev = |run: &mut AgeingRun,
                  t: f64,
                  kind: &str,
                  subj: Vec<String>,
                  why: String,
                  level: &str,
                  nums: &[(&str, f64)]| {
            run.events.push(TimeEvent {
                age_myr: t,
                kind: kind.into(),
                subjects: subj,
                why,
                level: level.into(),
                plausible_future: t > horizon,
                numbers: nums.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            });
        };
        let ts = self.schedule(t0, target_myr);
        let l = self.star.line.clone();
        let mut phase = self.star.line.phase_at(t0);
        let mut last_hill: Option<f64> = None;
        let mut colour = self.star_now(t0).colour;
        let mut polluted = false;
        let mut r_max = 0.0_f64;
        run.snapshots.push(self.snapshot(t0, &live));
        for w in ts.windows(2) {
            let (ta, tb) = (w[0], w[1]);
            run.work += 1;
            let (ma, mb) = (self.star.mass_at(ta), self.star.mass_at(tb));
            let st_b = self.star.state_at(tb);
            // Orbits widen as the star loses mass.
            for lp in live.iter_mut().filter(|x| !x.engulfed) {
                lp.p.a *= ma / mb;
            }
            // Giant tides, in fixed substeps at each substep's star.
            if matches!(
                self.star.line.phase_at(tb),
                Phase::Giant | Phase::HertzsprungGap
            ) {
                let n = s.tide_substeps.max(1);
                for k in 0..n {
                    let tm = ta + (tb - ta) * (f64::from(k) + 0.5) / f64::from(n);
                    let st = self.star.state_at(tm);
                    let r_au = st.star.radius / AU_RSUN;
                    let menv = (st.mass - st.core_mass).max(0.0);
                    if menv <= 0.0 {
                        continue;
                    }
                    // Convective turnover time (yr) = (Menv (R - Renv)^2 / (3 L))^(1/3), Renv ~ 0.
                    let msun = 1.988_47e30;
                    let r_m = st.star.radius * 6.957e8;
                    let tau_s =
                        m::cbrt(menv * msun * r_m * r_m / (3.0 * st.star.luminosity * 3.828e26));
                    let tau = tau_s / 3.155_76e7;
                    for lp in live.iter_mut().filter(|x| !x.engulfed) {
                        let a = lp.p.a;
                        let period = m::sqrt(a * a * a / st.mass);
                        let f = (tau / (2.0 * period) * (tau / (2.0 * period))).min(1.0);
                        let q = lp.p.mass / st.mass;
                        let rate = f / tau * menv / st.mass * q * (1.0 + q) * m::pow(r_au / a, 8.0);
                        let dt_yr = (tb - ta) / f64::from(n) * 1.0e6;
                        lp.p.a = a * m::exp(-rate * dt_yr);
                    }
                }
            }
            // Secular change, the modes rebuilt for this step's star and orbits.
            let alive: Vec<Planet> = live.iter().filter(|x| !x.engulfed).map(|x| x.p).collect();
            if alive.len() >= 2 {
                let next = Secular::new(mb, &alive).advance(&alive, (tb - ta) * 1.0e6);
                for (lp, np) in live.iter_mut().filter(|x| !x.engulfed).zip(next) {
                    // First-order theory is linear in e and i; for crowded,
                    // massive generated systems it can overshoot. Keep the last
                    // good value if it fails, and stay bound.
                    if !(np.e.is_finite()
                        && np.i.is_finite()
                        && np.varpi.is_finite()
                        && np.node.is_finite())
                    {
                        continue;
                    }
                    lp.p.e = np.e.clamp(0.0, 0.99);
                    lp.p.varpi = np.varpi;
                    lp.p.i = np.i;
                    lp.p.node = np.node;
                }
            }
            // Transitions.
            let r_au = st_b.star.radius / AU_RSUN;
            r_max = r_max.max(r_au);
            for lp in live.iter_mut().filter(|x| !x.engulfed) {
                if lp.p.a * (1.0 - lp.p.e) <= r_au {
                    lp.engulfed = true;
                    let a0 = lp.spec.a_au;
                    ev(
                        &mut run,
                        tb,
                        "engulfed",
                        vec![lp.spec.address.clone(), self.spec.star_address.clone()],
                        format!(
                            "{}'s surface reached {}: the star's radius grew to {:.2} au while the orbit, widened by the star's mass loss from {:.2} to {:.3} au and dragged in by tides, closed to {:.3} au at its nearest",
                            self.spec.name, lp.spec.name, r_au, a0, a0 * m0 / mb, lp.p.a * (1.0 - lp.p.e)
                        ),
                        "sketch",
                        &[("star_radius_au", r_au), ("orbit_au", lp.p.a), ("star_mass", mb)],
                    );
                }
            }
            for lp in live
                .iter_mut()
                .filter(|x| !x.engulfed && x.spec.oceans && !x.boiled)
            {
                let flux = |t: f64, a: f64| self.star.state_at(t).star.luminosity / (a * a);
                if flux(tb, lp.p.a) >= s.runaway_greenhouse_flux {
                    // Date the crossing by fixed bisection (orbit taken at the step's end).
                    let (mut lo, mut hi) = (ta, tb);
                    for _ in 0..60 {
                        let mid = 0.5 * (lo + hi);
                        if flux(mid, lp.p.a) >= s.runaway_greenhouse_flux {
                            hi = mid;
                        } else {
                            lo = mid;
                        }
                    }
                    lp.boiled = true;
                    let lum = self.star.state_at(hi).star.luminosity;
                    ev(
                        &mut run,
                        hi,
                        "oceans_lost",
                        vec![lp.spec.address.clone()],
                        format!(
                            "{} has brightened to {:.3} times the Sun's present luminosity, so {} receives {:.2} times the sunlight that reaches Earth today; above {:.2} an ocean world's water vapour greenhouse runs away and the oceans boil off (Kopparapu et al. 2013)",
                            self.spec.name, lum, lp.spec.name, lum / (lp.p.a * lp.p.a), s.runaway_greenhouse_flux
                        ),
                        "standard",
                        &[("luminosity", lum), ("flux", lum / (lp.p.a * lp.p.a))],
                    );
                }
            }
            let ph = self.star.line.phase_at(tb);
            if ph != phase {
                let sn = self.star_now(tb);
                let (kind, why) = match ph {
                    Phase::HertzsprungGap => ("leaves_main_sequence", format!(
                        "hydrogen in {}'s core has run out after {:.2} Gyr; the core contracts and a hydrogen-burning shell takes over (Hurley, Pols and Tout 2000)",
                        self.spec.name, l.t_tms / 1000.0)),
                    Phase::Giant => ("red_giant", format!(
                        "{}'s envelope swells into a red giant as the helium core grows; its luminosity rises from {:.1} towards {:.0} times the Sun's (core mass-luminosity relation, Boothroyd and Sackmann 1988)",
                        self.spec.name, l.l_bgb, l.l_tip)),
                    Phase::WhiteDwarf => ("planetary_nebula", format!(
                        "{} throws off its remaining envelope as a planetary nebula; the {:.3} solar-mass carbon-oxygen core is left as a white dwarf (Cummings et al. 2018). With {:.3} of its {:.3} solar masses gone, every surviving orbit has widened by {:.2} times. The giant reached {:.2} au at its largest. The newborn white dwarf is at {:.0} K and glows {}",
                        self.spec.name, l.remnant_mass, m0 - l.remnant_mass, m0, m0 / l.remnant_mass, r_max, sn.teff, sn.colour)),
                    _ => ("phase", format!("{} is now a {}", self.spec.name, sn.phase)),
                };
                let at = match ph {
                    Phase::HertzsprungGap => l.t_tms,
                    Phase::Giant => l.t_bgb,
                    _ => l.t_end,
                };
                ev(
                    &mut run,
                    at,
                    kind,
                    vec![self.spec.star_address.clone()],
                    why,
                    "sketch",
                    &[
                        ("luminosity", sn.luminosity),
                        ("mass", sn.mass),
                        ("radius_rsun", sn.radius_rsun),
                    ],
                );
                // Transitions open a detailed window.
                let reason = format!("{kind}: check the orbits directly");
                if let Some(wr) = self.window(tb, &mut live, &reason) {
                    run.work += wr.steps;
                    run.windows.push(wr);
                }
                if ph == Phase::WhiteDwarf {
                    colour = sn.colour.clone();
                }
                phase = ph;
            }
            // Close pairs open a detailed window too: once, and again only if
            // the pair has closed in further by a tenth since (a generated
            // system can be born with a tight but stable pair; checking it at
            // every step would cost much and show nothing new).
            if let Some((x, y, d)) = Self::closest_pair(mb, &live) {
                if d < s.window_hill && last_hill.is_none_or(|h| d < 0.9 * h) {
                    last_hill = Some(d);
                    let reason = format!("{x} and {y} are {d:.2} mutual Hill radii apart");
                    if let Some(wr) = self.window(tb, &mut live, &reason) {
                        run.work += wr.steps;
                        run.windows.push(wr);
                    }
                }
            }
            if ph == Phase::WhiteDwarf {
                let sn = self.star_now(tb);
                if sn.colour != colour {
                    // Date the colour change by fixed bisection on Teff.
                    let bound = s
                        .colours
                        .iter()
                        .find(|(_, n)| *n == colour)
                        .map_or(0.0, |b| b.0);
                    let (mut lo, mut hi) = (ta.max(l.t_end), tb);
                    for _ in 0..60 {
                        let mid = 0.5 * (lo + hi);
                        if self.star.state_at(mid).star.teff < bound {
                            hi = mid;
                        } else {
                            lo = mid;
                        }
                    }
                    let st = self.star_now(hi);
                    ev(&mut run, hi, "cooling_colour", vec![self.spec.star_address.clone()], format!(
                        "the white dwarf has cooled to {:.0} K and now looks {} (it shines only by stored heat, fading as Mestel's law says: Hurley, Pols and Tout 2000 eq. 90)",
                        st.teff, sn.colour), "standard", &[("teff", st.teff), ("luminosity", st.luminosity)]);
                    colour = sn.colour.clone();
                }
                if !polluted && tb >= l.t_end + s.pollution_delay_myr {
                    if let Some(d) = &self.spec.debris {
                        let giant = live
                            .iter()
                            .filter(|x| !x.engulfed && x.p.mass > 30.0 * ME_MSUN)
                            .find(|x| x.spec.a_au > d.a_out_au);
                        if let Some(g) = giant {
                            polluted = true;
                            let at = l.t_end + s.pollution_delay_myr;
                            ev(&mut run, at, "pollution", vec![self.spec.star_address.clone(), d.address.clone()], format!(
                                "the {} widened with the star's mass loss while {}'s resonances shifted; asteroids scattered inward are torn apart and rain metals onto the white dwarf, as seen in about a third of white dwarfs (Debes and Sigurdsson 2002; Bonsor, Mustill and Wyatt 2011; Koester et al. 2014)",
                                d.name, g.spec.name), "sketch", &[("belt_inner_au", d.a_in_au * m0 / l.remnant_mass), ("belt_outer_au", d.a_out_au * m0 / l.remnant_mass)]);
                        }
                    }
                }
            }
            run.snapshots.push(self.snapshot(tb, &live));
            if !go(run.work) {
                return None;
            }
        }
        run.events
            .sort_by(|a, b| a.age_myr.total_cmp(&b.age_myr).then(a.kind.cmp(&b.kind)));
        Some(run)
    }

    fn closest_pair(mass: f64, live: &[Live]) -> Option<(String, String, f64)> {
        let mut alive: Vec<&Live> = live.iter().filter(|x| !x.engulfed).collect();
        alive.sort_by(|a, b| a.p.a.total_cmp(&b.p.a));
        alive
            .windows(2)
            .map(|w| {
                let (x, y) = (w[0], w[1]);
                let rh = m::cbrt((x.p.mass + y.p.mass) / (3.0 * mass)) * 0.5 * (x.p.a + y.p.a);
                (
                    x.spec.name.clone(),
                    y.spec.name.clone(),
                    (y.p.a - x.p.a) / rh,
                )
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
    }

    /// A detailed window at age `t`: the surviving planets integrated for
    /// the window's years with the star's mass following its wind table;
    /// the orbits are written back from the window's end.
    fn window(&self, t: f64, live: &mut [Live], reason: &str) -> Option<WindowReport> {
        let s = self.settings;
        // A planet the giant's tide has dragged to the star in this step is
        // engulfed by the check that follows; it is not integrated.
        let r_star = self.star.state_at(t).star.radius / AU_RSUN;
        let idx: Vec<usize> = (0..live.len())
            .filter(|&k| {
                !live[k].engulfed
                    && live[k].p.a.is_finite()
                    && live[k].p.a * (1.0 - live[k].p.e) > r_star
            })
            .collect();
        if idx.is_empty() {
            return None;
        }
        let mass = self.star.mass_at(t);
        let planets: Vec<(String, f64, Elements)> = idx
            .iter()
            .map(|&k| {
                let lp = &live[k];
                (
                    lp.spec.name.clone(),
                    lp.p.mass,
                    Elements {
                        a: lp.p.a,
                        e: lp.p.e,
                        i: lp.p.i,
                        node: lp.p.node,
                        peri: lp.p.varpi - lp.p.node,
                        mean: lp.mean,
                    },
                )
            })
            .collect();
        let inner = planets.iter().map(|p| p.2.a).fold(f64::INFINITY, f64::min);
        let dt = m::sqrt(inner * inner * inner / mass) / s.window_steps_per_orbit;
        let mut w = Window::from_elements(mass, &planets, dt);
        let e0 = w.energy();
        if !(dt.is_finite() && dt > 0.0) {
            return None;
        }
        let n = m::ceil(s.window_years / dt) as u64;
        let star = &self.star;
        let mass_of = move |yr: f64| star.mass_at(t + yr / 1.0e6);
        w.run(n, Order::Fourth, &mass_of);
        let e1 = w.energy();
        let els: Vec<Elements> = (0..idx.len()).map(|j| w.elements(j + 1)).collect();
        let mut max_da = 0.0_f64;
        let mut max_da_planet = String::new();
        for (el, &k) in els.iter().zip(&idx) {
            let da = (el.a / live[k].p.a - 1.0).abs();
            if da > max_da {
                max_da = da;
                max_da_planet = live[k].spec.name.clone();
            }
        }
        // Osculating elements carry short-period terms (Saturn's near 5:2
        // with Jupiter swings its a by a few parts in a thousand), so the
        // mean orbits are kept unless the window found a real change.
        let written_back = max_da > s.window_rewrite_da;
        for (el, &k) in els.iter().zip(&idx) {
            let lp = &mut live[k];
            lp.mean = el.mean;
            if written_back {
                lp.p.a = el.a;
                lp.p.e = el.e;
                lp.p.i = el.i;
                lp.p.node = el.node;
                lp.p.varpi = el.node + el.peri;
            }
        }
        Some(WindowReport {
            age_myr: t,
            reason: reason.into(),
            years: n as f64 * dt,
            steps: n,
            energy_error: ((e1 - e0) / e0).abs(),
            max_da,
            max_da_planet,
            written_back,
        })
    }

    fn snapshot(&self, t: f64, live: &[Live]) -> Snapshot {
        Snapshot {
            age_myr: t,
            star: self.star_now(t),
            planets: live
                .iter()
                .map(|x| PlanetNow {
                    name: x.spec.name.clone(),
                    a_au: x.p.a,
                    e: x.p.e,
                    i_deg: x.p.i / DEG,
                    engulfed: x.engulfed,
                })
                .collect(),
        }
    }
}
