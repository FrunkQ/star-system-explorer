# TM3a: the Run desk's clock (track TM, TM3 part 1)

Status: done 26 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 7.3 and question 4 (accepted 25 Nov 2026).

## What it does

- **On the Run desk the clock takes the ruler's place.** One line: the campaign's date and time in the pack's calendar, large, with the "Now in the campaign" stamp (and the calendar's name when a pack gives one); step chips from the calendar's units, **+ 1 hour, + 1 day, + 1 week**; and **Run ▸** with a rate (real time, a minute a second, an hour a second, a day a second), which lets human time flow: worlds turn and ships move. A step or a tick moves the campaign's now and the time in view together, as stepping the ruler at now does, so players' screens follow.
- **Above the system** (Cosmic web, Galaxies nearby, Galaxy) the clock shows the date and has no steps: nothing there moves in human time.
- **Not on Run:** the ruler, What if…, Look ahead and Light (on Run, light is the Band control). Sound stays beside the clock. A what-if left open on another desk still shows its Keep it, Compare and Throw away on Run, so it can be resolved.
- New `CampaignClock.svelte` in chart-grammar (presentation only: each step goes to `onstep`). New `SHELL_PARTS` rows: `ruler` and `light` Explore and Build, `clock` Run.
- `known.e2e.mjs` stepped the ruler on the Run desk; it now presses the clock's "+ 1 day" chip three times (the same days).

## Not in this part (TM3b)

What is due (the next dated things as ticks, "Go to then"), "Jump to a date…" with the dialog that says what it will do, the ruler's span by desk (top-level 12.1), and "Keep on Build…" for a look ahead on Explore.

## Checks

- `desk.test.ts` pins the ruler, clock and light rows.
- `npm run ui:check` clean.
- Looked at in the browser at 1440 x 900: on Run the clock reads "Year 1, day 1, 00:00"; + 1 day gives day 2; Run ▸ at a day a second for two seconds gives day 4, 00:05; no ruler, Light or What if; on Explore the ruler is back. Screen in project files `sse4/handover-screens/TM3a-screens/`.
- Browser runs on Chromium passed: shell, walk, time, whatif, lookahead, light, lenses, bookmarks, delivery, hold-grounds, kits, known, landing-table, object-menu, playbook, steer, switchboard, table.
