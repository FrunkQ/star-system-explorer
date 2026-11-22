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

pub mod expr;
pub mod lookup;
pub mod phases;

/// Relative tolerance between a sheet unit and the built-in one of the
/// same id.
const UNIT_TOL: f64 = 1e-9;

/// A unit row's dimension: `dim` (seven SI exponents) or a named `measures`.
fn unit_dim(row: &Map<String, J>) -> Option<sse_core::Dimension> {
    use sse_core::Dimension as D;
    if let Some(a) = row.get("dim").and_then(J::as_array) {
        let v: Vec<i8> = a
            .iter()
            .filter_map(|x| x.as_i64())
            .map(|x| x as i8)
            .collect();
        return <[i8; 7]>::try_from(v).ok().map(D);
    }
    Some(match row.get("measures").and_then(J::as_str)? {
        "dimensionless" | "angle" => D::NONE,
        "length" => D::LENGTH,
        "mass" => D::MASS,
        "time" => D::TIME,
        "temperature" => D::TEMPERATURE,
        "current" => D([0, 0, 0, 1, 0, 0, 0]),
        "amount" => D([0, 0, 0, 0, 0, 1, 0]),
        "luminous_intensity" => D([0, 0, 0, 0, 0, 0, 1]),
        "power" => D::POWER,
        "energy" => D([2, 1, -2, 0, 0, 0, 0]),
        "force" => D([1, 1, -2, 0, 0, 0, 0]),
        "frequency" => D([0, 0, -1, 0, 0, 0, 0]),
        "pressure" => D::PRESSURE,
        "flux" => D::FLUX,
        "velocity" => D::VELOCITY,
        "acceleration" => D::ACCELERATION,
        "density" => D::DENSITY,
        "magnetic_field" => D::MAGNETIC_FIELD,
        "magnetic_moment" => D::MAGNETIC_MOMENT,
        _ => return None,
    })
}

/// Relative tolerance between a derived constant's stored value and the one
/// worked out from its formula.
const DERIVED_TOL: f64 = 1e-9;

/// The tables, in the order they are checked.
pub const TABLES: [&str; 7] = [
    "constants",
    "units",
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
    /// The Constants page's groups (DM1): name, about and cited notes, by
    /// group id (`nature`, `convention`, `factor`, `cosmos`).
    pub groups: BTreeMap<String, Map<String, J>>,
    /// The pack each row came from (`table/id`), and the packs that changed it.
    pub origin: BTreeMap<String, Vec<String>>,
    /// Each substance's phase table, decoded at load (MP17).
    #[serde(skip)]
    pub phase_tables: BTreeMap<String, phases::PhaseTable>,
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
                    || table == "origin_legend"
                    || table == "groups";
                if !known {
                    problems.push(format!(
                        "{pack}: reference table {table} is not one of {}, sources, origin_legend, groups",
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
                    if table == "groups" {
                        r.groups.insert(id.to_string(), row.clone());
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
            // Phase tables decoded once (MP17), so `phase_at` is a lookup.
            let mut built = BTreeMap::new();
            for (id, row) in r.tables.get("substances").into_iter().flatten() {
                if let Some(t) = row.get("phases").and_then(|p| p.get("table")) {
                    if let Ok(t) = phases::PhaseTable::from_json(t) {
                        built.insert(id.clone(), t);
                    }
                }
            }
            r.phase_tables = built;
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

    /// A constant's value in SI. A derived constant (`derived_from`) is
    /// worked out from its formula every time, never read as typed, so a
    /// changed input moves it (design section 7A.1).
    pub fn constant(&self, id: &str) -> Option<f64> {
        self.constant_at(id, 0).ok()
    }

    fn constant_at(&self, id: &str, depth: u32) -> Result<f64, String> {
        if depth > 16 {
            return Err(format!("constant {id}: its formula goes round in a circle"));
        }
        let row = self
            .row("constants", id)
            .ok_or_else(|| format!("names {id}, which is not a constant"))?;
        match row.get("derived_from").and_then(J::as_str) {
            Some(f) => {
                let mut err = None;
                let v = expr::eval(f, |n| match self.constant_at(n, depth + 1) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        err.get_or_insert(e);
                        None
                    }
                });
                match (v, err) {
                    (Ok(v), _) => Ok(v),
                    (Err(_), Some(e)) => Err(e),
                    (Err(e), None) => Err(e),
                }
            }
            None => num(row.get("value")).ok_or_else(|| format!("constant {id} has no value")),
        }
    }

    /// Whether constant `id` is `on` or is worked out from it, at any depth
    /// (DM1: Omega_m rests on Omega_c, so a block reading Omega_m uses a
    /// changed dark matter amount).
    pub fn rests_on(&self, id: &str, on: &str) -> bool {
        self.rests_on_at(id, on, 0)
    }

    fn rests_on_at(&self, id: &str, on: &str, depth: u32) -> bool {
        if id == on {
            return true;
        }
        if depth > 16 {
            return false;
        }
        self.row("constants", id)
            .and_then(|r| r.get("derived_from"))
            .and_then(J::as_str)
            .is_some_and(|f| {
                expr::names(f)
                    .iter()
                    .any(|n| self.rests_on_at(n, on, depth + 1))
            })
    }

    /// Whether a derived constant and every input it names come from one
    /// pack, so its typed copy stands for it.
    /// The check reaches through derived inputs (DM1: Omega_Lambda rests on
    /// Omega_m, which rests on an edited Omega_c).
    fn one_pack(&self, id: &str, f: &str) -> bool {
        self.one_pack_at(id, f, 0)
    }

    fn one_pack_at(&self, id: &str, f: &str, depth: u32) -> bool {
        if depth > 16 {
            return false;
        }
        std::iter::once(id.to_string())
            .chain(expr::names(f))
            .all(|n| {
                let unedited = self
                    .origin
                    .get(&format!("constants/{n}"))
                    .is_none_or(|o| o.len() == 1);
                let inner = match self
                    .row("constants", &n)
                    .and_then(|r| r.get("derived_from"))
                    .and_then(J::as_str)
                {
                    Some(g) if n != id => self.one_pack_at(&n, g, depth + 1),
                    _ => true,
                };
                unedited && inner
            })
    }

    /// The constants handle blocks read (`sse_core::constants`), from the
    /// sheet: each id the sheet holds replaces the compiled value. A derived
    /// constant uses its typed copy while nothing it rests on was edited
    /// (so an unedited sheet gives the compiled values bit for bit), and its
    /// worked-out value once something was.
    pub fn handle(&self) -> sse_core::constants::Constants {
        let mut c = sse_core::constants::Constants::REAL;
        for id in sse_core::constants::Constants::IDS {
            let Some(row) = self.row("constants", id) else {
                continue;
            };
            let v = match row.get("derived_from").and_then(J::as_str) {
                Some(f) if !self.one_pack(id, f) => self.constant(id),
                _ => num(row.get("value")).or_else(|| self.constant(id)),
            };
            if let Some(v) = v {
                c.set(id, v);
            }
        }
        c
    }

    /// A unit's factor to SI: worked out from `factor_from` (a formula
    /// over constants) when it has one, else its typed `factor`.
    pub fn unit_factor(&self, id: &str) -> Result<f64, String> {
        let row = self
            .row("units", id)
            .ok_or_else(|| format!("no unit {id}"))?;
        match row.get("factor_from").and_then(J::as_str) {
            Some(f) => {
                let mut miss = None;
                let v = expr::eval(f, |n| match self.constant_at(n, 1) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        miss.get_or_insert(e);
                        None
                    }
                });
                v.map_err(|e| miss.unwrap_or(e))
            }
            None => num(row.get("factor")).ok_or_else(|| format!("unit {id} has no factor")),
        }
    }

    /// The units the sheet adds to the session, and the built-in units it
    /// moves because a constant behind them was edited (design 7A.6: change
    /// c and the light year follows). A built-in unit the sheet agrees with
    /// is left as compiled, bit for bit.
    pub fn units(&self) -> Vec<sse_core::quantity::Unit> {
        let standard = sse_core::Units::standard();
        let empty = BTreeMap::new();
        let mut out = vec![];
        for (id, row) in self.tables.get("units").unwrap_or(&empty) {
            let (Ok(factor), Some(dim)) = (self.unit_factor(id), unit_dim(row)) else {
                continue;
            };
            let offset = num(row.get("offset")).unwrap_or(0.0);
            if let Ok(b) = standard.get(id) {
                if ((b.scale - factor) / b.scale).abs() <= UNIT_TOL && b.offset == offset {
                    continue;
                }
            }
            out.push(sse_core::quantity::Unit {
                id: id.clone(),
                symbol: row
                    .get("symbol")
                    .and_then(J::as_str)
                    .unwrap_or(id)
                    .to_string(),
                dim,
                scale: factor,
                offset,
            });
        }
        out
    }

    /// The load checks of design section 4.6.
    fn check(&self) -> Vec<String> {
        let mut p = vec![];
        let empty_c = BTreeMap::new();
        // Element origins (design 8.1): shares name legend entries and sum
        // to 1; every source named resolves.
        for (id, row) in self.tables.get("elements").unwrap_or(&empty_c) {
            p.extend(self.check_origin(id, row));
        }
        // Units (design 7A.6): a known dimension, a factor that resolves,
        // a typed copy that agrees with its formula, and no redefinition of
        // a unit the kernel already has while nothing behind it was edited.
        let standard = sse_core::Units::standard();
        for (id, row) in self.tables.get("units").unwrap_or(&empty_c) {
            let Some(dim) = unit_dim(row) else {
                p.push(format!(
                    "unit {id}: measures {:?} is not a dimension the reader knows (or give dim as seven exponents)",
                    row.get("measures").and_then(J::as_str).unwrap_or("")
                ));
                continue;
            };
            let factor = match self.unit_factor(id) {
                Ok(f) => f,
                Err(e) => {
                    p.push(format!("unit {id}: {e}"));
                    continue;
                }
            };
            let f = row.get("factor_from").and_then(J::as_str).unwrap_or("");
            let one_pack = self.one_pack(&format!("@unit:{id}"), f)
                && self
                    .origin
                    .get(&format!("units/{id}"))
                    .is_none_or(|o| o.len() == 1);
            if let (Some(typed), false) = (num(row.get("factor")), f.is_empty()) {
                if one_pack && ((typed - factor) / factor).abs() > DERIVED_TOL {
                    p.push(format!(
                        "unit {id}: factor typed as {typed:e} but {f} gives {factor:e}"
                    ));
                }
            }
            if let Ok(b) = standard.get(id) {
                if b.dim != dim {
                    p.push(format!(
                        "unit {id} measures something else than the built-in {id}"
                    ));
                } else if one_pack && ((b.scale - factor) / b.scale).abs() > UNIT_TOL {
                    p.push(format!(
                        "unit {id} redefines the built-in unit: {factor:e} against {:e} (add a unit with a new id instead)",
                        b.scale
                    ));
                }
            }
        }
        for (id, row) in self.tables.get("constants").unwrap_or(&empty_c) {
            // `checked_by`: a relation the typed value must meet on the sheet
            // as shipped, never used to work it out (M_sun against GM_sun / G:
            // doubling G keeps the Sun's mass).
            if let Some(f) = row.get("checked_by").and_then(J::as_str) {
                let typed = num(row.get("value"));
                let mut miss = None;
                let worked = expr::eval(f, |n| match self.constant_at(n, 1) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        miss.get_or_insert(e);
                        None
                    }
                });
                match (typed, worked) {
                    (_, Err(e)) => p.push(format!(
                        "constant {id}: checked_by \"{f}\" {}",
                        miss.unwrap_or(e)
                    )),
                    (Some(t), Ok(w))
                        if self.one_pack(id, f) && ((t - w) / w).abs() > DERIVED_TOL =>
                    {
                        p.push(format!("constant {id}: typed as {t:e} but {f} gives {w:e}"))
                    }
                    _ => {}
                }
            }
            let Some(f) = row.get("derived_from").and_then(J::as_str) else {
                if num(row.get("value")).is_none() {
                    p.push(format!("constant {id} has no value"));
                }
                continue;
            };
            let worked = match self.constant_at(id, 0) {
                Ok(v) => v,
                Err(e) => {
                    p.push(format!("constant {id}: derived_from \"{f}\" {e}"));
                    continue;
                }
            };
            // The typed copy must agree while it and every input come from
            // one pack. Once a later pack edits an input, the worked-out
            // value is the constant and the typed copy is only a note.
            let one_pack = self.one_pack(id, f);
            if let Some(typed) = num(row.get("value")) {
                if one_pack && ((typed - worked) / worked).abs() > DERIVED_TOL {
                    p.push(format!(
                        "constant {id}: typed as {typed:e} but {f} gives {worked:e}"
                    ));
                }
            }
        }
        // DM1: a constant's group is one the page knows, a mark is the one
        // row mark there is ("theoretical", with the line saying what would
        // change the picture), and a group's notes cite known sources.
        for (id, row) in self.tables.get("constants").unwrap_or(&empty_c) {
            if let Some(g) = row.get("group").and_then(J::as_str) {
                if !self.groups.is_empty() && !self.groups.contains_key(g) {
                    p.push(format!(
                        "constant {id}: group {g} is not one of the sheet's groups"
                    ));
                }
            }
            match row.get("mark").and_then(J::as_str) {
                None => {}
                Some("theoretical") => {
                    if row.get("mark_note").and_then(J::as_str).is_none_or(str::is_empty) {
                        p.push(format!(
                            "constant {id}: a theoretical row needs a mark_note saying what would change the picture"
                        ));
                    }
                }
                Some(m) => p.push(format!(
                    "constant {id}: mark {m} is not a row mark (only \"theoretical\"; authored and fictional come from the packs)"
                )),
            }
        }
        for (g, row) in &self.groups {
            for (i, n) in row
                .get("notes")
                .and_then(J::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                let cited = n
                    .get("src")
                    .into_iter()
                    .chain(n.get("also").and_then(J::as_array).into_iter().flatten());
                for s in cited.filter_map(J::as_str) {
                    if !self.sources.contains_key(s) {
                        p.push(format!(
                            "group {g}: note {i} cites source {s}, which is not in reference.sources"
                        ));
                    }
                }
            }
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
                p.extend(check_vapour(id, ph, self.molar_mass_kg_mol(id)));
                let known: std::collections::BTreeSet<String> =
                    self.sources.keys().cloned().collect();
                let linked: std::collections::BTreeSet<String> = self
                    .sources
                    .iter()
                    .filter(|(_, r)| r.contains_key("doi") || r.contains_key("url"))
                    .map(|(k, _)| k.clone())
                    .collect();
                p.extend(phases::check(id, ph, &known, &linked));
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
        if table == "constants" {
            if let Some(f) = row.get("derived_from").and_then(J::as_str) {
                if let Some(v) = self.constant(id) {
                    out["worked"] = serde_json::json!({"value": {"v": v, "from": f}});
                }
            }
        }
        if table == "units" {
            if let Ok(f) = self.unit_factor(id) {
                out["worked"] = serde_json::json!({"factor": {"v": f, "from": row.get("factor_from").cloned().unwrap_or(J::Null)}});
            }
        }
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
        // The page groups are few and short, so they come whole (DM1).
        if table.is_none_or(|x| x == "groups") {
            out.insert("groups".into(), serde_json::json!(self.groups));
        }
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
        // Water ice, Feistel and Wagner 2007 as Fray and Schmitt 2009 give
        // it: ln(P/Pt) = 1.5 ln(T/Tt) + (1 − Tt/T) Σ e_i (T/Tt)^i, with
        // coeffs [Tt K, Pt Pa, e0..e6].
        "feistel-wagner-2007" => {
            let (tt, pt) = (*coeffs.first()?, *coeffs.get(1)?);
            let th = t / tt;
            let mut h = 0.0;
            let mut x = 1.0;
            for e in &coeffs[2..] {
                h += e * x;
                x *= th;
            }
            Some(pt * m::exp(1.5 * m::ln(th) + (1.0 - 1.0 / th) * h))
        }
        // [p_ref Pa, t_ref K, latent heat J/mol].
        "clausius-clapeyron" => {
            let (p0, t0, l) = (*coeffs.first()?, *coeffs.get(1)?, *coeffs.get(2)?);
            Some(p0 * m::exp(-l / R_GAS * (1.0 / t - 1.0 / t0)))
        }
        _ => None,
    }
}

fn valid_lo(fit: &J) -> Option<f64> {
    fit.get("valid_k")
        .and_then(J::as_array)
        .and_then(|a| a.first())
        .and_then(J::as_f64)
}

/// Sublimation pressure in Pa of a substance's solid at `t` K from its
/// `phases` group: `vapour_pressure` (the fit that reaches the triple point)
/// or, below that fit's range, `vapour_pressure_low` where the row has one
/// (alpha nitrogen under beta, for instance). Below the lowest fit's range
/// the curve goes on as Clausius-Clapeyron with the slope at the range's
/// edge: the polynomials are not made to be read outside their data and
/// run away there (CO2's A5/T⁵ overflows near 1 K).
pub fn sublimation_pa(phases: &Map<String, J>, t: f64) -> Option<f64> {
    let main = phases.get("vapour_pressure")?;
    let lo = valid_lo(main);
    let (fit, edge) = match (phases.get("vapour_pressure_low"), lo) {
        (Some(low), Some(lo)) if t < lo => (low, valid_lo(low)),
        _ => (main, lo),
    };
    match edge {
        Some(e) if t < e && t > 0.0 => {
            let h = 1e-3 * e;
            let pe = vapour_pressure_pa(fit, e)?;
            let a = vapour_pressure_pa(fit, e - h)?;
            let b = vapour_pressure_pa(fit, e + h)?;
            let slope = (m::ln(b) - m::ln(a)) / (1.0 / (e + h) - 1.0 / (e - h));
            Some(pe * m::exp(slope * (1.0 / t - 1.0 / e)))
        }
        _ => vapour_pressure_pa(fit, t),
    }
}

/// Latent heat of sublimation in J/mol from a substance's own vapour curve
/// at `t` K (Clausius-Clapeyron: L = −R d ln P / d(1/T)).
pub fn sublimation_latent_j_mol(phases: &Map<String, J>, t: f64) -> Option<f64> {
    let h = 1e-3 * t;
    let a = sublimation_pa(phases, t - h)?;
    let b = sublimation_pa(phases, t + h)?;
    Some(-R_GAS * (m::ln(b) - m::ln(a)) / (1.0 / (t + h) - 1.0 / (t - h)))
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

fn check_vapour(id: &str, ph: &Map<String, J>, molar_kg: Option<f64>) -> Vec<String> {
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
    // A low-temperature piece meets the main fit where the main one starts.
    if let Some(low) = ph.get("vapour_pressure_low") {
        let lo = fit
            .get("valid_k")
            .and_then(J::as_array)
            .and_then(|a| a.first())
            .and_then(J::as_f64);
        match (lo, lo.and_then(|t| vapour_pressure_pa(low, t))) {
            (Some(t), Some(a)) => {
                let b = vapour_pressure_pa(fit, t).unwrap_or(f64::NAN);
                let gap = m::ln(a / b).abs();
                if gap.is_nan() || gap > TRIPLE_TOL {
                    p.push(format!(
                        "substance {id}: its two vapour curves disagree where they meet ({t} K): {a:.4e} Pa against {b:.4e} Pa"
                    ));
                }
            }
            _ => p.push(format!(
                "substance {id}: vapour_pressure_low needs vapour_pressure to have a valid_k range and a fit the reader knows"
            )),
        }
    }
    // A typed sublimation heat (J/kg) agrees with the curve's slope where it
    // was measured.
    if let (Some(l), Some(mk)) = (num(ph.get("latent_sublimation")), molar_kg) {
        let at = ph
            .get("latent_sublimation")
            .and_then(|v| v.get("at_k"))
            .and_then(J::as_f64)
            .unwrap_or(tt);
        if let Some(from_curve) = sublimation_latent_j_mol(ph, at) {
            let typed = l * mk;
            if ((from_curve - typed) / typed).abs() > LATENT_TOL {
                p.push(format!(
                    "substance {id}: sublimation heat {typed:.0} J/mol at {at} K disagrees with the vapour curve's slope, which gives {from_curve:.0} J/mol"
                ));
            }
        }
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
mod lookup_tests;
#[cfg(test)]
mod phases_tests;
#[cfg(test)]
mod tests;
