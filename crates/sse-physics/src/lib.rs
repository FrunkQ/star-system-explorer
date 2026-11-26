//! The physics in force (track ER, package ER1; design
//! `handover/design/SSE4-edits-reach-design.md` sections 3 and 4).
//!
//! When a stack loads, the kernel builds one unchanging [`Physics`] from it:
//! the stacked data sheet, the block library, the compiled flows, each
//! crate's settings block as this stack has it, and a physics hash over
//! only what can change a worked-out number. [`with`] puts it in force for
//! the length of one kernel call; the two older scopes (the constants handle
//! and the sheet for the Look up block) are set from it, so they are views
//! of this one object.
//!
//! ER1 changes no reader. Compiled code still reads through the old doors;
//! moving each read here is ER2 (blocks), ER4 and ER5 (the sheet) and ER6
//! (settings), and the side-door ledger (`scripts/audit-side-doors.py`)
//! counts what is left and may only shrink.

use serde::Serialize;
use sse_core::cbor::Value;
use sse_core::constants::Constants;
use sse_core::ContentHash;
use sse_flow::sheet::Sheet;
use sse_flow::Flow;
use sse_formula::FormulaAtom;
use sse_pack::{PackManifest, ResolvedStack};
use sse_reference::Reference;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

/// `provides.other` keys that can change no worked-out number: words,
/// credits, how screens look. They are left out of the physics hash, so
/// adding a name scheme throws away no physics. Anything not listed here
/// counts as physics: a key wrongly counted costs only a cache refill, a
/// key wrongly left out would keep a wrong answer.
pub const NOT_PHYSICS: &[&str] = &[
    "about",
    "credits",
    "holding_cards",
    "lenses",
    "names",
    "notice",
    "overlays",
    "views",
    "vocabulary",
];

/// Whether a `provides.other` key is part of the physics.
pub fn is_physics_key(key: &str) -> bool {
    !NOT_PHYSICS.contains(&key)
}

/// The physics in force: built once when a stack loads, never changed.
pub struct Physics {
    sheet: Arc<Reference>,
    flows: Arc<BTreeMap<String, Flow>>,
    /// Every `provides.other` block by key: (pack id, value), bottom first.
    settings: BTreeMap<String, Vec<(String, Value)>>,
    hash: ContentHash,
}

impl std::fmt::Debug for Physics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Physics")
            .field("hash", &self.hash.hex())
            .field("flows", &self.flows.len())
            .field("settings", &self.settings.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Serialize)]
struct HashInput<'a> {
    format: &'static str,
    sheet: &'a BTreeMap<String, BTreeMap<String, serde_json::Map<String, serde_json::Value>>>,
    units: Vec<&'a sse_core::quantity::Unit>,
    kinds: Vec<&'a sse_core::QuantityKind>,
    blocks: &'a str,
    flows: BTreeMap<&'a str, &'a sse_flow::FlowDoc>,
    boxes: &'a BTreeMap<String, sse_flow::BoxDoc>,
    settings: BTreeMap<&'a str, Vec<&'a Value>>,
}

/// The hash of the shipped block library: every formula file's text and
/// every block's id. It is the same on every device (no physics edit can
/// reach it until ER3 loads blocks from the stack).
pub fn blocks_hash() -> &'static str {
    static H: OnceLock<String> = OnceLock::new();
    H.get_or_init(|| {
        let ids: Vec<String> = sse_atoms::atoms()
            .iter()
            .map(|a| format!("{}@{}", a.decl().id, a.decl().version))
            .collect();
        let files: Vec<(&str, &str)> = sse_atoms::formulas::FILES.to_vec();
        ContentHash::of(&(files, ids))
            .expect("the block library hashes")
            .hex()
    })
}

impl Physics {
    /// The physics before any stack has loaded: the given sheet (usually
    /// empty), the shipped blocks, no flows and no settings.
    pub fn bare(sheet: Arc<Reference>) -> Self {
        Self::build(&[], None, sheet, Arc::new(BTreeMap::new()))
    }

    /// Build the physics in force from a loaded stack (bottom first), its
    /// resolution, the stacked sheet and the compiled flows.
    pub fn build(
        packs: &[PackManifest],
        resolved: Option<&ResolvedStack>,
        sheet: Arc<Reference>,
        flows: Arc<BTreeMap<String, Flow>>,
    ) -> Self {
        let mut settings: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
        for p in packs {
            for (k, v) in &p.provides.other {
                settings
                    .entry(k.clone())
                    .or_default()
                    .push((p.id.clone(), v.clone()));
            }
        }
        let empty_boxes = BTreeMap::new();
        let input = HashInput {
            format: "sse.physics/1",
            sheet: &sheet.tables,
            units: packs.iter().flat_map(|p| &p.provides.units).collect(),
            kinds: packs
                .iter()
                .flat_map(|p| &p.provides.quantity_kinds)
                .collect(),
            blocks: blocks_hash(),
            flows: resolved
                .map(|r| r.flows.iter().map(|(k, f)| (k.as_str(), &f.doc)).collect())
                .unwrap_or_default(),
            boxes: resolved.map(|r| &r.boxes).unwrap_or(&empty_boxes),
            settings: settings
                .iter()
                .filter(|(k, _)| is_physics_key(k))
                .map(|(k, vs)| (k.as_str(), vs.iter().map(|(_, v)| v).collect()))
                .collect(),
        };
        let hash = ContentHash::of(&input).expect("the physics hashes");
        Physics {
            sheet,
            flows,
            settings,
            hash,
        }
    }

    /// The physics hash: covers only what can change a worked-out number
    /// (the sheet's tables, units and kinds, the blocks, flows and boxes,
    /// and every settings block not in [`NOT_PHYSICS`]).
    pub fn hash(&self) -> ContentHash {
        self.hash
    }

    /// The stacked data sheet.
    pub fn sheet(&self) -> &Arc<Reference> {
        &self.sheet
    }

    /// The sheet as a Look up block reads it; none when the stack has no
    /// master data sheet, so the block says so instead of finding no rows.
    pub fn sheet_door(&self) -> Option<Arc<dyn Sheet>> {
        (!self.sheet.tables.is_empty()).then(|| self.sheet.clone() as Arc<dyn Sheet>)
    }

    /// The constants handle the sheet gives (undated).
    pub fn constants(&self) -> Constants {
        self.sheet.handle()
    }

    /// The block library in force (the shipped formula blocks until ER3).
    pub fn blocks(&self) -> &'static [FormulaAtom] {
        sse_atoms::formulas::loaded()
    }

    /// The compiled flows of the stack, by id.
    pub fn flows(&self) -> &Arc<BTreeMap<String, Flow>> {
        &self.flows
    }

    /// One compiled flow of the stack.
    pub fn flow(&self, id: &str) -> Option<&Flow> {
        self.flows.get(id)
    }

    /// A settings block as this stack has it: the topmost pack's
    /// `provides.other.<key>`.
    pub fn settings(&self, key: &str) -> Option<&Value> {
        self.settings
            .get(key)
            .and_then(|vs| vs.last())
            .map(|(_, v)| v)
    }

    /// Every pack's block under `key`, bottom first, with the pack's id.
    pub fn settings_all(&self, key: &str) -> &[(String, Value)] {
        self.settings.get(key).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The settings keys this stack provides.
    pub fn settings_keys(&self) -> impl Iterator<Item = &str> {
        self.settings.keys().map(String::as_str)
    }
}

thread_local! {
    static NOW: RefCell<Option<Arc<Physics>>> = const { RefCell::new(None) };
}

/// The physics in force, if a call has put one in force.
pub fn now() -> Option<Arc<Physics>> {
    NOW.with(|n| n.borrow().clone())
}

/// A sheet constant in force by its sheet id (`"M_sun"`, `"G"`), read from
/// the stack's sheet; none outside a call or when the sheet lacks the row.
/// The door for a new constant read until ER4's fast table lands.
pub fn constant(id: &str) -> Option<f64> {
    now().and_then(|p| p.sheet().constant(id))
}

/// A settings block in force (the topmost pack's `provides.other.<key>`);
/// none outside a call or when the stack has none. The door for a new
/// settings read: parse it as the crate's own `from_packs` does.
pub fn settings(key: &str) -> Option<Value> {
    now().and_then(|p| p.settings(key).cloned())
}

/// Run `f` with `p` in force, and with its two views set: the constants
/// handle (`c`, which the kernel reads at the call's date) and the sheet
/// for the Look up block. Everything is restored afterwards, also on a panic.
pub fn with<R>(p: &Arc<Physics>, c: Constants, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<Arc<Physics>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let back = self.0.take();
            NOW.with(|n| *n.borrow_mut() = back);
        }
    }
    let _r = Restore(NOW.with(|n| n.replace(Some(p.clone()))));
    sse_core::constants::with(c, || sse_flow::sheet::with(p.sheet_door(), f))
}

#[cfg(test)]
mod tests;
