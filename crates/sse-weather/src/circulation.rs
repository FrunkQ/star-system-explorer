//! W1: the circulation regime from the facts (weather design section 2, step
//! 1), what condenses in the air, and the annual climate averages that slow
//! physics reads.
//!
//! - Hadley cell edge from Held and Hou (1980): the cell reaches
//!   φ_H = sqrt(5/3 · g H Δh / (Ω² a²)) radians, so it shrinks as spin rises.
//! - Cells per hemisphere from the Rhines scale (Rhines 1975): the arrest
//!   wavenumber is k_β = (β / 2U)^½, β = 2Ω / a, jets sit π / k_β apart, so a
//!   hemisphere holds ½ sqrt(a Ω / U).
//! - The regime is the first pack rule whose conditions hold.

use crate::conditions::Conditions;
use crate::contract::{Band, ClimateAverages};
use crate::settings::{Substance, WeatherSettings};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const R_GAS: f64 = 8.314462618;

/// Where the cells, rising and sinking air and storm tracks sit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Circulation {
    pub regime: String,
    pub regime_label: String,
    pub because: String,
    /// Overturning cells per hemisphere.
    pub cells: u32,
    pub hadley_edge_deg: f64,
    /// Cell boundaries, equator to pole (first 0, last 90).
    pub edges_deg: Vec<f64>,
    /// Latitudes of air sinking (dry) and rising (wet) at the cell edges.
    pub sinking_deg: Vec<f64>,
    pub rising_deg: Vec<f64>,
    /// Centres of the storm tracks (one per eddy-driven cell).
    pub storm_tracks_deg: Vec<f64>,
    pub solar_day_hours: f64,
    /// Every step, in words, for "Why?".
    pub why: Vec<String>,
}

/// The cloud substance and the height it forms at.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Condensation {
    pub substance: String,
    /// Height of the condensation level above the surface, m.
    pub base_m: f64,
    pub colour: [f64; 3],
}

fn deg(x: f64) -> f64 {
    x * 180.0 / m::PI
}

/// The regime and cell layout.
pub fn circulation(s: &WeatherSettings, c: &Conditions) -> Circulation {
    let cs = &s.circulation;
    let omega = c.omega();
    let a = c.radius_m;
    let h = cs.tropopause_scale_heights * c.scale_height_m();
    let phi_h = m::sqrt(5.0 / 3.0 * c.gravity_m_s2 * h * cs.delta_h / (omega * omega * a * a));
    let hadley_raw = deg(phi_h).min(90.0);
    // Rhines: k_β = (β / 2U)^½ with β = 2Ω/a; jets sit π / k_β apart, so a
    // quarter circumference (π a / 2) holds ½ sqrt(a Ω / U) of them.
    let n_raw = 0.5 * m::sqrt(a * omega / cs.wind_m_s);
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
            _ => f64::NAN,
        }
    };
    let rule = s
        .rules
        .iter()
        .find(|r| r.when.iter().all(|(k, rg)| rg.holds(q(k))))
        .expect("fallback rule");
    let mut why = vec![
        format!(
            "Hadley cell edge {hadley_raw:.0}° (Held and Hou 1980: g {:.2} m/s², tropopause {:.0} km, spin {:.1} h, radius {:.0} km).",
            c.gravity_m_s2,
            h / 1000.0,
            c.rotation_hours,
            a / 1000.0
        ),
        format!(
            "{n_raw:.1} cells per hemisphere from the Rhines scale with winds of {:.0} m/s (Rhines 1975).",
            cs.wind_m_s
        ),
        rule.because.clone(),
    ];
    // Lay out the cells. A one-cell or slow world reaches the pole.
    let (cells, hadley) = match rule.regime.as_str() {
        "slow-rotator" | "tidally-locked" => (1, 90.0),
        "thin-air" | "airless" => (cells.min(3), hadley_raw.min(90.0)),
        _ if cells <= 1 => (1, 90.0),
        _ => {
            let room = 90.0 - 8.0 * (cells - 1) as f64;
            (cells, hadley_raw.min(room).max(8.0))
        }
    };
    let mut edges = vec![0.0, hadley];
    for i in 1..cells {
        edges.push(hadley + (90.0 - hadley) * i as f64 / (cells - 1) as f64);
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
    Circulation {
        regime: rule.regime.clone(),
        regime_label: s.regimes[&rule.regime].clone(),
        because: rule.because.clone(),
        cells,
        hadley_edge_deg: hadley,
        edges_deg: edges,
        sinking_deg: sinking,
        rising_deg: rising,
        storm_tracks_deg: storms,
        solar_day_hours: day,
        why,
    }
}

/// Saturation vapour pressure, Pa: Clausius-Clapeyron from the triple point.
pub fn p_sat(sub: &Substance, t_k: f64) -> f64 {
    let l_r = sub.latent_j_kg * sub.molar_kg / R_GAS;
    sub.triple_pa * m::exp(l_r * (1.0 / sub.triple_k - 1.0 / t_k))
}

/// What condenses first going up from the ground, and where.
pub fn condensation(s: &WeatherSettings, c: &Conditions) -> Option<Condensation> {
    let cp = 3.5 * R_GAS / c.mean_molar_kg;
    let lapse = c.gravity_m_s2 / cp;
    let h = c.scale_height_m();
    let t_top = c.t_eff_k() * m::pow(0.5, 0.25);
    let p0 = c.pressure_bar * 1.0e5;
    let mut best: Option<Condensation> = None;
    for sub in &s.substances {
        // Mixing ratio: over its own liquid, humid air; otherwise as given.
        let x = if c.liquid.as_deref() == Some(sub.name.as_str()) {
            s.supply.humidity_over_liquid * p_sat(sub, c.t_surface_k) / p0
        } else {
            c.gases.get(&sub.gas).copied().unwrap_or(0.0)
        };
        if x <= 0.0 {
            continue;
        }
        // Walk up in 50 m steps (or 1 % of a scale height) to the tropopause.
        let dz = (h * 0.01).clamp(20.0, 500.0);
        let mut z = 0.0;
        while z < 40.0 * h {
            let t = c.t_surface_k - lapse * z;
            if t < t_top {
                break;
            }
            let p_i = x * p0 * m::exp(-z / h);
            if p_i >= p_sat(sub, t) {
                if best.as_ref().is_none_or(|b| z < b.base_m) {
                    best = Some(Condensation {
                        substance: sub.name.clone(),
                        base_m: z,
                        colour: sub.colour,
                    });
                }
                break;
            }
            z += dz;
        }
    }
    best
}

/// Gaussian bump, unit height.
pub(crate) fn bump(x: f64, w: f64) -> f64 {
    m::exp(-0.5 * (x / w) * (x / w))
}

/// Sun's latitude overhead on this fraction of the year (0 = northern spring).
pub fn declination_deg(c: &Conditions, year_phase: f64) -> f64 {
    let e = c.obliquity_deg * m::PI / 180.0;
    deg(m::asin(m::sin(e) * m::sin(2.0 * m::PI * year_phase)))
}

/// The rising belt's latitude at a season.
pub fn itcz_deg(s: &WeatherSettings, c: &Conditions, circ: &Circulation, year_phase: f64) -> f64 {
    let k = s.circulation.itcz_follow_ocean
        + (s.circulation.itcz_follow_land - s.circulation.itcz_follow_ocean) * c.land_fraction();
    (k * declination_deg(c, year_phase)).clamp(-circ.hadley_edge_deg, circ.hadley_edge_deg)
}

/// Relative wetness at a latitude and season (not normalised).
pub fn wetness(s: &WeatherSettings, circ: &Circulation, lat: f64, itcz: f64) -> f64 {
    let cs = &s.circulation;
    let cell = if circ.cells > 1 {
        (90.0 - circ.hadley_edge_deg) / (circ.cells - 1) as f64
    } else {
        90.0
    };
    let mut w =
        cs.rain_floor + cs.itcz_weight * bump(lat - itcz, cs.itcz_width * circ.hadley_edge_deg);
    for st in &circ.storm_tracks_deg {
        let shift = 0.5 * itcz;
        w += cs.storm_weight * bump(lat.abs() - st - shift * lat.signum(), cs.storm_width * cell);
    }
    // Polar caps dry out, sinking air at the Hadley edges dries more.
    for sk in &circ.sinking_deg {
        if *sk < 89.0 {
            w *= 1.0 - 0.6 * bump(lat.abs() - sk, 0.12 * circ.hadley_edge_deg.max(10.0));
        }
    }
    w
}

/// Fraction of absorbed sunlight that reaches the ground.
pub fn surface_sun_fraction(s: &WeatherSettings, c: &Conditions) -> f64 {
    let col = c.column_kg_m2() / crate::conditions::EARTH_COLUMN_KG_M2;
    if col <= 0.0 {
        return 1.0;
    }
    (s.supply.earth_fraction * m::pow(col, -s.supply.column_power)).min(1.0)
}

/// Latent heat flux from the wet surface, W/m² (zero with no surface liquid).
pub fn latent_w_m2(s: &WeatherSettings, c: &Conditions) -> f64 {
    if c.liquid.is_none() || !c.solid_surface {
        return 0.0;
    }
    let wet = (c.ocean_fraction / s.supply.wet_saturation).min(1.0);
    s.supply.latent_share * surface_sun_fraction(s, c) * c.absorbed_w_m2() * m::sqrt(wet)
}

/// Annual climate averages, by latitude band.
pub fn climate(s: &WeatherSettings, c: &Conditions) -> ClimateAverages {
    let circ = circulation(s, c);
    let cond = condensation(s, c);
    let liquid = c
        .liquid
        .as_ref()
        .and_then(|l| s.substances.iter().find(|x| &x.name == l));
    let latent = latent_w_m2(s, c);
    // Mean precipitation: the latent heat flux falls back out as liquid.
    let precip_mean = match liquid {
        Some(l) if latent > 0.0 => latent / l.latent_j_kg / l.condensed_kg_m3 * 3.15576e7,
        _ => 0.0,
    };
    let deck = deck_cover(s, c);
    let cover_supply = s.supply.cover_max * (1.0 - m::exp(-latent / s.supply.latent_scale_w_m2));
    let cover_base = match circ.regime.as_str() {
        "airless" => 0.0,
        "thin-air" => {
            if cond.is_some() {
                s.supply.thin_cover
            } else {
                0.0
            }
        }
        _ => {
            if cond.is_some() {
                cover_supply.max(if latent == 0.0 {
                    s.supply.thin_cover
                } else {
                    0.0
                })
            } else {
                0.0
            }
        }
    };
    let nb = s.bands;
    let ns = s.season_samples.max(1);
    let mut raw = vec![0.0; nb as usize];
    let lat_of = |i: u32| -90.0 + 180.0 * (i as f64 + 0.5) / nb as f64;
    for k in 0..ns {
        let phase = (k as f64 + 0.5) / ns as f64;
        let itcz = itcz_deg(s, c, &circ, phase);
        for i in 0..nb {
            raw[i as usize] += wetness(s, &circ, lat_of(i), itcz) / ns as f64;
        }
    }
    // Area-weighted mean of the profile is 1.
    let (mut sw, mut sa) = (0.0, 0.0);
    for i in 0..nb {
        let wgt = m::cos(lat_of(i) * m::PI / 180.0);
        sw += raw[i as usize] * wgt;
        sa += wgt;
    }
    let norm = if sw > 0.0 { sa / sw } else { 1.0 };
    let peak = raw.iter().cloned().fold(0.0, f64::max) * norm;
    let bands: Vec<Band> = (0..nb)
        .map(|i| {
            let r = raw[i as usize] * norm;
            let lat = lat_of(i);
            // Cloud follows wetness around its mean, capped at 1; decks cover all.
            let cover = (cover_base * (0.55 + 0.45 * r / peak.max(1e-9)) / 0.80).min(1.0);
            Band {
                lat_deg: lat,
                cloud_cover: cover.max(deck),
                precip_m_yr: precip_mean * r,
                wind_m_s: zonal_wind(s, &circ, lat),
            }
        })
        .collect();
    let mut cm = 0.0;
    for b in &bands {
        cm += b.cloud_cover * m::cos(b.lat_deg * m::PI / 180.0);
    }
    let condensate = if deck > 0.0 {
        deck_substance(s, c).unwrap_or_default()
    } else {
        cond.map(|x| x.substance).unwrap_or_default()
    };
    // S3's erosion term alternates wet and dry `rain_cells` times pole to equator.
    let rain_cells = circ.cells.max(1);
    ClimateAverages {
        regime: circ.regime,
        bands,
        cloud_cover_mean: cm / sa,
        precip_mean_m_yr: precip_mean,
        condensate,
        rain_cells,
        surface_sun_fraction: surface_sun_fraction(s, c),
    }
}

/// Near-surface eastward wind at a latitude: easterly trades in the rising
/// cells, westerlies in the eddy-driven ones (sign alternates cell by cell).
pub fn zonal_wind(s: &WeatherSettings, circ: &Circulation, lat: f64) -> f64 {
    let a = lat.abs();
    let e = &circ.edges_deg;
    for i in 0..e.len().saturating_sub(1) {
        if a >= e[i] && a <= e[i + 1] {
            let f = (a - e[i]) / (e[i + 1] - e[i]).max(1e-9);
            let shape = m::sin(f * m::PI);
            let sign = if i % 2 == 0 { -1.0 } else { 1.0 };
            return sign * 0.4 * s.circulation.wind_m_s * shape;
        }
    }
    0.0
}

fn deck_rule<'a>(
    s: &'a WeatherSettings,
    c: &Conditions,
) -> Option<&'a crate::settings::Photochemical> {
    s.photochemical.iter().find(|p| {
        p.cloud_type == "deck"
            && c.flux_w_m2 >= p.min_flux_w_m2
            && c.pressure_bar >= p.min_pressure_bar
            && p.needs
                .iter()
                .all(|(g, x)| c.gases.get(g).copied().unwrap_or(0.0) >= *x)
    })
}

/// Cover of a world-wide chemical deck (Venus's sulphuric acid), or 0.
pub fn deck_cover(s: &WeatherSettings, c: &Conditions) -> f64 {
    deck_rule(s, c).map(|p| p.cover).unwrap_or(0.0)
}

fn deck_substance(s: &WeatherSettings, c: &Conditions) -> Option<String> {
    deck_rule(s, c).map(|p| p.makes.clone())
}
