//! wasm-bindgen surface. Thin: every function here forwards to a native crate
//! so the same code is tested natively and in the browser.
use wasm_bindgen::prelude::*;

/// Run the cross-device hash harness; returns the report as JSON text.
#[wasm_bindgen]
pub fn harness_run() -> String {
    sse_core::harness::report_json(&sse_core::harness::run())
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
