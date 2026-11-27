# TM3b: what is due, and Jump to a date… (track TM, TM3 part 2a)

Status: done 26 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 7.3; built against the shape of the `happenings` op in `handover/design/SSE4-time-scales-design.md` section 8 (package TP6, proposed), as the coordinator asked: no second list of our own.

## What it does

- **What is due** on the Run desk's clock: a short line, three days wide, with a tick for each of the next few marks (at most five). A press on a tick names it and offers **Go to then**, which moves the campaign to it. With nothing marked the line says "Nothing due".
- **Jump to a date…**: a dialog with an amount and a calendar unit (days, weeks, months, years). It says where the jump lands ("Lands on Year 1, day 17, 01:40, 14 days from now. Players' screens follow.") and what lies on the way ("On the way: 3 eclipses, 1 arrival", or "Nothing is marked on the way."). Cancel moves nothing; Jump moves the campaign's now like a step.
- New `packages/ui-contract/src/due.ts`: the `happenings` reply's shape (`TimeMark`, `Summarised`, `Happenings`), `dueAhead` (the due line, in whole seconds as `bigint`), `passedWords` and `checkHappenings`. `CampaignClock.svelte` takes `due` and `passing` from the shell.

## Waiting on other tracks

- **TP6 (`happenings`)** is not built, so the shell hands the clock nothing: the line always says "Nothing due" and the dialog "Nothing is marked on the way". When TP6 lands, the shell passes `dueAhead(happenings(now, now + window))` and `passing(span) = happenings(now, now + span)`; nothing in the clock changes.
- **The ships part of the dialog** ("3 ships estimated, 2 retired, about 4 seconds", top-level design 13) needs the jump planner of track J. Not built; the dialog says only where it lands and what is marked.
- **TP3 (the clock object)**: the clock still steps the screen's years (`campaign_now_yr`), as TM3a did; no new year field or absolute double was added (marks and offsets are whole seconds; the hand-over to the clock's years is at the one `onstep` call). The kernel's `set_campaign_now` is still not called by the app (time scales audit finding 2); TP3 makes the clock write it.
- Evidence for TP3: a jump of exactly 14 days from day 3, 01:39 reads 01:40 on landing, because the clock is years in a double. The browser test checks the day exactly and leaves the minute until TP3.
- **CLK1 (PR 652)**: once both are on main, the clock calls `ctx.workspace.setRate` when Run starts, stops or changes rate. Whichever merges second adds it.

## Checks

- `packages/ui-contract/test/due.test.ts`: the window's edges, the fixed order on ties, whole seconds at 13.8 Gyr, counting listed and summarised marks, the reply check.
- `tests/ui/time.e2e.mjs` (in CI) now goes on to the Run desk: the clock shows the same now as the ruler, "Nothing due", Cancel moves nothing, a jump of 2 weeks lands 14 days on, and the page does not overflow, at 1440 x 900 and on a phone. Passed on Chromium locally (Firefox is not installed here; CI runs it).
- `npm run ui:check` clean; ui-contract, chart-grammar and tests/ui unit tests pass (705).
