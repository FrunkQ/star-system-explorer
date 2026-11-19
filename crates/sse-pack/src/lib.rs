//! SSE4 packs (H7): one declarative format, several roles, stacked in a
//! declared order. This crate validates a stack and resolves it into the
//! active quantity kinds, units and flows, with exactly one publisher per
//! (hook, quantity kind), explicit overrides only, and fictional labels that
//! propagate to everything downstream.
#![forbid(unsafe_code)]

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::quantity::Unit;
use sse_core::{ContentHash, QuantityKind};
use sse_flow::FlowDoc;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const PACK_FORMAT: &str = "sse.pack/1";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackKind {
    BasePhysics,
    Physics,
    Universe,
    GameSystem,
    Presentation,
    Suite,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    /// Semver range, e.g. "^0.1".
    pub range: String,
}

/// An explicit override of a lower pack's flow or box (`replaces: id@range`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Replacement {
    /// Id of the flow or box being replaced (they share one namespace).
    pub flow: String,
    /// Pack that provides it.
    pub pack: String,
    /// Versions of that pack this override is written against.
    pub range: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Provides {
    #[serde(default)]
    pub quantity_kinds: Vec<QuantityKind>,
    #[serde(default)]
    pub units: Vec<Unit>,
    #[serde(default)]
    pub flows: Vec<FlowDoc>,
    /// Boxes (J3): flows of blocks used as one block, `id@version`. Left out
    /// when empty, so a pack without boxes hashes as before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boxes: Vec<sse_flow::BoxDoc>,
    /// Content not yet interpreted by the kernel (rules, sifters, data views,
    /// screens, skins, vocab, catalogues...). Carried and hashed.
    #[serde(default)]
    pub other: BTreeMap<String, Value>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PackManifest {
    pub format: String,
    /// Namespaced: `publisher.name`.
    pub id: String,
    pub version: String,
    pub kind: Vec<PackKind>,
    #[serde(default)]
    pub requires: Vec<Requirement>,
    pub kernel_range: String,
    /// Invented physics: everything downstream is labelled fictional.
    #[serde(default)]
    pub fictional: bool,
    #[serde(default)]
    pub replaces: Vec<Replacement>,
    #[serde(default)]
    pub provides: Provides,
    #[serde(default)]
    pub seed_namespace: Option<String>,
    pub licence: String,
}

impl PackManifest {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("pack manifest: {e}"))
    }
    pub fn content_hash(&self) -> ContentHash {
        ContentHash::of(self).expect("pack holds no NaN")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StackIssue {
    pub pack: String,
    pub message: String,
}
impl fmt::Display for StackIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pack, self.message)
    }
}

/// Human-readable report of a rejected stack.
pub fn report(issues: &[StackIssue]) -> String {
    let mut s = format!(
        "Pack stack rejected: {} problem{}\n",
        issues.len(),
        if issues.len() == 1 { "" } else { "s" }
    );
    for (i, x) in issues.iter().enumerate() {
        s += &format!("  {}. [{}] {}\n", i + 1, x.pack, x.message);
    }
    s
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct ActiveFlow {
    pub pack: String,
    pub doc: FlowDoc,
}

/// Who publishes a quantity kind under a hook.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct PublisherRef {
    pub pack: String,
    pub flow: String,
}

#[derive(Clone, Debug)]
pub struct ResolvedStack {
    /// Bottom to top: (id, version, content hash).
    pub packs: Vec<(String, String, ContentHash)>,
    pub hash: ContentHash,
    pub kinds: BTreeMap<String, QuantityKind>,
    pub units: sse_core::Units,
    /// Active flows by flow id.
    pub flows: BTreeMap<String, ActiveFlow>,
    /// (hook, quantity kind) -> publisher.
    pub publishers: BTreeMap<(String, String), PublisherRef>,
    /// Quantity kinds whose value depends on fictional physics.
    pub fictional_kinds: BTreeSet<String>,
    /// Active boxes by id (J3), and the pack each comes from.
    pub boxes: BTreeMap<String, sse_flow::BoxDoc>,
    pub box_packs: BTreeMap<String, String>,
}

impl ResolvedStack {
    pub fn publisher_of(&self, hook: &str, kind: &str) -> Option<&PublisherRef> {
        self.publishers.get(&(hook.to_string(), kind.to_string()))
    }
    pub fn is_fictional(&self, kind: &str) -> bool {
        self.fictional_kinds.contains(kind)
    }
}

fn valid_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('.').collect();
    parts.len() >= 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
        })
}

/// Resolve a stack (bottom first). Collects every problem before failing.
pub fn resolve(
    stack: &[PackManifest],
    kernel_version: &str,
) -> Result<ResolvedStack, Vec<StackIssue>> {
    let mut issues = Vec::new();
    let mut issue = |pack: &str, m: String| {
        issues.push(StackIssue {
            pack: pack.into(),
            message: m,
        })
    };
    let kernel = Version::parse(kernel_version).expect("kernel version is semver");
    let mut seen: BTreeMap<String, Version> = BTreeMap::new();
    let mut kinds: BTreeMap<String, (QuantityKind, String)> = BTreeMap::new();
    let mut units = sse_core::Units::standard();
    let mut unit_owner: BTreeMap<String, String> = BTreeMap::new();
    let mut flows: BTreeMap<String, ActiveFlow> = BTreeMap::new();
    let mut boxes: BTreeMap<String, sse_flow::BoxDoc> = BTreeMap::new();
    let mut box_packs: BTreeMap<String, String> = BTreeMap::new();
    let mut fictional_packs = BTreeSet::new();

    for p in stack {
        let pid = p.id.as_str();
        if p.format != PACK_FORMAT {
            issue(pid, format!("format must be {PACK_FORMAT:?}"));
        }
        if !valid_id(pid) {
            issue(
                pid,
                "pack id must be namespaced lowercase, like `publisher.name`".into(),
            );
        }
        let version = match Version::parse(&p.version) {
            Ok(v) => v,
            Err(e) => {
                issue(pid, format!("version {:?} is not semver: {e}", p.version));
                continue;
            }
        };
        match VersionReq::parse(&p.kernel_range) {
            Ok(r) if r.matches(&kernel) => {}
            Ok(_) => issue(
                pid,
                format!("needs kernel {}, this kernel is {kernel}", p.kernel_range),
            ),
            Err(e) => issue(pid, format!("kernel_range {:?}: {e}", p.kernel_range)),
        }
        for r in &p.requires {
            match (VersionReq::parse(&r.range), seen.get(&r.id)) {
                (Err(e), _) => issue(pid, format!("requires {} {:?}: {e}", r.id, r.range)),
                (_, None) => issue(
                    pid,
                    format!(
                        "requires {} {}, which is not lower in the stack",
                        r.id, r.range
                    ),
                ),
                (Ok(req), Some(v)) if !req.matches(v) => issue(
                    pid,
                    format!("requires {} {}, but the stack has {v}", r.id, r.range),
                ),
                _ => {}
            }
        }
        if seen.insert(p.id.clone(), version).is_some() {
            issue(pid, "appears twice in the stack".into());
        }
        if p.fictional {
            fictional_packs.insert(p.id.clone());
        }
        for k in &p.provides.quantity_kinds {
            match kinds.get(&k.id) {
                Some((old, owner)) if old.dim != k.dim => {
                    issue(
                        pid,
                        format!(
                            "redefines quantity kind {} as [{}], but {owner} defines it as [{}]",
                            k.id, k.dim, old.dim
                        ),
                    );
                }
                _ => {
                    kinds.insert(k.id.clone(), (k.clone(), p.id.clone()));
                }
            }
        }
        for u in &p.provides.units {
            if units.get(&u.id).is_ok_and(|x| x != u) {
                let owner = unit_owner
                    .get(&u.id)
                    .map_or("the built-in units", String::as_str);
                issue(
                    pid,
                    format!("redefines unit {} already defined by {owner}", u.id),
                );
                continue;
            }
            unit_owner.insert(u.id.clone(), p.id.clone());
            units.insert(u.clone());
        }
        // explicit replacements first
        let mut replaced_here = BTreeSet::new();
        for r in &p.replaces {
            let from = flows
                .get(&r.flow)
                .map(|a| a.pack.clone())
                .or_else(|| box_packs.get(&r.flow).cloned());
            match (from, VersionReq::parse(&r.range)) {
                (_, Err(e)) => issue(pid, format!("replaces {}@{}: {e}", r.flow, r.range)),
                (None, _) => issue(
                    pid,
                    format!(
                        "replaces {} from {}, but no lower pack provides a flow or box of that id",
                        r.flow, r.pack
                    ),
                ),
                (Some(owner), Ok(req)) => {
                    if owner != r.pack {
                        issue(
                            pid,
                            format!(
                                "replaces {} from {}, but it comes from {owner}",
                                r.flow, r.pack
                            ),
                        );
                    } else if !seen.get(&r.pack).is_some_and(|v| req.matches(v)) {
                        issue(
                            pid,
                            format!(
                                "replaces {}@{}, but the stack has {} {}",
                                r.flow, r.range, r.pack, seen[&r.pack]
                            ),
                        );
                    } else {
                        flows.remove(&r.flow);
                        boxes.remove(&r.flow);
                        box_packs.remove(&r.flow);
                        replaced_here.insert(r.flow.clone());
                    }
                }
            }
        }
        for b in &p.provides.boxes {
            if let Some(old) = box_packs
                .get(&b.id)
                .or_else(|| flows.get(&b.id).map(|f| &f.pack))
            {
                issue(pid, format!("provides box {} already provided by {old}; add `replaces` to override it on purpose", b.id));
                continue;
            }
            if b.format != sse_flow::boxes::BOX_FORMAT {
                issue(
                    pid,
                    format!(
                        "box {}: format must be {:?}",
                        b.id,
                        sse_flow::boxes::BOX_FORMAT
                    ),
                );
            }
            if Version::parse(&b.version).is_err() {
                issue(
                    pid,
                    format!("box {} version {:?} is not semver", b.id, b.version),
                );
            }
            boxes.insert(b.id.clone(), b.clone());
            box_packs.insert(b.id.clone(), p.id.clone());
        }
        for f in &p.provides.flows {
            if let Some(old) = flows
                .get(&f.id)
                .map(|f| &f.pack)
                .or_else(|| box_packs.get(&f.id))
            {
                issue(pid, format!("provides flow {} already provided by {old}; add `replaces` to override it on purpose", f.id));
                continue;
            }
            flows.insert(
                f.id.clone(),
                ActiveFlow {
                    pack: p.id.clone(),
                    doc: f.clone(),
                },
            );
        }
    }

    // one publisher per (hook, kind) across the whole stack
    let mut publishers: BTreeMap<(String, String), PublisherRef> = BTreeMap::new();
    for (fid, af) in &flows {
        for pubd in &af.doc.publishes {
            let key = (af.doc.hook.clone(), pubd.kind.clone());
            if let Some(other) = publishers.get(&key) {
                issue(
                    &af.pack,
                    format!(
                        "flow {fid} publishes {} at hook {}, already published by flow {} from {}; only one publisher is allowed (use `replaces`)",
                        pubd.kind, af.doc.hook, other.flow, other.pack
                    ),
                );
            } else {
                publishers.insert(
                    key,
                    PublisherRef {
                        pack: af.pack.clone(),
                        flow: fid.clone(),
                    },
                );
            }
            if !kinds.contains_key(&pubd.kind) {
                issue(
                    &af.pack,
                    format!("flow {fid} publishes unknown quantity kind {}", pubd.kind),
                );
            }
        }
    }

    // fictional propagation to a fixed point: a flow's outputs are fictional if
    // its pack is fictional or any of its inputs' kinds are.
    let mut fictional_kinds = BTreeSet::new();
    loop {
        let before = fictional_kinds.len();
        for af in flows.values() {
            let tainted = fictional_packs.contains(&af.pack)
                || uses_fictional_box(&af.doc.nodes, &boxes, &box_packs, &fictional_packs, 0)
                || af
                    .doc
                    .inputs
                    .iter()
                    .any(|i| fictional_kinds.contains(&i.kind));
            if tainted {
                for p in &af.doc.publishes {
                    fictional_kinds.insert(p.kind.clone());
                }
            }
        }
        if fictional_kinds.len() == before {
            break;
        }
    }

    if !issues.is_empty() {
        return Err(issues);
    }
    let packs: Vec<(String, String, ContentHash)> = stack
        .iter()
        .map(|p| (p.id.clone(), p.version.clone(), p.content_hash()))
        .collect();
    let hash = ContentHash::of(&packs).expect("canonical");
    Ok(ResolvedStack {
        packs,
        hash,
        kinds: kinds.into_iter().map(|(k, (v, _))| (k, v)).collect(),
        units,
        flows,
        publishers,
        fictional_kinds,
        boxes,
        box_packs,
    })
}

/// Does any box these nodes use (at any depth) carry invented physics: a box
/// flagged `fictional`, or one from a fictional pack?
fn uses_fictional_box(
    nodes: &[sse_flow::flow::NodeDoc],
    boxes: &BTreeMap<String, sse_flow::BoxDoc>,
    packs: &BTreeMap<String, String>,
    fictional_packs: &BTreeSet<String>,
    depth: usize,
) -> bool {
    if depth > sse_flow::boxes::MAX_DEPTH {
        return false; // reported when the flow is compiled
    }
    nodes.iter().any(|n| {
        let Some((id, _)) = n
            .r#box
            .as_deref()
            .and_then(|r| sse_flow::boxes::parse_ref(r).ok())
        else {
            return false;
        };
        let Some(b) = boxes.get(&id) else {
            return false;
        };
        b.fictional
            || packs.get(&id).is_some_and(|p| fictional_packs.contains(p))
            || uses_fictional_box(&b.nodes, boxes, packs, fictional_packs, depth + 1)
    })
}

#[cfg(test)]
mod tests;
