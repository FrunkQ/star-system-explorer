//! W7b: the circulation as process strengths. Each process computes its own
//! strength, 0 to 1, from the physics of the world, and runs on every world
//! weighted by it. No regime gate picks a process; the regime is a label read
//! off the strengths afterwards (processes, not classes).
//!
//! - **Thermal wind and the thermal Rossby number** (Mitchell and Vallis 2010,
//!   eq. 3.3b): Ro_T = U / (2Ωa) = R T0 Δ_H / (2Ωa)². The wind scale U every
//!   other length reads (the Rhines cells, the storm growth, the surface wind)
//!   comes from it, capped where Ro_T ≫ 1 by the speed the same contrast can
//!   drive with no spin, sqrt(R T0 Δ_H). The eddy wind is a share of it.
//! - **Super-rotation** turns on as Ro_T passes about 1 (Mitchell and Vallis
//!   2010): strength x^k / (1 + x^k), x = Ro_T / critical.
//! - **Baroclinic growth** σ = 0.31 f (dU/dz) / N (Lindzen and Farrell 1980)
//!   against radiative damping 1/τ_rad, τ_rad = c_p p / (g σ T³) (Koll and
//!   Abbot 2016): strength σ τ_rad / (1 + σ τ_rad).
//! - **Day-night contrast** A = [1 + τ_rad τ_eff Δln p / τ_wave²]⁻¹ (Komacek
//!   and Showman 2016, eq. 31), τ_wave = a / (N H), τ_eff the shorter of the
//!   drag time τ_wave H / (C_D a) (Koll and Abbot 2016) and 1/f; weighted by how
//!   much of a night the air has to cool, 1 − exp(−day / 2τ_rad), so a locked
//!   world sees all of it and Earth almost none.
//! - **Collapse**: the night side, T_n⁴ = T⁴ (1 − A) but no colder than the
//!   thin radiator [(1−α) F κ p / (4 σ g)]^¼ (Wordsworth 2015, eq. 27), against
//!   the main gas's condensation temperature at the surface pressure
//!   (Clausius-Clapeyron from the substance table).
//! - **Collisional air**: the Knudsen number λ / H, λ = k T / (√2 π d² p); air is
//!   a fluid where Kn ≪ 1 (Pluto) and an exosphere where Kn ≫ 1 (the Moon).
//! - **Regime lengths** (Haqq-Misra et al. 2018, eqs. 8 and 9): the deformation
//!   radius λ_R² = sqrt(gH) / (2β) and the Rhines length L_R = π sqrt(U / β)
//!   against the radius, published as ratios.

use crate::conditions::{Conditions, R_GAS};
use crate::settings::ClimateSettings;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const SIGMA_SB: f64 = 5.670374419e-8;
const K_B: f64 = 1.380649e-23;

/// Every strength and the ratios behind it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Strengths {
    /// Thermal Rossby number R T0 Δ_H / (2Ωa)².
    pub thermal_rossby: f64,
    /// Thermal wind scale before the cap, m/s, and as used.
    pub thermal_wind_m_s: f64,
    pub wind_scale_m_s: f64,
    /// Eddy wind the Rhines length reads, m/s.
    pub eddy_wind_m_s: f64,
    /// 0 to 1.
    pub super_rotation: f64,
    /// Upper-air equatorial wind the super-rotation drives, m/s (along the spin).
    pub super_rotation_m_s: f64,
    /// Baroclinic growth rate, 1/s, and the strength against damping.
    pub baroclinic_growth_per_s: f64,
    pub baroclinic: f64,
    /// Radiative, wave and drag times, s.
    pub radiative_s: f64,
    pub wave_s: f64,
    pub drag_s: f64,
    /// Buoyancy frequency used, 1/s.
    pub brunt_s: f64,
    /// Day-night contrast as a fraction of the no-transport contrast, 0 to 1.
    pub day_night: f64,
    /// Night-side temperature, K.
    pub night_k: f64,
    /// The main gas and its condensation temperature at the surface, K.
    pub main_gas: Option<String>,
    pub condense_k: Option<f64>,
    /// 0 to 1: the air freezing out on the night side.
    pub collapse: f64,
    /// Knudsen number at the surface, and the strength 1 / (1 + Kn).
    pub knudsen: f64,
    pub collisional: f64,
    /// Deformation radius and Rhines length over the radius.
    pub deformation_ratio: f64,
    pub rhines_ratio: f64,
    /// Processes this world takes outside the range their papers tested,
    /// one line each (an extrapolation, not an error).
    pub outside: Vec<String>,
    /// One line per process for "Why?".
    pub why: Vec<String>,
}

fn sigmoid(x: f64, k: f64) -> f64 {
    let p = m::pow(x.max(0.0), k);
    p / (1.0 + p)
}

/// Condensation temperature of a substance at partial pressure `p_pa`
/// (Clausius-Clapeyron through the triple point, as `ebm::p_sat`).
pub fn condense_k(sub: &crate::settings::Substance, p_pa: f64) -> f64 {
    let l_r = sub.latent_j_kg * sub.molar_kg / R_GAS;
    1.0 / (1.0 / sub.triple_k - m::ln(p_pa / sub.triple_pa) / l_r)
}

/// The strengths of `c`'s circulation processes.
pub fn strengths(cs: &ClimateSettings, c: &Conditions) -> Strengths {
    let st = &cs.strengths;
    let cc = &cs.circulation;
    let omega = c.spin_rate().max(1e-12);
    let a = c.radius_m;
    let g = c.gravity_m_s2.max(1e-6);
    let t = c.t_surface_k.max(1.0);
    let r_s = R_GAS / c.mean_molar_kg;
    let cp = 3.5 * r_s;
    let p_pa = c.pressure_bar * 1.0e5;
    let h = c.scale_height_m();
    let two_oa = 2.0 * omega * a;
    let forcing = r_s * t * cc.delta_h;
    let ro_t = forcing / (two_oa * two_oa);
    let u_t = forcing / two_oa;
    let u_cap = m::sqrt(forcing);
    let u = u_t.min(u_cap);
    let eddy = st.eddy_share * u;
    let sr = sigmoid(
        ro_t / st.super_rotation_critical,
        st.super_rotation_steepness,
    );
    // Buoyancy frequency of a troposphere a share of the way from neutral to
    // isothermal: N² = (g / T)(g / c_p) × share.
    let n = m::sqrt(g / t * g / cp * st.stable_share);
    let f45 = m::sqrt(2.0) * omega;
    let h_trop = cc.tropopause_scale_heights * h;
    let growth = st.lindzen_farrell * f45 * (u / h_trop) / n;
    let tau_rad = cp * p_pa / (g * SIGMA_SB * t * t * t);
    let bc = growth * tau_rad / (1.0 + growth * tau_rad);
    let tau_wave = a / (n * h);
    let tau_drag = tau_wave * h / (st.drag_coefficient * a);
    let tau_eff = tau_drag.min(1.0 / f45);
    let a_ks = 1.0 / (1.0 + tau_rad * tau_eff * st.log_pressure_span / (tau_wave * tau_wave));
    let day_s = if c.star_locked {
        f64::INFINITY
    } else {
        c.solar_day_hours() * 3600.0
    };
    let exposure = if tau_rad <= 0.0 {
        1.0
    } else {
        1.0 - m::exp(-day_s / (2.0 * tau_rad))
    };
    let dn = (a_ks * exposure).clamp(0.0, 1.0);
    let thin = m::pow(
        ((1.0 - c.albedo) * c.flux_w_m2 * st.thin_opacity_m2_kg * p_pa / (4.0 * SIGMA_SB * g))
            .max(0.0),
        0.25,
    );
    let night = (t * m::pow(1.0 - dn, 0.25)).max(thin.min(t));
    // The main condensable gas: the largest share with a phase table.
    let main = cs
        .substances
        .iter()
        .filter_map(|s| c.gases.get(&s.gas).map(|x| (s, *x)))
        .filter(|(_, x)| *x > 0.0)
        .fold(
            None::<(&crate::settings::Substance, f64)>,
            |b, (s, x)| match b {
                Some((_, bx)) if bx >= x => b,
                _ => Some((s, x)),
            },
        );
    let (main_gas, t_cond, collapse) = match main {
        Some((s, x)) if p_pa > 0.0 => {
            let tc = condense_k(s, x * p_pa);
            let w = st.collapse_width * tc;
            let k = 1.0 / (1.0 + m::exp((night - tc) / w.max(1e-6)));
            (Some(s.name.clone()), Some(tc), k)
        }
        _ => (None, None, 0.0),
    };
    let kn = if p_pa > 0.0 {
        K_B * t
            / (m::sqrt(2.0) * m::PI * st.molecule_diameter_m * st.molecule_diameter_m * p_pa)
            / h
    } else {
        f64::INFINITY
    };
    let coll = if kn.is_finite() {
        1.0 / (1.0 + kn)
    } else {
        0.0
    };
    // Fluid processes run only as strongly as the air is a fluid.
    let sr = sr * coll;
    let bc = bc * coll;
    let beta = 2.0 * omega / a;
    let deformation = m::sqrt(m::sqrt(g * h) / (2.0 * beta)) / a;
    let rhines = m::PI * m::sqrt(eddy / beta) / a;
    let q = |k: &str| -> Option<f64> {
        Some(match k {
            "thermal_rossby" => ro_t,
            "pressure_bar" => c.pressure_bar,
            "t_surface_k" => t,
            "flux_w_m2" => c.flux_w_m2,
            "solid" => c.solid_surface as u8 as f64,
            "solar_day_hours" => c.solar_day_hours(),
            "rotation_days" => c.rotation_hours.abs() / 24.0,
            _ => return None,
        })
    };
    let outside: Vec<String> = st
        .tested
        .iter()
        .filter_map(|r| {
            let v = q(&r.quantity)?;
            let low = r.min.is_some_and(|x| v < x);
            let high = r.max.is_some_and(|x| v > x);
            (low || high).then(|| {
                format!(
                    "{}: outside the tested range ({} {} is {} {}, {}); an extrapolation.",
                    r.process,
                    r.quantity,
                    fmt(v),
                    if low { "below" } else { "above" },
                    fmt(if low {
                        r.min.unwrap_or(0.0)
                    } else {
                        r.max.unwrap_or(0.0)
                    }),
                    r.by
                )
            })
        })
        .collect();
    let why = vec![
        format!(
            "Thermal Rossby number {ro_t:.3} (Mitchell and Vallis 2010): thermal wind {u_t:.0} m/s, used {u:.0} m/s, eddies {eddy:.1} m/s."
        ),
        format!(
            "Super-rotation {sr:.2}: Ro_T {ro_t:.3} against {} (it turns on near 1).",
            st.super_rotation_critical
        ),
        format!(
            "Baroclinic storms {bc:.2}: growth {:.2} per day (Lindzen and Farrell 1980) against a radiative time of {:.1} days.",
            growth * 86400.0,
            tau_rad / 86400.0
        ),
        format!(
            "Day-night contrast {dn:.2} of the full contrast (Komacek and Showman 2016): wave time {:.1} h, {:.0} % of a night to cool in.",
            tau_wave / 3600.0,
            exposure * 100.0
        ),
        match (&main_gas, t_cond) {
            (Some(gname), Some(tc)) => format!(
                "Collapse {collapse:.2}: night side {night:.0} K against {gname} freezing at {tc:.0} K (Wordsworth 2015)."
            ),
            _ => "No condensable main gas, so no collapse.".into(),
        },
        format!("Collisional air {coll:.2}: Knudsen number {kn:.2e} at the ground."),
    ];
    Strengths {
        thermal_rossby: ro_t,
        thermal_wind_m_s: u_t,
        wind_scale_m_s: u,
        eddy_wind_m_s: eddy,
        super_rotation: sr,
        super_rotation_m_s: sr * u,
        baroclinic_growth_per_s: growth,
        baroclinic: bc,
        radiative_s: tau_rad,
        wave_s: tau_wave,
        drag_s: tau_drag,
        brunt_s: n,
        day_night: dn,
        night_k: night,
        main_gas,
        condense_k: t_cond,
        collapse,
        knudsen: kn,
        collisional: coll,
        deformation_ratio: deformation,
        rhines_ratio: rhines,
        why: why.into_iter().chain(outside.iter().cloned()).collect(),
        outside,
    }
}

fn fmt(v: f64) -> String {
    if v != 0.0 && (v.abs() < 0.01 || v.abs() >= 1.0e4) {
        format!("{v:.2e}")
    } else {
        format!("{v:.3}")
    }
}
