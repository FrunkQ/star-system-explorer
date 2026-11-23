# V3 gap 18 part 1: the Traveller sector grid on the stars

Owner: Traveller pack track. Built on the renderer thread's overlay slot
(`packages/ui-contract/src/overlay.ts`, PR 271).

## What it does

- The Traveller pack declares one overlay as data:
  `provides.other.overlays: [{id: "traveller.sector-grid", label: "Sector grid",
  frame: "galaxy-plane-pc", grid: "sse.view.hex-map"}]`. The grid is the pack's
  own hex map grid (8 by 10 subsectors, 4 by 4 of them, 1 pc a hex), so the map
  and the overlay can never disagree. No code in the pack.
- The GM places it on Voyages, Between stars: pick a star, **Put its first
  hex on …**; **Take it off the map** removes it. The anchor is kept in the
  campaign (`workspace.overlay_anchors`). Subsector bounds, names and hex
  labels come and go with the zoom (the slot's work).

## Golden

`tests/golden/traveller-codes.json`: the Traveller pack hash only.

## Checks

`tests/ui/sector.e2e.mjs` now goes on to Voyages: the overlay row says not
placed, a star is picked, the grid is placed (shapes drawn) and taken off
again. `SSE_SHOTS=<dir>` saves `sector-grid-on-stars.png`.

## Next

- The grid on the galaxy chart, with drag and turn (the renderer thread's
  next step on the slot); nothing to change in the pack.
- Worlds imported from a sector file sit on the map's hexes, not yet at the
  placed grid's real star positions: tie an imported world's system to the
  star under its hex once the grid is placed.
