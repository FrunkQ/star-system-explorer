//! W5: what other parts of SSE4 read from the weather (design sections 5 and
//! 7): the column a descending craft passes through with its hazards, the
//! cues the sound engine plays, and the GM's controls (pin a storm, order
//! clear skies or calm for a scene, trigger a storm), all as plain data.
//!
//! GM controls are [`Pin`]s with `evidence: "authored"`: a save keeps them
//! like any recorded system, every block must honour them, and the GM view
//! marks them as authored.

use crate::conditions::Conditions;
use crate::contract::*;
use crate::formations::Terrain;
use crate::rules::RuleBlock;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// One height band on the way down.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ColumnLevel {
    pub layer: String,
    /// Height above the ground, km.
    pub from_km: f64,
    pub to_km: f64,
    pub cloud_cover: f64,
    pub cloud_type: String,
    pub condensate: String,
    pub wind_km_h: f64,
    pub turbulence: f64,
    /// Supercooled water cloud: ice builds on the craft.
    pub icing: bool,
    pub lightning: bool,
    /// Visibility inside this band, km.
    pub visibility_km: f64,
    /// 0 none, 1 caution, 2 warning, 3 danger.
    pub hazard: u8,
    /// Plain telemetry line: "4 to 9 km: cumulonimbus of water, wind 90 km/h, severe turbulence, lightning".
    pub line: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Column {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub day: f64,
    /// Top down, as a descent meets them.
    pub levels: Vec<ColumnLevel>,
    /// At the ground.
    pub report: Report,
    /// Highest hazard on the way down.
    pub hazard: u8,
}

/// What the sound engine plays (sound design N5 reads these).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SoundCues {
    pub wind_m_s: f64,
    /// How much the wind gusts, 0 to 1.
    pub gustiness: f64,
    pub precip_kind: String,
    pub precip_mm_day: f64,
    /// Thunder heard per hour (strikes within about 15 km).
    pub thunder_per_hour: f64,
    /// The air that carries the sound: pressure and main gas.
    pub pressure_bar: f64,
    pub main_gas: String,
    /// Nothing to carry sound.
    pub silent: bool,
}

const HEARD_KM: f64 = 15.0;

pub(crate) fn level_bounds(c: &Conditions, s: &crate::WeatherSettings) -> [(f64, f64); 3] {
    let h = c.scale_height_m() / 1000.0;
    let top = s.climate.circulation.tropopause_scale_heights * h;
    [
        (0.0, 0.24 * h),
        (0.24 * h, 0.83 * h),
        (0.83 * h, top.max(0.9 * h)),
    ]
}

/// The column over a place on a date, with hazards, for descent scenes.
pub fn column(
    b: &RuleBlock,
    c: &Conditions,
    date: Date,
    lat: f64,
    lon: f64,
    pins: &[Pin],
    terrain: Option<&dyn Terrain>,
) -> Column {
    let p = b.prepare(c, date);
    let systems = b.systems_at(c, &p, date, pins);
    let cell = b.cell_at(c, &p, date, &systems, terrain, lat, lon);
    let report =
        crate::formations::report(&b.formations, &cell, c, &systems, date.day, b.provenance());
    let bounds = level_bounds(c, &b.settings);
    let cp = 3.5 * sse_core::sheet::R_GAS / c.mean_molar_kg;
    let lapse = c.gravity_m_s2 / cp;
    let mut levels = vec![];
    if p.circ.regime != "airless" {
        for (i, l) in cell.layers.iter().enumerate().rev() {
            let (lo, hi) = bounds[i];
            let mid_t = c.t_surface_k - lapse * 1000.0 * 0.5 * (lo + hi);
            let wind = m::sqrt(l.wind_m_s[0] * l.wind_m_s[0] + l.wind_m_s[1] * l.wind_m_s[1]);
            let icing =
                l.condensate == "water" && l.cloud_cover > 0.3 && (248.0..273.15).contains(&mid_t);
            let lightning = l.cloud_type == "cumulonimbus" && cell.lightning_km2_yr > 1.0;
            let vis = if l.cloud_cover > 0.05 {
                (b.formations.clear_visibility_km / (1.0 + l.optical_depth * l.cloud_cover * 5.0))
                    .max(0.05)
            } else {
                b.formations.clear_visibility_km
            };
            let turb = l.turbulence.max(if l.cloud_type == "cumulonimbus" {
                0.6
            } else {
                0.0
            });
            let mut hazard = 0u8;
            let mut notes = vec![];
            if turb > 0.7 {
                hazard = 3;
                notes.push("severe turbulence".to_string());
            } else if turb > 0.3 {
                hazard = hazard.max(1);
                notes.push("turbulence".into());
            }
            if icing {
                hazard = hazard.max(2);
                notes.push("icing".into());
            }
            if lightning {
                hazard = hazard.max(2);
                notes.push("lightning".into());
            }
            if wind > 30.0 {
                hazard = hazard.max(if wind > 50.0 { 2 } else { 1 });
            }
            if l.cloud_type == "deck" && l.condensate.contains("acid") {
                hazard = hazard.max(2);
                notes.push("corrosive cloud".into());
            }
            let sky = if l.cloud_cover < 0.05 {
                "clear".to_string()
            } else {
                format!("{} of {}", l.cloud_type, l.condensate)
            };
            let mut line = format!(
                "{:.0} to {:.0} km: {sky}, wind {:.0} km/h",
                lo,
                hi,
                wind * 3.6
            );
            for n in &notes {
                line.push_str(", ");
                line.push_str(n);
            }
            levels.push(ColumnLevel {
                layer: LAYERS[i].into(),
                from_km: lo,
                to_km: hi,
                cloud_cover: l.cloud_cover,
                cloud_type: l.cloud_type.clone(),
                condensate: l.condensate.clone(),
                wind_km_h: wind * 3.6,
                turbulence: turb,
                icing,
                lightning,
                visibility_km: vis,
                hazard,
                line,
            });
        }
    }
    for a in &cell.aerosols {
        if a.kind == "dust" || a.kind == "ash" {
            if let Some(l) = levels.last_mut() {
                l.hazard = l.hazard.max(if a.optical_depth > 1.0 { 2 } else { 1 });
                l.line.push_str(&format!(", {} in the air", a.kind));
            }
        }
    }
    let hazard = levels.iter().map(|l| l.hazard).max().unwrap_or(0);
    Column {
        lat_deg: lat,
        lon_deg: lon,
        day: date.day,
        levels,
        report,
        hazard,
    }
}

/// Sound cues at a place on a date.
pub fn sound(
    b: &RuleBlock,
    c: &Conditions,
    date: Date,
    lat: f64,
    lon: f64,
    pins: &[Pin],
    terrain: Option<&dyn Terrain>,
) -> SoundCues {
    let p = b.prepare(c, date);
    let systems = b.systems_at(c, &p, date, pins);
    let cell = b.cell_at(c, &p, date, &systems, terrain, lat, lon);
    let w = cell.layers[0].wind_m_s;
    let wind = m::sqrt(w[0] * w[0] + w[1] * w[1]);
    let main_gas = c
        .gases
        .iter()
        .fold((String::new(), 0.0), |acc, (g, x)| {
            if *x > acc.1 {
                (g.clone(), *x)
            } else {
                acc
            }
        })
        .0;
    let silent = p.circ.regime == "airless";
    SoundCues {
        wind_m_s: if silent { 0.0 } else { wind },
        gustiness: cell.layers[0].turbulence,
        precip_kind: cell.precip_kind.clone(),
        precip_mm_day: cell.precip_mm_day,
        thunder_per_hour: cell.lightning_km2_yr * m::PI * HEARD_KM * HEARD_KM / 8766.0,
        pressure_bar: c.pressure_bar,
        main_gas,
        silent,
    }
}

#[allow(clippy::too_many_arguments)]
fn order(
    kind: &str,
    id: &str,
    lat: f64,
    lon: f64,
    radius_km: f64,
    from_day: f64,
    to_day: f64,
    why: &str,
) -> Pin {
    Pin {
        id: id.into(),
        name: String::new(),
        system: WeatherSystem {
            id: id.into(),
            kind: kind.into(),
            lat_deg: lat,
            lon_deg: lon,
            track: vec![[from_day, lat, lon]],
            size_km: 2.0 * radius_km,
            strength: 0.0,
            born: from_day,
            ends: to_day,
            count: 0,
            why: why.into(),
            evidence: "authored".into(),
        },
    }
}

/// GM: clear skies over a place for a scene.
pub fn order_clear(
    id: &str,
    lat: f64,
    lon: f64,
    radius_km: f64,
    from_day: f64,
    to_day: f64,
) -> Pin {
    order(
        "order-clear",
        id,
        lat,
        lon,
        radius_km,
        from_day,
        to_day,
        "Clear skies ordered by the GM for a scene.",
    )
}

/// GM: still air over a place for a scene.
pub fn order_calm(id: &str, lat: f64, lon: f64, radius_km: f64, from_day: f64, to_day: f64) -> Pin {
    order(
        "order-calm",
        id,
        lat,
        lon,
        radius_km,
        from_day,
        to_day,
        "Calm ordered by the GM for a scene.",
    )
}

/// GM: a storm where and when the story needs one. It drifts with the
/// local steering wind like a drawn storm of its kind.
#[allow(clippy::too_many_arguments)]
pub fn trigger_storm(
    b: &RuleBlock,
    c: &Conditions,
    id: &str,
    kind: &str,
    lat: f64,
    lon: f64,
    from_day: f64,
    days: f64,
) -> Pin {
    let p = b.prepare(c, Date { day: from_day });
    let u = crate::circulation::zonal_wind(&b.settings, &p.circ, lat) * 2.5;
    let (size, strength) = if kind == "tropical-cyclone" {
        (
            b.formations.tropical.size_km,
            b.formations.tropical.peak_wind_m_s,
        )
    } else {
        (2000.0, b.formations.cyclones.peak_wind_m_s)
    };
    let mut track = vec![];
    let mut t = from_day;
    let mut lo = lon;
    while t <= from_day + days + 1e-9 {
        track.push([t, lat, lo]);
        let cl = m::cos(lat * m::PI / 180.0).max(0.05);
        lo += u * 21_600.0 / (c.radius_m * cl) * 180.0 / m::PI;
        lo = m::fmod(lo + 540.0, 360.0) - 180.0;
        t += 0.25;
    }
    Pin {
        id: id.into(),
        name: String::new(),
        system: WeatherSystem {
            id: id.into(),
            kind: kind.into(),
            lat_deg: lat,
            lon_deg: lon,
            track,
            size_km: size,
            strength,
            born: from_day,
            ends: from_day + days,
            count: 0,
            why: "A storm triggered by the GM.".into(),
            evidence: "authored".into(),
        },
    }
}
