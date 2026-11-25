//! The golden scenario (`tests/golden/gravity.json`): the bend table's hash
//! and a few of its rows, run natively and in the browser harness so the
//! table is shown to be the same bits on every device.

use crate::bend::{self, BendTable};
use serde_json::{json, Value};
use std::sync::OnceLock;

/// The table, worked out once per process.
pub fn table() -> &'static BendTable {
    static T: OnceLock<BendTable> = OnceLock::new();
    T.get_or_init(BendTable::compute)
}

pub fn golden() -> Value {
    let t = table();
    let rows: Vec<Value> = [0usize, 64, 127, 128, 160, 200, 255]
        .iter()
        .map(|&i| {
            let r = &t.rows[i];
            json!({
                "row": i,
                "b": r.b,
                "captured": r.captured,
                "end_phi": r.end_phi,
                "turn": r.turn,
                "closest_u": r.closest_u,
                "u_mid": r.u[bend::COLS / 2],
            })
        })
        .collect();
    json!({
        "bend_table": {
            "hash": t.hash().hex(),
            "rows": bend::ROWS,
            "cols": bend::COLS,
            "b_crit": bend::b_crit(),
            "b_max": bend::B_MAX,
            "phi_max": bend::PHI_MAX,
            "sample": rows,
        }
    })
}
