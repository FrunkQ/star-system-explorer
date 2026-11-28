# AU-TRUE: aurorae in system True

Render audit (26 Nov 2026, https://claude.ai/artifact/AGx6F4eKYSR4rYBRnv3BCu) build item 2, "draw what the kernel already knows". Thread "Draw comets, small bodies and nebulae", 26 Nov 2026, after PR 617 (shadows) landed.

## What was built
- `renderer/src/true-aurora.ts` (new): `loadSystemAurorae` reads the field once for the system (the Solar System's worlds from read_field's measured fixture, matched by name; any other system by address), the resolved facts once (MF3 `field.tilt`, the air), and each aurora world's air optics (scale height), then plans each world's shells with the same `planAurora` the Body globe and Land use. Worlds with no aurora are left out.
- `renderer/src/gpu-aurora.ts`: `set` takes an optional centre; the shell normal is taken from that centre and the view ray from the camera when given. With no centre (the Body globe) the shader is as before.
- `renderer/src/gpu.ts`: `GpuItem.aurora` draws one set of shells per world in the draw loop, placed at the world, turned to its field pole, lit from the star's direction.
- `renderer/src/true-body.ts`: `aurora` on `TrueBody`, passed through the shared `trueLook` door (never for a star).
- `System3D.svelte`: the exposure is the shared `exposeAurora` (Long or Eye), with the star's flux at the world (σT⁴(R/d)²); a note says what is drawn (`data-true-aurorae`). `SystemChart.svelte` loads the plans once per system on the GM seat.
- Status: small-bodies SB4b row (PR 621, `packages/SB4b.md`).

No kernel, golden, fudge or citation change. The exposure stays on fudge "Aurora exposure, temporary" (VP1 finish replaces it for every view).

## Drawing paths covered
System True (`gpu.ts`) now. The Body globe (`gpu-globe.ts`) and Land (`gpu-surface.ts`) already drew aurorae; all three plan from `planAurora`.

## Evidence
- `tests/ui/system3d-aurora.test.ts` (3 pass): Sol matched by name with one field read; a generated system matched by address with its field tilt applied; `trueLook` passes the shells with the pole, and none for a star.
- `packages/renderer/test/aurora.test.ts` (14 pass, unchanged): the globe path is untouched.
- Browser (Chromium, WebGL2, built shell), Solar System in True: Earth's green [O I] 557.7 oval on its night side, drowned by daylight on the day side, as it should be. Screenshots in project files `sse4/handover-screens/AU-TRUE-screens/`.

## Not done
- Generated worlds have ovals from read_field but no emission lines yet, so they draw nothing until MF9 part 2 gives the lines. Only Sol's worlds show today.
- Aurora is drawn still (the kernel gives a steady oval); any motion would run on the campaign clock (PR 652).
