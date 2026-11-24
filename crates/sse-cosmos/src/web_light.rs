//! V9w: the web's light. Matter in the cosmic web only shines where it has
//! fallen into haloes deep enough to hold galaxies; voids and sheets are dark
//! gas and dark matter. This gives the galaxies inside a cube of the web
//! (`Web::volume`) from the same seed and the same fields:
//! - which haloes: G2's catalogue (`Haloes::cell_above`), cell by cell, from a
//!   mass floor up (`WebLightParams::floor_msun`), exactly the haloes the
//!   cluster map shows there;
//! - where: each halo sits on the peak of the starting field it collapsed
//!   from, so it is moved by the same truncated Zel'dovich step as the cube's
//!   parcels (Coles, Melott and Shandarin 1993) and lands on the filaments and
//!   knots that matter gathered into;
//! - how massive then: the halo's own growth history (McBride et al. 2009 form,
//!   G2), so a cube read early in time has fewer and smaller haloes;
//! - its galaxy: G3 on the median halo of its mass today, once per half dex
//!   (as the DM3 ledger does), read at the cube's time as shares of the halo's
//!   mass then: stars, star formation and whether it has quenched. Each
//!   halo's own full G3 history is what the galaxy chart draws; using the
//!   median for its mass is in the fudge register ("web light: median
//!   galaxies").

use crate::cosmology::Cosmology;
use crate::galaxy::{self, GalaxyParams};
use crate::haloes::Haloes;
use crate::web::Cache;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Pack settings (`provides.other.cosmology.web_light`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WebLightParams {
    /// Smallest halo whose galaxy is drawn, solar masses (at the cube's
    /// time). 10^11 holds a galaxy of about 10^8.7 solar masses of stars
    /// (Behroozi et al. 2019, MNRAS 488, 3143, z = 0 median), around the Small
    /// Magellanic Cloud's; the haloes above it hold almost all of the
    /// starlight (the stellar-to-halo ratio falls steeply below 10^11.5 and
    /// the mass function climbs only as about M^-0.9 in number per log mass).
    pub floor_msun: f64,
    /// Most galaxies in one cube (the biggest are kept).
    pub limit: u32,
}

impl Default for WebLightParams {
    fn default() -> Self {
        WebLightParams {
            floor_msun: 1.0e11,
            limit: 20_000,
        }
    }
}

/// One galaxy in the cube (data only; the renderer turns it into light).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebGalaxy {
    /// Where it is at the cube's time, comoving Mpc.
    pub position: [f64; 3],
    /// Its halo then, solar masses.
    pub halo_msun: f64,
    pub stars_msun: f64,
    /// Star formation, solar masses per year.
    pub sfr_msun_yr: f64,
    pub quenched: bool,
}

/// G3 on median haloes, half a dex apart, kept through time.
pub struct MedianGalaxies {
    lm0: f64,
    step: f64,
    /// Per row: (t Gyr, stars / halo, sfr / halo per year, quenched).
    rows: Vec<Vec<(f64, f64, f64, bool)>>,
}

impl MedianGalaxies {
    pub fn new(hs: &Haloes, gp: &GalaxyParams, lo: f64, hi: f64) -> Self {
        let step = 0.5;
        let lm0 = m::log10(lo);
        let n = m::ceil((m::log10(hi) - lm0) / step) as u32 + 1;
        let rows = (0..n)
            .map(|k| {
                let mh = m::pow(10.0, lm0 + step * f64::from(k));
                let h = hs.median(mh, &format!("web-light-{k}"));
                let g = galaxy::evolve(hs.cosmo, gp, &h);
                g.history
                    .iter()
                    .chain(std::iter::once(&g.now))
                    .map(|s| {
                        let mh = s.halo.max(1.0);
                        (s.t_gyr, s.stars / mh, s.sfr / mh, s.quenched)
                    })
                    .collect()
            })
            .collect();
        MedianGalaxies { lm0, step, rows }
    }

    /// One row at time t: shares linear in time between its states (none
    /// before the first).
    fn row_at(row: &[(f64, f64, f64, bool)], t: f64) -> (f64, f64, bool) {
        let Some(first) = row.first() else {
            return (0.0, 0.0, false);
        };
        if t < first.0 {
            return (0.0, 0.0, false);
        }
        let i = row.partition_point(|s| s.0 <= t);
        if i >= row.len() {
            let l = row[row.len() - 1];
            return (l.1, l.2, l.3);
        }
        let (a, b) = (row[i - 1], row[i]);
        let f = if b.0 > a.0 {
            ((t - a.0) / (b.0 - a.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (a.1 + (b.1 - a.1) * f, a.2 + (b.2 - a.2) * f, a.3)
    }

    /// Stars and star formation as shares of the halo's mass, and quenched,
    /// for a halo of `m0` today read at time `t` (linear in log mass between
    /// rows, clamped at the ends).
    pub fn at(&self, m0: f64, t: f64) -> (f64, f64, bool) {
        let x = ((m::log10(m0.max(1.0)) - self.lm0) / self.step).max(0.0);
        let i = (x as usize).min(self.rows.len() - 1);
        let j = (i + 1).min(self.rows.len() - 1);
        let f = (x - i as f64).clamp(0.0, 1.0);
        let a = Self::row_at(&self.rows[i], t);
        let b = Self::row_at(&self.rows[j], t);
        (
            a.0 + (b.0 - a.0) * f,
            a.1 + (b.1 - a.1) * f,
            if f < 0.5 { a.2 } else { b.2 },
        )
    }
}

/// The galaxies in a cube of the web at scale factor `a` (the same cube as
/// `Web::volume(cosmo, a, centre, extent, n, upto)`), biggest first: those
/// that started inside it, where the step has moved them.
pub fn cube_galaxies(
    hs: &Haloes,
    median: &MedianGalaxies,
    lp: &WebLightParams,
    a: f64,
    centre: [f64; 3],
    extent: f64,
    upto: u32,
) -> Vec<WebGalaxy> {
    let c: &Cosmology = hs.cosmo;
    let web = hs.web;
    let d = c.growth(a);
    let t = web.truncation(d).min(upto);
    let z = 1.0 / a - 1.0;
    let t_gyr = c.time_of_scale(a);
    let side = hs.params.cell_mpc;
    let half = 0.5 * extent;
    // As with the cube's parcels, the galaxies are those whose matter
    // started inside the cube: the field outside it was not read, so what
    // would have moved in is not there either (and the renderer leaves out
    // what moved well outside, as it does the parcels).
    let lo = |k: usize| m::floor((centre[k] - half) / side) as i64;
    let hi = |k: usize| m::floor((centre[k] + half) / side) as i64;
    let bins = hs.bin_table();
    let mut cache = Cache::default();
    let mut out = vec![];
    for iz in lo(2)..=hi(2) {
        for iy in lo(1)..=hi(1) {
            for ix in lo(0)..=hi(0) {
                for h in hs.cell_above([ix, iy, iz], &bins, lp.floor_msun) {
                    let mh = h.mass_at(z);
                    if mh < lp.floor_msun
                        || (0..3).any(|k| (h.position[k] - centre[k]).abs() > half)
                    {
                        continue;
                    }
                    let p = web.displaced(&mut cache, h.position, t, d);
                    let (stars, sfr, quenched) = median.at(h.mass, t_gyr);
                    out.push(WebGalaxy {
                        position: p,
                        halo_msun: mh,
                        stars_msun: stars * mh,
                        sfr_msun_yr: sfr * mh,
                        quenched,
                    });
                }
            }
        }
    }
    out.sort_by(|x, y| {
        y.halo_msun.total_cmp(&x.halo_msun).then_with(|| {
            x.position
                .iter()
                .zip(&y.position)
                .map(|(a, b)| a.total_cmp(b))
                .find(|o| o.is_ne())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    out.truncate(lp.limit as usize);
    out
}
