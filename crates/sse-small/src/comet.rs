//! Comets whole (track SB4, part 1): when a resolved member's ices wake.
//!
//! Each ice of the nucleus sublimates freely from its share of the active
//! surface: sunlight absorbed = thermal emission + sublimation flux x latent
//! heat (Cowan and A'Hearn 1979), solved for the surface temperature. The
//! vapour curve and the latent heat (Clausius-Clapeyron on that curve) and
//! the molar mass are the sheet's, so water wakes near 3 AU, CO2 near 10
//! and CO far out without any of those distances being typed anywhere.
//! Coma size, dust tail and ion tail are drawn from the numbers (fudge
//! SB-comet-coma). Nothing here is stored.
//!
//! Only ice still at the surface can wake. A member that has sat where its
//! surface is warmer than the pack's `ice.survives_below_k` for the system's
//! age has lost its surface ice (the same rule its type label reads), so it
//! has no activity however much ice its bulk makeup holds. A GM-made comet
//! nucleus is a fresh arrival and keeps its ice.

use crate::settings::{CometSettings, Settings, VolatileSettings};
use crate::{Host, Member};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_formation::leftovers::Cloud;

/// One ice's part in the activity.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Volatile {
    /// The sheet's substance id.
    pub id: String,
    /// Its share of the nucleus's ice (by mass, from the cloud it formed in).
    pub share_of_ice: f64,
    /// Temperature of a surface of this ice in balance with the sunlight, K.
    pub temperature_k: f64,
    /// Mass sublimating per square metre of that ice, kg m^-2 s^-1.
    pub flux_kg_m2_s: f64,
    /// Molecules leaving the whole nucleus each second.
    pub production_per_s: f64,
    /// Share of the absorbed sunlight that goes into sublimation.
    pub sunlight_share: f64,
    /// Distance from the star inside which it takes over half the sunlight, AU.
    pub onset_au: f64,
}

/// What a nucleus is doing at one distance from its star.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Activity {
    pub r_au: f64,
    /// Any ice past its onset.
    pub active: bool,
    /// The ice making the most gas, when active.
    pub dominant: Option<String>,
    pub volatiles: Vec<Volatile>,
    pub gas_kg_s: f64,
    pub dust_kg_s: f64,
    /// The dominant gas's scale length (outflow speed x lifetime), km.
    pub coma_km: f64,
    /// How far radiation pressure has carried the tail's dust, km.
    pub dust_tail_km: f64,
    /// How far the solar wind has carried the tail's ions over the same
    /// tail age as the dust, km.
    pub ion_tail_km: f64,
}

/// The shares of water, CO2 and CO in a cloud's ice (formation's solid
/// groups 3, 4 and 5), in the order of the settings' volatiles.
pub fn ice_shares(cloud: &Cloud, s: &CometSettings) -> Vec<f64> {
    let kg: Vec<f64> = s
        .volatiles
        .iter()
        .map(|v| cloud.solid.get(v.group).copied().unwrap_or(0) as f64)
        .collect();
    let t: f64 = kg.iter().sum();
    kg.iter()
        .map(|x| if t > 0.0 { x / t } else { 0.0 })
        .collect()
}

fn absorbed_w_m2(albedo: f64, r_au: f64, host: &Host, s: &CometSettings, share: f64) -> f64 {
    let bond = (albedo * s.phase_integral).clamp(0.0, 1.0);
    (1.0 - bond) * s.s_sun_w_m2 * host.luminosity_lsun / (r_au * r_au) * share
}

/// Free-sublimation flux of an ice at `t` K, kg m^-2 s^-1 (Hertz-Knudsen,
/// sticking 1): P sqrt(M / (2 pi R T)).
fn flux(v: &VolatileSettings, t: f64, s: &CometSettings) -> f64 {
    let p = sse_reference::sublimation_pa(&v.phases, t).unwrap_or(0.0);
    p * m::sqrt(v.molar_kg_mol / (2.0 * std::f64::consts::PI * s.r_gas * t))
}

fn latent_j_kg(v: &VolatileSettings, t: f64) -> f64 {
    sse_reference::sublimation_latent_j_mol(&v.phases, t).unwrap_or(0.0) / v.molar_kg_mol
}

/// The balance for one ice: (temperature, flux, share of sunlight).
fn balance(v: &VolatileSettings, absorbed: f64, s: &CometSettings) -> (f64, f64, f64) {
    let out =
        |t: f64| s.emissivity * s.sigma_sb * t * t * t * t + flux(v, t, s) * latent_j_kg(v, t);
    let (mut lo, mut hi) = (2.0_f64, 600.0_f64);
    if out(lo) >= absorbed {
        return (lo, 0.0, 0.0);
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if out(mid) > absorbed {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let t = 0.5 * (lo + hi);
    let z = flux(v, t, s);
    let share = if absorbed > 0.0 {
        (z * latent_j_kg(v, t) / absorbed).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (t, z, share)
}

/// Where an ice's sublimation takes `onset_share` of the sunlight at the
/// subsolar point (`insolation` 1: activity starts where the Sun is
/// overhead), AU, searched in log distance between 0.05 and 2,000 AU.
pub fn onset_au(
    v: &VolatileSettings,
    albedo: f64,
    host: &Host,
    s: &CometSettings,
    insolation: f64,
) -> f64 {
    let share_at = |r: f64| balance(v, absorbed_w_m2(albedo, r, host, s, insolation), s).2;
    let (mut lo, mut hi) = (m::ln(0.05), m::ln(2000.0));
    if share_at(m::exp(hi)) >= s.onset_share {
        return m::exp(hi);
    }
    if share_at(m::exp(lo)) < s.onset_share {
        return 0.0;
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if share_at(m::exp(mid)) >= s.onset_share {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    m::exp(0.5 * (lo + hi))
}

/// Whether a member still has ice at its surface: it formed (or has sat
/// for the system's age) where its blackbody temperature, read at the
/// distance it spends its time at, is below `ice.survives_below_k`; or it
/// is a GM-made comet nucleus, a fresh arrival.
pub fn keeps_surface_ice(member: &Member, s: &Settings) -> bool {
    member.makeup.ice > 0.0
        && (member.kind == "made-comet" || member.temperature_k < s.ice.survives_below_k)
}

/// A member's activity at `r_au` from its star; `shares` from [`ice_shares`].
pub fn activity(
    member: &Member,
    shares: &[f64],
    r_au: f64,
    host: &Host,
    settings: &Settings,
) -> Activity {
    let s = &settings.comet;
    let r_au = r_au.max(1e-3);
    let radius_m = member.diameter_km * 500.0;
    let surface = 4.0 * std::f64::consts::PI * radius_m * radius_m;
    let icy = if keeps_surface_ice(member, settings) {
        member.makeup.ice
    } else {
        0.0
    };
    let absorbed = absorbed_w_m2(member.albedo, r_au, host, s, s.insolation_share);
    let onset_sun = s.onset_insolation_share;
    let mut vols: Vec<Volatile> = vec![];
    let mut kg = vec![];
    for (v, share) in s.volatiles.iter().zip(shares) {
        if icy <= 0.0 || *share <= 0.0 {
            continue;
        }
        let (t, z, sun) = balance(v, absorbed, s);
        kg.push(z * surface * s.active_fraction * share);
        vols.push(Volatile {
            id: v.id.clone(),
            share_of_ice: *share,
            temperature_k: t,
            flux_kg_m2_s: z,
            production_per_s: 0.0,
            sunlight_share: sun,
            onset_au: onset_au(v, member.albedo, host, s, onset_sun),
        });
    }
    // The first ice (water) is the matrix: the others are trapped in it and
    // leave with it once it sublimates, in proportion to their shares; on
    // their own they sublimate from under a dust mantle at `mantle_factor`
    // of the free rate (fudge SB-comet-mantle).
    let matrix = vols
        .first()
        .map(|v| (v.share_of_ice, kg[0], v.onset_au >= r_au));
    for (i, v) in vols.iter_mut().enumerate() {
        if i > 0 {
            let free = kg[i] * s.mantle_factor;
            let trapped = match matrix {
                Some((sh, w, true)) if sh > 0.0 => w * v.share_of_ice / sh,
                _ => 0.0,
            };
            kg[i] = free.max(trapped);
        }
        let mol = s
            .volatiles
            .iter()
            .find(|x| x.id == v.id)
            .map(|x| x.molar_kg_mol)
            .unwrap_or(1.0);
        v.production_per_s = kg[i] / mol * s.n_a;
    }
    let gas: f64 = kg.iter().sum();
    let active = vols.iter().any(|v| v.onset_au >= r_au);
    let lead = vols
        .iter()
        .filter(|v| v.onset_au >= r_au)
        .max_by(|a, b| a.production_per_s.total_cmp(&b.production_per_s));
    let (dominant, coma_km) = match lead.and_then(|l| {
        s.volatiles
            .iter()
            .find(|v| v.id == l.id)
            .map(|v| (l.id.clone(), v.lifetime_s_at_1au * r_au * r_au))
    }) {
        Some((id, life_s)) => {
            let speed = s.outflow_km_s_at_1au / m::sqrt(r_au);
            (Some(id), speed * life_s)
        }
        None => (None, 0.0),
    };
    // Both tails are what the star has carried off in the last tail age:
    // the dust by radiation pressure, the ions by the solar wind. The gas's
    // lifetime only says where in the coma its molecules become ions; it is
    // not how long an ion travels down the tail (see engine note in
    // handover/packages/SB4b.md).
    let tail_s = s.tail_age_days * s.day_s;
    let dust_kg_s = if active { gas * s.dust_to_gas } else { 0.0 };
    let dust_tail_km = if active {
        let g = s.gm_sun * host.mass_msun / (r_au * s.au_m * r_au * s.au_m);
        0.5 * s.dust_beta * g * tail_s * tail_s / 1000.0
    } else {
        0.0
    };
    let ion_tail_km = if dominant.is_some() {
        s.solar_wind_km_s * tail_s
    } else {
        0.0
    };
    Activity {
        r_au,
        active,
        dominant,
        volatiles: vols,
        gas_kg_s: gas,
        dust_kg_s,
        coma_km,
        dust_tail_km,
        ion_tail_km,
    }
}
