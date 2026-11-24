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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WeatheringSettings {
    pub t_myr_at_1au: f64,
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
    pub weathering: WeatheringSettings,
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
