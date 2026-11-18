//! Habitat graphs (life design section 2, box A): pack rules turn a world
//! stack's shells, regions and features into places life could be, linked
//! where material and organisms can pass.
//!
//! A habitat is a candidate place, not a claim that anything lives there:
//! Venus's 737 K plains are listed with their temperature, and box B (L1)
//! sets their cap to zero with the reason.

use crate::{EnergyChannels, LifeRequirementProfile, Range};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, ContentHash};
use sse_pack::PackManifest;
use sse_world::{Extent, WorldStack};
use std::collections::{BTreeMap, BTreeSet};

/// Which kind of stack object a rule reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Object {
    Shell,
    Region,
    Feature,
}

/// Where in the object the habitat lies. `top` and `bottom` are the
/// object's altitude span (a feature's is its altitude).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "snake_case")]
pub enum Place {
    /// The whole span.
    All,
    /// The top `depth_m` (a sunlit layer).
    Top { depth_m: f64 },
    /// Everything below the top `depth_m`.
    BelowTop { depth_m: f64 },
    /// A layer `depth_m` thick just under the bottom (a sea floor).
    UnderBottom { depth_m: f64 },
    /// From `up_m` above the top down to the top (the ground and the air on it).
    OnTop { up_m: f64 },
    /// From `from_m` to `to_m` below the top (soils, deep rock).
    Under { from_m: f64, to_m: f64 },
    /// `half_m` either side of the altitude (a cave, a lake feature).
    Around { half_m: f64 },
}

/// One pack rule (`provides.other.habitat_rules`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HabitatRule {
    /// Unique across a stack of packs.
    pub id: String,
    /// The habitat kind it makes (several rules may make one kind).
    pub habitat: String,
    pub from: Object,
    /// Stack object kinds it applies to.
    pub kinds: Vec<String>,
    pub place: Place,
    pub medium: String,
    pub phase: String,
    pub solvent: String,
    /// Energy channels open in principle.
    #[serde(default)]
    pub channels: Vec<String>,
    /// Adds `chemical` when an object starting at or below the habitat's top
    /// provides one of these (a warm mantle under a sea floor).
    #[serde(default)]
    pub chemical_from_below: Vec<String>,
    /// The shell just below the habitat must be one of these kinds.
    #[serde(default)]
    pub needs_below: Vec<String>,
    /// The object must carry this property.
    #[serde(default)]
    pub needs_property: Option<String>,
    /// Sits inside what surrounds it (a cave in rock, vents in the sea
    /// floor), so it links to every habitat it overlaps.
    #[serde(default)]
    pub embedded: bool,
    pub about: String,
}

/// Horizontal footprint of a habitat.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Footprint {
    Global,
    /// A region, with its latitude range when it has a drawn outline.
    Region {
        key: String,
        lat_deg: Option<Range>,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Habitat {
    pub address: Address,
    pub kind: String,
    /// Stack object key it came from, and the rule.
    pub source: String,
    pub rule: String,
    pub top_m: f64,
    pub bottom_m: f64,
    pub footprint: Footprint,
    pub embedded: bool,
    pub profile: LifeRequirementProfile,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Contact {
    /// One lies on the other.
    Vertical,
    /// Side by side across a border.
    Lateral,
    /// One sits inside the other.
    Embedded,
}

/// A route for material and organisms. The transfer rate arrives with box
/// B (L1); at L0 a link says only that a route exists.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Link {
    pub a: Address,
    pub b: Address,
    pub contact: Contact,
    /// False when an unmapped region makes the contact a guess.
    pub certain: bool,
    pub rate_per_s: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HabitatGraph {
    pub body: Address,
    pub habitats: Vec<Habitat>,
    pub links: Vec<Link>,
}

/// Habitat rules of a pack stack, bottom to top; an id may be defined once.
pub fn rules(stack: &[PackManifest]) -> Result<Vec<HabitatRule>, Vec<String>> {
    let mut out: Vec<HabitatRule> = Vec::new();
    let mut errs = Vec::new();
    let mut owner: BTreeMap<String, String> = BTreeMap::new();
    for p in stack {
        let Some(v) = p.provides.other.get("habitat_rules") else {
            continue;
        };
        let parsed: Result<Vec<HabitatRule>, String> = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("{}: habitat_rules: {e}", p.id));
        match parsed {
            Ok(rs) => {
                for r in rs {
                    if let Some(prev) = owner.insert(r.id.clone(), p.id.clone()) {
                        errs.push(format!(
                            "{}: habitat rule {} is already defined by {prev}",
                            p.id, r.id
                        ));
                    }
                    out.push(r);
                }
            }
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(out)
    } else {
        Err(errs)
    }
}

/// A stack object, seen uniformly.
struct Obj<'a> {
    key: &'a str,
    kind: &'a str,
    what: Object,
    top: f64,
    bottom: f64,
    footprint: Footprint,
    properties: BTreeMap<String, f64>,
    provides: &'a [String],
    extent_area_m2: Option<f64>,
}

fn lat_range(e: &Extent) -> Option<Range> {
    match e {
        Extent::Global => Some(Range {
            lo: -90.0,
            hi: 90.0,
        }),
        Extent::Cap { from_lat_deg: f } if *f >= 0.0 => Some(Range { lo: *f, hi: 90.0 }),
        Extent::Cap { from_lat_deg: f } => Some(Range { lo: -90.0, hi: *f }),
        Extent::Band {
            min_lat_deg,
            max_lat_deg,
        } => Some(Range {
            lo: *min_lat_deg,
            hi: *max_lat_deg,
        }),
        Extent::Unmapped => None,
    }
}

fn band_area(r: f64, lat: Range) -> f64 {
    let rad = m::PI / 180.0;
    2.0 * m::PI * r * r * (m::sin(lat.hi * rad) - m::sin(lat.lo * rad))
}

fn objects(w: &WorldStack) -> Vec<Obj<'_>> {
    let r = w.radius_m;
    let globe = |alt: f64| 4.0 * m::PI * (r + alt) * (r + alt);
    let region_fp = |key: &str| -> Footprint {
        match w.regions.iter().find(|x| x.key == key) {
            Some(x) if x.extent == Extent::Global => Footprint::Global,
            Some(x) => Footprint::Region {
                key: x.key.clone(),
                lat_deg: lat_range(&x.extent),
            },
            None => Footprint::Global,
        }
    };
    let region_area = |key: &str| -> Option<f64> {
        let x = w.regions.iter().find(|x| x.key == key)?;
        let base = match lat_range(&x.extent) {
            Some(l) => band_area(r, l),
            None => globe(0.0) * x.properties.get("area_fraction")?,
        };
        Some(base * x.properties.get("cover_fraction").copied().unwrap_or(1.0))
    };
    let mut out = Vec::new();
    for s in &w.shells {
        out.push(Obj {
            key: &s.key,
            kind: &s.kind,
            what: Object::Shell,
            top: s.top_m,
            bottom: s.bottom_m,
            footprint: s.within.as_deref().map_or(Footprint::Global, region_fp),
            properties: s.properties.clone(),
            provides: &s.provides,
            extent_area_m2: match &s.within {
                None => Some(globe(s.top_m)),
                Some(k) => region_area(k),
            },
        });
    }
    for g in &w.regions {
        let sh = w.shells.iter().find(|s| s.key == g.on);
        let (top, bottom) = (
            g.top_m.or(sh.map(|s| s.top_m)).unwrap_or(0.0),
            g.bottom_m.or(sh.map(|s| s.bottom_m)).unwrap_or(0.0),
        );
        // A region reads its shell's numbers unless it has its own.
        let mut properties = sh.map(|s| s.properties.clone()).unwrap_or_default();
        properties.extend(g.properties.iter().map(|(k, v)| (k.clone(), *v)));
        out.push(Obj {
            key: &g.key,
            kind: &g.kind,
            what: Object::Region,
            top,
            bottom,
            footprint: region_fp(&g.key),
            properties,
            provides: &g.provides,
            extent_area_m2: region_area(&g.key),
        });
    }
    for f in &w.features {
        // A feature stands in the footprint of what contains it.
        let mut c = f.container.as_deref();
        let mut fp = Footprint::Global;
        for _ in 0..w.features.len() + 1 {
            match c {
                Some(k) if w.regions.iter().any(|x| x.key == k) => {
                    fp = region_fp(k);
                    break;
                }
                Some(k) => match (
                    w.shells.iter().find(|s| s.key == k),
                    w.features.iter().find(|x| x.key == k),
                ) {
                    (Some(s), _) => {
                        fp = s.within.as_deref().map_or(Footprint::Global, region_fp);
                        break;
                    }
                    (None, Some(p)) => c = p.container.as_deref(),
                    _ => break,
                },
                None => break,
            }
        }
        out.push(Obj {
            key: &f.key,
            kind: &f.kind,
            what: Object::Feature,
            top: f.altitude_m,
            bottom: f.altitude_m,
            footprint: fp,
            properties: f.properties.clone(),
            provides: &f.provides,
            extent_area_m2: f.properties.get("area_m2").copied(),
        });
    }
    out
}

fn span(p: Place, top: f64, bottom: f64) -> (f64, f64) {
    match p {
        Place::All => (top, bottom),
        Place::Top { depth_m } => (top, (top - depth_m).max(bottom)),
        Place::BelowTop { depth_m } => ((top - depth_m).max(bottom), bottom),
        Place::UnderBottom { depth_m } => (bottom, bottom - depth_m),
        Place::OnTop { up_m } => (top + up_m, top),
        Place::Under { from_m, to_m } => (top - from_m, top - to_m),
        Place::Around { half_m } => (top + half_m, bottom - half_m),
    }
}

/// The global shell holding altitude `z` from below (its top at or above
/// `z`, its bottom below), ignoring regional shells.
fn shell_below(w: &WorldStack, z: f64) -> Option<&sse_world::Shell> {
    w.shells
        .iter()
        .filter(|s| s.within.is_none() && s.top_m >= z - 1e-6 && s.bottom_m < z - 1e-6)
        .min_by(|a, b| a.top_m.total_cmp(&b.top_m))
}

const CONSUMED: [&str; 13] = [
    "temperature_k",
    "temperature_min_k",
    "temperature_max_k",
    "temperature_swing_k",
    "pressure_pa",
    "pressure_min_pa",
    "pressure_max_pa",
    "light_w_m2",
    "light_peak_nm",
    "chemical_w_m2",
    "tidal_w_m2",
    "radiogenic_w_m2",
    "radiation_dose_msv_yr",
];

fn range(p: &BTreeMap<String, f64>, one: &str, lo: &str, hi: &str) -> Option<Range> {
    match (p.get(one), p.get(lo), p.get(hi)) {
        (Some(v), _, _) => Some(Range { lo: *v, hi: *v }),
        (None, Some(a), Some(b)) => Some(Range { lo: *a, hi: *b }),
        _ => None,
    }
}

fn profile(
    address: &Address,
    rule: &HabitatRule,
    o: &Obj,
    (radius_m, top_m, bottom_m): (f64, f64, f64),
    chemical_below: bool,
) -> LifeRequirementProfile {
    let p = &o.properties;
    let mut channels: BTreeSet<String> = rule.channels.iter().cloned().collect();
    if chemical_below {
        channels.insert("chemical".into());
    }
    let energy = EnergyChannels {
        available: channels.into_iter().collect(),
        light_w_m2: p.get("light_w_m2").copied(),
        light_peak_nm: p.get("light_peak_nm").copied(),
        chemical_w_m2: p.get("chemical_w_m2").copied(),
        tidal_w_m2: p.get("tidal_w_m2").copied(),
        radiogenic_w_m2: p.get("radiogenic_w_m2").copied(),
    };
    let area = o.extent_area_m2;
    // A global layer is a spherical shell; a regional one is its area times
    // its thickness.
    let volume = match (rule.medium.as_str(), &o.footprint) {
        ("surface", _) => None,
        (_, Footprint::Global) => {
            let (a, b) = (radius_m + top_m, radius_m + bottom_m);
            Some(4.0 / 3.0 * m::PI * (a * a * a - b * b * b))
        }
        _ => area.map(|a| a * (top_m - bottom_m)),
    }
    .filter(|v| *v > 0.0);
    let mut materials = BTreeMap::new();
    let mut properties = BTreeMap::new();
    for (k, v) in p {
        if let Some(e) = k.strip_prefix("material.") {
            materials.insert(e.to_string(), *v);
        } else if !CONSUMED.contains(&k.as_str())
            && ![
                "persistence_s",
                "shielding_kg_m2",
                "area_fraction",
                "cover_fraction",
                "area_m2",
            ]
            .contains(&k.as_str())
        {
            properties.insert(k.clone(), *v);
        }
    }
    let mut prof = LifeRequirementProfile {
        habitat: address.clone(),
        kind: rule.habitat.clone(),
        medium: rule.medium.clone(),
        phase: rule.phase.clone(),
        solvent: rule.solvent.clone(),
        temperature_k: range(p, "temperature_k", "temperature_min_k", "temperature_max_k"),
        temperature_swing_k: p.get("temperature_swing_k").copied(),
        pressure_pa: range(p, "pressure_pa", "pressure_min_pa", "pressure_max_pa"),
        energy,
        materials,
        area_m2: area,
        volume_m3: volume,
        persistence_s: p.get("persistence_s").copied(),
        radiation_dose_msv_yr: p.get("radiation_dose_msv_yr").copied(),
        shielding_kg_m2: p.get("shielding_kg_m2").copied(),
        properties,
        events: Vec::new(),
        missing: Vec::new(),
    };
    let mut missing = Vec::new();
    let mut need = |name: &str, have: bool| {
        if !have {
            missing.push(name.to_string());
        }
    };
    need("temperature_k", prof.temperature_k.is_some());
    need("pressure_pa", prof.pressure_pa.is_some());
    need("area_m2", prof.area_m2.is_some());
    need("persistence_s", prof.persistence_s.is_some());
    need(
        "radiation_dose_msv_yr",
        prof.radiation_dose_msv_yr.is_some(),
    );
    let has = |c: &str| prof.energy.available.iter().any(|x| x == c);
    need(
        "light_w_m2",
        !has("light") || prof.energy.light_w_m2.is_some(),
    );
    need(
        "chemical_w_m2",
        !has("chemical") || prof.energy.chemical_w_m2.is_some(),
    );
    prof.missing = missing;
    prof
}

/// Horizontal relation of two footprints.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Side {
    Overlap { certain: bool },
    Touch,
    Apart,
}
fn side(a: &Footprint, b: &Footprint) -> Side {
    match (a, b) {
        (Footprint::Global, _) | (_, Footprint::Global) => Side::Overlap { certain: true },
        (Footprint::Region { key: x, .. }, Footprint::Region { key: y, .. }) if x == y => {
            Side::Overlap { certain: true }
        }
        (
            Footprint::Region {
                lat_deg: Some(p), ..
            },
            Footprint::Region {
                lat_deg: Some(q), ..
            },
        ) => {
            let gap = p.hi.min(q.hi) - p.lo.max(q.lo);
            if gap > 1e-9 {
                Side::Overlap { certain: true }
            } else if gap >= -1e-9 {
                Side::Touch
            } else {
                Side::Apart
            }
        }
        _ => Side::Overlap { certain: false },
    }
}

fn link(a: &Habitat, b: &Habitat) -> Option<Link> {
    const TOL: f64 = 1e-6;
    let touch_v = (a.bottom_m - b.top_m).abs() <= TOL || (b.bottom_m - a.top_m).abs() <= TOL;
    let overlap_v = a.top_m.min(b.top_m) - a.bottom_m.max(b.bottom_m) > TOL;
    let make = |contact, certain| Link {
        a: a.address.clone(),
        b: b.address.clone(),
        contact,
        certain,
        rate_per_s: None,
    };
    match side(&a.footprint, &b.footprint) {
        Side::Overlap { certain: true } if overlap_v && (a.embedded || b.embedded) => {
            Some(make(Contact::Embedded, true))
        }
        Side::Overlap { certain } if touch_v => Some(make(Contact::Vertical, certain)),
        // Side by side across a border; a point-like embedded habitat has no border.
        Side::Touch if overlap_v && !a.embedded && !b.embedded => {
            Some(make(Contact::Lateral, true))
        }
        _ => None,
    }
}

impl HabitatGraph {
    /// Derive the habitat graph of a world stack. A stack that does not
    /// validate is refused with its problems.
    pub fn derive(w: &WorldStack, rules: &[HabitatRule]) -> Result<Self, Vec<String>> {
        let mut errs: Vec<String> = w.validate().iter().map(|i| i.to_string()).collect();
        let body = w.body_address().map_err(|e| vec![e])?;
        let objs = objects(w);
        let mut habitats: Vec<Habitat> = Vec::new();
        for o in &objs {
            let mut matched = false;
            for r in rules {
                if r.from != o.what || !r.kinds.iter().any(|k| k == o.kind) {
                    continue;
                }
                if r.needs_property
                    .as_ref()
                    .is_some_and(|p| !o.properties.contains_key(p))
                {
                    continue;
                }
                let (top, bottom) = span(r.place, o.top, o.bottom);
                if !r.needs_below.is_empty()
                    && !shell_below(w, bottom).is_some_and(|s| r.needs_below.contains(&s.kind))
                {
                    continue;
                }
                let chemical_below = !r.chemical_from_below.is_empty()
                    && objs.iter().any(|x| {
                        x.key != o.key
                            && x.top <= top + 1e-6
                            && x.provides.iter().any(|p| r.chemical_from_below.contains(p))
                    });
                matched = true;
                let Some(addr) = w
                    .address_of(o.key)
                    .and_then(|a| a.child_key("habitat", &format!("@{}", r.habitat)).ok())
                else {
                    errs.push(format!(
                        "{}: habitat kind {:?} cannot be an address",
                        o.key, r.habitat
                    ));
                    continue;
                };
                if habitats.iter().any(|h| h.address == addr) {
                    errs.push(format!("{}: two rules make habitat {}", o.key, r.habitat));
                    continue;
                }
                habitats.push(Habitat {
                    profile: profile(&addr, r, o, (w.radius_m, top, bottom), chemical_below),
                    address: addr,
                    kind: r.habitat.clone(),
                    source: o.key.to_string(),
                    rule: r.id.clone(),
                    top_m: top,
                    bottom_m: bottom,
                    footprint: o.footprint.clone(),
                    embedded: r.embedded,
                });
            }
            // An object the author marked as a habitat but no rule covers is
            // still listed, as itself, so authored habitats are never dropped.
            if !matched && o.provides.iter().any(|p| p == "habitat") {
                let r = HabitatRule {
                    id: "authored".into(),
                    habitat: o.kind.to_string(),
                    from: o.what,
                    kinds: vec![o.kind.to_string()],
                    place: Place::All,
                    medium: "unknown".into(),
                    phase: "unknown".into(),
                    solvent: "unknown".into(),
                    channels: vec![],
                    chemical_from_below: vec![],
                    needs_below: vec![],
                    needs_property: None,
                    embedded: false,
                    about: "marked as a habitat by its author; no pack rule covers this kind"
                        .into(),
                };
                if let Some(addr) = w
                    .address_of(o.key)
                    .and_then(|a| a.child_key("habitat", &format!("@{}", r.habitat)).ok())
                {
                    habitats.push(Habitat {
                        profile: profile(&addr, &r, o, (w.radius_m, o.top, o.bottom), false),
                        address: addr,
                        kind: r.habitat.clone(),
                        source: o.key.to_string(),
                        rule: r.id.clone(),
                        top_m: o.top,
                        bottom_m: o.bottom,
                        footprint: o.footprint.clone(),
                        embedded: false,
                    });
                }
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        let mut links = Vec::new();
        for (i, a) in habitats.iter().enumerate() {
            for b in &habitats[i + 1..] {
                links.extend(link(a, b));
            }
        }
        Ok(HabitatGraph {
            body,
            habitats,
            links,
        })
    }

    pub fn hash(&self) -> ContentHash {
        ContentHash::of(self).expect("graph holds no NaN")
    }

    pub fn profiles(&self) -> Vec<LifeRequirementProfile> {
        self.habitats.iter().map(|h| h.profile.clone()).collect()
    }

    /// Habitat by kind and source key.
    pub fn get(&self, source: &str, kind: &str) -> Option<&Habitat> {
        self.habitats
            .iter()
            .find(|h| h.source == source && h.kind == kind)
    }

    /// Is there a link between the habitats at these two (source, kind) places?
    pub fn linked(&self, a: (&str, &str), b: (&str, &str)) -> Option<&Link> {
        let (x, y) = (&self.get(a.0, a.1)?.address, &self.get(b.0, b.1)?.address);
        self.links
            .iter()
            .find(|l| (&l.a == x && &l.b == y) || (&l.a == y && &l.b == x))
    }

    /// One line per habitat and per link, for reading and for tests.
    pub fn summary(&self) -> Vec<String> {
        let name = |a: &Address| {
            self.habitats
                .iter()
                .find(|h| &h.address == a)
                .map_or(a.to_string(), |h| format!("{}/{}", h.source, h.kind))
        };
        let mut out: Vec<String> = self
            .habitats
            .iter()
            .map(|h| format!("{}/{} {} to {} m", h.source, h.kind, h.top_m, h.bottom_m))
            .collect();
        out.extend(self.links.iter().map(|l| {
            format!(
                "{} -- {} ({:?}{})",
                name(&l.a),
                name(&l.b),
                l.contact,
                if l.certain { "" } else { ", guessed" }
            )
        }));
        out
    }
}
