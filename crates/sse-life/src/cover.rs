//! E6 part 1: regional cover and biomes (evolution design section 9).
//!
//! One region's climate, light and soil go in, and its **cover** comes out:
//! closure, canopy height, production, standing mass and fire. A **biome**
//! is a label read off the cover by an editable atlas (section 9.2: never
//! stored as the only record). Nothing here asks what kind of body it is.
//!
//! The steps follow BIOME (Prentice et al. 1992), the first physiology-based
//! global vegetation model, which the design names as the comparison. BIOME
//! uses the same three climate variables: warmth, the coldest season, and
//! the moisture index α (actual over potential evaporation). Here they are
//! smooth curves rather than gates (processes, not classes):
//!
//! - **Woody potential.** Woody plants need water: their potential closure
//!   rises from α_open to α_closed (BIOME's woodland and forest limits).
//!   Trees need warmth: the global tree line sits where the growing season
//!   averages 6.4 °C (Körner and Paulsen 2004).
//! - **Ground cover.** Herbs and grasses need less water, and reach full
//!   cover at α_grass.
//! - **Production.** Light-use efficiency (CASA's ε* = 0.389 g C per MJ of
//!   PAR, Potter et al. 1993) times the light at the ground, the green
//!   cover, and the share of the year warm enough to grow (above 0 °C).
//! - **Fire.** The grass layer is the fuel, and it burns in proportion to the
//!   water deficit (1 − α), the fire weather (saturation vapour pressure at
//!   the warm season, relative to 27 °C), and the oxygen ignition index
//!   relative to today's (E3). Trees that cannot escape the flames between
//!   fires are kept open. Without fire, closed forest would cover about
//!   twice the area it does (Bond, Woodward and Midgley 2005), so the
//!   no-fire what-if is a test. Shade suppresses the grass under a canopy,
//!   so the fixed point is solved from an open start: savannah where fire
//!   wins, forest where trees outgrow it.
//! - **Height.** The tallest strategy that pays takes the light (section
//!   9.2): a typical tall canopy, scaled by 1/g like E5's hydraulic limit,
//!   by the water and by the season.
//!
//! Fudge F33 holds the curve shapes and the fire constants.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Settings (`life_cover`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CoverParams {
    /// Moisture index where woody cover starts, and where it can close
    /// (after Prentice et al. 1992's woodland and forest limits).
    pub alpha_open: f64,
    pub alpha_closed: f64,
    /// Moisture index at which ground cover is complete.
    pub alpha_grass: f64,
    /// Growing-season temperature of the tree line, K (Körner and Paulsen
    /// 2004: 6.4 °C), and the width of the transition, K.
    pub treeline_k: f64,
    pub treeline_width_k: f64,
    /// Base temperature of the growing season, K (0 °C: BIOME's GDD0, the
    /// base it uses for tundra, so the shortest arctic summers still count).
    pub grow_base_k: f64,
    /// Light-use efficiency, g C per MJ of photosynthetically active
    /// radiation (Potter et al. 1993, CASA).
    pub lue_gc_mj: f64,
    /// A typical tall closed canopy at Earth's gravity, m (Simard et al.
    /// 2011: tropical and temperate forest tops of 25 to 40 m).
    pub canopy_m: f64,
    /// Fires a year in fully cured, continuous fuel in hot dry weather
    /// (annual burning in mesic savannah, Bond et al. 2005; fudge F33).
    pub fire_rate_max: f64,
    /// Grass production, g C m-2 yr-1, at which fuel is half continuous.
    pub fuel_half_gc: f64,
    /// Years a sapling needs to escape the flame zone (fudge F33).
    pub escape_yr: f64,
    /// Fire weather reference temperature, K (27 °C).
    pub fire_ref_k: f64,
    /// Residence time of carbon in wood and in herbs, years (Carvalhais et
    /// al. 2014's ecosystem turnover; herbs about a year).
    pub wood_residence_yr: f64,
    pub herb_residence_yr: f64,
    /// Labels, in order: the first row whose ranges all hold names the cover.
    pub atlas: Vec<BiomeRow>,
    pub sources: Vec<String>,
}

/// One atlas row. Every bound is optional; a row with none always matches.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BiomeRow {
    /// Structural name (section 9.4), the label on any world.
    pub structure: String,
    /// Earth's name, offered when the numbers also fit Earth's definition.
    pub earth: String,
    /// The broad type of the design's Earth check.
    pub broad: String,
    pub closure_min: Option<f64>,
    pub closure_max: Option<f64>,
    pub height_min_m: Option<f64>,
    pub height_max_m: Option<f64>,
    pub npp_min_gc: Option<f64>,
    pub npp_max_gc: Option<f64>,
    pub t_cold_min_k: Option<f64>,
    pub t_cold_max_k: Option<f64>,
    pub t_warm_min_k: Option<f64>,
    pub t_warm_max_k: Option<f64>,
    /// Matches only where snow outlasts the year (or only where it does not).
    pub ice: Option<bool>,
}

fn row(structure: &str, earth: &str, broad: &str) -> BiomeRow {
    BiomeRow {
        structure: structure.into(),
        earth: earth.into(),
        broad: broad.into(),
        ..Default::default()
    }
}

/// The starting atlas. Köppen's 18 °C coldest month marks the tropics and
/// 10 °C warmest month the tundra (Kottek et al. 2006). Boreal below a
/// −15 °C coldest season follows BIOME's boreal limit.
pub fn default_atlas() -> Vec<BiomeRow> {
    let tropic = 291.15;
    let boreal = 258.15;
    let tundra = 283.15;
    vec![
        BiomeRow {
            ice: Some(true),
            ..row("ice", "ice sheet", "ice")
        },
        BiomeRow {
            npp_max_gc: Some(50.0),
            t_warm_max_k: Some(tundra),
            ..row("bare", "polar desert", "scrub and desert")
        },
        BiomeRow {
            npp_max_gc: Some(50.0),
            ..row("bare", "desert", "scrub and desert")
        },
        BiomeRow {
            closure_min: Some(0.6),
            height_min_m: Some(10.0),
            t_cold_min_k: Some(tropic),
            ..row("closed tall canopy", "tropical rainforest", "closed forest")
        },
        BiomeRow {
            closure_min: Some(0.6),
            height_min_m: Some(10.0),
            t_cold_max_k: Some(boreal),
            ..row("closed tall canopy", "boreal forest", "closed forest")
        },
        BiomeRow {
            closure_min: Some(0.6),
            height_min_m: Some(10.0),
            ..row("closed tall canopy", "temperate forest", "closed forest")
        },
        BiomeRow {
            closure_min: Some(0.1),
            height_min_m: Some(3.0),
            t_cold_min_k: Some(tropic),
            ..row(
                "open canopy over ground cover",
                "savannah",
                "open woodland and savannah",
            )
        },
        BiomeRow {
            closure_min: Some(0.1),
            height_min_m: Some(3.0),
            ..row(
                "open canopy over ground cover",
                "open woodland",
                "open woodland and savannah",
            )
        },
        BiomeRow {
            closure_min: Some(0.1),
            t_warm_max_k: Some(tundra),
            ..row("low woody cover", "shrub tundra", "tundra")
        },
        BiomeRow {
            closure_min: Some(0.1),
            ..row("low woody cover", "shrubland", "scrub and desert")
        },
        BiomeRow {
            t_warm_max_k: Some(tundra),
            ..row("low mat", "tundra", "tundra")
        },
        row("ground cover", "grassland", "grassland"),
    ]
}

impl Default for CoverParams {
    fn default() -> Self {
        CoverParams {
            alpha_open: 0.28,
            alpha_closed: 0.65,
            alpha_grass: 0.4,
            treeline_k: 279.55,
            treeline_width_k: 2.0,
            grow_base_k: sse_core::sheet::T0C,
            lue_gc_mj: 0.389,
            canopy_m: 35.0,
            fire_rate_max: 1.0,
            fuel_half_gc: 200.0,
            escape_yr: 3.0,
            fire_ref_k: 300.15,
            wood_residence_yr: 20.0,
            herb_residence_yr: 1.0,
            atlas: default_atlas(),
            sources: vec![],
        }
    }
}

/// One region's inputs (the shared climate's water supply at the place,
/// with light, gravity and air).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoverInput {
    /// Annual mean, coldest and warmest season temperature, K.
    pub t_k: f64,
    pub t_cold_k: f64,
    pub t_warm_k: f64,
    /// Potential and actual evaporation, m a year.
    pub pet_m_yr: f64,
    pub evap_m_yr: f64,
    /// Snow less melt, m a year (above zero a glacier grows).
    pub ice_balance_m_yr: f64,
    /// Photosynthetically active light at the ground, MJ m-2 yr-1.
    pub par_mj_m2_yr: f64,
    pub gravity_m_s2: f64,
    /// The oxygen fire ignition index relative to today's air (E3), 1 on
    /// Earth today, 0 with no free oxygen.
    pub ignition_rel: f64,
}

impl CoverInput {
    /// From the shared climate's water supply at one place.
    pub fn from_water(
        w: &sse_climate::water::WaterSupply,
        par_mj_m2_yr: f64,
        gravity_m_s2: f64,
        ignition_rel: f64,
    ) -> Self {
        CoverInput {
            t_k: w.t_k,
            t_cold_k: w.t_cold_k,
            t_warm_k: w.t_warm_k,
            pet_m_yr: w.pet_m_yr,
            evap_m_yr: w.evap_m_yr,
            ice_balance_m_yr: w.ice_balance_m_yr,
            par_mj_m2_yr,
            gravity_m_s2,
            ignition_rel,
        }
    }

    /// Moisture index: actual over potential evaporation (BIOME's α).
    pub fn alpha(&self) -> f64 {
        if self.pet_m_yr <= 0.0 {
            // Nothing can evaporate: cold or dark, not dry.
            1.0
        } else {
            (self.evap_m_yr / self.pet_m_yr).clamp(0.0, 1.0)
        }
    }
}

/// A region's cover: the record. The biome is read off it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Cover {
    pub alpha: f64,
    /// Share of the year above the growing base.
    pub season: f64,
    /// Woody closure the water and warmth allow, and what fire leaves.
    pub woody_potential: f64,
    pub closure: f64,
    pub ground_cover: f64,
    pub canopy_m: f64,
    pub npp_gc_m2_yr: f64,
    pub biomass_kg_c_m2: f64,
    /// Fires a year at the fixed point.
    pub fire_per_yr: f64,
    pub ice: bool,
}

/// The label read off a cover.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Biome {
    pub structure: String,
    pub earth: String,
    pub broad: String,
}

fn smooth(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Saturation vapour pressure of water, Pa (Bolton 1980).
fn e_sat(t_k: f64) -> f64 {
    let c = t_k - sse_core::sheet::T0C;
    611.2 * m::exp(17.67 * c / (c + 243.5)) // not-sheet: the Magnus fit's coefficients (Bolton 1980), not the triple point
}

impl CoverParams {
    /// Share of the year above the growing base, for a sinusoidal year
    /// between the coldest and warmest season.
    pub fn season(&self, x: &CoverInput) -> f64 {
        let amp = 0.5 * (x.t_warm_k - x.t_cold_k);
        let mid = 0.5 * (x.t_warm_k + x.t_cold_k);
        if amp <= 1e-9 {
            return if mid > self.grow_base_k { 1.0 } else { 0.0 };
        }
        let c = ((self.grow_base_k - mid) / amp).clamp(-1.0, 1.0);
        m::acos(c) / m::PI
    }

    /// The cover; `fire` false is the no-fire what-if.
    pub fn cover(&self, x: &CoverInput, fire: bool) -> Cover {
        let alpha = x.alpha();
        let season = self.season(x);
        let ice = x.ice_balance_m_yr > 0.0;
        if ice || season <= 0.0 {
            return Cover {
                alpha,
                season,
                ice,
                ..Cover::default()
            };
        }
        let warm = smooth((x.t_warm_k - self.treeline_k) / self.treeline_width_k + 0.5);
        let wet = smooth((alpha - self.alpha_open) / (self.alpha_closed - self.alpha_open));
        let woody_potential = wet * warm;
        let ground = smooth(alpha / self.alpha_grass);
        let light = self.lue_gc_mj * x.par_mj_m2_yr * season;
        let weather = (e_sat(x.t_warm_k) / e_sat(self.fire_ref_k)).min(1.0);
        let dry = 1.0 - alpha;
        let rate = |c: f64| {
            if !fire {
                return 0.0;
            }
            let fuel = light * ground * (1.0 - c);
            self.fire_rate_max * x.ignition_rel.max(0.0) * dry * weather * fuel
                / (fuel + self.fuel_half_gc)
        };
        // From an open start: shade lowers the fuel as trees close in.
        let mut c = 0.0;
        for _ in 0..40 {
            c = woody_potential * m::exp(-self.escape_yr * rate(c));
        }
        let green = c.max(ground);
        let npp = light * green;
        let g0 = sse_core::sheet::G0;
        let g = if x.gravity_m_s2 > 0.0 {
            x.gravity_m_s2
        } else {
            g0
        };
        let canopy = if c > 0.0 {
            self.canopy_m
                * (g0 / g)
                * (alpha / self.alpha_closed).min(1.0)
                * smooth(season / 0.5)
                * warm
        } else {
            0.0
        };
        let biomass =
            npp * (c * self.wood_residence_yr + (1.0 - c) * self.herb_residence_yr) / 1000.0;
        Cover {
            alpha,
            season,
            woody_potential,
            closure: c,
            ground_cover: ground,
            canopy_m: canopy,
            npp_gc_m2_yr: npp,
            biomass_kg_c_m2: biomass,
            fire_per_yr: rate(c),
            ice,
        }
    }

    /// The label: the first atlas row whose ranges hold.
    pub fn biome(&self, cv: &Cover, x: &CoverInput) -> Biome {
        let within = |v: f64, lo: Option<f64>, hi: Option<f64>| {
            lo.is_none_or(|l| v >= l) && hi.is_none_or(|h| v < h)
        };
        let hit = self.atlas.iter().find(|r| {
            r.ice.is_none_or(|i| i == cv.ice)
                && within(cv.closure, r.closure_min, r.closure_max)
                && within(cv.canopy_m, r.height_min_m, r.height_max_m)
                && within(cv.npp_gc_m2_yr, r.npp_min_gc, r.npp_max_gc)
                && within(x.t_cold_k, r.t_cold_min_k, r.t_cold_max_k)
                && within(x.t_warm_k, r.t_warm_min_k, r.t_warm_max_k)
        });
        match hit {
            Some(r) => Biome {
                structure: r.structure.clone(),
                earth: r.earth.clone(),
                broad: r.broad.clone(),
            },
            None => Biome {
                structure: "unlabelled".into(),
                earth: String::new(),
                broad: String::new(),
            },
        }
    }
}
