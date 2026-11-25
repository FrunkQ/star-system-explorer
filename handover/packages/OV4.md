# OV4: the Atlas look for territory areas (24 Nov 2026)

Owner: "Build chart pencil and Atlas look", taken over from the renderer at the coordinator's word. The chart room
agreed the edits to `Overlays.svelte`, the theme tokens and the one line in `Symbols.svelte`. Design:
`design/SSE4-territory-overlays-design.md` section 6, with OV2 (the contract) and OV3 (the kernel's holdings).

## Map palette

- The themes gain tokens `map-1` to `map-8` (`--sse-map-n`). Chart room and Night chart each have eight colours tuned to their ground. Deck takes the Night chart's colours.
- Paper and Phosphor (one colour) set every slot to their ink, and the slot's pattern carries the answer: plain, stripes, dots, cross, waves, grid, back-stripes, chevrons.
- `colour-vision.test`: every full-colour theme keeps all 28 pairs of slots at ΔE 10 or more apart, for typical vision, protanopia, deuteranopia and tritanopia. Every slot is at least 3:1 against its ground.

## Geometry (`ui-contract/src/areas.ts`, pure)

- `areasFromHoldings(holdings, ask)` turns the kernel's holdings into `AreaItem`s in the view's frame. The view gives positions; the kernel sends no geometry.
  - Holds compete by OV3's reach rule: nearest within reach wins, ties go by address, borders run midway, and space beyond every reach stays bare.
  - Each claimant's claims and each influence are drawn on their own, so they may lie over a holder.
  - An outline in the view's frame is drawn as stored.
  - Ended holdings and places the view cannot place are left out.
  - The trace runs a reach past the view, so a territory that runs off the view has no border along its edge.
- `traceRings` is marching squares on a field. The border sits where the field crosses zero along each cell edge, so reaches come out round and borders between stars straight. Points on a straight run are dropped.
- `letterArea` gives the long axis of the largest piece, its length and width, and an angle that never reads upside down. `areaShares` gives the key's shares. Also exported: `insideRings`, `itemArea`, `nearestWithin` and `parsecs`.

## Drawing (`chart-grammar/src/Overlays.svelte`, areas branch; `AreaKey.svelte`)

- **Atlas look.** Three clipped strokes, each wider and fainter, give a band of the answer's colour along the inside of its border, fading to a bare middle.
- **Lines and fills by status.**
  - The line says the status: solid holds, dashed claims, dotted influences. Influence tint follows strength.
  - A claim is hatched in the claimant's colour over the holder's tint.
  - A divided place is striped in its other holders' colours.
- **Lettering.** A holder's name is lettered along the area's long axis in spaced italic capitals, sized to the area. It drops out below 9 px or when the name is longer than the area.
- **Pencil.** An area the table does not know is drawn in pencil (SV3).
- **Pointing and pressing.** With `onpick`, pointing inks the edge and calls `onpoint`; pressing calls `onpick`. `lift` quietens every answer but one.
- **AreaKey.** One row per answer: its colour, with the pattern where colour cannot carry it; how it stands (holds, claims, influence); and its share in view. Pointing or focusing a row lifts its territory, and pressing a row calls `onpick`.
  - Rows are 44 px tall on touch. `compact` collapses the key to chips. `floating` sets `data-chart-overlay`, so the chart keeps names clear of it.
  - Answers the table does not know are shown in pencil.

## Where to see it

The Symbols screen has a new last section, "Map colours" (`views/symbols/src/MapColours.svelte`). It shows the palette and a made-up sector drawn by the same code: three powers, a claim across a border, a guild's influence, and a secret holding that only the GM's Build and Run desks show, in pencil. Views carry real area layers in OV5 (the galaxy and travel threads feed positions to `areasFromHoldings`).

## Checks

- `ui-contract/test/areas.test.ts`:
  - rings with holes;
  - drawn borders agree with the reach rule at 1,000 random points (points within one cell of a border are skipped, and more than 850 are checked);
  - every held place lies inside its own power;
  - shares sum to one;
  - lettering axes;
  - ended holdings, unplaced places, pencil and outlines.
- `skins/player/test/colour-vision.test.ts`: the map palette, as above.
- `tests/ui/symbols.e2e.mjs` checks:
  - 8 slots, with bands, hatching and lettering present;
  - neighbours in different colours;
  - pointing names the holder, and the key lifts a territory;
  - the secret in pencil on Build;
  - the Night chart;
  - the phone;
  - the state hash unchanged.
- Screens are in project files: `sse4/handover-screens/OV4-screens/` (map-colours-desk, map-colours-night, map-colours-phone).

## Next

- OV5: views feed `areasFromHoldings` from `holdings_in` and draw `Overlays` with `AreaKey`.
- Comparing with the kernel's own `who_holds` on real drawn systems belongs in OV5, where a view has the positions. The rule here is the same one, checked at 1,000 points.
- OV8: the ruler ticks and the 240 ms fade across a border change.
