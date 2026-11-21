//! Box B: energy and material budget (life design section 3, stage L1a).
//!
//! Life cannot use more free energy or more atoms than a habitat supplies.
//! Per habitat:
//! - **closed** (cap zero, with the reason) when the solvent has no chemistry
//!   in the pack, the temperature range misses the chemistry's window, water
//!   cannot be liquid (below its triple-point pressure), water activity is
//!   below the limit, or the radiation dose is above it;
//! - **energy cap**: for each metabolism the world has (an input: which
//!   metabolisms exist is box D's business, L1b), carbon fixed per year from
//!   light (flux x the share of a blackbody at the light's peak inside the
//!   metabolism's band x an efficiency) or from chemical supply (electron
//!   donors and oxidants in mol per year x carbon per mole). Light
//!   metabolisms compete for the same light (the best one counts); supply
//!   metabolisms add;
//! - **nutrient cap**: new phosphorus per year x how often an atom is reused
//!   before burial x the carbon-to-phosphorus ratio (Liebig's law: the smaller
//!   cap wins);
//! - standing biomass = production x a residence time by habitat kind;
//! - feeding levels = log(production density / minimum viable) / log(1 /
//!   transfer efficiency), at most `low_power_levels` without a high-power
//!   (oxygen-making) metabolism somewhere on the world.
//!
//! A value nobody supplied stays `None` and is listed in `missing`: a gap is
//! never a zero. Every closed habitat says why.

use crate::habitat::HabitatGraph;
use crate::*;
use sse_core::math as m;

/// One way of making a living (`provides.other.life_budget.metabolisms`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Metabolism {
    pub id: String,
    pub label: String,
    /// `light` or `supply`.
    pub channel: String,
    /// Light: usable band (nm) and the share of usable light energy stored as fixed carbon.
    #[serde(default)]
    pub band_nm: Option<(f64, f64)>,
    #[serde(default)]
    pub light_efficiency: f64,
    /// Supply: the electron donor and (optionally) the oxidant it needs, as
    /// habitat materials in mol per year, and moles of carbon fixed per mole.
    #[serde(default)]
    pub donor: Option<String>,
    #[serde(default)]
    pub c_per_donor: f64,
    #[serde(default)]
    pub oxidant: Option<String>,
    #[serde(default)]
    pub c_per_oxidant: f64,
    /// Light metabolisms that also need a donor (anoxygenic photosynthesis).
    #[serde(default)]
    pub light_donor: Option<String>,
    /// Habitat energy channels it can use (any one is enough).
    pub channels: Vec<String>,
    /// Habitat media it works in.
    pub media: Vec<String>,
    /// Makes oxygen (or a pack's equivalent), allowing high-power bodies.
    #[serde(default)]
    pub high_power: bool,
}

/// Box B settings (`provides.other.life_budget`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BudgetParams {
    /// The stack's chemistries (E0): each names its solvent, window and
    /// limits. Box B's closure tests read the one a habitat's solvent names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chemistry: Vec<crate::chemistry::Chemistry>,
    /// Pre-E0 packs and manifests: solvents with a chemistry, and the
    /// water-carbon window and limits. Read only when `chemistry` is empty
    /// (`chemistry::chemistry_for`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chemistries: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_k: Option<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water_triple_pa: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_water_activity: Option<f64>,
    pub max_dose_msv_yr: f64,
    /// Free energy stored per gram of carbon fixed as carbohydrate, J.
    pub j_per_g_carbon: f64,
    /// New phosphorus supply (a habitat material, mol per year), its reuse
    /// factor by habitat kind, and C:P by medium.
    pub p_material: String,
    pub reuse: BTreeMap<String, f64>,
    pub default_reuse: f64,
    pub c_per_p: BTreeMap<String, f64>,
    pub default_c_per_p: f64,
    /// Biomass residence time by habitat kind, years.
    pub residence_yr: BTreeMap<String, f64>,
    pub default_residence_yr: f64,
    pub transfer_efficiency: f64,
    /// Production density that supports one level, g C per m^2 per year.
    pub min_viable_g_m2_yr: f64,
    pub low_power_levels: u32,
    pub metabolisms: Vec<Metabolism>,
    /// `listed/1` (the metabolisms above) or `gradients/1` (the gradient
    /// catalogue, E1). The listed model stays the default until the
    /// fidelity contract (evolution design 3.5) is reviewed.
    #[serde(default = "listed_model", skip_serializing_if = "is_listed")]
    pub life_model: String,
}

fn listed_model() -> String {
    "listed/1".into()
}
fn is_listed(s: &String) -> bool {
    s == "listed/1"
}

impl BudgetParams {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("life_budget"))
            .ok_or("no pack provides life_budget")?;
        let bp: BudgetParams = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_budget: {e}"))?;
        for c in &bp.chemistry {
            c.check()?;
        }
        Ok(bp)
    }
}

/// One habitat's budget.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HabitatBudget {
    pub habitat: Address,
    pub kind: String,
    /// Why the cap is zero, if it is.
    pub closed: Option<String>,
    /// Carbon fixed per year, g: the energy cap, the nutrient cap and the result.
    pub energy_cap_gc_yr: Option<f64>,
    pub nutrient_cap_gc_yr: Option<f64>,
    pub production_gc_yr: Option<f64>,
    /// `energy`, `nutrients`, `closed` or `unknown`.
    pub limited_by: String,
    /// Energy cap by metabolism, g C per year.
    pub by_metabolism: BTreeMap<String, f64>,
    /// Free energy stored by production, W.
    pub power_w: Option<f64>,
    pub biomass_gc: Option<f64>,
    pub feeding_levels: Option<u32>,
    pub missing: Vec<String>,
    /// Heterotrophs' production on imported organic carbon, g C per year
    /// (`gradients/1` only; not new carbon, so not in `production_gc_yr`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_gc_yr: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldBudget {
    pub body: Address,
    pub habitats: Vec<HabitatBudget>,
    /// Sums over habitats with a known production (unknown ones are listed).
    pub production_gc_yr: f64,
    pub biomass_gc: f64,
    pub power_w: f64,
    pub unknown: Vec<Address>,
    pub high_power: bool,
    pub feeding_levels: u32,
    pub biomass_class: String,
}

const H: f64 = 6.626_070_15e-34;
const C: f64 = 299_792_458.0;
const K_B: f64 = 1.380_649e-23;
const SIGMA: f64 = 5.670_374_419e-8;
const WIEN_NM_K: f64 = 2.897_771_955e6;
const YEAR_S: f64 = 3.155_76e7;

/// Share of a blackbody's energy between two wavelengths (nm), for the
/// temperature whose Wien peak is `peak_nm`. Simpson's rule, 400 intervals.
pub fn band_fraction(peak_nm: f64, lo_nm: f64, hi_nm: f64) -> f64 {
    let t = WIEN_NM_K / peak_nm;
    let b = |nm: f64| {
        let l = nm * 1.0e-9;
        2.0 * H * C * C / (l * l * l * l * l) / (m::exp(H * C / (l * K_B * t)) - 1.0)
    };
    let n = 400;
    let h = (hi_nm - lo_nm) / f64::from(n);
    let mut s = b(lo_nm) + b(hi_nm);
    for i in 1..n {
        let w = if i % 2 == 1 { 4.0 } else { 2.0 };
        s += w * b(lo_nm + h * f64::from(i));
    }
    let integral = s * h * 1.0e-9 / 3.0;
    (integral * m::PI / (SIGMA * t * t * t * t)).clamp(0.0, 1.0)
}

pub(crate) fn closed_reason(p: &LifeRequirementProfile, bp: &BudgetParams) -> Option<String> {
    let Some(c) = crate::chemistry::chemistry_for(bp, &p.solvent) else {
        return Some(format!("no chemistry for {} in this pack stack", p.solvent));
    };
    if let Some(t) = p.temperature_k {
        if t.hi < c.window_k.0 || t.lo > c.window_k.1 {
            return Some(format!(
                "{:.0} to {:.0} K is outside the {:.0} to {:.0} K window",
                t.lo, t.hi, c.window_k.0, c.window_k.1
            ));
        }
    }
    if let (Some(min_pa), Some(pr)) = (c.liquid_min_pa, p.pressure_pa) {
        if pr.hi < min_pa {
            return Some(format!(
                "at {:.0} Pa {} cannot be liquid (triple point {:.0} Pa)",
                pr.hi, c.solvent, min_pa
            ));
        }
    }
    if let (Some(prop), Some(lim)) = (&c.activity_property, c.min_activity) {
        if let Some(aw) = p.properties.get(prop) {
            if *aw < lim {
                return Some(format!(
                    "{} activity {aw} is below the {lim} limit",
                    c.solvent
                ));
            }
        }
    }
    let max_dose = c.max_dose_msv_yr.unwrap_or(bp.max_dose_msv_yr);
    if let Some(d) = p.radiation_dose_msv_yr {
        if d > max_dose {
            return Some(format!(
                "a dose of {d:.0} mSv/yr is above the {max_dose:.0} limit"
            ));
        }
    }
    None
}

/// The budget of one habitat for the given metabolisms.
pub fn habitat_budget(
    p: &LifeRequirementProfile,
    bp: &BudgetParams,
    have: &[&Metabolism],
) -> HabitatBudget {
    let mut out = HabitatBudget {
        habitat: p.habitat.clone(),
        kind: p.kind.clone(),
        closed: None,
        energy_cap_gc_yr: None,
        nutrient_cap_gc_yr: None,
        production_gc_yr: None,
        limited_by: "unknown".into(),
        by_metabolism: BTreeMap::new(),
        power_w: None,
        biomass_gc: None,
        feeding_levels: None,
        missing: vec![],
        secondary_gc_yr: None,
    };
    if let Some(r) = closed_reason(p, bp) {
        out.closed = Some(r);
        out.energy_cap_gc_yr = Some(0.0);
        out.production_gc_yr = Some(0.0);
        out.power_w = Some(0.0);
        out.biomass_gc = Some(0.0);
        out.feeding_levels = Some(0);
        out.limited_by = "closed".into();
        return out;
    }
    let usable: Vec<&&Metabolism> = have
        .iter()
        .filter(|mb| mb.media.contains(&p.medium))
        .filter(|mb| mb.channels.iter().any(|c| p.energy.available.contains(c)))
        .collect();
    if usable.is_empty() {
        out.closed = Some("no metabolism here can use the energy this habitat has".into());
        out.energy_cap_gc_yr = Some(0.0);
        out.production_gc_yr = Some(0.0);
        out.power_w = Some(0.0);
        out.biomass_gc = Some(0.0);
        out.feeding_levels = Some(0);
        out.limited_by = "closed".into();
        return out;
    }
    let mut best_light: Option<f64> = None;
    let mut supply = 0.0;
    let mut any_known = false;
    let mut missing = vec![];
    for mb in usable {
        let g = if mb.channel == "light" {
            match (p.energy.light_w_m2, p.area_m2) {
                (Some(flux), Some(area)) => {
                    let peak = p.energy.light_peak_nm.unwrap_or_else(|| {
                        missing.push("light_peak_nm".to_string());
                        500.0
                    });
                    let (lo, hi) = mb.band_nm.unwrap_or((400.0, 700.0));
                    let w = flux * band_fraction(peak, lo, hi) * mb.light_efficiency * area;
                    let mut g = w * YEAR_S / bp.j_per_g_carbon;
                    if let Some(d) = &mb.light_donor {
                        match p.materials.get(d) {
                            Some(s) => g = g.min(s * mb.c_per_donor * 12.0),
                            None => {
                                missing.push(format!("material.{d}"));
                                continue;
                            }
                        }
                    }
                    Some(g)
                }
                _ => {
                    if p.energy.light_w_m2.is_none() {
                        missing.push("light_w_m2".into());
                    }
                    if p.area_m2.is_none() {
                        missing.push("area_m2".into());
                    }
                    None
                }
            }
        } else {
            let donor = mb
                .donor
                .as_ref()
                .and_then(|d| p.materials.get(d).map(|s| s * mb.c_per_donor));
            let oxidant = mb
                .oxidant
                .as_ref()
                .map(|o| p.materials.get(o).map(|s| s * mb.c_per_oxidant));
            match (donor, oxidant) {
                (Some(dc), None) => Some(dc * 12.0),
                (Some(dc), Some(Some(oc))) => Some(dc.min(oc) * 12.0),
                _ => {
                    if let Some(d) = &mb.donor {
                        if !p.materials.contains_key(d) {
                            missing.push(format!("material.{d}"));
                        }
                    }
                    if let Some(o) = &mb.oxidant {
                        if !p.materials.contains_key(o) {
                            missing.push(format!("material.{o}"));
                        }
                    }
                    None
                }
            }
        };
        if let Some(g) = g {
            any_known = true;
            out.by_metabolism.insert(mb.id.clone(), g);
            if mb.channel == "light" {
                best_light = Some(best_light.map_or(g, |b: f64| b.max(g)));
            } else {
                supply += g;
            }
        }
    }
    missing.sort();
    missing.dedup();
    if !any_known {
        out.missing = missing;
        return out;
    }
    let energy = best_light.unwrap_or(0.0) + supply;
    finish(p, bp, out, energy, missing)
}

/// The rest of box B once a habitat's energy cap is known (g C per year):
/// the nutrient cap (Liebig), production, power, biomass and feeding levels.
/// Shared by the listed model and the gradient catalogue (E1).
pub(crate) fn finish(
    p: &LifeRequirementProfile,
    bp: &BudgetParams,
    mut out: HabitatBudget,
    energy: f64,
    mut missing: Vec<String>,
) -> HabitatBudget {
    out.energy_cap_gc_yr = Some(energy);
    let nutrient = p.materials.get(&bp.p_material).map(|pn| {
        let reuse = bp.reuse.get(&p.kind).copied().unwrap_or(bp.default_reuse);
        let cp = bp
            .c_per_p
            .get(&p.medium)
            .copied()
            .unwrap_or(bp.default_c_per_p);
        pn * reuse * cp * 12.0
    });
    if nutrient.is_none() {
        missing.push(format!("material.{}", bp.p_material));
    }
    out.nutrient_cap_gc_yr = nutrient;
    let (prod, by) = match nutrient {
        Some(n) if n < energy => (n, "nutrients"),
        _ => (energy, "energy"),
    };
    out.production_gc_yr = Some(prod);
    out.limited_by = by.into();
    out.power_w = Some(prod * bp.j_per_g_carbon / YEAR_S);
    let res = bp
        .residence_yr
        .get(&p.kind)
        .copied()
        .unwrap_or(bp.default_residence_yr);
    out.biomass_gc = Some(prod * res);
    let area = p
        .area_m2
        .or_else(|| p.volume_m3.map(|v| m::pow(v, 2.0 / 3.0)));
    out.feeding_levels = area.map(|a| {
        let dens = prod / a;
        if dens <= bp.min_viable_g_m2_yr {
            0
        } else {
            m::floor(m::ln(dens / bp.min_viable_g_m2_yr) / m::ln(1.0 / bp.transfer_efficiency))
                as u32
                + 1
        }
    });
    if area.is_none() {
        missing.push("area_m2".into());
    }
    missing.sort();
    missing.dedup();
    out.missing = missing;
    out
}

/// Biomass class by standing carbon, g: trace below a tonne, sparse below a
/// million tonnes, modest below a billion, rich below 100 Gt, teeming above
/// (Earth's 550 Gt C; Bar-On et al. 2018).
pub fn biomass_class(g: f64) -> &'static str {
    match g {
        x if x <= 0.0 => "none",
        x if x < 1.0e6 => "trace",
        x if x < 1.0e12 => "sparse",
        x if x < 1.0e15 => "modest",
        x if x < 1.0e17 => "rich",
        _ => "teeming",
    }
}

/// Box B for a whole world: every habitat, for the metabolisms in `have` (ids).
pub fn world_budget(g: &HabitatGraph, bp: &BudgetParams, have: &[String]) -> WorldBudget {
    let mets: Vec<&Metabolism> = bp
        .metabolisms
        .iter()
        .filter(|mb| have.contains(&mb.id))
        .collect();
    let mut habitats: Vec<HabitatBudget> = g
        .habitats
        .iter()
        .map(|h| habitat_budget(&h.profile, bp, &mets))
        .collect();
    let high_power = habitats.iter().any(|h| {
        h.by_metabolism
            .iter()
            .any(|(id, v)| *v > 0.0 && mets.iter().any(|mb| &mb.id == id && mb.high_power))
            && h.production_gc_yr.is_some_and(|x| x > 0.0)
    });
    if !high_power {
        for h in &mut habitats {
            if let Some(l) = h.feeding_levels.as_mut() {
                *l = (*l).min(bp.low_power_levels);
            }
        }
    }
    let known = |f: fn(&HabitatBudget) -> Option<f64>| habitats.iter().filter_map(f).sum::<f64>();
    let production = known(|h| h.production_gc_yr);
    let biomass = known(|h| h.biomass_gc);
    let power = known(|h| h.power_w);
    WorldBudget {
        body: g.body.clone(),
        unknown: habitats
            .iter()
            .filter(|h| h.production_gc_yr.is_none())
            .map(|h| h.habitat.clone())
            .collect(),
        feeding_levels: habitats
            .iter()
            .filter_map(|h| h.feeding_levels)
            .max()
            .unwrap_or(0),
        biomass_class: biomass_class(biomass).into(),
        habitats,
        production_gc_yr: production,
        biomass_gc: biomass,
        power_w: power,
        high_power,
    }
}

/// The box B part of a world's life envelope: power and biomass class, with
/// the evidence label and inputs hash. Origin and presence are box C (L1b).
pub fn envelope(b: &WorldBudget, epoch: UniverseTime) -> LifeEnvelope {
    let mut complexity = BTreeMap::new();
    complexity.insert("feeding_levels".to_string(), f64::from(b.feeding_levels));
    complexity.insert(
        "high_power".to_string(),
        if b.high_power { 1.0 } else { 0.0 },
    );
    LifeEnvelope {
        address: b.body.clone(),
        epoch,
        p_present: None,
        origin_time: None,
        energy_budget_w: Some(b.power_w),
        biomass_class: Some(b.biomass_class.clone()),
        capabilities: BTreeMap::new(),
        complexity,
        fluxes: vec![],
        signatures: BTreeMap::new(),
        uncertainty: if b.unknown.is_empty() {
            "caps from box B only: whether life is present is box C's (L1b)".into()
        } else {
            format!(
                "caps from box B only; {} habitats lack inputs",
                b.unknown.len()
            )
        },
        evidence: LifeEvidence::PlausibleReconstruction,
        inputs: ContentHash::of(b).ok(),
    }
}
