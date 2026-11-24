# ED3: Dragging on the chart

Status: part 1 done 24 Nov 2026 (thread "Build planet and star editing"). Design: `handover/design/SSE4-editing-design.md` section 4.

## Part 1

- `packages/views/system-chart/src/SystemChart.svelte`: dragging a world along its radius now leaves the world where it is and draws a ghost (its mark at 55%) with the new orbit line, as the design says ("the world itself stays put until you let go"). The readout beside the ghost is the units engine's: AU round a star, thousand km round a planet, in the campaign's chosen reading, at the workspace depth. The same words go into the edit's display, so Undo lists it the same way.
- Esc during a drag cancels it: nothing is written (a capture listener for the length of the drag, so the shell's Esc does not also clear the selection).
- The chart does not select text while dragging.
- `packages/views/editing/src/Strip.svelte`: the strip's Esc stops there, so it no longer also clears the world's selection.

Checks: `tests/ui/editing.e2e.mjs` (desk): drag Mars outward, the ghost and an AU readout appear and Mars stays put; Esc takes the ghost away and the state hash does not move. Screens in project files `sse4/handover-screens/ED3-screens/`.

## Part 2 (queued)

The readout's year, the surface's quick look and the ST1 verdict (lasts, watch, unstable); the "Where things can orbit" layer turning on for the drag; geared dragging (further from the line counts for less) and Shift finer; light magnetism to tipping points, round figures and resonances, named as it catches, Alt off (question 10, Alex yes); the far-point grip (shape and turn) and the lean grip; dropping on a new parent is ED4. The year and quick look want the kernel's sweep (K1) or a resolve per release, never physics in the view.
