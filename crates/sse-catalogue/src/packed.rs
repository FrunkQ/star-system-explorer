//! The packed form: a byte-ranged file whose prefix is everything above a
//! size cut.
//!
//! ```text
//! 0        8 bytes   magic  "SSECAT", 0x00, 0x01
//! 8        4 bytes   header length H, u32 little-endian
//! 12       H bytes   header, UTF-8 JSON, with `packed` (the block index)
//! 12 + H   blocks, back to back, biggest rows first
//! ```
//!
//! A block is its rows (fixed width, little-endian, columns in header order)
//! followed by the UTF-8 heap of its strings. A block always ends at a named
//! step, so a cut at a step reads exactly to it; any other cut reads at most
//! one block past it and decodes none of that block's rows below the cut.
//! Every block carries its SHA-256 in the index, so a piece fetched on its
//! own from a store is checked on its own.

use crate::header::{BlockInfo, ColType, Header, PackedIndex};
use crate::table::{Table, Values, NULL_U32};
use crate::{Catalogue, Cut, Refusal, Warning};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MAGIC: [u8; 8] = *b"SSECAT\x00\x01";
/// Bytes before the header.
pub const PREAMBLE: u64 = 12;
/// Rows per block when the writer is not told.
pub const DEFAULT_BLOCK_ROWS: u32 = 1024;
/// A header larger than this is refused (a 1.5 million row file at 1024
/// rows a block has an index of about 250 kB).
pub const MAX_HEADER: u32 = 16 << 20;

/// Something bytes can be asked of by range: a slice here, a byte-range
/// fetch from a store in the shell. `end` is exclusive.
pub trait ByteSource {
    fn read(&mut self, start: u64, end: u64) -> Result<Vec<u8>, Refusal>;
}

impl ByteSource for &[u8] {
    fn read(&mut self, start: u64, end: u64) -> Result<Vec<u8>, Refusal> {
        if end > self.len() as u64 || start > end {
            return Err(Refusal::need(
                "truncated",
                format!("bytes {start}..{end} asked of a {} byte file", self.len()),
                end,
            ));
        }
        Ok(self[start as usize..end as usize].to_vec())
    }
}

fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 15) as usize] as char);
    }
    s
}

fn sha(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}

/// What to fetch for a cut.
#[derive(Clone, PartialEq, Debug)]
pub struct Plan {
    pub cut: Cut,
    /// Blocks `0..blocks`.
    pub blocks: usize,
    /// Absolute byte range of those blocks, end exclusive.
    pub start: u64,
    pub end: u64,
    /// Rows in those blocks (the cut may keep fewer).
    pub rows_in_blocks: u64,
}

/// An opened packed file: its header, checked, and where its body starts.
#[derive(Clone, PartialEq, Debug)]
pub struct Packed {
    pub header: Header,
    pub body_start: u64,
    /// SHA-256 of the header bytes: the file's identity for a pack manifest.
    pub header_sha256: String,
}

/// What was read, with the warnings for rows that could not be placed.
#[derive(Clone, PartialEq, Debug)]
pub struct Reading {
    pub table: Table,
    pub warnings: Vec<Warning>,
    /// Bytes asked of the source, header included.
    pub bytes_read: u64,
}

impl Packed {
    /// How many leading bytes the header needs, from at least the 12-byte
    /// preamble.
    pub fn header_end(prefix: &[u8]) -> Result<u64, Refusal> {
        if prefix.len() < PREAMBLE as usize {
            return Err(Refusal::need(
                "need_more",
                "the preamble is 12 bytes".into(),
                PREAMBLE,
            ));
        }
        if prefix[..8] != MAGIC {
            return Err(Refusal::new(
                "unknown_format",
                "not a packed sse.catalogue/1 file (magic bytes differ)".into(),
            ));
        }
        let h = u32::from_le_bytes(prefix[8..12].try_into().expect("4 bytes"));
        if h == 0 || h > MAX_HEADER {
            return Err(Refusal::new("bad_index", format!("header length {h}")));
        }
        Ok(PREAMBLE + h as u64)
    }

    /// Open from a prefix that holds at least the header. Reads nothing past it.
    pub fn open(prefix: &[u8]) -> Result<Packed, Refusal> {
        let end = Packed::header_end(prefix)?;
        if (prefix.len() as u64) < end {
            return Err(Refusal::need(
                "need_more",
                format!("the header ends at byte {end}"),
                end,
            ));
        }
        let hb = &prefix[PREAMBLE as usize..end as usize];
        let header: Header = serde_json::from_slice(hb)
            .map_err(|e| Refusal::new("bad_header", format!("header: {e}")))?;
        header.check()?;
        let p = Packed {
            header,
            body_start: end,
            header_sha256: sha(hb),
        };
        p.check_index()?;
        Ok(p)
    }

    pub fn index(&self) -> &PackedIndex {
        self.header.packed.as_ref().expect("checked at open")
    }

    fn check_index(&self) -> Result<(), Refusal> {
        let bad = |why: String| Err(Refusal::new("bad_index", why));
        let Some(ix) = &self.header.packed else {
            return bad("a packed file's header has a block index".into());
        };
        if ix.row_bytes as usize != self.header.row_bytes() {
            return bad(format!(
                "row_bytes {} but the columns take {}",
                ix.row_bytes,
                self.header.row_bytes()
            ));
        }
        let mut at = 0u64;
        let mut rows = 0u64;
        let mut prev_min: Option<f64> = None;
        for (i, b) in ix.blocks.iter().enumerate() {
            if b.rows == 0 || b.offset != at {
                return bad(format!(
                    "block {i} is empty or not where the last one ended"
                ));
            }
            if b.bytes < b.rows as u64 * ix.row_bytes as u64 {
                return bad(format!("block {i} is shorter than its rows"));
            }
            if !(b.size_max.is_finite() && b.size_min.is_finite() && b.size_max >= b.size_min) {
                return bad(format!("block {i} has no size range"));
            }
            if prev_min.is_some_and(|p| b.size_max > p) {
                return bad(format!("block {i} is bigger than the block before it"));
            }
            if b.sha256.len() != 64 || !b.sha256.bytes().all(|c| c.is_ascii_hexdigit()) {
                return bad(format!("block {i} has no SHA-256"));
            }
            prev_min = Some(b.size_min);
            at += b.bytes;
            rows += b.rows as u64;
        }
        if rows != ix.rows {
            return bad(format!(
                "blocks hold {rows} rows, the index says {}",
                ix.rows
            ));
        }
        if ix.step_ranks.len() != self.header.steps.len() {
            return bad("one rank per step".into());
        }
        // Each step's rank is a block boundary, with the blocks on either side
        // on the right side of the step.
        for (s, &k) in self.header.steps.iter().zip(&ix.step_ranks) {
            let mut acc = 0u64;
            let mut j = 0usize;
            while j < ix.blocks.len() && acc < k {
                acc += ix.blocks[j].rows as u64;
                j += 1;
            }
            if acc != k {
                return bad(format!("step {} falls inside a block", s.size));
            }
            if j > 0 && ix.blocks[j - 1].size_min < s.size {
                return bad(format!("step {}: a row above it is smaller", s.size));
            }
            if j < ix.blocks.len() && ix.blocks[j].size_max >= s.size {
                return bad(format!("step {}: a row below it is as big", s.size));
            }
        }
        Ok(())
    }

    /// The blocks and bytes a cut needs: nothing past them is asked for.
    pub fn plan(&self, cut: Cut) -> Plan {
        let ix = self.index();
        let mut n = 0usize;
        let mut rows = 0u64;
        for b in &ix.blocks {
            let need = match cut {
                Cut::All => true,
                Cut::Size(s) => b.size_max >= s,
                Cut::Rank(k) => rows < k,
            };
            if !need {
                break;
            }
            rows += b.rows as u64;
            n += 1;
        }
        let end = ix.blocks[..n].last().map_or(0, |b| b.offset + b.bytes);
        Plan {
            cut,
            blocks: n,
            start: self.body_start,
            end: self.body_start + end,
            rows_in_blocks: rows,
        }
    }

    /// The absolute byte range of block `i`, for fetching one piece.
    pub fn block_range(&self, i: usize) -> Option<(u64, u64)> {
        let b = self.index().blocks.get(i)?;
        Some((
            self.body_start + b.offset,
            self.body_start + b.offset + b.bytes,
        ))
    }

    /// Decode block `i` from exactly its bytes, checking its hash.
    pub fn read_block(&self, i: usize, bytes: &[u8]) -> Result<Table, Refusal> {
        let h = &self.header;
        let ix = self.index();
        let b = ix
            .blocks
            .get(i)
            .ok_or_else(|| Refusal::new("bad_index", format!("no block {i}")))?;
        if bytes.len() as u64 != b.bytes {
            return Err(Refusal::new(
                "truncated",
                format!("block {i} is {} bytes, {} given", b.bytes, bytes.len()),
            ));
        }
        if sha(bytes) != b.sha256 {
            return Err(Refusal::new(
                "bad_hash",
                format!("block {i} does not match its SHA-256: the piece is damaged or from another version"),
            ));
        }
        let t = decode_block(h, b, ix.row_bytes as usize, bytes)?;
        let (si, _) = h.column(&h.size.column).expect("checked");
        let first = t.columns[si].number(0);
        let last = t.columns[si].number(t.rows - 1);
        if first != Some(b.size_max) || last != Some(b.size_min) {
            return Err(Refusal::new(
                "bad_index",
                format!("block {i}'s rows do not span the sizes its index states"),
            ));
        }
        Ok(t)
    }

    /// Decode a plan's bytes (exactly `plan.start..plan.end`) and apply the cut.
    pub fn read_plan(&self, plan: &Plan, bytes: &[u8]) -> Result<Reading, Refusal> {
        if bytes.len() as u64 != plan.end - plan.start {
            return Err(Refusal::new(
                "truncated",
                format!(
                    "the plan needs {} bytes, {} given",
                    plan.end - plan.start,
                    bytes.len()
                ),
            ));
        }
        let ix = self.index();
        let mut t = Table::empty(&self.header);
        for (i, b) in ix.blocks[..plan.blocks].iter().enumerate() {
            let s = b.offset as usize;
            t.append(self.read_block(i, &bytes[s..s + b.bytes as usize])?);
        }
        finish(
            &self.header,
            t,
            plan.cut,
            self.body_start + bytes.len() as u64,
        )
    }

    /// Read a cut from a source: asks for the plan's range and nothing else.
    pub fn read(&self, src: &mut dyn ByteSource, cut: Cut) -> Result<Reading, Refusal> {
        let plan = self.plan(cut);
        let bytes = if plan.end > plan.start {
            src.read(plan.start, plan.end)?
        } else {
            Vec::new()
        };
        let mut r = self.read_plan(&plan, &bytes)?;
        r.bytes_read = self.body_start + bytes.len() as u64;
        Ok(r)
    }
}

/// Open a packed file from a source and read a cut: three ranged reads
/// (the preamble, the header, the blocks above the cut).
pub fn read_packed(src: &mut dyn ByteSource, cut: Cut) -> Result<(Packed, Reading), Refusal> {
    let pre = src.read(0, PREAMBLE)?;
    let end = Packed::header_end(&pre)?;
    let mut prefix = pre;
    prefix.extend(src.read(PREAMBLE, end)?);
    let p = Packed::open(&prefix)?;
    let r = p.read(src, cut)?;
    Ok((p, r))
}

/// The cut applied to decoded rows, then every row checked.
pub(crate) fn finish(
    h: &Header,
    mut t: Table,
    cut: Cut,
    bytes_read: u64,
) -> Result<Reading, Refusal> {
    let keep = match cut {
        Cut::All => t.rows,
        Cut::Rank(k) => t.rows.min(usize::try_from(k).unwrap_or(usize::MAX)),
        Cut::Size(s) => (0..t.rows)
            .find(|&r| t.size(h, r).is_some_and(|x| x < s))
            .unwrap_or(t.rows),
    };
    t.truncate(keep);
    let mut warnings = Vec::new();
    t.check_rows(h, 0, &mut BTreeSet::new(), &mut warnings)?;
    Ok(Reading {
        table: t,
        warnings,
        bytes_read,
    })
}

fn decode_block(h: &Header, b: &BlockInfo, rb: usize, bytes: &[u8]) -> Result<Table, Refusal> {
    let n = b.rows as usize;
    let (rows, heap) = bytes.split_at(n * rb);
    let mut t = Table::empty(h);
    t.rows = n;
    let mut at = 0usize;
    for (ci, c) in h.columns.iter().enumerate() {
        let w = c.ty.width();
        let cell = |r: usize| &rows[r * rb + at..r * rb + at + w];
        match &mut t.columns[ci] {
            Values::F64(v) => {
                v.extend((0..n).map(|r| f64::from_le_bytes(cell(r).try_into().expect("8"))))
            }
            Values::F32(v) => {
                v.extend((0..n).map(|r| f32::from_le_bytes(cell(r).try_into().expect("4"))))
            }
            Values::I32(v) => {
                v.extend((0..n).map(|r| i32::from_le_bytes(cell(r).try_into().expect("4"))))
            }
            Values::U32(v) => {
                v.extend((0..n).map(|r| u32::from_le_bytes(cell(r).try_into().expect("4"))))
            }
            Values::U16(v) => {
                v.extend((0..n).map(|r| u16::from_le_bytes(cell(r).try_into().expect("2"))))
            }
            Values::U8(v) => v.extend((0..n).map(|r| cell(r)[0])),
            Values::Str(v) => {
                for r in 0..n {
                    let x = cell(r);
                    let off = u32::from_le_bytes(x[..4].try_into().expect("4"));
                    let len = u16::from_le_bytes(x[4..6].try_into().expect("2")) as usize;
                    if off == NULL_U32 {
                        v.push(None);
                        continue;
                    }
                    let s = heap.get(off as usize..off as usize + len).ok_or_else(|| {
                        Refusal::new("bad_block", format!("{}: string outside its block", c.name))
                    })?;
                    let s = std::str::from_utf8(s).map_err(|_| {
                        Refusal::new("bad_block", format!("{}: string is not UTF-8", c.name))
                    })?;
                    v.push(Some(s.to_string()));
                }
            }
        }
        at += w;
    }
    Ok(t)
}

fn encode_block(h: &Header, t: &Table, from: usize, to: usize) -> Vec<u8> {
    let rb = h.row_bytes();
    let mut rows = Vec::with_capacity((to - from) * rb);
    let mut heap: Vec<u8> = Vec::new();
    for r in from..to {
        for (ci, c) in h.columns.iter().enumerate() {
            match (&t.columns[ci], c.ty) {
                (Values::F64(v), _) => {
                    let x = if v[r].is_nan() { f64::NAN } else { v[r] };
                    rows.extend(x.to_le_bytes())
                }
                (Values::F32(v), _) => {
                    let x = if v[r].is_nan() { f32::NAN } else { v[r] };
                    rows.extend(x.to_le_bytes())
                }
                (Values::I32(v), _) => rows.extend(v[r].to_le_bytes()),
                (Values::U32(v), _) => rows.extend(v[r].to_le_bytes()),
                (Values::U16(v), _) => rows.extend(v[r].to_le_bytes()),
                (Values::U8(v), _) => rows.push(v[r]),
                (Values::Str(v), ColType::Str) => match &v[r] {
                    None => {
                        rows.extend(NULL_U32.to_le_bytes());
                        rows.extend(0u16.to_le_bytes());
                    }
                    Some(s) => {
                        rows.extend((heap.len() as u32).to_le_bytes());
                        rows.extend((s.len() as u16).to_le_bytes());
                        heap.extend(s.as_bytes());
                    }
                },
                _ => unreachable!("columns follow the header"),
            }
        }
    }
    rows.extend(heap);
    rows
}

/// Write the packed form. The catalogue is checked as a reader would check
/// it; blocks hold at most `block_rows` rows and always end at each step.
pub(crate) fn pack(cat: &Catalogue, block_rows: u32) -> Result<Vec<u8>, Refusal> {
    let h0 = &cat.header;
    let t = &cat.table;
    if block_rows == 0 {
        return Err(Refusal::new(
            "bad_request",
            "block_rows must be at least 1".into(),
        ));
    }
    if t.rows == 0 {
        return Err(Refusal::new(
            "empty",
            "a packed catalogue holds at least one row".into(),
        ));
    }
    let mut header = h0.clone();
    header.packed = None;
    header.check()?;
    cat.check_whole()?;
    // Where each step falls, then cut the gaps into blocks.
    let step_ranks: Vec<u64> = header
        .steps
        .iter()
        .map(|s| {
            (0..t.rows)
                .take_while(|&r| t.size(&header, r).is_some_and(|x| x >= s.size))
                .count() as u64
        })
        .collect();
    let mut bounds: BTreeSet<usize> = step_ranks.iter().map(|&k| k as usize).collect();
    bounds.insert(t.rows);
    bounds.remove(&0);
    let mut blocks = Vec::new();
    let mut body = Vec::new();
    let mut from = 0usize;
    for end in bounds {
        while from < end {
            let to = end.min(from + block_rows as usize);
            let bytes = encode_block(&header, t, from, to);
            blocks.push(BlockInfo {
                rows: (to - from) as u32,
                offset: body.len() as u64,
                bytes: bytes.len() as u64,
                size_max: t.size(&header, from).expect("checked"),
                size_min: t.size(&header, to - 1).expect("checked"),
                sha256: sha(&bytes),
            });
            body.extend(bytes);
            from = to;
        }
    }
    header.packed = Some(PackedIndex {
        rows: t.rows as u64,
        row_bytes: header.row_bytes() as u32,
        blocks,
        step_ranks,
    });
    let hb = serde_json::to_vec(&header).expect("header serialises");
    if hb.len() as u64 > MAX_HEADER as u64 {
        return Err(Refusal::new(
            "too_big",
            "the block index is too large; use bigger blocks".into(),
        ));
    }
    let mut out = Vec::with_capacity(PREAMBLE as usize + hb.len() + body.len());
    out.extend(MAGIC);
    out.extend((hb.len() as u32).to_le_bytes());
    out.extend(hb);
    out.extend(body);
    Ok(out)
}
