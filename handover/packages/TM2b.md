# TM2b: the Run desk's bar, tools and status line (track TM, TM2 part 2)

Status: done 26 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 3.1 (accepted 25 Nov 2026). Each change is a `desks` field on a `SHELL_PARTS` row (`packages/ui-contract/src/desk.ts`) that the shell reads with `partShows()`.

## What it does (Run only; Explore and Build are unchanged)

- **Depth leaves the bar and lives in More.** The dial is not drawn in the bar on Run; More carries it at every width there (on a narrow screen it was already in More). Run remembering its own depth and starting at Curious is TM6.
- **The Workshop door leaves the foot of the rail.** It is in More, as before (design section 5: "In More only"). `Rail.svelte` takes `workshopDoor`.
- **The stage's tools drop Select and Keyboard shortcuts.** The ? key still opens the shortcuts. Bookmarks stay. Measure and Send on the stage are not built here; Send is the Playbook's.
- **The status line drops the seat and the engine's numbers.** "Kept" stays, and so does the table's own line (`TableStatus`, without its chip, which is in the bar on Run as before). The design says "Kept only"; the table's line is kept because it carries the acts of play waiting to be sent (a landing, an orbit), which belong on Run.
- `Stage.svelte`'s Undo and Redo now ask `partShows("tools", desk, "undo")` instead of naming Build: the desk-test ledger goes from 27 to 26 and Stage.svelte leaves it.

## Not in this part

The folded rail (one button naming the instrument in view), the place line as the stepper (TM7), the clock in place of the ruler and Light leaving it (TM3), the Run card (TM5, chart room), See as with three kinds only, More without the edit group (it already hides on Run: `clipArmed` is Build's).

## Checks

- `packages/ui-contract/test/desk.test.ts` pins the rows: depth and the Workshop door Explore and Build; Select and Keyboard shortcuts Explore and Build; the status line's seat and engine off Run, Kept on every desk.
- `npm run ui:check` clean.
- Browser runs on Chromium passed: shell, walk (press budgets), lenses, bookmarks, delivery, hold-grounds, kits, known, landing-table, object-menu, playbook, steer, switchboard, table.
- Looked at in the browser at 1440 x 900 on the Kestrel chart: on Run the bar has no depth dial, the stage tools are Bookmarks alone, the rail has no Workshop door, the status line reads "Kept 16:51 · Share with players"; on Explore all are as before. Screens in project files `sse4/handover-screens/TM2b-screens/` (Explore.png, Run.png).
