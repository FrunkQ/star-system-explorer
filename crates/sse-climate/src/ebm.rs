//! The energy-balance core: a one-dimensional diffusive energy-balance model
//! in latitude (North 1975), solved season by season:
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
//! - Rain comes from the model's own temperatures: evaporation is a bulk
//!   formula on the saturation humidity (capped by the energy available),
//!   moisture diffuses down its own gradient, rain is what converges. Cloud
//!   grows with the latent heat released and with the temperature gradient
//!   (baroclinic zones). On worlds without a surface liquid, cloud forms
//!   where a gas in the air saturates at the band's temperature.
//! - Wind near the ground is the thermal wind u = −(g h / f T) ∂T/∂y, a
//!   share of it felt at the surface. f carries the spin's sign, so a
//!   retrograde world's winds blow the other way.
//!
//! This was `sse-weather`'s second block (W6); W7a moved it here so the
//! ground, the cover and both weather blocks read one climate.

use crate::conditions::{Conditions, EARTH_COLUMN_KG_M2, R_GAS};
use crate::settings::{ClimateSettings, Substance};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const EARTH_OMEGA: f64 = 2.0 * m::PI / (23.934 * 3600.0);

/// One season's (or the year's) zonal state, band by band, south to north.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Season {
    pub lat_deg: Vec<f64>,
    pub t_k: Vec<f64>,
    pub albedo: Vec<f64>,
    /// Absorbed sunlight, W/m².
    pub absorbed_w_m2: Vec<f64>,
    /// Evaporation and precipitation, kg m⁻² s⁻¹.
    pub evap: Vec<f64>,
    pub precip: Vec<f64>,
    /// Cloud from condensation (chemical decks are added by weather).
    pub cover: Vec<f64>,
    /// dT/dφ, K per degree.
    pub gradient_k_deg: Vec<f64>,
    /// Eastward near-surface wind, m/s (negative blows from the east).
    pub wind_m_s: Vec<f64>,
    /// What forms the cloud, or empty.
    pub condensate: String,
    pub d_w_m2_k: f64,
    pub b_w_m2_k: f64,
}

pub(crate) fn rad(x: f64) -> f64 {
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

/// Area-weighted mean.
pub fn weighted_mean(w: &[f64], f: &[f64]) -> f64 {
    let (mut s, mut a) = (0.0, 0.0);
    for (x, y) in w.iter().zip(f) {
        s += x * y;
        a += x;
    }
    s / a
}

/// Band centres, south to north, degrees.
pub fn lats(cs: &ClimateSettings) -> Vec<f64> {
    let n = cs.bands;
    (0..n)
        .map(|i| -90.0 + 180.0 * (i as f64 + 0.5) / n as f64)
        .collect()
}

/// The surface liquid's substance, when the world has one.
pub fn liquid<'a>(cs: &'a ClimateSettings, c: &Conditions) -> Option<&'a Substance> {
    if !c.solid_surface {
        return None;
    }
    let l = c.liquid.as_ref()?;
    cs.substances.iter().find(|s| &s.name == l)
}

/// Fraction of absorbed sunlight that reaches the ground.
pub fn surface_sun_fraction(cs: &ClimateSettings, c: &Conditions) -> f64 {
    let col = c.column_kg_m2() / EARTH_COLUMN_KG_M2;
    if col <= 0.0 {
        return 1.0;
    }
    (cs.supply.earth_fraction * m::pow(col, -cs.supply.column_power)).min(1.0)
}

/// Diffusivity and longwave slope for a world, W m⁻² K⁻¹.
pub fn coefficients(cs: &ClimateSettings, c: &Conditions) -> (f64, f64) {
    let olr = c.absorbed_w_m2() + c.internal_flux_w_m2;
    let b = cs.olr.b_hat * 4.0 * olr.max(1e-6) / c.t_surface_k.max(1.0);
    let col = (c.column_kg_m2() / EARTH_COLUMN_KG_M2).max(0.0);
    let spin = EARTH_OMEGA / c.spin_rate().max(1e-12);
    let d = cs.diffusion.d_earth_w_m2_k
        * m::pow(col, cs.diffusion.column_power)
        * m::pow(spin, cs.diffusion.spin_power);
    (d.min(cs.diffusion.max_over_b * b), b)
}

/// Steady temperature for a season's declination (no heat capacity), with
/// the moisture, cloud and wind it implies.
pub fn season(cs: &ClimateSettings, c: &Conditions, decl_deg: f64) -> Season {
    let q: Vec<f64> = lats(cs)
        .iter()
        .map(|l| insolation(c.flux_w_m2, *l, decl_deg))
        .collect();
    let (t, alb) = steady(cs, c, &q);
    diagnose(cs, c, &q, &t, &alb)
}

struct Grid {
    w: Vec<f64>,
    lo: Vec<f64>,
    up: Vec<f64>,
    b: f64,
}

fn grid(cs: &ClimateSettings, c: &Conditions) -> Grid {
    let lat = lats(cs);
    let n = lat.len();
    let dphi = m::PI / n as f64;
    let w: Vec<f64> = lat.iter().map(|l| m::cos(rad(*l))).collect();
    let ce: Vec<f64> = (0..n.saturating_sub(1))
        .map(|j| m::cos(rad(-90.0 + 180.0 * (j as f64 + 1.0) / n as f64)))
        .collect();
    let (d, b) = coefficients(cs, c);
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
    Grid { w, lo, up, b }
}

/// Ice fraction and albedo for band temperatures, with the base albedo set
/// so the sunlight-weighted albedo is the world's Bond albedo.
fn albedo(cs: &ClimateSettings, c: &Conditions, g: &Grid, q: &[f64], t: &[f64]) -> Vec<f64> {
    let water = liquid(cs, c).is_some_and(|s| s.name == "water");
    let ice: Vec<f64> = t
        .iter()
        .map(|x| {
            if water {
                0.5 * (1.0 - m::tanh((x - cs.ice.t_ice_k) / cs.ice.width_k))
            } else {
                0.0
            }
        })
        .collect();
    let q_mean = weighted_mean(&g.w, q).max(1e-12);
    let qi: Vec<f64> = q.iter().zip(&ice).map(|(a, i)| a * i).collect();
    let a0 = c.albedo - cs.ice.albedo_step * weighted_mean(&g.w, &qi) / q_mean;
    ice.iter()
        .map(|i| (a0 + cs.ice.albedo_step * i).clamp(0.0, 0.95))
        .collect()
}

/// The steady state under sunlight `q`: relaxed tridiagonal solves.
fn steady(cs: &ClimateSettings, c: &Conditions, q: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let g = grid(cs, c);
    let n = q.len();
    let di: Vec<f64> = (0..n).map(|j| g.b + g.lo[j] + g.up[j]).collect();
    let t_bar = c.t_surface_k;
    let mut t = vec![t_bar; n];
    let mut alb = vec![c.albedo; n];
    for _ in 0..cs.iterations {
        alb = albedo(cs, c, &g, q, &t);
        let absorbed: Vec<f64> = q.iter().zip(&alb).map(|(a, al)| a * (1.0 - al)).collect();
        // A so the global mean temperature is the world's own.
        let a_olr = weighted_mean(&g.w, &absorbed) + c.internal_flux_w_m2;
        let r: Vec<f64> = absorbed
            .iter()
            .map(|x| x + c.internal_flux_w_m2 - a_olr + g.b * t_bar)
            .collect();
        let tn = thomas(&g.lo, &di, &g.up, &r);
        for j in 0..n {
            t[j] += cs.relax * (tn[j] - t[j]);
        }
    }
    (t, alb)
}

/// Heat capacity of a square metre of the world for seasonal change,
/// J m⁻² K⁻¹: the sea's mixed layer, the ground's seasonal skin
/// (thermal inertia × sqrt(year / π)) and the air column above.
pub fn heat_capacity(cs: &ClimateSettings, c: &Conditions) -> f64 {
    let h = &cs.heat_capacity;
    let year_s = (c.year_days * 86_400.0).max(1.0);
    let cp_air = 3.5 * R_GAS / c.mean_molar_kg.max(1e-6);
    let air = cp_air * c.column_kg_m2().max(0.0);
    if !c.solid_surface {
        return air;
    }
    let sea = if liquid(cs, c).is_some() {
        c.ocean_fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let skin = h.ground_inertia_si * m::sqrt(year_s / m::PI);
    sea * h.water_j_m3_k * h.mixed_layer_m + (1.0 - sea) * skin + air
}

/// The seasonal cycle with heat capacity (North and Coakley 1979): start
/// from the steady state under the year's mean sunlight, step the year
/// implicitly for a fixed number of years, and return each season's mean
/// state in the last year, phase (k + ½) / n.
pub fn cycle(cs: &ClimateSettings, c: &Conditions) -> Vec<Season> {
    let hc = &cs.heat_capacity;
    let ns = cs.seasons.max(1) as usize;
    let sub = hc.steps_per_season.max(1) as usize;
    let steps = ns * sub;
    let lat = lats(cs);
    let n = lat.len();
    let g = grid(cs, c);
    let qs: Vec<Vec<f64>> = (0..steps)
        .map(|k| {
            let phase = (k as f64 + 0.5) / steps as f64;
            let decl = declination_deg(c, phase);
            let near = orbit_at(c, phase).1;
            lat.iter()
                .map(|l| insolation(c.flux_w_m2 * near, *l, decl))
                .collect()
        })
        .collect();
    let q_year: Vec<f64> = (0..n)
        .map(|j| qs.iter().map(|q| q[j]).sum::<f64>() / steps as f64)
        .collect();
    let (mut t, _) = steady(cs, c, &q_year);
    let t_bar = c.t_surface_k;
    let cap = heat_capacity(cs, c);
    let dt = (c.year_days * 86_400.0).max(1.0) / steps as f64;
    let k = cap / dt;
    let di: Vec<f64> = (0..n).map(|j| k + g.b + g.lo[j] + g.up[j]).collect();
    // A from the steady start, then from each year's own mean absorbed light.
    let alb0 = albedo(cs, c, &g, &q_year, &t);
    let abs0: Vec<f64> = q_year
        .iter()
        .zip(&alb0)
        .map(|(a, al)| a * (1.0 - al))
        .collect();
    let mut a_olr = weighted_mean(&g.w, &abs0) + c.internal_flux_w_m2 - g.b * t_bar;
    let mut out: Vec<(Vec<f64>, Vec<f64>, Vec<f64>)> = vec![];
    let years = hc.spin_up_years.max(1);
    for y in 0..years {
        let mut year_abs = 0.0;
        let last = y + 1 == years;
        let mut acc = vec![(vec![0.0; n], vec![0.0; n], vec![0.0; n]); ns];
        for (s, q) in qs.iter().enumerate() {
            let alb = albedo(cs, c, &g, q, &t);
            let absorbed: Vec<f64> = q.iter().zip(&alb).map(|(a, al)| a * (1.0 - al)).collect();
            year_abs += weighted_mean(&g.w, &absorbed);
            let r: Vec<f64> = (0..n)
                .map(|j| k * t[j] + absorbed[j] + c.internal_flux_w_m2 - a_olr)
                .collect();
            t = thomas(&g.lo, &di, &g.up, &r);
            if last {
                let e = &mut acc[s / sub];
                for j in 0..n {
                    e.0[j] += t[j] / sub as f64;
                    e.1[j] += alb[j] / sub as f64;
                    e.2[j] += q[j] / sub as f64;
                }
            }
        }
        a_olr = year_abs / steps as f64 + c.internal_flux_w_m2 - g.b * t_bar;
        if last {
            out = acc;
        }
    }
    out.iter()
        .map(|(t, alb, q)| diagnose(cs, c, q, t, alb))
        .collect()
}

/// Everything a band temperature implies: absorbed sunlight, gradients,
/// thermal wind, evaporation, rain and cloud.
pub fn diagnose(cs: &ClimateSettings, c: &Conditions, q: &[f64], t: &[f64], alb: &[f64]) -> Season {
    let lat = lats(cs);
    let n = lat.len();
    let dphi = m::PI / n as f64;
    let w: Vec<f64> = lat.iter().map(|l| m::cos(rad(*l))).collect();
    let ce: Vec<f64> = (0..n.saturating_sub(1))
        .map(|j| m::cos(rad(-90.0 + 180.0 * (j as f64 + 1.0) / n as f64)))
        .collect();
    let (d, b) = coefficients(cs, c);
    let (t, alb) = (t.to_vec(), alb.to_vec());
    let absorbed: Vec<f64> = q.iter().zip(&alb).map(|(a, al)| a * (1.0 - al)).collect();
    let grad: Vec<f64> = (0..n)
        .map(|j| {
            let (a, bb) = (j.saturating_sub(1), (j + 1).min(n - 1));
            (t[bb] - t[a]) / (lat[bb] - lat[a])
        })
        .collect();
    // Thermal wind: u = −(g h / f T) ∂T/∂y, a share of it at the ground. The
    // Coriolis parameter f carries the spin's sign.
    let h = c.scale_height_m() * cs.thermal_depth_scale_heights;
    let omega = c.omega();
    let wind: Vec<f64> = (0..n)
        .map(|j| {
            if c.pressure_bar < cs.cloud.min_pressure_bar {
                return 0.0;
            }
            let s = m::sin(rad(lat[j]));
            let f = 2.0 * omega * s.signum() * s.abs().max(m::sin(rad(10.0)));
            let dtdy = grad[j] * 180.0 / m::PI / c.radius_m;
            let u = -c.gravity_m_s2 * h / (f * t[j]) * dtdy * cs.surface_wind_share;
            u.clamp(-60.0, 60.0)
        })
        .collect();
    let (evap, precip, cover, condensate) =
        moisture(cs, c, &lat, &w, &ce, dphi, d, &t, &absorbed, &grad);
    Season {
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

/// Saturation vapour pressure, Pa. Below the triple point, over the solid,
/// from the master data sheet's vapour curve (Fray and Schmitt 2009's fits,
/// `sse.volatile.species`); above it, over the liquid, Clausius-Clapeyron
/// from the triple point with the heat of vaporisation.
pub fn p_sat(sub: &Substance, t_k: f64) -> f64 {
    if t_k < sub.triple_k {
        if let Some(p) = sse_reference::sublimation_pa(&sub.phases, t_k) {
            return p;
        }
    }
    let l_r = sub.latent_j_kg * sub.molar_kg / R_GAS;
    sub.triple_pa * m::exp(l_r * (1.0 / sub.triple_k - 1.0 / t_k))
}

/// The temperature at which `sub`'s vapour saturates at `p_pa`, K: the
/// inverse of [`p_sat`] (bisection in ln T on the solid's curve, a fixed 60
/// steps so every device agrees; closed form over the liquid).
pub fn frost_k(sub: &Substance, p_pa: f64) -> f64 {
    let l_r = sub.latent_j_kg * sub.molar_kg / R_GAS;
    let liquid = || 1.0 / (1.0 / sub.triple_k - m::ln(p_pa / sub.triple_pa) / l_r);
    if p_pa >= sub.triple_pa || sse_reference::sublimation_pa(&sub.phases, sub.triple_k).is_none() {
        return liquid();
    }
    let target = m::ln(p_pa.max(1e-300));
    let (mut lo, mut hi) = (m::ln(1.0), m::ln(sub.triple_k));
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if m::ln(p_sat(sub, m::exp(mid)).max(1e-300)) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    m::exp(0.5 * (lo + hi))
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn moisture(
    cs: &ClimateSettings,
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
    let n = lat.len();
    let zero = vec![0.0; n];
    if c.pressure_bar < cs.cloud.min_pressure_bar {
        return (zero.clone(), zero.clone(), zero, String::new());
    }
    let p0 = c.pressure_bar * 1.0e5;
    if let Some(sub) = liquid(cs, c) {
        let rh = cs.moisture.humidity;
        let qsat: Vec<f64> = t
            .iter()
            .map(|x| (sub.molar_kg / c.mean_molar_kg * p_sat(sub, *x) / p0).min(1.0))
            .collect();
        let wet = m::sqrt((c.ocean_fraction / cs.supply.wet_saturation).min(1.0));
        let ssf = surface_sun_fraction(cs, c);
        let evap: Vec<f64> = (0..n)
            .map(|j| {
                let rho = p0 * c.mean_molar_kg / (R_GAS * t[j]);
                let bulk = rho * cs.moisture.exchange_m_s * qsat[j] * (1.0 - rh);
                let energy = cs.supply.latent_share * ssf * absorbed[j].max(0.0) / sub.latent_j_kg;
                wet * bulk.min(energy)
            })
            .collect();
        // Moisture as a temperature (L q / c_p) diffuses down its gradient.
        let cp = 3.5 * R_GAS / c.mean_molar_kg;
        let hq: Vec<f64> = qsat
            .iter()
            .map(|x| sub.latent_j_kg * rh * x / cp * wet)
            .collect();
        // Moisture transport (Siler, Roe and Armour 2018): eddies carry it
        // down its gradient; the Hadley cell's lower branch carries it toward
        // the rising air. The Hadley share of the total moist static energy
        // flux is w = exp(−x² / σ²) in x = sin φ; its latent part is
        // −F_HC · (L q / c_p) / g, with gross moist stability g = h_T − h and
        // h_T a fixed ratio above the equator's h (Neelin and Held 1987).
        let mo = &cs.moisture;
        let hadley = crate::circulation::circulation(cs, c).hadley_edge_deg;
        let sigma = mo.hadley_width_x * m::sin(rad(hadley.clamp(1.0, 90.0)))
            / m::sin(rad(mo.hadley_reference_deg));
        let h: Vec<f64> = (0..n).map(|j| t[j] + hq[j]).collect();
        // The rising branch sits where the moist static energy peaks this
        // season (the energy-flux equator), not on the geographic equator, so
        // on a steeply tilted world the rain belt chases the overhead sun.
        // The peak is placed between bands by a parabola through the top three.
        let jm = (0..n).fold(0, |b, j| if h[j] > h[b] { j } else { b });
        let (x0, h_peak) = if jm > 0 && jm + 1 < n {
            let (a, b, cc) = (h[jm - 1], h[jm], h[jm + 1]);
            let den = a - 2.0 * b + cc;
            let off = if den < 0.0 { 0.5 * (a - cc) / den } else { 0.0 };
            let off = off.clamp(-0.5, 0.5);
            let lat0 = lat[jm] + off * (lat[1] - lat[0]);
            (m::sin(rad(lat0)), b - 0.25 * (a - cc) * off)
        } else {
            (m::sin(rad(lat[jm])), h[jm])
        };
        let h_top = mo.h_top_ratio * h_peak;
        // Northward fluxes at the band edges, in the temperature units the
        // energy balance diffuses (times D), zero at the poles.
        let flux_q: Vec<f64> = (0..n.saturating_sub(1))
            .map(|j| {
                let x = m::sin(rad(-90.0 + 180.0 * (j as f64 + 1.0) / n as f64));
                let wh = if sigma > 0.0 {
                    m::exp(-((x - x0) * (x - x0)) / (sigma * sigma))
                } else {
                    0.0
                };
                let eddy = -ce[j] * (hq[j + 1] - hq[j]) / dphi;
                let total = -ce[j] * (h[j + 1] - h[j]) / dphi;
                let hq_e = 0.5 * (hq[j] + hq[j + 1]);
                let g = (h_top - 0.5 * (h[j] + h[j + 1])).max(mo.min_stability_k);
                d * ((1.0 - wh) * mo.eddy_ratio * eddy - wh * total * hq_e / g)
            })
            .collect();
        let conv: Vec<f64> = (0..n)
            .map(|j| {
                let up = if j + 1 < n { flux_q[j] } else { 0.0 };
                let dn = if j > 0 { flux_q[j - 1] } else { 0.0 };
                -(up - dn) / (w[j] * dphi)
            })
            .collect();
        let mut precip: Vec<f64> = (0..n)
            .map(|j| (evap[j] + conv[j] / sub.latent_j_kg).max(0.0))
            .collect();
        // Clipping dry bands must not create or lose water.
        let (pe, pp) = (weighted_mean(w, &evap), weighted_mean(w, &precip));
        if pp > 0.0 {
            for x in precip.iter_mut() {
                *x *= pe / pp;
            }
        }
        let cl = &cs.cloud;
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
    let cl = &cs.cloud;
    let mut cover = zero.clone();
    let mut best = (0.0, String::new());
    for sub in &cs.substances {
        let x = c.gases.get(&sub.gas).copied().unwrap_or(0.0);
        if x <= 0.0 {
            continue;
        }
        let mut area = 0.0;
        for j in 0..n {
            let s = x * p0 / p_sat(sub, t[j]).max(1e-300);
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

/// Sun's latitude overhead on this fraction of the year (0 = northern spring).
pub fn declination_deg(c: &Conditions, year_phase: f64) -> f64 {
    let e = c.obliquity_deg * m::PI / 180.0;
    m::asin(m::sin(e) * m::sin(rad(orbit_at(c, year_phase).0))) * 180.0 / m::PI
}

/// Where the world is at a fraction of its year counted from the northern
/// spring equinox (so dates keep their seasons whatever the orbit): its
/// solar longitude (degrees from that equinox) and the sunlight factor
/// (a / r)². Kepler's equation M = E − e sin E by Newton's method (a fixed
/// 12 steps, so every device gets the same answer).
pub fn orbit_at(c: &Conditions, year_phase: f64) -> (f64, f64) {
    let e = c.eccentricity.clamp(0.0, 0.95);
    // Mean anomaly at the equinox, where the true anomaly is −ϖ.
    let nu0 = -rad(c.perihelion_lon_deg);
    let e0 = 2.0
        * m::atan2(
            m::sqrt(1.0 - e) * m::sin(nu0 / 2.0),
            m::sqrt(1.0 + e) * m::cos(nu0 / 2.0),
        );
    let mm = e0 - e * m::sin(e0) + 2.0 * m::PI * year_phase;
    let mut ea = if e > 0.8 { m::PI } else { mm };
    for _ in 0..12 {
        ea -= (ea - e * m::sin(ea) - mm) / (1.0 - e * m::cos(ea));
    }
    let nu = 2.0
        * m::atan2(
            m::sqrt(1.0 + e) * m::sin(ea / 2.0),
            m::sqrt(1.0 - e) * m::cos(ea / 2.0),
        );
    let r = 1.0 - e * m::cos(ea);
    (nu * 180.0 / m::PI + c.perihelion_lon_deg, 1.0 / (r * r))
}

/// The mean of several seasons, band by band.
pub fn mean(seasons: &[Season]) -> Season {
    let mut a = seasons[0].clone();
    let add = |x: &mut Vec<f64>, y: &[f64]| {
        for (p, q) in x.iter_mut().zip(y) {
            *p += q;
        }
    };
    for s in &seasons[1..] {
        add(&mut a.t_k, &s.t_k);
        add(&mut a.albedo, &s.albedo);
        add(&mut a.absorbed_w_m2, &s.absorbed_w_m2);
        add(&mut a.evap, &s.evap);
        add(&mut a.precip, &s.precip);
        add(&mut a.cover, &s.cover);
        add(&mut a.gradient_k_deg, &s.gradient_k_deg);
        add(&mut a.wind_m_s, &s.wind_m_s);
        if a.condensate.is_empty() {
            a.condensate = s.condensate.clone();
        }
    }
    let k = 1.0 / seasons.len() as f64;
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
