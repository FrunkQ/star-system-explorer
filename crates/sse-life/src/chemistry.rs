//! Chemistries and reaction energetics (evolution design sections 3 and 4,
//! stage E0).
//!
//! A **chemistry** is pack data: a solvent and when it is liquid, a scaffold
//! element and the temperature window its bonds suit, the element recipe and
//! build cost, the limits it can bear, and an evidence tag. Box B's closure
//! tests are questions asked of the chemistry a habitat's solvent names;
//! nothing in code names a solvent.
//!
//! The **thermodynamic table** holds the standard free energy and enthalpy
//! of formation of every species the life suite tracks, each with its
//! formula and charge, so a reaction's balance can be checked. A reaction's
//! free energy under local conditions comes from the two `sse.chem` blocks:
//! the standard value moved to the local temperature, then ΔG = ΔG° + RT ln Q.

use crate::*;
use sse_atoms::chem as blocks;

/// Evolution Lab's six evidence tags (evolution design section 1).
pub const EVIDENCE: [&str; 6] = [
    "observed",
    "demonstrated",
    "inferred",
    "plausible",
    "speculative",
    "fictional",
];

/// One way of being alive, as data (section 4.1).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Chemistry {
    pub id: String,
    pub label: String,
    /// The habitat solvent this chemistry lives in (habitat rules name it).
    pub solvent: String,
    pub scaffold: String,
    /// Temperature window in which the scaffold's bonds hold information but
    /// still react, K.
    pub window_k: (f64, f64),
    /// Below this pressure the solvent cannot be liquid (its triple point), Pa.
    #[serde(default)]
    pub liquid_min_pa: Option<f64>,
    /// The habitat property holding the solvent's activity, and the lowest
    /// activity the chemistry can work at.
    #[serde(default)]
    pub activity_property: Option<String>,
    #[serde(default)]
    pub min_activity: Option<f64>,
    /// A dose limit of its own; otherwise box B's.
    #[serde(default)]
    pub max_dose_msv_yr: Option<f64>,
    /// Element recipe of living matter, molar ratios (C:N:P for Redfield).
    pub recipe: BTreeMap<String, f64>,
    /// Free energy stored per gram of scaffold element built into living matter, J.
    pub build_j_per_g: f64,
    /// Light bands it can couple to, nm.
    #[serde(default)]
    pub light_bands_nm: Vec<(f64, f64)>,
    /// Thermodynamic species it can use as electron donors or acceptors.
    #[serde(default)]
    pub redox_species: Vec<String>,
    /// One of `EVIDENCE`.
    pub evidence: String,
    /// Origin base-rate multiplier (box C, section 4.3).
    #[serde(default = "one")]
    pub origin_dial: f64,
    #[serde(default)]
    pub sources: Vec<String>,
}

fn one() -> f64 {
    1.0
}

impl Chemistry {
    pub fn check(&self) -> Result<(), String> {
        if !EVIDENCE.contains(&self.evidence.as_str()) {
            return Err(format!(
                "chemistry {}: evidence {:?} is not one of {EVIDENCE:?}",
                self.id, self.evidence
            ));
        }
        if self.window_k.0 >= self.window_k.1 || self.build_j_per_g <= 0.0 {
            return Err(format!("chemistry {}: bad window or build cost", self.id));
        }
        Ok(())
    }
}

/// One species in the thermodynamic table, standard state at 298.15 K
/// (gases at 1 bar, solutes at 1 molal, H+ at activity 1).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Species {
    pub id: String,
    pub label: String,
    /// `g`, `l`, `s` or `aq`.
    pub phase: String,
    /// Elements in one formula unit.
    pub elements: BTreeMap<String, f64>,
    #[serde(default)]
    pub charge: f64,
    /// Standard free energy and enthalpy of formation, kJ per mol.
    pub dgf_kj: f64,
    pub dhf_kj: f64,
    /// Short source key (one of the table's `sources`).
    pub source: String,
}

/// A reaction as written: species and coefficients (negative for
/// reactants, positive for products).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Reaction {
    pub id: String,
    pub label: String,
    pub terms: Vec<(String, f64)>,
    /// Electrons moved per reaction as written (redox reactions only).
    #[serde(default)]
    pub electrons: Option<f64>,
    /// A published free energy to check against, kJ per mol, at `published_ph`
    /// (absent: standard state, pH 0), 298.15 K.
    #[serde(default)]
    pub published_kj: Option<f64>,
    #[serde(default)]
    pub published_ph: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
}

/// `provides.other.life_chemistry`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ThermoTable {
    pub reference_t_k: f64,
    pub species: Vec<Species>,
    pub reactions: Vec<Reaction>,
    /// Source key to full reference.
    pub sources: BTreeMap<String, String>,
}

/// Free energy of one reaction, J per mol of reaction as written.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Energetics {
    pub reaction: String,
    pub t_k: f64,
    pub standard_298_j: f64,
    pub enthalpy_j: f64,
    pub standard_j: f64,
    pub ln_q: f64,
    pub gibbs_j: f64,
    /// Per electron moved, when the reaction says how many.
    pub per_electron_j: Option<f64>,
}

impl ThermoTable {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("life_chemistry"))
            .ok_or("no pack provides life_chemistry")?;
        let t: ThermoTable = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("life_chemistry: {e}"))?;
        t.check()?;
        Ok(t)
    }

    pub fn species(&self, id: &str) -> Option<&Species> {
        self.species.iter().find(|s| s.id == id)
    }

    pub fn reaction(&self, id: &str) -> Option<&Reaction> {
        self.reactions.iter().find(|r| r.id == id)
    }

    /// Every species unique and sourced; every reaction names known species
    /// and balances in elements and charge.
    pub fn check(&self) -> Result<(), String> {
        let mut seen: Vec<&str> = vec![];
        for s in &self.species {
            if seen.contains(&s.id.as_str()) {
                return Err(format!("species {} listed twice", s.id));
            }
            seen.push(&s.id);
            if !self.sources.contains_key(&s.source) {
                return Err(format!("species {}: unknown source {}", s.id, s.source));
            }
        }
        for r in &self.reactions {
            self.balance(r)?;
        }
        Ok(())
    }

    /// Error unless the reaction balances in every element and in charge.
    pub fn balance(&self, r: &Reaction) -> Result<(), String> {
        let mut el: BTreeMap<String, f64> = BTreeMap::new();
        let mut charge = 0.0;
        for (id, nu) in &r.terms {
            let s = self
                .species(id)
                .ok_or_else(|| format!("reaction {}: unknown species {id}", r.id))?;
            for (e, n) in &s.elements {
                *el.entry(e.clone()).or_insert(0.0) += nu * n;
            }
            charge += nu * s.charge;
        }
        for (e, n) in &el {
            if n.abs() > 1e-9 {
                return Err(format!("reaction {}: {e} does not balance ({n})", r.id));
            }
        }
        if charge.abs() > 1e-9 {
            return Err(format!(
                "reaction {}: charge does not balance ({charge})",
                r.id
            ));
        }
        Ok(())
    }

    /// ΔG of a reaction at `t_k` and the given activities (species id to
    /// activity; missing species are at their standard state, activity 1).
    pub fn energetics(
        &self,
        r: &Reaction,
        t_k: f64,
        activities: &BTreeMap<String, f64>,
    ) -> Result<Energetics, String> {
        let mut g = 0.0;
        let mut h = 0.0;
        let mut q = vec![];
        for (id, nu) in &r.terms {
            let s = self
                .species(id)
                .ok_or_else(|| format!("reaction {}: unknown species {id}", r.id))?;
            g += nu * s.dgf_kj * 1000.0;
            h += nu * s.dhf_kj * 1000.0;
            q.push((*nu, activities.get(id).copied().unwrap_or(1.0)));
        }
        let ln_q = blocks::ln_quotient(&q).map_err(|e| format!("reaction {}: {e}", r.id))?;
        let standard = blocks::standard_gibbs_at(g, h, t_k);
        let gibbs = blocks::reaction_gibbs(standard, t_k, ln_q);
        Ok(Energetics {
            reaction: r.id.clone(),
            t_k,
            standard_298_j: g,
            enthalpy_j: h,
            standard_j: standard,
            ln_q,
            gibbs_j: gibbs,
            per_electron_j: r.electrons.filter(|n| *n > 0.0).map(|n| gibbs / n),
        })
    }

    /// The activities of a reaction's published conditions: everything at
    /// its standard state, hydrogen ion at 10^-pH.
    pub fn published_activities(&self, r: &Reaction) -> BTreeMap<String, f64> {
        let mut a = BTreeMap::new();
        if let Some(ph) = r.published_ph {
            a.insert(HYDROGEN_ION.into(), sse_core::math::pow(10.0, -ph));
        }
        a
    }
}

/// The thermodynamic table's id for the hydrogen ion.
pub const HYDROGEN_ION: &str = "h+";

/// The chemistry for a solvent: the stack's own definitions first; a pack
/// written before E0 (only a list of solvents and box B's water limits)
/// gets one built from those, so old manifests read exactly as before.
pub fn chemistry_for(bp: &crate::budget::BudgetParams, solvent: &str) -> Option<Chemistry> {
    if let Some(c) = bp.chemistry.iter().find(|c| c.solvent == solvent) {
        return Some(c.clone());
    }
    if !bp.chemistries.iter().any(|s| s == solvent) {
        return None;
    }
    Some(Chemistry {
        id: format!("legacy-{solvent}"),
        label: format!("carbon in {solvent} (pre-E0 pack)"),
        solvent: solvent.into(),
        scaffold: "C".into(),
        window_k: bp.window_k.unwrap_or((0.0, f64::MAX)),
        liquid_min_pa: bp.water_triple_pa,
        activity_property: Some(format!("{solvent}_activity")),
        min_activity: bp.min_water_activity,
        max_dose_msv_yr: None,
        recipe: BTreeMap::new(),
        build_j_per_g: bp.j_per_g_carbon,
        light_bands_nm: vec![],
        redox_species: vec![],
        evidence: "observed".into(),
        origin_dial: 1.0,
        sources: vec![],
    })
}
