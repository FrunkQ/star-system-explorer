//! Region classification, generic (classification design sections 3 to 5).
//!
//! A class is a region in a space of named axes: per-axis cores (fully in)
//! and fringes (fading to zero). Fit is the lowest per-axis fit, so one
//! disqualifying fact cannot be outvoted. Base classes compete (deepest,
//! then highest fit, then smallest region, then id); modifiers stack. Labels
//! are sticky: a label changes only when the newcomer beats it by a margin.
//! C1 brings the real axes and atlas; this is the machinery.

use crate::predicate::{as_num, Subject};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::math as m;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scale {
    Linear,
    Log,
    Fraction,
    Category,
    Flag,
}

/// One registered axis (classification design section 3): a quantity the
/// kernel publishes, read for classification on a scale.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Axis {
    /// Fact name the axis reads.
    pub field: String,
    pub scale: Scale,
    /// Range used to normalise distances and region sizes on this axis.
    #[serde(default)]
    pub span: Option<(f64, f64)>,
    #[serde(default)]
    pub label: String,
    /// Unit of the axis value as classes write it (after `divide_by`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    /// The fact is divided by this to give the axis value (a unit change,
    /// e.g. kg to Earth masses).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divide_by: Option<f64>,
    /// Value used when the fact is absent. Only for schemes that must
    /// reproduce an older classifier's defaults (SSE 3 classic); "absent is
    /// not typical" otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<Value>,
    /// Body kinds the axis applies to (empty: any).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub explain_ref: String,
    /// No flow publishes this fact yet: it is read as authored or imported.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub provisional: bool,
}

impl Axis {
    /// The subject's value on this axis.
    pub fn value(&self, s: &Subject) -> Option<Value> {
        let v = s
            .get(&self.field)
            .cloned()
            .or_else(|| self.missing.clone())?;
        match (self.divide_by, as_num(&v)) {
            (Some(d), Some(x)) => Some(Value::Float(x / d)),
            _ => Some(v),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Bound {
    Numeric {
        axis: String,
        core: (f64, f64),
        fringe: (f64, f64),
    },
    AnyOf {
        axis: String,
        any_of: Vec<String>,
    },
    Flag {
        axis: String,
        is: bool,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassKind {
    Base,
    Modifier,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Meaning {
    /// May drive generation and gates.
    #[default]
    Identity,
    /// Only changes how a thing reads or draws.
    Flourish,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    #[default]
    Science,
    Speculative,
    Fictional,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Names {
    #[serde(default)]
    pub physics: String,
    #[serde(default)]
    pub plain: String,
    #[serde(default)]
    pub story: String,
}

/// Where members usually sit: for placing and generating, never for recognising.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Typical {
    pub axis: String,
    pub centre: f64,
    pub spread: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Image {
    pub file: String,
    #[serde(default)]
    pub licence: String,
    #[serde(default)]
    pub credit: String,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Look {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mark: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub colour: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Learn {
    #[serde(default)]
    pub summary: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub explain_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Expect {
    In,
    Out,
}

/// A known world that must (or must not) match: the class's own test.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Example {
    pub name: String,
    pub values: BTreeMap<String, Value>,
    pub expect: Expect,
}

/// One class record (classification design section 4). Everything a
/// picker, generator, symbol, image, explanation or test needs reads this.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Class {
    pub id: String,
    #[serde(default)]
    pub parent: Option<String>,
    pub scheme: String,
    pub kind: ClassKind,
    /// Own bounds; a child also inherits its ancestors' bounds.
    #[serde(default)]
    pub region: Vec<Bound>,
    /// Least fit to count (base: below it the body is unclassified; modifier: added at or above). Default 0.6, as SSE 3.
    #[serde(default = "point_six")]
    pub threshold: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub names: Names,
    #[serde(default, skip_serializing_if = "is_default")]
    pub meaning: Meaning,
    /// Preconditions: failing one rules the class out, passing earns nothing (SSE 3 `gate`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gate: Vec<Bound>,
    /// Score weight (classic scoring); 1 by default.
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub weight: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub typical: Vec<Typical>,
    /// Cross-references: class ids or conditions another suite settles (C3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    /// 0..1, the GM's exotic dial.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rarity: Option<f64>,
    /// Conditions under which generation may aim for this class (SSE 3 `formation`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generate: Vec<Bound>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub look: Look,
    #[serde(default, skip_serializing_if = "is_default")]
    pub learn: Learn,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Example>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub evidence: EvidenceKind,
}
fn point_six() -> f64 {
    0.6
}
impl Class {
    /// The same class with these bounds (tests and builders).
    pub fn with_region(mut self, region: Vec<Bound>) -> Self {
        self.region = region;
        self
    }
}
fn one() -> f64 {
    1.0
}
fn is_one(x: &f64) -> bool {
    *x == 1.0
}
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == T::default()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scoring {
    /// Lowest per-axis fit; deepest, then fittest, then smallest region (section 5).
    #[default]
    Region,
    /// SSE 3's fingerprint scoring, reproduced exactly (`classic.rs`).
    Classic,
}

/// What a scheme says when nothing fits (SSE 3's last resort, made explicit).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Fallback {
    pub axis: String,
    pub above: f64,
    pub high: String,
    pub low: String,
    pub unknown: String,
}

/// One atlas answering one question (section 6). A body gets one base label per scheme.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub scoring: Scoring,
    /// Classic: relative soft-edge tolerance (SSE 3: 0.15).
    #[serde(default = "point_one_five")]
    pub tolerance: f64,
    /// Classic: most classes on a body, base included (SSE 3 `maxClasses`).
    #[serde(default = "four")]
    pub max_classes: usize,
    /// Classic: a runner-up within this share of the winner is borderline (SSE 3: 0.9).
    #[serde(default = "point_nine")]
    pub borderline: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fallback>,
}
fn point_one_five() -> f64 {
    0.15
}
fn four() -> usize {
    4
}
fn point_nine() -> f64 {
    0.9
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Atlas {
    pub axes: BTreeMap<String, Axis>,
    pub classes: Vec<Class>,
    /// Schemes by id; a scheme with no record scores by region.
    #[serde(default)]
    pub schemes: BTreeMap<String, Scheme>,
    /// Margin a new base class must beat the held one by (stickiness).
    #[serde(default = "point_one")]
    pub sticky_margin: f64,
}
fn point_one() -> f64 {
    0.1
}

/// One axis's fit for a class, and what would put the subject in its core.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AxisFit {
    pub axis: String,
    pub fit: f64,
    pub value: Option<Value>,
    /// Signed change needed to reach the core (numeric axes), 0 if inside.
    pub change_to_core: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClassFit {
    pub class: String,
    pub fit: f64,
    pub depth: usize,
    /// The axis that set the fit (lowest).
    pub limiting: Option<AxisFit>,
    pub axes: Vec<AxisFit>,
}

/// The working behind a scheme's label (EXPLAIN-ready).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Label {
    pub scheme: String,
    /// `None`: unclassified (nearest classes still listed).
    pub base: Option<String>,
    pub fit: f64,
    pub modifiers: Vec<String>,
    /// Runners-up and near misses, best first (at most five).
    pub nearest: Vec<ClassFit>,
    /// True when the held label was kept by stickiness although another class fits better.
    pub kept_by_stickiness: bool,
    /// Classic scoring only: SSE 3's own output and working.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classic: Option<crate::classic::ClassicWorking>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtlasError(pub String);
impl std::fmt::Display for AtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub(crate) fn bound_axis(b: &Bound) -> &str {
    match b {
        Bound::Numeric { axis, .. } | Bound::AnyOf { axis, .. } | Bound::Flag { axis, .. } => axis,
    }
}

impl Atlas {
    pub fn validate(&self) -> Result<(), AtlasError> {
        let ids: BTreeSet<&str> = self.classes.iter().map(|c| c.id.as_str()).collect();
        if ids.len() != self.classes.len() {
            return Err(AtlasError("a class id is defined twice".into()));
        }
        for c in &self.classes {
            if let Some(p) = &c.parent {
                if !ids.contains(p.as_str()) {
                    return Err(AtlasError(format!("class {} has unknown parent {p}", c.id)));
                }
                if self.class(p).is_some_and(|pc| pc.scheme != c.scheme) {
                    return Err(AtlasError(format!(
                        "class {} and its parent are in different schemes",
                        c.id
                    )));
                }
            }
            if self.ancestry(&c.id).len() > self.classes.len() {
                return Err(AtlasError(format!("class {} has a parent cycle", c.id)));
            }
            if let Some(sc) = self.schemes.get(&c.scheme) {
                if sc.scoring == Scoring::Classic && c.region.is_empty() {
                    return Err(AtlasError(format!("class {} has no bands", c.id)));
                }
            }
            for b in c.gate.iter().chain(&c.generate) {
                if !self.axes.contains_key(bound_axis(b)) {
                    return Err(AtlasError(format!(
                        "class {} uses unknown axis {}",
                        c.id,
                        bound_axis(b)
                    )));
                }
            }
            for b in &c.region {
                let a = bound_axis(b);
                if !self.axes.contains_key(a) {
                    return Err(AtlasError(format!("class {} uses unknown axis {a}", c.id)));
                }
                if let Bound::Numeric { core, fringe, .. } = b {
                    if !(fringe.0 <= core.0 && core.0 <= core.1 && core.1 <= fringe.1) {
                        return Err(AtlasError(format!(
                            "class {}: on {a} the fringe must enclose the core",
                            c.id
                        )));
                    }
                }
            }
        }
        for (id, sc) in &self.schemes {
            if *id != sc.id {
                return Err(AtlasError(format!("scheme {id} is filed under another id")));
            }
            if let Some(f) = &sc.fallback {
                if !self.axes.contains_key(&f.axis) {
                    return Err(AtlasError(format!(
                        "scheme {id}: fallback uses unknown axis {}",
                        f.axis
                    )));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn class(&self, id: &str) -> Option<&Class> {
        self.classes.iter().find(|c| c.id == id)
    }

    /// The class and its ancestors, nearest first (bounded against cycles).
    pub(crate) fn ancestry(&self, id: &str) -> Vec<&Class> {
        let mut out = Vec::new();
        let mut cur = self.class(id);
        while let Some(c) = cur {
            out.push(c);
            if out.len() > self.classes.len() {
                break;
            }
            cur = c.parent.as_deref().and_then(|p| self.class(p));
        }
        out
    }

    fn axis_pos(&self, axis: &Axis, x: f64) -> f64 {
        match axis.scale {
            Scale::Log if x > 0.0 => m::log10(x),
            _ => x,
        }
    }

    fn axis_fit(&self, b: &Bound, s: &Subject) -> AxisFit {
        let a = bound_axis(b).to_string();
        let axis = &self.axes[&a];
        let value = axis.value(s);
        match b {
            Bound::Numeric { core, fringe, .. } => {
                let Some(x) = value.as_ref().and_then(as_num) else {
                    return AxisFit {
                        axis: a,
                        fit: 0.0,
                        value,
                        change_to_core: None,
                    };
                };
                let p = |v: f64| self.axis_pos(axis, v);
                let (xp, c0, c1, f0, f1) = (p(x), p(core.0), p(core.1), p(fringe.0), p(fringe.1));
                let fit = if xp >= c0 && xp <= c1 {
                    1.0
                } else if xp < c0 {
                    if xp <= f0 || c0 == f0 {
                        0.0
                    } else {
                        (xp - f0) / (c0 - f0)
                    }
                } else if xp >= f1 || c1 == f1 {
                    0.0
                } else {
                    (f1 - xp) / (f1 - c1)
                };
                let change = if x < core.0 {
                    core.0 - x
                } else if x > core.1 {
                    core.1 - x
                } else {
                    0.0
                };
                AxisFit {
                    axis: a,
                    fit,
                    value,
                    change_to_core: Some(change),
                }
            }
            Bound::AnyOf { any_of, .. } => {
                let fit = matches!(&value, Some(Value::Text(t)) if any_of.contains(t)) as u8 as f64;
                AxisFit {
                    axis: a,
                    fit,
                    value,
                    change_to_core: None,
                }
            }
            Bound::Flag { is, .. } => {
                let fit = matches!(&value, Some(Value::Bool(v)) if v == is) as u8 as f64;
                AxisFit {
                    axis: a,
                    fit,
                    value,
                    change_to_core: None,
                }
            }
        }
    }

    /// Fit of one class including its ancestors' bounds.
    pub fn fit(&self, class: &str, s: &Subject) -> Option<ClassFit> {
        let chain = self.ancestry(class);
        let c = *chain.first()?;
        let mut axes: Vec<AxisFit> = chain
            .iter()
            .flat_map(|k| k.region.iter())
            .map(|b| self.axis_fit(b, s))
            .collect();
        axes.sort_by(|a, b| a.axis.cmp(&b.axis).then(a.fit.total_cmp(&b.fit)));
        let limiting = axes
            .iter()
            .min_by(|a, b| a.fit.total_cmp(&b.fit).then(a.axis.cmp(&b.axis)))
            .cloned();
        Some(ClassFit {
            class: c.id.clone(),
            fit: limiting.as_ref().map_or(1.0, |l| l.fit),
            depth: chain.len(),
            limiting,
            axes,
        })
    }

    /// Normalised core volume (smaller is more specific); unconstrained axes count 1.
    fn volume(&self, class: &str) -> f64 {
        let mut per: BTreeMap<&str, f64> = BTreeMap::new();
        for b in self.ancestry(class).iter().flat_map(|k| k.region.iter()) {
            let a = bound_axis(b);
            let axis = &self.axes[a];
            let frac = match b {
                Bound::Numeric { core, .. } => match axis.span {
                    Some((lo, hi)) => {
                        let w = self.axis_pos(axis, core.1) - self.axis_pos(axis, core.0);
                        let s = self.axis_pos(axis, hi) - self.axis_pos(axis, lo);
                        if s > 0.0 {
                            (w / s).clamp(0.0, 1.0)
                        } else {
                            1.0
                        }
                    }
                    None => 1.0,
                },
                Bound::AnyOf { .. } => 1.0,
                Bound::Flag { .. } => 0.5,
            };
            let e = per.entry(a).or_insert(1.0);
            *e = e.min(frac);
        }
        per.values().product()
    }

    /// Label a subject in one scheme. `held` is the base label it had before.
    pub fn label(&self, scheme: &str, s: &Subject, held: Option<&str>) -> Label {
        if let Some(sc) = self
            .schemes
            .get(scheme)
            .filter(|sc| sc.scoring == Scoring::Classic)
        {
            return crate::classic::label(self, sc, s);
        }
        let mut fits: Vec<(ClassFit, f64, ClassKind)> = self
            .classes
            .iter()
            .filter(|c| c.scheme == scheme)
            .filter_map(|c| self.fit(&c.id, s).map(|f| (f, c.threshold, c.kind)))
            .collect();
        let rank = |a: &ClassFit, b: &ClassFit| {
            b.depth
                .cmp(&a.depth)
                .then(b.fit.total_cmp(&a.fit))
                .then(self.volume(&a.class).total_cmp(&self.volume(&b.class)))
                .then(a.class.cmp(&b.class))
        };
        let mut eligible: Vec<&ClassFit> = fits
            .iter()
            .filter(|(f, t, k)| *k == ClassKind::Base && f.fit >= *t)
            .map(|(f, _, _)| f)
            .collect();
        eligible.sort_by(|a, b| rank(a, b));
        let mut best = eligible.first().map(|f| (*f).clone());
        let mut kept = false;
        if let (Some(h), Some(b)) = (held, &best) {
            if b.class != h {
                if let Some(hf) = fits
                    .iter()
                    .find(|(f, t, _)| f.class == h && f.fit >= *t - self.sticky_margin)
                {
                    if b.fit < hf.0.fit + self.sticky_margin {
                        best = Some(hf.0.clone());
                        kept = true;
                    }
                }
            }
        }
        let modifiers: Vec<String> = fits
            .iter()
            .filter(|(f, t, k)| *k == ClassKind::Modifier && f.fit >= *t)
            .map(|(f, _, _)| f.class.clone())
            .collect();
        fits.retain(|(f, _, k)| {
            *k == ClassKind::Base && best.as_ref().is_none_or(|b| b.class != f.class)
        });
        fits.sort_by(|a, b| b.0.fit.total_cmp(&a.0.fit).then(a.0.class.cmp(&b.0.class)));
        Label {
            scheme: scheme.into(),
            fit: best.as_ref().map_or(0.0, |b| b.fit),
            base: best.map(|b| b.class),
            modifiers,
            nearest: fits.into_iter().take(5).map(|f| f.0).collect(),
            kept_by_stickiness: kept,
            classic: None,
        }
    }

    /// Every class example that does not come out as it says: an `in`
    /// example must carry the class (as base, modifier or ancestor of the
    /// base), an `out` example must not. Empty when all pass.
    pub fn check_examples(&self) -> Vec<String> {
        let mut bad = Vec::new();
        for c in &self.classes {
            for ex in &c.examples {
                let l = self.label(&c.scheme, &Subject::new(ex.values.clone()), None);
                let carries = l.modifiers.contains(&c.id)
                    || l.base
                        .as_deref()
                        .is_some_and(|b| self.ancestry(b).iter().any(|k| k.id == c.id));
                if carries != (ex.expect == Expect::In) {
                    bad.push(format!(
                        "{}: example {} should be {} but is labelled {:?} {:?}",
                        c.id,
                        ex.name,
                        if ex.expect == Expect::In { "in" } else { "out" },
                        l.base,
                        l.modifiers
                    ));
                }
            }
        }
        bad
    }

    /// Content hash of the atlas (which classes and axes produced a label).
    pub fn content_hash(&self) -> sse_core::ContentHash {
        sse_core::ContentHash::of(self).expect("atlas holds no NaN")
    }

    /// Label a time series; returns (time, old, new) for every change.
    pub fn track(
        &self,
        scheme: &str,
        series: &[Subject],
    ) -> Vec<(i64, Option<String>, Option<String>)> {
        let mut held: Option<String> = None;
        let mut out = Vec::new();
        for s in series {
            let l = self.label(scheme, s, held.as_deref());
            if l.base != held {
                out.push((s.now_s, held.clone(), l.base.clone()));
                held = l.base;
            }
        }
        out
    }
}
