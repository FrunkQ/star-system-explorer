//! SSE4 code sheets: a game's short codes for a world, read from the facts.
//!
//! A code sheet is pack data (`provides.other.code_sheets`). It names a few
//! **segments**, each one symbol long, and **layouts** that string the
//! symbols into lines such as a world profile (`A867974-C`). Every segment
//! is read by machinery that already exists:
//!
//! - `classify`: a classification scheme (K8, C1) whose classes each carry
//!   a symbol and a value. The symbol's "why" is the atlas's own label, with
//!   the facts it read and the near misses.
//! - `leading`: the leading digit of a fact at the scale another segment
//!   settled (a population multiplier).
//! - `count`: how many bodies in the same system carry a rule tag (belts,
//!   gas giants).
//!
//! Before the segments, `prepare` rules (K8 rules) may turn physical facts
//! into flag facts the classes read ("this air carries enough oxygen").
//! After them, each segment's value is written as a fact
//! `<fact_prefix><segment>`, and `groups` of rules read those values and the
//! physical facts to put tags on the world (trade codes, travel zones,
//! bases). Nothing here knows about any one game: a game-system pack is the
//! sheet, the atlas and the rules.
//!
//! The way back: [`pins`] turns a code line into ranges on the facts (each
//! symbol's class cores) and a typical value for each, which is what a
//! constrained resolve starts from (X3b), and [`typical_world`] builds the
//! plainest world that reads back to the same code.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_pack::PackManifest;
use sse_sift::rules::TagState;
use sse_sift::{Atlas, Bound, Label, RuleSet, SiftContent, Subject};
use std::collections::{BTreeMap, BTreeSet};

pub mod import;
pub mod mix;
pub mod plan;
pub mod roll;

#[cfg(test)]
mod import_tests;
#[cfg(test)]
mod plan_tests;
#[cfg(test)]
mod tests;

/// The extended-hexadecimal digits game codes use: 0 to 9, then letters
/// without I and O (which read as 1 and 0), up to 33.
pub const EHEX: &str = "0123456789ABCDEFGHJKLMNPQRSTUVWXYZ";

/// The symbol for a whole number in extended hex (None past 33 or below 0).
pub fn ehex(v: i64) -> Option<char> {
    usize::try_from(v).ok().and_then(|i| EHEX.chars().nth(i))
}

/// The value of an extended-hex symbol (case-insensitive).
pub fn ehex_value(c: char) -> Option<i64> {
    let u = c.to_ascii_uppercase();
    EHEX.chars().position(|x| x == u).map(|i| i as i64)
}

/// One class's symbol and value inside a `classify` segment.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Symbol {
    pub class: String,
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Facts (in fact units) that a plain world with this symbol has, beyond
    /// the class's numeric `typical` values: text facts such as the main gas.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub plain: BTreeMap<String, Value>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "kebab-case")]
pub enum Source {
    /// A scheme's base label, through a class-to-symbol table.
    Classify {
        scheme: String,
        symbols: Vec<Symbol>,
    },
    /// The leading digit of `field` at the power of ten that `segment`'s
    /// value names (population 8.1 billion with exponent 9 gives 8).
    Leading { field: String, segment: String },
    /// Bodies in the same system (the world itself included) that carry
    /// `tag` from the stack's rules, capped at `max`.
    Count { tag: String, max: i64 },
    /// A fact read as a number (an extended profile's line: diameter in km,
    /// gravity in g): `field` times `times`, over the sheet constant `per`
    /// (an id of `sse_core::sheet::ALL`, such as `g0`, `atm` or `day`), shown
    /// to `digits` significant figures. Read only: never a pin for planning.
    Value {
        field: String,
        #[serde(default = "one")]
        times: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        per: Option<String>,
        #[serde(default = "three")]
        digits: u32,
    },
}

fn one() -> f64 {
    1.0
}
fn three() -> u32 {
    3
}

/// `x` to `digits` significant figures, plainly: no exponent between a
/// millionth and a billion, whole numbers without a point.
pub fn sig_figures(x: f64, digits: u32) -> String {
    if x == 0.0 || !x.is_finite() {
        return if x == 0.0 { "0".into() } else { UNKNOWN.into() };
    }
    let digits = digits.clamp(1, 12) as i32;
    let mut mag = sse_core::math::floor(sse_core::math::log10(x.abs())) as i32;
    let round_at = |mag: i32| {
        let step = sse_core::math::pow(10.0, f64::from(mag - digits + 1));
        sse_core::math::round(x / step) * step
    };
    let mut rounded = round_at(mag);
    // 0.99997 to three figures is 1.00, a power of ten up.
    if rounded.abs() >= sse_core::math::pow(10.0, f64::from(mag + 1)) {
        mag += 1;
        rounded = round_at(mag);
    }
    if !(-6..9).contains(&mag) {
        return format!("{:.*e}", (digits - 1) as usize, x);
    }
    let decimals = (digits - 1 - mag).max(0) as usize;
    format!("{rounded:.decimals$}")
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    #[serde(default)]
    pub label: String,
    pub source: Source,
    /// An authored fact holding the symbol itself. When present the code
    /// wins over the physics (an imported or hand-set code), and the reading
    /// keeps what the physics would have given beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored: Option<String>,
}

/// A family of tags read after the segments (trade codes, zones, bases):
/// every rule in the stack whose tag starts with `prefix`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    #[serde(default)]
    pub label: String,
    pub prefix: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CodeSheet {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub about: String,
    /// Rule tag prefixes run first; each such rule's tag becomes a flag fact
    /// (true or false) that classes can read.
    #[serde(default)]
    pub prepare: Vec<String>,
    pub segments: Vec<Segment>,
    /// Layout id to template: `{segment}` is one segment's symbol, any other
    /// character is written as it stands.
    pub layouts: BTreeMap<String, String>,
    /// Segment values are published to the groups as `<fact_prefix><segment id>`.
    pub fact_prefix: String,
    #[serde(default)]
    pub groups: Vec<Group>,
    /// The sheet's own rules (K8 rules): prepare flags, counts and groups.
    /// Kept here, not in the stack's `rules`, so they tag nothing outside a
    /// code reading.
    #[serde(default)]
    pub rules: Vec<sse_sift::Rule>,
    /// How a coded world's system is grown (X3b), if the sheet supports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<plan::SystemHints>,
    /// How community sector files map onto the sheet (X3c), if they do.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import: Option<import::ImportHints>,
}

/// Why a segment shows its symbol.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Why {
    /// The scheme's label; `defaulted` lists the facts that were absent and
    /// read as the axis's stated missing value.
    Classified {
        label: Label,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        defaulted: Vec<String>,
    },
    /// No class fits: the symbol is the sheet's unknown mark.
    Unclassified {
        label: Label,
    },
    /// An authored code. `physics` is what the facts would give.
    Authored {
        field: String,
        physics: Option<String>,
    },
    Leading {
        field: String,
        value: f64,
        exponent: i64,
    },
    Counted {
        tag: String,
        members: Vec<String>,
    },
    /// A count segment read without the rest of the system.
    NoSystem {
        tag: String,
    },
    /// A fact read as a number; `value` is the fact before scaling, absent
    /// when the world does not have it (the symbol is then the unknown mark).
    Value {
        field: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        per: Option<String>,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SegmentReading {
    pub id: String,
    pub symbol: String,
    pub value: Option<f64>,
    pub why: Why,
}

impl SegmentReading {
    /// True when an authored code disagrees with what the facts give.
    pub fn disagrees(&self) -> bool {
        matches!(&self.why, Why::Authored { physics: Some(p), .. } if *p != self.symbol)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GroupReading {
    pub id: String,
    /// The short code (tag with the prefix taken off) and the rule's evidence.
    pub tags: Vec<(String, TagState)>,
}

impl GroupReading {
    pub fn codes(&self) -> Vec<&str> {
        self.tags.iter().map(|(c, _)| c.as_str()).collect()
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Reading {
    pub sheet: String,
    pub lines: BTreeMap<String, String>,
    pub segments: Vec<SegmentReading>,
    pub groups: Vec<GroupReading>,
    /// Flag facts the prepare rules set, with evidence.
    pub prepared: Vec<TagState>,
}

impl Reading {
    pub fn segment(&self, id: &str) -> Option<&SegmentReading> {
        self.segments.iter().find(|s| s.id == id)
    }
    pub fn group(&self, id: &str) -> Option<&GroupReading> {
        self.groups.iter().find(|g| g.id == id)
    }
    pub fn line(&self, layout: &str) -> Option<&str> {
        self.lines.get(layout).map(String::as_str)
    }
}

/// The unknown mark for a segment no class fits.
pub const UNKNOWN: &str = "?";

/// Sheets and sift content from a pack stack.
#[derive(Clone, Debug, Default)]
pub struct Codes {
    pub sheets: Vec<CodeSheet>,
    pub sift: SiftContent,
    /// Gas mixes from every pack (`provides.other.gas_mixes`), in stack order.
    pub mixes: Vec<mix::GasMix>,
    /// Rolled profiles from every pack (`provides.other.rolls`), in stack order.
    pub rolls: Vec<roll::Program>,
}

impl Codes {
    pub fn load(stack: &[PackManifest]) -> Result<Codes, Vec<String>> {
        let sift = sse_sift::load(stack)?;
        let mut sheets: Vec<CodeSheet> = Vec::new();
        let mut errs = Vec::new();
        for p in stack {
            let Some(v) = p.provides.other.get("code_sheets") else {
                continue;
            };
            match serde_json::to_value(v).and_then(serde_json::from_value::<Vec<CodeSheet>>) {
                Ok(ss) => {
                    for s in ss {
                        if sheets.iter().any(|o| o.id == s.id) {
                            errs.push(format!("{}: code sheet {} is already defined", p.id, s.id));
                        }
                        sheets.push(s);
                    }
                }
                Err(e) => errs.push(format!("{}: code_sheets: {e}", p.id)),
            }
        }
        let mut mixes: Vec<mix::GasMix> = Vec::new();
        for p in stack {
            let Some(v) = p.provides.other.get("gas_mixes") else {
                continue;
            };
            match serde_json::to_value(v).and_then(serde_json::from_value::<Vec<mix::GasMix>>) {
                Ok(ms) => {
                    for m in ms {
                        if mixes.iter().any(|o| o.id == m.id) {
                            errs.push(format!("{}: gas mix {} is already defined", p.id, m.id));
                        }
                        mixes.push(m);
                    }
                }
                Err(e) => errs.push(format!("{}: gas_mixes: {e}", p.id)),
            }
        }
        let mut rolls: Vec<roll::Program> = Vec::new();
        for p in stack {
            let Some(v) = p.provides.other.get("rolls") else {
                continue;
            };
            match serde_json::to_value(v).and_then(serde_json::from_value::<Vec<roll::Program>>) {
                Ok(rs) => {
                    for r in rs {
                        if rolls.iter().any(|o| o.id == r.id) {
                            errs.push(format!(
                                "{}: rolled profile {} is already defined",
                                p.id, r.id
                            ));
                        }
                        rolls.push(r);
                    }
                }
                Err(e) => errs.push(format!("{}: rolls: {e}", p.id)),
            }
        }
        let out = Codes {
            sheets,
            sift,
            mixes,
            rolls,
        };
        for r in &out.rolls {
            match out.sheet(&r.sheet) {
                None => errs.push(format!(
                    "rolled profile {}: no code sheet {}",
                    r.id, r.sheet
                )),
                Some(sh) => {
                    for st in &r.steps {
                        let segs = st
                            .add
                            .iter()
                            .chain(st.dm.keys())
                            .chain(st.zero_when.iter().chain(&st.only_when).map(|c| &c.segment))
                            .chain(
                                st.floors
                                    .iter()
                                    .flat_map(|f| f.when.iter().map(|c| &c.segment)),
                            );
                        for g in segs {
                            let earlier = r
                                .steps
                                .iter()
                                .take_while(|x| x.id != st.id)
                                .any(|x| &x.id == g);
                            if !earlier && !sh.segments.iter().any(|x| &x.id == g) {
                                errs.push(format!("rolled profile {}: step {} reads {g}, which is not a segment of {}", r.id, st.id, r.sheet));
                            }
                        }
                    }
                }
            }
        }
        for s in &out.sheets {
            if let Err(e) = out.check(s) {
                errs.push(e);
            }
        }
        if errs.is_empty() {
            Ok(out)
        } else {
            Err(errs)
        }
    }

    pub fn sheet(&self, id: &str) -> Option<&CodeSheet> {
        self.sheets.iter().find(|s| s.id == id)
    }

    /// A sheet must name schemes and classes that exist, give each class one
    /// one-character symbol, and use only its own segments in its layouts.
    pub fn check(&self, s: &CodeSheet) -> Result<(), String> {
        let ids: BTreeSet<&str> = s.segments.iter().map(|g| g.id.as_str()).collect();
        if ids.len() != s.segments.len() {
            return Err(format!("code sheet {}: a segment id is used twice", s.id));
        }
        let atlas = &self.sift.atlas;
        for g in &s.segments {
            match &g.source {
                Source::Classify { scheme, symbols } => {
                    if !atlas.classes.iter().any(|c| &c.scheme == scheme) {
                        return Err(format!(
                            "code sheet {}: scheme {scheme} has no classes",
                            s.id
                        ));
                    }
                    let mut seen = BTreeSet::new();
                    for y in symbols {
                        if !atlas
                            .classes
                            .iter()
                            .any(|c| c.id == y.class && &c.scheme == scheme)
                        {
                            return Err(format!(
                                "code sheet {}: {} names {} which is not a class of {scheme}",
                                s.id, g.id, y.class
                            ));
                        }
                        if y.symbol.chars().count() != 1 {
                            return Err(format!(
                                "code sheet {}: {} symbol for {} must be one character",
                                s.id, g.id, y.class
                            ));
                        }
                        if !seen.insert(&y.class) {
                            return Err(format!(
                                "code sheet {}: {} lists {} twice",
                                s.id, g.id, y.class
                            ));
                        }
                    }
                    for c in atlas.classes.iter().filter(|c| &c.scheme == scheme) {
                        if !seen.contains(&c.id) {
                            return Err(format!(
                                "code sheet {}: class {} of {scheme} has no symbol",
                                s.id, c.id
                            ));
                        }
                    }
                }
                Source::Leading { segment, .. } => {
                    if !ids.contains(segment.as_str()) {
                        return Err(format!(
                            "code sheet {}: {} reads unknown segment {segment}",
                            s.id, g.id
                        ));
                    }
                }
                Source::Count { tag, .. } => {
                    if !s.rules.iter().any(|r| &r.tag == tag) {
                        return Err(format!("code sheet {}: no rule sets the tag {tag}", s.id));
                    }
                }
                Source::Value { per, times, .. } => {
                    if let Some(p) = per {
                        if sheet_constant(p).is_none() {
                            return Err(format!(
                                "code sheet {}: {} divides by {p}, which is not a constant on the sheet",
                                s.id, g.id
                            ));
                        }
                    }
                    if !times.is_finite() || *times == 0.0 {
                        return Err(format!(
                            "code sheet {}: {} times must be a number other than 0",
                            s.id, g.id
                        ));
                    }
                }
            }
        }
        for (l, t) in &s.layouts {
            for name in template_segments(t)? {
                if !ids.contains(name.as_str()) {
                    return Err(format!(
                        "code sheet {}: layout {l} uses unknown segment {name}",
                        s.id
                    ));
                }
            }
        }
        Ok(())
    }

    fn rules_with(sheet: &CodeSheet, prefix: &str) -> RuleSet {
        RuleSet {
            rules: sheet
                .rules
                .iter()
                .filter(|r| r.tag.starts_with(prefix))
                .cloned()
                .collect(),
        }
    }

    /// Read a world's codes. `system` is every subject in the world's system
    /// (the world may be among them); count segments need it.
    pub fn read(&self, sheet: &CodeSheet, world: &Subject, system: Option<&[Subject]>) -> Reading {
        let atlas = &self.sift.atlas;
        let mut subj = world.clone();
        // 1. prepare: flag facts from rules.
        let mut prepared = Vec::new();
        for p in &sheet.prepare {
            let rs = Self::rules_with(sheet, p);
            let got = rs.apply(&subj, &BTreeMap::new());
            for r in &rs.rules {
                subj.facts
                    .insert(r.tag.clone(), Value::Bool(got.contains_key(&r.tag)));
            }
            prepared.extend(got.into_values());
        }
        // 2. segments.
        let mut segments: Vec<SegmentReading> = Vec::new();
        for g in &sheet.segments {
            let physics = self.segment(sheet, atlas, g, &subj, system, &segments);
            let reading = match g
                .authored
                .as_ref()
                .and_then(|f| world.facts.get(f).map(|v| (f, v)))
            {
                Some((f, v)) => {
                    let symbol = match v {
                        Value::Text(t) => t.clone(),
                        Value::Int(i) => i64::try_from(*i)
                            .ok()
                            .and_then(ehex)
                            .map(String::from)
                            .unwrap_or_else(|| UNKNOWN.into()),
                        _ => UNKNOWN.into(),
                    };
                    SegmentReading {
                        id: g.id.clone(),
                        value: symbol_value(g, &symbol),
                        symbol,
                        why: Why::Authored {
                            field: f.clone(),
                            physics: Some(physics.symbol.clone()).filter(|s| s != UNKNOWN),
                        },
                    }
                }
                None => physics,
            };
            segments.push(reading);
        }
        // 3. publish values, run the groups.
        for s in &segments {
            if let Some(v) = s.value {
                subj.facts
                    .insert(format!("{}{}", sheet.fact_prefix, s.id), Value::Float(v));
            }
        }
        let groups = sheet
            .groups
            .iter()
            .map(|gr| {
                let got = Self::rules_with(sheet, &gr.prefix).apply(&subj, &BTreeMap::new());
                GroupReading {
                    id: gr.id.clone(),
                    tags: got
                        .into_iter()
                        .map(|(t, st)| (t[gr.prefix.len()..].to_string(), st))
                        .collect(),
                }
            })
            .collect();
        let lines = sheet
            .layouts
            .iter()
            .map(|(l, t)| (l.clone(), fill(t, &segments)))
            .collect();
        Reading {
            sheet: sheet.id.clone(),
            lines,
            segments,
            groups,
            prepared,
        }
    }

    fn segment(
        &self,
        sheet: &CodeSheet,
        atlas: &Atlas,
        g: &Segment,
        subj: &Subject,
        system: Option<&[Subject]>,
        before: &[SegmentReading],
    ) -> SegmentReading {
        let (symbol, value, why) = match &g.source {
            Source::Classify { scheme, symbols } => {
                let label = atlas.label(scheme, subj, None);
                match label
                    .base
                    .as_ref()
                    .and_then(|b| symbols.iter().find(|y| &y.class == b))
                {
                    Some(y) => {
                        let defaulted = defaulted(atlas, &label, subj);
                        (
                            y.symbol.clone(),
                            y.value,
                            Why::Classified { label, defaulted },
                        )
                    }
                    None => (UNKNOWN.into(), None, Why::Unclassified { label }),
                }
            }
            Source::Leading { field, segment } => {
                let exp = before
                    .iter()
                    .find(|s| &s.id == segment)
                    .and_then(|s| s.value);
                match (subj.num(field), exp) {
                    (Some(x), Some(e)) => {
                        let d = leading(x, e as i64);
                        (
                            ehex(d).map(String::from).unwrap_or_else(|| UNKNOWN.into()),
                            Some(d as f64),
                            Why::Leading {
                                field: field.clone(),
                                value: x,
                                exponent: e as i64,
                            },
                        )
                    }
                    _ => (
                        "0".into(),
                        Some(0.0),
                        Why::Leading {
                            field: field.clone(),
                            value: 0.0,
                            exponent: exp.unwrap_or(0.0) as i64,
                        },
                    ),
                }
            }
            Source::Value {
                field,
                times,
                per,
                digits,
            } => {
                let div = per.as_deref().and_then(sheet_constant).unwrap_or(1.0);
                match subj.num(field) {
                    Some(x) => {
                        let v = x * times / div;
                        (
                            sig_figures(v, *digits),
                            Some(v),
                            Why::Value {
                                field: field.clone(),
                                value: Some(x),
                                per: per.clone(),
                            },
                        )
                    }
                    None => (
                        UNKNOWN.into(),
                        None,
                        Why::Value {
                            field: field.clone(),
                            value: None,
                            per: per.clone(),
                        },
                    ),
                }
            }
            Source::Count { tag, max } => match system {
                None => (UNKNOWN.into(), None, Why::NoSystem { tag: tag.clone() }),
                Some(sys) => {
                    let rs = Self::rules_with(sheet, tag);
                    let mut members: Vec<String> = sys
                        .iter()
                        .filter(|b| rs.apply(b, &BTreeMap::new()).contains_key(tag))
                        .map(|b| {
                            b.address
                                .as_ref()
                                .map(|a| a.to_string())
                                .unwrap_or_default()
                        })
                        .collect();
                    members.sort();
                    let n = (members.len() as i64).min(*max);
                    (
                        ehex(n).map(String::from).unwrap_or_else(|| UNKNOWN.into()),
                        Some(n as f64),
                        Why::Counted {
                            tag: tag.clone(),
                            members,
                        },
                    )
                }
            },
        };
        SegmentReading {
            id: g.id.clone(),
            symbol,
            value,
            why,
        }
    }
}

/// The facts a label read that the subject did not have (the axis's missing value stood in).
fn defaulted(atlas: &Atlas, label: &Label, s: &Subject) -> Vec<String> {
    let Some(base) = &label.base else {
        return Vec::new();
    };
    let Some(fit) = atlas.fit(base, s) else {
        return Vec::new();
    };
    let mut out: Vec<String> = fit
        .axes
        .iter()
        .filter_map(|a| atlas.axes.get(&a.axis))
        .filter(|ax| ax.missing.is_some() && s.get(&ax.field).is_none())
        .map(|ax| ax.field.clone())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The leading digit of `x` at the power of ten `exp`: floor(x / 10^exp),
/// held to 1..9 (0 when x is below 1). Whole-number arithmetic only.
pub fn leading(x: f64, exp: i64) -> i64 {
    if x.is_nan() || x < 1.0 {
        return 0;
    }
    let mut n = x as u128;
    for _ in 0..exp.max(0) {
        n /= 10;
    }
    (n as i64).clamp(1, 9)
}

/// A constant on the master data sheet by its id (`g0`, `atm`, `day`...).
fn sheet_constant(id: &str) -> Option<f64> {
    sse_core::sheet::ALL
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
}

fn symbol_value(g: &Segment, symbol: &str) -> Option<f64> {
    match &g.source {
        Source::Classify { symbols, .. } => symbols
            .iter()
            .find(|y| y.symbol == symbol)
            .and_then(|y| y.value),
        _ => symbol.chars().next().and_then(ehex_value).map(|v| v as f64),
    }
}

fn template_segments(t: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut rest = t;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else {
            return Err(format!("layout {t}: unclosed brace"));
        };
        out.push(rest[i + 1..i + j].to_string());
        rest = &rest[i + j + 1..];
    }
    Ok(out)
}

fn fill(t: &str, segs: &[SegmentReading]) -> String {
    let mut out = String::new();
    let mut rest = t;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let j = rest[i..].find('}').map_or(rest.len(), |j| i + j);
        let name = &rest[i + 1..j];
        out.push_str(
            segs.iter()
                .find(|s| s.id == name)
                .map_or(UNKNOWN, |s| s.symbol.as_str()),
        );
        rest = rest.get(j + 1..).unwrap_or("");
    }
    out.push_str(rest);
    out
}

/// Split a code line by a layout into segment symbols.
pub fn parse(sheet: &CodeSheet, layout: &str, line: &str) -> Result<Vec<(String, String)>, String> {
    let t = sheet
        .layouts
        .get(layout)
        .ok_or_else(|| format!("code sheet {} has no layout {layout}", sheet.id))?;
    let mut out = Vec::new();
    let mut chars = line.trim().chars();
    let mut rest = t.as_str();
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('{') {
            let j = r.find('}').ok_or("unclosed brace")?;
            let c = chars
                .next()
                .ok_or_else(|| format!("{line}: too short for {t}"))?;
            out.push((r[..j].to_string(), c.to_ascii_uppercase().to_string()));
            rest = &r[j + 1..];
        } else {
            let want = rest.chars().next().unwrap_or_default();
            let got = chars.next();
            if got != Some(want) {
                return Err(format!("{line}: expected '{want}' where {t} has it"));
            }
            rest = &rest[want.len_utf8()..];
        }
    }
    if chars.next().is_some() {
        return Err(format!("{line}: longer than {t}"));
    }
    Ok(out)
}

/// A range or set on one fact that a symbol stands for.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Constraint {
    /// Fact value in [min, max], in the fact's own units.
    Range {
        field: String,
        min: f64,
        max: f64,
    },
    AnyOf {
        field: String,
        any_of: Vec<String>,
    },
    Flag {
        field: String,
        is: bool,
    },
}

/// What one class stands for: the bounds of its core and its typical values.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClassPins {
    pub class: String,
    pub constraints: Vec<Constraint>,
    /// Fact to typical value (fact units), for building the plainest world.
    pub typical: BTreeMap<String, Value>,
}

/// One segment of a code line read backwards.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SegmentPins {
    pub segment: String,
    pub symbol: String,
    /// The classes the symbol may stand for (several classes can share one
    /// symbol), in sheet order. Empty for leading and count segments.
    pub classes: Vec<ClassPins>,
    /// The symbol's value (counts, digits).
    pub value: Option<f64>,
    /// Set when the symbol is not one the sheet knows.
    pub unknown: bool,
}

fn scale(atlas: &Atlas, axis: &str) -> (String, f64) {
    let a = &atlas.axes[axis];
    (a.field.clone(), a.divide_by.unwrap_or(1.0))
}

/// A code line read backwards into pins on the facts.
pub fn pins(
    codes: &Codes,
    sheet: &CodeSheet,
    layout: &str,
    line: &str,
) -> Result<Vec<SegmentPins>, String> {
    let atlas = &codes.sift.atlas;
    let mut out = Vec::new();
    for (seg, symbol) in parse(sheet, layout, line)? {
        let g = sheet
            .segments
            .iter()
            .find(|g| g.id == seg)
            .ok_or_else(|| format!("unknown segment {seg}"))?;
        let mut classes = Vec::new();
        if let Source::Classify { symbols, .. } = &g.source {
            for y in symbols.iter().filter(|y| y.symbol == symbol) {
                let mut cp = class_pins(atlas, &y.class);
                cp.typical
                    .extend(y.plain.iter().map(|(k, v)| (k.clone(), v.clone())));
                classes.push(cp);
            }
        }
        let unknown = matches!(g.source, Source::Classify { .. }) && classes.is_empty();
        out.push(SegmentPins {
            value: symbol_value(g, &symbol),
            segment: seg,
            symbol,
            classes,
            unknown,
        });
    }
    Ok(out)
}

fn class_pins(atlas: &Atlas, class: &str) -> ClassPins {
    let c = atlas
        .classes
        .iter()
        .find(|c| c.id == class)
        .expect("checked by Codes::check");
    let mut constraints = Vec::new();
    for b in c.region.iter().chain(&c.gate) {
        match b {
            Bound::Numeric { axis, core, .. } => {
                let (field, d) = scale(atlas, axis);
                constraints.push(Constraint::Range {
                    field,
                    min: core.0 * d,
                    max: core.1 * d,
                });
            }
            Bound::AnyOf { axis, any_of } => constraints.push(Constraint::AnyOf {
                field: atlas.axes[axis].field.clone(),
                any_of: any_of.clone(),
            }),
            Bound::Flag { axis, is } => constraints.push(Constraint::Flag {
                field: atlas.axes[axis].field.clone(),
                is: *is,
            }),
        }
    }
    let typical = c
        .typical
        .iter()
        .map(|t| {
            let a = &atlas.axes[&t.axis];
            (
                a.field.clone(),
                Value::Float(t.centre * a.divide_by.unwrap_or(1.0)),
            )
        })
        .collect();
    ClassPins {
        class: class.into(),
        constraints,
        typical,
    }
}

/// The plainest world a code line stands for: each segment's first class's
/// typical values, over `base` facts, and a leading digit times its power of
/// ten. Count segments are left to the caller (they are about the system).
pub fn typical_world(
    sheet: &CodeSheet,
    pins: &[SegmentPins],
    base: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut f = base.clone();
    for p in pins {
        if let Some(c) = p.classes.first() {
            for (k, v) in &c.typical {
                f.insert(k.clone(), v.clone());
            }
        }
    }
    for p in pins {
        let Some(g) = sheet.segments.iter().find(|g| g.id == p.segment) else {
            continue;
        };
        if let Source::Leading { field, segment } = &g.source {
            let exp = pins
                .iter()
                .find(|q| &q.segment == segment)
                .and_then(|q| q.value);
            if let (Some(d), Some(e)) = (p.value, exp) {
                let mut x = d;
                for _ in 0..(e as i64).max(0) {
                    x *= 10.0;
                }
                f.insert(field.clone(), Value::Float(x));
            }
        }
    }
    f
}
