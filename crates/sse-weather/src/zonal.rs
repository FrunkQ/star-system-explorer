//! W6: a second, deliberately different weather block behind the same
//! contract, `sse.weather.zonal-ebm`. Since W7a both blocks draw on the one
//! climate (`sse-climate`): the diffusive energy balance in latitude and
//! season that used to live here is now the climate core every reader
//! shares, so the two blocks agree on rain, temperature and cloud by
//! construction and differ in how they draw the weather on top.
//!
//! - This block draws each cell straight from the climate's season: its
//!   cloud, rain and thermal wind, and the departure of the season from the
//!   year.
//! - Storms: one simple rule, a cyclone every few days in each hemisphere at
//!   the steepest temperature gradient when the world spins fast enough.
//!   Pins are honoured exactly as the rule block honours them.

use crate::circulation;
use crate::conditions::Conditions;
use crate::contract::*;
use crate::formations::{distance_km, position_at};
use crate::rules::{cell_dirs, haze, lat_lon, year_phase};
use crate::settings::{Reference, Substance, WeatherSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use sse_surface::atlas::TileKey;

pub const BLOCK_ID: &str = "sse.weather.zonal-ebm";
pub const BLOCK_VERSION: &str = "1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Storms {
    /// Only where the solar day is shorter than this and the world is not star-locked.
    pub max_day_hours: f64,
    /// Least steepest gradient in a hemisphere, K per degree.
    pub min_gradient_k_deg: f64,
    pub every_days: f64,
    pub life_days: f64,
    pub size_km: f64,
    pub peak_wind_m_s: f64,
    /// Steering: the storm moves east at this multiple of the band wind.
    pub steering: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ZonalSettings {
    pub storms: Storms,
    pub references: Vec<Reference>,
}

impl ZonalSettings {
    /// The last pack in the stack that provides `weather_zonal` wins.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("weather_zonal"))
            .ok_or("no pack provides weather_zonal settings")?;
        let s: ZonalSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("weather_zonal settings: {e}"))?;
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every weather_zonal reference needs a DOI or a URL".into());
        }
        Ok(s)
    }
}

pub struct ZonalBlock {
    /// The shared weather pack: substances, chemical decks and hazes, the
    /// regime classification and the sunlight-at-ground form.
    pub shared: WeatherSettings,
    pub z: ZonalSettings,
}

/// One season's (or the year's) zonal state: the one climate's season.
pub type Zonal = sse_climate::ebm::Season;

fn rad(x: f64) -> f64 {
    x * m::PI / 180.0
}

pub use sse_climate::ebm::insolation;

impl ZonalBlock {
    pub fn new(shared: WeatherSettings, z: ZonalSettings) -> Self {
        ZonalBlock { shared, z }
    }

    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        Ok(ZonalBlock::new(
            WeatherSettings::from_packs(stack)?,
            ZonalSettings::from_packs(stack)?,
        ))
    }

    fn liquid<'a>(&'a self, c: &Conditions) -> Option<&'a Substance> {
        sse_climate::ebm::liquid(&self.shared.climate, c)
    }

    /// Diffusivity and longwave slope for a world, W m⁻² K⁻¹.
    pub fn coefficients(&self, c: &Conditions) -> (f64, f64) {
        sse_climate::ebm::coefficients(&self.shared.climate, c)
    }

    /// The one climate this block draws on.
    pub fn core(&self, c: &Conditions) -> sse_climate::Climate {
        circulation::core(&self.shared, c)
    }

    /// The one climate's season holding a date, with any chemical deck.
    pub fn at(&self, c: &Conditions, date: Date) -> Zonal {
        let k = self.core(c);
        self.with_deck(c, k.season_at(year_phase(c, date)).clone())
    }

    /// Annual means, with any chemical deck.
    pub fn annual(&self, c: &Conditions) -> Zonal {
        self.with_deck(c, self.core(c).annual)
    }

    fn with_deck(&self, c: &Conditions, mut z: Zonal) -> Zonal {
        let deck = circulation::deck_cover(&self.shared, c);
        if deck > 0.0 {
            for x in z.cover.iter_mut() {
                *x = x.max(deck);
            }
            z.condensate = circulation::deck_substance(&self.shared, c).unwrap_or(z.condensate);
        }
        z
    }

    fn liquid_density(&self, c: &Conditions) -> f64 {
        self.liquid(c).map(|s| s.condensed_kg_m3).unwrap_or(1000.0)
    }

    /// The drawn cyclones alive on a date, before pins.
    pub fn drawn(&self, c: &Conditions, date: Date) -> Vec<WeatherSystem> {
        let st = &self.z.storms;
        if c.star_locked
            || c.solar_day_hours() >= st.max_day_hours
            || c.pressure_bar < self.shared.climate.cloud.min_pressure_bar
            || st.every_days <= 0.0
            || !date.day.is_finite()
            || date.day.abs() >= 1.0e12
        {
            return vec![];
        }
        let d = Draws::new(
            Seed::from_text("sse4-weather-zonal"),
            c.address.clone(),
            Stream::new("weather.zonal_cyclones.v1").expect("stream"),
        );
        let first = m::floor((date.day - st.life_days) / st.every_days) as i64;
        let last = m::floor(date.day / st.every_days) as i64;
        let mut out = vec![];
        for k in first..=last {
            let born = k as f64 * st.every_days;
            let ends = born + st.life_days;
            if !(born <= date.day && date.day < ends) {
                continue;
            }
            let zs = self.at(c, Date { day: born });
            let n = zs.lat_deg.len();
            for (hi, hemi) in ["s", "n"].iter().enumerate() {
                // Cyclones need the Coriolis turn: none within 20° of the equator.
                let band: Vec<usize> = (0..n)
                    .filter(|j| {
                        let l = zs.lat_deg[*j];
                        (if hi == 0 { l < 0.0 } else { l > 0.0 }) && l.abs() >= 20.0
                    })
                    .collect();
                let Some(&start) = band.first() else {
                    continue;
                };
                let mut jb = start;
                for j in band {
                    if zs.gradient_k_deg[j].abs() > zs.gradient_k_deg[jb].abs() {
                        jb = j;
                    }
                }
                if zs.gradient_k_deg[jb].abs() < st.min_gradient_k_deg {
                    continue;
                }
                let lat = zs.lat_deg[jb];
                let counter = ((k + (1i64 << 40)) as u64) * 2 + hi as u64;
                let lon0 = -180.0 + 360.0 * d.unit(counter);
                let u = zs.wind_m_s[jb] * st.steering;
                let mut track = vec![];
                let mut t = born;
                while t <= ends + 1e-9 {
                    let dl = u * (t - born) * 86_400.0 / (c.radius_m * m::cos(rad(lat)).max(0.05))
                        * 180.0
                        / m::PI;
                    let lon = m::fmod(lon0 + dl + 540.0, 360.0);
                    let lon = if lon < 0.0 { lon + 180.0 } else { lon - 180.0 };
                    track.push([t, lat, lon]);
                    t += 0.25;
                }
                out.push(WeatherSystem {
                    id: format!("zonal-cyclone-{hemi}-{k}"),
                    kind: "cyclone".into(),
                    lat_deg: lat,
                    lon_deg: lon0,
                    track,
                    size_km: st.size_km,
                    strength: st.peak_wind_m_s,
                    born,
                    ends,
                    count: 0,
                    why: format!(
                        "A cyclone where the temperature falls fastest with latitude ({:.2} K per degree) in the energy-balance model.",
                        zs.gradient_k_deg[jb].abs()
                    ),
                    evidence: "drawn".into(),
                });
            }
        }
        out
    }

    /// Systems with pins honoured (a pinned system replaces the drawn one of
    /// the same id and lives for its recorded life), as the rule block does.
    pub fn systems_with(&self, c: &Conditions, date: Date, pins: &[Pin]) -> Vec<WeatherSystem> {
        let mut out: Vec<WeatherSystem> = self
            .drawn(c, date)
            .into_iter()
            .filter(|s| !pins.iter().any(|q| q.id == s.id))
            .collect();
        for q in pins {
            if q.system.born <= date.day && date.day < q.system.ends {
                let mut s = q.system.clone();
                if s.evidence != "authored" {
                    s.evidence = "pinned".into();
                }
                out.push(s);
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn cell(
        &self,
        c: &Conditions,
        zs: &Zonal,
        za: &Zonal,
        date: Date,
        systems: &[WeatherSystem],
        lat: f64,
        lon: f64,
    ) -> Cell {
        let n = zs.lat_deg.len();
        let j = (((lat + 90.0) / 180.0 * n as f64) as usize).min(n - 1);
        let cover = zs.cover[j];
        let rho_l = self.liquid_density(c);
        let mut precip = zs.precip[j] * 86_400.0 / rho_l * 1000.0;
        let cond = zs.condensate.clone();
        let mut layers: Vec<Layer> = (0..LAYERS.len())
            .map(|k| Layer {
                cloud_cover: 0.0,
                cloud_type: "none".into(),
                condensate: String::new(),
                optical_depth: 0.0,
                wind_m_s: [zs.wind_m_s[j] * (1.0 + 1.5 * k as f64), 0.0],
                turbulence: 0.0,
            })
            .collect();
        let set = |l: &mut Layer, cv: f64, kind: &str, tau: f64| {
            l.cloud_cover = cv.clamp(0.0, 1.0);
            l.cloud_type = kind.into();
            l.condensate = cond.clone();
            l.optical_depth = tau;
        };
        let deck = circulation::deck_cover(&self.shared, c);
        if cover > 0.0 && deck <= 0.0 {
            let wet = precip > 2.0;
            set(
                &mut layers[0],
                cover * 0.8,
                if wet { "nimbostratus" } else { "stratocumulus" },
                2.0 + 10.0 * cover,
            );
            set(&mut layers[1], cover * 0.4, "altostratus", 2.0 * cover);
            set(&mut layers[2], cover * 0.3, "cirrus", 0.5);
        } else if deck > 0.0 {
            if let Some(pr) = self
                .shared
                .photochemical
                .iter()
                .find(|x| x.cloud_type == "deck")
            {
                let li = pr.layer.min(LAYERS.len() - 1);
                set(&mut layers[li], deck, "deck", pr.optical_depth);
            }
        }
        // Storms and pinned systems: thick cloud and rain over their cores.
        for s in systems {
            if s.kind.starts_with("order-") {
                continue;
            }
            let (sl, so) = position_at(s, date.day);
            let r = distance_km(c, (lat, lon), (sl, so));
            if r <= s.size_km / 2.0 {
                let core = 1.0 - r / (s.size_km / 2.0).max(1e-9);
                let l = &mut layers[1];
                l.cloud_cover = l.cloud_cover.max(0.5 + 0.5 * core);
                l.cloud_type = "nimbostratus".into();
                l.condensate = if cond.is_empty() {
                    "water".into()
                } else {
                    cond.clone()
                };
                l.optical_depth = l.optical_depth.max(10.0 * core);
                for l in layers.iter_mut() {
                    l.wind_m_s[1] += s.strength * core * if sl >= 0.0 { 0.3 } else { -0.3 };
                }
                if self.liquid(c).is_some() {
                    precip = precip.max(10.0 * core);
                }
            }
        }
        let here = |kind: &str| {
            systems.iter().any(|s| {
                s.kind == kind
                    && s.born <= date.day
                    && date.day < s.ends
                    && distance_km(c, (lat, lon), (s.lat_deg, s.lon_deg)) <= s.size_km / 2.0
            })
        };
        if here("order-clear") {
            for l in layers.iter_mut() {
                l.cloud_cover = 0.0;
                l.cloud_type = "none".into();
                l.condensate.clear();
                l.optical_depth = 0.0;
            }
            precip = 0.0;
        }
        if here("order-calm") {
            for l in layers.iter_mut() {
                let v = m::sqrt(l.wind_m_s[0] * l.wind_m_s[0] + l.wind_m_s[1] * l.wind_m_s[1]);
                let k = if v > 1.0 { 1.0 / v } else { 1.0 };
                l.wind_m_s = [k * l.wind_m_s[0], k * l.wind_m_s[1]];
                l.turbulence = 0.0;
            }
        }
        let kind = if precip <= 0.0 {
            "none".to_string()
        } else {
            match self.liquid(c).map(|s| s.name.as_str()) {
                Some("water") if zs.t_k[j] < 273.15 => "snow".into(),
                Some("water") => "rain".into(),
                Some(l) => format!("{l} rain"),
                None => "none".into(),
            }
        };
        if kind == "none" {
            precip = 0.0;
        }
        Cell {
            lat_deg: lat,
            lon_deg: lon,
            layers,
            precip_kind: kind,
            precip_mm_day: precip,
            temp_offset_k: zs.t_k[j] - za.t_k[j],
            lightning_km2_yr: 0.0,
            aerosols: haze(&self.shared, c),
        }
    }
}

impl WeatherBlock for ZonalBlock {
    fn id(&self) -> BlockId {
        BlockId {
            id: BLOCK_ID.into(),
            version: BLOCK_VERSION.into(),
            level: "simulated".into(),
        }
    }

    fn climate(&self, c: &Conditions) -> ClimateAverages {
        circulation::averages(&self.shared, c, &self.core(c))
    }

    fn field(&self, c: &Conditions, date: Date, key: TileKey, n: u32, pins: &[Pin]) -> FieldTile {
        let zs = self.at(c, date);
        let za = self.annual(c);
        let systems = self.systems_with(c, date, pins);
        let cells = cell_dirs(key, n)
            .into_iter()
            .map(|d| {
                let (lat, lon) = lat_lon(d);
                self.cell(c, &zs, &za, date, &systems, lat, lon)
            })
            .collect();
        FieldTile { key, n, cells }
    }

    fn systems(&self, c: &Conditions, date: Date, pins: &[Pin]) -> Vec<WeatherSystem> {
        self.systems_with(c, date, pins)
    }

    fn provenance(&self) -> Provenance {
        Provenance {
            block: self.id(),
            evidence: "calculated".into(),
        }
    }
}
