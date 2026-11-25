# ED3: Dragging on the chart

Status: part 1 done 24 Nov 2026 (thread "Build planet and star editing"). Design: `handover/design/SSE4-editing-design.md` section 4.

## Part 1

- `packages/views/system-chart/src/SystemChart.svelte`: dragging a world along its radius now leaves the world where it is and draws a ghost (its mark at 55%) with the new orbit line, as the design says ("the world itself stays put until you let go"). The readout beside the ghost is the units engine's: AU round a star, thousand km round a planet, in the campaign's chosen reading, at the workspace depth. The same words go into the edit's display, so Undo lists it the same way.
- Esc during a drag cancels it: nothing is written (a capture listener for the length of the drag, so the shell's Esc does not also clear the selection).
- The chart does not select text while dragging.
- `packages/views/editing/src/Strip.svelte`: the strip's Esc stops there, so it no longer also clears the world's selection.

Checks: `tests/ui/editing.e2e.mjs` (desk): drag Mars outward, the ghost and an AU readout appear and Mars stays put; Esc takes the ghost away and the state hash does not move. Screens in project files `sse4/handover-screens/ED3-screens/`.

## Part 2a (done 25 Nov 2026)

- `packages/ui-contract/src/drag.ts`: `gearing` (all of a move along the radius counts on the line, less off it, 1/(1 + off/80 px); Shift a tenth), `roundFigures` (1, 1.5 … 9.5 times a power of ten in the unit shown: AU round a star, km round a planet), `resonanceCatches` (the nearest of 2:1, 3:2, 4:3, 5:3, 5:2, 3:1 with the nearest neighbour inside and out, years as size to the 3/2 round one host; the Orbit box's chip uses the kernel's own years), `nearestCatch` (within 6 px on the chart).
- `SystemChart.svelte`: the drag keeps its own radius and moves it by the geared pointer; a catch sets the value exactly and a resonance is named beside the readout ("3.89 AU · 5:3 with Ceres"); Alt runs free; the screen-reader hint says so.
- Checks: `packages/ui-contract/test/drag.test.ts` (4 tests); `tests/ui/editing.e2e.mjs` accepts a named catch on the readout and checks Alt leaves no catch.

## Part 2b (queued)

The readout's year, the surface's quick look and the ST1 verdict (lasts, watch, unstable); the "Where things can orbit" layer turning on for the drag; magnetism to tipping points (with the sweep); the far-point grip (shape and turn) and the lean grip; dropping on a new parent is ED4. The year and quick look want the kernel's sweep (K1) or a resolve per release, never physics in the view.
