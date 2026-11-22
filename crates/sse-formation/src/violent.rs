//! Stage 6, the violent phase and the upheaval system (F4, design section 3A).
//!
//! After the gas leaves, a system is measured for how close it is to trouble
//! (spacing in mutual Hill radii, resonant chains, giants beside a heavy outer
//! belt). Keyed draws against published instability timescales give the date
//! of each break, or never. A break starts a cascade of typed events (impacts
//! by scaling law, scatterings, ejections, engulfment, belts stirred into
//! bombardment spikes and reservoirs) that runs until every neighbour pair is
//! stable; then orbits settle.
//!
//! Budgets: every unit of solid and gas that F3 left in bodies and rings is
//! somewhere at the end (a world, a belt, a rogue planet, an impact moon, a
//! reservoir for F5, ejected as small bodies, or the star), in integer budget
//! units, exactly. Orbital energy and angular momentum are booked to named
//! sinks (heat, rogues, the star, belts, impact debris, tides) and close to
//! rounding. Each world's ledger holds its F3 growth record, what it merged
//! with, what was delivered to it and what it lost, so its water sums by band.
//!
//! Standard level: rules and calibrated draws, no orbit integration. The
//! optional Detailed block (F8's integrator) replaces the cascade later and
//! writes the same events.

use crate::disc::{Disc, MASS_UNIT};
use crate::growth::{Growth, GrowthEntry, GrowthLedger, MSUN_IN_EARTH};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

pub const REF_OBERTAS17: &str = "Obertas, A., Van Laerhoven, C. and Tamayo, D. 2017, Icarus 293, 52: instability time of compact multi-planet systems of equal Earth-mass planets, log10(t/P1) = 0.951 Delta - 1.202 in mutual Hill radii (fit over 3 to 8; used here further out and for unequal masses). arXiv:1703.08426";
pub const REF_GLADMAN93: &str = "Gladman, B. 1993, Icarus 106, 247: two planets more than 2 sqrt(3) mutual Hill radii apart never cross (Hill stability)";
pub const REF_IZIDORO: &str = "Izidoro, A. et al. 2017, MNRAS 470, 1750; 2021, A&A 650, A152: about 90 to 95 percent of resonant chains break after the gas leaves, mostly within 100 Myr";
pub const REF_SAFRONOV: &str = "Safronov number Theta = (M_p/M_star)(a/R_p); Ford, E. B. and Rasio, F. A. 2008, ApJ 686, 621: collisions dominate for Theta < 1, scattering and ejection for Theta > 1";
pub const REF_GENDA12: &str = "Genda, H., Kokubo, E. and Ida, S. 2012, ApJ 744, 137: merging criterion v_cr/v_esc = 2.43 G (1-sin t)^2.5 - 0.0408 G + 1.86 (1-sin t)^2.5 + 1.08, G = (M1-M2)/(M1+M2)";
pub const REF_LS12: &str = "Leinhardt, Z. M. and Stewart, S. T. 2012, ApJ 745, 79: largest remnant M_lr/M_tot = 1 - Q_R/(2 Q*_RD), super-catastrophic 0.1 (Q_R/1.8 Q*_RD)^-1.5; Q*_RD = c* (4/5) pi rho1 G R_C1^2, c* = 1.9";
pub const REF_ASPHAUG10: &str = "Asphaug, E. 2010, Chemie der Erde 70, 199: grazing impacts above the critical impact parameter are hit-and-run; the runner loses mantle";
pub const REF_MARCUS10: &str = "Marcus, R. A. et al. 2009, ApJ 700, L118 (collisional stripping and disruption of super-Earths): erosive impacts strip mantle first, raising the iron fraction. arXiv:0907.0234";
pub const REF_SHOEMAKER: &str =
    "Shoemaker, E. M. 1962: impact angles are distributed as sin 2 theta, most likely 45 degrees";
pub const REF_CANUP04: &str = "Canup, R. M. 2004, Icarus 168, 433; Elser, S. et al. 2011, Icarus 214, 357: moon-forming impacts are oblique (30 to 60 degrees), impactor 5 to 50 percent of the total; a few to tens of percent of rocky planets get one";
pub const REF_NICE: &str = "Tsiganis, K. et al. 2005, Nature 435, 459; Gomes, R. et al. 2005, Nature 435, 466; Nesvorny, D. 2018, ARA&A 56, 137: giants crossing a resonance stir the outer belt, removing about 99 percent of it, between a few and about 500 Myr after the gas";
pub const REF_MALHOTRA: &str = "Malhotra, R. 1995, AJ 110, 420: an outer giant migrates outward by scattering belt planetesimals (about 17 Earth masses moved 20 to 30 AU through a 35 Earth-mass belt)";
pub const REF_OORT_EFF: &str = "Vokrouhlicky, D., Nesvorny, D. and Dones, L. 2019, AJ 157, 181: a few percent of scattered planetesimals end in the Oort cloud";
pub const REF_TROJANS: &str = "Nesvorny, D., Vokrouhlicky, D. and Morbidelli, A. 2013, ApJ 768, 45: Jupiter Trojans captured with probability about 6e-7 to 8e-7 of the stirred belt";
pub const REF_IRREG: &str = "Nesvorny, D., Vokrouhlicky, D. and Morbidelli, A. 2007, AJ 133, 1962: irregular moons captured in encounters during the instability";
pub const REF_GOMES05_LHB: &str = "Gomes et al. 2005: the Moon received about 6e18 kg from the stirred belt; Earth about 20 times more (about 1e-6 of the belt)";
pub const REF_FR06: &str = "Ford, E. B. and Rasio, F. A. 2006, ApJ 638, L45: planets scattered onto orbits with pericentre below about 0.05 AU circularise by tides at about twice the pericentre";
pub const REF_CHATTERJEE08: &str = "Chatterjee, S. et al. 2008, ApJ 686, 580; Juric, M. and Tremaine, S. 2008, ApJ 686, 603: survivors of giant scattering have eccentricities near a Rayleigh distribution of width about 0.3";
pub const REF_KG10: &str = "Kokubo, E. and Genda, H. 2010, ApJ 714, L21: after giant impacts, spins are near the critical rate and obliquities isotropic";
pub const REF_BRYAN18: &str = "Bryan, M. L. et al. 2018, Nature Astronomy 2, 138: planetary-mass companions spin at about 0.1 to 0.3 of breakup";

const R_EARTH_AU: f64 = 4.258_75e-5;
const R_EARTH_M: f64 = sse_core::sheet::R_EARTH_MEAN;
const M_EARTH_KG: f64 = 5.972_2e24;
const G: f64 = sse_core::sheet::G;
/// Escape speed from the surface of Earth, km/s; orbital speed at 1 AU around the Sun, km/s.
const V_ESC_EARTH: f64 = 11.186;
const V_KEP_1AU: f64 = 29.784_7;
const W: usize = 3;
/// Loss order for erosive impacts, outermost first (Co, Co2, water, carbon, rock, metal).
const LAYERS: [usize; 6] = [5, 4, 3, 2, 1, 0];

/// Fraction of resonant chains that break after the gas leaves (Izidoro et al.).
pub const CHAIN_BREAK_FRACTION: f64 = 0.9;
/// Outer-belt mass, Earth masses, at which the Nice trigger is likely (1 - 1/e).
pub const NICE_BELT_EARTH: f64 = 10.0;
/// Outward migration of the outer giant: angular momentum gained per unit belt mass
/// per unit sqrt(a) (Malhotra 1995's Neptune: 0.1).
pub const NICE_KAPPA: f64 = 0.1;
/// Pairs with spacing below this, in mutual Hill radii, next to another such
/// pair, follow the multi-planet instability time; others the two-planet rule.
pub const DELTA_MULTI: f64 = 12.0;
/// Scatter of the instability time, dex (Obertas et al. 2017 find about 0.4 to 0.5).
pub const T_SCATTER_DEX: f64 = 0.45;
/// Share of strong scatterings that eject the lighter body, times Theta^2/(1+Theta^2).
pub const EJECT_SHARE: f64 = 0.5;
/// Eccentricity left after a giant impact, as a Rayleigh width per v_esc/v_kep (calibrated on Kepler multis).
pub const RESIDUAL_E: f64 = 0.25;
/// Chance that an oblique (30 to 60 degree) merging impact with 5 to 50 percent
/// impactor leaves a large moon (1: the window alone decides; Elser et al. 2011's rate follows).
pub const MOON_CHANCE: f64 = 1.0;
/// Belt stirring: share removed, and where the removed share goes.
pub const STIR_DEPLETION: f64 = 0.99;
pub const OORT_SHARE: f64 = 0.04;
pub const SCATTERED_SHARE: f64 = 0.01;
pub const TO_STAR_SHARE: f64 = 0.1;
pub const TROJAN_CAPTURE: f64 = 7.0e-7;
pub const IRREGULAR_CAPTURE: f64 = 1.0e-7;
/// Share of the removed belt that hits an Earth at 1 AU (Gomes et al. 2005 scale).
pub const HIT_EARTH: f64 = 1.0e-6;
/// E-folding time of a bombardment spike, Myr (calibrated).
pub const SPIKE_EFOLD_MYR: f64 = 50.0;
/// Pericentres below this, AU, circularise by tides (Ford and Rasio 2006).
pub const CIRC_Q_AU: f64 = 0.05;
/// How far ahead the cascade looks, Myr after the gas leaves (later instabilities are F7's).
pub const HORIZON_MYR: f64 = 10_000.0;
pub const MAX_EVENTS: u64 = 200;

pub type WaterFrom = Vec<(String, u64)>;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum WorldEntry {
    /// The F3 growth record of a body that entered the violent phase.
    Grown { ledger: GrowthLedger },
    /// A world that merged into this one, whole.
    Merged { world: Box<WorldLedger> },
    /// Bombardment from a stirred belt.
    Delivered {
        t_myr: f64,
        belt: String,
        solid: [u64; 6],
        water_from: WaterFrom,
    },
    /// Stripped in an impact, spun off as a moon, or envelope blown away.
    Lost {
        t_myr: f64,
        cause: String,
        solid: [u64; 6],
        gas: u64,
        water_from: WaterFrom,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldLedger {
    pub id: String,
    pub entries: Vec<WorldEntry>,
}

fn growth_water(l: &GrowthLedger, out: &mut Vec<(String, i128)>) {
    for e in &l.entries {
        match e {
            GrowthEntry::Claim(c) => bump(out, &c.band, c.solid[W] as i128),
            GrowthEntry::Merged { body } => growth_water(body, out),
        }
    }
}

fn bump(out: &mut Vec<(String, i128)>, k: &str, x: i128) {
    if x == 0 {
        return;
    }
    match out.iter_mut().find(|(b, _)| b == k) {
        Some(e) => e.1 += x,
        None => out.push((k.to_string(), x)),
    }
}

impl WorldLedger {
    pub fn solid(&self) -> [u64; 6] {
        let mut s = [0i128; 6];
        self.add_solid(&mut s);
        s.map(|x| x as u64)
    }
    fn add_solid(&self, s: &mut [i128; 6]) {
        for e in &self.entries {
            match e {
                WorldEntry::Grown { ledger } => {
                    let x = ledger.solid();
                    for g in 0..6 {
                        s[g] += x[g] as i128;
                    }
                }
                WorldEntry::Merged { world } => world.add_solid(s),
                WorldEntry::Delivered { solid, .. } => {
                    for g in 0..6 {
                        s[g] += solid[g] as i128;
                    }
                }
                WorldEntry::Lost { solid, .. } => {
                    for g in 0..6 {
                        s[g] -= solid[g] as i128;
                    }
                }
            }
        }
    }
    pub fn gas(&self) -> u64 {
        let mut s: i128 = 0;
        self.add_gas(&mut s);
        s as u64
    }
    fn add_gas(&self, s: &mut i128) {
        for e in &self.entries {
            match e {
                WorldEntry::Grown { ledger } => *s += ledger.gas() as i128,
                WorldEntry::Merged { world } => world.add_gas(s),
                WorldEntry::Delivered { .. } => {}
                WorldEntry::Lost { gas, .. } => *s -= *gas as i128,
            }
        }
    }
    /// Water units by source band (F1 band keys, or a belt's band), signed sums over the whole tree.
    pub fn water_by_band(&self, out: &mut Vec<(String, i128)>) {
        for e in &self.entries {
            match e {
                WorldEntry::Grown { ledger } => growth_water(ledger, out),
                WorldEntry::Merged { world } => world.water_by_band(out),
                WorldEntry::Delivered { water_from, .. } => {
                    for (b, x) in water_from {
                        bump(out, b, *x as i128)
                    }
                }
                WorldEntry::Lost { water_from, .. } => {
                    for (b, x) in water_from {
                        bump(out, b, -(*x as i128))
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct World {
    pub id: String,
    pub a_au: f64,
    pub e: f64,
    pub solid: [u64; 6],
    pub gas: u64,
    pub ledger: WorldLedger,
    /// Inner neighbour's id and the p:q ratio while a resonant chain holds.
    pub resonance: Option<(String, u32, u32)>,
    pub impacts: u32,
    pub last_impact_myr: Option<f64>,
    pub scattered: bool,
    pub spin_hours: f64,
    pub obliquity_deg: f64,
    #[serde(skip)]
    version: u64,
    #[serde(skip)]
    changed_myr: f64,
    #[serde(skip)]
    last_event: Option<usize>,
}

pub(crate) fn earth(units: u64) -> f64 {
    units as f64 * MASS_UNIT * MSUN_IN_EARTH
}

impl World {
    pub fn solid_earth(&self) -> f64 {
        earth(self.solid.iter().sum())
    }
    pub fn gas_earth(&self) -> f64 {
        earth(self.gas)
    }
    pub fn mass_earth(&self) -> f64 {
        self.solid_earth() + self.gas_earth()
    }
    pub fn water_earth(&self) -> f64 {
        earth(self.solid[W])
    }
    pub fn is_giant(&self) -> bool {
        self.gas_earth() > self.solid_earth() && self.mass_earth() > 30.0
    }
    /// Radius, Earth radii, for encounter physics: 11.2 for giants (as F6),
    /// Zeng 2016 rock with water, plus a Lopez and Fortney 2014 envelope.
    pub fn radius_earth(&self) -> f64 {
        if self.is_giant() {
            return 11.2;
        }
        let s = self.solid_earth().max(1e-6);
        let rm = (self.solid[0] + self.solid[1]).max(1) as f64;
        let cmf = self.solid[0] as f64 / rm;
        let wf = self.solid[W] as f64 / self.solid.iter().sum::<u64>().max(1) as f64;
        let core = crate::early::rocky_radius(s, cmf) * (1.0 + 0.5 * wf);
        let fenv = self.gas_earth() / self.mass_earth().max(1e-9);
        if fenv > 1e-3 {
            core + crate::early::envelope_thickness(s.clamp(1.0, 20.0), fenv.min(0.2), 1.0, 0.1)
        } else {
            core
        }
    }
    fn energy(&self) -> f64 {
        -self.mass_earth() / (2.0 * self.a_au)
    }
    fn angmom(&self) -> f64 {
        self.mass_earth() * m::sqrt(self.a_au * (1.0 - self.e * self.e))
    }
    fn bump(&mut self, t: f64, ev: usize) {
        self.version += 1;
        self.changed_myr = t;
        self.last_event = Some(ev);
        self.resonance = None;
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Belt {
    pub id: String,
    pub a_au: f64,
    /// The F3 body that held it up, or the impact that made it.
    pub origin: String,
    pub solid: [u64; 6],
    pub water_from: WaterFrom,
    pub stirred: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Reservoir {
    /// "oort", "scattered-disc", "trojans" or "irregular-moons".
    pub kind: String,
    /// The giant that holds Trojans or irregular moons.
    pub of: Option<String>,
    pub from_belt: String,
    pub t_myr: f64,
    pub solid: [u64; 6],
    pub water_from: WaterFrom,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ImpactMoon {
    pub parent: String,
    pub t_myr: f64,
    pub solid: [u64; 6],
    pub water_from: WaterFrom,
    pub angle_deg: f64,
    pub v_over_vesc: f64,
}

/// A planet thrown out of the system: a real, addressable object.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rogue {
    pub address: Address,
    pub id: String,
    pub t_myr: f64,
    pub by: String,
    pub v_inf_km_s: f64,
    pub solid: [u64; 6],
    pub gas: u64,
    pub ledger: WorldLedger,
}

impl Rogue {
    pub fn mass_earth(&self) -> f64 {
        earth(self.solid.iter().sum::<u64>() + self.gas)
    }
}

/// Mass books in integer units: start = where everything ended, per group.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct UpheavalBudget {
    pub solid_start: [u64; 6],
    pub gas_start: u64,
    pub solid_in_worlds: [u64; 6],
    pub solid_in_belts: [u64; 6],
    pub solid_in_rogues: [u64; 6],
    pub solid_in_moons: [u64; 6],
    pub solid_in_reservoirs: [u64; 6],
    /// Belt material thrown out of the system as small bodies.
    pub solid_ejected: [u64; 6],
    pub solid_to_star: [u64; 6],
    pub gas_in_worlds: u64,
    pub gas_in_rogues: u64,
    /// Envelopes blown off in impacts.
    pub gas_lost: u64,
    pub gas_to_star: u64,
}

impl UpheavalBudget {
    pub fn closes(&self) -> bool {
        (0..6).all(|g| {
            self.solid_start[g]
                == self.solid_in_worlds[g]
                    + self.solid_in_belts[g]
                    + self.solid_in_rogues[g]
                    + self.solid_in_moons[g]
                    + self.solid_in_reservoirs[g]
                    + self.solid_ejected[g]
                    + self.solid_to_star[g]
        }) && self.gas_start
            == self.gas_in_worlds + self.gas_in_rogues + self.gas_lost + self.gas_to_star
    }
}

/// Orbital energy (-m/2a) and angular momentum (m sqrt(a(1-e^2))) of the
/// worlds, in Earth masses, AU and G M_star = 1. Each sink is what left the
/// bound worlds that way; start = bound + sinks.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Dynamics {
    pub energy_start: f64,
    pub energy_bound: f64,
    /// Dissipated in impacts and tides (never negative).
    pub energy_heat: f64,
    pub energy_rogues: f64,
    pub energy_star: f64,
    /// Exchanged with belts (migration, delivered material).
    pub energy_belts: f64,
    /// Carried by impact debris and lost envelopes.
    pub energy_debris: f64,
    pub angmom_start: f64,
    pub angmom_bound: f64,
    pub angmom_rogues: f64,
    pub angmom_star: f64,
    pub angmom_belts: f64,
    pub angmom_debris: f64,
}

impl Dynamics {
    pub fn closes(&self) -> bool {
        let e = self.energy_bound
            + self.energy_heat
            + self.energy_rogues
            + self.energy_star
            + self.energy_belts
            + self.energy_debris;
        let l = self.angmom_bound
            + self.angmom_rogues
            + self.angmom_star
            + self.angmom_belts
            + self.angmom_debris;
        let es = self.energy_start.abs() + self.energy_heat.abs() + self.energy_rogues.abs();
        let ls = self.angmom_start.abs() + self.angmom_rogues.abs();
        (e - self.energy_start).abs() <= 1e-9 * es.max(1e-30)
            && (l - self.angmom_start).abs() <= 1e-9 * ls.max(1e-30)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Priming {
    /// Smallest neighbour spacing, mutual Hill radii, at the gas's end.
    pub min_spacing: f64,
    pub chained: bool,
    pub giants: u32,
    /// Mass of the belt outside the outermost giant, Earth masses.
    pub outer_belt_earth: f64,
    /// When the chain breaks, if it does.
    pub chain_break_myr: Option<f64>,
    /// When giants cross a resonance (Nice trigger), if they do.
    pub nice_myr: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum UpheavalEvent {
    ChainBreaks {
        t_myr: f64,
        ids: Vec<String>,
    },
    GiantsCrossResonance {
        t_myr: f64,
        inner: String,
        outer: String,
        outer_from_au: f64,
        outer_to_au: f64,
    },
    Impact {
        t_myr: f64,
        cause: Option<usize>,
        target: String,
        impactor: String,
        /// "merge", "merge-moon", "hit-and-run", "erosion" or "shattered".
        outcome: String,
        v_over_vesc: f64,
        angle_deg: f64,
        debris_earth: f64,
    },
    Scatter {
        t_myr: f64,
        cause: Option<usize>,
        kept: String,
        thrown: String,
        to_au: f64,
        e: f64,
    },
    Ejected {
        t_myr: f64,
        cause: Option<usize>,
        id: String,
        by: String,
        v_inf_km_s: f64,
        rogue: String,
    },
    Engulfed {
        t_myr: f64,
        cause: Option<usize>,
        id: String,
    },
    BeltStirred {
        t_myr: f64,
        cause: Option<usize>,
        belt: String,
        by: String,
        removed_earth: f64,
        oort_earth: f64,
    },
    Captured {
        t_myr: f64,
        cause: Option<usize>,
        by: String,
        kind: String,
        earth: f64,
    },
    /// A bombardment spike on one world: hand-off to surfaces (craters) and life (catastrophe).
    Bombardment {
        t_myr: f64,
        cause: Option<usize>,
        world: String,
        earth: f64,
        water_earth: f64,
        efold_myr: f64,
    },
    Circularised {
        t_myr: f64,
        cause: Option<usize>,
        id: String,
        a_au: f64,
    },
    Settled {
        t_myr: f64,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Upheaval {
    /// Sorted by semi-major axis.
    pub worlds: Vec<World>,
    pub belts: Vec<Belt>,
    pub rogues: Vec<Rogue>,
    pub moons: Vec<ImpactMoon>,
    pub reservoirs: Vec<Reservoir>,
    pub events: Vec<UpheavalEvent>,
    pub priming: Priming,
    pub budget: UpheavalBudget,
    pub dynamics: Dynamics,
    pub gas_end_myr: f64,
    /// First break, if any.
    pub break_myr: Option<f64>,
    pub settled_myr: f64,
}

impl Upheaval {
    /// A rogue planet by its address.
    pub fn rogue(&self, address: &Address) -> Option<&Rogue> {
        self.rogues.iter().find(|r| &r.address == address)
    }
}

pub(crate) fn fnv(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in (p.len() as u64).to_be_bytes().iter().chain(p.iter()) {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

pub(crate) fn rayleigh(u: f64, sigma: f64) -> f64 {
    sigma * m::sqrt(-2.0 * m::ln(1.0 - u))
}

fn band_key(disc: &Disc, r: f64) -> String {
    disc.bands
        .iter()
        .find(|b| r >= b.inner_au && r < b.outer_au)
        .unwrap_or_else(|| disc.bands.last().expect("bands"))
        .key
        .clone()
}

/// Take `n` units from a water list proportionally (floor, then the remainder from the largest).
pub(crate) fn take_water(from: &mut WaterFrom, n: u64) -> WaterFrom {
    let total: u64 = from.iter().map(|x| x.1).sum();
    assert!(n <= total, "water books: want {n} of {total}");
    if n == 0 {
        return vec![];
    }
    let mut out: Vec<(String, u64)> = from
        .iter()
        .map(|(b, x)| (b.clone(), ((*x as u128 * n as u128) / total as u128) as u64))
        .collect();
    let mut left = n - out.iter().map(|x| x.1).sum::<u64>();
    while left > 0 {
        // Largest remaining capacity first, first in list on ties.
        let mut best = 0;
        let mut cap = 0;
        for (i, (_, x)) in from.iter().enumerate() {
            let c = x - out[i].1;
            if c > cap {
                cap = c;
                best = i;
            }
        }
        let k = left.min(cap);
        out[best].1 += k;
        left -= k;
    }
    for (i, (_, x)) in out.iter().enumerate() {
        from[i].1 -= x;
    }
    from.retain(|x| x.1 > 0);
    out.retain(|x| x.1 > 0);
    out
}

pub(crate) fn world_water(w: &World) -> WaterFrom {
    let mut v = vec![];
    w.ledger.water_by_band(&mut v);
    v.into_iter()
        .filter(|x| x.1 > 0)
        .map(|(b, x)| (b, x as u64))
        .collect()
}

pub(crate) fn add(a: &mut [u64; 6], b: &[u64; 6]) {
    for g in 0..6 {
        a[g] += b[g];
    }
}

pub(crate) fn sum6(a: &[u64; 6]) -> u64 {
    a.iter().sum()
}

/// Proportional share of each group.
fn share(from: &[u64; 6], f: f64) -> [u64; 6] {
    from.map(|x| m::floor(x as f64 * f) as u64)
}

/// Take `want` units from the layers in `order`, outermost first.
fn strip(solid: &mut [u64; 6], mut want: u64, order: &[usize]) -> [u64; 6] {
    let mut out = [0u64; 6];
    for &g in order {
        let x = want.min(solid[g]);
        solid[g] -= x;
        out[g] = x;
        want -= x;
    }
    out
}

/// Take `want` units proportionally from the allowed groups.
pub(crate) fn take_prop(solid: &mut [u64; 6], want: u64, allowed: &[usize]) -> [u64; 6] {
    let total: u64 = allowed.iter().map(|g| solid[*g]).sum();
    let mut out = [0u64; 6];
    if total == 0 || want == 0 {
        return out;
    }
    let want = want.min(total);
    for &g in allowed {
        out[g] = ((solid[g] as u128 * want as u128) / total as u128) as u64;
    }
    let mut left = want - sum6(&out);
    for &g in allowed {
        let k = left.min(solid[g] - out[g]);
        out[g] += k;
        left -= k;
    }
    for g in 0..6 {
        solid[g] -= out[g];
    }
    out
}

struct Ctx<'a> {
    d: Draws,
    system: &'a Address,
    host: f64,
    v_unit: f64,
    engulf_au: f64,
    worlds: Vec<World>,
    belts: Vec<Belt>,
    rogues: Vec<Rogue>,
    moons: Vec<ImpactMoon>,
    reservoirs: Vec<Reservoir>,
    events: Vec<UpheavalEvent>,
    budget: UpheavalBudget,
    dy: Dynamics,
    k: u64,
}

impl Ctx<'_> {
    fn ev(&self, slot: u64) -> f64 {
        self.d
            .unit(fnv(&[b"ev", &self.k.to_be_bytes(), &slot.to_be_bytes()]))
    }
    fn v_kep(&self, a: f64) -> f64 {
        V_KEP_1AU * m::sqrt(self.host / a)
    }
    fn theta(&self, w: &World) -> f64 {
        (w.mass_earth() / (MSUN_IN_EARTH * self.host)) * (w.a_au / (w.radius_earth() * R_EARTH_AU))
    }
    fn hill(&self, i: usize) -> f64 {
        let (a, b) = (&self.worlds[i], &self.worlds[i + 1]);
        let rh = m::cbrt((a.mass_earth() + b.mass_earth()) / (3.0 * MSUN_IN_EARTH * self.host))
            * 0.5
            * (a.a_au + b.a_au);
        (b.a_au * (1.0 - b.e) - a.a_au * (1.0 + a.e)) / rh
    }
    fn protected(&self, i: usize) -> bool {
        matches!(&self.worlds[i + 1].resonance, Some((id, _, _)) if *id == self.worlds[i].id)
    }
    fn sort(&mut self) {
        self.worlds.sort_by(|x, y| x.a_au.total_cmp(&y.a_au));
    }
    fn push(&mut self, e: UpheavalEvent) -> usize {
        self.events.push(e);
        self.events.len() - 1
    }

    /// Stir a neighbour pair to new eccentricities, spreading the orbits so that
    /// the pair's energy and angular momentum are both kept exactly (spreading
    /// at fixed energy raises angular momentum; higher eccentricity lowers it).
    fn stir_pair(&mut self, i: usize, e1: f64, e2: f64) -> bool {
        let (m1, m2) = (self.worlds[i].mass_earth(), self.worlds[i + 1].mass_earth());
        let (a1, a2) = (self.worlds[i].a_au, self.worlds[i + 1].a_au);
        let c_e = m1 / a1 + m2 / a2;
        let c_l = self.worlds[i].angmom() + self.worlds[i + 1].angmom();
        let (k1, k2) = (1.0 - e1 * e1, 1.0 - e2 * e2);
        let f = |x: f64| {
            let y = m2 / (c_e - m1 / x);
            m1 * m::sqrt(x * k1) + m2 * m::sqrt(y * k2) - c_l
        };
        if f(a1) >= 0.0 {
            return false;
        }
        // Bisection between the energy limit and the inner planet's orbit, fixed 80 steps.
        let (mut lo, mut hi) = (m1 / c_e * (1.0 + 1e-12), a1);
        if f(lo) <= 0.0 {
            return false;
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if f(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let x = hi;
        let y = m2 / (c_e - m1 / x);
        if y / x > 4.0 {
            return false;
        }
        let w = &mut self.worlds[i];
        w.a_au = x;
        w.e = e1;
        let w = &mut self.worlds[i + 1];
        w.a_au = y;
        w.e = e2;
        // Angular momentum left over from the bisection's last step goes to the bound
        // books through the final recount; it is below 1e-15 of the pair's.
        true
    }

    /// The earliest instability among unprotected neighbour pairs.
    fn next_pair(&self) -> Option<(f64, usize)> {
        let n = self.worlds.len();
        if n < 2 {
            return None;
        }
        let deltas: Vec<f64> = (0..n - 1).map(|i| self.hill(i)).collect();
        let mut best: Option<(f64, usize)> = None;
        for i in 0..n - 1 {
            if self.protected(i) {
                continue;
            }
            let (a, b) = (&self.worlds[i], &self.worlds[i + 1]);
            let dl = deltas[i];
            let multi = dl < DELTA_MULTI
                && ((i > 0 && deltas[i - 1] < DELTA_MULTI)
                    || (i + 2 < n && deltas[i + 1] < DELTA_MULTI));
            let key = fnv(&[
                b"pair",
                a.id.as_bytes(),
                b.id.as_bytes(),
                &a.version.to_be_bytes(),
                &b.version.to_be_bytes(),
            ]);
            let p_yr = m::pow(a.a_au, 1.5) / m::sqrt(self.host);
            let log_orbits = if multi {
                0.951 * dl.max(0.0) - 1.202 + T_SCATTER_DEX * self.d.normal(key >> 2)
            } else if dl < 2.0 * m::sqrt(3.0) {
                1.0 + 0.5 * self.d.unit(key)
            } else {
                continue;
            };
            let start = a.changed_myr.max(b.changed_myr);
            let t = start + p_yr * m::pow(10.0, log_orbits) * 1e-6;
            if best.is_none_or(|(bt, _)| t < bt) {
                best = Some((t, i));
            }
        }
        best
    }

    fn lose(&mut self, wi: usize, t: f64, cause: &str, solid: [u64; 6], gas: u64) -> WaterFrom {
        let mut wf = world_water(&self.worlds[wi]);
        let water = take_water(&mut wf, solid[W]);
        let w = &mut self.worlds[wi];
        for (x, y) in w.solid.iter_mut().zip(solid.iter()) {
            *x -= y;
        }
        w.gas -= gas;
        w.ledger.entries.push(WorldEntry::Lost {
            t_myr: t,
            cause: cause.into(),
            solid,
            gas,
            water_from: water.clone(),
        });
        water
    }

    /// Remove mass from a world as debris at its orbit, booking energy and angular momentum.
    fn shed(&mut self, wi: usize, t: f64, cause: &str, solid: [u64; 6], gas: u64) -> WaterFrom {
        let (e0, l0) = (self.worlds[wi].energy(), self.worlds[wi].angmom());
        let water = self.lose(wi, t, cause, solid, gas);
        let w = &self.worlds[wi];
        self.dy.energy_debris += e0 - w.energy();
        self.dy.angmom_debris += l0 - w.angmom();
        water
    }

    fn debris_belt(&mut self, a: f64, origin: &str, solid: [u64; 6], water_from: WaterFrom) {
        if sum6(&solid) == 0 {
            return;
        }
        let id = format!("debris-{}", self.belts.len());
        self.belts.push(Belt {
            id,
            a_au: a,
            origin: origin.into(),
            solid,
            water_from,
            stirred: false,
        });
    }

    fn engulf(&mut self, t: f64) {
        let mut i = 0;
        while i < self.worlds.len() {
            let w = &self.worlds[i];
            if w.a_au * (1.0 - w.e) < self.engulf_au {
                let w = self.worlds.remove(i);
                self.dy.energy_star += w.energy();
                self.dy.angmom_star += w.angmom();
                add(&mut self.budget.solid_to_star, &w.solid);
                self.budget.gas_to_star += w.gas;
                let cause = w.last_event;
                self.push(UpheavalEvent::Engulfed {
                    t_myr: t,
                    cause,
                    id: w.id,
                });
            } else {
                i += 1;
            }
        }
    }

    fn collide(&mut self, t: f64, i: usize, j: usize, cause: Option<usize>) {
        let (ib, is) = if self.worlds[i].mass_earth() >= self.worlds[j].mass_earth() {
            (i, j)
        } else {
            (j, i)
        };
        let (u_angle, u_v, u_e, u_moon, u_moonf, u_gas) = (
            self.ev(0),
            self.ev(1),
            self.ev(2),
            self.ev(3),
            self.ev(4),
            self.ev(5),
        );
        let big = &self.worlds[ib];
        let small = &self.worlds[is];
        let (m1, m2) = (big.mass_earth(), small.mass_earth());
        let (r1, r2) = (big.radius_earth(), small.radius_earth());
        let mt = m1 + m2;
        let gamma = m2 / mt;
        let gam = 1.0 - 2.0 * gamma;
        let theta = 0.5 * m::acos(1.0 - 2.0 * u_angle);
        let s = m::sin(theta);
        let vesc = V_ESC_EARTH * m::sqrt(mt / (r1 + r2));
        let abar = (m1 * big.a_au + m2 * small.a_au) / mt;
        let vk = self.v_kep(abar);
        let e_rel = m::hypot(big.e, small.e);
        let v_inf = m::hypot(u_v * vesc, e_rel * vk);
        let vimp = m::hypot(vesc, v_inf);
        let ratio = vimp / vesc;
        let x = m::pow(1.0 - s, 2.5);
        let vcr = 2.43 * gam * x - 0.0408 * gam + 1.86 * x + 1.08;
        let angle_deg = theta * 180.0 / m::PI;
        let (bid, sid) = (big.id.clone(), small.id.clone());
        let e_before = big.energy() + small.energy();
        let l_tot = big.angmom() + small.angmom();
        let (a_big, a_small) = (big.a_au, small.a_au);

        if ratio >= vcr && s > r1 / (r1 + r2) {
            // Hit and run: the runner loses mantle, both go on.
            let f = (0.2 * (ratio - 1.0)).clamp(0.02, 0.5);
            let want = m::floor(sum6(&self.worlds[is].solid) as f64 * f) as u64;
            let mut sol = self.worlds[is].solid;
            let lost = strip(&mut sol, want, &LAYERS);
            let gas = self.worlds[is].gas;
            let a = self.worlds[is].a_au;
            let wf = self.shed(is, t, "hit-and-run", lost, gas);
            self.debris_belt(a, &format!("impact:{sid}"), lost, wf);
            self.budget.gas_lost += gas;
            let ev = self.push(UpheavalEvent::Impact {
                t_myr: t,
                cause,
                target: bid,
                impactor: sid,
                outcome: "hit-and-run".into(),
                v_over_vesc: ratio,
                angle_deg,
                debris_earth: earth(sum6(&lost)),
            });
            for k in [ib, is] {
                self.worlds[k].bump(t, ev);
                self.worlds[k].impacts += 1;
                self.worlds[k].last_impact_myr = Some(t);
            }
            return;
        }

        // Merge or erosion: the two become one, then shed debris.
        let mu = m1 * m2 / mt;
        // Momentum (so angular momentum) is kept exactly; the orbit can lose energy
        // to heat but not gain it, which caps the eccentricity left over.
        let a_e = mt / (m1 / a_big + m2 / a_small);
        let lm = l_tot / mt;
        let e_d = rayleigh(u_e, RESIDUAL_E * vesc / vk).min(0.9);
        let (a_new, e_new, l_excess) = if lm * lm <= a_e {
            let e_max = m::sqrt(1.0 - lm * lm / a_e);
            let e = e_d.min(e_max);
            (lm * lm / (1.0 - e * e), e, 0.0)
        } else {
            (a_e, 0.0, l_tot - mt * m::sqrt(a_e))
        };
        let small_w = self.worlds.remove(is);
        let ib = if is < ib { ib - 1 } else { ib };
        let w = &mut self.worlds[ib];
        add(&mut w.solid, &small_w.solid);
        w.gas += small_w.gas;
        w.ledger.entries.push(WorldEntry::Merged {
            world: Box::new(small_w.ledger),
        });
        w.a_au = a_new;
        w.e = e_new;
        w.impacts += 1;
        w.last_impact_myr = Some(t);
        w.scattered |= small_w.scattered;
        let e_after = w.energy();
        self.dy.energy_heat += e_before - e_after;
        self.dy.angmom_debris += l_excess;

        // Largest remnant by Leinhardt and Stewart 2012.
        let merge = ratio < vcr;
        let mut keep_frac = 1.0;
        if !merge {
            let mt_kg = mt * M_EARTH_KG;
            let mu_kg = mu * M_EARTH_KG;
            let v = vimp * 1000.0;
            let q_r = 0.5 * mu_kg * v * v / mt_kg;
            let rc1 = m::cbrt(3.0 * mt_kg / (4.0 * m::PI * 1000.0));
            let q_star = 1.9 * 0.8 * m::PI * 1000.0 * G * rc1 * rc1;
            keep_frac = if q_r < 1.8 * q_star {
                1.0 - q_r / (2.0 * q_star)
            } else {
                0.1 * m::pow(q_r / (1.8 * q_star), -1.5)
            }
            .clamp(0.0, 1.0);
        }
        let total = sum6(&self.worlds[ib].solid);
        let a_here = self.worlds[ib].a_au;
        let gas_loss = m::floor(self.worlds[ib].gas as f64 * (0.1 + 0.4 * u_gas)) as u64;
        let (outcome, debris) = if keep_frac < 0.1 {
            // Both shattered into a belt.
            let w = self.worlds.remove(ib);
            self.dy.energy_debris += w.energy();
            self.dy.angmom_debris += w.angmom();
            self.budget.gas_lost += w.gas;
            let wf = world_water(&w);
            self.debris_belt(w.a_au, &format!("impact:{bid}"), w.solid, wf);
            ("shattered", sum6(&w.solid))
        } else {
            let want = total - m::floor(total as f64 * keep_frac) as u64;
            let mut sol = self.worlds[ib].solid;
            let lost = strip(&mut sol, want, &LAYERS);
            let gas = if merge { gas_loss } else { self.worlds[ib].gas };
            let wf = self.shed(ib, t, "impact", lost, gas);
            self.budget.gas_lost += gas;
            self.debris_belt(a_here, &format!("impact:{bid}"), lost, wf);
            (if merge { "merge" } else { "erosion" }, sum6(&lost))
        };
        let mut outcome = outcome.to_string();
        // A large moon from an oblique merging impact (Canup 2004).
        if merge
            && (0.5..0.866).contains(&s)
            && (0.05..0.5).contains(&gamma)
            && !self.worlds[ib].is_giant()
            && u_moon < MOON_CHANCE
        {
            let mass = sum6(&self.worlds[ib].solid);
            let want = m::floor(mass as f64 * (0.005 + 0.015 * u_moonf)) as u64;
            let mut sol = self.worlds[ib].solid;
            let taken = take_prop(&mut sol, want, &[1, 2, 3]);
            // The moon leaves the world's orbital books under debris (it is its own object now).
            let wf = self.shed(ib, t, "moon", taken, 0);
            self.moons.push(ImpactMoon {
                parent: bid.clone(),
                t_myr: t,
                solid: taken,
                water_from: wf,
                angle_deg,
                v_over_vesc: ratio,
            });
            outcome = "merge-moon".into();
        }
        let ev = self.push(UpheavalEvent::Impact {
            t_myr: t,
            cause,
            target: bid.clone(),
            impactor: sid,
            outcome: outcome.clone(),
            v_over_vesc: ratio,
            angle_deg,
            debris_earth: earth(debris),
        });
        if outcome != "shattered" {
            self.worlds[ib].bump(t, ev);
        }
        self.sort();
    }

    fn scatter(&mut self, t: f64, i: usize, j: usize, theta: f64, cause: Option<usize>) {
        let (ib, is) = if self.worlds[i].mass_earth() >= self.worlds[j].mass_earth() {
            (i, j)
        } else {
            (j, i)
        };
        let p_eject = EJECT_SHARE * theta * theta / (1.0 + theta * theta);
        let eject = self.ev(10) < p_eject;
        let (mb, ms) = (self.worlds[ib].mass_earth(), self.worlds[is].mass_earth());
        let (eb, es) = (self.worlds[ib].energy(), self.worlds[is].energy());
        let l_tot = self.worlds[ib].angmom() + self.worlds[is].angmom();
        let ab = self.worlds[ib].a_au;
        if !eject {
            for tr in 0..8u64 {
                let out = self.ev(20 + 5 * tr) < 0.6;
                let a_s = if out {
                    ab * (1.3 + 2.7 * self.ev(21 + 5 * tr))
                } else {
                    ab * (0.25 + 0.55 * self.ev(21 + 5 * tr))
                };
                let eb2 = eb + es + ms / (2.0 * a_s);
                if eb2 >= 0.0 {
                    continue;
                }
                let ab2 = -mb / (2.0 * eb2);
                let e_s = if out {
                    1.0 - ab2 * (0.9 + 0.7 * self.ev(22 + 5 * tr)) / a_s
                } else {
                    ab2 * (0.6 + 0.5 * self.ev(22 + 5 * tr)) / a_s - 1.0
                };
                if !(0.0..0.95).contains(&e_s) {
                    continue;
                }
                let l_s = ms * m::sqrt(a_s * (1.0 - e_s * e_s));
                let l_b = l_tot - l_s;
                if l_b <= 0.0 {
                    continue;
                }
                let xx = (l_b / mb) * (l_b / mb) / ab2;
                if xx > 1.0 {
                    continue;
                }
                let e_b = m::sqrt(1.0 - xx);
                let sid = self.worlds[is].id.clone();
                let bid = self.worlds[ib].id.clone();
                {
                    let w = &mut self.worlds[ib];
                    w.a_au = ab2;
                    w.e = e_b;
                    w.scattered = true;
                }
                {
                    let w = &mut self.worlds[is];
                    w.a_au = a_s;
                    w.e = e_s;
                    w.scattered = true;
                }
                let ev = self.push(UpheavalEvent::Scatter {
                    t_myr: t,
                    cause,
                    kept: bid,
                    thrown: sid,
                    to_au: a_s,
                    e: e_s,
                });
                self.worlds[ib].bump(t, ev);
                self.worlds[is].bump(t, ev);
                self.sort();
                return;
            }
        }
        // Ejection of the lighter body.
        let a_s = self.worlds[is].a_au;
        let v_inf = self.v_kep(a_s) * (0.1 + 0.25 * self.ev(11));
        let e_rogue = 0.5 * ms * (v_inf / self.v_unit) * (v_inf / self.v_unit);
        let eb2 = eb + es - e_rogue;
        let ab2 = -mb / (2.0 * eb2);
        let e_b = rayleigh(self.ev(12), 0.3).min(0.9);
        let l_b = mb * m::sqrt(ab2 * (1.0 - e_b * e_b));
        let bid = self.worlds[ib].id.clone();
        {
            let w = &mut self.worlds[ib];
            w.a_au = ab2;
            w.e = e_b;
            w.scattered = true;
        }
        self.dy.energy_rogues += e_rogue;
        self.dy.angmom_rogues += l_tot - l_b;
        let s = self.worlds.remove(is);
        let ib = if is < ib { ib - 1 } else { ib };
        let address = self
            .system
            .child("rogue", self.rogues.len() as u64)
            .expect("valid");
        add(&mut self.budget.solid_in_rogues, &s.solid);
        self.budget.gas_in_rogues += s.gas;
        let ev = self.push(UpheavalEvent::Ejected {
            t_myr: t,
            cause,
            id: s.id.clone(),
            by: bid.clone(),
            v_inf_km_s: v_inf,
            rogue: address.to_string(),
        });
        self.rogues.push(Rogue {
            address,
            id: s.id,
            t_myr: t,
            by: bid,
            v_inf_km_s: v_inf,
            solid: s.solid,
            gas: s.gas,
            ledger: s.ledger,
        });
        self.worlds[ib].bump(t, ev);
        self.sort();
    }

    /// A giant stirs a belt: the bombardment spike, reservoirs and captures.
    fn stir(&mut self, bi: usize, by: &str, t: f64, cause: Option<usize>) {
        let removed = share(&self.belts[bi].solid, STIR_DEPLETION);
        let total = sum6(&removed);
        self.belts[bi].stirred = true;
        if total == 0 {
            return;
        }
        let belt_id = self.belts[bi].id.clone();
        let belt_a = self.belts[bi].a_au;
        let mut left = removed;
        for (x, y) in self.belts[bi].solid.iter_mut().zip(removed.iter()) {
            *x -= y;
        }
        // Water removed comes out of the belt's list; what stays is the rest.
        let mut water = take_water(&mut self.belts[bi].water_from, removed[W]);

        let take = |f: f64, left: &mut [u64; 6], water: &mut WaterFrom| -> ([u64; 6], WaterFrom) {
            let mut s = share(&removed, f);
            for g in 0..6 {
                s[g] = s[g].min(left[g]);
                left[g] -= s[g];
            }
            let wf = take_water(water, s[W]);
            (s, wf)
        };
        // Hits on every world inside or near the belt (Opik-style focused cross-sections).
        let earth_focus =
            1.0 + (V_ESC_EARTH / (0.3 * V_KEP_1AU)) * (V_ESC_EARTH / (0.3 * V_KEP_1AU));
        let mut hits = vec![];
        for (wi, w) in self.worlds.iter().enumerate() {
            if w.a_au > belt_a * 1.3 {
                continue;
            }
            let r = w.radius_earth();
            let vrel = 0.3 * self.v_kep(w.a_au);
            let vesc = V_ESC_EARTH * m::sqrt(w.mass_earth() / r);
            let focus = 1.0 + (vesc / vrel) * (vesc / vrel);
            let h = (HIT_EARTH * r * r * focus / earth_focus / w.a_au).min(0.05);
            hits.push((wi, h));
        }
        let mut delivered = vec![];
        for (wi, h) in hits {
            let (s, wf) = take(h, &mut left, &mut water);
            if sum6(&s) == 0 {
                continue;
            }
            delivered.push((wi, s, wf));
        }
        let oort = take(OORT_SHARE, &mut left, &mut water);
        let scat = take(SCATTERED_SHARE, &mut left, &mut water);
        let star = take(TO_STAR_SHARE, &mut left, &mut water);
        let giants: Vec<String> = self
            .worlds
            .iter()
            .filter(|w| w.is_giant())
            .map(|w| w.id.clone())
            .collect();
        let mut caps = vec![];
        for gid in &giants {
            if gid == by {
                caps.push((
                    gid.clone(),
                    "trojans",
                    take(TROJAN_CAPTURE, &mut left, &mut water),
                ));
            }
            caps.push((
                gid.clone(),
                "irregular-moons",
                take(IRREGULAR_CAPTURE, &mut left, &mut water),
            ));
        }
        // The rest is thrown out of the system as small bodies.
        add(&mut self.budget.solid_ejected, &left);
        add(&mut self.budget.solid_to_star, &star.0);

        let ev = self.push(UpheavalEvent::BeltStirred {
            t_myr: t,
            cause,
            belt: belt_id.clone(),
            by: by.into(),
            removed_earth: earth(total),
            oort_earth: earth(sum6(&oort.0)),
        });
        for (kind, (s, wf)) in [("oort", oort), ("scattered-disc", scat)] {
            if sum6(&s) > 0 {
                self.reservoirs.push(Reservoir {
                    kind: kind.into(),
                    of: None,
                    from_belt: belt_id.clone(),
                    t_myr: t,
                    solid: s,
                    water_from: wf,
                });
            }
        }
        for (gid, kind, (s, wf)) in caps {
            if sum6(&s) == 0 {
                continue;
            }
            self.push(UpheavalEvent::Captured {
                t_myr: t,
                cause: Some(ev),
                by: gid.clone(),
                kind: kind.into(),
                earth: earth(sum6(&s)),
            });
            self.reservoirs.push(Reservoir {
                kind: kind.into(),
                of: Some(gid),
                from_belt: belt_id.clone(),
                t_myr: t,
                solid: s,
                water_from: wf,
            });
        }
        for (wi, s, wf) in delivered {
            let (e0, l0) = (self.worlds[wi].energy(), self.worlds[wi].angmom());
            let w = &mut self.worlds[wi];
            add(&mut w.solid, &s);
            w.ledger.entries.push(WorldEntry::Delivered {
                t_myr: t,
                belt: belt_id.clone(),
                solid: s,
                water_from: wf,
            });
            self.dy.energy_belts += e0 - w.energy();
            self.dy.angmom_belts += l0 - w.angmom();
            let id = w.id.clone();
            let water_earth = earth(s[W]);
            self.push(UpheavalEvent::Bombardment {
                t_myr: t,
                cause: Some(ev),
                world: id,
                earth: earth(sum6(&s)),
                water_earth,
                efold_myr: SPIKE_EFOLD_MYR,
            });
        }
    }

    /// Belts a giant's new orbit sweeps through.
    fn sweep(&mut self, t: f64) {
        for gi in 0..self.worlds.len() {
            let w = &self.worlds[gi];
            if !w.is_giant() || w.last_event.is_none() || w.changed_myr != t {
                continue;
            }
            let (q, big_q) = (w.a_au * (1.0 - w.e) * 0.85, w.a_au * (1.0 + w.e) * 1.15);
            let (id, cause) = (w.id.clone(), w.last_event);
            for bi in 0..self.belts.len() {
                let b = &self.belts[bi];
                if !b.stirred && b.a_au >= q && b.a_au <= big_q {
                    self.stir(bi, &id, t, cause);
                }
            }
        }
    }

    /// Nice trigger: the outer two giants cross a resonance; the outer one migrates into the belt.
    fn nice(&mut self, t: f64) {
        let gs: Vec<usize> = (0..self.worlds.len())
            .filter(|i| self.worlds[*i].is_giant())
            .collect();
        if gs.len() < 2 {
            return;
        }
        let (g1, g2) = (gs[gs.len() - 2], gs[gs.len() - 1]);
        let a2 = self.worlds[g2].a_au;
        let Some(bi) = (0..self.belts.len())
            .filter(|b| self.belts[*b].a_au > a2 && !self.belts[*b].stirred)
            .max_by(|x, y| sum6(&self.belts[*x].solid).cmp(&sum6(&self.belts[*y].solid)))
        else {
            return;
        };
        let belt_m = earth(sum6(&self.belts[bi].solid));
        let belt_a = self.belts[bi].a_au;
        let (u1, u2, u3) = (self.ev(30), self.ev(31), self.ev(32));
        let before: f64 = self.worlds[g1].energy() + self.worlds[g2].energy();
        let lbefore: f64 = self.worlds[g1].angmom() + self.worlds[g2].angmom();
        {
            let w = &mut self.worlds[g1];
            w.a_au *= 1.0 - (0.02 + 0.03 * u1);
            w.e = 0.03 + 0.07 * u2;
        }
        let from = self.worlds[g2].a_au;
        {
            let w = &mut self.worlds[g2];
            let mw = w.mass_earth();
            let e2 = 0.03 + 0.07 * u3;
            let l = w.angmom() + NICE_KAPPA * belt_m * m::sqrt(from);
            w.e = e2;
            w.a_au = ((l / mw) * (l / mw) / (1.0 - e2 * e2)).min(belt_a * 1.3);
        }
        let to = self.worlds[g2].a_au;
        self.dy.energy_belts += before - self.worlds[g1].energy() - self.worlds[g2].energy();
        self.dy.angmom_belts += lbefore - self.worlds[g1].angmom() - self.worlds[g2].angmom();
        let (i1, i2) = (self.worlds[g1].id.clone(), self.worlds[g2].id.clone());
        let ev = self.push(UpheavalEvent::GiantsCrossResonance {
            t_myr: t,
            inner: i1,
            outer: i2.clone(),
            outer_from_au: from,
            outer_to_au: to,
        });
        self.worlds[g1].bump(t, ev);
        self.worlds[g2].bump(t, ev);
        self.stir(bi, &i2, t, Some(ev));
        self.sort();
    }
}

/// Run the violent phase on what F3 grew.
pub fn upheave(seed: &Seed, system: &Address, disc: &Disc, g: &Growth) -> Upheaval {
    let d = Draws::new(
        *seed,
        system.clone(),
        Stream::new("formation.upheaval.v1").expect("valid"),
    );
    let host = disc.params.host_mass;
    let t0 = g.end_myr;
    let worlds: Vec<World> = g
        .bodies
        .iter()
        .map(|b| World {
            id: b.id.clone(),
            a_au: b.a_au,
            e: b.e,
            solid: b.solid,
            gas: b.gas,
            ledger: WorldLedger {
                id: b.id.clone(),
                entries: vec![WorldEntry::Grown {
                    ledger: b.ledger.clone(),
                }],
            },
            resonance: b.resonance.map(|(j, p, q)| (g.bodies[j].id.clone(), p, q)),
            impacts: 0,
            last_impact_myr: None,
            scattered: false,
            spin_hours: 0.0,
            obliquity_deg: 0.0,
            version: 0,
            changed_myr: t0,
            last_event: None,
        })
        .collect();
    let belts: Vec<Belt> = g
        .rings
        .iter()
        .enumerate()
        .map(|(k, r)| Belt {
            id: format!("belt-{k}"),
            a_au: r.a_au,
            origin: r.held_by.clone(),
            solid: r.solid,
            water_from: if r.solid[W] > 0 {
                vec![(band_key(disc, r.a_au), r.solid[W])]
            } else {
                vec![]
            },
            stirred: false,
        })
        .collect();
    let mut budget = UpheavalBudget::default();
    for w in &worlds {
        add(&mut budget.solid_start, &w.solid);
        budget.gas_start += w.gas;
    }
    for b in &belts {
        add(&mut budget.solid_start, &b.solid);
    }
    let dy = Dynamics {
        energy_start: worlds.iter().map(|w| w.energy()).sum(),
        angmom_start: worlds.iter().map(|w| w.angmom()).sum(),
        ..Dynamics::default()
    };
    let r_star_au = 0.004_65 * m::pow(host, 0.8);
    let mut c = Ctx {
        d,
        system,
        host,
        v_unit: V_KEP_1AU * m::sqrt(host),
        engulf_au: (2.0 * r_star_au).max(0.005),
        worlds,
        belts,
        rogues: vec![],
        moons: vec![],
        reservoirs: vec![],
        events: vec![],
        budget,
        dy,
        k: 0,
    };

    // Step 1 and 2: is it primed, and when does it break?
    let n = c.worlds.len();
    let min_spacing = (0..n.saturating_sub(1))
        .map(|i| c.hill(i))
        .fold(f64::INFINITY, f64::min);
    let chained = c.worlds.iter().any(|w| w.resonance.is_some());
    let chain_break = if chained && c.d.unit(fnv(&[b"chain"])) < CHAIN_BREAK_FRACTION {
        Some(t0 + m::pow(10.0, 2.0 * c.d.unit(fnv(&[b"chain-t"]))))
    } else {
        None
    };
    let giants: Vec<usize> = (0..n).filter(|i| c.worlds[*i].is_giant()).collect();
    let outer_belt = giants.last().map_or(0.0, |&gi| {
        let a = c.worlds[gi].a_au;
        c.belts
            .iter()
            .filter(|b| b.a_au > a)
            .map(|b| earth(sum6(&b.solid)))
            .fold(0.0, f64::max)
    });
    let nice = if giants.len() >= 2
        && outer_belt > 0.0
        && c.d.unit(fnv(&[b"nice"])) < 1.0 - m::exp(-outer_belt / NICE_BELT_EARTH)
    {
        Some(t0 + m::pow(10.0, 1.0 + 1.7 * c.d.unit(fnv(&[b"nice-t"]))))
    } else {
        None
    };
    let priming = Priming {
        min_spacing: if min_spacing.is_finite() {
            min_spacing
        } else {
            0.0
        },
        chained,
        giants: giants.len() as u32,
        outer_belt_earth: outer_belt,
        chain_break_myr: chain_break,
        nice_myr: nice,
    };

    // Step 3: the cascade.
    let mut pending: Vec<(f64, u8)> = vec![];
    if let Some(t) = chain_break {
        pending.push((t, 0));
    }
    if let Some(t) = nice {
        pending.push((t, 1));
    }
    let horizon = t0 + HORIZON_MYR;
    let mut first_break: Option<f64> = None;
    let mut last_t = t0;
    while c.k < MAX_EVENTS {
        let pair = c.next_pair();
        pending.sort_by(|x, y| x.0.total_cmp(&y.0));
        let trig = pending.first().copied();
        let t_pair = pair.map_or(f64::INFINITY, |p| p.0);
        let t_trig = trig.map_or(f64::INFINITY, |p| p.0);
        let t = t_pair.min(t_trig);
        if t > horizon {
            break;
        }
        first_break.get_or_insert(t);
        last_t = t;
        if t_trig <= t_pair {
            pending.remove(0);
            match trig.expect("trigger").1 {
                0 => {
                    let ids: Vec<String> = c
                        .worlds
                        .iter()
                        .filter(|w| w.resonance.is_some())
                        .map(|w| w.id.clone())
                        .collect();
                    let ev = c.push(UpheavalEvent::ChainBreaks { t_myr: t, ids });
                    // Breaking stirs each chained pair to eccentricities of order
                    // v_esc/v_kep, spreading it (energy and angular momentum kept).
                    for i in 0..c.worlds.len().saturating_sub(1) {
                        if !c.protected(i) {
                            continue;
                        }
                        let sig = |w: &World| {
                            RESIDUAL_E * V_ESC_EARTH * m::sqrt(w.mass_earth() / w.radius_earth())
                                / (V_KEP_1AU * m::sqrt(host / w.a_au))
                        };
                        let key = fnv(&[b"stir", c.worlds[i].id.as_bytes()]);
                        let e1 = c.worlds[i]
                            .e
                            .max(rayleigh(c.d.unit(key), sig(&c.worlds[i])).min(0.3));
                        let e2 = c.worlds[i + 1]
                            .e
                            .max(rayleigh(c.d.unit(key ^ 1), sig(&c.worlds[i + 1])).min(0.3));
                        let (eb, lb) = (
                            c.worlds[i].energy() + c.worlds[i + 1].energy(),
                            c.worlds[i].angmom() + c.worlds[i + 1].angmom(),
                        );
                        if c.stir_pair(i, e1, e2) {
                            c.dy.energy_heat +=
                                eb - c.worlds[i].energy() - c.worlds[i + 1].energy();
                            c.dy.angmom_debris +=
                                lb - c.worlds[i].angmom() - c.worlds[i + 1].angmom();
                        }
                    }
                    for w in c.worlds.iter_mut() {
                        if w.resonance.is_some() {
                            w.bump(t, ev);
                        }
                    }
                }
                _ => c.nice(t),
            }
        } else {
            let (_, i) = pair.expect("pair");
            let cause = c.worlds[i].last_event.max(c.worlds[i + 1].last_event);
            let th = {
                let (a, b) = (&c.worlds[i], &c.worlds[i + 1]);
                if a.mass_earth() >= b.mass_earth() {
                    c.theta(a)
                } else {
                    c.theta(b)
                }
            };
            if c.ev(9) < 1.0 / (1.0 + th * th) {
                c.collide(t, i, i + 1, cause);
            } else {
                c.scatter(t, i, i + 1, th, cause);
            }
        }
        c.engulf(t);
        c.sweep(t);
        c.k += 1;
    }

    // Step 4: settling. Tides circularise close pericentres (angular momentum kept).
    let settle_t = if first_break.is_some() {
        last_t + 10.0 + 90.0 * c.d.unit(fnv(&[b"settle"]))
    } else {
        t0
    };
    for wi in 0..c.worlds.len() {
        let w = &c.worlds[wi];
        if w.e > 0.05 && w.a_au * (1.0 - w.e) < CIRC_Q_AU {
            let e0 = w.energy();
            let a2 = w.a_au * (1.0 - w.e * w.e);
            let cause = w.last_event;
            let id = w.id.clone();
            let w = &mut c.worlds[wi];
            w.a_au = a2;
            w.e = 0.0;
            let e1 = w.energy();
            c.dy.energy_heat += e0 - e1;
            c.push(UpheavalEvent::Circularised {
                t_myr: settle_t,
                cause,
                id,
                a_au: a2,
            });
        }
    }
    c.sort();
    // Spins and tilts (Kokubo and Genda 2010; Bryan et al. 2018).
    for w in c.worlds.iter_mut() {
        let key = |s: &[u8]| fnv(&[b"spin", w.id.as_bytes(), s]);
        let (u1, u2) = (c.d.unit(key(b"w")), c.d.unit(key(b"o")));
        let r = w.radius_earth() * R_EARTH_M;
        let mass = w.mass_earth() * M_EARTH_KG;
        let p_crit_h = 2.0 * m::PI / m::sqrt(G * mass / (r * r * r)) / 3600.0;
        let (frac, obl) = if w.is_giant() {
            (
                0.1 + 0.25 * u1,
                if w.scattered { 30.0 * u2 } else { 5.0 * u2 },
            )
        } else if w.impacts > 0 {
            (0.3 + 0.7 * u1, m::acos(1.0 - 2.0 * u2) * 180.0 / m::PI)
        } else {
            (0.05 + 0.35 * u1, 10.0 * u2)
        };
        w.spin_hours = p_crit_h / frac;
        w.obliquity_deg = obl;
    }
    c.push(UpheavalEvent::Settled { t_myr: settle_t });

    // Close the books.
    let mut b = c.budget.clone();
    for w in &c.worlds {
        add(&mut b.solid_in_worlds, &w.solid);
        b.gas_in_worlds += w.gas;
    }
    for x in &c.belts {
        add(&mut b.solid_in_belts, &x.solid);
    }
    for x in &c.moons {
        add(&mut b.solid_in_moons, &x.solid);
    }
    for x in &c.reservoirs {
        add(&mut b.solid_in_reservoirs, &x.solid);
    }
    c.dy.energy_bound = c.worlds.iter().map(|w| w.energy()).sum();
    c.dy.angmom_bound = c.worlds.iter().map(|w| w.angmom()).sum();
    Upheaval {
        worlds: c.worlds,
        belts: c.belts,
        rogues: c.rogues,
        moons: c.moons,
        reservoirs: c.reservoirs,
        events: c.events,
        priming,
        budget: b,
        dynamics: c.dy,
        gas_end_myr: t0,
        break_myr: first_break,
        settled_myr: settle_t,
    }
}

/// The hand-over to stage 8 (F6) for every world that came through.
pub fn protoplanets(u: &Upheaval, system: &Address) -> Vec<crate::early::Protoplanet> {
    protoplanets_of(&u.worlds, system)
}

/// The same hand-over for a list of worlds (F5 passes its own, after moons are taken).
pub fn protoplanets_of(worlds: &[World], system: &Address) -> Vec<crate::early::Protoplanet> {
    let k = MASS_UNIT * MSUN_IN_EARTH;
    worlds
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let by = w.solid.map(|x| x as f64 * k);
            let mut grown: f64 = 0.0;
            for e in &w.ledger.entries {
                if let WorldEntry::Grown { ledger } = e {
                    grown = grown.max(ledger.last_claim());
                }
            }
            crate::early::Protoplanet {
                address: system.child("body", i as u64).expect("valid"),
                mass: by.iter().sum(),
                a_au: w.a_au,
                e: w.e,
                formed_myr: w.last_impact_myr.unwrap_or(grown).max(grown),
                by_group: by,
                gas: w.gas as f64 * k,
            }
        })
        .collect()
}
