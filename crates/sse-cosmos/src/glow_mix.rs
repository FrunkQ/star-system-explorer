//! The light of a place's stars as a mix of points (night sky Y3). The glow
//! map needs the light of the stars the sky draw does not show, so it needs
//! every star's luminosity and temperature in expectation, not one draw.
//! This is the same quadrature as the count tree's class shares (Y1): the
//! IMF on three grids (the bulk, the brightest per cent and per mille) by
//! 48 log-age bins of the birth history at the place's radius. Each point
//! carries its share of the systems there, its class, luminosity and
//! temperature. Companions are not in it (fudge F36).

use crate::stars::{imf_quantile, Stars};
use sse_core::math as m;

#[derive(Clone, Debug, PartialEq)]
pub struct LightPoint {
    /// Share of the place's systems this point stands for (all points sum to 1).
    pub weight: f64,
    pub class: usize,
    pub luminosity_lsun: f64,
    pub teff_k: f64,
}

impl Stars<'_> {
    /// The quadrature points of the stars at galactocentric radius `r_kpc`
    /// at time `t_myr`. Points with no light (unborn, remnants too faint to
    /// matter, black holes) keep their weight but have zero luminosity.
    pub fn light_points(&self, r_kpc: f64, t_myr: f64) -> Vec<LightPoint> {
        let (cdf, acc) = self.birth_cdf_at(r_kpc, t_myr / 1000.0);
        self.light_points_of(&cdf, acc, t_myr)
    }

    /// The quadrature points of the bulge's stars at `t_myr` (its births
    /// are one history everywhere).
    pub fn bulge_light_points(&self, t_myr: f64) -> Vec<LightPoint> {
        let (cdf, acc) = self.bulge_cdf(t_myr / 1000.0);
        self.light_points_of(&cdf, acc, t_myr)
    }

    /// The quadrature points of stars with these births (cumulative, Gyr).
    fn light_points_of(&self, cdf: &[(f64, f64)], acc: f64, t_myr: f64) -> Vec<LightPoint> {
        let p = &self.params;
        let t_now = t_myr / 1000.0;
        let born_by = |t: f64| -> f64 {
            if acc <= 0.0 {
                return 0.0;
            }
            let v = match cdf.windows(2).find(|w| w[1].0 >= t) {
                None => acc,
                Some(w) if t <= w[0].0 => w[0].1,
                Some(w) => {
                    let f = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-300);
                    w[0].1 + f * (w[1].1 - w[0].1)
                }
            };
            (v / acc).clamp(0.0, 1.0)
        };
        let oldest = (t_now - cdf.first().map_or(self.env.disc.t_start_gyr, |c| c.0)).max(1e-3);
        let na = 48;
        let edges: Vec<f64> = (0..=na)
            .map(|k| {
                if k == 0 {
                    0.0
                } else {
                    1.0e-4 * m::pow(oldest / 1.0e-4, (k - 1) as f64 / (na - 1) as f64)
                }
            })
            .collect();
        let bins: Vec<(f64, f64)> = edges
            .windows(2)
            .map(|e| {
                let mid = if e[0] > 0.0 {
                    m::sqrt(e[0] * e[1])
                } else {
                    0.5 * e[1]
                };
                (mid, born_by(t_now - e[0]) - born_by(t_now - e[1]))
            })
            .filter(|b| b.1 > 0.0)
            .collect();
        let mut out = vec![];
        for (lo, hi, nm) in [(0.0, 0.99, 256), (0.99, 0.999, 128), (0.999, 1.0, 128)] {
            let du = (hi - lo) / nm as f64;
            for i in 0..nm {
                let u = lo + (i as f64 + 0.5) * du;
                let mass = imf_quantile(p, u);
                for (age, share) in &bins {
                    let s = p.state(mass, *age);
                    out.push(LightPoint {
                        weight: du * share,
                        class: p.class_of(s.luminosity_lsun),
                        luminosity_lsun: s.luminosity_lsun,
                        teff_k: s.teff_k,
                    });
                }
            }
        }
        let total: f64 = out.iter().map(|q| q.weight).sum();
        if total > 0.0 {
            for q in out.iter_mut() {
                q.weight /= total;
            }
        }
        out
    }
}
