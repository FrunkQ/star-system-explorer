//! SSE 3's fingerprint scoring, reproduced exactly, as the scoring mode of
//! the "SSE 3 classic" scheme (classification design sections 6 and 11).
//!
//! Source: SSE 3.1 `src/lib/system/classification.ts` (`bandFit`,
//! `fingerprintScore`, `compareBases`, `classifyByFingerprint`,
//! `explainClassification`, `fallbackBaseClass`). Checked against that code
//! on 3,000 generated bodies (`tests/fixtures/classify/v3-oracle.json`).
//!
//! - A band's fit is 1 inside it; outside, it falls linearly to 0 over a
//!   relative tolerance (15%) of the boundary it crossed.
//! - A class's score is mean(band fits) x (1 + 0.1 x bands) x weight; one
//!   band at 0 disqualifies it, and a failed gate rules it out.
//! - Bases: a clean match (every band at 1) outranks a partial one, then
//!   score, then pack order. Modifiers at or above their threshold stack, in
//!   score order, up to `max_classes` in all.
//! - Nothing scores: the scheme's fallback (by mass in SSE 3).
//!
//! The arithmetic is the same IEEE operations in the same order as the
//! JavaScript, so scores agree to the last bit; band order is the pack's.

use crate::classify::{bound_axis, Atlas, AxisFit, Bound, ClassFit, ClassKind, Label, Scheme};
use crate::predicate::{as_num, Subject};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;

/// SSE 3's own answer for one body, beside the SSE4 label it maps to.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClassicWorking {
    /// The class list SSE 3 puts on the body: base first, then modifiers.
    /// SSE 3 leaves the base out when only modifiers score (kept as is).
    pub classes: Vec<String>,
    /// True when no base scored and the scheme's fallback named the base.
    pub fallback: bool,
    /// The runner-up base scored within `borderline` of the winner.
    pub borderline: bool,
    /// Ranked bases that scored (winner first, at most five), with band fits.
    pub candidates: Vec<ClassFit>,
    /// Every modifier at or above its threshold, with its score (as SSE 3's
    /// explanation: empty when the base is a fallback).
    pub modifiers: Vec<(String, f64)>,
}

fn band_fit(atlas: &Atlas, b: &Bound, s: &Subject, tol: f64) -> AxisFit {
    let axis = bound_axis(b).to_string();
    let value = atlas.axes.get(&axis).and_then(|a| a.value(s));
    let (fit, change) = match b {
        Bound::Numeric { core: (lo, hi), .. } => match value.as_ref().and_then(as_num) {
            None => (0.0, None),
            Some(x) => {
                let (lo, hi) = (*lo, *hi);
                if x >= lo && x <= hi {
                    (1.0, Some(0.0))
                } else if x < lo {
                    let r = if lo != 0.0 {
                        lo.abs()
                    } else if hi != 0.0 {
                        hi.abs()
                    } else {
                        1.0
                    };
                    (f64::max(0.0, 1.0 - ((lo - x) / r) / tol), Some(lo - x))
                } else {
                    let r = if hi != 0.0 { hi.abs() } else { 1.0 };
                    (f64::max(0.0, 1.0 - ((x - hi) / r) / tol), Some(hi - x))
                }
            }
        },
        Bound::AnyOf { any_of, .. } => {
            let ok = matches!(&value, Some(Value::Text(t)) if any_of.contains(t));
            (ok as u8 as f64, None)
        }
        Bound::Flag { is, .. } => {
            let ok = matches!(&value, Some(Value::Bool(v)) if v == is);
            (ok as u8 as f64, None)
        }
    };
    AxisFit {
        axis,
        fit,
        value,
        change_to_core: change,
    }
}

struct Scored<'a> {
    class: &'a crate::classify::Class,
    score: f64,
    clean: bool,
    bands: Vec<AxisFit>,
}

fn score<'a>(atlas: &'a Atlas, c: &'a crate::classify::Class, s: &Subject, tol: f64) -> Scored<'a> {
    let chain = atlas.ancestry(&c.id);
    let gates = chain.iter().rev().flat_map(|k| k.gate.iter());
    let bands: Vec<AxisFit> = chain
        .iter()
        .rev()
        .flat_map(|k| k.region.iter())
        .map(|b| band_fit(atlas, b, s, tol))
        .collect();
    let clean = bands.iter().all(|b| b.fit >= 1.0);
    let mut out = Scored {
        class: c,
        score: 0.0,
        clean,
        bands,
    };
    for g in gates {
        if band_fit(atlas, g, s, tol).fit <= 0.0 {
            return out;
        }
    }
    let mut sum = 0.0;
    let mut n = 0u32;
    for b in &out.bands {
        if b.fit <= 0.0 {
            return out;
        }
        sum += b.fit;
        n += 1;
    }
    if n == 0 {
        return out;
    }
    let n = n as f64;
    out.score = (sum / n) * (1.0 + 0.1 * n) * c.weight;
    out
}

fn class_fit(s: &Scored) -> ClassFit {
    let limiting = s
        .bands
        .iter()
        .min_by(|a, b| a.fit.total_cmp(&b.fit).then(a.axis.cmp(&b.axis)))
        .cloned();
    ClassFit {
        class: s.class.id.clone(),
        fit: s.score,
        depth: 1,
        limiting,
        axes: s.bands.clone(),
    }
}

fn fallback(atlas: &Atlas, sc: &Scheme, s: &Subject) -> Option<String> {
    let f = sc.fallback.as_ref()?;
    let x = atlas
        .axes
        .get(&f.axis)
        .and_then(|a| a.value(s))
        .as_ref()
        .and_then(as_num)
        .filter(|x| x.is_finite());
    Some(match x {
        None => f.unknown.clone(),
        Some(x) if x > f.above => f.high.clone(),
        Some(_) => f.low.clone(),
    })
}

/// Label one subject in a classic scheme. `fit` on the label and on each
/// candidate is SSE 3's score (it can exceed 1); stickiness does not apply,
/// because SSE 3 re-derives every class on every pass.
pub fn label(atlas: &Atlas, sc: &Scheme, s: &Subject) -> Label {
    let mut scored: Vec<Scored> = atlas
        .classes
        .iter()
        .filter(|c| c.scheme == sc.id)
        .map(|c| score(atlas, c, s, sc.tolerance))
        .filter(|x| x.score > 0.0)
        .collect();
    // Stable sorts, as JavaScript's: ties keep pack order.
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut bases: Vec<&Scored> = scored
        .iter()
        .filter(|x| x.class.kind == ClassKind::Base)
        .collect();
    bases.sort_by(|a, b| b.clean.cmp(&a.clean).then(b.score.total_cmp(&a.score)));

    let mut classes: Vec<String> = Vec::new();
    if let Some(b) = bases.first() {
        classes.push(b.class.id.clone());
    }
    for x in &scored {
        if x.class.kind == ClassKind::Modifier
            && x.score >= x.class.threshold
            && !classes.contains(&x.class.id)
        {
            classes.push(x.class.id.clone());
        }
        if classes.len() >= sc.max_classes {
            break;
        }
    }
    let fb = if bases.is_empty() {
        fallback(atlas, sc, s)
    } else {
        None
    };
    if classes.is_empty() {
        if let Some(f) = &fb {
            classes.push(f.clone());
        }
    }
    // SSE 3's explanation lists no modifiers when no base scored (it returns
    // early with the fallback), although the body's class list has them.
    let modifiers: Vec<(String, f64)> = scored
        .iter()
        .filter(|_| !bases.is_empty())
        .filter(|x| x.class.kind == ClassKind::Modifier && x.score >= x.class.threshold)
        .map(|x| (x.class.id.clone(), x.score))
        .collect();
    let candidates: Vec<ClassFit> = bases.iter().take(5).map(|x| class_fit(x)).collect();
    let borderline = bases.len() > 1 && bases[1].score >= bases[0].score * sc.borderline;
    let base = bases.first().map(|b| b.class.id.clone()).or(fb.clone());
    Label {
        scheme: sc.id.clone(),
        fit: bases.first().map_or(0.0, |b| b.score),
        modifiers: classes
            .iter()
            .filter(|c| base.as_ref() != Some(*c))
            .cloned()
            .collect(),
        nearest: candidates.iter().skip(1).cloned().collect(),
        base,
        kept_by_stickiness: false,
        classic: Some(ClassicWorking {
            classes,
            fallback: fb.is_some(),
            borderline,
            candidates,
            modifiers,
        }),
    }
}
