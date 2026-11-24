---
name: traveller-overlay-contract
description: Agreed shape (23 Nov) for pack overlays on other charts; renderer owns the slot, Traveller thread fills it with the sector grid (gap 18)
metadata:
  type: project
  modified: 2026-11-23T14:34:30.776Z
---
Agreed 23 Nov 2026 between the renderer thread and the Traveller pack thread, for V3 gap 18 (Traveller hex grid on other charts).

- Renderer owns `ui-contract/overlay.ts`: ViewOverlay {id, label, pack, frame, under_hud: true, layers}; layers are lines, hexes {centre, size, orient flat|point, label?} or labels {at, text}; frames galaxy-plane-pc, cluster-pc, system-au. A view says `overlays: {frames}` in its spec, draws them only while HUD is on, lists them by label in its Layers tool.
- The campaign holds `overlay_anchors: {[pack]: {origin_pc: [x,y], rotation_deg}}` (origin is hex 0101 on the galaxy plane); the GM drags the grid on the galaxy chart to place it. The request is {frame, bounds_pc, anchor}. No anchor: return no overlays and a reason ("place the sector on the galaxy first").
- Revised (PR 271): no generator. A pack declares `provides.other.overlays: [{id, label, frame, grid: "sse.view.hex-map"}]`; the slot reuses the hex map's grid and draws subsector bounds, names and hex labels by zoom. Extras go in `layers` in the sheet frame (hex 0101 at 0,0, rows down to -y). Anchor in workspace.overlay_anchors (in the save); GM places it on Voyages for now (pick a star, "Put its first hex on …").
- Traveller thread adds the entry to the Traveller pack and a voyage e2e counting [data-overlay] polygons.

**Why:** Alex 23 Nov: style, view, HUD at every level, consistent; a pack's grid is an overlay under HUD, not a switch of its own.
**How to apply:** don't add per-view switches for pack overlays; build to this contract. Related: [[traveller-pack-track]].
