//! Importing community sector files (X3c, handoff 7A part 4): every world
//! line is mapped through its code sheet to a plan (X3b) and a set of
//! authored facts, with a plain report of what was imported, what is
//! impossible (kept and marked, never altered), what was guessed, and where
//! the listed codes differ from what the physics reads back. Nothing here
//! writes: the kernel turns a preview into authored events.

use crate::plan::{plan, Edit, Plan};
use crate::{CodeSheet, Codes};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::Address;
use sse_sector_io::{SectorFile, WorldLine};
use sse_sift::Subject;
use std::collections::BTreeMap;

/// How a sheet reads sector files (pack data, `CodeSheet.import`).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ImportHints {
    /// Layout id to world-line column: `uwp`, `pbg`.
    pub lines: BTreeMap<String, String>,
    /// The layout a line must have to be imported (the world profile).
    pub profile: String,
    /// Rule group id to the column listing the same codes: `remarks`,
    /// `zone`, `bases`. The listed codes are kept as facts and compared.
    #[serde(default)]
    pub listed: BTreeMap<String, String>,
    /// Prefix for the listed facts (`traveller.listed.`).
    pub fact_prefix: String,
    /// Columns that are the referee's call rather than physics (a travel
    /// zone, bases), written as authored facts the sheet's rules then read.
    #[serde(default)]
    pub facts: Vec<ColumnFact>,
}

/// One column written as a fact: `field` takes the first item as text, or
/// with `{}` in it, one true flag per item (`traveller.bases.{}`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ColumnFact {
    pub column: String,
    pub field: String,
}

/// One world line, imported.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportedWorld {
    pub hex: String,
    pub name: String,
    /// The system and its main world.
    pub system: String,
    pub main_world: String,
    /// The original line.
    pub raw: String,
    /// The plan (codes as pins, orbit, counts), when the line could be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<Plan>,
    /// Facts on the system itself (map position, listed stars).
    pub system_edits: Vec<Edit>,
    /// Facts on the main world: the plan's edits and the listed columns.
    pub world_edits: Vec<Edit>,
    /// Why the line was not imported, when it was not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
    /// What had to be assumed, in plain words.
    pub guessed: Vec<String>,
    /// Groups whose listed codes differ from those read back from the facts.
    pub differs: Vec<Difference>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Difference {
    pub group: String,
    /// Listed in the file but not read from the facts.
    pub listed_only: Vec<String>,
    /// Read from the facts but not listed.
    pub read_only: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Import {
    pub sheet: String,
    pub sector: Option<String>,
    pub worlds: Vec<ImportedWorld>,
    /// The parser's warnings, as "line N: text".
    pub warnings: Vec<String>,
    pub summary: Summary,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Summary {
    pub lines: u32,
    pub imported: u32,
    pub refused: u32,
    /// Worlds with at least one impossible fact.
    pub impossible: u32,
    /// Worlds with at least one guess.
    pub guessed: u32,
    /// Worlds whose listed codes differ from those read back.
    pub differ: u32,
}

impl Summary {
    /// "12 lines: 11 imported, 1 refused; 2 impossible, 1 guessed, 3 differ."
    pub fn sentence(&self) -> String {
        format!(
            "{} lines: {} imported, {} refused; {} with impossible values (kept and marked), {} with guesses, {} whose listed codes differ from the physics",
            self.lines, self.imported, self.refused, self.impossible, self.guessed, self.differ
        )
    }
}

fn column<'a>(w: &'a WorldLine, name: &str) -> Vec<String> {
    let one = |s: &'a str| {
        if s.is_empty() {
            vec![]
        } else {
            vec![s.to_string()]
        }
    };
    match name {
        "uwp" => one(&w.uwp),
        "pbg" => one(&w.pbg),
        "name" => one(&w.name),
        "hex" => one(&w.hex),
        "allegiance" => one(&w.allegiance),
        "remarks" => w.remarks.clone(),
        "bases" => w.bases.clone(),
        "stars" => w.stars.clone(),
        "zone" => w.zone.map(|c| vec![c.to_string()]).unwrap_or_default(),
        _ => vec![],
    }
}

fn edit(field: &str, value: Value) -> Edit {
    Edit {
        field: field.into(),
        value,
        impossible: false,
        reason: None,
    }
}

fn texts(v: &[String]) -> Value {
    Value::Array(v.iter().map(|s| Value::Text(s.clone())).collect())
}

/// Codes a rule group can give (its tags with the prefix taken off).
fn known_codes(sheet: &CodeSheet, group: &str) -> Vec<String> {
    let Some(g) = sheet.groups.iter().find(|g| g.id == group) else {
        return vec![];
    };
    let mut out: Vec<String> = sheet
        .rules
        .iter()
        .filter_map(|r| r.tag.strip_prefix(g.prefix.as_str()).map(str::to_string))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// sqrt(3) / 2: the column spacing of flat-topped hexes one unit apart.
const COLUMN_STEP: f64 = 0.866_025_403_784_438_6;
/// Half-thickness of the slab flat maps are lifted into, parsecs.
const SLAB_PC: f64 = 0.25;

/// Where a mapped system sits, parsecs from the sector's corner (handoff 7A
/// part 3): one hex is one parsec, even columns half a hex lower, and a
/// small deterministic offset across the map plane so the sky and 3D views
/// have depth. Authored, like the hex itself.
fn map_position(universe: &Address, w: &WorldLine) -> Option<(f64, f64, f64)> {
    let (c, r) = w.xy()?;
    let x = c as f64 * COLUMN_STEP;
    let y = r as f64 + if c % 2 == 0 { 0.5 } else { 0.0 };
    let u = sse_core::draw::draw(
        &sse_core::draw::Seed::from_text(&universe.to_string()),
        universe,
        &sse_core::draw::Stream::new("codes.map_depth.v1").expect("a fixed stream"),
        u64::from(c) * 1000 + u64::from(r),
    ) as f64
        / 18_446_744_073_709_551_616.0;
    Some((x, y, (u * 2.0 - 1.0) * SLAB_PC))
}

/// A system address for a hex: `<universe>/sys:<hex>`, or the line number
/// when the line had no hex.
fn system_address(universe: &Address, hex: &str, n: usize) -> Result<Address, String> {
    let key = if hex.is_empty() {
        format!("line{n}")
    } else {
        hex.to_string()
    };
    universe
        .child_key("sys", &key)
        .map_err(|e| format!("no address for {key}: {e}"))
}

/// Map a parsed sector file through `sheet` into a preview. `universe` is
/// the root the systems go under; seeds are `<universe>:<hex>`.
pub fn import(
    codes: &Codes,
    sheet: &CodeSheet,
    file: &SectorFile,
    universe: &Address,
) -> Result<Import, String> {
    let hints = sheet
        .import
        .clone()
        .ok_or_else(|| format!("code sheet {} says nothing about sector files", sheet.id))?;
    let mut worlds = Vec::new();
    let mut summary = Summary::default();
    for (n, w) in file.worlds.iter().enumerate() {
        summary.lines += 1;
        let sys = system_address(universe, &w.hex, n + 1)?;
        let main = sys.authored_child("body", 1).map_err(|e| format!("{e}"))?;
        let mut out = ImportedWorld {
            hex: w.hex.clone(),
            name: w.name.clone(),
            system: sys.to_string(),
            main_world: main.to_string(),
            raw: w.raw.clone(),
            plan: None,
            system_edits: vec![],
            world_edits: vec![],
            refused: None,
            guessed: vec![],
            differs: vec![],
        };
        // Map position and listed stars on the system.
        if !w.hex.is_empty() {
            out.system_edits
                .push(edit("map.hex", Value::Text(w.hex.clone())));
        }
        if let Some((x, y, z)) = map_position(universe, w) {
            out.system_edits.push(edit("map.x_pc", Value::Float(x)));
            out.system_edits.push(edit("map.y_pc", Value::Float(y)));
            out.system_edits.push(edit("map.z_pc", Value::Float(z)));
        }
        if let Some(s) = &file.name {
            out.system_edits
                .push(edit("map.sector", Value::Text(s.clone())));
        }
        if let Some(s) = &file.abbreviation {
            out.system_edits
                .push(edit("map.sector_abbr", Value::Text(s.clone())));
        }
        if !w.name.is_empty() {
            out.system_edits
                .push(edit("name", Value::Text(w.name.clone())));
        }
        if !w.stars.is_empty() {
            out.system_edits.push(edit("map.stars", texts(&w.stars)));
        }
        // Code lines: a line holding '?' is not a pin (unknown is not a value).
        let mut lines: Vec<(String, String)> = Vec::new();
        for (layout, col) in &hints.lines {
            if let Some(text) = column(w, col).into_iter().next() {
                if text.contains('?') {
                    out.guessed.push(format!(
                        "{layout} {text} has unknown digits; left to the physics"
                    ));
                } else {
                    lines.push((layout.clone(), text));
                }
            }
        }
        let stars: Vec<&str> = w.stars.iter().map(String::as_str).collect();
        let readable: Vec<&&str> = stars
            .iter()
            .filter(|s| crate::plan::star(s).is_some())
            .collect();
        if stars.is_empty() {
            out.guessed
                .push("no stellar data: a star of one solar mass assumed".into());
        } else if readable.is_empty() {
            out.guessed.push(format!(
                "stellar data {:?} not readable: a star of one solar mass assumed",
                w.stars
            ));
        } else if readable.len() < stars.len() {
            out.guessed.push(format!(
                "only {} of the stars {:?} readable",
                readable.len(),
                w.stars
            ));
        }
        for s in readable.iter().filter_map(|s| crate::plan::star(s)) {
            if let Some(note) = s.note {
                out.guessed.push(note);
            }
        }
        // Listed columns kept as facts.
        let mut extra = BTreeMap::new();
        if !w.name.is_empty() {
            extra.insert("name".to_string(), Value::Text(w.name.clone()));
        }
        for (group, col) in &hints.listed {
            let v = column(w, col);
            extra.insert(format!("{}{group}", hints.fact_prefix), texts(&v));
        }
        for cf in &hints.facts {
            let items = column(w, &cf.column);
            if cf.field.contains("{}") {
                for i in &items {
                    extra.insert(cf.field.replace("{}", i), Value::Bool(true));
                }
            } else if let Some(i) = items.first() {
                extra.insert(cf.field.clone(), Value::Text(i.clone()));
            }
        }
        if !w.allegiance.is_empty() && w.allegiance != "-" {
            extra.insert(
                format!("{}allegiance", hints.fact_prefix),
                Value::Text(w.allegiance.clone()),
            );
        }
        let refs: Vec<(&str, &str)> = lines
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let has_profile = refs.iter().any(|(l, _)| *l == hints.profile);
        if !has_profile {
            out.refused = Some("no readable code line".into());
            summary.refused += 1;
            worlds.push(out);
            continue;
        }
        let seed = format!(
            "{universe}:{}",
            if w.hex.is_empty() { &w.name } else { &w.hex }
        );
        match plan(codes, sheet, &refs, &stars, &extra, &seed) {
            Err(e) => {
                out.refused = Some(e);
                summary.refused += 1;
            }
            Ok(p) => {
                // Read the codes back from the facts written, and compare
                // with what the file lists.
                let facts: BTreeMap<String, Value> = p
                    .edits
                    .iter()
                    .map(|e| (e.field.clone(), e.value.clone()))
                    .collect();
                let back = codes.read(sheet, &Subject::new(facts), None);
                for (group, col) in &hints.listed {
                    let known = known_codes(sheet, group);
                    let listed: Vec<String> = column(w, col)
                        .into_iter()
                        .filter(|c| known.contains(c))
                        .collect();
                    let read: Vec<String> = back
                        .groups
                        .iter()
                        .find(|g| &g.id == group)
                        .map(|g| g.codes().iter().map(|c| c.to_string()).collect())
                        .unwrap_or_default();
                    let listed_only: Vec<String> = listed
                        .iter()
                        .filter(|c| !read.contains(c))
                        .cloned()
                        .collect();
                    let read_only: Vec<String> = read
                        .iter()
                        .filter(|c| !listed.contains(c))
                        .cloned()
                        .collect();
                    if !listed_only.is_empty() || !read_only.is_empty() {
                        out.differs.push(Difference {
                            group: group.clone(),
                            listed_only,
                            read_only,
                        });
                    }
                }
                out.world_edits = p.edits.clone();
                if p.edits.iter().any(|e| e.impossible) {
                    summary.impossible += 1;
                }
                out.plan = Some(p);
                summary.imported += 1;
            }
        }
        if !out.guessed.is_empty() {
            summary.guessed += 1;
        }
        if !out.differs.is_empty() {
            summary.differ += 1;
        }
        worlds.push(out);
    }
    Ok(Import {
        sheet: sheet.id.clone(),
        sector: file.name.clone(),
        worlds,
        warnings: file
            .warnings
            .iter()
            .map(|w| format!("line {}: {}", w.line, w.text))
            .collect(),
        summary,
    })
}

/// The world line for one coded main world, the other way round (export):
/// its code lines and rule groups from a reading, the referee's columns
/// from its facts, and the map facts of its system.
pub fn world_line(
    hints: &ImportHints,
    reading: &crate::Reading,
    world: &BTreeMap<String, Value>,
    system: &BTreeMap<String, Value>,
) -> WorldLine {
    let text = |f: &BTreeMap<String, Value>, k: &str| match f.get(k) {
        Some(Value::Text(t)) => t.clone(),
        _ => String::new(),
    };
    let mut w = WorldLine {
        hex: text(system, "map.hex"),
        name: {
            let n = text(world, "name");
            if n.is_empty() {
                text(system, "name")
            } else {
                n
            }
        },
        stars: match system.get("map.stars") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| match v {
                    Value::Text(t) => Some(t.clone()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        },
        allegiance: text(world, &format!("{}allegiance", hints.fact_prefix)),
        ..Default::default()
    };
    for (layout, col) in &hints.lines {
        if let Some(l) = reading.line(layout) {
            match col.as_str() {
                "uwp" => w.uwp = l.to_string(),
                "pbg" => w.pbg = l.to_string(),
                _ => {}
            }
        }
    }
    for (group, col) in &hints.listed {
        if col == "remarks" {
            if let Some(g) = reading.groups.iter().find(|g| &g.id == group) {
                w.remarks = g.codes().iter().map(|c| c.to_string()).collect();
            }
        }
    }
    for cf in &hints.facts {
        let items: Vec<String> = if let Some((pre, post)) = cf.field.split_once("{}") {
            world
                .iter()
                .filter(|(k, v)| {
                    **v == Value::Bool(true) && k.starts_with(pre) && k.ends_with(post)
                })
                .map(|(k, _)| k[pre.len()..k.len() - post.len()].to_string())
                .collect()
        } else {
            match world.get(&cf.field) {
                Some(Value::Text(t)) => vec![t.clone()],
                _ => vec![],
            }
        };
        match cf.column.as_str() {
            "zone" => {
                w.zone = items
                    .first()
                    .and_then(|z| z.chars().next())
                    .filter(|c| *c == 'A' || *c == 'R')
            }
            "bases" => w.bases = items,
            "remarks" => w.remarks.extend(items),
            _ => {}
        }
    }
    w
}
