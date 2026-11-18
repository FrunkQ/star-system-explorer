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
pub const SEED_ALGORITHM: &str = "system-lab-seed-v1";
pub const SOURCE: &str = "System Lab 0.1.0-beta.1 (C:/Development/system-lab, src/engine/accrete/kernel.ts and AccreteAdapter.ts), adapting FrunkQ/star-system-generator@61da68ed (src/lib/vendor/accrete-js), from tmanderson/Accrete.js by Mitch Anderson (ISC)";
pub const REF_DOLE: &str =
    "Dole, S. H. 1970, Icarus 13, 494: computer simulation of the formation of planetary systems";
pub const REF_FOGG: &str =
    "Fogg, M. J. 1985, JBIS 38, 501: extra-solar planetary systems, a microcomputer simulation";

/// Behaviours kept from the source so the outputs match; each is a candidate
/// for a separately versioned rule (the design's "Classic" block stays as is).
pub const KNOWN_DIFFERENCES: [&str; 3] = [
    "Partial sweeps are one-sided: a nucleus whose reach starts inside a band and ends beyond it takes nothing from that band (the source's first partial-overlap test referred to a missing property and was always false).",
    "A body's gravitational reach is 1 - (m/(1+m))^(1/4) and 1 + (m/(1+m))^(1/4) AU, not r (m/(1+m))^(1/4) as in Dole and Fogg, so every nucleus reaches about 1 AU either side.",
    "Coalescence conserves angular momentum with sqrt(sqrt(1 - e^2)) for the second body instead of sqrt(1 - e^2).",
];

const ALPHA: f64 = 5.0;
const N: f64 = 3.0;
const Q: f64 = 0.077;
pub const PROTOPLANET_MASS: f64 = 1e-15;
pub const SOLAR_MASS_IN_EARTH: f64 = 332_775.64;
pub const SOLAR_MASS_KG: f64 = 1.989e30;
pub const CLOUD_RADIUS_AU: f64 = 50.0;
pub const MAX_ITERATIONS: u32 = 100;

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
}

impl Controls {
    pub const DEFAULT: Controls = Controls {
        A: 0.0015,
        K: 50.0,
        W: 0.2,
        B: 1.2e-5,
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
    fn normalized_mass(&self) -> f64 {
        self.mass / (1.0 + self.mass)
    }
    fn critical_mass(&self, c: &Controls) -> f64 {
        c.B * m::pow(self.rp(), -3.0 / 4.0)
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
        let lower = p.rp() - p.xp();
        let upper = p.ra() + p.xa();
        self.bands
            .iter()
            .any(|b| b.dust && upper > b.upper_au - p.xa() && lower < b.lower_au + p.xp())
    }
    /// Returns (density, dust-only part of it).
    fn sweep(&mut self, c: &Controls, p: &Planet) -> (f64, f64) {
        let mut density = 0.0;
        let mut dust_only = 0.0;
        let include_gas = p.gas_giant;
        let lower = p.rp() - p.xp();
        let upper = p.ra() + p.xa();
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
    fn collect_dust(&mut self, mut p: Planet, iteration: u32) -> Planet {
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
        let (xp, xa) = (x.rp() - x.xp(), x.ra() + x.xa());
        let (yp, ya) = (y.rp() - y.xp(), y.ra() + y.xa());
        // The source tests both orders, which are the same overlap test.
        xp < ya && xa > yp
    }

    fn coalesce(&mut self, first: Planet, second: Planet, id: String, iteration: u32) -> Planet {
        let mass = first.mass + second.mass;
        let axis = mass / (first.mass / first.a + second.mass / second.a);
        let t1 = first.mass * m::sqrt(first.a) * m::sqrt(1.0 - first.e * first.e);
        let t2 = second.mass * m::sqrt(second.a) * m::sqrt(m::sqrt(1.0 - second.e * second.e));
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
    req.controls.check()?;
    if req.master_seed.is_empty() {
        return Err("master seed must not be empty".into());
    }
    if !(req.star_mass > 0.0 && req.star_luminosity > 0.0) {
        return Err("star mass and luminosity must be positive".into());
    }
    let mut sys = System {
        c: req.controls,
        cloud: Cloud {
            bands: vec![band(0.0, CLOUD_RADIUS_AU, true, true)],
        },
        planets: Vec::new(),
        collisions: 0,
    };
    let duration = duration_years(req.star_mass);
    let step = duration / MAX_ITERATIONS as f64;
    let mut events = Vec::new();
    let mut iterations = 0;
    while sys.cloud.has_dust() && iterations < MAX_ITERATIONS {
        let it = iterations + 1;
        let id = format!("nucleus-{it:03}");
        let a = legacy_rand(
            &mut stream(&req.master_seed, &["formation", "accrete", &id, "axis"]),
            0.3,
            50.0,
        );
        let er = legacy_rand(
            &mut stream(
                &req.master_seed,
                &["formation", "accrete", &id, "eccentricity"],
            ),
            0.0,
            1.0,
        );
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
        rule_version: RULE_VERSION.into(),
        request: req.clone(),
        iterations,
        exhausted_dust: !sys.cloud.has_dust(),
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
    let step_myr = run.duration_years / MAX_ITERATIONS as f64 / 1e6;
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
