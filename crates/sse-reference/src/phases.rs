//! MP17: many phases per substance (matter pack design section 4.3A).
//!
//! A substance's `phases` group keeps its familiar values (triple point,
//! vapour curve, densities) and may add:
//!
//! - `forms`: one row per phase, `{id, name{physics,plain,story}, kind,
//!   stability, evidence{grade, p_pa, t_k, src}, props{...}}`. `kind` is a
//!   label read by nothing (solid, glass, liquid, gas, supercritical fluid,
//!   ionic fluid, superionic solid, metallic fluid); blocks read `props`.
//! - `boundaries`: `{between: [a, b], curve, latent_heat?, dv?, evidence,
//!   src}`. A curve is `{form: "points", points: [[p_pa, t_k], ...]}`,
//!   `{form: "vapour"}` (the row's own `vapour_pressure` fit) or
//!   `{form: "iapws_melt", tn, pn, log, terms: [[a, b], ...], valid_k}`:
//!   π = 1 + Σ aᵢ(1 − θ^bᵢ), or ln π = Σ aᵢ(1 − θ^bᵢ) when `log`, with
//!   θ = T/Tn and π = p/pn (Wagner, Riethmann, Feistel and Harvey 2011,
//!   IAPWS R14-08).
//! - `meets`: triple points `{phases: [a, b, c], t_k, p_pa, src}`, each
//!   written once. Solid, liquid and vapour meet at the row's own
//!   `triple_point_k` and `triple_point_pa`, so the two must agree.
//! - `table`: built at pack-build time from the boundaries,
//!   `{log10_p_pa: {lo, hi, n}, t_k or log10_t_k: {lo, hi, n}, phases:
//!   [ids], cells or rle}`. `cells` is one string of base-36 indices into
//!   `phases`, row-major with pressure outer; `rle` is the same grid run
//!   length encoded: rows (low pressure first) split by "/", runs (low
//!   temperature first) by spaces, each the phase digit then the run's
//!   length in base 36. A boundary with no sample inside the table is not
//!   checked against it. `phase_at` reads the nearest grid node, so asking is a
//!   lookup and every device gives the same answer.
//!
//! Without a table, `phase_at` falls back on the familiar three phases from
//! the triple point and vapour curve, so a row with one phase still answers.

use super::{num, sublimation_pa, Reference, R_GAS};
use serde::Serialize;
use serde_json::{Map, Value as J};
use sse_core::math as m;
use std::collections::BTreeSet;

pub const KINDS: [&str; 8] = [
    "solid",
    "glass",
    "liquid",
    "gas",
    "supercritical fluid",
    "ionic fluid",
    "superionic solid",
    "metallic fluid",
];
/// `convention`: a line where nothing physical changes (supercritical
/// fluid against liquid and vapour), drawn by agreement.
const GRADES: [&str; 4] = ["measured", "computed", "extrapolated", "convention"];
/// A triple point lies on a boundary when the curve passes within this, in
/// the check's scaled plane (decades of pressure, hundreds of kelvin).
const MEET_TOL: f64 = 0.01;
/// Share of sampled boundary points whose table cells, a grid step either
/// side, must belong to the boundary's own two phases.
const TABLE_AGREE: f64 = 0.8;

/// A built phase table, held as runs along each pressure row (a fine grid
/// of millions of nodes is a few thousand runs).
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseTable {
    pub lp: (f64, f64, usize),
    /// Temperature axis; `log_t` says whether it is in log10 kelvin.
    pub t: (f64, f64, usize),
    pub log_t: bool,
    pub phases: Vec<String>,
    /// Per pressure row: (last node index of the run, phase).
    rows: Vec<Vec<(u32, u8)>>,
}

fn axis(v: Option<&J>) -> Option<(f64, f64, usize)> {
    let v = v?;
    let (lo, hi, n) = (
        v.get("lo")?.as_f64()?,
        v.get("hi")?.as_f64()?,
        v.get("n")?.as_u64()? as usize,
    );
    (hi > lo && n >= 2).then_some((lo, hi, n))
}

fn nearest(x: f64, a: (f64, f64, usize)) -> Option<usize> {
    if !(x >= a.0 && x <= a.1) {
        return None;
    }
    let f = (x - a.0) / (a.1 - a.0) * (a.2 - 1) as f64;
    Some((m::round(f) as usize).min(a.2 - 1))
}

fn digit(c: char, n: usize) -> Result<u8, String> {
    let k = c
        .to_digit(36)
        .ok_or_else(|| format!("table: {c:?} is not base 36"))?;
    if k as usize >= n {
        return Err(format!(
            "table: cell {c:?} names phase {k}, past the list's end"
        ));
    }
    Ok(k as u8)
}

impl PhaseTable {
    pub fn from_json(v: &J) -> Result<Self, String> {
        let lp = axis(v.get("log10_p_pa")).ok_or("table: log10_p_pa needs lo < hi and n ≥ 2")?;
        let (t, log_t) =
            match (axis(v.get("log10_t_k")), axis(v.get("t_k"))) {
                (Some(a), None) => (a, true),
                (None, Some(a)) => (a, false),
                _ => return Err(
                    "table: needs one temperature axis, t_k or log10_t_k, with lo < hi and n ≥ 2"
                        .into(),
                ),
            };
        let phases: Vec<String> = v
            .get("phases")
            .and_then(J::as_array)
            .ok_or("table: phases must be a list of ids")?
            .iter()
            .map(|x| {
                x.as_str()
                    .map(str::to_string)
                    .ok_or("table: a phase id is not text")
            })
            .collect::<Result<_, _>>()?;
        if phases.is_empty() || phases.len() > 36 {
            return Err(format!(
                "table: {} phases (1 to 36 fit base 36)",
                phases.len()
            ));
        }
        let np = phases.len();
        let mut rows = Vec::with_capacity(lp.2);
        match (
            v.get("cells").and_then(J::as_str),
            v.get("rle").and_then(J::as_str),
        ) {
            (Some(text), None) => {
                if text.chars().count() != lp.2 * t.2 {
                    return Err(format!(
                        "table: {} cells, not {} × {} = {}",
                        text.chars().count(),
                        lp.2,
                        t.2,
                        lp.2 * t.2
                    ));
                }
                let all: Vec<char> = text.chars().collect();
                for row in all.chunks(t.2) {
                    let mut runs: Vec<(u32, u8)> = vec![];
                    for (j, c) in row.iter().enumerate() {
                        let k = digit(*c, np)?;
                        match runs.last_mut() {
                            Some(r) if r.1 == k => r.0 = j as u32,
                            _ => runs.push((j as u32, k)),
                        }
                    }
                    rows.push(runs);
                }
            }
            (None, Some(text)) => {
                // Rows split by "/", runs by spaces: phase digit then the
                // run's length in base 36.
                for (i, row) in text.split('/').enumerate() {
                    let mut runs: Vec<(u32, u8)> = vec![];
                    let mut at = 0u64;
                    for run in row.split(' ').filter(|r| !r.is_empty()) {
                        let mut ch = run.chars();
                        let k = digit(ch.next().unwrap_or(' '), np)?;
                        let len = u64::from_str_radix(ch.as_str(), 36).map_err(|_| {
                            format!("table: row {i} run {run:?} has no base-36 length")
                        })?;
                        if len == 0 {
                            return Err(format!("table: row {i} has an empty run"));
                        }
                        at += len;
                        runs.push(((at - 1) as u32, k));
                    }
                    if at as usize != t.2 {
                        return Err(format!("table: row {i} holds {at} nodes, not {}", t.2));
                    }
                    rows.push(runs);
                }
                if rows.len() != lp.2 {
                    return Err(format!("table: {} rows, not {}", rows.len(), lp.2));
                }
            }
            _ => return Err("table: needs cells or rle (one of them)".into()),
        }
        Ok(PhaseTable {
            lp,
            t,
            log_t,
            phases,
            rows,
        })
    }

    /// The phase at the grid node nearest (p, T); `None` outside the table.
    pub fn at(&self, p_pa: f64, t_k: f64) -> Option<&str> {
        if p_pa <= 0.0 || t_k <= 0.0 {
            return None;
        }
        let i = nearest(m::log10(p_pa), self.lp)?;
        let x = if self.log_t { m::log10(t_k) } else { t_k };
        let j = nearest(x, self.t)? as u32;
        let row = &self.rows[i];
        let k = row.partition_point(|r| r.0 < j);
        Some(&self.phases[row[k].1 as usize])
    }

    /// A temperature `steps` grid steps from `t_k`.
    fn t_step(&self, t_k: f64, steps: f64) -> f64 {
        let d = steps * (self.t.1 - self.t.0) / (self.t.2 - 1) as f64;
        if self.log_t {
            t_k * m::pow(10.0, d)
        } else {
            t_k + d
        }
    }

    /// A pressure `steps` grid steps from `p_pa`.
    fn p_step(&self, p_pa: f64, steps: f64) -> f64 {
        p_pa * m::pow(
            10.0,
            steps * (self.lp.1 - self.lp.0) / (self.lp.2 - 1) as f64,
        )
    }
}

/// A sampled boundary: `(log10 p, T)` points.
type Line = Vec<(f64, f64)>;

/// A boundary curve sampled as a line of `(log10 p, T)` points.
fn sample(curve: &J, ph: &Map<String, J>) -> Result<Option<Line>, String> {
    let form = curve.get("form").and_then(J::as_str).unwrap_or("");
    match form {
        "points" | "table" => {
            let pts = curve
                .get("points")
                .and_then(J::as_array)
                .ok_or("a points curve needs points [[p_pa, t_k], ...]")?;
            let mut out = vec![];
            for q in pts {
                let (p, t) = (q.get(0).and_then(J::as_f64), q.get(1).and_then(J::as_f64));
                match (p, t) {
                    (Some(p), Some(t)) if p > 0.0 && t > 0.0 => out.push((m::log10(p), t)),
                    _ => return Err(format!("curve point {q} is not [p_pa > 0, t_k > 0]")),
                }
            }
            if out.len() < 2 {
                return Err("a points curve needs two points or more".into());
            }
            Ok(Some(out))
        }
        "vapour" => {
            let Some(fit) = ph.get("vapour_pressure") else {
                return Err("a vapour curve needs the row's vapour_pressure".into());
            };
            let r = fit
                .get("valid_k")
                .and_then(J::as_array)
                .and_then(|a| Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?)))
                .ok_or("vapour_pressure needs valid_k for a boundary")?;
            let mut out = vec![];
            for k in 0..=64 {
                let t = r.0 + (r.1 - r.0) * k as f64 / 64.0;
                if let Some(p) = sublimation_pa(ph, t).filter(|p| *p > 0.0) {
                    out.push((m::log10(p), t));
                }
            }
            Ok(Some(out))
        }
        "iapws_melt" => {
            let g = |k: &str| curve.get(k).and_then(J::as_f64);
            let (tn, pn) = (
                g("tn").ok_or("iapws_melt needs tn")?,
                g("pn").ok_or("iapws_melt needs pn")?,
            );
            let log = curve.get("log").and_then(J::as_bool).unwrap_or(false);
            let terms: Vec<(f64, f64)> = curve
                .get("terms")
                .and_then(J::as_array)
                .ok_or("iapws_melt needs terms [[a, b], ...]")?
                .iter()
                .filter_map(|x| Some((x.get(0)?.as_f64()?, x.get(1)?.as_f64()?)))
                .collect();
            let r = curve
                .get("valid_k")
                .and_then(J::as_array)
                .and_then(|a| Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?)))
                .ok_or("iapws_melt needs valid_k")?;
            let mut out = vec![];
            for k in 0..=64 {
                let t = r.0 + (r.1 - r.0) * k as f64 / 64.0;
                if let Some(p) = iapws_melt_pa(tn, pn, log, &terms, t) {
                    out.push((m::log10(p), t));
                }
            }
            Ok(Some(out))
        }
        // Other forms load (the data may run ahead of the reader) but are
        // not drawn into the checks.
        "" => Err("a curve needs a form".into()),
        _ => Ok(None),
    }
}

/// IAPWS R14-08 melting pressure, Pa.
pub fn iapws_melt_pa(tn: f64, pn: f64, log: bool, terms: &[(f64, f64)], t: f64) -> Option<f64> {
    if t <= 0.0 {
        return None;
    }
    let th = t / tn;
    let s: f64 = terms.iter().map(|(a, b)| a * (1.0 - m::pow(th, *b))).sum();
    let p = if log { pn * m::exp(s) } else { pn * (1.0 + s) };
    (p > 0.0 && p.is_finite()).then_some(p)
}

/// Distance from a point to a sampled line, in decades and hundreds of K.
fn distance(line: &[(f64, f64)], lp: f64, t: f64) -> f64 {
    let sc = |q: (f64, f64)| (q.0, q.1 / 100.0);
    let x = (lp, t / 100.0);
    let mut best = f64::INFINITY;
    for w in line.windows(2) {
        let (a, b) = (sc(w[0]), sc(w[1]));
        let d = (b.0 - a.0, b.1 - a.1);
        let l2 = d.0 * d.0 + d.1 * d.1;
        let u = if l2 > 0.0 {
            (((x.0 - a.0) * d.0 + (x.1 - a.1) * d.1) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let q = (a.0 + u * d.0 - x.0, a.1 + u * d.1 - x.1);
        best = best.min(m::sqrt(q.0 * q.0 + q.1 * q.1));
    }
    best
}

fn ids_of(list: &[J]) -> Vec<String> {
    list.iter()
        .filter_map(|x| x.as_str().map(str::to_string))
        .collect()
}

/// Load checks for one substance's phases (design section 4.3A).
///
/// `linked` holds the sources with a DOI or URL: a computed phase's evidence
/// must name one of them, so the paper that computed it is a link wherever
/// it is shown (Alex, 22 Nov 01:08).
pub fn check(
    id: &str,
    ph: &Map<String, J>,
    sources: &BTreeSet<String>,
    linked: &BTreeSet<String>,
) -> Vec<String> {
    let mut p = vec![];
    let src_ok = |v: Option<&J>, what: &str, p: &mut Vec<String>| match v.and_then(J::as_str) {
        Some(s) if sources.contains(s) => {}
        Some(s) => p.push(format!(
            "substance {id}: {what} names source {s:?}, not in sources"
        )),
        None => p.push(format!("substance {id}: {what} has no src")),
    };
    // Forms.
    let mut forms: Vec<(String, bool)> = vec![];
    if let Some(list) = ph.get("forms") {
        let Some(list) = list.as_array() else {
            p.push(format!("substance {id}: phases.forms must be a list"));
            return p;
        };
        for f in list {
            let Some(fid) = f.get("id").and_then(J::as_str) else {
                p.push(format!("substance {id}: a phase form has no id"));
                continue;
            };
            if !fid.starts_with(&format!("{id}.")) {
                p.push(format!(
                    "substance {id}: phase {fid} should be named {id}.<phase>"
                ));
            }
            if forms.iter().any(|x| x.0 == fid) {
                p.push(format!("substance {id}: phase {fid} is written twice"));
            }
            match f.get("kind").and_then(J::as_str) {
                Some(k) if KINDS.contains(&k) => {}
                k => p.push(format!(
                    "substance {id}: phase {fid} kind {k:?} is not one of {}",
                    KINDS.join(", ")
                )),
            }
            let stable = match f.get("stability").and_then(J::as_str) {
                None | Some("stable") => true,
                Some("metastable") => false,
                Some(s) => {
                    p.push(format!(
                        "substance {id}: phase {fid} stability {s:?} is not stable or metastable"
                    ));
                    true
                }
            };
            let ev = f.get("evidence");
            match ev.and_then(|e| e.get("grade")).and_then(J::as_str) {
                Some("computed") => {
                    let s = ev.and_then(|e| e.get("src")).and_then(J::as_str);
                    if s.is_some_and(|s| sources.contains(s) && !linked.contains(s)) {
                        p.push(format!(
                            "substance {id}: phase {fid} is computed, so its source {} needs a doi or url",
                            s.unwrap_or("")
                        ));
                    }
                }
                Some(g) if GRADES.contains(&g) => {}
                g => p.push(format!(
                    "substance {id}: phase {fid} evidence grade {g:?} is not one of {}",
                    GRADES.join(", ")
                )),
            }
            src_ok(
                ev.and_then(|e| e.get("src")),
                &format!("phase {fid}'s evidence"),
                &mut p,
            );
            if let Some(props) = f.get("props").and_then(J::as_object) {
                let mut vals = vec![];
                super::values(props, "", &mut vals);
                for (path, v) in vals {
                    src_ok(v.get("src"), &format!("phase {fid} {path}"), &mut p);
                }
            }
            forms.push((fid.to_string(), stable));
        }
    }
    let known = |x: &str| forms.iter().any(|f| f.0 == x);
    // Boundaries.
    let mut lines: Vec<(BTreeSet<String>, Line)> = vec![];
    if let Some(list) = ph.get("boundaries").and_then(J::as_array) {
        for b in list {
            let between = b
                .get("between")
                .and_then(J::as_array)
                .map(|a| ids_of(a))
                .unwrap_or_default();
            let name = between.join(" | ");
            if between.len() != 2 || between[0] == between[1] {
                p.push(format!(
                    "substance {id}: a boundary must be between two phases, not [{name}]"
                ));
                continue;
            }
            for x in &between {
                if !known(x) {
                    p.push(format!(
                        "substance {id}: boundary {name} names {x}, not in phases.forms"
                    ));
                }
            }
            src_ok(b.get("src"), &format!("boundary {name}"), &mut p);
            let computed = b
                .get("evidence")
                .and_then(|e| e.get("grade"))
                .and_then(J::as_str)
                == Some("computed");
            if let Some(src) = b.get("src").and_then(J::as_str) {
                if computed && sources.contains(src) && !linked.contains(src) {
                    p.push(format!(
                        "substance {id}: boundary {name} is computed, so its source {src} needs a doi or url"
                    ));
                }
            }
            match b.get("curve").map(|c| sample(c, ph)) {
                None => p.push(format!("substance {id}: boundary {name} has no curve")),
                Some(Err(e)) => p.push(format!("substance {id}: boundary {name}: {e}")),
                Some(Ok(Some(line))) => lines.push((between.into_iter().collect(), line)),
                Some(Ok(None)) => {}
            }
        }
    }
    // Triple points.
    let mut seen: Vec<BTreeSet<String>> = vec![];
    if let Some(list) = ph.get("meets").and_then(J::as_array) {
        for t in list {
            let set: BTreeSet<String> = t
                .get("phases")
                .and_then(J::as_array)
                .map(|a| ids_of(a).into_iter().collect())
                .unwrap_or_default();
            let name = set.iter().cloned().collect::<Vec<_>>().join(", ");
            if set.len() != 3 {
                p.push(format!(
                    "substance {id}: a triple point needs three different phases, not [{name}]"
                ));
                continue;
            }
            if seen.contains(&set) {
                p.push(format!(
                    "substance {id}: triple point of {name} is written twice"
                ));
            }
            for x in &set {
                if !known(x) {
                    p.push(format!(
                        "substance {id}: triple point {name} names {x}, not in phases.forms"
                    ));
                }
            }
            src_ok(t.get("src"), &format!("triple point {name}"), &mut p);
            let (Some(tk), Some(pa)) = (num(t.get("t_k")), num(t.get("p_pa"))) else {
                p.push(format!(
                    "substance {id}: triple point {name} needs t_k and p_pa"
                ));
                continue;
            };
            // The familiar one is the row's own triple point.
            let familiar: BTreeSet<String> = ["ice_ih", "liquid", "vapour"]
                .iter()
                .map(|s| format!("{id}.{s}"))
                .collect();
            if set == familiar {
                if let (Some(t0), Some(p0)) = (
                    num(ph.get("triple_point_k")),
                    num(ph.get("triple_point_pa")),
                ) {
                    if (tk - t0).abs() > 1e-6 * t0 || (pa - p0).abs() > 1e-6 * p0 {
                        p.push(format!(
                            "substance {id}: triple point of {name} ({tk} K, {pa} Pa) differs from triple_point_k and triple_point_pa ({t0} K, {p0} Pa)"
                        ));
                    }
                }
            }
            // Every boundary between two of its phases passes through it.
            for (pair, line) in &lines {
                if pair.is_subset(&set) {
                    let d = distance(line, m::log10(pa), tk);
                    if d > MEET_TOL {
                        p.push(format!(
                            "substance {id}: boundary {} misses the triple point of {name} ({tk} K, {pa:.4e} Pa) by {d:.3} (decades and hundreds of K)",
                            pair.iter().cloned().collect::<Vec<_>>().join(" | ")
                        ));
                    }
                }
            }
            seen.push(set);
        }
    }
    // The built table.
    if let Some(tv) = ph.get("table") {
        match PhaseTable::from_json(tv) {
            Err(e) => p.push(format!("substance {id}: {e}")),
            Ok(table) => {
                for x in &table.phases {
                    match forms.iter().find(|f| &f.0 == x) {
                        None => p.push(format!("substance {id}: table names {x}, not in phases.forms")),
                        Some((_, false)) => p.push(format!(
                            "substance {id}: table holds {x}, which is metastable: the map shows what forms on a slow path"
                        )),
                        _ => {}
                    }
                }
                // Each sampled boundary has its own two phases either side,
                // across it in temperature or in pressure (a boundary nearly
                // flat in one is crossed in the other).
                for (pair, line) in &lines {
                    let (mut on, mut n) = (0usize, 0usize);
                    for (lp, t) in line {
                        let pa = m::pow(10.0, *lp);
                        let sides = [
                            (
                                table.at(pa, table.t_step(*t, -1.5)),
                                table.at(pa, table.t_step(*t, 1.5)),
                            ),
                            (
                                table.at(table.p_step(pa, -1.5), *t),
                                table.at(table.p_step(pa, 1.5), *t),
                            ),
                        ];
                        let inside: Vec<(&str, &str)> = sides
                            .iter()
                            .filter_map(|(a, b)| Some(((*a)?, (*b)?)))
                            .collect();
                        if inside.is_empty() {
                            continue;
                        }
                        n += 1;
                        if inside
                            .iter()
                            .any(|(a, b)| a != b && pair.contains(*a) && pair.contains(*b))
                        {
                            on += 1;
                        }
                    }
                    if n >= 4 && (on as f64) < TABLE_AGREE * n as f64 {
                        p.push(format!(
                            "substance {id}: the table puts other phases beside boundary {} at {} of {n} sampled points",
                            pair.iter().cloned().collect::<Vec<_>>().join(" | "),
                            n - on
                        ));
                    }
                }
            }
        }
    }
    p
}

/// `phase_at`'s answer.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PhaseAt {
    pub substance: String,
    pub p_pa: f64,
    pub t_k: f64,
    /// The phase id (`h2o.ice_vii`), or the familiar fallback
    /// (`<id>.solid`, `.liquid`, `.vapour`).
    pub phase: String,
    /// The phase's row from `phases.forms`, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<J>,
    /// "table" (the built phase table) or "familiar" (triple point and
    /// vapour curve only).
    pub from: String,
    /// The phase's evidence grade (measured, computed, extrapolated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade: Option<String>,
    /// The source row its evidence names (`{id, text, doi?, url?}`), so a
    /// computed phase links to the paper that computed it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<J>,
    pub why: String,
}

impl Reference {
    /// The phase of a substance at a pressure and temperature, and its row.
    pub fn phase_at(&self, substance: &str, p_pa: f64, t_k: f64) -> Result<PhaseAt, String> {
        let row = self
            .row("substances", substance)
            .ok_or_else(|| format!("no substance {substance}"))?;
        if !(p_pa > 0.0 && t_k > 0.0) {
            return Err("pressure and temperature must be above zero".into());
        }
        let ph = row.get("phases").and_then(J::as_object);
        let form_of = |pid: &str| -> Option<J> {
            ph?.get("forms")?
                .as_array()?
                .iter()
                .find(|f| f.get("id").and_then(J::as_str) == Some(pid))
                .cloned()
        };
        if let Some(table) = self.phase_tables.get(substance) {
            if let Some(pid) = table.at(p_pa, t_k) {
                let form = form_of(pid);
                let ev = form.as_ref().and_then(|f| f.get("evidence"));
                let grade = ev
                    .and_then(|e| e.get("grade"))
                    .and_then(J::as_str)
                    .map(str::to_string);
                let source = ev
                    .and_then(|e| e.get("src"))
                    .and_then(J::as_str)
                    .and_then(|s| {
                        let mut row = self.sources.get(s)?.clone();
                        row.insert("id".into(), J::String(s.into()));
                        Some(J::Object(row))
                    });
                return Ok(PhaseAt {
                    substance: substance.into(),
                    p_pa,
                    t_k,
                    phase: pid.into(),
                    form,
                    from: "table".into(),
                    grade,
                    source,
                    why: format!(
                        "read from {substance}'s phase table, built from its phase boundaries, at the grid point nearest {p_pa:.4e} Pa and {t_k} K"
                    ),
                });
            }
        }
        // The familiar three phases.
        let ph = ph.ok_or_else(|| format!("{substance} has no phases"))?;
        let (Some(tt), Some(pt)) = (
            num(ph.get("triple_point_k")),
            num(ph.get("triple_point_pa")),
        ) else {
            return Err(format!(
                "{substance} has no phase table and no triple point"
            ));
        };
        let ps = sublimation_pa(ph, t_k.min(tt));
        let (phase, why) = if t_k < tt {
            match ps {
                Some(ps) if p_pa < ps => ("vapour", format!("below the triple point ({tt} K) and under the sublimation pressure {ps:.4e} Pa")),
                _ => ("solid", format!("below the triple point ({tt} K) and above the sublimation pressure")),
            }
        } else {
            // Boiling line: Clausius-Clapeyron from the triple point with the
            // row's latent heat of vaporisation, else the triple pressure.
            let lv = num(ph.get("latent_vaporisation"))
                .zip(self.molar_mass_kg_mol(substance))
                .map(|(l, mm)| l * mm);
            let (pb, how) = match lv {
                Some(l) => (
                    pt * m::exp(-l / R_GAS * (1.0 / t_k - 1.0 / tt)),
                    "the boiling line (Clausius-Clapeyron from the triple point with the latent heat of vaporisation)",
                ),
                None => (pt, "the triple point's pressure (the row has no latent heat of vaporisation)"),
            };
            if p_pa < pb {
                (
                    "vapour",
                    format!("above the triple point ({tt} K) and below {how}, {pb:.4e} Pa"),
                )
            } else {
                ("liquid", format!("above the triple point ({tt} K) and above {how}, {pb:.4e} Pa; the row has no melting line or critical point, so this is the familiar reading"))
            }
        };
        Ok(PhaseAt {
            substance: substance.into(),
            p_pa,
            t_k,
            phase: format!("{substance}.{phase}"),
            form: None,
            from: "familiar".into(),
            grade: None,
            source: None,
            why: format!("{substance} has no phase table: {why}"),
        })
    }
}
