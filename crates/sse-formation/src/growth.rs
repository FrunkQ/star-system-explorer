//! Stages 4 and 5, the modern growth block (F3): planetesimal rings, pebble
//! growth, gas capture, gaps, migration and resonant chains, stepped in time
//! on the F1 disc. Standard level: published rate laws per step, keyed draws
//! where outcomes are chaotic.
//!
//! Budgets: every unit of solid and gas comes from an F1 band and goes
//! somewhere (a body, a ring, sublimated, into the star), in integer budget
//! units, so the books close exactly. Each body's ledger lists every claim:
//! when, from which band, how much of each element group.

use crate::disc::{Disc, GROUPS, MASS_UNIT};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

pub const REF_LJ14: &str = "Lambrechts, M. and Johansen, A. 2014, A&A 572, A107: pebble flux and accretion, 2D Hill rate 2 (St/0.1)^(2/3) Omega R_H^2 Sigma_p";
pub const REF_BITSCH18: &str = "Bitsch, B. et al. 2018, A&A 612, A30: pebble isolation mass 25 Earth masses (h/0.05)^3 (0.34 (log 0.001 / log alpha)^4 + 0.66) (eqs. 10-11) plus the diffusion term alpha/(2 St)/lambda (eq. 26); the (M/M_sun) scaling is an extrapolation. arXiv:1801.02341";
pub const REF_DA17: &str = "Drazkowska, J. and Alibert, Y. 2017, A&A 608, A92: planetesimals form first at the water ice line";
pub const REF_LC15: &str = "Lee, E. J. and Chiang, E. 2015, ApJ 811, 41: gas captured by cores in the disc, GCR rising as t^0.4 M^1.7";
pub const REF_IKOMA00: &str = "Ikoma, M., Nakazawa, K. and Emori, H. 2000, ApJ 537, 1013; Ida, S. and Lin, D. N. C. 2004, ApJ 604, 388: Kelvin-Helmholtz runaway, tau = 10^k1 yr (M/M_earth)^-3 with k1 8 to 10; SSE4 uses 1e11 (KH_COEFF_YR, tuned, outside the published range)";
pub const REF_TT16: &str = "Tanigawa, T. and Tanaka, H. 2016, ApJ 823, 48: gas accretion through a gap, 0.29 h^-2 q^(4/3) Sigma_gap r^2 Omega";
pub const REF_LD06: &str = "Lubow, S. H. and D'Angelo, G. 2006, ApJ 641, 526: a gap-opening giant takes up to about 80 percent of the disc's accretion flow";
pub const REF_TANAKA02: &str = "Tanaka, H., Takeuchi, T. and Ward, W. R. 2002, ApJ 565, 1257: type I migration rate (2.7 + 1.1 beta) q (Sigma r^2 / M) h^-2 Omega, reduced by C_I = 0.5 (TYPE_I_REDUCTION; Ida and Lin 2008, ApJ 673, 487, explore a range of C_I)";
pub const REF_KANAGAWA18: &str = "Kanagawa, K. D., Tanaka, H. and Szuszkiewicz, E. 2018, ApJ 861, 140: migration in a gap is type I divided by 1 + 0.04 K, K = q^2 h^-5 / alpha";
pub const REF_IZIDORO17: &str = "Izidoro, A. et al. 2017, MNRAS 470, 1750; Izidoro et al. 2021, A&A 650, A152: migration piles super-Earths into resonant chains at the disc's inner edge";
pub const REF_HP11: &str = "Hasegawa, Y. and Pudritz, R. E. 2011, MNRAS 417, 1236: planet traps at the ice line, heat transition and dead-zone edge";
pub const REF_LC17: &str = "Lee, E. J. and Chiang, E. 2017, ApJ 842, 40: magnetospheric truncation of the disc explains the fall-off of Kepler planets inside about 10-day orbits";
pub const REF_GLADMAN93: &str = "Gladman, B. 1993, Icarus 106, 247: two orbits closer than 2 sqrt(3) mutual Hill radii cannot stay apart";

const G: f64 = sse_core::sheet::G;
const K_B: f64 = sse_core::sheet::K_B;
const M_H: f64 = 1.673_5e-27;
const MU: f64 = 2.34;
const AU_M: f64 = sse_core::sheet::AU;
const MSUN_KG: f64 = sse_core::sheet::M_SUN;
const YR_S: f64 = sse_core::sheet::YR_JULIAN;
pub const MSUN_IN_EARTH: f64 = 332_946.0;

/// Pebble flux as a fraction of the gas accretion flow, per unit of solid mass
/// fraction (Lambrechts and Johansen 2014 find pebble fluxes of about 100 Earth
/// masses a Myr when the gas flows at about 1e-8 solar masses a year).
pub const PEBBLE_FLUX_PER_Z: f64 = 1.0;
/// Stokes number of drifting pebbles.
pub const STOKES: f64 = 0.1;
/// Type I reduction (Ida and Lin 2008's C_I).
pub const TYPE_I_REDUCTION: f64 = 0.5;
/// Extra slowing of gap-opening giants, calibrated on the observed piling of
/// giants beyond 1 AU and the rarity of hot Jupiters (about 1 percent, Wright et al. 2012).
pub const TYPE_II_REDUCTION: f64 = 0.02;
/// Masses held by the water-line trap, Earth masses (from a few Earth masses until the core opens a gap).
pub const TRAP_MASS: (f64, f64) = (3.0, 15.0);
/// Share of the disc flow a gap-opening giant can take.
pub const GIANT_FLOW_SHARE: f64 = 0.8;
/// Steps in log time from the first planetesimals to the disc's end.
pub const STEPS: usize = 80;
pub const T_START_MYR: f64 = 0.05;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub t_myr: f64,
    /// F1 band key the material came from.
    pub band: String,
    pub solid: [u64; 6],
    pub gas: u64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum GrowthEntry {
    Claim(Claim),
    Merged { body: Box<GrowthLedger> },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GrowthLedger {
    pub id: String,
    pub entries: Vec<GrowthEntry>,
}

impl GrowthLedger {
    pub fn solid(&self) -> [u64; 6] {
        let mut s = [0u64; 6];
        for e in &self.entries {
            let x = match e {
                GrowthEntry::Claim(c) => c.solid,
                GrowthEntry::Merged { body } => body.solid(),
            };
            for g in 0..6 {
                s[g] += x[g];
            }
        }
        s
    }
    pub fn gas(&self) -> u64 {
        self.entries
            .iter()
            .map(|e| match e {
                GrowthEntry::Claim(c) => c.gas,
                GrowthEntry::Merged { body } => body.gas(),
            })
            .sum()
    }
    /// Solid units by source band, over the whole tree, in band order of first appearance.
    pub fn by_band(&self, out: &mut Vec<(String, u64)>) {
        for e in &self.entries {
            match e {
                GrowthEntry::Claim(c) => {
                    let s: u64 = c.solid.iter().sum();
                    match out.iter_mut().find(|(k, _)| *k == c.band) {
                        Some(x) => x.1 += s,
                        None => out.push((c.band.clone(), s)),
                    }
                }
                GrowthEntry::Merged { body } => body.by_band(out),
            }
        }
    }
    pub fn last_claim(&self) -> f64 {
        let mut t: f64 = 0.0;
        for e in &self.entries {
            t = t.max(match e {
                GrowthEntry::Claim(c) => c.t_myr,
                GrowthEntry::Merged { body } => body.last_claim(),
            });
        }
        t
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GrownBody {
    pub id: String,
    pub born_myr: f64,
    pub born_au: f64,
    pub born_band: String,
    pub a_au: f64,
    pub e: f64,
    pub solid: [u64; 6],
    pub gas: u64,
    pub ledger: GrowthLedger,
    /// Reached pebble isolation (stopped the pebble flow inward).
    pub isolated: bool,
    pub opened_gap: bool,
    /// Index (in `bodies`) of the inner neighbour it is held in resonance with, and the ratio p:q.
    pub resonance: Option<(usize, u32, u32)>,
}

impl GrownBody {
    pub fn solid_earth(&self) -> f64 {
        self.solid.iter().sum::<u64>() as f64 * MASS_UNIT * MSUN_IN_EARTH
    }
    pub fn gas_earth(&self) -> f64 {
        self.gas as f64 * MASS_UNIT * MSUN_IN_EARTH
    }
    pub fn mass_earth(&self) -> f64 {
        self.solid_earth() + self.gas_earth()
    }
    pub fn is_giant(&self) -> bool {
        self.gas_earth() > self.solid_earth() && self.mass_earth() > 30.0
    }
}

/// Where the solids went: initial + condensed = bands + bodies + rings + sublimated + star, per group, exactly;
/// and the gas: initial = bands + bodies.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GrowthBudget {
    pub solid_initial: [u64; 6],
    /// Band vapour that condensed as the disc cooled (added to the solids).
    pub solid_condensed: [u64; 6],
    pub solid_in_bands: [u64; 6],
    pub solid_in_bodies: [u64; 6],
    /// Pebbles held up outside isolated planets: the rings F5 turns into belts.
    pub solid_in_rings: [u64; 6],
    /// Ices that turned to vapour as pebbles drifted inside their lines.
    pub solid_sublimated: [u64; 6],
    /// Pebbles that drifted past every body into the star.
    pub solid_to_star: [u64; 6],
    pub gas_initial: u64,
    pub gas_in_bands: u64,
    pub gas_in_bodies: u64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ring {
    /// Just outside the body that holds it up.
    pub a_au: f64,
    pub held_by: String,
    pub solid: [u64; 6],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum GrowthEvent {
    EmbryoForms {
        t_myr: f64,
        id: String,
        a_au: f64,
        band: String,
    },
    PebbleIsolation {
        t_myr: f64,
        id: String,
        mass_earth: f64,
    },
    RunawayGas {
        t_myr: f64,
        id: String,
        core_earth: f64,
    },
    GapOpens {
        t_myr: f64,
        id: String,
    },
    CapturedInResonance {
        t_myr: f64,
        id: String,
        inner: String,
        p: u32,
        q: u32,
    },
    EmbryosMerge {
        t_myr: f64,
        into: String,
        from: [String; 2],
    },
    ParkedAtInnerEdge {
        t_myr: f64,
        id: String,
        a_au: f64,
    },
    GasGone {
        t_myr: f64,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Growth {
    /// Sorted by semi-major axis.
    pub bodies: Vec<GrownBody>,
    pub rings: Vec<Ring>,
    pub budget: GrowthBudget,
    pub events: Vec<GrowthEvent>,
    pub inner_edge_au: f64,
    pub end_myr: f64,
}

impl Growth {
    /// Resonant chains: runs of three or more bodies each held by its inner neighbour.
    pub fn budget_closes(&self) -> bool {
        let b = &self.budget;
        (0..6).all(|g| {
            b.solid_initial[g] + b.solid_condensed[g]
                == b.solid_in_bands[g]
                    + b.solid_in_bodies[g]
                    + b.solid_in_rings[g]
                    + b.solid_sublimated[g]
                    + b.solid_to_star[g]
        }) && b.gas_initial == b.gas_in_bands + b.gas_in_bodies
    }
    pub fn chains(&self) -> Vec<Vec<usize>> {
        let mut out = vec![];
        let mut run: Vec<usize> = vec![];
        for (i, b) in self.bodies.iter().enumerate() {
            match b.resonance {
                Some((j, _, _))
                    if j + 1 == i && !run.is_empty() && *run.last().expect("run") == j =>
                {
                    run.push(i)
                }
                Some((j, _, _)) if j + 1 == i => run = vec![j, i],
                _ => {
                    if run.len() >= 3 {
                        out.push(std::mem::take(&mut run));
                    }
                    run.clear();
                }
            }
        }
        if run.len() >= 3 {
            out.push(run);
        }
        out
    }
}

struct Embryo {
    body: GrownBody,
    t_form: f64,
    formed: bool,
    ratio: (u32, u32),
    gap_mass_checked: bool,
    parked: bool,
}

fn scale_height(disc: &Disc, r: f64, t: f64) -> f64 {
    let temp = disc.temperature_at(r, t);
    let cs = m::sqrt(K_B * temp / (MU * M_H));
    let vk = m::sqrt(G * disc.params.host_mass * MSUN_KG / (r * AU_M));
    cs / vk
}

fn omega(disc: &Disc, r: f64) -> f64 {
    let rm = r * AU_M;
    m::sqrt(G * disc.params.host_mass * MSUN_KG / (rm * rm * rm))
}

/// Pebble isolation mass, Earth masses (Bitsch et al. 2018 eq. 26: eqs. 10-11's
/// 25 f_fit plus the turbulent-diffusion term Pi_crit / lambda, Pi_crit =
/// alpha / (2 St), lambda = 0.00476 / f_fit; the pressure-gradient factor is
/// near 1 for these discs). The paper fits a solar-mass star; the host-mass
/// factor is an extrapolation (docs/fudges.md).
pub fn isolation_mass(h: f64, host_mass: f64, alpha: f64) -> f64 {
    let la = m::log10(0.001) / m::log10(alpha);
    let visc = 0.34 * la * la * la * la + 0.66;
    let f_fit = m::pow(h / 0.05, 3.0) * visc;
    let diffusion = alpha / (2.0 * STOKES) / 0.00476;
    (25.0 + diffusion) * f_fit * host_mass
}

/// Share of the passing pebble flux a body of `mass_earth` takes (2D Hill
/// regime, Lambrechts and Johansen 2014): (St/0.1)^(2/3) (q/3)^(2/3) / (2 pi St eta), eta = h^2.
pub fn pebble_efficiency(mass_earth: f64, host_mass: f64, h: f64) -> f64 {
    let q = mass_earth / MSUN_IN_EARTH / host_mass;
    let eta = h * h;
    (m::pow(STOKES / 0.1, 2.0 / 3.0) * m::pow(q / 3.0, 2.0 / 3.0) / (2.0 * m::PI * STOKES * eta))
        .min(1.0)
}

/// Lee and Chiang 2015's dusty-envelope gas-to-core ratio for a 5 Earth-mass core after 1 Myr.
pub const LC_COEFF: f64 = 0.02;
/// Gas captured by a core still in the pebble flow, as a fraction of its mass (Lee and Chiang 2015, scaled).
pub fn core_gas_fraction(core_earth: f64, age_myr: f64) -> f64 {
    LC_COEFF * m::pow(age_myr.max(0.0), 0.4) * m::pow(core_earth / 5.0, 1.7)
}

/// Kelvin-Helmholtz coefficient, years (Ida and Lin 2004's 10^k1 with k1 between 8 and 10; dusty envelopes sit high).
pub const KH_COEFF_YR: f64 = 1.0e11;
/// Kelvin-Helmholtz contraction time, years.
pub fn kh_time(mass_earth: f64) -> f64 {
    KH_COEFF_YR * m::pow(mass_earth.max(1e-3), -3.0)
}

fn band_index(disc: &Disc, r: f64) -> usize {
    disc.bands
        .iter()
        .position(|b| r >= b.inner_au && r < b.outer_au)
        .unwrap_or(disc.bands.len() - 1)
}

fn take_fraction(from: &mut [u64; 6], want: u64, allowed: &[bool; 6]) -> [u64; 6] {
    let total: u64 = (0..6).filter(|g| allowed[*g]).map(|g| from[g]).sum();
    let mut out = [0u64; 6];
    if total == 0 || want == 0 {
        return out;
    }
    for g in 0..6 {
        if !allowed[g] {
            continue;
        }
        let x = if want >= total {
            from[g]
        } else {
            ((from[g] as u128 * want as u128) / total as u128) as u64
        };
        out[g] = x;
        from[g] -= x;
    }
    out
}

fn add(a: &mut [u64; 6], b: &[u64; 6]) {
    for g in 0..6 {
        a[g] += b[g];
    }
}

/// Run the modern growth block on a system's disc. `host_mass` is the disc's host.
pub fn grow(seed: &Seed, system: &Address, disc: &Disc) -> Growth {
    let d = Draws::new(
        *seed,
        system.clone(),
        Stream::new("formation.growth.v1").expect("valid"),
    );
    let host = disc.params.host_mass;
    let mut bands = disc.bands.clone();
    let solid_initial = {
        let mut s = [0u64; 6];
        for b in &bands {
            add(&mut s, &b.solid);
        }
        s
    };
    let gas_initial: u64 = bands.iter().map(|b| b.gas).sum();
    let mut budget = GrowthBudget {
        solid_initial,
        solid_condensed: [0; 6],
        solid_in_bands: [0; 6],
        solid_in_bodies: [0; 6],
        solid_in_rings: [0; 6],
        solid_sublimated: [0; 6],
        solid_to_star: [0; 6],
        gas_initial,
        gas_in_bands: 0,
        gas_in_bodies: 0,
    };
    let mut events = vec![];
    let mut rings: Vec<Ring> = vec![];
    // Inner edge: the magnetospheric cavity at the young star's corotation radius
    // (about 8-day orbits, Lee and Chiang 2017's explanation of Kepler's 10-day
    // break), or a circumbinary cavity.
    let inner_edge = (0.08 * m::pow(host, 1.0 / 3.0)).max(disc.params.inner_edge * 1.05);
    let outer = disc
        .params
        .truncation
        .unwrap_or(f64::MAX)
        .min(disc.params.rc0 * 1.5);
    let t_end = disc.t_disperse.max(T_START_MYR * 1.5);
    let water_line = disc.line_at(135.0, 0.2);

    // Embryos: a geometric ladder of planetesimal belts, spacing drawn, with the
    // first and largest ring at the water line (Drazkowska and Alibert 2017).
    let mut embryos: Vec<Embryo> = vec![];
    let mut r = inner_edge * (1.5 + d.unit(0));
    let mut k = 1u64;
    while r < outer && embryos.len() < 40 {
        let u = d.unit(10 * k);
        let at_line = (r / water_line).max(water_line / r) < 1.2;
        let rr = if at_line { water_line } else { r };
        let t_form = if at_line {
            0.1 + 0.2 * u
        } else {
            0.2 + 1.0 * u
        };
        let id = format!("embryo-{k:02}");
        let ratio = {
            let x = d.unit(10 * k + 1);
            if x < 0.25 {
                (2, 1)
            } else if x < 0.75 {
                (3, 2)
            } else {
                (4, 3)
            }
        };
        embryos.push(Embryo {
            body: GrownBody {
                id: id.clone(),
                born_myr: t_form,
                born_au: rr,
                born_band: String::new(),
                a_au: rr,
                e: 0.002 + 0.008 * d.unit(10 * k + 2),
                solid: [0; 6],
                gas: 0,
                ledger: GrowthLedger {
                    id,
                    entries: vec![],
                },
                isolated: false,
                opened_gap: false,
                resonance: None,
            },
            t_form,
            formed: false,
            ratio,
            gap_mass_checked: false,
            parked: false,
        });
        r = rr * (1.35 + 0.35 * d.unit(10 * k + 3));
        k += 1;
    }
    // Solid mass fraction of the disc's material (sets the pebble flux).
    let z: f64 = disc.params.mix.iter().sum();

    let mut t_prev = T_START_MYR;
    for step in 1..=STEPS {
        let t = T_START_MYR * m::pow(t_end / T_START_MYR, step as f64 / STEPS as f64);
        let dt_yr = (t - t_prev) * 1.0e6;
        let tm = 0.5 * (t + t_prev);
        // 1. Embryos that form now claim their seed mass from the local band.
        for e in embryos.iter_mut() {
            if e.formed || e.t_form > t {
                continue;
            }
            let rr = e.body.a_au;
            let temp = disc.temperature_at(rr, tm);
            let allowed = GROUPS.map(|g| temp < g.condenses_below());
            let bi = band_index(disc, rr);
            // Vapour in this band condenses once the disc here has cooled enough.
            for (g, ok) in allowed.iter().enumerate() {
                if *ok && bands[bi].vapour[g] > 0 {
                    let v = bands[bi].vapour[g];
                    bands[bi].vapour[g] = 0;
                    bands[bi].solid[g] += v;
                    budget.solid_condensed[g] += v;
                }
            }
            let m0 =
                (0.01 * m::sqrt(rr)).clamp(0.002, 0.05) * if rr == water_line { 3.0 } else { 1.0 };
            let want = m::round(m0 / MSUN_IN_EARTH / MASS_UNIT) as u64;
            let got = take_fraction(&mut bands[bi].solid, want, &allowed);
            if got.iter().sum::<u64>() < want / 2 {
                continue; // too hot or too poor here yet
            }
            e.formed = true;
            e.body.born_band = bands[bi].key.clone();
            e.body.born_myr = t;
            add(&mut e.body.solid, &got);
            e.body.ledger.entries.push(GrowthEntry::Claim(Claim {
                t_myr: t,
                band: bands[bi].key.clone(),
                solid: got,
                gas: 0,
            }));
            events.push(GrowthEvent::EmbryoForms {
                t_myr: t,
                id: e.body.id.clone(),
                a_au: rr,
                band: bands[bi].key.clone(),
            });
        }
        // 2. The pebble stream: drawn from the outermost bands first, drifting inward.
        let want = m::round(PEBBLE_FLUX_PER_Z * z * disc.mdot_at(tm) * dt_yr / MASS_UNIT) as u64;
        let mut stream: Vec<(usize, [u64; 6])> = vec![];
        let mut need = want;
        for bi in (0..bands.len()).rev() {
            if need == 0 {
                break;
            }
            let got = take_fraction(&mut bands[bi].solid, need, &[true; 6]);
            let s: u64 = got.iter().sum();
            if s > 0 {
                stream.push((bi, got));
                need = need.saturating_sub(s);
            }
        }
        embryos.sort_by(|x, y| y.body.a_au.partial_cmp(&x.body.a_au).expect("finite"));
        for e in embryos.iter_mut().filter(|e| e.formed) {
            let rr = e.body.a_au;
            let temp = disc.temperature_at(rr, tm);
            // Ices sublimate as the stream passes inside their lines.
            for (_, c) in stream.iter_mut() {
                for (g, grp) in GROUPS.iter().enumerate() {
                    if temp >= grp.condenses_below() && c[g] > 0 {
                        budget.solid_sublimated[g] += c[g];
                        c[g] = 0;
                    }
                }
            }
            let h = scale_height(disc, rr, tm);
            if e.body.isolated {
                // Pebbles pile up in the pressure bump outside: a ring.
                let mut held = [0u64; 6];
                for (_, c) in stream.drain(..) {
                    add(&mut held, &c);
                }
                if held.iter().any(|x| *x > 0) {
                    add(&mut budget.solid_in_rings, &held);
                    match rings
                        .iter_mut()
                        .find(|x: &&mut Ring| x.held_by == e.body.id)
                    {
                        Some(ring) => add(&mut ring.solid, &held),
                        None => rings.push(Ring {
                            a_au: rr * 1.2,
                            held_by: e.body.id.clone(),
                            solid: held,
                        }),
                    }
                }
                break;
            }
            let eff = pebble_efficiency(e.body.solid_earth(), host, h);
            for (bi, c) in stream.iter_mut() {
                let taken: [u64; 6] = std::array::from_fn(|g| m::floor(c[g] as f64 * eff) as u64);
                if taken.iter().sum::<u64>() == 0 {
                    continue;
                }
                for g in 0..6 {
                    c[g] -= taken[g];
                }
                add(&mut e.body.solid, &taken);
                e.body.ledger.entries.push(GrowthEntry::Claim(Claim {
                    t_myr: t,
                    band: bands[*bi].key.clone(),
                    solid: taken,
                    gas: 0,
                }));
            }
            if e.body.solid_earth() >= isolation_mass(h, host, disc.params.alpha) {
                e.body.isolated = true;
                events.push(GrowthEvent::PebbleIsolation {
                    t_myr: t,
                    id: e.body.id.clone(),
                    mass_earth: e.body.solid_earth(),
                });
            }
        }
        for (_, c) in stream {
            add(&mut budget.solid_to_star, &c);
        }
        // 3. Gas: cores in the flow hold a thin envelope; isolated cores contract and may run away.
        for e in embryos.iter_mut().filter(|e| e.formed) {
            let core = e.body.solid_earth();
            let env = e.body.gas_earth();
            let target = if e.body.isolated {
                let total = core + env;
                let dm = total / kh_time(total) * dt_yr;
                // Supply: a share of the disc's flow, and what reaches the planet
                // through its gap (Tanigawa and Tanaka 2016).
                let rr = e.body.a_au;
                let h = scale_height(disc, rr, tm);
                let q = total / MSUN_IN_EARTH / host;
                let kk = q * q / (h * h * h * h * h) / disc.params.alpha;
                let sigma_gap = disc.sigma_at(rr, tm) * 10.0 / (1.0 + 0.04 * kk);
                let gap_rate = 0.29 / (h * h)
                    * m::pow(q, 4.0 / 3.0)
                    * sigma_gap
                    * (rr * AU_M)
                    * (rr * AU_M)
                    * omega(disc, rr);
                let cap = (GIANT_FLOW_SHARE * disc.mdot_at(tm) * MSUN_IN_EARTH)
                    .min(gap_rate * YR_S / (MSUN_KG / MSUN_IN_EARTH))
                    * dt_yr;
                env + dm.min(cap)
            } else {
                (core_gas_fraction(core, t - e.t_form) * core).min(core)
            };
            if target > env {
                let bi = band_index(disc, e.body.a_au);
                let want = m::round((target - env) / MSUN_IN_EARTH / MASS_UNIT) as u64;
                let got = want.min(bands[bi].gas);
                if got > 0 {
                    bands[bi].gas -= got;
                    let before_runaway = env < core;
                    e.body.gas += got;
                    e.body.ledger.entries.push(GrowthEntry::Claim(Claim {
                        t_myr: t,
                        band: bands[bi].key.clone(),
                        solid: [0; 6],
                        gas: got,
                    }));
                    if before_runaway && e.body.gas_earth() >= core {
                        events.push(GrowthEvent::RunawayGas {
                            t_myr: t,
                            id: e.body.id.clone(),
                            core_earth: core,
                        });
                    }
                }
            }
        }
        // 4. Migration (type I, or type II once a gap opens), then resonances and merges.
        let water_now = disc.line_at(135.0, tm);
        for e in embryos.iter_mut().filter(|e| e.formed) {
            let rr = e.body.a_au;
            let h = scale_height(disc, rr, tm);
            let q = e.body.mass_earth() / MSUN_IN_EARTH / host;
            let om = omega(disc, rr);
            let sigma = disc.sigma_at(rr, tm) * 10.0; // kg/m^2
            let disc_local = sigma * (rr * AU_M) * (rr * AU_M) / (host * MSUN_KG);
            // Kanagawa et al. 2018: one torque for both regimes, type I divided by
            // the gap depth 1 + 0.04 K, K = q^2 h^-5 / alpha; a gap is open past K = 25.
            let kk = q * q / (h * h * h * h * h) / disc.params.alpha;
            let gap = kk > 25.0;
            if gap && !e.gap_mass_checked {
                e.gap_mass_checked = true;
                e.body.opened_gap = true;
                events.push(GrowthEvent::GapOpens {
                    t_myr: t,
                    id: e.body.id.clone(),
                });
            }
            let rate = TYPE_I_REDUCTION * 3.8 * q * disc_local / (h * h) * om / (1.0 + 0.04 * kk)
                * if gap { TYPE_II_REDUCTION } else { 1.0 };
            let mut na = rr * m::exp(-rate * dt_yr * YR_S);
            // Planet trap at the water line (Hasegawa and Pudritz 2011): the
            // opacity jump turns the corotation torque outward for cores from a
            // few Earth masses until they open a gap, which then ride the line inward.
            let mt = e.body.mass_earth();
            if !gap && (TRAP_MASS.0..=TRAP_MASS.1).contains(&mt) && rr >= water_now {
                na = na.max(water_now);
            }
            e.body.a_au = na.max(inner_edge);
            if na < inner_edge && rr > inner_edge && !e.parked {
                e.parked = true;
                events.push(GrowthEvent::ParkedAtInnerEdge {
                    t_myr: t,
                    id: e.body.id.clone(),
                    a_au: inner_edge,
                });
            }
        }
        embryos.sort_by(|x, y| x.body.a_au.partial_cmp(&y.body.a_au).expect("finite"));
        let mut i = 1;
        while i < embryos.len() {
            if !embryos[i].formed || !embryos[i - 1].formed {
                i += 1;
                continue;
            }
            let (ai, ao) = (embryos[i - 1].body.a_au, embryos[i].body.a_au);
            let mi = embryos[i - 1].body.mass_earth() / MSUN_IN_EARTH / host;
            let mo = embryos[i].body.mass_earth() / MSUN_IN_EARTH / host;
            let rh = 0.5 * (ai + ao) * m::pow((mi + mo) / 3.0, 1.0 / 3.0);
            let (p, qq) = embryos[i].ratio;
            let res_a = ai * m::pow(p as f64 / qq as f64, 2.0 / 3.0);
            if ao < res_a && res_a - ai >= 2.0 * m::sqrt(3.0) * rh {
                // Convergent migration captures the outer body in resonance.
                if embryos[i].body.resonance.is_none() {
                    events.push(GrowthEvent::CapturedInResonance {
                        t_myr: t,
                        id: embryos[i].body.id.clone(),
                        inner: embryos[i - 1].body.id.clone(),
                        p,
                        q: qq,
                    });
                }
                embryos[i].body.a_au = res_a;
                embryos[i].body.resonance = Some((i - 1, p, qq));
                i += 1;
                continue;
            }
            if ao - ai < 2.0 * m::sqrt(3.0) * rh {
                // Too close to stay apart: they merge (Gladman 1993).
                let outer_e = embryos.remove(i);
                let inner_e = &mut embryos[i - 1];
                let w = (mi * ai + mo * ao) / (mi + mo);
                let id = format!("{}+{}", inner_e.body.id, outer_e.body.id);
                let from = [inner_e.body.id.clone(), outer_e.body.id.clone()];
                let old = std::mem::replace(
                    &mut inner_e.body.ledger,
                    GrowthLedger {
                        id: id.clone(),
                        entries: vec![],
                    },
                );
                inner_e.body.ledger.entries.push(GrowthEntry::Merged {
                    body: Box::new(old),
                });
                inner_e.body.ledger.entries.push(GrowthEntry::Merged {
                    body: Box::new(outer_e.body.ledger),
                });
                add(&mut inner_e.body.solid, &outer_e.body.solid);
                inner_e.body.gas += outer_e.body.gas;
                inner_e.body.a_au = w;
                inner_e.body.isolated |= outer_e.body.isolated;
                inner_e.body.opened_gap |= outer_e.body.opened_gap;
                inner_e.body.id = id.clone();
                events.push(GrowthEvent::EmbryosMerge {
                    t_myr: t,
                    into: id,
                    from,
                });
                continue;
            }
            i += 1;
        }
        t_prev = t;
    }
    events.push(GrowthEvent::GasGone { t_myr: t_end });
    let mut bodies: Vec<GrownBody> = embryos
        .into_iter()
        .filter(|e| e.formed)
        .map(|e| e.body)
        .collect();
    bodies.sort_by(|x, y| x.a_au.partial_cmp(&y.a_au).expect("finite"));
    // Resonance links point at the inner neighbour by index after sorting; refresh them.
    for i in 0..bodies.len() {
        if let Some((_, p, q)) = bodies[i].resonance {
            let ok = i > 0 && {
                let ratio = m::pow(bodies[i].a_au / bodies[i - 1].a_au, 1.5);
                // Held near commensurability (the last step of gas drag moves it slightly).
                (ratio / (p as f64 / q as f64) - 1.0).abs() < 0.02
            };
            bodies[i].resonance = if ok { Some((i - 1, p, q)) } else { None };
        }
    }
    for b in &bands {
        add(&mut budget.solid_in_bands, &b.solid);
        budget.gas_in_bands += b.gas;
    }
    for b in &bodies {
        add(&mut budget.solid_in_bodies, &b.solid);
        budget.gas_in_bodies += b.gas;
    }
    Growth {
        bodies,
        rings,
        budget,
        events,
        inner_edge_au: inner_edge,
        end_myr: t_end,
    }
}

/// The hand-over to stage 8 (F6) for every grown body.
pub fn protoplanets(g: &Growth, system: &Address) -> Vec<crate::early::Protoplanet> {
    let k = MASS_UNIT * MSUN_IN_EARTH;
    g.bodies
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let by = b.solid.map(|u| u as f64 * k);
            crate::early::Protoplanet {
                address: system.child("body", i as u64).expect("valid"),
                mass: by.iter().sum(),
                a_au: b.a_au,
                e: b.e,
                formed_myr: b.ledger.last_claim(),
                by_group: by,
                gas: b.gas as f64 * k,
            }
        })
        .collect()
}
