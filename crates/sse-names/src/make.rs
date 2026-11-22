//! Making names (naming design 4.4, 4.5, 4.7, 4.9).
//!
//! A generated name is a keyed draw on its own stream (`names.<layer>.v1`),
//! from the universe seed, the object's address, the layer and the content
//! hash of the scheme. Nothing else reaches it: not the order objects are
//! looked at, the device, or any other pack.

use crate::scheme::{Library, Row, Scheme, Tok, Weighted, MAX_TRIES};
use crate::text;
use serde::Serialize;
use sse_core::{Address, Draws, Seed, Stream};
use std::collections::BTreeMap;

/// What a name is asked for.
#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub seed: &'a Seed,
    pub address: &'a Address,
    /// `designation`, `common`, `local`, or a layer a pack declares.
    pub layer: &'a str,
    pub scheme: &'a str,
    /// `star`, `planet`, `moon`, `peak`, ... (the engine knows none of them).
    pub kind: &'a str,
    /// Position among its siblings from 0 (orbit order): `{roman}`, `{letter}`.
    pub index: u32,
    /// A running number from the caller (a survey's catalogue): `{serial}`.
    pub serial: Option<u64>,
    /// The parent's and the system's names as shown now (for clash checks;
    /// the stored form keeps them as references).
    pub parent: Option<&'a str>,
    pub system: Option<&'a str>,
    /// Facts as words, for `{means:<fact>?}` and `{coord}` (`ra_deg`, `dec_deg`).
    pub facts: &'a BTreeMap<String, String>,
}

/// One drawn part of a name, for the name card.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Part {
    pub text: String,
    /// `word`, `list:<name>`, `means:<id>`, `number`.
    pub from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub means: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// A made name: its stored form and its faces.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Named {
    /// The scheme that made it (a mix names its member).
    pub scheme: String,
    /// The stored form: drawn parts filled in, `{parent}`, `{parent.short}`
    /// and `{system}` kept, so a renamed parent flows down (design 4.5).
    pub form: String,
    /// Whether the name follows its parent.
    pub follows: bool,
    pub parts: Vec<Part>,
    /// "KHAZ-rak": syllables, the stressed one in capitals.
    pub say: String,
    /// "/ˈxaz.rak/".
    pub ipa: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<String>,
    /// Plain hints for the unusual sounds it holds ("kh": "ch as in loch").
    pub hints: BTreeMap<String, String>,
    /// Draws it took (1: the first).
    pub tries: u32,
    /// Made only from numbers and fixed text: unique by construction.
    pub numbered: bool,
}

impl Named {
    /// The name as shown, with the parent's and system's names as they
    /// stand now.
    pub fn show(&self, parent: Option<&str>, system: Option<&str>) -> String {
        let p = parent.unwrap_or("");
        let short = p.split_whitespace().next().unwrap_or("");
        let s = self
            .form
            .replace("{parent.short}", short)
            .replace("{parent}", p)
            .replace("{system}", system.unwrap_or(""));
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn base_of(s: &Scheme) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&s.hash.0[..8]);
    u64::from_le_bytes(b)
}

fn pick<'a>(d: &Draws, i: u64, items: &'a [Weighted]) -> &'a Weighted {
    let total: u64 = items.iter().map(|w| u64::from(w.weight())).sum();
    let mut r = d.below(i, total.max(1));
    for w in items {
        let k = u64::from(w.weight());
        if r < k {
            return w;
        }
        r -= k;
    }
    &items[items.len() - 1]
}

fn roman(mut n: u32) -> String {
    const T: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut s = String::new();
    for (v, t) in T {
        while n >= v {
            s.push_str(t);
            n -= v;
        }
    }
    s
}

/// Letters in order from `first`, past z as two letters (aa, ab ...).
fn letters(mut n: u32, first: u8, upper: bool) -> String {
    let a = if upper { b'A' } else { b'a' };
    n += u32::from(first - a);
    let mut s = vec![];
    loop {
        s.push((a + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    s.iter().rev().collect()
}

fn coord(facts: &BTreeMap<String, String>) -> Option<String> {
    let ra: f64 = facts.get("ra_deg")?.parse().ok()?;
    let dec: f64 = facts.get("dec_deg")?.parse().ok()?;
    if !(0.0..360.0).contains(&ra) || !(-90.0..=90.0).contains(&dec) {
        return None;
    }
    let ra_min = (ra / 15.0 * 60.0) as u32;
    let dec_min = (dec.abs() * 60.0) as u32;
    Some(format!(
        "J{:02}{:02}{}{:02}{:02}",
        ra_min / 60,
        ra_min % 60,
        if dec < 0.0 { '-' } else { '+' },
        dec_min / 60,
        dec_min % 60
    ))
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// A word from the Sounds maker: (spelling, syllables as sounds, hints).
fn word(s: &Scheme, d: &Draws, i0: u64) -> Option<Vec<Vec<String>>> {
    let so = s.doc.sounds.as_ref()?;
    // A few redraws inside one try, for a "never" sequence.
    for sub in 0..16u64 {
        let i = i0.wrapping_add(sub << 16);
        let (lo, hi) = so.length;
        let n = u64::from(lo) + d.below(i, u64::from(hi - lo + 1));
        let mut syl: Vec<Vec<String>> = vec![];
        let mut j = 1u64;
        for _ in 0..n {
            let shape = pick(d, i.wrapping_add(j), &so.shapes).text().to_string();
            j += 1;
            let mut one = vec![];
            for c in shape.chars() {
                let cls = &so.classes[&c.to_string()];
                one.push(pick(d, i.wrapping_add(j), cls).text().to_string());
                j += 1;
            }
            syl.push(one);
        }
        let flat: Vec<&String> = syl.iter().flatten().collect();
        let spelt: String = flat.iter().map(|x| x.as_str()).collect();
        let bad = so.never.iter().any(|n| {
            if n.len() == 1 {
                return spelt.contains(n[0].as_str());
            }
            flat.windows(n.len())
                .any(|w| w.iter().zip(n).all(|(a, b)| *a == b))
        });
        if !bad {
            return Some(syl);
        }
    }
    None
}

/// Read a spelling back into sounds, longest sound first (for the faces
/// of a meaning word). Unknown letters stand for themselves.
fn sounds_of(s: &Scheme, w: &str) -> Vec<String> {
    let mut out = vec![];
    let mut rest = w;
    'outer: while !rest.is_empty() {
        for snd in &s.inventory {
            if let Some(r) = rest.strip_prefix(snd.as_str()) {
                out.push(snd.clone());
                rest = r;
                continue 'outer;
            }
        }
        let c = rest.chars().next().expect("not empty");
        out.push(c.to_string());
        rest = &rest[c.len_utf8()..];
    }
    out
}

struct Faces<'s> {
    s: &'s Scheme,
    say: Vec<String>,
    ipa: Vec<String>,
    hints: BTreeMap<String, String>,
    /// Syllables of the written word being built (parts that join).
    cur: Vec<Vec<String>>,
}

impl Faces<'_> {
    /// Add syllables of sounds: to the word being built when the part joins
    /// it, else as a new word.
    fn add(&mut self, joins: bool, syl: Vec<Vec<String>>) {
        if !joins {
            self.flush();
        }
        self.cur.extend(syl);
    }
    /// A finished piece (fixed text, a list row): ends the word being built.
    fn piece(&mut self, say: &str, ipa: &str) {
        self.flush();
        if !say.trim().is_empty() {
            self.say.push(say.trim().to_string());
            self.ipa.push(ipa.trim().to_string());
        }
    }
    /// Write the word being built: syllables joined, the stressed one in
    /// capitals (say it) or marked (IPA).
    fn flush(&mut self) {
        let syl = std::mem::take(&mut self.cur);
        if syl.is_empty() {
            return;
        }
        let Some(so) = self.s.doc.sounds.as_ref() else {
            let w: String = syl.iter().flatten().cloned().collect();
            self.say.push(w.clone());
            self.ipa.push(w);
            return;
        };
        let k = syl.len();
        let stressed = match so.stress.as_deref() {
            Some("last") => k.saturating_sub(1),
            Some("penult") => k.saturating_sub(2),
            _ => 0,
        };
        let mut say = vec![];
        let mut ipa = vec![];
        for (n, one) in syl.iter().enumerate() {
            let sp: String = one.concat();
            say.push(if n == stressed && k > 1 {
                sp.to_uppercase()
            } else {
                sp
            });
            let ip: String = one
                .iter()
                .map(|x| so.ipa.get(x).cloned().unwrap_or_else(|| x.clone()))
                .collect();
            ipa.push(if n == stressed && k > 1 {
                format!("ˈ{ip}")
            } else {
                ip
            });
            for x in one {
                if let Some(h) = so.say.get(x) {
                    self.hints.insert(x.clone(), h.clone());
                }
            }
        }
        self.say.push(say.join("-"));
        self.ipa.push(ipa.join("."));
    }
}

fn avoided_text(avoid: &std::collections::BTreeSet<String>, folded: &str) -> bool {
    let joined: String = folded.split(' ').collect();
    folded.split(' ').any(|w| avoid.contains(w))
        || avoid
            .iter()
            .any(|a| a.chars().count() >= 4 && joined.contains(a.as_str()))
}

/// Whether a name is, or holds, a word on the avoid list (after the fold).
pub fn avoided(avoid: &std::collections::BTreeSet<String>, name: &str) -> bool {
    avoided_text(avoid, &text::fold(name))
}

/// The scheme a mix hands an object to, followed down to one that names.
fn resolve_mix<'l>(lib: &'l Library, r: &Request, mut s: &'l Scheme) -> Result<&'l Scheme, String> {
    for _ in 0..16 {
        let Some(m) = &s.doc.mix else { return Ok(s) };
        let stream =
            Stream::new(&format!("names.{}.mix.v1", r.layer)).map_err(|e| e.to_string())?;
        let own = Draws::new(*r.seed, r.address.clone(), stream.clone());
        let base = base_of(s);
        let stray = own.below(base, 1000) < u64::from(m.strays_permille);
        let at = match (&m.clump, stray) {
            (Some(k), false) => {
                let mut a = r.address.clone();
                loop {
                    if a.last().kind() == k {
                        break a;
                    }
                    match a.parent() {
                        Some(p) => a = p,
                        None => break r.address.clone(),
                    }
                }
            }
            _ => r.address.clone(),
        };
        let d = Draws::new(*r.seed, at, stream);
        let id = pick(&d, base.wrapping_add(1), &m.of).text().to_string();
        s = lib
            .schemes
            .get(&id)
            .ok_or_else(|| format!("the mix names no scheme {id:?}"))?;
    }
    Err("a mix nests more than 16 deep".into())
}

fn draw_one(s: &Scheme, r: &Request, d: &Draws, attempt: u32) -> Result<Named, String> {
    let base = base_of(s).wrapping_add(u64::from(attempt) << 32);
    let forms = s.forms_for(r.kind).ok_or_else(|| {
        format!(
            "scheme {:?} has no form for {} and no default",
            s.doc.id, r.kind
        )
    })?;
    let items: Vec<Weighted> = forms
        .iter()
        .enumerate()
        .map(|(k, f)| Weighted::With(k.to_string(), f.1))
        .collect();
    let k: usize = pick(d, base, &items).text().parse().unwrap_or(0);
    let toks = &forms[k].0;
    let mut form = String::new();
    let mut parts = vec![];
    let mut faces = Faces {
        s,
        say: vec![],
        ipa: vec![],
        hints: BTreeMap::new(),
        cur: vec![],
    };
    let mut native: Option<String> = None;
    let mut numbered = true;
    let mut j = 0x100u64;
    // Whether the text so far ends inside a word (a drawn part then joins it
    // in lower case: "Khaz" + "rak").
    let in_word = |f: &str| {
        f.chars()
            .last()
            .is_some_and(|c| c.is_alphanumeric() || c == '}')
    };
    for t in toks {
        j += 0x100;
        let i = base.wrapping_add(j);
        match t {
            Tok::Text(x) => {
                form.push_str(x);
                faces.piece(x, x);
            }
            Tok::Word => {
                numbered = false;
                let syl = word(s, d, i)
                    .ok_or("every word drawn holds a sequence the scheme says never occurs")?;
                let w: String = syl.iter().flatten().cloned().collect();
                let joins = in_word(&form);
                let w = if joins { w } else { capitalise(&w) };
                faces.add(joins, syl);
                form.push_str(&w);
                parts.push(Part {
                    text: w,
                    from: "word".into(),
                    means: None,
                    source: None,
                });
            }
            Tok::List(l) => {
                numbered = false;
                let rows: &Vec<Row> = &s.lists[l];
                let row = &rows[d.below(i, rows.len() as u64) as usize];
                form.push_str(&row.name);
                faces.piece(
                    row.say.as_deref().unwrap_or(&row.name),
                    row.ipa.as_deref().unwrap_or(&row.name),
                );
                if row.native.is_some() {
                    native = Some(native.unwrap_or_default() + row.native.as_deref().unwrap_or(""));
                }
                parts.push(Part {
                    text: row.name.clone(),
                    from: format!("list:{l}"),
                    means: row.means.clone(),
                    source: row.source.clone(),
                });
            }
            Tok::Means(m) | Tok::MeansOf(m) => {
                let id = match t {
                    Tok::MeansOf(f) => match r.facts.get(f) {
                        Some(v) => v.clone(),
                        None => continue,
                    },
                    _ => m.clone(),
                };
                let Some(w) = s.doc.means.get(&id) else {
                    continue;
                };
                numbered = false;
                let joins = in_word(&form);
                let w = if joins { w.clone() } else { capitalise(w) };
                faces.add(joins, vec![sounds_of(s, &w.to_lowercase())]);
                form.push_str(&w);
                parts.push(Part {
                    text: w,
                    from: format!("means:{id}"),
                    means: Some(id),
                    source: None,
                });
            }
            Tok::Roman => form.push_str(&roman(r.index + 1)),
            Tok::Letter => form.push_str(&letters(r.index, b'b', false)),
            Tok::CapLetter => form.push_str(&letters(r.index, b'A', true)),
            Tok::Number => form.push_str(&(r.index + 1).to_string()),
            Tok::Serial(w) => {
                let n = r
                    .serial
                    .ok_or("this form needs a running number ({serial}) the caller did not give")?;
                form.push_str(&format!("{n:0w$}", w = *w));
            }
            Tok::Coord => form.push_str(&coord(r.facts).ok_or("{coord} needs ra_deg and dec_deg")?),
            Tok::Parent => form.push_str("{parent}"),
            Tok::ParentShort => form.push_str("{parent.short}"),
            Tok::System => form.push_str("{system}"),
        }
    }
    faces.flush();
    let follows = s.follows(r.kind)
        && toks
            .iter()
            .any(|t| matches!(t, Tok::Parent | Tok::ParentShort));
    let say: Vec<String> = faces.say.into_iter().filter(|x| !x.is_empty()).collect();
    let ipa: Vec<String> = faces.ipa.into_iter().filter(|x| !x.is_empty()).collect();
    Ok(Named {
        scheme: s.doc.id.clone(),
        form,
        follows,
        parts,
        say: say.join(" "),
        ipa: if ipa.is_empty() {
            String::new()
        } else {
            format!("/{}/", ipa.join(" "))
        },
        native,
        hints: faces.hints,
        tries: attempt + 1,
        numbered,
    })
}

/// Make one name, moving to the next draw while `taken` says the folded
/// name is in use (or on the avoid list). After [`MAX_TRIES`] the object
/// keeps only its designation, and the reason says so.
pub fn name_with(
    lib: &Library,
    r: &Request,
    taken: &mut dyn FnMut(&str, bool) -> bool,
) -> Result<Named, String> {
    if r.layer.is_empty()
        || !r
            .layer
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    {
        return Err(format!(
            "layer {:?} is lower-case letters, digits and _",
            r.layer
        ));
    }
    let top = lib
        .schemes
        .get(r.scheme)
        .ok_or_else(|| format!("no name scheme {:?} is stacked", r.scheme))?;
    let s = resolve_mix(lib, r, top)?;
    let stream = Stream::new(&format!("names.{}.v1", r.layer)).map_err(|e| e.to_string())?;
    let d = Draws::new(*r.seed, r.address.clone(), stream);
    for attempt in 0..MAX_TRIES {
        let n = draw_one(s, r, &d, attempt)?;
        let shown = n.show(r.parent, r.system);
        if text::check(&shown).is_err() {
            continue;
        }
        let folded = text::fold(&shown);
        if avoided_text(&lib.avoid, &folded) || taken(&folded, n.numbered) {
            if n.numbered {
                // A designation does not change with another draw.
                return Err(format!(
                    "{shown} is already in use (a designation draws only once)"
                ));
            }
            continue;
        }
        return Ok(n);
    }
    Err(format!(
        "no free name after {MAX_TRIES} draws: it keeps its designation only"
    ))
}

/// Make one name with nothing else to clash with.
pub fn name(lib: &Library, r: &Request) -> Result<Named, String> {
    name_with(lib, r, &mut |_, _| false)
}

/// Name siblings made together (a system's planets, a world's peaks), in
/// address order: a later sibling whose name is the same as an earlier one,
/// or one letter from it (worded names only), moves to its next draw. The
/// result is in the order given.
pub fn siblings(lib: &Library, rs: &[Request]) -> Vec<Result<Named, String>> {
    let mut order: Vec<usize> = (0..rs.len()).collect();
    order.sort_by(|a, b| rs[*a].address.cmp(rs[*b].address).then(a.cmp(b)));
    let mut out: Vec<Option<Result<Named, String>>> = vec![None; rs.len()];
    let mut seen: Vec<(String, bool)> = vec![];
    for i in order {
        let res = name_with(lib, &rs[i], &mut |f, numbered| {
            seen.iter()
                .any(|(g, gn)| g == f || (!numbered && !gn && text::near(f, g)))
        });
        if let Ok(n) = &res {
            seen.push((text::fold(&n.show(rs[i].parent, rs[i].system)), n.numbered));
        }
        out[i] = Some(res);
    }
    out.into_iter()
        .map(|x| x.expect("every sibling named"))
        .collect()
}
