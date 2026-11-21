//! W1: the circulation regime (now read from the one climate, W7a), what
//! condenses in the air, and the annual climate averages slow physics reads.
//!
//! The Hadley edge, the Rhines cells, their layout, the regime label and the
//! prevailing winds all live in `sse-climate` ([`sse_climate::circulation`]);
//! this module draws on top of them: the rising belt and storm tracks by
//! season (wetness), the condensation level, and the chemical decks.

use crate::conditions::Conditions;
use crate::contract::{Band, ClimateAverages};
use crate::settings::WeatherSettings;
use serde::{Deserialize, Serialize};
pub use sse_climate::circulation::Circulation;
pub use sse_climate::ebm::{declination_deg, p_sat};
use sse_core::math as m;

const R_GAS: f64 = 8.314462618;

/// The cloud substance and the height it forms at.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Condensation {
    pub substance: String,
    /// Height of the condensation level above the surface, m.
    pub base_m: f64,
    pub colour: [f64; 3],
}

/// The regime and cell layout, from the one climate.
pub fn circulation(s: &WeatherSettings, c: &Conditions) -> Circulation {
    sse_climate::circulation::circulation(&s.climate, c)
}

/// What condenses first going up from the ground, and where.
pub fn condensation(s: &WeatherSettings, c: &Conditions) -> Option<Condensation> {
    let cp = 3.5 * R_GAS / c.mean_molar_kg;
    let lapse = c.gravity_m_s2 / cp;
    let h = c.scale_height_m();
    let t_top = c.t_eff_k() * m::pow(0.5, 0.25);
    let p0 = c.pressure_bar * 1.0e5;
    let mut best: Option<Condensation> = None;
    for sub in &s.climate.substances {
        // Mixing ratio: over its own liquid, humid air; otherwise as given.
        let x = if c.liquid.as_deref() == Some(sub.name.as_str()) {
            s.climate.supply.humidity_over_liquid * p_sat(sub, c.t_surface_k) / p0
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
    sse_climate::ebm::surface_sun_fraction(&s.climate, c)
}

/// The one climate of a world now (W7a): what both weather blocks, the
/// surface's wear and its cover all read.
pub fn core(s: &WeatherSettings, c: &Conditions) -> sse_climate::Climate {
    sse_climate::orbit(&s.climate, c, 0.0)
}

/// Annual climate averages, by latitude band: the one climate's rain and
/// cloud, the shared prevailing winds, and any chemical deck on top.
pub fn climate(s: &WeatherSettings, c: &Conditions) -> ClimateAverages {
    averages(s, c, &core(s, c))
}

/// [`climate`] from a climate already computed.
pub fn averages(s: &WeatherSettings, c: &Conditions, k: &sse_climate::Climate) -> ClimateAverages {
    let deck = deck_cover(s, c);
    let bands: Vec<Band> = (0..k.lat_deg.len())
        .map(|j| Band {
            lat_deg: k.lat_deg[j],
            cloud_cover: k.cover[j].max(deck),
            precip_m_yr: k.precip_m_yr[j],
            wind_m_s: k.circulation.prevailing_wind(k.lat_deg[j]),
        })
        .collect();
    let cover: Vec<f64> = bands.iter().map(|b| b.cloud_cover).collect();
    let condensate = if deck > 0.0 {
        deck_substance(s, c).unwrap_or_default()
    } else {
        k.condensate.clone()
    };
    ClimateAverages {
        regime: k.circulation.regime.clone(),
        cloud_cover_mean: k.mean_of(&cover),
        precip_mean_m_yr: k.mean_of(&k.precip_m_yr),
        bands,
        condensate,
        rain_cells: wet_maxima(&k.precip_m_yr),
        surface_sun_fraction: surface_sun_fraction(s, c),
    }
}

/// Wet maxima from pole to equator in the northern half, at least one.
pub fn wet_maxima(p: &[f64]) -> u32 {
    let n = p.len();
    let north = &p[n / 2..];
    let mut k = 0;
    for j in 0..north.len() {
        let l = if j > 0 { north[j - 1] } else { f64::MIN };
        let r = if j + 1 < north.len() {
            north[j + 1]
        } else {
            f64::MIN
        };
        if north[j] > 0.0 && north[j] > l && north[j] >= r {
            k += 1;
        }
    }
    k.max(1)
}

/// Near-surface eastward wind at a latitude: the one climate's prevailing
/// wind (easterly trades, westerlies, reversed on a backward spin).
pub fn zonal_wind(_s: &WeatherSettings, circ: &Circulation, lat: f64) -> f64 {
    circ.prevailing_wind(lat)
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

pub fn deck_substance(s: &WeatherSettings, c: &Conditions) -> Option<String> {
    deck_rule(s, c).map(|p| p.makes.clone())
}
