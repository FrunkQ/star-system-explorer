# Status: landing run-through (track LR)

Owner thread: "Build landing run-through" (from 24 Nov 2026, taken over from the chart room). Covers Alex's milestone of
23 Nov 2026: "landing sequence and end to end run through ... the GM plotting the Players experience of the events".
The journey: the party's ship flies a voyage to a world, arrives, the world is revealed, the GM picks a site, the
ship descends and lands, the party looks about; players on their own seats see each step the GM sends.

Owned here: this file, `tests/ui/run-through.e2e.mjs` (the whole journey, GM and a player side by side), the hand-off
from Voyages to Land, and the Land screen's glue (`packages/views/surface/src/Surface.svelte` controls; the renderer
thread owns its drawing). Not owned: `shell/src/run/`, `Players.svelte`, `landing-table.e2e`, `party.e2e` (Run desk
thread, status/run-desk.md); `shell/src/chart/` (chart room); `nav/` (selector); the drawing in `renderer/` (renderer).
Only the owner thread edits this file. Earlier landing rows stay where they were written: chart-room.md ("What the table
is shown", "The table watches a landing", "Players see the party's ship"), galaxy-and-travel.md (party ship, blocker 4),
renderer.md (SL1, V10g, V10i, V10j, V10k), run-desk.md (RD1 reveal on arrival).

## The journey on main 9a5912c (24 Nov 2026, walked by `run-through.e2e`)

| Step | GM | Players | State |
|---|---|---|---|
| Party's ship picked (Ships and stations) | yes | header "Aboard Beowulf, at Earth" | works |
| Voyage Earth to Mars, "Fly Beowulf there" | yes, At-world card with dangers | header follows to Mars | works |
| Arrival reveal | RD1 slip (PR 421) | through set_known | in PR 421 |
| Approach and orbit | At-world card (aerocapture, entry, powered landing) | nothing until sent | gap LR4 |
| Pick a site (Land here, landmark or anywhere) | yes, party's ship as lander | nothing until sent | works |
| Send this view | yes | Landing screen, same site and verdict on own kernel | works (send race fixed in RD1) |
| Descend | flown from the kernel's log | flies too, a few seconds behind | works; verdict wrong, gap LR1 |
| Land, look about, walk | yes | follows the GM's pose | works; night and 3D ground, gaps LR2, LR3 |
| Take off | yes | back to the site | works |

## Gaps and packages

| Package | Owner | State | Notes |
|---|---|---|---|
| LR0 this file and `run-through.e2e` (the whole journey as one check) | here | done | not in CI (workflow edits need Alex's yes); in `npm run ui:e2e` |
| LR1 a ship that can fly down flies down: plan_descent with a capable engine (thrust over weight, delta-v enough, as ship_at_world's powered-landing check) plans a powered descent from orbit instead of a ballistic entry, picking the better of the two and saying which and why. Today a Traveller far trader at Mars "burns up" (aerobrake only, 7 W/cm² against a 1 W/cm² hull, 2392 m/s touchdown) while the At-world card says "Powered landing from orbit: within limits" | here (kernel thread agreed, sse-nav descent) | next | may move golden plan_descent cases: ask the coordinator for a lane slot first |
| LR2 choosing the hour: the Land screen's "Hours from now" slider binds degrees (±180) under an hours label; label it in hours and add Dawn, Midday, Dusk and Night presses on the world's own clock, so the GM can land in daylight | here (renderer agreed) | next | Surface.svelte only |
| LR3 the ground at night and in 3D: True at night is black with no stars, 3D is a flat grey slab (headless WebGL2) | renderer | asked 24 Nov | |
| LR4 approach and orbit for the table: between arrival and the site the players see only the header; a "Beowulf in orbit of Mars" view (the world turning below, the At-world card's chosen act) for Send this view | here, with the Run desk thread for the preset | queued | |
| LR5 a landing preset for the Playbook (PB3 "A landing"): an exported pure helper `landingFor(world, site) -> LandingShow`, so a preset stores {world, site} and builds the landing when fired | here, for the Run desk thread | queued | |
| Gap 10, Mass on the Run desk card at Curious | Run desk thread (asking Alex) | open | |

## Next
LR1, then LR2, then LR5 (the Run desk needs it for PB3), then LR4.
