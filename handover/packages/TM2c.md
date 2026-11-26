# TM2c: the folded rail on Run (track TM, TM2 part 3)

Status: done 26 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 3.1 (Rail row) and question 6 (accepted 25 Nov 2026).

## What it does

- **On the Run desk the rail folds to one button at the head of the view controls** that names the instrument in view ("Chart ▾", "Route ▾"). It opens the rail as a list in words: the Run desk's instruments (TM2a's filter) and the doors in and out (Open a world, Out to the galaxy, Back). A press on anything in the list goes there and closes it; Esc and a press elsewhere close it too. The picture gets the rail's 172 px.
- **One rail, two places.** `App.svelte` draws the same `Rail` through one snippet: beside the picture on Explore and Build, inside `RailMenu` (new, `shell/src/nav/RailMenu.svelte`) on Run. `Stage.svelte` takes a `viewLead` snippet drawn first in the view-controls row (and alone where a screen has no view controls), so the stage imports nothing of the rail's.
- **The desk decides by data:** a new `SHELL_PARTS` row, `rail-open`, Explore and Build. `railFolded` is `!partShows("rail-open", desk)`.
- **A phone keeps its row of chips** on every desk; the phone's Run frame is TM9.
- **Test helper:** `tests/ui/nav.mjs` `go()` opens the folded rail before pressing an entry, finds entries by `:visible` (the folded rail is still in the page, hidden), and uses More for the Workshop where the rail has no door. This follows the accepted design; no budget or expectation changed.

## Checks

- `desk.test.ts` pins `rail-open` on Explore and Build.
- `npm run ui:check` clean.
- Looked at in the browser at 1440 x 900: on Run the rail column is gone, "Chart ▾" sits before View style, the list opens under it with Open a world, Chart, Sky from here, Route, Voyages, Hazard map, Atlas, Switchboard and Out to the galaxy; choosing Route closes the list and the button reads "Route ▾". Screens in project files `sse4/handover-screens/TM2c-screens/`.
- Four browser tests pressed the rail's Switchboard button by role on the Run desk; they now use `go(gm, "switchboard")` (delivery, hold-grounds, steer, switchboard).
- Browser runs on Chromium passed: shell, walk, lenses, bookmarks, delivery, hold-grounds, kits, known, landing-table, object-menu, playbook, steer, switchboard, table. `view-controls.e2e.mjs` fails at Galaxies nearby's Chart style, as it does on main (not in CI; an Explore step, untouched here).
