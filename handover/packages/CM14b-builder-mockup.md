# CM14b: the ship builder, to the mock-up (compound ships)

Track CM. Asked for by Alex 28 Nov 2026 13:17 UTC ("why are you still using the old 2d system we are deleting ... You have the mockup"), relayed by the coordinator with the interactive mock-up (`sse4/handover-screens/CM-builder-mockup/ship-builder-mockup.html`, published at https://claude.ai/artifact/UkAcdXMBZhHSnRmL13ewzN). Built 28 Nov 2026 by the thread "Build the 3D ship builder from the mock-up". TypeScript, theme tokens and tests; no kernel change, no golden moves.

## What it does

- **The mock-up's working screen**, `packages/views/ship/src/ShipBuilder.svelte`: top bar (Edit › Ship builder, name, "Parts from" year slider with "predicted parts", Example ships, What can it do?, full screen); parts on the left by era, every part listed with its year and dry mass, parts not flown by the year faint, future parts marked predicted, "Clip on ×1 to ×4" copies round the hull; the stage in the middle with View style (Chart, 3D, Wireframe; True held), Seen from (Above, Tilted, Edge-on, Free; a short turn), Undo, the connection-point legend and the hint line; Selected, Build tree and Paint and name on the right; the shopping list along the bottom. On a phone the parts and the inspector are sheets, as the mock-up's.
- **3D part by part over the blueprint room**, `packages/renderer/src/builder-scene.ts`: each part is its own mesh (from the same `ship-mesh.ts` every ship uses), so a part can be pressed, glows when selected and takes its own paint. Behind it a room whose far wall, floor and end wall carry the build's side, top and front drawings and a grid with their measures, drawn by a fixed camera: the blueprint never turns with the model (ruling 14A.2, the mock-up's still backdrop).
- **Glowing connection points**: arm a part (press it, or drag it from the list) and the free points light up, coloured by kind (ends blue, hull green, bays violet); hovering one shows where it is ("On the hull of ... · 90° round, 30% along") and a see-through ghost of the part there; pressing or dropping clips it on. The part stays picked; Esc stops. Hull points sit at stations along the part (more on a longer part) and at fewer clocks when copies go round together (`freePoints(..., { sym })`).
- **Editing**: wheel over the selected part stretches it, Shift + wheel widens it (real hardware says it comes in its pack's sizes); Length, Across and Round sliders; Remove takes off a part, its copies and what hangs from it; Delete and Ctrl+Z; paint swatches per part and "Paint the whole hull" (the paint is kept in the recipe).
- **Shopping list**: "Nothing set" shows the build's figures (Δv, masses, tank room, room aboard, radiator, size); "As capable as" any catalogue ship gives bars that count down, go green when covered and show overshoot; "Parts you should use" lists what is short; mass and thrust to weight at Earth; the notes popover.
- **The old designer is gone from every screen**: the style switch and Remix are removed from ShipPlans; a ship's Picture card (Explore, Run and Build) is the builder (the picture alone off the Build desk); the ship at a world (Voyages) is drawn from its parts; the party's ship landed on a surface is built from its parts (a kept build, else its record opened as parts).
- **Workbench theme tokens** (`packages/skins/chart-room/src/schema.ts`): `bench-*`, the dark blueprint ground of a builder, kept dark in every theme (Paper draws it ink on paper, Phosphor in its green), so views name no colour (theme-free rule).

## What it does not do yet

- Cutaway and Sensor views are shown but held (bays holding parts come with CM8; sensors with CM13 part 3). "Build it for me" is left out: it was the generator, which is being retired.
- Lettering and badges on the hull; finishes other than paint.
- A ship fitted for a world (heat shield, lift, legs) and the aerobraking pass are still drawn by the old fitting rules (`ShipPlans`, now without style or remix) until shields, lift and legs are parts a build carries.
- Catalogue ships still open by converting the old generator's layout (convert on open); the generator code goes once every catalogue ship is saved as parts.
- Hulls take the base-ships style's skin colours (darker than the mock-up's white) until a finish is chosen.

## Checks

- `tests/ui/ship-builder.e2e.mjs` (desk and phone): the catalogue ship opens as parts in 3D; a longer tank raises Δv; a radiator armed from the list lights points, the pointer finds one by its tip and a press clips it on; a plan's bars show; Chart draws every part; What can it do?; Example ships to a blank build started by a part; no sideways scroll on a phone; nothing written. It puts the system on Chart first: True (the alpha first-visit default) drawn in software starves headless Chromium.
- `tests/ui/manifest.e2e.mjs`, `tests/ui/voyage.e2e.mjs` and `tests/ui/run-through.e2e.mjs` pass with the pictures from parts.
- `packages/renderer/test/ship-build.test.ts`: free points by stations and by copies round the hull (seen red with three clocks for two copies).
- `npm run ui:test` (1,229 pass), `npm run ui:check` clean; theme-free, figures ledger and both skin theme suites pass with the new tokens.

## Pictures

Project files `sse4/handover-screens/CM14b-screens/`: catalogue, placing, plan and can-do, desk and phone.
