//! Loose "one world per line" text, as a GM pastes it.
//!
//! The only thing a line must have is a UWP. Before it: an optional hex
//! (first or last) and the name. After it, in this order and each optional:
//! bases, remarks, zone, PBG, allegiance (only after a PBG), stars, then the
//! T5 extensions `{Ix} (Ex) [Cx]` and nobility, W, RU.

use crate::token::{
    decode_bases, decode_zone, is_base_token, is_hex, is_pbg, is_remark, is_uwp, join,
    parse_after_pbg, parse_stars_and_tail, star_at, strip_bom, tokenize, write_tail, Tok,
};
use crate::{hex_in_range, read_comment, write_comments, SectorFile, WorldLine};

pub fn parse_uwp_lines(text: &str) -> SectorFile {
    let mut out = SectorFile::default();
    for (n, line) in strip_bom(text).lines().enumerate() {
        let ln = n + 1;
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') {
            read_comment(t, &mut out);
            continue;
        }
        let toks = tokenize(line);
        let Some(u) = toks.iter().position(|t| is_uwp(&t.text)) else {
            out.warn(ln, "no UWP found; line skipped", line);
            continue;
        };
        let mut msgs = Vec::new();
        let mut w = WorldLine {
            uwp: toks[u].text.clone(),
            raw: line.to_string(),
            ..Default::default()
        };
        let pre = &toks[..u];
        let name_toks = match pre {
            [first, rest @ ..] if is_hex(&first.text) => {
                w.hex.clone_from(&first.text);
                rest
            }
            [rest @ .., last] if is_hex(&last.text) => {
                w.hex.clone_from(&last.text);
                rest
            }
            _ => pre,
        };
        w.name = join(name_toks);
        if !w.hex.is_empty() && !hex_in_range(&w.hex) {
            msgs.push(format!("hex {} is outside a 32 by 40 sector", w.hex));
        }
        parse_post(&toks[u + 1..], &mut w, &mut msgs);
        out.warn_all(ln, msgs, line);
        out.worlds.push(w);
    }
    out
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Stage {
    Bases,
    Remarks,
    ZoneDone,
}

fn is_zone_letter(s: &str) -> bool {
    matches!(s, "A" | "R" | "G" | "-")
}

fn parse_post(post: &[Tok], w: &mut WorldLine, msgs: &mut Vec<String>) {
    let mut stage = Stage::Bases;
    let mut i = 0;
    while i < post.len() {
        let s = post[i].text.as_str();
        let next = post.get(i + 1).map(|t| t.text.as_str());
        if is_pbg(s) {
            w.pbg = s.to_string();
            parse_after_pbg(&post[i + 1..], w, msgs);
            return;
        }
        if stage == Stage::Bases {
            if s == "-" {
                stage = Stage::Remarks;
                i += 1;
                continue;
            }
            if is_base_token(s) {
                // A lone A, R or G just before the PBG (or at the end) is a
                // zone; anywhere else straight after the UWP it is a base.
                if matches!(s, "A" | "R" | "G") && next.is_none_or(is_pbg) {
                    w.zone = decode_zone(s, msgs);
                    stage = Stage::ZoneDone;
                } else {
                    w.bases = decode_bases(s, true, msgs);
                    stage = Stage::Remarks;
                }
                i += 1;
                continue;
            }
        }
        if stage != Stage::ZoneDone && is_zone_letter(s) {
            w.zone = decode_zone(s, msgs);
            stage = Stage::ZoneDone;
            i += 1;
            continue;
        }
        if stage != Stage::ZoneDone && is_remark(s) {
            w.remarks.push(s.to_string());
            stage = Stage::Remarks;
            i += 1;
            continue;
        }
        if star_at(post, i).is_some() {
            parse_stars_and_tail(post, i, w, msgs);
            return;
        }
        msgs.push(format!("unrecognised token '{s}' ignored"));
        i += 1;
    }
}

/// One world per line: `hex name UWP bases remarks zone PBG allegiance
/// stars` and the T5 extensions when the world has any. An empty
/// allegiance is written "-" when something follows it. Without a PBG the
/// allegiance cannot be written (loose lines read it only after a PBG).
pub fn write_uwp_lines(file: &SectorFile) -> String {
    let mut out = String::new();
    write_comments(file, &mut out);
    for w in &file.worlds {
        let mut parts: Vec<String> = Vec::new();
        if !w.hex.is_empty() {
            parts.push(w.hex.clone());
        }
        if !w.name.is_empty() {
            parts.push(w.name.clone());
        }
        parts.push(w.uwp.clone());
        if !w.bases.is_empty() {
            parts.push(w.bases.concat());
        }
        parts.extend(w.remarks.iter().cloned());
        if let Some(z) = w.zone {
            parts.push(z.to_string());
        }
        let tail = write_tail(w);
        if !w.pbg.is_empty() {
            parts.push(w.pbg.clone());
            if !w.allegiance.is_empty() {
                parts.push(w.allegiance.clone());
            } else if !w.stars.is_empty() || tail.is_some() {
                parts.push("-".to_string());
            }
        }
        parts.extend(w.stars.iter().cloned());
        parts.extend(tail);
        out.push_str(&parts.join(" "));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_lines() {
        let f = parse_uwp_lines(
            "Somewhere A867975-9\n1910 Other Place B544620-8\nThird 0203 C000000-0\n",
        );
        assert!(f.warnings.is_empty(), "{:?}", f.warnings);
        assert_eq!(f.worlds.len(), 3);
        assert_eq!(
            (f.worlds[0].hex.as_str(), f.worlds[0].name.as_str()),
            ("", "Somewhere")
        );
        assert_eq!(
            (f.worlds[1].hex.as_str(), f.worlds[1].name.as_str()),
            ("1910", "Other Place")
        );
        assert_eq!(
            (f.worlds[2].hex.as_str(), f.worlds[2].name.as_str()),
            ("0203", "Third")
        );
    }

    #[test]
    fn zone_or_base() {
        let f =
            parse_uwp_lines("X A867975-9 A 703 Im\nY A867975-9 A Ri 703\nZ A867975-9 N A 703\n");
        assert_eq!(f.worlds[0].zone, Some('A'));
        assert!(f.worlds[0].bases.is_empty());
        assert_eq!(f.worlds[1].bases, ["N", "S"]);
        assert_eq!(f.worlds[1].zone, None);
        assert_eq!(f.worlds[2].bases, ["N"]);
        assert_eq!(f.worlds[2].zone, Some('A'));
    }

    #[test]
    fn unknown_tokens_warn() {
        let f = parse_uwp_lines("Odd A867975-9 Ri @@ 703 Im G2 V\nno world here\n");
        assert_eq!(f.worlds.len(), 1);
        assert_eq!(f.worlds[0].remarks, ["Ri"]);
        assert_eq!(f.worlds[0].stars, ["G2 V"]);
        assert_eq!(f.warnings.len(), 2);
        assert_eq!(f.warnings[0].line, 1);
        assert_eq!(f.warnings[1].line, 2);
    }
}
