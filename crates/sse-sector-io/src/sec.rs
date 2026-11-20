//! Legacy fixed-column SEC files.
//!
//! Column widths differ between files, so a world line is read by token
//! shape: the name is everything before the hex, the hex is the four digits
//! just before the UWP, the PBG is the first three-digit token after it,
//! and allegiance and stars follow the PBG. Between the UWP and the PBG sit
//! the base letter, the remarks and the zone; a lone letter there is placed
//! by the dash ruler line when the file has one and the line lines up with
//! it, and otherwise by its position (a letter right after the UWP is a
//! base, a lone letter at the end is a zone).

use crate::token::{
    decode_bases, decode_zone, encode_bases_legacy, is_base_token, is_hex, is_pbg, is_uwp, join,
    parse_after_pbg, parse_stars_and_tail, ruler, star_at, strip_bom, tokenize, write_tail, Tok,
};
use crate::{hex_in_range, read_comment, write_comments, SectorFile, WorldLine};

/// Columns of the dash groups that matter: where the UWP starts, and the
/// base and zone spans that follow it.
struct Layout {
    base: (usize, usize),
    zone: (usize, usize),
}

fn layout(groups: &[(usize, usize)], uwp_start: usize) -> Option<Layout> {
    let u = groups.iter().position(|g| g.0 == uwp_start)?;
    if groups.len() < u + 5 {
        return None;
    }
    Some(Layout {
        base: groups[u + 1],
        zone: groups[u + 3],
    })
}

pub fn parse_sec(text: &str) -> SectorFile {
    let mut out = SectorFile::default();
    let mut groups: Option<Vec<(usize, usize)>> = None;
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
        if let Some(g) = ruler(line) {
            groups = Some(g);
            continue;
        }
        let toks = tokenize(line);
        let Some(u) = toks.iter().position(|t| is_uwp(&t.text)) else {
            if !t.to_ascii_lowercase().contains("uwp") {
                out.warn(ln, "no UWP found; line skipped", line);
            }
            continue;
        };
        let mut msgs = Vec::new();
        let mut w = WorldLine {
            uwp: toks[u].text.clone(),
            raw: line.to_string(),
            ..Default::default()
        };
        if u >= 1 && is_hex(&toks[u - 1].text) {
            w.hex.clone_from(&toks[u - 1].text);
            w.name = join(&toks[..u - 1]);
            if !hex_in_range(&w.hex) {
                msgs.push(format!("hex {} is outside a 32 by 40 sector", w.hex));
            }
        } else {
            w.name = join(&toks[..u]);
            msgs.push("no hex just before the UWP".to_string());
        }
        let post = &toks[u + 1..];
        let lay = groups.as_deref().and_then(|g| layout(g, toks[u].start));
        match post.iter().position(|t| is_pbg(&t.text)) {
            Some(p) => {
                middle(&post[..p], toks[u].end, lay.as_ref(), &mut w, &mut msgs);
                w.pbg.clone_from(&post[p].text);
                parse_after_pbg(&post[p + 1..], &mut w, &mut msgs);
            }
            None => {
                msgs.push("no PBG found".to_string());
                let s = (0..post.len())
                    .find(|&i| star_at(post, i).is_some())
                    .unwrap_or(post.len());
                middle(&post[..s], toks[u].end, lay.as_ref(), &mut w, &mut msgs);
                parse_stars_and_tail(post, s, &mut w, &mut msgs);
            }
        }
        out.warn_all(ln, msgs, line);
        out.worlds.push(w);
    }
    out
}

fn single(t: &Tok) -> bool {
    t.text.chars().count() == 1
}

/// Bases, remarks and zone: everything between the UWP and the PBG.
fn middle(
    m: &[Tok],
    uwp_end: usize,
    lay: Option<&Layout>,
    w: &mut WorldLine,
    msgs: &mut Vec<String>,
) {
    let inside = |t: &Tok, span: (usize, usize)| t.start >= span.0 && t.start < span.1;
    if let Some(l) = lay {
        for t in m {
            if inside(t, l.base) {
                let mut b = decode_bases(&t.text, true, msgs);
                w.bases.append(&mut b);
            } else if inside(t, l.zone) && single(t) {
                w.zone = decode_zone(&t.text, msgs);
            } else {
                w.remarks.push(t.text.clone());
            }
        }
        return;
    }
    let mut rest = m;
    if let Some(first) = rest.first() {
        let base_like = is_base_token(&first.text) || first.text == "-";
        let near = first.start <= uwp_end + 3;
        if base_like && near {
            w.bases = decode_bases(&first.text, true, msgs);
            rest = &rest[1..];
        }
    }
    if let Some(last) = rest.last() {
        if single(last) && !last.text.chars().all(|c| c.is_ascii_lowercase()) {
            w.zone = decode_zone(&last.text, msgs);
            rest = &rest[..rest.len() - 1];
        }
    }
    w.remarks.extend(rest.iter().map(|t| t.text.clone()));
}

/// Aligned columns under a header and a dash ruler. Bases use the legacy
/// letters A (N + S) and B (N + W) where they fit, else T5 letters. Files
/// whose worlds carry T5 extensions get them as extra columns after the
/// stars (legacy SEC has no place for them).
pub fn write_sec(file: &SectorFile) -> String {
    let mut out = String::new();
    write_comments(file, &mut out);
    let width = |f: &dyn Fn(&WorldLine) -> String, min: usize| {
        file.worlds
            .iter()
            .map(|w| f(w).chars().count())
            .max()
            .unwrap_or(0)
            .max(min)
    };
    let base = |w: &WorldLine| encode_bases_legacy(&w.bases);
    let remarks = |w: &WorldLine| w.remarks.join(" ");
    let alleg = |w: &WorldLine| {
        if w.allegiance.is_empty() {
            "-".to_string()
        } else {
            w.allegiance.clone()
        }
    };
    let stars = |w: &WorldLine| w.stars.join(" ");
    let nw = width(&|w| w.name.clone(), 4);
    let pw = width(&|w| w.pbg.clone(), 3);
    let bw = width(&base, 1);
    let rw = width(&remarks, 7);
    let aw = width(&alleg, 2);
    let sw = width(&stars, 5);
    let tails: Vec<Option<String>> = file.worlds.iter().map(write_tail).collect();
    let any_tail = tails.iter().any(Option::is_some);

    let mut head = format!(
        "{:<nw$} {:<4} {:<9} {:<bw$} {:<rw$} Z {:<pw$} {:<aw$} {:<sw$}",
        "Name", "Hex", "UWP", "B", "Remarks", "PBG", "Al", "Stars"
    );
    if any_tail {
        head.push_str(" {Ix} (Ex) [Cx] Nobility W RU");
    }
    out.push_str(head.trim_end());
    out.push('\n');
    let dash = |n: usize| "-".repeat(n);
    out.push_str(&format!(
        "{} ---- --------- {} {} - {} {} {}\n",
        dash(nw),
        dash(bw),
        dash(rw),
        dash(pw),
        dash(aw),
        dash(sw)
    ));
    for (w, tail) in file.worlds.iter().zip(&tails) {
        let zone = w.zone.unwrap_or(' ');
        let mut row = format!(
            "{:<nw$} {:<4} {:<9} {:<bw$} {:<rw$} {zone} {:<pw$} {:<aw$} {:<sw$}",
            w.name,
            w.hex,
            w.uwp,
            base(w),
            remarks(w),
            w.pbg,
            alleg(w),
            stars(w)
        );
        if let Some(t) = tail {
            row.push(' ');
            row.push_str(t);
        }
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The format example from the task notes, used only as a format example.
    #[test]
    fn legacy_example_line() {
        let f = parse_sec(
            "# a comment\nRegina      1910 A788899-C  A Ri Pa Ph An Cp          703 Im G2 V M8 D\n",
        );
        assert!(f.warnings.is_empty(), "{:?}", f.warnings);
        let w = &f.worlds[0];
        assert_eq!(
            (w.name.as_str(), w.hex.as_str(), w.uwp.as_str()),
            ("Regina", "1910", "A788899-C")
        );
        assert_eq!(w.bases, ["N", "S"]);
        assert_eq!(w.remarks, ["Ri", "Pa", "Ph", "An", "Cp"]);
        assert_eq!(
            (w.zone, w.pbg.as_str(), w.allegiance.as_str()),
            (None, "703", "Im")
        );
        assert_eq!(w.stars, ["G2 V", "M8 D"]);
    }

    #[test]
    fn names_with_spaces_and_zone_only() {
        let f = parse_sec(
            "#\nNew Vale Station 0304 B544620-8                      A 503 Na M2 V\nDust 0305 C000000-0 R 001 Na\n",
        );
        let w = &f.worlds[0];
        assert_eq!(w.name, "New Vale Station");
        assert!(w.bases.is_empty());
        assert_eq!(w.zone, Some('A'));
        // A lone letter right next to the UWP sits in the base column; R is
        // not a legacy base letter this crate is sure of, so it is kept and
        // warned about.
        assert_eq!(f.worlds[1].bases, ["R"]);
        assert_eq!(f.worlds[1].zone, None);
        assert_eq!(f.warnings.len(), 1);
    }

    #[test]
    fn unsure_legacy_letter_warns() {
        let f = parse_sec("#\nX 0101 A867975-9 G Ri 703 Im\n");
        assert_eq!(f.worlds[0].bases, ["G"]);
        assert_eq!(f.warnings.len(), 1);
    }
}
