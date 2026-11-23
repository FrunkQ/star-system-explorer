//! Sheet references in pack data (MP22): a pack writes a number the master
//! data sheet already holds as a reference to it, never as a typed copy.
//!
//! ```json
//! "gravity_m_s2": { "sheet": "constants/g0" }
//! "molar_kg": { "sheet": "mixtures/dry-air", "property": "molar_mass" }
//! "k40_half_life_gyr": { "sheet": "isotopes/k40", "property": "half_life_s", "in": "gyr" }
//! "divide_by": { "sheet": "units/au" }
//! ```
//!
//! `sheet` and `property` are the Look up block's own `what` and property, so
//! a pack value and a visible block read the sheet the same way; `in` names a
//! unit on the sheet to give the value in (SI otherwise). A unit row stands for
//! its factor to SI. An object counts as
//! a reference only when `sheet` is a string and its other keys are among
//! `property`, `in` and `note`, so a row that merely names its sheet row
//! (`{"id": "air", "sheet": "dry-air", ...}`) is left alone.
//!
//! Blocks read a pack's settings through [`with_sheet`], which replaces every
//! reference with the number from the sheet in force for that stack, so an
//! edited sheet reaches the pack's numbers too.

use crate::Reference;
use serde_json::Value as J;
use sse_flow::sheet::Sheet;

const KEYS: [&str; 4] = ["sheet", "property", "in", "note"];

fn is_ref(o: &serde_json::Map<String, J>) -> bool {
    o.get("sheet").is_some_and(J::is_string) && o.keys().all(|k| KEYS.contains(&k.as_str()))
}

/// True when `v` holds a sheet reference anywhere.
pub fn has_refs(v: &J) -> bool {
    match v {
        J::Object(o) => is_ref(o) || o.values().any(has_refs),
        J::Array(a) => a.iter().any(has_refs),
        _ => false,
    }
}

/// The number one reference names.
pub fn value_of(o: &serde_json::Map<String, J>, r: &Reference) -> Result<f64, String> {
    let what = o.get("sheet").and_then(J::as_str).unwrap_or_default();
    let property = o.get("property").and_then(J::as_str).unwrap_or("");
    // A unit is its factor to SI, worked out from the constants behind it.
    if let Some(u) = what.strip_prefix("units/") {
        return r.unit_factor(u);
    }
    let got = r.look_up(what, property, None)?;
    if !got.known {
        return Err(format!(
            "sheet reference {what} {property}: {}",
            got.reason.unwrap_or_else(|| "not on the sheet".into())
        ));
    }
    match o.get("in").and_then(J::as_str) {
        Some(u) => Ok(got.value / r.unit_factor(u)?),
        None => Ok(got.value),
    }
}

/// Replace each sheet reference in `v` by its number from `r`.
pub fn resolve_refs(v: &mut J, r: &Reference) -> Result<(), String> {
    match v {
        J::Object(o) if is_ref(o) => {
            let x = value_of(o, r)?;
            *v = serde_json::Number::from_f64(x)
                .map(J::Number)
                .ok_or_else(|| format!("sheet reference gave {x}, not a finite number"))?;
            Ok(())
        }
        J::Object(o) => o.values_mut().try_for_each(|x| resolve_refs(x, r)),
        J::Array(a) => a.iter_mut().try_for_each(|x| resolve_refs(x, r)),
        _ => Ok(()),
    }
}

/// The sheet in force for a stack: its own reference packs, or the base
/// sheet when the stack carries none.
pub fn sheet_of(stack: &[sse_pack::PackManifest]) -> Reference {
    Reference::from_packs(stack)
        .ok()
        .filter(|r| r.row("constants", "c").is_some())
        .unwrap_or_else(Reference::base)
}

/// A pack's settings block (`provides.other.<key>`) as JSON, with its sheet
/// references resolved against the stack's sheet. Settings with no
/// reference are passed through untouched, at no cost.
pub fn with_sheet<T: serde::Serialize>(
    stack: &[sse_pack::PackManifest],
    v: &T,
) -> Result<J, String> {
    let mut j = serde_json::to_value(v).map_err(|e| e.to_string())?;
    if has_refs(&j) {
        resolve_refs(&mut j, &sheet_of(stack))?;
    }
    Ok(j)
}
