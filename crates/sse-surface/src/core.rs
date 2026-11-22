//! MF1: a rocky world's core heat history (`sse.core.history`), from which
//! its dynamo switches on, weakens and dies by itself.
//!
//! A two-layer thermal history in the parametrised-convection tradition
//! (Stevenson, Spohn and Schubert 1983; Driscoll and Bercovici 2014): the
//! mantle loses heat through its top boundary layer (mobile lid or stagnant
//! lid, from the world's regime at the date) and gains it from radioactivity,
//! tides and the core; the core loses heat through the boundary layer at its
//! top. As the core cools, an inner core freezes where the core's adiabat
//! drops below the iron alloy's melting curve, and its latent heat and light
//! elements add power.
//!
//! The dynamo's power is the convected flux, the heat leaving the core less
//! what conduction alone carries along the adiabat (Labrosse 2015), plus the
//! buoyancy of light elements: those let out as the inner core freezes, and
//! magnesium oxide coming out of the cooling core (O'Rourke and Stevenson
//! 2016), as equivalent heat fluxes. The field strength is the existing
//! `sse.magnetic.dynamo_field` law fed this power.
//!
//! One ODE pair per world over a few hundred steps: well under a millisecond.

use crate::{Body, SurfaceEnvelope};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

pub const CORE_VERSION: &str = "core-history-v1";

use crate::cover::R_GAS;
use crate::{M_EARTH_KG as M_EARTH, R_EARTH_M as R_EARTH};

pub const REFS: [&str; 9] = [
    "Stevenson, D. J., Spohn, T. and Schubert, G. 1983, Icarus 54, 466: magnetism and thermal evolution of the terrestrial planets. doi:10.1016/0019-1035(83)90241-5",
    "Driscoll, P. and Bercovici, D. 2014, Physics of the Earth and Planetary Interiors 236, 36: on the thermal and magnetic histories of Earth and Venus (lower-mantle viscosity jump, local critical Rayleigh number). doi:10.1016/j.pepi.2014.08.004",
    "Solomatov, V. S. and Moresi, L.-N. 2000, JGR 105, 21795: scaling of time-dependent stagnant lid convection. doi:10.1029/2000JB900197",
    "Labrosse, S. 2015, Physics of the Earth and Planetary Interiors 247, 36: thermal evolution of the core with a high thermal conductivity (adiabat, latent and gravitational energy). doi:10.1016/j.pepi.2015.02.002",
    "Anzellini, S. et al. 2013, Science 340, 464: melting of iron at Earth's inner core boundary (Simon-Glatzel fit). doi:10.1126/science.1233514",
    "O'Rourke, J. G. and Stevenson, D. J. 2016, Nature 529, 387: powering Earth's dynamo with magnesium precipitation from the core. doi:10.1038/nature16495",
    "Masters, G. and Gubbins, D. 2003, Physics of the Earth and Planetary Interiors 140, 159: on the resolution of density within the Earth (inner core boundary density jump). doi:10.1016/j.pepi.2003.07.008",
    "Christensen, U. R. 2010, Space Science Reviews 152, 565: dynamo scaling laws (the field from the convected buoyancy flux). doi:10.1007/s11214-009-9553-2",
    "Zeng, L. and Jacobsen, S. B. 2017, ApJ 837, 164: a simple analytical model for rocky planet interiors (core radius about the square root of the core mass fraction). doi:10.3847/1538-4357/aa6266",
];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreSettings {
    /// Core thermal conductivity, W/m/K. Disputed: 30 to 150 (Konôpková et
    /// al. 2016 low, Pozzo et al. 2012 high); the number that decides how
    /// old Earth's dynamo can be.
    pub k_core: f64,
    pub alpha_core: f64,
    pub cp_core: f64,
    /// Latent and gravitational energy per kg of inner core frozen, J/kg.
    pub latent_j_kg: f64,
    pub gravitational_j_kg: f64,
    /// Density jump at the inner core boundary from light elements, over
    /// the core's density.
    pub icb_density_jump: f64,
    /// Magnesium oxide let out per kelvin of core cooling (mass fraction).
    /// Zero switches the magnesium route off.
    pub mg_per_k: f64,
    /// Melting point of the core alloy over pure iron's, per unit mass
    /// fraction of light elements (a stand-in until MP21's iron rows).
    pub melt_depression_per_light: f64,
    /// Light elements in the core when the body does not say.
    pub light_fraction: f64,
    pub k_mantle: f64,
    pub k_lower_mantle: f64,
    pub alpha_mantle: f64,
    pub cp_mantle: f64,
    pub kappa_mantle: f64,
    /// Upper-mantle viscosity at the reference temperature, Pa s, and the
    /// activation energy, J/mol.
    pub eta_ref: f64,
    /// A dry mantle is this many times stiffer than a wet one (water weakens
    /// olivine; Hirth and Kohlstedt 2003).
    pub dry_viscosity_factor: f64,
    /// Every mantle starts wet from its magma ocean. Under a stagnant lid it
    /// loses its water to volcanism with this e-folding time, Gyr; sinking
    /// plates carry water back down at the same rate.
    pub degas_gyr: f64,
    /// How far below its melting point the core's centre must cool before a
    /// solid inner core first nucleates, K (Huguet et al. 2018: homogeneous
    /// nucleation needs hundreds of kelvin; impurities need less).
    pub nucleation_undercooling_k: f64,
    pub t_ref_k: f64,
    pub activation_j_mol: f64,
    /// Lower mantle over upper mantle viscosity (Driscoll and Bercovici 2014).
    pub lower_mantle_factor: f64,
    pub ra_critical: f64,
    pub ra_critical_boundary: f64,
    /// Mantle potential temperature after the magma ocean, K.
    pub t_mantle_start_k: f64,
    /// The core starts hotter than the base of the magma ocean it sank
    /// through by this much, K: the heat of its own sinking.
    pub core_superheat_k: f64,
    /// The mantle's liquidus, Simon form T₀ (P/P₀ + 1)^(1/c) (Andrault et al.
    /// 2011, chondritic mantle): the base of the magma ocean the core leaves.
    pub mantle_liquidus_t0_k: f64,
    pub mantle_liquidus_p0_gpa: f64,
    pub mantle_liquidus_c: f64,
    /// Magnesium comes out only of a core that formed hot: below this
    /// temperature at the core's top, none is left in solution to come out.
    pub mg_floor_k: f64,
    /// Mantle potential temperature above which melting vents the excess heat
    /// as volcanism (a cap standing in for a melt model).
    pub t_mantle_max_k: f64,
    /// Iron's equation of state (Vinet, hcp iron; Dewaele et al. 2006):
    /// density at no pressure, kg/m³, bulk modulus, GPa, and its slope.
    pub iron_rho0: f64,
    pub iron_k0_gpa: f64,
    pub iron_k0_prime: f64,
    /// Core alloy's density below pure iron's, per unit mass fraction of
    /// light elements.
    pub density_deficit_per_light: f64,
    pub step_myr: f64,
    /// Field strength law (Christensen 2010): c and f_ohm.
    pub field_c: f64,
    pub field_f_ohm: f64,
}

impl Default for CoreSettings {
    fn default() -> Self {
        CoreSettings {
            k_core: 130.0,
            alpha_core: 1.25e-5,
            cp_core: 840.0,
            latent_j_kg: 7.5e5,
            gravitational_j_kg: 3.0e5,
            icb_density_jump: 0.05,
            mg_per_k: 1.0e-5,
            melt_depression_per_light: 1.0,
            light_fraction: 0.1,
            k_mantle: 3.0,
            k_lower_mantle: 10.0,
            alpha_mantle: 1.5e-5,
            cp_mantle: 1200.0,
            kappa_mantle: 1.0e-6,
            eta_ref: 1.0e21,
            dry_viscosity_factor: 100.0,
            degas_gyr: 1.0,
            nucleation_undercooling_k: 150.0,
            t_ref_k: 1600.0,
            activation_j_mol: 3.0e5,
            lower_mantle_factor: 100.0,
            ra_critical: 660.0,
            ra_critical_boundary: 660.0,
            t_mantle_start_k: 1950.0,
            core_superheat_k: 400.0,
            mantle_liquidus_t0_k: 1940.0,
            mantle_liquidus_p0_gpa: 29.0,
            mantle_liquidus_c: 1.9,
            mg_floor_k: 3000.0,
            t_mantle_max_k: 2200.0,
            iron_rho0: 8268.0,
            iron_k0_gpa: 163.4,
            iron_k0_prime: 5.38,
            density_deficit_per_light: 1.0,
            step_myr: 20.0,
            field_c: 0.63,
            field_f_ohm: 0.9,
        }
    }
}

/// One step of the history.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoreStep {
    pub t_gyr: f64,
    pub t_mantle_k: f64,
    pub t_cmb_k: f64,
    /// Heat out of the core and out of the mantle's top, TW.
    pub q_core_tw: f64,
    pub q_surface_tw: f64,
    /// Through the core's top, W/m²: total, conducted along the adiabat,
    /// and the dynamo's power as an equivalent flux (thermal, inner core,
    /// magnesium).
    pub q_cmb_w_m2: f64,
    pub q_adiabat_w_m2: f64,
    pub q_dynamo_w_m2: f64,
    pub inner_core_km: f64,
    /// rms field inside the core, μT (0: no dynamo).
    pub field_ut: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoreEvent {
    pub t_gyr: f64,
    /// dynamo-start, inner-core-birth, dynamo-death.
    pub kind: String,
    pub why: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoreHistory {
    pub version: String,
    pub core_radius_km: f64,
    pub core_density_kg_m3: f64,
    pub cmb_pressure_gpa: f64,
    pub steps: Vec<CoreStep>,
    pub events: Vec<CoreEvent>,
    /// Is a dynamo running now, and why (or why not).
    pub dynamo_now: bool,
    pub why_now: String,
}

pub use sse_atoms::magnetic::{adiabatic_flux, compositional_flux};

/// A power in terawatts, to two figures.
fn tw(w: f64) -> String {
    let x = w / 1e12;
    if x.abs() >= 10.0 {
        format!("{x:.0} TW")
    } else if x.abs() >= 1.0 {
        format!("{x:.1} TW")
    } else if x.abs() >= 0.01 {
        format!("{x:.2} TW")
    } else {
        format!("{:.1} GW", x * 1000.0)
    }
}

/// Melting point of iron, K, at `p_gpa` (Anzellini et al. 2013, Simon-Glatzel).
fn iron_melt_k(p_gpa: f64) -> f64 {
    1991.0 * m::pow(((p_gpa - 5.2) / 27.39 + 1.0).max(1e-6), 1.0 / 2.38)
}

/// Pressure at radius `rc` under a mantle of density `rho_m` from `rc` to
/// `r` around a core of mass `mc`, Pa: its weight, integrated down.
fn mantle_weight(r: f64, rc: f64, mc: f64, rho_m: f64) -> f64 {
    let g_n = sse_core::constants::g();
    let d = r - rc;
    let n = 64;
    let mut p = 0.0;
    for i in 0..n {
        let rr = r - d * (i as f64 + 0.5) / n as f64;
        let menc = mc + rho_m * 4.0 / 3.0 * m::PI * (rr * rr * rr - rc * rc * rc);
        p += rho_m * g_n * menc / (rr * rr) * d / n as f64;
    }
    p
}

/// ρ/ρ₀ of a solid at `p_gpa` by the Vinet equation of state
/// P = 3K₀ (1−x)/x² exp(1.5 (K₀′−1)(1−x)), x = (ρ₀/ρ)^⅓, inverted by bisection.
fn vinet_compression(p_gpa: f64, k0: f64, k0p: f64) -> f64 {
    if p_gpa <= 0.0 {
        return 1.0;
    }
    let pres = |c: f64| {
        let x = m::cbrt(1.0 / c);
        3.0 * k0 * (1.0 - x) / (x * x) * m::exp(1.5 * (k0p - 1.0) * (1.0 - x))
    };
    let (mut lo, mut hi) = (1.0, 4.0);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if pres(mid) < p_gpa {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The history, or None for a world with no iron core to speak of (or no
/// solid interior).
pub fn core_history(
    s: &CoreSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    heat_w_m2: &dyn Fn(f64) -> f64,
) -> Option<CoreHistory> {
    if b.deep_envelope || b.core_mass_fraction < 0.01 || b.mass_earth <= 0.0 {
        return None;
    }
    let g_n = sse_core::constants::g();
    let mass = b.mass_earth * M_EARTH;
    let r = b.radius_earth * R_EARTH;
    let cmf = b.core_mass_fraction.clamp(0.0, 0.95);
    let mc = mass * cmf;
    let mm = mass - mc;
    let light = b.core_light_fraction.unwrap_or(s.light_fraction);
    let rho_alloy0 = s.iron_rho0 * (1.0 - s.density_deficit_per_light * light).max(0.3);
    // The core's size from iron squeezed by its own and the mantle's weight:
    // guess a density, find the radius and the mean pressure, read iron's
    // density there, repeat.
    let mut rho_c = rho_alloy0;
    let mut rc = 0.0;
    let mut p_cmb = 0.0;
    for _ in 0..30 {
        rc = m::cbrt(3.0 * mc / (4.0 * m::PI * rho_c));
        if rc >= r {
            return None;
        }
        let vm = 4.0 / 3.0 * m::PI * (r * r * r - rc * rc * rc);
        let rho_m = mm / vm.max(1.0);
        p_cmb = mantle_weight(r, rc, mc, rho_m);
        let p_mean = p_cmb + 0.4 * (2.0 / 3.0 * m::PI * g_n * rho_c * rho_c * rc * rc);
        let next = rho_alloy0 * vinet_compression(p_mean / 1e9, s.iron_k0_gpa, s.iron_k0_prime);
        if (next - rho_c).abs() < 0.1 {
            rho_c = next;
            break;
        }
        rho_c = next;
    }
    let vm = 4.0 / 3.0 * m::PI * (r * r * r - rc * rc * rc);
    let rho_m = mm / vm.max(1.0);
    let d = r - rc;
    let a_s = 4.0 * m::PI * r * r;
    let a_c = 4.0 * m::PI * rc * rc;
    let g_s = g_n * mass / (r * r);
    let g_c = g_n * mc / (rc * rc);
    let melt =
        |p_pa: f64| iron_melt_k(p_pa / 1e9) * (1.0 - s.melt_depression_per_light * light).max(0.3);
    let p_at = |rr: f64| p_cmb + 2.0 / 3.0 * m::PI * g_n * rho_c * rho_c * (rc * rc - rr * rr);
    // The core's adiabat: T(r) = T_cmb exp((R_c² − r²) / D²).
    let d2 = 3.0 * s.cp_core / (2.0 * m::PI * s.alpha_core * rho_c * g_n);
    let inner = |t_cmb: f64| -> f64 {
        let f = |rr: f64| melt(p_at(rr)) - t_cmb * m::exp((rc * rc - rr * rr) / d2);
        if f(0.0) <= 0.0 {
            return 0.0;
        }
        if f(rc) >= 0.0 {
            return rc;
        }
        let (mut lo, mut hi) = (0.0, rc);
        for _ in 0..30 {
            let mid = 0.5 * (lo + hi);
            if f(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    };
    let adiabat_ratio = m::exp(s.alpha_mantle * g_s * d / s.cp_mantle);
    let eps_m = 0.5 * (1.0 + adiabat_ratio);
    let eps_c = {
        // Mean of the adiabat over the core's mass, relative to its top.
        let n = 32;
        let mut num = 0.0;
        let mut den = 0.0;
        for i in 0..n {
            let rr = rc * (i as f64 + 0.5) / n as f64;
            let w = rr * rr;
            num += w * m::exp((rc * rc - rr * rr) / d2);
            den += w;
        }
        num / den
    };
    let mut wet = 1.0_f64;
    let eta = |t: f64, wet: f64| {
        s.eta_ref
            * m::pow(s.dry_viscosity_factor.max(1.0), 1.0 - wet.clamp(0.0, 1.0))
            * m::exp(s.activation_j_mol / R_GAS * (1.0 / t.max(100.0) - 1.0 / s.t_ref_k))
    };
    let t_s = env.t_surface_now_k.max(1.0);
    let age = b.age_gyr.max(0.0);
    let start = (b.magma_end_myr / 1000.0).clamp(0.0, age);
    let mut t_u = s.t_mantle_start_k;
    // The core leaves the magma ocean at its base's temperature (liquid
    // itself at least), plus the heat of sinking.
    let liquidus = s.mantle_liquidus_t0_k
        * m::pow(
            p_cmb / 1e9 / s.mantle_liquidus_p0_gpa.max(1e-9) + 1.0,
            1.0 / s.mantle_liquidus_c.max(1e-9),
        );
    let mut t_c = liquidus.max(melt(p_cmb)) + s.core_superheat_k;
    let mobile_at = |t: f64| {
        env.timeline
            .iter()
            .find(|x| t >= x.from_gyr && t <= x.to_gyr)
            .map(|x| x.regime == "mobile-lid" || x.regime.contains("magma"))
            .unwrap_or(false)
    };
    let dt_gyr = (s.step_myr / 1000.0).max(1e-4);
    let sec = 3.155_76e16; // seconds a Gyr
    let mut steps = vec![];
    let mut events = vec![];
    let mut t = start;
    let mut nucleated = false;
    let mut ric_prev = 0.0;
    let mut on_prev = false;
    let mut first = true;
    let mut why_now;
    loop {
        // Fluxes at this state.
        let dtm = (t_u - t_s).max(0.0);
        let ra = s.alpha_mantle * rho_m * g_s * dtm * d * d * d / (s.kappa_mantle * eta(t_u, wet));
        let mobile = mobile_at(t);
        let q_top = if mobile {
            s.k_mantle * dtm / d * m::cbrt((ra / s.ra_critical).max(0.0))
        } else {
            let theta = s.activation_j_mol * dtm / (R_GAS * t_u * t_u);
            0.53 * s.k_mantle * dtm / d * m::pow(theta.max(1.0), -4.0 / 3.0) * m::cbrt(ra.max(0.0))
        };
        let t_l = t_u * adiabat_ratio;
        let dtc = t_c - t_l;
        let q_cmb = if dtc > 0.0 {
            let eta_l = s.lower_mantle_factor * eta(t_u, wet);
            let delta = m::cbrt(
                s.kappa_mantle * eta_l * s.ra_critical_boundary
                    / (s.alpha_mantle * rho_m * g_c * dtc),
            );
            s.k_lower_mantle * dtc / delta.min(d)
        } else {
            0.0
        };
        let heat = heat_w_m2(t) * a_s;
        let qc = q_cmb * a_c;
        let qs = q_top * a_s;
        // The core: cooling plus freezing pays for what leaves its top.
        if !nucleated {
            let under = melt(p_at(0.0)) - t_c * m::exp(rc * rc / d2);
            if under >= s.nucleation_undercooling_k {
                // The undercooled centre freezes at once to the size the
                // melting curve allows; its latent heat warms the core back.
                nucleated = true;
                let t0 = t_c;
                for _ in 0..12 {
                    let ri = inner(t_c);
                    let m_ic = rho_c * 4.0 / 3.0 * m::PI * ri * ri * ri;
                    t_c = t0
                        + (s.latent_j_kg + s.gravitational_j_kg) * m_ic / (mc * s.cp_core * eps_c);
                }
            }
        }
        let ric = if nucleated { inner(t_c) } else { 0.0 };
        let dr_dt = if nucleated {
            let h = 1.0;
            (inner(t_c - h) - inner(t_c + h)) / (2.0 * h)
        } else {
            0.0
        }; // m per K of cooling
        let freeze =
            (s.latent_j_kg + s.gravitational_j_kg) * rho_c * 4.0 * m::PI * ric * ric * dr_dt;
        let cool_c = qc / (mc * s.cp_core * eps_c + freeze); // K/s
        let cool_m = (qs - heat - qc) / (mm * s.cp_mantle * eps_m);
        // The dynamo's power at this state.
        let q_ad = adiabatic_flux(s.k_core, s.alpha_core, g_c, t_c, s.cp_core);
        let mdot_ic = rho_c * 4.0 * m::PI * ric * ric * dr_dt * cool_c;
        // Latent heat from freezing is already in what crosses the top (the
        // energy balance above), so only the light elements are added here.
        let q_ic = compositional_flux(
            mdot_ic.max(0.0) / a_c,
            s.icb_density_jump,
            s.alpha_core,
            s.cp_core,
        );
        let mg_rate = if t_c > s.mg_floor_k {
            s.mg_per_k * mc * cool_c.max(0.0)
        } else {
            0.0
        };
        let q_mg = compositional_flux(mg_rate / a_c, 1.0, s.alpha_core, s.cp_core);
        let q_dyn = q_cmb - q_ad + q_ic + q_mg;
        // A core frozen through has no liquid left to stir.
        let frozen = ric >= rc * 0.999;
        let on = q_dyn > 0.0 && !frozen;
        let field = if on {
            sse_atoms::magnetic::dynamo_field_rms(rho_c, q_dyn, s.field_c, s.field_f_ohm) * 1e6
        } else {
            0.0
        };
        let ago = age - t;
        if ric > 0.0 && ric_prev <= 0.0 && !first {
            events.push(CoreEvent {
                t_gyr: t,
                kind: "inner-core-birth".into(),
                why: format!(
                    "The core's centre cooled to the iron alloy's melting point ({:.0} K at {:.0} GPa), {:.2} Gyr ago: a solid inner core starts to grow, letting out latent heat and light elements that stir the liquid above.",
                    melt(p_at(0.0)),
                    p_at(0.0) / 1e9,
                    ago
                ),
            });
        }
        if on != on_prev || first {
            let (out, cond, light) = (tw(qc), tw(q_ad * a_c), tw((q_ic + q_mg) * a_c));
            let reason = if frozen {
                "the core has frozen through, leaving no liquid iron to stir".to_string()
            } else if on && q_cmb > q_ad {
                let extra = if q_ic + q_mg > 0.0 {
                    format!(", and light elements add the buoyancy of {light} more")
                } else {
                    String::new()
                };
                format!("{out} leaves the core, more than the {cond} conduction alone carries along its adiabat{extra}")
            } else if on {
                format!("{out} leaves the core, less than the {cond} conduction carries along its adiabat, but light elements let out as it freezes add the buoyancy of {light}")
            } else {
                format!("only {out} leaves the core, less than the {cond} conduction carries along its adiabat, so the liquid iron no longer convects")
            };
            if !(first && !on) {
                events.push(CoreEvent {
                    t_gyr: t,
                    kind: if on { "dynamo-start" } else { "dynamo-death" }.into(),
                    why: format!("{:.2} Gyr ago: {reason}.", ago),
                });
            }
            on_prev = on;
        }
        why_now = if on {
            format!(
                "A dynamo runs: {} leaves the core against {} conducted along its adiabat, field about {:.0} μT inside the core.",
                tw(qc),
                tw(q_ad * a_c),
                field
            )
        } else if frozen {
            "No dynamo: the core has frozen through.".into()
        } else {
            format!(
                "No dynamo: the core is not cooling fast enough ({} leaves it, conduction alone carries {}).",
                tw(qc),
                tw(q_ad * a_c)
            )
        };
        steps.push(CoreStep {
            t_gyr: t,
            t_mantle_k: t_u,
            t_cmb_k: t_c,
            q_core_tw: qc / 1e12,
            q_surface_tw: qs / 1e12,
            q_cmb_w_m2: q_cmb,
            q_adiabat_w_m2: q_ad,
            q_dynamo_w_m2: q_dyn.max(0.0),
            inner_core_km: ric / 1000.0,
            field_ut: field,
        });
        ric_prev = ric;
        first = false;
        if t >= age - 1e-12 {
            break;
        }
        let h = dt_gyr.min(age - t);
        t_c -= cool_c * h * sec;
        t_u -= cool_m * h * sec;
        t_u = t_u.clamp(t_s, s.t_mantle_max_k.max(t_s));
        // Water: lost to volcanism under a lid, carried back down by plates.
        let relax = 1.0 - m::exp(-h / s.degas_gyr.max(1e-6));
        wet += ((if mobile { 1.0 } else { 0.0 }) - wet) * relax;
        t += h;
    }
    Some(CoreHistory {
        version: CORE_VERSION.into(),
        core_radius_km: rc / 1000.0,
        core_density_kg_m3: rho_c,
        cmb_pressure_gpa: p_cmb / 1e9,
        dynamo_now: on_prev,
        why_now,
        steps,
        events,
    })
}
