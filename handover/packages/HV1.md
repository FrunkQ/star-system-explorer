# HV1: Energy flow on the Body screen

Thread "Show a planet's heat flows over time", 26 Nov 2026. Status: `handover/status/heat-screen.md`.

## What landed

- `packages/ui-contract/src/heat-flow.ts`: the heat books at one step of the core history as Flow rows (`heatBooks`): sources (core cooling and inner core freezing into the core; the core, radioactive decay, tides, mantle cooling, lid cooling into the world), the world, and where it goes (through the rock, carried up by lava, or into a warming store). Every amount is a published figure times the world's area; nothing sums physics. When the step carries only today's `CoreStep` (no section 7 fields), it draws just the core's heat and the heat out of the surface, and says what is missing. `heatWhy` gives the words (what it loses, the biggest source, temperatures, how it compares with its youth, the kernel's own dynamo sentence); `heatCaveats` the honesty marks, chosen by the world's regime and figures, never its name: lid mantles too hot (HF1, HF4b), tides with one response (HF3, HF6, shown only where tides are a tenth of what leaves), inner cores too big (nucleation fudge), surface features still reading heat made (HF2).
- `Session.surveyHeat` (GM only): the quick survey's envelope (kernel `survey_surface`, the same cached survey the climate reads), whose `core` is the history.
- `packages/views/body/src/BodyHeat.svelte`, under the Cutaway on the Body screen: a "Energy flow" button opens it from itself. Slider "Looking at" along the history's own steps (the world's own grid; offsets from its birth, never a date; "Looking back: nothing changes in the campaign"; "Back to now"). The thermal line is two small charts (temperatures; heat lost and out of the core, log scale), never two scales on one axis, with the history's marks as ticks (each with its why) and the slider's step as the accent cursor; pressing a chart moves the slider. "What had happened by then" lists the marks up to the step; "How far to trust these numbers" holds the caveats.
- Flow view: a middle column's figure now has a gap after its name, and labels carry a halo of the ground so a band under them does not hide them (the Power card gains both).

## Desks and depth

- Explore: Curious shows the flow with names only and two sentences; Science adds figures, the line, the marks; Engine adds the history's version, step count and which fields the amounts came from, and which package fixes each caveat.
- Build: the same reading. After an edit to the world's makeup, picking the world again re-reads the survey (the kernel keeps the survey cached, so this is quick).
- Run: GM seat only (the survey is GM only); players never see it.
- Phone: the flow runs top to bottom; no sideways scroll (tested).

## Today's numbers (main at `dc1cf06`, sample Solar System)

Earth loses 34.6 TW now, 7.8 TW of it out of the core; 115 TW just after it formed (3.3 times as much). The Moon loses 0.28 TW, its mantle at the 2,200 K cap (the lid caveat shows). Earth's inner core comes out about 3,000 km (caveat shows).

## Tests

- `packages/ui-contract/test/heat-flow.test.ts`: today's partial books and words; section 7's books close in watts (absolute, under 1 W out of 4 × 10¹³) including a warming store; caveats by condition; against the real kernel, Earth's history runs down with age, a player is refused.
- `tests/ui/heat.e2e.mjs` (and its CI line): Earth at Curious then Science, slide to its youth (more heat lost, state hash unchanged), Back to now, the Moon's lid caveat, phone fit.

## Not done

- No preview of the full books in the browser yet: the fields arrive with HF4b.1; the unit test covers the layout. HV2 checks it on real worlds.
- The thermal line does not zoom or slide (interface style rule 11 for flat charts); it is short enough today.
- Sunlight is left out on purpose (status file, open point 2).

## Named energy flow (Alex, 26 Nov 22:10 UTC: "Heat flow in planets == energy flow")

One idea and one grammar for ships and worlds: the panel is "Energy flow", its subtitle and columns are the Power card's words ("Comes from", "Ends up as"), and it is drawn by the same Flow view. A world's sources, stores and sinks are nodes in that grammar; a store that warms is drawn as where some energy ends up.
