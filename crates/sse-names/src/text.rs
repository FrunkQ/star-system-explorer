//! Text that is safe to draw, and the fold used for the avoid list and for
//! clashes (naming design 4.9, 4.10).
//!
//! - [`check`]: refuses control and direction characters, zero-width and
//!   other invisible characters, private-use and tag characters, markup and
//!   template characters, more than two combining marks on one letter, and
//!   names longer than [`MAX_NAME`] characters.
//! - [`compose`]: the canonical composition (NFC) of Latin, Greek and
//!   Cyrillic letters with their marks, and of Hangul syllables, so "e" plus
//!   a combining acute is stored as "é". Scripts without composed forms pass
//!   unchanged. A full NFC table is not carried: those three scripts and
//!   Hangul are where precomposed letters exist in names.
//! - [`fold`]: lower case, marks removed, look-alike letters mapped to one
//!   (Cyrillic "а" and Latin "a", "0" and "o"), so a look-alike of a name
//!   counts as the same name.

use crate::compose_table::PAIRS;

/// Longest name, in characters.
pub const MAX_NAME: usize = 64;

fn combining(c: char) -> bool {
    matches!(c as u32,
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

fn invisible(c: char) -> bool {
    matches!(c as u32,
        0x0000..=0x001F | 0x007F..=0x009F   // controls
        | 0x00AD                            // soft hyphen
        | 0x034F                            // combining grapheme joiner
        | 0x061C                            // Arabic letter mark
        | 0x115F | 0x1160 | 0x3164 | 0xFFA0 // Hangul fillers
        | 0x180E
        | 0x200B..=0x200F                   // zero widths, LRM, RLM
        | 0x2028..=0x202E                   // separators, embeddings, overrides
        | 0x2060..=0x206F                   // word joiner, isolates
        | 0xFE00..=0xFE0F                   // variation selectors
        | 0xFEFF                            // byte order mark
        | 0xFFF0..=0xFFFF                   // specials
        | 0xE000..=0xF8FF                   // private use
        | 0xE0000..=0xE0FFF                 // tags, variation selectors supplement
        | 0xF0000..=0x10FFFF) // private use planes
}

/// Why a piece of text cannot be a name, or `Ok`.
pub fn check(s: &str) -> Result<(), String> {
    let n = s.chars().count();
    if n == 0 {
        return Err("a name is not empty".into());
    }
    if n > MAX_NAME {
        return Err(format!(
            "{n} characters is longer than the {MAX_NAME} a name holds"
        ));
    }
    let mut marks = 0;
    for c in s.chars() {
        if invisible(c) {
            return Err(format!(
                "U+{:04X} is an invisible or control character, which a name may not hold",
                c as u32
            ));
        }
        if c.is_ascii() && !(c.is_ascii_alphanumeric() || " -'.,()".contains(c)) {
            return Err(format!("{c:?} is not a letter a name may hold"));
        }
        if combining(c) {
            marks += 1;
            if marks > 2 {
                return Err("more than two marks on one letter".into());
            }
        } else {
            marks = 0;
        }
    }
    if s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace) || s.contains("  ") {
        return Err("a name has no spaces at its ends and no double spaces".into());
    }
    Ok(())
}

fn pair(a: char, b: char) -> Option<char> {
    let key = (a as u32, b as u32);
    PAIRS
        .binary_search_by(|p| (p.0, p.1).cmp(&key))
        .ok()
        .and_then(|i| char::from_u32(PAIRS[i].2))
}

/// Canonical composition for Latin, Greek, Cyrillic and Hangul.
pub fn compose(s: &str) -> String {
    let mut out: Vec<char> = Vec::with_capacity(s.len());
    for c in s.chars() {
        if let Some(last) = out.last_mut() {
            if let Some(x) = pair(*last, c) {
                *last = x;
                continue;
            }
            // Hangul: leading consonant + vowel, then + trailing consonant.
            let (l, v) = (*last as u32, c as u32);
            if (0x1100..0x1113).contains(&l) && (0x1161..0x1176).contains(&v) {
                let s = 0xAC00 + ((l - 0x1100) * 21 + (v - 0x1161)) * 28;
                *last = char::from_u32(s).unwrap_or(*last);
                continue;
            }
            if (0xAC00..0xD7A4).contains(&l)
                && (l - 0xAC00) % 28 == 0
                && (0x11A8..0x11C3).contains(&v)
            {
                *last = char::from_u32(l + (v - 0x11A7)).unwrap_or(*last);
                continue;
            }
        }
        out.push(c);
    }
    out.into_iter().collect()
}

fn decompose_into(c: char, out: &mut Vec<char>) {
    match PAIRS.iter().find(|p| p.2 == c as u32) {
        Some(p) => {
            decompose_into(char::from_u32(p.0).unwrap_or(c), out);
            out.push(char::from_u32(p.1).unwrap_or(c));
        }
        None => out.push(c),
    }
}

fn look_alike(c: char) -> &'static str {
    match c {
        // Cyrillic and Greek letters drawn like Latin ones.
        'а' | 'α' | '@' | '4' => "a",
        'в' | 'β' | '8' => "b",
        'с' | 'ς' => "c",
        'е' | 'ε' | 'ё' | '3' => "e",
        'һ' => "h",
        'і' | 'ι' | 'ı' | '1' | '!' | '|' => "i",
        'ј' => "j",
        'к' | 'κ' => "k",
        'ł' => "l",
        'м' => "m",
        'п' | 'η' => "n",
        'о' | 'ο' | 'σ' | '0' | 'ø' => "o",
        'р' | 'ρ' => "p",
        'ѕ' | '5' | '$' => "s",
        'т' | 'τ' | '7' => "t",
        'υ' | 'ս' => "u",
        'ν' => "v",
        'ш' | 'ω' => "w",
        'х' | 'χ' => "x",
        'у' | 'γ' => "y",
        'æ' => "ae",
        'œ' => "oe",
        'ß' => "ss",
        'ð' => "d",
        'þ' => "th",
        '’' | 'ʻ' | 'ʼ' | '`' | '\'' => "",
        _ => "",
    }
}

/// The fold: lower case, marks dropped, look-alikes made one, apostrophes
/// and hyphens dropped, spaces kept as word breaks.
pub fn fold(s: &str) -> String {
    let mut chars = vec![];
    for c in s.chars() {
        decompose_into(c, &mut chars);
    }
    let mut out = String::new();
    for c in chars {
        if combining(c) || c == '-' || c == '.' {
            continue;
        }
        for l in c.to_lowercase() {
            let m = look_alike(l);
            if !m.is_empty() || matches!(l, '’' | 'ʻ' | 'ʼ' | '`' | '\'') {
                out.push_str(m);
            } else {
                out.push(l);
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Two folded names one edit apart (a letter changed, added or dropped):
/// they sound alike at the table.
pub fn near(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a == b {
        return true;
    }
    let (s, l) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    if l.len() - s.len() > 1 || s.len() < 3 {
        return false;
    }
    let mut i = 0;
    while i < s.len() && s[i] == l[i] {
        i += 1;
    }
    if s.len() == l.len() {
        s[i + 1..] == l[i + 1..]
    } else {
        s[i..] == l[i + 1..]
    }
}
