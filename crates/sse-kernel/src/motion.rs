//! Measured motion in the kernel (RW17; real-catalogues design 6A.2).
//!
//! A body may name a motion model (`orbit.motion`, the model's id) beside
//! its orbit. Inside the model's span the body is where the model says;
//! outside it the body hands over to a plain orbit fitted to the model at
//! the span's edge, and every time it is placed says which ("measured" or
//! "plausible"). A model reads universe time through its system's anchor,
//! `time.j2000_s` on the root star: the universe second of J2000.0 TT.
//! Generated systems never name a model.
//!
//! The models are the published series as cited pack tables
//! (`packs/base-sol/motion/`, built by `scripts/build-sol-motion.py`), built
//! into the kernel's shipped scenario (`scenario.rs`), read once and kept.

use sse_core::math as m;
use sse_time::earth::EarthOrientation;
use sse_time::series::Series;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The fact that names a body's motion model.
pub const MOTION_FIELD: &str = "orbit.motion";
/// The fact on a system's root star that anchors its motion models in
/// universe time.
pub const ANCHOR_FIELD: &str = "time.j2000_s";

/// Every motion model the kernel holds, by id.
pub fn models() -> &'static BTreeMap<String, Series> {
    static CELL: OnceLock<BTreeMap<String, Series>> = OnceLock::new();
    CELL.get_or_init(|| {
        crate::scenario::MOTION_TABLES
            .iter()
            .map(|t| Series::from_json(t).expect("built-in motion table"))
            .map(|s| (s.id.clone(), s))
            .collect()
    })
}

/// The model of this id.
pub fn model(id: &str) -> Option<&'static Series> {
    models().get(id)
}

/// Which way the Earth faces at a date.
pub fn earth() -> &'static EarthOrientation {
    static CELL: OnceLock<EarthOrientation> = OnceLock::new();
    CELL.get_or_init(|| {
        EarthOrientation::from_json(crate::scenario::EARTH_ORIENTATION)
            .expect("built-in earth orientation table")
    })
}

/// Universe seconds to the Julian date the series read (TT, which they
/// treat as TDB: the two differ by under 2 ms). The gap from the anchor is
/// taken in whole seconds first, so no absolute time is held in a double.
pub fn jd(at_s: i64, anchor_s: i64) -> f64 {
    sse_time::series::J2000_JD + (at_s - anchor_s) as f64 / sse_core::sheet::DAY
}

/// Julian date back to universe seconds (to the nearest second).
pub fn at_s(jd: f64, anchor_s: i64) -> i64 {
    anchor_s + m::round((jd - sse_time::series::J2000_JD) * sse_core::sheet::DAY) as i64
}
