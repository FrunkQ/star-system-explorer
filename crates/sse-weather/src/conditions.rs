//! The facts a weather block reads. Weather is keyed on **conditions**, never
//! on what kind of object a body is. Since W7a the conditions belong to the
//! one climate (`sse-climate`), with a signed spin, and a body's conditions
//! are built in one place (`sse_surface::climate::conditions`) so the ground
//! and the sky read the same facts.

pub use sse_climate::conditions::{Conditions, Extra, EARTH_COLUMN_KG_M2};

/// Conditions from the surface track's body and envelope, for worlds that
/// come through the formation pipeline or the V3 importer. Missing facts use
/// the climate pack's defaults, and the result says which were assumed.
pub fn from_surface(
    ss: &sse_surface::SurfaceSettings,
    b: &sse_surface::Body,
    env: &sse_surface::SurfaceEnvelope,
    x: &Extra,
) -> (Conditions, Vec<String>) {
    sse_surface::climate::conditions(ss, b, env, x)
}
