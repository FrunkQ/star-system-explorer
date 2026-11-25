# ED1: Handles and the strip

Status: parts 1 and 2 done 24 Nov 2026 (thread "Build planet and star editing"); part 3 waits on the kernel's BC1. Design: `handover/design/SSE4-editing-design.md` sections 2, 3, 3B and 6 (accepted by Alex 24 Nov 2026).

## Part 1: the handle sheet and the strip model (this PR)

- `packages/ui-contract/src/handles.json`, the handle sheet: a world's six handles (Orbit, Mass, Makeup, Liquids, Air, Spin) and a star's two (Mass, and Spin per the 10B finding F5), each with its kernel field, kind and "about" for the units engine, its scale (log, linear, bar), the Solar System bodies drawn as reference dots, and the desks it moves on. Surface water is held in Earth oceans (`per: M_ocean_earth`). Spin's figure carries the tilt. Makeup is a bar (ED9) and waits on kernel K3 for hand-made worlds. The sheet also lists the results "what followed" watches.
- `packages/ui-contract/src/handles.ts`: the scale class for the units engine (star, moon, giant from ten Earth masses, planet); a handle's figure; the strip's range (the dots and the value, a factor of four either side on a log strip), position and value along it, "none" at the left end for air and seas, stretch for a typed value past the end, arrow steps (Shift ten times bigger); reference dots from packs/base-sol; `resultsOf` (a world's results from `resolved_system`'s surface entry) and `whatFollowed` (results that moved, said by the units engine).
- Found in a probe of the Solar System in Node: `resolved_system` returns only the facts as set (catalogued or authored), so results come from its `surfaces` entry: moving Mars to 1 AU moves its surface from 214 K to 264 K and its heat from tides. A resolve takes about 0.1 to 0.2 s.

Checks: `packages/ui-contract/test/handles.test.ts`.

## Part 2: the row and the strip on the body card

- `packages/views/editing/src/Handles.svelte`: the row of handles under the card's sentence (three columns, two on a phone). Figures are the kernel's facts for the body (`resolved_system`, reread on each revision), said by the units engine at the workspace depth. One strip open at a time; another body closes it, a reread of the same body does not. On Build (GM, no branch open) a strip moves the value; elsewhere it opens read-only, "where this world sits among the reference worlds". A change is one `commitEdit` (set_value) with the before value, so Undo takes it back; the card then lists what followed (the change and each watched result that moved).
- `packages/views/editing/src/Strip.svelte`: the line, the accent mark (a slider for keyboard and screen readers), reference dots from packs/base-sol and this system's other worlds (round the same body for an orbit), names thinned so they never run together. Dragging writes nothing: the figure goes to pencil and its rung is held. Letting go, or Enter after arrows, is one change; Esc puts it back. Pressing the figure takes typing in any unit of the kind (`readFigure`); a typed value is kept exactly and stretches the strip.
- Mounted from `packages/views/body-card/src/BodyCard.svelte` for the GM off the Run desk.
- Found: Enter in the entry bubbled to the strip and set the value twice (the entry now stops its keys); the card's subject is a new object after each change, so the strip is keyed to the address and the commit holds the address before it awaits.

Checks: `tests/ui/editing.e2e.mjs` (desk and phone: the six handles with Mars's figures, read-only on Explore with the Earth and Jupiter dots, typed "1 AU" is one change that warms Mars and Undo puts it back, a drag is one change, Esc writes nothing, no overflow on a phone). Screens in project files `sse4/handover-screens/ED1-screens/`.

## Part 3a: a what-if's own figures (done 25 Nov 2026, after kernel BC1, PR 468)

- `Session.resolvedSystem(system, at_s?, branch?)`: with a branch, the kernel resolves the what-if's own line and commits nothing.
- The handles row reads the open what-if's own resolve, and rereads it after each change there. So on a what-if, "What followed" lists what moved there: "Try changing it" with Mars at 1 AU on the example shows Surface about 214 K → about 264 K. The main line does not move. Before, the row said "What followed shows once it is kept".
- In the Orbit and Spin boxes on a what-if, the tides line (the surface's `locked`) now comes from the branch too. The year and the verdict stay hidden there ("The year and the verdict follow once it is kept"), because `orbits_at` and `orbit_stability` read the main line only. A `branch` on those two ops would finish it.
- `editing.e2e.mjs`: on the example, Mars at 1 AU is at least 20 K warmer. The handle's figure after a Build change is now waited for, not read once (the card rereads twice).

## Part 3b (still to do)

The strip's exact answer on a pause (the held value resolved on a scratch branch); Titan on the Air strip's moon dots; the walk step "Change a world".

## Part 3b (25 Nov): the held value's exact answer on a pause

- `Strip.svelte` takes `onpause(si)`: after 150 ms with the pencil still, it asks once; a later move makes an answer on
  its way stale; the line shows "Held here: …" before "Let go". `Handles.svelte` `pause(h, si)` calls `resolveWith`
  (`ui-contract/src/reach.ts`): a scratch branch gets the what-if's own edits (if any) and the held value, the kernel
  resolves it (BC1), the branch is discarded. The words are `whatFollowed` rows (first three). The Orbit box's own strip
  gets the same through Box's `pause` prop.
- e2e (`editing.e2e.mjs`, desk and phone): a drag held still shows the line and the state hash does not move.
  Screen: project files `sse4/handover-screens/ED1-screens/held-desk.png`.
- Left: Titan on the Air strip's moon dots, the walk step "Change a world" (part 3c).
