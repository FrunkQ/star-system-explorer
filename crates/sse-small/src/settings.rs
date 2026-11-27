//! The cited, editable numbers (`packs/base-small-bodies`, `provides.other.small_bodies`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PACK: &str = include_str!("../../../packs/base-small-bodies/pack.json");

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MakeupSettings {
    pub scatter_ln: f64,
    pub differentiated_share: BTreeMap<String, f64>,
    pub layer_purity: f64,
    pub core_keeps: f64,
    /// Grain densities of metal, rock, carbon and ice, kg/m^3: read from the
    /// master data sheet (mixtures `formation-metal`, `-rock`, `-carbon`,
    /// `-water`, the same rows formation sizes its bodies by), never typed.
    #[serde(skip)]
    pub grain_density_kg_m3: [f64; 4],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct IceSettings {
    pub survives_below_k: f64,
    pub reddened_below_k: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TypeSettings {
    pub metal_min: f64,
    pub icy_min: f64,
    pub carbon_min: f64,
    pub fresh_below: f64,
    pub albedo: BTreeMap<String, f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CollisionSettings {
    pub t0_rock_myr: f64,
    pub t0_metal_myr: f64,
    pub gravity_transition_km: f64,
    pub main_belt_over_1km: f64,
    pub main_belt_a_au: [f64; 2],
    pub main_belt_e: f64,
    pub main_belt_i_deg: f64,
    pub porosity_rubble: f64,
    pub porosity_monolith: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct YorpSettings {
    pub t_myr_at_1km_1au: f64,
    pub flat_below_hours: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SpinSettings {
    pub maxwell_mean_per_day: f64,
    pub monolith_fastest_hours: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PairSettings {
    pub yorp_contact: f64,
    pub yorp_separated: f64,
    pub cold_speed_km_s: f64,
    pub cold_contact: f64,
    pub cold_separated: f64,
    pub large_moon_km: f64,
    pub large_moon_share: f64,
    pub yorp_secondary: [f64; 2],
    pub contact_secondary: [f64; 2],
    pub cold_secondary: [f64; 2],
    pub yorp_separation_primary_radii: [f64; 2],
    pub cold_separation_primary_radii: [f64; 2],
    pub moon_separation_primary_radii: [f64; 2],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FigureSettings {
    pub round_km_rock: f64,
    pub round_km_icy: f64,
    pub elongation_b_over_a: [f64; 2],
    pub flattening_c_over_b: [f64; 2],
    pub top_shape_share_of_barrier: f64,
    pub cold_lobe_c_over_a: f64,
    pub neck_overlap: f64,
    pub relief_monolith: [f64; 2],
    pub relief_rubble: [f64; 2],
}

/// What real material holds, for the designer's "what it would take"
/// (takes.rs), filled at load from the data sheet's mixtures
/// `formation-rock` and `formation-water` (fudge SB-strength-sheet).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct StrengthSettings {
    pub regolith_cohesion_pa: f64,
    pub rock_tensile_pa: f64,
    pub ice_tensile_pa: f64,
    pub rock_compressive_pa: f64,
    pub ice_compressive_pa: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WeatheringSettings {
    pub t_myr_at_1au: f64,
}

/// One ice a nucleus can lose, with its sheet values filled at load.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VolatileSettings {
    /// The sheet's substance id (h2o, co2, co).
    pub id: String,
    /// Which of formation's solid groups it is (3 water, 4 CO2, 5 CO).
    pub group: usize,
    /// Lifetime in sunlight at 1 AU, s (scales as r^2): the sheet's
    /// `photochemistry.lifetime_1au`, filled at load.
    #[serde(skip)]
    pub lifetime_s_at_1au: f64,
    #[serde(skip)]
    pub molar_kg_mol: f64,
    #[serde(skip)]
    pub phases: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CometSettings {
    /// Share of the surface that sublimates (the rest is a dust mantle).
    pub active_fraction: f64,
    /// Share of the subsolar insolation a fast-turning nucleus gets on average (1/4 isothermal).
    pub insolation_share: f64,
    pub emissivity: f64,
    /// Bond albedo over geometric albedo.
    pub phase_integral: f64,
    pub dust_to_gas: f64,
    /// Gas outflow speed at 1 AU, km/s (scales as r^-0.5).
    pub outflow_km_s_at_1au: f64,
    /// Sublimation counts as activity once it takes this share of the absorbed sunlight.
    pub onset_share: f64,
    /// Insolation share for the onset: 1, the subsolar point (Cowan and A'Hearn 1979's subsolar model).
    pub onset_insolation_share: f64,
    /// Free sublimation rate of an ice under the dust mantle, as a share of the bare rate.
    pub mantle_factor: f64,
    /// How long dust has been leaving when the tail is drawn, days.
    pub tail_age_days: f64,
    /// Radiation pressure over gravity for the dust that makes the visible tail.
    pub dust_beta: f64,
    pub solar_wind_km_s: f64,
    pub volatiles: Vec<VolatileSettings>,
    #[serde(skip)]
    pub sigma_sb: f64,
    #[serde(skip)]
    pub r_gas: f64,
    #[serde(skip)]
    pub n_a: f64,
    #[serde(skip)]
    pub s_sun_w_m2: f64,
    #[serde(skip)]
    pub gm_sun: f64,
    #[serde(skip)]
    pub au_m: f64,
    #[serde(skip)]
    pub day_s: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RecipeInput {
    pub id: String,
    pub default: f64,
    pub unit: String,
    pub kind: String,
}

/// One "Add an object" entry (owned by the editing track's Add list).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub label: String,
    pub kernel_kind: String,
    pub inputs: Vec<RecipeInput>,
    pub citation: String,
}

/// Everything the resolve reads. Unknown keys such as `about` are ignored.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub makeup: MakeupSettings,
    pub ice: IceSettings,
    pub types: TypeSettings,
    pub collisions: CollisionSettings,
    pub yorp: YorpSettings,
    pub spin: SpinSettings,
    pub pairs: PairSettings,
    pub figure: FigureSettings,
    #[serde(skip)]
    pub strength: StrengthSettings,
    pub weathering: WeatheringSettings,
    pub comet: CometSettings,
    pub recipes: Vec<Recipe>,
}

impl Settings {
    /// Read from a pack manifest's JSON (`provides.other.small_bodies`).
    pub fn from_pack_json(text: &str) -> Result<Self, String> {
        Self::from_pack_json_with(text, &sse_reference::Reference::base())
    }
    /// The same, reading grain densities from a given sheet.
    pub fn from_pack_json_with(
        text: &str,
        sheet: &sse_reference::Reference,
    ) -> Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let s = v
            .pointer("/provides/other/small_bodies")
            .ok_or("no provides.other.small_bodies")?;
        Self::from_block_with(s, sheet)
    }
    /// From a `small_bodies` settings block itself (the stack's, as the
    /// physics in force holds it), reading the sheet's values from `sheet`.
    pub fn from_block_with(
        s: &serde_json::Value,
        sheet: &sse_reference::Reference,
    ) -> Result<Self, String> {
        let mut out: Settings = serde_json::from_value(s.clone()).map_err(|e| e.to_string())?;
        for (g, id) in [
            "formation-metal",
            "formation-rock",
            "formation-carbon",
            "formation-water",
        ]
        .iter()
        .enumerate()
        {
            out.makeup.grain_density_kg_m3[g] = sheet
                .row("mixtures", id)
                .and_then(|r| r.get("density"))
                .and_then(|d| d.get("v"))
                .and_then(|v| v.as_f64())
                .ok_or_else(|| format!("the sheet has no density for {id}"))?;
        }
        let held = |id: &str, key: &str| {
            sheet
                .row("mixtures", id)
                .and_then(|r| r.get(key))
                .and_then(|d| d.get("v"))
                .and_then(|v| v.as_f64())
                .ok_or_else(|| format!("the sheet has no {key} for {id}"))
        };
        out.strength = StrengthSettings {
            regolith_cohesion_pa: held("formation-rock", "rubble_cohesion")?,
            rock_tensile_pa: held("formation-rock", "tensile_strength")?,
            ice_tensile_pa: held("formation-water", "tensile_strength")?,
            rock_compressive_pa: held("formation-rock", "compressive_strength")?,
            ice_compressive_pa: held("formation-water", "compressive_strength")?,
        };
        let c = &mut out.comet;
        let k = |id: &str| {
            sheet
                .constant(id)
                .ok_or_else(|| format!("the sheet has no constant {id}"))
        };
        c.sigma_sb = k("sigma_SB")?;
        c.r_gas = k("R_gas")?;
        c.n_a = k("N_A")?;
        c.s_sun_w_m2 = k("S_sun")?;
        c.gm_sun = k("GM_sun")?;
        c.au_m = k("au")?;
        c.day_s = k("day")?;
        for v in &mut c.volatiles {
            v.molar_kg_mol = sheet
                .molar_mass_kg_mol(&v.id)
                .ok_or_else(|| format!("the sheet has no molar mass for {}", v.id))?;
            v.lifetime_s_at_1au = sheet
                .row("substances", &v.id)
                .and_then(|r| r.get("photochemistry"))
                .and_then(|p| p.get("lifetime_1au"))
                .and_then(|l| l.get("v"))
                .and_then(|x| x.as_f64())
                .ok_or_else(|| format!("the sheet has no lifetime in sunlight for {}", v.id))?;
            v.phases = sheet
                .row("substances", &v.id)
                .and_then(|r| r.get("phases"))
                .and_then(|p| p.as_object())
                .cloned()
                .ok_or_else(|| format!("the sheet has no phases for {}", v.id))?;
        }
        Ok(out)
    }
    pub fn albedo(&self, label: &str) -> f64 {
        self.types.albedo.get(label).copied().unwrap_or(0.1)
    }
}

impl Default for Settings {
    /// The base pack's numbers.
    fn default() -> Self {
        Settings::from_pack_json(PACK).expect("base-small-bodies pack parses")
    }
}
