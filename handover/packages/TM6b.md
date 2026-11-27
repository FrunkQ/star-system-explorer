# TM6b: the ruler's span by desk (track TM, TM6 part 2)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-top-level-ui-and-onboarding.md` 12.1 ("The ruler opens at the span that suits the desk and the scale. Changing it is always one click, and the last choice is remembered per desk") and three desks design question 9.

## What it does

- **Explore** at a system or world opens the ruler on human time (a week around now) with **Whole life ›** beside it, one press to birth-to-death. Above the system Explore opens on the whole history.
- **Build** opens on the full ruler: the whole life of the place in view.
- **Run** has the clock, not the ruler (TM3a).
- **The span the GM chooses** (zoom, Show, Whole life ›) is kept for that desk at that level, on the device (`desk.<desk>.<level>.ruler-span`). There is no fallback to another desk's span: the desks differ on purpose. Play closing in to make time visible is not a choice and is not kept.
- `TimeRuler.svelte` (the chart room's; no chart room thread runs on this account, coordinator's go-ahead) has two optional props: `opens` (`{ key, span }`, applied at once, not glided, when the key changes, so a zoom is never re-centred and the events in view are settled the moment the desk changes; the span is a length of time, a Show id, or null) and `onspan`. Without them it behaves as before. Spans are lengths (differences), never absolute times; no new year field.
- `desk.ts`: `rulerOpens(desk, level, kept)` and `keepRulerSpan(span)`. The workspace carries `ruler_opens` and `setRulerSpan`; `RulerView` passes them to the ruler.

## Tests followed the design

- `time.e2e.mjs` checked the ruler opened in deep time. On Explore it now checks the ruler opens on a week at the campaign's start with Whole life › beside it, and that Whole life › shows the deep time, then carries on as before.

## Not in this part

Run's first-visit True on a capable device: waits on the quality tiers probe. When the time design's TP7 gives the ruler a break with late events after it, `opens` needs nothing new.

## Checks

- `desk.test.ts`: the defaults by desk and level, a kept span and a kept whole history coming back, an odd kept value falling back.
- `desk-memory.e2e.mjs`: Explore opens on human time with Whole life ›; Explore shown a year; Build opens on the whole life without Whole life ›; back on Explore the year is still there.
- Chromium: shell, walk, lenses, known, table, landing-table, bookmarks, playbook, steer, switchboard, delivery, hold-grounds, kits, object-menu, time, whatif, lookahead, light, system3d, compare, symbols, tactile, adopt, aurora, formation pass. `npm run ui:check` clean.
