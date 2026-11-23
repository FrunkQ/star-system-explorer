# Sector map on the shared view controls (V10a)

Owner: Traveller pack track. Alex, 23 Nov: style, view, HUD and so on at every
level of the interface, consistent and repeatable.

## What it does

- The Sector map declares the renderer thread's shared controls
  (`SECTOR_CONTROLS` in `packages/views/sector/src/controls.ts`):
  `flatChart(...)` plus `hud: { chart: true }`. View style, Seen from and Scale
  list their options as on every other screen, with the reasons the hex map
  cannot take them (it is a flat grid).
- The view reads `viewOptions(SECTOR_CONTROLS, ws.view).shown.hud`. On a
  diagram HUD starts on. Off leaves the grid and each world's own mark (the
  `world` and `world-wet` glyphs) and strips hex numbers, names, every other
  glyph (port letters, bases, giants, belts), zone rings and the reach shading.
  The picked hex keeps its outline: that is the press, not the HUD.
- `HexMap` takes `hud` (default on) and marks the SVG `data-hud`.
- Fixes found by the new browser test: a hex took a press only on its outline
  (no fill), and a world's mark swallowed the press over it. Hexes now take a
  press anywhere inside; marks and words let it through.

## Checks

- `tests/ui/sector.e2e.mjs`: the first steps of the stage 5 walk (add the
  Traveller pack, Sector map from the galaxy rail, import four Lanternfall
  worlds, pick Ashgrove), then HUD on and off by counts. `SSE_SHOTS=<dir>`
  saves the two screenshots. In `ui:e2e`.

## Next

- The Traveller grid on other charts as a pack overlay under HUD (gap 18): needs
  a hook from the renderer thread for pack-contributed overlays.
- Naming the map's overlays one by one in the view's Layers tool.
