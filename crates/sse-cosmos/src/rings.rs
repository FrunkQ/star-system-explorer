//! R: rings and what died here before (galaxy design section 6, stage G3).
//!
//! A disc split into radial rings (default 24, log-spaced in units of the
//! disc scale length), classic multi-zone chemical evolution with no
//! particles (Matteucci and Francois 1989; Chiappini et al. 1997, 2001).
//! Per ring, per step (default 50 Myr, five galaxy steps):
//! - infall: the galaxy's inflow shared out with weights
//!   area x exp(-R/R_d) x exp(-t/tau(R)) / tau(R), tau growing with radius
//!   (inside-out; Chiappini et al. 2001 use tau = 1.033 R/kpc - 1.267 Gyr);
//!   infall is primordial except for the share that is earlier wind
//!   material falling back, which carries the disc's mean abundances;
//! - star formation: Kennicutt-Schmidt, Sigma_SFR = A Sigma_gas^1.4
//!   (Kennicutt 1998), dropping steeply below a threshold density
//!   (Martin and Kennicutt 2001);
//! - winds: the galaxy's mass loading at that time;
//! - element groups, each a conserved budget in solar units (mass / solar
//!   mass fraction, Asplund et al. 2009), so [X/H] = log10(units / gas):
//!   alpha (oxygen as tracer) from core-collapse supernovae, prompt;
//!   iron peak from core collapse (prompt) and type Ia (delay-time
//!   distribution proportional to 1/t from 40 Myr, Maoz and Mannucci 2012);
//!   C and N and the s-process from dying intermediate stars (AGB, 1/t
//!   from 100 Myr); r-process from neutron-star mergers (1/t from 30 Myr),
//!   rare enough to be a keyed Poisson draw per ring and step, so one ring
//!   can be uranium-rich and the next poor; K-40 from core collapse,
//!   decaying with its 1.248 Gyr half-life.
//! - the ledger: per ring, the stars formed in each step with the gas
//!   abundances they were born from, and the counts of each kind of death.
//!
//! Al-26 needs a supernova within about a megayear of a birth cluster: it
//! belongs to the patch and cluster stages (G4, G5), not to rings.

use crate::galaxy::{Galaxy, GalaxyParams};
use crate::haloes::poisson;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};

/// Element groups, in this order everywhere.
pub const GROUPS: [&str; 6] = ["alpha", "fe", "cn", "s", "r", "k40"];
pub const ALPHA: usize = 0;
pub const FE: usize = 1;
pub const CN: usize = 2;
pub const S: usize = 3;
pub const R: usize = 4;
pub const K40: usize = 5;

/// Pack settings (`provides.other.cosmology.rings`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RingParams {
    pub rings: u32,
    /// Inner and outer ring edges, in disc scale lengths.
    pub inner_rd: f64,
    pub outer_rd: f64,
    /// Galaxy steps per ring step.
    pub every: u32,
    /// Inside-out infall: tau = tau0 + tau1 (R / R_d), floored at tau_min (Gyr).
    pub tau0_gyr: f64,
    pub tau1_gyr: f64,
    pub tau_min_gyr: f64,
    /// Kennicutt-Schmidt: A (solar masses per year per kpc^2 at 1 solar mass per pc^2) and n.
    pub ks_a: f64,
    pub ks_n: f64,
    /// Below this gas density (solar masses per pc^2) star formation falls as (Sigma / Sigma_th)^k extra.
    pub threshold: f64,
    pub threshold_slope: f64,
    pub return_fraction: f64,
    /// Share of the disc's mean abundances that fallback gas carries (it mixes with
    /// pristine gas in the halo on the way back).
    pub fallback_metals: f64,
    /// Winds leave the thin outer disc more easily: ring mass loading is the
    /// galaxy's times exp(k (R/R_d - R_ref/R_d)) (radially rising loading is what
    /// observed gradients need, Belfiore et al. 2019).
    pub eta_radial: f64,
    pub eta_ref_rd: f64,
    /// Slow inward gas drift, km/s (Lacey and Fall 1985; Spitoni and Matteucci 2011
    /// find about 1 km/s steepens the gradient as observed). Gas drifting out of the
    /// innermost ring feeds the nucleus.
    pub drift_kms: f64,
    /// Yields in solar units per solar mass of stars formed.
    pub yield_alpha: f64,
    pub yield_fe_cc: f64,
    pub yield_cn: f64,
    pub yield_s: f64,
    pub yield_k40: f64,
    /// Type Ia: events per solar mass formed, iron per event (solar units), delay range (Gyr).
    pub ia_per_mass: f64,
    pub ia_fe: f64,
    pub ia_min_gyr: f64,
    /// AGB delay range (Gyr).
    pub agb_min_gyr: f64,
    /// Neutron-star mergers: events per solar mass formed, r-process per event (solar units), minimum delay (Gyr).
    pub nsm_per_mass: f64,
    pub nsm_r: f64,
    pub nsm_min_gyr: f64,
    /// Core-collapse supernovae per solar mass formed (8 to 40 solar masses of a Chabrier IMF).
    pub cc_per_mass: f64,
    pub k40_half_life_gyr: f64,
}

impl Default for RingParams {
    fn default() -> Self {
        RingParams {
            rings: 24,
            inner_rd: 0.1,
            outer_rd: 8.0,
            every: 5,
            // Chiappini et al. 2001 in units of the Milky Way's R_d = 2.6 kpc.
            tau0_gyr: -1.267,
            tau1_gyr: 2.686,
            tau_min_gyr: 0.5,
            ks_a: 2.5e-4,
            ks_n: 1.4,
            threshold: 4.0,
            threshold_slope: 2.0,
            return_fraction: 0.4,
            fallback_metals: 0.5,
            eta_radial: 0.6,
            eta_ref_rd: 3.15,
            drift_kms: 1.0,
            yield_alpha: 2.0,
            yield_fe_cc: 0.7,
            yield_cn: 1.5,
            yield_s: 1.5,
            yield_k40: 1.5,
            ia_per_mass: 2.0e-3,
            ia_fe: 540.0,
            ia_min_gyr: 0.04,
            agb_min_gyr: 0.1,
            nsm_per_mass: 1.0e-5,
            nsm_r: 1.5e5,
            nsm_min_gyr: 0.03,
            cc_per_mass: 0.0118,
            k40_half_life_gyr: 1.248,
        }
    }
}

/// What died here: counts of each kind of stellar death, cumulative.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Deaths {
    pub core_collapse: f64,
    pub type_ia: f64,
    pub neutron_star_mergers: u64,
    /// Gas returned by dying intermediate stars, solar masses.
    pub agb_returned: f64,
}

/// One ring at the end of the run, with its history.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ring {
    pub inner_kpc: f64,
    pub outer_kpc: f64,
    pub radius_kpc: f64,
    pub gas: f64,
    /// Gas abundances in solar units (see the module notes), by group.
    pub units: [f64; 6],
    /// Stars formed in each ring step (solar masses).
    pub formed: Vec<f64>,
    /// [Fe/H] of the gas each step's stars formed from.
    pub feh: Vec<f64>,
    /// [alpha/Fe] of that gas.
    pub alpha_fe: Vec<f64>,
    pub deaths: Deaths,
    /// Present rates per year: core collapse, type Ia.
    pub cc_rate: f64,
    pub ia_rate: f64,
}

impl Ring {
    pub fn abundance(&self, g: usize) -> f64 {
        if self.gas > 0.0 && self.units[g] > 0.0 {
            m::log10(self.units[g] / self.gas)
        } else {
            -9.0
        }
    }
    pub fn stars(&self) -> f64 {
        (1.0 - 0.4) * self.formed.iter().sum::<f64>()
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Disc {
    pub rings: Vec<Ring>,
    pub step_gyr: f64,
    pub t_start_gyr: f64,
    /// Ring-step [Fe/H] history per ring (gas), every step.
    pub feh_history: Vec<Vec<f64>>,
    /// Every ring at the end of every step: `states[ring][step]`.
    pub states: Vec<Vec<RingState>>,
}

/// One ring at the end of one step (abundances as [X/H] in dex, rates per year).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RingState {
    pub gas: f64,
    pub stars: f64,
    pub feh: f64,
    pub alpha_h: f64,
    pub cn_h: f64,
    pub s_h: f64,
    pub r_h: f64,
    pub k40_h: f64,
    pub sfr: f64,
    pub cc_rate: f64,
    pub ia_rate: f64,
    pub nsm_so_far: u64,
}

impl Disc {
    /// Time (Gyr) at the start of ring step k.
    pub fn time(&self, k: usize) -> f64 {
        self.t_start_gyr + self.step_gyr * k as f64
    }
    /// The ring holding radius r (kpc).
    pub fn ring_at(&self, r: f64) -> &Ring {
        let i = self
            .rings
            .partition_point(|g| g.outer_kpc < r)
            .min(self.rings.len() - 1);
        &self.rings[i]
    }
    pub fn core_collapse_rate(&self) -> f64 {
        self.rings.iter().map(|r| r.cc_rate).sum()
    }
    pub fn ia_rate(&self) -> f64 {
        self.rings.iter().map(|r| r.ia_rate).sum()
    }
}

/// Share of a 1/t delay-time distribution on [lo, hi] falling in each step of length dt.
fn dtd_weights(lo: f64, hi: f64, dt: f64, n: usize) -> Vec<f64> {
    let norm = m::ln(hi / lo);
    (0..n)
        .map(|j| {
            let a = (j as f64 * dt).max(lo);
            let b = ((j + 1) as f64 * dt).min(hi);
            if b > a {
                m::ln(b / a) / norm
            } else {
                0.0
            }
        })
        .collect()
}

/// Run the rings of one galaxy (`address` is its halo's, for the lumpy draws).
pub fn evolve(
    gal: &Galaxy,
    gp: &GalaxyParams,
    p: &RingParams,
    seed: Seed,
    address: &Address,
) -> Disc {
    evolve_clocked(gal, gp, p, seed, address, None)
}

/// The inside-out clock's total: all the gas a galaxy takes in over its life.
/// An edited history keeps the unedited galaxy's clock (`evolve_clocked`), so
/// the rings before an edit are bit-identical; with its own total, a later
/// change in inflow would reshape the whole inside-out profile back to the start.
pub fn inflow_clock(gal: &Galaxy, p: &RingParams) -> f64 {
    let every = p.every.max(1) as usize;
    let n = gal.inflow.len() / every;
    gal.inflow[..n * every].iter().sum::<f64>().max(1.0e-30)
}

/// `evolve` with the inside-out clock's total given (None: the galaxy's own).
pub fn evolve_clocked(
    gal: &Galaxy,
    gp: &GalaxyParams,
    p: &RingParams,
    seed: Seed,
    address: &Address,
    clock_total: Option<f64>,
) -> Disc {
    let every = p.every.max(1) as usize;
    let n = gal.inflow.len() / every;
    let dt = gal.step_gyr * every as f64;
    let rd = gal.now.disc_kpc.max(0.1);
    let nr = p.rings.max(1) as usize;
    let edges: Vec<f64> = (0..=nr)
        .map(|i| rd * p.inner_rd * m::pow(p.outer_rd / p.inner_rd, i as f64 / nr as f64))
        .collect();
    let mid: Vec<f64> = (0..nr).map(|i| m::sqrt(edges[i] * edges[i + 1])).collect();
    // Area in pc^2 (for densities) and the infall profile weights.
    let area: Vec<f64> = (0..nr)
        .map(|i| m::PI * (edges[i + 1] * edges[i + 1] - edges[i] * edges[i]) * 1.0e6)
        .collect();
    let tau: Vec<f64> = mid
        .iter()
        .map(|r| (p.tau0_gyr + p.tau1_gyr * r / rd).max(p.tau_min_gyr))
        .collect();
    let ia_w = dtd_weights(p.ia_min_gyr, 13.8, dt, n);
    let agb_w = dtd_weights(p.agb_min_gyr, 13.8, dt, n);
    let nsm_w = dtd_weights(p.nsm_min_gyr, 13.8, dt, n);
    let decay = m::exp(-core::f64::consts::LN_2 * dt / p.k40_half_life_gyr);
    let draws = Draws::new(
        seed,
        address.clone(),
        Stream::new("cosmos.rings.nsm.v1").expect("stream"),
    );

    let mut gas = vec![0.0; nr];
    let mut units = vec![[0.0f64; 6]; nr];
    let mut formed = vec![vec![0.0f64; n]; nr];
    let mut feh = vec![vec![-9.0f64; n]; nr];
    let mut afe = vec![vec![0.0f64; n]; nr];
    let mut deaths = vec![Deaths::default(); nr];
    let mut rates = vec![(0.0, 0.0); nr];
    let mut feh_hist = vec![vec![]; nr];
    let mut states: Vec<Vec<RingState>> = vec![Vec::with_capacity(n); nr];
    let mut stars_so_far = vec![0.0f64; nr];
    let total: f64 = clock_total.unwrap_or_else(|| inflow_clock(gal, p));
    let span = dt * n as f64;
    let mut cum = 0.0;
    for k in 0..n {
        let hist =
            &gal.history[(k * every / gp.history_every.max(1) as usize).min(gal.history.len() - 1)];
        let eta = gp.eta(hist.v_c, hist.z);
        let inflow: f64 = gal.inflow[k * every..(k + 1) * every].iter().sum();
        let back: f64 = gal.returned[k * every..(k + 1) * every].iter().sum();
        // Mean abundances of the disc's gas, for the fallback share.
        let gsum: f64 = gas.iter().sum();
        let mut mean = [0.0f64; 6];
        if gsum > 0.0 {
            for g in 0..6 {
                mean[g] = units.iter().map(|u| u[g]).sum::<f64>() / gsum;
            }
        }
        // Inside-out in the galaxy's own clock: x = share of all inflow so far, so a
        // ring with tau has received (1 - e^(-x/x_i)) / (1 - e^(-1/x_i)) of its total.
        let x0 = cum / total;
        cum += inflow;
        let x1 = cum / total;
        let wts: Vec<f64> = (0..nr)
            .map(|i| {
                let xi = tau[i] / span;
                let f = |x: f64| (1.0 - m::exp(-x / xi)) / (1.0 - m::exp(-1.0 / xi));
                area[i] * m::exp(-mid[i] / rd) * (f(x1) - f(x0))
            })
            .collect();
        let wsum: f64 = wts.iter().sum();
        for i in 0..nr {
            let add = if wsum > 0.0 {
                inflow * wts[i] / wsum
            } else {
                0.0
            };
            gas[i] += add;
            if inflow > 0.0 {
                for g in 0..6 {
                    units[i][g] += add * (back / inflow) * p.fallback_metals * mean[g];
                }
            }
            // Star formation this step.
            let sigma = gas[i] / area[i];
            let mut rate = p.ks_a * m::pow(sigma, p.ks_n) * area[i] / 1.0e6; // per year
            if sigma < p.threshold {
                rate *= m::pow(sigma / p.threshold, p.threshold_slope);
            }
            let eta = (eta * m::exp(p.eta_radial * (mid[i] / rd - p.eta_ref_rd))).min(gp.eta_max);
            let sf = (rate * dt * 1.0e9).min(gas[i] / (1.0 - p.return_fraction + eta) * 0.9);
            formed[i][k] = sf;
            feh[i][k] = if gas[i] > 0.0 && units[i][FE] > 0.0 {
                m::log10(units[i][FE] / gas[i])
            } else {
                -9.0
            };
            afe[i][k] = if units[i][FE] > 0.0 && units[i][ALPHA] > 0.0 {
                m::log10(units[i][ALPHA] / units[i][FE])
            } else {
                0.0
            };
            // Gas leaves into stars and winds, carrying its abundances.
            let out = (1.0 - p.return_fraction + eta) * sf;
            let keep = if gas[i] > 0.0 {
                (1.0 - out / gas[i]).max(0.0)
            } else {
                0.0
            };
            gas[i] *= keep;
            for u in units[i].iter_mut() {
                *u *= keep;
            }
            // Prompt enrichment from this step's massive stars.
            units[i][ALPHA] += p.yield_alpha * sf;
            units[i][FE] += p.yield_fe_cc * sf;
            units[i][K40] += p.yield_k40 * sf;
            deaths[i].core_collapse += p.cc_per_mass * sf;
            // Delayed channels: every earlier step's stars, through the delay-time weights.
            let (mut ia, mut agb, mut nsm) = (0.0, 0.0, 0.0);
            for j in 0..=k {
                let f = formed[i][k - j];
                if f > 0.0 {
                    ia += f * ia_w[j];
                    agb += f * agb_w[j];
                    nsm += f * nsm_w[j];
                }
            }
            let n_ia = p.ia_per_mass * ia;
            units[i][FE] += p.ia_fe * n_ia;
            units[i][CN] += p.yield_cn * agb;
            units[i][S] += p.yield_s * agb;
            deaths[i].type_ia += n_ia;
            deaths[i].agb_returned += p.return_fraction * agb;
            let lambda = p.nsm_per_mass * nsm;
            let events = poisson(&draws, (k * nr + i) as u64, lambda);
            units[i][R] += p.nsm_r * events as f64;
            deaths[i].neutron_star_mergers += events;
            units[i][K40] *= decay;
            rates[i] = (p.cc_per_mass * sf / (dt * 1.0e9), n_ia / (dt * 1.0e9));
            let ab = |g: usize| {
                if gas[i] > 0.0 && units[i][g] > 0.0 {
                    m::log10(units[i][g] / gas[i])
                } else {
                    -9.0
                }
            };
            feh_hist[i].push(ab(FE));
            stars_so_far[i] += (1.0 - p.return_fraction) * sf;
            states[i].push(RingState {
                gas: gas[i],
                stars: stars_so_far[i],
                feh: ab(FE),
                alpha_h: ab(ALPHA),
                cn_h: ab(CN),
                s_h: ab(S),
                r_h: ab(R),
                k40_h: ab(K40),
                sfr: sf / (dt * 1.0e9),
                cc_rate: rates[i].0,
                ia_rate: rates[i].1,
                nsm_so_far: deaths[i].neutron_star_mergers,
            });
        }
        // Inward drift: a share v dt / width of each ring's gas moves one ring in.
        let reach = p.drift_kms * 1.0227 * dt; // kpc per step (1 km/s = 1.0227 kpc/Gyr)
        for i in 0..nr {
            let share = (reach / (edges[i + 1] - edges[i])).min(0.5);
            let moved = gas[i] * share;
            let mu: [f64; 6] = core::array::from_fn(|g| units[i][g] * share);
            gas[i] -= moved;
            for g in 0..6 {
                units[i][g] -= mu[g];
            }
            if i > 0 {
                gas[i - 1] += moved;
                for g in 0..6 {
                    units[i - 1][g] += mu[g];
                }
            }
        }
    }
    let rings = (0..nr)
        .map(|i| Ring {
            inner_kpc: edges[i],
            outer_kpc: edges[i + 1],
            radius_kpc: mid[i],
            gas: gas[i],
            units: units[i],
            formed: formed[i].clone(),
            feh: feh[i].clone(),
            alpha_fe: afe[i].clone(),
            deaths: deaths[i].clone(),
            cc_rate: rates[i].0,
            ia_rate: rates[i].1,
        })
        .collect();
    Disc {
        rings,
        step_gyr: dt,
        t_start_gyr: gal.t_start_gyr,
        feh_history: feh_hist,
        states,
    }
}
