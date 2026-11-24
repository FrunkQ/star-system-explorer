//! The weather contract (weather design section 6): what every weather block
//! publishes and every reader uses. The rule-based block ([`crate::RuleBlock`])
//! fills it from rules and keyed draws; a later simulation block fills the same
//! structures. Nothing outside this module may depend on how a block works.
//!
//! Why saves do not break: a save never stores a [`WeatherField`]. It stores
//! the block's [`BlockId`] in the manifest, authored weather events, and the
//! short records of systems that were seen and named ([`Pin`]). Everything
//! else is regenerated on demand by whichever block the campaign names.

use crate::conditions::Conditions;
use serde::{Deserialize, Serialize};
use sse_surface::atlas::TileKey;

/// Which block made a field, and how (design section 6, `provenance`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct BlockId {
    /// Stable block name, e.g. `sse.weather.rules`.
    pub id: String,
    pub version: String,
    /// `rule` (formations placed by rule) or `simulated` (grown by a model).
    pub level: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub block: BlockId,
    /// `drawn` (keyed draws from climate statistics) or `calculated`.
    pub evidence: String,
}

/// A date on the world: days since the campaign epoch. The season comes from
/// the orbit ([`Conditions::year_days`]); weather systems are drawn per day.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Date {
    pub day: f64,
}

/// The height layers every cell carries, bottom up.
pub const LAYERS: [&str; 3] = ["low", "middle", "high"];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Aerosol {
    /// `ash`, `smoke`, `dust`, `haze`, `smog` or `spray`.
    pub kind: String,
    pub optical_depth: f64,
    /// Linear sRGB, 0 to 1.
    pub colour: [f64; 3],
    /// Typical particle radius, µm (an aggregate's equal-volume radius);
    /// absent where no measured size is held (visual polish K-c).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub particle_radius_um: f64,
}

/// One height layer of one cell.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Layer {
    /// Fraction of the cell covered by cloud in this layer, 0 to 1.
    pub cloud_cover: f64,
    /// `none`, `cumulus`, `cumulonimbus`, `stratus`, `stratocumulus`,
    /// `nimbostratus`, `altostratus`, `cirrus`, `deck` (a world-wide deck),
    /// `haze` or `frost`.
    pub cloud_type: String,
    /// What the cloud is made of (`water`, `methane`, `sulphuric acid`, ...),
    /// or empty where there is none.
    pub condensate: String,
    pub optical_depth: f64,
    /// Eastward and northward wind, m/s.
    pub wind_m_s: [f64; 2],
    /// Turbulence, 0 (smooth) to 1 (severe).
    pub turbulence: f64,
    /// Typical cloud particle effective radius, µm; absent where no
    /// measured size is held (visual polish K-c).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub particle_radius_um: f64,
}

/// One coarse cell on the surface atlas's cube sphere.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub layers: Vec<Layer>,
    /// `none`, `rain`, `drizzle`, `snow`, `virga` (falls but evaporates
    /// before the ground) or the condensate's own name (`methane rain`).
    pub precip_kind: String,
    /// Precipitation reaching the ground, mm per day.
    pub precip_mm_day: f64,
    /// Departure of near-surface temperature from the climate mean, K.
    pub temp_offset_k: f64,
    /// Lightning flashes per km² per year.
    pub lightning_km2_yr: f64,
    pub aerosols: Vec<Aerosol>,
    /// Mean wind speed 10 m above the ground or sea, m/s, over the band's
    /// storms and calms (the climate's 10 m wind, W-wind10); what a glint's
    /// slope spread reads (Cox and Munk 1954). Absent with no surface.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub wind10_m_s: f64,
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

/// The field over one atlas tile at one date: `n` × `n` cells in row order.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FieldTile {
    pub key: TileKey,
    pub n: u32,
    pub cells: Vec<Cell>,
}

/// A nameable thing: a cyclone, an oval, a plume, a polar feature.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WeatherSystem {
    /// Stable within a block and a world: the same storm has the same id on
    /// every device and at every date of its life.
    pub id: String,
    /// `cyclone`, `tropical-cyclone`, `oval`, `dark-spot`, `outbreak`,
    /// `polar-vortex`, `cyclone-ring`, `polygon-jet`, `diffuse-cap`, `plume`,
    /// `dust-storm`, `substellar-storm`, `gyre`.
    pub kind: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// Positions along its life, oldest first: (day, lat, lon).
    pub track: Vec<[f64; 3]>,
    pub size_km: f64,
    /// Kind-specific strength: peak wind m/s for storms, height km for plumes.
    pub strength: f64,
    /// Days (since the campaign epoch); `f64::MAX` stands for "permanent".
    pub born: f64,
    pub ends: f64,
    /// Polygon sides, cyclones in a ring, or 0.
    #[serde(default)]
    pub count: u32,
    /// Why it is here, in a sentence a player could read.
    pub why: String,
    /// `drawn` (from statistics), `rule` (placed by a stated condition) or
    /// `pinned` (a recorded system honoured from a save).
    pub evidence: String,
}

/// The short record of a system someone has seen and named. Saves keep
/// these; every block must honour them for the system's natural life.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pin {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub system: WeatherSystem,
}

/// One latitude band of the annual climate (annual means).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Band {
    /// Band centre, degrees.
    pub lat_deg: f64,
    pub cloud_cover: f64,
    pub precip_m_yr: f64,
    /// Eastward near-surface wind, m/s.
    pub wind_m_s: f64,
}

/// The averages slow physics reads (erosion, climate, life). Both kinds of
/// block must publish them, and must agree within [`TOLERANCE`] so a swap
/// leaves terrain, climate and life stable (design section 6, last point).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClimateAverages {
    pub regime: String,
    pub bands: Vec<Band>,
    pub cloud_cover_mean: f64,
    pub precip_mean_m_yr: f64,
    /// What forms the clouds, or empty for a cloudless world.
    pub condensate: String,
    /// The erosion term's own form (S3 `wear.rain_cells`): rain alternates
    /// wet and dry this many times pole to equator.
    pub rain_cells: u32,
    /// Fraction of the absorbed sunlight that reaches the ground.
    pub surface_sun_fraction: f64,
}

/// How far a replacement block's averages may stray from the rule block's.
pub const TOLERANCE: Tolerance = Tolerance {
    cloud_cover: 0.10,
    precip_rel: 0.25,
};

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub struct Tolerance {
    /// Absolute, as a fraction of sky.
    pub cloud_cover: f64,
    /// Relative, on the mean precipitation.
    pub precip_rel: f64,
}

/// A place-and-date weather report for the table (design section 5).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Report {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub day: f64,
    /// Plain lines: "Wind 40 km/h from the west", "Visibility 2 km".
    pub lines: Vec<String>,
    pub wind_km_h: f64,
    /// Direction the wind comes from, degrees clockwise from north.
    pub wind_from_deg: f64,
    pub visibility_km: f64,
    pub cloud_cover: f64,
    pub precip_mm_day: f64,
    /// Systems within reach, nearest first, with their distance in km.
    pub nearby: Vec<(String, f64)>,
    pub provenance: Provenance,
}

/// The one contract. A block is pure: the same conditions, date and pins give
/// the same answer on every device.
pub trait WeatherBlock {
    fn id(&self) -> BlockId;
    /// Annual means for slow physics.
    fn climate(&self, c: &Conditions) -> ClimateAverages;
    /// The field over one atlas tile, `n` × `n` cells.
    fn field(&self, c: &Conditions, date: Date, key: TileKey, n: u32, pins: &[Pin]) -> FieldTile;
    /// The nameable systems alive on this date.
    fn systems(&self, c: &Conditions, date: Date, pins: &[Pin]) -> Vec<WeatherSystem>;
    fn provenance(&self) -> Provenance;
}
