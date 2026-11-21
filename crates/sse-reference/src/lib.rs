//! The master data sheet (matter pack design, MP1): one place for the
//! constants, elements, isotopes, substances, mixtures and reactions that
//! every block reads, as pack data under `provides.reference`.
//!
//! - **Tables.** `constants`, `elements`, `isotopes`, `substances`,
//!   `mixtures`, `reactions` (arrays of rows with an `id`) and `sources`
//!   (`[{id, text, doi?, url?}]`).
//! - **Values.** `{"v": number, "unit": "...", "src": "<source id>"}`, with
//!   optional `valid` and `status`; a fit is `{"fit": name, "coeffs": [...],
//!   "valid_k": [lo, hi], "src": ...}`. A value is any object holding `v` or
//!   `fit`, at the top of a row or inside a property group (`phases`,
//!   `thermal`, ...).
//! - **Stacking.** A later pack's row with an id already present overrides
//!   value by value; each value it sets must say what it `replaces` (the
//!   pack it overrides), so no real value changes silently. New rows need
//!   nothing.
//! - **Checks at load** (design section 4.6): every `src` names a source;
//!   a substance's formula names known elements, and a typed molar mass
//!   agrees with the one worked out from it; a vapour curve passes through
//!   its own triple point; a mixture's recipe sums to one and names known
//!   substances; every reaction conserves elements and charge.

use serde::Serialize;
use serde_json::{Map, Value as J};
use sse_core::math as m;
use std::collections::BTreeMap;

/// The tables, in the order they are checked.
pub const TABLES: [&str; 6] = [
    "constants",
    "elements",
    "isotopes",
    "substances",
    "mixtures",
    "reactions",
];

/// Gas constant for the vapour-curve slope check, J/(mol K) (CODATA 2018,
/// exact). MP1c moves every constant onto the sheet; until then this one
/// stays here, the only one the reader itself needs.
const R_GAS: f64 = 8.314_462_618;

/// Relative tolerance on a typed molar mass.
const MOLAR_TOL: f64 = 1e-3;
/// Tolerance on ln P at the triple point (5 percent).
const TRIPLE_TOL: f64 = 0.05;
/// Tolerance on a recipe's sum: room for trace gases below 0.01 percent
/// that a recipe leaves out (US Standard Atmosphere 1976 dry air lists
/// four gases summing to 0.99997).
const RECIPE_TOL: f64 = 1e-4;
/// Relative tolerance between a latent heat and the vapour curve's slope.
const LATENT_TOL: f64 = 0.15;

#[derive(Clone, PartialEq, Debug, Default, Serialize)]
pub struct Reference {
    /// Table name to rows by id.
    pub tables: BTreeMap<String, BTreeMap<String, Map<String, J>>>,
    pub sources: BTreeMap<String, Map<String, J>>,
    /// Where elements were made (design 8.1): source id to its name in the
    /// three wordings. Element rows point at these from `origin.shares`.
    pub origin_legend: BTreeMap<String, Map<String, J>>,
    /// The pack each row came from (`table/id`), and the packs that changed it.
    pub origin: BTreeMap<String, Vec<String>>,
}

fn is_value(v: &J) -> bool {
    v.as_object()
        .is_some_and(|o| o.contains_key("v") || o.contains_key("fit"))
}

/// Every value in a row, with its path (`phases.triple_point_k`).
fn values<'a>(row: &'a Map<String, J>, prefix: &str, out: &mut Vec<(String, &'a J)>) {
    for (k, v) in row {
        let path = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        if is_value(v) {
            out.push((path, v));
        } else if let Some(o) = v.as_object() {
            values(o, &path, out);
        }
    }
}

/// Merge `over` into `base` value by value (property groups merge too).
fn merge(base: &mut Map<String, J>, over: &Map<String, J>) {
    for (k, v) in over {
        match (base.get_mut(k), v) {
            (Some(J::Object(b)), J::Object(o)) if !is_value(v) => merge(b, o),
            _ => {
                base.insert(k.clone(), v.clone());
            }
        }
    }
}

fn num(v: Option<&J>) -> Option<f64> {
    let v = v?;
    v.get("v").and_then(J::as_f64).or_else(|| v.as_f64())
}

impl Reference {
    /// Read `provides.reference` from each pack, bottom to top, and check
    /// the result. Problems are readable sentences, all of them at once.
    pub fn from_packs(packs: &[sse_pack::PackManifest]) -> Result<Self, Vec<String>> {
        let found: Vec<(String, J)> = packs
            .iter()
            .filter(|p| !p.provides.reference.is_empty())
            .map(|p| {
                (
                    p.id.clone(),
                    serde_json::to_value(&p.provides.reference).unwrap_or(J::Null),
                )
            })
            .collect();
        Self::from_json(&found)
    }

    /// As [`from_packs`](Self::from_packs), from `(pack id, reference block)` pairs.
    pub fn from_json(packs: &[(String, J)]) -> Result<Self, Vec<String>> {
        let mut r = Reference::default();
        let mut problems = vec![];
        for (pack, block) in packs {
            let Some(block) = block.as_object() else {
                problems.push(format!("{pack}: reference must be an object of tables"));
                continue;
            };
            for (table, rows) in block {
                // A note on the block, for people and Orac.
                if table == "about" && rows.is_string() {
                    continue;
                }
                let known = TABLES.contains(&table.as_str())
                    || table == "sources"
                    || table == "origin_legend";
                if !known {
                    problems.push(format!(
                        "{pack}: reference table {table} is not one of {}, sources, origin_legend",
                        TABLES.join(", ")
                    ));
                    continue;
                }
                let Some(rows) = rows.as_array() else {
                    problems.push(format!("{pack}: reference.{table} must be a list of rows"));
                    continue;
                };
                for row in rows {
                    let Some(row) = row.as_object() else {
                        problems.push(format!("{pack}: a row of {table} is not an object"));
                        continue;
                    };
                    let Some(id) = row.get("id").and_then(J::as_str) else {
                        problems.push(format!("{pack}: a row of {table} has no id"));
                        continue;
                    };
                    if table == "sources" {
                        r.sources.insert(id.to_string(), row.clone());
                        continue;
                    }
                    if table == "origin_legend" {
                        r.origin_legend.insert(id.to_string(), row.clone());
                        continue;
                    }
                    let key = format!("{table}/{id}");
                    let t = r.tables.entry(table.clone()).or_default();
                    match t.get_mut(id) {
                        None => {
                            t.insert(id.to_string(), row.clone());
                            r.origin.insert(key, vec![pack.clone()]);
                        }
                        Some(base) => {
                            let mut vs = vec![];
                            values(row, "", &mut vs);
                            for (path, v) in &vs {
                                if v.get("replaces").and_then(J::as_str).is_none() {
                                    problems.push(format!(
                                        "{pack}: {table} {id}: {path} overrides a value from {} without saying what it replaces (add \"replaces\")",
                                        r.origin.get(&key).and_then(|o| o.last()).cloned().unwrap_or_default()
                                    ));
                                }
                            }
                            merge(base, row);
                            r.origin.entry(key).or_default().push(pack.clone());
                        }
                    }
                }
            }
        }
        problems.extend(r.check());
        if problems.is_empty() {
            Ok(r)
        } else {
            Err(problems)
        }
    }

    pub fn row(&self, table: &str, id: &str) -> Option<&Map<String, J>> {
        self.tables.get(table)?.get(id)
    }

    /// A substance's formula as element counts.
    pub fn formula(&self, id: &str) -> Option<BTreeMap<String, f64>> {
        let f = self.row("substances", id)?.get("formula")?.as_object()?;
        Some(
            f.iter()
                .filter_map(|(e, n)| n.as_f64().map(|n| (e.clone(), n)))
                .collect(),
        )
    }

    /// Molar mass worked out from the formula, kg/mol (atomic weights are g/mol).
    /// `None` when an element in the formula has no standard atomic weight
    /// (the elements with no stable isotope).
    pub fn molar_mass_kg_mol(&self, id: &str) -> Option<f64> {
        let f = self.formula(id)?;
        let mut sum = 0.0;
        for (e, n) in f {
            let w = num(self.row("elements", &e)?.get("atomic_weight"))?;
            sum += n * w;
        }
        Some(sum / 1000.0)
    }

    fn charge(&self, id: &str) -> f64 {
        self.row("substances", id)
            .and_then(|r| r.get("charge"))
            .and_then(J::as_f64)
            .unwrap_or(0.0)
    }

    /// The load checks of design section 4.6.
    fn check(&self) -> Vec<String> {
        let mut p = vec![];
        // Element origins (design 8.1): shares name legend entries and sum
        // to 1; every source named resolves.
        for (id, row) in self.tables.get("elements").into_iter().flatten() {
            p.extend(self.check_origin(id, row));
        }
        // Every value names a source.
        for (table, rows) in &self.tables {
            for (id, row) in rows {
                let mut vs = vec![];
                values(row, "", &mut vs);
                for (path, v) in vs {
                    match v.get("src").and_then(J::as_str) {
                        None => p.push(format!("{table} {id}: {path} has no source (src)")),
                        Some(s) if !self.sources.contains_key(s) => p.push(format!(
                            "{table} {id}: {path} cites source {s}, which is not in reference.sources"
                        )),
                        _ => {}
                    }
                }
            }
        }
        let empty = BTreeMap::new();
        let elements = self.tables.get("elements").unwrap_or(&empty);
        for (id, row) in self.tables.get("substances").unwrap_or(&empty) {
            match row.get("formula").and_then(J::as_object) {
                None => p.push(format!("substance {id} has no formula (element counts)")),
                Some(f) => {
                    for e in f.keys().filter(|e| !elements.contains_key(*e)) {
                        p.push(format!("substance {id}: formula names element {e}, which is not in the elements table"));
                    }
                }
            }
            if let (Some(typed), Some(worked)) =
                (num(row.get("molar_mass")), self.molar_mass_kg_mol(id))
            {
                // A typed molar mass is in the unit it declares (kg/mol or g/mol).
                let unit = row
                    .get("molar_mass")
                    .and_then(|v| v.get("unit"))
                    .and_then(J::as_str)
                    .unwrap_or("kg/mol");
                let typed = if unit == "g/mol" {
                    typed / 1000.0
                } else {
                    typed
                };
                if ((typed - worked) / worked).abs() > MOLAR_TOL {
                    p.push(format!(
                        "substance {id}: molar mass {typed:.6} kg/mol disagrees with its formula, which gives {worked:.6} kg/mol"
                    ));
                }
            }
            if let Some(ph) = row.get("phases").and_then(J::as_object) {
                p.extend(check_vapour(id, ph));
            }
        }
        let substances = self.tables.get("substances").unwrap_or(&empty);
        for (id, row) in self.tables.get("mixtures").unwrap_or(&empty) {
            let Some(recipe) = row.get("recipe").and_then(J::as_object) else {
                p.push(format!("mixture {id} has no recipe"));
                continue;
            };
            // An empty recipe is a bulk-only mixture (rock, magma and the
            // formation materials for now): its values are its own.
            if recipe.is_empty() {
                continue;
            }
            let mut sum = 0.0;
            for (s, share) in recipe {
                if !substances.contains_key(s)
                    && !self
                        .tables
                        .get("mixtures")
                        .is_some_and(|x| x.contains_key(s))
                {
                    p.push(format!("mixture {id}: recipe names substance {s}, which is not in the substances table"));
                }
                sum += num(Some(share)).unwrap_or(0.0);
            }
            if (sum - 1.0).abs() > RECIPE_TOL {
                p.push(format!("mixture {id}: recipe sums to {sum}, not 1"));
            }
        }
        for (id, row) in self.tables.get("reactions").unwrap_or(&empty) {
            let Some(terms) = row.get("terms").and_then(J::as_array) else {
                p.push(format!("reaction {id} has no terms"));
                continue;
            };
            let mut atoms: BTreeMap<String, f64> = BTreeMap::new();
            let mut charge = 0.0;
            let mut ok = true;
            for t in terms {
                let (Some(s), Some(nu)) =
                    (t.get(0).and_then(J::as_str), t.get(1).and_then(J::as_f64))
                else {
                    p.push(format!(
                        "reaction {id}: a term is not [substance, coefficient]"
                    ));
                    ok = false;
                    continue;
                };
                let Some(f) = self.formula(s) else {
                    p.push(format!(
                        "reaction {id}: names substance {s}, which is not in the substances table"
                    ));
                    ok = false;
                    continue;
                };
                for (e, n) in f {
                    *atoms.entry(e).or_default() += nu * n;
                }
                charge += nu * self.charge(s);
            }
            if ok {
                let off: Vec<String> = atoms
                    .iter()
                    .filter(|(_, n)| n.abs() > 1e-9)
                    .map(|(e, n)| format!("{e} {n:+}"))
                    .collect();
                if !off.is_empty() {
                    p.push(format!(
                        "reaction {id} does not conserve elements: {}",
                        off.join(", ")
                    ));
                }
                if charge.abs() > 1e-9 {
                    p.push(format!(
                        "reaction {id} does not conserve charge: {charge:+}"
                    ));
                }
            }
        }
        p
    }

    /// One row as `read_matter` shows it: the row, its worked-out values
    /// (molar mass), its sources and the packs it came from.
    pub fn describe(&self, table: &str, id: &str) -> Option<J> {
        let row = self.row(table, id)?;
        let mut vs = vec![];
        values(row, "", &mut vs);
        let mut cited: Vec<String> = vs
            .iter()
            .filter_map(|(_, v)| v.get("src").and_then(J::as_str).map(str::to_string))
            .collect();
        cited.sort();
        cited.dedup();
        let sources: Vec<J> = cited
            .iter()
            .filter_map(|s| self.sources.get(s).cloned().map(J::Object))
            .collect();
        let mut out = serde_json::json!({
            "table": table,
            "id": id,
            "row": row,
            "sources": sources,
            "packs": self.origin.get(&format!("{table}/{id}")).cloned().unwrap_or_default(),
            // Blocks declare what they read in MP1c; until then nobody is listed.
            "used_by": [],
            "skipped_by": [],
        });
        if table == "substances" {
            if let Some(mm) = self.molar_mass_kg_mol(id) {
                out["worked"] = serde_json::json!({"molar_mass": {"v": mm, "unit": "kg/mol", "from": "formula and atomic weights"}});
            }
        }
        Some(out)
    }

    /// `read_matter` with no id: each table's row ids.
    pub fn listing(&self, table: Option<&str>) -> J {
        let mut out = Map::new();
        for t in TABLES.iter().filter(|t| table.is_none_or(|x| x == **t)) {
            let ids: Vec<&String> = self
                .tables
                .get(*t)
                .map(|r| r.keys().collect())
                .unwrap_or_default();
            out.insert(t.to_string(), serde_json::json!(ids));
        }
        if table.is_none_or(|x| x == "sources") {
            out.insert(
                "sources".into(),
                serde_json::json!(self.sources.keys().collect::<Vec<_>>()),
            );
        }
        // The legend is seven short rows, so it comes whole: the Reference
        // screen colours elements by it.
        if table.is_none_or(|x| x == "origin_legend") {
            out.insert(
                "origin_legend".into(),
                serde_json::json!(self.origin_legend),
            );
        }
        J::Object(out)
    }
}

/// Pressure in Pa from a vapour-pressure fit at `t` K.
pub fn vapour_pressure_pa(fit: &J, t: f64) -> Option<f64> {
    let coeffs: Vec<f64> = fit
        .get("coeffs")?
        .as_array()?
        .iter()
        .filter_map(J::as_f64)
        .collect();
    match fit.get("fit")?.as_str()? {
        // ln P(bar) = A0 + sum_i A_i / T^i (Fray and Schmitt 2009).
        "fray-schmitt-2009" => {
            let mut lnp = *coeffs.first()?;
            let mut tp = 1.0;
            for a in &coeffs[1..] {
                tp *= t;
                lnp += a / tp;
            }
            Some(m::exp(lnp) * 1e5)
        }
        // [p_ref Pa, t_ref K, latent heat J/mol].
        "clausius-clapeyron" => {
            let (p0, t0, l) = (*coeffs.first()?, *coeffs.get(1)?, *coeffs.get(2)?);
            Some(p0 * m::exp(-l / R_GAS * (1.0 / t - 1.0 / t0)))
        }
        _ => None,
    }
}

impl Reference {
    fn check_origin(&self, id: &str, row: &Map<String, J>) -> Vec<String> {
        let mut p = vec![];
        let Some(o) = row.get("origin").and_then(J::as_object) else {
            return p;
        };
        if let Some(shares) = o.get("shares").and_then(J::as_object) {
            let mut sum = 0.0;
            for (k, v) in shares {
                if !self.origin_legend.contains_key(k) {
                    p.push(format!(
                        "element {id}: origin share {k} is not in origin_legend"
                    ));
                }
                match v.as_f64() {
                    Some(x) if (0.0..=1.0).contains(&x) => sum += x,
                    _ => p.push(format!("element {id}: origin share {k} is not a fraction")),
                }
            }
            if !shares.is_empty() && (sum - 1.0).abs() > RECIPE_TOL {
                p.push(format!("element {id}: origin shares sum to {sum}, not 1"));
            }
        }
        let named = o.get("src").and_then(J::as_str).into_iter().chain(
            o.get("also")
                .and_then(J::as_array)
                .into_iter()
                .flatten()
                .filter_map(J::as_str),
        );
        for s in named {
            if !self.sources.contains_key(s) {
                p.push(format!(
                    "element {id}: origin names source {s}, which is not in sources"
                ));
            }
        }
        p
    }
}

fn check_vapour(id: &str, ph: &Map<String, J>) -> Vec<String> {
    let mut p = vec![];
    let Some(fit) = ph.get("vapour_pressure") else {
        return p;
    };
    let name = fit.get("fit").and_then(J::as_str).unwrap_or("");
    let (Some(tt), Some(pt)) = (
        num(ph.get("triple_point_k")),
        num(ph.get("triple_point_pa")),
    ) else {
        return p;
    };
    let Some(at) = vapour_pressure_pa(fit, tt) else {
        p.push(format!("substance {id}: vapour curve fit {name:?} is not one the reader knows (fray-schmitt-2009, clausius-clapeyron)"));
        return p;
    };
    let miss = m::ln(at / pt);
    if miss.abs() > TRIPLE_TOL {
        p.push(format!(
            "substance {id}: vapour curve gives {at:.4e} Pa at the triple point ({tt} K), not the triple-point pressure {pt:.4e} Pa"
        ));
    }
    // The latent heat agrees with the curve's slope: d ln P / d(1/T) = -L/R.
    if let Some(l) = num(ph.get("latent_heat_sublimation_j_mol")) {
        let h = 1e-3 * tt;
        if let (Some(a), Some(b)) = (
            vapour_pressure_pa(fit, tt - h),
            vapour_pressure_pa(fit, tt + h),
        ) {
            let slope = (m::ln(b) - m::ln(a)) / (1.0 / (tt + h) - 1.0 / (tt - h));
            let from_curve = -slope * R_GAS;
            if ((from_curve - l) / l).abs() > LATENT_TOL {
                p.push(format!(
                    "substance {id}: latent heat {l:.0} J/mol disagrees with the vapour curve's slope, which gives {from_curve:.0} J/mol"
                ));
            }
        }
    }
    p
}

#[cfg(test)]
mod tests;
