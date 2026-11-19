//! Cosmic epochs on the time ruler (galaxy design section 2, stage G1).
//! Each epoch is derived from the cosmology, never a hard-coded date; the
//! far-future eras follow Adams and Laughlin 1997 and are labelled speculative.

use crate::cosmology::{erfc, Cosmology, DELTA_C};
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::Address;
use std::collections::BTreeMap;

/// Pack settings (`provides.other.cosmology.epochs`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EpochParams {
    /// Mass of the minihaloes that host the first stars, solar masses.
    pub first_star_halo: f64,
    /// Peak height of the first such haloes to collapse.
    pub first_star_nu: f64,
    /// Halo mass at a virial temperature of 1e4 K at z = 9 (atomic cooling), solar masses;
    /// scales as (1+z)^-1.5 (Barkana and Loeb 2001 eq. 26).
    pub cooling_mass_z9: f64,
    /// Ionising photons escaping per collapsed baryon, as an efficiency zeta (Q = zeta f_coll).
    pub reionisation_zeta: f64,
    /// Star formation ends, log10 years (Adams and Laughlin 1997: about 1e14).
    pub last_stars_log10_yr: f64,
    /// Protons decay (a pack switch); lifetime, log10 years.
    pub proton_decay: bool,
    pub proton_lifetime_log10_yr: f64,
    /// Largest black hole, solar masses: sets the end of the black hole era.
    pub largest_black_hole: f64,
}

impl Default for EpochParams {
    fn default() -> Self {
        EpochParams {
            first_star_halo: 1.0e6,
            first_star_nu: 3.0,
            cooling_mass_z9: 1.0e8,
            reionisation_zeta: 11.0,
            last_stars_log10_yr: 14.0,
            proton_decay: true,
            proton_lifetime_log10_yr: 37.0,
            largest_black_hole: 1.0e11,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Epoch {
    pub id: String,
    pub name: String,
    /// Start, log10 years after the origin.
    pub log10_yr: f64,
    /// Redshift, where it has one.
    pub z: Option<f64>,
    pub speculative: bool,
    pub why: String,
}

impl Epoch {
    /// On the chronicle's clock (universe time ends at 2^63 s, about 292 Gyr).
    pub fn time(&self) -> Option<UniverseTime> {
        let s = m::pow(10.0, self.log10_yr) * crate::cosmology::YEAR_S;
        (s < 9.0e18).then(|| UniverseTime::seconds(m::round(s) as i64))
    }
}

/// Recombination redshift z*, Hu and Sugiyama 1996 eq. E-1.
pub fn recombination_z(c: &Cosmology) -> f64 {
    let wb = c.p.omega_b * c.h * c.h;
    let wm = c.p.omega_m * c.h * c.h;
    let g1 = 0.0783 * m::pow(wb, -0.238) / (1.0 + 39.5 * m::pow(wb, 0.763));
    let g2 = 0.560 / (1.0 + 21.1 * m::pow(wb, 1.81));
    1048.0 * (1.0 + 0.001_24 * m::pow(wb, -0.738)) * (1.0 + g1 * m::pow(wm, g2))
}

/// Ionised fraction of the intergalactic medium at redshift z: zeta times
/// the fraction of matter collapsed into haloes that can cool by atomic
/// hydrogen (Press-Schechter: erfc(delta_c / (sqrt 2 D sigma(M_min)))), capped at 1.
pub fn ionised_fraction(c: &Cosmology, ep: &EpochParams, z: f64) -> f64 {
    let mmin = ep.cooling_mass_z9 * m::pow((1.0 + z) / 10.0, -1.5);
    let d = c.growth(1.0 / (1.0 + z));
    let fcoll = erfc(DELTA_C / (m::sqrt(2.0) * d * c.sigma_m(mmin)));
    (ep.reionisation_zeta * fcoll).min(1.0)
}

/// Redshift at which the ionised fraction reaches q (fixed 80-step bisection over z 0 to 40).
pub fn reionisation_z(c: &Cosmology, ep: &EpochParams, q: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 40.0_f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if ionised_fraction(c, ep, mid) >= q {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Redshift of peak cosmic star formation in the Madau and Dickinson 2014 fit
/// psi(z) proportional to (1+z)^2.7 / (1 + ((1+z)/2.9)^5.6) (eq. 15), by 100 fixed ternary steps.
pub fn star_formation_peak_z() -> f64 {
    let psi = |z: f64| m::pow(1.0 + z, 2.7) / (1.0 + m::pow((1.0 + z) / 2.9, 5.6));
    let (mut lo, mut hi) = (0.0_f64, 8.0_f64);
    for _ in 0..100 {
        let a = lo + (hi - lo) / 3.0;
        let b = hi - (hi - lo) / 3.0;
        if psi(a) < psi(b) {
            lo = a;
        } else {
            hi = b;
        }
    }
    0.5 * (lo + hi)
}

/// Every epoch, in order.
pub fn epochs(c: &Cosmology, ep: &EpochParams) -> Vec<Epoch> {
    let yr = |z: f64| m::log10(c.time_of_scale(1.0 / (1.0 + z)) * 1.0e9);
    let e =
        |id: &str, name: &str, log10_yr: f64, z: Option<f64>, speculative: bool, why: String| {
            Epoch {
                id: id.into(),
                name: name.into(),
                log10_yr,
                z,
                speculative,
                why,
            }
        };
    let zr = recombination_z(c);
    let zf = c.collapse_redshift(ep.first_star_halo, ep.first_star_nu);
    let zh = reionisation_z(c, ep, 0.5);
    let ze = reionisation_z(c, ep, 0.999);
    let zp = star_formation_peak_z();
    let now = c.p.now_gyr;
    let a_now = c.scale_of_time(now);
    let tau = ep.proton_lifetime_log10_yr;
    // Hawking evaporation, Page 1976 (Phys. Rev. D 13, 198): a hole well above 1e17 g radiates
    // P = 2e-4 hbar c^6 / (G^2 M^2) in massless species, 81% neutrinos, 17% photons, 2% gravitons.
    // Neutrinos have mass, far above the Hawking temperature of any stellar or larger hole, so only
    // photons and gravitons (19%) are emitted: t = M^3 c^2 / (3 x 0.19 x 2e-4 hbar c^6 / G^2)
    // = 1.14e67 (M / Msun)^3 years. (All massless species would give 2.2e66; photons only, no
    // greybody, 2.1e67, the value used before 19 Nov 2026.)
    let bh_end = m::log10(1.14e67) + 3.0 * m::log10(ep.largest_black_hole);
    let mut v = vec![
        e("recombination", "Recombination", yr(zr), Some(zr), false, format!(
            "the universe cools below about 3000 K at z = {zr:.0}: electrons and protons combine into hydrogen and light runs free, the background we now see as the CMB (Hu and Sugiyama 1996 fit to Omega_b h^2 and Omega_m h^2)")),
        e("dark_ages", "Dark ages", yr(zr) + 1.0e-9, Some(zr), false,
            "neutral gas and no stars yet; overdensities grow in proportion to the growth factor D(a)".into()),
        e("first_stars", "First stars", yr(zf), Some(zf), false, format!(
            "the first {:.0e} solar-mass minihaloes from {:.0}-sigma peaks collapse at z = {zf:.1} (D sigma(M) nu = 1.686): Population III stars light up", ep.first_star_halo, ep.first_star_nu)),
        e("reionisation", "Reionisation (half way)", yr(zh), Some(zh), false, format!(
            "half the intergalactic hydrogen is ionised at z = {zh:.1}: zeta = {} times the mass in haloes able to cool by atomic hydrogen (Planck 2018 measure z = 7.8 +- 0.7 with BAO, 7.7 +- 0.7 without)", ep.reionisation_zeta)),
        e("reionisation_end", "Reionisation complete", yr(ze), Some(ze), false, format!(
            "the last neutral pockets are gone by z = {ze:.1}; haloes below the filtering mass can no longer gather gas, leaving fossil dwarfs")),
        e("peak_star_formation", "Cosmic noon", yr(zp), Some(zp), false, format!(
            "the universe forms stars fastest at z = {zp:.2} (Madau and Dickinson 2014); since then the rate has fallen about tenfold")),
        e("now", "Now", m::log10(now * 1.0e9), Some(1.0 / a_now - 1.0), false, format!(
            "the pack's present, {now} Gyr after the origin; dark energy now dominates and expansion accelerates")),
        e("last_stars", "Last stars form", ep.last_stars_log10_yr, None, true,
            "gas is used up and star formation ends; the longest-lived red dwarfs shine on (Adams and Laughlin 1997)".into()),
        e("degenerate_era", "Degenerate era", ep.last_stars_log10_yr + 1.0e-9, None, true,
            "white dwarfs, neutron stars, brown dwarfs and black holes hold almost all the baryons; occasional collisions make new stars (Adams and Laughlin 1997)".into()),
    ];
    if ep.proton_decay {
        v.push(e("black_hole_era", "Black hole era", tau + 2.0, None, true, format!(
            "protons decay (lifetime 1e{tau:.0} years in this pack) and degenerate matter dissolves; only black holes remain (Adams and Laughlin 1997)")));
    } else {
        v.push(e("black_hole_era", "Black hole era", (bh_end - 30.0).max(ep.last_stars_log10_yr + 1.0), None, true,
            "with stable protons degenerate matter lasts; black holes come to dominate the energy budget (Adams and Laughlin 1997)".into()));
    }
    v.push(e("dark_era", "Dark era", bh_end, None, true, format!(
        "the largest black holes ({:.0e} solar masses) evaporate by Hawking radiation; photons, leptons and dark energy remain", ep.largest_black_hole)));
    v.sort_by(|a, b| a.log10_yr.total_cmp(&b.log10_yr));
    v
}

/// Record the epochs that fall on the chronicle's clock as derived `cosmo.epoch` events.
pub fn record(
    ch: &mut Chronicle,
    universe: &Address,
    eps: &[Epoch],
) -> Result<Vec<sse_core::ContentHash>, String> {
    let mut ids = vec![];
    for ep in eps {
        let Some(at) = ep.time() else { continue };
        let mut data = BTreeMap::new();
        data.insert("id".into(), Value::Text(ep.id.clone()));
        data.insert("name".into(), Value::Text(ep.name.clone()));
        data.insert("why".into(), Value::Text(ep.why.clone()));
        data.insert("speculative".into(), Value::Bool(ep.speculative));
        if let Some(z) = ep.z {
            data.insert("z".into(), Value::Float(z));
        }
        let e = Event::new(EventBody {
            kind: "cosmo.epoch".into(),
            subject: vec![universe.clone()],
            occurred_at: at,
            activated_at: at,
            recognised_at: ch.next_stamp(),
            causes: vec![],
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Derived,
            publisher: Publisher::new("sse.base-cosmos", "cosmology", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        ids.push(ch.append(e).map_err(|e| format!("{e:?}"))?);
    }
    Ok(ids)
}
