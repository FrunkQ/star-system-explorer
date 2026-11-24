---
name: sse4-landing-run-through
description: Alex's target for the week of 23 Nov 2026: GM plots events, players experience them end to end, ending in a landing; gap list and owners
metadata:
  type: project
  modified: 2026-11-24T01:35:00.000Z
---

Alex, 23 Nov 2026 22:16: next week's level is "landing sequence and end to end run through. The GM plotting the Players experience of the events."

Gap list: /mnt/project-files/sse4/plans/landing-run-through-gaps.md (walked on main fccd80ac).

Blockers and owners:
- Players' page hard-loads kestrel.player-facts.json and never follows the GM's system (chart room).
- No Send this view / Screens off for seats (chart room; first cut of Looks 4C/4D seat rows).
- Players never see the voyage or landing; Land runs only on the GM machine (renderer: players' Land screen; travel: descent state; chart room: screen doc).
- No party ship through voyage and land (travel).
- Plotting events: Q1 to Alex, run sheet of beats on Run desk (recommended) or live only.

Progress 24 Nov 02:20 UTC: chart-room blockers done on main. Players follow the GM (PR 344), adopted systems replicate (352), players' Landing screen (356), party ship in players' header (359), per-seat Send/Screens off in Seats panel via Seat.shown (360). Left: Q1 run sheet (waiting on Alex); sealed seat's own screens-off; Look menu, signal dial, pinned preview with renderer looks.

**Why:** this is the integration test-drive goal for that week; check with Alex before widening scope.
**How to apply:** prioritise these over queued packages (OV6-9, FT6) in the chart room; see [[sse4-secrets-and-visibility]].

Also 23 Nov: Alex's standard for placing controls is "how often it is used, easy flow and newbie friendly". View bar: HUD and Band folded into one Show chip (PR 335); ship card uses one "Aboard" button (PR 318).

State 24 Nov 01:30 UTC: the players' landing is on main. The pieces:
- the renderer's PR 351, playerSurface plus Session grantLanding;
- the chart room's PR 344, Send this view, and PR 356, players.land.screen.json with landing-table.e2e;
- the travel thread's PR 339, the party ship as lander.

The renderer also merged:
- PR 355, the site link `~surface?site=lat,lon` and the sun on the world's own clock;
- PR 357, waves and tides (open at the time of writing).

The seam is ui-contract landing.ts `LandingShow`, sent by the GM through ws.sendLanding. Seats replay the landing on their own kernel. Detail on surfaces in [[sse4-surface-descent]].
