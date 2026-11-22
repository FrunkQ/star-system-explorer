//! The one climate of a world at one epoch, at a detail level:
//!
//! - **Glance (D0)**: scalars only (mean and emission temperature, the
//!   circulation, what condenses). Microseconds.
//! - **Orbit (D1)**: the energy balance in every band and season, the annual
//!   means and the seasonal range. About a millisecond. This is what erosion,
//!   cover and both weather blocks read.
//!
//! Survey (D2: land and sea per cell) and Site (D3) arrive with W7c and W11.
//! A result is keyed by world, code version, level and epoch, and readers
//! reuse a kept result before computing one ([`Cache`]).

use crate::circulation::{circulation, Circulation};
use crate::conditions::Conditions;
use crate::ebm::{self, Season};
use crate::settings::ClimateSettings;
use crate::CLIMATE_VERSION;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Glance,
    Orbit,
}

impl Level {
    pub fn word(&self) -> &'static str {
        match self {
            Level::Glance => "glance",
            Level::Orbit => "orbit",
        }
    }
}

/// Key a climate result is kept under: world, code version, level, epoch.
pub fn key(c: &Conditions, level: Level, epoch_myr: f64) -> String {
    let facts = serde_json::to_string(c).expect("plain data");
    let h = sse_core::ContentHash::of(&facts).expect("hash").hex();
    format!(
        "{}|{CLIMATE_VERSION}|{}|{epoch_myr}|{}",
        c.address,
        level.word(),
        &h[..16]
    )
}

/// D0: climate scalars from orbit and air.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Glance {
    pub key: String,
    pub t_mean_k: f64,
    pub t_eff_k: f64,
    pub circulation: Circulation,
    pub solar_day_hours: f64,
    pub retrograde: bool,
    /// The substances that saturate somewhere between the warmest and
    /// coldest plausible band (the mean ± 30 %), lowest first.
    pub condensables: Vec<String>,
}

pub fn glance(cs: &ClimateSettings, c: &Conditions, epoch_myr: f64) -> Glance {
    let p0 = c.pressure_bar * 1.0e5;
    let mut condensables = vec![];
    for sub in &cs.substances {
        let x = if c.liquid.as_deref() == Some(sub.name.as_str()) {
            1.0
        } else {
            c.gases.get(&sub.gas).copied().unwrap_or(0.0)
        };
        if x > 0.0 && x * p0 >= ebm::p_sat(sub, 0.7 * c.t_surface_k) {
            condensables.push(sub.name.clone());
        }
    }
    Glance {
        key: key(c, Level::Glance, epoch_myr),
        t_mean_k: c.t_surface_k,
        t_eff_k: c.t_eff_k(),
        circulation: circulation(cs, c),
        solar_day_hours: c.solar_day_hours(),
        retrograde: c.retrograde(),
        condensables,
    }
}

/// D1: the energy balance in every band and season.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Climate {
    pub key: String,
    pub level: Level,
    pub conditions: Conditions,
    pub circulation: Circulation,
    /// Band centres, south to north, degrees.
    pub lat_deg: Vec<f64>,
    /// Annual mean temperature, and the coldest and warmest season, K.
    pub t_k: Vec<f64>,
    pub t_min_k: Vec<f64>,
    pub t_max_k: Vec<f64>,
    /// Annual rain and snow reaching the ground, m of liquid a year.
    pub precip_m_yr: Vec<f64>,
    /// Annual evaporation, m of liquid a year.
    pub evap_m_yr: Vec<f64>,
    /// Annual condensation cloud cover, 0 to 1.
    pub cover: Vec<f64>,
    /// Annual eastward wind near the ground, m/s (negative: from the east).
    pub wind_m_s: Vec<f64>,
    pub albedo: Vec<f64>,
    /// Each season's steady state at phase (k + ½) / n of the year.
    pub seasons: Vec<Season>,
    /// The annual means as one season.
    pub annual: Season,
    pub condensate: String,
    /// Density of the surface liquid the rain is measured in, kg/m³.
    pub liquid_kg_m3: f64,
    /// Ice of the main gas moving between ground and air (W8), when its
    /// frost point is within reach.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange: Option<crate::exchange::Exchange>,
    /// W7c: land fraction of each band, when the world runs a land and a sea
    /// column (empty otherwise; the seasons then carry no column values).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub land: Vec<f64>,
    pub why: Vec<String>,
}

const YEAR_S: f64 = sse_core::sheet::YR_JULIAN;

/// D1: the one climate of a world at an epoch.
pub fn orbit(cs: &ClimateSettings, c: &Conditions, epoch_myr: f64) -> Climate {
    let ns = cs.seasons.max(1);
    let mut seasons: Vec<Season> = ebm::cycle(cs, c);
    let mut exchange = None;
    if let Some((sub, x)) = crate::exchange::candidate(cs, c, &seasons) {
        let (s, ex) = crate::exchange::cycle(cs, c, sub, x);
        seasons = s;
        exchange = Some(ex);
    }
    let annual = ebm::mean(&seasons);
    let n = annual.lat_deg.len();
    let rho = ebm::liquid(cs, c)
        .map(|s| s.condensed_kg_m3)
        .unwrap_or(1000.0);
    let to_m_yr = YEAR_S / rho;
    let t_min_k: Vec<f64> = (0..n)
        .map(|j| {
            seasons
                .iter()
                .map(|s| s.t_k[j])
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    let t_max_k: Vec<f64> = (0..n)
        .map(|j| {
            seasons
                .iter()
                .map(|s| s.t_k[j])
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect();
    let cl = circulation(cs, c);
    let w: Vec<f64> = annual
        .lat_deg
        .iter()
        .map(|l| m::cos(ebm::rad(*l)))
        .collect();
    let pm = ebm::weighted_mean(&w, &annual.precip) * to_m_yr;
    let mut why = cl.why.clone();
    why.push(format!(
        "Energy balance in {n} bands and {ns} seasons with heat capacity (North 1975; North and Coakley 1979): equator {:.0} K, poles {:.0} K and {:.0} K; heat moved with D = {:.3} W m⁻² K⁻¹ against B = {:.2}.",
        annual.t_k[n / 2],
        annual.t_k[0],
        annual.t_k[n - 1],
        annual.d_w_m2_k,
        annual.b_w_m2_k
    ));
    if pm > 0.0 {
        why.push(format!(
            "{pm:.2} m of {} a year on average, falling where moist air converges.",
            ebm::liquid(cs, c)
                .map(|s| s.name.as_str())
                .unwrap_or("rain")
        ));
    }
    if let Some(ex) = &exchange {
        why.extend(ex.why.iter().cloned());
    }
    let land = if annual.t_land_k.len() == n {
        ebm::band_land(cs, c)
    } else {
        vec![]
    };
    if !land.is_empty() {
        // The band with the widest gap between the land's and the sea's year.
        let range = |v: &dyn Fn(&Season) -> f64| {
            let (lo, hi) = seasons
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), s| {
                    (a.min(v(s)), b.max(v(s)))
                });
            hi - lo
        };
        let mut best = (0usize, 0.0, 0.0);
        for (j, f) in land.iter().enumerate() {
            if *f <= 0.0 || *f >= 1.0 {
                continue;
            }
            let rl = range(&|s: &Season| s.t_land_k[j]);
            let rs = range(&|s: &Season| s.t_sea_k[j]);
            if rl - rs > best.1 - best.2 {
                best = (j, rl, rs);
            }
        }
        if best.1 > 0.0 {
            why.push(format!(
                "Land and sea warm at their own pace (North, Mengel and Short 1983): at {:.0}° the land swings {:.0} K through the year and the sea {:.0} K, because the sea's mixed layer holds far more heat; the band is {:.0} % land.",
                annual.lat_deg[best.0],
                best.1,
                best.2,
                100.0 * land[best.0]
            ));
        }
    }
    if c.retrograde() {
        why.push("It spins backwards, so its winds and storms turn the other way.".into());
    }
    Climate {
        key: key(c, Level::Orbit, epoch_myr),
        level: Level::Orbit,
        conditions: c.clone(),
        circulation: cl,
        lat_deg: annual.lat_deg.clone(),
        t_k: annual.t_k.clone(),
        t_min_k,
        t_max_k,
        precip_m_yr: annual.precip.iter().map(|x| x * to_m_yr).collect(),
        evap_m_yr: annual.evap.iter().map(|x| x * to_m_yr).collect(),
        cover: annual.cover.clone(),
        wind_m_s: annual.wind_m_s.clone(),
        albedo: annual.albedo.clone(),
        condensate: annual.condensate.clone(),
        seasons,
        annual,
        liquid_kg_m3: rho,
        exchange,
        land,
        why,
    }
}

impl Climate {
    /// Linear interpolation between band centres at a latitude (degrees),
    /// held flat beyond the outermost centres.
    pub fn lerp(&self, v: &[f64], lat_deg: f64) -> f64 {
        let n = self.lat_deg.len();
        let x = (lat_deg + 90.0) / 180.0 * n as f64 - 0.5;
        if x <= 0.0 {
            return v[0];
        }
        if x >= (n - 1) as f64 {
            return v[n - 1];
        }
        let i = m::floor(x) as usize;
        let f = x - i as f64;
        v[i] + (v[i + 1] - v[i]) * f
    }

    /// Annual mean temperature at a latitude, K (at the datum).
    pub fn t_at(&self, lat_deg: f64) -> f64 {
        self.lerp(&self.t_k, lat_deg)
    }

    /// W7c: annual mean over the land at a latitude, K (the band's mean on
    /// a world without both land and sea).
    pub fn t_land_at(&self, lat_deg: f64) -> f64 {
        self.column(lat_deg, |s| &s.t_land_k).0
    }

    /// W7c: annual mean over the sea at a latitude, K.
    pub fn t_sea_at(&self, lat_deg: f64) -> f64 {
        self.column(lat_deg, |s| &s.t_sea_k).0
    }

    /// W7c: the land's coldest and warmest season at a latitude, K.
    pub fn land_range_at(&self, lat_deg: f64) -> (f64, f64) {
        let c = self.column(lat_deg, |s| &s.t_land_k);
        (c.1, c.2)
    }

    /// W7c: the sea's coldest and warmest season at a latitude, K.
    pub fn sea_range_at(&self, lat_deg: f64) -> (f64, f64) {
        let c = self.column(lat_deg, |s| &s.t_sea_k);
        (c.1, c.2)
    }

    /// A column's annual mean, coldest and warmest season at a latitude,
    /// falling back on the band's own values.
    fn column(&self, lat_deg: f64, pick: impl Fn(&Season) -> &Vec<f64>) -> (f64, f64, f64) {
        let n = self.lat_deg.len();
        if pick(&self.annual).len() != n || self.seasons.iter().any(|s| pick(s).len() != n) {
            return (
                self.t_at(lat_deg),
                self.lerp(&self.t_min_k, lat_deg),
                self.lerp(&self.t_max_k, lat_deg),
            );
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for s in &self.seasons {
            let v = self.lerp(pick(s), lat_deg);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        (self.lerp(pick(&self.annual), lat_deg), lo, hi)
    }

    /// Annual rain and snow at a latitude, m a year.
    pub fn precip_at(&self, lat_deg: f64) -> f64 {
        self.lerp(&self.precip_m_yr, lat_deg).max(0.0)
    }

    /// Wettest band's precipitation, m a year.
    pub fn precip_peak(&self) -> f64 {
        self.precip_m_yr.iter().cloned().fold(0.0, f64::max)
    }

    /// Rain at a latitude relative to the wettest band, 0 to 1.
    pub fn wetness_at(&self, lat_deg: f64) -> f64 {
        let p = self.precip_peak();
        if p <= 0.0 {
            0.0
        } else {
            (self.precip_at(lat_deg) / p).clamp(0.0, 1.0)
        }
    }

    /// Eastward wind near the ground at a latitude, m/s.
    pub fn wind_at(&self, lat_deg: f64) -> f64 {
        self.lerp(&self.wind_m_s, lat_deg)
    }

    /// Area-weighted mean of band values.
    pub fn mean_of(&self, v: &[f64]) -> f64 {
        let w: Vec<f64> = self.lat_deg.iter().map(|l| m::cos(ebm::rad(*l))).collect();
        ebm::weighted_mean(&w, v)
    }

    /// The season holding a phase of the year (0 = northern spring).
    pub fn season_at(&self, year_phase: f64) -> &Season {
        let ns = self.seasons.len();
        let p = year_phase - m::floor(year_phase);
        let k = (m::floor(p * ns as f64) as usize).min(ns - 1);
        &self.seasons[k]
    }
}

/// Kept climates, newest last, by key. Deterministic: a plain list.
#[derive(Clone, Debug, Default)]
pub struct Cache {
    pub cap: usize,
    items: Vec<(String, Climate)>,
}

impl Cache {
    pub fn new(cap: usize) -> Self {
        Cache { cap, items: vec![] }
    }

    /// The kept climate for this world and epoch, or a new one.
    pub fn orbit(&mut self, cs: &ClimateSettings, c: &Conditions, epoch_myr: f64) -> &Climate {
        let k = key(c, Level::Orbit, epoch_myr);
        if let Some(i) = self.items.iter().position(|(x, _)| *x == k) {
            let it = self.items.remove(i);
            self.items.push(it);
        } else {
            self.items.push((k, orbit(cs, c, epoch_myr)));
            while self.items.len() > self.cap.max(1) {
                self.items.remove(0);
            }
        }
        &self.items.last().expect("just pushed").1
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
