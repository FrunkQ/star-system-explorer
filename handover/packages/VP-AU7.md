# VP-AU7: aurora seen from the ground

Track VP-AU (visual polish design section 16). Owner: "Build aurorae and night lights".

## Done
- `aurora.ts`: `toSite` (globe frame to site east, north, up), `skyGlowAlong` (one shell met by one sight line: ring value times slant path, path capped at the tangent chord), `auroraSky` (each layer's colour and lift for a site; long exposure normalised on the brightest point of the sky, sampled every 6° of elevation and 15° of azimuth, to 0.35 of white after the site's metered exposure; Eye keeps true brightness and greys below colour vision).
- `gpu-aurora-sky.ts`: a back-side dome drawn additively after the sky; the shader meets up to four shells from the eye, applies ring, path, night side and the air's own transmission along the airmass, above the horizon only.
- Seams: optional `aurora` in `gpu-surface.ts` `SurfaceFrame`; `Surface.svelte` loads the plan with `loadAurora` once the world loads and passes `auroraSky` in True; the note sits in the ground note (`data-aurora-sky`).
- Same switch-on as AU5 (rule 5): drawn only where the kernel places an oval and the air can glow; nothing on Mars or airless worlds.

## Proof
- Unit (`packages/renderer/test/aurora.test.ts`): under the oval the zenith is the brightest; 8° equatorward the arc sits low toward the pole and overhead and the equator side are dark; nothing from the tropics; red peaks higher in the sky than green.
- E2E (`tests/ui/aurora.e2e.mjs`, desk): Earth, land at 64° N 10° E, True, darkest hour, the sky note names the wind-driven oval and the long exposure; four headings screenshotted to `sse4/handover-screens/VP-AU7-screens/`.
- Checks: ui:check clean, ui:test 613 pass.

## Caveats
- Fudge row "Aurora from the ground" (temporary until VP1's finish meters it). No curtains (VP-AU6) or scattering of aurora light by the air.
