//! MP17a: the sheet as blocks read it (matter pack design section 3A).
//!
//! [`Reference`] answers the Look up block through `sse_flow::sheet::Sheet`:
//! which row a block names, what dimension a property carries, and the
//! property's value, phase, source and grade at given conditions.
//!
//! How a property is found on a row, first match wins:
//! 1. at given conditions on a substance, the phase there (MP17) and that
//!    phase's own `props` (so "density of water at 1 GPa and 270 K" answers
//!    for ice VI);
//! 2. for a familiar phase (solid, liquid, vapour), `<property>_<phase>` on
//!    the row (`phases.density_liquid`);
//! 3. the property by its path (`phases.triple_point_k`) or by its last name
//!    when only one value on the row has it;
//! 4. worked-out values: a substance's `molar_mass` from its formula.
//!
//! Point values are read as written. Fits are read where this reader knows
//! the form (the vapour-pressure fits and `k = a / T`) and need a
//! temperature; any other fit gives `known` false with the reason.

use crate::{num, sublimation_pa, vapour_pressure_pa, Reference, TABLES};
use serde_json::{Map, Value as J};
use sse_core::Dimension;
use sse_flow::sheet::{LookedUp, Sheet};

/// The SI dimension of a unit as the sheet writes it ("kg m-3", "J mol-1",
/// "m3 kg-1 s-2", "1").
pub fn unit_dimension(unit: &str) -> Result<Dimension, String> {
    let mut d = Dimension::NONE;
    for tok in unit.split_whitespace() {
        let split = tok
            .find(|c: char| c == '-' || c.is_ascii_digit())
            .unwrap_or(tok.len());
        let (sym, exp) = tok.split_at(split);
        let n: i8 = if exp.is_empty() {
            1
        } else {
            exp.parse()
                .map_err(|_| format!("unit {unit:?}: cannot read the power in {tok:?}"))?
        };
        if sym.is_empty() && exp == "1" {
            continue;
        }
        let base = match sym {
            "m" => Dimension::LENGTH,
            "kg" => Dimension::MASS,
            "s" => Dimension::TIME,
            "A" => Dimension([0, 0, 0, 1, 0, 0, 0]),
            "K" => Dimension::TEMPERATURE,
            "mol" => Dimension([0, 0, 0, 0, 0, 1, 0]),
            "cd" => Dimension([0, 0, 0, 0, 0, 0, 1]),
            "J" => Dimension([2, 1, -2, 0, 0, 0, 0]),
            "W" => Dimension::POWER,
            "N" => Dimension([1, 1, -2, 0, 0, 0, 0]),
            "Pa" => Dimension::PRESSURE,
            "Hz" => Dimension([0, 0, -1, 0, 0, 0, 0]),
            "C" => Dimension([0, 0, 1, 1, 0, 0, 0]),
            "V" => Dimension([2, 1, -3, -1, 0, 0, 0]),
            "S" => Dimension([-2, -1, 3, 2, 0, 0, 0]),
            "T" => Dimension::MAGNETIC_FIELD,
            "H" => Dimension([2, 1, -2, -2, 0, 0, 0]),
            "F" => Dimension([-2, -1, 4, 2, 0, 0, 0]),
            "Wb" => Dimension([2, 1, -2, -1, 0, 0, 0]),
            "ohm" | "Ω" => Dimension([2, 1, -3, -2, 0, 0, 0]),
            _ => {
                return Err(format!(
                    "unit {unit:?}: {sym:?} is not an SI unit this reader knows"
                ))
            }
        };
        d = d.mul(base.powi(n));
    }
    Ok(d)
}

/// Every value on a row with its path, leaving out the phase map's own
/// structure (forms, boundaries, meets, table), which are read as phases.
fn row_values(row: &Map<String, J>) -> Vec<(String, &J)> {
    fn walk<'a>(o: &'a Map<String, J>, prefix: &str, out: &mut Vec<(String, &'a J)>) {
        for (k, v) in o {
            if prefix == "phases" && ["forms", "boundaries", "meets", "table"].contains(&k.as_str())
            {
                continue;
            }
            let path = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            if crate::is_value(v) {
                out.push((path, v));
            } else if let Some(o) = v.as_object() {
                walk(o, &path, out);
            }
        }
    }
    let mut out = vec![];
    walk(row, "", &mut out);
    out
}

/// The values on a row a property name matches: by path, else by last name.
fn matches<'a>(row: &'a Map<String, J>, property: &str) -> Vec<(String, &'a J)> {
    let vs = row_values(row);
    if let Some(hit) = vs.iter().find(|(p, _)| p == property) {
        return vec![hit.clone()];
    }
    vs.into_iter()
        .filter(|(p, _)| p.rsplit('.').next() == Some(property))
        .collect()
}

fn forms(row: &Map<String, J>) -> impl Iterator<Item = &J> {
    row.get("phases")
        .and_then(|p| p.get("forms"))
        .and_then(J::as_array)
        .into_iter()
        .flatten()
}

fn unit_of(v: &J) -> Option<&str> {
    v.get("unit").and_then(J::as_str)
}

fn range(v: Option<&J>) -> Option<(f64, f64)> {
    let a = v?.as_array()?;
    Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?))
}

impl Reference {
    fn source_row(&self, src: Option<&str>) -> Option<J> {
        let s = src?;
        let mut row = self.sources.get(s)?.clone();
        row.insert("id".into(), J::String(s.into()));
        Some(J::Object(row))
    }

    /// The table and id a block's `what` names.
    pub fn resolve_row(&self, what: &str) -> Result<(String, String), String> {
        if let Some((t, id)) = what.split_once('/') {
            if !TABLES.contains(&t) {
                return Err(format!(
                    "{what}: {t:?} is not a table of the sheet ({})",
                    TABLES.join(", ")
                ));
            }
            return self
                .row(t, id)
                .map(|_| (t.to_string(), id.to_string()))
                .ok_or_else(|| format!("the sheet has no {t} row {id:?}"));
        }
        let found: Vec<&str> = TABLES
            .iter()
            .copied()
            .filter(|t| self.row(t, what).is_some())
            .collect();
        match found.as_slice() {
            [t] => Ok((t.to_string(), what.to_string())),
            [] => Err(format!("the sheet has no row {what:?}")),
            many => Err(format!(
                "{what:?} is in more than one table ({}): write it as {}/{what}",
                many.join(", "),
                many[0]
            )),
        }
    }

    /// The unit a property is written in on one row, wherever it is found.
    fn unit_on(&self, table: &str, id: &str, property: &str) -> Option<String> {
        let row = self.row(table, id)?;
        if table == "constants" && (property.is_empty() || property == "value") {
            return unit_of(row.get("value")?).map(str::to_string);
        }
        if table == "substances" && property == "molar_mass" {
            return Some("kg mol-1".into());
        }
        if let Some(u) = forms(row).find_map(|f| unit_of(f.get("props")?.get(property)?)) {
            return Some(u.to_string());
        }
        for kind in ["solid", "liquid", "vapour"] {
            if let [(_, v)] = matches(row, &format!("{property}_{kind}")).as_slice() {
                if let Some(u) = unit_of(v) {
                    return Some(u.to_string());
                }
            }
        }
        match matches(row, property).as_slice() {
            [(_, v)] => unit_of(v)
                .map(str::to_string)
                .or_else(|| fit_unit(v).map(str::to_string)),
            _ => None,
        }
    }

    fn look(&self, what: &str, property: &str, at: Option<(f64, f64)>) -> Result<LookedUp, String> {
        let (table, id) = self.resolve_row(what)?;
        let row = self.row(&table, &id).expect("resolved");
        let mut out = LookedUp {
            row: format!("{table}/{id}"),
            property: property.to_string(),
            in_range: true,
            packs: self
                .origin
                .get(&format!("{table}/{id}"))
                .cloned()
                .unwrap_or_default(),
            ..Default::default()
        };
        if let Some((p, t)) = at {
            if !(p > 0.0 && t > 0.0 && p.is_finite() && t.is_finite()) {
                return Err("pressure and temperature must be above zero".into());
            }
        }
        if table == "constants" && (property.is_empty() || property == "value") {
            let v = row.get("value");
            out.value = self
                .constant(&id)
                .ok_or_else(|| format!("constant {id} has no value"))?;
            out.property = "value".into();
            out.unit = v.and_then(unit_of).unwrap_or("").to_string();
            out.known = true;
            out.source = self.source_row(v.and_then(|v| v.get("src")).and_then(J::as_str));
            out.grade = Some(
                if row.get("derived_from").is_some() {
                    "computed"
                } else {
                    "measured"
                }
                .into(),
            );
            if v.and_then(|v| v.get("exact")).and_then(J::as_bool) == Some(true) {
                out.grade = Some("convention".into());
                out.reason = Some("exact by definition".into());
            }
            return Ok(out);
        }
        // 1. The phase at the conditions, and its own properties.
        let mut familiar = None;
        if let (Some((p, t)), "substances") = (at, table.as_str()) {
            if let Ok(pa) = self.phase_at(&id, p, t) {
                out.phase = Some(pa.phase.clone());
                out.grade = pa.grade.clone();
                if let Some(form) = &pa.form {
                    if let Some(v) = form.get("props").and_then(|x| x.get(property)) {
                        if let Some(x) = num(Some(v)) {
                            let ev = form.get("evidence");
                            let inside = |r: Option<(f64, f64)>, x: f64| {
                                r.is_none_or(|(lo, hi)| lo <= x && x <= hi)
                            };
                            out.in_range = inside(range(ev.and_then(|e| e.get("t_k"))), t)
                                && inside(range(ev.and_then(|e| e.get("p_pa"))), p);
                            out.value = x;
                            out.unit = unit_of(v).unwrap_or("").to_string();
                            out.known = true;
                            out.property = format!("{}.{property}", pa.phase);
                            out.source = self.source_row(v.get("src").and_then(J::as_str));
                            out.reason = point_note(v);
                            return Ok(out);
                        }
                    }
                }
                if pa.from == "familiar" {
                    familiar = pa.phase.rsplit('.').next().map(str::to_string);
                }
            }
        }
        // 2. The familiar phase's own copy, `<property>_<phase>`.
        if let Some(kind) = &familiar {
            if let [(path, v)] = matches(row, &format!("{property}_{kind}")).as_slice() {
                return self.read_value(out, path, v, at);
            }
        }
        // 4. Worked out.
        if table == "substances" && property == "molar_mass" {
            if let Some(mm) = self.molar_mass_kg_mol(&id) {
                out.value = mm;
                out.unit = "kg mol-1".into();
                out.known = true;
                out.grade = Some("computed".into());
                out.reason =
                    Some("worked out from the formula and the elements' atomic weights".into());
                return Ok(out);
            }
        }
        // 3. By path or last name.
        match matches(row, property).as_slice() {
            [(path, v)] => self.read_value(out, path, v, at),
            [] => {
                out.reason = Some(format!("{table}/{id} has no {property}"));
                Ok(out)
            }
            many => Err(format!(
                "{table}/{id} has {property} in more than one place: name one of {}",
                many.iter()
                    .map(|(p, _)| p.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    fn read_value(
        &self,
        mut out: LookedUp,
        path: &str,
        v: &J,
        at: Option<(f64, f64)>,
    ) -> Result<LookedUp, String> {
        out.property = path.to_string();
        out.source = self.source_row(v.get("src").and_then(J::as_str));
        if let Some(x) = v.get("v").and_then(J::as_f64) {
            out.value = x;
            out.unit = unit_of(v).unwrap_or("").to_string();
            out.known = true;
            out.reason = point_note(v);
            if let (Some((lo, hi)), Some((_, t))) = (range(v.get("valid_k")), at) {
                out.in_range = lo <= t && t <= hi;
            }
            return Ok(out);
        }
        let Some(fit) = v.get("fit").and_then(J::as_str) else {
            out.reason = Some(format!("{path} has no value"));
            return Ok(out);
        };
        let Some((_, t)) = at else {
            out.reason = Some(format!("{path} is a fit ({fit}): it needs a temperature, so turn on the block's conditions"));
            return Ok(out);
        };
        let x = match fit {
            "k = a / T" => v
                .get("coeffs")
                .and_then(J::as_array)
                .and_then(|c| c.first())
                .and_then(J::as_f64)
                .map(|a| a / t),
            _ if path.ends_with("vapour_pressure") => {
                // The phases group, so the low-temperature branch and the
                // extension below the fit's range are read as `phase_at` reads them.
                let (t_, id) = out.row.split_once('/').expect("table/id");
                self.row(t_, id)
                    .and_then(|r| r.get("phases"))
                    .and_then(J::as_object)
                    .and_then(|ph| sublimation_pa(ph, t))
                    .or_else(|| vapour_pressure_pa(v, t))
            }
            _ => vapour_pressure_pa(v, t),
        };
        match x {
            Some(x) if x.is_finite() => {
                out.value = x;
                out.unit = unit_of(v).or_else(|| fit_unit(v)).unwrap_or("").to_string();
                out.known = true;
                out.in_range = range(v.get("valid_k")).is_none_or(|(lo, hi)| lo <= t && t <= hi);
                out.reason = Some(format!("read from the fit {fit} at {t} K"));
            }
            _ => {
                out.reason = Some(format!(
                    "{path} is a fit ({fit}) this reader cannot evaluate"
                ))
            }
        }
        Ok(out)
    }
}

/// A vapour-pressure fit gives pascals even where the row writes no unit.
fn fit_unit(v: &J) -> Option<&'static str> {
    match v.get("fit").and_then(J::as_str)? {
        "fray-schmitt-2009" | "feistel-wagner-2007" | "clausius-clapeyron" => Some("Pa"),
        _ => None,
    }
}

fn point_note(v: &J) -> Option<String> {
    match (
        v.get("at_k").and_then(J::as_f64),
        v.get("at_pa").and_then(J::as_f64),
    ) {
        (Some(t), Some(p)) => Some(format!("measured at {t} K and {p} Pa")),
        (Some(t), None) => Some(format!("measured at {t} K")),
        _ => None,
    }
}

impl Sheet for Reference {
    fn row_of(&self, what: &str) -> Result<String, String> {
        self.resolve_row(what).map(|(t, id)| format!("{t}/{id}"))
    }

    fn dimension(&self, what: Option<&str>, property: &str) -> Result<Dimension, String> {
        let unit = match what {
            Some(w) => {
                let (t, id) = self.resolve_row(w)?;
                // A row without the property still loads (it answers
                // known = false); the unit comes from a row that has it.
                self.unit_on(&t, &id, property)
                    .or_else(|| self.any_unit(property))
            }
            None => self.any_unit(property),
        };
        let unit = unit.ok_or_else(|| match what {
            Some(w) => {
                format!("neither {w} nor any other row has {property}, so its unit is not known")
            }
            None => format!("no row on the sheet has {property}"),
        })?;
        unit_dimension(&unit)
    }

    fn look_up(
        &self,
        what: &str,
        property: &str,
        at: Option<(f64, f64)>,
    ) -> Result<LookedUp, String> {
        self.look(what, property, at)
    }
}

impl Reference {
    fn any_unit(&self, property: &str) -> Option<String> {
        ["substances", "mixtures", "elements", "isotopes"]
            .iter()
            .flat_map(|t| {
                self.tables
                    .get(*t)
                    .into_iter()
                    .flat_map(move |rows| rows.keys().map(move |id| (*t, id)))
            })
            .find_map(|(t, id)| self.unit_on(t, id, property))
    }
}
