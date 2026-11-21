//! Boxes F and G: ecosystem shape and resolved guilds with traits derived
//! from the world (life design section 7, stage L1c).
//!
//! `resolve` turns a world's box B budget and box D capabilities into
//! 1 to about 40 aggregate guilds (never individuals): producers per
//! metabolism and habitat kind with their share of production, decomposers,
//! consumers per feeding level, symbionts and parasites; relations in the
//! four lenses (ancestry, resource, habitat, capability); a diversity
//! estimate (species-area law with provinces and climate variety); and
//! world traits derived from physics, then varied by keyed draws inside
//! physical limits (producer colour, largest land animal, flight, tallest
//! plants, pace of life, day strategies, senses, biochemistry). Names come in
//! two vocabulary layers: `physics` and `plain`.

use crate::budget::{self, BudgetParams, WorldBudget};
use crate::habitat::HabitatGraph;
use crate::history::HistoryParams;
use crate::*;
use sse_core::math as m;
use sse_core::{Draws, Seed, Stream};

/// The world's physics that traits need, at the epoch (from the planet
/// layer; fixture values here).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldPhysics {
    pub surface_gravity_m_s2: f64,
    /// Surface air density, kg/m^3; `None`: no atmosphere.
    pub air_density_kg_m3: Option<f64>,
    pub star_teff_k: f64,
    pub day_hours: f64,
    pub obliquity_deg: f64,
    #[serde(default)]
    pub tidally_locked: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GuildDetail {
    pub id: String,
    /// `producer`, `decomposer`, `consumer`, `symbiont`, `parasite`.
    pub role: String,
    /// 1 for producers and decomposers, 2 grazers, 3+ predators.
    pub level: u32,
    pub habitat_kind: String,
    /// Primary production fixed by this guild, g C per year (producers).
    pub production_gc_yr: f64,
    /// Its share of the world's primary production (producers sum to 1).
    pub share: f64,
    /// Carbon it takes in, g C per year (consumers and decomposers).
    pub intake_gc_yr: f64,
    pub physics_name: String,
    pub plain_name: String,
    /// `microbial`, `protist`, `multicellular`, `animal`, `plant`.
    pub grade: String,
    pub can_fly: bool,
    /// Typical largest body, kg.
    pub max_mass_kg: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Diversity {
    /// Habitable area in use, m^2.
    pub area_m2: f64,
    /// Separate living habitats (provinces).
    pub provinces: u32,
    /// Distinct 10 K climate bins among living habitats.
    pub climate_bins: u32,
    pub since_catastrophe_myr: Option<f64>,
    /// Rough count of lineages at the rank of a family (sketch).
    pub lineages: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldTraits {
    /// Wavelength of peak photon flux at the surface, nm.
    pub photon_peak_nm: f64,
    pub producer_colour: Option<String>,
    /// The colours physics allows here; the draw picked one.
    pub colour_options: Vec<String>,
    /// `feasible`, `marginal`, `impossible`, `not_evolved`.
    pub flight: String,
    pub max_flyer_kg: Option<f64>,
    pub max_land_animal_kg: Option<f64>,
    pub tallest_plant_m: Option<f64>,
    /// Generation time relative to Earth's at 288 K.
    pub pace: f64,
    pub day_strategies: Vec<String>,
    pub senses: Vec<String>,
    pub handedness: String,
    pub solvent: String,
    pub origin: String,
    pub to_visitors: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Biosphere {
    pub resolved: ResolvedBiosphere,
    pub guilds: Vec<GuildDetail>,
    pub diversity: Diversity,
    pub traits: WorldTraits,
    pub budget: WorldBudget,
    pub capabilities: Vec<String>,
}

/// Box B for the capabilities a world holds (the same rule as the history:
/// metabolisms and media unlock with capabilities).
pub fn budget_with(
    g: &HabitatGraph,
    bp: &BudgetParams,
    hp: &HistoryParams,
    caps: &[String],
) -> WorldBudget {
    let held: Vec<&history::Capability> = hp
        .capabilities
        .iter()
        .filter(|c| caps.contains(&c.id))
        .collect();
    let mets: Vec<String> = held
        .iter()
        .flat_map(|c| c.unlocks_metabolisms.clone())
        .collect();
    let mut media = hp.origin_media.clone();
    for c in &held {
        media.extend(c.unlocks_media.clone());
    }
    let mut bp2 = bp.clone();
    for mb in &mut bp2.metabolisms {
        mb.media.retain(|x| media.contains(x));
    }
    budget::world_budget(g, &bp2, &mets)
}

const SIZE_EARTH_LAND_KG: f64 = 7.0e4;
const SIZE_EARTH_FLYER_KG: f64 = 250.0;
const PLANT_EARTH_M: f64 = 130.0;
const EARTH_G: f64 = 9.81;
const EARTH_AIR: f64 = 1.225;

/// Plain words for a habitat kind.
fn plain_kind(kind: &str) -> &str {
    match kind {
        "ocean-sunlit" => "sunlit sea",
        "ocean-interior" => "deep water",
        "sea-floor" => "sea floor",
        "under-ice-ocean" => "dark ocean under the ice",
        "land-surface" => "land",
        "deep-crust" => "deep rock",
        "cloud-deck" => "clouds",
        "terminator-zone" => "twilight ring",
        other => other,
    }
}

fn pick(d: &Draws, i: u64, opts: &[&str]) -> String {
    let k = (d.unit(i) * opts.len() as f64) as usize;
    opts[k.min(opts.len() - 1)].to_string()
}

/// Resolve a living world at an epoch. `caps` are the capabilities held
/// (from the history); `since` the first age of each (Myr since the world's
/// birth), `birth_year` dates them.
#[allow(clippy::too_many_arguments)]
pub fn resolve(
    g: &HabitatGraph,
    bp: &BudgetParams,
    hp: &HistoryParams,
    caps: &[String],
    first: &BTreeMap<String, f64>,
    since_catastrophe_myr: Option<f64>,
    physics: &WorldPhysics,
    seed: Seed,
    birth_year: i64,
) -> Result<Biosphere, String> {
    let mut caps: Vec<String> = caps.to_vec();
    caps.sort();
    caps.dedup();
    let has = |c: &str| caps.iter().any(|x| x == c);
    // Order-independent: habitats sorted by address.
    let mut g = g.clone();
    g.habitats.sort_by(|a, b| a.address.cmp(&b.address));
    let wb = budget_with(&g, bp, hp, &caps);
    let draws = Draws::new(
        seed,
        g.body.clone(),
        Stream::new("life.traits.v1").map_err(|e| e.to_string())?,
    );
    let when = |c: &str| {
        first
            .get(c)
            .map(|t| UniverseTime::years(birth_year + m::round(t * 1.0e6) as i64))
    };

    // Producers: per metabolism and habitat kind, attributing each habitat's
    // production to the metabolisms in proportion to their energy caps (the
    // best light one only, as box B counts it).
    let mut prod: BTreeMap<(String, String), (f64, BTreeMap<Address, f64>)> = BTreeMap::new();
    let mut kinds_alive: Vec<String> = vec![];
    for h in &wb.habitats {
        let Some(p) = h.production_gc_yr.filter(|x| *x > 0.0) else {
            continue;
        };
        let lights: Vec<(&String, &f64)> = h
            .by_metabolism
            .iter()
            .filter(|(id, _)| {
                bp.metabolisms
                    .iter()
                    .any(|mb| &mb.id == *id && mb.channel == "light")
            })
            .collect();
        let best = lights
            .iter()
            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
            .map(|x| x.0.clone());
        let counted: Vec<(String, f64)> = h
            .by_metabolism
            .iter()
            .filter(|(id, _)| {
                let light = lights.iter().any(|l| l.0 == *id);
                !light || best.as_ref() == Some(*id)
            })
            .map(|(id, v)| (id.clone(), *v))
            .collect();
        let total: f64 = counted.iter().map(|x| x.1).sum();
        if total <= 0.0 {
            continue;
        }
        for (id, v) in counted {
            let e = prod.entry((id, h.kind.clone())).or_default();
            e.0 += p * v / total;
            *e.1.entry(h.habitat.clone()).or_default() += p * v / total;
        }
        if !kinds_alive.contains(&h.kind) {
            kinds_alive.push(h.kind.clone());
        }
    }
    let world_prod: f64 = prod.values().map(|x| x.0).sum();
    let mut guilds: Vec<GuildDetail> = vec![];
    let mut out_guilds: Vec<Guild> = vec![];
    let mut relations: Vec<Relation> = vec![];
    let rel = |rels: &mut Vec<Relation>, kind: RelationKind, from: &str, to: &str| {
        rels.push(Relation {
            kind,
            from: from.into(),
            to: to.into(),
        })
    };
    let multicell = has("multicellularity");
    let animals = has("large_bodies");
    let land_life = has("land");
    let complex = has("complex_cells");
    let lineage_of = |metab: &str| -> &'static str {
        match metab {
            "oxygenic" => "oxygenic",
            "anoxygenic-iron" | "anoxygenic-hydrogen" => "anoxygenic",
            _ => "chemotrophy",
        }
    };
    let res_years = |kind: &str| {
        bp.residence_yr
            .get(kind)
            .copied()
            .unwrap_or(bp.default_residence_yr)
    };
    for ((metab, kind), (p, habs)) in &prod {
        let id = format!("producer.{metab}.{kind}");
        let pk = plain_kind(kind);
        let mb = bp.metabolisms.iter().find(|x| &x.id == metab);
        let label = mb.map_or(metab.clone(), |x| x.label.clone());
        let on_land = kind == "land-surface";
        let grade = if metab == "oxygenic" && on_land && land_life {
            "plant"
        } else if multicell && metab == "oxygenic" {
            "multicellular"
        } else {
            "microbial"
        };
        let plain = match (metab.as_str(), grade) {
            ("oxygenic", "plant") => "forests and grasslands".to_string(),
            ("oxygenic", "multicellular") => format!("seaweeds and green scums of the {pk}"),
            ("oxygenic", _) => format!("green films and plankton of the {pk}"),
            ("anoxygenic-iron", _) => "rusty purple mats".into(),
            ("anoxygenic-hydrogen", _) => "purple and brown slimes".into(),
            ("methanogen", _) => "gas-making microbes".into(),
            _ => "rock-eating microbes".into(),
        };
        guilds.push(GuildDetail {
            id: id.clone(),
            role: "producer".into(),
            level: 1,
            habitat_kind: kind.clone(),
            production_gc_yr: *p,
            share: if world_prod > 0.0 {
                p / world_prod
            } else {
                0.0
            },
            intake_gc_yr: 0.0,
            physics_name: format!("{label} ({kind})"),
            plain_name: plain,
            grade: grade.into(),
            can_fly: false,
            max_mass_kg: None,
        });
        let cap = lineage_of(metab);
        out_guilds.push(Guild {
            id: id.clone(),
            name: format!("{label} ({kind})"),
            capabilities: vec![cap.to_string()],
            habitats: habs.iter().map(|(a, v)| (a.clone(), v / p)).collect(),
            biomass_kg: Some(p * res_years(kind) / 1000.0 / 0.5),
            since: when(cap),
        });
        rel(&mut relations, RelationKind::Capability, &id, cap);
        rel(
            &mut relations,
            RelationKind::Ancestry,
            &id,
            &format!("lineage:{cap}"),
        );
        for a in habs.keys() {
            rel(&mut relations, RelationKind::Habitat, &id, &a.to_string());
        }
    }
    // Physics of the world: traits.
    let gr = physics.surface_gravity_m_s2 / EARTH_G;
    let air = physics.air_density_kg_m3.filter(|x| *x > 0.0);
    let max_flyer =
        air.map(|rho| SIZE_EARTH_FLYER_KG * m::pow(rho / EARTH_AIR, 1.5) * m::pow(gr, -3.5));
    let flight_physics = match max_flyer {
        None => "impossible",
        Some(x) if x >= 1.0 => "feasible",
        Some(x) if x >= 1.0e-4 => "marginal",
        _ => "impossible",
    };
    let flight = if flight_physics == "impossible" {
        "impossible"
    } else if !(animals && land_life) {
        "not_evolved"
    } else {
        flight_physics
    };
    let max_land = (animals && land_life).then(|| SIZE_EARTH_LAND_KG * m::pow(gr, -3.0));
    let tallest = (land_life && has("oxygenic")).then(|| PLANT_EARTH_M / gr);
    // Producer colour: pigments absorb where photons are plentiful.
    let photon_peak = 3.670e6 / physics.star_teff_k;
    let light_producers: Vec<&String> = prod
        .keys()
        .map(|k| &k.0)
        .filter(|k| k.starts_with("oxygenic") || k.starts_with("anoxygenic"))
        .collect();
    let options: Vec<&str> = if light_producers.is_empty() {
        vec![]
    } else if !light_producers.iter().any(|k| k.as_str() == "oxygenic") {
        vec!["purple", "brown", "olive green"]
    } else if photon_peak < 550.0 {
        vec!["blue-green", "yellow", "orange"]
    } else if photon_peak < 750.0 {
        vec!["green", "green", "yellow-green", "red-brown"]
    } else if photon_peak < 1000.0 {
        vec!["dark red", "black", "purple"]
    } else {
        vec!["black", "dark grey"]
    };
    let colour = (!options.is_empty()).then(|| pick(&draws, 0, &options));
    let mut opts_out: Vec<String> = options.iter().map(|s| s.to_string()).collect();
    opts_out.dedup();
    // Pace of life from temperature (metabolic theory).
    let temps: Vec<f64> = wb
        .habitats
        .iter()
        .filter(|h| h.production_gc_yr.is_some_and(|p| p > 0.0))
        .filter_map(|h| g.habitats.iter().find(|x| x.address == h.habitat))
        .filter_map(|h| h.profile.temperature_k.map(|t| 0.5 * (t.lo + t.hi)))
        .collect();
    let t_mean = if temps.is_empty() {
        hp.reference_k
    } else {
        temps.iter().sum::<f64>() / temps.len() as f64
    };
    let pace = m::exp(hp.activation_ev / 8.617_333e-5 * (1.0 / t_mean - 1.0 / hp.reference_k));
    let mut day = vec![];
    if physics.tidally_locked
        && kinds_alive
            .iter()
            .any(|k| k == "land-surface" || k == "terminator-zone")
    {
        day.push("terminator dwellers and migrators along the twilight ring".to_string());
    }
    if physics.day_hours > 100.0 && !physics.tidally_locked {
        day.push("long sleepers through days and nights of many Earth days".to_string());
    }
    if physics.obliquity_deg > 40.0 {
        day.push("strong seasons: long migrations and seasonal dormancy".to_string());
    }
    if day.is_empty() && physics.day_hours <= 100.0 {
        day.push("day and night rhythms".to_string());
    }
    let sunlit = kinds_alive
        .iter()
        .any(|k| k == "ocean-sunlit" || k == "land-surface" || k == "lake" || k == "cloud-deck");
    let salty = g.habitats.iter().any(|h| {
        h.profile
            .properties
            .get("salinity")
            .is_some_and(|s| *s > 0.01)
    });
    let mut senses = vec!["chemical sensing".to_string()];
    if multicell {
        if sunlit {
            senses.push("vision".into());
        }
        senses.push("hearing in water".into());
        if air.is_some() && land_life {
            senses.push("hearing in air".into());
        }
        if salty {
            senses.push("electric sense in salty water".into());
        }
        if !sunlit {
            senses.push("no vision: darkness everywhere life lives".into());
        }
    }
    let solvent = g
        .habitats
        .first()
        .map(|h| h.profile.solvent.clone())
        .or_else(|| bp.chemistry.first().map(|c| c.solvent.clone()))
        .or_else(|| bp.chemistries.first().cloned())
        .unwrap_or_default();
    let traits = WorldTraits {
        photon_peak_nm: photon_peak,
        producer_colour: colour,
        colour_options: opts_out,
        flight: flight.into(),
        max_flyer_kg: if flight == "feasible" || flight == "marginal" {
            max_flyer
        } else {
            None
        },
        max_land_animal_kg: max_land,
        tallest_plant_m: tallest,
        pace,
        day_strategies: day,
        senses,
        handedness: pick(
            &draws,
            1,
            &["left-handed amino acids", "right-handed amino acids"],
        ),
        solvent,
        origin: "its own origin (no transfer between worlds is modelled yet)".into(),
        to_visitors:
            "biochemically unrelated to visitors: probably inert, rarely toxic, not infectious"
                .into(),
    };

    // Decomposers and consumers per habitat kind with producers.
    let eff = bp.transfer_efficiency;
    let mut kind_prod: BTreeMap<String, f64> = BTreeMap::new();
    for ((_, kind), (p, _)) in &prod {
        *kind_prod.entry(kind.clone()).or_default() += p;
    }
    for (kind, p) in &kind_prod {
        let levels = wb
            .habitats
            .iter()
            .filter(|h| &h.kind == kind)
            .filter_map(|h| h.feeding_levels)
            .max()
            .unwrap_or(1);
        let did = format!("decomposer.{kind}");
        let pk = plain_kind(kind);
        guilds.push(GuildDetail {
            id: did.clone(),
            role: "decomposer".into(),
            level: 1,
            habitat_kind: kind.clone(),
            production_gc_yr: 0.0,
            share: 0.0,
            intake_gc_yr: p * (1.0 - eff),
            physics_name: format!("heterotrophic decomposers ({kind})"),
            plain_name: if kind == "land-surface" && land_life {
                "fungi and soil life".into()
            } else {
                format!("rotters of the {pk}")
            },
            grade: if complex {
                "protist".into()
            } else {
                "microbial".into()
            },
            can_fly: false,
            max_mass_kg: None,
        });
        for pid in guilds
            .iter()
            .filter(|x| x.role == "producer" && &x.habitat_kind == kind)
            .map(|x| x.id.clone())
            .collect::<Vec<_>>()
        {
            rel(&mut relations, RelationKind::Resource, &did, &pid);
        }
        let mut below: Vec<String> = guilds
            .iter()
            .filter(|x| x.role == "producer" && &x.habitat_kind == kind)
            .map(|x| x.id.clone())
            .collect();
        for lv in 2..=levels.max(1) {
            let id = format!("consumer.{lv}.{kind}");
            let on_land = kind == "land-surface";
            let (grade, plain, phys) = if animals && (!on_land || land_life) {
                (
                    "animal",
                    if lv == 2 {
                        format!("grazers of the {pk}")
                    } else {
                        if lv == 3 {
                            format!("hunters of the {pk}")
                        } else {
                            format!("top hunters (level {lv}) of the {pk}")
                        }
                    },
                    format!("metazoan consumers, level {lv}"),
                )
            } else if complex {
                (
                    "protist",
                    format!("single-celled grazers of the {pk}"),
                    format!("protist consumers, level {lv}"),
                )
            } else {
                (
                    "microbial",
                    format!("microbe-eating microbes and viruses of the {pk}"),
                    format!("microbial predators and viruses, level {lv}"),
                )
            };
            let max_mass = match grade {
                "animal" if on_land => max_land.map(|x| x * m::pow(eff, f64::from(levels - lv))),
                "animal" => Some(1.5e5 * m::pow(eff, f64::from(levels - lv))),
                "protist" => Some(1.0e-9),
                _ => Some(1.0e-15),
            };
            let can_fly = grade == "animal"
                && on_land
                && (flight == "feasible" || flight == "marginal")
                && lv >= 2
                && draws.unit(100 + u64::from(lv)) < 0.5;
            let max_mass = if can_fly {
                max_mass.map(|x| x.min(max_flyer.unwrap_or(0.0)))
            } else {
                max_mass
            };
            guilds.push(GuildDetail {
                id: id.clone(),
                role: "consumer".into(),
                level: lv,
                habitat_kind: kind.clone(),
                production_gc_yr: 0.0,
                share: 0.0,
                intake_gc_yr: p * m::pow(eff, f64::from(lv - 1)),
                physics_name: phys,
                plain_name: if can_fly {
                    format!("{plain}, some flying")
                } else {
                    plain
                },
                grade: grade.into(),
                can_fly,
                max_mass_kg: max_mass,
            });
            for b in &below {
                rel(&mut relations, RelationKind::Resource, &id, b);
            }
            rel(
                &mut relations,
                RelationKind::Capability,
                &id,
                if grade == "animal" {
                    "large_bodies"
                } else if grade == "protist" {
                    "complex_cells"
                } else {
                    "chemotrophy"
                },
            );
            below = vec![id];
        }
        if complex {
            let id = format!("symbiont.{kind}");
            guilds.push(GuildDetail {
                id: id.clone(),
                role: "symbiont".into(),
                level: 1,
                habitat_kind: kind.clone(),
                production_gc_yr: 0.0,
                share: 0.0,
                intake_gc_yr: 0.0,
                physics_name: format!("mutualist symbionts ({kind})"),
                plain_name: format!("symbiotic partners in the {pk}"),
                grade: "microbial".into(),
                can_fly: false,
                max_mass_kg: Some(1.0e-15),
            });
            rel(
                &mut relations,
                RelationKind::Capability,
                &id,
                "complex_cells",
            );
        }
        if levels >= 2 {
            let id = format!("parasite.{kind}");
            guilds.push(GuildDetail {
                id: id.clone(),
                role: "parasite".into(),
                level: 2,
                habitat_kind: kind.clone(),
                production_gc_yr: 0.0,
                share: 0.0,
                intake_gc_yr: 0.0,
                physics_name: format!("parasites and pathogens ({kind})"),
                plain_name: format!("parasites of the {pk}"),
                grade: if complex {
                    "protist".into()
                } else {
                    "microbial".into()
                },
                can_fly: false,
                max_mass_kg: Some(if animals { 1.0 } else { 1.0e-12 }),
            });
            rel(
                &mut relations,
                RelationKind::Resource,
                &id,
                &format!("consumer.2.{kind}"),
            );
        }
    }
    for gd in guilds.iter().filter(|x| x.role != "producer") {
        out_guilds.push(Guild {
            id: gd.id.clone(),
            name: gd.physics_name.clone(),
            capabilities: vec![],
            habitats: wb
                .habitats
                .iter()
                .filter(|h| {
                    h.kind == gd.habitat_kind && h.production_gc_yr.is_some_and(|p| p > 0.0)
                })
                .map(|h| (h.habitat.clone(), 1.0))
                .collect(),
            biomass_kg: None,
            since: None,
        });
    }
    // Lineage ancestry: chemotrophy -> anoxygenic -> oxygenic; complex cells
    // descend from chemotrophs (and take in symbionts).
    for (child, parent) in [
        ("anoxygenic", "chemotrophy"),
        ("oxygenic", "anoxygenic"),
        ("complex_cells", "chemotrophy"),
        ("multicellularity", "complex_cells"),
        ("large_bodies", "multicellularity"),
    ] {
        if has(child) {
            rel(
                &mut relations,
                RelationKind::Ancestry,
                &format!("lineage:{child}"),
                &format!("lineage:{parent}"),
            );
        }
    }
    relations
        .sort_by(|a, b| (a.from.as_str(), a.to.as_str()).cmp(&(b.from.as_str(), b.to.as_str())));
    out_guilds.sort_by(|a, b| a.id.cmp(&b.id));
    guilds.sort_by(|a, b| a.id.cmp(&b.id));
    // F: diversity by species-area, provinces and climate variety.
    let living: Vec<&budget::HabitatBudget> = wb
        .habitats
        .iter()
        .filter(|h| h.production_gc_yr.is_some_and(|p| p > 0.0))
        .collect();
    let area: f64 = living
        .iter()
        .filter_map(|h| g.habitats.iter().find(|x| x.address == h.habitat))
        .filter_map(|h| h.profile.area_m2)
        .sum();
    let mut bins: Vec<i64> = living
        .iter()
        .filter_map(|h| g.habitats.iter().find(|x| x.address == h.habitat))
        .filter_map(|h| {
            h.profile
                .temperature_k
                .map(|t| m::floor(0.5 * (t.lo + t.hi) / 10.0) as i64)
        })
        .collect();
    bins.sort();
    bins.dedup();
    let recovered = since_catastrophe_myr.map_or(1.0, |t| 1.0 - m::exp(-t / hp.recovery_myr));
    let grade = if animals {
        1.0
    } else if multicell {
        0.3
    } else if complex {
        0.1
    } else {
        0.01
    };
    let lineages = if area > 0.0 {
        0.05 * m::pow(area, 0.25)
            * m::sqrt(living.len() as f64)
            * m::sqrt(bins.len().max(1) as f64)
            * recovered
            * grade
    } else {
        0.0
    };
    Ok(Biosphere {
        resolved: ResolvedBiosphere {
            address: g.body.clone(),
            guilds: out_guilds,
            relations,
            evidence: LifeEvidence::PlausibleReconstruction,
        },
        guilds,
        diversity: Diversity {
            area_m2: area,
            provinces: living.len() as u32,
            climate_bins: bins.len() as u32,
            since_catastrophe_myr,
            lineages,
        },
        traits,
        budget: wb,
        capabilities: caps,
    })
}
