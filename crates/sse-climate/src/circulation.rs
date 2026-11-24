//! The circulation every reader shares: how far the Hadley cell reaches, how
//! many cells a hemisphere holds, where air rises and sinks, where the storm
//! tracks run and which way the wind blows near the ground. Weather draws
//! its belts and storms from this; cover reads its prevailing winds from it.
//! No other crate computes any of it.
//!
//! - Hadley cell edge from Held and Hou (1980): the cell reaches
//!   φ_H = sqrt(5/3 · g H Δh / (Ω² a²)) radians, so it shrinks as spin rises.
//! - Cells per hemisphere from the Rhines scale (Rhines 1975): the arrest
//!   wavenumber is k_β = (β / 2U)^½, β = 2Ω / a, jets sit π / k_β apart, so a
//!   hemisphere holds ½ sqrt(a Ω / U).
//! - The wind scale U is the thermal wind (W7b, [`crate::strengths`]), so the
//!   cell count follows each world's own forcing, not one tuned speed.
//! - The layout reads only these numbers; the regime is a label read off the
//!   strengths and facts afterwards, the first pack rule whose conditions hold.
//!
//! Lengths and rates read the spin's magnitude; its sign turns the winds.

use crate::conditions::Conditions;
use crate::settings::ClimateSettings;
use crate::strengths::{strengths, Strengths};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Where the cells, rising and sinking air and storm tracks sit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Circulation {
    pub regime: String,
    pub regime_label: String,
    pub because: String,
    /// Overturning cells per hemisphere, as laid out.
    pub cells: u32,
    /// Cells per hemisphere from the Rhines scale, before layout.
    pub cells_raw: f64,
    /// Hadley edge before layout, degrees (at most 90).
    pub hadley_raw_deg: f64,
    /// Hadley edge as laid out.
    pub hadley_edge_deg: f64,
    /// Cell boundaries, equator to pole (first 0, last 90).
    pub edges_deg: Vec<f64>,
    /// Latitudes of air sinking (dry) and rising (wet) at the cell edges.
    pub sinking_deg: Vec<f64>,
    pub rising_deg: Vec<f64>,
    /// Centres of the storm tracks (one per eddy-driven cell).
    pub storm_tracks_deg: Vec<f64>,
    pub solar_day_hours: f64,
    /// +1 for a forward spin, −1 for a backward one: the sense of every wind.
    pub spin_sign: f64,
    /// Wind scale used, m/s.
    pub wind_m_s: f64,
    /// Tropopause height used, m.
    pub tropopause_m: f64,
    /// The process strengths (W7b) the layout and winds read.
    pub strengths: Strengths,
    /// Every step, in words, for "Why?".
    pub why: Vec<String>,
}

fn deg(x: f64) -> f64 {
    x * 180.0 / m::PI
}

/// The regime and cell layout.
pub fn circulation(cs: &ClimateSettings, c: &Conditions) -> Circulation {
    let cc = &cs.circulation;
    let omega = c.spin_rate();
    let a = c.radius_m;
    let h = cc.tropopause_scale_heights * c.scale_height_m();
    let phi_h = m::sqrt(5.0 / 3.0 * c.gravity_m_s2 * h * cc.delta_h / (omega * omega * a * a));
    let hadley_raw = deg(phi_h).min(90.0);
    // Rhines: k_β = (β / 2U)^½ with β = 2Ω/a; one jet, or one cell, is
    // π / k_β wide (eastward jets sit 2π / k_β apart), so a quarter
    // circumference (π a / 2) holds ½ sqrt(a Ω / U) cells. U is the eddy
    // speed, not the jet's.
    let st = strengths(cs, c);
    let wind = st.eddy_wind_m_s.max(1e-6);
    let n_raw = 0.5 * m::sqrt(a * omega / wind);
    let cells = (m::round(n_raw) as u32).clamp(1, 20);
    let day = c.solar_day_hours();
    let q = |k: &str| -> f64 {
        match k {
            "pressure_bar" => c.pressure_bar,
            "solid" => c.solid_surface as u8 as f64,
            "star_locked" => c.star_locked as u8 as f64,
            "hadley_edge_deg" => hadley_raw,
            "cells" => cells as f64,
            "solar_day_hours" => day,
            "radiative_days" => st.radiative_s / sse_core::sheet::DAY,
            "super_rotation" => st.super_rotation,
            "baroclinic" => st.baroclinic,
            "day_night" => st.day_night,
            "collapse" => st.collapse,
            "collisional" => st.collisional,
            "thermal_rossby" => st.thermal_rossby,
            _ => f64::NAN,
        }
    };
    let rule = cc
        .rules
        .iter()
        .find(|r| r.when.iter().all(|(k, rg)| rg.holds(q(k))))
        .expect("fallback rule");
    let mut why = vec![
        format!(
            "Hadley cell edge {hadley_raw:.0}° (Held and Hou 1980: g {:.2} m/s², tropopause {:.0} km, spin {:.1} h{}, radius {:.0} km).",
            c.gravity_m_s2,
            h / 1000.0,
            c.rotation_hours.abs(),
            if c.retrograde() { " backwards" } else { "" },
            a / 1000.0
        ),
        format!(
            "{n_raw:.1} cells per hemisphere from the Rhines scale with eddy winds of {wind:.1} m/s from the thermal wind (Rhines 1975).",
        ),
        rule.because.clone(),
    ];
    // Lay out the cells from the numbers alone: a cell that reaches the pole,
    // or a single Rhines cell, is one cell; otherwise the Hadley cell keeps at
    // least 8° and leaves 8° for each cell poleward (a registered layout rule).
    let (cells_n, hadley) = if hadley_raw >= 90.0 || cells <= 1 {
        (1, 90.0)
    } else {
        let room = 90.0 - 8.0 * (cells - 1) as f64;
        (cells, hadley_raw.min(room).max(8.0))
    };
    let mut edges = vec![0.0, hadley];
    for i in 1..cells_n {
        edges.push(hadley + (90.0 - hadley) * i as f64 / (cells_n - 1) as f64);
    }
    edges.dedup();
    // Edges alternate sinking (at the Hadley edge) and rising, poleward.
    let (mut sinking, mut rising, mut storms) = (vec![], vec![0.0], vec![]);
    for (i, e) in edges.iter().enumerate().skip(1) {
        if i % 2 == 1 {
            sinking.push(*e);
        } else {
            rising.push(*e);
        }
        // Eddy-driven (Ferrel-type) cells: between a sinking and a rising edge.
        if i % 2 == 1 && i + 1 < edges.len() {
            storms.push(0.5 * (e + edges[i + 1]));
        }
    }
    if !storms.is_empty() {
        why.push(format!(
            "Storm tracks near {} from the eddy-driven cells.",
            storms
                .iter()
                .map(|x| format!("{x:.0}°"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let spin_sign = if c.retrograde() { -1.0 } else { 1.0 };
    if c.retrograde() {
        why.push("The spin runs backwards, so trade winds blow from the west and storms turn the other way.".into());
    }
    Circulation {
        regime: rule.regime.clone(),
        regime_label: cc.regimes[&rule.regime].clone(),
        because: rule.because.clone(),
        cells: cells_n,
        cells_raw: n_raw,
        hadley_raw_deg: hadley_raw,
        hadley_edge_deg: hadley,
        edges_deg: edges,
        sinking_deg: sinking,
        rising_deg: rising,
        storm_tracks_deg: storms,
        solar_day_hours: day,
        spin_sign,
        wind_m_s: wind,
        tropopause_m: h,
        why: why.into_iter().chain(st.why.iter().cloned()).collect(),
        strengths: st,
    }
}

impl Circulation {
    /// Near-surface eastward wind at a latitude (degrees): easterly trades in
    /// the rising cells, westerlies in the eddy-driven ones (sign alternates
    /// cell by cell), all reversed on a backward-spinning world.
    pub fn prevailing_wind(&self, lat_deg: f64) -> f64 {
        let a = lat_deg.abs();
        let e = &self.edges_deg;
        for i in 0..e.len().saturating_sub(1) {
            if a >= e[i] && a <= e[i + 1] {
                let f = (a - e[i]) / (e[i + 1] - e[i]).max(1e-9);
                let shape = m::sin(f * m::PI);
                let sign = if i % 2 == 0 { -1.0 } else { 1.0 };
                // Air that is not a fluid moves no wind (collisional strength).
                return self.spin_sign
                    * sign
                    * 0.4
                    * self.wind_m_s
                    * shape
                    * self.strengths.collisional;
            }
        }
        0.0
    }

    /// Whether the prevailing surface wind at a latitude blows from the east.
    pub fn easterly(&self, lat_deg: f64) -> bool {
        let a = lat_deg.abs().min(90.0);
        let e = &self.edges_deg;
        let mut i = 0;
        while i + 2 < e.len() && a > e[i + 1] {
            i += 1;
        }
        (i % 2 == 0) == (self.spin_sign > 0.0)
    }
}

/// A giant's prograde jets pole to pole from the Rhines scale, 1 to 60 (the
/// S6 cloud deck and the giant weather read this one count). One jet is
/// π sqrt(2U/β) wide and eastward jets sit twice that apart, so with
/// β = 2Ω/a the half circumference π a holds round(½ sqrt(a |Ω| / U)) of
/// them. `eddy_m_s` is the local eddy speed (10 to 40 m/s on Jupiter), not
/// the jet speed (Rhines 1975; Gastine et al. 2014).
pub fn giant_jets(radius_m: f64, spin_rate: f64, eddy_m_s: f64) -> u32 {
    m::round(0.5 * m::sqrt(radius_m * spin_rate / eddy_m_s)).clamp(1.0, 60.0) as u32
}

/// The Rhines width of one jet at a latitude, m: π sqrt(2U/β) with
/// β = 2Ω cos φ / a.
pub fn rhines_width_m(radius_m: f64, spin_rate: f64, eddy_m_s: f64, lat_deg: f64) -> f64 {
    let beta = 2.0 * spin_rate.abs() * m::cos(lat_deg * m::PI / 180.0).max(1e-6) / radius_m;
    m::PI * m::sqrt(2.0 * eddy_m_s / beta)
}

/// A giant's band edges pole to pole, degrees: 2 × jets bands evenly spaced
/// in sin(latitude), asin(−1 + 2i / 2J).
pub fn giant_band_edges(jets: u32) -> Vec<f64> {
    let nb = 2 * jets;
    (0..=nb)
        .map(|i| m::asin(-1.0 + 2.0 * i as f64 / nb as f64) * (180.0 / m::PI))
        .collect()
}

/// A giant's circumpolar jet latitude: one polar Rhines length
/// L = (U a² / 2|Ω|)^⅓ from the pole (Rhines arrest on the polar γ-plane).
pub fn polar_jet_lat(radius_m: f64, spin_rate: f64, wind_m_s: f64) -> f64 {
    let l = m::cbrt(wind_m_s * radius_m * radius_m / (2.0 * spin_rate));
    (90.0 - l / radius_m * (180.0 / m::PI)).max(0.0)
}
