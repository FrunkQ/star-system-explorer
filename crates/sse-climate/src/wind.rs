//! The wind at 10 m: one distribution per band, read by everything that
//! needs the wind near the ground (blown sand, sea waves).
//!
//! The wind vector is its mean plus a transient part, each component of
//! which is Gaussian with the same spread σ (the isotropic Gaussian vector
//! wind, Monahan 2006). The mean is the circulation's prevailing wind:
//! easterly trades, westerlies in the eddy-driven cells. The transient part
//! is the storms: a share of the eddy wind, carried by the share of the
//! heat transport that eddies do rather than the Hadley cell (the same
//! w = exp(−x²/σ²) the moisture block uses, Siler, Roe and Armour 2018),
//! slowing as the cube root of the air's density over Earth's, and scaled
//! by how well the air acts as a fluid.

use crate::settings::ClimateSettings;
use crate::Climate;
use sse_core::math as m;

/// Gauss-Hermite nodes and weights for ∫ e^{−t²} f(t) dt, 10 points
/// (Abramowitz and Stegun 1964, table 25.10).
const GH: [(f64, f64); 10] = [
    (-3.436_159_118_837_738, 7.640_432_855_232_62e-6),
    (-2.532_731_674_232_79, 1.343_645_746_781_233e-3),
    (-1.756_683_649_299_882, 3.387_439_445_548_106e-2),
    (-1.036_610_829_789_514, 2.401_386_110_823_147e-1),
    (-0.342_901_327_223_704_6, 6.108_626_337_353_258e-1),
    (0.342_901_327_223_704_6, 6.108_626_337_353_258e-1),
    (1.036_610_829_789_514, 2.401_386_110_823_147e-1),
    (1.756_683_649_299_882, 3.387_439_445_548_106e-2),
    (2.532_731_674_232_79, 1.343_645_746_781_233e-3),
    (3.436_159_118_837_738, 7.640_432_855_232_62e-6),
];

/// Per band: the mean eastward wind at 10 m and the transient spread σ of
/// each component, m/s.
pub fn ten_metre(cs: &ClimateSettings, cl: &Climate) -> (Vec<f64>, Vec<f64>) {
    let circ = &cl.circulation;
    let st = &circ.strengths;
    let mo = &cs.moisture;
    let width = mo.hadley_width_x * m::sin(rad(circ.hadley_edge_deg.clamp(1.0, 90.0)))
        / m::sin(rad(mo.hadley_reference_deg));
    let mean: Vec<f64> = cl
        .lat_deg
        .iter()
        .map(|l| circ.prevailing_wind(*l))
        .collect();
    let gust = cl
        .lat_deg
        .iter()
        .enumerate()
        .map(|(j, l)| {
            let x = m::sin(rad(*l));
            let hadley = if width > 0.0 {
                m::exp(-(x * x) / (width * width))
            } else {
                0.0
            };
            let rho = crate::saltation::air_density(cl, cl.t_k[j]);
            let dense = if rho > 0.0 {
                m::cbrt(cs.saltation.impact_reference_kg_m3 / rho).min(1.0)
            } else {
                0.0
            };
            cs.wind10.gust_share * st.eddy_wind_m_s * (1.0 - hadley) * dense * st.collisional
        })
        .collect();
    (mean, gust)
}

fn rad(d: f64) -> f64 {
    d * m::PI / 180.0
}

/// The mean over the wind's distribution of `f(speed)`, with mean eastward
/// wind `u` and spread `sigma` per component.
pub fn expect(u: f64, sigma: f64, f: impl Fn(f64) -> f64) -> f64 {
    if sigma <= 0.0 {
        return f(u.abs());
    }
    let s = m::sqrt(2.0) * sigma;
    let mut sum = 0.0;
    for (a, wa) in GH {
        for (b, wb) in GH {
            let e = u + s * a;
            let n = s * b;
            sum += wa * wb * f(m::sqrt(e * e + n * n));
        }
    }
    sum / m::PI
}

/// Mean wind speed at 10 m, m/s.
pub fn mean_speed(u: f64, sigma: f64) -> f64 {
    expect(u, sigma, |w| w)
}
