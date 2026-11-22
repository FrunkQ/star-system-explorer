//! A name scheme as pack data (`provides.names`, naming design 4.1 and 4.11),
//! its load checks, and the library of schemes a stack holds.
//!
//! ```json
//! { "id": "vesh", "label": "Vesh", "about": "...",
//!   "sounds": { "classes": { "C": [["k", 5], ["kh", 4]], "V": [["a", 6]] },
//!               "shapes": [["CV", 4], ["CVE", 5]], "length": [2, 3],
//!               "never": [["aa", "a"], ["x", "x"]], "stress": "first",
//!               "say": { "kh": "ch as in loch" }, "ipa": { "kh": "x" } },
//!   "lists": { "rivers": [ { "name": "Tiber", "means": "river", "source": "..." } ] },
//!   "means": { "red": "khaz", "mountain": "rak" },
//!   "forms": { "planet": [["{word}", 1]], "peak": [["{means:colour?}{means:mountain}", 1]],
//!              "default": [["{word}", 1]] },
//!   "mix": { "of": [["english", 1], ["vesh", 1]], "clump": "sys", "strays_permille": 100 },
//!   "follow": ["planet", "moon"],
//!   "avoid": ["..."] }
//! ```
//!
//! Makers (design 4.1): List (`lists`, `{list:<name>}`), Sounds (`sounds`,
//! `{word}`), Number (`{roman}`, `{letter}`, `{Letter}`, `{n}`, `{serial}`,
//! `{serial:4}`, `{coord}`), Pattern (`forms`, fixed text around tokens), Mix
//! (`mix`). `{means:<id>}` and `{means:<fact>?}` read the scheme's word list
//! (the fact form reads the object's own fact as a meaning id; design 4.1,
//! fuller in NM5). `{parent}`, `{parent.short}` and `{system}` follow the
//! parent's or the system's name as it stands when shown (design 4.5).

use crate::text;
use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use sse_core::ContentHash;
use std::collections::{BTreeMap, BTreeSet};

/// Largest scheme, as canonical JSON bytes.
pub const MAX_SCHEME_BYTES: usize = 1 << 20;
/// Most rows in one list.
pub const MAX_ROWS: usize = 50_000;
/// Most sounds in a class, forms for a kind, shapes.
pub const MAX_ITEMS: usize = 256;
/// Most syllables in a word.
pub const MAX_SYLLABLES: u32 = 8;
/// Draws tried before a name gives up (as the feature namer has done).
pub const MAX_TRIES: u32 = 64;

/// A weighted item: `["CV", 4]` or `"CV"` (weight 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Weighted {
    Plain(String),
    With(String, u32),
}
impl Weighted {
    pub fn text(&self) -> &str {
        match self {
            Weighted::Plain(s) | Weighted::With(s, _) => s,
        }
    }
    pub fn weight(&self) -> u32 {
        match self {
            Weighted::Plain(_) => 1,
            Weighted::With(_, w) => *w,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sounds {
    /// Sound classes by one-letter name: each sound's spelling and weight.
    pub classes: BTreeMap<String, Vec<Weighted>>,
    /// Syllable shapes, each a string of class letters.
    pub shapes: Vec<Weighted>,
    /// Fewest and most syllables.
    pub length: (u32, u32),
    /// Sound sequences that never occur (`["aa", "a"]`: "aa" then "a",
    /// across syllables too). One entry is a spelling that never occurs.
    #[serde(default)]
    pub never: Vec<Vec<String>>,
    /// `first`, `last` or `penult`.
    #[serde(default)]
    pub stress: Option<String>,
    /// A plain hint per sound ("ch as in loch").
    #[serde(default)]
    pub say: BTreeMap<String, String>,
    /// The IPA symbol per sound.
    #[serde(default)]
    pub ipa: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub say: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipa: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub means: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

/// A list row may be written as a bare name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RowDoc {
    Name(String),
    Row(Row),
}
impl RowDoc {
    pub fn row(&self) -> Row {
        match self {
            RowDoc::Name(n) => Row {
                name: n.clone(),
                ..Row::default()
            },
            RowDoc::Row(r) => r.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mix {
    /// Schemes by id, with weights.
    pub of: Vec<Weighted>,
    /// The address segment kind the choice is held beneath (`sys`: once
    /// per system). Left out: chosen per object.
    #[serde(default)]
    pub clump: Option<String>,
    /// Share of objects that stray from the clump's choice, per thousand.
    #[serde(default)]
    pub strays_permille: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeDoc {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub about: String,
    /// `latin`, `cjk`, ... (how the spelling is written).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    /// `invented`, `real`, `catalogue`, `list`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sounds: Option<Sounds>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, Vec<RowDoc>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub means: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub forms: BTreeMap<String, Vec<Weighted>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mix: Option<Mix>,
    /// Kinds whose names follow their parent when the form says so; left
    /// out, every form with `{parent}` follows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow: Option<Vec<String>>,
    /// Words this scheme never makes (added to the library's avoid list).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub avoid: Vec<String>,
}

/// One token of a form.
#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Text(String),
    Word,
    List(String),
    Means(String),
    /// A meaning id read from the object's fact of this name.
    MeansOf(String),
    Roman,
    Letter,
    CapLetter,
    Number,
    Serial(usize),
    Coord,
    Parent,
    ParentShort,
    System,
}
impl Tok {
    /// A number token: unique by construction, so no near-clash check.
    pub fn is_number(&self) -> bool {
        matches!(
            self,
            Tok::Roman | Tok::Letter | Tok::CapLetter | Tok::Number | Tok::Serial(_) | Tok::Coord
        )
    }
}

pub fn parse_form(f: &str) -> Result<Vec<Tok>, String> {
    let mut out = vec![];
    let mut rest = f;
    while let Some(a) = rest.find('{') {
        if a > 0 {
            out.push(Tok::Text(rest[..a].to_string()));
        }
        let b = rest[a..]
            .find('}')
            .ok_or_else(|| format!("form {f:?}: a `{{` has no `}}`"))?
            + a;
        let t = &rest[a + 1..b];
        out.push(match t {
            "word" => Tok::Word,
            "roman" => Tok::Roman,
            "letter" => Tok::Letter,
            "Letter" => Tok::CapLetter,
            "n" => Tok::Number,
            "serial" => Tok::Serial(0),
            "coord" => Tok::Coord,
            "parent" => Tok::Parent,
            "parent.short" => Tok::ParentShort,
            "system" => Tok::System,
            _ => {
                if let Some(l) = t.strip_prefix("list:") {
                    Tok::List(l.to_string())
                } else if let Some(m) = t.strip_prefix("means:") {
                    match m.strip_suffix('?') {
                        Some(fact) => Tok::MeansOf(fact.to_string()),
                        None => Tok::Means(m.to_string()),
                    }
                } else if let Some(d) = t.strip_prefix("serial:") {
                    let d: usize = d
                        .parse()
                        .ok()
                        .filter(|d| (1..=12).contains(d))
                        .ok_or_else(|| format!("form {f:?}: {{serial:N}} takes 1 to 12 digits"))?;
                    Tok::Serial(d)
                } else {
                    return Err(format!("form {f:?}: {{{t}}} is not a part a form knows"));
                }
            }
        });
        rest = &rest[b + 1..];
    }
    if rest.contains('}') {
        return Err(format!("form {f:?}: a `}}` has no `{{`"));
    }
    if !rest.is_empty() {
        out.push(Tok::Text(rest.to_string()));
    }
    Ok(out)
}

/// A checked scheme, ready to draw from.
#[derive(Clone, Debug)]
pub struct Scheme {
    pub doc: SchemeDoc,
    /// Content hash of the scheme as written; names are keyed on it.
    pub hash: ContentHash,
    pub forms: BTreeMap<String, Vec<(Vec<Tok>, u32)>>,
    pub lists: BTreeMap<String, Vec<Row>>,
    /// Sounds in inventory order (for reading a word back into sounds).
    pub inventory: Vec<String>,
}

fn weights_ok(items: &[Weighted], what: &str) -> Result<(), String> {
    if items.is_empty() || items.len() > MAX_ITEMS {
        return Err(format!("{what}: 1 to {MAX_ITEMS} entries"));
    }
    if items.iter().all(|w| w.weight() == 0) || items.iter().any(|w| w.weight() > 1_000_000) {
        return Err(format!("{what}: weights are 0 to 1,000,000, not all 0"));
    }
    Ok(())
}

fn spelling_ok(s: &str, what: &str) -> Result<(), String> {
    text::check(s).map_err(|e| format!("{what} {s:?}: {e}"))
}

impl Scheme {
    /// Load and check one scheme document.
    pub fn load(v: &J) -> Result<Scheme, String> {
        let at = v
            .get("id")
            .and_then(J::as_str)
            .map(|s| format!("name scheme {s:?}"))
            .unwrap_or_else(|| "a name scheme".into());
        let bytes = serde_json::to_vec(v).map_err(|e| format!("{at}: {e}"))?;
        if bytes.len() > MAX_SCHEME_BYTES {
            return Err(format!(
                "{at}: {} bytes is more than the {MAX_SCHEME_BYTES} a scheme holds",
                bytes.len()
            ));
        }
        let doc: SchemeDoc = serde_json::from_value(v.clone()).map_err(|e| format!("{at}: {e}"))?;
        let hash = ContentHash::of(v).map_err(|e| format!("{at}: {e:?}"))?;
        Self::from_doc(doc, hash).map_err(|e| format!("{at}: {e}"))
    }

    fn from_doc(doc: SchemeDoc, hash: ContentHash) -> Result<Scheme, String> {
        if doc.id.is_empty()
            || !doc
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"-_.".contains(&c))
        {
            return Err("the id is lower-case letters, digits, `-`, `_` and `.`".into());
        }
        spelling_ok(&doc.label, "the label")?;
        let mut inventory: Vec<String> = vec![];
        if let Some(s) = &doc.sounds {
            if s.classes.is_empty() {
                return Err("sounds: at least one class".into());
            }
            for (k, items) in &s.classes {
                if k.chars().count() != 1 || !k.chars().all(|c| c.is_ascii_uppercase()) {
                    return Err(format!("sounds: class {k:?} is one capital letter"));
                }
                weights_ok(items, &format!("sounds: class {k}"))?;
                for w in items {
                    let t = w.text();
                    if t.chars().count() > 8 || t.contains(' ') {
                        return Err(format!("sounds: {t:?} is a sound of 1 to 8 letters"));
                    }
                    spelling_ok(t, "sounds: the sound")?;
                    if !inventory.iter().any(|x| x == t) {
                        inventory.push(t.to_string());
                    }
                }
            }
            weights_ok(&s.shapes, "sounds: shapes")?;
            for sh in &s.shapes {
                for c in sh.text().chars() {
                    if !s.classes.contains_key(&c.to_string()) {
                        return Err(format!("sounds: shape {:?} names no class {c}", sh.text()));
                    }
                }
                if sh.text().is_empty() || sh.text().chars().count() > 8 {
                    return Err(format!("sounds: shape {:?} is 1 to 8 classes", sh.text()));
                }
            }
            let (lo, hi) = s.length;
            if lo == 0 || lo > hi || hi > MAX_SYLLABLES {
                return Err(format!(
                    "sounds: length is 1 to {MAX_SYLLABLES} syllables, fewest first"
                ));
            }
            if s.never.len() > MAX_ITEMS || s.never.iter().any(|n| n.is_empty() || n.len() > 4) {
                return Err("sounds: never lists up to 256 sequences of 1 to 4 sounds".into());
            }
            if let Some(st) = &s.stress {
                if !["first", "last", "penult"].contains(&st.as_str()) {
                    return Err(format!("sounds: stress {st:?} is first, last or penult"));
                }
            }
            // Longest first, for reading spellings back into sounds.
            inventory.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()).then(a.cmp(b)));
        }
        let mut lists = BTreeMap::new();
        for (k, rows) in &doc.lists {
            if rows.is_empty() || rows.len() > MAX_ROWS {
                return Err(format!("list {k}: 1 to {MAX_ROWS} rows"));
            }
            let rows: Vec<Row> = rows
                .iter()
                .map(|r| {
                    let mut r = r.row();
                    r.name = text::compose(&r.name);
                    r.native = r.native.map(|n| text::compose(&n));
                    r
                })
                .collect();
            for r in &rows {
                spelling_ok(&r.name, &format!("list {k}: the name"))?;
                if let Some(n) = &r.native {
                    spelling_ok(n, &format!("list {k}: the native form"))?;
                }
            }
            lists.insert(k.clone(), rows);
        }
        for (k, w) in &doc.means {
            spelling_ok(w, &format!("means {k}:"))?;
        }
        if doc.forms.is_empty() && doc.mix.is_none() {
            return Err("a scheme has forms (or is a mix of schemes)".into());
        }
        if doc.mix.is_some() && !doc.forms.is_empty() {
            return Err("a mix names nothing itself: its forms are its schemes'".into());
        }
        let mut forms = BTreeMap::new();
        for (kind, fs) in &doc.forms {
            weights_ok(fs, &format!("forms for {kind}"))?;
            let mut parsed = vec![];
            for f in fs {
                let toks = parse_form(f.text())?;
                for t in &toks {
                    match t {
                        Tok::Text(s) => {
                            if s.contains(['{', '}']) || s.chars().any(text_bad) {
                                return Err(format!(
                                    "form {:?}: its fixed text holds a character a name may not",
                                    f.text()
                                ));
                            }
                        }
                        Tok::Word if doc.sounds.is_none() => {
                            return Err(format!("form {:?}: {{word}} needs sounds", f.text()))
                        }
                        Tok::List(l) if !lists.contains_key(l) => {
                            return Err(format!("form {:?}: there is no list {l}", f.text()))
                        }
                        Tok::Means(m) if !doc.means.contains_key(m) => {
                            return Err(format!("form {:?}: no word means {m}", f.text()))
                        }
                        _ => {}
                    }
                }
                parsed.push((toks, f.weight()));
            }
            forms.insert(kind.clone(), parsed);
        }
        if let Some(m) = &doc.mix {
            weights_ok(&m.of, "mix")?;
            if m.strays_permille > 1000 {
                return Err("mix: strays are 0 to 1000 per thousand".into());
            }
        }
        Ok(Scheme {
            doc,
            hash,
            forms,
            lists,
            inventory,
        })
    }

    /// The forms for a kind, else the default ones.
    pub fn forms_for(&self, kind: &str) -> Option<&Vec<(Vec<Tok>, u32)>> {
        self.forms.get(kind).or_else(|| self.forms.get("default"))
    }

    /// Whether a kind's names follow their parent.
    pub fn follows(&self, kind: &str) -> bool {
        self.doc
            .follow
            .as_ref()
            .is_none_or(|k| k.iter().any(|x| x == kind))
    }

    /// The old feature settings (`base-surface`'s `naming`: syllables, a
    /// length and a pattern per kind with `{}`) read as a scheme: a Sounds
    /// maker with one class, each syllable a sound (design section 5).
    pub fn from_naming_settings(id: &str, v: &J) -> Result<Scheme, String> {
        #[derive(Deserialize)]
        struct Old {
            syllables: Vec<String>,
            length: [u32; 2],
            patterns: BTreeMap<String, String>,
            #[serde(default)]
            source: String,
        }
        let o: Old =
            serde_json::from_value(v.clone()).map_err(|e| format!("naming settings: {e}"))?;
        let doc = SchemeDoc {
            id: id.into(),
            label: "Syllables".into(),
            about: o.source,
            script: Some("latin".into()),
            kind: Some("invented".into()),
            sounds: Some(Sounds {
                classes: [(
                    "S".to_string(),
                    o.syllables.into_iter().map(Weighted::Plain).collect(),
                )]
                .into_iter()
                .collect(),
                shapes: vec![Weighted::Plain("S".into())],
                length: (o.length[0].max(1), o.length[1].max(o.length[0].max(1))),
                ..Sounds::default()
            }),
            lists: BTreeMap::new(),
            means: BTreeMap::new(),
            forms: o
                .patterns
                .into_iter()
                .map(|(k, p)| (k, vec![Weighted::Plain(p.replacen("{}", "{word}", 1))]))
                .collect(),
            mix: None,
            follow: None,
            avoid: vec![],
        };
        let hash = ContentHash::of(v).map_err(|e| format!("{e:?}"))?;
        Self::from_doc(doc, hash).map_err(|e| format!("naming settings: {e}"))
    }
}

fn text_bad(c: char) -> bool {
    text::check(&c.to_string()).is_err() && c != ' '
}

/// Every scheme a stack holds, and the avoid list.
#[derive(Clone, Debug, Default)]
pub struct Library {
    pub schemes: BTreeMap<String, Scheme>,
    /// Folded words generated names never are or hold.
    pub avoid: BTreeSet<String>,
}

impl Library {
    /// Load `provides.names` from each pack, bottom to top: `{schemes: [...],
    /// avoid: [...]}`. A later pack's scheme with the same id replaces an
    /// earlier one. Mixes must name schemes that exist, with no cycle.
    pub fn load(packs: &[(String, J)]) -> Result<Library, Vec<String>> {
        let mut lib = Library::default();
        let mut errs = vec![];
        for (pack, v) in packs {
            let Some(o) = v.as_object() else {
                errs.push(format!("{pack}: names is an object of schemes and avoid"));
                continue;
            };
            for k in o.keys() {
                if k != "schemes" && k != "avoid" {
                    errs.push(format!("{pack}: names takes schemes and avoid, not {k:?}"));
                }
            }
            for s in o.get("schemes").and_then(J::as_array).into_iter().flatten() {
                match Scheme::load(s) {
                    Ok(s) => {
                        lib.schemes.insert(s.doc.id.clone(), s);
                    }
                    Err(e) => errs.push(format!("{pack}: {e}")),
                }
            }
            for w in o.get("avoid").and_then(J::as_array).into_iter().flatten() {
                match w.as_str() {
                    Some(w) if !w.trim().is_empty() => {
                        lib.avoid.insert(text::fold(w));
                    }
                    _ => errs.push(format!("{pack}: avoid lists words")),
                }
            }
        }
        for s in lib.schemes.values() {
            for w in &s.doc.avoid {
                lib.avoid.insert(text::fold(w));
            }
        }
        // Mixes: every member exists, and no mix reaches itself.
        for (id, s) in &lib.schemes {
            let Some(m) = &s.doc.mix else { continue };
            for w in &m.of {
                if !lib.schemes.contains_key(w.text()) {
                    errs.push(format!(
                        "name scheme {id:?}: the mix names no scheme {:?}",
                        w.text()
                    ));
                }
            }
            let mut stack: Vec<(String, Vec<String>)> = vec![(id.clone(), vec![id.clone()])];
            while let Some((at, path)) = stack.pop() {
                let Some(m) = lib.schemes.get(&at).and_then(|s| s.doc.mix.as_ref()) else {
                    continue;
                };
                for w in &m.of {
                    if w.text() == id {
                        errs.push(format!(
                            "name scheme {id:?}: the mix contains itself ({} > {id})",
                            path.join(" > ")
                        ));
                    } else if !path.iter().any(|p| p == w.text()) && path.len() < 16 {
                        let mut p = path.clone();
                        p.push(w.text().to_string());
                        stack.push((w.text().to_string(), p));
                    }
                }
            }
        }
        errs.dedup();
        if errs.is_empty() {
            Ok(lib)
        } else {
            Err(errs)
        }
    }

    /// How many rows of a scheme's lists the avoid list matches: shown
    /// before a GM accepts a pack (design 4.9).
    pub fn avoid_hits(&self, scheme: &str) -> usize {
        self.schemes.get(scheme).map_or(0, |s| {
            s.lists
                .values()
                .flatten()
                .filter(|r| crate::make::avoided(&self.avoid, &r.name))
                .count()
        })
    }
}
