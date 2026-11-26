# XO4: the light-path stage and the black hole in True

Track XO (exotic objects), design `handover/design/SSE4-exotic-objects-design.md` sections 2.1, 3.1 to 3.4, 7 and 11. Built 26 Nov 2026 by the thread "Draw the first black hole picture", on top of XO2 and XO3 (PR 618).

## What was done

- **`packages/renderer/src/light-path.ts`**: the reference repeat of the kernel's light near a mass, in TypeScript, one function per kernel step: the bend table's row and cell lookup (`rowOfB`, `uAt`, never mixing rows across √27), the weak bender past 64 r_g, `landing` (the crossing angles φ₀ = π/2 + atan(cos α tan i) and φ₀ + π, first hit inside the disc's edges), the thin-disc temperature, the block `sse.light.disc_shift`, `traceRay`, the band table (Planck through the sheet's bands, and CIE to linear sRGB for natural colour), `brightest` (the exposure and each band's top) and `marksOf` (where the shadow's edge and the arch fall in the frame). Every constant through `constant()`.
- **`packages/renderer/src/gpu-light-path.ts`**: the GPU pass, a TSL transliteration of the functions above, one ray per pixel: the bend table and band table as float textures, the sky behind as two half-float textures, the shared finish (VP1) at exposure 1. Its own renderer and canvas; nothing in the shared renderer code changed.
- **`packages/renderer/src/hole-sky.ts`**: the kernel's `sky_from_here` stars laid into a fine patch round the hole and the whole sky, as radiance, so the lensed arcs and the Einstein ring keep their true surface brightness with no rule of their own.
- **`packages/views/black-hole/`**: the view "Black holes" (`sse.view.black-hole`), on the galaxy's rail (`black-hole.gm.json`). The three fixtures as chips; Band (every band, Composite), Seen from (edge is Luminet's 10° above the disc), drag to tilt, wheel to zoom; a HUD marks the shadow's edge, photon ring and inner edge. The words give the kernel's numbers: shadow, feed, brightest point, side ratio, peak band, how far visible light was dimmed, and which band's light is mostly the hot flow (not drawn).
- `ui-contract` `Session.readLightPaths`.

## Which drawing paths show a black hole

Only the new Black holes view, which has its own True path. The Body globe, the system's True view and Land do not draw one, and a generated system's remnants are not drawn by any view yet. The stage is self-contained (three new renderer files, no shared file touched) so the later merge of the three paths can take it in as one stage.

## Checks

| Check | Result |
|---|---|
| `node --test packages/renderer/test/light-path.test.ts` | the shift reproduces every worked example of `sse.light.disc_shift`; through a real kernel, rays through the kernel's three traced points per fixture land at the same distance with the same shift and seen temperature (to 10⁻⁹); the side ratio from two of the picture's own rays; the dimming figure within 0.1% (fed) and 1% (starved, Planck's tail; the kernel integrates by Simpson at 64 a decade, ui-contract by 48 trapezoids); the sky keeps each star's light to 2%. Seen red by flipping α's sign in the shift. |
| `node tests/ui/black-hole.e2e.mjs` (desk 1440 × 900, phone 390 × 844, in CI) | from the galaxy's rail; straight up from the middle of the X-ray frame the shadow is dark to the edge the table gives (√27 r_g) and the arch starts within two pixels of where the table puts it; the approaching (left) half is brighter; every band and Composite draw; the quasar peaks in the ultraviolet; the starved centre is nearly dark in visible light; Seen from Above draws at 88°; reading changes no state |

Screenshots: project files `sse4/handover-screens/XO4-screens/`.

## Results

The 10 Sun hole in visible light is a blue-white disc dimmed 3.2 × 10⁴ times, its left side 7.1 times as bright; in X-rays the far side's arch, the thin first ring and the near side crossing in front all show. The quasar's disc is fainter and wider; the starved centre is dark in visible light, and its picture in every band is the thin disc's share only (the hot flow is a total in the kernel). Stars to magnitude 6.5 are squeezed into a ring at the shadow's edge.

## Known and handed on

- The kernel's 64 r_g edge (rays straight beyond) shows as a faint seam arc in wide views; the fudge row already names the fix (the second weak-bend term).
- Deep star cones (`sky_from_here` with a cone) take minutes and a magnitude-11 cone ran out of memory in the browser's wasm, so only the whole-sky read to 6.5 is used: a kernel speed-up would fill the sky.
- XO5 (phones' WebGL2 path) and XO6 (close-up, bent grid, time stretch chip) are next. Headless chromium already runs this pass on WebGL2 through three's fallback, so XO5 starts from a working pass and has the frame budget to measure.
- ~~The picture repeats the disc's physics.~~ Done by ER11a (26 Nov, `handover/packages/ER11a-bake-tables.md`): the shift and the ring's temperature are read off the kernel's baked table `sse.light.disc` (against r and b sin i sin α / r, so dragging the tilt needs no rebake); `discShift`, `discTemperature` and `discTScale` are gone. The landing rule (geometry) and the band table (Planck and CIE, ER11b) stay in the renderer.
- Fudge rows: `docs/fudges.md`, exotic objects, the six XO4 rows.
