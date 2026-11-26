//! The catalogue format `sse.catalogue/1` and its reader (track RW, package
//! RW3; design `handover/design/SSE4-real-catalogues-design.md` section 3.1,
//! format spec `handover/design/SSE4-catalogue-format.md`).
//!
//! A catalogue is a list of measured (or authored) things, stored **biggest
//! first** and indexed by size, so a reader takes a prefix and stops: data
//! below the Detail control's cut is never fetched, parsed or held (Alex,
//! 26 Nov 17:29: "we don't even READ the data we won't use"). Real stars,
//! real asteroids, real galaxies, a fan setting's invented stars and a
//! Traveller sector are the same kind of record.
//!
//! The rules the reader enforces:
//! - **Credits or nothing.** Every catalogue names its sources, each with
//!   the provider's own credit text word for word; one with no credit is
//!   refused before any row is read.
//! - **Every value records where it came from:** each value column has a
//!   grade (measured, estimated, convention, hypothetical) and a source,
//!   fixed for the column or given per row.
//! - **Size order is part of the format.** Rows that rise in size are
//!   refused, not re-sorted.
//! - **Never invent.** Absent values stay absent; a row that cannot be
//!   placed is kept and listed as a warning. Nothing here estimates
//!   anything: sizes, masses, completeness curves and thinning are block
//!   flows (RW3b). The reader is code because reading a file is not physics.
//!
//! Two forms share one header: the text form (JSON, for authored lists and
//! build-script output) and the packed form (byte-ranged blocks with a
//! SHA-256 each, for store packs served in pieces).
#![forbid(unsafe_code)]

pub mod b64;
pub mod header;
pub mod packed;
pub mod table;
mod text;

#[cfg(test)]
mod tests;

pub use header::{
    flags, Anchor, BlockInfo, ColType, ColumnDef, Completeness, Epoch, Frame, Grade, Header,
    LengthUnit, Level, Motion, PackedIndex, Plane, Quantity, Role, SizeKey, Source, SourceKind,
    Step, TimeScale, FORMAT,
};
pub use packed::{read_packed, ByteSource, Packed, Plan, Reading, DEFAULT_BLOCK_ROWS, MAGIC};
pub use table::{Provenance, Table, Values};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::collections::BTreeSet;

/// Why a catalogue (or a piece of one) was not read. `code` is stable;
/// `reason` is plain words for the preview.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Refusal {
    pub code: String,
    pub reason: String,
    /// For `need_more` and `truncated`: the byte count the reader needs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_bytes: Option<u64>,
}

impl Refusal {
    pub fn new(code: &str, reason: String) -> Refusal {
        Refusal {
            code: code.into(),
            reason,
            need_bytes: None,
        }
    }
    pub(crate) fn need(code: &str, reason: String, n: u64) -> Refusal {
        Refusal {
            code: code.into(),
            reason,
            need_bytes: Some(n),
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.reason)
    }
}

/// A row kept with something the reader could not use.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Warning {
    pub id: String,
    pub column: String,
    pub what: String,
}

impl Warning {
    pub(crate) fn new(id: &str, column: &str, what: String) -> Warning {
        Warning {
            id: id.into(),
            column: column.into(),
            what,
        }
    }
}

/// Which rows to read: everything, everything at or above a size (in the
/// size key's unit), or the first `k`.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cut {
    All,
    Size(f64),
    Rank(u64),
}

/// A whole catalogue in memory (the text form, or a packed file read in full).
#[derive(Clone, PartialEq, Debug)]
pub struct Catalogue {
    pub header: Header,
    pub table: Table,
}

impl Catalogue {
    /// Read the text form, checking every row.
    pub fn from_text(text: &str) -> Result<(Catalogue, Vec<Warning>), Refusal> {
        let (header, table) = text::parse(text)?;
        let cat = Catalogue { header, table };
        let w = cat.check_whole()?;
        Ok((cat, w))
    }

    /// Read the text form without checking order, for a build tool that
    /// will sort it next ([`Catalogue::sort_by_size`]).
    pub fn from_text_unsorted(text: &str) -> Result<Catalogue, Refusal> {
        let (header, table) = text::parse(text)?;
        Ok(Catalogue { header, table })
    }

    pub fn to_text(&self) -> String {
        text::write(self)
    }

    /// The packed form, `block_rows` rows a block at most.
    pub fn pack(&self, block_rows: u32) -> Result<Vec<u8>, Refusal> {
        packed::pack(self, block_rows)
    }

    /// Every rule a whole catalogue meets: the row checks, and the anchor's
    /// id (when it names one) present.
    pub fn check_whole(&self) -> Result<Vec<Warning>, Refusal> {
        let mut w = Vec::new();
        self.table
            .check_rows(&self.header, 0, &mut BTreeSet::new(), &mut w)?;
        if let Some(a) = &self.header.anchor.id {
            if !(0..self.table.rows).any(|r| self.table.id(&self.header, r) == Some(a)) {
                return Err(Refusal::new(
                    "bad_anchor",
                    format!("the anchor {a:?} is not an entry"),
                ));
            }
        }
        Ok(w)
    }

    /// Put the rows biggest first (ties by id), for build tools. A row with
    /// no size is refused: sizes are worked out before a catalogue is written.
    pub fn sort_by_size(&mut self) -> Result<(), Refusal> {
        let h = &self.header;
        let t = &self.table;
        let mut keys = Vec::with_capacity(t.rows);
        for r in 0..t.rows {
            let s = t.size(h, r).ok_or_else(|| {
                Refusal::new(
                    "missing_size",
                    format!("{}: no size", t.id(h, r).unwrap_or("?")),
                )
            })?;
            keys.push((s, t.id(h, r).unwrap_or("").to_string(), r));
        }
        keys.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        let order: Vec<usize> = keys.into_iter().map(|k| k.2).collect();
        let mut out = Table::empty(h);
        out.rows = t.rows;
        for (o, c) in out.columns.iter_mut().zip(&t.columns) {
            *o = permute(c, &order);
        }
        self.table = out;
        Ok(())
    }

    /// The rows a cut keeps.
    pub fn cut(&self, cut: Cut) -> Result<Reading, Refusal> {
        packed::finish(&self.header, self.table.clone(), cut, 0)
    }
}

fn permute(v: &Values, order: &[usize]) -> Values {
    match v {
        Values::F64(x) => Values::F64(order.iter().map(|&i| x[i]).collect()),
        Values::F32(x) => Values::F32(order.iter().map(|&i| x[i]).collect()),
        Values::I32(x) => Values::I32(order.iter().map(|&i| x[i]).collect()),
        Values::U32(x) => Values::U32(order.iter().map(|&i| x[i]).collect()),
        Values::U16(x) => Values::U16(order.iter().map(|&i| x[i]).collect()),
        Values::U8(x) => Values::U8(order.iter().map(|&i| x[i]).collect()),
        Values::Str(x) => Values::Str(order.iter().map(|&i| x[i].clone()).collect()),
    }
}

/// Either form from its bytes, with a cut: the packed form is read through
/// its index (only the blocks above the cut are decoded), the text form is
/// parsed and cut. Returns the header, the reading and, for a packed file,
/// what it opened.
pub fn read_bytes(bytes: &[u8], cut: Cut) -> Result<(Header, Reading, Option<Packed>), Refusal> {
    if bytes.starts_with(&MAGIC) {
        let mut src: &[u8] = bytes;
        let (p, r) = read_packed(&mut src, cut)?;
        Ok((p.header.clone(), r, Some(p)))
    } else {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Refusal::new("unknown_format", "neither packed nor UTF-8 text".into()))?;
        let (cat, mut w) = Catalogue::from_text(text)?;
        let mut r = cat.cut(cut)?;
        // Warnings for rows the cut kept only.
        let kept: BTreeSet<&str> = (0..r.table.rows)
            .filter_map(|i| r.table.id(&cat.header, i))
            .collect();
        w.retain(|x| kept.contains(x.id.as_str()));
        r.warnings = w;
        r.bytes_read = bytes.len() as u64;
        Ok((cat.header, r, None))
    }
}

/// The import preview (design 5.1 and 5.3): what it is, its credits in
/// full, the price rule, how many rows each step holds, what the cut read,
/// the first `limit` rows and the warnings. Shown before anything is applied.
pub fn preview(h: &Header, r: &Reading, packed: Option<&Packed>, limit: usize) -> Json {
    let rows: Vec<Json> = (0..r.table.rows.min(limit))
        .map(|i| r.table.row_json(h, i))
        .collect();
    let steps: Vec<Json> = match packed {
        Some(p) => h
            .steps
            .iter()
            .zip(&p.index().step_ranks)
            .map(|(s, k)| json!({"size": s.size, "label": s.label, "rows": k}))
            .collect(),
        None => h
            .steps
            .iter()
            .map(|s| json!({"size": s.size, "label": s.label}))
            .collect(),
    };
    let sizes: Vec<f64> = (0..r.table.rows)
        .filter_map(|i| r.table.size(h, i))
        .collect();
    json!({
        "id": h.id,
        "title": h.title,
        "version": h.version,
        "level": h.level,
        "frame": h.frame,
        "anchor": h.anchor,
        "epoch": h.epoch,
        "size": h.size,
        "columns": h.columns.iter().map(|c| json!({"name": c.name, "type": c.ty, "role": c.role, "unit": c.unit})).collect::<Vec<_>>(),
        "credits": h.sources,
        "price": match h.may_carry_price() {
            Ok(()) => json!({"may_sell": true}),
            Err(why) => json!({"may_sell": false, "why": why}),
        },
        "completeness": h.completeness,
        "steps": steps,
        "motions": h.motions,
        "notes": h.notes,
        "total_rows": packed.map(|p| p.index().rows),
        "total_bytes": packed.map(|p| p.body_start + p.index().blocks.last().map_or(0, |b| b.offset + b.bytes)),
        "header_sha256": packed.map(|p| p.header_sha256.clone()),
        "read": {
            "rows": r.table.rows,
            "bytes": r.bytes_read,
            "largest": sizes.first(),
            "smallest": sizes.last(),
        },
        "rows": rows,
        "warnings_total": r.warnings.len(),
        "warnings": r.warnings.iter().take(limit).collect::<Vec<_>>(),
    })
}
