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

use sse_core::sheet::{M_EARTH, R_EARTH_MEAN as R_EARTH, R_GAS};

pub const REFS: [&str; 11] = [
    "Stevenson, D. J., Spohn, T. and Schubert, G. 1983, Icarus 54, 466: magnetism and thermal evolution of the terrestrial planets. doi:10.1016/0019-1035(83)90241-5",
    "Driscoll, P. and Bercovici, D. 2014, Physics of the Earth and Planetary Interiors 236, 36: on the thermal and magnetic histories of Earth and Venus (lower-mantle viscosity jump, local critical Rayleigh number). doi:10.1016/j.pepi.2014.08.004",
    "Solomatov, V. S. and Moresi, L.-N. 2000, JGR 105, 21795: scaling of time-dependent stagnant lid convection. doi:10.1029/2000JB900197",
    "Labrosse, S. 2015, Physics of the Earth and Planetary Interiors 247, 36: thermal evolution of the core with a high thermal conductivity (adiabat, latent and gravitational energy). doi:10.1016/j.pepi.2015.02.002",
    "Anzellini, S. et al. 2013, Science 340, 464: melting of iron at Earth's inner core boundary (Simon-Glatzel fit). doi:10.1126/science.1233514",
    "O'Rourke, J. G. and Stevenson, D. J. 2016, Nature 529, 387: powering Earth's dynamo with magnesium precipitation from the core. doi:10.1038/nature16495",
    "Masters, G. and Gubbins, D. 2003, Physics of the Earth and Planetary Interiors 140, 159: on the resolution of density within the Earth (inner core boundary density jump). doi:10.1016/j.pepi.2003.07.008",
    "Christensen, U. R. 2010, Space Science Reviews 152, 565: dynamo scaling laws (the field from the convected buoyancy flux). doi:10.1007/s11214-009-9553-2",
    "Hirschmann, M. M. 2000, Geochemistry, Geophysics, Geosystems 1, 1042: mantle solidus, experimental constraints and the effects of peridotite composition (the dry solidus fit). doi:10.1029/2000GC000070",
    "Moore, W. B. and Webb, A. A. G. 2013, Nature 501, 501: heat-pipe Earth (volcanism carrying a lid world's heat). doi:10.1038/nature12473",
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
    /// as volcanism under moving plates (a cap standing in for a melt model).
    pub t_mantle_max_k: f64,
    /// The mantle's solidus, T = a P² + b P + c in K with P in GPa, for dry
    /// peridotite (Hirschmann 2000's °C fit, valid 0 to 10 GPa, with c
    /// moved to kelvin: 1120.661 °C = 1393.811 K). Under a lid that
    /// does not move, a mantle hotter than this at the lid's base melts, and
    /// the melt carries the excess heat up through the lid as volcanism (a
    /// heat pipe, Moore and Webb 2013), so the mantle cannot warm past it.
    pub mantle_solidus_c: [f64; 3],
    /// The pressure range the solidus fit holds over, GPa.
    pub mantle_solidus_p_max_gpa: f64,
    /// Iron's equation of state (Vinet, hcp iron; Dewaele et al. 2006):
    /// density at no pressure, kg/m³, bulk modulus, GPa, and its slope.
    pub iron_rho0: f64,
    pub iron_k0_gpa: f64,
    pub iron_k0_prime: f64,
    /// Core alloy's density below pure iron's, per unit mass fraction of
    /// light elements.
    pub density_deficit_per_light: f64,
    /// The march's grid (HF4b.1, melt design 6.1): a function of the body
    /// alone, never of the date asked for. The first step, Myr, each next
    /// one `grid_growth` times the last, capped at `step_myr`.
    pub grid_first_myr: f64,
    pub grid_growth: f64,
    pub step_myr: f64,
    /// Field strength law (Christensen 2010): c and f_ohm.
    pub field_c: f64,
    pub field_f_ohm: f64,
    /// MF2: electrical conductivity of the liquid alloy, S/m. A stand-in
    /// (Pozzo et al. 2012 give about 1.5e6) until the sheet's liquid iron
    /// row lands and Look up reads it.
    pub sigma_core: f64,
    /// Spin for a world that publishes none, hours (the climate's default).
    pub spin_hours_default: f64,
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
            mantle_solidus_c: [-5.104, 132.899, 1393.811],
            mantle_solidus_p_max_gpa: 10.0,
            iron_rho0: 8268.0,
            iron_k0_gpa: 163.4,
            iron_k0_prime: 5.38,
            density_deficit_per_light: 1.0,
            grid_first_myr: 1.0,
            grid_growth: 1.25,
            step_myr: 20.0,
            field_c: 0.63,
            field_f_ohm: 0.9,
            sigma_core: crate::giant_field::core_sigma().unwrap_or(1.0e6),
            spin_hours_default: 24.0,
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
    /// MF2, the field ledger's Make test: the stirring flow's speed (m/s),
    /// the magnetic Reynolds number of the liquid shell, and how far the
    /// dynamo runs (0 to 1).
    pub flow_m_s: f64,
    pub rm: f64,
    pub onset: f64,
    /// rms field inside the core, μT (0: no dynamo).
    pub field_ut: f64,
    /// HF4b.1, melt design section 7: where the heat goes, W/m² of surface,
    /// one set of names. They close at every step:
    /// heat_lost = heat_conducted + heat_erupted;
    /// heat_made = heat_radiogenic + heat_tidal;
    /// heat_lost = heat_made + mantle_cooling + lid_cooling + core_heat;
    /// core_heat = core_freezing + core_cooling.
    /// A store's cooling is negative while it warms.
    #[serde(default)]
    pub heat_lost: f64,
    #[serde(default)]
    pub heat_conducted: f64,
    #[serde(default)]
    pub heat_erupted: f64,
    #[serde(default)]
    pub heat_made: f64,
    #[serde(default)]
    pub heat_radiogenic: f64,
    #[serde(default)]
    pub heat_tidal: f64,
    #[serde(default)]
    pub core_heat: f64,
    #[serde(default)]
    pub core_freezing: f64,
    #[serde(default)]
    pub core_cooling: f64,
    #[serde(default)]
    pub mantle_cooling: f64,
    #[serde(default)]
    pub lid_cooling: f64,
    /// The stiff lid's thickness (a plate's, under moving plates), and the
    /// crust's (zero until HF4b.2 makes crust), km.
    #[serde(default)]
    pub lid_km: f64,
    #[serde(default)]
    pub crust_km: f64,
    /// Melt made and erupted, m of rock over the whole surface per Myr, and
    /// the melt zone's liquid share (zero until HF4b.2).
    #[serde(default)]
    pub melt_made: f64,
    #[serde(default)]
    pub melt_erupted: f64,
    #[serde(default)]
    pub melt_retained: f64,
}

impl CoreStep {
    /// The step a share `f` of the way from `a` to `b`: every number
    /// linearly. A read between two grid points (melt design 6.1).
    pub(crate) fn between(a: &CoreStep, b: &CoreStep, f: f64) -> CoreStep {
        let l = |x: f64, y: f64| x + (y - x) * f;
        CoreStep {
            t_gyr: l(a.t_gyr, b.t_gyr),
            t_mantle_k: l(a.t_mantle_k, b.t_mantle_k),
            t_cmb_k: l(a.t_cmb_k, b.t_cmb_k),
            q_core_tw: l(a.q_core_tw, b.q_core_tw),
            q_surface_tw: l(a.q_surface_tw, b.q_surface_tw),
            q_cmb_w_m2: l(a.q_cmb_w_m2, b.q_cmb_w_m2),
            q_adiabat_w_m2: l(a.q_adiabat_w_m2, b.q_adiabat_w_m2),
            q_dynamo_w_m2: l(a.q_dynamo_w_m2, b.q_dynamo_w_m2),
            inner_core_km: l(a.inner_core_km, b.inner_core_km),
            flow_m_s: l(a.flow_m_s, b.flow_m_s),
            rm: l(a.rm, b.rm),
            onset: l(a.onset, b.onset),
            field_ut: l(a.field_ut, b.field_ut),
            heat_lost: l(a.heat_lost, b.heat_lost),
            heat_conducted: l(a.heat_conducted, b.heat_conducted),
            heat_erupted: l(a.heat_erupted, b.heat_erupted),
            heat_made: l(a.heat_made, b.heat_made),
            heat_radiogenic: l(a.heat_radiogenic, b.heat_radiogenic),
            heat_tidal: l(a.heat_tidal, b.heat_tidal),
            core_heat: l(a.core_heat, b.core_heat),
            core_freezing: l(a.core_freezing, b.core_freezing),
            core_cooling: l(a.core_cooling, b.core_cooling),
            mantle_cooling: l(a.mantle_cooling, b.mantle_cooling),
            lid_cooling: l(a.lid_cooling, b.lid_cooling),
            lid_km: l(a.lid_km, b.lid_km),
            crust_km: l(a.crust_km, b.crust_km),
            melt_made: l(a.melt_made, b.melt_made),
            melt_erupted: l(a.melt_erupted, b.melt_erupted),
            melt_retained: l(a.melt_retained, b.melt_retained),
        }
    }
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

use sse_atoms::ledger as lg;
pub use sse_atoms::magnetic::{adiabatic_flux, compositional_flux};

/// The magnetic Reynolds number at which the ledger's dynamo starts.
pub fn onset_from() -> f64 {
    lg::ONSET_LO
}

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
/// solid interior). `heat_w_m2(t)` is the heat made at `t` Gyr, W/m² of
/// surface: radiogenic, tidal.
pub fn core_history(
    s: &CoreSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    heat_w_m2: &dyn Fn(f64) -> (f64, f64),
) -> Option<CoreHistory> {
    if b.deep_envelope || b.core_mass_fraction < 0.01 || b.mass_earth <= 0.0 {
        return None;
    }
    let g_n = sse_core::constants::g();
    // The spin the dynamo turns with: the orbit's period when locked, else
    // the body's own, else the climate's default.
    let spin_s = if env.locked {
        b.orbit
            .as_ref()
            .filter(|o| o.a_au > 0.0 && o.host_msun > 0.0)
            .map(|o| {
                let a = o.a_au * sse_core::sheet::AU;
                2.0 * m::PI * m::sqrt(a * a * a / (o.host_msun * sse_core::sheet::GM_SUN))
            })
    } else {
        None
    }
    .or_else(|| b.spin_hours.map(|h| h.abs() * 3600.0).filter(|x| *x > 0.0))
    .unwrap_or(s.spin_hours_default * 3600.0);
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
    // The grid (melt design 6.1): from the march's start, each step
    // `grid_growth` times the last, capped; a function of the body alone.
    let step_cap = (s.step_myr / 1000.0).max(1e-4);
    let growth = s.grid_growth.max(1.0);
    let mut h_next = (s.grid_first_myr / 1000.0).clamp(1e-6, step_cap);
    let sec = 3.155_76e16; // seconds a Gyr
    let mut steps = vec![];
    let mut events = vec![];
    let mut t = start;
    let mut nucleated = false;
    let mut ric_prev = 0.0;
    let mut on_prev = false;
    let mut first = true;
    let mut why_now;
    // Heat melting carried out through the lid over the last step, W.
    let mut vented = 0.0;
    let solidus = |p_pa: f64| {
        let p = (p_pa / 1e9).clamp(0.0, s.mantle_solidus_p_max_gpa);
        let [a, b, c] = s.mantle_solidus_c;
        a * p * p + b * p + c
    };
    // Heat out of the mantle's top, W/m², for a mantle at `t_u`: moving
    // plates, or a stagnant lid (Solomatov and Moresi 2000).
    let top = |t_u: f64, mobile: bool, wet: f64| {
        let dtm = (t_u - t_s).max(0.0);
        let ra = s.alpha_mantle * rho_m * g_s * dtm * d * d * d / (s.kappa_mantle * eta(t_u, wet));
        if mobile {
            s.k_mantle * dtm / d * m::cbrt((ra / s.ra_critical).max(0.0))
        } else {
            let theta = s.activation_j_mol * dtm / (R_GAS * t_u * t_u);
            0.53 * s.k_mantle * dtm / d * m::pow(theta.max(1.0), -4.0 / 3.0) * m::cbrt(ra.max(0.0))
        }
    };
    // HF4b.1: the stiff lid is kept state (melt design 3.2). Its base is a
    // fixed step below the mantle (`sse.heat.lid_base_temperature`), it
    // conducts what its thickness allows (`sse.heat.lid_conduction_with_burial`,
    // nothing buried until HF4b.2), and it thickens when its base loses more
    // than the mantle delivers (`sse.heat.lid_growth`, the Stefan rule,
    // stepped backward so it is stable at any step: a young lid grows as √t
    // and its surface heat falls as 1/√t while the mantle below loses only
    // what convection brings it).
    let ph = sse_physics::in_force();
    let block = |id: &str, x: &[f64]| -> Vec<f64> {
        ph.blocks()
            .run(id, x)
            .unwrap_or_else(|e| panic!("base block {id}: {e}"))
    };
    let lid_base = |t_u: f64| {
        block("sse.heat.lid_base_temperature", &[t_u, s.activation_j_mol])[0]
            .clamp(t_s, t_u.max(t_s))
    };
    let conduct = |t_l: f64, lid: f64| {
        block(
            "sse.heat.lid_conduction_with_burial",
            &[s.k_mantle, t_l, t_s, lid, 0.0, 0.0, s.kappa_mantle],
        )[0]
    };
    // The lid's backward step: ρc(T_m − T_l)(D′ − D)/h = k(T_l − T_s)/D′ − q_m,
    // the positive root of its quadratic in D′.
    let grow = |lid: f64, t_u: f64, q_m: f64, h_s: f64| {
        let t_l = lid_base(t_u);
        let a2 = rho_m * s.cp_mantle * (t_u - t_l).max(1.0);
        let bq = a2 * lid - h_s * q_m;
        let c = h_s * s.k_mantle * (t_l - t_s).max(0.0);
        ((bq + m::sqrt(bq * bq + 4.0 * a2 * c)) / (2.0 * a2)).clamp(1.0, d.max(1.0))
    };
    // A plate's thickness: the rock that conducts what the mantle loses.
    let plate = |t_u: f64, q: f64| (s.k_mantle * (t_u - t_s) / q.max(1e-12)).clamp(1.0, d.max(1.0));
    let mut lid = if mobile_at(t) {
        plate(t_u, top(t_u, true, wet))
    } else {
        grow(0.0, t_u, 0.0, h_next * sec)
    };
    // The hottest the mantle can be. Under moving plates, the cap. Under a
    // lid, the solidus at the lid's base, the lid as thick as the rock that
    // conducts what the mantle's top loses: a hotter mantle loses more, so
    // its lid is thinner and its base melts sooner. The temperature that is
    // its own lid's solidus, by bisection.
    let hottest = |mobile: bool, wet: f64| {
        let max = s.t_mantle_max_k.max(t_s);
        if mobile {
            return max;
        }
        let over = |t_u: f64| {
            let lid = (s.k_mantle * (t_u - t_s) / top(t_u, false, wet).max(1e-12)).clamp(0.0, d);
            t_u - solidus(rho_m * g_s * lid)
        };
        if over(max) <= 0.0 {
            return max;
        }
        let (mut lo, mut hi) = (t_s, max);
        for _ in 0..50 {
            let mid = 0.5 * (lo + hi);
            if over(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        lo
    };
    let mut why_kept = String::new();
    let mut on_kept = false;
    loop {
        // Fluxes at this state.
        let mobile = mobile_at(t);
        let q_top = top(t_u, mobile, wet);
        if mobile {
            lid = plate(t_u, q_top);
        }
        let q_cond = if mobile {
            q_top
        } else {
            conduct(lid_base(t_u), lid)
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
        let (radiogenic, tidal) = heat_w_m2(t);
        let heat = (radiogenic + tidal) * a_s;
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
        // MF2, the ledger's Make test on the liquid shell: the flow the
        // power drives, whether it outruns the leak (Rm), and the spin's
        // factor (1 for every planet).
        let shell = (rc - ric).max(0.0);
        let (flow, rm, onset, spin_factor) = if q_dyn > 0.0 && !frozen && shell > 0.0 {
            let (u, _) = lg::flow_speed(
                q_dyn,
                g_c,
                s.alpha_core,
                rho_c,
                s.cp_core,
                spin_s,
                shell,
                lg::FLOW_C,
                lg::FLOW_B,
            );
            let (rm, _) = lg::reynolds(u, shell, s.sigma_core, sse_core::sheet::MU0);
            let (_, f) = lg::rossby(spin_s, shell, u, lg::ROSSBY_SAT, lg::ROSSBY_SLOPE);
            (u, rm, lg::onset(rm, lg::ONSET_LO, lg::ONSET_HI), f)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };
        let on = onset > 0.0;
        let field = if on {
            sse_atoms::magnetic::dynamo_field_rms(rho_c, q_dyn, s.field_c, s.field_f_ohm)
                * 1e6
                * onset
                * spin_factor
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
            } else if !on && q_dyn > 0.0 {
                format!("the liquid iron still stirs, but at {:.2} mm/s its field leaks away faster than it is stretched (magnetic Reynolds number {rm:.0}, a dynamo needs about {:.0})", flow * 1e3, lg::ONSET_LO)
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
                "A dynamo runs: {} leaves the core against {} conducted along its adiabat, stirring the liquid at {:.2} mm/s (magnetic Reynolds number {rm:.0}), field about {:.0} μT inside the core.",
                tw(qc),
                tw(q_ad * a_c),
                flow * 1e3,
                field
            )
        } else if frozen {
            "No dynamo: the core has frozen through.".into()
        } else if q_dyn > 0.0 {
            format!(
                "No dynamo: the core stirs at {:.2} mm/s, too slowly to outrun the field's leak (magnetic Reynolds number {rm:.0}, below about {:.0}).",
                flow * 1e3,
                lg::ONSET_LO
            )
        } else {
            format!(
                "No dynamo: the core is not cooling fast enough ({} leaves it, conduction alone carries {}).",
                tw(qc),
                tw(q_ad * a_c)
            )
        };
        let erupted = vented / a_s;
        let core_heat = qc / a_s;
        let core_freezing = freeze * cool_c / a_s;
        if t <= age + 1e-12 {
            why_kept = why_now.clone();
            on_kept = on;
        }
        steps.push(CoreStep {
            t_gyr: t,
            t_mantle_k: t_u,
            t_cmb_k: t_c,
            q_core_tw: qc / 1e12,
            q_surface_tw: (q_cond + erupted) * a_s / 1e12,
            q_cmb_w_m2: q_cmb,
            q_adiabat_w_m2: q_ad,
            q_dynamo_w_m2: q_dyn.max(0.0),
            inner_core_km: ric / 1000.0,
            flow_m_s: flow,
            rm,
            onset,
            field_ut: field,
            heat_lost: q_cond + erupted,
            heat_conducted: q_cond,
            heat_erupted: erupted,
            heat_made: radiogenic + tidal,
            heat_radiogenic: radiogenic,
            heat_tidal: tidal,
            core_heat,
            core_freezing,
            core_cooling: core_heat - core_freezing,
            mantle_cooling: q_top + erupted - (radiogenic + tidal) - core_heat,
            lid_cooling: q_cond - q_top,
            lid_km: lid / 1000.0,
            crust_km: 0.0,
            melt_made: 0.0,
            melt_erupted: 0.0,
            melt_retained: 0.0,
        });
        ric_prev = ric;
        first = false;
        if t >= age - 1e-12 {
            break;
        }
        // March on the fixed grid past the body's age; the read at its age
        // falls between two grid points (6.1).
        let h = h_next;
        h_next = (h_next * growth).min(step_cap);
        if !mobile {
            lid = grow(lid, t_u, q_top, h * sec);
        }
        t_c -= cool_c * h * sec;
        t_u -= cool_m * h * sec;
        // Melting carries out whatever would lift the mantle past the
        // hottest it can be, as volcanism through the lid. HF1's stand-in,
        // read on the steady lid it was built on, not the kept one: on the
        // kept young lid (a few km thick) it caps a young mantle at the
        // surface solidus and vents 500 K of it in one step, which no melt
        // supply could carry and which killed Mars's field at birth.
        // HF4b.2 replaces the cap with melting itself.
        let cap = hottest(mobile, wet);
        vented = if t_u > cap {
            (t_u - cap) * mm * s.cp_mantle * eps_m / (h * sec)
        } else {
            0.0
        };
        t_u = t_u.clamp(t_s, cap);
        // Water: lost to volcanism under a lid, carried back down by plates.
        let relax = 1.0 - m::exp(-h / s.degas_gyr.max(1e-6));
        wet += ((if mobile { 1.0 } else { 0.0 }) - wet) * relax;
        t += h;
    }
    // The read at the body's age: between the last two grid points.
    let n = steps.len();
    if n >= 2 && steps[n - 1].t_gyr > age + 1e-12 {
        let (a, b) = (&steps[n - 2], &steps[n - 1]);
        let f = ((age - a.t_gyr) / (b.t_gyr - a.t_gyr)).clamp(0.0, 1.0);
        steps[n - 1] = CoreStep::between(a, b, f);
    }
    events.retain(|e| e.t_gyr <= age + 1e-12);
    let (on_prev, why_now) = (on_kept, why_kept);
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

/// MF2: the field facts a world publishes now, from its core history's last
/// step (the ledger's answer at the world's age): the rms field inside the
/// dynamo, the equatorial dipole at the surface and the dipole moment (the
/// `sse.magnetic.surface_dipole` sum, its dipole ratio until MF3's shape),
/// the dynamo's outer radius, the stirring flow, Rm and onset. SI units.
/// None for a world without an iron core history.
pub fn field_facts(env: &SurfaceEnvelope, b: &Body) -> Option<Vec<(&'static str, f64)>> {
    if b.deep_envelope {
        return crate::giant_field::field_facts(b);
    }
    let c = env.core.as_ref()?;
    let now = c.steps.last()?;
    let rms_t = now.field_ut * 1e-6;
    let rc = c.core_radius_km * 1000.0;
    let r = b.radius_earth * R_EARTH;
    let (b_eq, moment) = if rms_t > 0.0 && r > 0.0 {
        sse_atoms::magnetic::surface_dipole(rms_t, rc, r, sse_atoms::magnetic::DIPOLE_RATIO)
    } else {
        (0.0, 0.0)
    };
    Some(vec![
        ("magnetic.dynamo_field", rms_t),
        ("body.surface_magnetic_field", b_eq),
        ("body.magnetic_moment", moment),
        ("interior.dynamo_radius", rc),
        (
            "interior.dynamo_shell_thickness",
            rc - now.inner_core_km * 1000.0,
        ),
        ("interior.dynamo_flow_speed", now.flow_m_s),
        ("interior.magnetic_reynolds", now.rm),
        ("interior.dynamo_onset", now.onset),
    ])
}
