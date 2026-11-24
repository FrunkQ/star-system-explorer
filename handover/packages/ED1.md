# ED1: Handles and the strip

Status: part 1 done 24 Nov 2026 (thread "Build planet and star editing"); part 2 next. Design: `handover/design/SSE4-editing-design.md` sections 2, 3, 3B and 6 (accepted by Alex 24 Nov 2026).

## Part 1: the handle sheet and the strip model (this PR)

- `packages/ui-contract/src/handles.json`, the handle sheet: a world's six handles (Orbit, Mass, Makeup, Liquids, Air, Spin) and a star's two (Mass, and Spin per the 10B finding F5), each with its kernel field, kind and "about" for the units engine, its scale (log, linear, bar), the Solar System bodies drawn as reference dots, and the desks it moves on. Surface water is held in Earth oceans (`per: M_ocean_earth`). Spin's figure carries the tilt. Makeup is a bar (ED9) and waits on kernel K3 for hand-made worlds. The sheet also lists the results "what followed" watches.
- `packages/ui-contract/src/handles.ts`: the scale class for the units engine (star, moon, giant from ten Earth masses, planet); a handle's figure; the strip's range (the dots and the value, a factor of four either side on a log strip), position and value along it, "none" at the left end for air and seas, stretch for a typed value past the end, arrow steps (Shift ten times bigger); reference dots from packs/base-sol; `resultsOf` (a world's results from `resolved_system`'s surface entry) and `whatFollowed` (results that moved, said by the units engine).
- Found in a probe of the Solar System in Node: `resolved_system` returns only the facts as set (catalogued or authored), so results come from its `surfaces` entry: moving Mars to 1 AU moves its surface from 214 K to 264 K and its heat from tides. A resolve takes about 0.1 to 0.2 s.

Checks: `packages/ui-contract/test/handles.test.ts`.

## Part 2 (next)

The row and the strip on the body card, in `packages/views/editing/`; see the ED1 row of `handover/status/editing.md`.
