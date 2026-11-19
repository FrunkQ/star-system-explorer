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
pub const COVERS: [&str; 4] = ["none", "water", "magma", "hydrocarbon"];

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
    let (a, b) = (crate::sketch::dot(d, r) / k, crate::sketch::dot(d, t) / k);
    (best, m::atan(a) * 4.0 / m::PI, m::atan(b) * 4.0 / m::PI)
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
            .child_key("shell", &format!("surface@{epoch_myr}"))
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
        }
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
        let x = m::PI * sp / 0.077;
        let sketch = self.model.relief_m() * (0.25 * x * x).min(1.0);
        added + carried + sketch + 0.1
    }

    /// Height at a direction carrying `octaves` detail octaves, metres.
    pub fn height(&self, d: [f64; 3], octaves: u32) -> (f64, crate::sketch::Point) {
        let p = self.model.at(d);
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

    /// Build a tile regardless of disclosure (the GM's own machine, tests).
    pub fn build(&self, key: TileKey) -> Tile {
        let s = self.settings.tile_samples as i64;
        let n = (1u64 << key.level) as f64;
        let total = s as f64 * n;
        let oct = self.octaves_at(key.level);
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
                let (h, p) = self.height(d, oct);
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
                let (mat, cov) = match style {
                    "plates" if liquid => (0u8, 1u8),
                    "ocean" => (0, 1),
                    "smooth" if liquid => (5, 2),
                    "dunes" if liquid => (3, 3),
                    _ => (self.material_land, 0),
                };
                material.push(mat);
                cover.push(cov);
                // Ocean floor is young everywhere on a plate world; the plate
                // history (S2) writes real crust ages.
                let a = if style == "plates" && liquid {
                    self.regime_age_myr.min(200.0)
                } else {
                    self.regime_age_myr
                };
                age.push(m::round(a) as u32);
                albedo.push(m::round(p.albedo.clamp(0.0, 1.0) * 1000.0) as u16);
            }
        }
        let hash = ContentHash::of(&(&elevation_dm, &material, &cover, &slope, &age, &albedo))
            .expect("hash")
            .hex();
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
            tags: vec![],
            pending: vec![
                "flow and channels (S3)".into(),
                "feature tags (S4, S7)".into(),
                "crust ages (S2)".into(),
                "biome (S11)".into(),
                "structures (S7)".into(),
            ],
            hash,
        }
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
