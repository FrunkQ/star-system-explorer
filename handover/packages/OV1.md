# OV1: pack overlays under the HUD switch

V3 gap 18 ("grids are layers in the view controls on the map of your systems, nothing per screen"), asked for by the Traveller thread on 23 Nov 2026: the Traveller hex grid laid over the galaxy-scale maps.

## What it does

- A stacked pack declares overlays as data in `provides.other.overlays`: `{ id, label, frame, grid?, layers? }`. `grid` is either a grid (the hex map's GridConfig shape) or the id of a view config whose grid to use, such as `"sse.view.hex-map"`. `layers` holds fixed lines, hexes and labels in the overlay's own coordinates, such as borders. A higher pack's overlay with the same id wins.
- Frames: `galaxy-plane-pc` (x and y in parsecs on the galactic plane, the star cells' `xyz_pc`), `cluster-pc` and `system-au`. A view draws the overlays in the frames it carries.
- Where the GM put each overlay is kept with the campaign: `workspace.overlay_anchors[id] = { origin, rotation_deg }`. For a grid, the origin is hex 0101's centre. It goes into the campaign save. Players don't see the galaxy-scale maps yet. When a players' map shows overlays, the anchors move into the chronicle.
- Detail by zoom: the sheet's outline and its subsector bounds always show; subsector names show when a block is at least 24 px; hexes show from 6 px across (at most 4,000, and only those in view); hex labels show from 28 px.
- Overlays are drawn only while the view's HUD is on. A view with no HUD switch always shows its labels, so it always shows them.
- The first view to carry them is Voyages, between stars. Pick a star and "Put its first hex on …" places hex 0101 there; "Take it off the map" removes it. The overlay is drawn by chart-grammar `Overlays.svelte` in theme ink.

## Code

- `packages/ui-contract/src/overlay.ts`: types, `overlaySpecsOf`, `overlayGrid`, `hexCentre`, `hexLabel`, `placeBy`, `gridOverlay`, `overlaysFor`, `hexCorners`.
- `packages/chart-grammar/src/Overlays.svelte`.
- `Workspace.overlay_anchors` and `setOverlayAnchor`, kept in `CampaignSave.workspace` (App.svelte save and open).
- `voyage/src/Stars.svelte`: draws the overlays and has the Overlays rows in the ledger.

## Checks

`packages/ui-contract/test/overlay.test.ts` (3 tests): stack reading and precedence, hex geometry and labels, anchor placement, detail by zoom.

There is no e2e yet: no shipped pack declares an overlay. The Traveller thread adds `overlays` to the Traveller pack. The screen in `OV1-screens/overlay-voyage.png` was made with that entry injected: hex 0101 on the picked star, 1 pc hexes.

## Next

- The Traveller pack's `overlays` entry (Traveller thread), then an e2e.
- Turning a placed grid (rotation_deg is kept; there is no handle for it yet) and dragging it.
- The galaxy chart and the "Your systems" map (V3 gap 17) carrying `galaxy-plane-pc`, and the Layers tool listing overlays.
