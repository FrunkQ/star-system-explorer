//! Rows as flat columns, the shape the populations read (RW4) and the GPU
//! (RW6) want. A row is never an object until someone opens it.
//!
//! Absent values are kept absent, never filled: floats are NaN, integers are
//! their type's sentinel ([`NULL_U8`] and friends), strings are `None`.

use crate::header::{flags, ColType, Epoch, Frame, Grade, Header, Role};
use crate::{Refusal, Warning};
use serde_json::{json, Map, Value as Json};
use std::collections::BTreeSet;

pub const NULL_U8: u8 = u8::MAX;
pub const NULL_U16: u16 = u16::MAX;
pub const NULL_U32: u32 = u32::MAX;
pub const NULL_I32: i32 = i32::MIN;

/// One column's values. Equality compares floats by their bits, so an
/// absent (NaN) value equals an absent value.
#[derive(Clone, Debug)]
pub enum Values {
    F64(Vec<f64>),
    F32(Vec<f32>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    U16(Vec<u16>),
    U8(Vec<u8>),
    Str(Vec<Option<String>>),
}

impl PartialEq for Values {
    fn eq(&self, o: &Values) -> bool {
        match (self, o) {
            (Values::F64(a), Values::F64(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
            }
            (Values::F32(a), Values::F32(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
            }
            (Values::I32(a), Values::I32(b)) => a == b,
            (Values::U32(a), Values::U32(b)) => a == b,
            (Values::U16(a), Values::U16(b)) => a == b,
            (Values::U8(a), Values::U8(b)) => a == b,
            (Values::Str(a), Values::Str(b)) => a == b,
            _ => false,
        }
    }
}

impl Values {
    pub fn empty(ty: ColType) -> Values {
        match ty {
            ColType::F64 => Values::F64(Vec::new()),
            ColType::F32 => Values::F32(Vec::new()),
            ColType::I32 => Values::I32(Vec::new()),
            ColType::U32 => Values::U32(Vec::new()),
            ColType::U16 => Values::U16(Vec::new()),
            ColType::U8 => Values::U8(Vec::new()),
            ColType::Str => Values::Str(Vec::new()),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Values::F64(v) => v.len(),
            Values::F32(v) => v.len(),
            Values::I32(v) => v.len(),
            Values::U32(v) => v.len(),
            Values::U16(v) => v.len(),
            Values::U8(v) => v.len(),
            Values::Str(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn truncate(&mut self, n: usize) {
        match self {
            Values::F64(v) => v.truncate(n),
            Values::F32(v) => v.truncate(n),
            Values::I32(v) => v.truncate(n),
            Values::U32(v) => v.truncate(n),
            Values::U16(v) => v.truncate(n),
            Values::U8(v) => v.truncate(n),
            Values::Str(v) => v.truncate(n),
        }
    }
    fn extend(&mut self, other: Values) {
        match (self, other) {
            (Values::F64(a), Values::F64(b)) => a.extend(b),
            (Values::F32(a), Values::F32(b)) => a.extend(b),
            (Values::I32(a), Values::I32(b)) => a.extend(b),
            (Values::U32(a), Values::U32(b)) => a.extend(b),
            (Values::U16(a), Values::U16(b)) => a.extend(b),
            (Values::U8(a), Values::U8(b)) => a.extend(b),
            (Values::Str(a), Values::Str(b)) => a.extend(b),
            _ => unreachable!("columns of one header have one type"),
        }
    }
    /// Row `r` as a number, None when absent or a string.
    pub fn number(&self, r: usize) -> Option<f64> {
        let x = match self {
            Values::F64(v) => v[r],
            Values::F32(v) => v[r] as f64,
            Values::I32(v) => return (v[r] != NULL_I32).then_some(v[r] as f64),
            Values::U32(v) => return (v[r] != NULL_U32).then_some(v[r] as f64),
            Values::U16(v) => return (v[r] != NULL_U16).then_some(v[r] as f64),
            Values::U8(v) => return (v[r] != NULL_U8).then_some(v[r] as f64),
            Values::Str(_) => return None,
        };
        (!x.is_nan()).then_some(x)
    }
    /// Row `r` as a string, None when absent or a number.
    pub fn text(&self, r: usize) -> Option<&str> {
        match self {
            Values::Str(v) => v[r].as_deref(),
            _ => None,
        }
    }
    /// Row `r` as a u8 code, None when absent or not a u8 column.
    pub fn code(&self, r: usize) -> Option<u8> {
        match self {
            Values::U8(v) => (v[r] != NULL_U8).then_some(v[r]),
            _ => None,
        }
    }
    /// Row `r` as a u16 index, None when absent or not a u16 column.
    pub fn index16(&self, r: usize) -> Option<u16> {
        match self {
            Values::U16(v) => (v[r] != NULL_U16).then_some(v[r]),
            _ => None,
        }
    }
    pub fn is_present(&self, r: usize) -> bool {
        match self {
            Values::Str(v) => v[r].is_some(),
            _ => self.number(r).is_some(),
        }
    }
}

/// The rows read, as columns in the header's order, biggest first.
#[derive(Clone, PartialEq, Debug)]
pub struct Table {
    pub rows: usize,
    pub columns: Vec<Values>,
}

/// Where one value came from: its grade and its source's index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Provenance {
    pub grade: Grade,
    pub source: usize,
}

impl Table {
    pub fn empty(h: &Header) -> Table {
        Table {
            rows: 0,
            columns: h.columns.iter().map(|c| Values::empty(c.ty)).collect(),
        }
    }

    pub fn truncate(&mut self, n: usize) {
        if n < self.rows {
            self.rows = n;
            for c in &mut self.columns {
                c.truncate(n);
            }
        }
    }

    pub(crate) fn append(&mut self, other: Table) {
        self.rows += other.rows;
        for (a, b) in self.columns.iter_mut().zip(other.columns) {
            a.extend(b);
        }
    }

    /// The size of row `r` (the size key), None when absent.
    pub fn size(&self, h: &Header, r: usize) -> Option<f64> {
        let (i, _) = h.column(&h.size.column)?;
        self.columns[i].number(r)
    }

    pub fn id(&self, h: &Header, r: usize) -> Option<&str> {
        let (i, _) = h.column("id")?;
        self.columns[i].text(r)
    }

    /// The grade and source of value `col` in row `r` (every imported value
    /// records which catalogue it came from). None for an absent value, an
    /// id or a meta column.
    pub fn provenance(&self, h: &Header, r: usize, col: usize) -> Option<Provenance> {
        let c = h.columns.get(col)?;
        if c.role != Role::Value || !self.columns[col].is_present(r) {
            return None;
        }
        let grade = match (&c.grade, &c.grade_column) {
            (Some(g), _) => *g,
            (None, Some(gc)) => Grade::from_code(self.columns[h.column(gc)?.0].code(r)?)?,
            _ => return None,
        };
        let source = match (&c.source, &c.source_column) {
            (Some(s), _) => h.source(s)?.0,
            (None, Some(sc)) => {
                let i = self.columns[h.column(sc)?.0].code(r)? as usize;
                (i < h.sources.len()).then_some(i)?
            }
            _ => return None,
        };
        Some(Provenance { grade, source })
    }

    /// Row `r` as JSON: present values only, each value column with its
    /// grade and source id beside it under `_from`.
    pub fn row_json(&self, h: &Header, r: usize) -> Json {
        let mut o = Map::new();
        let mut from = Map::new();
        for (i, c) in h.columns.iter().enumerate() {
            let v = &self.columns[i];
            if !v.is_present(r) {
                continue;
            }
            let j = match v {
                Values::Str(_) => json!(v.text(r)),
                Values::U8(_) | Values::U16(_) | Values::U32(_) | Values::I32(_) => {
                    json!(v.number(r).map(|x| x as i64))
                }
                _ => json!(v.number(r)),
            };
            o.insert(c.name.clone(), j);
            if let Some(p) = self.provenance(h, r, i) {
                from.insert(
                    c.name.clone(),
                    json!({"grade": p.grade, "source": h.sources[p.source].id}),
                );
            }
        }
        if !from.is_empty() {
            o.insert("_from".into(), Json::Object(from));
        }
        Json::Object(o)
    }

    /// Checks rows `from..` against the header and the row before `from`:
    /// sizes present and never rising, ids present and unique within what
    /// was read, codes and indices in range. Format faults refuse; rows that
    /// cannot be placed become warnings and are kept.
    pub(crate) fn check_rows(
        &self,
        h: &Header,
        from: usize,
        seen: &mut BTreeSet<String>,
        warnings: &mut Vec<Warning>,
    ) -> Result<(), Refusal> {
        let (si, _) = h.column(&h.size.column).expect("checked header");
        let (ii, _) = h.column("id").expect("checked header");
        let size = &self.columns[si];
        let ids = &self.columns[ii];
        let grade_cols: BTreeSet<usize> = h
            .columns
            .iter()
            .filter_map(|c| c.grade_column.as_deref())
            .filter_map(|g| h.column(g).map(|(i, _)| i))
            .collect();
        let source_cols: BTreeSet<usize> = h
            .columns
            .iter()
            .filter_map(|c| c.source_column.as_deref())
            .filter_map(|g| h.column(g).map(|(i, _)| i))
            .collect();
        let flags_col = h.column("flags").map(|(i, _)| i);
        let motion_col = h.column("motion").map(|(i, _)| i);
        let epoch_col = match &h.epoch {
            Epoch::PerRow { column, .. } => h.column(column).map(|(i, _)| i),
            _ => None,
        };
        let placing: Vec<Vec<usize>> = h
            .frame
            .placing()
            .iter()
            .map(|alts| {
                alts.iter()
                    .filter_map(|n| h.column(n).map(|(i, _)| i))
                    .collect()
            })
            .collect();
        let primary_col = match h.frame {
            Frame::Orbit { primary: None, .. } => h.column("primary").map(|(i, _)| i),
            _ => None,
        };
        let mut prev = if from > 0 {
            size.number(from - 1)
        } else {
            None
        };
        for r in from..self.rows {
            let Some(id) = ids.text(r) else {
                return Err(Refusal::new("missing_id", format!("row {r} has no id")));
            };
            if !seen.insert(id.to_string()) {
                return Err(Refusal::new(
                    "duplicate_id",
                    format!("id {id:?} appears twice"),
                ));
            }
            let Some(s) = size.number(r) else {
                return Err(Refusal::new(
                    "missing_size",
                    format!("{id}: no size, so it has no rank; sizes are worked out before a catalogue is written"),
                ));
            };
            if let Some(p) = prev {
                if s > p {
                    return Err(Refusal::new(
                        "not_sorted",
                        format!("{id}: size {s} follows {p}; rows are stored biggest first"),
                    ));
                }
            }
            prev = Some(s);
            for &g in &grade_cols {
                if let Some(c) = self.columns[g].code(r) {
                    if Grade::from_code(c).is_none() {
                        return Err(Refusal::new(
                            "bad_grade",
                            format!("{id}: grade code {c} in {}", h.columns[g].name),
                        ));
                    }
                }
            }
            for &g in &source_cols {
                if let Some(c) = self.columns[g].code(r) {
                    if c as usize >= h.sources.len() {
                        return Err(Refusal::new(
                            "unknown_source",
                            format!("{id}: source index {c} in {}", h.columns[g].name),
                        ));
                    }
                }
            }
            // A present value whose per-row grade or source is absent has no provenance.
            for (ci, c) in h.columns.iter().enumerate() {
                if c.role == Role::Value
                    && self.columns[ci].is_present(r)
                    && self.provenance(h, r, ci).is_none()
                {
                    return Err(Refusal::new(
                        "no_provenance",
                        format!("{id}: {} has a value but no grade or source", c.name),
                    ));
                }
            }
            if let Some(f) = flags_col.and_then(|i| self.columns[i].code(r)) {
                if f & !flags::KNOWN != 0 {
                    warnings.push(Warning::new(
                        id,
                        "flags",
                        format!("unknown flag bits {f:#04x} kept, not read"),
                    ));
                }
            }
            if let Some(m) = motion_col.and_then(|i| self.columns[i].index16(r)) {
                if m as usize >= h.motions.len() {
                    return Err(Refusal::new(
                        "bad_motion",
                        format!("{id}: motion {m} is not listed"),
                    ));
                }
            }
            if let Some(e) = epoch_col {
                if !self.columns[e].is_present(r) {
                    warnings.push(Warning::new(
                        id,
                        &h.columns[e].name,
                        "no epoch: unplaced".into(),
                    ));
                }
            }
            let missing: Vec<&str> = h
                .frame
                .placing()
                .iter()
                .zip(&placing)
                .filter(|(_, cols)| !cols.iter().any(|&c| self.columns[c].is_present(r)))
                .map(|(alts, _)| alts[0])
                .collect();
            if !missing.is_empty() {
                warnings.push(Warning::new(
                    id,
                    missing[0],
                    format!("unplaced: no {}", missing.join(", ")),
                ));
            }
            if let Some(p) = primary_col {
                if !self.columns[p].is_present(r) {
                    warnings.push(Warning::new(id, "primary", "unplaced: no primary".into()));
                }
            }
        }
        Ok(())
    }
}
