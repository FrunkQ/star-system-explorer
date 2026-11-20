//! Travellermap's tab-delimited "T5 Second Survey" format (what
//! `/data/<sector>/<subsector>/tab` returns). The header row names the
//! columns; they may come in any order and any may be missing.

use crate::token::{
    decode_bases, decode_zone, is_hex, is_uwp, star_at, strip_bom, tokenize, unwrap_field, words,
};
use crate::{hex_in_range, subsector_of, SectorFile, WorldLine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Col {
    Sector,
    Ss,
    Hex,
    Name,
    Uwp,
    Bases,
    Remarks,
    Zone,
    Pbg,
    Allegiance,
    Stars,
    Ix,
    Ex,
    Cx,
    Nobility,
    W,
    Ru,
}

fn column(name: &str) -> Option<Col> {
    let n = name.trim().to_ascii_lowercase();
    Some(match n.as_str() {
        "sector" => Col::Sector,
        "ss" | "subsector" => Col::Ss,
        "hex" => Col::Hex,
        "name" => Col::Name,
        "uwp" => Col::Uwp,
        "bases" | "base" | "b" => Col::Bases,
        "remarks" => Col::Remarks,
        "zone" | "z" => Col::Zone,
        "pbg" => Col::Pbg,
        "allegiance" | "a" => Col::Allegiance,
        "stars" | "stellar" => Col::Stars,
        "{ix}" | "ix" | "importance" => Col::Ix,
        "(ex)" | "ex" | "economic" => Col::Ex,
        "[cx]" | "cx" | "cultural" => Col::Cx,
        "nobility" | "n" => Col::Nobility,
        "w" | "worlds" => Col::W,
        "ru" => Col::Ru,
        _ => return None,
    })
}

const ORDER: [(Col, &str); 17] = [
    (Col::Sector, "Sector"),
    (Col::Ss, "SS"),
    (Col::Hex, "Hex"),
    (Col::Name, "Name"),
    (Col::Uwp, "UWP"),
    (Col::Bases, "Bases"),
    (Col::Remarks, "Remarks"),
    (Col::Zone, "Zone"),
    (Col::Pbg, "PBG"),
    (Col::Allegiance, "Allegiance"),
    (Col::Stars, "Stars"),
    (Col::Ix, "{Ix}"),
    (Col::Ex, "(Ex)"),
    (Col::Cx, "[Cx]"),
    (Col::Nobility, "Nobility"),
    (Col::W, "W"),
    (Col::Ru, "RU"),
];

fn opt(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty() && s != "-").then(|| s.to_string())
}

pub fn parse_t5_tab(text: &str) -> SectorFile {
    let mut out = SectorFile::default();
    let mut header: Option<Vec<Option<Col>>> = None;
    let mut sector_col: Vec<String> = Vec::new();
    for (n, line) in strip_bom(text).lines().enumerate() {
        let ln = n + 1;
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        let Some(cols) = &header else {
            let cols: Vec<Option<Col>> = cells.iter().map(|c| column(c)).collect();
            for (c, cell) in cols.iter().zip(&cells) {
                if c.is_none() && !cell.trim().is_empty() {
                    out.warn(
                        ln,
                        format!("column '{}' not known; ignored", cell.trim()),
                        line,
                    );
                }
            }
            if !cols.contains(&Some(Col::Uwp)) {
                out.warn(ln, "header has no UWP column; no worlds read", line);
            }
            header = Some(cols);
            continue;
        };
        let get = |want: Col| -> &str {
            cols.iter()
                .position(|c| *c == Some(want))
                .and_then(|i| cells.get(i))
                .map_or("", |s| s.trim())
        };
        let uwp = get(Col::Uwp);
        if uwp.is_empty() {
            out.warn(ln, "no UWP in this row; row skipped", line);
            continue;
        }
        let mut msgs = Vec::new();
        if !is_uwp(uwp) {
            msgs.push(format!(
                "UWP '{uwp}' is not seven ehex digits, '-', one digit; kept as written"
            ));
        }
        let mut w = WorldLine {
            hex: get(Col::Hex).to_string(),
            name: crate::token::collapse(get(Col::Name)),
            uwp: uwp.to_string(),
            remarks: words(get(Col::Remarks)),
            pbg: get(Col::Pbg).to_string(),
            allegiance: get(Col::Allegiance).to_string(),
            importance: unwrap_field(get(Col::Ix), '{', '}'),
            economic: unwrap_field(get(Col::Ex), '(', ')'),
            cultural: unwrap_field(get(Col::Cx), '[', ']'),
            nobility: opt(get(Col::Nobility)),
            ru: opt(get(Col::Ru)),
            raw: line.to_string(),
            ..Default::default()
        };
        if !w.hex.is_empty() && (!is_hex(&w.hex) || !hex_in_range(&w.hex)) {
            msgs.push(format!(
                "hex '{}' is not a hex inside a 32 by 40 sector",
                w.hex
            ));
        }
        w.bases = decode_bases(get(Col::Bases), false, &mut msgs);
        w.zone = decode_zone(get(Col::Zone), &mut msgs);
        if let Some(wc) = opt(get(Col::W)) {
            match wc.parse::<u32>() {
                Ok(v) => w.worlds = Some(v),
                Err(_) => msgs.push(format!("W '{wc}' is not a count; ignored")),
            }
        }
        let stars = tokenize(get(Col::Stars));
        let mut i = 0;
        while i < stars.len() {
            match star_at(&stars, i) {
                Some((s, used)) => {
                    w.stars.push(s);
                    i += used;
                }
                None => {
                    msgs.push(format!("star '{}' not recognised; ignored", stars[i].text));
                    i += 1;
                }
            }
        }
        sector_col.push(get(Col::Sector).to_string());
        out.warn_all(ln, msgs, line);
        out.worlds.push(w);
    }
    if let Some(first) = sector_col.first() {
        if !first.is_empty() && sector_col.iter().all(|s| s == first) {
            out.abbreviation = Some(first.clone());
        }
    }
    out
}

/// All seventeen columns in Travellermap's order. Sector is the file's
/// abbreviation; SS is worked out from the hex.
pub fn write_t5_tab(file: &SectorFile) -> String {
    let mut out = ORDER.iter().map(|(_, n)| *n).collect::<Vec<_>>().join("\t");
    out.push('\n');
    let abbr = file.abbreviation.clone().unwrap_or_default();
    for w in &file.worlds {
        let cells: Vec<String> = ORDER
            .iter()
            .map(|(c, _)| match c {
                Col::Sector => abbr.clone(),
                Col::Ss => w
                    .xy()
                    .map(|(x, y)| subsector_of(x, y).to_string())
                    .unwrap_or_default(),
                Col::Hex => w.hex.clone(),
                Col::Name => w.name.clone(),
                Col::Uwp => w.uwp.clone(),
                Col::Bases => w.bases.concat(),
                Col::Remarks => w.remarks.join(" "),
                Col::Zone => w.zone.map(String::from).unwrap_or_default(),
                Col::Pbg => w.pbg.clone(),
                Col::Allegiance => w.allegiance.clone(),
                Col::Stars => w.stars.join(" "),
                Col::Ix => w
                    .importance
                    .as_deref()
                    .map(|v| format!("{{ {v} }}"))
                    .unwrap_or_default(),
                Col::Ex => w
                    .economic
                    .as_deref()
                    .map(|v| format!("({v})"))
                    .unwrap_or_default(),
                Col::Cx => w
                    .cultural
                    .as_deref()
                    .map(|v| format!("[{v}]"))
                    .unwrap_or_default(),
                Col::Nobility => w.nobility.clone().unwrap_or_default(),
                Col::W => w.worlds.map(|n| n.to_string()).unwrap_or_default(),
                Col::Ru => w.ru.clone().unwrap_or_default(),
            })
            .collect();
        out.push_str(&cells.join("\t"));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_in_any_order() {
        let text = "Name\tUWP\tHex\tStars\tZone\n Far  Away \tA867975-9\t0102\tG2 V M3 V\tR\nGap\t\t0103\t\t\n";
        let f = parse_t5_tab(text);
        assert_eq!(f.worlds.len(), 1);
        let w = &f.worlds[0];
        assert_eq!(
            (w.name.as_str(), w.hex.as_str(), w.zone),
            ("Far Away", "0102", Some('R'))
        );
        assert_eq!(w.stars, ["G2 V", "M3 V"]);
        assert_eq!(f.warnings.len(), 1, "{:?}", f.warnings);
        assert_eq!(f.warnings[0].line, 3);
    }

    #[test]
    fn extensions() {
        let text = "Hex\tUWP\t{Ix}\t(Ex)\t[Cx]\tNobility\tW\tRU\tBases\n0101\tA867975-9\t{ +1 }\t(A46+2)\t[1716]\tBcC\t7\t-12\tNS\n";
        let w = &parse_t5_tab(text).worlds[0];
        assert_eq!(w.importance.as_deref(), Some("+1"));
        assert_eq!(w.economic.as_deref(), Some("A46+2"));
        assert_eq!(w.cultural.as_deref(), Some("1716"));
        assert_eq!(w.nobility.as_deref(), Some("BcC"));
        assert_eq!((w.worlds, w.ru.as_deref()), (Some(7), Some("-12")));
        assert_eq!(w.bases, ["N", "S"]);
    }
}
