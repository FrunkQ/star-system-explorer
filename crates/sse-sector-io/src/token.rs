//! Tokens and the shapes every format recognises (UWP, hex, PBG, stars ...).
//!
//! Token positions are counted in characters from the start of the line, so
//! fixed columns still line up when a name holds non-ASCII letters.

use crate::WorldLine;

/// A whitespace-separated token with its character columns `[start, end)`.
///
/// A token that opens with `{`, `(` or `[` runs to the matching closer even
/// across spaces, so `{ +1 }` and `(Droyne Ancients)` stay one token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tok {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

fn closer(c: char) -> Option<char> {
    match c {
        '{' => Some('}'),
        '(' => Some(')'),
        '[' => Some(']'),
        _ => None,
    }
}

pub(crate) fn tokenize(line: &str) -> Vec<Tok> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if let Some(close) = closer(chars[i]) {
            if let Some(off) = chars[i + 1..].iter().position(|&c| c == close) {
                i += off + 2;
            }
        }
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        out.push(Tok {
            text: chars[start..i].iter().collect(),
            start,
            end: i,
        });
    }
    out
}

/// Splits a free-text field (remarks, stars) into token texts.
pub(crate) fn words(s: &str) -> Vec<String> {
    tokenize(s).into_iter().map(|t| t.text).collect()
}

/// Collapses runs of whitespace to single spaces and trims.
pub(crate) fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn join(toks: &[Tok]) -> String {
    collapse(
        &toks
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

pub(crate) fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// Extended hex: 0-9 and A-Z without I and O.
pub fn is_ehex(c: char) -> bool {
    c.is_ascii_digit() || (c.is_ascii_uppercase() && c != 'I' && c != 'O')
}

/// Seven ehex digits, '-', one ehex digit. '?' stands for an unknown digit.
pub fn is_uwp(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    c.len() == 9
        && c[7] == '-'
        && c.iter()
            .enumerate()
            .all(|(i, &ch)| i == 7 || is_ehex(ch) || ch == '?')
}

/// Four ASCII digits, as written ("0101".."3240").
pub fn is_hex(s: &str) -> bool {
    s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit())
}

/// Population multiplier, belts, gas giants: a digit then two ehex digits.
pub(crate) fn is_pbg(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    c.len() == 3
        && (c[0].is_ascii_digit() || c[0] == '?')
        && c[1..].iter().all(|&ch| is_ehex(ch) || ch == '?')
}

/// One to four capital letters: a base field ("N", "NS", legacy "A").
pub(crate) fn is_base_token(s: &str) -> bool {
    (1..=4).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_uppercase())
}

/// A trade code or other remark: "Ri", "RsA", "(Droyne)", "[Sophont]", "O:1910".
pub(crate) fn is_remark(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    if c.is_empty() {
        return false;
    }
    if c[0] == '(' || c[0] == '[' {
        return true;
    }
    if c.len() >= 2 && c[0].is_ascii_uppercase() && c[1].is_ascii_lowercase() {
        return true;
    }
    // "O:1910", "Mr:..." style owned / military-rule remarks.
    let caps = c.iter().take_while(|ch| ch.is_ascii_alphabetic()).count();
    (1..=2).contains(&caps) && c.get(caps) == Some(&':') && c[0].is_ascii_uppercase()
}

/// Allegiance code: two to four letters or digits, starting with a letter,
/// or a run of two to four dashes. Never a spectral type.
pub(crate) fn is_allegiance(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    if !(2..=4).contains(&c.len()) {
        return false;
    }
    if c.iter().all(|&ch| ch == '-') {
        return true;
    }
    c[0].is_ascii_alphabetic()
        && c.iter().all(|ch| ch.is_ascii_alphanumeric())
        && !is_spectral(s)
        && compact_star(s).is_none()
}

const LUMINOSITY: [&str; 9] = ["Ia", "Ib", "II", "III", "IV", "V", "VI", "VII", "D"];

/// "G2", "K7", "M0.5": spectral class letter, digit, optional decimal.
fn is_spectral(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    let head = !c.is_empty() && "OBAFGKMLTY".contains(c[0]);
    match c.len() {
        2 => head && c[1].is_ascii_digit(),
        4 => head && c[1].is_ascii_digit() && c[2] == '.' && c[3].is_ascii_digit(),
        _ => false,
    }
}

/// "M0V", "G2IV": a spectral type with its luminosity class attached.
fn compact_star(s: &str) -> Option<()> {
    for split in [2usize, 4] {
        if s.len() > split && s.is_char_boundary(split) {
            let (a, b) = s.split_at(split);
            if is_spectral(a) && LUMINOSITY.contains(&b) {
                return Some(());
            }
        }
    }
    None
}

/// White dwarfs ("D", "DA", "DB3") and brown dwarfs ("BD").
fn is_remnant(s: &str) -> bool {
    if s == "BD" || s == "D" {
        return true;
    }
    let c: Vec<char> = s.chars().collect();
    if c.first() != Some(&'D') || c.len() > 4 {
        return false;
    }
    let letters = c[1..]
        .iter()
        .take_while(|ch| "ABCOQZX".contains(**ch))
        .count();
    let rest = &c[1 + letters..];
    (1..=2).contains(&letters) && (rest.is_empty() || (rest.len() == 1 && rest[0].is_ascii_digit()))
}

/// The star starting at token `i`, with the number of tokens it uses.
/// A spectral type takes the following luminosity class with it, so
/// "G2 V" is one star and "M8 D" (the legacy "M8 dwarf") is one star.
pub(crate) fn star_at(toks: &[Tok], i: usize) -> Option<(String, usize)> {
    let t = toks.get(i)?.text.as_str();
    if is_spectral(t) {
        if let Some(next) = toks.get(i + 1) {
            if LUMINOSITY.contains(&next.text.as_str()) {
                return Some((format!("{t} {}", next.text), 2));
            }
        }
        return Some((t.to_string(), 1));
    }
    if compact_star(t).is_some() || is_remnant(t) {
        return Some((t.to_string(), 1));
    }
    None
}

/// A ruler line of dash groups; returns each group's columns `[start, end)`.
pub(crate) fn ruler(line: &str) -> Option<Vec<(usize, usize)>> {
    if line.trim().is_empty() || !line.chars().all(|c| c == '-' || c == ' ') {
        return None;
    }
    let groups: Vec<(usize, usize)> = tokenize(line).iter().map(|t| (t.start, t.end)).collect();
    (groups.len() >= 3).then_some(groups)
}

/// `{x}` / `(x)` / `[x]`: the trimmed inside, `None` when empty.
/// Returns `None` when `s` is not wrapped in this pair at all.
pub(crate) fn delimited(s: &str, open: char, close: char) -> Option<Option<String>> {
    let inner = s.strip_prefix(open)?.strip_suffix(close)?;
    let inner = inner.trim();
    Some((!inner.is_empty()).then(|| inner.to_string()))
}

/// Strips the delimiters when present; a field may be written either way.
pub(crate) fn unwrap_field(s: &str, open: char, close: char) -> Option<String> {
    let s = s.trim();
    match delimited(s, open, close) {
        Some(v) => v,
        None if s.is_empty() || s == "-" => None,
        None => Some(s.to_string()),
    }
}

fn is_unsigned(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_signed(s: &str) -> bool {
    let body = s.strip_prefix(['-', '+']).unwrap_or(s);
    is_unsigned(body)
}

pub(crate) fn decode_zone(s: &str, msgs: &mut Vec<String>) -> Option<char> {
    match s.trim() {
        "A" => Some('A'),
        "R" => Some('R'),
        "" | "-" | "G" => None,
        other => {
            msgs.push(format!("zone '{other}' not recognised; read as green"));
            None
        }
    }
}

/// Legacy one-letter base codes, as documented for the SEC format.
/// Only letters whose meaning is certain are expanded; others are kept.
fn legacy_base(c: char) -> Option<&'static [&'static str]> {
    Some(match c {
        'A' => &["N", "S"],
        'B' => &["N", "W"],
        'C' => &["C"],
        'D' => &["D"],
        'E' => &["E"],
        'K' => &["K"],
        'M' => &["M"],
        'N' => &["N"],
        'S' => &["S"],
        'W' => &["W"],
        _ => return None,
    })
}

/// A base field. A single letter (legacy files, loose pastes) goes through
/// the legacy table; several letters are T5 codes, one per letter.
pub(crate) fn decode_bases(s: &str, legacy: bool, msgs: &mut Vec<String>) -> Vec<String> {
    let s = s.trim();
    if s.is_empty() || s == "-" {
        return Vec::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if legacy && chars.len() == 1 {
        return match legacy_base(chars[0]) {
            Some(list) => list.iter().map(|b| b.to_string()).collect(),
            None => {
                msgs.push(format!(
                    "legacy base code '{s}' kept as written (its T5 meaning is not certain)"
                ));
                vec![s.to_string()]
            }
        };
    }
    if !chars.iter().all(|c| c.is_ascii_alphabetic()) {
        msgs.push(format!(
            "base field '{s}' has characters that are not base letters"
        ));
    }
    chars.iter().map(|c| c.to_string()).collect()
}

/// The legacy SEC spelling of a base list: A and B for the two pairs,
/// a single letter as itself, anything else as T5 letters.
pub(crate) fn encode_bases_legacy(bases: &[String]) -> String {
    match bases
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["N", "S"] => "A".to_string(),
        ["N", "W"] => "B".to_string(),
        _ => bases.concat(),
    }
}

/// Everything after the PBG: allegiance, stars, then the T5 extensions.
pub(crate) fn parse_after_pbg(toks: &[Tok], w: &mut WorldLine, msgs: &mut Vec<String>) {
    let mut i = 0;
    if let Some(t) = toks.first() {
        if t.text == "-" {
            i = 1;
        } else if star_at(toks, 0).is_none() && is_allegiance(&t.text) {
            w.allegiance = t.text.clone();
            i = 1;
        }
    }
    parse_stars_and_tail(toks, i, w, msgs);
}

pub(crate) fn parse_stars_and_tail(
    toks: &[Tok],
    mut i: usize,
    w: &mut WorldLine,
    msgs: &mut Vec<String>,
) {
    while let Some((star, used)) = star_at(toks, i) {
        w.stars.push(star);
        i += used;
    }
    parse_tail(&toks[i.min(toks.len())..], w, msgs);
}

/// `{Ix} (Ex) [Cx]` by shape, then nobility, W and RU in that order,
/// with "-" for an empty one.
fn parse_tail(toks: &[Tok], w: &mut WorldLine, msgs: &mut Vec<String>) {
    let mut pos = 0u8;
    for t in toks {
        let s = t.text.as_str();
        if pos == 0 {
            if let Some(v) = delimited(s, '{', '}') {
                w.importance = v;
                continue;
            }
            if let Some(v) = delimited(s, '(', ')') {
                w.economic = v;
                continue;
            }
            if let Some(v) = delimited(s, '[', ']') {
                w.cultural = v;
                continue;
            }
        }
        let ok = match pos {
            0 if s == "-" => {
                pos = 1;
                true
            }
            0 if s.bytes().all(|b| b.is_ascii_alphabetic()) => {
                w.nobility = Some(s.to_string());
                pos = 1;
                true
            }
            0 | 1 if is_unsigned(s) => match s.parse::<u32>() {
                Ok(n) => {
                    w.worlds = Some(n);
                    pos = 2;
                    true
                }
                Err(_) => false,
            },
            1 if s == "-" => {
                pos = 2;
                true
            }
            2 if s == "-" => {
                pos = 3;
                true
            }
            0..=2 if is_signed(s) => {
                w.ru = Some(s.to_string());
                pos = 3;
                true
            }
            _ => false,
        };
        if !ok {
            msgs.push(format!("unrecognised token '{s}' ignored"));
        }
    }
}

/// The T5 extension columns as SEC and UWP lines write them, or `None`
/// when the world has none of them.
pub(crate) fn write_tail(w: &WorldLine) -> Option<String> {
    let any = w.importance.is_some()
        || w.economic.is_some()
        || w.cultural.is_some()
        || w.nobility.is_some()
        || w.worlds.is_some()
        || w.ru.is_some();
    if !any {
        return None;
    }
    let ix = w
        .importance
        .as_deref()
        .map_or("{}".to_string(), |v| format!("{{ {v} }}"));
    let ex = w
        .economic
        .as_deref()
        .map_or("()".to_string(), |v| format!("({v})"));
    let cx = w
        .cultural
        .as_deref()
        .map_or("[]".to_string(), |v| format!("[{v}]"));
    let nob = w.nobility.clone().unwrap_or_else(|| "-".to_string());
    let wc = w.worlds.map_or("-".to_string(), |n| n.to_string());
    let ru = w.ru.clone().unwrap_or_else(|| "-".to_string());
    Some(format!("{ix} {ex} {cx} {nob} {wc} {ru}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        assert!(is_uwp("A867975-9"));
        assert!(is_uwp("X???000-0"));
        assert!(!is_uwp("A8679I5-9"), "I is not ehex");
        assert!(!is_uwp("A867975+9"));
        assert!(is_pbg("703") && !is_pbg("70") && !is_pbg("Ri3"));
        assert!(
            is_remark("Ri") && is_remark("RsA") && is_remark("(Droyne)W") && is_remark("O:1910")
        );
        assert!(!is_remark("NS") && !is_remark("814"));
        assert!(is_allegiance("Im") && is_allegiance("ImDd") && is_allegiance("--"));
        assert!(!is_allegiance("G2") && !is_allegiance("M0V") && !is_allegiance("I"));
    }

    #[test]
    fn grouped_tokens() {
        let t = words("Ri { +1 } (A46+2) [1716] (Droyne Ancients)");
        assert_eq!(
            t,
            ["Ri", "{ +1 }", "(A46+2)", "[1716]", "(Droyne Ancients)"]
        );
        // An unclosed bracket ends at whitespace like any token.
        assert_eq!(words("(open close"), ["(open", "close"]);
    }

    #[test]
    fn stars() {
        let toks = tokenize("G2 V M8 D BD D DA K7V M0 III F5.5 IV");
        let mut i = 0;
        let mut got = Vec::new();
        while let Some((s, n)) = star_at(&toks, i) {
            got.push(s);
            i += n;
        }
        assert_eq!(
            got,
            ["G2 V", "M8 D", "BD", "D", "DA", "K7V", "M0 III", "F5.5 IV"]
        );
    }
}
