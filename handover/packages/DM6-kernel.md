# DM6 kernel half: blocks asked at a date

Owner: kernel thread. Design: `design/SSE4-galaxy-histories-design.md` (G may vary in time as a what-if); the reader is DM6 (`packages/DM6.md`).

## What it does

- `derive` and `explain` take an optional `at_s` (campaign seconds, the same clock as `set_campaign_now`). When the stack carries a constant with a history (a `law` on its sheet row), the call runs on `Reference::handle_on(t)`, where `t` is the cosmic age at `at_s`: the cosmos settings' `now_gyr` at the clock's first now, plus the gap in Julian years. With no `at_s` (or `i64::MAX`) the date is the latest campaign now.
- A stack with no law on any row takes the plain handle, so nothing changes for every existing call and no golden moved.
- The handle is set for the one call only (`constants::with`); nothing leaks into the next call.

## Checks

- `a_block_asked_at_a_date_reads_g_at_that_date` (matter_tests): with the drifting-G fixture, a year asked a billion years back is shorter by 1/sqrt(1.1) to 1e-9; asked now it is today's year; on a campaign clock the date counts from its first now; the base stack gives the same year at every date.
- Contract goldens unchanged.

## Not yet

- Look-ahead integrations (`sse-time`) and the formation replay read G at the call's date for the whole span; G does not vary along an integration yet, and nothing marks such a span yet. A later package steps the handle per slice and marks spans it cannot follow.
- Star brightness follows through the homology blocks at the call's date and keeps its ESTIMATED stamp in the reach report.
