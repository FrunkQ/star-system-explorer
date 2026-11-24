# W-sea2 to W-sea4, drawing side: the sea's colour, waves and froth from the liquid's own numbers

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

- W-sea3 (waves): `trainsToDraw` gives the shader six trains (k, amplitude, ω, heading, phase): the kernel's own
  when sent, else `standInTrains` (gpu-surface.ts, the V10k stand-in in the same form). The shader sums six per-train
  uniforms; the note names the kernel's trains when they are drawn.
- W-sea4 (froth): `frothToDraw` (share, reflectance, heading: the kernel's streak heading or the wind's); the shader
  whitens soft jittered streaks in 1 × 6 m cells along it, chosen so the white area is the kernel's share
  (`FOAM_FILL` 0.35, the mean fill of one streak). The note gives the froth's share.

## Evidence
- `packages/renderer/test/sea.test.ts` 4 of 4: reader and missing names; Fresnel at normal incidence equals
  ((n−1)/(n+1))², 1 at grazing, half the s term at Brewster's angle, methane (n 1.27) reflects less than water;
  shallow water shows the floor with red fading first, deep water is the deep colour, no numbers keep the old look;
  the landing's sea carries the numbers.
- A synthetic flat sea through `createSurfaceGpu` in Chromium (scratch page, not committed), screens in project files
  `sse4/handover-screens/W-sea-screens/`: with no kernel numbers it draws exactly as before; water numbers give a teal
  body with the floor showing in the shallows, three trains and 4% froth streaks; methane numbers under Titan's
  gravity give a clear brownish liquid with slow long swell. No console or page errors.
- `sea.test.ts` adds: the kernel's trains drawn as given (k = 2π/λ, ω = 2π/T, heading), padding to six, the stand-in
  without them, methane's slower trains; froth null until sent, clamped, heading from the wind or the kernel.
- `npm run ui:check` clean; `npm run ui:test` 589 of 589; `surface.e2e` chromium passes (the shader compiles with the new uniforms and attributes; with no kernel numbers yet the look is unchanged).

## Limits
- The diffuse path through the water is taken as the depth (fudge register, V10k row).
- The globe's sea per climate band (`survey.climate.sea.bands[]`) is read by the same reader; drawing it on the globe
  waits for the bands to arrive (climate PR 418 adds wind_m_s per band).
- Ripple detail to `ripple_cutoff_m` is not drawn yet; `superfluid` and `liquid_note` reach the note.
- Where froth lies is a picture choice (fudge register); whitecaps are not tied to the crests of the trains.
