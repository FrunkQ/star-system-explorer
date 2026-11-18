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
}
fn point_six() -> f64 {
    0.6
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Atlas {
    pub axes: BTreeMap<String, Axis>,
    pub classes: Vec<Class>,
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtlasError(pub String);
impl std::fmt::Display for AtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn bound_axis(b: &Bound) -> &str {
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
        Ok(())
    }

    fn class(&self, id: &str) -> Option<&Class> {
        self.classes.iter().find(|c| c.id == id)
    }

    /// The class and its ancestors, nearest first (bounded against cycles).
    fn ancestry(&self, id: &str) -> Vec<&Class> {
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
        let value = s.get(&axis.field).cloned();
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
        }
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
