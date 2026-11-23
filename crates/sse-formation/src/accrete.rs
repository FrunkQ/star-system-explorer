//! Stage 4 to 5, Classic Accrete block (F2): Dole's 1970 accretion model as
//! Alex's System Lab runs it (`accrete-frunkq-v1`), ported from System Lab's
//! TypeScript `KernelStarSystem` and `AccreteAdapter` (0.1.0-beta.1), which
//! adapt Mitch Anderson's Accrete.js (ISC).
//!
//! The rules are kept exactly, including the three behaviours System Lab
//! records as inherited (see `KNOWN_DIFFERENCES`), so that the same request
//! gives the same planets here and there. What SSE4 adds is the provenance
//! ledger: every gram of every body is traced to the seed nucleus, the sweeps
//! (where, how much dust and how much gas) and the bodies it merged from.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

pub const RULE_VERSION: &str = "accrete-frunkq-v1";
/// The corrected rule: Dole 1970 and Fogg 1985 as published (the three
/// `KNOWN_DIFFERENCES` undone), with the star and the inheritance setting the cloud.
pub const RULE_DOLE: &str = "accrete-dole1970-v1";

/// Which Accrete rules a run follows.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rule {
    /// System Lab's, exactly (`accrete-frunkq-v1`).
    #[default]
    SystemLab,
    /// `accrete-dole1970-v1`.
    Dole1970,
}

/// Dole 1970's dust density at the centre for the Sun, and Fogg 1985's scaling with the star.
pub const DOLE_A: f64 = 0.0015;
pub const DOLE_K: f64 = 50.0;
pub const SEED_ALGORITHM: &str = "system-lab-seed-v1";
pub const SOURCE: &str = "System Lab 0.1.0-beta.1 (C:/Development/system-lab, src/engine/accrete/kernel.ts and AccreteAdapter.ts), adapting FrunkQ/star-system-generator@61da68ed (src/lib/vendor/accrete-js), from tmanderson/Accrete.js by Mitch Anderson (ISC)";
pub const REF_DOLE: &str =
    "Dole, S. H. 1970, Icarus 13, 494: computer simulation of the formation of planetary systems";
pub const REF_FOGG: &str =
    "Fogg, M. J. 1985, JBIS 38, 501: extra-solar planetary systems, a microcomputer simulation";

/// Behaviours kept from the source so the outputs match; each is a candidate
/// for a separately versioned rule (the design's "Classic" block stays as is).
pub const KNOWN_DIFFERENCES: [&str; 4] = [
    "Partial sweeps are one-sided: a nucleus whose reach starts inside a band and ends beyond it takes nothing from that band (the source's first partial-overlap test referred to a missing property and was always false).",
    "A body's gravitational reach is 1 - (m/(1+m))^(1/4) and 1 + (m/(1+m))^(1/4) AU, not r (m/(1+m))^(1/4) as in Dole and Fogg, so every nucleus reaches about 1 AU either side.",
    "Coalescence conserves angular momentum with sqrt(sqrt(1 - e^2)) for the second body instead of sqrt(1 - e^2).",
    "Nuclei are placed by the mean of two draws times the outer bound (a triangle peaking at 25 AU, clamped at 0.3 AU), not uniformly between the bounds, so the inner system is rarely seeded.",
];

const ALPHA: f64 = 5.0;
const N: f64 = 3.0;
const Q: f64 = 0.077;
pub const PROTOPLANET_MASS: f64 = 1e-15;
pub const SOLAR_MASS_IN_EARTH: f64 = sse_core::sheet::GM_SUN / sse_core::sheet::GM_EARTH;
pub const SOLAR_MASS_KG: f64 = sse_core::sheet::M_SUN;
pub const CLOUD_RADIUS_AU: f64 = 50.0;
pub const MAX_ITERATIONS: u32 = 100;
/// The corrected rule runs until the planet zone's dust is gone (Dole and Fogg
/// have no cap); this bound only stops a pathological request.
pub const DOLE_MAX_ITERATIONS: u32 = 5000;

/// Dole's controls: A dust density at the centre, K gas to dust ratio, W the
/// relative velocity spread, B the critical-mass coefficient (System Lab's
/// slider ranges are enforced).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Controls {
    pub A: f64,
    pub K: f64,
    pub W: f64,
    pub B: f64,
    /// Set by `run` from the request's rule; not part of the controls' data.
    #[serde(skip)]
    pub dole: bool,
    /// sqrt(L) of the star, for the corrected rule's critical mass (Fogg 1985); set by `run`.
    #[serde(skip)]
    pub sqrt_l: f64,
}

impl Controls {
    pub const DEFAULT: Controls = Controls {
        A: 0.0015,
        K: 50.0,
        W: 0.2,
        B: 1.2e-5,
        dole: false,
        sqrt_l: 1.0,
    };
    pub fn check(&self) -> Result<(), String> {
        let ok = (0.0005..=0.005).contains(&self.A)
            && (10.0..=200.0).contains(&self.K)
            && (0.05..=0.8).contains(&self.W)
            && (0.000005..=0.00005).contains(&self.B);
        if ok {
            Ok(())
        } else {
            Err(format!(
                "controls {self:?} are outside System Lab's ranges (A 0.0005 to 0.005, K 10 to 200, W 0.05 to 0.8, B 5e-6 to 5e-5)"
            ))
        }
    }
}

/// System Lab's named random streams: FNV-1a 32 of a length-framed path seeds
/// a mulberry32 generator.
pub fn fnv1a32(text: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for u in text.encode_utf16() {
        h ^= u as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

pub struct Mulberry32(u32);

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Mulberry32(seed)
    }
    pub fn next_float(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut v = self.0;
        v = (v ^ (v >> 15)).wrapping_mul(v | 1);
        v ^= v.wrapping_add((v ^ (v >> 7)).wrapping_mul(v | 61));
        (v ^ (v >> 14)) as f64 / 4_294_967_296.0
    }
}

pub fn stream(master_seed: &str, path: &[&str]) -> Mulberry32 {
    let mut parts = vec![SEED_ALGORITHM, master_seed];
    parts.extend_from_slice(path);
    let framed: Vec<String> = parts
        .iter()
        .map(|p| format!("{}:{p}", p.encode_utf16().count()))
        .collect();
    Mulberry32::new(fnv1a32(&framed.join("|")))
}

/// Accrete's draw: the mean of two 16-bit draws, scaled to `max`, clamped.
pub fn legacy_rand(s: &mut Mulberry32, min: f64, max: f64) -> f64 {
    let first = m::floor(s.next_float() * 65_536.0) / 65_536.0;
    let second = m::floor(s.next_float() * 65_536.0) / 65_536.0;
    let v = (max * (first + second)) / 2.0;
    v.min(max).max(min)
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct DustBand {
    pub lower_au: f64,
    pub upper_au: f64,
    pub dust: bool,
    pub gas: bool,
}

fn band(lower: f64, upper: f64, dust: bool, gas: bool) -> DustBand {
    DustBand {
        lower_au: lower,
        upper_au: upper,
        dust,
        gas,
    }
}

/// One line of a body's provenance ledger. Summed in order, the masses give
/// the body's mass exactly (the same additions in the same order).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Entry {
    /// The starting nucleus.
    Seed { mass: f64 },
    /// One sweep: mass taken while centred on `a_au`; `gas` of it was gas.
    Sweep {
        iteration: u32,
        a_au: f64,
        mass: f64,
        gas: f64,
    },
    /// A whole body merged in, with its own ledger.
    Merged { mass: f64, body: Box<Ledger> },
}

impl Entry {
    pub fn mass(&self) -> f64 {
        match self {
            Entry::Seed { mass } | Entry::Sweep { mass, .. } | Entry::Merged { mass, .. } => *mass,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub id: String,
    pub entries: Vec<Entry>,
}

impl Ledger {
    /// Sum in order: equals the body's mass bit for bit.
    pub fn total(&self) -> f64 {
        let mut t = 0.0;
        for e in &self.entries {
            t += e.mass();
        }
        t
    }
    /// Gas over the whole tree (solar masses).
    pub fn gas(&self) -> f64 {
        let mut g = 0.0;
        for e in &self.entries {
            match e {
                Entry::Sweep { gas, .. } => g += gas,
                Entry::Merged { body, .. } => g += body.gas(),
                Entry::Seed { .. } => {}
            }
        }
        g
    }
    /// Solid mass by sweep radius, flattened over the tree: (a_au, solid mass).
    pub fn solids(&self, out: &mut Vec<(f64, f64)>) {
        for e in &self.entries {
            match e {
                Entry::Seed { mass } => out.push((f64::NAN, *mass)),
                Entry::Sweep {
                    a_au, mass, gas, ..
                } => out.push((*a_au, mass - gas)),
                Entry::Merged { body, .. } => body.solids(out),
            }
        }
    }
    /// Last iteration that added mass anywhere in the tree.
    pub fn last_iteration(&self) -> u32 {
        let mut it = 0;
        for e in &self.entries {
            match e {
                Entry::Sweep { iteration, .. } => it = it.max(*iteration),
                Entry::Merged { body, .. } => it = it.max(body.last_iteration()),
                Entry::Seed { .. } => {}
            }
        }
        it
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Planet {
    id: String,
    formed_from: Vec<String>,
    a: f64,
    e: f64,
    mass: f64,
    quad_mass: f64,
    gas_giant: bool,
    delta_mass: f64,
    ledger: Ledger,
    dole: bool,
    w: f64,
}

impl Planet {
    fn new(
        c: &Controls,
        id: String,
        formed_from: Vec<String>,
        a: f64,
        e: f64,
        mass: f64,
        ledger: Ledger,
    ) -> Self {
        let mut p = Planet {
            id,
            formed_from,
            a,
            e,
            mass,
            quad_mass: 0.0,
            gas_giant: false,
            delta_mass: 1.0,
            ledger,
            dole: c.dole,
            w: c.W,
        };
        p.quad_mass = m::pow(p.normalized_mass(), 1.0 / 4.0);
        p.gas_giant = mass >= p.critical_mass(c);
        p
    }
    fn rp(&self) -> f64 {
        self.a * (1.0 - self.e)
    }
    fn ra(&self) -> f64 {
        self.a * (1.0 + self.e)
    }
    fn xp(&self) -> f64 {
        1.0 - self.quad_mass
    }
    fn xa(&self) -> f64 {
        1.0 + self.quad_mass
    }
    /// Inner and outer edge of the band the body sweeps. System Lab: rp - (1 - q)
    /// and ra + (1 + q) AU. Dole 1970 and Fogg 1985: rp (1 - q) / (1 + W) and
    /// ra (1 + q) / (1 - W), the reach scaling with distance and the dust's own eccentricity.
    fn lower(&self) -> f64 {
        if self.dole {
            self.rp() * (1.0 - self.quad_mass) / (1.0 + self.w)
        } else {
            self.rp() - self.xp()
        }
    }
    fn upper(&self) -> f64 {
        if self.dole {
            self.ra() * (1.0 + self.quad_mass) / (1.0 - self.w)
        } else {
            self.ra() + self.xa()
        }
    }
    fn normalized_mass(&self) -> f64 {
        self.mass / (1.0 + self.mass)
    }
    fn critical_mass(&self, c: &Controls) -> f64 {
        if c.dole {
            c.B * m::pow(self.rp() * c.sqrt_l, -3.0 / 4.0)
        } else {
            c.B * m::pow(self.rp(), -3.0 / 4.0)
        }
    }
    fn mass_density(&self, c: &Controls, density: f64) -> f64 {
        let w = c.W;
        let first = (8.0 * m::PI * m::pow(self.a, N) * density * self.quad_mass) / (1.0 - w * w);
        let second = self.e + self.quad_mass + w + w * self.e * self.quad_mass;
        first * second
    }
    fn add_mass(&mut self, c: &Controls, mass: f64) {
        self.delta_mass = mass;
        self.mass += mass;
        self.quad_mass = m::pow(self.normalized_mass(), 1.0 / 4.0);
        self.gas_giant = self.mass >= self.critical_mass(c);
    }
}

fn dust_density(c: &Controls, r: f64, mass: f64, critical: f64, include_gas: bool) -> f64 {
    let coefficient = if include_gas { c.K } else { 1.0 };
    let denominator = if include_gas {
        1.0 + m::sqrt(critical / mass) * (c.K - 1.0)
    } else {
        1.0
    };
    (coefficient * (c.A * m::exp(-ALPHA * m::pow(r, 1.0 / N)))) / denominator
}

struct Cloud {
    bands: Vec<DustBand>,
}

impl Cloud {
    fn has_dust(&self) -> bool {
        self.bands.iter().any(|b| b.dust && b.gas)
    }
    fn contains_dust(&self, p: &Planet) -> bool {
        let (lower, upper) = (p.lower(), p.upper());
        if p.dole {
            return self
                .bands
                .iter()
                .any(|b| b.dust && upper > b.lower_au && lower < b.upper_au);
        }
        self.bands
            .iter()
            .any(|b| b.dust && upper > b.upper_au - p.xa() && lower < b.lower_au + p.xp())
    }
    /// Returns (density, dust-only part of it).
    fn sweep(&mut self, c: &Controls, p: &Planet) -> (f64, f64) {
        let mut density = 0.0;
        let mut dust_only = 0.0;
        let include_gas = p.gas_giant;
        let (lower, upper) = (p.lower(), p.upper());
        let mut take = |gas: bool| {
            let g = gas && include_gas;
            density += dust_density(c, p.a, p.mass, p.critical_mass(c), g);
            dust_only += dust_density(c, p.a, p.mass, p.critical_mass(c), false);
            g
        };
        let mut out: Vec<DustBand> = Vec::new();
        for b in &self.bands {
            if b.upper_au < lower || b.lower_au > upper || !b.dust {
                out.push(*b);
            } else if upper < b.upper_au && upper > b.lower_au && lower < b.lower_au {
                let g = take(b.gas);
                out.push(band(b.lower_au, upper, false, !g));
                out.push(band(upper, b.upper_au, b.dust, b.gas));
            } else if b.lower_au < lower && b.upper_au > upper {
                let g = take(b.gas);
                out.push(band(b.lower_au, lower, b.dust, b.gas));
                out.push(band(lower, upper, false, !g));
                out.push(band(upper, b.upper_au, b.dust, b.gas));
            } else if p.dole && lower > b.lower_au && lower < b.upper_au && upper >= b.upper_au {
                // The partial sweep System Lab never took: reach starts inside the band, ends beyond it.
                let g = take(b.gas);
                out.push(band(b.lower_au, lower, b.dust, b.gas));
                out.push(band(lower, b.upper_au, false, !g));
            } else if lower < b.lower_au && upper > b.upper_au {
                let g = take(b.gas);
                out.push(band(b.lower_au, b.upper_au, false, !g));
            } else {
                out.push(*b);
            }
        }
        // Stable sort by inner edge (as JavaScript's), then merge alike neighbours.
        out.sort_by(|x, y| x.lower_au.partial_cmp(&y.lower_au).expect("finite"));
        let mut merged: Vec<DustBand> = Vec::new();
        for b in out {
            match merged.last_mut() {
                Some(prev) if prev.dust == b.dust && prev.gas == b.gas => {
                    *prev = band(prev.lower_au, b.upper_au, b.dust, b.gas)
                }
                _ => merged.push(b),
            }
        }
        self.bands = merged;
        (density, dust_only)
    }
}

/// A finished body, with its whole provenance.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Body {
    pub id: String,
    pub formed_from: Vec<String>,
    pub a_au: f64,
    pub e: f64,
    /// Solar masses.
    pub mass: f64,
    pub gas_giant: bool,
    pub ledger: Ledger,
}

impl Body {
    pub fn earth_mass(&self) -> f64 {
        self.mass * SOLAR_MASS_IN_EARTH
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Event {
    NucleusInjected {
        iteration: u32,
        time_years: f64,
        id: String,
        a_au: f64,
        e: f64,
    },
    BodyAccreted {
        iteration: u32,
        time_years: f64,
        id: String,
        mass: f64,
        gas_giant: bool,
    },
    BodiesCoalesced {
        iteration: u32,
        time_years: f64,
        from: [String; 2],
        into: String,
    },
    FormationComplete {
        iteration: u32,
        time_years: f64,
        bodies: Vec<String>,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Request {
    pub master_seed: String,
    /// Host mass in solar masses (sets the run's duration only, as in System Lab).
    pub star_mass: f64,
    pub star_luminosity: f64,
    pub controls: Controls,
    #[serde(default, skip_serializing_if = "is_system_lab")]
    pub rule: Rule,
}

fn is_system_lab(r: &Rule) -> bool {
    *r == Rule::SystemLab
}

impl Request {
    /// The corrected rule for a formed system: the dust follows the metals the
    /// system inherited from its patch (Dole's A times 10^[Fe/H], the gas-to-dust
    /// ratio K divided by the same, so the gas is unchanged) and Fogg 1985's
    /// scaling with the star (A times sqrt(M)).
    pub fn from_history(h: &crate::FormationHistory, master_seed: &str) -> Request {
        let star = &h.stars[0];
        let metals = m::pow(10.0, h.inheritance.feh);
        Request {
            master_seed: master_seed.into(),
            star_mass: star.mass,
            star_luminosity: star.life.l_zams.max(1e-4),
            controls: Controls {
                A: DOLE_A * m::sqrt(star.mass) * metals,
                K: DOLE_K / metals,
                ..Controls::DEFAULT
            },
            rule: Rule::Dole1970,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Run {
    pub rule_version: String,
    pub request: Request,
    pub iterations: u32,
    pub exhausted_dust: bool,
    pub duration_years: f64,
    /// Sorted by semi-major axis.
    pub bodies: Vec<Body>,
    pub bands: Vec<DustBand>,
    pub events: Vec<Event>,
}

/// System Lab's run length: 90 percent of a 10 Gyr M^-2.5 lifetime, at most 100 Myr.
pub fn duration_years(star_mass: f64) -> f64 {
    let life = (10.0 / m::pow(star_mass, 2.5)) * 1e9;
    (life * 0.9).min(100_000_000.0)
}

struct System {
    c: Controls,
    cloud: Cloud,
    planets: Vec<Planet>,
    collisions: u32,
}

impl System {
    /// Dole 1970 as Fogg 1985 and Burrows's accrete code run it: the mass within
    /// the body's reach (volume 4 pi a^2 q (1 - e (t1 - t2)/w) times the overlap
    /// width, times the dust or dust-and-gas density) is found by iterating to a
    /// fixed point, then the lanes it reached are cleared once.
    fn collect_dust_dole(&mut self, mut p: Planet, iteration: u32) -> Planet {
        let seed = p.mass;
        let crit = p.critical_mass(&self.c);
        let at = |c: &Controls, cloud: &Cloud, mass: f64| -> (f64, f64, f64, f64) {
            let q = m::pow(mass / (1.0 + mass), 0.25);
            let inner = (p.rp() * (1.0 - q) / (1.0 + c.W)).max(0.0);
            let outer = p.ra() * (1.0 + q) / (1.0 - c.W);
            let width_all = outer - inner;
            let (mut total, mut dust) = (0.0, 0.0);
            for b in &cloud.bands {
                if b.upper_au <= inner || b.lower_au >= outer || !b.dust {
                    continue;
                }
                let d = c.A * m::exp(-ALPHA * m::pow(p.a, 1.0 / N));
                let rho = if mass < crit || !b.gas {
                    d
                } else {
                    c.K * d / (1.0 + m::sqrt(crit / mass) * (c.K - 1.0))
                };
                let t1 = (outer - b.upper_au).max(0.0);
                let t2 = (b.lower_au - inner).max(0.0);
                let width = width_all - t1 - t2;
                let vol = 4.0 * m::PI * p.a * p.a * q * (1.0 - p.e * (t1 - t2) / width_all) * width;
                total += vol * rho;
                dust += vol * d;
            }
            (total, dust, inner, outer)
        };
        let mut swept = 0.0;
        let mut dust = 0.0;
        let (mut inner, mut outer) = (0.0, 0.0);
        for _ in 0..1000 {
            let (t, d, i, o) = at(&self.c, &self.cloud, seed + swept);
            let grew = t - swept;
            swept = t;
            dust = d;
            inner = i;
            outer = o;
            if grew < 1e-4 * (seed + swept).max(f64::MIN_POSITIVE) {
                break;
            }
        }
        if swept > 0.0 {
            p.add_mass(&self.c, swept);
            p.ledger.entries.push(Entry::Sweep {
                iteration,
                a_au: p.a,
                mass: swept,
                gas: (swept - dust).max(0.0),
            });
            // Clear the lanes: dust always, gas too once the body is a giant.
            let clear_gas = p.gas_giant;
            let mut out: Vec<DustBand> = Vec::new();
            for b in &self.cloud.bands {
                if b.upper_au <= inner || b.lower_au >= outer {
                    out.push(*b);
                    continue;
                }
                if b.lower_au < inner {
                    out.push(band(b.lower_au, inner, b.dust, b.gas));
                }
                out.push(band(
                    b.lower_au.max(inner),
                    b.upper_au.min(outer),
                    false,
                    b.gas && !clear_gas,
                ));
                if b.upper_au > outer {
                    out.push(band(outer, b.upper_au, b.dust, b.gas));
                }
            }
            let mut merged: Vec<DustBand> = Vec::new();
            for b in out {
                match merged.last_mut() {
                    Some(prev) if prev.dust == b.dust && prev.gas == b.gas => {
                        *prev = band(prev.lower_au, b.upper_au, b.dust, b.gas)
                    }
                    _ => merged.push(b),
                }
            }
            self.cloud.bands = merged;
        }
        p
    }

    fn collect_dust(&mut self, mut p: Planet, iteration: u32) -> Planet {
        if self.c.dole {
            return self.collect_dust_dole(p, iteration);
        }
        let (mut density, mut dust) = self.cloud.sweep(&self.c, &p);
        while self.cloud.contains_dust(&p) && density > 0.0 {
            let new_mass = p.mass_density(&self.c, density);
            let delta = new_mass - p.mass;
            p.add_mass(&self.c, delta);
            let gas = delta * (1.0 - dust / density);
            p.ledger.entries.push(Entry::Sweep {
                iteration,
                a_au: p.a,
                mass: delta,
                gas,
            });
            if p.delta_mass < p.mass * 1e-4 {
                break;
            }
            (density, dust) = self.cloud.sweep(&self.c, &p);
        }
        p
    }

    fn collides(x: &Planet, y: &Planet) -> bool {
        let (xp, xa) = (x.lower(), x.upper());
        let (yp, ya) = (y.lower(), y.upper());
        // The source tests both orders, which are the same overlap test.
        xp < ya && xa > yp
    }

    fn coalesce(&mut self, first: Planet, second: Planet, id: String, iteration: u32) -> Planet {
        let mass = first.mass + second.mass;
        let axis = mass / (first.mass / first.a + second.mass / second.a);
        let t1 = first.mass * m::sqrt(first.a) * m::sqrt(1.0 - first.e * first.e);
        let t2 = if self.c.dole {
            second.mass * m::sqrt(second.a) * m::sqrt(1.0 - second.e * second.e)
        } else {
            second.mass * m::sqrt(second.a) * m::sqrt(m::sqrt(1.0 - second.e * second.e))
        };
        let term = (t1 + t2) / (mass * m::sqrt(axis));
        let e = m::sqrt((1.0 - term * term).abs());
        let mut from: Vec<String> = first
            .formed_from
            .iter()
            .chain(&second.formed_from)
            .cloned()
            .collect();
        from.sort();
        from.dedup();
        let ledger = Ledger {
            id: id.clone(),
            entries: vec![
                Entry::Merged {
                    mass: first.mass,
                    body: Box::new(first.ledger),
                },
                Entry::Merged {
                    mass: second.mass,
                    body: Box::new(second.ledger),
                },
            ],
        };
        let p = Planet::new(&self.c, id, from, axis, e, mass, ledger);
        self.collect_dust(p, iteration)
    }

    fn check_collisions(&mut self, iteration: u32) -> Vec<([String; 2], String)> {
        let mut seen = Vec::new();
        let mut out: Vec<Planet> = Vec::new();
        for p in std::mem::take(&mut self.planets) {
            match out.iter().position(|o| Self::collides(&p, o)) {
                None => out.push(p),
                Some(i) => {
                    self.collisions += 1;
                    let id = format!("collision-{:03}", self.collisions);
                    let other = out.remove(i);
                    let names = [p.id.clone(), other.id.clone()];
                    let merged = self.coalesce(p, other, id.clone(), iteration);
                    seen.push((names, id));
                    out.insert(i, merged);
                }
            }
        }
        self.planets = out;
        seen
    }
}

/// Run the Classic Accrete block.
pub fn run(req: &Request) -> Result<Run, String> {
    let dole = req.rule == Rule::Dole1970;
    if dole {
        let c = &req.controls;
        let ok = [c.A, c.K, c.W, c.B]
            .iter()
            .all(|x| x.is_finite() && *x > 0.0)
            && c.W < 1.0
            && c.K >= 1.0;
        if !ok {
            return Err(format!(
                "controls {c:?} are not usable (all positive, W below 1, K at least 1)"
            ));
        }
    } else {
        req.controls.check()?;
    }
    if req.master_seed.is_empty() {
        return Err("master seed must not be empty".into());
    }
    if !(req.star_mass > 0.0 && req.star_luminosity > 0.0) {
        return Err("star mass and luminosity must be positive".into());
    }
    // Fogg 1985: the cloud and the nuclei's range scale as M^(1/3); the cloud reaches 200 M^(1/3) AU.
    let scale = if dole { m::cbrt(req.star_mass) } else { 1.0 };
    let (inner_nucleus, outer_nucleus, cloud_au) = if dole {
        (0.3 * scale, 50.0 * scale, 200.0 * scale)
    } else {
        (0.3, 50.0, CLOUD_RADIUS_AU)
    };
    let mut sys = System {
        c: Controls {
            dole,
            sqrt_l: m::sqrt(req.star_luminosity),
            ..req.controls
        },
        cloud: Cloud {
            bands: vec![band(0.0, cloud_au, true, true)],
        },
        planets: Vec::new(),
        collisions: 0,
    };
    let duration = duration_years(req.star_mass);
    let max_it = if dole {
        DOLE_MAX_ITERATIONS
    } else {
        MAX_ITERATIONS
    };
    let step = duration / max_it as f64;
    let mut events = Vec::new();
    let mut iterations = 0;
    // The corrected rule stops when the planet zone (the nuclei's range) has no dust left.
    let zone_has_dust = |cloud: &Cloud| {
        cloud
            .bands
            .iter()
            .any(|b| b.dust && b.upper_au > inner_nucleus && b.lower_au < outer_nucleus)
    };
    while (if dole {
        zone_has_dust(&sys.cloud)
    } else {
        sys.cloud.has_dust()
    }) && iterations < max_it
    {
        let it = iterations + 1;
        let id = format!("nucleus-{it:03}");
        let mut sa = stream(&req.master_seed, &["formation", "accrete", &id, "axis"]);
        let mut se = stream(
            &req.master_seed,
            &["formation", "accrete", &id, "eccentricity"],
        );
        let (a, er) = if dole {
            // Uniform between the bounds, as Dole and Fogg.
            (
                inner_nucleus + (outer_nucleus - inner_nucleus) * sa.next_float(),
                se.next_float(),
            )
        } else {
            (
                legacy_rand(&mut sa, inner_nucleus, outer_nucleus),
                legacy_rand(&mut se, 0.0, 1.0),
            )
        };
        let t = iterations as f64 * step;
        let e = 1.0 - m::pow(er, Q);
        let nucleus = Planet::new(
            &sys.c,
            id.clone(),
            vec![id.clone()],
            a,
            e,
            PROTOPLANET_MASS,
            Ledger {
                id: id.clone(),
                entries: vec![Entry::Seed {
                    mass: PROTOPLANET_MASS,
                }],
            },
        );
        events.push(Event::NucleusInjected {
            iteration: it,
            time_years: t,
            id: id.clone(),
            a_au: a,
            e,
        });
        let p = sys.collect_dust(nucleus, it);
        if p.mass > PROTOPLANET_MASS {
            events.push(Event::BodyAccreted {
                iteration: it,
                time_years: t,
                id: id.clone(),
                mass: p.mass,
                gas_giant: p.gas_giant,
            });
            sys.planets.push(p);
        }
        for (from, into) in sys.check_collisions(it) {
            events.push(Event::BodiesCoalesced {
                iteration: it,
                time_years: t,
                from,
                into,
            });
        }
        iterations = it;
    }
    let mut bodies: Vec<Body> = sys
        .planets
        .into_iter()
        .map(|p| Body {
            id: p.id,
            formed_from: p.formed_from,
            a_au: p.a,
            e: p.e,
            mass: p.mass,
            gas_giant: p.gas_giant,
            ledger: p.ledger,
        })
        .collect();
    bodies.sort_by(|x, y| x.a_au.partial_cmp(&y.a_au).expect("finite"));
    events.push(Event::FormationComplete {
        iteration: iterations,
        time_years: duration,
        bodies: bodies.iter().map(|b| b.id.clone()).collect(),
    });
    Ok(Run {
        rule_version: if dole { RULE_DOLE } else { RULE_VERSION }.into(),
        request: req.clone(),
        iterations,
        exhausted_dust: if dole {
            !zone_has_dust(&sys.cloud)
        } else {
            !sys.cloud.has_dust()
        },
        duration_years: duration,
        bodies,
        bands: sys.cloud.bands,
        events,
    })
}

/// The hand-over to stage 8 (F6): each body as a `Protoplanet`, its solids
/// split by element group from the SSE4 disc band at each sweep's radius (or
/// the rock band's mix inside the disc's water line when no disc is given).
pub fn protoplanets(
    run: &Run,
    system: &sse_core::Address,
    disc: Option<&crate::disc::Disc>,
) -> Vec<crate::early::Protoplanet> {
    let mix_at = |a: f64| -> [f64; 6] {
        let fallback = crate::disc::mix(0.0, 0.0, 0.55);
        let x = match disc {
            Some(d) => {
                let b = d
                    .bands
                    .iter()
                    .find(|b| a.is_nan() || (a >= b.inner_au && a < b.outer_au))
                    .or(d.bands.last());
                match b {
                    Some(b) if b.solid.iter().any(|v| *v > 0) => b.solid.map(|v| v as f64),
                    _ => fallback,
                }
            }
            None => [fallback[0], fallback[1], fallback[2] * 0.1, 0.0, 0.0, 0.0],
        };
        let s: f64 = x.iter().sum();
        x.map(|v| v / s)
    };
    let max_it = if run.rule_version == RULE_DOLE {
        DOLE_MAX_ITERATIONS
    } else {
        MAX_ITERATIONS
    };
    let step_myr = run.duration_years / max_it as f64 / 1e6;
    run.bodies
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let mut solids = Vec::new();
            b.ledger.solids(&mut solids);
            let mut by = [0.0; 6];
            for (a, s) in solids {
                let x = mix_at(a);
                for g in 0..6 {
                    by[g] += s * x[g] * SOLAR_MASS_IN_EARTH;
                }
            }
            let gas = b.ledger.gas() * SOLAR_MASS_IN_EARTH;
            crate::early::Protoplanet {
                address: system.child("body", i as u64).expect("valid"),
                mass: by.iter().sum(),
                a_au: b.a_au,
                e: b.e,
                formed_myr: (b.ledger.last_iteration().max(1) - 1) as f64 * step_myr,
                by_group: by,
                gas,
            }
        })
        .collect()
}
