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
| Descend | flown from the kernel's log | flies too, a few seconds behind | works (a torch ship flies down from orbit since LR1) |
| Land, look about, walk | yes, hour by Dawn/Midday/Dusk/Night (LR2) | follows the GM's pose | works; 3D ground a flat slab and night black, gap LR3 |
| Take off | yes | back to the site | works |

## Gaps and packages

| Package | Owner | State | Notes |
|---|---|---|---|
| LR0 this file and `run-through.e2e` (the whole journey as one check) | here | done | not in CI (workflow edits need Alex's yes); in `npm run ui:e2e` |
| LR1 a ship that can fly down flies down: `sse-nav` `plan_down` flies the entry and, for a lander whose thrust is over its weight, a powered descent from the entry interface, keeping the powered one only when it ends better (method `engines-from-orbit`, the reason first in `why`). The far trader at Mars now lands ("engines from orbit, 7 min 50 s"); stock landers keep their entries | here | done | packages/LR1.md; no golden change |
| LR2 choosing the hour: the Land slider in the world's local hours with the local time beside it, and Dawn, Midday, Dusk, Night presses on the site card and the walk pad | here | done | packages/LR2.md |
| LR3 the ground at night and in 3D: True at night is black with no stars, 3D is a flat grey slab (headless WebGL2) | renderer | asked 24 Nov | |
| LR4 approach and orbit for the table: between arrival and the site the players see only the header; a "Beowulf in orbit of Mars" view (the world turning below, the At-world card's chosen act) for Send this view | here, with the Run desk thread for the preset | queued | |
| LR5 a whole landing in its link: `~surface?site=lat,lon&time=midday&lander=party` (place.ts); Land applies hour and lander with the site, and a site link is followed even with Land open, so a bookmark or Playbook preset is a place link | here | done | packages/LR2.md |
| LR5b the Land screen's own link and title: on Land the place (and so the address bar) carries the site, the quick hour and the lander as the GM picks them (`landingLinkSite`), and `here()`'s title names the landing ("Mars, Dribeque Planitia at midday", `landingTitle`), so a kept beat or a copied link brings it back. Bookmarks (PB2b `Whereabouts`) do not carry the site yet: Run desk thread told | here | done | packages/LR2.md |
| Gap 10, Mass on the Run desk card at Curious | Run desk thread (asking Alex) | open | |

## Next
LR4 (an in-orbit view for the table), after PR 437 merges.
