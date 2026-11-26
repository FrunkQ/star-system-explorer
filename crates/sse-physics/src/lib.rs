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
//! ER2 adds the block door: the block library in force ([`Blocks`], each
//! block with the settings this stack gives it through `provides.blocks`),
//! [`run`] to run one, [`registry`] to compile a flow against them, and
//! [`memo`] for a value worked out from them once per physics. Compiled code
//! still reads the sheet and settings through the old doors; moving those is
//! ER4 and ER5 (the sheet) and ER6 (settings), and the side-door ledger
//! (`scripts/audit-side-doors.py`) counts what is left and may only shrink.

use serde::Serialize;
use sse_core::cbor::Value;
use sse_core::constants::Constants;
use sse_core::ContentHash;
use sse_flow::sheet::Sheet;
use sse_flow::Flow;
use sse_formula::FormulaAtom;
use sse_pack::{PackManifest, ResolvedStack};
use sse_reference::Reference;
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

mod blocks;
pub use blocks::{Blocks, SettingEdit, SHIPPED_BY};

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
    blocks: Arc<Blocks>,
    /// Values worked out once from this physics ([`memo`]); they die with it.
    memo: Mutex<BTreeMap<&'static str, Arc<dyn Any + Send + Sync>>>,
    hash: ContentHash,
}

impl std::fmt::Debug for Physics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Physics")
            .field("hash", &self.hash.hex())
            .field("flows", &self.flows.len())
            .field("settings", &self.settings.keys().collect::<Vec<_>>())
            .field("block_edits", &self.blocks.edits().len())
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
    /// Block edits (ER2), each pack's in stack order. Left out when there
    /// are none, so a stack without them keeps its physics hash.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    block_edits: Vec<(&'a str, &'a sse_pack::BlockEdit)>,
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
        Self::build_with(
            &[],
            None,
            sheet,
            Arc::new(BTreeMap::new()),
            shipped_blocks(),
        )
    }

    /// Build the physics in force from a loaded stack (bottom first), its
    /// resolution, the stacked sheet and the compiled flows. The block
    /// library is worked out from the packs; a stack whose block edits are
    /// refused panics here, so a loader checks them first with
    /// [`Blocks::from_packs`] and calls [`Physics::build_with`].
    pub fn build(
        packs: &[PackManifest],
        resolved: Option<&ResolvedStack>,
        sheet: Arc<Reference>,
        flows: Arc<BTreeMap<String, Flow>>,
    ) -> Self {
        let blocks = Blocks::from_packs(packs).unwrap_or_else(|e| panic!("block edits: {e:?}"));
        Self::build_with(packs, resolved, sheet, flows, Arc::new(blocks))
    }

    /// As [`Physics::build`], with the block library already worked out
    /// (the one the flows were compiled against).
    pub fn build_with(
        packs: &[PackManifest],
        resolved: Option<&ResolvedStack>,
        sheet: Arc<Reference>,
        flows: Arc<BTreeMap<String, Flow>>,
        blocks: Arc<Blocks>,
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
            block_edits: packs
                .iter()
                .flat_map(|p| p.provides.blocks.iter().map(move |b| (p.id.as_str(), b)))
                .collect(),
        };
        let hash = ContentHash::of(&input).expect("the physics hashes");
        Physics {
            sheet,
            flows,
            settings,
            blocks,
            memo: Mutex::new(BTreeMap::new()),
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

    /// The block library in force: every shipped block with the settings
    /// this stack gives it (the shipped formulas until ER3).
    pub fn blocks(&self) -> &Arc<Blocks> {
        &self.blocks
    }

    /// The shipped formula blocks as parsed, before any edit (their
    /// formulas; the settings in force are on [`Physics::blocks`]).
    pub fn formulas(&self) -> &'static [FormulaAtom] {
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

/// The shipped block library, built once.
fn shipped_blocks() -> Arc<Blocks> {
    static B: OnceLock<Arc<Blocks>> = OnceLock::new();
    B.get_or_init(|| Arc::new(Blocks::shipped())).clone()
}

/// The physics outside any call: no sheet, no flows, no settings, the
/// shipped blocks. What [`run`], [`registry`] and [`memo`] use in a test or
/// a tool that runs no kernel call.
fn shipped() -> Arc<Physics> {
    static P: OnceLock<Arc<Physics>> = OnceLock::new();
    P.get_or_init(|| Arc::new(Physics::bare(Arc::new(Reference::default()))))
        .clone()
}

/// The physics in force, or the shipped physics outside a call.
pub fn in_force() -> Arc<Physics> {
    now().unwrap_or_else(shipped)
}

/// Run one block by id with its settings in force (the block door, ER2):
/// its outputs in port order. Outside a call, the shipped block and its
/// shipped settings. Read once at the top of a stage, not in a hot loop,
/// where that is natural; the lookup is cheap but not free.
pub fn run(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    in_force().blocks.run(id, inputs)
}

/// One block in force by id (its face carries the settings in force), for
/// a caller that needs its ports or references as well as its maths.
pub fn block(id: &str) -> Option<Arc<dyn sse_flow::Atom>> {
    in_force().blocks.get(id).cloned()
}

/// The blocks in force as a registry, to compile a flow against: a flow
/// compiled this way gets each block's settings in force, with its own
/// node settings on top.
pub fn registry() -> sse_flow::AtomRegistry {
    in_force().blocks.registry()
}

/// A value worked out from the physics in force once and kept with it
/// (design section 4 rule 2: every cache names its physics). `key` names
/// the value and its type; the same key must always give the same type.
/// A new physics (an edit) starts with nothing kept.
pub fn memo<T: Send + Sync + 'static>(key: &'static str, make: impl FnOnce() -> T) -> Arc<T> {
    let p = in_force();
    if let Some(v) = p.memo.lock().expect("memo lock").get(key) {
        return v
            .clone()
            .downcast::<T>()
            .unwrap_or_else(|_| panic!("memo {key} kept with another type"));
    }
    // Worked out without the lock held, so `make` may itself use `memo`.
    let v: Arc<T> = Arc::new(make());
    let mut m = p.memo.lock().expect("memo lock");
    let kept = m
        .entry(key)
        .or_insert_with(|| v.clone() as Arc<dyn Any + Send + Sync>)
        .clone();
    kept.downcast::<T>()
        .unwrap_or_else(|_| panic!("memo {key} kept with another type"))
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

/// A guard that puts `p` in force (only this object, not the two older
/// views) until it is dropped, also on a panic. For a loader that has just
/// built a new physics and must read it for the rest of the load.
pub struct Scope(Option<Arc<Physics>>);

impl Scope {
    pub fn new(p: &Arc<Physics>) -> Self {
        Scope(NOW.with(|n| n.replace(Some(p.clone()))))
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        let back = self.0.take();
        NOW.with(|n| *n.borrow_mut() = back);
    }
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

#[cfg(test)]
mod block_tests;
