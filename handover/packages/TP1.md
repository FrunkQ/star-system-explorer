# TP1: dates stop losing their seconds

Kernel, 26 Nov 2026. Track TP (one time pipeline); design `handover/design/SSE4-time-scales-design.md` section 5, audit finding 3.3.

## The fault
`orbits_at` turned the whole-second date into a double before taking off the epoch (`orbits.rs:104`, `t as f64 - self.epoch`). A double holds 53 bits, so at a star's age (1.45e17 s) the smallest step is 32 s and at 13.8 Gyr it is 64 s. The audit measured a low station taking 3 distinct places in 64 one-second steps, jumping 424 km. Sol escaped only because its zero point is 2,000 years back.

Integer subtraction alone is not enough. A generated system's bodies have no dated phase, so their epoch is the universe's origin and the span itself is a star's age: `n * dt` for a station is about 5e14 rad, where a double's step is 0.06 rad. So the angle has to be formed exactly as well.

## What changed
- **`sse_core::time::Instant`** (`crates/sse-core/src/time.rs`): the one date type, whole seconds plus nanoseconds. `UniverseTime` in `sse-chronicle` is now this type re-exported, with the same fields and encoding, so no hash moves.
  - `since(anchor) -> f64`: whole seconds subtracted as integers (i128, no overflow), then the small difference into a double.
  - `angle_since(anchor, rate) -> [0, 2π)`: rate times span formed exactly (Dekker's two-product on a split of the whole seconds; no fused multiply-add, so it is the same on every device), whole turns taken off before rounding with 2π held as two doubles. Good to about 1e-16 rad at any age.
  - `from_years` refuses past the axis's end with a plain sentence; `years` now holds at the end instead of wrapping (finding 3.7, the arithmetic part; TP9 still owns checking every conversion). `from_seconds_f64` keeps a double date's value exactly; `checked_add_seconds`.
- **`orbits_at`**: each orbit's epoch is a date (`epoch_of`: integer `orbit.epoch_s` exactly, a float one, or SSE 3's `orbit.epoch_v3_ms`, integer milliseconds exactly), and the mean anomaly is `m0 + angle_since(epoch, n)`. An epoch outside the axis leaves the body unplaced with the reason named.
- **Ledger** `handover/packages/TP1-float-dates.txt`, checked by `crates/sse-core/tests/float_dates.rs` (so it runs in `cargo test`, no CI change): per file, the casts of an absolute date to a double in the Rust crates (`at_s as f64` and its kin, a bare `t`, `<instant>.s`) and the screens' year clock fields (`now_yr`, `campaign_now_yr`, `campaign_epoch_yr`, `clock_yr`). A count may only go down, and must be lowered in the change that lowers it. A few counted lines are not dates (a temperature `t` in `sse-sense/src/fog.rs`); renaming them is welcome.

## Tests, each seen red first
- `orbits::tests::one_second_moves_a_station_at_any_age`: an Earth and a station at 6,778 km, 64 one-second steps at 4.61 Gyr and at 13.8 Gyr, with the epoch at the origin and a day before. Every step moves the station 7.67 km and the Earth 29.8 km (within 2 %), 64 distinct places each. Before the fix: "moved 0.0 m in one second".
- `sse-core` `time::tests`: one second is one second at any age; angles step by the rate every second at 4.61 and 13.8 Gyr and before the origin, agree with the plain product on short spans, and add up across legs; range refusals.
- The ledger: with the old `orbits.rs` line put back it fails ("1 new date(s) held as a double").

## Goldens
None moved: the full workspace tests and the contract harness pass unchanged.

## Not in TP1 (left honestly)
- **Flares and navigation** (TP1b): their kernel ops take and give star-age seconds as JSON doubles (`sse-activity` `flares`, `Flare.at_s`; `sse-nav` `t_s: f64`, `NavSystem.t0_s`), and `activity_read` and the flight log are core golden lines, so moving them to dates is a core lane slot of its own. The flight log already keeps integer times and subtracts its own epochs in integers (`flightlog.rs:374`); its `sys.at` call does not yet.
- **Spin**: the kernel computes no spin phase at a date; the screens do, from the picture clock (TP3).
- **The screens** hold time in years and in JS numbers (the `_yr` lines of the ledger): TP3.
- `sse-cosmos/src/patch.rs`, `sse-sift`, `sse-suite`, `sse-logic` `Now`: Gyr-scale or rule-level reads where a second does not matter; on the ledger so they cannot grow.
