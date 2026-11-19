//! The facts a weather block reads. Weather is keyed on **conditions**, never
//! on what kind of object a body is: the same rules run on a planet, a moon,
//! a brown dwarf or anything else with an atmosphere, once the stars and
//! planets become one body object (physics track).

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Address;
use std::collections::BTreeMap;

const R_GAS: f64 = 8.314462618;
const R_EARTH_M: f64 = 6.371e6;
const G: f64 = 6.6743e-11;
const M_EARTH_KG: f64 = 5.9722e24;
/// Earth's air column, kg/m² (1.01325 bar over 9.80665 m/s²).
pub const EARTH_COLUMN_KG_M2: f64 = 101_325.0 / 9.80665;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Conditions {
    pub address: Address,
    pub label: String,
    pub radius_m: f64,
    pub gravity_m_s2: f64,
    /// A surface the air sits on (false: the air goes down until it is no
    /// longer air, as on a giant; weather is then read at the 1 bar level).
    pub solid_surface: bool,
    /// Sidereal rotation period, hours (always positive).
    pub rotation_hours: f64,
    /// Keeps one face to the star that lights it.
    #[serde(default)]
    pub star_locked: bool,
    #[serde(default)]
    pub obliquity_deg: f64,
    /// Orbital period around the star, days.
    pub year_days: f64,
    #[serde(default)]
    pub eccentricity: f64,
    /// Mean stellar flux at the orbit, W/m².
    pub flux_w_m2: f64,
    /// Bond albedo.
    pub albedo: f64,
    /// Mean temperature at the surface (or at 1 bar), K.
    pub t_surface_k: f64,
    /// At the surface (or the reference 1 bar on a giant).
    pub pressure_bar: f64,
    pub mean_molar_kg: f64,
    /// Volume mixing ratios of the air's gases, by formula (`n2`, `co2`, ...).
    #[serde(default)]
    pub gases: BTreeMap<String, f64>,
    /// The surface liquid (`water`, `methane`, ...) when there is one.
    #[serde(default)]
    pub liquid: Option<String>,
    /// Fraction of the surface under that liquid.
    #[serde(default)]
    pub ocean_fraction: f64,
    /// Heat from inside, W/m² (giants radiate several times what they absorb).
    #[serde(default)]
    pub internal_flux_w_m2: f64,
}

impl Conditions {
    /// Spin rate, rad/s.
    pub fn omega(&self) -> f64 {
        2.0 * m::PI / (self.rotation_hours * 3600.0)
    }

    /// Length of the day under the star, hours: infinite when star-locked.
    pub fn solar_day_hours(&self) -> f64 {
        if self.star_locked {
            return f64::INFINITY;
        }
        let year_h = self.year_days * 24.0;
        let f = 1.0 / self.rotation_hours - 1.0 / year_h;
        if f.abs() < 1e-12 {
            f64::INFINITY
        } else {
            (1.0 / f).abs()
        }
    }

    /// Pressure scale height at the surface, m.
    pub fn scale_height_m(&self) -> f64 {
        R_GAS * self.t_surface_k / (self.mean_molar_kg * self.gravity_m_s2)
    }

    /// Mass of air above each square metre of ground, kg/m².
    pub fn column_kg_m2(&self) -> f64 {
        self.pressure_bar * 1.0e5 / self.gravity_m_s2
    }

    /// Sunlight absorbed, averaged over the sphere, W/m².
    pub fn absorbed_w_m2(&self) -> f64 {
        self.flux_w_m2 * (1.0 - self.albedo) / 4.0
    }

    /// Effective (emission) temperature from sunlight and internal heat, K.
    pub fn t_eff_k(&self) -> f64 {
        let s = 5.670374419e-8;
        m::pow((self.absorbed_w_m2() + self.internal_flux_w_m2) / s, 0.25)
    }

    pub fn land_fraction(&self) -> f64 {
        if self.solid_surface {
            (1.0 - self.ocean_fraction).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Facts beyond the surface envelope that weather needs. Fixtures and the
/// chart give them; generated worlds take the stated defaults until the
/// physics track publishes spin, tilt and air for every body.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Extra {
    pub rotation_hours: Option<f64>,
    pub obliquity_deg: Option<f64>,
    pub year_days: Option<f64>,
    pub flux_w_m2: Option<f64>,
    pub albedo: Option<f64>,
    pub mean_molar_kg: Option<f64>,
    pub gases: Option<BTreeMap<String, f64>>,
    pub liquid: Option<String>,
    pub ocean_fraction: Option<f64>,
    pub internal_flux_w_m2: Option<f64>,
}

/// Conditions from the surface track's body and envelope, for worlds that
/// come through the formation pipeline or the V3 importer. Missing facts use
/// the defaults in the weather pack, and the result says which were assumed.
pub fn from_surface(
    b: &sse_surface::Body,
    env: &sse_surface::SurfaceEnvelope,
    x: &Extra,
    d: &crate::settings::Defaults,
) -> (Conditions, Vec<String>) {
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
        365.256_363 * m::sqrt(a_au * a_au * a_au / host)
    });
    let rotation_hours = x.rotation_hours.unwrap_or_else(|| {
        assumed.push(format!(
            "spin {} h (not yet published for this body)",
            d.rotation_hours
        ));
        d.rotation_hours
    });
    let flux = x.flux_w_m2.unwrap_or_else(|| {
        assumed.push("sunlight from a sun-like star at the orbit".into());
        1361.0 / (a_au * a_au)
    });
    let gases = x.gases.clone().unwrap_or_else(|| {
        assumed.push("air of the default mix".into());
        d.gases.clone()
    });
    let liquid = x.liquid.clone().or_else(|| {
        (b.oceans > 0.0 && env.t_surface_now_k > 273.15 && env.t_surface_now_k < 373.0)
            .then(|| "water".to_string())
    });
    // Earth's 1 ocean covers 71 % of its surface; a fuller ocean drowns more.
    let ocean_fraction = x.ocean_fraction.unwrap_or(if liquid.is_some() {
        (0.71 * m::pow(b.oceans.max(0.0), 0.25)).min(1.0)
    } else {
        0.0
    });
    let c = Conditions {
        address: b.address.clone(),
        label: b.label.clone(),
        radius_m: r,
        gravity_m_s2: g,
        solid_surface: !b.deep_envelope,
        rotation_hours,
        star_locked: env.locked
            && b.address
                .segments()
                .last()
                .is_some_and(|s| s.kind() == "body"),
        obliquity_deg: x.obliquity_deg.unwrap_or(d.obliquity_deg),
        year_days,
        eccentricity: b.orbit.as_ref().map(|o| o.e).unwrap_or(0.0),
        flux_w_m2: flux,
        albedo: x.albedo.unwrap_or(0.3),
        t_surface_k: env.t_surface_now_k,
        pressure_bar: if b.deep_envelope { 1.0 } else { b.pressure_bar },
        mean_molar_kg: x
            .mean_molar_kg
            .unwrap_or(if b.deep_envelope { 0.0023 } else { 0.029 }),
        gases,
        liquid,
        ocean_fraction,
        internal_flux_w_m2: x.internal_flux_w_m2.unwrap_or(0.0),
    };
    (c, assumed)
}
