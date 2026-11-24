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

## Part 3 (waits on kernel BC1)

The strip's exact answer on a pause through `resolved_system {branch}`; Titan on the Air strip's moon dots; the walk step "Change a world".
