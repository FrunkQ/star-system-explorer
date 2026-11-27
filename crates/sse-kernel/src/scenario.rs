//! The kernel's shipped scenario: data built into the kernel rather than
//! read from the stack in force.
//!
//! The measured-motion tables (RW17): the published series as cited pack
//! tables (`packs/base-sol/motion/`, built by `scripts/build-sol-motion.py`)
//! and which way the Earth faces. They are not yet in a pack the kernel
//! loads, so an edit to a universe cannot reach them; that move is RW17's
//! "tables as a pack the kernel loads" (handover/packages/RW17.md), with
//! RW8's data service.

/// The motion series, one `sse.series/1` table each.
pub(crate) const MOTION_TABLES: [&str; 9] = [
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

/// Which way the Earth faces: ERA, precession, obliquity, Delta T.
pub(crate) const EARTH_ORIENTATION: &str =
    include_str!("../../../packs/base-sol/motion/earth-orientation.json");
