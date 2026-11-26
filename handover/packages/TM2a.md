# TM2a: the rail by desk (track TM, TM2 part 1)

Status: done 26 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 5 and question 7 (accepted 25 Nov 2026).

## What it does

- **Seven instruments are off the Run desk's rail:** Sizes, Life, Stories, Roll call, The physics, Look up and Postcards, at every scale they serve. Each has `"desks": ["explore", "build"]` on its rail entries in its screen document, so moving one back is a one-word change. On Explore and Build the rail is exactly as before.
- **The shell passes the desk to `railOf`** (`App.svelte`), so the rail is rebuilt when the desk changes. Off Run means not listed: a screen already open stays open, and Find and the desk switch still reach it.
- Kept on Run: Chart, Globe, Galaxy chart, Your systems, Sector map, Cosmic web, Galaxies nearby, Sky from here, Cutaway, Codes, Route and Route to here, Voyages, Ships and stations, Land, Hazard map, Atlas, Switchboard. The Hazard map is not in section 5's table; it is kept on Run because "what will hurt us on the way" is a question asked at the table.
- Not in this part: the folded rail (one button naming the instrument in view), the Workshop door in More only, and the frame's other parts (TM2 part 2 on). The Workshop scale's own entries are unchanged.

## Checks

- `tests/ui/desk-rail.test.ts` reads every shipped screen document: the desks are well formed, Explore and Build list every entry as before, and Run lists all but the seven. Seen red with the documents' change taken out.
- `npm run ui:check` clean. Browser runs on Chromium passed: shell, bookmarks, delivery, hold-grounds, kits, known, landing-table, object-menu, playbook, steer, switchboard, table (every e2e file that uses the Run desk).
- `run-through.e2e.mjs` fails at step 4 ("Beowulf flies down from orbit on its drive": the verdict is `landed-on-a-bad-site`). It fails the same way on main at 1bdcc04 without this change, it is not in CI, and it touches no rail. It arrived with this morning's kernel merges (ship sizing, icy ground); reported to Alex.
