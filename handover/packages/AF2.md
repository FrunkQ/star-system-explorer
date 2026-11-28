# AF2: a first visit opens in True (alpha setting)

27 Nov 2026. Alex chose "True everywhere" on the question "Should a fresh visit open in True view instead of Chart?",
then: "true everywhere FOR NOW... during alpha - phones WILL gate later when we have all the performance profiles
aligned". Follows AF1 (most of the new pictures are True only and the app opened on Chart).

**Alpha setting, revisit when the performance profiles are aligned.** To gate phones again, set `TRUE_FIRST_ON` in
`packages/ui-contract/src/view-controls.ts` to the device tiers that should open in True; every other tier, and a
device not yet measured, opens on Chart. One line.

## What changed

- `firstView(tier)` and `TRUE_FIRST_ON` (view-controls.ts): what a viewer who has chosen nothing opens in. True on
  every device for now. `DEFAULT_VIEW` (Chart) stays the fallback for an unreadable kept choice and for the views that
  use it directly (cutaway, galaxies nearby, formation replay).
- `readViewChoice(json, first)`: nothing kept gives `first`. The shell passes `firstView()` for the plain choice and
  for each desk's recalled choice (TM6), so a desk visited first also opens in True.
- A viewer's own choice still wins, kept per desk as before.

## Tests

- Browser tests written for the flat chart (clicking its bodies, tabbing to them, counting presses on it) import
  `tests/ui/chart-first.mjs`, which keeps Chart as the viewer's choice in every page they open, exactly as a viewer who
  chose Chart would have it. That is every `tests/ui/*.e2e.mjs` except `fresh.e2e.mjs`, which now checks that a first
  visit opens in True. A new test of a first visit should not import it.
- Press budgets (`walk.e2e.mjs`) are unchanged and still walk the Chart path. Not yet measured: the same walk in True.

## Known gaps in True (told to Alex)

- Bodies cannot be reached with Tab in True ("could not reach the target with Tab" when shell.e2e runs in True), so a
  keyboard-only newcomer must switch to Chart to pick a body.
- Entering the Solar System took about 27 s in True in headless Chromium (software WebGL) against a few seconds on
  Chart; the True view's drawing competes with the door. Unmeasured on a real GPU or a phone.

## Checks

- `ui:check` clean; `ui:test` 1181 pass (new `firstView` test).
- fresh, walk, shell, start, campaign, desk-memory and delivery e2e pass in Chromium. `view-controls.e2e.mjs` fails at
  the Galaxies nearby step on main as well (a known failing test on main), unchanged by this.
