//! The gradient catalogue (evolution design section 3, stage E1).
//!
//! Instead of a list of metabolisms, each habitat gets a catalogue of the
//! free energy it offers:
//! - **light**: photons in a band, with an electron donor where the
//!   photosynthesis needs one (water for oxygenic, Fe2+ or H2 for the
//!   anoxygenic kinds);
//! - **redox**: every pack reaction whose reactants the habitat supplies or
//!   holds, with ΔG at the habitat's own temperature and activities (the
//!   `sse.chem` blocks through the thermodynamic table);
//! - **organic**: organic carbon arriving from producers in linked habitats
//!   (the heterotrophs: soils, the ocean interior, the deep sea floor).
//!
//! Each entry carries the supply of its limiting reactant, the power, whether
//! the uncatalysed reaction is already fast (the gradient is spent before
//! life can reach it), and the three tests: energy per turnover, power
//! against the maintenance cost of a minimal population, and materials.
//! Supplies are spent once: the most energetic use of a species takes it
//! first, and a supply shared by every habitat of one shell is split among
//! them rather than counted in each.

use crate::budget::{finish, BudgetParams, HabitatBudget, WorldBudget};
use crate::chemistry::{chemistry_for, ThermoTable, HYDROGEN_ION};
use crate::habitat::HabitatGraph;
use crate::*;
use sse_core::math as m;
use std::collections::BTreeSet;

pub const MODEL: &str = "gradients/1";
const YEAR_S: f64 = sse_core::sheet::YR_JULIAN;

/// A light gradient (`life_gradients.light`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LightGradient {
    pub id: String,
    pub band_nm: (f64, f64),
    /// Share of usable light stored as fixed carbon.
    pub efficiency: f64,
    /// Electron donor (thermodynamic species) and its habitat material, if
    /// the photosynthesis needs one other than the solvent.
    #[serde(default)]
    pub donor: Option<String>,
    /// Carbon fixed per mole of donor.
    #[serde(default)]
    pub c_per_donor: f64,
    /// Makes free oxygen (allows high-power bodies).
    #[serde(default)]
    pub high_power: bool,
    pub media: Vec<String>,
    /// The listed metabolism it regenerates (fidelity contract, 3.5).
    #[serde(default)]
    pub matches: Option<String>,
    /// New chemical steps a biosphere needs to use it (E2, evolution design 5.1).
    #[serde(default)]
    pub reach: Option<f64>,
    /// The hard-step dial that scales its waiting time (E2).
    #[serde(default)]
    pub dial: Option<String>,
}

/// A redox or organic gradient (`life_gradients.reactions`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ReactionGradient {
    /// A reaction in the thermodynamic table.
    pub reaction: String,
    /// `redox`, or `organic` when organic carbon is the fuel.
    pub kind: String,
    pub media: Vec<String>,
    /// The uncatalysed reaction is fast above this temperature (K) or pH:
    /// the gradient is spent before life can reach it.
    #[serde(default)]
    pub fast_above_k: Option<f64>,
    #[serde(default)]
    pub fast_above_ph: Option<f64>,
    #[serde(default)]
    pub fast_why: Option<String>,
    #[serde(default)]
    pub matches: Option<String>,
    /// New chemical steps a biosphere needs to use it (E2, evolution design 5.1).
    #[serde(default)]
    pub reach: Option<f64>,
    /// The hard-step dial that scales its waiting time (E2).
    #[serde(default)]
    pub dial: Option<String>,
}

/// `provides.other.life_gradients`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GradientParams {
    /// Minimum free energy per turnover, kJ per mol of reaction as written.
    pub min_quantum_kj: f64,
    // Maintenance and the smallest population moved to the shared
    // `life_budget.maintenance` block (LE1): both models read one number.
    /// Share of a catabolic reaction's free energy that ends up in built biomass.
    pub growth_efficiency: f64,
    /// Habitat material key to thermodynamic species, for supplies (mol per year).
    pub supplies: BTreeMap<String, String>,
    /// Activities by medium, then species.
    pub activities: BTreeMap<String, BTreeMap<String, f64>>,
    /// A species standing at or above this activity never limits.
    pub abundant_above: f64,
    /// pH by medium when the habitat does not say (`properties.ph`).
    pub ph: BTreeMap<String, f64>,
    /// Energy channels that mean free oxygen stands in the habitat.
    pub oxic_channels: Vec<String>,
    /// The species that stands in an oxic habitat.
    pub oxygen: String,
    /// The species organic carbon is counted as.
    pub organic: String,
    /// Share of a producer habitat's production exported to a linked
    /// habitat, by producer kind then receiver kind.
    pub export: BTreeMap<String, BTreeMap<String, f64>>,
    pub light: Vec<LightGradient>,
    pub reactions: Vec<ReactionGradient>,
    /// Most entries kept per habitat (evolution design section 13).
    pub cap: usize,
}

impl GradientParams {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("life_gradients"))
            .ok_or("no pack provides life_gradients")?;
        serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_gradients: {e}"))
    }
}

/// The three tests of section 3.2.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tests {
    /// Energy per turnover clears the minimum quantum.
    pub quantum: bool,
    /// The power keeps at least a minimal population alive.
    pub power: bool,
    /// The chemistry's limiting element is supplied (`None`: unknown).
    pub materials: Option<bool>,
}

/// One entry in a habitat's catalogue.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Gradient {
    pub id: String,
    /// `light`, `redox` or `organic`.
    pub kind: String,
    /// Free energy per mol of reaction as written at local conditions, kJ
    /// (light: absent).
    pub dg_kj: Option<f64>,
    pub per_electron_kj: Option<f64>,
    /// The reactant that limits it, its supply (mol per year) and the power.
    pub limiting: Option<String>,
    pub supply_mol_yr: Option<f64>,
    pub power_w: Option<f64>,
    /// `slow` (life can catalyse it) or `spent` (it runs on its own).
    pub abiotic: String,
    pub tests: Tests,
    pub open: bool,
    /// Carbon fixed (or, for organic, built by heterotrophs) per year, g.
    pub production_gc_yr: Option<f64>,
    /// Most biomass the power keeps alive at maintenance, g C.
    pub max_biomass_gc: Option<f64>,
    pub matches: Option<String>,
    /// Why it is closed, or what is unknown, in words.
    pub why: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Catalogue {
    pub habitat: Address,
    pub kind: String,
    pub gradients: Vec<Gradient>,
    /// Entries beyond the cap, dropped (lowest power first).
    pub dropped: u32,
}

/// Starved (basal) maintenance power per gram of biomass carbon at `t_k`,
/// W, from the shared block (LE1): the test "can anything live on this
/// gradient at all" and the most biomass its power can hold.
pub fn maintenance_w_per_gc(bp: &BudgetParams, t_k: f64) -> Option<f64> {
    bp.maintenance.as_ref().map(|mt| mt.starved_w_per_gc(t_k))
}

fn mid_t(p: &LifeRequirementProfile) -> Option<f64> {
    p.temperature_k.map(|t| 0.5 * (t.lo + t.hi))
}

fn ph(p: &LifeRequirementProfile, gp: &GradientParams) -> f64 {
    p.properties
        .get("ph")
        .copied()
        .or_else(|| gp.ph.get(&p.medium).copied())
        .unwrap_or(7.0)
}

fn activities(p: &LifeRequirementProfile, gp: &GradientParams) -> BTreeMap<String, f64> {
    let mut a = gp.activities.get(&p.medium).cloned().unwrap_or_default();
    a.insert(HYDROGEN_ION.into(), m::pow(10.0, -ph(p, gp)));
    for (k, v) in &p.properties {
        if let Some(s) = k.strip_prefix("activity.") {
            a.insert(s.to_string(), *v);
        }
    }
    a
}

/// What each habitat has to spend: supplied species in mol per year, split
/// among the habitats of one shell that report the same supply.
pub fn supplies(g: &HabitatGraph, gp: &GradientParams) -> Vec<BTreeMap<String, f64>> {
    let parent = |a: &Address| a.parent().map(|p| p.to_string()).unwrap_or_default();
    g.habitats
        .iter()
        .map(|h| {
            let mut out = BTreeMap::new();
            for (key, species) in &gp.supplies {
                if let Some(v) = h.profile.materials.get(key) {
                    let sharing = g
                        .habitats
                        .iter()
                        .filter(|o| parent(&o.address) == parent(&h.address))
                        .filter(|o| o.profile.materials.get(key) == Some(v))
                        .count()
                        .max(1);
                    *out.entry(species.clone()).or_insert(0.0) += v / sharing as f64;
                }
            }
            out
        })
        .collect()
}

/// Organic carbon arriving at each habitat from producers, mol C per year.
pub fn organic_imports(
    g: &HabitatGraph,
    gp: &GradientParams,
    production: &[Option<f64>],
) -> Vec<f64> {
    let mut out = vec![0.0; g.habitats.len()];
    let index = |a: &Address| g.habitats.iter().position(|h| &h.address == a);
    for l in &g.links {
        let (Some(i), Some(j)) = (index(&l.a), index(&l.b)) else {
            continue;
        };
        for (from, to) in [(i, j), (j, i)] {
            let f = gp
                .export
                .get(&g.habitats[from].kind)
                .and_then(|x| x.get(&g.habitats[to].kind));
            if let (Some(f), Some(prod)) = (f, production[from]) {
                out[to] += prod * f / 12.0;
            }
        }
    }
    out
}

/// One habitat's catalogue and budget. `supply` is what it has to spend
/// (species, mol per year); `organic_mol_yr` is imported organic carbon.
pub fn habitat_catalogue(
    p: &LifeRequirementProfile,
    bp: &BudgetParams,
    gp: &GradientParams,
    table: &ThermoTable,
    supply: &BTreeMap<String, f64>,
    organic_mol_yr: f64,
) -> (Catalogue, HabitatBudget) {
    habitat_catalogue_using(p, bp, gp, table, supply, organic_mol_yr, None)
}

/// As [`habitat_catalogue`], with only the gradients in `usable` tapped
/// (E2: the metabolisms a biosphere has evolved). The rest stay in the
/// catalogue with their tests, but draw nothing and produce nothing.
pub fn habitat_catalogue_using(
    p: &LifeRequirementProfile,
    bp: &BudgetParams,
    gp: &GradientParams,
    table: &ThermoTable,
    supply: &BTreeMap<String, f64>,
    organic_mol_yr: f64,
    usable: Option<&BTreeSet<String>>,
) -> (Catalogue, HabitatBudget) {
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
        keep_alive_w: None,
        barely: false,
        biomass_limited_by: None,
    };
    let mut cat = Catalogue {
        habitat: p.habitat.clone(),
        kind: p.kind.clone(),
        gradients: vec![],
        dropped: 0,
    };
    if let Some(r) = crate::budget::closed_reason(p, bp) {
        out.closed = Some(r);
        out.energy_cap_gc_yr = Some(0.0);
        out.production_gc_yr = Some(0.0);
        out.power_w = Some(0.0);
        out.biomass_gc = Some(0.0);
        out.feeding_levels = Some(0);
        out.limited_by = "closed".into();
        return (cat, out);
    }
    let chem = chemistry_for(bp, &p.solvent).expect("open habitats have a chemistry");
    let mut missing: Vec<String> = vec![];
    let materials = p.materials.get(&bp.p_material).map(|x| *x > 0.0);
    if materials.is_none() {
        missing.push(format!("material.{}", bp.p_material));
    }
    let t = mid_t(p);
    let maint = t.and_then(|t| maintenance_w_per_gc(bp, t));
    let min_power = maint.and_then(|w| {
        bp.maintenance
            .as_ref()
            .map(|mt| w * mt.min_population_cmol * 12.0)
    });
    let mut gradients: Vec<Gradient> = vec![];
    // Light-limited production of each light gradient, before its donor.
    let mut photon_gc: BTreeMap<String, f64> = BTreeMap::new();

    // Light.
    for lg in gp.light.iter().filter(|l| l.media.contains(&p.medium)) {
        let mut why = vec![];
        let mut g = Gradient {
            id: lg.id.clone(),
            kind: "light".into(),
            dg_kj: None,
            per_electron_kj: None,
            limiting: None,
            supply_mol_yr: None,
            power_w: None,
            abiotic: "slow".into(),
            tests: Tests {
                quantum: true,
                power: false,
                materials,
            },
            open: false,
            production_gc_yr: None,
            max_biomass_gc: None,
            matches: lg.matches.clone(),
            why: vec![],
        };
        if !p.energy.available.iter().any(|c| c == "light") {
            why.push("no light reaches this habitat".into());
        }
        let in_reach = chem
            .light_bands_nm
            .iter()
            .any(|(lo, hi)| *lo <= lg.band_nm.0 && lg.band_nm.1 <= *hi)
            || chem
                .light_bands_nm
                .iter()
                .any(|(lo, hi)| *lo <= lg.band_nm.0 && lg.band_nm.0 < *hi);
        if !in_reach {
            why.push(format!("{} cannot use light in this band", chem.label));
        }
        match (p.energy.light_w_m2, p.area_m2) {
            (Some(flux), Some(area)) if why.is_empty() => {
                let peak = p.energy.light_peak_nm.unwrap_or_else(|| {
                    missing.push("light_peak_nm".to_string());
                    500.0
                });
                let w = flux
                    * crate::budget::band_fraction(peak, lg.band_nm.0, lg.band_nm.1)
                    * lg.efficiency
                    * area;
                let mut gc = w * YEAR_S / chem.build_j_per_g;
                photon_gc.insert(lg.id.clone(), gc);
                if let Some(d) = &lg.donor {
                    g.limiting = Some(d.clone());
                    match supply.get(d) {
                        Some(s) => {
                            g.supply_mol_yr = Some(*s);
                            gc = gc.min(s * lg.c_per_donor * 12.0);
                        }
                        None => {
                            why.push(format!("no supply of {d}"));
                        }
                    }
                }
                if why.is_empty() {
                    let power = gc * chem.build_j_per_g / YEAR_S;
                    g.power_w = Some(power);
                    g.tests.power = min_power.is_none_or(|mp| power >= mp);
                    g.production_gc_yr = Some(gc);
                    g.max_biomass_gc = maint.map(|w| power / w);
                }
            }
            (flux, area) => {
                if why.is_empty() {
                    if flux.is_none() {
                        missing.push("light_w_m2".into());
                    }
                    if area.is_none() {
                        missing.push("area_m2".into());
                    }
                    why.push("light or area unknown".into());
                }
            }
        }
        if !g.tests.power && g.power_w.is_some() {
            why.push("too little power to keep a population alive".into());
        }
        g.open = why.is_empty() && g.tests.power && g.tests.materials != Some(false);
        g.why = why;
        gradients.push(g);
    }

    // Redox and organic reactions.
    let acts = activities(p, gp);
    // Species standing above the threshold never limit (bicarbonate in
    // seawater, sulfate in today's ocean); a habitat can lower them with
    // `activity.<species>` (the Archean ocean had almost no sulfate).
    let abundant: Vec<String> = acts
        .iter()
        .filter(|(_, a)| **a >= gp.abundant_above)
        .map(|(s, _)| s.clone())
        .collect();
    let oxic = p
        .energy
        .available
        .iter()
        .any(|c| gp.oxic_channels.contains(c));
    let ph_here = ph(p, gp);
    for rg in gp.reactions.iter().filter(|r| r.media.contains(&p.medium)) {
        let Some(r) = table.reaction(&rg.reaction) else {
            continue;
        };
        let mut why = vec![];
        let mut g = Gradient {
            id: rg.reaction.clone(),
            kind: rg.kind.clone(),
            dg_kj: None,
            per_electron_kj: None,
            limiting: None,
            supply_mol_yr: None,
            power_w: None,
            abiotic: "slow".into(),
            tests: Tests {
                quantum: false,
                power: false,
                materials,
            },
            open: false,
            production_gc_yr: None,
            max_biomass_gc: None,
            matches: rg.matches.clone(),
            why: vec![],
        };
        // Reactants and their supply: organic carbon by import, oxygen
        // standing in an oxic habitat, abundant species never limiting.
        let mut limit: Option<(String, f64)> = None;
        for (s, nu) in r.terms.iter().filter(|(_, nu)| *nu < 0.0) {
            // The solvent and the hydrogen ion (set by the habitat's pH) never limit.
            if s == "h2o" || s == HYDROGEN_ION || abundant.contains(s) {
                continue;
            }
            let have = if *s == gp.organic {
                (organic_mol_yr > 0.0).then_some(organic_mol_yr)
            } else if *s == gp.oxygen && oxic && !supply.contains_key(s) {
                continue;
            } else {
                supply.get(s).copied()
            };
            match have {
                Some(v) => {
                    let turns = v / -nu;
                    if limit.as_ref().is_none_or(|(_, x)| turns < *x) {
                        limit = Some((s.clone(), turns));
                    }
                }
                None => why.push(format!("no supply of {s}")),
            }
        }
        let Some(t) = t else {
            why.push("temperature unknown".into());
            missing.push("temperature_k".into());
            g.why = why;
            gradients.push(g);
            continue;
        };
        if let Ok(e) = table.energetics(r, t, &acts) {
            g.dg_kj = Some(e.gibbs_j / 1000.0);
            g.per_electron_kj = e.per_electron_j.map(|x| x / 1000.0);
            g.tests.quantum = e.gibbs_j <= -gp.min_quantum_kj * 1000.0;
            if !g.tests.quantum {
                why.push(format!(
                    "{:.0} kJ per turnover is short of the {:.0} kJ quantum",
                    -e.gibbs_j / 1000.0,
                    gp.min_quantum_kj
                ));
            }
            if why.is_empty() {
                if let Some((s, turns)) = &limit {
                    let power = turns * -e.gibbs_j / YEAR_S;
                    g.limiting = Some(s.clone());
                    g.supply_mol_yr = Some(*turns);
                    g.power_w = Some(power);
                    g.tests.power = min_power.is_none_or(|mp| power >= mp);
                    g.max_biomass_gc = maint.map(|w| power / w);
                    if !g.tests.power {
                        why.push("too little power to keep a population alive".into());
                    }
                }
            }
        }
        let spent =
            rg.fast_above_k.is_some_and(|k| t > k) || rg.fast_above_ph.is_some_and(|x| ph_here > x);
        if spent {
            g.abiotic = "spent".into();
            why.push(
                rg.fast_why
                    .clone()
                    .unwrap_or_else(|| "it runs fast without life".into()),
            );
        }
        g.open =
            why.is_empty() && g.tests.quantum && g.tests.power && g.tests.materials != Some(false);
        g.why = why;
        gradients.push(g);
    }

    // Spend each supply once. The use that builds most living carbon per
    // mole of the contested reactant takes it first (among chemical uses
    // that is the most energetic, since yield follows energy); photosynthesis
    // that needs a donor competes on the same terms.
    let mut left = supply.clone();
    let mut organic_left = organic_mol_yr;
    let light_of = |id: &str| gp.light.iter().find(|l| l.id == id);
    let yield_per_mol = |g: &Gradient| -> f64 {
        if g.kind == "light" {
            return light_of(&g.id).map_or(0.0, |l| l.c_per_donor * 12.0);
        }
        let r = table.reaction(&g.id).expect("checked");
        let nu = g
            .limiting
            .as_ref()
            .and_then(|s| r.terms.iter().find(|(x, _)| x == s))
            .map_or(1.0, |(_, nu)| -nu);
        -g.dg_kj.unwrap_or(0.0) * 1000.0 * gp.growth_efficiency / chem.build_j_per_g / nu
    };
    let mut order: Vec<usize> = (0..gradients.len())
        .filter(|i| {
            let g = &gradients[*i];
            g.open
                && (g.kind != "light" || g.limiting.is_some())
                && usable.is_none_or(|u| u.contains(&g.id))
        })
        .collect();
    order.sort_by(|a, b| {
        yield_per_mol(&gradients[*b])
            .total_cmp(&yield_per_mol(&gradients[*a]))
            .then(gradients[*a].id.cmp(&gradients[*b].id))
    });
    for i in order {
        if gradients[i].kind == "light" {
            let g = &mut gradients[i];
            let d = g.limiting.clone().expect("filtered");
            let c = light_of(&g.id).map_or(0.0, |l| l.c_per_donor);
            let have = left.get(&d).copied().unwrap_or(0.0);
            let cap = photon_gc.get(&g.id).copied().unwrap_or(0.0);
            let gc = cap.min(have * c * 12.0);
            if let Some(v) = left.get_mut(&d) {
                *v -= if c > 0.0 { gc / (c * 12.0) } else { 0.0 };
            }
            g.supply_mol_yr = Some(if c > 0.0 { gc / (c * 12.0) } else { 0.0 });
            g.production_gc_yr = Some(gc);
            g.power_w = Some(gc * chem.build_j_per_g / YEAR_S);
            g.max_biomass_gc = maint.map(|w| gc * chem.build_j_per_g / YEAR_S / w);
            continue;
        }
        let r = table.reaction(&gradients[i].id).expect("checked");
        let mut turns = gradients[i].supply_mol_yr.unwrap_or(0.0);
        for (s, nu) in r.terms.iter().filter(|(_, nu)| *nu < 0.0) {
            if *s == gp.organic {
                turns = turns.min(organic_left / -nu);
            } else if let Some(v) = left.get(s) {
                turns = turns.min(v / -nu);
            }
        }
        let turns = turns.max(0.0);
        for (s, nu) in r.terms.iter().filter(|(_, nu)| *nu < 0.0) {
            if *s == gp.organic {
                organic_left -= turns * -nu;
            } else if let Some(v) = left.get_mut(s) {
                *v -= turns * -nu;
            }
        }
        let g = &mut gradients[i];
        let dg = -g.dg_kj.unwrap_or(0.0) * 1000.0;
        g.supply_mol_yr = Some(turns);
        g.power_w = Some(turns * dg / YEAR_S);
        g.production_gc_yr = Some(turns * dg * gp.growth_efficiency / chem.build_j_per_g);
        g.max_biomass_gc = maint.map(|w| turns * dg / YEAR_S / w);
    }

    // Budget: the best light gradient (they share the photons) plus every
    // chemical one; organic gradients build heterotrophs, not new carbon.
    let mut best_light: Option<f64> = None;
    let mut chemical = 0.0;
    let mut secondary = 0.0;
    let mut any = false;
    if let Some(u) = usable {
        // Open but not yet evolved: nothing drawn, nothing made.
        for g in gradients
            .iter_mut()
            .filter(|g| g.open && !u.contains(&g.id))
        {
            g.production_gc_yr = Some(0.0);
            g.power_w = Some(0.0);
            g.max_biomass_gc = Some(0.0);
        }
    }
    for g in gradients.iter().filter(|g| g.open) {
        let Some(gc) = g.production_gc_yr else {
            continue;
        };
        any = true;
        match g.kind.as_str() {
            "light" => best_light = Some(best_light.map_or(gc, |b: f64| b.max(gc))),
            "organic" => secondary += gc,
            _ => chemical += gc,
        }
        out.by_metabolism.insert(g.id.clone(), gc);
    }
    gradients.sort_by(|a, b| {
        b.open
            .cmp(&a.open)
            .then(
                b.power_w
                    .unwrap_or(0.0)
                    .total_cmp(&a.power_w.unwrap_or(0.0)),
            )
            .then(a.id.cmp(&b.id))
    });
    if gradients.len() > gp.cap {
        cat.dropped = (gradients.len() - gp.cap) as u32;
        gradients.truncate(gp.cap);
    }
    cat.gradients = gradients;
    missing.sort();
    missing.dedup();
    if !any {
        // A missing supply or input is a gap, never a zero (as box B).
        let gap = cat.gradients.iter().any(|g| {
            g.why
                .iter()
                .any(|w| w.starts_with("no supply") || w.contains("unknown"))
        });
        if gap {
            for g in &cat.gradients {
                for w in &g.why {
                    if let Some(s) = w.strip_prefix("no supply of ") {
                        if let Some((k, _)) = gp.supplies.iter().find(|(_, v)| *v == s) {
                            missing.push(format!("material.{k}"));
                        }
                    }
                }
            }
            missing.sort();
            missing.dedup();
            out.missing = missing;
            return (cat, out);
        }
        out.closed = Some(if cat.gradients.is_empty() {
            "no gradient this chemistry can reach".into()
        } else {
            "every gradient here fails a test".into()
        });
        out.energy_cap_gc_yr = Some(0.0);
        out.production_gc_yr = Some(0.0);
        out.power_w = Some(0.0);
        out.biomass_gc = Some(0.0);
        out.feeding_levels = Some(0);
        out.limited_by = "closed".into();
        out.missing = missing;
        return (cat, out);
    }
    let energy = best_light.unwrap_or(0.0) + chemical;
    let mut b = finish(p, bp, out, energy, missing);
    if secondary > 0.0 {
        b.secondary_gc_yr = Some(secondary);
        if energy == 0.0 {
            b.limited_by = "imported organic carbon".into();
        }
    }
    (cat, b)
}

/// A whole world under `gradients/1`: producers first (light and redox),
/// then heterotrophs on the organic carbon they export.
pub fn world_catalogue(
    g: &HabitatGraph,
    bp: &BudgetParams,
    gp: &GradientParams,
    table: &ThermoTable,
) -> (Vec<Catalogue>, WorldBudget) {
    world_catalogue_using(g, bp, gp, table, None)
}

/// As [`world_catalogue`], with only the gradients in `usable` tapped (E2).
pub fn world_catalogue_using(
    g: &HabitatGraph,
    bp: &BudgetParams,
    gp: &GradientParams,
    table: &ThermoTable,
    usable: Option<&BTreeSet<String>>,
) -> (Vec<Catalogue>, WorldBudget) {
    let sup = supplies(g, gp);
    let first: Vec<(Catalogue, HabitatBudget)> = g
        .habitats
        .iter()
        .zip(&sup)
        .map(|(h, s)| habitat_catalogue_using(&h.profile, bp, gp, table, s, 0.0, usable))
        .collect();
    let production: Vec<Option<f64>> = first.iter().map(|(_, b)| b.production_gc_yr).collect();
    let imports = organic_imports(g, gp, &production);
    let pairs: Vec<(Catalogue, HabitatBudget)> = g
        .habitats
        .iter()
        .zip(&sup)
        .zip(&imports)
        .map(|((h, s), imp)| habitat_catalogue_using(&h.profile, bp, gp, table, s, *imp, usable))
        .collect();
    let high_power = pairs.iter().any(|(c, b)| {
        c.gradients.iter().any(|x| {
            x.open
                && x.production_gc_yr.is_some_and(|v| v > 0.0)
                && gp.light.iter().any(|l| l.id == x.id && l.high_power)
        }) && b.production_gc_yr.is_some_and(|x| x > 0.0)
    });
    let (cats, mut habitats): (Vec<Catalogue>, Vec<HabitatBudget>) = pairs.into_iter().unzip();
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
    let wb = WorldBudget {
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
        biomass_class: crate::budget::biomass_class(biomass).into(),
        habitats,
        production_gc_yr: production,
        biomass_gc: biomass,
        power_w: power,
        high_power,
    };
    (cats, wb)
}

/// A world's budget under the stack's `life_model`: the listed metabolisms
/// in `have`, or the gradient catalogue (which needs no list; which
/// capabilities exist becomes box D's business in E2).
pub fn world_budget_by_model(
    g: &HabitatGraph,
    bp: &BudgetParams,
    gp: &GradientParams,
    table: &ThermoTable,
    have: &[String],
) -> WorldBudget {
    if bp.life_model == MODEL {
        world_catalogue(g, bp, gp, table).1
    } else {
        crate::budget::world_budget(g, bp, have)
    }
}
