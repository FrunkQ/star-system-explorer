//! The door onto the master data sheet for blocks (matter pack design
//! section 3A, MP17a).
//!
//! The Look up block (`sse.reference.lookup`) and any heavy block that needs
//! a property of matter read it through [`Sheet`]; no block keeps its own
//! number about nature. The kernel puts the stack's sheet in force for the
//! length of one call with [`with`], as it does the constants handle. Outside
//! a call there is no sheet, and a Look up block says so instead of guessing.

use sse_core::Dimension;
use std::cell::RefCell;
use std::sync::Arc;

/// One answer from the sheet.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct LookedUp {
    /// The row asked for, as `table/id`.
    pub row: String,
    /// The property as found (its path on the row, or on the phase).
    pub property: String,
    /// The value in SI (0 when not known).
    pub value: f64,
    /// The sheet's unit for it, as written there.
    pub unit: String,
    /// The phase at the conditions, when conditions were given.
    pub phase: Option<String>,
    /// False when the row lacks the property (the flow skips it).
    pub known: bool,
    /// False when the conditions lie outside what the source covers.
    pub in_range: bool,
    /// The evidence grade (measured, computed, extrapolated, convention).
    pub grade: Option<String>,
    /// The source row, with its DOI or URL.
    pub source: Option<serde_json::Value>,
    /// The packs that set this row, bottom first; more than one means a
    /// pack changed it.
    pub packs: Vec<String>,
    /// Why the value is not known, or a note on how it was read.
    pub reason: Option<String>,
}

/// What a block may ask of the sheet.
pub trait Sheet: Send + Sync {
    /// The row a block's `what` names, as `table/id`: a bare id when only
    /// one table has it, or `table/id`.
    fn row_of(&self, what: &str) -> Result<String, String>;
    /// The dimension a property carries: on `what`'s row when given,
    /// otherwise on the first row that has it.
    fn dimension(&self, what: Option<&str>, property: &str) -> Result<Dimension, String>;
    /// A property of a row, at pressure (Pa) and temperature (K) when given.
    /// A missing property is an answer (`known` false), not an error.
    fn look_up(
        &self,
        what: &str,
        property: &str,
        at: Option<(f64, f64)>,
    ) -> Result<LookedUp, String>;
}

thread_local! {
    static NOW: RefCell<Option<Arc<dyn Sheet>>> = const { RefCell::new(None) };
}

/// The sheet in force, if any.
pub fn now() -> Option<Arc<dyn Sheet>> {
    NOW.with(|n| n.borrow().clone())
}

/// Run `f` with `s` in force, then restore what was there (also on a panic).
pub fn with<R>(s: Option<Arc<dyn Sheet>>, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<Arc<dyn Sheet>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let back = self.0.take();
            NOW.with(|n| *n.borrow_mut() = back);
        }
    }
    let _r = Restore(NOW.with(|n| n.replace(s)));
    f()
}

/// Puts a sheet in force until dropped, then restores what was there.
pub struct Scope(Option<Option<Arc<dyn Sheet>>>);

impl Scope {
    pub fn new(s: Option<Arc<dyn Sheet>>) -> Self {
        Scope(Some(NOW.with(|n| n.replace(s))))
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if let Some(back) = self.0.take() {
            NOW.with(|n| *n.borrow_mut() = back);
        }
    }
}
