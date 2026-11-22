//! The water a place receives (W9, weather review section 4): rain, snow,
//! melt, evaporation and runoff at any latitude and height, read off the one
//! climate's seasons, so geology (S3 wear's rivers and glaciers, S9 strata,
//! S14b karst) and weather share one supply. The surface asks once per cell
//! and epoch with the cell's own height, so a mountain gets its own snow line.
//!
//! - **Temperature** at the place: the band's season temperature less the
//!   lapse rate times the height.
//! - **Rain or snow**: precipitation falls as snow when the air at the time is
//!   colder than the freezing point plus `snow_offset_k` (1.0 K, the mean 50 %
//!   rain-snow threshold over the Northern Hemisphere, Jennings et al. 2018).
//!   Air temperature about a season's mean scatters with `sigma_k` (5 K,
//!   Calov and Greve 2005), so a season's snow share is
//!   Φ((T_freeze + offset − T) / σ).
//! - **Melt**: positive degree days with the same scatter, the expected
//!   excess over freezing σ/√(2π) e^(−T²/2σ²) + (T/2) erfc(−T/(√2σ)) per day
//!   (Calov and Greve 2005), melting snow at `ddf_snow_mm` and bare ice at
//!   `ddf_ice_mm` per degree day (Braithwaite 1995: about 8 mm for ice, snow
//!   less than half of that).
//! - **Ice balance**: snow less what melts in the year; above zero the snow
//!   outlasts the summer and builds a glacier.
//! - **Potential evaporation**: the energy-limited latent flux the climate
//!   already uses (the share of sunlight reaching wet ground that evaporates,
//!   `supply.latent_share`), only while the ground is not frozen.
//! - **Evaporation and runoff** from the water that reaches the ground (rain
//!   and melt) by Fu's (1981) Budyko curve, E/W = 1 + Φ − (1 + Φ^ω)^(1/ω),
//!   Φ = PE/W, with ω = 2.6, the value that matches Budyko's curve (Zhang et
//!   al. 2004). Runoff is W − E.
//!
//! A world without a surface liquid gets no water at all (its frosts and
//! moving ices are W8's `sse.volatile.exchange`).

use crate::climate::Climate;
use crate::ebm;
use crate::settings::{ClimateSettings, AREA_QUANTITIES};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const YEAR_S: f64 = sse_core::sheet::YR_JULIAN;

/// The water supply at one place, per year (metres of the liquid).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct WaterSupply {
    /// Annual mean, coldest and warmest season temperature at the place, K.
    pub t_k: f64,
    pub t_cold_k: f64,
    pub t_warm_k: f64,
    pub precip_m_yr: f64,
    pub rain_m_yr: f64,
    pub snow_m_yr: f64,
    /// Snow that melts in the year.
    pub melt_m_yr: f64,
    /// Snow less what melts (positive: a glacier grows).
    pub ice_balance_m_yr: f64,
    pub pet_m_yr: f64,
    pub evap_m_yr: f64,
    pub runoff_m_yr: f64,
    /// Precipitation over potential evaporation (Lancaster's P / PE).
    pub aridity: f64,
    /// Share of the year's precipitation in the warmer half of the seasons.
    pub summer_share: f64,
    /// Annual precipitation over ten times Köppen's dryness threshold
    /// (Kottek et al. 2006): below 1 is arid, below 0.5 desert.
    pub p_over_pth: f64,
    /// The surface liquid's freezing point, K (0 without one).
    pub freeze_k: f64,
    /// The liquid is water.
    pub water: bool,
}

/// Standard normal cumulative distribution.
fn phi(x: f64) -> f64 {
    0.5 * m::erfc(-x / core::f64::consts::SQRT_2)
}

/// Expected degrees above freezing per day for a mean `t` (relative to
/// freezing) with scatter `s` (Calov and Greve 2005).
pub fn degree_days_per_day(t: f64, s: f64) -> f64 {
    if s <= 0.0 {
        return t.max(0.0);
    }
    s / m::sqrt(2.0 * m::PI) * m::exp(-t * t / (2.0 * s * s))
        + 0.5 * t * m::erfc(-t / (core::f64::consts::SQRT_2 * s))
}

/// Fu's evaporation for water input `w` and potential evaporation `pe`.
pub fn fu_evaporation(w: f64, pe: f64, omega: f64) -> f64 {
    if w <= 0.0 || pe <= 0.0 {
        return 0.0;
    }
    let x = pe / w;
    let e = w * (1.0 + x - m::pow(1.0 + m::pow(x, omega), 1.0 / omega));
    e.clamp(0.0, w.min(pe))
}

/// The water supply at `lat_deg` and `height_m` above the datum, under the
/// one climate with a lapse rate of `lapse_k_km`.
pub fn water_at(
    cs: &ClimateSettings,
    cl: &Climate,
    lapse_k_km: f64,
    lat_deg: f64,
    height_m: f64,
) -> WaterSupply {
    let ws = &cs.water;
    let c = &cl.conditions;
    let cool = lapse_k_km * height_m.max(0.0) / 1000.0;
    let ns = cl.seasons.len().max(1);
    let temps: Vec<f64> = cl
        .seasons
        .iter()
        .map(|s| cl.lerp(&s.t_k, lat_deg) - cool)
        .collect();
    let t_k = temps.iter().sum::<f64>() / ns as f64;
    let t_cold_k = temps.iter().cloned().fold(f64::INFINITY, f64::min);
    let t_warm_k = temps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let Some(liq) = ebm::liquid(cs, c) else {
        return WaterSupply {
            t_k,
            t_cold_k,
            t_warm_k,
            ..Default::default()
        };
    };
    let rho = liq.condensed_kg_m3;
    let to_m_yr = YEAR_S / rho;
    let tf = liq.triple_k;
    let sun = ebm::surface_sun_fraction(cs, c) * cs.supply.latent_share;
    let days = c.year_days.max(1e-6) / ns as f64;
    let (mut precip, mut snow, mut pet, mut pdd, mut summer) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (s, t) in cl.seasons.iter().zip(&temps) {
        let p = cl.lerp(&s.precip, lat_deg).max(0.0) * to_m_yr / ns as f64;
        let share = phi((tf + ws.snow_offset_k - t) / ws.sigma_k.max(1e-9));
        precip += p;
        snow += p * share;
        if *t >= t_k {
            summer += p;
        }
        // Energy-limited evaporation while the ground is not frozen.
        let unfrozen = phi((t - tf) / ws.sigma_k.max(1e-9));
        let e = cl.lerp(&s.absorbed_w_m2, lat_deg).max(0.0) * sun / liq.latent_j_kg;
        pet += e * to_m_yr / ns as f64 * unfrozen;
        pdd += degree_days_per_day(t - tf, ws.sigma_k) * days;
    }
    // Snow melts first at the snow factor; degree days left over melt ice.
    let melt_cap = ws.ddf_snow_mm * 1.0e-3 * pdd * 1000.0 / rho;
    let melt = melt_cap.min(snow);
    let left_dd = if ws.ddf_snow_mm > 0.0 {
        (melt_cap - melt) / (ws.ddf_snow_mm * 1.0e-3 * 1000.0 / rho)
    } else {
        0.0
    };
    let ablate = left_dd * ws.ddf_ice_mm * 1.0e-3 * 1000.0 / rho;
    let ice_balance = snow - melt - ablate;
    let input = precip - snow + melt;
    let evap = fu_evaporation(input, pet, ws.fu_omega);
    let runoff = (input - evap).max(0.0);
    let aridity = if pet > 0.0 {
        precip / pet
    } else if precip > 0.0 {
        1.0e6
    } else {
        0.0
    };
    let summer_share = if precip > 0.0 { summer / precip } else { 0.5 };
    // Köppen's dryness threshold in mm (Kottek et al. 2006 eq. 2.1).
    let t_c = t_k - sse_core::sheet::T0C;
    let pth = 2.0 * t_c
        + if summer_share >= 2.0 / 3.0 {
            28.0
        } else if summer_share <= 1.0 / 3.0 {
            0.0
        } else {
            14.0
        };
    let p_mm = precip * 1000.0;
    let p_over_pth = if pth > 0.0 {
        p_mm / (10.0 * pth)
    } else {
        1.0e6
    };
    WaterSupply {
        t_k,
        t_cold_k,
        t_warm_k,
        precip_m_yr: precip,
        rain_m_yr: precip - snow,
        snow_m_yr: snow,
        melt_m_yr: melt,
        ice_balance_m_yr: ice_balance,
        pet_m_yr: pet,
        evap_m_yr: evap,
        runoff_m_yr: runoff,
        aridity,
        summer_share,
        p_over_pth,
        freeze_k: tf,
        water: liq.gas == "h2o",
    }
}

/// A quantity an area rule may read.
pub fn quantity(w: &WaterSupply, q: &str) -> f64 {
    match q {
        "liquid" => f64::from(u8::from(w.freeze_k > 0.0)),
        "liquid_water" => f64::from(u8::from(w.water)),
        "ice_balance_m_yr" => w.ice_balance_m_yr,
        "precip_m_yr" => w.precip_m_yr,
        "runoff_m_yr" => w.runoff_m_yr,
        "aridity" => w.aridity,
        "p_over_pth" => w.p_over_pth,
        "t_cold_c" => w.t_cold_k - sse_core::sheet::T0C,
        "t_warm_c" => w.t_warm_k - sse_core::sheet::T0C,
        "t_mean_c" => w.t_k - sse_core::sheet::T0C,
        "t_warm_over_freeze_k" => w.t_warm_k - w.freeze_k,
        "snow_share" => {
            if w.precip_m_yr > 0.0 {
                w.snow_m_yr / w.precip_m_yr
            } else {
                0.0
            }
        }
        _ => {
            debug_assert!(!AREA_QUANTITIES.contains(&q), "unhandled {q}");
            f64::NAN
        }
    }
}

/// The area's climate label: the first rule whose conditions all hold
/// (a label read off the numbers; nothing reads it back).
pub fn area<'a>(cs: &'a ClimateSettings, w: &WaterSupply) -> (&'a str, &'a str) {
    cs.areas
        .rules
        .iter()
        .find(|r| r.when.iter().all(|(q, range)| range.holds(quantity(w, q))))
        .map(|r| (r.area.as_str(), r.because.as_str()))
        .unwrap_or(("unknown", ""))
}
