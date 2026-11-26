//! Life on a world of the chart (L3): a world's own physical description, as
//! the surface and climate suites resolve it (its climate by latitude band,
//! its air, water, ice and inner heat), becomes a world stack for the habitat
//! rules, and the life pipeline runs on that exactly as it does on the
//! fixture worlds: habitats, box B, a history coupled to the air box, a
//! biosphere when life holds on, what a scan would see, epochs and hooks.
//!
//! What the world stack holds, and the supplies each habitat gets, are set by
//! `provides.other.life_from_world` (base-life): every supply is scaled from
//! Earth by the physical quantity that drives it (rain for weathered
//! phosphorus, inner heat for vent chemistry), a sketch calibrated on one
//! world and labelled so. Today's physical state is held through the whole
//! history (one epoch from when the magma ocean froze): the faint young star,
//! a changing climate and the world's own catastrophes are not yet inputs,
//! and the result says so in `held`.

use crate::history::{Epoch, LifeHistory, WorldTimeline};
use crate::HabitatGraph;
use serde::{Deserialize, Serialize};
use sse_climate::{Climate, ClimateSettings, Conditions};
use sse_core::math as m;
use sse_core::sheet;
use sse_world::{Extent, Region, Shell, WorldStack};
use std::collections::BTreeMap;

const MYR_S: f64 = 1.0e6 * sheet::YR_JULIAN;

/// Settings (`provides.other.life_from_world`); see the pack's sources.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FromWorldParams {
    /// Edges of the land's latitude bands, south to north, degrees.
    pub band_edges_deg: Vec<f64>,
    /// One name per band.
    pub band_names: Vec<String>,
    pub atmosphere_top_m: f64,
    pub crust_depth_m: f64,
    pub pore_water_fraction: f64,
    /// Depth of the deep-crust habitat, m: each W/m² of heat flowing out
    /// warms the rock there by this over the crust rock's thermal
    /// conductivity (the data sheet's `crust-rock`).
    pub warming_depth_m: f64,
    /// Radiation at the ground: `floor + open × exp(-column / column_scale)`.
    pub dose_open_msv_yr: f64,
    pub dose_column_kg_m2: f64,
    pub dose_floor_msv_yr: f64,
    /// New phosphorus from weathering, mol per m² of land a year, at the
    /// reference rainfall (m of water a year); it scales with rain.
    pub weathering_p_mol_m2_yr: f64,
    pub weathering_precip_m_yr: f64,
    /// Share of the land's weathered phosphorus that reaches the sea.
    pub river_p_share: f64,
    /// Vent hydrogen, mol a year per W of the world's inner heat.
    pub vent_h2_mol_yr_per_w: f64,
    /// O2 mixing ratio at which the sea keeps half its ferrous iron
    /// (graded as 1 / (1 + O2 / this)).
    pub anoxic_below_o2: f64,
    pub anoxic_fe2_mol_yr_per_w: f64,
    /// Heat flow below which there are no vents, W/m². Still a switch: see
    /// fudge L3-vents (Earth's modelled heat flow sits just above it).
    pub vents_above_w_m2: f64,
    /// How long a surface lasts by its tectonic regime, Gyr.
    #[serde(default)]
    pub recycling_gyr: BTreeMap<String, f64>,
    /// The air box's volcanic reducing gases, mol O2-equivalent a year per W.
    pub air_reductant_mol_yr_per_w: f64,
    /// H2 mixing ratio at which the air counts half as reducing (graded as
    /// x^n / (x^n + this^n), n `reducing_hill`).
    pub reducing_above_h2: f64,
    /// How sharply air turns reducing with its hydrogen.
    pub reducing_hill: f64,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
}

impl FromWorldParams {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("life_from_world"))
            .ok_or("no pack provides life_from_world")?;
        let p: FromWorldParams = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_from_world: {e}"))?;
        if p.band_edges_deg.len() < 2 || p.band_names.len() + 1 != p.band_edges_deg.len() {
            return Err("life_from_world: one band name per pair of band edges".into());
        }
        Ok(p)
    }
}

/// A world as the physics suites describe it now: what this module reads.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldReading {
    pub body: String,
    pub name: String,
    pub conditions: Conditions,
    /// The banded climate now (`sse_climate::orbit` of `conditions`).
    pub climate: Climate,
    /// Age now, Myr since the world formed.
    pub age_myr: f64,
    /// When its surface could first hold water (the magma ocean froze), Myr.
    pub habitable_from_myr: f64,
    /// Heat flowing out now, radiogenic plus tidal, W/m².
    pub heat_flow_w_m2: f64,
    pub star_teff_k: f64,
    /// Tectonic regime now (`mobile-lid`, `stagnant-lid`, `ice-shell`, ...).
    pub regime: String,
    /// Surface water, Earth oceans.
    pub oceans: f64,
    /// Ice and water spread over the surface as one layer, m, when the
    /// world's ground is ice (the surface suite's ice ground).
    #[serde(default)]
    pub ice_water_m: Option<f64>,
    /// Any water on or in the world (oceans, ice, a wet makeup).
    pub has_water: bool,
    /// Universe year the world formed (dates the biosphere's guilds).
    pub birth_year: i64,
}

/// One world's life, every stage kept.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldLife {
    pub stack: WorldStack,
    pub graph: HabitatGraph,
    pub history: LifeHistory,
    /// When O2 first held above 1% of the air (coupled runs), Myr.
    pub oxygenated_myr: Option<f64>,
    pub biosphere: Option<crate::biosphere::Biosphere>,
    pub signatures: crate::signatures::Signatures,
    pub view: crate::sift::LifeView,
    /// What was held at today's value or left out, in words.
    pub held: Vec<String>,
}

/// A gas's mixing ratio in the air; none without air (the climate suite
/// still lists a default mix for an airless world).
fn gas(c: &Conditions, g: &str) -> f64 {
    if c.pressure_bar <= 0.0 {
        return 0.0;
    }
    c.gases.get(g).copied().unwrap_or(0.0)
}

/// A band's land: mean temperature, coldest and warmest season, sunlight at
/// the ground, rain (m a year, water only) and land share, cosine-weighted
/// over the climate's own bands inside it.
struct BandClimate {
    t_lo: f64,
    t_hi: f64,
    light: f64,
    rain: f64,
    land: f64,
}

fn band_climate(
    cs: &ClimateSettings,
    r: &WorldReading,
    lo: f64,
    hi: f64,
    sea: bool,
) -> BandClimate {
    let cl = &r.climate;
    let mut lats: Vec<f64> = cl
        .lat_deg
        .iter()
        .copied()
        .filter(|l| *l >= lo && *l <= hi)
        .collect();
    if lats.is_empty() {
        lats.push((lo + hi) / 2.0);
    }
    let to_ground = sse_climate::ebm::surface_sun_fraction(cs, &r.conditions);
    let wet = cl.condensate == "h2o" || cl.condensate == "water";
    let land_share = r.conditions.land_fraction();
    let (mut w, mut t_lo, mut t_hi, mut light, mut rain, mut land) =
        (0.0, f64::INFINITY, f64::NEG_INFINITY, 0.0, 0.0, 0.0);
    for l in &lats {
        let k = m::cos(*l * m::PI / 180.0).max(0.0);
        let (a, b) = if sea {
            cl.sea_range_at(*l)
        } else {
            cl.land_range_at(*l)
        };
        t_lo = t_lo.min(a);
        t_hi = t_hi.max(b);
        light += k * cl.lerp(&cl.annual.absorbed_w_m2, *l).max(0.0) * to_ground;
        if wet {
            rain += k * cl.precip_at(*l);
        }
        land += k * if cl.land.len() == cl.lat_deg.len() {
            cl.lerp(&cl.land, *l)
        } else {
            land_share
        };
        w += k;
    }
    let w = if w > 0.0 { w } else { 1.0 };
    BandClimate {
        t_lo,
        t_hi,
        light: light / w,
        rain: rain / w,
        land: (land / w).clamp(0.0, 1.0),
    }
}

/// Share of the vents' ferrous iron the sea keeps, 0 to 1, from the air's
/// O2 mixing ratio: half at `anoxic_below_o2`.
pub fn anoxic_share(o2: f64, p: &FromWorldParams) -> f64 {
    if p.anoxic_below_o2 <= 0.0 {
        return 0.0;
    }
    1.0 / (1.0 + o2.max(0.0) / p.anoxic_below_o2)
}

/// How reducing the air is, 0 to 1, from its H2 mixing ratio: half at
/// `reducing_above_h2`.
pub fn reducing_share(h2: f64, p: &FromWorldParams) -> f64 {
    if h2 <= 0.0 || p.reducing_above_h2 <= 0.0 {
        return if h2 > 0.0 { 1.0 } else { 0.0 };
    }
    let x = m::pow(h2 / p.reducing_above_h2, p.reducing_hill.max(0.0));
    x / (1.0 + x)
}

fn band_area(radius_m: f64, lo: f64, hi: f64) -> f64 {
    let rad = m::PI / 180.0;
    2.0 * m::PI * radius_m * radius_m * (m::sin(hi * rad) - m::sin(lo * rad))
}

fn props(v: &[(&str, f64)]) -> BTreeMap<String, f64> {
    v.iter()
        .filter(|(_, x)| x.is_finite())
        .map(|(k, x)| (k.to_string(), *x))
        .collect()
}

/// The world stack the habitat rules read, and what it leaves out.
pub fn stack(
    r: &WorldReading,
    p: &FromWorldParams,
    cs: &ClimateSettings,
) -> (WorldStack, Vec<String>) {
    let c = &r.conditions;
    let mut held: Vec<String> = vec![];
    let rad = c.radius_m.max(1.0);
    let globe = 4.0 * m::PI * rad * rad;
    let g = c.gravity_m_s2;
    let p_pa = c.pressure_bar * 1.0e5;
    let t_s = c.t_surface_k;
    let column = if g > 0.0 { p_pa / g } else { 0.0 };
    let dose = p.dose_floor_msv_yr + p.dose_open_msv_yr * m::exp(-column / p.dose_column_kg_m2);
    let persistence = (r.age_myr - r.habitable_from_myr).max(0.0) * MYR_S;
    let peak_nm = if r.star_teff_k > 0.0 {
        sheet::B_WIEN / r.star_teff_k * 1.0e9
    } else {
        f64::NAN
    };
    let q = r.heat_flow_w_m2.max(0.0);
    let heat_w = q * globe;
    let vents = q >= p.vents_above_w_m2;
    let anoxic = anoxic_share(gas(c, "o2"), p);
    let vent_supply = |extra: &mut Vec<(&'static str, f64)>| {
        if vents {
            let w = heat_w;
            extra.push(("material.h2_mol_yr", p.vent_h2_mol_yr_per_w * w));
            if anoxic > 0.0 {
                extra.push((
                    "material.fe2_mol_yr",
                    p.anoxic_fe2_mol_yr_per_w * w * anoxic,
                ));
            }
        }
    };
    let mut shells: Vec<Shell> = vec![];
    let mut regions: Vec<Region> = vec![];
    if p_pa > 0.0 {
        shells.push(Shell {
            key: "atmosphere".into(),
            kind: "atmosphere".into(),
            name: "Atmosphere".into(),
            top_m: p.atmosphere_top_m,
            bottom_m: 0.0,
            within: None,
            properties: props(&[("pressure_pa", p_pa), ("temperature_k", t_s)]),
            provides: vec![],
        });
    }
    let mantle = |top: f64| Shell {
        key: "mantle".into(),
        kind: "mantle".into(),
        name: "Mantle".into(),
        top_m: top,
        bottom_m: -rad / 2.0,
        within: None,
        properties: BTreeMap::new(),
        provides: if vents { vec!["heat".into()] } else { vec![] },
    };
    let t_melt = sheet::T0C;
    let ice_world = r.ice_water_m.filter(|_| c.solid_surface && t_s < t_melt);
    if !c.solid_surface {
        held.push("no ground: a giant's cloud decks are not yet resolved into habitats".into());
    } else if let Some(layer) = ice_world {
        // Conductive lid: d = k ln(T_melt / T_surface) / q with k = 651 / T.
        let k_ice = sheet::conductivity_solid::H2O[0];
        let lid = if q > 0.0 {
            k_ice * m::ln(t_melt / t_s.max(1.0)) / q
        } else {
            f64::INFINITY
        };
        let ice_bottom = -lid.min(layer);
        shells.push(Shell {
            key: "ice".into(),
            kind: "ice_shell".into(),
            name: "Ice shell".into(),
            top_m: 0.0,
            bottom_m: ice_bottom,
            within: None,
            properties: props(&[("top_temperature_k", t_s), ("radiation_dose_msv_yr", dose)]),
            provides: vec![],
        });
        if lid < layer {
            let p_top = p_pa + sheet::density_solid::H2O * g * lid;
            let p_bottom = p_top + sheet::density_liquid::H2O * g * (layer - lid);
            let mut extra = vec![
                ("temperature_k", t_melt),
                ("pressure_min_pa", p_top),
                ("pressure_max_pa", p_bottom),
                ("radiation_dose_msv_yr", 0.0),
                ("persistence_s", persistence),
            ];
            vent_supply(&mut extra);
            shells.push(Shell {
                key: "ocean".into(),
                kind: "sub_ice_ocean".into(),
                name: "Ocean under the ice".into(),
                top_m: -lid,
                bottom_m: -layer,
                within: None,
                properties: props(&extra),
                provides: vec![],
            });
            held.push(format!(
                "the ocean under the ice is the water below a conductive lid about {:.0} km thick (its heat flow of {:.3} W/m²); high-pressure ice below it is not modelled",
                lid / 1000.0,
                q
            ));
        }
        shells.push(mantle(-layer));
    } else {
        let wet = c.liquid.as_deref() == Some("water") && c.ocean_fraction > 0.0;
        let edges = &p.band_edges_deg;
        let mut land_p = 0.0;
        let mut dry: Vec<&str> = vec![];
        for (i, name) in p.band_names.iter().enumerate() {
            let (lo, hi) = (edges[i], edges[i + 1]);
            let b = band_climate(cs, r, lo, hi, false);
            if b.land <= 0.0 {
                continue;
            }
            let area = band_area(rad, lo, hi) * b.land;
            let pn = if p.weathering_precip_m_yr > 0.0 {
                p.weathering_p_mol_m2_yr * area * b.rain / p.weathering_precip_m_yr
            } else {
                0.0
            };
            land_p += pn;
            if b.rain <= 0.0 {
                dry.push(name);
            }
            regions.push(Region {
                key: format!("band-{i}"),
                kind: "climate_band".into(),
                name: name.clone(),
                on: "crust".into(),
                extent: Extent::Band {
                    min_lat_deg: lo,
                    max_lat_deg: hi,
                },
                top_m: None,
                bottom_m: None,
                properties: props(&[
                    ("temperature_min_k", b.t_lo),
                    ("temperature_max_k", b.t_hi),
                    ("cover_fraction", b.land),
                    ("pressure_pa", p_pa),
                    ("radiation_dose_msv_yr", dose),
                    ("persistence_s", persistence),
                    ("light_w_m2", b.light),
                    ("light_peak_nm", peak_nm),
                    ("material.p_new_mol_yr", pn),
                    // No rain reaches it: its ground holds no liquid water.
                    ("water_activity", if b.rain > 0.0 { f64::NAN } else { 0.0 }),
                ]),
                provides: vec![],
            });
        }
        if p_pa > 0.0 && !dry.is_empty() {
            let lands = if dry.len() == regions.len() {
                "its land".to_string()
            } else {
                dry.join(", ").to_lowercase()
            };
            held.push(format!(
                "no rain falls on {lands}: dry ground holds no liquid water for life"
            ));
        }
        if wet {
            let depth = r.oceans * sheet::V_OCEAN_EARTH / (c.ocean_fraction * globe);
            let s = band_climate(cs, r, -90.0, 90.0, true);
            let mut extra = vec![
                ("temperature_min_k", s.t_lo.max(t_melt)),
                ("temperature_max_k", s.t_hi.max(t_melt)),
                ("pressure_min_pa", p_pa),
                (
                    "pressure_max_pa",
                    p_pa + sheet::density_liquid::H2O * g * depth,
                ),
                ("radiation_dose_msv_yr", dose),
                ("persistence_s", persistence),
                ("light_w_m2", s.light),
                ("light_peak_nm", peak_nm),
                ("material.p_new_mol_yr", p.river_p_share * land_p),
            ];
            vent_supply(&mut extra);
            regions.push(Region {
                key: "ocean-basins".into(),
                kind: "ocean_basin".into(),
                name: "Ocean basins".into(),
                on: "crust".into(),
                extent: Extent::Unmapped,
                top_m: None,
                bottom_m: None,
                properties: props(&[("area_fraction", c.ocean_fraction)]),
                provides: vec![],
            });
            shells.push(Shell {
                key: "ocean".into(),
                kind: "ocean".into(),
                name: "World ocean".into(),
                top_m: 0.0,
                bottom_m: -depth.max(1.0),
                within: Some("ocean-basins".into()),
                properties: props(&extra),
                provides: vec![],
            });
        }
        let mut crust = vec![
            ("temperature_min_k", t_s),
            (
                "temperature_max_k",
                t_s + q * p.warming_depth_m / sheet::mixture_thermal_conductivity::CRUST_ROCK,
            ),
            ("pressure_pa", p_pa),
            ("radiation_dose_msv_yr", 0.0),
            ("persistence_s", persistence),
        ];
        if r.has_water {
            crust.push(("pore_water_fraction", p.pore_water_fraction));
        }
        shells.push(Shell {
            key: "crust".into(),
            kind: "crust".into(),
            name: "Crust".into(),
            top_m: 0.0,
            bottom_m: -p.crust_depth_m,
            within: None,
            properties: props(&crust),
            provides: vec![],
        });
        shells.push(mantle(-p.crust_depth_m));
    }
    held.push("today's climate, air and heat are held through the whole history: the young star's faintness and the world's own catastrophes are not yet inputs".into());
    (
        WorldStack {
            format: sse_world::FORMAT.into(),
            body: r.body.clone(),
            name: r.name.clone(),
            radius_m: rad,
            shells,
            regions,
            features: vec![],
        },
        held,
    )
}

/// What an observer could measure, from the world's facts and its history.
fn observables(
    r: &WorldReading,
    p: &FromWorldParams,
    h: &LifeHistory,
    lid_km: Option<f64>,
) -> crate::signatures::Observables {
    let c = &r.conditions;
    let alive = h.points.last().is_some_and(|x| x.alive);
    let cover = {
        let cl = &r.climate;
        let (mut s, mut w) = (0.0, 0.0);
        for (i, l) in cl.lat_deg.iter().enumerate() {
            let k = m::cos(*l * m::PI / 180.0).max(0.0);
            s += k * cl.cover.get(i).copied().unwrap_or(0.0);
            w += k;
        }
        if w > 0.0 {
            s / w
        } else {
            0.0
        }
    };
    crate::signatures::Observables {
        star_teff_k: r.star_teff_k,
        atmosphere_bar: (c.pressure_bar > 0.0).then_some(c.pressure_bar),
        // Oxygen and methane in the air count as made by rock and light
        // unless life is there to have made them.
        abiotic_o2: if h.has("oxygenic") { 0.0 } else { gas(c, "o2") },
        abiotic_ch4_ppm: if alive { 0.0 } else { gas(c, "ch4") * 1.0e6 },
        ice_shell_km: lid_km,
        cloud_cover: cover,
        land_fraction: c.land_fraction(),
        obliquity_deg: c.obliquity_deg,
        recycling_gyr: p.recycling_gyr.get(&r.regime).copied(),
        erosion_gyr: None,
        age_myr: r.age_myr,
        reducing_air: reducing_share(gas(c, "h2"), p),
    }
}

/// The whole pipeline on one world of the chart.
pub fn life(
    r: &WorldReading,
    stack_packs: &[sse_pack::PackManifest],
    cs: &ClimateSettings,
    seed_text: &str,
) -> Result<WorldLife, String> {
    let p = FromWorldParams::from_packs(stack_packs)?;
    let rules = crate::habitat::rules(stack_packs).map_err(|e| e.join("; "))?;
    let bp = crate::budget::BudgetParams::from_packs(stack_packs)?;
    let hp = crate::history::HistoryParams::from_packs(stack_packs)?;
    let other = |k: &str| {
        stack_packs
            .iter()
            .rev()
            .find_map(|x| x.provides.other.get(k))
            .cloned()
    };
    let sp: crate::signatures::SignatureParams = match other("life_signatures") {
        Some(v) => serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_signatures: {e}"))?,
        None => Default::default(),
    };
    let sift: crate::sift::SiftParams = match other("life_sift") {
        Some(v) => serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_sift: {e}"))?,
        None => Default::default(),
    };
    let mut air: crate::feedback::AirParams = match other("life_feedback") {
        Some(v) => serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_feedback: {e}"))?,
        None => Default::default(),
    };
    let (ws, mut held) = stack(r, &p, cs);
    let lid_km = ws
        .shells
        .iter()
        .find(|s| s.kind == "ice_shell")
        .map(|s| -s.bottom_m / 1000.0);
    let graph = HabitatGraph::derive(&ws, &rules).map_err(|e| format!("{e:?}"))?;
    let c = &r.conditions;
    let o2 = |v: f64| [("o2_fraction".to_string(), v)].into_iter().collect();
    let mut epochs = vec![];
    if r.habitable_from_myr > 0.0 {
        epochs.push(Epoch {
            id: format!("{}.molten", r.name),
            from_myr: 0.0,
            graph: None,
            facts: o2(0.0),
        });
    }
    epochs.push(Epoch {
        id: format!("{}.today-held", r.name),
        from_myr: r.habitable_from_myr.max(0.0),
        graph: Some(graph.clone()),
        facts: o2(0.0),
    });
    let world = WorldTimeline {
        body: graph.body.clone(),
        end_myr: r.age_myr.max(r.habitable_from_myr),
        epochs,
        catastrophes: vec![],
    };
    let globe = 4.0 * m::PI * c.radius_m * c.radius_m;
    let (history, oxygenated_myr) = if c.pressure_bar > 0.0 && c.mean_molar_kg > 0.0 {
        // The air box sized to this world: its air, its area, its volcanism.
        air.air_mol = c.column_kg_m2() * globe / c.mean_molar_kg;
        air.area_rel = globe / hp.earth_area_m2;
        air.reductant_mol_yr = p.air_reductant_mol_yr_per_w * r.heat_flow_w_m2.max(0.0) * globe;
        air.reductant_ref_myr = r.age_myr;
        let man = crate::feedback::CoupledManifest {
            format: crate::feedback::MANIFEST_FORMAT.into(),
            world,
            budget: bp.clone(),
            history: hp.clone(),
            air,
            seed: seed_text.into(),
        };
        let run = crate::feedback::coupled(&man)?;
        (run.history, run.oxygenated_myr)
    } else {
        held.push("no air: oxygen cannot build up, so the history runs without the air box".into());
        let h = crate::history::history(&world, &bp, &hp, sse_core::Seed::from_text(seed_text))?;
        (h, None)
    };
    let last = history.points.last();
    let biosphere = match last.filter(|x| x.alive) {
        Some(pt) => {
            let rho = (c.pressure_bar > 0.0 && c.t_surface_k > 0.0)
                .then(|| c.pressure_bar * 1.0e5 * c.mean_molar_kg / (sheet::R_GAS * c.t_surface_k));
            let day = c.solar_day_hours();
            let physics = crate::biosphere::WorldPhysics {
                surface_gravity_m_s2: c.gravity_m_s2,
                air_density_kg_m3: rho,
                star_teff_k: r.star_teff_k,
                day_hours: if day.is_finite() {
                    day
                } else {
                    c.rotation_hours.abs()
                },
                obliquity_deg: c.obliquity_deg,
                tidally_locked: c.star_locked,
            };
            Some(crate::biosphere::resolve(
                &graph,
                &bp,
                &hp,
                &pt.capabilities,
                &history.first,
                None,
                &physics,
                sse_core::Seed::from_text(&format!("{seed_text}/biosphere")),
                r.birth_year,
            )?)
        }
        None => None,
    };
    let obs = observables(r, &p, &history, lid_km);
    let signatures = crate::signatures::signatures(biosphere.as_ref(), Some(&history), &obs, &sp);
    let inp = crate::sift::HookInputs {
        habitable_until_myr: None,
        o2_fraction: signatures.o2,
        now_myr: r.age_myr,
    };
    let view = crate::sift::view(
        &r.body,
        biosphere.as_ref(),
        Some(&history),
        &signatures,
        &inp,
        &sift,
    );
    Ok(WorldLife {
        stack: ws,
        graph,
        history,
        oxygenated_myr,
        biosphere,
        signatures,
        view,
        held,
    })
}
