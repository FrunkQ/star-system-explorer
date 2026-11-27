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

## TP1b: flares and the flight log (26 Nov 2026)
- **Flares** (`sse-activity`): `flares` takes the window as two dates. The bins are whole seconds (a power of two, or a minute), so they are counted from the dates in integers, and each flare's peak is the bin's start plus a span: an exact date, `Flare.at`. Before, a flare at 4.6 Gyr could only fall on a multiple of 32 s. The keyed draws are the same, so the same flares come out in the same order. The activity cycle's phase and a target planet's place round the star use `angle_since`, so they too move by the second. `Moment.at`, `Flares.from`/`to` and `Hit.arrives` are the exact dates; the `_s` doubles stay beside them for display only. `cycle_factor`, `moment` and `Target::direction` take a date.
- **`read_activity`**: `at_s` and the window's `from_s`/`to_s` are read with `sse_core::time::Seconds` / `time::seconds`, so a whole number of seconds from the screens is taken exactly (a fraction keeps its double's value).
- **Navigation**: `NavSystem::at_date` and `kepler::state_at` place a body at a date with the angle formed exactly; the flight log's ephemeris (`Ephemeris::body`) takes a date, so a log's positions, which were already kept in whole seconds, no longer go through a double. `NavSystem::at` (a double date) stays for the ops that still take one from the screens.
- **Tests, each seen red first**: `flares_keep_their_seconds_at_a_stars_age` (with the old double date: 0 of 53 flares off the 32 s grid; now most are, and moving the window's start by 7 s keeps the same flares); `worlds_move_every_second_at_a_stars_age` in the flight log tests (with the old double date: "moved 0.0 m in a second"; now the Earth moves about 29.8 km every second at 4.61 Gyr, with the elements dated far back or near). The ledger lost three lines (the flight log and its two test files).
- **Goldens, on purpose**: `tests/golden/activity.json` (area) and one core line in `tests/golden/contract.jsonl`, `activity_read` (response 149). In that line the new date fields are added; the flares are the same, in the same order; five flares' display doubles change in the last digit; the Sun's cycle phase moves from 0.4361207485 to 0.4361206371, about 40 s of an 11-year cycle. Neither figure is better than that: 4e8 cycles from the origin, both are only as good as the period's last digit (the old one divided the date by the period, the new one multiplies by a rounded rate), which is the physics's limit the design names. What changed is that the phase now moves every second; before, it moved in steps of about 20 s. The moment's X-rays, spots and flare rates follow from that phase in the seventh figure.

## Not in TP1 (left honestly)
- **Navigation's own ops** (`plan_route`, `plan_voyage`, the jump map, drift, descent): they take and give star-age seconds as JSON doubles from the screens (`depart_s`, `at_s`, `t_s: f64`), and the screens hold time as JS numbers, which lose seconds the same way. They move to dates with the screens' clock (TP3); `kernel/src/nav.rs`'s two lines stay on the ledger until then.
- **Prominences** (PR 658, beside the flares in `read_activity`, in the same double form on purpose): to be moved to dates in the same way once that PR is on main.
- **Spin**: the kernel computes no spin phase at a date; the screens do, from the picture clock (TP3).
- **The screens** hold time in years and in JS numbers (the `_yr` lines of the ledger): TP3.
- `sse-cosmos/src/patch.rs`, `sse-sift`, `sse-suite`, `sse-logic` `Now`: Gyr-scale or rule-level reads where a second does not matter; on the ledger so they cannot grow.
