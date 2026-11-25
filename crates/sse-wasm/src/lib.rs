//! wasm-bindgen surface. Thin: every function here forwards to a native crate
//! so the same code is tested natively and in the browser.
use wasm_bindgen::prelude::*;

/// Run the cross-device hash harness; returns the report as JSON text.
#[wasm_bindgen]
pub fn harness_run() -> String {
    sse_core::harness::report_json(&sse_core::harness::run())
}

/// The engine golden scenarios, by name (the keys of tests/golden/*.json).
/// Each runs alone through `engine_run`, so the device check page and the
/// bench (PF2) can time them one by one; `engines_run` runs them all.
macro_rules! engines {
    ($($name:literal => $f:expr,)*) => {
        const ENGINE_NAMES: &[&str] = &[$($name),*];
        fn engine(name: &str) -> Option<serde_json::Value> {
            match name {
                $($name => Some(serde_json::json!($f)),)*
                _ => None,
            }
        }
    };
}

engines! {
    "resolve" => sse_resolve::scenario::golden(),
    "sift" => sse_sift::scenario::golden(),
    "observe" => sse_observe::scenario::golden(),
    "suite" => sse_suite::scenario::golden(),
    "formation" => sse_formation::scenario::golden(),
    "formation-early" => sse_formation::scenario::early_golden(),
    "classify" => sse_sift::scenario::classify_golden(),
    "life" => sse_life::scenario::golden(),
    "place" => sse_place::scenario::golden(),
    "time" => sse_time::scenario::golden(),
    "time-generated" => sse_time::scenario::generated_golden(),
    "cosmology" => sse_cosmos::scenario::cosmology_golden(),
    "haloes" => sse_cosmos::scenario::haloes_golden(),
    "galaxy" => sse_cosmos::scenario::galaxy_golden(),
    "envelope" => sse_cosmos::scenario::envelope_golden(),
    "life-budget" => sse_life::scenario::budget_golden(),
    "life-history" => sse_life::scenario::history_golden(),
    "life-biosphere" => sse_life::scenario::biosphere_golden(),
    "stars" => sse_cosmos::scenario::stars_golden(),
    "galaxy-events" => sse_cosmos::scenario::galaxy_events_golden(),
    "nucleus" => sse_cosmos::scenario::nucleus_golden(),
    "life-signatures" => sse_life::scenario::signatures_golden(),
    "life-feedback" => sse_life::scenario::feedback_golden(),
    "nav" => sse_nav::scenario::golden(),
    "formation-accrete" => sse_formation::scenario::accrete_golden(),
    "formation-accrete-dole" => sse_formation::scenario::accrete_dole_golden(),
    "formation-growth" => sse_formation::scenario::growth_golden(),
    "formation-upheaval" => sse_formation::scenario::upheaval_golden(),
    "formation-leftovers" => sse_formation::scenario::leftovers_golden(),
    "formation-pipeline" => sse_formation::scenario::pipeline_golden(),
    "cosmos" => sse_cosmos::scenario::golden(),
    "surface" => sse_surface::scenario::golden(),
    "surface-atlas" => sse_surface::scenario::atlas_golden(),
    "surface-plates" => sse_surface::scenario::plates_golden(),
    "surface-wear" => sse_surface::scenario::wear_golden(),
    "surface-impacts" => sse_surface::scenario::impacts_golden(),
    "surface-regimes" => sse_surface::scenario::regimes_golden(),
    "surface-cover" => sse_surface::scenario::cover_golden(),
    "gravity" => sse_gravity::scenario::golden(),
}

/// Run the engine golden scenarios (resolve, sift, observe, suite and the
/// rest); returns their digests as JSON text, keyed like tests/golden/*.json.
#[wasm_bindgen]
pub fn engines_run() -> String {
    let mut all = serde_json::Map::new();
    for name in ENGINE_NAMES {
        all.insert((*name).to_string(), engine(name).unwrap_or_default());
    }
    serde_json::Value::Object(all).to_string()
}

/// The engine scenarios' names, as a JSON array (PF2).
#[wasm_bindgen]
pub fn engine_names() -> String {
    serde_json::json!(ENGINE_NAMES).to_string()
}

/// Run one engine scenario; its digests as JSON text, or `null` for an
/// unknown name (PF2). Timed by the caller, outside the kernel.
#[wasm_bindgen]
pub fn engine_run(name: &str) -> String {
    engine(name).unwrap_or_default().to_string()
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
