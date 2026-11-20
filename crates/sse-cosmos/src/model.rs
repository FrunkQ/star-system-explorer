//! The patch model's parameters: pack data, never constants in code (galaxy
//! design, preamble). The shipped defaults live in `packs/base-cosmos`.

use serde::{Deserialize, Serialize};
use sse_pack::PackManifest;
use sse_resolve::Distribution;
use std::collections::BTreeMap;

pub const BASE_COSMOS_JSON: &str = include_str!("../../../packs/base-cosmos/pack.json");

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Geometry {
    Disc,
    Halo,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Population {
    pub name: String,
    pub geometry: Geometry,
    /// Systems per cubic parsec at the Sun's radius, in the midplane, now.
    pub density_pc3: f64,
    #[serde(default)]
    pub scale_length_kpc: f64,
    #[serde(default)]
    pub scale_height_pc: f64,
    /// Halo: density falls as r^-power, with vertical flattening q.
    #[serde(default)]
    pub power: f64,
    #[serde(default)]
    pub flattening: f64,
}

/// A birth cohort: stars of one population born in one span of time
/// (galaxy design 8.1, "a cohort is birth patch and birth time step").
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CohortSpec {
    pub population: String,
    /// Universe time span, in Gyr from the origin.
    pub from_gyr: f64,
    pub to_gyr: f64,
    /// Share of its population's systems (relative within the population).
    pub share: f64,
    pub feh_sd: f64,
    /// A fixed mean [Fe/H] (the halo), instead of the ISM enrichment curve.
    #[serde(default)]
    pub feh_fixed: Option<f64>,
    /// Added to the curve's mean (thick-disc stars formed further in).
    #[serde(default)]
    pub feh_offset: f64,
    #[serde(default)]
    pub alpha_fixed: Option<f64>,
    pub r_sigma_dex: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Enrichment {
    pub feh_now_at_sun: f64,
    pub gradient_dex_per_kpc: f64,
    pub tau_gyr: f64,
    pub floor: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AlphaFe {
    pub plateau: f64,
    pub now: f64,
    pub knee_gyr: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CO {
    pub solar: f64,
    pub dex_per_feh: f64,
    pub sd: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MeanSd {
    pub mean: f64,
    pub sd: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Spread {
    pub sigma_dex: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MedianSpread {
    pub median: f64,
    pub sigma_dex: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cluster {
    pub alpha: f64,
    pub lo_msun: f64,
    pub hi_msun: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Fuv {
    pub median_g0: f64,
    pub sigma_dex: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sfr {
    pub galaxy_msun_per_yr: f64,
    pub scale_length_kpc: f64,
    pub young_scale_height_pc: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ia {
    pub per_msun: f64,
    pub min_delay_gyr: f64,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub sigma_at_10gyr_kms: f64,
    pub power: f64,
    pub halo_kms: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PatchModel {
    pub id: String,
    #[serde(default)]
    pub about: String,
    pub now_gyr: f64,
    pub galaxy_start_gyr: f64,
    pub sun_radius_kpc: f64,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
    pub imf: Distribution,
    pub cc_sn_min_mass: f64,
    pub enrichment: Enrichment,
    pub alpha_fe: AlphaFe,
    pub c_o: CO,
    pub mg_si: MeanSd,
    pub k40: Spread,
    pub al26: MedianSpread,
    pub cluster: Cluster,
    pub fuv: Fuv,
    pub sfr: Sfr,
    pub ia: Ia,
    pub velocity: Velocity,
    pub rocky_feh: f64,
    pub sterilising_pc: f64,
    pub encounter_pc: f64,
    pub populations: Vec<Population>,
    pub cohorts: Vec<CohortSpec>,
    /// What lights up a patch: dying and massive stars (nebulae and light design 4).
    #[serde(default)]
    pub glows: Glows,
}

/// How a patch counts the nebulae it expects without placing them.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Glows {
    /// Main-sequence lifetime = sun_life_gyr x M^-lifetime_slope (the same
    /// rule as the star settings).
    pub sun_life_gyr: f64,
    pub lifetime_slope: f64,
    /// Initial-final mass relation, white dwarf = wd_a x M + wd_b (Kalirai et al. 2008).
    pub wd_a: f64,
    pub wd_b: f64,
    /// Stars at least this heavy light an H II region (late O and early B).
    pub ionising_min_msun: f64,
    /// Mean gas density nebulae expand into, hydrogen per m^3.
    pub ambient_h_m3: f64,
}

impl Default for Glows {
    fn default() -> Self {
        Glows {
            sun_life_gyr: 10.0,
            lifetime_slope: 2.5,
            wd_a: 0.109,
            wd_b: 0.394,
            ionising_min_msun: 15.0,
            ambient_h_m3: 1e6,
        }
    }
}

impl PatchModel {
    /// The model a pack provides under `provides.other.patch_model`.
    pub fn from_pack(p: &PackManifest) -> Result<Self, String> {
        let v = p
            .provides
            .other
            .get("patch_model")
            .ok_or_else(|| format!("{}: no patch_model", p.id))?;
        let json = serde_json::to_value(v).map_err(|e| e.to_string())?;
        let m: PatchModel =
            serde_json::from_value(json).map_err(|e| format!("{}: patch_model: {e}", p.id))?;
        m.validate()
            .map_err(|e| format!("{}: patch_model: {e}", p.id))?;
        Ok(m)
    }

    /// The shipped Milky-Way-like default.
    pub fn milky_way_like() -> Self {
        let p = PackManifest::from_json(BASE_COSMOS_JSON).expect("base-cosmos pack parses");
        Self::from_pack(&p).expect("base-cosmos patch model is valid")
    }

    pub fn validate(&self) -> Result<(), String> {
        self.imf.validate().map_err(|e| format!("imf: {e}"))?;
        if !matches!(
            self.imf,
            Distribution::PowerLaw { .. } | Distribution::BrokenPowerLaw { .. }
        ) {
            return Err("imf must be a power law or broken power law".into());
        }
        if !(self.galaxy_start_gyr >= 0.0 && self.galaxy_start_gyr < self.now_gyr) {
            return Err("galaxy_start_gyr must be before now_gyr".into());
        }
        for c in &self.cohorts {
            if !self.populations.iter().any(|p| p.name == c.population) {
                return Err(format!("cohort names unknown population {}", c.population));
            }
            if !(c.from_gyr < c.to_gyr && c.from_gyr >= self.galaxy_start_gyr) {
                return Err(format!(
                    "cohort {} {}..{} Gyr must be a forward span after the galaxy starts",
                    c.population, c.from_gyr, c.to_gyr
                ));
            }
            if !(c.share > 0.0 && c.feh_sd > 0.0 && c.r_sigma_dex > 0.0) {
                return Err(format!(
                    "cohort {} needs share, feh_sd, r_sigma_dex > 0",
                    c.population
                ));
            }
        }
        for p in &self.populations {
            let ok = match p.geometry {
                Geometry::Disc => p.scale_length_kpc > 0.0 && p.scale_height_pc > 0.0,
                Geometry::Halo => p.power > 0.0 && p.flattening > 0.0,
            };
            if !ok || p.density_pc3 < 0.0 {
                return Err(format!("population {} is missing its geometry", p.name));
            }
        }
        Ok(())
    }
}
