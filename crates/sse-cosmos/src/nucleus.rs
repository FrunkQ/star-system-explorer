//! Galactic nuclei: supermassive black holes, their seeds, accretion,
//! active phases (quasars, Seyferts, low-luminosity nuclei), mergers and
//! tidal disruptions (star census, galaxy scale; kernel G track).
//!
//! G3's reservoir already grows each galaxy's black hole: steadily with its
//! star formation and in merger or authored bursts (`Galaxy::active_nuclei`).
//! This module reads that growth as physics a chart can show:
//! - **Seeds.** A halo that is already an atomic-cooling halo (virial speed
//!   above about 16.5 km/s, T_vir 1e4 K) by z = 10 may host a direct-collapse
//!   seed of about 1e5 Msun (Loeb and Rasio 1994; Bromm and Loeb 2003);
//!   otherwise the seed is a Population III remnant of about 100 Msun, and
//!   the module says how long Eddington-limited growth takes to reach the
//!   reservoir's starting mass. A keyed draw decides, with the chance as pack data.
//! - **Accretion and light.** Steady feeding follows star formation; a burst
//!   is swallowed at the Eddington limit, so it lasts t_Sal ln(1 + feed / M)
//!   (Salpeter 1964). Luminosity is eps Mdot c^2 for a thin disc
//!   (Shakura and Sunyaev 1973), falling as the Eddington ratio squared below
//!   about 1% where the flow turns radiatively inefficient (Narayan and Yi 1995).
//! - **Class** from luminosity and Eddington ratio: quasar, Seyfert,
//!   low-luminosity (LINER-like), dormant. Blazar, type 1 or type 2 depend on
//!   where the observer stands relative to the jet axis and the dusty torus
//!   (unified model, Urry and Padovani 1995): `seen_from`.
//! - **Mergers.** Each major galaxy merger brings a second hole, which
//!   coalesces after a delay (dynamical friction, hardening), with its chirp
//!   mass for gravitational waves.
//! - **Tidal disruptions.** Rate per galaxy falls gently with hole mass and
//!   stops above the Hills mass, where Sun-like stars are swallowed whole
//!   (Stone and Metzger 2016; Hills 1975).
//!
//! The nucleus is published as a body at the halo centre (`<halo>/nucleus:0`)
//! with the same body facts as any body, so the one-body plan can classify it.
//! Deterministic: fixed steps, keyed draws, no wall clock.

use crate::cosmology::Cosmology;
use crate::galaxy::{mergers_with, Edit, Galaxy, GalaxyParams};
use crate::haloes::{poisson, Halo};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};
use std::collections::BTreeMap;

const M_SUN_KG: f64 = 1.988_47e30;
const C_M_S: f64 = 299_792_458.0;
const G_SI: f64 = 6.674_30e-11;
const SIGMA_SB: f64 = 5.670_374_419e-8;
const YEAR_S: f64 = 31_557_600.0;
/// Eddington luminosity per solar mass (ionised hydrogen), W.
pub const L_EDD_PER_MSUN: f64 = 1.257e31;
/// Eddington time sigma_T c / (4 pi G m_p), Gyr.
pub const T_EDD_GYR: f64 = 0.4505;

/// Pack settings (`provides.other.cosmology.nucleus`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct NucleusParams {
    /// Thin-disc radiative efficiency (0.057 non-spinning to 0.42 maximal; 0.1 typical, Soltan argument).
    pub radiative_efficiency: f64,
    /// Below this Eddington ratio the flow is radiatively inefficient (L falls as lambda^2).
    pub thin_lambda: f64,
    /// Class thresholds: bolometric luminosity (W) and Eddington ratio.
    pub quasar_w: f64,
    pub seyfert_w: f64,
    pub seyfert_lambda: f64,
    pub active_lambda: f64,
    /// Seeds.
    pub light_seed_msun: f64,
    pub heavy_seed_msun: f64,
    pub direct_collapse_z: f64,
    pub direct_collapse_v_kms: f64,
    pub direct_collapse_chance: f64,
    /// Radio-loud share at 1e8 Msun, its slope with hole mass, and its cap (Best et al. 2005
    /// section 3.3 fit at 1e23 W/Hz; the fit breaks down above about 0.25, a ceiling near 30%).
    pub radio_loud_1e8: f64,
    pub radio_loud_slope: f64,
    pub radio_loud_max: f64,
    /// Unified model: torus half-opening and blazar cone, degrees from the jet axis.
    pub torus_deg: f64,
    pub blazar_deg: f64,
    /// Delay from a galaxy merger to the holes' coalescence, Gyr.
    pub merge_delay_gyr: f64,
    /// Tidal disruptions per year at 1e8 Msun and the slope with mass; none above the Hills mass.
    pub tde_rate_1e8: f64,
    pub tde_slope: f64,
    pub hills_mass_msun: f64,
}

impl Default for NucleusParams {
    fn default() -> Self {
        NucleusParams {
            radiative_efficiency: 0.1,
            thin_lambda: 0.01,
            quasar_w: 1.0e38,
            seyfert_w: 1.0e35,
            seyfert_lambda: 1.0e-3,
            active_lambda: 1.0e-5,
            light_seed_msun: 100.0,
            heavy_seed_msun: 1.0e5,
            direct_collapse_z: 10.0,
            direct_collapse_v_kms: 16.5,
            direct_collapse_chance: 0.1,
            radio_loud_1e8: 0.0125,
            radio_loud_slope: 1.6,
            radio_loud_max: 0.3,
            torus_deg: 45.0,
            blazar_deg: 10.0,
            merge_delay_gyr: 1.0,
            tde_rate_1e8: 2.9e-5,
            tde_slope: -0.404,
            hills_mass_msun: 1.1e8,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NucleusClass {
    Quasar,
    Seyfert,
    LowLuminosity,
    Dormant,
}

impl NucleusClass {
    pub fn name(&self) -> &'static str {
        match self {
            NucleusClass::Quasar => "quasar",
            NucleusClass::Seyfert => "seyfert",
            NucleusClass::LowLuminosity => "low_luminosity",
            NucleusClass::Dormant => "dormant",
        }
    }
}

/// The nucleus at one time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NucleusPoint {
    pub t_gyr: f64,
    pub mass_msun: f64,
    /// Mass accreted per year (before radiative losses), Msun/yr.
    pub mdot_msun_yr: f64,
    pub eddington_ratio: f64,
    pub luminosity_w: f64,
    /// Peak temperature of a thin disc around a non-spinning hole, K (0 when not thin).
    pub disc_peak_k: f64,
    pub class: NucleusClass,
}

/// A burst swallowed at the Eddington limit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub start_gyr: f64,
    pub end_gyr: f64,
    pub feed_msun: f64,
    pub peak_luminosity_w: f64,
    pub class: NucleusClass,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HoleMerger {
    pub galaxy_merger_gyr: f64,
    /// When the holes coalesce (None: still a pair today).
    pub coalesce_gyr: Option<f64>,
    pub primary_msun: f64,
    pub secondary_msun: f64,
    /// Chirp mass (m1 m2)^(3/5) / (m1 + m2)^(1/5), Msun: what a gravitational-wave detector measures.
    pub chirp_msun: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SeedInfo {
    /// `direct_collapse` or `pop3_remnant`.
    pub channel: String,
    pub seed_msun: f64,
    pub born_gyr: f64,
    /// When Eddington-limited growth reaches the reservoir's starting mass
    /// (G3 starts every hole at `seed_black_hole`).
    pub reaches_reservoir_gyr: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Nucleus {
    pub halo: String,
    /// The body's address: `<halo>/nucleus:0`.
    pub address: String,
    pub seed: SeedInfo,
    pub history: Vec<NucleusPoint>,
    pub episodes: Vec<Episode>,
    pub mergers: Vec<HoleMerger>,
    pub radio_loud: bool,
    /// Unit vector of the spin and jet axis (in the galaxy's frame).
    pub jet_axis: [f64; 3],
    pub now: NucleusPoint,
    /// Tidal disruptions per year now.
    pub tde_rate_per_yr: f64,
}

fn stream(s: &str) -> Stream {
    Stream::new(s).expect("fixed stream names are valid")
}

impl NucleusParams {
    /// Salpeter (e-folding) time of Eddington-limited growth, Gyr.
    pub fn salpeter_gyr(&self) -> f64 {
        let e = self.radiative_efficiency;
        T_EDD_GYR * e / (1.0 - e)
    }

    /// Luminosity, Eddington ratio and disc peak temperature for a hole of
    /// `mass` accreting `mdot` (Msun/yr).
    pub fn light(&self, mass: f64, mdot: f64) -> (f64, f64, f64) {
        if mass <= 0.0 || mdot <= 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let mdot_kg_s = mdot * M_SUN_KG / YEAR_S;
        let thin = self.radiative_efficiency * mdot_kg_s * C_M_S * C_M_S;
        let l_edd = L_EDD_PER_MSUN * mass;
        let raw = thin / l_edd;
        let (l, lambda) = if raw >= self.thin_lambda {
            (thin, raw)
        } else {
            // Radiatively inefficient: L falls as the square of the accretion rate.
            let l = thin * raw / self.thin_lambda;
            (l, l / l_edd)
        };
        let peak = if raw >= self.thin_lambda {
            // Shakura-Sunyaev: T(r)^4 = 3 G M Mdot / (8 pi sigma r^3) (1 - sqrt(r_in / r)),
            // r_in = 6 G M / c^2, peak at r = (49/36) r_in.
            let m_kg = mass * M_SUN_KG;
            let r_in = 6.0 * G_SI * m_kg / (C_M_S * C_M_S);
            let r = 49.0 / 36.0 * r_in;
            let t4 = 3.0 * G_SI * m_kg * mdot_kg_s / (8.0 * m::PI * SIGMA_SB * r * r * r)
                * (1.0 - m::sqrt(r_in / r));
            m::sqrt(m::sqrt(t4))
        } else {
            0.0
        };
        (l, lambda, peak)
    }

    pub fn class(&self, l: f64, lambda: f64) -> NucleusClass {
        if l >= self.quasar_w {
            NucleusClass::Quasar
        } else if l >= self.seyfert_w && lambda >= self.seyfert_lambda {
            NucleusClass::Seyfert
        } else if lambda >= self.active_lambda {
            NucleusClass::LowLuminosity
        } else {
            NucleusClass::Dormant
        }
    }

    pub fn tde_rate(&self, mass: f64) -> f64 {
        if mass <= 0.0 || mass > self.hills_mass_msun {
            0.0
        } else {
            self.tde_rate_1e8 * m::pow(mass / 1.0e8, self.tde_slope)
        }
    }
}

/// Time of redshift z on the galaxy grid, Gyr.
fn t_of_z(c: &Cosmology, z: f64) -> f64 {
    c.time_of_scale(1.0 / (1.0 + z))
}

/// The nucleus of a galaxy evolved by G3 (with the same edits).
pub fn nucleus(
    c: &Cosmology,
    gp: &GalaxyParams,
    np: &NucleusParams,
    halo: &Halo,
    g: &Galaxy,
    edits: &[Edit],
    seed: Seed,
) -> Nucleus {
    let addr: Address = halo.address.parse().expect("halo addresses are valid");
    let draws = Draws::new(seed, addr.clone(), stream("cosmos.nucleus.v1"));
    // Seed channel: an atomic-cooling halo by z_dc may collapse directly.
    let z_dc = np.direct_collapse_z;
    let a_dc = 1.0 / (1.0 + z_dc);
    let (_, v_dc) = gp.virial(c, halo.mass_at(z_dc), a_dc);
    let t_s = np.salpeter_gyr();
    let seed_info = if v_dc >= np.direct_collapse_v_kms && draws.unit(0) < np.direct_collapse_chance
    {
        SeedInfo {
            channel: "direct_collapse".into(),
            seed_msun: np.heavy_seed_msun,
            born_gyr: t_of_z(c, z_dc),
            reaches_reservoir_gyr: t_of_z(c, z_dc)
                + t_s * m::ln((gp.seed_black_hole / np.heavy_seed_msun).max(1.0)),
        }
    } else {
        let born = g.t_start_gyr;
        SeedInfo {
            channel: "pop3_remnant".into(),
            seed_msun: np.light_seed_msun,
            born_gyr: born,
            reaches_reservoir_gyr: born
                + t_s * m::ln((gp.seed_black_hole / np.light_seed_msun).max(1.0)),
        }
    };
    // Bursts swallowed at the Eddington limit.
    let mass_hist = |t: f64| -> f64 {
        let i = g.history.partition_point(|s| s.t_gyr <= t);
        if i == 0 {
            gp.seed_black_hole
        } else {
            g.history[i - 1].black_hole
        }
    };
    let mut episodes = vec![];
    for &(tm, feed) in &g.active_nuclei {
        let m0 = (mass_hist(tm) - feed).max(gp.seed_black_hole);
        let dur = t_s * m::ln(1.0 + feed / m0);
        let peak = L_EDD_PER_MSUN * (m0 + feed);
        episodes.push(Episode {
            start_gyr: tm,
            end_gyr: tm + dur,
            feed_msun: feed,
            peak_luminosity_w: peak,
            class: np.class(peak, 1.0),
        });
    }
    // The history: steady accretion from star formation plus any episode under way.
    let eps = np.radiative_efficiency;
    let point = |t: f64, mass_g3: f64, sfr: f64| -> NucleusPoint {
        let mut mass = mass_g3;
        let mut mdot = gp.black_hole_steady * sfr / (1.0 - eps);
        for e in &episodes {
            if t >= e.start_gyr && t < e.end_gyr {
                // G3 booked the whole feed at once; part of it is still to come.
                let m0 = (mass_hist(e.start_gyr) - e.feed_msun).max(gp.seed_black_hole);
                let grown = m0 * (m::exp((t - e.start_gyr) / t_s) - 1.0);
                mass -= (e.feed_msun - grown).max(0.0);
                mdot += (m0 + grown) / (t_s * 1.0e9) / (1.0 - eps);
            }
        }
        let (l, lambda, peak) = np.light(mass, mdot);
        NucleusPoint {
            t_gyr: t,
            mass_msun: mass,
            mdot_msun_yr: mdot,
            eddington_ratio: lambda,
            luminosity_w: l,
            disc_peak_k: peak,
            class: np.class(l, lambda),
        }
    };
    let history: Vec<NucleusPoint> = g
        .history
        .iter()
        .map(|s| point(s.t_gyr, s.black_hole, s.sfr))
        .collect();
    let now = point(g.now.t_gyr, g.now.black_hole, g.now.sfr);
    // Mergers: the incoming galaxy's hole, in proportion to the mass ratio.
    let now_gyr = g.now.t_gyr;
    let mergers = mergers_with(halo, edits)
        .into_iter()
        .filter(|(t, _)| *t >= g.t_start_gyr && *t <= now_gyr)
        .map(|(t, ratio)| {
            let m1 = mass_hist(t);
            let m2 = ratio * m1;
            let tc = t + np.merge_delay_gyr;
            HoleMerger {
                galaxy_merger_gyr: t,
                coalesce_gyr: (tc <= now_gyr).then_some(tc),
                primary_msun: m1,
                secondary_msun: m2,
                chirp_msun: m::pow(m1 * m2, 0.6) / m::pow(m1 + m2, 0.2),
            }
        })
        .collect();
    // Radio loudness and the jet axis: keyed, isotropic axis.
    let f_rl = (np.radio_loud_1e8 * m::pow(now.mass_msun / 1.0e8, np.radio_loud_slope))
        .min(np.radio_loud_max);
    let radio_loud = draws.unit(1) < f_rl;
    let cz = 2.0 * draws.unit(2) - 1.0;
    let ph = m::TAU * draws.unit(3);
    let sz = m::sqrt((1.0 - cz * cz).max(0.0));
    let jet_axis = [sz * m::cos(ph), sz * m::sin(ph), cz];
    Nucleus {
        halo: halo.address.clone(),
        address: addr
            .child_key("nucleus", "0")
            .expect("valid key")
            .to_string(),
        seed: seed_info,
        tde_rate_per_yr: np.tde_rate(now.mass_msun),
        history,
        episodes,
        mergers,
        radio_loud,
        jet_axis,
        now,
    }
}

/// How an active nucleus looks from direction `to_observer` (unit vector,
/// galaxy frame): `blazar` (a radio-loud jet within the blazar cone),
/// `type_1` (broad lines seen past the torus), `type_2` (hidden by it), or
/// the class itself when dormant or low-luminosity.
pub fn seen_from(
    n: &Nucleus,
    np: &NucleusParams,
    p: &NucleusPoint,
    to_observer: [f64; 3],
) -> String {
    match p.class {
        NucleusClass::Dormant | NucleusClass::LowLuminosity => return p.class.name().into(),
        _ => {}
    }
    let a = n.jet_axis;
    let norm = m::sqrt(to_observer.iter().map(|x| x * x).sum::<f64>()).max(1e-300);
    let c = ((a[0] * to_observer[0] + a[1] * to_observer[1] + a[2] * to_observer[2]) / norm).abs();
    let deg = m::acos(c.min(1.0)) * 180.0 / m::PI;
    if n.radio_loud && deg < np.blazar_deg {
        "blazar".into()
    } else if deg < np.torus_deg {
        "type_1".into()
    } else {
        "type_2".into()
    }
}

/// Tidal disruptions in [from_gyr, to_gyr): keyed per 10 kyr bin, so any
/// window gives the same events as the bins it covers. Times in Gyr.
pub fn tidal_disruptions(
    n: &Nucleus,
    np: &NucleusParams,
    seed: Seed,
    from_gyr: f64,
    to_gyr: f64,
) -> Vec<f64> {
    let addr: Address = n.address.parse().expect("valid");
    let d = Draws::new(seed, addr, stream("cosmos.nucleus.tde.v1"));
    let mass_at = |t: f64| {
        let i = n.history.partition_point(|p| p.t_gyr <= t);
        if i == 0 {
            n.history.first().map(|p| p.mass_msun).unwrap_or(0.0)
        } else {
            n.history[i - 1].mass_msun
        }
    };
    // Bins of 10 kyr: one Poisson draw each (well under one event in any
    // hole that can disrupt a star), times in their own index space.
    const BINS_PER_GYR: f64 = 1.0e5;
    let mut out = vec![];
    let b0 = (m::floor(from_gyr * BINS_PER_GYR) as i64).max(0);
    let b1 = (m::ceil(to_gyr * BINS_PER_GYR) as i64).min(b0 + 2_000_000);
    for b in b0..b1 {
        let t = b as f64 / BINS_PER_GYR;
        let lambda = np.tde_rate(mass_at(t)) * 1.0e9 / BINS_PER_GYR;
        let k = poisson(&d, b as u64, lambda);
        for j in 0..k.min(64) {
            let at = t + d.unit((1u64 << 62) + (b as u64) * 64 + j) / BINS_PER_GYR;
            if at >= from_gyr && at < to_gyr {
                out.push(at);
            }
        }
    }
    out
}

/// The nucleus as body facts at its point `p` (SI, the importer's names
/// where they exist), for the chronicle and the one-body classifier.
pub fn facts(n: &Nucleus, p: &NucleusPoint) -> BTreeMap<String, Value> {
    let f = |x: f64| Value::Float(x);
    let t = |s: &str| Value::Text(s.into());
    let m_kg = p.mass_msun * M_SUN_KG;
    [
        ("body.mass", f(m_kg)),
        ("body.radius", f(2.0 * G_SI * m_kg / (C_M_S * C_M_S))),
        ("body.compact", t("black_hole")),
        ("star.luminosity", f(p.luminosity_w)),
        ("nucleus.class", t(p.class.name())),
        ("nucleus.eddington_ratio", f(p.eddington_ratio)),
        ("nucleus.mdot_msun_yr", f(p.mdot_msun_yr)),
        ("nucleus.disc_peak_k", f(p.disc_peak_k)),
        ("nucleus.seed_channel", t(&n.seed.channel)),
        ("nucleus.radio_loud", Value::Bool(n.radio_loud)),
        ("nucleus.tde_rate_per_yr", f(n.tde_rate_per_yr)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}
