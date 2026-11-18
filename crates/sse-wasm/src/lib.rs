//! wasm-bindgen surface. Thin: every function here forwards to a native crate
//! so the same code is tested natively and in the browser.
use wasm_bindgen::prelude::*;

/// Run the cross-device hash harness; returns the report as JSON text.
#[wasm_bindgen]
pub fn harness_run() -> String {
    sse_core::harness::report_json(&sse_core::harness::run())
}
