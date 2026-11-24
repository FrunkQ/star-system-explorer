# W-sea2, drawing side: the sea's colour from the liquid's own optics

Owner: "Build surface time-lapse". Kernel side: "Build shared climate and weather" (W-sea2), which sends the numbers
in the weather report's `data.sea` (site) and `survey.climate.sea.bands[]` (globe). Contract:
`handover/status/surface-timelapse.md`.

## What was built
- `packages/renderer/src/sea.ts`: `readSeaOptics` reads every agreed key (colour, wave trains, froth), drops
  malformed ones and names what is missing per step; `fresnel(n, cosI)` (exact, unpolarised); `seaBody` (deep
  reflectance and the floor through exp(−2az)); `seaWords` for the Land screen's note.
- `packages/renderer/src/sea-node.ts`: the Fresnel split as a TSL node.
- `packages/renderer/src/gpu-surface.ts` (sea block, agreed with the renderer thread): uniforms for the deep colour,
  absorption and index; mesh attributes `depth` and `floor`; the water's body is the kernel's when sent, the old
  ground colour otherwise; Fresnel at the liquid's index when sent, Schlick otherwise.
- `packages/renderer/src/surface-patch.ts`: `groundMesh` fills `depth` (liquid over the ground) and `floor` (tile
  albedo).
- `packages/ui-contract/src/landing.ts`: `LandingSea.optics` carries those keys, so seats watching a landing draw the
  same sea. `packages/views/surface/src/Surface.svelte` passes them to the GPU and says them in the note.

## Evidence
- `packages/renderer/test/sea.test.ts` 4 of 4: reader and missing names; Fresnel at normal incidence equals
  ((n−1)/(n+1))², 1 at grazing, half the s term at Brewster's angle, methane (n 1.27) reflects less than water;
  shallow water shows the floor with red fading first, deep water is the deep colour, no numbers keep the old look;
  the landing's sea carries the numbers.
- `npm run ui:check` clean; `npm run ui:test` 589 of 589; `surface.e2e` chromium passes (the shader compiles with the new uniforms and attributes; with no kernel numbers yet the look is unchanged).

## Limits
- The diffuse path through the water is taken as the depth (fudge register, V10k row).
- The globe's sea per climate band (`survey.climate.sea.bands[]`) is read by the same reader; drawing it on the globe
  comes with W-sea3, once the bands arrive.
