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
//! (`packs/base-sol/motion/`, built by `scripts/build-sol-motion.py`), read
//! once and kept.

use sse_time::earth::EarthOrientation;
use sse_time::series::Series;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The fact that names a body's motion model.
pub const MOTION_FIELD: &str = "orbit.motion";
/// The fact on a system's root star that anchors its motion models in
/// universe time.
pub const ANCHOR_FIELD: &str = "time.j2000_s";

const TABLES: [&str; 9] = [
    include_str!("../../../packs/base-sol/motion/vsop87a.mer.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.ven.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.emb.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.mar.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.jup.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.sat.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.ura.json"),
    include_str!("../../../packs/base-sol/motion/vsop87a.nep.json"),
    include_str!("../../../packs/base-sol/motion/elpmpp02.moon.json"),
];

/// Every motion model the kernel holds, by id.
pub fn models() -> &'static BTreeMap<String, Series> {
    static CELL: OnceLock<BTreeMap<String, Series>> = OnceLock::new();
    CELL.get_or_init(|| {
        TABLES
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
        EarthOrientation::from_json(include_str!(
            "../../../packs/base-sol/motion/earth-orientation.json"
        ))
        .expect("built-in earth orientation table")
    })
}

/// Universe seconds to the Julian date the series read (TT, which they
/// treat as TDB: the two differ by under 2 ms).
pub fn jd(at_s: f64, anchor_s: f64) -> f64 {
    sse_time::series::J2000_JD + (at_s - anchor_s) / sse_core::sheet::DAY
}

/// Julian date back to universe seconds.
pub fn at_s(jd: f64, anchor_s: f64) -> f64 {
    anchor_s + (jd - sse_time::series::J2000_JD) * sse_core::sheet::DAY
}
