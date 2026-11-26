# EC1: shadows, eclipses and sea glint in True

Renderer audit (26 Nov 2026, https://claude.ai/artifact/AGx6F4eKYSR4rYBRnv3BCu) build list items 1 and 2. Thread
"Draw shadows, eclipses and sea glint".

## What it does

- **Moons on planets, planets on moons, moons on moons.** Each world drawn in True wears the shadows of up to four
  bodies between it and its star. They are worked out where the bodies really are (the kernel's `orbits_at` places,
  now carried onto the scene as `orbit.r_m`; a body not placed by date uses its distance and phase in its parent's
  plane), then turned into the drawing so the real direction to the star lies along the way the drawing lights the
  world. A view that draws distances on a log scale and sizes for reading still puts each shadow where it falls.
  The Moon passing 5° to one side casts nothing, as most months.
- **Soft edges.** The star is a disc: a pixel's light is the share of the star's disc left uncovered (the lens of two
  circles), so umbra, penumbra and annular eclipses all come from one sum.
- **The light inside a shadow.** Where a caster has air, the kernel's `read_shadow_light` (VP-L3) gives the colour of
  the light bent into its umbra; this package adds `share`, how bright it is (Earth's at the Moon: 2.6e-4 of full
  sunlight, about 9 magnitudes down, unverified). Copper behind Earth, black behind an airless world. At the
  daylight exposure an umbra is still black on screen, as a photograph would be.
- **Rings both ways.** A ringed world's shadow lies across its rings, and the rings' shadow across the world: the ray
  to the star crosses the ring plane and passes (1 − cover)^(1/μ₀) there, from the same τ the ring is drawn by.
- **Sea glint from orbit.** Liquid ground (the face pictures' liquid mask, now in the bake's colour cube alpha) adds
  the sun's glint: Cox and Munk's Gaussian slopes, σ² = 0.003 + 5.12e-3 W, with Fresnel at the liquid's index.

## Drawing paths covered

| Path | Shadows | Ring shadows | Glint |
|---|---|---|---|
| System True (gpu.ts via System3D) | yes | yes | yes |
| Size comparison and class gallery (gpu.ts) | no casters (bodies drawn alone) | yes | yes |
| Body globe (gpu-globe.ts) | no | no | no |
| Land (gpu-surface.ts) | no (the sun is not dimmed in an eclipse) | no | already had its own glint |

The Body globe and Land are left for the "one face for every view" job (audit item 5), which moves the globe onto
gpu.ts. Nothing in shared renderer code was restructured.

## Files

- New: `packages/renderer/src/eclipse.ts` (the sums, processor twin), `gpu-shade.ts` (the same per pixel, a lighting
  model over three's physical one: the star's direct light only is shadowed), `eclipse-read.ts` (the kernel reads),
  `packages/renderer/test/eclipse.test.ts` (9 checks, absolute numbers).
- Changed: `gpu.ts` (lit, cloud and ring materials become node materials wearing the shade; new optional `shade`
  and `glint` on GpuItem), `texture-ladder-gpu.ts` (liquid in the colour cube's alpha, written unblended),
  `true-body.ts` (glint for liquid ground), `System3D.svelte` and `bodies.ts` (`placedBodies`, casters per world),
  `SystemChart.svelte` (reads umbras), `App.svelte` and `ui-contract` `orbits.ts`, `scene.ts`, `session.ts`
  (`placesFrom`, `orbit.r_m`, `readShadowLight`).
- Kernel: `crates/sse-light/src/shadow.rs` adds `share` and one `why` line.

## Golden

One line moves, in the light area only: `tests/golden/areas/light.jsonl` line 4 (the `read_shadow_light` reply)
gains `"share":0.0002557557275882054` and the why line "It is about 9.0 magnitudes fainter than the star's full
light (2.6e-4 of it): ...". Core golden is untouched.

## Evidence

- `node --test packages/renderer/test/eclipse.test.ts`: 9 pass. `npm run ui:test`: 1,050 pass. `npm run ui:check`: clean.
- In the browser (headless Chromium, WebGL2): a test page drawing gpu.ts directly shows a moon's umbra and penumbra,
  the world's shadow on its rings and the rings' banded shadow on the world (a gap in the ring shows as a lit stripe),
  and the glint on a wet world. Sol in the system's True view draws without errors and shows Saturn's shadow across
  its rings. Screens: project files `sse4/handover-screens/EC1-screens/`.
- Found on the way: a texture swapped into a compiled node is not rebound on WebGL2, so the rings' profile is set
  before the world's materials first compile.

## Not done, for the owner's eyes

- Glint on Sol's Earth in the system view was not obvious in a headless shot (clouds, air shell, near full phase);
  the mechanism is shown on the test page. Thirty-second eyeball: Earth in True at three-quarter light, look for a
  soft bright spot on open ocean.
- A real eclipse on screen needs a date when one happens (e.g. Io's shadow on Jupiter is near every 42 hours);
  not shot.
- For the light thread: the colour sum in `shadow.rs` weights each height by the dilution φ as well as selecting
  the band of heights whose image covers the middle; for an extended star the band selection alone may be the right
  weighting (that is how `share` is summed). The colour was left as it was.
- Next (EC2, asked by the coordinator's audit 26 Nov): the glint law into a kernel block, and the liquid's refractive
  index read from the data sheet through Look up, in place of the renderer's typed 1.333 and its Cox and Munk sums.
  Both are fudge rows until then.
