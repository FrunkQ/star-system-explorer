//! The one climate on the ground (W7a). A body's climate conditions are
//! built here, once, and every surface stage that needs weather (S3 wear's
//! rain and glaciers, S7 cover's temperature, wetness and winds) reads the
//! same [`WorldClimate`]. The weather blocks build their conditions with the
//! same function, so the rain that cuts the valleys is the rain on the
//! weather map.

use crate::cover::Air;
use crate::{Body, Climate as Source, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_climate::{Conditions, Extra};
use sse_core::math as m;

const R_EARTH_M: f64 = 6.371e6;
const G: f64 = 6.6743e-11;
const M_EARTH_KG: f64 = 5.9722e24;

/// Climate conditions for a body: the facts it has, the inputs given (spin,
/// tilt and so on) and, for anything still missing, the climate pack's
/// defaults, listed in the second value.
pub fn conditions(
    s: &SurfaceSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    x: &Extra,
) -> (Conditions, Vec<String>) {
    let d = &s.climate.defaults;
    let mut assumed = vec![];
    let r = b.radius_earth * R_EARTH_M;
    let g = G * b.mass_earth * M_EARTH_KG / (r * r);
    let (a_au, host) = b
        .orbit
        .as_ref()
        .map(|o| (o.a_au, o.host_msun))
        .unwrap_or((1.0, 1.0));
    let year_days = x.year_days.unwrap_or_else(|| {
        assumed.push("year from the orbit (Kepler)".into());
        365.256_363 * m::sqrt(a_au * a_au * a_au / host.max(1e-12))
    });
    let locked = env.locked;
    let rotation_hours = x.rotation_hours.unwrap_or_else(|| {
        if locked {
            year_days * 24.0
        } else {
            assumed.push(format!(
                "spin {} h (not yet published for this body)",
                d.rotation_hours
            ));
            d.rotation_hours
        }
    });
    let obliquity_deg = x.obliquity_deg.unwrap_or_else(|| {
        assumed.push(format!("tilt {}° (not yet published)", d.obliquity_deg));
        d.obliquity_deg
    });
    let flux = x.flux_w_m2.unwrap_or_else(|| {
        assumed.push("sunlight from a sun-like star at the orbit".into());
        1361.0 / (a_au * a_au)
    });
    let gases = x.gases.clone().unwrap_or_else(|| {
        assumed.push("air of the default mix".into());
        d.gases.clone()
    });
    let t = env.t_surface_now_k;
    let liquid = x
        .liquid
        .clone()
        .or_else(|| (b.oceans > 0.0 && t > 273.15 && t < 373.0).then(|| "water".to_string()));
    // Earth's 1 ocean covers 71 % of its surface; a fuller ocean drowns more.
    let ocean_fraction = x.ocean_fraction.unwrap_or(if liquid.is_some() {
        (0.71 * m::pow(b.oceans.max(0.0), 0.25)).min(1.0)
    } else {
        0.0
    });
    let air = Air::guess(&s.cover, env, b.pressure_bar, b.oceans > 0.0);
    let albedo = x.albedo.unwrap_or(match &b.climate {
        Source::Star { albedo, .. } => *albedo,
        Source::Fixed { .. } => s.albedo_default,
    });
    let c = Conditions {
        address: b.address.clone(),
        label: b.label.clone(),
        radius_m: r,
        gravity_m_s2: g,
        solid_surface: !b.deep_envelope,
        rotation_hours,
        star_locked: locked
            && b.address
                .segments()
                .last()
                .is_some_and(|s| s.kind() == "body"),
        obliquity_deg,
        year_days,
        eccentricity: b.orbit.as_ref().map(|o| o.e).unwrap_or(0.0),
        flux_w_m2: flux,
        albedo,
        t_surface_k: t,
        pressure_bar: if b.deep_envelope { 1.0 } else { b.pressure_bar },
        mean_molar_kg: x.mean_molar_kg.unwrap_or(air.molar_kg),
        gases,
        liquid,
        ocean_fraction,
        internal_flux_w_m2: x.internal_flux_w_m2.unwrap_or(0.0),
    };
    (c, assumed)
}

/// The one climate a surveyed world's ground reads: the Orbit-level energy
/// balance plus the lapse rate of its air, so a temperature can be read at
/// any height.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldClimate {
    pub climate: sse_climate::Climate,
    /// Cooling with height above the datum, K per km.
    pub lapse_k_km: f64,
    /// Facts the climate assumed because the body does not publish them.
    pub assumed: Vec<String>,
}

impl WorldClimate {
    /// Mean temperature at latitude (radians) and height above the datum (m).
    pub fn t_at(&self, lat: f64, h_m: f64) -> f64 {
        self.climate.t_at(lat * 180.0 / m::PI) - self.lapse_k_km * h_m.max(0.0) / 1000.0
    }

    /// Rain and snow reaching the ground at a latitude (radians), m a year.
    pub fn precip_m_yr(&self, lat: f64) -> f64 {
        self.climate.precip_at(lat * 180.0 / m::PI)
    }

    /// Rain relative to the wettest band, 0 to 1, at a latitude (radians).
    pub fn wetness(&self, lat: f64) -> f64 {
        self.climate.wetness_at(lat * 180.0 / m::PI)
    }

    /// Whether the prevailing surface wind blows from the east (radians).
    pub fn easterly(&self, lat: f64) -> bool {
        self.climate.circulation.easterly(lat * 180.0 / m::PI)
    }
}

/// The climate of `b` at `epoch_myr`, with the given extra facts.
pub fn world_climate(
    s: &SurfaceSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    x: &Extra,
    epoch_myr: f64,
) -> WorldClimate {
    let (c, assumed) = conditions(s, b, env, x);
    let air = Air::guess(&s.cover, env, b.pressure_bar, b.oceans > 0.0);
    WorldClimate {
        climate: sse_climate::orbit(&s.climate, &c, epoch_myr),
        lapse_k_km: air.lapse_k_km(&s.cover, env.gravity_m_s2.max(1e-6)),
        assumed,
    }
}

/// The climate of the epochs when the world had surface water, which is the
/// rain that wear reads. A world wet now with the same water is simply the
/// present climate. A world wet only in the past (early Mars) gets its water
/// back: liquid water at the history's volume and pressure, and a mean
/// temperature of at least `wet_floor_k`, because a sea that rains needs one
/// (a registered fudge until the atmosphere history publishes past
/// temperatures).
pub fn wet_climate(
    s: &SurfaceSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    water: &crate::wear::WaterHistory,
    x: &Extra,
    epoch_myr: f64,
) -> WorldClimate {
    let wet_now =
        b.oceans > 0.0 && b.oceans == water.oceans && b.pressure_bar == water.pressure_bar;
    if wet_now || water.oceans <= 0.0 {
        return world_climate(s, b, env, x, epoch_myr);
    }
    let mut b2 = b.clone();
    b2.oceans = water.oceans;
    b2.pressure_bar = water.pressure_bar;
    let mut e2 = env.clone();
    e2.t_surface_now_k = env.t_surface_now_k.max(s.wear.wet_floor_k);
    let mut x2 = x.clone();
    if x2.liquid.is_none() {
        x2.liquid = Some("water".into());
    }
    let mut w = world_climate(s, &b2, &e2, &x2, epoch_myr);
    w.assumed.push(format!(
        "while wet: {} oceans under {} bar, at least {} K",
        water.oceans, water.pressure_bar, s.wear.wet_floor_k
    ));
    w
}
