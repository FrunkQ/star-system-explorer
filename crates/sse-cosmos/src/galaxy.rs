//! Gx: one galaxy as a regulated reservoir (galaxy design section 5, stage G3).
//!
//! A small set of rate equations per galaxy, stepped on a fixed grid (default
//! 10 Myr) from z = 20 to now: the "bathtub" or regulator model (Bouche et
//! al. 2010; Lilly et al. 2013; Dave et al. 2012).
//! - Gas supply = baryon fraction x halo growth x an efficiency that is zero
//!   below the reionisation filtering mass after reionisation (Gnedin 2000)
//!   and falls in hot haloes above about 1e12 solar masses.
//! - Star formation = gas / depletion time, t_dep = t0 (1+z)^-k (Tacconi et
//!   al. 2018 find depletion times fall towards the past).
//! - Preventive feedback: small haloes heat their surroundings so less gas
//!   falls in at all (Dave, Finlator and Oppenheimer 2012), which lets small
//!   galaxies stay metal-rich enough while staying small.
//! - Outflow = eta x star formation, eta = (v_c / v_eta)^-a (slope 2 is the
//!   energy-driven-wind scaling; the (1+z) term after Muratov et al. 2015, tuned).
//! - Gas returned by dying stars: an instantaneous fraction R (the delays are
//!   in the rings, section 6).
//! - Metals: a yield per mass formed; outflows carry gas-phase metals out.
//! - Major mergers bring the incoming galaxy's stars into the bulge (ex-situ
//!   growth, ratio^2 of the host's stars; in big haloes the smooth ex-situ
//!   term below counts the satellites instead).
//! - Black hole: seeded, grows steadily with its galaxy and in merger-driven
//!   bursts; a big black hole in a big halo stops the hot halo cooling
//!   (radio-mode feedback, Croton et al. 2006), smoothly, with no threshold.
//! - Shape: disc scale length from spin and virial radius (Mo, Mao and White
//!   1998, simplified); a major merger moves stars into a bulge and triggers
//!   a starburst; a disc heavy for its halo grows a bar (Efstathiou, Lake and
//!   Negroponte 1982). Morphology is a classification rule over these numbers.

use crate::cosmology::{Cosmology, YEAR_S};
use crate::haloes::Halo;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// G in kpc (km/s)^2 per solar mass.
pub const G_KPC: f64 = sse_core::sheet::GM_SUN / (1e3 * sse_core::sheet::PC) / 1e6;

/// Pack settings (`provides.other.cosmology.galaxy`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GalaxyParams {
    pub step_myr: f64,
    pub z_start: f64,
    /// Reionisation redshift (from G1's epochs) and the filtering mass after it.
    pub z_reion: f64,
    pub filtering_mass: f64,
    /// Hot-halo suppression of cooling: 1 / (1 + (M_h / M_hot)^k).
    pub hot_mass: f64,
    pub hot_slope: f64,
    pub depletion_gyr: f64,
    pub depletion_z_slope: f64,
    /// Small galaxies are atomic-gas dominated and slow to form stars:
    /// t_dep times (v_c / v_dep)^-k below v_dep (Leroy et al. 2008; Huang et al. 2012).
    pub depletion_v: f64,
    pub depletion_v_slope: f64,
    /// Preventive feedback: inflow times 1 / (1 + (v_c / v_prev)^-k) (Dave, Finlator and Oppenheimer 2012).
    pub prevent_v: f64,
    pub prevent_slope: f64,
    pub eta_v: f64,
    pub eta_slope: f64,
    pub eta_max: f64,
    /// Winds are stronger in the young universe: eta times (1+z)^k (Muratov et al. 2015 find k near 1.3).
    pub eta_z_slope: f64,
    /// Ejected gas falls back on t = gamma (1e10 / M_h) (Henriques et al. 2013),
    /// so small galaxies form their stars late, as observed.
    pub return_gamma_gyr: f64,
    pub return_fraction: f64,
    /// Metal yield per mass of stars formed (all metals).
    pub yield_z: f64,
    /// Smooth ex-situ growth in big haloes: satellites sinking by dynamical
    /// friction bring a fraction of the halo's new mass times a typical
    /// stellar-to-halo ratio, weighted by (1 - hot-halo factor). Massive
    /// galaxies are mostly built this way (Rodriguez-Gomez et al. 2016).
    pub exsitu_fraction: f64,
    pub exsitu_ratio: f64,
    pub seed_black_hole: f64,
    /// Fraction of a burst's stars that feeds the black hole.
    pub black_hole_feed: f64,
    /// Steady feeding: black hole growth per mass of stars formed. Holes
    /// track bulges (Kormendy and Ho 2013); feeding them from all star
    /// formation, disc as well, is our simplification.
    pub black_hole_steady: f64,
    /// Black holes brought in with ex-situ stars: the merging galaxies' holes
    /// sink with their stars and coalesce, at this hole-to-stars ratio: the
    /// bulge ratio of Kormendy and Ho 2013 (Reines and Volonteri 2015 find
    /// 2.5e-4 against total stellar mass, so it runs high for disc-rich
    /// arrivals). This is how the
    /// biggest galaxies get billion-sun holes.
    #[serde(default = "exsitu_black_hole_default")]
    pub exsitu_black_hole: f64,
    /// Gas turned into stars in a major merger, times the mass ratio.
    pub burst_fraction: f64,
    /// Radio-mode feedback: cooling times 1 / (1 + (M_h / M_q)^k (M_bh / M_bh,q)),
    /// so a big black hole in a big halo shuts off the gas supply (Croton et al. 2006).
    pub quench_halo: f64,
    pub quench_halo_slope: f64,
    pub quench_black_hole: f64,
    /// A galaxy counts as quenched below this specific star formation rate, per year.
    pub quenched_ssfr: f64,
    /// Disc scale length = f (spin / sqrt 2) R_vir.
    pub disc_factor: f64,
    /// Bar instability below this Efstathiou-Lake-Negroponte value.
    pub bar_threshold: f64,
    /// Steps between stored history points.
    pub history_every: u32,
    /// DM4: the dark matter profile and the rotation curve.
    #[serde(default)]
    pub structure: crate::structure::StructureParams,
}

fn exsitu_black_hole_default() -> f64 {
    1.0e-3
}

impl Default for GalaxyParams {
    fn default() -> Self {
        GalaxyParams {
            step_myr: 10.0,
            z_start: 20.0,
            z_reion: 7.7,
            filtering_mass: 1.0e9,
            hot_mass: 5.0e11,
            hot_slope: 2.0,
            depletion_gyr: 1.5,
            depletion_z_slope: 1.0,
            depletion_v: 150.0,
            depletion_v_slope: 1.5,
            prevent_v: 60.0,
            prevent_slope: 3.0,
            eta_v: 200.0,
            eta_slope: 2.0,
            eta_max: 60.0,
            eta_z_slope: 1.0,
            return_gamma_gyr: 60.0,
            return_fraction: 0.4,
            yield_z: 0.02,
            exsitu_fraction: 0.3,
            exsitu_ratio: 0.02,
            seed_black_hole: 1.0e5,
            black_hole_feed: 0.01,
            black_hole_steady: 5.0e-4,
            exsitu_black_hole: exsitu_black_hole_default(),
            burst_fraction: 0.5,
            quench_halo: 1.0e12,
            quench_halo_slope: 2.0,
            quench_black_hole: 3.0e7,
            quenched_ssfr: 1.0e-11,
            disc_factor: 0.6,
            bar_threshold: 1.1,
            history_every: 5,
            structure: Default::default(),
        }
    }
}

/// One point on a galaxy's history curves.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GalaxyState {
    pub t_gyr: f64,
    pub z: f64,
    pub halo: f64,
    pub gas: f64,
    pub stars: f64,
    pub bulge: f64,
    /// Star formation rate, solar masses per year.
    pub sfr: f64,
    /// Gas-phase metal mass fraction.
    pub z_gas: f64,
    pub black_hole: f64,
    pub quenched: bool,
    /// Virial circular speed, km/s.
    pub v_c: f64,
    /// Disc scale length, kpc.
    pub disc_kpc: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Morphology {
    Dwarf,
    Irregular,
    Disc,
    Barred,
    Lenticular,
    Elliptical,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Galaxy {
    pub halo: String,
    pub history: Vec<GalaxyState>,
    pub now: GalaxyState,
    pub morphology: Morphology,
    pub barred: bool,
    /// When the bar formed (universe time, Gyr): the earliest time from which
    /// the disc has been bar-unstable ever since; none for an unbarred galaxy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bar_from_gyr: Option<f64>,
    /// Active-nucleus episodes (universe time, Gyr; black hole mass gained).
    pub active_nuclei: Vec<(f64, f64)>,
    /// Gas inflow per step (for the rings), solar masses, and the part of it
    /// that is earlier wind material falling back (enriched).
    pub inflow: Vec<f64>,
    pub returned: Vec<f64>,
    pub step_gyr: f64,
    pub t_start_gyr: f64,
    /// DM4: the circular speed now, (r kpc, v km/s) on log-spaced radii
    /// from 0.05 kpc to r200 (`structure::Structure::speeds`).
    #[serde(default)]
    pub rotation: Vec<(f64, f64)>,
    /// GH3: what its assembly tree brought, when it was fed by one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembled: Option<crate::feed::Assembled>,
    /// DM3: gas thrown out by winds and not yet back, now (solar masses);
    /// the matter ledger reads it, the wire does not carry it.
    #[serde(default, skip_serializing)]
    pub ejected_now: f64,
}

/// a(t) on the galaxy's grid, by stepping the cosmology's time integral in log a.
pub fn time_table(c: &Cosmology, z_start: f64) -> Vec<(f64, f64)> {
    let a0 = 1.0 / (1.0 + z_start);
    let n = 2000;
    let la0 = m::ln(a0);
    let mut out = vec![];
    let mut t = c.time_of_scale(a0);
    let mut a_prev = a0;
    out.push((t, a0));
    for k in 1..=n {
        let a = m::exp(la0 + (0.2 - la0) * f64::from(k) / f64::from(n));
        t += crate::cosmology::quad(&|x| 1.0 / (x * c.e(x)), a_prev, a, 1) * c.hubble_time_gyr;
        out.push((t, a));
        a_prev = a;
    }
    out
}

pub fn a_of_t(table: &[(f64, f64)], t: f64) -> f64 {
    let i = table
        .partition_point(|r| r.0 <= t)
        .clamp(1, table.len() - 1);
    let (p, q) = (table[i - 1], table[i]);
    p.1 + (q.1 - p.1) * (t - p.0) / (q.0 - p.0)
}

impl GalaxyParams {
    /// Wind mass loading at circular speed v_c (km/s) and redshift z.
    pub fn eta(&self, v_c: f64, z: f64) -> f64 {
        (m::pow(v_c / self.eta_v, -self.eta_slope) * m::pow(1.0 + z, self.eta_z_slope))
            .min(self.eta_max)
    }

    pub fn virial(&self, c: &Cosmology, mh: f64, a: f64) -> (f64, f64) {
        // R_200 against the critical density at a; rho_crit0 in Msun / kpc^3.
        let rho_c = crate::cosmology::RHO_CRIT_H2 * c.h * c.h * 1.0e-9 * c.e(a) * c.e(a);
        let r = m::cbrt(3.0 * mh / (4.0 * m::PI * 200.0 * rho_c));
        (r, m::sqrt(G_KPC * mh / r))
    }
}

/// An authored change to a galaxy's history (galaxy design section 11:
/// "rewrite history in a few clicks"). Each edit acts from its own time
/// onward; everything before the earliest edit is bit-identical to the
/// unedited galaxy (the loop is causal and an edit adds no work before its time).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Edit {
    /// Ink a major merger at a universe time (Gyr) with a mass ratio (smaller over larger, up to 1).
    Merger { t_gyr: f64, ratio: f64 },
    /// Take out one of the halo's own mergers (its index in `halo.mergers`).
    RemoveMerger { index: usize },
    /// A starburst (a bar inflow, a fly-by): this fraction of the gas turns to
    /// disc stars at once, and the black hole feeds as in a merger burst.
    Starburst { t_gyr: f64, gas_fraction: f64 },
    /// The nucleus stays quiet from one time to another: no black hole growth, no active episodes.
    QuietNucleus { from_gyr: f64, to_gyr: f64 },
}

impl Edit {
    /// The earliest time this edit can change anything (Gyr).
    pub fn from_gyr(&self, halo: &Halo) -> f64 {
        match self {
            Edit::Merger { t_gyr, .. } | Edit::Starburst { t_gyr, .. } => *t_gyr,
            Edit::RemoveMerger { index } => {
                halo.mergers.get(*index).map_or(f64::INFINITY, |m| m.t_gyr)
            }
            Edit::QuietNucleus { from_gyr, .. } => *from_gyr,
        }
    }
}

/// Check a list of edits against a halo: ratios in (0, 1], fractions in [0, 1],
/// windows the right way round, merger indices that exist.
pub fn check_edits(halo: &Halo, edits: &[Edit]) -> Result<(), String> {
    for (i, e) in edits.iter().enumerate() {
        let bad = |why: &str| Err(format!("edit {i}: {why}"));
        match e {
            Edit::Merger { t_gyr, ratio } => {
                if !(*ratio > 0.0 && *ratio <= 1.0) || !t_gyr.is_finite() {
                    return bad("a merger needs a finite time and a ratio in (0, 1]");
                }
            }
            Edit::RemoveMerger { index } => {
                if *index >= halo.mergers.len() {
                    return bad("no such merger in this halo");
                }
            }
            Edit::Starburst {
                t_gyr,
                gas_fraction,
            } => {
                if !(0.0..=1.0).contains(gas_fraction) || !t_gyr.is_finite() {
                    return bad("a starburst needs a finite time and a gas fraction in [0, 1]");
                }
            }
            Edit::QuietNucleus { from_gyr, to_gyr } => {
                if from_gyr >= to_gyr || !from_gyr.is_finite() {
                    return bad("a quiet window must end after it starts");
                }
            }
        }
    }
    Ok(())
}

/// Evolve one galaxy in its halo.
pub fn evolve(c: &Cosmology, p: &GalaxyParams, halo: &Halo) -> Galaxy {
    evolve_with(c, p, halo, &[])
}

/// The merger times and ratios a galaxy evolves with: the halo's own, less any
/// removed, plus any inked, in time order.
pub fn mergers_with(halo: &Halo, edits: &[Edit]) -> Vec<(f64, f64)> {
    let removed = |i: usize| {
        edits
            .iter()
            .any(|e| matches!(e, Edit::RemoveMerger { index } if *index == i))
    };
    let mut mergers: Vec<(f64, f64)> = halo
        .mergers
        .iter()
        .enumerate()
        .filter(|(i, _)| !removed(*i))
        .map(|(_, mg)| (mg.t_gyr, mg.ratio))
        .collect();
    for e in edits {
        if let Edit::Merger { t_gyr, ratio } = e {
            mergers.push((*t_gyr, *ratio));
        }
    }
    mergers.sort_by(|a, b| a.0.total_cmp(&b.0));
    mergers
}

/// Evolve one galaxy in its halo with authored edits (call `check_edits` first).
pub fn evolve_with(c: &Cosmology, p: &GalaxyParams, halo: &Halo, edits: &[Edit]) -> Galaxy {
    evolve_feeding(c, p, halo, edits, None)
}

/// As `evolve_with`; with a feed (GH3), the tree's arrivals replace G3's
/// merger and smooth ex-situ stand-ins, and inked mergers keep G3's rule.
pub fn evolve_feeding(
    c: &Cosmology,
    p: &GalaxyParams,
    halo: &Halo,
    edits: &[Edit],
    feed: Option<(&crate::feed::ResponseParams, &[crate::feed::Arrival])>,
) -> Galaxy {
    let table = time_table(c, p.z_start);
    let t0 = table[0].0;
    let now = c.p.now_gyr;
    let dt = p.step_myr / 1000.0;
    let n = m::floor((now - t0) / dt) as u32;
    let fb = c.p.omega_b / c.p.omega_m;
    let (mut gas, mut stars, mut bulge, mut metals, mut bh) =
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64, p.seed_black_hole);
    let (mut ejected, mut ejected_metals) = (0.0_f64, 0.0_f64);
    let mut history = vec![];
    let mut inflow_steps = vec![];
    let mut returned_steps = vec![];
    let mut agn = vec![];
    let mergers = if feed.is_some() {
        let mut inked: Vec<(f64, f64)> = edits
            .iter()
            .filter_map(|e| match e {
                Edit::Merger { t_gyr, ratio } => Some((*t_gyr, *ratio)),
                _ => None,
            })
            .collect();
        inked.sort_by(|a, b| a.0.total_cmp(&b.0));
        inked
    } else {
        mergers_with(halo, edits)
    };
    let mut feeding = feed.map(|(rp, arrivals)| crate::feed::Feeding {
        rp,
        arrivals,
        next: 0,
        out: Default::default(),
        halo_metals: 0.0,
        exsitu_metals: 0.0,
        by_member: Default::default(),
        owed: 0.0,
        next_infall: 0,
    });
    let mut next_merger = 0;
    let mut bursts: Vec<(f64, f64)> = edits
        .iter()
        .filter_map(|e| match e {
            Edit::Starburst {
                t_gyr,
                gas_fraction,
            } => Some((*t_gyr, *gas_fraction)),
            _ => None,
        })
        .collect();
    bursts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut next_burst = 0;
    let quiet = |t: f64| {
        edits.iter().any(
            |e| matches!(e, Edit::QuietNucleus { from_gyr, to_gyr } if t >= *from_gyr && t < *to_gyr),
        )
    };
    let mut state = None;
    for k in 0..n {
        let t = t0 + dt * f64::from(k);
        let a = a_of_t(&table, t);
        let z = 1.0 / a - 1.0;
        let a1 = a_of_t(&table, t + dt);
        let mh = halo.mass_at(z);
        let dmh = (halo.mass_at(1.0 / a1 - 1.0) - mh).max(0.0);
        let (r_vir, v_c) = p.virial(c, mh, a);
        let hot = 1.0 / (1.0 + m::pow(mh / p.hot_mass, p.hot_slope));
        let mut eps = hot;
        eps /= 1.0 + m::pow(v_c / p.prevent_v, -p.prevent_slope);
        if z < p.z_reion && mh < p.filtering_mass {
            eps = 0.0;
        }
        let radio = 1.0
            / (1.0 + m::pow(mh / p.quench_halo, p.quench_halo_slope) * bh / p.quench_black_hole);
        eps *= radio;
        let t_ret = p.return_gamma_gyr * 1.0e10 / mh.max(1.0);
        // Returning gas must cool through the hot halo too.
        let back = ejected * (dt / t_ret).min(1.0) * hot * radio;
        let back_metals = if ejected > 0.0 {
            ejected_metals * back / ejected
        } else {
            0.0
        };
        ejected -= back;
        ejected_metals -= back_metals;
        metals += back_metals;
        let mut inflow = fb * dmh * eps + back;
        if let Some(f) = feeding.as_mut() {
            inflow = f.pay(t + dt, inflow);
        }
        let t_dep = p.depletion_gyr
            * m::pow(1.0 + z, -p.depletion_z_slope)
            * m::pow((v_c / p.depletion_v).min(1.0), -p.depletion_v_slope);
        let eta = p.eta(v_c, z);
        // Implicit step for the gas (stable at any dt): gas' = (gas + inflow) / (1 + dt (1 - R + eta) / t_dep).
        let loss = (1.0 - p.return_fraction + eta) * dt / t_dep;
        let new_gas = (gas + inflow) / (1.0 + loss);
        let sfr_gyr = new_gas / t_dep; // solar masses per Gyr
        let formed = sfr_gyr * dt;
        let zg = if gas + inflow > 0.0 {
            metals / (gas + inflow)
        } else {
            0.0
        };
        metals =
            (metals + p.yield_z * formed - zg * (1.0 - p.return_fraction + eta) * formed).max(0.0);
        ejected += eta * formed;
        ejected_metals += zg * eta * formed;
        gas = new_gas;
        stars += (1.0 - p.return_fraction) * formed;
        if !quiet(t) {
            bh += p.black_hole_steady * formed;
        }
        if feeding.is_none() {
            let smooth = p.exsitu_fraction * p.exsitu_ratio * (1.0 - hot) * dmh;
            stars += smooth;
            bulge += smooth;
            bh += p.exsitu_black_hole * smooth;
        }
        let mut fed_inflow = 0.0;
        if let Some(f) = feeding.as_mut() {
            let mut pools = crate::feed::Pools {
                gas: &mut gas,
                stars: &mut stars,
                bulge: &mut bulge,
                metals: &mut metals,
                bh: &mut bh,
                ejected: &mut ejected,
                ejected_metals: &mut ejected_metals,
                inflow: &mut fed_inflow,
                agn: &mut agn,
            };
            f.step(t + dt, mh, p, &quiet, &mut pools);
        }
        // Major mergers in this step: bulge, burst, black hole.
        while next_merger < mergers.len() && mergers[next_merger].0 < t + dt {
            let (tm, ratio) = mergers[next_merger];
            next_merger += 1;
            if tm < t0 {
                continue;
            }
            let moved = (stars - bulge).max(0.0) * ratio;
            bulge += moved;
            // The incoming galaxy's own stars join the bulge (ex-situ growth). Below the
            // hot-halo mass M* ~ M_h^2, so it brings ratio^2 of the host's stars; in big
            // haloes the smooth ex-situ term already counts the satellites.
            let accreted = stars * m::pow(ratio, 2.0) * hot;
            stars += accreted;
            bulge += accreted;
            bh += p.exsitu_black_hole * accreted;
            let burst = (p.burst_fraction * ratio * gas).min(gas);
            // The burst's gas takes its metals with it into stars.
            if gas > 0.0 {
                metals -= metals * burst / gas;
            }
            gas -= burst;
            let born = (1.0 - p.return_fraction) * burst;
            stars += born;
            bulge += born;
            let feed = p.black_hole_feed * burst;
            if !quiet(tm) {
                bh += feed;
                agn.push((tm, feed));
            }
        }
        // Authored starbursts in this step: disc stars, and the black hole feeds.
        while next_burst < bursts.len() && bursts[next_burst].0 < t + dt {
            let (tb, f) = bursts[next_burst];
            next_burst += 1;
            if tb < t0 {
                continue;
            }
            let burst = f * gas;
            // The burst's gas takes its metals with it into stars.
            if gas > 0.0 {
                metals -= metals * burst / gas;
            }
            gas -= burst;
            stars += (1.0 - p.return_fraction) * burst;
            let feed = p.black_hole_feed * burst;
            if !quiet(tb) {
                bh += feed;
                agn.push((tb, feed));
            }
        }
        bulge = bulge.min(stars);
        let quenched = stars > 0.0 && sfr_gyr / 1.0e9 < p.quenched_ssfr * stars;
        inflow_steps.push(inflow + fed_inflow);
        returned_steps.push(back);
        let disc = p.disc_factor * halo.spin / m::sqrt(2.0) * r_vir;
        let st = GalaxyState {
            t_gyr: t,
            z,
            halo: mh,
            gas,
            stars,
            bulge,
            sfr: sfr_gyr / 1.0e9,
            z_gas: if gas > 0.0 { metals / gas } else { 0.0 },
            black_hole: bh,
            quenched,
            v_c,
            disc_kpc: disc,
        };
        if k % p.history_every == 0 || k + 1 == n {
            history.push(st.clone());
        }
        state = Some(st);
    }
    let now_state = state.expect("at least one step");
    // The rotation curve now (DM4), and its peak for the bar test.
    let structure = crate::structure::Structure::new(
        c,
        &p.structure,
        &|z| halo.mass_at(z),
        &now_state,
        1.0 / (1.0 + now_state.z),
    );
    let rotation = crate::structure::curve_table(&structure);
    // Efstathiou, Lake and Negroponte 1982: v_max / sqrt(G M_d / R_d) below about 1.1 is bar-unstable.
    let barred = bar_unstable(p, &rotation, &now_state);
    // When the bar formed: the earliest history point from which the disc
    // has been bar-unstable ever since (bars, once formed, are long-lived
    // and are not destroyed by the disc's own gas, so the run back stops at
    // the first stable point). The rings sweep the bar's gas from then on.
    let bar_from_gyr = if barred {
        let mut from = now_state.t_gyr;
        for st in history.iter().rev() {
            let s = crate::structure::Structure::new(
                c,
                &p.structure,
                &|z| halo.mass_at(z),
                st,
                1.0 / (1.0 + st.z),
            );
            if !bar_unstable(p, &crate::structure::curve_table(&s), st) {
                break;
            }
            from = st.t_gyr;
        }
        Some(from)
    } else {
        None
    };
    let morphology = classify(&now_state, barred);
    Galaxy {
        halo: halo.address.clone(),
        history,
        now: now_state,
        morphology,
        barred,
        bar_from_gyr,
        active_nuclei: agn,
        inflow: inflow_steps,
        returned: returned_steps,
        step_gyr: dt,
        t_start_gyr: t0,
        rotation,
        assembled: feeding.map(crate::feed::Feeding::finish),
        ejected_now: ejected,
    }
}

/// The Efstathiou, Lake and Negroponte (1982) test on a galaxy state and its
/// rotation curve: v_max / sqrt(G M_disc / R_d) below the threshold, for a
/// disc of more than 1e9 solar masses of stars.
fn bar_unstable(p: &GalaxyParams, rotation: &[(f64, f64)], s: &GalaxyState) -> bool {
    let v_max = rotation.iter().map(|e| e.1).fold(0.0, f64::max);
    let disc_mass = (s.stars - s.bulge) + s.gas;
    let enl = v_max / m::sqrt(G_KPC * disc_mass.max(1.0) / s.disc_kpc.max(1.0e-3));
    enl < p.bar_threshold && s.stars > 1.0e9
}

/// Morphology as a rule over the numbers (pack-adjustable later).
pub fn classify(s: &GalaxyState, barred: bool) -> Morphology {
    let bt = if s.stars > 0.0 {
        s.bulge / s.stars
    } else {
        0.0
    };
    let ssfr = if s.stars > 0.0 { s.sfr / s.stars } else { 0.0 };
    if s.stars < 1.0e9 {
        if ssfr > 1.0e-10 {
            Morphology::Irregular
        } else {
            Morphology::Dwarf
        }
    } else if bt > 0.7 {
        Morphology::Elliptical
    } else if ssfr < 1.0e-11 {
        Morphology::Lenticular
    } else if barred {
        Morphology::Barred
    } else {
        Morphology::Disc
    }
}

/// Write the galaxy's active-nucleus episodes to the chronicle as dated
/// `galaxy.active_nucleus` events on the halo, in time order.
pub fn record(
    ch: &mut sse_chronicle::Chronicle,
    g: &Galaxy,
) -> Result<Vec<sse_core::ContentHash>, String> {
    use sse_chronicle::{Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
    use sse_core::cbor::Value;
    let mut eps = g.active_nuclei.clone();
    eps.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut ids = vec![];
    for (t, gained) in eps {
        let at = UniverseTime::seconds(m::round(t * 1.0e9 * YEAR_S) as i64);
        let mut data = std::collections::BTreeMap::new();
        data.insert("black_hole_gain".into(), Value::Float(gained));
        data.insert(
            "why".into(),
            Value::Text(format!(
                "a major merger at {t:.2} Gyr drives gas to the centre; the black hole gains {gained:.1e} solar masses and shines as an active nucleus"
            )),
        );
        let e = Event::new(EventBody {
            kind: "galaxy.active_nucleus".into(),
            subject: vec![g.halo.parse().map_err(|e| format!("{e:?}"))?],
            occurred_at: at,
            activated_at: at,
            recognised_at: ch.next_stamp(),
            causes: vec![],
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Derived,
            publisher: Publisher::new("sse.base-cosmos", "galaxy", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        ids.push(ch.append(e).map_err(|e| format!("{e:?}"))?);
    }
    Ok(ids)
}
