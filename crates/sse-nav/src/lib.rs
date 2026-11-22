//! SSE4 NAV (T3, design handoff 6.8): the astrogation service. Given a
//! ship's capabilities, an origin, a destination, a departure window and a
//! preference, it returns a small set of candidate courses, each with its
//! burns, time, propellant and reserve, peak acceleration, radiation dose,
//! heat load, gravity and debris hazards, communication delays and a
//! difficulty rating with reasons. "Ship not under power" is the same
//! service asked for ways out of a drifting state.
//!
//! The model (honest about its edges): two-body conics around the star,
//! with each planet's own gravity only at the ends of a course (patched
//! conics: escape from and capture into a parking orbit). Coasting courses
//! are Lambert arcs; torch courses (continuous thrust) fly straight lines,
//! with the star's pull checked against the thrust and flagged when it is
//! not small. Moons and planets' pulls along the way come later.

pub mod courses;
pub mod descent;
pub mod drift;
pub mod hazards;
pub mod insystem;
pub mod kepler;
pub mod scenario;
pub mod sky;
pub mod voyage;

use kepler::{Elements, V3};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, G, L_SUN, R_SUN};
use std::collections::BTreeMap;

pub const NAV_VERSION: &str = "nav-v1";
pub const M_SUN_KG: f64 = 1.988_47e30;
pub const M_EARTH_KG: f64 = 5.972_2e24;
pub const G0: f64 = 9.806_65;

// ------------------------------------------------------------------ settings

/// Pack settings (`provides.other.nav` in `packs/base-nav`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NavSettings {
    /// Departure times tried across the window.
    pub departures: u32,
    /// Flight times tried per departure, as fractions of the Hohmann time.
    pub tof_fractions: [f64; 2],
    pub tofs: u32,
    /// Rounds of refinement around the cheapest grid point.
    pub refine: u32,
    /// Parking orbit radius in body radii, for escape and capture burns.
    pub parking_radii: f64,
    /// Samples along each course for hazards.
    pub samples: u32,
    /// Torch courses: cruise speeds tried, as fractions of the usable Δv.
    pub torch_fractions: Vec<f64>,
    /// Fixed-point steps for a moving target.
    pub target_steps: u32,
    pub radiation: RadiationSettings,
    pub hazards: HazardSettings,
    pub difficulty: DifficultySettings,
    pub drift: DriftSettings,
    pub descent: descent::DescentPack,
    /// Ship templates by id.
    pub ships: BTreeMap<String, Ship>,
    pub sources: BTreeMap<String, String>,
    /// The Window's sensors and brightness constants (V8).
    #[serde(default)]
    pub window: sky::WindowSettings,
    /// Voyages between stars (V8 part 3).
    #[serde(default)]
    pub voyage: voyage::VoyageSettings,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RadiationSettings {
    /// Galactic cosmic ray dose rate in cruise behind the reference shielding, mSv/day.
    pub gcr_msv_day: f64,
    pub gcr_ref_shield_g_cm2: f64,
    /// e-folding depth of the GCR dose with extra shielding, g/cm².
    pub gcr_atten_g_cm2: f64,
    /// Solar particle events per year at 1 au from a Sun-like star.
    pub spe_per_year_1au: f64,
    /// Dose of one large event behind the reference shielding at 1 au, mSv.
    pub spe_msv_1au: f64,
    /// A body with a dipole moment holds radiation belts when its
    /// magnetopause stands at least this many radii above its centre
    /// (MF0 stop-gap, magnetic fields design; MF7 replaces it).
    #[serde(default = "belt_min_standoff")]
    pub belt_min_standoff_radii: f64,
    /// Solar wind dynamic pressure at 1 au, Pa; it falls as the distance squared.
    #[serde(default = "wind_pressure")]
    pub wind_pressure_1au_pa: f64,
    /// With no moment known, bodies of at least this mass are taken to have
    /// belts (giants all have dynamos); a fallback until MF7.
    pub belt_min_mass_earth: f64,
    /// Belts reach out to this many radii, or the magnetopause if nearer.
    pub belt_radii: f64,
    /// Belt dose rate inside the belts, mSv/hour (Jupiter-like order).
    pub belt_msv_hour: f64,
    /// The equatorial surface field that `belt_msv_hour` belongs to, gauss (Jupiter's).
    #[serde(default = "belt_ref_field")]
    pub belt_ref_field_g: f64,
    /// A belt dose under this, mSv, is listed but adds no difficulty.
    #[serde(default = "belt_reason")]
    pub belt_reason_msv: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HazardSettings {
    /// A pass closer than this many radii of a body is a graze.
    pub graze_radii: f64,
    /// Closer than this many stellar radii is too close to the star.
    pub star_radii: f64,
    /// A torch course is flagged when the star's pull exceeds this fraction of the thrust.
    pub gravity_fraction: f64,
    /// Burns longer than this fraction of the trip lose efficiency (finite-burn losses).
    pub long_burn_fraction: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DifficultySettings {
    /// Points per reason.
    pub points: BTreeMap<String, f64>,
    /// Labels by lowest score, ascending.
    pub bands: Vec<(f64, String)>,
    /// Dose limit for one trip, mSv.
    pub dose_limit_msv: f64,
    pub long_trip_days: [f64; 2],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DriftSettings {
    /// How far ahead a drifting ship's encounters are looked for, years.
    pub horizon_years: f64,
    pub steps: u32,
    /// Encounters listed.
    pub encounters: u32,
    /// The rescue ship when none is given (a template id).
    pub rescue_ship: String,
}

impl NavSettings {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("nav"))
            .ok_or("no pack provides nav settings")?;
        let s: NavSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("nav settings: {e}"))?;
        s.check()?;
        Ok(s)
    }

    pub fn check(&self) -> Result<(), String> {
        for k in self.difficulty.points.keys() {
            if !REASONS.contains(&k.as_str()) {
                return Err(format!("difficulty names unknown reason {k}"));
            }
        }
        for r in REASONS {
            if !self.difficulty.points.contains_key(*r) {
                return Err(format!("difficulty gives no points for {r}"));
            }
        }
        if self.departures == 0 || self.tofs < 2 || self.samples < 2 {
            return Err("departures, tofs and samples must be positive".into());
        }
        if !self.ships.contains_key(&self.drift.rescue_ship) {
            return Err(format!(
                "rescue ship {} is not a template",
                self.drift.rescue_ship
            ));
        }
        Ok(())
    }
}

/// Difficulty reasons the pack scores.
pub const REASONS: &[&str] = &[
    "fuel-tight",
    "fuel-close",
    "over-g",
    "dose",
    "dose-high",
    "heat",
    "graze",
    "star",
    "belts",
    "debris",
    "long-trip",
    "very-long-trip",
    "long-burns",
    "gravity-ignored",
];

// ------------------------------------------------------------------ inputs

/// A ship's capabilities (stock templates in the pack; the GM's own).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ship {
    #[serde(default)]
    pub name: String,
    /// Thrust acceleration, g. Torch courses need a sustained drive.
    pub accel_g: f64,
    /// Sustained: can thrust for days (a torch); otherwise burns are short.
    #[serde(default)]
    pub sustained: bool,
    /// Exhaust velocity, km/s (Isp × g0).
    pub exhaust_km_s: f64,
    /// Mass with full tanks and without propellant, tonnes.
    pub wet_t: f64,
    pub dry_t: f64,
    /// Propellant now, tonnes (default: full).
    #[serde(default)]
    pub propellant_t: Option<f64>,
    /// Fraction of the Δv kept back.
    #[serde(default = "reserve")]
    pub reserve: f64,
    /// Crew limit, g, for hours or more.
    #[serde(default = "g_limit")]
    pub g_limit: f64,
    /// Areal shielding, g/cm².
    #[serde(default = "shield")]
    pub shield_g_cm2: f64,
    /// Most stellar flux the ship can shed, W/m².
    #[serde(default = "flux")]
    pub flux_limit_w_m2: f64,
    /// A jump drive (a pack's FTL rule), if the ship has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump: Option<voyage::JumpDrive>,
}
fn belt_reason() -> f64 {
    10.0
}
fn belt_ref_field() -> f64 {
    4.17
}
fn belt_min_standoff() -> f64 {
    2.0
}
fn wind_pressure() -> f64 {
    1.6e-9
}
fn reserve() -> f64 {
    0.1
}
fn g_limit() -> f64 {
    3.0
}
fn shield() -> f64 {
    20.0
}
fn flux() -> f64 {
    10_000.0
}

impl Ship {
    pub fn propellant(&self) -> f64 {
        self.propellant_t
            .unwrap_or(self.wet_t - self.dry_t)
            .clamp(0.0, (self.wet_t - self.dry_t).max(0.0))
    }
    /// Δv with the propellant aboard, m/s (Tsiolkovsky).
    pub fn dv_m_s(&self) -> f64 {
        let m0 = self.dry_t + self.propellant();
        if self.dry_t <= 0.0 || m0 <= self.dry_t {
            return 0.0;
        }
        self.exhaust_km_s * 1000.0 * m::ln(m0 / self.dry_t)
    }
    pub fn usable_dv_m_s(&self) -> f64 {
        self.dv_m_s() * (1.0 - self.reserve.clamp(0.0, 1.0))
    }
    /// Propellant a burn of `dv` uses, tonnes.
    pub fn propellant_for(&self, dv: f64) -> f64 {
        let m0 = self.dry_t + self.propellant();
        let ve = self.exhaust_km_s * 1000.0;
        if ve <= 0.0 {
            return f64::INFINITY;
        }
        m0 * (1.0 - m::exp(-dv / ve))
    }
    pub fn accel_m_s2(&self) -> f64 {
        self.accel_g * G0
    }
}

/// A body NAV can fly between.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NavBody {
    pub address: String,
    pub name: String,
    pub mass_earth: f64,
    pub radius_m: f64,
    pub a_au: f64,
    pub e: f64,
    #[serde(default)]
    pub i_deg: f64,
    #[serde(default)]
    pub node_deg: f64,
    #[serde(default)]
    pub varpi_deg: f64,
    /// Mean longitude at the system's reference time.
    #[serde(default)]
    pub mean_longitude_deg: f64,
    /// Has a port (rescue can come from here).
    #[serde(default)]
    pub port: bool,
    /// Magnetic dipole moment, A m², where it is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magnetic_moment_a_m2: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Belt {
    pub name: String,
    pub a_in_au: f64,
    pub a_out_au: f64,
}

/// The system as NAV reads it, at a reference time `t0_s` (universe seconds).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NavSystem {
    pub name: String,
    pub star_mass_msun: f64,
    pub star_luminosity_lsun: f64,
    #[serde(default = "one")]
    pub star_radius_rsun: f64,
    pub t0_s: f64,
    pub bodies: Vec<NavBody>,
    #[serde(default)]
    pub belts: Vec<Belt>,
}
fn one() -> f64 {
    1.0
}

impl NavSystem {
    pub fn mu(&self) -> f64 {
        G * self.star_mass_msun * M_SUN_KG
    }
    pub fn star_radius_m(&self) -> f64 {
        self.star_radius_rsun * R_SUN
    }
    pub fn luminosity_w(&self) -> f64 {
        self.star_luminosity_lsun * L_SUN
    }
    pub fn body(&self, address: &str) -> Option<&NavBody> {
        self.bodies
            .iter()
            .find(|b| b.address == address || b.name.eq_ignore_ascii_case(address))
    }
    /// A body's position and velocity at universe time `t_s`.
    pub fn at(&self, b: &NavBody, t_s: f64) -> (V3, V3) {
        let el = Elements {
            a_m: b.a_au * AU,
            e: b.e,
            i_deg: b.i_deg,
            node_deg: b.node_deg,
            varpi_deg: b.varpi_deg,
            mean_longitude_deg: b.mean_longitude_deg,
        };
        kepler::state(&el, self.mu(), t_s - self.t0_s)
    }
}

impl NavBody {
    pub fn mu(&self) -> f64 {
        G * self.mass_earth * M_EARTH_KG
    }
    /// Hill radius around the star, m.
    pub fn hill_m(&self, star_msun: f64) -> f64 {
        let q = self.mass_earth * M_EARTH_KG / (3.0 * star_msun * M_SUN_KG);
        self.a_au * AU * (1.0 - self.e) * m::cbrt(q)
    }
}

/// Where a course starts or ends.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Endpoint {
    /// A body by address or name; the ship sits in its parking orbit.
    Body(String),
    /// A state around the star (a drifting ship, a point in space).
    State { r_m: V3, v_m_s: V3 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Prefer {
    Fastest,
    SaveFuel,
    #[default]
    Balanced,
}

/// One NAV question.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RouteRequest {
    pub ship: Ship,
    pub from: Endpoint,
    /// Absent: the ship is drifting and asks for ways out.
    #[serde(default)]
    pub to: Option<Endpoint>,
    /// Earliest departure, universe seconds (default the system's t0).
    #[serde(default)]
    pub depart_s: Option<f64>,
    #[serde(default = "window")]
    pub window_days: f64,
    #[serde(default)]
    pub prefer: Prefer,
    /// A ship to come to the rescue (drift); the pack's rescue template otherwise.
    #[serde(default)]
    pub rescue: Option<Ship>,
}
fn window() -> f64 {
    365.0
}

// ------------------------------------------------------------------ outputs

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Burn {
    /// Seconds after departure.
    pub at_s: f64,
    pub dv_m_s: f64,
    pub duration_s: f64,
    pub what: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Hazard {
    pub kind: String,
    pub text: String,
    /// Seconds after departure, where it matters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at_s: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Difficulty {
    pub score: f64,
    pub label: String,
    pub reasons: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    /// hohmann, lambert or torch.
    pub kind: String,
    pub label: String,
    pub depart_s: f64,
    pub arrive_s: f64,
    pub days: f64,
    pub dv_m_s: f64,
    pub propellant_t: f64,
    /// Fraction of the propellant left on arrival.
    pub reserve_left: f64,
    pub peak_g: f64,
    pub burns: Vec<Burn>,
    pub dose_msv: f64,
    /// Chance of a large solar particle event during the trip.
    pub spe_chance: f64,
    pub peak_flux_w_m2: f64,
    pub closest_star_au: f64,
    /// One-way light delay to the origin at arrival, and the longest on the way, s.
    pub delay_home_s: f64,
    pub delay_max_s: f64,
    pub hazards: Vec<Hazard>,
    pub difficulty: Difficulty,
    pub why: Vec<String>,
    /// Course positions around the star at the samples, m (for drawing).
    pub track: Vec<V3>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Refused {
    pub kind: String,
    pub why: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RoutePlan {
    pub version: String,
    pub system: String,
    pub from: String,
    pub to: String,
    pub prefer: Prefer,
    pub ship: Ship,
    pub dv_budget_m_s: f64,
    pub usable_dv_m_s: f64,
    /// Candidates, best first by the preference.
    pub candidates: Vec<Course>,
    /// The candidate to recommend (an index into candidates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended: Option<usize>,
    pub refused: Vec<Refused>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drift: Option<drift::DriftReport>,
    pub model: Vec<String>,
}

pub const MODEL_NOTES: &[&str] = &[
    "two-body conics around the star; planets pull only at the ends (escape and capture burns from a parking orbit)",
    "coasting courses are single-revolution Lambert arcs found on a grid of departure and flight times",
    "torch courses fly a straight line with a turnover, the star's pull checked against the thrust",
    "radiation: cosmic rays by days and shielding, solar storms by time spent close to the star; rough",
];

/// Answer one NAV question.
pub fn plan(s: &NavSettings, sys: &NavSystem, req: &RouteRequest) -> Result<RoutePlan, String> {
    let t0 = req.depart_s.unwrap_or(sys.t0_s);
    let name = |e: &Endpoint| -> Result<String, String> {
        match e {
            Endpoint::Body(a) => sys
                .body(a)
                .map(|b| b.address.clone())
                .ok_or_else(|| format!("{a} is not a body NAV knows in {}", sys.name)),
            Endpoint::State { .. } => Ok("state".into()),
        }
    };
    let from = name(&req.from)?;
    let mut out = RoutePlan {
        version: NAV_VERSION.into(),
        system: sys.name.clone(),
        from,
        to: String::new(),
        prefer: req.prefer,
        ship: req.ship.clone(),
        dv_budget_m_s: req.ship.dv_m_s(),
        usable_dv_m_s: req.ship.usable_dv_m_s(),
        candidates: vec![],
        recommended: None,
        refused: vec![],
        drift: None,
        model: MODEL_NOTES.iter().map(|x| x.to_string()).collect(),
    };
    match &req.to {
        None => {
            let (r, v) = match &req.from {
                Endpoint::State { r_m, v_m_s } => (*r_m, *v_m_s),
                Endpoint::Body(_) => {
                    return Err("ways out need the drifting ship's state {r_m, v_m_s}".into())
                }
            };
            out.to = "ways out".into();
            out.drift = Some(drift::ways_out(
                s,
                sys,
                &req.ship,
                req.rescue.as_ref(),
                r,
                v,
                t0,
            )?);
        }
        Some(to) => {
            out.to = name(to)?;
            if out.to == out.from && out.from != "state" {
                return Err("origin and destination are the same body".into());
            }
            let (cands, mut refused) = courses::candidates(s, sys, req, t0, to)?;
            let usable = req.ship.usable_dv_m_s();
            let (ok, over): (Vec<Course>, Vec<Course>) =
                cands.into_iter().partition(|c| c.dv_m_s <= usable);
            refused.extend(over.into_iter().map(|c| Refused {
                kind: c.id,
                why: format!(
                    "needs {:.2} km/s; the ship has {:.2} km/s usable",
                    c.dv_m_s / 1000.0,
                    usable / 1000.0
                ),
            }));
            out.refused = refused;
            out.candidates = order(ok, req.prefer);
            out.recommended = (!out.candidates.is_empty()).then_some(0);
        }
    }
    Ok(out)
}

/// Best first by the preference; ties by id.
fn order(mut c: Vec<Course>, p: Prefer) -> Vec<Course> {
    let key = |x: &Course| -> f64 {
        match p {
            Prefer::Fastest => x.days,
            Prefer::SaveFuel => x.dv_m_s,
            Prefer::Balanced => x.days * x.dv_m_s,
        }
    };
    c.sort_by(|a, b| {
        let fa = a.difficulty.label == "impossible";
        let fb = b.difficulty.label == "impossible";
        fa.cmp(&fb)
            .then(key(a).total_cmp(&key(b)))
            .then(a.id.cmp(&b.id))
    });
    c
}

#[cfg(test)]
mod descent_tests;
#[cfg(test)]
mod tests;
