//! Strong gravity for the screen (exotic objects design, track XO).
//!
//! Package XO1: the **bend table** (design section 3.2). For a mass that does
//! not spin, a light ray stays in one plane and obeys
//!
//! ```text
//!   d²u/dφ² + u = 3 u²      (u = r_g / r, φ the angle swept round the mass)
//! ```
//!
//! (Schwarzschild 1916; Darwin 1959; the method of Luminet 1979). Measured
//! in gravitational radii r_g = GM/c², every ray depends on one number, its
//! impact parameter b, so one small table serves every non-spinning mass in
//! the universe. The kernel works it out once with plain arithmetic (RK4 on
//! `+ - * /` only: no transcendental function, no fused multiply-add), so it
//! is the same bits on every device; its content hash says so.
//!
//! The gravity formula blocks (sizes, weak bending, clocks, tides) are in
//! `packs/base-physics/formulas/gravity.json`; this crate holds only what is
//! a table, as "heavy physics ships as tables behind swappable blocks".

pub mod bend;
pub mod scenario;

pub use bend::{BendRow, BendTable};

#[cfg(test)]
mod blocks_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod golden {
    /// Compare with `tests/golden/<name>`; `SSE_BLESS=1` rewrites it.
    pub fn check(name: &str, got: &serde_json::Value) {
        let path = format!("{}/../../tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
        let text = serde_json::to_string_pretty(got).unwrap() + "\n";
        if std::env::var("SSE_BLESS").is_ok() {
            std::fs::write(&path, &text).unwrap();
            return;
        }
        let want = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing {path}; run with SSE_BLESS=1"));
        assert_eq!(
            text.replace("\r\n", "\n"),
            want.replace("\r\n", "\n"),
            "golden {name} changed"
        );
    }
}
