# RK12 part 2: range rings as a HUD overlay

Part 1 (light thread, `handover/packages/RK12.md`) built `effect_at_range`: what arrives, the three readings, the odds,
and the distances where each changes. This part draws those distances as rings round the one who holds the weapon.
Plan section 13 and question 16 ("a ring on the chart, an overlay under HUD").

## What landed
- **`packages/ui-contract/src/range-rings.ts`**: the kernel's ask and answer typed (`RangeAsk`, `RangeEffect`);
  `rangeRings(effect, name)` gives the rings to draw (in range, marginal, even odds; a slug, whose energy does not
  fall with distance, gets only the odds ring); `ringPoints` places true points round a centre for a view to project;
  `RANGE_RING_LAYER = "range"`; `rangeText` for distances.
- **Session `effectAtRange(asks)`** on `Session` and `ViewSession`.
- **`packages/views/system-chart/src/RangeRings.svelte`**: an SVG group for the chart. The chart's distances are a
  log scale about the primary, so a ring round a ship is bent: each point is placed true and projected with the
  chart's own `project` (metres to px). Solid for in range, dashed for marginal, dotted for even odds; each with its
  words as a tooltip and its kind as a small label.
- **`packages/views/system-3d/src/range-rings.ts`**: `rangeRingLines(rings, centre_m, toScene, colours)` gives point
  loops for the 3D view's ring pass (and its canvas fallback), in whatever scale the view shows.

## Mounting (chart room thread, agreed in the brief)
- `HUD_OVERLAY_LAYERS` gains `"range"` (ui-contract `view-controls.ts`).
- System chart: inside the SVG, `{#if on("range")}<RangeRings {rings} centre_m={…} project={…} />{/if}`.
- System 3D: spread `rangeRingLines(…)` into the frame's `rings` when `hud`.
- Where the rings come from: a weapon on a construct and a chosen target. No pack carries weapons yet; the Traveller
  pack maps its own (its thread), so the first source is likely there.

## Tests
- Fixture for views until a pack carries weapons: `packages/ui-contract/test/fixtures/range-effects.json` (asks and the kernel's answers; a test checks it still matches the kernel).
- `packages/ui-contract/test/range-rings.test.ts`: points on the circle, distance words, and against the real kernel a
  20 MW laser against a typed armour figure (in range inside marginal, "just enough" between) and a slug with only an
  odds ring.
