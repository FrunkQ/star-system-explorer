//! wasm-bindgen surface. Thin: every function here forwards to a native crate
//! so the same code is tested natively and in the browser.
use wasm_bindgen::prelude::*;

/// Run the cross-device hash harness; returns the report as JSON text.
#[wasm_bindgen]
pub fn harness_run() -> String {
    sse_core::harness::report_json(&sse_core::harness::run())
}

/// Run the K7 to K10 engine golden scenarios (resolve, sift, observe,
/// suite); returns their digests as JSON text, keyed like tests/golden/*.json.
#[wasm_bindgen]
pub fn engines_run() -> String {
    serde_json::json!({
        "resolve": sse_resolve::scenario::golden(),
        "sift": sse_sift::scenario::golden(),
        "observe": sse_observe::scenario::golden(),
        "suite": sse_suite::scenario::golden(),
        "formation": sse_formation::scenario::golden(),
        "formation-early": sse_formation::scenario::early_golden(),
        "classify": sse_sift::scenario::classify_golden(),
        "life": sse_life::scenario::golden(),
        "place": sse_place::scenario::golden(),
        "time": sse_time::scenario::golden(),
        "time-generated": sse_time::scenario::generated_golden(),
        "formation-accrete": sse_formation::scenario::accrete_golden(),
        "formation-accrete-dole": sse_formation::scenario::accrete_dole_golden(),
        "formation-growth": sse_formation::scenario::growth_golden(),
        "formation-upheaval": sse_formation::scenario::upheaval_golden(),
        "formation-leftovers": sse_formation::scenario::leftovers_golden(),
        "formation-pipeline": sse_formation::scenario::pipeline_golden(),
        "cosmos": sse_cosmos::scenario::golden(),
        "surface": sse_surface::scenario::golden(),
        "surface-atlas": sse_surface::scenario::atlas_golden(),
        "surface-plates": sse_surface::scenario::plates_golden(),
    })
    .to_string()
}

/// The kernel behind the worker boundary: JSON request in, JSON response out.
#[wasm_bindgen]
pub struct Kernel(sse_kernel::Kernel);

#[wasm_bindgen]
impl Kernel {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Kernel {
        Kernel(sse_kernel::Kernel::new())
    }
    pub fn handle(&mut self, request: &str) -> String {
        self.0.handle_json(request)
    }
}
