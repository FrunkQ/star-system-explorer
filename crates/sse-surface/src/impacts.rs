//! S4: impacts and volcanism from the chronicle (surface design section 2.3 and
//! section 3 terms 3 and 5; research report section 3.2 stage 4).
//!
//! Craters are drawn from the world's impact history: a declining early
//! bombardment (a lunar-type chronology, scaled to the world by gravitational
//! focusing and crater-size scaling) plus the chronicle's bombardment spikes
//! (F4 stirred belts). Every crater has a date, size, place and shape (bowl,
//! central peak, peak ring, multi-ring basin from size and gravity). What
//! survives to the epoch depends on how the surface was renewed: the regime's
//! resurfacing rate, young plate crust, later lava plains, and degradation by
//! gardening, water and wind. Craters that wore away to nothing are kept as
//! ghosts (the structure is on record, the relief is gone).
//!
//! The listed population (craters at least `listed_d_frac` of the radius
//! across) is global. Smaller craters are a pure function of a cube-sphere
//! cell per size band, so any tile resolves them alone down to the fact floor.
//!
//! Volcanism on lid worlds: lava plains erupt while heat flow is high, and
//! burying the craters under them; long-lived shields grow as tall as gravity
//! allows. Every feature carries a cause sentence with its date and sources:
//! "why is this here".
//!
//! Each stage's rule is a named slot (`mechanisms`), so a pack can swap an
//! alien mechanism in once an atom exists for it; unknown names are refused.

use crate::plates::PlateHistory;
use crate::sketch::dot;
use crate::wear::WaterHistory;
use crate::{SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use std::collections::BTreeMap;

pub const IMPACTS_VERSION: &str = "surface-impacts-v1";

const R_EARTH_KM: f64 = 6371.0;
const M_EARTH_KG: f64 = 5.972e24;

pub const CHRONOLOGIES: [&str; 1] = ["neukum-2001"];
pub const CRATER_SHAPES: [&str; 1] = ["pike-1977"];
pub const VOLCANISMS: [&str; 2] = ["heat-flow-plains", "none"];

/// Which rule fills each slot. A pack names the rule; the kernel refuses names
/// it has no atom for.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Mechanisms {
    pub chronology: String,
    pub crater_shape: String,
    pub volcanism: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VolcanismSettings {
    /// Regimes whose lid lets melt through as plains and shields.
    pub regimes: Vec<String>,
    pub q_min_w_m2: f64,
    pub q_ref_w_m2: f64,
    /// Area fraction resurfaced per Gyr at the reference heat flow.
    pub plains_per_gyr: f64,
    /// Most of the surface plains may cover in all.
    pub plains_cap: f64,
    /// Unit radius range, fraction of the world's radius (log-uniform).
    pub plains_radius_frac: [f64; 2],
    /// Craters smaller than this are buried by a plains unit; larger ones
    /// keep a flooded rim.
    pub bury_d_km: f64,
    pub shields_per_gyr: f64,
    /// Regimes that build long-lived shields over fixed sources, and the
    /// smallest world that does (smaller ones vent as plains only).
    pub shield_regimes: Vec<String>,
    pub shield_min_radius_km: f64,
    pub shields_max: u32,
    /// Tallest shield at Earth gravity, metres (scales as 1/g).
    pub shield_height_m: f64,
    /// Shield radius over height.
    pub shield_aspect: f64,
    pub shield_growth_myr: f64,
    pub step_myr: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Renewal {
    pub myr: f64,
    pub q_ref_w_m2: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ImpactSettings {
    pub mechanisms: Mechanisms,
    /// Chronology N(>1 km) per km² = a (e^(b T) − 1) + c T, T Gyr before the
    /// reference age (Neukum et al. 2001 lunar form). Constrained by lunar samples
    /// only back to about 4.1 Gyr; earlier is extrapolation.
    pub chron_a: f64,
    pub chron_b: f64,
    pub chron_c: f64,
    pub chron_ref_gyr: f64,
    /// Cumulative size-frequency slopes below and above the break.
    pub sfd_slope_small: f64,
    pub sfd_slope_large: f64,
    pub sfd_break_km: f64,
    /// Empirical saturation: N(>D) = saturation_n1 × D^-2 per km² (D in km).
    pub saturation_n1: f64,
    /// Crater size scales as g^-mu (gravity regime pi-scaling) against g_ref.
    pub gravity_exponent: f64,
    pub g_ref_m_s2: f64,
    /// Impactor speed at infinity for gravitational focusing, km/s.
    pub v_inf_km_s: f64,
    /// Crater over impactor diameter at g_ref, impactor density and the
    /// largest impactor, for turning spike mass into craters.
    pub crater_impactor_ratio: f64,
    pub impactor_density: f64,
    pub impactor_max_km: f64,
    /// Listed craters are at least this fraction of the radius across.
    pub listed_d_frac: f64,
    pub listed_max: u32,
    /// Largest crater, fraction of the radius.
    pub max_d_frac: f64,
    /// Morphology transitions at g_ref, km (scale as 1/g).
    pub simple_complex_km: f64,
    pub peak_ring_km: f64,
    pub multi_ring_km: f64,
    /// Smallest crater a thick atmosphere lets through: atm_cut_km × P^atm_cut_exp (bar).
    pub atm_cut_km: f64,
    pub atm_cut_exp: f64,
    /// Degradation: relief left = exp(−K t / (degrade_coeff D²)), K in m²/Myr.
    pub kappa_dry_m2_myr: f64,
    pub kappa_wet_m2_myr: f64,
    pub kappa_air_m2_myr_bar: f64,
    pub degrade_coeff: f64,
    pub ghost_fraction: f64,
    /// Global renewal by regime (absent: none): mean renewal time at a
    /// reference heat flow; the rate scales as (q / q_ref)^resurface_exponent.
    pub resurface: BTreeMap<String, Renewal>,
    pub resurface_exponent: f64,
    /// Regimes with no crater-holding surface.
    pub no_surface: Vec<String>,
    pub volcanism: VolcanismSettings,
}

/// A bombardment spike from the chronicle (F4 `Bombardment`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Spike {
    /// Myr since the world formed.
    pub t_myr: f64,
    /// Earth masses delivered.
    pub earth: f64,
    pub efold_myr: f64,
    /// The chronicle event it came from, in words.
    pub event: String,
}

/// What hit the world and when.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ImpactHistory {
    /// Background flux relative to the Moon's, before focusing.
    pub flux_scale: f64,
    pub spikes: Vec<Spike>,
    pub source: String,
}

impl ImpactHistory {
    /// The Moon's background flux and no spikes (fixtures, imports).
    pub fn lunar() -> Self {
        ImpactHistory {
            flux_scale: 1.0,
            spikes: vec![],
            source: "lunar-type chronology, no chronicle spikes".into(),
        }
    }
}

/// One impact history per world of a formation run: the background flux plus
/// each F4 bombardment spike on that world.
pub fn impact_histories(run: &sse_formation::pipeline::SystemRun) -> Vec<ImpactHistory> {
    use sse_formation::violent::UpheavalEvent;
    let ids: Vec<String> = match &run.leftovers {
        Some(l) => l.worlds.iter().map(|w| w.id.clone()).collect(),
        None => vec![],
    };
    run.protoplanets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut h = ImpactHistory::lunar();
            h.source = "lunar-type chronology".into();
            if let (Some(u), Some(id)) = (&run.upheaval, ids.get(i)) {
                for (k, e) in u.events.iter().enumerate() {
                    if let UpheavalEvent::Bombardment {
                        t_myr,
                        world,
                        earth,
                        efold_myr,
                        ..
                    } = e
                    {
                        if world == id && *t_myr >= p.formed_myr {
                            h.spikes.push(Spike {
                                t_myr: t_myr - p.formed_myr,
                                earth: *earth,
                                efold_myr: *efold_myr,
                                event: format!("upheaval event {k} (a stirred belt)"),
                            });
                        }
                    }
                }
                if !h.spikes.is_empty() {
                    h.source = format!(
                        "lunar-type chronology plus {} chronicle bombardment spike(s)",
                        h.spikes.len()
                    );
                }
            }
            h
        })
        .collect()
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Crater {
    pub id: String,
    pub dir: [f64; 3],
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub d_km: f64,
    /// bowl, central-peak, peak-ring or multi-ring.
    pub shape: String,
    /// Today's relief, metres (fresh values times `fresh`).
    pub depth_m: f64,
    pub rim_m: f64,
    pub peak_m: f64,
    /// Myr since the world formed.
    pub formed_myr: f64,
    /// Fraction of fresh relief left.
    pub fresh: f64,
    pub ghost: bool,
    /// Floor flooded by later lava.
    pub flooded: bool,
    pub cause: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plains {
    pub id: String,
    pub dir: [f64; 3],
    pub radius_km: f64,
    pub formed_myr: f64,
    pub cause: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Shield {
    pub id: String,
    pub dir: [f64; 3],
    pub height_m: f64,
    pub radius_km: f64,
    pub from_myr: f64,
    pub to_myr: f64,
    pub cause: String,
}

/// A world's impact and volcanic features at an epoch.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Features {
    pub address: String,
    pub epoch_myr: f64,
    /// When the current crater-holding surface began, Myr since formation
    /// (None: no solid surface to hold craters).
    pub surface_from_myr: Option<f64>,
    pub listed_d_km: f64,
    /// Expected listed craters kept by saturation alone, before renewal.
    pub expected_listed: f64,
    pub craters: Vec<Crater>,
    pub plains: Vec<Plains>,
    pub shields: Vec<Shield>,
    /// Listed craters lost, by reason.
    pub erased: BTreeMap<String, u32>,
    pub history_source: String,
    pub refs: Vec<String>,
}

pub const REFS: [&str; 7] = [
    "Neukum, G., Ivanov, B. A. and Hartmann, W. K. 2001, Space Science Reviews 96, 55: cratering chronology N(1) = 5.44e-14 (e^(6.93 T) − 1) + 8.38e-4 T. doi:10.1023/A:1011989004263",
    "Pike, R. J. 1977, in Impact and Explosion Cratering, 489: depth and rim height against diameter, simple and complex lunar craters. https://ui.adsabs.harvard.edu/abs/1977iecp.symp..489P",
    "Holsapple, K. A. 1993, Annual Review of Earth and Planetary Sciences 21, 333: gravity-regime crater scaling, D proportional to g^-0.22 for rock. doi:10.1146/annurev.ea.21.050193.002001",
    "Melosh, H. J. 1989, Impact Cratering: A Geologic Process (Oxford): simple-to-complex transition scaling as 1/g; peak-ring and multi-ring basins. https://global.oup.com/academic/product/impact-cratering-9780195042849",
    "Gault, D. E. 1970, Radio Science 5, 273, and Hartmann, W. K. 1984, Icarus 60, 56: empirical crater saturation at 1 to 10 percent of geometric saturation (1.54 D^-2); 0.15 D^-2 is taken, the top of that range. doi:10.1016/0019-1035(84)90138-6",
    "Fassett, C. I. and Thomson, B. J. 2014, JGR Planets 119, 2255: lunar crater degradation as topographic diffusion, about 5.5 m² per Myr. doi:10.1002/2014JE004698",
    "Zahnle, K. et al. 2003, Icarus 163, 263: cratering rates in the outer Solar System; Europa's surface about 60 Myr old (Schenk et al. 2004, in Jupiter, ch. 18). doi:10.1016/S0019-1035(03)00048-4",
];

/// Everything a crater draw needs, shared by the listed population and the
/// per-cell small bands.
#[derive(Clone, Debug)]
pub struct CraterField {
    s: ImpactSettings,
    d: Draws,
    radius_km: f64,
    g: f64,
    gscale: f64,
    flux: f64,
    /// (t_myr, N(>1 km) per km² on this world, efold, event).
    spikes: Vec<(f64, f64, f64, String)>,
    t_from: f64,
    epoch: f64,
    atm_cut_km: f64,
    /// (from, to, regime, heat flow start, end): the timeline in Myr.
    spans: Vec<(f64, f64, String, f64, f64)>,
    wet: (f64, f64),
    pressure_bar: f64,
    /// Plate crust birth per cell, and the grid, on plate worlds.
    crust: Option<(crate::plates::Grid, Vec<f64>)>,
    pub plains: Vec<Plains>,
}

fn unit_dir(d: &Draws, i: u64) -> [f64; 3] {
    let v = [d.normal(i), d.normal(i + 1), d.normal(i + 2)];
    let l = m::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).max(1e-300);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn lat_lon(v: [f64; 3]) -> (f64, f64) {
    (
        m::asin(v[2].clamp(-1.0, 1.0)).to_degrees(),
        m::atan2(v[1], v[0]).to_degrees(),
    )
}

/// Keyed Poisson draw.
fn poisson(d: &Draws, i: u64, lam: f64) -> u64 {
    if lam <= 0.0 {
        return 0;
    }
    if lam < 30.0 {
        let u = d.unit(i);
        let mut p = m::exp(-lam);
        let mut c = p;
        let mut k = 0u64;
        while u > c && k < 400 {
            k += 1;
            p *= lam / k as f64;
            c += p;
        }
        k
    } else {
        m::round(lam + m::sqrt(lam) * d.normal(i)).max(0.0) as u64
    }
}

/// Fresh crater geometry (Pike 1977 forms, transitions scaled as 1/g):
/// (shape, depth m, rim m, peak m).
pub fn crater_shape(s: &ImpactSettings, g: f64, d_km: f64) -> (&'static str, f64, f64, f64) {
    let gs = s.g_ref_m_s2 / g.max(1e-6);
    let (sc, pr, mr) = (
        s.simple_complex_km * gs,
        s.peak_ring_km * gs,
        s.multi_ring_km * gs,
    );
    if d_km < sc {
        return ("bowl", 196.0 * d_km, 36.0 * d_km, 0.0);
    }
    // Pike's lunar complex fits (d = 1.044 D^0.301, h = 0.236 D^0.399, km),
    // self-similar in D g / g_ref.
    let depth = 1044.0 * gs * m::pow(d_km / gs, 0.301);
    let rim = 236.0 * gs * m::pow(d_km / gs, 0.399);
    let shape = if d_km >= mr {
        "multi-ring"
    } else if d_km >= pr {
        "peak-ring"
    } else {
        "central-peak"
    };
    (shape, depth, rim, 0.4 * depth)
}

/// Height of a crater at a unit direction, metres (smooth: C1 with bounded
/// curvature, so band-limiting holds).
pub fn crater_height(c: &Crater, v: [f64; 3], radius_km: f64) -> f64 {
    crater_height_within(c, v, radius_km, -1.0)
}

/// The cosine of the angle beyond which a crater adds nothing (2.2 radii).
pub fn crater_reach_cos(c: &Crater, radius_km: f64) -> f64 {
    m::cos((1.1 * c.d_km / radius_km).min(m::PI))
}

/// As [`crater_height`], skipping the work outside `reach_cos`.
pub fn crater_height_within(c: &Crater, v: [f64; 3], radius_km: f64, reach_cos: f64) -> f64 {
    let cd = dot(c.dir, v);
    if cd < reach_cos {
        return 0.0;
    }
    let ang = m::acos(cd.clamp(-1.0, 1.0));
    let r = ang * radius_km / (0.5 * c.d_km);
    if r >= 2.2 {
        return 0.0;
    }
    let mut h = if r < 1.0 {
        let q = 1.0 - r * r;
        c.rim_m - (c.depth_m + c.rim_m) * q * q
    } else {
        c.rim_m * m::exp(-3.0 * (r - 1.0) * (r - 1.0))
    };
    match c.shape.as_str() {
        "central-peak" => h += c.peak_m * m::exp(-(r / 0.2) * (r / 0.2)),
        "peak-ring" | "multi-ring" => {
            h += c.peak_m * m::exp(-((r - 0.5) / 0.1) * ((r - 0.5) / 0.1))
        }
        _ => {}
    }
    h
}

/// Worst error of a 2 × 2 child average against a parent sample for a crater
/// of this size, metres, at parent spacing `sp_m` (f'' s² / 16, with |f''| up
/// to 8 (depth + rim) + 50 peak (central) or 200 peak (ring) over R²).
pub fn crater_curvature_err(s: &ImpactSettings, g: f64, d_km: f64, sp_m: f64) -> f64 {
    let (shape, depth, rim, peak) = crater_shape(s, g, d_km);
    let k = 8.0 * (depth + rim)
        + match shape {
            "central-peak" => 50.0 * peak,
            "peak-ring" | "multi-ring" => 200.0 * peak,
            _ => 0.0,
        };
    let rr = 0.5 * d_km * 1000.0;
    k * (sp_m / rr) * (sp_m / rr) / 16.0
}

pub fn shield_height(sh: &Shield, v: [f64; 3], radius_km: f64) -> f64 {
    let ang = m::acos(dot(sh.dir, v).clamp(-1.0, 1.0));
    let r = ang * radius_km / sh.radius_km.max(1e-9);
    if r >= 1.0 {
        return 0.0;
    }
    let q = 1.0 - r * r;
    // A broad shield with a summit caldera a tenth of its width.
    sh.height_m * q * q - 0.03 * sh.height_m * m::exp(-(r / 0.05) * (r / 0.05))
}

impl CraterField {
    fn regime_at(&self, t: f64) -> &str {
        self.spans
            .iter()
            .find(|x| t >= x.0 && t <= x.1)
            .or(self.spans.last())
            .map(|x| x.2.as_str())
            .unwrap_or("")
    }

    fn heat_at(&self, t: f64) -> f64 {
        match self.spans.iter().find(|x| t >= x.0 && t <= x.1) {
            Some(x) if x.1 > x.0 => x.3 + (x.4 - x.3) * (t - x.0) / (x.1 - x.0),
            Some(x) => x.3,
            None => 0.0,
        }
    }

    /// The Moon's N(>1 km) per km², cumulative from formation to `t` Myr.
    fn n1_moon(&self, t: f64) -> f64 {
        let s = &self.s;
        let f = |big_t: f64| s.chron_a * (m::exp(s.chron_b * big_t) - 1.0) + s.chron_c * big_t;
        let tg = t / 1000.0;
        let before = (s.chron_ref_gyr - tg).max(0.0);
        f(s.chron_ref_gyr) - f(before) + s.chron_c * (tg - s.chron_ref_gyr).max(0.0)
    }

    /// N(>D) / N(>1 km) for the production size-frequency distribution.
    fn shape(&self, d_km: f64) -> f64 {
        let s = &self.s;
        let b = s.sfd_break_km;
        let at_break = m::pow(b, -s.sfd_slope_small);
        if d_km < b {
            m::pow(d_km, -s.sfd_slope_small)
        } else {
            at_break * m::pow(d_km / b, -s.sfd_slope_large)
        }
    }

    /// Spike part of N(>1 km) between t0 and t1, one spike.
    fn spike_n1(sp: &(f64, f64, f64, String), t0: f64, t1: f64) -> f64 {
        let cum = |t: f64| {
            if t <= sp.0 {
                0.0
            } else {
                sp.1 * (1.0 - m::exp(-(t - sp.0) / sp.2.max(1e-6)))
            }
        };
        cum(t1) - cum(t0)
    }

    /// Craters at least `d_km` across formed between t0 and t1, per km².
    fn n_prod(&self, d_km: f64, t0: f64, t1: f64) -> f64 {
        let bg = self.flux * (self.n1_moon(t1) - self.n1_moon(t0)) * self.shape(d_km * self.gscale);
        let sp: f64 = self.spikes.iter().map(|x| Self::spike_n1(x, t0, t1)).sum();
        bg + sp * self.shape(d_km)
    }

    fn n_sat(&self, d_km: f64) -> f64 {
        self.s.saturation_n1 / (d_km * d_km)
    }

    /// Kept by saturation at size d: N_sat (1 − e^(−N/N_sat)).
    fn n_kept(&self, d_km: f64) -> f64 {
        let ns = self.n_sat(d_km);
        let x = self.n_prod(d_km, self.t_from, self.epoch) / ns;
        ns * (1.0 - m::exp(-x))
    }

    /// A date for a kept crater of size d: production-weighted, with older
    /// craters thinned by the ones that came after.
    fn date(&self, d_km: f64, u: f64) -> f64 {
        let ns = self.n_sat(d_km);
        let xe = self.n_prod(d_km, self.t_from, self.epoch) / ns;
        let e = m::exp(-xe);
        let x = xe + m::ln((u * (1.0 - e) + e).max(1e-300));
        let (mut lo, mut hi) = (self.t_from, self.epoch);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if self.n_prod(d_km, self.t_from, mid) / ns < x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// What made a crater at date t: the background or one spike.
    fn attribution(&self, t: f64, u: f64) -> Option<usize> {
        let dt = 0.5;
        let (a, b) = ((t - dt).max(0.0), t + dt);
        let bg = self.flux * (self.n1_moon(b) - self.n1_moon(a));
        let sp: Vec<f64> = self
            .spikes
            .iter()
            .map(|x| Self::spike_n1(x, a, b))
            .collect();
        let total = bg + sp.iter().sum::<f64>();
        if total <= 0.0 {
            return None;
        }
        let mut c = bg / total;
        if u < c {
            return None;
        }
        for (k, x) in sp.iter().enumerate() {
            c += x / total;
            if u < c {
                return Some(k);
            }
        }
        None
    }

    /// Chance a crater from t escapes the regime's global renewal: exp of
    /// minus the integrated renewal rate, which follows the heat flow.
    fn renewal_survival(&self, t: f64) -> f64 {
        let mut z = 0.0;
        for x in &self.spans {
            let (a, b) = (x.0.max(t), x.1.min(self.epoch));
            let Some(r) = self.s.resurface.get(&x.2) else {
                continue;
            };
            if b <= a {
                continue;
            }
            let steps = m::ceil((b - a) / 25.0).max(1.0) as u32;
            let dt = (b - a) / steps as f64;
            for k in 0..steps {
                let q = self.heat_at(a + (k as f64 + 0.5) * dt);
                let f = m::pow(
                    (q / r.q_ref_w_m2.max(1e-12)).max(0.0),
                    self.s.resurface_exponent,
                );
                z += dt * f / r.myr.max(1e-9);
            }
        }
        m::exp(-z)
    }

    fn crust_born(&self, v: [f64; 3]) -> Option<f64> {
        self.crust.as_ref().map(|(g, born)| born[g.cell_at(v)])
    }

    /// Build a crater and decide its fate: Ok(crater) or Err(reason erased).
    #[allow(clippy::too_many_arguments)]
    fn crater(
        &self,
        id: String,
        d_km: f64,
        t: f64,
        v: [f64; 3],
        u_attr: f64,
        u_renew: f64,
    ) -> Result<Crater, &'static str> {
        if d_km < self.atm_cut_km {
            return Err("stopped by the atmosphere");
        }
        if u_renew > self.renewal_survival(t) {
            return Err("resurfaced by the regime");
        }
        if let Some(born) = self.crust_born(v) {
            if born > t {
                return Err("on crust younger than the crater");
            }
        }
        let mut flooded = None;
        for p in &self.plains {
            let ang = m::acos(dot(p.dir, v).clamp(-1.0, 1.0));
            if p.formed_myr > t && ang * self.radius_km < p.radius_km {
                if d_km < self.s.volcanism.bury_d_km {
                    return Err("buried by lava plains");
                }
                flooded = Some(p.id.clone());
            }
        }
        let age = self.epoch - t;
        let (w0, w1) = (self.wet.0.max(t), self.wet.1.min(self.epoch));
        let wet = (w1 - w0).max(0.0);
        let k = self.s.kappa_dry_m2_myr * age
            + self.s.kappa_wet_m2_myr * wet
            + self.s.kappa_air_m2_myr_bar * self.pressure_bar * age;
        let dm = d_km * 1000.0;
        let fresh = m::exp(-k / (self.s.degrade_coeff * dm * dm));
        let ghost = fresh < self.s.ghost_fraction;
        let f = if ghost { 0.0 } else { fresh };
        let (shape, mut depth, rim, mut peak) = crater_shape(&self.s, self.g, d_km);
        if flooded.is_some() {
            // Lava fills the floor to a third of the rim-to-floor depth.
            depth = (0.33 * (depth + rim) - rim).max(0.0);
            peak = 0.0;
        }
        let (lat, lon) = lat_lon(v);
        let by = match self.attribution(t, u_attr) {
            None => "the declining background bombardment (lunar-type chronology)".to_string(),
            Some(k) => format!("a bombardment spike from {}", self.spikes[k].3),
        };
        let mut cause = format!(
            "Impact {:.2} Gyr ago from {by}. {:.0} km across under {:.2} m/s² gravity, so {}. ",
            age / 1000.0,
            d_km,
            self.g,
            match shape {
                "bowl" => "a simple bowl",
                "central-peak" => "complex, with a central peak",
                "peak-ring" => "a peak-ring basin",
                _ => "a multi-ring basin",
            }
        );
        if ghost {
            cause += "Worn away since: only the buried structure is left (a ghost crater).";
        } else {
            cause += &format!("{:.0}% of its fresh relief is left.", 100.0 * fresh);
        }
        if let Some(p) = &flooded {
            cause += &format!(" Its floor was flooded by {p}.");
        }
        Ok(Crater {
            id,
            dir: v,
            lat_deg: lat,
            lon_deg: lon,
            d_km,
            shape: shape.into(),
            depth_m: depth * f,
            rim_m: rim * f,
            peak_m: peak * f,
            formed_myr: t,
            fresh: f,
            ghost,
            flooded: flooded.is_some(),
            cause,
        })
    }

    /// A size in [lo, hi) from the production slopes.
    fn size(&self, lo: f64, hi: f64, u: f64) -> f64 {
        let b = if lo >= self.s.sfd_break_km {
            self.s.sfd_slope_large
        } else {
            self.s.sfd_slope_small
        };
        let r = m::pow(hi / lo, -b);
        lo * m::pow(1.0 - u * (1.0 - r), -1.0 / b)
    }

    /// Size band k below the listed size: [listed / 2^(k+1), listed / 2^k).
    pub fn band(&self, listed_km: f64, k: u32) -> (f64, f64) {
        let hi = listed_km / (1u64 << k) as f64;
        (0.5 * hi, hi)
    }

    /// Cells per face edge for band k: a cell is at least 1.5 diameters wide,
    /// so a crater (ejecta out to one diameter from its centre) reaches only
    /// its own cell and the eight around it.
    pub fn band_cells(&self, hi_km: f64) -> u32 {
        let n = m::floor((m::PI / 2.0) * self.radius_km / (1.5 * hi_km));
        n.clamp(1.0, (1u64 << 20) as f64) as u32
    }

    /// The band-k cell holding a direction.
    pub fn cell_of(&self, listed_km: f64, k: u32, v: [f64; 3]) -> (u8, u32, u32) {
        let (_, hi) = self.band(listed_km, k);
        let n = self.band_cells(hi);
        let (f, u, w) = crate::atlas::face_of(v);
        let ix = (((u + 1.0) * 0.5 * n as f64) as u32).min(n - 1);
        let iy = (((w + 1.0) * 0.5 * n as f64) as u32).min(n - 1);
        (f, ix, iy)
    }

    /// A cell and the eight around it (across face edges too).
    pub fn neighbourhood(&self, listed_km: f64, k: u32, c: (u8, u32, u32)) -> Vec<(u8, u32, u32)> {
        let (_, hi) = self.band(listed_km, k);
        let n = self.band_cells(hi) as f64;
        let mut out = vec![];
        for dj in -1i32..=1 {
            for di in -1i32..=1 {
                let u = -1.0 + 2.0 * (c.1 as f64 + 0.5 + di as f64) / n;
                let w = -1.0 + 2.0 * (c.2 as f64 + 0.5 + dj as f64) / n;
                let v = crate::atlas::direction(c.0, u, w);
                out.push(self.cell_of(listed_km, k, v));
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// Small craters of one band in one cell (any tile resolves these alone).
    pub fn cell_craters(&self, listed_km: f64, k: u32, face: u8, i: u32, j: u32) -> Vec<Crater> {
        let (lo, hi) = self.band(listed_km, k);
        let n = self.band_cells(hi);
        let area = 4.0 * m::PI * self.radius_km * self.radius_km / (6.0 * n as f64 * n as f64);
        let lam = area * (self.n_kept(lo) - self.n_kept(hi)).max(0.0);
        let key = |idx: u64, field: u64| -> u64 {
            ((((((k as u64 & 63) << 3) | face as u64) << 20 | i as u64) << 20 | j as u64) << 6
                | (idx & 63))
                << 4
                | field
        };
        let count = poisson(&self.d, key(63, 15), lam).min(62);
        let mut out = vec![];
        for c in 0..count {
            let du = (i as f64 + self.d.unit(key(c, 0))) / n as f64;
            let dv = (j as f64 + self.d.unit(key(c, 1))) / n as f64;
            let v = crate::atlas::direction(face, 2.0 * du - 1.0, 2.0 * dv - 1.0);
            let dk = self.size(lo, hi, self.d.unit(key(c, 2)));
            let t = self.date(dk, self.d.unit(key(c, 3)));
            let id = format!("crater:{k}.{face}.{i}.{j}.{c}");
            if let Ok(cr) =
                self.crater(id, dk, t, v, self.d.unit(key(c, 4)), self.d.unit(key(c, 5)))
            {
                if !cr.ghost {
                    out.push(cr);
                }
            }
        }
        out
    }

    /// Expected listed-size craters (at least `d_km`) kept after global renewal
    /// and crust recycling are ignored only where stated: production times the
    /// regime survival, integrated over the surface's life (unsaturated sizes).
    pub fn expected_after_renewal(&self, d_km: f64) -> f64 {
        let area = 4.0 * m::PI * self.radius_km * self.radius_km;
        let steps = 2000u32;
        let dt = (self.epoch - self.t_from) / steps as f64;
        let mut n = 0.0;
        for k in 0..steps {
            let a = self.t_from + k as f64 * dt;
            n += self.n_prod(d_km, a, a + dt) * self.renewal_survival(a + 0.5 * dt);
        }
        area * n
    }

    /// The chance that ground `d_km` across, made at `t0_myr`, is not yet
    /// erased by craters at least its size by `t1_myr`: e^(−N/N_sat), the
    /// same saturation law that thins old craters.
    pub fn erasure_survival(&self, d_km: f64, t0_myr: f64, t1_myr: f64) -> f64 {
        let d = d_km.max(1e-3);
        m::exp(-self.n_prod(d, t0_myr, t1_myr.max(t0_myr)) / self.n_sat(d))
    }

    /// Production of craters at least `d_km` across between two dates, per km².
    pub fn production(&self, d_km: f64, t0_myr: f64, t1_myr: f64) -> f64 {
        self.n_prod(d_km, t0_myr, t1_myr)
    }

    pub fn epoch_myr(&self) -> f64 {
        self.epoch
    }

    pub fn radius_km(&self) -> f64 {
        self.radius_km
    }
    pub fn gravity(&self) -> f64 {
        self.g
    }
    pub fn settings(&self) -> &ImpactSettings {
        &self.s
    }
    pub fn atm_cut_km(&self) -> f64 {
        self.atm_cut_km
    }
}

fn check_mechanisms(s: &ImpactSettings) -> Result<(), String> {
    let mm = &s.mechanisms;
    for (slot, name, known) in [
        ("chronology", &mm.chronology, &CHRONOLOGIES[..]),
        ("crater_shape", &mm.crater_shape, &CRATER_SHAPES[..]),
        ("volcanism", &mm.volcanism, &VOLCANISMS[..]),
    ] {
        if !known.contains(&name.as_str()) {
            return Err(format!(
                "no atom is registered for the {slot} slot named {name} (known: {})",
                known.join(", ")
            ));
        }
    }
    Ok(())
}

/// The inputs a world's impacts and volcanism need beyond its envelope.
#[derive(Clone, Debug)]
pub struct ImpactInputs<'a> {
    pub history: &'a ImpactHistory,
    pub water: &'a WaterHistory,
    pub pressure_bar: f64,
    /// Plate crust ages on plate worlds.
    pub plates: Option<&'a PlateHistory>,
}

/// Impacts and volcanism up to `epoch_gyr` (Gyr since the world formed).
pub fn impacts(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    inputs: &ImpactInputs,
    epoch_gyr: f64,
) -> Result<(Features, CraterField), String> {
    let is = &s.impacts;
    check_mechanisms(is)?;
    let d = Draws::new(
        *seed,
        env.address.clone(),
        Stream::new("surface.impacts.v1").expect("valid"),
    );
    let radius_km = env.reference_radius_m / 1000.0;
    let g = env.gravity_m_s2.max(1e-6);
    let end = env.timeline.last().map(|x| x.to_gyr).unwrap_or(0.0);
    let epoch = epoch_gyr.clamp(0.0, end) * 1000.0;
    let spans: Vec<(f64, f64, String, f64, f64)> = env
        .timeline
        .iter()
        .map(|x| {
            (
                x.from_gyr * 1000.0,
                x.to_gyr * 1000.0,
                x.regime.clone(),
                x.heat_flow_w_m2[0],
                x.heat_flow_w_m2[1],
            )
        })
        .collect();
    // The crater-holding surface began when the last surfaceless span ended.
    let now_regime = spans
        .iter()
        .find(|x| epoch >= x.0 && epoch <= x.1)
        .or(spans.last())
        .map(|x| x.2.clone())
        .unwrap_or_default();
    let holds = !is.no_surface.contains(&now_regime) && radius_km > 0.0;
    let t_from = spans
        .iter()
        .filter(|x| x.0 <= epoch && is.no_surface.contains(&x.2))
        .map(|x| x.1.min(epoch))
        .fold(0.0, f64::max);
    let v_esc = m::sqrt(2.0 * g * env.reference_radius_m) / 1000.0;
    let focus = 1.0 + (v_esc / is.v_inf_km_s) * (v_esc / is.v_inf_km_s);
    let gscale = m::pow(g / is.g_ref_m_s2, is.gravity_exponent);
    let area_km2 = 4.0 * m::PI * radius_km * radius_km;
    // Spike mass to N(>1 km) on this world: impactors with cumulative slope b
    // between the 1 km crater's impactor and the largest.
    let b = is.sfd_slope_large.min(2.999);
    let ratio = is.crater_impactor_ratio / gscale;
    let l1_km = 1.0 / ratio;
    let m1 = is.impactor_density * m::PI / 6.0 * m::pow(l1_km * 1000.0, 3.0);
    let span_l = m::pow(is.impactor_max_km / l1_km, 3.0 - b) - 1.0;
    let spikes = inputs
        .history
        .spikes
        .iter()
        .map(|sp| {
            let n = sp.earth * M_EARTH_KG * (3.0 - b) / (b * m1 * span_l.max(1e-9));
            (sp.t_myr, n / area_km2, sp.efold_myr, sp.event.clone())
        })
        .collect();
    let pressure = inputs.pressure_bar.max(0.0);
    let atm_cut_km = if pressure > 0.0 {
        is.atm_cut_km * m::pow(pressure, is.atm_cut_exp)
    } else {
        0.0
    };
    let crust = inputs.plates.filter(|h| h.steps > 0).map(|h| {
        (
            crate::plates::grid_for(h),
            h.cells.iter().map(|c| c.born_myr).collect::<Vec<_>>(),
        )
    });
    let mut field = CraterField {
        s: is.clone(),
        d,
        radius_km,
        g,
        gscale,
        flux: inputs.history.flux_scale * focus,
        spikes,
        t_from,
        epoch,
        atm_cut_km,
        spans,
        wet: (
            inputs.water.wet_from_gyr * 1000.0,
            inputs.water.wet_until_gyr * 1000.0,
        ),
        pressure_bar: pressure,
        crust,
        plains: vec![],
    };
    // The listed size grows until the expected list fits `listed_max`, so the
    // list is never truncated (a truncated list would bias the statistics).
    let mut listed = (is.listed_d_frac * radius_km).max(1e-3);
    while area_km2 * field.n_kept(listed) > is.listed_max as f64 && listed < radius_km {
        listed *= 1.25;
    }
    let mut f = Features {
        address: env.address.to_string(),
        epoch_myr: epoch,
        surface_from_myr: holds.then_some(t_from),
        listed_d_km: listed,
        expected_listed: 0.0,
        craters: vec![],
        plains: vec![],
        shields: vec![],
        erased: BTreeMap::new(),
        history_source: inputs.history.source.clone(),
        refs: REFS.iter().map(|x| x.to_string()).collect(),
    };
    if !holds {
        return Ok((f, field));
    }
    // Volcanism first: plains bury the craters older than them.
    if is.mechanisms.volcanism == "heat-flow-plains" {
        volcanism(&mut field, &mut f, env);
    }
    field.plains = f.plains.clone();
    let lam = area_km2 * field.n_kept(listed);
    f.expected_listed = lam;
    let base: u64 = 1 << 60;
    let n = poisson(&field.d, base, lam);
    let dmax = (is.max_d_frac * radius_km).max(listed * 1.0001);
    for j in 0..n {
        let k = |x: u64| base + 16 * (j + 1) + x;
        let dk = field.size(listed, dmax, field.d.unit(k(0)));
        let t = field.date(dk, field.d.unit(k(1)));
        let v = unit_dir(&field.d, k(2));
        match field.crater(
            format!("crater:{j}"),
            dk,
            t,
            v,
            field.d.unit(k(5)),
            field.d.unit(k(6)),
        ) {
            Ok(c) => f.craters.push(c),
            Err(why) => *f.erased.entry(why.into()).or_insert(0) += 1,
        }
    }
    Ok((f, field))
}

fn volcanism(field: &mut CraterField, f: &mut Features, env: &SurfaceEnvelope) {
    let v = field.s.volcanism.clone();
    let area = 4.0 * m::PI * field.radius_km * field.radius_km;
    let (lo, hi) = (v.plains_radius_frac[0], v.plains_radius_frac[1]);
    let mean_area = {
        // Mean of π r² over log-uniform r in [lo, hi] R.
        let rr = field.radius_km;
        m::PI * rr * rr * (hi * hi - lo * lo) / (2.0 * m::ln(hi / lo).max(1e-9))
    };
    let base: u64 = 1 << 61;
    let mut covered = 0.0;
    let mut t = field.t_from;
    let mut step = 0u64;
    while t < field.epoch && step < 4000 {
        let t1 = (t + v.step_myr).min(field.epoch);
        let mid = 0.5 * (t + t1);
        let regime = field.regime_at(mid).to_string();
        let q = field.heat_at(mid);
        let drive = ((q - v.q_min_w_m2) / (v.q_ref_w_m2 - v.q_min_w_m2).max(1e-9)).max(0.0);
        if v.regimes.contains(&regime) && drive > 0.0 {
            let frac = (v.plains_per_gyr * drive * (t1 - t) / 1000.0)
                .min((v.plains_cap - covered).max(0.0));
            covered += frac;
            let k = |x: u64| base + 64 * step + x;
            let units = poisson(&field.d, k(0), frac * area / mean_area).min(20);
            for u in 0..units {
                let kk = |x: u64| base + (1 << 40) + 64 * (32 * step + u) + x;
                let r = lo * m::pow(hi / lo, field.d.unit(kk(0))) * field.radius_km;
                let when = t + (t1 - t) * field.d.unit(kk(1));
                f.plains.push(Plains {
                    id: format!("plains:{}", f.plains.len()),
                    dir: unit_dir(&field.d, kk(2)),
                    radius_km: r,
                    formed_myr: when,
                    cause: format!(
                        "Lava plains {:.2} Gyr ago: heat flow {:.3} W/m² under a {} let melt reach the surface (it needs more than {:.3} W/m²).",
                        (field.epoch - when) / 1000.0,
                        q,
                        env.regime_label_of(&regime),
                        v.q_min_w_m2
                    ),
                });
            }
            let shields_here =
                v.shield_regimes.contains(&regime) && field.radius_km >= v.shield_min_radius_km;
            let ns = if shields_here {
                poisson(
                    &field.d,
                    k(10),
                    v.shields_per_gyr * drive * (t1 - t) / 1000.0,
                )
            } else {
                0
            };
            for u in 0..ns {
                if f.shields.len() >= v.shields_max as usize {
                    break;
                }
                let kk = |x: u64| base + (1 << 50) + 64 * (32 * step + u) + x;
                let born = t + (t1 - t) * field.d.unit(kk(0));
                // Active while the heat flow stays above the threshold.
                let mut end = born;
                while end < field.epoch && field.heat_at(end) > v.q_min_w_m2 {
                    end += v.step_myr.max(1.0);
                }
                let end = end.min(field.epoch);
                let grow = 1.0 - m::exp(-(end - born) / v.shield_growth_myr.max(1e-9));
                let h = v.shield_height_m * (9.81 / field.g) * grow.max(0.05);
                f.shields.push(Shield {
                    id: format!("shield:{}", f.shields.len()),
                    dir: unit_dir(&field.d, kk(2)),
                    height_m: h,
                    radius_km: v.shield_aspect * h / 1000.0,
                    from_myr: born,
                    to_myr: end,
                    cause: format!(
                        "A shield volcano over a fixed source under the lid, erupting from {:.2} to {:.2} Gyr ago. Gravity {:.2} m/s² lets it stand {:.1} km tall.",
                        (field.epoch - born) / 1000.0,
                        (field.epoch - end) / 1000.0,
                        field.g,
                        h / 1000.0
                    ),
                });
            }
        }
        t = t1;
        step += 1;
    }
}

impl SurfaceEnvelope {
    fn regime_label_of(&self, regime: &str) -> String {
        regime.replace('-', " ")
    }
}

impl Features {
    /// Why the ground at a place looks as it does, newest first.
    pub fn explain(&self, lat_deg: f64, lon_deg: f64, radius_km: f64) -> Vec<String> {
        let (la, lo) = (lat_deg.to_radians(), lon_deg.to_radians());
        let v = [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)];
        let dist = |d: [f64; 3]| m::acos(dot(d, v).clamp(-1.0, 1.0)) * radius_km;
        let mut out: Vec<(f64, String)> = vec![];
        for c in &self.craters {
            if dist(c.dir) <= 0.5 * c.d_km {
                out.push((c.formed_myr, format!("{}: {}", c.id, c.cause)));
            }
        }
        for p in &self.plains {
            if dist(p.dir) <= p.radius_km {
                out.push((p.formed_myr, format!("{}: {}", p.id, p.cause)));
            }
        }
        for s in &self.shields {
            if dist(s.dir) <= s.radius_km {
                out.push((s.from_myr, format!("{}: {}", s.id, s.cause)));
            }
        }
        out.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut lines: Vec<String> = out.into_iter().map(|x| x.1).collect();
        match self.surface_from_myr {
            Some(t) => lines.push(format!(
                "The surface has held craters since {:.2} Gyr ago ({}).",
                (self.epoch_myr - t) / 1000.0,
                self.history_source
            )),
            None => lines.push("No solid surface holds craters here.".into()),
        }
        lines
    }

    /// Crater-count model age, Myr before the epoch, from craters at least
    /// `d_km` across over an area (the inverse of the chronology; production
    /// only, so valid below saturation).
    pub fn model_age_myr(field: &CraterField, d_km: f64, count: usize, area_km2: f64) -> f64 {
        let target = count as f64 / area_km2.max(1e-12);
        let (mut lo, mut hi) = (0.0, field.epoch - field.t_from);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if field.n_prod(d_km, field.epoch - mid, field.epoch) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

/// Earth radii to km, for callers.
pub fn radius_km_of(radius_earth: f64) -> f64 {
    radius_earth * R_EARTH_KM
}
