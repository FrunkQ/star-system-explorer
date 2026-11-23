//! Stage 7, the leftovers (F5): belts, the outer belt, the scattered disc, the
//! Oort cloud, Trojans, irregular and regular moons, and rings.
//!
//! Small-body populations stay **clouds**: an envelope (orbits, mass by element
//! group, water by source band, a size law) with an address. Looking at one
//! resolves its k-th largest object from the seed and the object's address
//! alone, so the same object appears on every device, in any order of asking.
//! A cloud's mass splits exactly (integer budget units) into its resolved
//! objects and an unresolved remainder.
//!
//! The few large moons are resolved at once: regular satellites from each
//! giant's circumplanetary disc (Canup and Ward 2006), and the large moons
//! F4's oblique impacts made.

use crate::disc::{Disc, MASS_UNIT};
use crate::growth::MSUN_IN_EARTH;
use crate::violent::{
    add, earth, rayleigh, sum6, take_prop, take_water, world_water, Upheaval, WaterFrom, World,
    WorldEntry,
};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

pub const REF_DOHNANYI69: &str = "Dohnanyi, J. S. 1969, JGR 74, 2531: a collisional cascade settles to N(>D) proportional to D^-2.5";
pub const REF_CW06: &str = "Canup, R. M. and Ward, W. R. 2006, Nature 441, 834: regular satellite systems hold about 1e-4 of their planet's mass";
pub const REF_CANUP10: &str = "Canup, R. M. 2010, Nature 468, 943: massive rings from a large moon's mantle stripped as it spirals in";
pub const REF_OORT: &str = "Duncan, M., Quinn, T. and Tremaine, S. 1987, AJ 94, 1330: the Oort cloud spans about 3,000 to 100,000 AU, isotropic, thermal eccentricities";
pub const REF_TROJAN_RATIO: &str =
    "Grav, T. et al. 2011, ApJ 742, 40: Jupiter's L4 swarm outnumbers L5 by about 1.4 to 1.6";
pub const REF_JEWITT07: &str = "Jewitt, D. and Haghighipour, N. 2007, ARA&A 45, 261: irregular moons orbit at 0.1 to 0.5 Hill radii, eccentric, many retrograde";
pub const REF_BELTS: &str = "Asteroid belt (Bottke et al. 2002): mean e about 0.15, i about 10 degrees; cold Kuiper belt e about 0.05 (Petit et al. 2011)";

/// Size law: N(>D) = (D/D1)^-SLOPE, so the k-th largest has D1 k^(-1/SLOPE)
/// and mass m1 k^(-3/SLOPE); the masses sum to m1 zeta(3/SLOPE).
pub const SLOPE: f64 = 2.5;
/// zeta(1.2).
pub const ZETA_1_2: f64 = 5.591_582_441_177_751;
/// Regular satellites' share of a giant (Canup and Ward 2006).
pub const SATELLITE_SHARE: f64 = 1.0e-4;
/// Chance a giant keeps a massive ring, and its share of the satellite disc
/// (Saturn's rings are about 1e-4 of Titan).
pub const RING_CHANCE: f64 = 0.25;
pub const RING_SHARE: f64 = 1.0e-4;
/// Share of Trojans in the leading (L4) swarm.
pub const L4_SHARE: f64 = 0.6;
/// Largest small body, km: Pluto and Eris class (the primordial outer disc held
/// 1,000 to 4,000 of them, Nesvorny and Vokrouhlicky 2016, ApJ 825, 94). Ring
/// particles stop at metres.
pub const SIZE_CAP_KM: f64 = 2_400.0;
pub const RING_CAP_KM: f64 = 0.01;

const R_EARTH_KM: f64 = sse_core::sheet::R_EARTH_MEAN / 1e3;
const AU_KM: f64 = sse_core::sheet::AU / 1e3;
const MSUN_KG: f64 = sse_core::sheet::M_SUN;
/// Densities by element group, kg/m^3 (metal, rock, carbon, water, CO2, CO).
const DENSITY: [f64; 6] = [7_800.0, 3_000.0, 2_000.0, 1_000.0, 1_500.0, 900.0];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "law", rename_all = "kebab-case")]
pub enum Spread {
    Rayleigh {
        sigma: f64,
    },
    Uniform {
        lo: f64,
        hi: f64,
    },
    /// f(e) = 2e.
    Thermal,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "law", rename_all = "kebab-case")]
pub enum Tilt {
    Rayleigh { sigma_deg: f64 },
    Isotropic,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cloud {
    pub address: Address,
    /// "asteroid-belt", "belt", "outer-belt", "debris-belt", "scattered-disc",
    /// "oort", "trojans", "irregular-moons" or "ring".
    pub kind: String,
    /// World id it orbits, for Trojans (shares the orbit), irregular moons and rings.
    pub around: Option<String>,
    /// The F4 belt or reservoir it came from.
    pub source: String,
    /// Semi-major axes: AU around the star, or AU around `around` for moons and rings.
    pub a_min_au: f64,
    pub a_max_au: f64,
    /// Semi-major axis drawn uniform in log (true) or linear (false).
    pub a_log: bool,
    pub e: Spread,
    pub i: Tilt,
    pub solid: [u64; 6],
    pub water_from: WaterFrom,
    /// Largest object, km, and how many are larger than 1 km.
    pub largest_km: f64,
    pub over_1km: f64,
    /// How many objects share the largest size (1 unless the cloud is massive
    /// enough to hold many bodies at the size cap), and the sum of the weights.
    pub n_top: f64,
    pub norm: f64,
}

/// One object of a cloud, resolved from the seed and its address.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SmallBody {
    pub address: Address,
    pub kind: String,
    pub rank: u64,
    pub diameter_km: f64,
    pub mass_kg: f64,
    pub a_au: f64,
    pub e: f64,
    pub i_deg: f64,
    pub node_deg: f64,
    pub peri_deg: f64,
    pub mean_anomaly_deg: f64,
    /// Trojans: "L4" or "L5".
    pub swarm: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Moon {
    pub address: Address,
    pub parent: String,
    /// "regular" (satellite disc) or "impact" (an F4 giant impact).
    pub origin: String,
    /// Distance from the planet in its radii (impact moons at their birth distance; tides move them, F7).
    pub a_planet_radii: f64,
    pub e: f64,
    pub i_deg: f64,
    pub solid: [u64; 6],
    pub water_from: WaterFrom,
    pub diameter_km: f64,
}

/// start (what F4 left in worlds, belts, reservoirs and impact moons) = worlds + clouds + moons, per group.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct LeftoverBudget {
    pub solid_start: [u64; 6],
    pub solid_in_worlds: [u64; 6],
    pub solid_in_clouds: [u64; 6],
    pub solid_in_moons: [u64; 6],
}

impl LeftoverBudget {
    pub fn closes(&self) -> bool {
        (0..6).all(|g| {
            self.solid_start[g]
                == self.solid_in_worlds[g] + self.solid_in_clouds[g] + self.solid_in_moons[g]
        })
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Leftovers {
    /// F4's worlds after giants gave up their satellite discs (ledger entry `satellite-disc`).
    pub worlds: Vec<World>,
    pub clouds: Vec<Cloud>,
    pub moons: Vec<Moon>,
    pub budget: LeftoverBudget,
}

fn density(solid: &[u64; 6]) -> f64 {
    let t = sum6(solid);
    if t == 0 {
        return 2_000.0;
    }
    let inv: f64 = (0..6)
        .map(|g| solid[g] as f64 / t as f64 / DENSITY[g])
        .sum();
    1.0 / inv
}

fn diameter_km(mass_kg: f64, rho: f64) -> f64 {
    m::cbrt(6.0 * mass_kg / (m::PI * rho)) / 1000.0
}

/// Weight of the k-th object: the top `n` share the cap size, then the size law.
fn weight(k: u64, n: f64) -> f64 {
    let x = k as f64 / n;
    if x <= 1.0 {
        1.0
    } else {
        m::pow(x, -3.0 / SLOPE)
    }
}

/// Sum of k^-1.2 over k > `top`: exact below 1,000, Euler-Maclaurin above
/// (error far below 1e-12 of the sum there).
fn zeta_tail(top: u64) -> f64 {
    let s = 3.0 / SLOPE;
    if top <= 1000 {
        let mut head = 0.0;
        for k in 1..=top {
            head += m::pow(k as f64, -s);
        }
        return ZETA_1_2 - head;
    }
    let k = top as f64;
    m::pow(k, 1.0 - s) / (s - 1.0) - 0.5 * m::pow(k, -s) - s / 12.0 * m::pow(k, -s - 1.0)
}

/// Sum of all weights for `n` top objects: floor(n) ones plus n^1.2 times the zeta tail.
fn weight_sum(n: f64) -> f64 {
    let top = m::floor(n) as u64;
    top as f64 + m::pow(n, 3.0 / SLOPE) * zeta_tail(top)
}

/// Fit the number of cap-size objects so the largest is no bigger than the cap.
fn fit_top(total_kg: f64, cap_kg: f64) -> (f64, f64) {
    if total_kg / ZETA_1_2 <= cap_kg {
        return (1.0, ZETA_1_2);
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

fn kg(units: u64) -> f64 {
    units as f64 * MASS_UNIT * MSUN_KG
}

impl Cloud {
    pub fn mass_earth(&self) -> f64 {
        earth(sum6(&self.solid))
    }
    /// Mass of the k-th largest object (k from 1), kg.
    pub fn object_mass_kg(&self, k: u64) -> f64 {
        kg(sum6(&self.solid)) * weight(k, self.n_top) / self.norm
    }
    /// The exact integer split of the cloud into its `n` largest objects and the rest.
    pub fn split(&self, n: u64) -> (Vec<u64>, u64) {
        let total = sum6(&self.solid);
        let parts: Vec<u64> = (1..=n)
            .map(|k| m::floor(total as f64 * weight(k, self.n_top) / self.norm) as u64)
            .collect();
        let used: u64 = parts.iter().sum();
        (parts, total - used.min(total))
    }
    /// Resolve the k-th largest object (k from 1). A pure function of the seed, the cloud and k.
    pub fn resolve(&self, seed: &Seed, k: u64) -> SmallBody {
        assert!(k >= 1, "objects are ranked from 1");
        let address = self.address.child("obj", k).expect("valid");
        let d = Draws::new(
            *seed,
            address.clone(),
            Stream::new("formation.leftovers.v1").expect("valid"),
        );
        let a = if self.a_log {
            self.a_min_au * m::pow(self.a_max_au / self.a_min_au, d.unit(0))
        } else {
            self.a_min_au + (self.a_max_au - self.a_min_au) * d.unit(0)
        };
        let e = match &self.e {
            Spread::Rayleigh { sigma } => rayleigh(d.unit(1), *sigma),
            Spread::Uniform { lo, hi } => lo + (hi - lo) * d.unit(1),
            Spread::Thermal => m::sqrt(d.unit(1)),
        }
        .min(0.99);
        let i_deg = match &self.i {
            Tilt::Rayleigh { sigma_deg } => rayleigh(d.unit(2), *sigma_deg).min(180.0),
            Tilt::Isotropic => m::acos(1.0 - 2.0 * d.unit(2)) * 180.0 / m::PI,
        };
        let mass_kg = self.object_mass_kg(k);
        SmallBody {
            address,
            kind: self.kind.clone(),
            rank: k,
            diameter_km: diameter_km(mass_kg, density(&self.solid)),
            mass_kg,
            a_au: a,
            e,
            i_deg,
            node_deg: 360.0 * d.unit(3),
            peri_deg: 360.0 * d.unit(4),
            mean_anomaly_deg: 360.0 * d.unit(5),
            swarm: (self.kind == "trojans").then(|| {
                if d.unit(6) < L4_SHARE {
                    "L4".to_string()
                } else {
                    "L5".to_string()
                }
            }),
        }
    }
}

/// Run stage 7 on what the violent phase left.
pub fn leftovers(seed: &Seed, system: &Address, disc: &Disc, u: &Upheaval) -> Leftovers {
    let d = Draws::new(
        *seed,
        system.clone(),
        Stream::new("formation.leftovers.v1").expect("valid"),
    );
    let host = disc.params.host_mass;
    let mut budget = LeftoverBudget::default();
    for w in &u.worlds {
        add(&mut budget.solid_start, &w.solid);
    }
    for b in &u.belts {
        add(&mut budget.solid_start, &b.solid);
    }
    for r in &u.reservoirs {
        add(&mut budget.solid_start, &r.solid);
    }
    for mo in &u.moons {
        add(&mut budget.solid_start, &mo.solid);
    }
    let mut worlds = u.worlds.clone();
    let giants: Vec<(f64, f64)> = worlds
        .iter()
        .filter(|w| w.is_giant())
        .map(|w| (w.a_au, w.e))
        .collect();
    let inner_giant = giants.iter().map(|g| g.0).fold(f64::INFINITY, f64::min);
    let outer_giant = giants.iter().map(|g| g.0).fold(0.0, f64::max);
    let hill =
        |w: &World| w.a_au * (1.0 - w.e) * m::cbrt(w.mass_earth() / (3.0 * MSUN_IN_EARTH * host));
    let r_au = |w: &World| w.radius_earth() * R_EARTH_KM / AU_KM;

    let mut clouds: Vec<Cloud> = vec![];
    let push = |clouds: &mut Vec<Cloud>, mut c: Cloud| {
        if sum6(&c.solid) == 0 {
            return;
        }
        c.address = system.child("cloud", clouds.len() as u64).expect("valid");
        let rho = density(&c.solid);
        let cap_km = if c.kind == "ring" {
            RING_CAP_KM
        } else {
            SIZE_CAP_KM
        };
        let cap_kg = m::PI / 6.0 * rho * m::pow(cap_km * 1000.0, 3.0);
        let (n_top, norm) = fit_top(kg(sum6(&c.solid)), cap_kg);
        c.n_top = n_top;
        c.norm = norm;
        let m1 = c.object_mass_kg(1);
        c.largest_km = diameter_km(m1, rho);
        c.over_1km = n_top * m::pow(c.largest_km.max(1.0), SLOPE);
        clouds.push(c);
    };
    let blank = |kind: &str, source: &str, solid: [u64; 6], water_from: WaterFrom| Cloud {
        address: system.clone(),
        kind: kind.into(),
        around: None,
        source: source.into(),
        a_min_au: 1.0,
        a_max_au: 1.0,
        a_log: false,
        e: Spread::Rayleigh { sigma: 0.05 },
        i: Tilt::Rayleigh { sigma_deg: 3.0 },
        solid,
        water_from,
        largest_km: 0.0,
        over_1km: 0.0,
        n_top: 1.0,
        norm: ZETA_1_2,
    };

    // Belts: F3's rings and F4's impact debris. Stirred belts are hot.
    for b in &u.belts {
        let kind = if b.origin.starts_with("impact:") {
            "debris-belt"
        } else if b.a_au < inner_giant {
            "asteroid-belt"
        } else if b.a_au > outer_giant {
            "outer-belt"
        } else {
            "belt"
        };
        let (sig_e, sig_i) = if b.stirred { (0.15, 8.0) } else { (0.05, 3.0) };
        let mut c = blank(kind, &b.id, b.solid, b.water_from.clone());
        c.a_min_au = b.a_au * 0.85;
        c.a_max_au = b.a_au * 1.2;
        c.e = Spread::Rayleigh { sigma: sig_e };
        c.i = Tilt::Rayleigh { sigma_deg: sig_i };
        push(&mut clouds, c);
    }
    // Reservoirs F4 filled when giants stirred belts.
    for r in &u.reservoirs {
        let belt_a = u
            .belts
            .iter()
            .find(|b| b.id == r.from_belt)
            .map_or(30.0, |b| b.a_au);
        let mut c = blank(&r.kind, &r.from_belt, r.solid, r.water_from.clone());
        match r.kind.as_str() {
            "oort" => {
                let s = m::cbrt(host);
                c.a_min_au = 3_000.0 * s;
                c.a_max_au = 100_000.0 * s;
                c.a_log = true;
                c.e = Spread::Thermal;
                c.i = Tilt::Isotropic;
            }
            "scattered-disc" => {
                c.a_min_au = belt_a * 1.5;
                c.a_max_au = belt_a * 30.0;
                c.a_log = true;
                c.e = Spread::Uniform { lo: 0.2, hi: 0.8 };
                c.i = Tilt::Rayleigh { sigma_deg: 15.0 };
            }
            "trojans" => {
                let gw = worlds.iter().find(|w| Some(&w.id) == r.of.as_ref());
                let ga = gw.map_or(belt_a, |w| w.a_au);
                c.around = r.of.clone();
                c.a_min_au = ga * 0.98;
                c.a_max_au = ga * 1.02;
                c.e = Spread::Rayleigh { sigma: 0.07 };
                c.i = Tilt::Rayleigh { sigma_deg: 12.0 };
            }
            _ => {
                // Irregular moons: 0.1 to 0.5 Hill radii of their giant, eccentric, half retrograde.
                let rh = worlds
                    .iter()
                    .find(|w| Some(&w.id) == r.of.as_ref())
                    .map_or(0.3, hill);
                c.around = r.of.clone();
                c.a_min_au = 0.1 * rh;
                c.a_max_au = 0.5 * rh;
                c.e = Spread::Uniform { lo: 0.1, hi: 0.6 };
                c.i = Tilt::Isotropic;
            }
        }
        push(&mut clouds, c);
    }

    // Regular satellites and rings from each giant's circumplanetary disc.
    let mut moons: Vec<Moon> = vec![];
    // Indexed: the world is changed in place while its index names the moons' address.
    #[allow(clippy::needless_range_loop)]
    for wi in 0..worlds.len() {
        if !worlds[wi].is_giant() {
            continue;
        }
        let id = worlds[wi].id.clone();
        let key = |s: &[u8]| crate::violent::fnv(&[b"moons", id.as_bytes(), s]);
        let total = sum6(&worlds[wi].solid) + worlds[wi].gas;
        let want = m::floor(total as f64 * SATELLITE_SHARE) as u64;
        let mut sol = worlds[wi].solid;
        let taken = take_prop(&mut sol, want, &[0, 1, 2, 3, 4, 5]);
        if sum6(&taken) == 0 {
            continue;
        }
        let mut wf = world_water(&worlds[wi]);
        let mut water = take_water(&mut wf, taken[3]);
        {
            let w = &mut worlds[wi];
            for (x, y) in w.solid.iter_mut().zip(taken.iter()) {
                *x -= y;
            }
            w.ledger.entries.push(WorldEntry::Lost {
                t_myr: u.gas_end_myr,
                cause: "satellite-disc".into(),
                solid: taken,
                gas: 0,
                water_from: water.clone(),
            });
        }
        let mut pool = taken;
        let body = system.child("body", wi as u64).expect("valid");
        let rp_au = r_au(&worlds[wi]);
        // A massive ring from a moon stripped as it spiralled in (Canup 2010).
        if d.unit(key(b"ring")) < RING_CHANCE {
            let want =
                m::floor(sum6(&pool) as f64 * RING_SHARE * (0.5 + 1.5 * d.unit(key(b"ring-m"))))
                    as u64;
            let ring = take_prop(&mut pool, want, &[1, 2, 3]);
            if sum6(&ring) > 0 {
                let rw = take_water(&mut water, ring[3]);
                let mut c = blank("ring", &id, ring, rw);
                c.around = Some(id.clone());
                c.a_min_au = 1.2 * rp_au;
                c.a_max_au = 2.3 * rp_au;
                c.e = Spread::Rayleigh { sigma: 1e-4 };
                c.i = Tilt::Rayleigh { sigma_deg: 0.01 };
                push(&mut clouds, c);
            }
        }
        let n = 2 + d.below(key(b"n"), 4);
        let weights: Vec<f64> = (0..n)
            .map(|j| 0.3 + 0.7 * d.unit(key(&j.to_be_bytes())))
            .collect();
        let wsum: f64 = weights.iter().sum();
        let pool_total = sum6(&pool);
        let mut a_rp = 4.0 + 4.0 * d.unit(key(b"a0"));
        for (j, wj) in weights.iter().enumerate() {
            let last = j as u64 + 1 == n;
            let want = if last {
                sum6(&pool)
            } else {
                m::floor(pool_total as f64 * wj / wsum) as u64
            };
            let part = take_prop(&mut pool, want, &[0, 1, 2, 3, 4, 5]);
            let pw = take_water(&mut water, part[3]);
            let mk = kg(sum6(&part));
            moons.push(Moon {
                address: body.child("moon", j as u64).expect("valid"),
                parent: id.clone(),
                origin: "regular".into(),
                a_planet_radii: a_rp,
                e: 0.01 * d.unit(key(&[b'e', j as u8])),
                i_deg: 0.5 * d.unit(key(&[b'i', j as u8])),
                solid: part,
                water_from: pw,
                diameter_km: diameter_km(mk, density(&part)),
            });
            a_rp *= 1.5 + 0.5 * d.unit(key(&[b'r', j as u8]));
        }
    }
    // F4's impact moons, at their birth distance of a few planet radii.
    for (j, im) in u.moons.iter().enumerate() {
        let wi = worlds.iter().position(|w| w.id == im.parent);
        let body = match wi {
            Some(i) => system.child("body", i as u64).expect("valid"),
            // Its world later merged away or was lost: the moon is addressed under the system.
            None => system.child("lost_moon", j as u64).expect("valid"),
        };
        let mk = kg(sum6(&im.solid));
        moons.push(Moon {
            address: body.child("impact_moon", j as u64).expect("valid"),
            parent: im.parent.clone(),
            origin: "impact".into(),
            a_planet_radii: 3.0
                + 3.0
                    * d.unit(crate::violent::fnv(&[
                        b"impact-moon",
                        &(j as u64).to_be_bytes(),
                    ])),
            e: 0.0,
            i_deg: 0.0,
            solid: im.solid,
            water_from: im.water_from.clone(),
            diameter_km: diameter_km(mk, density(&im.solid)),
        });
    }

    for w in &worlds {
        add(&mut budget.solid_in_worlds, &w.solid);
    }
    for c in &clouds {
        add(&mut budget.solid_in_clouds, &c.solid);
    }
    for mo in &moons {
        add(&mut budget.solid_in_moons, &mo.solid);
    }
    Leftovers {
        worlds,
        clouds,
        moons,
        budget,
    }
}
