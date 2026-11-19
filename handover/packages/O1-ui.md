# O1 (UI side): the Light switch

Design: `SSE4-nebulae-and-light-design.md` sections 1 and 2 (light descriptions, the band set, recipes and their honesty labels).

## What landed

- `packages/ui-contract/src/light.ts`
  - Band set (gamma to radio) and line table ([O III], H-beta, H-alpha, [N II], [S II]).
  - `LightDescription`: continuum parts (thermal, reflected), lines, `stubbed`, `why`.
  - Planck radiance, band and narrowband integrals, CIE 1931 2° matching functions (Wyman, Sloan and Shirley 2013 fit), XYZ to sRGB.
  - `thermalColour(T)`; `paint(recipe, lights)` for four modes: eye, camera (white-balanced to 6504 K, saturation raised), composite, single (log ramp across the view, with a floor 30 decades below the brightest visible light).
  - Draper point: heat below 798 K gives no light in visible-light recipes.
  - `checkLightRecipe` (schema `sse.band-recipe/0`): only eye may claim actual colour; composite and single are mapped; every recipe has a legend and a why.
  - `describeLight` builds **stubbed** light from scene facts: a star is a blackbody at its derived `teff_k`; a planet reflects its star with a grey albedo (diluted by (R*/a)²/4) plus its own heat at `surface_k`.
  - `litFromSystem` (GM scene) and `litFromPlayers` (players' facts; a star's temperature from its spectral type, stubbed).
- `packages/shell/src/documents/light.recipes.json`: Eye, Camera, Hubble SHO, bicolour HOO, near IR, thermal IR, UV, X-ray, radio. Data only.
- `packages/shell/src/LightControl.svelte`:
  - On the GM status bar and the players' bar.
  - "Chart colours" (the symbology) is the default.
  - Any recipe shows a legend chip with the honesty label, plus "stubbed spectra" while the light is stubbed.
  - The panel says what the colours mean and lists channel swatches, and has a "why?" for every body and a line naming bodies with no known light (drawn in ink).
  - The choice is kept in localStorage (`sse4.light.gm`, `sse4.light.players`) and never reaches the session.
- The system chart's body groups carry `data-address` (for tests).

## Evidence

- `packages/ui-contract/test/light.test.ts` (7 tests):
  - 3000, 6500 and 10000 K land within 10/255 per channel of Mitchell Charity's blackbody table.
  - Planck at the Sun's peak is right to 2 %.
  - All 9 shipped recipes pass; overclaiming recipes are refused.
  - Stubbed descriptions say so.
  - The mid infrared shows Tarn by its own heat; X-rays show nothing from thermal light alone.
  - Players' light uses only facts their view gives.
- `tests/ui/light.e2e.mjs`:
  - Every recipe shows its legend chip and legend, and the star's colour equals Node's for the same derived scene.
  - The state hash never moves, and going back to chart colours restores the symbology mark exactly.
  - The choice survives a reload, and players have the control.
- Screens: `O1-screens/`.

## Not done, and why

- **Real light facts.** The kernel has no light op, so every description is stubbed, and the UI says so. When the kernel publishes `LightDescription` facts (lines, dust, coronae, redshift), swap `describeLight` for them; recipes and the control stay as they are.
- **No nebulae or line emitters in any scene yet**, so the SHO and HOO palettes show continuum only. Their legend says so.
- **3D view.** It takes its colours from the same mark function, so it follows the switch. It has no exposure or stretch control yet (design 2.3).
- **Sensor-limited seats (O8)** come later.

## Caveats

- The Charity table values and the Wyman 2013 reference were written from memory. Verify them along with the other design citations.
