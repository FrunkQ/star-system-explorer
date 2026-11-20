//! W6: a second, deliberately different weather block behind the same
//! contract, `sse.weather.zonal-ebm`. Where the rule block draws belts from
//! the circulation regime and wetness bumps, this one solves a
//! one-dimensional diffusive energy-balance model in latitude (North 1975):
//!
//! C dT/dt = Q(φ)(1 − α(T)) − (A + B T) + D (1/cos φ) ∂/∂φ (cos φ ∂T/∂φ)
//!
//! - Q(φ) is the daily-mean sunlight at the season's declination; each
//!   season is solved to its steady state (dT/dt = 0) by a fixed number of
//!   relaxed tridiagonal solves, so every device gives the same answer.
//! - Outgoing longwave is linear in temperature (Budyko 1969). B is the
//!   world's blackbody slope 4 OLR / T scaled by a dimensionless factor taken
//!   from Budyko's Earth fit; A is set so the global mean temperature is the
//!   world's own, and the base albedo so the sunlight-weighted albedo is the
//!   world's Bond albedo. The model moves heat and ice; it does not change
//!   how warm or bright a world is.
//! - Clouds and rain come from the model's own temperatures: evaporation is
//!   a bulk formula on the saturation humidity (capped by the energy
//!   available), moisture diffuses down its own gradient, rain is what
//!   converges, cloud grows with the latent heat released and with the
//!   temperature gradient (baroclinic zones). On worlds without a surface
//!   liquid, cloud forms where a gas in the air saturates at the band's
//!   temperature. Chemical decks and hazes are chemistry, not weather: both
//!   blocks read the same pack rules for them.
//! - Storms: one simple rule, a cyclone every few days in each hemisphere at
//!   the steepest temperature gradient when the world spins fast enough.
//!   Pins are honoured exactly as the rule block honours them.

use crate::circulation;
use crate::conditions::{Conditions, EARTH_COLUMN_KG_M2};
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

const R_GAS: f64 = 8.314462618;
const EARTH_OMEGA: f64 = 2.0 * m::PI / (23.934 * 3600.0);
const YEAR_S: f64 = 3.15576e7;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Olr {
    /// B as a fraction of the world's blackbody slope 4 OLR / T.
    pub b_hat: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Diffusion {
    /// D on Earth, W m⁻² K⁻¹.
    pub d_earth_w_m2_k: f64,
    /// D scales as (column / Earth's)^`column_power` × (Ω_Earth / Ω)^`spin_power`.
    pub column_power: f64,
    pub spin_power: f64,
    /// Upper bound on D / B (beyond it the world is isothermal anyway).
    pub max_over_b: f64,
    /// Moisture diffuses with this multiple of D.
    pub moisture_ratio: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ice {
    /// Albedo added where water freezes over, smoothly around `t_ice_k`.
    pub albedo_step: f64,
    pub t_ice_k: f64,
    pub width_k: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Moisture {
    /// Bulk exchange velocity C_E |U|, m/s.
    pub exchange_m_s: f64,
    /// Relative humidity of the air over the liquid.
    pub humidity: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cloud {
    /// Cover from latent heat released: `cover_max` (1 − exp(−L P / `latent_scale_w_m2`)).
    pub cover_max: f64,
    pub latent_scale_w_m2: f64,
    /// Extra cover in baroclinic zones: `gradient_cover` × min(1, |dT/dφ| / `gradient_scale_k_deg`).
    pub gradient_cover: f64,
    pub gradient_scale_k_deg: f64,
    /// Worlds without a surface liquid: cover where a gas saturates,
    /// `saturation_cover` × min(1, ln(p / p_sat) / `saturation_ln_scale`).
    pub saturation_cover: f64,
    pub saturation_ln_scale: f64,
    /// Below this surface pressure there is no cloud at all, bar.
    pub min_pressure_bar: f64,
    pub source: String,
}

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
    pub bands: u32,
    pub seasons: u32,
    pub iterations: u32,
    pub relax: f64,
    pub olr: Olr,
    pub diffusion: Diffusion,
    pub ice: Ice,
    pub moisture: Moisture,
    pub cloud: Cloud,
    pub storms: Storms,
    /// Thermal wind depth in scale heights, and the share of it felt near the ground.
    pub thermal_depth_scale_heights: f64,
    pub surface_wind_share: f64,
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
        if s.bands < 4 || s.seasons == 0 || s.iterations == 0 {
            return Err("weather_zonal needs at least 4 bands, 1 season and 1 iteration".into());
        }
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

/// One season's (or the year's) zonal state, band by band, south to north.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Zonal {
    pub lat_deg: Vec<f64>,
    pub t_k: Vec<f64>,
    pub albedo: Vec<f64>,
    /// Absorbed sunlight, W/m².
    pub absorbed_w_m2: Vec<f64>,
    /// Evaporation and precipitation, kg m⁻² s⁻¹.
    pub evap: Vec<f64>,
    pub precip: Vec<f64>,
    pub cover: Vec<f64>,
    /// dT/dφ, K per degree.
    pub gradient_k_deg: Vec<f64>,
    /// Eastward near-surface wind, m/s.
    pub wind_m_s: Vec<f64>,
    /// What forms the cloud, or empty.
    pub condensate: String,
    pub d_w_m2_k: f64,
    pub b_w_m2_k: f64,
}

fn rad(x: f64) -> f64 {
    x * m::PI / 180.0
}

/// Daily-mean sunlight at a latitude for a declination, W/m².
pub fn insolation(s0: f64, lat_deg: f64, decl_deg: f64) -> f64 {
    let (p, d) = (rad(lat_deg), rad(decl_deg));
    let x = -m::tan(p) * m::tan(d);
    let h0 = if x >= 1.0 {
        0.0
    } else if x <= -1.0 {
        m::PI
    } else {
        m::acos(x)
    };
    s0 / m::PI * (h0 * m::sin(p) * m::sin(d) + m::cos(p) * m::cos(d) * m::sin(h0))
}

/// Solve the tridiagonal system `-lo[j] x[j-1] + di[j] x[j] - up[j] x[j+1] = r[j]`.
fn thomas(lo: &[f64], di: &[f64], up: &[f64], r: &[f64]) -> Vec<f64> {
    let n = di.len();
    let mut c = vec![0.0; n];
    let mut d = vec![0.0; n];
    c[0] = -up[0] / di[0];
    d[0] = r[0] / di[0];
    for j in 1..n {
        let den = di[j] + lo[j] * c[j - 1];
        c[j] = -up[j] / den;
        d[j] = (r[j] + lo[j] * d[j - 1]) / den;
    }
    let mut x = vec![0.0; n];
    x[n - 1] = d[n - 1];
    for j in (0..n - 1).rev() {
        x[j] = d[j] - c[j] * x[j + 1];
    }
    x
}

/// Discrete (1/cos φ) ∂/∂φ (cos φ ∂f/∂φ), per radian², for band values `f`.
fn laplacian(w: &[f64], ce: &[f64], dphi: f64, f: &[f64]) -> Vec<f64> {
    let n = f.len();
    (0..n)
        .map(|j| {
            let up = if j + 1 < n {
                ce[j] * (f[j + 1] - f[j])
            } else {
                0.0
            };
            let dn = if j > 0 {
                ce[j - 1] * (f[j] - f[j - 1])
            } else {
                0.0
            };
            (up - dn) / (w[j] * dphi * dphi)
        })
        .collect()
}

fn weighted_mean(w: &[f64], f: &[f64]) -> f64 {
    let (mut s, mut a) = (0.0, 0.0);
    for (x, y) in w.iter().zip(f) {
        s += x * y;
        a += x;
    }
    s / a
}

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

    fn lats(&self) -> Vec<f64> {
        let n = self.z.bands;
        (0..n)
            .map(|i| -90.0 + 180.0 * (i as f64 + 0.5) / n as f64)
            .collect()
    }

    fn liquid<'a>(&'a self, c: &Conditions) -> Option<&'a Substance> {
        if !c.solid_surface {
            return None;
        }
        let l = c.liquid.as_ref()?;
        self.shared.substances.iter().find(|s| &s.name == l)
    }

    /// Diffusivity and longwave slope for a world, W m⁻² K⁻¹.
    pub fn coefficients(&self, c: &Conditions) -> (f64, f64) {
        let z = &self.z;
        let olr = c.absorbed_w_m2() + c.internal_flux_w_m2;
        let b = z.olr.b_hat * 4.0 * olr.max(1e-6) / c.t_surface_k.max(1.0);
        let col = (c.column_kg_m2() / EARTH_COLUMN_KG_M2).max(0.0);
        let spin = EARTH_OMEGA / c.omega().max(1e-12);
        let d = z.diffusion.d_earth_w_m2_k
            * m::pow(col, z.diffusion.column_power)
            * m::pow(spin, z.diffusion.spin_power);
        (d.min(z.diffusion.max_over_b * b), b)
    }

    /// Steady temperature for a season's declination, with the moisture,
    /// cloud and wind it implies.
    pub fn season(&self, c: &Conditions, decl_deg: f64) -> Zonal {
        let z = &self.z;
        let lat = self.lats();
        let n = lat.len();
        let dphi = m::PI / n as f64;
        let w: Vec<f64> = lat.iter().map(|l| m::cos(rad(*l))).collect();
        let ce: Vec<f64> = (0..n.saturating_sub(1))
            .map(|j| m::cos(rad(-90.0 + 180.0 * (j as f64 + 1.0) / n as f64)))
            .collect();
        let q: Vec<f64> = lat
            .iter()
            .map(|l| insolation(c.flux_w_m2, *l, decl_deg))
            .collect();
        let (d, b) = self.coefficients(c);
        let t_bar = c.t_surface_k;
        let water = self.liquid(c).is_some_and(|s| s.name == "water");
        // The coupling to each neighbour, per band.
        let lo: Vec<f64> = (0..n)
            .map(|j| {
                if j > 0 {
                    d * ce[j - 1] / (w[j] * dphi * dphi)
                } else {
                    0.0
                }
            })
            .collect();
        let up: Vec<f64> = (0..n)
            .map(|j| {
                if j + 1 < n {
                    d * ce[j] / (w[j] * dphi * dphi)
                } else {
                    0.0
                }
            })
            .collect();
        let di: Vec<f64> = (0..n).map(|j| b + lo[j] + up[j]).collect();
        let mut t = vec![t_bar; n];
        let mut alb = vec![c.albedo; n];
        let q_mean = weighted_mean(&w, &q).max(1e-12);
        for _ in 0..z.iterations {
            let ice: Vec<f64> = t
                .iter()
                .map(|x| {
                    if water {
                        0.5 * (1.0 - m::tanh((x - z.ice.t_ice_k) / z.ice.width_k))
                    } else {
                        0.0
                    }
                })
                .collect();
            // Base albedo so the sunlight-weighted albedo is the world's own.
            let qi: Vec<f64> = q.iter().zip(&ice).map(|(a, i)| a * i).collect();
            let a0 = c.albedo - z.ice.albedo_step * weighted_mean(&w, &qi) / q_mean;
            alb = ice
                .iter()
                .map(|i| (a0 + z.ice.albedo_step * i).clamp(0.0, 0.95))
                .collect();
            let absorbed: Vec<f64> = q.iter().zip(&alb).map(|(a, al)| a * (1.0 - al)).collect();
            // A so the global mean temperature is the world's own.
            let a_olr = weighted_mean(&w, &absorbed) + c.internal_flux_w_m2;
            let r: Vec<f64> = absorbed
                .iter()
                .map(|x| x + c.internal_flux_w_m2 - a_olr + b * t_bar)
                .collect();
            let tn = thomas(&lo, &di, &up, &r);
            for j in 0..n {
                t[j] += z.relax * (tn[j] - t[j]);
            }
        }
        let absorbed: Vec<f64> = q.iter().zip(&alb).map(|(a, al)| a * (1.0 - al)).collect();
        let grad: Vec<f64> = (0..n)
            .map(|j| {
                let (a, bb) = (j.saturating_sub(1), (j + 1).min(n - 1));
                (t[bb] - t[a]) / (lat[bb] - lat[a])
            })
            .collect();
        // Thermal wind: u = −(g h / f T) ∂T/∂y, a share of it at the ground.
        let h = c.scale_height_m() * z.thermal_depth_scale_heights;
        let omega = c.omega();
        let wind: Vec<f64> = (0..n)
            .map(|j| {
                if c.pressure_bar < z.cloud.min_pressure_bar {
                    return 0.0;
                }
                let s = m::sin(rad(lat[j]));
                let f = 2.0 * omega * s.signum() * s.abs().max(m::sin(rad(10.0)));
                let dtdy = grad[j] * 180.0 / m::PI / c.radius_m;
                let u = -c.gravity_m_s2 * h / (f * t[j]) * dtdy * z.surface_wind_share;
                u.clamp(-60.0, 60.0)
            })
            .collect();
        let (evap, precip, mut cover, mut condensate) =
            self.moisture(c, &lat, &w, &ce, dphi, d, &t, &absorbed, &grad);
        let deck = circulation::deck_cover(&self.shared, c);
        if deck > 0.0 {
            for x in cover.iter_mut() {
                *x = x.max(deck);
            }
            condensate = deck_substance(&self.shared, c).unwrap_or(condensate);
        }
        Zonal {
            lat_deg: lat,
            t_k: t,
            albedo: alb,
            absorbed_w_m2: absorbed,
            evap,
            precip,
            cover,
            gradient_k_deg: grad,
            wind_m_s: wind,
            condensate,
            d_w_m2_k: d,
            b_w_m2_k: b,
        }
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn moisture(
        &self,
        c: &Conditions,
        lat: &[f64],
        w: &[f64],
        ce: &[f64],
        dphi: f64,
        d: f64,
        t: &[f64],
        absorbed: &[f64],
        grad: &[f64],
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>, String) {
        let z = &self.z;
        let n = lat.len();
        let zero = vec![0.0; n];
        if c.pressure_bar < z.cloud.min_pressure_bar {
            return (zero.clone(), zero.clone(), zero, String::new());
        }
        let p0 = c.pressure_bar * 1.0e5;
        if let Some(sub) = self.liquid(c) {
            let rh = z.moisture.humidity;
            let qsat: Vec<f64> = t
                .iter()
                .map(|x| {
                    (sub.molar_kg / c.mean_molar_kg * circulation::p_sat(sub, *x) / p0).min(1.0)
                })
                .collect();
            let wet = m::sqrt((c.ocean_fraction / self.shared.supply.wet_saturation).min(1.0));
            let ssf = circulation::surface_sun_fraction(&self.shared, c);
            let evap: Vec<f64> = (0..n)
                .map(|j| {
                    let rho = p0 * c.mean_molar_kg / (R_GAS * t[j]);
                    let bulk = rho * z.moisture.exchange_m_s * qsat[j] * (1.0 - rh);
                    let energy = self.shared.supply.latent_share * ssf * absorbed[j].max(0.0)
                        / sub.latent_j_kg;
                    wet * bulk.min(energy)
                })
                .collect();
            // Moisture as a temperature (L q / c_p) diffuses down its gradient.
            let cp = 3.5 * R_GAS / c.mean_molar_kg;
            let hq: Vec<f64> = qsat
                .iter()
                .map(|x| sub.latent_j_kg * rh * x / cp * wet)
                .collect();
            let conv = laplacian(w, ce, dphi, &hq);
            let mut precip: Vec<f64> = (0..n)
                .map(|j| {
                    (evap[j] + z.diffusion.moisture_ratio * d * conv[j] / sub.latent_j_kg).max(0.0)
                })
                .collect();
            // Clipping dry bands must not create or lose water.
            let (pe, pp) = (weighted_mean(w, &evap), weighted_mean(w, &precip));
            if pp > 0.0 {
                for x in precip.iter_mut() {
                    *x *= pe / pp;
                }
            }
            let cl = &z.cloud;
            let cover: Vec<f64> = (0..n)
                .map(|j| {
                    let latent = sub.latent_j_kg * precip[j];
                    let a = cl.cover_max * (1.0 - m::exp(-latent / cl.latent_scale_w_m2));
                    let g = cl.gradient_cover * (grad[j].abs() / cl.gradient_scale_k_deg).min(1.0);
                    (a + if latent > 0.0 { g } else { 0.0 }).min(1.0)
                })
                .collect();
            let name = if cover.iter().any(|x| *x > 0.0) {
                sub.name.clone()
            } else {
                String::new()
            };
            return (evap, precip, cover, name);
        }
        // No surface liquid: cloud where a gas of the air saturates.
        let cl = &z.cloud;
        let mut cover = zero.clone();
        let mut best = (0.0, String::new());
        for sub in &self.shared.substances {
            let x = c.gases.get(&sub.gas).copied().unwrap_or(0.0);
            if x <= 0.0 {
                continue;
            }
            let mut area = 0.0;
            for j in 0..n {
                let s = x * p0 / circulation::p_sat(sub, t[j]).max(1e-300);
                if s > 1.0 {
                    let k = cl.saturation_cover * (m::ln(s) / cl.saturation_ln_scale).min(1.0);
                    if k > cover[j] {
                        cover[j] = k;
                    }
                    area += k * w[j];
                }
            }
            if area > best.0 {
                best = (area, sub.name.clone());
            }
        }
        (zero.clone(), zero, cover, best.1)
    }

    /// The zonal state on a date (the season's steady state).
    pub fn at(&self, c: &Conditions, date: Date) -> Zonal {
        let phase = year_phase(c, date);
        self.season(c, circulation::declination_deg(c, phase))
    }

    /// Annual means over the pack's seasons.
    pub fn annual(&self, c: &Conditions) -> Zonal {
        let ns = self.z.seasons;
        let mut acc: Option<Zonal> = None;
        for k in 0..ns {
            let phase = (k as f64 + 0.5) / ns as f64;
            let s = self.season(c, circulation::declination_deg(c, phase));
            acc = Some(match acc {
                None => s,
                Some(mut a) => {
                    let add = |x: &mut Vec<f64>, y: &[f64]| {
                        for (p, q) in x.iter_mut().zip(y) {
                            *p += q;
                        }
                    };
                    add(&mut a.t_k, &s.t_k);
                    add(&mut a.albedo, &s.albedo);
                    add(&mut a.absorbed_w_m2, &s.absorbed_w_m2);
                    add(&mut a.evap, &s.evap);
                    add(&mut a.precip, &s.precip);
                    add(&mut a.cover, &s.cover);
                    add(&mut a.gradient_k_deg, &s.gradient_k_deg);
                    add(&mut a.wind_m_s, &s.wind_m_s);
                    if a.condensate.is_empty() {
                        a.condensate = s.condensate;
                    }
                    a
                }
            });
        }
        let mut a = acc.expect("at least one season");
        let k = 1.0 / ns as f64;
        for v in [
            &mut a.t_k,
            &mut a.albedo,
            &mut a.absorbed_w_m2,
            &mut a.evap,
            &mut a.precip,
            &mut a.cover,
            &mut a.gradient_k_deg,
            &mut a.wind_m_s,
        ] {
            for x in v.iter_mut() {
                *x *= k;
            }
        }
        a
    }

    fn liquid_density(&self, c: &Conditions) -> f64 {
        self.liquid(c).map(|s| s.condensed_kg_m3).unwrap_or(1000.0)
    }

    /// The drawn cyclones alive on a date, before pins.
    pub fn drawn(&self, c: &Conditions, date: Date) -> Vec<WeatherSystem> {
        let st = &self.z.storms;
        if c.star_locked
            || c.solar_day_hours() >= st.max_day_hours
            || c.pressure_bar < self.z.cloud.min_pressure_bar
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

fn deck_substance(s: &WeatherSettings, c: &Conditions) -> Option<String> {
    s.photochemical
        .iter()
        .find(|p| {
            p.cloud_type == "deck"
                && c.flux_w_m2 >= p.min_flux_w_m2
                && c.pressure_bar >= p.min_pressure_bar
                && p.needs
                    .iter()
                    .all(|(g, x)| c.gases.get(g).copied().unwrap_or(0.0) >= *x)
        })
        .map(|p| p.makes.clone())
}

/// Wet maxima from pole to equator in the northern half, at least one.
fn wet_maxima(p: &[f64]) -> u32 {
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

impl WeatherBlock for ZonalBlock {
    fn id(&self) -> BlockId {
        BlockId {
            id: BLOCK_ID.into(),
            version: BLOCK_VERSION.into(),
            level: "simulated".into(),
        }
    }

    fn climate(&self, c: &Conditions) -> ClimateAverages {
        let a = self.annual(c);
        let rho_l = self.liquid_density(c);
        let w: Vec<f64> = a.lat_deg.iter().map(|l| m::cos(rad(*l))).collect();
        let to_m_yr = YEAR_S / rho_l;
        let bands = (0..a.lat_deg.len())
            .map(|j| Band {
                lat_deg: a.lat_deg[j],
                cloud_cover: a.cover[j],
                precip_m_yr: a.precip[j] * to_m_yr,
                wind_m_s: a.wind_m_s[j],
            })
            .collect();
        ClimateAverages {
            // The regime is a classification of the facts, shared by blocks.
            regime: circulation::circulation(&self.shared, c).regime,
            bands,
            cloud_cover_mean: weighted_mean(&w, &a.cover),
            precip_mean_m_yr: weighted_mean(&w, &a.precip) * to_m_yr,
            condensate: a.condensate.clone(),
            rain_cells: wet_maxima(&a.precip),
            surface_sun_fraction: circulation::surface_sun_fraction(&self.shared, c),
        }
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
