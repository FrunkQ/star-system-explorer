//! MP6: a body's elements, worked out on a click (matter pack design
//! section 5, steps 3 and 4).
//!
//! - **Materials.** Formation hands each body its mass of each material
//!   (metal, rock, carbon, water; the CO2 and CO ices count with the water
//!   until they get rows of their own). A world built from facts gets them
//!   from its core share and its water.
//! - **Elements in each material.** Every element rides in the material its
//!   host phase condenses with (`condensation.rides_in` on the sheet). Its
//!   share of that material is its CI chondrite share times its star's ratio
//!   to the Sun, normalised over the material, the oxygen its oxide binds
//!   included (`sse.composition.share_in_material`). Water is the substance
//!   h2o by its formula, with what rides in it beside.
//! - **Reservoirs (Survey only).** How far the world separated
//!   (`sse.composition.separation_strength`) weights the iron-loving
//!   elements' sinking to the core (`sse.composition.core_share`) and the
//!   incompatible ones' rising into the crust (`sse.composition.crust_share`);
//!   the water's elements go to sea and air as far as the world outgassed;
//!   the mantle holds the rest. Totals are the same at both levels.
//!
//! Nothing here runs a surface stage: a click on a body never surveyed reads
//! only its bulk and the sheet.

use serde::{Deserialize, Serialize};
use sse_atoms::matter as mt;
use sse_core::sheet::{M_EARTH, R_EARTH_MEAN};
use sse_core::Address;
use sse_reference::Reference;
use std::collections::BTreeMap;

pub const VERSION: &str = "composition-v1";

/// The formation materials an element can ride in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Material {
    Metal,
    Rock,
    Carbon,
    Water,
    /// Stays in the gas: no share of the solids.
    Gas,
}

impl Material {
    fn from_sheet(id: &str) -> Option<Self> {
        Some(match id {
            "formation-metal" => Material::Metal,
            "formation-rock" => Material::Rock,
            "formation-carbon" => Material::Carbon,
            "formation-water" => Material::Water,
            "gas" => Material::Gas,
            _ => return None,
        })
    }
}

/// One element as the composition chain reads it from the sheet.
#[derive(Clone, PartialEq, Debug)]
pub struct ElementRow {
    pub id: String,
    pub rides_in: Material,
    /// CI chondrite mass share (0 when the sheet has none).
    pub ci: f64,
    /// Oxygen bound per kg of the element in its rock oxide (0: none).
    pub oxide_oxygen: f64,
    /// Concentration in Earth's core over its silicate (None: no datum).
    pub partition: Option<f64>,
    /// Concentration in Earth's crust over its silicate (None: no datum).
    pub enrichment: Option<f64>,
}

/// The water material's make-up: h2o's elements by mass.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Water {
    /// Element mass shares of h2o (H, O).
    pub formula: BTreeMap<String, f64>,
}

/// What the chain reads from the sheet, once per stack.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Sheet {
    pub elements: Vec<ElementRow>,
    pub water: Water,
    /// A gas envelope's elements by mass: hydrogen and helium at their
    /// protosolar shares (MP8c).
    pub gas: BTreeMap<String, f64>,
    /// Isotopes with a protosolar share: id to (element, atom fraction of
    /// the element, mass in u), for isotope resources (helium-3).
    pub isotopes: BTreeMap<String, (String, f64, f64)>,
}

fn num(v: Option<&serde_json::Value>) -> Option<f64> {
    let v = v?;
    v.get("v").and_then(|x| x.as_f64()).or_else(|| v.as_f64())
}

/// Mass shares of a substance's elements from its formula and the atomic
/// weights.
fn mass_shares(r: &Reference, substance: &str) -> BTreeMap<String, f64> {
    let Some(f) = r.formula(substance) else {
        return BTreeMap::new();
    };
    let mut w: BTreeMap<String, f64> = BTreeMap::new();
    for (e, n) in &f {
        if let Some(aw) = r
            .row("elements", e)
            .and_then(|row| num(row.get("atomic_weight")))
        {
            w.insert(e.clone(), n * aw);
        }
    }
    let total: f64 = w.values().sum();
    if total > 0.0 {
        for v in w.values_mut() {
            *v /= total;
        }
    }
    w
}

impl Sheet {
    /// Read the element rows and h2o from a loaded reference.
    pub fn read(r: &Reference) -> Self {
        let water = Water {
            formula: mass_shares(r, "h2o"),
        };
        let mut elements = vec![];
        let mut gas = BTreeMap::new();
        for id in ["h", "he"] {
            let row = r.row("elements", id);
            let n = row.and_then(|x| num(x.get("abundance_protosolar")));
            let aw = row.and_then(|x| num(x.get("atomic_weight")));
            if let (Some(n), Some(aw)) = (n, aw) {
                gas.insert(id.to_string(), n * aw);
            }
        }
        let gsum: f64 = gas.values().sum();
        if gsum > 0.0 {
            for v in gas.values_mut() {
                *v /= gsum;
            }
        }
        let mut isotopes = BTreeMap::new();
        if let Some(rows) = r.tables.get("isotopes") {
            for (id, row) in rows {
                let el = row.get("element").and_then(|x| x.as_str());
                let f = num(row.get("abundance_protosolar"));
                let mu = num(row.get("mass"));
                if let (Some(el), Some(f), Some(mu)) = (el, f, mu) {
                    isotopes.insert(id.clone(), (el.to_string(), f, mu));
                }
            }
        }
        let Some(rows) = r.tables.get("elements") else {
            return Sheet {
                elements,
                water,
                gas,
                isotopes,
            };
        };
        for (id, row) in rows {
            let cond = row.get("condensation");
            let Some(rides_in) = cond
                .and_then(|c| c.get("rides_in"))
                .and_then(|x| x.as_str())
                .and_then(Material::from_sheet)
            else {
                continue;
            };
            let ci = num(row.get("ci_mass_fraction")).unwrap_or(0.0);
            // The oxygen its rock oxide binds, by stoichiometry.
            let oxide_oxygen = cond
                .and_then(|c| c.get("oxide"))
                .and_then(|x| x.as_str())
                .map(|ox| {
                    let s = mass_shares(r, ox);
                    match (s.get(id), s.get("o")) {
                        (Some(e), Some(o)) if *e > 0.0 && id != "o" => o / e,
                        _ => 0.0,
                    }
                })
                .unwrap_or(0.0);
            let earth = row.get("earth_mass_fraction");
            let at = |k: &str| num(earth.and_then(|e| e.get(k)));
            let ratio = |top: Option<f64>| match (top, at("silicate")) {
                (Some(t), Some(s)) if s > 0.0 => Some(t / s),
                _ => None,
            };
            elements.push(ElementRow {
                id: id.clone(),
                rides_in,
                ci,
                oxide_oxygen,
                partition: ratio(at("core")),
                enrichment: ratio(at("crust")),
            });
        }
        Sheet {
            elements,
            water,
            gas,
            isotopes,
        }
    }

    /// An isotope's mass share of its element (atom fraction times its
    /// mass over the element's weight), if the sheet carries its share.
    pub fn isotope_mass_share(&self, r: &Reference, id: &str) -> Option<(String, f64)> {
        let (el, f, mu) = self.isotopes.get(id)?;
        let aw = r
            .row("elements", el)
            .and_then(|row| num(row.get("atomic_weight")))?;
        (aw > 0.0).then(|| (el.clone(), f * mu / aw))
    }
}

/// A body's bulk, kg, as the chain needs it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Bulk {
    pub address: Address,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub metal_kg: f64,
    pub rock_kg: f64,
    pub carbon_kg: f64,
    pub water_kg: f64,
    /// Water at the surface now (oceans, ice caps), kg.
    pub surface_water_kg: f64,
    /// Air, kg (surface pressure times area over gravity).
    pub air_kg: f64,
    /// A gas envelope of hydrogen and helium, kg (MP8c).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub gas_kg: f64,
    /// Al-26 heat per kg left after it formed (0 when unknown).
    pub al26_j_kg: f64,
    /// Where the materials came from, in words.
    pub source: String,
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

/// The mass of one Earth ocean, the unit `oceans` counts in (the sheet's
/// `M_ocean_earth`).
pub const OCEAN_KG: f64 = sse_core::sheet::M_OCEAN_EARTH;

impl Bulk {
    /// From a surface body: metal is the core share, water its water share
    /// or, failing that, its oceans as the outgassed part of what formation
    /// left (Elkins-Tanton 2008's retained share), rock the rest.
    pub fn of_body(b: &crate::Body) -> Self {
        if let Some(mk) = &b.makeup {
            return Self::of_makeup(b, mk);
        }
        let mass = b.mass_earth * M_EARTH;
        let radius = b.radius_earth * R_EARTH_MEAN;
        let surface_water = b.oceans.max(0.0) * OCEAN_KG;
        let water = if b.water_fraction > 0.0 {
            b.water_fraction * mass
        } else {
            surface_water / (1.0 - sse_formation::early::MANTLE_RETAINED)
        };
        let solid = mass.min(mass - water).max(0.0);
        let metal = b.core_mass_fraction.clamp(0.0, 1.0) * solid;
        let g = sse_core::constants::g() * mass / (radius * radius).max(1.0);
        let air = if g > 0.0 {
            b.pressure_bar.max(0.0) * 1e5 * 4.0 * sse_core::math::PI * radius * radius / g
        } else {
            0.0
        };
        Bulk {
            address: b.address.clone(),
            mass_kg: mass,
            radius_m: radius,
            metal_kg: metal,
            rock_kg: (solid - metal).max(0.0),
            carbon_kg: 0.0,
            water_kg: water,
            surface_water_kg: surface_water.min(water),
            air_kg: air,
            gas_kg: 0.0,
            al26_j_kg: 0.0,
            source: "the body's core share and water".into(),
        }
    }

    /// From a body's makeup facts: each share of the mass is its material;
    /// ice is water; gas is the envelope's hydrogen and helium.
    fn of_makeup(b: &crate::Body, mk: &crate::Makeup) -> Self {
        let mut x = Self::of_body(&crate::Body {
            makeup: None,
            ..b.clone()
        });
        let mass = x.mass_kg;
        x.metal_kg = mk.metal.clamp(0.0, 1.0) * mass;
        x.rock_kg = mk.rock.clamp(0.0, 1.0) * mass;
        x.carbon_kg = mk.carbon.clamp(0.0, 1.0) * mass;
        x.water_kg = mk.ice.clamp(0.0, 1.0) * mass;
        x.gas_kg = mk.gas.clamp(0.0, 1.0) * mass;
        x.surface_water_kg = x.surface_water_kg.min(x.water_kg);
        x.source = "the body's makeup facts".into();
        x
    }

    /// A giant whose makeup is not known: its heavy elements are the
    /// mass-metallicity relation's share of its mass
    /// (`sse.composition.heavy_element_share`), the rest an envelope. The
    /// solids keep their proportions.
    pub fn with_envelope(mut self, z_star: f64) -> Self {
        let zp = mt::heavy_element_share(
            z_star,
            self.mass_kg,
            sse_core::sheet::M_JUP,
            mt::GIANT_METAL_COEFFICIENT,
            mt::GIANT_METAL_SLOPE,
        );
        let solid = self.metal_kg + self.rock_kg + self.carbon_kg + self.water_kg;
        if solid <= 0.0 {
            return self;
        }
        let k = zp * self.mass_kg / solid;
        self.metal_kg *= k;
        self.rock_kg *= k;
        self.carbon_kg *= k;
        self.water_kg *= k;
        self.surface_water_kg = self.surface_water_kg.min(self.water_kg);
        self.gas_kg = (1.0 - zp) * self.mass_kg;
        self.source = format!("{} with a giant's envelope", self.source);
        self
    }

    /// From formation's ledger: each material's mass as the body collected
    /// it (Earth masses, `GROUPS` order: metal, rock, carbon, water, CO2,
    /// CO), scaled to the body's solid mass.
    pub fn with_ledger(self, by_group: &[f64; 6]) -> Self {
        self.with_ledger_and_gas(by_group, 0.0)
    }

    /// As `with_ledger`, with the gas the body captured (Earth masses): the
    /// ledger and the gas share the body's whole mass.
    pub fn with_ledger_and_gas(mut self, by_group: &[f64; 6], gas: f64) -> Self {
        let sum: f64 = by_group.iter().sum();
        if sum <= 0.0 {
            return self;
        }
        let gas = gas.max(0.0);
        let solid = if gas > 0.0 {
            self.mass_kg
        } else {
            self.metal_kg + self.rock_kg + self.carbon_kg + self.water_kg
        };
        let k = solid / (sum + gas);
        self.gas_kg = gas * k;
        self.metal_kg = by_group[0] * k;
        self.rock_kg = by_group[1] * k;
        self.carbon_kg = by_group[2] * k;
        self.water_kg = (by_group[3] + by_group[4] + by_group[5]) * k;
        self.surface_water_kg = self.surface_water_kg.min(self.water_kg);
        self.source = "formation's ledger of what the body collected".into();
        self
    }
}

/// Where one element is (Survey).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Reservoirs {
    pub core_kg: f64,
    pub mantle_kg: f64,
    pub crust_kg: f64,
    pub sea_kg: f64,
    pub air_kg: f64,
    /// A giant's hydrogen and helium envelope (MP8c).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub envelope_kg: f64,
}

impl Reservoirs {
    pub fn total(&self) -> f64 {
        self.core_kg + self.mantle_kg + self.crust_kg + self.sea_kg + self.air_kg + self.envelope_kg
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ElementAmount {
    pub id: String,
    pub kg: f64,
    /// Mass share of the whole body.
    pub mass_fraction: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservoirs: Option<Reservoirs>,
}

/// The detail a read is asked at.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// Element totals only.
    Orbit,
    /// Totals and where each element is.
    Survey,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Composition {
    pub version: String,
    pub address: Address,
    pub level: Level,
    pub bulk: Bulk,
    /// How far the world separated into metal and silicate, 0 to 1.
    pub separation_strength: f64,
    /// Metal's share of metal and rock.
    pub metal_fraction: f64,
    /// The crust's share of the silicate (Survey).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crust_fraction: Option<f64>,
    /// Elements, heaviest first.
    pub elements: Vec<ElementAmount>,
    /// What this read leaves out or stands in for, in words.
    pub stand_ins: Vec<String>,
}

/// Settings the chain's blocks take (their defaults are the blocks').
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub e_melt_j_kg: f64,
    pub crust_thickness_m: f64,
    pub crust_density: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            e_melt_j_kg: 1.8e6,
            crust_thickness_m: 2.0e4,
            crust_density: sse_core::sheet::mixture_density::CRUST_ROCK,
        }
    }
}

/// An element with no partition datum that rides in the metal stays with
/// it: an effectively endless preference for metal.
const METAL_RIDER_PARTITION: f64 = 1.0e6;

/// A body's elements at `level`. `ratio` is the star's abundance of each
/// element over the Sun's (missing reads as 1).
pub fn compose(
    sheet: &Sheet,
    bulk: &Bulk,
    ratio: &BTreeMap<String, f64>,
    level: Level,
    s: &Settings,
) -> Composition {
    let r = |id: &str| ratio.get(id).copied().unwrap_or(1.0).max(0.0);
    // Riders: hydrogen and oxygen come with their carriers (water's formula,
    // rock's oxides), not as CI riders of their own.
    let riders: Vec<&ElementRow> = sheet
        .elements
        .iter()
        .filter(|e| e.id != "o" && e.id != "h" && e.ci > 0.0)
        .collect();
    let weight = |mat: Material| -> f64 {
        riders
            .iter()
            .filter(|e| e.rides_in == mat)
            .map(|e| {
                let o = if mat == Material::Rock {
                    e.oxide_oxygen
                } else {
                    0.0
                };
                e.ci * r(&e.id) * (1.0 + o)
            })
            .sum()
    };
    let (w_metal, w_rock, w_carbon) = (
        weight(Material::Metal),
        weight(Material::Rock),
        weight(Material::Carbon),
    );
    // Water: h2o by formula, CI-weighted by its hydrogen (CI water is the
    // hydrogen over hydrogen's share of h2o), with the riders beside it.
    let h_share = sheet.water.formula.get("h").copied().unwrap_or(0.0);
    let ci_h = sheet
        .elements
        .iter()
        .find(|e| e.id == "h")
        .map(|e| e.ci * r("h"))
        .unwrap_or(0.0);
    let w_h2o = if h_share > 0.0 { ci_h / h_share } else { 0.0 };
    let w_water = w_h2o + weight(Material::Water);

    // Mass of each element by the material it came in.
    let mut by_mat: BTreeMap<String, [f64; 5]> = BTreeMap::new();
    let mut add = |id: &str, slot: usize, kg: f64| {
        if kg > 0.0 {
            by_mat.entry(id.to_string()).or_insert([0.0; 5])[slot] += kg;
        }
    };
    for e in &riders {
        let (mat_kg, total, slot) = match e.rides_in {
            Material::Metal => (bulk.metal_kg, w_metal, 0),
            Material::Rock => (bulk.rock_kg, w_rock, 1),
            Material::Carbon => (bulk.carbon_kg, w_carbon, 2),
            Material::Water => (bulk.water_kg, w_water, 3),
            Material::Gas => continue,
        };
        let oxygen = if e.rides_in == Material::Rock {
            e.oxide_oxygen
        } else {
            0.0
        };
        let (share, bound) = mt::share_in_material(e.ci, r(&e.id), oxygen, total);
        add(&e.id, slot, mat_kg * share);
        add("o", slot, mat_kg * bound);
    }
    if w_water > 0.0 {
        let h2o_kg = bulk.water_kg * w_h2o / w_water;
        for (el, f) in &sheet.water.formula {
            add(el, 3, h2o_kg * f);
        }
    }
    // A giant's envelope: hydrogen and helium at their protosolar shares.
    for (el, f) in &sheet.gas {
        add(el, 4, bulk.gas_kg * f);
    }

    let g = sse_core::constants::g();
    let strength = mt::separation_strength(
        g,
        bulk.mass_kg,
        bulk.radius_m,
        bulk.al26_j_kg,
        s.e_melt_j_kg,
    );
    let metal_fraction = {
        let mr = bulk.metal_kg + bulk.rock_kg;
        if mr > 0.0 {
            bulk.metal_kg / mr
        } else {
            0.0
        }
    };
    let silicate_kg = bulk.rock_kg;
    let crust_fraction = (level == Level::Survey).then(|| {
        mt::crust_fraction(
            bulk.radius_m,
            silicate_kg,
            s.crust_thickness_m,
            s.crust_density,
        )
    });
    let outgassed = if bulk.water_kg > 0.0 {
        (bulk.surface_water_kg / bulk.water_kg).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let rows: BTreeMap<&str, &ElementRow> =
        sheet.elements.iter().map(|e| (e.id.as_str(), e)).collect();
    let in_formula = |id: &str| sheet.water.formula.contains_key(id);

    let mut elements: Vec<ElementAmount> = by_mat
        .iter()
        .map(|(id, kg)| {
            let total: f64 = kg.iter().sum();
            let reservoirs = crust_fraction.map(|fc| {
                let row = rows.get(id.as_str());
                let d = row
                    .and_then(|e| e.partition)
                    .or_else(|| {
                        (row.map(|e| e.rides_in) == Some(Material::Metal))
                            .then_some(METAL_RIDER_PARTITION)
                    })
                    .unwrap_or(0.0);
                let e = row.and_then(|e| e.enrichment).unwrap_or(1.0);
                // What came in solids separates by partition; what came in
                // the water goes to sea and air as far as the world
                // outgassed, the rest staying in the mantle.
                let solid = kg[0] + kg[1] + kg[2];
                let core = solid * mt::core_share(d, metal_fraction, strength);
                let crust = (solid - core) * mt::crust_share(e, fc, strength);
                let up = kg[3] * outgassed;
                let (sea, air) = if in_formula(id) {
                    (up, 0.0)
                } else if bulk.air_kg > 0.0 {
                    (0.0, up)
                } else {
                    (0.0, 0.0)
                };
                let envelope = kg[4];
                let kept = sea + air + envelope;
                Reservoirs {
                    core_kg: core,
                    crust_kg: crust,
                    sea_kg: sea,
                    air_kg: air,
                    envelope_kg: envelope,
                    mantle_kg: (total - core - crust - kept).max(0.0),
                }
            });
            ElementAmount {
                id: id.clone(),
                kg: total,
                mass_fraction: if bulk.mass_kg > 0.0 {
                    total / bulk.mass_kg
                } else {
                    0.0
                },
                reservoirs,
            }
        })
        .collect();
    elements.sort_by(|a, b| b.kg.total_cmp(&a.kg).then_with(|| a.id.cmp(&b.id)));

    let mut stand_ins = vec![
        "elements are shares of six coarse formation materials, by CI chondrite make-up; no element is lost to its own volatility yet (MP14)".to_string(),
    ];
    if ratio.is_empty() {
        stand_ins.push(
            "no formation run for this system, so its star's elements read as the Sun's".into(),
        );
    }
    if bulk.carbon_kg <= 0.0 {
        stand_ins.push("no carbon material known for this body, so no carbon".into());
    }
    if level == Level::Survey {
        stand_ins.push("core and crust shares are Earth's measured partitions as single values, not functions of pressure and oxygen".into());
        stand_ins.push(format!(
            "the crust is a {:.0} km shell until the surface stage hands each world its own",
            s.crust_thickness_m / 1000.0
        ));
    }
    Composition {
        version: VERSION.into(),
        address: bulk.address.clone(),
        level,
        bulk: bulk.clone(),
        separation_strength: strength,
        metal_fraction,
        crust_fraction,
        elements,
        stand_ins,
    }
}

// ---------------------------------------------------------------- MP8 deposits

pub const HOLDINGS_VERSION: &str = "deposit-holdings-v1";

/// One element a deposit kind holds.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Held {
    pub id: String,
    pub kg: f64,
    /// Mass share of the ore.
    pub grade: f64,
}

/// What one deposit kind holds across a world.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Holding {
    pub kind: String,
    /// Plate cells (provinces) and point deposits of this kind.
    pub places: usize,
    pub footprint_km2: f64,
    pub ore_kg: f64,
    pub elements: Vec<Held>,
}

/// One point deposit's share (an impact melt sheet, a cold trap).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlaceHolding {
    pub id: String,
    pub kind: String,
    pub elements: Vec<Held>,
}

/// An element's crust split into deposits and the rest: the two always add
/// to the crust that `read_composition` gives at Survey.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CrustSplit {
    pub id: String,
    pub crust_kg: f64,
    pub in_deposits_kg: f64,
    pub disseminated_kg: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Holdings {
    pub version: String,
    pub depth_m: f64,
    pub ore_share: f64,
    pub kinds: Vec<Holding>,
    pub places: Vec<PlaceHolding>,
    pub crust: Vec<CrustSplit>,
    pub references: Vec<String>,
    pub stand_ins: Vec<String>,
}

/// MP8: what a world's deposits hold, drawn from its crust (Survey
/// composition). Each kind's footprint is its plate cells (a sphere's area
/// shared evenly over the cube grid) plus its point deposits' discs; the
/// amounts are `sse.composition.deposit_amount`; where the deposits would
/// hold more of an element than the crust has, all are scaled to the crust.
pub fn holdings(
    comp: &Composition,
    layers: &crate::cover::Layers,
    ores: &crate::cover::Ores,
    density: f64,
) -> Holdings {
    use crate::cover::DEPOSITS;
    let r = comp.bulk.radius_m;
    let crust_of = |id: &str| -> f64 {
        comp.elements
            .iter()
            .find(|e| e.id == id)
            .and_then(|e| e.reservoirs.as_ref())
            .map(|x| x.crust_kg)
            .unwrap_or(0.0)
    };
    let crust_mass: f64 = comp
        .elements
        .iter()
        .filter_map(|e| e.reservoirs.as_ref())
        .map(|x| x.crust_kg)
        .sum();
    let cell_area = if layers.cell_n > 0 {
        4.0 * sse_core::math::PI * r * r / (6.0 * (layers.cell_n as f64) * (layers.cell_n as f64))
    } else {
        0.0
    };
    let conc = |id: &str| {
        if crust_mass > 0.0 {
            crust_of(id) / crust_mass
        } else {
            0.0
        }
    };
    // Footprints: plate cells and point discs, per kind.
    let mut kinds: Vec<Holding> = vec![];
    let mut places: Vec<PlaceHolding> = vec![];
    for (kind, grades) in &ores.kinds {
        let code = DEPOSITS.iter().position(|k| k == kind).unwrap_or(0) as u8;
        let cells = if code == 0 {
            0
        } else {
            layers.cell_deposit.iter().filter(|c| **c == code).count()
        };
        let points: Vec<_> = layers.deposits.iter().filter(|d| d.kind == *kind).collect();
        let disc = |d: &crate::cover::Deposit| {
            let rr = d.radius_km * 1000.0;
            sse_core::math::PI * rr * rr
        };
        let area = cells as f64 * cell_area + points.iter().map(|d| disc(d)).sum::<f64>();
        let held = |a: f64| -> Vec<Held> {
            grades
                .iter()
                .map(|(id, k)| {
                    let (kg, grade) =
                        mt::deposit_amount(a, ores.depth_m, density, ores.ore_share, conc(id), *k);
                    Held {
                        id: id.clone(),
                        kg,
                        grade,
                    }
                })
                .collect()
        };
        for d in &points {
            places.push(PlaceHolding {
                id: d.id.clone(),
                kind: kind.clone(),
                elements: held(disc(d)),
            });
        }
        kinds.push(Holding {
            kind: kind.clone(),
            places: cells + points.len(),
            footprint_km2: area / 1.0e6,
            ore_kg: area * ores.depth_m.max(0.0) * density * ores.ore_share.clamp(0.0, 1.0),
            elements: held(area),
        });
    }
    // Never more than the crust holds: scale each element down to it.
    let mut ids: Vec<String> = kinds
        .iter()
        .flat_map(|h| h.elements.iter().map(|e| e.id.clone()))
        .collect();
    ids.sort();
    ids.dedup();
    let mut crust = vec![];
    let mut capped = vec![];
    for id in &ids {
        let have = crust_of(id);
        let want: f64 = kinds
            .iter()
            .flat_map(|h| h.elements.iter())
            .filter(|e| e.id == *id)
            .map(|e| e.kg)
            .sum();
        if want > have && want > 0.0 {
            let k = have / want;
            for e in kinds
                .iter_mut()
                .flat_map(|h| h.elements.iter_mut())
                .chain(places.iter_mut().flat_map(|p| p.elements.iter_mut()))
                .filter(|e| e.id == *id)
            {
                e.kg *= k;
            }
            capped.push(id.clone());
        }
        let held = want.min(have);
        crust.push(CrustSplit {
            id: id.clone(),
            crust_kg: have,
            in_deposits_kg: held,
            disseminated_kg: have - held,
        });
    }
    let mut stand_ins = vec![
        format!(
            "one ore share ({:.1e}) for every kind, calibrated on Earth's porphyry copper; each kind's own endowment is not yet separate",
            ores.ore_share
        ),
        "plate cells count as equal shares of the sphere; a province's ore is counted evenly over it".into(),
    ];
    let unheld: Vec<&str> = DEPOSITS[1..]
        .iter()
        .copied()
        .filter(|k| !ores.kinds.contains_key(*k))
        .collect();
    if !unheld.is_empty() {
        stand_ins.push(format!("not yet counted: {}", unheld.join(", ")));
    }
    if !capped.is_empty() {
        stand_ins.push(format!(
            "deposits would hold more {} than the crust has, so they are scaled to the crust",
            capped.join(", ")
        ));
    }
    Holdings {
        version: HOLDINGS_VERSION.into(),
        depth_m: ores.depth_m,
        ore_share: ores.ore_share,
        kinds,
        places,
        crust,
        references: ores.references.clone(),
        stand_ins,
    }
}
