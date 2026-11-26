//! The header: everything about a catalogue except its rows.
//!
//! The same header heads the text form and the packed form; the packed form
//! adds its block index (`packed`). Every rule here is checked by
//! [`Header::check`] before a single row is read, so a catalogue that cannot
//! say where its values came from, or whose credit is missing, is refused
//! before it costs anything.

use crate::Refusal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The format string every catalogue carries, word for word.
pub const FORMAT: &str = "sse.catalogue/1";

/// What kind of slot the entries claim (design 3.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Galaxy,
    Star,
    /// A planet or a moon.
    World,
    SmallBody,
}

/// The plane an orbit frame's angles are measured in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plane {
    /// The ecliptic and equinox of J2000.0.
    EclipticJ2000,
    /// The primary's own equator (most moon tables).
    PrimaryEquator,
    /// Each body's local Laplace plane (JPL's satellite mean elements).
    Laplace,
}

/// Length units a cartesian frame may use.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
    Pc,
    Ly,
    Au,
    Km,
    M,
}

/// The frame the positions are in. Each frame names the columns that place
/// an entry (see [`Frame::placing`]); a row missing one is kept and listed
/// as unplaced, never given a made-up place.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Frame {
    /// Right ascension and declination (ICRS), with a distance: columns
    /// `ra_deg`, `dec_deg`, `distance_pc`. Optional motion columns
    /// `pm_ra_cosdec_mas_yr`, `pm_dec_mas_yr`, `rv_km_s`.
    Icrs,
    /// Galactic longitude and latitude with a distance: `l_deg`, `b_deg`,
    /// `distance_pc`.
    Galactic,
    /// Orbital elements round a primary: `a_au` or `q_au`, `e`, `i_deg`,
    /// `node_deg`, `peri_deg`, and `mean_anomaly_deg` or `tp_jd`. The
    /// primary is `primary` here, or a `primary` string column per row.
    Orbit {
        plane: Plane,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        primary: Option<String>,
    },
    /// Authored positions: `x`, `y`, `z` in `unit`; `axes` says what the
    /// axes point at, in words.
    Cartesian { unit: LengthUnit, axes: String },
    /// A hex map (a Traveller sector): a `hex` string column.
    HexGrid { grid: String, parsecs_per_hex: f64 },
}

impl Frame {
    /// The columns that place an entry: each inner list is "one of these".
    pub fn placing(&self) -> Vec<Vec<&'static str>> {
        match self {
            Frame::Icrs => vec![vec!["ra_deg"], vec!["dec_deg"], vec!["distance_pc"]],
            Frame::Galactic => vec![vec!["l_deg"], vec!["b_deg"], vec!["distance_pc"]],
            Frame::Orbit { .. } => vec![
                vec!["a_au", "q_au"],
                vec!["e"],
                vec!["i_deg"],
                vec!["node_deg"],
                vec!["peri_deg"],
                vec!["mean_anomaly_deg", "tp_jd"],
            ],
            Frame::Cartesian { .. } => vec![vec!["x"], vec!["y"], vec!["z"]],
            Frame::HexGrid { .. } => vec![vec!["hex"]],
        }
    }
}

/// The one entry (or named body) everything is placed from (design 3.1).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    /// "Sun", "Jupiter", "Regina".
    pub name: String,
    /// Its id in this catalogue, when the anchor is itself an entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A time scale for Julian dates.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeScale {
    Tdb,
    Tt,
    Utc,
}

/// When the positions were true.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Epoch {
    /// One Julian date for every row.
    Jd { jd: f64, scale: TimeScale },
    /// Each row's own Julian date, in a float column (MPC orbits).
    PerRow { column: String, scale: TimeScale },
    /// Universe seconds, for authored catalogues of invented places.
    Universe { s: i64 },
}

/// What "size" measures at this level.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quantity {
    Mass,
    Diameter,
}

/// The size key: the column every row is sorted on, biggest first.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizeKey {
    pub column: String,
    pub quantity: Quantity,
    /// mass: `kg`, `msun`, `mearth`; diameter: `km`, `m`.
    pub unit: String,
}

/// How a column is stored.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColType {
    F64,
    F32,
    I32,
    U32,
    U16,
    U8,
    Str,
}

impl ColType {
    /// Bytes this column takes in a packed row.
    pub fn width(self) -> usize {
        match self {
            ColType::F64 => 8,
            ColType::F32 | ColType::I32 | ColType::U32 => 4,
            ColType::U16 => 2,
            ColType::U8 => 1,
            // u32 offset into the block's string heap, u16 length.
            ColType::Str => 6,
        }
    }
    pub fn is_float(self) -> bool {
        matches!(self, ColType::F64 | ColType::F32)
    }
}

/// What a column is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The stable id. Exactly one column, named `id`, of type `str`.
    Id,
    /// A value taken from a source: it has a grade and a source.
    Value,
    /// Bookkeeping about other columns (grade codes, source indices,
    /// `flags`, `motion`): no grade and no source of its own.
    Meta,
}

/// How a value was arrived at. Stored in a grade column as its code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    Measured,
    Estimated,
    Convention,
    /// Argued for and not seen (design 6A.3): never counted as measured.
    Hypothetical,
}

impl Grade {
    pub const ALL: [Grade; 4] = [
        Grade::Measured,
        Grade::Estimated,
        Grade::Convention,
        Grade::Hypothetical,
    ];
    pub fn code(self) -> u8 {
        self as u8
    }
    pub fn from_code(c: u8) -> Option<Grade> {
        Grade::ALL.get(c as usize).copied()
    }
}

/// One column.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: ColType,
    pub role: Role,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Every value in the column has this grade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grade: Option<Grade>,
    /// Or each row's grade is in this `u8` meta column ([`Grade::code`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grade_column: Option<String>,
    /// Every value in the column came from this source id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Or each row's source is in this `u8` meta column (an index into
    /// `sources`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

/// Downloaded from an outside catalogue, or written by an author.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Downloaded,
    Authored,
}

/// A catalogue the values came from, with its credit word for word
/// (Alex, 26 Nov 18:01: "the actual credits are shipped in packs as clear
/// readable and show at initial import").
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: SourceKind,
    pub licence: String,
    /// The provider's own credit text, word for word. Required.
    pub credit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// When it was fetched (an ISO date, as written).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved: Option<String>,
}

/// How much of what exists the catalogue holds (design 3.1, 3.3). Bands,
/// not a curve: the reader only reports the stated band; smoothing between
/// bands is the thinning flow's job (RW3b), not the reader's.
///
/// `size_edges` are the lower bounds of size bands, biggest first: band `i`
/// holds sizes `>= size_edges[i]` and `< size_edges[i - 1]` (band 0 has no
/// upper bound). Below the last edge nothing is stated. `distance_edges_pc`
/// are the upper bounds of distance bands, nearest first (empty: one band for
/// every distance, as inside one system). `fraction[i][j]` is the share
/// held, 0 to 1.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completeness {
    pub size_edges: Vec<f64>,
    #[serde(default)]
    pub distance_edges_pc: Vec<f64>,
    pub fraction: Vec<Vec<f64>>,
    pub grade: Grade,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Completeness {
    /// The stated share for a size (and distance, pc), or None outside every
    /// band.
    pub fn stated(&self, size: f64, distance_pc: Option<f64>) -> Option<f64> {
        let i = self.size_edges.iter().position(|&e| size >= e)?;
        let j = if self.distance_edges_pc.is_empty() {
            0
        } else {
            let d = distance_pc?;
            self.distance_edges_pc.iter().position(|&e| d <= e)?
        };
        self.fraction.get(i)?.get(j).copied()
    }
}

/// A named stop in the file, where a block always ends (design 7.2).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// Rows with size `>=` this are above the stop.
    pub size: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A measured motion a row may name (design 6A.2, package RW17): inside its
/// span the body is where the model says; outside it the body hands over to
/// the kernel's own orbit. The reader checks the slot; it runs no model.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    pub id: String,
    /// "vsop87", "elp", "mean_orbit_with_rates", ...
    pub model: String,
    /// The pack table that holds its terms.
    pub table: String,
    /// Julian dates `[from, to]` over which it holds.
    pub span_jd: [f64; 2],
    pub scale: TimeScale,
    pub source: String,
}

/// One block of a packed file (offsets from the end of the header).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockInfo {
    pub rows: u32,
    pub offset: u64,
    pub bytes: u64,
    /// Size of its first (biggest) and last (smallest) row.
    pub size_max: f64,
    pub size_min: f64,
    /// SHA-256 of the block's bytes, lower-case hex.
    pub sha256: String,
}

/// The packed form's index.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackedIndex {
    pub rows: u64,
    pub row_bytes: u32,
    pub blocks: Vec<BlockInfo>,
    /// For each of `steps`, how many rows are at or above it.
    pub step_ranks: Vec<u64>,
}

/// The header of a catalogue.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub format: String,
    /// A stable slug: `sol-small-bodies`, `cns5`.
    pub id: String,
    pub title: String,
    pub version: String,
    pub level: Level,
    pub frame: Frame,
    pub anchor: Anchor,
    pub epoch: Epoch,
    pub size: SizeKey,
    pub columns: Vec<ColumnDef>,
    pub sources: Vec<Source>,
    pub completeness: Completeness,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub motions: Vec<Motion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Present only in the packed form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packed: Option<PackedIndex>,
}

/// Bits of the standard `flags` meta column.
pub mod flags {
    /// Shown whatever the Detail cut (design 7.3): a pack-flagged notable.
    pub const NOTABLE: u8 = 1;
    /// The entry itself is hypothetical (design 6A.3): off by default.
    pub const HYPOTHETICAL: u8 = 2;
    /// Every bit this version knows.
    pub const KNOWN: u8 = NOTABLE | HYPOTHETICAL;
}

fn slug_ok(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

fn name_ok(s: &str) -> bool {
    !s.is_empty()
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn units_for(q: Quantity) -> &'static [&'static str] {
    match q {
        Quantity::Mass => &["kg", "msun", "mearth"],
        Quantity::Diameter => &["km", "m"],
    }
}

impl Header {
    /// The column definition called `name`.
    pub fn column(&self, name: &str) -> Option<(usize, &ColumnDef)> {
        self.columns
            .iter()
            .enumerate()
            .find(|(_, c)| c.name == name)
    }

    /// The source called `id`.
    pub fn source(&self, id: &str) -> Option<(usize, &Source)> {
        self.sources.iter().enumerate().find(|(_, s)| s.id == id)
    }

    /// Bytes one packed row takes.
    pub fn row_bytes(&self) -> usize {
        self.columns.iter().map(|c| c.ty.width()).sum()
    }

    /// Whether anything in it was downloaded from an outside catalogue.
    pub fn holds_downloaded(&self) -> bool {
        self.sources
            .iter()
            .any(|s| s.kind == SourceKind::Downloaded)
    }

    /// The store's "may this carry a price" rule (Alex, 26 Nov 18:00: "you
    /// can give it away but you cannot 'sell it'"): `Err` with the reason in
    /// words when any value was downloaded.
    pub fn may_carry_price(&self) -> Result<(), String> {
        let names: Vec<&str> = self
            .sources
            .iter()
            .filter(|s| s.kind == SourceKind::Downloaded)
            .map(|s| s.name.as_str())
            .collect();
        if names.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "You can use this and give it away, but it cannot be sold: it contains data from {}.",
                names.join(", ")
            ))
        }
    }

    /// Every rule a header must meet, checked before any row is read.
    pub fn check(&self) -> Result<(), Refusal> {
        let bad = |code: &str, why: String| Err(Refusal::new(code, why));
        if self.format != FORMAT {
            return bad(
                "unknown_format",
                format!("format is {:?}; this reader reads {FORMAT}", self.format),
            );
        }
        if !slug_ok(&self.id) {
            return bad(
                "bad_header",
                format!("id {:?} must be a lower-case slug", self.id),
            );
        }
        if self.title.trim().is_empty() || self.version.trim().is_empty() {
            return bad(
                "bad_header",
                "a catalogue needs a title and a version".into(),
            );
        }
        // Credits first: a catalogue that cannot say whose data it is, is refused.
        if self.sources.is_empty() {
            return bad(
                "no_credit",
                "the catalogue names no source, so it has no credit to show".into(),
            );
        }
        let mut ids = BTreeSet::new();
        for s in &self.sources {
            if s.credit.trim().is_empty() {
                return bad(
                    "no_credit",
                    format!(
                        "source {:?} has no credit text; it cannot be shown, so it is not read",
                        s.id
                    ),
                );
            }
            if s.licence.trim().is_empty()
                || s.name.trim().is_empty()
                || s.version.trim().is_empty()
            {
                return bad(
                    "no_credit",
                    format!(
                        "source {:?} needs a name, a version and a licence beside its credit",
                        s.id
                    ),
                );
            }
            if !slug_ok(&s.id) || !ids.insert(s.id.as_str()) {
                return bad(
                    "bad_header",
                    format!("source id {:?} is not a unique slug", s.id),
                );
            }
        }
        if self.sources.len() > 255 {
            return bad("bad_header", "at most 255 sources (a u8 index)".into());
        }
        // Columns.
        let mut names = BTreeSet::new();
        for c in &self.columns {
            if !name_ok(&c.name) || !names.insert(c.name.as_str()) {
                return bad(
                    "bad_column",
                    format!("column name {:?} is not a unique snake_case name", c.name),
                );
            }
        }
        match self.column("id") {
            Some((_, c)) if c.ty == ColType::Str && c.role == Role::Id => {}
            _ => {
                return bad(
                    "missing_column",
                    "every catalogue has an `id` column of type str and role id".into(),
                )
            }
        }
        if self.columns.iter().filter(|c| c.role == Role::Id).count() != 1 {
            return bad("bad_column", "exactly one column has the role id".into());
        }
        for c in &self.columns {
            self.check_column(c)?;
        }
        // The size key.
        let Some((_, sc)) = self.column(&self.size.column) else {
            return bad(
                "missing_column",
                format!("the size column {:?} is not declared", self.size.column),
            );
        };
        if !sc.ty.is_float() || sc.role != Role::Value {
            return bad(
                "bad_column",
                "the size column is a float value column".into(),
            );
        }
        if !units_for(self.size.quantity).contains(&self.size.unit.as_str()) {
            return bad(
                "bad_header",
                format!(
                    "size unit {:?} is not one of {:?} for {:?}",
                    self.size.unit,
                    units_for(self.size.quantity),
                    self.size.quantity
                ),
            );
        }
        if sc.unit.as_deref().is_some_and(|u| u != self.size.unit) {
            return bad(
                "bad_header",
                "the size column's unit differs from the size key's".into(),
            );
        }
        // The frame's placing columns must be declared (rows may still lack values).
        for alts in self.frame.placing() {
            if !alts.iter().any(|n| self.column(n).is_some()) {
                return bad(
                    "missing_column",
                    format!(
                        "the frame places entries by {}, which is not declared",
                        alts.join(" or ")
                    ),
                );
            }
        }
        if let Frame::Orbit { primary: None, .. } = self.frame {
            if self.column("primary").map(|(_, c)| c.ty) != Some(ColType::Str) {
                return bad(
                    "missing_column",
                    "an orbit frame names its primary, or has a `primary` str column".into(),
                );
            }
        }
        if let Frame::HexGrid {
            parsecs_per_hex, ..
        } = self.frame
        {
            if !(parsecs_per_hex.is_finite() && parsecs_per_hex > 0.0) {
                return bad("bad_header", "parsecs_per_hex must be positive".into());
            }
        }
        // Epoch.
        match &self.epoch {
            Epoch::Jd { jd, .. } if !jd.is_finite() => {
                return bad("bad_header", "the epoch is not a finite Julian date".into())
            }
            Epoch::PerRow { column, .. } => match self.column(column) {
                Some((_, c)) if c.ty.is_float() => {}
                _ => {
                    return bad(
                        "missing_column",
                        format!("the epoch column {column:?} is not a declared float column"),
                    )
                }
            },
            _ => {}
        }
        if let Some(id) = &self.anchor.id {
            if id.is_empty() {
                return bad("bad_header", "the anchor's id is empty".into());
            }
        }
        if self.anchor.name.trim().is_empty() {
            return bad("bad_header", "the anchor needs a name".into());
        }
        // Standard meta columns.
        if let Some((_, c)) = self.column("flags") {
            if c.ty != ColType::U8 || c.role != Role::Meta {
                return bad("bad_column", "`flags` is a u8 meta column".into());
            }
        }
        if let Some((_, c)) = self.column("motion") {
            if c.ty != ColType::U16 || c.role != Role::Meta {
                return bad("bad_column", "`motion` is a u16 meta column".into());
            }
        }
        self.check_completeness()?;
        // Steps: strictly falling sizes.
        for w in self.steps.windows(2) {
            if w[1].size >= w[0].size {
                return bad(
                    "bad_header",
                    "steps are listed biggest first, each smaller".into(),
                );
            }
        }
        if self.steps.iter().any(|s| !s.size.is_finite()) {
            return bad("bad_header", "a step size is not finite".into());
        }
        // Motions.
        let mut mids = BTreeSet::new();
        for mo in &self.motions {
            if !mids.insert(mo.id.as_str()) {
                return bad("bad_header", format!("motion id {:?} repeats", mo.id));
            }
            if !(mo.span_jd[0].is_finite()
                && mo.span_jd[1].is_finite()
                && mo.span_jd[0] < mo.span_jd[1])
            {
                return bad("bad_header", format!("motion {:?} has no span", mo.id));
            }
            if self.source(&mo.source).is_none() {
                return bad(
                    "unknown_source",
                    format!(
                        "motion {:?} names source {:?}, which is not listed",
                        mo.id, mo.source
                    ),
                );
            }
            if mo.table.trim().is_empty() || mo.model.trim().is_empty() {
                return bad(
                    "bad_header",
                    format!("motion {:?} needs a model and a table", mo.id),
                );
            }
        }
        if !self.motions.is_empty() && self.column("motion").is_none() {
            return bad(
                "missing_column",
                "motions are listed but there is no `motion` column".into(),
            );
        }
        if self.motions.len() >= u16::MAX as usize {
            return bad("bad_header", "too many motions".into());
        }
        Ok(())
    }

    fn check_column(&self, c: &ColumnDef) -> Result<(), Refusal> {
        let bad = |why: String| Err(Refusal::new("bad_column", why));
        let meta_u8 = |name: &str| {
            self.column(name)
                .is_some_and(|(_, m)| m.ty == ColType::U8 && m.role == Role::Meta)
        };
        match c.role {
            Role::Value => {
                match (&c.grade, &c.grade_column) {
                    (Some(_), None) => {}
                    (None, Some(g)) if meta_u8(g) => {}
                    (None, Some(g)) => {
                        return bad(format!(
                            "{}: grade column {g:?} is not a u8 meta column",
                            c.name
                        ))
                    }
                    _ => {
                        return bad(format!(
                            "{}: a value column has exactly one of grade or grade_column",
                            c.name
                        ))
                    }
                }
                match (&c.source, &c.source_column) {
                    (Some(s), None) => {
                        if self.source(s).is_none() {
                            return Err(Refusal::new(
                                "unknown_source",
                                format!("{}: source {s:?} is not listed", c.name),
                            ));
                        }
                    }
                    (None, Some(s)) if meta_u8(s) => {}
                    (None, Some(s)) => {
                        return bad(format!("{}: source column {s:?} is not a u8 meta column", c.name))
                    }
                    _ => {
                        return bad(format!(
                            "{}: every value records its source: exactly one of source or source_column",
                            c.name
                        ))
                    }
                }
            }
            Role::Id | Role::Meta => {
                if c.grade.is_some()
                    || c.grade_column.is_some()
                    || c.source.is_some()
                    || c.source_column.is_some()
                {
                    return bad(format!(
                        "{}: only value columns carry a grade and a source",
                        c.name
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_completeness(&self) -> Result<(), Refusal> {
        let c = &self.completeness;
        let bad = |why: &str| Err(Refusal::new("bad_completeness", why.to_string()));
        if c.size_edges.is_empty() {
            return bad("completeness needs at least one size band");
        }
        if c.size_edges.iter().any(|e| !e.is_finite())
            || c.size_edges.windows(2).any(|w| w[1] >= w[0])
        {
            return bad("size edges are finite and fall, biggest first");
        }
        if c.distance_edges_pc.iter().any(|e| !e.is_finite())
            || c.distance_edges_pc.windows(2).any(|w| w[1] <= w[0])
        {
            return bad("distance edges are finite and rise, nearest first");
        }
        let cols = c.distance_edges_pc.len().max(1);
        if c.fraction.len() != c.size_edges.len() || c.fraction.iter().any(|r| r.len() != cols) {
            return bad("fraction has one row per size band and one entry per distance band");
        }
        if c.fraction
            .iter()
            .flatten()
            .any(|f| !(f.is_finite() && (0.0..=1.0).contains(f)))
        {
            return bad("every fraction is between 0 and 1");
        }
        if self.source(&c.source).is_none() {
            return Err(Refusal::new(
                "unknown_source",
                format!(
                    "completeness names source {:?}, which is not listed",
                    c.source
                ),
            ));
        }
        Ok(())
    }
}
