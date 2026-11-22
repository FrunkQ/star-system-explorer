//! `sse.volatile.exchange` (W8 part 2, weather review section 4.3): ice of
//! the air's main gas moving between the ground and the air, band by band
//! and season by season, inside the one climate's seasonal cycle.
//!
//! - **Frost pins the ground.** A band whose ground holds the gas's ice, or
//!   that would cool below the gas's frost point, sits at the frost point:
//!   the energy it lacks is paid by condensing (latent heat released) and
//!   the energy it gains goes into subliming, until its ice is gone. This is
//!   the surface energy balance of an ice that can move as vapour
//!   (εσT⁴ = (1 − A) S cos i + L ṁ + F, exogeology gaps 10.1), written with
//!   the climate's own outgoing-longwave line A + B T and its heat transport
//!   between bands.
//! - **The air is what is not on the ground.** The gas's column in the air
//!   is the world's inventory less its ice, so its partial pressure, and with
//!   it the frost point, follow the ice (vapour-pressure buffering). Each
//!   step solves for the pressure at which the air's change equals the
//!   ground's net exchange (bisection, a fixed number of steps). On a thin
//!   air such as Pluto's the air holds almost nothing, the net exchange is
//!   near zero and the pressure is set by the ice's temperature; on Mars the
//!   air holds most of the CO2 and the seasonal caps draw it down.
//! - **Inventory.** With no ground ice given, the air is all there is and
//!   each spin-up year restores the world's given mean pressure (the caps
//!   are then seasonal, out of the air). With ground ice given
//!   (`Conditions::ground_ice_kg_m2`), the pressure is free and the ice
//!   migrates: a year's net change in each band is carried forward over
//!   many years (`migration_*`), so ice ends where the year-averaged energy
//!   balance is coldest (Bertrand and Forget 2016's equilibrium; poles or
//!   equator decided by obliquity).
//! - Only the main gas exchanges, and only where its frost point comes
//!   within `trigger_margin_k` of the coldest band or ground ice is given.
//!   A world whose liquid is that gas is left to the water cycle.

use crate::conditions::Conditions;
use crate::ebm::{self, Season};
use crate::settings::{ClimateSettings, Substance};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Stefan-Boltzmann, W m⁻² K⁻⁴ (CODATA 2018).
const SIGMA_SB: f64 = 5.670374419e-8;

/// One season of the exchange.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ExchangeSeason {
    /// The gas's partial pressure, Pa (season mean).
    pub p_pa: f64,
    /// Its frost point at that pressure, K.
    pub frost_k: f64,
    /// Ice on the ground per band at the season's end, kg/m².
    pub ice_kg_m2: Vec<f64>,
    /// Mean condensation per band (negative: sublimation), kg m⁻² s⁻¹.
    pub flux_kg_m2_s: Vec<f64>,
}

/// The year of exchange for one gas.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Exchange {
    pub gas: String,
    pub name: String,
    pub seasons: Vec<ExchangeSeason>,
    pub p_mean_pa: f64,
    pub p_min_pa: f64,
    pub p_max_pa: f64,
    /// (max − min) / mean of the season pressures.
    pub p_swing: f64,
    /// Bands whose ground keeps ice all year.
    pub perennial: Vec<bool>,
    /// Annual mean ice per band, kg/m².
    pub ice_mean_kg_m2: Vec<f64>,
    /// Ground ice was given, so the pressure was free and the ice migrated.
    pub buffered: bool,
    /// Years the migration carried the ice forward.
    pub migrated_years: f64,
    pub why: Vec<String>,
}

/// The gas the exchange runs on, if any: the largest share of the air with a
/// vapour curve on the sheet, when its frost point comes near the coldest
/// band of `seasons` or ground ice of it is given.
pub fn candidate<'a>(
    cs: &'a ClimateSettings,
    c: &Conditions,
    seasons: &[Season],
) -> Option<(&'a Substance, f64)> {
    if !c.solid_surface || c.pressure_bar <= 0.0 {
        return None;
    }
    let (sub, x) = cs
        .substances
        .iter()
        .filter_map(|s| c.gases.get(&s.gas).map(|x| (s, *x)))
        .filter(|(_, x)| *x > 0.0)
        .fold(None::<(&Substance, f64)>, |b, (s, x)| match b {
            Some((_, bx)) if bx >= x => b,
            _ => Some((s, x)),
        })?;
    if c.liquid.as_deref() == Some(sub.name.as_str()) {
        return None;
    }
    let given = c.ground_ice_kg_m2.get(&sub.gas).copied().unwrap_or(0.0) > 0.0;
    let tf = ebm::frost_k(sub, x * c.pressure_bar * 1.0e5);
    let coldest = seasons
        .iter()
        .flat_map(|s| s.t_k.iter())
        .cloned()
        .fold(f64::INFINITY, f64::min);
    (given || coldest < tf + cs.exchange.trigger_margin_k).then_some((sub, x))
}

/// One season's sums over its steps: temperature, albedo, sunlight,
/// pressure, frost point, ice at its end and condensation.
type SeasonSum = (Vec<f64>, Vec<f64>, Vec<f64>, f64, f64, Vec<f64>, Vec<f64>);

struct Step<'a> {
    g: &'a ebm::Grid,
    k: f64,
    t_old: &'a [f64],
    absorbed: &'a [f64],
    internal: f64,
    a_olr: f64,
    /// What a frosted band radiates per εσT⁴: the world's own outgoing
    /// flux at its mean temperature over σT̄⁴ (its greenhouse), times the
    /// frost's emissivity.
    frost_emission: f64,
}

/// Band temperatures, condensation (kg m⁻² s⁻¹) and the pinned bands for a
/// frost point `tf`, with `ice` on the ground and a step of `dt`.
fn solve(st: &Step, latent: f64, tf: f64, ice: &[f64], dt: f64) -> (Vec<f64>, Vec<f64>) {
    let g = st.g;
    let n = ice.len();
    let mut pinned: Vec<bool> = ice.iter().map(|i| *i > 0.0).collect();
    let mut t = vec![tf; n];
    let mut flux = vec![0.0; n];
    for _ in 0..6 {
        let (mut lo, mut di, mut up, mut r) =
            (g.lo.clone(), vec![0.0; n], g.up.clone(), vec![0.0; n]);
        for j in 0..n {
            if pinned[j] {
                lo[j] = 0.0;
                up[j] = 0.0;
                di[j] = 1.0;
                r[j] = tf;
            } else {
                di[j] = st.k + g.b + g.lo[j] + g.up[j];
                r[j] = st.k * st.t_old[j] + st.absorbed[j] + st.internal - st.a_olr;
            }
        }
        t = ebm::thomas(&lo, &di, &up, &r);
        let mut changed = false;
        for j in 0..n {
            if pinned[j] {
                let tr_lo = if j > 0 {
                    g.lo[j] * (t[j - 1] - tf)
                } else {
                    0.0
                };
                let tr_up = if j + 1 < n {
                    g.up[j] * (t[j + 1] - tf)
                } else {
                    0.0
                };
                let olr = st.frost_emission * SIGMA_SB * tf * tf * tf * tf;
                let need =
                    st.k * (tf - st.t_old[j]) - st.absorbed[j] - st.internal + olr - tr_lo - tr_up;
                flux[j] = need / latent;
                if flux[j] < 0.0 && ice[j] + flux[j] * dt <= 0.0 {
                    // Not enough ice to sublime: the band goes bare.
                    pinned[j] = false;
                    changed = true;
                }
            } else {
                flux[j] = 0.0;
                if t[j] < tf {
                    pinned[j] = true;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    // A band that went bare sublimes what it had.
    for j in 0..n {
        if !pinned[j] && ice[j] > 0.0 {
            flux[j] = -ice[j] / dt;
        }
    }
    (t, flux)
}

/// The gas's vapour curve tabulated once per cycle, ln P against 1/T on an
/// even grid (ln P is close to linear in 1/T, so linear steps between 1,024
/// nodes keep it within a part in 10⁵ of the sheet's fit): the exchange reads
/// it thousands of times a year.
struct Curve {
    inv_lo: f64,
    d_inv: f64,
    lnp: Vec<f64>,
    t_min: f64,
    t_max: f64,
    molar: f64,
}

impl Curve {
    fn new(sub: &Substance, t_min: f64) -> Self {
        let t_max = sub.triple_k;
        let nodes = 1024;
        let (inv_lo, inv_hi) = (1.0 / t_max, 1.0 / t_min);
        let d_inv = (inv_hi - inv_lo) / (nodes - 1) as f64;
        let lnp = (0..nodes)
            .map(|i| m::ln(ebm::p_sat(sub, 1.0 / (inv_lo + d_inv * i as f64)).max(1e-300)))
            .collect();
        Curve {
            inv_lo,
            d_inv,
            lnp,
            t_min,
            t_max,
            molar: sub.molar_kg,
        }
    }
    fn at(&self, t: f64) -> (f64, f64) {
        let x = ((1.0 / t - self.inv_lo) / self.d_inv).clamp(0.0, (self.lnp.len() - 1) as f64);
        let i = (x as usize).min(self.lnp.len() - 2);
        let f = x - i as f64;
        let slope = (self.lnp[i + 1] - self.lnp[i]) / self.d_inv;
        (self.lnp[i] + f * (self.lnp[i + 1] - self.lnp[i]), slope)
    }
    /// Pressure, Pa.
    fn p(&self, t: f64) -> f64 {
        m::exp(self.at(t).0)
    }
    /// Sublimation heat, J/kg, from the slope: L = −R d ln P / d(1/T).
    fn latent(&self, t: f64) -> f64 {
        -crate::conditions::R_GAS * self.at(t).1 / self.molar
    }
}

/// The seasonal cycle with the exchange: as [`ebm::cycle`], with frost
/// pinning the ground and the gas's pressure solved each step.
pub fn cycle(
    cs: &ClimateSettings,
    c: &Conditions,
    sub: &Substance,
    share: f64,
) -> (Vec<Season>, Exchange) {
    let es = &cs.exchange;
    let hc = &cs.heat_capacity;
    let ns = cs.seasons.max(1) as usize;
    let per = hc.steps_per_season.max(1) as usize;
    let steps = ns * per;
    let lat = ebm::lats(cs);
    let n = lat.len();
    let g = ebm::grid(cs, c);
    let qs: Vec<Vec<f64>> = (0..steps)
        .map(|k| {
            let phase = (k as f64 + 0.5) / steps as f64;
            let decl = ebm::declination_deg(c, phase);
            let near = ebm::orbit_at(c, phase).1;
            lat.iter()
                .map(|l| ebm::insolation(c.flux_w_m2 * near, *l, decl))
                .collect()
        })
        .collect();
    let q_year: Vec<f64> = (0..n)
        .map(|j| qs.iter().map(|q| q[j]).sum::<f64>() / steps as f64)
        .collect();
    let (mut t, _) = ebm::steady(cs, c, &q_year);
    let t_bar = c.t_surface_k;
    let cap = ebm::heat_capacity(cs, c);
    let year_s = (c.year_days * 86_400.0).max(1.0);
    let dt = year_s / steps as f64;
    let k = cap / dt;
    let alb0 = ebm::albedo(cs, c, &g, &q_year, &t);
    let abs0: Vec<f64> = q_year
        .iter()
        .zip(&alb0)
        .map(|(a, al)| a * (1.0 - al))
        .collect();
    let mut a_olr = ebm::weighted_mean(&g.w, &abs0) + c.internal_flux_w_m2 - g.b * t_bar;
    let grav = c.gravity_m_s2.max(1e-9);
    let p_given = share * c.pressure_bar * 1.0e5;
    let given = c
        .ground_ice_kg_m2
        .get(&sub.gas)
        .copied()
        .unwrap_or(0.0)
        .max(0.0);
    let buffered = given > 0.0;
    let mut ice = vec![given; n];
    // The gas's column in the air, kg/m².
    let mut air = p_given / grav;
    let wsum: f64 = g.w.iter().sum();
    let mean_w = |v: &[f64]| v.iter().zip(&g.w).map(|(a, b)| a * b).sum::<f64>() / wsum;

    let curve = Curve::new(sub, cs.exchange.lowest_frost_k.min(0.5 * sub.triple_k));
    let run_year = |t: &mut Vec<f64>,
                    ice: &mut Vec<f64>,
                    air: &mut f64,
                    a_olr: &mut f64,
                    record: bool|
     -> (f64, Vec<SeasonSum>) {
        let mut year_abs = 0.0;
        let mut p_sum = 0.0;
        let mut acc = vec![
            (
                vec![0.0; n],
                vec![0.0; n],
                vec![0.0; n],
                0.0,
                0.0,
                vec![0.0; n],
                vec![0.0; n]
            );
            ns
        ];
        for (s, q) in qs.iter().enumerate() {
            let alb = ebm::albedo(cs, c, &g, q, t);
            let absorbed: Vec<f64> = q.iter().zip(&alb).map(|(a, al)| a * (1.0 - al)).collect();
            year_abs += ebm::weighted_mean(&g.w, &absorbed);
            let st = Step {
                g: &g,
                k,
                t_old: t,
                absorbed: &absorbed,
                internal: c.internal_flux_w_m2,
                a_olr: *a_olr,
                frost_emission: es.frost_emissivity * (*a_olr + g.b * t_bar)
                    / (SIGMA_SB * t_bar * t_bar * t_bar * t_bar).max(1e-30),
            };
            let total = *air + mean_w(ice);
            // Residual: the air's change plus the ground's net gain, for a
            // trial frost point; it rises with the frost point.
            let trial = |tf: f64| {
                let p = curve.p(tf);
                let (tt, fl) = solve(&st, curve.latent(tf), tf, ice, dt);
                let r = (p / grav - *air) + mean_w(&fl) * dt;
                (r, tf, tt, fl, p)
            };
            // Bracket: no air at all, or all of it in the air. Then
            // regula falsi with the Illinois halving, a fixed number of steps.
            let t_hi = ebm::frost_k(sub, (total * grav).max(1e-30)).clamp(curve.t_min, curve.t_max);
            let mut b = trial(t_hi);
            if b.0 > 0.0 {
                let mut a = trial(curve.t_min);
                if a.0 < 0.0 {
                    let mut side = 0i8;
                    for _ in 0..es.bisection_steps {
                        let x = (a.1 * b.0 - b.1 * a.0) / (b.0 - a.0);
                        let r = trial(x);
                        if r.0 > 0.0 {
                            b = r;
                            if side == 1 {
                                a.0 *= 0.5;
                            }
                            side = 1;
                        } else {
                            a = r;
                            if side == -1 {
                                b.0 *= 0.5;
                            }
                            side = -1;
                        }
                        if (b.1 - a.1).abs() < 1e-7 * b.1 {
                            break;
                        }
                    }
                } else {
                    b = a;
                }
            }
            let (_, tf, tt, fl, p) = b;
            for j in 0..n {
                ice[j] = (ice[j] + fl[j] * dt).max(0.0);
            }
            *air = (total - mean_w(ice)).max(0.0);
            *t = tt;
            p_sum += p;
            if record {
                let e = &mut acc[s / per];
                for j in 0..n {
                    e.0[j] += t[j] / per as f64;
                    e.1[j] += alb[j] / per as f64;
                    e.2[j] += q[j] / per as f64;
                    e.6[j] += fl[j] / per as f64;
                }
                e.3 += p / per as f64;
                e.4 += tf / per as f64;
                e.5 = ice.clone();
            }
        }
        *a_olr = year_abs / steps as f64 + c.internal_flux_w_m2 - g.b * t_bar;
        (p_sum / steps as f64, acc)
    };

    for _ in 0..hc.spin_up_years.max(1) {
        let (p_mean, _) = run_year(&mut t, &mut ice, &mut air, &mut a_olr, false);
        if !buffered {
            // The air is the inventory: hold the world's given mean pressure.
            air = (air + (p_given - p_mean) / grav).max(0.0);
        }
    }
    let mut migrated = 0.0;
    if buffered {
        for _ in 0..es.migration_steps {
            if migrated >= es.migration_span_yr {
                break;
            }
            let start = ice.clone();
            run_year(&mut t, &mut ice, &mut air, &mut a_olr, false);
            let years_per = c.year_days / 365.25;
            // Carry the year's change forward until the first band that is
            // losing ice would run out.
            let mut f = (es.migration_span_yr - migrated) / years_per.max(1e-9);
            let mut losing = false;
            for j in 0..n {
                let d = ice[j] - start[j];
                if d < 0.0 && ice[j] > 0.0 {
                    losing = true;
                    f = f.min(ice[j] / -d);
                }
            }
            if !losing {
                break;
            }
            let f = f.max(0.0);
            let kept = mean_w(&ice);
            for j in 0..n {
                ice[j] = (ice[j] + f * (ice[j] - start[j])).max(0.0);
            }
            // Mass kept: carrying forward moves ice between bands, and the
            // ground keeps what it held (the clamp at zero and the bands
            // that run out leave the sum off by a little).
            let now = mean_w(&ice);
            if now > 0.0 {
                for x in ice.iter_mut() {
                    *x *= kept / now;
                }
            }
            migrated += (1.0 + f) * years_per;
        }
    }
    let (_, acc) = run_year(&mut t, &mut ice, &mut air, &mut a_olr, true);
    let seasons: Vec<Season> = acc
        .iter()
        .map(|(t, alb, q, ..)| ebm::diagnose(cs, c, q, t, alb))
        .collect();
    let xs: Vec<ExchangeSeason> = acc
        .iter()
        .map(|e| ExchangeSeason {
            p_pa: e.3,
            frost_k: e.4,
            ice_kg_m2: e.5.clone(),
            flux_kg_m2_s: e.6.clone(),
        })
        .collect();
    let ps: Vec<f64> = xs.iter().map(|s| s.p_pa).collect();
    let p_mean = ps.iter().sum::<f64>() / ps.len().max(1) as f64;
    let p_min = ps.iter().cloned().fold(f64::INFINITY, f64::min);
    let p_max = ps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let perennial: Vec<bool> = (0..n)
        .map(|j| xs.iter().all(|s| s.ice_kg_m2[j] > 0.0))
        .collect();
    let ice_mean: Vec<f64> = (0..n)
        .map(|j| xs.iter().map(|s| s.ice_kg_m2[j]).sum::<f64>() / xs.len().max(1) as f64)
        .collect();
    let swing = if p_mean > 0.0 {
        (p_max - p_min) / p_mean
    } else {
        0.0
    };
    let mut why = vec![format!(
        "{} freezes onto the ground where it is colder than its frost point ({:.1} K at the mean {:.3e} Pa, Fray and Schmitt 2009's curve) and sublimes where it is warmer; the air keeps what the ground does not, so the pressure swings {:.0} % over the year ({:.3e} to {:.3e} Pa).",
        sub.name,
        ebm::frost_k(sub, p_mean.max(1e-30)),
        p_mean,
        100.0 * swing,
        p_min,
        p_max
    )];
    if buffered {
        why.push(format!(
            "With {given:.0} kg/m² of its ice on the ground the pressure is set by the ice's temperature, and the ice migrated over {migrated:.0} years to where the year-averaged energy balance is coldest."
        ));
    }
    let ex = Exchange {
        gas: sub.gas.clone(),
        name: sub.name.clone(),
        seasons: xs,
        p_mean_pa: p_mean,
        p_min_pa: p_min,
        p_max_pa: p_max,
        p_swing: swing,
        perennial,
        ice_mean_kg_m2: ice_mean,
        buffered,
        migrated_years: migrated,
        why,
    };
    (seasons, ex)
}
