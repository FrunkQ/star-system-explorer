# VP-A3 and VP-A4 part 1: the air's rim and terminator, and clouds as their own layer

Design: `handover/design/SSE4-visual-polish-design.md` sections 5.6 and 12 (accepted by Alex 24 Nov 2026).
Owner: "Build giant bands, clouds and hazes" (status `handover/status/true-air.md`).

## What it does

- `packages/renderer/src/air-luts.ts`: for one world's air (Rayleigh scattering at the ground per planet radius and
  the scale height over the radius, from the kernel's `read_air_optics` through atmosphere.ts), Hillaire's (2020)
  transmittance table (64 × 32: zenith cosine by height up to ten scale heights) and multiple-scattering table
  (32 × 32: sun zenith cosine by height; ψ_ms = L2 / (1 - f_ms), gathered over 64 directions with an isotropic
  phase). About 0.13 s per world on the processor, once.
- `packages/renderer/src/gpu-air-shell.ts`: a shell material (TSL; WebGPU and WebGL2). Each pixel marches its view
  ray through the shell in 16 steps from the point where it enters, so the numbers stay near 1 however far the camera
  is; at each step the sun's light is one read of the transmittance table and the many-times-scattered light one read
  of the multiple-scattering table. Premultiplied output: the ground behind is dimmed by the view ray's transmittance.
- `gpu.ts`: `GpuItem.air` (optional) draws the shell round a lit world; `GpuItem.clouds` (optional) draws a world's
  clouds as a lit premultiplied layer over its ground.
- `true-air.ts`: `readTrueAir` reads a world's air (`resolved_system` facts and surface, `read_air_optics`), and
  `readTrueFace` returns it with the face; `withAir` also returns `ground` (true-look.ts `trueSurface`'s own result,
  with its `faces`, so the detail ladder's bake keeps the relief) and `clouds` (RGB their own light, premultiplied
  and sRGB-encoded; alpha their reflectance).
- `true-body.ts`: `TrueBody.air` and `TrueBody.clouds` (optional); `trueLook` builds and caches the tables per world.
- `System3D.svelte`: optional `airs` (ground, clouds, air by body id); a world with clouds is drawn as its ground with
  the clouds over it. `SystemChart.svelte` passes them from its True reader.

Grade (design section 3): fact. Every number is the kernel's. Rayleigh only: hazes, dust and droplets join the shell
with VP-A5 and the light thread's K-e (phase tables per particle kind).

## Checks

- `packages/renderer/test/air-luts.test.ts`: straight up the table gives e^-(β H) per channel and blue is dimmed most;
  into the ground nothing; at the top everything; ψ_ms at the ground is blue, a few per cent of the sun as Hillaire's
  figure 6, smaller in thinner air and near nothing with the sun well below the horizon.
- `packages/renderer/test/true-air.test.ts`: clear air covers nothing; thick cloud covers most; the clouds laid over
  the ground come within 0.05 of the flat face.
- Pictures (WebGL2, a scratch page drawing gpu.ts with a test ground): `sse4/handover-screens/VP-A3-screens/`:
  `earth-day.png` (blue limb brightening, clouds over the ground), `earth-term.png` (the terminator softened and
  blued), `thick-term.png` (twenty times Earth's Rayleigh: the air itself whitens, as a thick Rayleigh air does).
- `npm run ui:check` clean.

## Fudges

`docs/fudges.md`: "Air's rim in True (VP-A3)" and "Clouds as their own layer (VP-A4 part 1)".

## Integration points

- Weather thread: layer `base_m` and `top_m` (agreed 24 Nov, after PR 418) put each deck at its height (VP-A4 part 2).
- Light thread: K-e phase tables per particle kind put hazes into the shell (VP-A5).
- Detail ladder thread: `AirLook.clouds` and `ground` as agreed 24 Nov; `GpuItem.clouds` is drawn by this track.
- The Body globe (`gpu-globe.ts`, `gpu-air.ts`) still uses its own Nishita march; it can move to these tables.
