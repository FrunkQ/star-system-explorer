//! One generated system, its reservoirs split into classes, the time bins,
//! the sinks and the close passes. Everything here is fixed by the seed.
//!
//! STAND-INS (prototype only): the class shares, drain laws, sink shares,
//! travel times, pass rate and kick strength are plausible round numbers,
//! not calibrated. They are what design item 6 ("one short spec per split")
//! should replace. The mechanism under test does not depend on their values.

use crate::draw::poisson;
use crate::perm::{mix, unit};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};
use sse_formation::leftovers::{Cloud, Leftovers};
use sse_formation::violent::{Upheaval, UpheavalEvent};

pub const MASS_UNIT: f64 = sse_formation::disc::MASS_UNIT; // Msun per system unit
/// One book sub-unit: 1e-21 Msun, about 2e9 kg (a 100 m boulder). The
/// formation unit (1e-12 Msun, about 2e18 kg) is too coarse to hold a
/// single 10 km body.
pub const SUB_PER_UNIT: u64 = 1_000_000_000;
pub const M_SUN: f64 = sse_core::sheet::M_SUN;
pub const SUB_KG: f64 = MASS_UNIT * M_SUN / SUB_PER_UNIT as f64;
const M_EARTH: f64 = sse_core::sheet::M_EARTH;
const R_EARTH_M: f64 = sse_core::sheet::R_EARTH_MEAN;
const G: f64 = sse_core::sheet::G;
const SLOPE: f64 = sse_formation::leftovers::SLOPE;
const ZETA: f64 = sse_formation::leftovers::ZETA_1_2;
const DENSITY: [f64; 6] = [7_800.0, 3_000.0, 2_000.0, 1_000.0, 1_500.0, 900.0];
pub const GROUPS: [&str; 6] = ["metal", "rock", "carbon", "water", "co2", "co"];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SinkKind {
    Star,
    Ejected,
    World(usize),
}

#[derive(Clone, Debug)]
pub struct WorldInfo {
    pub id: String,
    pub a_au: f64,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub surface: bool,
    pub v_esc_kms: f64,
    pub g: f64,
}

#[derive(Clone, Debug)]
pub struct Pass {
    pub id: String,
    pub t_myr: f64,
    pub b_pc: f64,
    pub v_kms: f64,
    pub mass_msun: f64,
    /// Share of a comet cloud's members sent inward.
    pub kick: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum Travel {
    /// Log-uniform delivery time, Myr.
    LogUniform(f64, f64),
    /// Half the member's own orbital period (a comet falling from aphelion).
    HalfPeriod,
}

#[derive(Clone, Debug)]
pub struct ClassDef {
    pub name: String,
    pub what: String,
    pub groups: [u64; 6],
    pub total_sub: u64,
    pub cloud: Cloud,
    pub n_roster: u64,
    pub n_exact: u64,
    /// Rank ranges [k0, k1), decade bands from the largest.
    pub bands: Vec<(u64, u64)>,
    pub key: u64,
    pub density: f64,
}

#[derive(Clone, Debug)]
pub struct Reservoir {
    pub idx: usize,
    pub name: String,
    pub kind: String,
    pub cloud_address: String,
    pub t0_myr: f64,
    pub classes: Vec<ClassDef>,
    pub fast: (f64, f64, f64), // (slow share, tau fast, tau slow), Myr
    pub kicked: bool,
    pub sinks: Vec<(SinkKind, f64)>,
    pub v_inf_kms: f64,
    pub travel: Travel,
    pub origin: String,
    pub loosened_by: String,
}

pub struct System {
    pub seed: Seed,
    pub address: Address,
    pub age_myr: f64,
    pub horizon_myr: f64,
    pub edges: Vec<f64>,
    pub worlds: Vec<WorldInfo>,
    pub reservoirs: Vec<Reservoir>,
    pub passes: Vec<Pass>,
    pub star_msun: f64,
}

// ---- the size law, as formation's (private there) ------------------------

fn zeta_tail(top: u64) -> f64 {
    let s = 3.0 / SLOPE;
    if top <= 1000 {
        let mut head = 0.0;
        for k in 1..=top {
            head += m::pow(k as f64, -s);
        }
        return ZETA - head;
    }
    let k = top as f64;
    m::pow(k, 1.0 - s) / (s - 1.0) - 0.5 * m::pow(k, -s) - s / 12.0 * m::pow(k, -s - 1.0)
}

fn weight_sum(n: f64) -> f64 {
    let top = m::floor(n) as u64;
    top as f64 + m::pow(n, 3.0 / SLOPE) * zeta_tail(top)
}

pub fn weight(k: u64, n: f64) -> f64 {
    let x = k as f64 / n;
    if x <= 1.0 {
        1.0
    } else {
        m::pow(x, -3.0 / SLOPE)
    }
}

/// Sum of the weights of ranks 1..=k (closed form above formation's top).
pub fn weight_upto(k: u64, n: f64) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let top = m::floor(n) as u64;
    if k <= top {
        return k as f64;
    }
    // ranks top+1..=k: n^1.2 (tail(top) - tail(k))
    top as f64 + m::pow(n, 3.0 / SLOPE) * (zeta_tail(top) - zeta_tail(k))
}

fn fit_top(total_kg: f64, cap_kg: f64) -> (f64, f64) {
    if total_kg / ZETA <= cap_kg {
        return (1.0, ZETA);
    }
    let (mut lo, mut hi) = (1.0, total_kg / cap_kg);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if total_kg / weight_sum(mid) > cap_kg {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (hi, weight_sum(hi))
}

fn density(solid: &[u64; 6]) -> f64 {
    let t: u64 = solid.iter().sum();
    if t == 0 {
        return 2_000.0;
    }
    let inv: f64 = (0..6)
        .map(|g| solid[g] as f64 / t as f64 / DENSITY[g])
        .sum();
    1.0 / inv
}

/// Split integer budgets by shares exactly (largest remainder, ties to the
/// lower index): the parts always add back to `total`.
pub fn split_exact(total: u64, shares: &[f64]) -> Vec<u64> {
    let s: f64 = shares.iter().sum();
    if total == 0 || s <= 0.0 {
        return vec![0; shares.len()];
    }
    let raw: Vec<f64> = shares.iter().map(|x| total as f64 * x / s).collect();
    let mut out: Vec<u64> = raw.iter().map(|x| m::floor(*x) as u64).collect();
    let used: u64 = out.iter().sum();
    let mut left = total.saturating_sub(used);
    if break_split() {
        left = 0; // red control: drop the remainder, as a plain floor would
    }
    let mut order: Vec<usize> = (0..shares.len()).collect();
    order.sort_by(|a, b| {
        let fa = raw[*a] - m::floor(raw[*a]);
        let fb = raw[*b] - m::floor(raw[*b]);
        fb.total_cmp(&fa).then(a.cmp(b))
    });
    for i in order {
        if left == 0 {
            break;
        }
        out[i] += 1;
        left -= 1;
    }
    // Rounding can only overshoot through float error; take it back from the largest.
    let mut sum: u64 = out.iter().sum();
    while sum > total {
        let i = (0..out.len()).max_by_key(|i| out[*i]).unwrap();
        out[i] -= 1;
        sum -= 1;
    }
    out
}

fn break_split() -> bool {
    static B: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *B.get_or_init(|| std::env::var("CC_BREAK").is_ok_and(|v| v == "split"))
}

// ---- class shares (STAND-IN for design item 6) ----------------------------

/// (name, what, share of each element group). Each group's shares across a
/// cloud's classes add to 1, so the class books add back to the cloud's.
fn class_shares(kind: &str, outcome: Option<&str>) -> Vec<(&'static str, &'static str, [f64; 6])> {
    match kind {
        "debris-belt" => {
            let core = if outcome == Some("shattered") {
                0.92
            } else {
                0.25
            };
            let mantle = 1.0 - core - 0.01;
            vec![
                (
                    "core",
                    "a fragment of the parent's iron core",
                    [core, 0.02, 0.0, 0.0, 0.0, 0.0],
                ),
                (
                    "mantle",
                    "a piece of the parent's rocky mantle",
                    [mantle, 0.83, 0.30, 0.30, 0.30, 0.30],
                ),
                (
                    "crust",
                    "a shard of the parent's crust",
                    [0.01, 0.15, 0.70, 0.70, 0.70, 0.70],
                ),
            ]
        }
        "oort" | "scattered-disc" => vec![
            (
                "icy",
                "an icy planetesimal",
                [0.60, 0.60, 0.80, 0.97, 0.97, 0.97],
            ),
            (
                "rocky",
                "a rocky planetesimal scattered out with the ices",
                [0.40, 0.40, 0.20, 0.03, 0.03, 0.03],
            ),
        ],
        _ => vec![
            (
                "melted",
                "a piece of a small world melted early by aluminium-26",
                [0.15, 0.10, 0.0, 0.0, 0.0, 0.0],
            ),
            (
                "primitive",
                "a primitive body that never melted",
                [0.60, 0.65, 0.50, 0.10, 0.10, 0.10],
            ),
            (
                "icy",
                "an icy body from beyond the snow line",
                [0.25, 0.25, 0.50, 0.90, 0.90, 0.90],
            ),
        ],
    }
}

fn split_classes(solid: &[u64; 6], shares: &[[f64; 6]]) -> Vec<[u64; 6]> {
    let mut out = vec![[0u64; 6]; shares.len()];
    for g in 0..6 {
        let col: Vec<f64> = shares.iter().map(|s| s[g]).collect();
        for (c, v) in split_exact(solid[g], &col).into_iter().enumerate() {
            out[c][g] = v;
        }
    }
    out
}

pub struct Settings {
    /// Largest members per class whose books are summed member by member.
    pub n_exact: u64,
    /// Members down to this diameter have names and fates; smaller is gravel.
    pub roster_min_km: f64,
    pub roster_cap: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            n_exact: 100_000,
            roster_min_km: 1.0,
            roster_cap: 1_000_000_000_000,
        }
    }
}

fn world_info(w: &sse_formation::violent::World) -> WorldInfo {
    let mass_kg = w.mass_earth() * M_EARTH;
    let radius_m = w.radius_earth() * R_EARTH_M;
    let surface = !w.is_giant() && w.gas_earth() < 1e-3 * w.mass_earth();
    WorldInfo {
        id: w.id.clone(),
        a_au: w.a_au,
        mass_kg,
        radius_m,
        surface,
        v_esc_kms: m::sqrt(2.0 * G * mass_kg / radius_m) / 1000.0,
        g: G * mass_kg / (radius_m * radius_m),
    }
}

fn impact_of(u: &Upheaval, world: &str) -> Option<(f64, String, String)> {
    u.events.iter().find_map(|e| match e {
        UpheavalEvent::Impact {
            t_myr,
            target,
            impactor,
            outcome,
            ..
        } if target == world => Some((*t_myr, impactor.clone(), outcome.clone())),
        _ => None,
    })
}

/// Kroupa (2001) stellar mass quantile, 0.08 to 20 Msun.
fn imf_quantile(u: f64) -> f64 {
    let (a1, a2, lo, mid, hi) = (1.3, 2.3, 0.08, 0.5, 20.0);
    let seg = |a: f64, x0: f64, x1: f64| (m::pow(x1, 1.0 - a) - m::pow(x0, 1.0 - a)) / (1.0 - a);
    let k2 = m::pow(mid, a2 - a1); // continuity at 0.5
    let w1 = seg(a1, lo, mid);
    let w2 = k2 * seg(a2, mid, hi);
    let t = u * (w1 + w2);
    if t < w1 {
        m::pow(m::pow(lo, 1.0 - a1) + t * (1.0 - a1), 1.0 / (1.0 - a1))
    } else {
        m::pow(
            m::pow(mid, 1.0 - a2) + (t - w1) / k2 * (1.0 - a2),
            1.0 / (1.0 - a2),
        )
    }
}

impl System {
    pub fn bin_of(&self, t: f64) -> usize {
        match self.edges.binary_search_by(|e| e.total_cmp(&t)) {
            Ok(i) => i.min(self.edges.len() - 2),
            Err(i) => i.saturating_sub(1).min(self.edges.len() - 2),
        }
    }
    pub fn nbins(&self) -> usize {
        self.edges.len() - 1
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(
        seed: Seed,
        address: Address,
        u: &Upheaval,
        l: &Leftovers,
        age_myr: f64,
        star_msun: f64,
        st: &Settings,
    ) -> System {
        let horizon = age_myr + 1000.0;
        let mut edges = vec![];
        let mut t = 0.0;
        while t < 100.0 {
            edges.push(t);
            t += 1.0;
        }
        while t < horizon {
            edges.push(t);
            t += 10.0;
        }
        edges.push(horizon);

        let worlds: Vec<WorldInfo> = l.worlds.iter().map(world_info).collect();
        let giant = l
            .worlds
            .iter()
            .filter(|w| w.is_giant())
            .max_by(|a, b| a.mass_earth().total_cmp(&b.mass_earth()))
            .map(|w| w.id.clone());

        // Close passes inside the shower distance, keyed by the system's address.
        // STAND-IN for sse-cosmos events.rs's encounter series (same form: a keyed
        // Poisson count per 10 Myr bin, b uniform over the target area, Maxwellian
        // speeds, IMF masses) at a fixed solar-neighbourhood rate: about 20
        // passes per Myr within 1 pc (Bailer-Jones et al. 2018), so 0.05 per Myr
        // within 0.05 pc.
        let pd = Draws::new(
            seed,
            address.clone(),
            Stream::new("cc.proto.passes.v0").unwrap(),
        );
        let pkey = pd.u64(0);
        let (b_max, rate, v_mean) = (0.05, 20.0 * 0.05 * 0.05, 26.0);
        let s1 = v_mean / m::sqrt(8.0 / m::PI);
        let mut passes = vec![];
        let mut bin = 0u64;
        while (bin as f64) * 10.0 < horizon {
            let n = poisson(rate * 10.0, pkey, bin);
            for j in 0..n {
                let k = mix(pkey ^ (bin << 20) ^ j);
                let tm = (bin as f64 + unit(k, 0)) * 10.0;
                let b = b_max * m::sqrt(unit(k, 1));
                let (x, y, z) = (
                    crate::draw::normal(k, 2),
                    crate::draw::normal(k, 3),
                    crate::draw::normal(k, 4),
                );
                let v = (s1 * m::sqrt(x * x + y * y + z * z)).max(1.0);
                let mass = imf_quantile(unit(k, 5));
                let dose = mass / (b.max(1e-4) * v);
                passes.push(Pass {
                    id: format!("{address}/encounter/{bin}/{j}"),
                    t_myr: tm,
                    b_pc: b,
                    v_kms: v,
                    mass_msun: mass,
                    kick: (0.003 * dose).min(0.2),
                });
            }
            bin += 1;
        }
        passes.sort_by(|a, b| a.t_myr.total_cmp(&b.t_myr));

        let mut reservoirs = vec![];
        for (ci, c) in l.clouds.iter().enumerate() {
            let kind = c.kind.as_str();
            if !matches!(
                kind,
                "debris-belt" | "outer-belt" | "asteroid-belt" | "belt" | "oort" | "scattered-disc"
            ) {
                continue;
            }
            let belt = u.belts.iter().find(|b| b.id == c.source);
            let parent = belt.and_then(|b| b.origin.strip_prefix("impact:").map(|s| s.to_string()));
            let imp = parent.as_ref().and_then(|p| impact_of(u, p));
            let t0 = match kind {
                "debris-belt" => imp.as_ref().map(|x| x.0).unwrap_or(0.0),
                "oort" | "scattered-disc" => u
                    .events
                    .iter()
                    .find_map(|e| match e {
                        UpheavalEvent::BeltStirred { t_myr, belt, .. } if *belt == c.source => {
                            Some(*t_myr)
                        }
                        _ => None,
                    })
                    .unwrap_or(0.0),
                _ => 0.0,
            };
            let origin = match (kind, &imp, &parent) {
                ("debris-belt", Some((t, by, out)), Some(p)) => {
                    format!(
                        "debris of {p}, which {by} {} {t:.1} million years after the system formed",
                        if out == "shattered" {
                            "shattered"
                        } else {
                            "struck"
                        }
                    )
                }
                ("oort", _, _) | ("scattered-disc", _, _) => format!(
                    "thrown out of {} by {} {:.1} million years after the system formed",
                    c.source,
                    giant.clone().unwrap_or("a giant".into()),
                    t0
                ),
                _ => format!("part of {}, formed with the system", c.source),
            };
            let (fast, kicked, v_inf, travel, loosened_by, mut sinks): (
                _,
                _,
                _,
                _,
                String,
                Vec<(SinkKind, f64)>,
            ) = match kind {
                "debris-belt" => (
                    (0.01, 15.0, 1500.0),
                    false,
                    5.0,
                    Travel::LogUniform(0.1, 5.0),
                    "the gravity of the neighbouring worlds".into(),
                    vec![(SinkKind::Star, 0.3), (SinkKind::Ejected, 0.2)],
                ),
                "oort" => (
                    (1.0, 1.0, 10_000.0),
                    true,
                    45.0,
                    Travel::HalfPeriod,
                    "the galaxy's tide".into(),
                    vec![(SinkKind::Star, 0.01)],
                ),
                "scattered-disc" => (
                    (0.3, 100.0, 5000.0),
                    false,
                    20.0,
                    Travel::LogUniform(1.0, 100.0),
                    format!(
                        "close passes of {}",
                        giant.clone().unwrap_or("a giant".into())
                    ),
                    vec![(SinkKind::Star, 0.05), (SinkKind::Ejected, 0.6)],
                ),
                _ => (
                    (0.2, 30.0, 4000.0),
                    false,
                    10.0,
                    Travel::LogUniform(0.5, 30.0),
                    format!(
                        "the resonances of {}",
                        giant.clone().unwrap_or("a giant".into())
                    ),
                    vec![(SinkKind::Star, 0.25), (SinkKind::Ejected, 0.6)],
                ),
            };
            // World shares: cross-section with gravitational focusing.
            let a_mid = 0.5 * (c.a_min_au + c.a_max_au);
            let wsum_target: f64 = match kind {
                "debris-belt" => 0.5,
                "oort" => 0.0, // absolute, below
                "scattered-disc" => 0.35,
                _ => 0.15,
            };
            let cross = |w: &WorldInfo| {
                let r = w.radius_m / R_EARTH_M;
                let f = (1.0 + (w.v_esc_kms / v_inf) * (w.v_esc_kms / v_inf)).min(10.0);
                r * r * f
            };
            if kind == "oort" {
                // Per comet entering the planetary region: about 2e-9 for the Earth
                // (STAND-IN, order of Weissman 1990), as cross-section over distance.
                let mut used = 0.01;
                for (i, w) in worlds.iter().enumerate() {
                    let p = 2e-9 * cross(w) / 1.062 / w.a_au.max(0.05);
                    sinks.push((SinkKind::World(i), p));
                    used += p;
                }
                sinks.push((SinkKind::Ejected, 1.0 - used));
            } else {
                let near = |w: &WorldInfo| {
                    let x = m::ln(w.a_au / a_mid).abs();
                    m::exp(-x / 0.25)
                };
                let ws: Vec<f64> = worlds.iter().map(|w| cross(w) * near(w)).collect();
                let tot: f64 = ws.iter().sum::<f64>().max(1e-300);
                for (i, wv) in ws.iter().enumerate() {
                    sinks.push((SinkKind::World(i), wsum_target * wv / tot));
                }
            }
            let outcome = imp.as_ref().map(|x| x.2.clone());
            let shares = class_shares(kind, outcome.as_deref());
            let parts = split_classes(&c.solid, &shares.iter().map(|s| s.2).collect::<Vec<_>>());
            let mut classes = vec![];
            for ((name, what, _), groups) in shares.iter().zip(parts) {
                let total: u64 = groups.iter().sum();
                if total == 0 {
                    continue;
                }
                let caddr = c.address.child_key("class", name).unwrap();
                let rho = density(&groups);
                let total_kg = total as f64 * MASS_UNIT * M_SUN;
                let cap_kg = rho * m::PI / 6.0 * m::pow(2400.0e3, 3.0);
                let (n_top, norm) = fit_top(total_kg, cap_kg);
                let m1 = total_kg / norm;
                let largest_km = m::cbrt(6.0 * m1 / (m::PI * rho)) / 1000.0;
                let mut cc = c.clone();
                cc.address = caddr.clone();
                cc.solid = groups;
                cc.n_top = n_top;
                cc.norm = norm;
                cc.largest_km = largest_km;
                let over = n_top * m::pow((largest_km / st.roster_min_km).max(1.0), SLOPE);
                cc.over_1km = n_top * m::pow(largest_km.max(1.0), SLOPE);
                let n_roster = (m::floor(over) as u64).clamp(1, st.roster_cap);
                let n_exact = st.n_exact.min(n_roster);
                let mut bands = vec![(1u64, 2u64)];
                let mut lo = 2u64;
                let mut hi = 11u64;
                while lo <= n_roster {
                    bands.push((lo, hi.min(n_roster + 1)));
                    lo = hi;
                    hi = (hi - 1) * 10 + 1;
                }
                let key = Draws::new(
                    seed,
                    caddr.clone(),
                    Stream::new("cc.proto.roster.v0").unwrap(),
                )
                .u64(0);
                classes.push(ClassDef {
                    name: name.to_string(),
                    what: what.to_string(),
                    groups,
                    total_sub: total * SUB_PER_UNIT,
                    cloud: cc,
                    n_roster,
                    n_exact,
                    bands,
                    key,
                    density: rho,
                });
            }
            reservoirs.push(Reservoir {
                idx: reservoirs.len(),
                name: format!("{}-{}", kind, ci),
                kind: kind.to_string(),
                cloud_address: c.address.to_string(),
                t0_myr: t0,
                classes,
                fast,
                kicked,
                sinks,
                v_inf_kms: v_inf,
                travel,
                origin,
                loosened_by,
            });
        }
        System {
            seed,
            address,
            age_myr,
            horizon_myr: horizon,
            edges,
            worlds,
            reservoirs,
            passes,
            star_msun,
        }
    }

    /// Background survival of a reservoir at t (passes not included).
    pub fn survival(&self, r: &Reservoir, t: f64) -> f64 {
        if t <= r.t0_myr {
            return 1.0;
        }
        let x = t - r.t0_myr;
        let (fs, tf, ts) = r.fast;
        (1.0 - fs) * m::exp(-x / tf) + fs * m::exp(-x / ts)
    }

    pub fn passes_in_bin(&self, b: usize) -> impl Iterator<Item = (usize, &Pass)> {
        let (lo, hi) = (self.edges[b], self.edges[b + 1]);
        self.passes
            .iter()
            .enumerate()
            .filter(move |(_, p)| p.t_myr >= lo && p.t_myr < hi)
    }
}
