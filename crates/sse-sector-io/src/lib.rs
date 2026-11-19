//! Sector data formats for the Traveller pack (build plan X3c, parsing half).
//!
//! Parsers and writers for the community formats a GM brings to a table:
//! loose "one world per line" pastes, legacy fixed-column SEC files,
//! Travellermap's T5 Second Survey tab-delimited files, and the sector
//! metadata XML. Nothing here knows any physics: a world is kept exactly as
//! written, and the rule for every parser is **never invent**. A token the
//! parser cannot place becomes a [`Warning`]; a line is kept whenever a UWP
//! was found in it, and the original text stays in [`WorldLine::raw`].
//!
//! Round trip: for every format, parsing what a writer wrote gives back the
//! same world fields (everything but `raw`). The sector XML carries only the
//! sector's name, abbreviation and subsector names.
#![forbid(unsafe_code)]

mod sec;
mod t5tab;
mod token;
mod uwp_lines;
mod xml;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use sec::{parse_sec, write_sec};
pub use t5tab::{parse_t5_tab, write_t5_tab};
pub use token::{is_ehex, is_hex, is_uwp};
pub use uwp_lines::{parse_uwp_lines, write_uwp_lines};
pub use xml::{parse_sector_xml, write_sector_xml};

/// One world, as written in the source.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldLine {
    /// "0101".."3240", four digits as written; empty when the source had none.
    pub hex: String,
    pub name: String,
    /// "A867975-9" as written, keeping any '?' or 'X'.
    pub uwp: String,
    /// One-letter T5 base codes, e.g. `["N", "S"]`. Legacy two-base letters
    /// (A = N + S, B = N + W) are expanded; the original stays in `raw`.
    pub bases: Vec<String>,
    /// Trade codes and other remarks, in their written order.
    pub remarks: Vec<String>,
    /// 'A' (amber) or 'R' (red); `None` for green, blank or '-'.
    pub zone: Option<char>,
    pub pbg: String,
    pub allegiance: String,
    /// One entry per star: "G2 V", "M3 V", "BD", "D".
    pub stars: Vec<String>,
    /// T5 importance `{Ix}`, without braces.
    pub importance: Option<String>,
    /// T5 economic extension `(Ex)`, without parentheses.
    pub economic: Option<String>,
    /// T5 cultural extension `[Cx]`, without brackets.
    pub cultural: Option<String>,
    pub nobility: Option<String>,
    pub worlds: Option<u32>,
    pub ru: Option<String>,
    /// The original line.
    pub raw: String,
}

impl WorldLine {
    /// Column and row of the hex, when it is a valid four-digit hex.
    pub fn xy(&self) -> Option<(u32, u32)> {
        hex_to_xy(&self.hex)
    }

    /// The subsector letter of the hex, when it has one.
    pub fn subsector(&self) -> Option<char> {
        self.xy().map(|(x, y)| subsector_of(x, y))
    }

    /// True when every field but `raw` is equal.
    pub fn same_fields(&self, other: &WorldLine) -> bool {
        let mut a = self.clone();
        a.raw.clone_from(&other.raw);
        a == *other
    }
}

/// Something a parser could not place. `line` is 1-based; `raw` is that line.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    pub line: usize,
    pub text: String,
    pub raw: String,
}

/// A parsed file: sector metadata (when the format carries any), worlds in
/// source order, and every warning.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectorFile {
    pub name: Option<String>,
    pub abbreviation: Option<String>,
    /// Subsector letter 'A'..'P' to its name.
    pub subsectors: BTreeMap<char, String>,
    pub worlds: Vec<WorldLine>,
    pub warnings: Vec<Warning>,
}

impl SectorFile {
    /// Takes the name, abbreviation and subsector names from `meta` wherever
    /// `meta` has them (the metadata file wins); keeps this file's worlds
    /// and appends `meta`'s warnings.
    pub fn with_metadata(mut self, meta: SectorFile) -> SectorFile {
        if meta.name.is_some() {
            self.name = meta.name;
        }
        if meta.abbreviation.is_some() {
            self.abbreviation = meta.abbreviation;
        }
        self.subsectors.extend(meta.subsectors);
        self.warnings.extend(meta.warnings);
        self
    }

    /// The worlds of one subsector ('A'..'P', either case). Worlds without
    /// a valid hex are left out. Name, abbreviation and the parse warnings
    /// are kept; only that subsector's name is kept.
    pub fn subsector(&self, letter: char) -> SectorFile {
        let letter = letter.to_ascii_uppercase();
        SectorFile {
            name: self.name.clone(),
            abbreviation: self.abbreviation.clone(),
            subsectors: self
                .subsectors
                .iter()
                .filter(|(k, _)| **k == letter)
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
            worlds: self
                .worlds
                .iter()
                .filter(|w| w.subsector() == Some(letter))
                .cloned()
                .collect(),
            warnings: self.warnings.clone(),
        }
    }

    pub(crate) fn warn(&mut self, line: usize, text: impl Into<String>, raw: &str) {
        self.warnings.push(Warning {
            line,
            text: text.into(),
            raw: raw.to_string(),
        });
    }

    pub(crate) fn warn_all(&mut self, line: usize, msgs: Vec<String>, raw: &str) {
        for m in msgs {
            self.warn(line, m, raw);
        }
    }
}

/// Hex "1910" to (column 19, row 10). `None` unless four ASCII digits.
pub fn hex_to_xy(hex: &str) -> Option<(u32, u32)> {
    if !is_hex(hex) {
        return None;
    }
    let x = hex[..2].parse().ok()?;
    let y = hex[2..].parse().ok()?;
    Some((x, y))
}

/// (19, 10) to "1910".
pub fn xy_to_hex(x: u32, y: u32) -> String {
    format!("{x:02}{y:02}")
}

/// Subsector letter of a hex: a sector is 32 columns by 40 rows, four by
/// four subsectors of 8 columns by 10 rows, lettered A..P across then down.
/// Positions outside 1..=32, 1..=40 are clamped to the nearest edge.
pub fn subsector_of(x: u32, y: u32) -> char {
    let cx = (x.clamp(1, 32) - 1) / 8;
    let cy = (y.clamp(1, 40) - 1) / 10;
    char::from(b'A' + (cy * 4 + cx) as u8)
}

/// True for a hex inside one sector (columns 01..32, rows 01..40).
pub(crate) fn hex_in_range(hex: &str) -> bool {
    hex_to_xy(hex).is_some_and(|(x, y)| (1..=32).contains(&x) && (1..=40).contains(&y))
}

/// The formats this crate reads and writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Format {
    /// Travellermap's tab-delimited T5 Second Survey format.
    T5Tab,
    /// Legacy fixed-column SEC.
    Sec,
    /// Sector metadata XML.
    Xml,
    /// Loose "one world per line" text.
    UwpLines,
}

/// Guesses the format of `text`:
/// XML when it starts with '<'; T5 tab when the first line with content has
/// tabs and a UWP column; SEC when it has a dash ruler line, or '#' comment
/// lines and world lines with the hex just before the UWP; otherwise loose
/// UWP lines.
pub fn detect(text: &str) -> Format {
    let text = token::strip_bom(text);
    if text.trim_start().starts_with('<') {
        return Format::Xml;
    }
    let first = text
        .lines()
        .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'));
    if let Some(first) = first {
        if first.contains('\t')
            && first
                .split('\t')
                .any(|c| c.trim().eq_ignore_ascii_case("uwp"))
        {
            return Format::T5Tab;
        }
    }
    let (mut comment, mut sec_shape, mut loose_shape) = (false, 0usize, 0usize);
    for line in text.lines() {
        if line.trim_start().starts_with('#') {
            comment = true;
            continue;
        }
        if token::ruler(line).is_some() {
            return Format::Sec;
        }
        let toks = token::tokenize(line);
        if let Some(u) = toks.iter().position(|t| is_uwp(&t.text)) {
            if u >= 1 && is_hex(&toks[u - 1].text) {
                sec_shape += 1;
            } else {
                loose_shape += 1;
            }
        }
    }
    if comment && sec_shape > 0 && sec_shape >= loose_shape {
        Format::Sec
    } else {
        Format::UwpLines
    }
}

/// Parses `text` in the format [`detect`] picks.
pub fn parse_any(text: &str) -> SectorFile {
    parse_as(detect(text), text)
}

/// Parses `text` as `format`.
pub fn parse_as(format: Format, text: &str) -> SectorFile {
    match format {
        Format::T5Tab => parse_t5_tab(text),
        Format::Sec => parse_sec(text),
        Format::Xml => parse_sector_xml(text),
        Format::UwpLines => parse_uwp_lines(text),
    }
}

/// Writes `file` in `format`.
pub fn write(format: Format, file: &SectorFile) -> String {
    match format {
        Format::T5Tab => write_t5_tab(file),
        Format::Sec => write_sec(file),
        Format::Xml => write_sector_xml(file),
        Format::UwpLines => write_uwp_lines(file),
    }
}

/// Reads "# Sector: Name", "# Abbreviation: Xxxx" and "# Subsector A: Name"
/// comment lines (written by this crate's SEC and UWP-lines writers, and
/// the subsector form by Travellermap). Other comments are ignored.
pub(crate) fn read_comment(line: &str, out: &mut SectorFile) {
    let body = line.trim().trim_start_matches('#').trim();
    let Some((key, value)) = body.split_once(':') else {
        return;
    };
    let (key, value) = (key.trim(), value.trim());
    if value.is_empty() {
        return;
    }
    if key.eq_ignore_ascii_case("sector") || key.eq_ignore_ascii_case("name") {
        out.name = Some(value.to_string());
    } else if key.eq_ignore_ascii_case("abbreviation") {
        out.abbreviation = Some(value.to_string());
    } else if key.len() == 11 && key.is_char_boundary(10) {
        let (word, letter) = key.split_at(10);
        if word.eq_ignore_ascii_case("subsector ") {
            if let Some(c) = subsector_letter(letter) {
                out.subsectors.insert(c, value.to_string());
            }
        }
    }
}

/// 'A'..'P' (either case) from a one-letter string.
pub(crate) fn subsector_letter(s: &str) -> Option<char> {
    let mut it = s.trim().chars();
    let c = it.next()?.to_ascii_uppercase();
    (it.next().is_none() && ('A'..='P').contains(&c)).then_some(c)
}

/// The comment block the SEC and UWP-lines writers put first.
pub(crate) fn write_comments(file: &SectorFile, out: &mut String) {
    if let Some(n) = &file.name {
        out.push_str(&format!("# Sector: {n}\n"));
    }
    if let Some(a) = &file.abbreviation {
        out.push_str(&format!("# Abbreviation: {a}\n"));
    }
    for (k, v) in &file.subsectors {
        out.push_str(&format!("# Subsector {k}: {v}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners() {
        assert_eq!(hex_to_xy("1910"), Some((19, 10)));
        assert_eq!(hex_to_xy("19x0"), None);
        assert_eq!(xy_to_hex(3, 7), "0307");
        let at = |h: &str| hex_to_xy(h).map(|(x, y)| subsector_of(x, y));
        assert_eq!(at("0101"), Some('A'));
        assert_eq!(at("0810"), Some('A'));
        assert_eq!(at("0901"), Some('B'));
        assert_eq!(at("0811"), Some('E'));
        assert_eq!(at("3201"), Some('D'));
        assert_eq!(at("0140"), Some('M'));
        assert_eq!(at("3240"), Some('P'));
        assert_eq!(subsector_of(0, 0), 'A');
        assert_eq!(subsector_of(99, 99), 'P');
    }

    #[test]
    fn every_hex_round_trips() {
        for x in 1..=32 {
            for y in 1..=40 {
                let h = xy_to_hex(x, y);
                assert_eq!(hex_to_xy(&h), Some((x, y)));
                let s = subsector_of(x, y);
                let i = (s as u32) - ('A' as u32);
                assert_eq!((i % 4, i / 4), ((x - 1) / 8, (y - 1) / 10));
            }
        }
    }

    #[test]
    fn comments() {
        let mut f = SectorFile::default();
        read_comment("# Sector: Far Shelf", &mut f);
        read_comment("# Abbreviation: Fsh", &mut f);
        read_comment("# Subsector c: Low Road", &mut f);
        read_comment("# Subsector Q: nowhere", &mut f);
        read_comment("# Generated by https://example.org", &mut f);
        assert_eq!(f.name.as_deref(), Some("Far Shelf"));
        assert_eq!(f.abbreviation.as_deref(), Some("Fsh"));
        assert_eq!(f.subsectors.len(), 1);
        assert_eq!(f.subsectors[&'C'], "Low Road");
    }

    #[test]
    fn serde_round_trip() {
        let mut f = SectorFile {
            name: Some("X".into()),
            ..Default::default()
        };
        f.subsectors.insert('B', "Bee".into());
        f.worlds.push(WorldLine {
            zone: Some('A'),
            ..Default::default()
        });
        let s = serde_json::to_string(&f).unwrap();
        let back: SectorFile = serde_json::from_str(&s).unwrap();
        assert_eq!(back, f);
    }
}
