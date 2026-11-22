//! `sse.wind.saltation` (W9, weather review section 4): how often the wind
//! near the ground moves sand, on any world. Shared with geology's dune flag
//! (S3 wear) and, from W10, weather's dust.
//!
//! - **Fluid threshold** (the wind that starts grains hopping), after Shao and
//!   Lu 2000 as given by Kok et al. 2012 (eq. 2.8):
//!   u*ft = A_N sqrt(ρp g D / ρa + γ / (ρa D)), A_N = 0.111 (Shao and Lu's
//!   0.0123 inside the root).
//! - **Impact threshold** (the wind that keeps them hopping once started),
//!   Kok 2010: about 0.82 of the fluid threshold for loose sand on Earth, and
//!   an order of magnitude below it on Mars, where the impact threshold is
//!   "comparable to that on Earth". We hold it at its dense-air value: the
//!   impact threshold is the fluid threshold's 0.82 evaluated with the air no
//!   thinner than `impact_reference_kg_m3` (a registered fudge).
//! - Wind at `wind_height_m` from the friction speed by the law of the wall,
//!   U = (u*/κ) ln(z/z0), with z0 = D/30 (Kok et al. 2012).
//! - **Gusts**: a Weibull distribution of the wind about its mean,
//!   P(U > Ur) = exp(−[(Ur/Ū) Γ(1 + 1/k)]^k), k = 2 (Kok 2010 eq. 1).
//! - **Lofting**: where the air's own main gas freezes onto the ground in some
//!   season, its ice sublimes in sunlight and lifts grains without wind
//!   (Telfer et al. 2018, Pluto), so only the impact threshold must be met.
//! - **Transport share**: the share of the time sand moves. Once moving, sand
//!   keeps moving while the wind stays above the impact threshold (Kok 2010's
//!   hysteresis), so the share is P(>impact) wherever transport starts at all:
//!   lofted grains start it, and otherwise the chance that at least one of the
//!   year's ten-minute winds (the law of the wall's averaging time, Kok et al.
//!   2012) passes the fluid threshold, 1 − exp(−P(>fluid) × N). Taking one
//!   start a year as enough is ours (docs/fudges.md).
//!
//! There is no pressure cut: the Moon has no wind, so no saltation; Pluto at
//! 10 µbar has a little.

use crate::climate::Climate;
use crate::conditions::R_GAS;
use crate::settings::{Grain, SaltationSettings};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Saltation {
    pub grain: String,
    pub air_kg_m3: f64,
    /// Friction speeds that start and keep sand moving, m/s.
    pub fluid_u_star_m_s: f64,
    pub impact_u_star_m_s: f64,
    /// The same as winds at the reference height, m/s.
    pub fluid_wind_m_s: f64,
    pub impact_wind_m_s: f64,
    /// Mean wind at the reference height, m/s.
    pub mean_wind_m_s: f64,
    /// Shares of the time the wind passes each threshold.
    pub initiation: f64,
    pub sustained: f64,
    /// 0 to 1: grains lifted by subliming ice.
    pub lofting: f64,
    /// Share of the time sand moves (0 to 1).
    pub transport_share: f64,
}

fn u_star(ss: &SaltationSettings, air: f64, g: f64, gr: &Grain) -> f64 {
    let d = gr.diameter_m.max(1e-9);
    ss.a_n * m::sqrt(gr.density_kg_m3 * g * d / air + ss.cohesion_n_m / (air * d))
}

/// Wind at the reference height for a friction speed (law of the wall).
pub fn wind_for(ss: &SaltationSettings, u_star: f64, gr: &Grain) -> f64 {
    let z0 = gr.diameter_m.max(1e-9) * ss.roughness_per_diameter;
    u_star / ss.karman * m::ln(ss.wind_height_m / z0)
}

/// Share of the time a Weibull wind of mean `mean` blows above `ur`.
pub fn above(ss: &SaltationSettings, ur: f64, mean: f64) -> f64 {
    if mean <= 0.0 || !mean.is_finite() {
        return 0.0;
    }
    let k = ss.weibull_k.max(0.1);
    let x = ur / mean * m::tgamma(1.0 + 1.0 / k);
    m::exp(-m::pow(x.max(0.0), k))
}

/// Saltation for an air density, gravity, grain, mean wind and lofting.
pub fn saltation(
    ss: &SaltationSettings,
    air_kg_m3: f64,
    gravity_m_s2: f64,
    grain: &Grain,
    mean_wind_m_s: f64,
    lofting: f64,
) -> Saltation {
    let g = gravity_m_s2.max(1e-9);
    let (fluid, impact, ini, sus) = if air_kg_m3 > 0.0 {
        let fluid = u_star(ss, air_kg_m3, g, grain);
        let impact =
            ss.impact_ratio * u_star(ss, air_kg_m3.max(ss.impact_reference_kg_m3), g, grain);
        let impact = impact.min(ss.impact_ratio * fluid);
        let ini = above(ss, wind_for(ss, fluid, grain), mean_wind_m_s);
        let sus = above(ss, wind_for(ss, impact, grain), mean_wind_m_s);
        (fluid, impact, ini, sus)
    } else {
        (f64::INFINITY, f64::INFINITY, 0.0, 0.0)
    };
    let lofting = lofting.clamp(0.0, 1.0);
    let start = 1.0 - m::exp(-ini * ss.gusts_per_year);
    let share = sus * start.max(lofting);
    Saltation {
        grain: grain.name.clone(),
        air_kg_m3,
        fluid_u_star_m_s: fluid,
        impact_u_star_m_s: impact,
        fluid_wind_m_s: if fluid.is_finite() {
            wind_for(ss, fluid, grain)
        } else {
            fluid
        },
        impact_wind_m_s: if impact.is_finite() {
            wind_for(ss, impact, grain)
        } else {
            impact
        },
        mean_wind_m_s,
        initiation: ini,
        sustained: sus,
        lofting,
        transport_share: share,
    }
}

/// Air density at the ground, kg/m³, from a band's temperature.
pub fn air_density(cl: &Climate, t_k: f64) -> f64 {
    let c = &cl.conditions;
    if !c.solid_surface || c.pressure_bar <= 0.0 {
        return 0.0;
    }
    c.pressure_bar * 1.0e5 * c.mean_molar_kg / (R_GAS * t_k.max(1.0))
}

/// The mean wind at the reference height at a latitude: a share
/// (`ten_metre_share`) of the climate's near-ground zonal wind together with
/// the eddies' gusts, a share of the eddy wind that falls as the cube root of
/// the air's density over Earth's (the same power carries a slower wind in
/// denser air), scaled by how well the air acts as a fluid.
pub fn mean_wind(ss: &SaltationSettings, cl: &Climate, lat_deg: f64) -> f64 {
    let st = &cl.circulation.strengths;
    let u = cl.wind_at(lat_deg);
    let rho = air_density(cl, cl.t_at(lat_deg));
    let dense = if rho > 0.0 {
        m::cbrt(ss.impact_reference_kg_m3 / rho).min(1.0)
    } else {
        0.0
    };
    let gust = ss.gust_share * st.eddy_wind_m_s * dense * st.collisional;
    ss.ten_metre_share * m::sqrt(u * u + gust * gust)
}

/// Lofting at a latitude: ice of the air's main gas sublimes from the
/// ground there. With the exchange (W8) it is the share of the year's
/// seasons in which the band holds ice and loses it to the air; without
/// one, whether the coldest season reaches the frost point (smoothly, over
/// `lofting_width` of it).
pub fn lofting(ss: &SaltationSettings, cl: &Climate, lat_deg: f64) -> f64 {
    if let Some(ex) = &cl.exchange {
        let ns = ex.seasons.len().max(1) as f64;
        let share: Vec<f64> = (0..cl.lat_deg.len())
            .map(|j| {
                ex.seasons
                    .iter()
                    .filter(|s| s.ice_kg_m2[j] > 0.0 && s.flux_kg_m2_s[j] < 0.0)
                    .count() as f64
                    / ns
            })
            .collect();
        return cl.lerp(&share, lat_deg).clamp(0.0, 1.0);
    }
    let Some(tc) = cl.circulation.strengths.condense_k else {
        return 0.0;
    };
    let t_min = cl.lerp(&cl.t_min_k, lat_deg);
    let w = (ss.lofting_width * tc).max(1e-6);
    1.0 / (1.0 + m::exp(-(tc - t_min) / w))
}

/// Saltation of `grain` at a latitude under the one climate.
pub fn at(ss: &SaltationSettings, cl: &Climate, lat_deg: f64, grain: &Grain) -> Saltation {
    let t = cl.t_at(lat_deg);
    saltation(
        ss,
        air_density(cl, t),
        cl.conditions.gravity_m_s2,
        grain,
        mean_wind(ss, cl, lat_deg),
        lofting(ss, cl, lat_deg),
    )
}

/// Dune activity, 0 to 1, from Lancaster's (1988) mobility index
/// M = W / (P / PE), W the percentage of the time the wind moves sand: fully
/// inactive below `inactive` (50), fully active above `active` (200),
/// linear between. P / PE is floored at `aridity_floor`, so a rainless world
/// still needs some wind.
pub fn dune_activity(ss: &SaltationSettings, transport_share: f64, aridity: f64) -> f64 {
    let mi = &ss.mobility;
    let w = 100.0 * transport_share.max(0.0);
    let mob = w / aridity.max(mi.aridity_floor);
    ((mob - mi.inactive) / (mi.active - mi.inactive).max(1e-9)).clamp(0.0, 1.0)
}

/// The grain a ground of this temperature is made of: the first whose
/// `below_k` is above it (ice sands on cold worlds), else the last.
pub fn grain_for(ss: &SaltationSettings, t_k: f64) -> &Grain {
    ss.grains
        .iter()
        .find(|g| g.below_k.is_some_and(|b| t_k < b))
        .unwrap_or_else(|| ss.grains.last().expect("at least one grain"))
}

/// Mobility settings check for `ClimateSettings::from_packs`.
pub(crate) fn check(ss: &SaltationSettings) -> Result<(), String> {
    if ss.grains.is_empty() || ss.grains.last().is_some_and(|g| g.below_k.is_some()) {
        return Err("saltation needs grains, the last with no below_k (the fallback)".into());
    }
    if ss.mobility.active <= ss.mobility.inactive {
        return Err("dune mobility: active must exceed inactive".into());
    }
    Ok(())
}
