//! The block door (track ER, package ER2; edits-reach design 3.1 and 4).
//!
//! The block library in force: every shipped block, with the settings this
//! stack gives it. A pack edits a block with `provides.blocks` (id, new
//! settings, and what it replaces); the edit applies everywhere the block
//! runs in this universe (Alex, 26 Nov, ruling 4). A value typed on one node
//! of one flow still wins there, because a flow is compiled against the
//! library in force and its own node settings are applied on top.
//!
//! Compiled code runs a block through [`crate::run`]; flows are compiled
//! against [`crate::registry`]. Only settings change here: a block's formula
//! comes from the stack in ER3.

use semver::{Version, VersionReq};
use sse_flow::{Atom, AtomDecl, AtomError, AtomRegistry, Call, DimRule, Out, Port, Val};
use sse_pack::PackManifest;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The pack the shipped blocks belong to (their formula files live in it).
pub const SHIPPED_BY: &str = "sse.base-physics";

/// One setting a stack changed: which block, which setting, the shipped
/// value, the value in force, and the pack that set it.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingEdit {
    pub block: String,
    pub setting: String,
    pub shipped: f64,
    pub value: f64,
    pub pack: String,
}

/// The block library in force.
pub struct Blocks {
    by_id: BTreeMap<String, Arc<dyn Atom>>,
    edits: Vec<SettingEdit>,
}

impl std::fmt::Debug for Blocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blocks")
            .field("blocks", &self.by_id.len())
            .field("edits", &self.edits)
            .finish()
    }
}

/// A block in force: the shipped block with a face whose setting defaults
/// are the values in force (the shipped ones where nothing edited it). A
/// caller that passes no settings gets the ones in force, so
/// `eval(inputs, &[])` is always "run it as this universe has it".
struct InForce {
    inner: Box<dyn Atom>,
    decl: AtomDecl,
}

impl InForce {
    fn settings<'a>(&'a self, params: &'a [f64]) -> std::borrow::Cow<'a, [f64]> {
        if params.is_empty() && !self.decl.params.is_empty() {
            self.decl.params.iter().map(|p| p.default).collect()
        } else {
            std::borrow::Cow::Borrowed(params)
        }
    }
}

impl Atom for InForce {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, inputs: &[f64], params: &[f64]) -> Result<Vec<f64>, AtomError> {
        self.inner.eval(inputs, &self.settings(params))
    }
    fn eval_typed(&self, inputs: &[Val], params: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        self.inner.eval_typed(inputs, &self.settings(params), cx)
    }
    fn ports(&self, config: Option<&serde_json::Value>) -> Option<(Vec<Port>, Vec<Port>)> {
        self.inner.ports(config)
    }
    fn dim_rule(&self, config: Option<&serde_json::Value>) -> Option<DimRule> {
        self.inner.dim_rule(config)
    }
    fn check_wiring(
        &self,
        config: Option<&serde_json::Value>,
        sources: &[Option<&str>],
    ) -> Result<(), String> {
        self.inner.check_wiring(config, sources)
    }
    fn check_config(&self, config: Option<&serde_json::Value>) -> Result<(), String> {
        self.inner.check_config(config)
    }
}

/// A block of the library in force, handed to a registry (which owns boxes).
struct Shared(Arc<dyn Atom>);

impl Atom for Shared {
    fn decl(&self) -> &AtomDecl {
        self.0.decl()
    }
    fn eval(&self, inputs: &[f64], params: &[f64]) -> Result<Vec<f64>, AtomError> {
        self.0.eval(inputs, params)
    }
    fn eval_typed(&self, inputs: &[Val], params: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        self.0.eval_typed(inputs, params, cx)
    }
    fn ports(&self, config: Option<&serde_json::Value>) -> Option<(Vec<Port>, Vec<Port>)> {
        self.0.ports(config)
    }
    fn dim_rule(&self, config: Option<&serde_json::Value>) -> Option<DimRule> {
        self.0.dim_rule(config)
    }
    fn check_wiring(
        &self,
        config: Option<&serde_json::Value>,
        sources: &[Option<&str>],
    ) -> Result<(), String> {
        self.0.check_wiring(config, sources)
    }
    fn check_config(&self, config: Option<&serde_json::Value>) -> Result<(), String> {
        self.0.check_config(config)
    }
}

impl Blocks {
    /// The shipped library, as built.
    pub fn shipped() -> Self {
        Self::from_packs(&[]).expect("no edits, nothing to refuse")
    }

    /// The library in force for a stack (bottom first): the shipped blocks
    /// with every pack's `provides.blocks` applied in order. Every problem
    /// is collected, as `(pack id, message)`, before refusing.
    pub fn from_packs(packs: &[PackManifest]) -> Result<Self, Vec<(String, String)>> {
        let mut decls: BTreeMap<String, AtomDecl> = BTreeMap::new();
        let mut owner: BTreeMap<String, String> = BTreeMap::new();
        let mut from: BTreeMap<(String, String), String> = BTreeMap::new();
        let mut shipped: BTreeMap<String, Box<dyn Atom>> = BTreeMap::new();
        for a in sse_atoms::atoms() {
            shipped.insert(a.decl().id.clone(), a);
        }
        let mut issues = vec![];
        let mut seen: BTreeMap<&str, Version> = BTreeMap::new();
        for p in packs {
            for e in &p.provides.blocks {
                let mut bad = |m: String| issues.push((p.id.clone(), m));
                let Some(a) = shipped.get(&e.id) else {
                    bad(format!("edits block {}, but no block has that id", e.id));
                    continue;
                };
                let now_from = owner.get(&e.id).map_or(SHIPPED_BY, String::as_str);
                if now_from == p.id {
                    bad(format!("edits block {} twice; give it one entry", e.id));
                    continue;
                }
                if e.replaces.pack != now_from {
                    bad(format!(
                        "edits block {} from {}, but it comes from {now_from}",
                        e.id, e.replaces.pack
                    ));
                    continue;
                }
                match (VersionReq::parse(&e.replaces.range), seen.get(now_from)) {
                    (Err(err), _) => {
                        bad(format!(
                            "edits block {}: range {:?}: {err}",
                            e.id, e.replaces.range
                        ));
                        continue;
                    }
                    (_, None) => {
                        bad(format!(
                            "edits block {} from {now_from}, which is not lower in the stack",
                            e.id
                        ));
                        continue;
                    }
                    (Ok(req), Some(v)) if !req.matches(v) => {
                        bad(format!(
                            "edits block {} from {now_from} {}, but the stack has {v}",
                            e.id, e.replaces.range
                        ));
                        continue;
                    }
                    _ => {}
                }
                let d = decls
                    .entry(e.id.clone())
                    .or_insert_with(|| a.decl().clone());
                let mut ok = true;
                for (name, v) in &e.settings {
                    let Some(k) = d.params.iter().position(|q| &q.name == name) else {
                        let have: Vec<&str> = d.params.iter().map(|q| q.name.as_str()).collect();
                        let have = if have.is_empty() {
                            "it has no settings".to_string()
                        } else {
                            format!("its settings are {}", have.join(", "))
                        };
                        bad(format!(
                            "edits block {}: no setting {name:?} ({have})",
                            e.id
                        ));
                        ok = false;
                        continue;
                    };
                    let q = &mut d.params[k];
                    if !v.is_finite() {
                        bad(format!("edits block {}: {name} must be a number", e.id));
                        ok = false;
                        continue;
                    }
                    if let Some((lo, hi)) = q.range {
                        if *v < lo || *v > hi {
                            bad(format!(
                                "edits block {}: {name} {v} is outside its range {lo} to {hi}",
                                e.id
                            ));
                            ok = false;
                            continue;
                        }
                    }
                    q.default = *v;
                    from.insert((e.id.clone(), name.clone()), p.id.clone());
                }
                if ok {
                    owner.insert(e.id.clone(), p.id.clone());
                }
            }
            if let Ok(v) = Version::parse(&p.version) {
                seen.insert(p.id.as_str(), v);
            }
        }
        if !issues.is_empty() {
            return Err(issues);
        }
        let mut edits = vec![];
        let mut by_id: BTreeMap<String, Arc<dyn Atom>> = BTreeMap::new();
        for (id, inner) in shipped {
            let decl = decls.remove(&id).unwrap_or_else(|| inner.decl().clone());
            for (q, s) in decl.params.iter().zip(&inner.decl().params) {
                if let Some(pack) = from.get(&(id.clone(), q.name.clone())) {
                    edits.push(SettingEdit {
                        block: id.clone(),
                        setting: q.name.clone(),
                        shipped: s.default,
                        value: q.default,
                        pack: pack.clone(),
                    });
                }
            }
            by_id.insert(id, Arc::new(InForce { inner, decl }));
        }
        Ok(Blocks { by_id, edits })
    }

    /// One block in force by id.
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Atom>> {
        self.by_id.get(id)
    }

    /// Every block in force, by id.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Arc<dyn Atom>)> {
        self.by_id.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// The settings this stack changed, by block then setting.
    pub fn edits(&self) -> &[SettingEdit] {
        &self.edits
    }

    /// Put every block in force into `r` (a registry that also holds the
    /// kernel's logic and toy blocks, say).
    pub fn insert_into(&self, r: &mut AtomRegistry) {
        for a in self.by_id.values() {
            r.insert(Box::new(Shared(a.clone())));
        }
    }

    /// The blocks in force as a registry, for compiling a flow.
    pub fn registry(&self) -> AtomRegistry {
        let mut r = AtomRegistry::default();
        self.insert_into(&mut r);
        r
    }

    /// Run one block with its settings in force: its outputs in port order.
    pub fn run(&self, id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
        let a = self.get(id).ok_or_else(|| format!("no block {id}"))?;
        a.eval(inputs, &[]).map_err(|e| format!("{id}: {e}"))
    }
}
