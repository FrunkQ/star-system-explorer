//! TG1: tags are declared (tags design section 4).
//!
//! A tag is a name on an address or a note, present or absent, nothing
//! more. Every pack that uses tags declares them in `provides.other.tags`:
//!
//! ```text
//! groups: [{ id, label, about?, colour?, applies_to?: [..], exclusive?: bool }]
//! tags:   [{ id, label, about?, group?, source?: authored | derived | either,
//!            tier?, colour? }]
//! ```
//!
//! Every tag a rule, sifter, template, trigger, program, data view,
//! disclosure entry, tag grant, lens or code sheet names must be declared
//! somewhere in the stack, or the stack is refused with the pack and the
//! reference named. The campaign keeps its own declarations (the GM's
//! tags) in the record; a tag the GM types freely joins them on first use.
//!
//! Reserved for the 4.1 economy pack: the tag group `trade` and the kept
//! kind prefix `economy.`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The group and the kept-kind prefix only the economy pack may use.
pub const RESERVED_GROUP: &str = "trade";
pub const RESERVED_KEPT_PREFIX: &str = "economy.";

/// The record field prefix that holds the campaign's own declarations.
pub const CAMPAIGN_FIELD: &str = "campaign.tag_decl.";
/// Record event kind of a campaign declaration.
pub const CAMPAIGN_KIND: &str = "tag.declared";

/// Content blocks that may name tags, and what a reference there is called.
const READERS: [(&str, &str); 9] = [
    ("rules", "rule"),
    ("sifters", "sifter"),
    ("templates", "template"),
    ("triggers", "trigger"),
    ("programs", "program"),
    ("observe", "view"),
    ("code_sheets", "code sheet"),
    ("lenses", "lens"),
    ("intents", "intent"),
];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TagGroup {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    /// Where the tag is offered (body, system, construct, actor, note...);
    /// not where it may exist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to: Vec<String>,
    /// At most one tag of the group per object.
    #[serde(default)]
    pub exclusive: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    Authored,
    Derived,
    #[default]
    Either,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TagDecl {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// Its group; when absent, the part before the first dot if that is a
    /// declared group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default)]
    pub source: Source,
    /// Default disclosure tier. Absent: the GM-only rung (tags design
    /// ruling 4), as for any fact the disclosure does not list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct TagBlock {
    #[serde(default)]
    pub groups: Vec<TagGroup>,
    #[serde(default)]
    pub tags: Vec<TagDecl>,
}

/// Every declaration in a stack, with the pack that made it.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TagSet {
    pub groups: BTreeMap<String, (String, TagGroup)>,
    pub tags: BTreeMap<String, (String, TagDecl)>,
}

/// One place a pack names a tag.
#[derive(Clone, PartialEq, Debug)]
pub struct TagRef {
    pub pack: String,
    /// `rule`, `template`, `trigger`, `view`, ...
    pub reader: &'static str,
    /// The item that names it (its id when it has one, else its path).
    pub item: String,
    pub tag: String,
}

fn economy_pack(id: &str) -> bool {
    id == "economy" || id.ends_with(".economy")
}

impl TagSet {
    /// The group a tag belongs to, if any.
    pub fn group_of(&self, tag: &str) -> Option<&TagGroup> {
        let named = self.tags.get(tag).and_then(|(_, d)| d.group.clone());
        let g = named.or_else(|| tag.split_once('.').map(|(g, _)| g.to_string()))?;
        self.groups.get(&g).map(|(_, g)| g)
    }

    pub fn declared(&self, tag: &str) -> bool {
        self.tags.contains_key(tag)
    }

    /// Add one pack's block (or the campaign's), refusing duplicates and
    /// the reserved names.
    pub fn add(&mut self, pack: &str, b: TagBlock, errs: &mut Vec<String>) {
        for g in b.groups {
            if g.id == RESERVED_GROUP && !economy_pack(pack) {
                errs.push(format!(
                    "{pack}: tag group {RESERVED_GROUP} is reserved for the economy pack"
                ));
                continue;
            }
            if let Some((prev, had)) = self.groups.get(&g.id) {
                if *had == g {
                    continue; // the same declaration twice: a shared vocabulary
                }
                errs.push(format!(
                    "{pack}: tag group {} is already declared by {prev}",
                    g.id
                ));
                continue;
            }
            self.groups.insert(g.id.clone(), (pack.to_string(), g));
        }
        for t in b.tags {
            let in_reserved = t.group.as_deref() == Some(RESERVED_GROUP)
                || t.id.starts_with(&format!("{RESERVED_GROUP}."));
            if in_reserved && !economy_pack(pack) {
                errs.push(format!(
                    "{pack}: tag {} is in group {RESERVED_GROUP}, reserved for the economy pack",
                    t.id
                ));
                continue;
            }
            if t.id.is_empty() || t.id.contains(char::is_whitespace) {
                errs.push(format!(
                    "{pack}: tag {:?} needs a name with no spaces",
                    t.id
                ));
                continue;
            }
            if let Some((prev, had)) = self.tags.get(&t.id) {
                if *had == t {
                    continue; // the same declaration twice: a shared vocabulary
                }
                errs.push(format!(
                    "{pack}: tag {} is already declared differently by {prev}",
                    t.id
                ));
                continue;
            }
            self.tags.insert(t.id.clone(), (pack.to_string(), t));
        }
    }

    /// Named groups must exist.
    fn check_groups(&self, errs: &mut Vec<String>) {
        for (pack, t) in self.tags.values() {
            if let Some(g) = &t.group {
                if !self.groups.contains_key(g) {
                    errs.push(format!(
                        "{pack}: tag {} names group {g}, which no pack declares",
                        t.id
                    ));
                }
            }
        }
    }

    /// Error if setting `tag` on an object that already carries `have`
    /// would give it two tags of an exclusive group.
    pub fn exclusive_clash<'a>(
        &self,
        tag: &str,
        have: impl IntoIterator<Item = &'a String>,
    ) -> Option<String> {
        let g = self.group_of(tag).filter(|g| g.exclusive)?;
        let other = have
            .into_iter()
            .find(|t| t.as_str() != tag && self.group_of(t).is_some_and(|h| h.id == g.id))?;
        Some(format!(
            "tag group {} is exclusive: the object already carries {other}",
            g.id
        ))
    }

    /// The block the campaign pack would export: its own declarations only.
    pub fn block_of(&self, pack: &str) -> TagBlock {
        TagBlock {
            groups: self
                .groups
                .values()
                .filter(|(p, _)| p == pack)
                .map(|(_, g)| g.clone())
                .collect(),
            tags: self
                .tags
                .values()
                .filter(|(p, _)| p == pack)
                .map(|(_, t)| t.clone())
                .collect(),
        }
    }
}

/// Every declaration in the stack (bottom to top).
pub fn declarations(ms: &[sse_pack::PackManifest]) -> Result<TagSet, Vec<String>> {
    let mut set = TagSet::default();
    let mut errs = vec![];
    for p in ms {
        if let Some(v) = p.provides.other.get("tags") {
            match serde_json::to_value(v).and_then(serde_json::from_value::<TagBlock>) {
                Ok(b) => set.add(&p.id, b, &mut errs),
                Err(e) => errs.push(format!("{}: tags: {e}", p.id)),
            }
        }
    }
    set.check_groups(&mut errs);
    if errs.is_empty() {
        Ok(set)
    } else {
        Err(errs)
    }
}

/// Every tag the stack's packs name, wherever they name one: a `tag` or
/// `*_tag` text, a `tags` list (or a view's `{mode, list}`), a disclosure's
/// `tags` table, a `tag.<name>` field, key or condition operand. Grant
/// patterns with `{audience}` in them are templates, not names.
pub fn references(ms: &[sse_pack::PackManifest]) -> Vec<TagRef> {
    let mut out = vec![];
    for p in ms {
        for (key, reader) in READERS {
            let Some(v) = p.provides.other.get(key) else {
                continue;
            };
            let Ok(j) = serde_json::to_value(v) else {
                continue;
            };
            walk(&j, &p.id, reader, key.to_string(), None, &mut out);
        }
    }
    out
}

fn walk(
    v: &serde_json::Value,
    pack: &str,
    reader: &'static str,
    path: String,
    item: Option<&str>,
    out: &mut Vec<TagRef>,
) {
    use serde_json::Value as J;
    match v {
        J::Object(m) => {
            let here = m.get("id").and_then(J::as_str).or(item);
            for (k, x) in m {
                let at = format!("{path}.{k}");
                let mut found: Vec<&str> = vec![];
                match (k.as_str(), x) {
                    (k, J::String(t)) if k == "tag" || k.ends_with("_tag") => found.push(t),
                    ("tags", J::Array(ts)) => found.extend(ts.iter().filter_map(J::as_str)),
                    ("tags", J::Object(o)) => {
                        if let Some(J::Array(ts)) = o.get("list") {
                            found.extend(ts.iter().filter_map(J::as_str));
                        } else if !o.contains_key("mode") {
                            // A disclosure's tag tiers: tag name to tier.
                            found.extend(o.keys().map(String::as_str));
                        }
                    }
                    (_, J::String(s)) if s.starts_with("tag.") => found.push(&s[4..]),
                    _ => {}
                }
                if let Some(t) = k.strip_prefix("tag.") {
                    found.push(t);
                }
                for tag in found {
                    if tag.contains('{') || tag.is_empty() {
                        continue;
                    }
                    out.push(TagRef {
                        pack: pack.to_string(),
                        reader,
                        item: here.unwrap_or(&at).to_string(),
                        tag: tag.to_string(),
                    });
                }
                walk(x, pack, reader, at, here, out);
            }
        }
        J::Array(xs) => {
            for (i, x) in xs.iter().enumerate() {
                walk(x, pack, reader, format!("{path}[{i}]"), item, out);
            }
        }
        _ => {}
    }
}

/// The load-time check: one message per undeclared reference (the same
/// tag named twice by one item is one message).
pub fn check(refs: &[TagRef], set: &TagSet) -> Vec<String> {
    let mut seen: Vec<(String, String, String)> = vec![];
    let mut out = vec![];
    for r in refs {
        if set.declared(&r.tag) {
            continue;
        }
        let key = (r.pack.clone(), r.item.clone(), r.tag.clone());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        out.push(format!(
            "{}: {} {} names tag {}, which no pack in the stack declares (provides.other.tags)",
            r.pack, r.reader, r.item, r.tag
        ));
    }
    out
}

/// Kept kinds in the reserved economy prefix, outside the economy pack.
pub fn reserved_kept(ms: &[sse_pack::PackManifest]) -> Vec<String> {
    let mut out = vec![];
    for p in ms.iter().filter(|p| !economy_pack(&p.id)) {
        let Some(v) = p.provides.other.get("record") else {
            continue;
        };
        let Ok(j) = serde_json::to_value(v) else {
            continue;
        };
        for k in j["kept"].as_array().into_iter().flatten() {
            if let Some(id) = k["id"]
                .as_str()
                .filter(|i| i.starts_with(RESERVED_KEPT_PREFIX))
            {
                out.push(format!(
                    "{}: kept kind {id}: the {RESERVED_KEPT_PREFIX} prefix is reserved for the economy pack",
                    p.id
                ));
            }
        }
    }
    out
}
