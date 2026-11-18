//! World stacks (build plan P4): what is on, in and above a world.
//!
//! Three kinds of object, matching the address segments of handoff H5.1:
//! - **shells**: layers by altitude (core, mantle, crust, ocean, ice shell,
//!   atmosphere). A shell is global, or `within` a region (a polar ice cap, a
//!   sub-ice ocean under it).
//! - **regions**: areas `on` a shell (plates, biomes, basins, shelves, polar
//!   caps, aerostat bands), optionally with their own altitude span.
//! - **features**: things `in` a region, shell or other feature (cities,
//!   volcanoes, magma chambers, aerostats, stations), at an altitude.
//!
//! Altitude is metres from the surface datum, positive up. Every object
//! declares what it `provides` (heat, pressure, a habitat, a dynamo), so the
//! next pipeline down can use it. `ladder()` is the "above, on and within"
//! depth strip of the Strata mockup; `physics_notes()` checks a stack against
//! the atoms (a brine ocean that would freeze is reported, never hidden).
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sse_core::Address;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const FORMAT: &str = "sse.world-stack/1";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Shell {
    pub key: String,
    pub kind: String,
    pub name: String,
    pub top_m: f64,
    pub bottom_m: f64,
    /// Region key when the shell covers only that region.
    #[serde(default)]
    pub within: Option<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, f64>,
    #[serde(default)]
    pub provides: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Extent {
    Global,
    /// Everything poleward of `from_lat_deg` (negative for the south).
    Cap {
        from_lat_deg: f64,
    },
    Band {
        min_lat_deg: f64,
        max_lat_deg: f64,
    },
    /// Unmapped: placed on the shell but without a drawn outline yet.
    Unmapped,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Region {
    pub key: String,
    pub kind: String,
    pub name: String,
    /// Shell key the region lies on.
    pub on: String,
    pub extent: Extent,
    /// Altitude span, when narrower than the shell's.
    #[serde(default)]
    pub top_m: Option<f64>,
    #[serde(default)]
    pub bottom_m: Option<f64>,
    #[serde(default)]
    pub properties: BTreeMap<String, f64>,
    #[serde(default)]
    pub provides: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub key: String,
    pub kind: String,
    /// Optional: an unnamed feature is shown by its kind.
    #[serde(default)]
    pub name: Option<String>,
    /// Region, shell or feature key; none for something in orbit.
    #[serde(default, rename = "in")]
    pub container: Option<String>,
    pub altitude_m: f64,
    #[serde(default)]
    pub lat_deg: Option<f64>,
    #[serde(default)]
    pub lon_deg: Option<f64>,
    #[serde(default)]
    pub properties: BTreeMap<String, f64>,
    #[serde(default)]
    pub provides: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldStack {
    pub format: String,
    /// Address of the body, e.g. `u:demo/sys:@1/body:@3`.
    pub body: String,
    pub name: String,
    pub radius_m: f64,
    #[serde(default)]
    pub shells: Vec<Shell>,
    #[serde(default)]
    pub regions: Vec<Region>,
    #[serde(default)]
    pub features: Vec<Feature>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Issue {
    pub at: String,
    pub message: String,
}
impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.at, self.message)
    }
}

/// Readable phrase for a kind ("lava_tube_city" reads "lava-tube city").
pub fn kind_phrase(kind: &str) -> String {
    match kind {
        "lava_tube_city" => "lava-tube city".into(),
        "magma_chamber" => "magma chamber".into(),
        "ice_shell" => "ice shell".into(),
        "sub_ice_ocean" => "sub-ice ocean".into(),
        "aerostat_band" => "aerostat band".into(),
        "polar_cap" => "polar cap".into(),
        k => k.replace('_', " "),
    }
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// What a `provides` entry adds to a ladder line, if anything.
fn provision_phrase(p: &str) -> Option<&'static str> {
    match p {
        "dynamo" => Some("dynamo on"),
        _ => None,
    }
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct LadderLine {
    /// Displayed depth: "+400 km", "0", "−3 to −11 km".
    pub depth: String,
    pub text: String,
    /// Where: a region or container name, "surface", "shell" or "orbit".
    #[serde(rename = "where")]
    pub place: String,
    /// Key of the object the line is about (empty for the surface summary).
    pub key: String,
}

/// Format an altitude in km the way the depth strip shows it.
pub fn format_km(alt_m: f64) -> String {
    let km = alt_m / 1000.0;
    if km == 0.0 {
        return "0".into();
    }
    let sign = if km > 0.0 { "+" } else { "\u{2212}" };
    let a = km.abs();
    let whole = a == a.trunc();
    let body = if whole {
        let n = a as u64;
        let s = n.to_string();
        let mut out = String::new();
        for (i, ch) in s.chars().enumerate() {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(ch);
        }
        out
    } else {
        format!("{a:.1}")
    };
    format!("{sign}{body}")
}

impl WorldStack {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("world stack: {e}"))
    }

    pub fn body_address(&self) -> Result<Address, String> {
        self.body.parse().map_err(|e| format!("body address: {e}"))
    }

    /// Address of any object by key: shells, regions and features each get an
    /// authored child of the body (`shell:@crust`, `region:@south-pole`, ...).
    pub fn address_of(&self, key: &str) -> Option<Address> {
        let b = self.body_address().ok()?;
        let seg = if self.shells.iter().any(|s| s.key == key) {
            "shell"
        } else if self.regions.iter().any(|r| r.key == key) {
            "region"
        } else if self.features.iter().any(|f| f.key == key) {
            "feature"
        } else {
            return None;
        };
        b.child_key(seg, &format!("@{key}")).ok()
    }

    fn shell(&self, k: &str) -> Option<&Shell> {
        self.shells.iter().find(|s| s.key == k)
    }
    fn region(&self, k: &str) -> Option<&Region> {
        self.regions.iter().find(|r| r.key == k)
    }
    fn feature(&self, k: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.key == k)
    }

    /// Altitude span (top, bottom) of a region: its own, else its shell's.
    fn region_span(&self, r: &Region) -> Option<(f64, f64)> {
        let s = self.shell(&r.on)?;
        Some((r.top_m.unwrap_or(s.top_m), r.bottom_m.unwrap_or(s.bottom_m)))
    }

    /// Every structural problem, all at once.
    pub fn validate(&self) -> Vec<Issue> {
        let mut out = Vec::new();
        let mut issue = |at: &str, m: String| {
            out.push(Issue {
                at: at.into(),
                message: m,
            })
        };
        if self.format != FORMAT {
            issue(
                "format",
                format!("expected {FORMAT}, got {:?}", self.format),
            );
        }
        if self.body_address().is_err() {
            issue("body", format!("{:?} is not an address", self.body));
        }
        if self.radius_m.is_nan() || self.radius_m <= 0.0 {
            issue("radius_m", "must be positive".into());
        }
        let mut keys = BTreeSet::new();
        let all_keys = self
            .shells
            .iter()
            .map(|s| &s.key)
            .chain(self.regions.iter().map(|r| &r.key))
            .chain(self.features.iter().map(|f| &f.key));
        for k in all_keys {
            if !keys.insert(k.clone()) {
                issue(k, "key used twice".into());
            }
            if sse_core::address::Segment::new("shell", &format!("@{k}")).is_err() {
                issue(k, "key may hold only letters, digits and . _ : -".into());
            }
        }
        for s in &self.shells {
            if s.top_m <= s.bottom_m {
                issue(
                    &s.key,
                    format!("top {} m is not above bottom {} m", s.top_m, s.bottom_m),
                );
            }
            if s.bottom_m < -self.radius_m * (1.0 + 1e-9) {
                issue(&s.key, "bottom is below the centre of the world".into());
            }
            if let Some(w) = &s.within {
                if self.region(w).is_none() {
                    issue(&s.key, format!("within unknown region {w:?}"));
                }
            }
        }
        // Global shells must not overlap.
        let mut global: Vec<&Shell> = self.shells.iter().filter(|s| s.within.is_none()).collect();
        global.sort_by(|a, b| b.top_m.total_cmp(&a.top_m));
        for w in global.windows(2) {
            if w[1].top_m > w[0].bottom_m + 1e-6 {
                issue(
                    &w[1].key,
                    format!(
                        "overlaps {} ({} m above its bottom)",
                        w[0].key,
                        w[1].top_m - w[0].bottom_m
                    ),
                );
            }
        }
        for r in &self.regions {
            match self.shell(&r.on) {
                None => issue(&r.key, format!("on unknown shell {:?}", r.on)),
                Some(s) => {
                    let (t, b) = (r.top_m.unwrap_or(s.top_m), r.bottom_m.unwrap_or(s.bottom_m));
                    if t > s.top_m + 1e-6 || b < s.bottom_m - 1e-6 || t <= b {
                        issue(
                            &r.key,
                            format!("span {t}..{b} m is not inside shell {}", s.key),
                        );
                    }
                }
            }
        }
        let highest = self.shells.iter().map(|s| s.top_m).fold(0.0, f64::max);
        for f in &self.features {
            let alt = f.altitude_m;
            match f.container.as_deref() {
                None => {
                    if alt <= highest {
                        issue(
                            &f.key,
                            "has no container but is not above every shell".into(),
                        );
                    }
                }
                Some(c) => {
                    if let Some(r) = self.region(c) {
                        if let Some((t, b)) = self.region_span(r) {
                            if alt > t + 1e-6 || alt < b - 1e-6 {
                                issue(&f.key, format!("altitude {alt} m is outside region {c}"));
                            }
                        }
                    } else if let Some(s) = self.shell(c) {
                        if alt > s.top_m + 1e-6 || alt < s.bottom_m - 1e-6 {
                            issue(&f.key, format!("altitude {alt} m is outside shell {c}"));
                        }
                    } else if let Some(p) = self.feature(c) {
                        if c == f.key {
                            issue(&f.key, "is inside itself".into());
                        } else if alt >= p.altitude_m {
                            issue(&f.key, format!("is in {c} but not beneath it"));
                        }
                    } else {
                        issue(&f.key, format!("in unknown container {c:?}"));
                    }
                }
            }
            if let Some(l) = f.lat_deg {
                if !(-90.0..=90.0).contains(&l) {
                    issue(&f.key, format!("latitude {l} is off the globe"));
                }
            }
        }
        out
    }

    fn container_name(&self, c: &str) -> String {
        if let Some(r) = self.region(c) {
            return r.name.clone();
        }
        if let Some(s) = self.shell(c) {
            return s.name.clone();
        }
        if let Some(f) = self.feature(c) {
            return format!(
                "under {}",
                f.name.clone().unwrap_or_else(|| kind_phrase(&f.kind))
            );
        }
        c.to_owned()
    }

    fn feature_text(f: &Feature, comma: bool) -> String {
        match &f.name {
            Some(n) if comma => format!("{n}, {}", kind_phrase(&f.kind)),
            Some(n) => format!("{n} {}", kind_phrase(&f.kind)),
            None => capitalised(&kind_phrase(&f.kind)),
        }
    }

    /// The "above, on and within" ladder, top to bottom.
    ///
    /// Rules: features above the surface, highest first, each with its
    /// container (or "orbit"); one surface line summarising biomes, cities and
    /// other surface features; below it, features at their altitude, regional
    /// shells at their base (or as a range when they start below the surface),
    /// and global shells at their base boundary ("Crust ends, 9 plates"), the
    /// innermost at its top. Where two global boundary lines fall at the same
    /// depth, the deeper shell's line stands. Shells above the surface are the
    /// setting for features and get no line of their own.
    pub fn ladder(&self) -> Vec<LadderLine> {
        let mut lines = Vec::new();
        let mut above: Vec<&Feature> = self
            .features
            .iter()
            .filter(|f| f.altitude_m > 0.0)
            .collect();
        above.sort_by(|a, b| {
            b.altitude_m
                .total_cmp(&a.altitude_m)
                .then(a.key.cmp(&b.key))
        });
        for f in above {
            lines.push(LadderLine {
                depth: format!("{} km", format_km(f.altitude_m)),
                text: Self::feature_text(f, true),
                place: f
                    .container
                    .as_deref()
                    .map(|c| self.container_name(c))
                    .unwrap_or_else(|| "orbit".into()),
                key: f.key.clone(),
            });
        }
        let mut parts = Vec::new();
        if self.regions.iter().any(|r| r.kind == "biome") {
            parts.push("Biomes".to_string());
        }
        let surface: Vec<&Feature> = self
            .features
            .iter()
            .filter(|f| f.altitude_m == 0.0)
            .collect();
        let cities = surface.iter().filter(|f| f.kind == "city").count();
        if cities > 0 {
            parts.push(format!(
                "{cities} {}",
                if cities == 1 { "city" } else { "cities" }
            ));
        }
        for f in surface.iter().filter(|f| f.kind != "city") {
            parts.push(Self::feature_text(f, false));
        }
        if !parts.is_empty() {
            lines.push(LadderLine {
                depth: "0".into(),
                text: parts.join(", "),
                place: "surface".into(),
                key: String::new(),
            });
        }
        // Below the surface: (sort top, sort bottom, line).
        let mut below: Vec<(f64, f64, LadderLine)> = Vec::new();
        for f in self.features.iter().filter(|f| f.altitude_m < 0.0) {
            below.push((
                f.altitude_m,
                f.altitude_m,
                LadderLine {
                    depth: format!("{} km", format_km(f.altitude_m)),
                    text: Self::feature_text(f, true),
                    place: f
                        .container
                        .as_deref()
                        .map(|c| self.container_name(c))
                        .unwrap_or_default(),
                    key: f.key.clone(),
                },
            ));
        }
        for s in self
            .shells
            .iter()
            .filter(|s| s.within.is_some() && s.bottom_m < 0.0)
        {
            let place = self.container_name(s.within.as_deref().unwrap_or_default());
            let (depth, key_top) = if s.top_m >= 0.0 {
                (format!("{} km", format_km(s.bottom_m)), s.bottom_m)
            } else {
                (
                    format!("{} to {} km", format_km(s.top_m), format_km(s.bottom_m)),
                    s.top_m,
                )
            };
            below.push((
                key_top,
                s.bottom_m,
                LadderLine {
                    depth,
                    text: s.name.clone(),
                    place,
                    key: s.key.clone(),
                },
            ));
        }
        let mut global: Vec<&Shell> = self
            .shells
            .iter()
            .filter(|s| s.within.is_none() && s.bottom_m < 0.0)
            .collect();
        global.sort_by(|a, b| b.top_m.total_cmp(&a.top_m));
        let mut boundary: BTreeMap<i64, (f64, LadderLine)> = BTreeMap::new();
        for (i, s) in global.iter().enumerate() {
            let innermost = i + 1 == global.len();
            let mut extras: Vec<String> = Vec::new();
            let plates = self
                .regions
                .iter()
                .filter(|r| r.on == s.key && r.kind == "plate")
                .count();
            if plates > 0 {
                extras.push(format!("{plates} plates"));
            }
            extras.extend(
                s.provides
                    .iter()
                    .filter_map(|p| provision_phrase(p))
                    .map(String::from),
            );
            let (at, mut text) = if innermost {
                (s.top_m, s.name.clone())
            } else {
                (s.bottom_m, format!("{} ends", s.name))
            };
            if at >= 0.0 {
                continue;
            }
            for e in extras {
                text = format!("{text}, {e}");
            }
            // Millimetre buckets: lines at the same depth collide; the deeper shell (later) wins.
            boundary.insert(
                (at * 1000.0) as i64,
                (
                    at,
                    LadderLine {
                        depth: format!("{} km", format_km(at)),
                        text,
                        place: "shell".into(),
                        key: s.key.clone(),
                    },
                ),
            );
        }
        for (_, (at, line)) in boundary {
            below.push((at, at, line));
        }
        below.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.total_cmp(&a.1)));
        lines.extend(below.into_iter().map(|(_, _, l)| l));
        lines
    }

    /// Check the stack's physics with the atoms. Returns notes (what was
    /// computed) and problems (what physics says cannot be).
    pub fn physics_notes(&self) -> (Vec<String>, Vec<Issue>) {
        let atoms = sse_atoms::registry();
        let eval = |id: &str, inputs: &[f64]| -> Option<f64> {
            let a = atoms.get(id)?;
            let p: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
            a.eval(inputs, &p).ok().map(|v| v[0])
        };
        let mut notes = Vec::new();
        let mut problems = Vec::new();
        for s in &self.shells {
            let (Some(w), Some(t)) = (
                s.properties.get("salinity"),
                s.properties.get("temperature_k"),
            ) else {
                continue;
            };
            let Some(tf) = eval("sse.ocean.brine_freezing_point", &[*w]) else {
                continue;
            };
            notes.push(format!(
                "{}: freezes at {:.1} K (sse.ocean.brine_freezing_point@1); at {:.0} K it is {}",
                s.key,
                tf,
                t,
                if *t > tf { "liquid" } else { "frozen" }
            ));
            if *t <= tf {
                problems.push(Issue {
                    at: s.key.clone(),
                    message: format!(
                        "at {t} K a brine of salinity {w} is frozen (freezes at {tf:.1} K)"
                    ),
                });
            }
            // An ice lid directly above, in the same region, conducts the heat out.
            let lid = self.shells.iter().find(|l| {
                l.kind == "ice_shell" && l.within == s.within && (l.bottom_m - s.top_m).abs() < 1e-6
            });
            if let Some(l) = lid {
                if let Some(ts) = l.properties.get("top_temperature_k") {
                    if let Some(q) = eval(
                        "sse.heat.ice_shell_heat_flow",
                        &[l.top_m - l.bottom_m, *ts, tf],
                    ) {
                        notes.push(format!(
                            "{}: holding a {:.1} km lid at {:.0} K over a base at {:.1} K needs {:.3} W/m² from below (sse.heat.ice_shell_heat_flow@1)",
                            l.key,
                            (l.top_m - l.bottom_m) / 1000.0,
                            ts,
                            tf,
                            q
                        ));
                    }
                }
            }
        }
        (notes, problems)
    }
}

#[cfg(test)]
mod tests;
