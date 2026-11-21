//! S0: the surface envelope (surface design sections 1, 2.1 and 6).
//!
//! Every world gets a [`SurfaceEnvelope`]: what kind of surface it has, its
//! reference radius and gravity, and its tectonic regime at every date, with
//! a dated, explained event for each change. The regime rules are pack data
//! (`packs/base-surface`): tried in order, the first whose conditions hold
//! names the regime. Inputs are the world's own physics: radiogenic heat flow
//! from the base-physics `planet-heat` flow at each date, tidal heating,
//! surface temperature from the ageing star, water, envelope and composition.
//!
//! [`sketch`] gives the cheap regime globe that unexamined and scanned worlds
//! show (section 1, "Survey and disclosure"): statistically of its kind, not a
//! history run. The full atlas (S1) and plate history (S2) replace it on
//! surveyed worlds.

pub mod atlas;
pub mod cover;
pub mod detail;
pub mod impacts;
pub mod named;
pub mod plates;
pub mod regimes;
pub mod scenario;
pub mod site;
pub mod sketch;
pub mod survey;
pub mod wear;

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Address;
use sse_formation::early::{Envelope, WorldStart};
use sse_formation::hurley::Fits;
use sse_formation::pipeline::{self, SystemRun};
use sse_formation::LifeLine;
use std::collections::BTreeMap;

pub use sketch::{sketch, Sketch, SurveyState};

pub const SURFACE_VERSION: &str = "surface-envelope-v1";

const M_EARTH_KG: f64 = 5.9722e24;
const R_EARTH_M: f64 = 6.371e6;
const M_SUN_KG: f64 = 1.98892e30;
const AU_M: f64 = 1.495978707e11;
const G: f64 = 6.6743e-11;
const GYR_S: f64 = 3.15576e16;

// ---------------------------------------------------------------- settings

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Range {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

impl Range {
    fn holds(&self, v: f64) -> bool {
        self.min.is_none_or(|lo| v >= lo) && self.max.is_none_or(|hi| v < hi)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub regime: String,
    pub when: BTreeMap<String, Range>,
    pub because: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RegimeDef {
    pub label: String,
    /// What the outermost solid or liquid layer is: solid, ice-shell, ocean,
    /// lava, cloud-deck or small-body.
    pub kind: String,
    /// Which sketch grammar draws it.
    pub style: String,
    pub relief_m: f64,
    pub albedo: [f64; 2],
    #[serde(default)]
    pub land_fraction: Option<f64>,
    /// Research is divided: the regimes a GM can choose between (first is the default).
    #[serde(default)]
    pub dial: Vec<String>,
    #[serde(default)]
    pub speculative: bool,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tide {
    pub q: f64,
    pub k2: f64,
    pub spin_hours: f64,
    pub moment: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SurfaceSettings {
    pub epoch_step_gyr: f64,
    pub bisect_steps: u32,
    pub q_crit_w_m2: f64,
    pub albedo_default: f64,
    pub greenhouse_k: f64,
    pub tide: Tide,
    pub atlas: atlas::AtlasSettings,
    pub plates: plates::PlateSettings,
    pub wear: wear::WearSettings,
    pub impacts: impacts::ImpactSettings,
    pub regime_features: regimes::RegimeSettings,
    pub cover: cover::CoverSettings,
    pub features: named::FeatureSettings,
    pub rules: Vec<Rule>,
    pub regimes: BTreeMap<String, RegimeDef>,
}

impl SurfaceSettings {
    /// The last pack in the stack that provides `surface` wins.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("surface"))
            .ok_or("no pack provides surface settings")?;
        let s: SurfaceSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("surface settings: {e}"))?;
        for r in &s.rules {
            if !s.regimes.contains_key(&r.regime) {
                return Err(format!("rule names unknown regime {}", r.regime));
            }
            for q in r.when.keys() {
                if !QUANTITIES.contains(&q.as_str()) {
                    return Err(format!("rule for {} reads unknown quantity {q}", r.regime));
                }
            }
        }
        for (id, d) in &s.regimes {
            for alt in &d.dial {
                if !s.regimes.contains_key(alt) {
                    return Err(format!("regime {id} dials to unknown regime {alt}"));
                }
            }
        }
        s.cover.check()?;
        if s.rules.last().is_none_or(|r| !r.when.is_empty()) {
            return Err("the last rule must have no conditions (the fallback)".into());
        }
        Ok(s)
    }
}

/// The quantities a rule may test, all evaluated at one date.
pub const QUANTITIES: [&str; 16] = [
    "deep_envelope",
    "molten",
    "t_surface_k",
    "radius_km",
    "pressure_bar",
    "water_fraction",
    "oceans",
    "heat_flow_w_m2",
    "q_ratio",
    "c_o",
    "mass_earth",
    "locked",
    "age_gyr",
    "tidal_w_m2",
    "round_ratio",
    "shell_activity",
];

// ---------------------------------------------------------------- bodies

/// Where the surface temperature comes from.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum Climate {
    /// A measured or authored value (fixtures, moons of giants).
    Fixed { t_surface_k: f64 },
    /// Equilibrium temperature under the ageing host star, plus greenhouse.
    Star {
        life: Box<LifeLine>,
        z: f64,
        a_au: f64,
        albedo: f64,
        greenhouse_k: f64,
        /// Myr after the star's birth that the world formed.
        formed_myr: f64,
        /// B1b: every other luminous body lighting this world (companion
        /// stars, a glowing host giant), each with its own life line.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        others: Vec<OtherLight>,
    },
}

/// Another body's light on a world.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OtherLight {
    pub life: Box<LifeLine>,
    /// Orbit-averaged distance, AU: √(a_companion² + a_world²) for a world and
    /// a companion on unrelated circular orbits (the mean of d²).
    pub d_au: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Orbit {
    pub a_au: f64,
    pub e: f64,
    /// Mass of what it orbits, solar masses.
    pub host_msun: f64,
}

/// One world, as the surface envelope sees it. Generated worlds come from the
/// formation pipeline ([`bodies_of`]); fixtures and imports fill it directly.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Body {
    pub address: Address,
    pub label: String,
    pub mass_earth: f64,
    /// Solid radius, Earth radii.
    pub radius_earth: f64,
    pub core_mass_fraction: f64,
    #[serde(default)]
    pub water_fraction: f64,
    /// Surface water, Earth oceans.
    #[serde(default)]
    pub oceans: f64,
    /// A gas envelope deep enough that there is no reachable solid surface.
    #[serde(default)]
    pub deep_envelope: bool,
    #[serde(default)]
    pub pressure_bar: f64,
    #[serde(default = "solar_c_o")]
    pub c_o: f64,
    #[serde(default = "one")]
    pub k40_rel: f64,
    #[serde(default = "one")]
    pub r_process_rel: f64,
    /// Age now, Gyr since the world formed.
    pub age_gyr: f64,
    /// Magma ocean solidified, Myr after the world formed.
    #[serde(default)]
    pub magma_end_myr: f64,
    pub climate: Climate,
    /// Tidal heat flow, W/m², when known (moons); otherwise computed from the orbit.
    #[serde(default)]
    pub tidal_w_m2: Option<f64>,
    #[serde(default)]
    pub orbit: Option<Orbit>,
    /// Spin-orbit locked; computed from the orbit when not given.
    #[serde(default)]
    pub locked: Option<bool>,
    /// Past tidal heating beyond today's (a resonance passage, the years
    /// after a capture): extra heat flow over spans of the world's life.
    /// Imports and packs give it; formation fills it when F makes captures.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tidal_history: Vec<TidalSpan>,
}

/// Extra tidal heat flow over a span of a world's life.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TidalSpan {
    /// Gyr since the world formed.
    pub from_gyr: f64,
    pub to_gyr: f64,
    pub w_m2: f64,
    /// Where the span comes from (a paper, an import, an authored story).
    #[serde(default)]
    pub source: String,
}

fn solar_c_o() -> f64 {
    0.55
}
fn one() -> f64 {
    1.0
}

// ---------------------------------------------------------------- output

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RegimeSpan {
    pub regime: String,
    /// Gyr since the world formed.
    pub from_gyr: f64,
    pub to_gyr: f64,
    /// Heat flow at the span's start and end, W/m² (radiogenic plus tidal).
    pub heat_flow_w_m2: [f64; 2],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RegimeEvent {
    pub t_gyr: f64,
    pub subject: Address,
    pub from: String,
    pub to: String,
    /// Why, in a sentence: the new rule's reason and the quantity that crossed.
    pub cause: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SurfaceEnvelope {
    pub address: Address,
    pub label: String,
    /// Outermost layer now (solid, ice-shell, ocean, lava, cloud-deck, small-body).
    pub kind: String,
    pub reference_radius_m: f64,
    pub gravity_m_s2: f64,
    pub regime_now: String,
    pub regime_label: String,
    /// Regimes a GM may choose between where research is divided (empty if not).
    pub dial: Vec<String>,
    pub speculative: bool,
    pub source: String,
    pub locked: bool,
    pub tidal_w_m2: f64,
    pub t_surface_now_k: f64,
    pub timeline: Vec<RegimeSpan>,
    pub events: Vec<RegimeEvent>,
}

// ---------------------------------------------------------------- physics

/// Despinning time, Gyr (Gladman et al. 1996 eq. 9 form).
pub fn lock_time_gyr(b: &Body, o: &Orbit, t: &Tide) -> f64 {
    let mp = b.mass_earth * M_EARTH_KG;
    let r = b.radius_earth * R_EARTH_M;
    let mh = o.host_msun * M_SUN_KG;
    let a = o.a_au * AU_M;
    let w = m::TAU / (t.spin_hours * 3600.0);
    let a3 = a * a * a;
    let r5 = r * r * r * r * r;
    let num = w * a3 * a3 * t.moment * mp * r * r * t.q;
    let den = 3.0 * G * mh * mh * t.k2 * r5;
    num / den / GYR_S
}

/// Eccentricity tidal heating, W/m² of surface (Peale and Cassen form,
/// synchronous rotation).
pub fn tidal_flux(b: &Body, o: &Orbit, t: &Tide) -> f64 {
    let r = b.radius_earth * R_EARTH_M;
    let mh = o.host_msun * M_SUN_KG;
    let a = o.a_au * AU_M;
    if a <= 0.0 || r <= 0.0 {
        return 0.0;
    }
    let n = m::sqrt(G * (mh + b.mass_earth * M_EARTH_KG) / (a * a * a));
    let a3 = a * a * a;
    let r5 = r * r * r * r * r;
    let p = 10.5 * (t.k2 / t.q) * G * mh * mh * r5 * n * o.e * o.e / (a3 * a3);
    p / (4.0 * m::PI * r * r)
}

fn start_of(b: &Body) -> WorldStart {
    WorldStart {
        address: b.address.clone(),
        mass_earth: b.mass_earth,
        core_mass_fraction: b.core_mass_fraction,
        water_fraction: b.water_fraction,
        solid_radius: b.radius_earth,
        radius: b.radius_earth,
        envelope: Envelope::None,
        envelope_fraction: 0.0,
        magma_ocean_end_myr: b.magma_end_myr,
        desiccated: false,
        surface_water_oceans: b.oceans,
        co2_bar: 0.0,
        early_insolation: 1.0,
        facts: vec![],
    }
}

struct Ctx<'a> {
    s: &'a SurfaceSettings,
    b: &'a Body,
    start: WorldStart,
    fits: Option<Fits>,
    tidal: f64,
    locked: bool,
}

impl Ctx<'_> {
    fn t_surface(&self, t_gyr: f64) -> f64 {
        match &self.b.climate {
            Climate::Fixed { t_surface_k } => *t_surface_k,
            Climate::Star {
                life,
                a_au,
                albedo,
                greenhouse_k,
                formed_myr,
                others,
                ..
            } => {
                let fits = self.fits.as_ref().expect("fits for a star climate");
                let age = formed_myr + t_gyr * 1000.0;
                let l = life.state_at(fits, age).luminosity;
                let mut flux = l / (a_au * a_au);
                for o in others {
                    flux += o.life.state_at(fits, age).luminosity / (o.d_au * o.d_au);
                }
                278.6 * m::pow(flux.max(0.0), 0.25) * m::pow((1.0 - albedo).max(0.0), 0.25)
                    + greenhouse_k
            }
        }
    }

    fn radiogenic(&self, t_gyr: f64) -> f64 {
        if self.b.deep_envelope {
            return 0.0;
        }
        pipeline::world_heat(&self.start, t_gyr, self.b.k40_rel, self.b.r_process_rel)
            .map(|h| h.heat_flow_w_m2)
            .unwrap_or(0.0)
    }

    fn quantities(&self, t_gyr: f64) -> BTreeMap<&'static str, f64> {
        let b = self.b;
        let tidal = self.tidal_at(t_gyr);
        let q = self.radiogenic(t_gyr) + tidal;
        let flag = |x: bool| if x { 1.0 } else { 0.0 };
        BTreeMap::from([
            ("deep_envelope", flag(b.deep_envelope)),
            ("molten", flag(t_gyr * 1000.0 < b.magma_end_myr)),
            ("t_surface_k", self.t_surface(t_gyr)),
            ("radius_km", b.radius_earth * R_EARTH_M / 1000.0),
            ("pressure_bar", b.pressure_bar),
            ("water_fraction", b.water_fraction),
            ("oceans", b.oceans),
            ("heat_flow_w_m2", q),
            ("q_ratio", q / self.s.q_crit_w_m2),
            ("c_o", b.c_o),
            ("mass_earth", b.mass_earth),
            ("locked", flag(self.locked)),
            ("age_gyr", t_gyr),
            ("tidal_w_m2", tidal),
            ("round_ratio", round_ratio(b)),
            ("shell_activity", shell_activity(self.t_surface(t_gyr), q)),
        ])
    }

    /// Tidal heat flow at a date: today's, plus any past span covering it.
    fn tidal_at(&self, t_gyr: f64) -> f64 {
        self.tidal
            + self
                .b
                .tidal_history
                .iter()
                .filter(|x| t_gyr >= x.from_gyr && t_gyr < x.to_gyr)
                .map(|x| x.w_m2)
                .sum::<f64>()
    }

    /// Index of the first rule that holds at `t`.
    fn rule_at(&self, t_gyr: f64) -> usize {
        let q = self.quantities(t_gyr);
        self.s
            .rules
            .iter()
            .position(|r| r.when.iter().all(|(k, range)| range.holds(q[k.as_str()])))
            .unwrap_or(self.s.rules.len() - 1)
    }
}

/// Strength of rock and of ice against self-gravity (Lineweaver and Norman
/// 2010): calibrated so a body of rock (3500 kg/m³) rounds at about 300 km
/// and one of ice (930 kg/m³) at about 200 km, from R² = 3σ / (2π G ρ²).
const SIGMA_ROCK_PA: f64 = 1.54e8;
const SIGMA_ICE_PA: f64 = 5.6e6;
const RHO_ROCK: f64 = 3500.0;
const RHO_ICE: f64 = 930.0;

/// Radius over the radius at which self-gravity pulls the body round
/// (gravity overcoming material strength, a real edge). The central
/// pressure (2π/3) G ρ² R² must exceed the yield strength; the strength of
/// an ice-rock mix is the geometric mean weighted by ice volume, since the
/// weak phase carries the strain once it is a large share. Below 1 the body
/// keeps the lumpy shape its accretion and impacts gave it.
pub fn round_ratio(b: &Body) -> f64 {
    let r = b.radius_earth * R_EARTH_M;
    if r <= 0.0 {
        return 0.0;
    }
    let rho = b.mass_earth * M_EARTH_KG / (4.0 / 3.0 * m::PI * r * r * r);
    let w = b.water_fraction.clamp(0.0, 1.0);
    let vi = w / RHO_ICE;
    let vr = (1.0 - w) / RHO_ROCK;
    let f = if vi + vr > 0.0 { vi / (vi + vr) } else { 0.0 };
    let sigma = m::exp(f * m::ln(SIGMA_ICE_PA) + (1.0 - f) * m::ln(SIGMA_ROCK_PA));
    let r_round = m::sqrt(3.0 * sigma / (2.0 * m::PI * G * rho * rho));
    r / r_round
}

/// Ice's melting point at low pressure, K.
const T_MELT_ICE_K: f64 = 273.15;
/// Ice's conductivity k = 651 / T W m⁻¹ K⁻¹ (Petrenko and Whitworth 1999).
const K_ICE_651: f64 = 651.0;
/// Rudolph et al. 2022: cracks cross Enceladus's shell where it is thinner
/// than about 15 km.
const CRACK_THROUGH_M: f64 = 15_000.0;

/// How active an ice shell's surface is, 0 to 1: whether tidal cracks can
/// cross it. The shell's conductive thickness from the heat flowing through
/// it, d = 651 ln(T_melt / T_surface) / q (k = 651/T), against the depth
/// cracks reach, about 15 km (Rudolph et al. 2022). A thin, warm-based
/// shell (Europa) breaks and renews its surface; a thick cold one
/// (Callisto) is a lid that keeps its craters, even if it convects
/// underneath. Smooth in (15 km / d)⁴, so the label read off it
/// (ice-shell at 0.5 and above) is not the physics.
pub fn shell_activity(t_surface_k: f64, q_w_m2: f64) -> f64 {
    if t_surface_k >= T_MELT_ICE_K || q_w_m2 <= 0.0 {
        return 0.0;
    }
    let d = K_ICE_651 * m::ln(T_MELT_ICE_K / t_surface_k.max(1.0)) / q_w_m2;
    let x = CRACK_THROUGH_M / d.max(1.0);
    let x4 = x * x * x * x;
    x4 / (1.0 + x4)
}

/// Classify one world over its life.
pub fn envelope(s: &SurfaceSettings, b: &Body) -> SurfaceEnvelope {
    let fits = match &b.climate {
        Climate::Star { z, .. } => Some(Fits::new(*z)),
        Climate::Fixed { .. } => None,
    };
    let tidal = match (&b.tidal_w_m2, &b.orbit) {
        (Some(x), _) => *x,
        (None, Some(o)) => tidal_flux(b, o, &s.tide),
        _ => 0.0,
    };
    let locked = match (&b.locked, &b.orbit) {
        (Some(x), _) => *x,
        (None, Some(o)) => lock_time_gyr(b, o, &s.tide) < b.age_gyr,
        _ => false,
    };
    let cx = Ctx {
        s,
        b,
        start: start_of(b),
        fits,
        tidal,
        locked,
    };
    // Sample dates: 0, the magma ocean's end, then every step to the present.
    let age = b.age_gyr.max(0.0);
    let mut dates = vec![0.0];
    let me = b.magma_end_myr / 1000.0;
    if me > 0.0 && me < age {
        dates.push(me);
    }
    let step = s.epoch_step_gyr.max(1e-3);
    let mut k = 1u32;
    loop {
        let t = step * k as f64;
        if t >= age {
            break;
        }
        if t > me {
            dates.push(t);
        }
        k += 1;
    }
    if age > 0.0 {
        dates.push(age);
    }
    let mut timeline: Vec<RegimeSpan> = vec![];
    let mut events = vec![];
    let mut cur = cx.rule_at(0.0);
    let mut start = 0.0;
    let mut prev = 0.0;
    for &t in dates.iter().skip(1) {
        let r = cx.rule_at(t);
        if r != cur {
            // Bisect for the crossing between prev and t.
            let (mut lo, mut hi) = (prev, t);
            if t != me {
                for _ in 0..s.bisect_steps {
                    let mid = 0.5 * (lo + hi);
                    if cx.rule_at(mid) == cur {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
            }
            let at = if t == me { me } else { hi };
            let new = cx.rule_at(at);
            timeline.push(span(&cx, &s.rules[cur].regime, start, at));
            events.push(RegimeEvent {
                t_gyr: at,
                subject: b.address.clone(),
                from: s.rules[cur].regime.clone(),
                to: s.rules[new].regime.clone(),
                cause: cause(&cx, cur, new, at),
            });
            cur = new;
            start = at;
        }
        prev = t;
    }
    timeline.push(span(&cx, &s.rules[cur].regime, start, age));
    let regime = s.rules[cur].regime.clone();
    let def = &s.regimes[&regime];
    let r = b.radius_earth * R_EARTH_M;
    SurfaceEnvelope {
        address: b.address.clone(),
        label: b.label.clone(),
        kind: def.kind.clone(),
        reference_radius_m: r,
        gravity_m_s2: if r > 0.0 {
            G * b.mass_earth * M_EARTH_KG / (r * r)
        } else {
            0.0
        },
        regime_label: def.label.clone(),
        dial: def.dial.clone(),
        speculative: def.speculative,
        source: def.source.clone(),
        regime_now: regime,
        locked,
        tidal_w_m2: tidal,
        t_surface_now_k: cx.t_surface(age),
        timeline,
        events,
    }
}

fn span(cx: &Ctx, regime: &str, from: f64, to: f64) -> RegimeSpan {
    let q = |t| cx.radiogenic(t) + cx.tidal_at(t);
    RegimeSpan {
        regime: regime.to_string(),
        from_gyr: from,
        to_gyr: to,
        heat_flow_w_m2: [q(from), q(to)],
    }
}

/// The new rule's reason, and which of the old rule's conditions stopped
/// holding (or the new rule's started), with the value at the crossing.
fn cause(cx: &Ctx, old: usize, new: usize, t: f64) -> String {
    let s = cx.s;
    let q = cx.quantities(t);
    let mut crossed: Vec<String> = vec![];
    for (k, range) in &s.rules[old].when {
        if !range.holds(q[k.as_str()]) {
            crossed.push(describe(k, q[k.as_str()]));
        }
    }
    if crossed.is_empty() {
        for k in s.rules[new].when.keys() {
            crossed.push(describe(k, q[k.as_str()]));
        }
    }
    let new_label = &s.regimes[&s.rules[new].regime].label;
    let mut text = format!("{new_label}: {}", s.rules[new].because);
    if !crossed.is_empty() {
        text.push_str(&format!(" ({})", crossed.join(", ")));
    }
    text.push('.');
    text
}

fn describe(k: &str, v: f64) -> String {
    match k {
        "molten" => "the magma ocean solidified".into(),
        "q_ratio" => format!("heat flow {:.3} of the critical value", v),
        "heat_flow_w_m2" => format!("heat flow {:.3} W/m²", v),
        "t_surface_k" => format!("surface {:.0} K", v),
        "age_gyr" => format!("age {:.2} Gyr", v),
        "round_ratio" => format!(
            "radius {:.2} times the size at which gravity pulls it round",
            v
        ),
        "shell_activity" => format!(
            "ice shell activity {:.2} (tidal cracks crossing the shell)",
            v
        ),
        "tidal_w_m2" => format!("tidal heat {:.3} W/m²", v),
        _ => format!("{} {}", k.replace('_', " "), fmt_num(v)),
    }
}

fn fmt_num(v: f64) -> String {
    if v != 0.0 && (v.abs() < 0.01 || v.abs() >= 1e5) {
        format!("{v:.2e}")
    } else {
        format!("{v:.2}")
    }
}

// ---------------------------------------------------------------- pipeline

/// The bodies of a formation run, in orbit order, for the surface envelope.
pub fn bodies_of(run: &SystemRun, s: &SurfaceSettings) -> Vec<Body> {
    let h = &run.history;
    let inh = &h.inheritance;
    let star = &h.stars[0];
    let now_myr = inh.age_gyr * 1000.0;
    run.worlds
        .iter()
        .zip(&run.protoplanets)
        .map(|(w, p)| {
            let volatiles = w.surface_water_oceans >= 0.05 || w.co2_bar >= 0.1;
            Body {
                address: w.address.clone(),
                label: w.address.to_string(),
                mass_earth: w.mass_earth,
                radius_earth: w.solid_radius,
                core_mass_fraction: w.core_mass_fraction,
                water_fraction: w.water_fraction,
                oceans: w.surface_water_oceans,
                deep_envelope: w.envelope == Envelope::Giant
                    || (w.envelope == Envelope::Kept && w.envelope_fraction > 1e-3),
                // Outgassed CO2 is not yet an atmosphere (it freezes out on cold
                // worlds, is drawn down on wet ones); 0 until an atmosphere stage.
                pressure_bar: 0.0,
                c_o: inh.c_o,
                k40_rel: inh.k40_rel,
                r_process_rel: inh.r_process_rel,
                age_gyr: ((now_myr - p.formed_myr) / 1000.0).max(0.0),
                magma_end_myr: (w.magma_ocean_end_myr - p.formed_myr).max(0.0),
                climate: Climate::Star {
                    life: Box::new(star.life.clone()),
                    z: inh.z,
                    a_au: p.a_au,
                    albedo: s.albedo_default,
                    greenhouse_k: if volatiles { s.greenhouse_k } else { 0.0 },
                    formed_myr: p.formed_myr,
                    others: h.stars[1..]
                        .iter()
                        .filter_map(|c| {
                            let a_c = c.orbit.as_ref()?.semi_major_axis_au;
                            Some(OtherLight {
                                life: Box::new(c.life.clone()),
                                d_au: m::sqrt(a_c * a_c + p.a_au * p.a_au),
                            })
                        })
                        .collect(),
                },
                tidal_w_m2: None,
                orbit: Some(Orbit {
                    a_au: p.a_au,
                    e: p.e,
                    host_msun: star.mass,
                }),
                locked: None,
                tidal_history: vec![],
            }
        })
        .collect()
}

/// Surface envelopes for every world of a run.
pub fn envelopes_of(run: &SystemRun, s: &SurfaceSettings) -> Vec<SurfaceEnvelope> {
    bodies_of(run, s).iter().map(|b| envelope(s, b)).collect()
}

#[cfg(test)]
mod atlas_tests;
#[cfg(test)]
mod cover_tests;
#[cfg(test)]
mod detail_tests;
#[cfg(test)]
mod impacts_tests;
#[cfg(test)]
mod plates_tests;
#[cfg(test)]
mod regimes_tests;
#[cfg(test)]
mod survey_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wear_tests;
