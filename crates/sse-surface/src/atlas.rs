//! S1: the surface atlas (surface design section 1; research report section 3).
//!
//! A world's surface at an epoch is a cube-sphere quadtree of tiles: six faces,
//! each split into 2^level × 2^level tiles of `tile_samples` × `tile_samples`
//! samples. A tile is a pure function of the world (its seed, address and
//! regime at the epoch) and its own key, so any tile builds alone, on any
//! device, with the same hash. Heights are band-limited per level: a level
//! holds only the detail octaves its sample spacing can carry, so a child
//! tile averaged 2 × 2 matches its parent within [`Atlas::tolerance_m`].
//!
//! The height source is the S0 regime sketch plus keyed detail octaves until
//! the plate history (S2) and wearing down (S3) replace the sketch. Layers that
//! later stages fill are listed in each tile's `pending`.
//!
//! Address: `<body>/shell:surface@<epoch Myr>/tile:<face>.<level>.<x>.<y>`
//! (the design's `tile:<face>/<level>/<x>/<y>`, written with dots because an
//! address key cannot hold a slash).

use crate::sketch::{SketchModel, SurveyState};
use crate::{SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use sse_core::{Address, ContentHash};
use std::collections::BTreeMap;

pub const ATLAS_VERSION: &str = "surface-atlas-v1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AtlasSettings {
    pub tile_samples: u32,
    pub fact_floor_m: f64,
    pub site_floor_m: f64,
    /// Longest detail octave, radians of arc.
    pub detail_wavelength_rad: f64,
    pub detail_hurst: f64,
    /// Amplitude of the longest detail octave, as a fraction of the sketch relief.
    pub detail_amplitude: f64,
    /// An octave is carried at a level when its wavelength is at least this many sample spacings.
    pub nyquist: f64,
    pub cache_tiles: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct TileKey {
    pub face: u8,
    pub level: u8,
    pub x: u32,
    pub y: u32,
}

impl TileKey {
    pub fn new(face: u8, level: u8, x: u32, y: u32) -> Option<Self> {
        let n = 1u64 << level.min(40);
        (face < 6 && level <= 30 && (x as u64) < n && (y as u64) < n).then_some(TileKey {
            face,
            level,
            x,
            y,
        })
    }
    pub fn parent(&self) -> Option<Self> {
        (self.level > 0).then(|| TileKey {
            face: self.face,
            level: self.level - 1,
            x: self.x / 2,
            y: self.y / 2,
        })
    }
    /// Children in row order: (0,0), (1,0), (0,1), (1,1).
    pub fn children(&self) -> [Self; 4] {
        let c = |dx, dy| TileKey {
            face: self.face,
            level: self.level + 1,
            x: 2 * self.x + dx,
            y: 2 * self.y + dy,
        };
        [c(0, 0), c(1, 0), c(0, 1), c(1, 1)]
    }
    pub fn text(&self) -> String {
        format!("{}.{}.{}.{}", self.face, self.level, self.x, self.y)
    }
}

/// A place resolved to the finer site floor (landing zone, base, walked area).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Site {
    pub label: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub radius_km: f64,
}

pub const MATERIALS: [&str; 9] = [
    "basalt",
    "granite",
    "regolith",
    "ice",
    "sediment",
    "lava",
    "cloud",
    "organic-sediment",
    "carbon",
];
pub const COVERS: [&str; 5] = ["none", "water", "magma", "hydrocarbon", "ice"];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub address: Address,
    pub key: TileKey,
    pub epoch_myr: i64,
    pub samples: u32,
    /// Sample spacing at the face centre, metres.
    pub spacing_m: f64,
    /// Detail octaves this level carries.
    pub octaves: u32,
    /// Height against the reference radius, decimetres; rows of `samples`.
    pub elevation_dm: Vec<i32>,
    /// Index into [`MATERIALS`].
    pub material: Vec<u8>,
    /// Index into [`COVERS`].
    pub cover: Vec<u8>,
    /// Tenths of a degree.
    pub slope_ddeg: Vec<u16>,
    /// Time since the surface here was last renewed, Myr (provisional until S2 and S4).
    pub surface_age_myr: Vec<u32>,
    /// Thousandths.
    pub albedo: Vec<u16>,
    /// Feature tags present in the tile (filled from S4 and S7).
    pub tags: Vec<String>,
    /// Channels (S3): 0 off the river network, else 1 + 8 × log2(drainage in
    /// cells), capped at 255. Empty before wearing down.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flow: Vec<u8>,
    /// S7: moisture, 0 to 255. Empty without S7's layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wetness: Vec<u8>,
    /// S7: index into [`crate::cover::DEPOSITS`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deposit: Vec<u8>,
    /// S7: 0 none, else 1 + index into the layers' built kinds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structure: Vec<u8>,
    /// S7: colour from material and cover, 0xRRGGBB.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colour: Vec<u32>,
    /// Layers a later stage fills.
    pub pending: Vec<String>,
    pub hash: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "refused", rename_all = "kebab-case")]
pub enum Refusal {
    /// The survey state does not disclose this level.
    Withheld {
        state: SurveyState,
        max_level: Option<u8>,
    },
    /// Finer than the fact floor here: decoration, made on the GPU.
    BelowFactFloor {
        level: u8,
        floor_level: u8,
    },
    BadKey,
}

/// The surface atlas of one world at one epoch.
#[derive(Clone, Debug)]
pub struct Atlas {
    pub address: Address,
    pub epoch_myr: i64,
    pub regime: String,
    pub radius_m: f64,
    pub sites: Vec<Site>,
    settings: AtlasSettings,
    model: SketchModel,
    detail_key: u64,
    /// Myr since the current regime began, at the epoch.
    regime_age_myr: f64,
    material_land: u8,
    plates: Option<PlateSurface>,
    impacts: Option<Box<ImpactLayer>>,
    regime_layer: Option<Box<crate::regimes::RegimeSurface>>,
    layers: Option<Box<crate::cover::Layers>>,
}

/// S4's craters, plains and shields on an atlas.
#[derive(Clone, Debug)]
struct ImpactLayer {
    features: crate::impacts::Features,
    field: crate::impacts::CraterField,
}

/// The plate history's surface at the atlas epoch, on the plate grid.
#[derive(Clone)]
struct PlateSurface {
    grid: crate::plates::Grid,
    /// Metres from the datum (sea level when the world has oceans).
    heights: Vec<f64>,
    continental: Vec<bool>,
    age_myr: Vec<f64>,
    oceans: bool,
    /// Worlds whose plates never moved: one land material everywhere.
    land_material: Option<u8>,
    /// From S3: per cell, (river, lake, thick sediment, drainage in cells).
    flow: Option<Vec<(bool, bool, bool, f64)>>,
}

impl std::fmt::Debug for PlateSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlateSurface({} cells)", self.heights.len())
    }
}

impl PlateSurface {
    /// Bilinear height across the plate grid (across face edges too), and the
    /// nearest cell's crust.
    fn at(&self, d: [f64; 3]) -> (f64, usize) {
        let (f, u, v) = face_of(d);
        let n = self.grid.n as f64;
        let ci = (u + 1.0) * 0.5 * n - 0.5;
        let cj = (v + 1.0) * 0.5 * n - 0.5;
        let (i0, j0) = (m::floor(ci), m::floor(cj));
        let (fx, fy) = (ci - i0, cj - j0);
        let cell = |i: f64, j: f64| {
            let uu = -1.0 + 2.0 * (i + 0.5) / n;
            let vv = -1.0 + 2.0 * (j + 0.5) / n;
            self.grid.cell_at(direction(f, uu, vv))
        };
        let (a, b, c, e) = (
            cell(i0, j0),
            cell(i0 + 1.0, j0),
            cell(i0, j0 + 1.0),
            cell(i0 + 1.0, j0 + 1.0),
        );
        let h0 = self.heights[a] + (self.heights[b] - self.heights[a]) * fx;
        let h1 = self.heights[c] + (self.heights[e] - self.heights[c]) * fx;
        (h0 + (h1 - h0) * fy, self.grid.cell_at(d))
    }
}

const FACES: [([f64; 3], [f64; 3], [f64; 3]); 6] = [
    ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
    ([-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]),
    ([0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
];

/// Unit direction of face coordinates (u, v) in [-1, 1], with the tangent
/// warp that evens out sample spacing across a face.
pub fn direction(face: u8, u: f64, v: f64) -> [f64; 3] {
    let (n, r, t) = FACES[face as usize];
    let (a, b) = (m::tan(u * m::PI / 4.0), m::tan(v * m::PI / 4.0));
    let d = [
        n[0] + a * r[0] + b * t[0],
        n[1] + a * r[1] + b * t[1],
        n[2] + a * r[2] + b * t[2],
    ];
    let l = m::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
    [d[0] / l, d[1] / l, d[2] / l]
}

/// The face and (u, v) a direction falls on.
pub fn face_of(d: [f64; 3]) -> (u8, f64, f64) {
    let (f, a, b) = face_tangents(d);
    (f, m::atan(a) * 4.0 / m::PI, m::atan(b) * 4.0 / m::PI)
}

/// The face a direction falls on and its two tangent coordinates (before
/// the warp `face_of` undoes).
pub fn face_tangents(d: [f64; 3]) -> (u8, f64, f64) {
    let mut best = 0u8;
    let mut bv = f64::NEG_INFINITY;
    for (i, f) in FACES.iter().enumerate() {
        let x = crate::sketch::dot(d, f.0);
        if x > bv {
            bv = x;
            best = i as u8;
        }
    }
    let (n, r, t) = FACES[best as usize];
    let k = crate::sketch::dot(d, n);
    (
        best,
        crate::sketch::dot(d, r) / k,
        crate::sketch::dot(d, t) / k,
    )
}

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn lattice(key: u64, oct: u32, i: i64, j: i64, k: i64) -> f64 {
    let h = mix(key ^ mix((oct as u64) ^ mix((i as u64) ^ mix((j as u64) ^ mix(k as u64)))));
    // Top 53 bits to [-1, 1).
    ((h >> 11) as f64) / ((1u64 << 52) as f64) - 1.0
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Value noise on a 3D lattice, smooth interpolation, in [-1, 1].
fn value_noise(key: u64, oct: u32, p: [f64; 3]) -> f64 {
    let f = [m::floor(p[0]), m::floor(p[1]), m::floor(p[2])];
    let t = [
        smooth(p[0] - f[0]),
        smooth(p[1] - f[1]),
        smooth(p[2] - f[2]),
    ];
    let (i, j, k) = (f[0] as i64, f[1] as i64, f[2] as i64);
    let c = |a: i64, b: i64, d: i64| lattice(key, oct, i + a, j + b, k + d);
    let x00 = c(0, 0, 0) + (c(1, 0, 0) - c(0, 0, 0)) * t[0];
    let x10 = c(0, 1, 0) + (c(1, 1, 0) - c(0, 1, 0)) * t[0];
    let x01 = c(0, 0, 1) + (c(1, 0, 1) - c(0, 0, 1)) * t[0];
    let x11 = c(0, 1, 1) + (c(1, 1, 1) - c(0, 1, 1)) * t[0];
    let y0 = x00 + (x10 - x00) * t[1];
    let y1 = x01 + (x11 - x01) * t[1];
    y0 + (y1 - y0) * t[2]
}

impl Atlas {
    /// The atlas of a world at `epoch_gyr` (Gyr since it formed; clamped to its life).
    pub fn new(
        s: &SurfaceSettings,
        env: &SurfaceEnvelope,
        seed: &Seed,
        epoch_gyr: f64,
        sites: Vec<Site>,
    ) -> Self {
        let end = env.timeline.last().map(|x| x.to_gyr).unwrap_or(0.0);
        let t = epoch_gyr.clamp(0.0, end);
        let span = env
            .timeline
            .iter()
            .find(|x| t >= x.from_gyr && t <= x.to_gyr)
            .or(env.timeline.last());
        let regime = span
            .map(|x| x.regime.clone())
            .unwrap_or_else(|| env.regime_now.clone());
        let regime_age_myr = span
            .map(|x| (t - x.from_gyr) * 1000.0)
            .unwrap_or(t * 1000.0);
        let model = SketchModel::new(s, env, &regime, seed);
        let epoch_myr = m::round(t * 1000.0) as i64;
        let address = env
            .address
            .child_key("shell", &format!("sketch@{epoch_myr}"))
            .expect("valid shell key");
        let d = Draws::new(
            *seed,
            env.address.clone(),
            Stream::new("surface.atlas.v1").expect("valid"),
        );
        let material_land = match (regime.as_str(), model.style.as_str()) {
            ("carbon-rich", _) => 8,
            (_, "plates") => 1,
            (_, "lid") => 2,
            (_, "plains") | (_, "smooth") => 0,
            (_, "ice") => 3,
            (_, "dunes") => 7,
            (_, "lumpy") => 2,
            (_, "bands") => 6,
            (_, "ocean") => 0,
            _ => 2,
        };
        Atlas {
            address,
            epoch_myr,
            regime,
            radius_m: env.reference_radius_m,
            sites,
            settings: s.atlas.clone(),
            model,
            detail_key: d.u64(0),
            regime_age_myr,
            material_land,
            plates: None,
            impacts: None,
            regime_layer: None,
            layers: None,
        }
    }

    /// The surveyed atlas: heights from the plate history (S2) instead of the
    /// regime sketch. Its address is `shell:surface@`; a sketch atlas is
    /// `shell:sketch@`, so the two never share tile addresses. A world whose
    /// plates never moved keeps the sketch atlas.
    pub fn from_plates(
        s: &SurfaceSettings,
        env: &SurfaceEnvelope,
        seed: &Seed,
        history: &crate::plates::PlateHistory,
        oceans: f64,
        sites: Vec<Site>,
    ) -> Self {
        let mut a = Atlas::new(s, env, seed, history.epoch_myr / 1000.0, sites);
        if history.steps == 0 {
            // No plates ever moved (stagnant lid and the other regimes until
            // S6): the regime sketch stays the height source.
            return a;
        }
        a.address = env
            .address
            .child_key("shell", &format!("surface@{}", a.epoch_myr))
            .expect("valid shell key");
        let raw = crate::plates::cell_heights(s, history, env.gravity_m_s2);
        let sl = crate::plates::sea_level(&raw, env.reference_radius_m, oceans);
        let heights = raw.iter().map(|h| h - sl.unwrap_or(0.0)).collect();
        a.plates = Some(PlateSurface {
            grid: crate::plates::grid_for(history),
            heights,
            continental: history.cells.iter().map(|c| c.continental).collect(),
            age_myr: history
                .cells
                .iter()
                .map(|c| {
                    let last = c
                        .orogeny
                        .last()
                        .map(|o| o.0)
                        .unwrap_or(c.born_myr)
                        .max(c.born_myr);
                    (history.epoch_myr - last).max(0.0)
                })
                .collect(),
            oceans: sl.is_some(),
            land_material: None,
            flow: None,
        });
        a
    }

    /// The surveyed atlas after wearing down (S3): heights from the worn cell
    /// grid, rivers, lakes and sediment from its drainage. Works for every
    /// solid world: plates where they moved, the regime sketch elsewhere.
    pub fn from_worn(
        s: &SurfaceSettings,
        env: &SurfaceEnvelope,
        seed: &Seed,
        history: &crate::plates::PlateHistory,
        worn: &crate::wear::Wear,
        sites: Vec<Site>,
    ) -> Self {
        let mut a = Atlas::new(s, env, seed, worn.epoch_myr / 1000.0, sites);
        a.address = env
            .address
            .child_key("shell", &format!("surface@{}", a.epoch_myr))
            .expect("valid shell key");
        let moved = history.steps > 0;
        let nc = worn.heights.len();
        let cell = 4.0 * m::PI * env.reference_radius_m * env.reference_radius_m / nc as f64;
        a.plates = Some(PlateSurface {
            grid: crate::plates::grid_for(history),
            heights: worn.heights.clone(),
            continental: if moved {
                history.cells.iter().map(|c| c.continental).collect()
            } else {
                vec![false; nc]
            },
            age_myr: if moved {
                history
                    .cells
                    .iter()
                    .map(|c| {
                        let last = c
                            .orogeny
                            .last()
                            .map(|o| o.0)
                            .unwrap_or(c.born_myr)
                            .max(c.born_myr);
                        (history.epoch_myr - last).max(0.0)
                    })
                    .collect()
            } else {
                vec![a.regime_age_myr; nc]
            },
            oceans: worn.sea_level_raw_m.is_some(),
            land_material: (!moved).then_some(a.material_land),
            flow: Some(
                (0..nc)
                    .map(|c| {
                        (
                            worn.river[c],
                            worn.lake[c],
                            worn.sediment_m[c] > 10.0,
                            worn.area_m2[c] / cell,
                        )
                    })
                    .collect(),
            ),
        });
        a
    }

    /// Add S4's impacts and volcanism: crater and shield relief to the fact
    /// floor (each feature only at levels whose spacing can carry it), lava
    /// plains as basalt with their own surface age, and feature tags.
    pub fn with_impacts(
        mut self,
        features: crate::impacts::Features,
        field: crate::impacts::CraterField,
    ) -> Self {
        self.impacts = Some(Box::new(ImpactLayer { features, field }));
        self
    }

    /// Add S6's regime landforms (and a giant's cloud deck): relief band-limited
    /// by each landform's width, cover and material inside regions, cloud
    /// albedo and cloud-top heights, tags.
    pub fn with_regime(mut self, rs: crate::regimes::RegimeSurface) -> Self {
        self.regime_layer = Some(Box::new(rs));
        self
    }

    /// Add S7's layers (cover, deposits, built layer, GM edits), then settle
    /// the GM's height pins: each becomes a smooth bump sized so that the
    /// fact-floor height at the pin equals the pinned height, measured on the
    /// terrain as it stands with every earlier pin in place.
    pub fn with_layers(mut self, mut l: crate::cover::Layers) -> Self {
        let pins = std::mem::take(&mut l.pins);
        let max = l.max_relief_m;
        self.layers = Some(Box::new(l));
        let level = self.level_for(self.settings.fact_floor_m);
        let rk = self.radius_m / 1000.0;
        let min_w = self.settings.nyquist * self.spacing_m(level) / 1000.0;
        for p in pins {
            let before = self.height_at(p.dir, level);
            let delta = p.height_m - before;
            let (la, lo) = (
                m::asin(p.dir[2].clamp(-1.0, 1.0)).to_degrees(),
                m::atan2(p.dir[1], p.dir[0]).to_degrees(),
            );
            let ly = self.layers.as_mut().expect("just set");
            ly.landforms.push(crate::regimes::Landform {
                id: p.id.clone(),
                kind: "pin".into(),
                dir: p.dir,
                axis: None,
                lat_deg: la,
                lon_deg: lo,
                size_km: 2.0 * p.radius_km,
                width_km: (2.0 * p.radius_km).max(min_w),
                height_m: delta,
                formed_myr: p.at_myr,
                cover: None,
                material: None,
                cause: format!(
                    "pinned by the GM at {:.0} Myr to {:.0} m (the ground stood at {:.0} m)",
                    p.at_myr, p.height_m, before
                ),
            });
            if let Some(a) = ly.applied.iter_mut().find(|a| a.id == p.id) {
                a.note = format!(
                    "{}; the ground stood at {:.0} m, raised by {:.0} m over {:.1} km",
                    a.note, before, delta, p.radius_km
                );
                if delta.abs() > max {
                    a.impossible = Some(format!(
                        "moving the ground {:.0} m by hand exceeds the {:.0} m rock can hold up here",
                        delta, max
                    ));
                }
            }
            let _ = rk;
        }
        self
    }

    pub fn layers(&self) -> Option<&crate::cover::Layers> {
        self.layers.as_deref()
    }

    /// Thick-sediment flags per plate cell (S3), when the atlas was worn.
    pub(crate) fn sediment_cells(&self) -> Option<Vec<bool>> {
        self.plates
            .as_ref()
            .and_then(|p| p.flow.as_ref())
            .map(|f| f.iter().map(|x| x.2).collect())
    }

    /// The height a tile at `level` gives at a direction, m (the same sum
    /// `build` makes for a sample there).
    pub fn height_at(&self, d: [f64; 3], level: u8) -> f64 {
        self.heights(level).at(d)
    }

    /// A reader of heights at `level` that keeps each tile's features and
    /// landforms once gathered, for many reads close together (a path).
    pub fn heights(&self, level: u8) -> Heights<'_> {
        Heights {
            atlas: self,
            level,
            tiles: vec![],
        }
    }

    fn tile_key_at(&self, d: [f64; 3], level: u8) -> TileKey {
        let (f, u, v) = face_of(d);
        let n = 1u64 << level;
        let ix = (((u + 1.0) * 0.5 * n as f64) as i64).clamp(0, n as i64 - 1) as u32;
        let iy = (((v + 1.0) * 0.5 * n as f64) as i64).clamp(0, n as i64 - 1) as u32;
        TileKey::new(f, level, ix, iy).expect("key in range")
    }

    /// Tile centre and half its diagonal (angle, with margin for the warp).
    fn centre_half(&self, key: TileKey) -> ([f64; 3], f64) {
        let n = (1u64 << key.level) as f64;
        let centre = direction(
            key.face,
            -1.0 + 2.0 * (key.x as f64 + 0.5) / n,
            -1.0 + 2.0 * (key.y as f64 + 0.5) / n,
        );
        let half =
            1.5 * self.spacing_m(key.level) * self.settings.tile_samples as f64 / self.radius_m;
        (centre, half)
    }

    pub fn regime_surface(&self) -> Option<&crate::regimes::RegimeSurface> {
        self.regime_layer.as_deref()
    }

    /// Tolerance terms from regime landforms and cloud bands, m.
    fn regime_tolerance_m(&self, child_level: u8) -> f64 {
        if self.regime_layer.is_none() && self.layers.is_none() {
            return 0.0;
        }
        let parent = child_level.saturating_sub(1);
        let (sp_p, min_p, min_c) = (
            self.spacing_m(parent),
            self.min_feature_km(parent),
            self.min_feature_km(child_level),
        );
        let mut added: f64 = 0.0;
        let mut carried: f64 = 0.0;
        for l in self.all_landforms() {
            let k = crate::regimes::landform_curvature(&l.kind);
            if k == 0.0 {
                continue;
            }
            if l.width_km >= min_c && l.width_km < min_p {
                added = added.max(1.2 * l.height_m.abs());
            }
            if l.width_km >= min_p {
                let hw = 0.5 * l.width_km * 1000.0;
                carried = carried.max(k * l.height_m.abs() * (sp_p / hw) * (sp_p / hw) / 16.0);
            }
        }
        // Authored craters (S7), as S4's craters.
        let mut cr_added: f64 = 0.0;
        let mut cr_carried: f64 = 0.0;
        if let Some(ly) = &self.layers {
            let s = &ly.impact_settings;
            let g = ly.gravity;
            for (d, relief) in &ly.crater_relief {
                if *d >= min_c && *d < min_p {
                    cr_added = cr_added.max(*relief);
                }
                if *d >= min_p {
                    cr_carried =
                        cr_carried.max(crate::impacts::crater_curvature_err(s, g, *d, sp_p));
                }
            }
        }
        added = added.max(cr_added);
        carried = carried.max(cr_carried);
        let clouds = self
            .regime_layer
            .as_ref()
            .and_then(|r| r.clouds.as_ref())
            .map_or(0.0, |c| {
                let amp = 0.5 * c.scale_height_km * 1000.0;
                let bw = c.narrowest_band_km(self.radius_m / 1000.0) * 1000.0;
                let x = m::PI * sp_p / bw.max(1.0);
                amp * (0.25 * x * x).min(1.0)
            });
        2.0 * (added + carried) + clouds
    }

    pub fn features(&self) -> Option<&crate::impacts::Features> {
        self.impacts.as_ref().map(|x| &x.features)
    }

    /// Smallest feature (crater diameter, shield width) a level carries, km.
    pub fn min_feature_km(&self, level: u8) -> f64 {
        self.settings.nyquist * self.spacing_m(level) / 1000.0
    }

    /// Size bands of small craters a level carries.
    fn bands_at(&self, level: u8) -> u32 {
        let Some(ip) = &self.impacts else { return 0 };
        if ip.features.surface_from_myr.is_none() {
            return 0;
        }
        let min = self.min_feature_km(level);
        let mut k = 0u32;
        while k < 40 && ip.field.band(ip.features.listed_d_km, k).1 > min {
            k += 1;
        }
        k
    }

    /// Tolerance terms from craters and shields between a parent and child level, m.
    fn impact_tolerance_m(&self, child_level: u8) -> f64 {
        let Some(ip) = &self.impacts else { return 0.0 };
        if ip.features.surface_from_myr.is_none() {
            return 0.0;
        }
        let parent = child_level.saturating_sub(1);
        let (sp_p, min_p, min_c) = (
            self.spacing_m(parent),
            self.min_feature_km(parent),
            self.min_feature_km(child_level),
        );
        let s = ip.field.settings();
        let g = ip.field.gravity();
        let relief = |d: f64| {
            let (_, a, b, c) = crate::impacts::crater_shape(s, g, d);
            a + b + c
        };
        // Craters entering at the child level (at most min_p across), fresh.
        let added = if min_c < min_p { relief(min_p) } else { 0.0 };
        // Curvature of craters both carry, worst at the smallest of each shape.
        let gs = s.g_ref_m_s2 / g.max(1e-6);
        let carried = [
            min_p,
            s.simple_complex_km * gs,
            s.peak_ring_km * gs,
            s.multi_ring_km * gs,
        ]
        .iter()
        .filter(|d| **d >= min_p)
        .map(|d| crate::impacts::crater_curvature_err(s, g, *d, sp_p))
        .fold(0.0, f64::max);
        let mut sh_added: f64 = 0.0;
        let mut sh_carried: f64 = 0.0;
        for sh in &ip.features.shields {
            let w = 2.0 * sh.radius_km;
            if w >= min_c && w < min_p {
                sh_added = sh_added.max(sh.height_m);
            }
            if w >= min_p {
                let rr = sh.radius_km * 1000.0;
                // |f''| up to 8 H / R² for the shield, plus the caldera's 0.03 H × 2 / 0.05².
                let k = 8.0 * sh.height_m + 24.0 * sh.height_m;
                sh_carried = sh_carried.max(k * (sp_p / rr) * (sp_p / rr) / 16.0);
            }
        }
        // Twice each, for overlapping features.
        2.0 * (added + carried + sh_added + sh_carried)
    }

    /// Whether heights come from the plate history.
    pub fn has_plates(&self) -> bool {
        self.plates.is_some()
    }

    pub fn relief_m(&self) -> f64 {
        self.model.relief_m()
    }

    pub fn settings(&self) -> &AtlasSettings {
        &self.settings
    }

    /// Sample spacing at the face centre at a level, metres.
    pub fn spacing_m(&self, level: u8) -> f64 {
        self.radius_m * (m::PI / 2.0) / (self.settings.tile_samples as f64 * (1u64 << level) as f64)
    }

    /// The coarsest level whose spacing is at or below `floor_m`.
    pub fn level_for(&self, floor_m: f64) -> u8 {
        let mut l = 0u8;
        while l < 30 && self.spacing_m(l) > floor_m {
            l += 1;
        }
        l
    }

    fn octave(&self, k: u32) -> (f64, f64) {
        let a = &self.settings;
        let lambda = a.detail_wavelength_rad / (1u64 << k) as f64;
        let amp =
            a.detail_amplitude * self.model.relief_m() * m::pow(2.0, -(k as f64) * a.detail_hurst);
        (lambda, amp)
    }

    /// Detail octaves carried at a level.
    pub fn octaves_at(&self, level: u8) -> u32 {
        let sp = self.spacing_m(level) / self.radius_m.max(1.0);
        let mut k = 0u32;
        while k < 40 && self.octave(k).0 >= self.settings.nyquist * sp {
            k += 1;
        }
        k
    }

    /// Stated tolerance for a child tile averaged 2 × 2 against its parent,
    /// metres: the full amplitude of the octaves the child adds; the
    /// curvature of each octave both carry over one parent cell (about
    /// (π s / λ)² / 4 of its amplitude, s the parent spacing); the same for the
    /// sketch's smallest bumps (about 0.077 rad across); and rounding.
    pub fn tolerance_m(&self, child_level: u8) -> f64 {
        let parent = child_level.saturating_sub(1);
        let (lo, hi) = (self.octaves_at(parent), self.octaves_at(child_level));
        let sp = self.spacing_m(parent) / self.radius_m.max(1.0);
        let added: f64 = (lo..hi).map(|k| self.octave(k).1).sum();
        let carried: f64 = (0..lo)
            .map(|k| {
                let (lambda, amp) = self.octave(k);
                let x = m::PI * sp / lambda;
                amp * (0.25 * x * x).min(1.0)
            })
            .sum();
        // Smallest smooth feature: the sketch's bumps, or two plate cells.
        let scale = match &self.plates {
            Some(ps) => 2.0 * (m::PI / 2.0) / ps.grid.n as f64,
            None => 0.077,
        };
        let x = m::PI * sp / scale;
        let sketch = self.model.relief_m() * (0.25 * x * x).min(1.0);
        added
            + carried
            + sketch
            + 0.1
            + self.impact_tolerance_m(child_level)
            + self.regime_tolerance_m(child_level)
    }

    /// Height at a direction carrying `octaves` detail octaves, metres.
    pub fn height(&self, d: [f64; 3], octaves: u32) -> (f64, crate::sketch::Point) {
        let mut p = self.model.at(d);
        if let Some(ps) = &self.plates {
            let (h, _) = ps.at(d);
            let [a_lo, a_hi] = self.model.def().albedo;
            p.height_m = h;
            p.liquid = ps.oceans && h < 0.0;
            p.albedo = if p.liquid { a_lo } else { a_hi };
        }
        let mut h = p.height_m;
        if self.model.style != "bands" {
            for k in 0..octaves {
                let (lambda, amp) = self.octave(k);
                let f = 1.0 / lambda;
                h += amp * value_noise(self.detail_key, k, [d[0] * f, d[1] * f, d[2] * f]);
            }
        }
        (h, p)
    }

    /// The deepest level a survey state discloses for this tile.
    pub fn max_level(&self, state: SurveyState, key: &TileKey) -> Option<u8> {
        match state {
            SurveyState::Unexamined => None,
            SurveyState::Scanned => Some(0),
            SurveyState::Surveyed => Some(self.level_for(self.settings.fact_floor_m)),
            SurveyState::Landed => {
                if self.in_site(key) {
                    Some(self.level_for(self.settings.site_floor_m))
                } else {
                    Some(self.level_for(self.settings.fact_floor_m))
                }
            }
        }
    }

    fn in_site(&self, key: &TileKey) -> bool {
        let n = (1u64 << key.level) as f64;
        let u = -1.0 + 2.0 * (key.x as f64 + 0.5) / n;
        let v = -1.0 + 2.0 * (key.y as f64 + 0.5) / n;
        let c = direction(key.face, u, v);
        let half = self.spacing_m(key.level) * self.settings.tile_samples as f64 * 0.75;
        self.sites.iter().any(|s| {
            let (la, lo) = (s.lat_deg.to_radians(), s.lon_deg.to_radians());
            let p = [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)];
            let ang = m::acos(crate::sketch::dot(c, p).clamp(-1.0, 1.0));
            ang * self.radius_m <= s.radius_km * 1000.0 + half
        })
    }

    pub fn tile_address(&self, key: &TileKey) -> Address {
        self.address
            .child_key("tile", &key.text())
            .expect("valid tile key")
    }

    /// Build one tile, if the survey state discloses it.
    pub fn tile(&self, key: TileKey, state: SurveyState) -> Result<Tile, Refusal> {
        if TileKey::new(key.face, key.level, key.x, key.y).is_none() {
            return Err(Refusal::BadKey);
        }
        let max = self.max_level(state, &key);
        match max {
            None => {
                return Err(Refusal::Withheld {
                    state,
                    max_level: None,
                })
            }
            Some(l) if key.level > l => {
                if state == SurveyState::Scanned {
                    return Err(Refusal::Withheld {
                        state,
                        max_level: Some(l),
                    });
                }
                return Err(Refusal::BelowFactFloor {
                    level: key.level,
                    floor_level: l,
                });
            }
            _ => {}
        }
        Ok(self.build(key))
    }

    /// Regime landforms (S6) and the GM's ranges and pins (S7).
    fn all_landforms(&self) -> impl Iterator<Item = &crate::regimes::Landform> {
        self.regime_layer
            .iter()
            .flat_map(|r| r.landforms.iter())
            .chain(self.layers.iter().flat_map(|l| l.landforms.iter()))
    }

    /// Regime landforms near a tile: those with relief at this level, and
    /// those that set cover or material (at every level).
    fn tile_landforms(&self, key: TileKey, tags: &mut Vec<String>) -> TileLandforms {
        let mut out = TileLandforms::default();
        if self.regime_layer.is_none() && self.layers.is_none() {
            return out;
        }
        let rk = self.radius_m / 1000.0;
        let n = (1u64 << key.level) as f64;
        let centre = direction(
            key.face,
            -1.0 + 2.0 * (key.x as f64 + 0.5) / n,
            -1.0 + 2.0 * (key.y as f64 + 0.5) / n,
        );
        let half =
            1.5 * self.spacing_m(key.level) * self.settings.tile_samples as f64 / self.radius_m;
        let min = self.min_feature_km(key.level);
        for l in self.all_landforms() {
            let reach = (0.5 * l.size_km + 2.0 * l.width_km) / rk;
            if m::acos(crate::sketch::dot(l.dir, centre).clamp(-1.0, 1.0)) > half + reach {
                continue;
            }
            let relief = l.width_km >= min && crate::regimes::landform_curvature(&l.kind) > 0.0;
            if relief {
                out.relief.push(l.clone());
            }
            if l.cover.is_some() || l.material.is_some() {
                out.regions.push(l.clone());
            }
            if relief || l.cover.is_some() || l.material.is_some() {
                tags.push(l.id.clone());
            }
        }
        if let Some(c) = self.regime_layer.as_ref().and_then(|r| r.clouds.as_ref()) {
            for st in &c.storms {
                let (la, lo) = (st.lat_deg.to_radians(), st.lon_deg.to_radians());
                let sd = [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)];
                if m::acos(crate::sketch::dot(sd, centre).clamp(-1.0, 1.0))
                    <= half + st.size_km / rk
                {
                    tags.push(st.id.clone());
                }
            }
        }
        out
    }

    /// The craters, shields and plains that can touch a tile at its level.
    fn tile_features(&self, key: TileKey) -> TileFeatures {
        let mut tf = self.tile_impact_features(key);
        if let Some(ly) = &self.layers {
            let rk = self.radius_m / 1000.0;
            let (centre, half) = self.centre_half(key);
            let min = self.min_feature_km(key.level);
            for c in &ly.craters {
                if c.d_km >= min
                    && m::acos(crate::sketch::dot(c.dir, centre).clamp(-1.0, 1.0))
                        <= half + 1.1 * c.d_km / rk
                {
                    tf.tags.push(c.id.clone());
                    let mut c = c.clone();
                    c.cause.clear();
                    tf.reach.push(crate::impacts::crater_reach_cos(&c, rk));
                    tf.craters.push(c);
                }
            }
        }
        tf
    }

    /// S4's craters, shields and plains that can touch a tile at its level.
    fn tile_impact_features(&self, key: TileKey) -> TileFeatures {
        let mut tf = TileFeatures::default();
        let Some(ip) = &self.impacts else { return tf };
        let f = &ip.features;
        if f.surface_from_myr.is_none() {
            return tf;
        }
        let rk = self.radius_m / 1000.0;
        let n = (1u64 << key.level) as f64;
        let centre = direction(
            key.face,
            -1.0 + 2.0 * (key.x as f64 + 0.5) / n,
            -1.0 + 2.0 * (key.y as f64 + 0.5) / n,
        );
        // Half the tile's diagonal, angle, with margin for the tangent warp.
        let half =
            1.5 * self.spacing_m(key.level) * self.settings.tile_samples as f64 / self.radius_m;
        let min = self.min_feature_km(key.level);
        let near = |d: [f64; 3], reach_km: f64| {
            m::acos(crate::sketch::dot(d, centre).clamp(-1.0, 1.0)) <= half + reach_km / rk
        };
        for c in &f.craters {
            if !c.ghost && c.d_km >= min && near(c.dir, 1.1 * c.d_km) {
                tf.tags.push(c.id.clone());
                tf.craters.push(c.clone());
            }
        }
        for sh in &f.shields {
            if 2.0 * sh.radius_km >= min && near(sh.dir, sh.radius_km) {
                tf.tags.push(sh.id.clone());
                tf.shields.push(sh.clone());
            }
        }
        for p in &f.plains {
            if near(p.dir, p.radius_km) {
                tf.tags.push(p.id.clone());
                tf.plains.push((p.dir, p.radius_km, p.formed_myr));
            }
        }
        // Small craters: every band this level carries, from the cells that
        // hold the tile's samples and their neighbours.
        let bands = self.bands_at(key.level);
        let s = self.settings.tile_samples as i64;
        for k in 0..bands {
            let (_, hi) = ip.field.band(f.listed_d_km, k);
            let cells_n = ip.field.band_cells(hi) as f64;
            // Probe the tile at about half a cell (at most every sample).
            let cell_ang = (m::PI / 2.0) / cells_n;
            let sample_ang = self.spacing_m(key.level) / self.radius_m;
            let step = ((0.5 * cell_ang / sample_ang).max(1.0) as i64).min(s);
            let mut cells = std::collections::BTreeSet::new();
            let total = s as f64 * n;
            let mut j = 0i64;
            while j <= s {
                let mut i = 0i64;
                while i <= s {
                    let u = -1.0 + 2.0 * ((key.x as i64 * s + i.min(s - 1)) as f64 + 0.5) / total;
                    let v = -1.0 + 2.0 * ((key.y as i64 * s + j.min(s - 1)) as f64 + 0.5) / total;
                    cells.insert(
                        ip.field
                            .cell_of(f.listed_d_km, k, direction(key.face, u, v)),
                    );
                    i += step;
                }
                j += step;
            }
            let mut all = std::collections::BTreeSet::new();
            for c in cells {
                for nb in ip.field.neighbourhood(f.listed_d_km, k, c) {
                    all.insert(nb);
                }
            }
            for (face, i, j) in all {
                for c in ip.field.cell_craters(f.listed_d_km, k, face, i, j) {
                    if c.d_km >= min && near(c.dir, 1.1 * c.d_km) {
                        tf.craters.push(c);
                    }
                }
            }
        }
        tf.craters.iter_mut().for_each(|c| {
            c.cause.clear();
        });
        tf.reach = tf
            .craters
            .iter()
            .map(|c| crate::impacts::crater_reach_cos(c, rk))
            .collect();
        tf
    }

    /// Build a tile regardless of disclosure (the GM's own machine, tests).
    pub fn build(&self, key: TileKey) -> Tile {
        let s = self.settings.tile_samples as i64;
        let n = (1u64 << key.level) as f64;
        let total = s as f64 * n;
        let oct = self.octaves_at(key.level);
        let mut feats = self.tile_features(key);
        let lands = self.tile_landforms(key, &mut feats.tags);
        let rk = self.radius_m / 1000.0;
        let deck = self.regime_layer.as_ref().and_then(|r| r.clouds.as_ref());
        let tl = self.layers.as_ref().map(|ly| {
            let (centre, half) = self.centre_half(key);
            ly.near_tile(centre, half, rk, &mut feats.tags)
        });
        // One-sample border for slopes.
        let w = (s + 2) as usize;
        let mut hh = vec![0.0; w * w];
        let mut dirs = vec![[0.0; 3]; w * w];
        let mut pts = vec![];
        for j in -1..=s {
            for i in -1..=s {
                let u = -1.0 + 2.0 * ((key.x as i64 * s + i) as f64 + 0.5) / total;
                let v = -1.0 + 2.0 * ((key.y as i64 * s + j) as f64 + 0.5) / total;
                let d = direction(key.face, u, v);
                let (mut h, mut p) = self.height(d, oct);
                h += feats.height(d, rk);
                for l in &lands.relief {
                    h += crate::regimes::landform_height(l, d, rk).0;
                }
                if let Some(c) = deck {
                    let (al, ch) = c.at(d, rk);
                    p.albedo = al;
                    h += ch;
                }
                let idx = ((j + 1) as usize) * w + (i + 1) as usize;
                hh[idx] = h;
                dirs[idx] = d;
                if i >= 0 && j >= 0 && i < s && j < s {
                    pts.push(p);
                }
            }
        }
        let cnt = (s * s) as usize;
        let mut elevation_dm = Vec::with_capacity(cnt);
        let mut material = Vec::with_capacity(cnt);
        let mut cover = Vec::with_capacity(cnt);
        let mut slope = Vec::with_capacity(cnt);
        let mut age = Vec::with_capacity(cnt);
        let mut albedo = Vec::with_capacity(cnt);
        let mut flow: Vec<u8> = vec![];
        let mut wetness: Vec<u8> = vec![];
        let mut deposit: Vec<u8> = vec![];
        let mut structure: Vec<u8> = vec![];
        let mut colour: Vec<u32> = vec![];
        let style = self.model.style.as_str();
        let dist = |a: [f64; 3], b: [f64; 3]| {
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            m::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) * self.radius_m
        };
        let mut k = 0;
        for j in 0..s as usize {
            for i in 0..s as usize {
                let c = (j + 1) * w + i + 1;
                let h = hh[c];
                let p = pts[k];
                k += 1;
                let (l, r, dn, up) = (c - 1, c + 1, c - w, c + w);
                let gx = (hh[r] - hh[l]) / dist(dirs[r], dirs[l]).max(1e-9);
                let gy = (hh[up] - hh[dn]) / dist(dirs[up], dirs[dn]).max(1e-9);
                let sl = m::atan(m::sqrt(gx * gx + gy * gy)).to_degrees();
                slope.push(m::round(sl * 10.0) as u16);
                elevation_dm.push(m::round(h * 10.0) as i32);
                let liquid = p.liquid;
                let plate_cell = self
                    .plates
                    .as_ref()
                    .map(|ps| (ps, ps.grid.cell_at(dirs[c])));
                let (mat, cov) = match (style, plate_cell) {
                    (_, Some((ps, pc))) => (
                        match (ps.land_material, &ps.flow) {
                            (_, Some(f)) if f[pc].2 => 4u8,
                            (Some(mat), _) if !liquid => mat,
                            _ if ps.continental[pc] => 1u8,
                            _ => 0,
                        },
                        if liquid || ps.flow.as_ref().is_some_and(|f| f[pc].1) {
                            1u8
                        } else {
                            0
                        },
                    ),
                    _ => match style {
                        "plates" if liquid => (0u8, 1u8),
                        "ocean" => (0, 1),
                        "smooth" if liquid => (5, 2),
                        "dunes" if liquid => (3, 3),
                        _ => (self.material_land, 0),
                    },
                };
                let mut mat_override: Option<u8> = None;
                material.push(mat);
                cover.push(cov);
                // Ocean floor is young everywhere on a plate world; the plate
                // history (S2) writes real crust ages.
                let a = if let Some((ps, pc)) = plate_cell {
                    ps.age_myr[pc]
                } else if style == "plates" && liquid {
                    self.regime_age_myr.min(200.0)
                } else {
                    self.regime_age_myr
                };
                let a = match feats.plains_at(dirs[c], self.radius_m / 1000.0) {
                    Some(t) if !liquid => {
                        mat_override = Some(0);
                        a.min((self.epoch_myr as f64 - t).max(0.0))
                    }
                    _ => a,
                };
                age.push(m::round(a) as u32);
                for l in &lands.regions {
                    if crate::regimes::landform_height(l, dirs[c], rk).1 {
                        if let Some(mi) = l
                            .material
                            .as_deref()
                            .and_then(|x| MATERIALS.iter().position(|y| *y == x))
                        {
                            mat_override = Some(mi as u8);
                        }
                        if let (Some(ci), Some(last)) = (
                            l.cover
                                .as_deref()
                                .and_then(|x| COVERS.iter().position(|y| *y == x)),
                            cover.last_mut(),
                        ) {
                            *last = ci as u8;
                        }
                    }
                }
                if let (Some(mm), Some(last)) = (mat_override, material.last_mut()) {
                    *last = mm;
                }
                if let Some((ps, pc)) = plate_cell {
                    if let Some(f) = &ps.flow {
                        let (river, _, _, cells) = f[pc];
                        flow.push(if river {
                            (1.0 + 8.0 * m::log2(cells.max(1.0))).min(255.0) as u8
                        } else {
                            0
                        });
                    }
                }
                let mut al = p.albedo;
                if let (Some(ly), Some(t)) = (&self.layers, &tl) {
                    let mat = *material.last().expect("pushed");
                    let cov = cover.last_mut().expect("pushed");
                    let sl = ly.sample(t, dirs[c], h, mat, *cov, al, rk);
                    *cov = sl.cover;
                    al = sl.albedo;
                    wetness.push(sl.wet);
                    deposit.push(sl.deposit);
                    structure.push(sl.structure);
                    colour.push(sl.colour);
                }
                albedo.push(m::round(al.clamp(0.0, 1.0) * 1000.0) as u16);
            }
        }
        let hash = if self.layers.is_some() {
            ContentHash::of(&(
                (&elevation_dm, &material, &cover, &slope, &age, &albedo),
                (&flow, &wetness, &deposit, &structure, &colour),
            ))
        } else if flow.is_empty() {
            ContentHash::of(&(&elevation_dm, &material, &cover, &slope, &age, &albedo))
        } else {
            ContentHash::of(&(
                &elevation_dm,
                &material,
                &cover,
                &slope,
                &age,
                &albedo,
                &flow,
            ))
        }
        .expect("hash")
        .hex();
        let mut pending: Vec<String> = vec![
            "flow and channels (S3)".into(),
            if self.impacts.is_some() {
                "named features (S7)".into()
            } else {
                "feature tags (S4, S7)".into()
            },
            "biome (S11)".into(),
            "structures (S7)".into(),
        ];
        if !flow.is_empty() {
            pending.remove(0);
        }
        if self.layers.is_some() {
            pending.retain(|x| x.contains("flow") || x.contains("biome"));
        }
        Tile {
            address: self.tile_address(&key),
            key,
            epoch_myr: self.epoch_myr,
            samples: s as u32,
            spacing_m: self.spacing_m(key.level),
            octaves: oct,
            elevation_dm,
            material,
            cover,
            slope_ddeg: slope,
            surface_age_myr: age,
            albedo,
            tags: feats.tags,
            pending,
            flow,
            wetness,
            deposit,
            structure,
            colour,
            hash,
        }
    }
}

#[derive(Default)]
struct TileLandforms {
    relief: Vec<crate::regimes::Landform>,
    regions: Vec<crate::regimes::Landform>,
}

/// Features that can touch one tile (gathered once per tile build).
#[derive(Default)]
struct TileFeatures {
    craters: Vec<crate::impacts::Crater>,
    /// Per crater, the cosine beyond which it adds nothing.
    reach: Vec<f64>,
    shields: Vec<crate::impacts::Shield>,
    /// (centre, radius km, formed Myr).
    plains: Vec<([f64; 3], f64, f64)>,
    tags: Vec<String>,
}

impl TileFeatures {
    fn height(&self, d: [f64; 3], radius_km: f64) -> f64 {
        let mut h = 0.0;
        for (c, reach) in self.craters.iter().zip(&self.reach) {
            h += crate::impacts::crater_height_within(c, d, radius_km, *reach);
        }
        for s in &self.shields {
            h += crate::impacts::shield_height(s, d, radius_km);
        }
        h
    }

    /// When the newest plains unit covering the point erupted, Myr since the
    /// world formed.
    fn plains_at(&self, d: [f64; 3], radius_km: f64) -> Option<f64> {
        self.plains
            .iter()
            .filter(|p| m::acos(crate::sketch::dot(p.0, d).clamp(-1.0, 1.0)) * radius_km < p.1)
            .map(|p| p.2)
            .fold(None, |a: Option<f64>, t| Some(a.map_or(t, |x| x.max(t))))
    }
}

/// Average a child tile 2 × 2 onto its quarter of the parent: returns the
/// largest difference in metres over that quarter.
pub fn downsample_error_m(parent: &Tile, child: &Tile) -> f64 {
    let s = parent.samples as usize;
    let half = s / 2;
    let (ox, oy) = (
        (child.key.x % 2) as usize * half,
        (child.key.y % 2) as usize * half,
    );
    let mut worst: f64 = 0.0;
    for j in 0..half {
        for i in 0..half {
            let c = |a: usize, b: usize| child.elevation_dm[(2 * j + b) * s + 2 * i + a] as f64;
            let avg = 0.25 * (c(0, 0) + c(1, 0) + c(0, 1) + c(1, 1));
            let p = parent.elevation_dm[(oy + j) * s + ox + i] as f64;
            worst = worst.max((avg - p).abs() / 10.0);
        }
    }
    worst
}

/// A bounded tile cache. Eviction is by use order (a counter, never a clock),
/// so it never changes what a tile contains.
#[derive(Clone, Debug, Default)]
pub struct TileCache {
    capacity: usize,
    tick: u64,
    entries: BTreeMap<(String, TileKey), (u64, Tile)>,
    pub hits: u64,
    pub misses: u64,
}

impl TileCache {
    pub fn new(capacity: usize) -> Self {
        TileCache {
            capacity: capacity.max(1),
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The tile from the cache, or built (and disclosure-checked) and kept.
    pub fn get(
        &mut self,
        atlas: &Atlas,
        key: TileKey,
        state: SurveyState,
    ) -> Result<Tile, Refusal> {
        // Disclosure is checked on every request, cached or not.
        if let Some(l) = atlas.max_level(state, &key) {
            if key.level > l {
                return atlas.tile(key, state);
            }
        } else {
            return atlas.tile(key, state);
        }
        self.tick += 1;
        let id = (atlas.address.to_string(), key);
        if let Some(e) = self.entries.get_mut(&id) {
            e.0 = self.tick;
            self.hits += 1;
            return Ok(e.1.clone());
        }
        self.misses += 1;
        let t = atlas.tile(key, state)?;
        if self.entries.len() >= self.capacity {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, v)| v.0)
                .map(|(k, _)| k.clone())
                .expect("non-empty");
            self.entries.remove(&oldest);
        }
        self.entries.insert(id, (self.tick, t.clone()));
        Ok(t)
    }
}

/// Heights at one level, with each tile's gathered features kept (see
/// [`Atlas::heights`]). The same sum as [`Atlas::height_at`].
pub struct Heights<'a> {
    atlas: &'a Atlas,
    level: u8,
    tiles: Vec<(TileKey, TileFeatures, TileLandforms)>,
}

impl Heights<'_> {
    pub fn at(&mut self, d: [f64; 3]) -> f64 {
        let a = self.atlas;
        let key = a.tile_key_at(d, self.level);
        let i = match self.tiles.iter().position(|t| t.0 == key) {
            Some(i) => i,
            None => {
                let mut tags = vec![];
                let lands = a.tile_landforms(key, &mut tags);
                self.tiles.push((key, a.tile_features(key), lands));
                self.tiles.len() - 1
            }
        };
        let (_, feats, lands) = &self.tiles[i];
        let rk = a.radius_m / 1000.0;
        let (mut h, _) = a.height(d, a.octaves_at(self.level));
        h += feats.height(d, rk);
        for l in &lands.relief {
            h += crate::regimes::landform_height(l, d, rk).0;
        }
        if let Some(c) = a.regime_layer.as_ref().and_then(|r| r.clouds.as_ref()) {
            h += c.at(d, rk).1;
        }
        h
    }
}
