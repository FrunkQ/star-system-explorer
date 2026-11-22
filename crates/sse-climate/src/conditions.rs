//! The facts a climate reads. Climate is keyed on **conditions**, never on
//! what kind of object a body is: the same energy balance runs on a planet, a
//! moon, a brown dwarf or anything else with an atmosphere.
//!
//! Spin is **signed**: a negative `rotation_hours` spins backwards against
//! its orbit (Venus, Uranus). The magnitude sets every rate; the sign sets
//! the sense of the Coriolis turn, so winds, storm spin and currents flip.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Address;
use std::collections::BTreeMap;

pub const R_GAS: f64 = 8.314462618;
/// Earth's air column, kg/m² (1.01325 bar over 9.80665 m/s²).
pub const EARTH_COLUMN_KG_M2: f64 = 101_325.0 / 9.80665;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Conditions {
    pub address: Address,
    pub label: String,
    pub radius_m: f64,
    pub gravity_m_s2: f64,
    /// A surface the air sits on (false: the air goes down until it is no
    /// longer air, as on a giant; climate is then read at the 1 bar level).
    pub solid_surface: bool,
    /// Sidereal rotation period, hours. Negative: spins backwards (retrograde).
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
    /// Where the closest approach falls in the seasons: the planet's solar
    /// longitude at perihelion, degrees from the northern spring equinox.
    #[serde(default)]
    pub perihelion_lon_deg: f64,
    /// Stellar flux at the orbit's semi-major axis, W/m² (the flux at each
    /// moment is this times (a/r)²).
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
    /// Ice of the air's gases lying on the ground at the start, by gas,
    /// kg per m² of the whole surface (W8's exchange moves it between ground
    /// and air). Empty: the air is all there is (Mars's seasonal caps come
    /// out of its air).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ground_ice_kg_m2: BTreeMap<String, f64>,
}

impl Conditions {
    /// Spin rate, rad/s, signed: negative for a retrograde spin.
    pub fn omega(&self) -> f64 {
        2.0 * m::PI / (self.rotation_hours * 3600.0)
    }

    /// Spin rate, rad/s, always positive (for rates and lengths).
    pub fn spin_rate(&self) -> f64 {
        self.omega().abs()
    }

    /// Spins backwards against its orbit.
    pub fn retrograde(&self) -> bool {
        self.rotation_hours < 0.0
    }

    /// Length of the day under the star, hours: infinite when star-locked.
    /// A retrograde spin adds to the orbit's turn (Venus: 117 days).
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

/// Facts beyond a body's surface envelope that the climate needs. Fixtures,
/// the chart, the formation run and V3 imports give them; whatever is still
/// missing takes the pack's stated defaults, and the reader is told which.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Extra {
    /// Signed: negative spins backwards.
    #[serde(default)]
    pub rotation_hours: Option<f64>,
    #[serde(default)]
    pub obliquity_deg: Option<f64>,
    #[serde(default)]
    pub year_days: Option<f64>,
    #[serde(default)]
    pub perihelion_lon_deg: Option<f64>,
    #[serde(default)]
    pub flux_w_m2: Option<f64>,
    #[serde(default)]
    pub albedo: Option<f64>,
    #[serde(default)]
    pub mean_molar_kg: Option<f64>,
    #[serde(default)]
    pub gases: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    pub liquid: Option<String>,
    #[serde(default)]
    pub ocean_fraction: Option<f64>,
    #[serde(default)]
    pub internal_flux_w_m2: Option<f64>,
}
