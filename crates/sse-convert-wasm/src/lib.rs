//! The V3 to V4 importer as a browser module for the converter
//! (FrunkQ/starsystem-convert). SSE4 stays the only source of the importer:
//! the converter vendors this module's built `.wasm` and glue, pinned by
//! SSE4 commit and sha256 (`scripts/build-convert-wasm.sh`).
//!
//! One call, `import_v3(save_text, universe_id)`, returns the same JSON as
//! the kernel's `import_v3` op without reconstruction: `{universe, facts,
//! report}`, or `{error}` when the save cannot be read.
#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

/// Import a V3 save. Returns JSON text: `{universe, facts, report}` or
/// `{error}`.
pub fn import(save_text: &str, universe_id: &str) -> serde_json::Value {
    match sse_import_v3::import_json(save_text, universe_id) {
        Ok(im) => im.to_value(),
        Err(e) => serde_json::json!({ "error": e.0 }),
    }
}

#[wasm_bindgen]
pub fn import_v3(save_text: &str, universe_id: &str) -> String {
    import(save_text, universe_id).to_string()
}

/// The importer's version and report format, for the converter's pin check.
#[wasm_bindgen]
pub fn importer_version() -> String {
    serde_json::json!({
        "importer": sse_import_v3::IMPORTER_VERSION,
        "report_format": sse_import_v3::REPORT_FORMAT,
    })
    .to_string()
}

#[cfg(test)]
mod tests;
