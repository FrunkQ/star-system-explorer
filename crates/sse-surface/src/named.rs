//! S7: named features (surface design section 4, L4).
//!
//! The register lists a world's addressable features: oceans, seas, lakes,
//! continents, islands, ranges and their peaks, ice caps and the deepest
//! place, read from the atlas's own tiles at one level; and the craters,
//! shields, plains, regime landforms, GM edits and built places the earlier
//! stages and S7's layers hold. Each has a kind, extent, date where known, a
//! cause and a generated name from the campaign's naming rules (pack data).
//!
//! Names can be changed like a planet's. A rename is an authored event with a
//! date and, optionally, an audience (a seat's own people's names), so it
//! never changes the feature's address, physics or history. The generated
//! name stays on record and can be restored. The GM can also name places the
//! generator left unnamed. Sifters pick out the remarkable ones.

use crate::atlas::{Atlas, TileKey, COVERS};
use crate::plates::Grid;
use crate::SurfaceEnvelope;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use std::collections::{BTreeMap, BTreeSet};

pub const NAMED_VERSION: &str = "surface-named-v1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NamingSettings {
    pub syllables: Vec<String>,
    /// Fewest and most syllables in a generated name.
    pub length: [u32; 2],
    /// Pattern per kind, `{}` standing for the generated word.
    pub patterns: BTreeMap<String, String>,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FeatureSettings {
    /// Atlas level the register reads (0: the six level-0 tiles).
    pub level: u8,
    pub crater_min_km: f64,
    pub landform_min_km: f64,
    pub lake_max_km2: f64,
    /// A water body at least this fraction of the surface is an ocean.
    pub ocean_min_fraction: f64,
    /// A landmass at least this fraction of the surface is a continent.
    pub continent_min_fraction: f64,
    pub land_min_km2: f64,
    /// Ranges: land this many standard deviations above the mean land height.
    pub range_sigma: f64,
    pub range_min_km2: f64,
    pub ice_min_km2: f64,
    pub canyon_orbit_km: f64,
    pub max_per_kind: u32,
    pub naming: NamingSettings,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NamedFeature {
    pub id: String,
    pub address: String,
    pub kind: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub extent_km: f64,
    #[serde(default)]
    pub area_km2: Option<f64>,
    /// Height of a peak, depth of a deep, relief of a crater, m.
    #[serde(default)]
    pub height_m: Option<f64>,
    #[serde(default)]
    pub formed_myr: Option<f64>,
    pub cause: String,
    /// The generator's name (or the GM's, for a place the GM named first).
    pub original_name: String,
    pub remarkable: Vec<String>,
    #[serde(default)]
    pub authored: bool,
}

/// A rename: an authored, dated event. `restore` brings back the original.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Rename {
    pub feature: String,
    #[serde(default)]
    pub name: String,
    pub at_myr: f64,
    #[serde(default)]
    pub audience: Option<String>,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub restore: bool,
}

/// A place the GM names that the generator left unnamed (a pass, a bay, a
/// landing site).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthoredName {
    pub id: String,
    pub kind: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub name: String,
    pub at_myr: f64,
}

/// One name a feature has carried.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NameRecord {
    pub name: String,
    pub from_myr: Option<f64>,
    pub audience: Option<String>,
    pub by: String,
    pub original: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Register {
    pub address: String,
    pub epoch_myr: f64,
    pub level: u8,
    pub features: Vec<NamedFeature>,
    pub renames: Vec<Rename>,
}

fn lat_lon(v: [f64; 3]) -> (f64, f64) {
    (
        m::asin(v[2].clamp(-1.0, 1.0)).to_degrees(),
        m::atan2(v[1], v[0]).to_degrees(),
    )
}

fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (la, lo) = (lat_deg.to_radians(), lon_deg.to_radians());
    [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)]
}

fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    m::acos((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0))
}

fn key_of(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._:@-".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// A stable counter from an id, so a feature's name does not depend on the
/// order features are found in.
fn counter_of(id: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h >> 16
}

struct Namer<'a> {
    s: &'a NamingSettings,
    d: Draws,
    used: BTreeSet<String>,
}

impl Namer<'_> {
    fn word(&self, id: &str, attempt: u64) -> String {
        let base = counter_of(id).wrapping_add(attempt.wrapping_mul(0x9e37_79b9));
        let (lo, hi) = (
            self.s.length[0].max(1),
            self.s.length[1].max(self.s.length[0].max(1)),
        );
        let n = lo as u64 + self.d.below(base, (hi - lo + 1) as u64);
        let mut w = String::new();
        for k in 0..n {
            let i = self.d.below(
                base.wrapping_add(17 * (k + 1)),
                self.s.syllables.len() as u64,
            );
            w.push_str(&self.s.syllables[i as usize]);
        }
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => w,
        }
    }

    fn name(&mut self, id: &str, kind: &str) -> String {
        let mut attempt = 0u64;
        loop {
            let w = self.word(id, attempt);
            if !self.used.contains(&w) || attempt > 64 {
                self.used.insert(w.clone());
                let pat = self
                    .s
                    .patterns
                    .get(kind)
                    .or_else(|| self.s.patterns.get("default"))
                    .map(|x| x.as_str())
                    .unwrap_or("{}");
                return pat.replacen("{}", &w, 1);
            }
            attempt += 1;
        }
    }
}

/// The atlas's tiles at one level, flattened onto a cube grid.
pub struct Globe {
    pub grid: Grid,
    pub height_m: Vec<f64>,
    pub cover: Vec<u8>,
    pub cell_km2: f64,
}

impl Globe {
    pub fn of(atlas: &Atlas, level: u8) -> Globe {
        let s = atlas.settings().tile_samples as usize;
        let tiles = 1usize << level;
        let n = s * tiles;
        let grid = Grid::new(n as u32);
        let mut height_m = vec![0.0; 6 * n * n];
        let mut cover = vec![0u8; 6 * n * n];
        for f in 0..6u8 {
            for ty in 0..tiles {
                for tx in 0..tiles {
                    let t = atlas.build(TileKey::new(f, level, tx as u32, ty as u32).expect("key"));
                    for j in 0..s {
                        for i in 0..s {
                            let c = f as usize * n * n + (ty * s + j) * n + tx * s + i;
                            height_m[c] = t.elevation_dm[j * s + i] as f64 / 10.0;
                            cover[c] = t.cover[j * s + i];
                        }
                    }
                }
            }
        }
        let r = atlas.radius_m / 1000.0;
        Globe {
            grid,
            height_m,
            cover,
            cell_km2: 4.0 * m::PI * r * r / (6 * n * n) as f64,
        }
    }

    /// Connected components of cells where `f` holds, largest first (ties by
    /// first cell).
    pub fn components(&self, f: impl Fn(usize) -> bool) -> Vec<Vec<usize>> {
        let mut seen = vec![false; self.grid.len()];
        let mut out = vec![];
        for start in 0..self.grid.len() {
            if seen[start] || !f(start) {
                continue;
            }
            let mut comp = vec![];
            let mut stack = vec![start];
            seen[start] = true;
            while let Some(c) = stack.pop() {
                comp.push(c);
                for nb in self.grid.neighbours(c) {
                    if !seen[nb] && f(nb) {
                        seen[nb] = true;
                        stack.push(nb);
                    }
                }
            }
            comp.sort_unstable();
            out.push(comp);
        }
        out.sort_by(|a, b| b.len().cmp(&a.len()).then(a[0].cmp(&b[0])));
        out
    }

    /// Mean direction of cells (normalised).
    fn centre(&self, cells: &[usize]) -> [f64; 3] {
        let mut v = [0.0; 3];
        for &c in cells {
            let d = self.grid.dirs[c];
            v = [v[0] + d[0], v[1] + d[1], v[2] + d[2]];
        }
        let l = m::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
        if l < 1e-9 {
            self.grid.dirs[cells[0]]
        } else {
            [v[0] / l, v[1] / l, v[2] / l]
        }
    }
}

fn water_body(cov: u8, h: f64, sea_ice: bool) -> bool {
    matches!(COVERS[cov as usize], "water" | "hydrocarbon")
        || (sea_ice && COVERS[cov as usize] == "ice" && h < 0.0)
}

/// Build a world's feature register. Renames and GM names are applied in the
/// order given; a rename of an unknown feature is refused.
pub fn register(
    fs: &FeatureSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    atlas: &Atlas,
    authored: &[AuthoredName],
    renames: Vec<Rename>,
) -> Result<Register, String> {
    let rk = env.reference_radius_m / 1000.0;
    let mut namer = Namer {
        s: &fs.naming,
        d: Draws::new(
            *seed,
            env.address.clone(),
            Stream::new("surface.names.v1").expect("valid"),
        ),
        used: BTreeSet::new(),
    };
    let mut out: Vec<NamedFeature> = vec![];
    let address = |id: &str| {
        env.address
            .child_key("feature", &key_of(id))
            .map(|a| a.to_string())
            .unwrap_or_default()
    };
    let mut push = |out: &mut Vec<NamedFeature>,
                    id: String,
                    kind: &str,
                    dir: [f64; 3],
                    extent_km: f64,
                    area_km2: Option<f64>,
                    height_m: Option<f64>,
                    formed_myr: Option<f64>,
                    cause: String| {
        let (la, lo) = lat_lon(dir);
        let name = namer.name(&id, kind);
        out.push(NamedFeature {
            address: address(&id),
            id,
            kind: kind.into(),
            lat_deg: la,
            lon_deg: lo,
            extent_km,
            area_km2,
            height_m,
            formed_myr,
            cause,
            original_name: name,
            remarkable: vec![],
            authored: false,
        });
    };
    let solid = env.kind != "cloud-deck";
    // Globe-derived features.
    let mut highest: Option<(f64, usize)> = None;
    if solid {
        let gl = Globe::of(atlas, fs.level);
        let total = gl.grid.len() as f64 * gl.cell_km2;
        let sea_ice = atlas
            .layers()
            .is_some_and(|l| l.climate.liquid.as_deref() == Some("water"));
        let wet: Vec<bool> = (0..gl.grid.len())
            .map(|c| water_body(gl.cover[c], gl.height_m[c], sea_ice))
            .collect();
        let any_water = wet.iter().any(|x| *x);
        let mut n_kind: BTreeMap<String, u32> = BTreeMap::new();
        let mut take = |k: &str| {
            let e = n_kind.entry(k.to_string()).or_insert(0);
            *e += 1;
            (*e <= fs.max_per_kind).then_some(*e)
        };
        for comp in gl.components(|c| wet[c]) {
            let area = comp.len() as f64 * gl.cell_km2;
            let kind = if area >= fs.ocean_min_fraction * total {
                "ocean"
            } else if area <= fs.lake_max_km2 {
                "lake"
            } else {
                "sea"
            };
            let Some(k) = take(kind) else { continue };
            let liquid = COVERS[gl.cover[comp[0]] as usize];
            let deep = comp
                .iter()
                .map(|c| gl.height_m[*c])
                .fold(f64::INFINITY, f64::min);
            push(
                &mut out,
                format!("{kind}-{k}"),
                kind,
                gl.centre(&comp),
                2.0 * m::sqrt(area / m::PI),
                Some(area),
                Some(deep),
                None,
                format!(
                    "{} filling low ground ({} cells at level {})",
                    if liquid == "ice" {
                        "frozen water"
                    } else {
                        liquid
                    },
                    comp.len(),
                    fs.level
                ),
            );
        }
        let land: Vec<usize> = (0..gl.grid.len()).filter(|c| !wet[*c]).collect();
        if any_water {
            for comp in gl.components(|c| !wet[c]) {
                let area = comp.len() as f64 * gl.cell_km2;
                if area < fs.land_min_km2 {
                    continue;
                }
                let kind = if area >= fs.continent_min_fraction * total {
                    "continent"
                } else {
                    "island"
                };
                let Some(k) = take(kind) else { continue };
                push(
                    &mut out,
                    format!("{kind}-{k}"),
                    kind,
                    gl.centre(&comp),
                    2.0 * m::sqrt(area / m::PI),
                    Some(area),
                    None,
                    None,
                    "ground above the sea, raised by its crust's thickness and history".into(),
                );
            }
        }
        if !land.is_empty() {
            let nl = land.len() as f64;
            let mean = land.iter().map(|c| gl.height_m[*c]).sum::<f64>() / nl;
            let var = land
                .iter()
                .map(|c| (gl.height_m[*c] - mean) * (gl.height_m[*c] - mean))
                .sum::<f64>()
                / nl;
            let cut = mean + fs.range_sigma * m::sqrt(var);
            let high: Vec<bool> = (0..gl.grid.len())
                .map(|c| !wet[c] && gl.height_m[c] > cut)
                .collect();
            for comp in gl.components(|c| high[c]) {
                let area = comp.len() as f64 * gl.cell_km2;
                if area < fs.range_min_km2 {
                    continue;
                }
                let Some(k) = take("range") else { continue };
                let top = *comp
                    .iter()
                    .max_by(|a, b| gl.height_m[**a].total_cmp(&gl.height_m[**b]).then(b.cmp(a)))
                    .expect("non-empty");
                push(
                    &mut out,
                    format!("range-{k}"),
                    "range",
                    gl.centre(&comp),
                    2.0 * m::sqrt(area / m::PI),
                    Some(area),
                    Some(gl.height_m[top]),
                    None,
                    format!(
                        "high ground over {:.0} m ({:.1} standard deviations above the mean land)",
                        cut, fs.range_sigma
                    ),
                );
                push(
                    &mut out,
                    format!("peak-{k}"),
                    "peak",
                    gl.grid.dirs[top],
                    0.0,
                    None,
                    Some(gl.height_m[top]),
                    None,
                    format!("the highest point of range-{k}"),
                );
            }
            let ice: Vec<bool> = (0..gl.grid.len())
                .map(|c| !wet[c] && COVERS[gl.cover[c] as usize] == "ice")
                .collect();
            for comp in gl.components(|c| ice[c]) {
                let area = comp.len() as f64 * gl.cell_km2;
                if area < fs.ice_min_km2 {
                    continue;
                }
                let Some(k) = take("ice-cap") else { continue };
                push(
                    &mut out,
                    format!("ice-cap-{k}"),
                    "ice-cap",
                    gl.centre(&comp),
                    2.0 * m::sqrt(area / m::PI),
                    Some(area),
                    None,
                    None,
                    "ground colder than ice can melt from, on a world that holds water".into(),
                );
            }
        }
        let top = (0..gl.grid.len())
            .max_by(|a, b| gl.height_m[*a].total_cmp(&gl.height_m[*b]).then(b.cmp(a)))
            .expect("cells");
        highest = Some((gl.height_m[top], top));
        let low = (0..gl.grid.len())
            .min_by(|a, b| gl.height_m[*a].total_cmp(&gl.height_m[*b]).then(a.cmp(b)))
            .expect("cells");
        push(
            &mut out,
            "deep-1".into(),
            "deep",
            gl.grid.dirs[low],
            0.0,
            None,
            Some(gl.height_m[low]),
            None,
            format!("the lowest point read at level {}", fs.level),
        );
        if !out
            .iter()
            .any(|f| f.kind == "peak" && f.height_m == Some(gl.height_m[top]))
        {
            push(
                &mut out,
                "summit-1".into(),
                "summit",
                gl.grid.dirs[top],
                0.0,
                None,
                Some(gl.height_m[top]),
                None,
                format!("the highest point read at level {}", fs.level),
            );
        }
        // Crater lakes need the globe: remember which cells are water.
        let lake_at = |d: [f64; 3]| wet[gl.grid.cell_at(d)];
        if let Some(f) = atlas.features() {
            for c in f
                .craters
                .iter()
                .filter(|c| !c.ghost && c.d_km >= fs.crater_min_km)
            {
                push(
                    &mut out,
                    c.id.clone(),
                    "crater",
                    c.dir,
                    c.d_km,
                    None,
                    Some(c.depth_m),
                    Some(c.formed_myr),
                    c.cause.clone(),
                );
                if lake_at(c.dir) {
                    let last = out.len() - 1;
                    out[last].remarkable.push("crater lake".into());
                }
            }
            for sh in &f.shields {
                push(
                    &mut out,
                    sh.id.clone(),
                    "shield",
                    sh.dir,
                    2.0 * sh.radius_km,
                    None,
                    Some(sh.height_m),
                    Some(sh.from_myr),
                    sh.cause.clone(),
                );
            }
            for p in &f.plains {
                push(
                    &mut out,
                    p.id.clone(),
                    "plains",
                    p.dir,
                    2.0 * p.radius_km,
                    Some(m::PI * p.radius_km * p.radius_km),
                    None,
                    Some(p.formed_myr),
                    p.cause.clone(),
                );
            }
        }
    }
    let landforms = atlas
        .regime_surface()
        .map(|r| r.landforms.clone())
        .unwrap_or_default();
    for l in landforms.iter().filter(|l| l.size_km >= fs.landform_min_km) {
        push(
            &mut out,
            l.id.clone(),
            &l.kind,
            l.dir,
            l.size_km,
            None,
            Some(l.height_m),
            Some(l.formed_myr),
            l.cause.clone(),
        );
        if matches!(l.kind.as_str(), "rift" | "chasma") && l.size_km >= fs.canyon_orbit_km {
            let last = out.len() - 1;
            out[last]
                .remarkable
                .push("canyon visible from orbit".into());
        }
    }
    if let Some(ly) = atlas.layers() {
        for l in ly.landforms.iter().filter(|l| l.kind != "pin") {
            push(
                &mut out,
                l.id.clone(),
                &l.kind,
                l.dir,
                l.size_km,
                None,
                Some(l.height_m),
                Some(l.formed_myr),
                l.cause.clone(),
            );
        }
        for c in &ly.craters {
            push(
                &mut out,
                c.id.clone(),
                "crater",
                c.dir,
                c.d_km,
                None,
                Some(c.depth_m),
                Some(c.formed_myr),
                c.cause.clone(),
            );
        }
        for b in &ly.built {
            push(
                &mut out,
                b.id.clone(),
                &b.kind,
                b.dir,
                2.0 * b.radius_km,
                None,
                None,
                Some(b.at_myr),
                b.why.join("; "),
            );
            if let Some(n) = &b.name {
                let last = out.len() - 1;
                out[last].original_name = n.clone();
                out[last].authored = true;
            }
        }
    }
    for a in authored {
        if out.iter().any(|f| f.id == a.id) {
            return Err(format!("the GM's name {} reuses a feature id", a.id));
        }
        out.push(NamedFeature {
            id: a.id.clone(),
            address: address(&a.id),
            kind: a.kind.clone(),
            lat_deg: a.lat_deg,
            lon_deg: a.lon_deg,
            extent_km: 0.0,
            area_km2: None,
            height_m: None,
            formed_myr: Some(a.at_myr),
            cause: format!("named by the GM at {:.0} Myr", a.at_myr),
            original_name: a.name.clone(),
            remarkable: vec![],
            authored: true,
        });
    }
    // Sifters.
    let mark = |out: &mut Vec<NamedFeature>, i: Option<usize>, what: &str| {
        if let Some(i) = i {
            out[i].remarkable.push(what.into());
        }
    };
    if let Some((h, _)) = highest {
        let i = out
            .iter()
            .position(|f| matches!(f.kind.as_str(), "peak" | "summit") && f.height_m == Some(h));
        mark(&mut out, i, "highest point");
    }
    let i = out.iter().position(|f| f.kind == "deep");
    mark(&mut out, i, "deepest point");
    let biggest = |out: &Vec<NamedFeature>, kind: &str| {
        out.iter()
            .enumerate()
            .filter(|(_, f)| f.kind == kind)
            .max_by(|a, b| a.1.extent_km.total_cmp(&b.1.extent_km).then(b.0.cmp(&a.0)))
            .map(|x| x.0)
    };
    let i = biggest(&out, "crater");
    mark(&mut out, i, "largest crater");
    let i = biggest(&out, "ocean").or_else(|| biggest(&out, "sea"));
    mark(&mut out, i, "largest body of liquid");
    let i = biggest(&out, "continent");
    mark(&mut out, i, "largest continent");
    let _ = rk;
    let mut reg = Register {
        address: env.address.to_string(),
        epoch_myr: atlas.epoch_myr as f64,
        level: fs.level,
        features: out,
        renames: vec![],
    };
    for r in renames {
        reg.rename(r)?;
    }
    Ok(reg)
}

impl Register {
    pub fn get(&self, id: &str) -> Option<&NamedFeature> {
        self.features.iter().find(|f| f.id == id)
    }

    /// Record a rename (refused for an unknown feature or an empty name).
    pub fn rename(&mut self, r: Rename) -> Result<(), String> {
        if self.get(&r.feature).is_none() {
            return Err(format!("no feature {} to rename", r.feature));
        }
        if !r.restore && r.name.trim().is_empty() {
            return Err(format!("an empty name for {}", r.feature));
        }
        self.renames.push(r);
        Ok(())
    }

    /// The name a feature carries at a date for an audience: the latest
    /// rename for that audience, else the latest general one, else the original.
    pub fn name_of(&self, id: &str, audience: Option<&str>, at_myr: f64) -> Option<String> {
        let f = self.get(id)?;
        let pick = |aud: Option<&str>| {
            self.renames
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    r.feature == id && r.at_myr <= at_myr && r.audience.as_deref() == aud
                })
                .max_by(|a, b| a.1.at_myr.total_cmp(&b.1.at_myr).then(a.0.cmp(&b.0)))
                .map(|x| x.1)
        };
        let r = audience.and_then(|a| pick(Some(a))).or_else(|| pick(None));
        Some(match r {
            Some(r) if !r.restore => r.name.clone(),
            _ => f.original_name.clone(),
        })
    }

    /// Every name a feature has carried, the original first.
    pub fn names(&self, id: &str) -> Vec<NameRecord> {
        let Some(f) = self.get(id) else { return vec![] };
        let mut v = vec![NameRecord {
            name: f.original_name.clone(),
            from_myr: f.formed_myr.filter(|_| f.authored),
            audience: None,
            by: if f.authored {
                "gm".into()
            } else {
                "generator".into()
            },
            original: true,
        }];
        for r in self.renames.iter().filter(|r| r.feature == id) {
            v.push(NameRecord {
                name: if r.restore {
                    f.original_name.clone()
                } else {
                    r.name.clone()
                },
                from_myr: Some(r.at_myr),
                audience: r.audience.clone(),
                by: r.by.clone(),
                original: false,
            });
        }
        v
    }

    /// Features carrying a remark, with the remarks.
    pub fn remarkable(&self) -> Vec<(&str, &[String])> {
        self.features
            .iter()
            .filter(|f| !f.remarkable.is_empty())
            .map(|f| (f.id.as_str(), f.remarkable.as_slice()))
            .collect()
    }

    /// The nearest feature of a kind to a place.
    pub fn nearest(&self, kind: &str, lat_deg: f64, lon_deg: f64) -> Option<&NamedFeature> {
        let v = from_lat_lon(lat_deg, lon_deg);
        self.features
            .iter()
            .filter(|f| f.kind == kind)
            .min_by(|a, b| {
                angle(from_lat_lon(a.lat_deg, a.lon_deg), v)
                    .total_cmp(&angle(from_lat_lon(b.lat_deg, b.lon_deg), v))
            })
    }
}
