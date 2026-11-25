# SB3: True draws small-body figures

Track SB, thread "Build asteroids and rings up close", 24 Nov 2026.

## What was built
- `packages/renderer/src/small-body.ts` (new): `smallBodyMesh(figure, address, segments)` turns the kernel's figure (axes, two lobes with offsets, relief share) into a cached mesh in units of the mean radius, long axis on x, spin axis on z. A ray from the centre leaves each lobe's ellipsoid; two lobes join through a smooth maximum (fudge SB-neck); relief is seeded value noise at the kernel's amplitude, seed the body's address so every seat sees the same lumps (fudge SB-lumps). `smallBodyColour` gives a grey from the albedo warmed by weathering (fudge SB-grey). `figureRadius`, `meanRadius` exported.
- `true-body.ts`: optional `figure` on `TrueBody`; `trueLook` then returns the mesh and colour. `gpu.ts`: optional `mesh` on `GpuItem`, geometry cached by key and disposed with the scene. Both additive; nothing else changes when `figure` is absent.

## Evidence
`packages/renderer/test/small-body.test.ts`: 5 pass (a plain ellipsoid keeps the unit sphere's volume within 2% and winds outward; the same address gives the same lumps, another address other lumps; a contact binary is longer than wide with a narrower neck than its lobes; colour follows albedo and weathering; trueLook carries the mesh).

## Not done
- A click on a belt member does not yet ask the kernel for its figure (chart room or selector to call `small_body` and pass `figure` on the TrueBody).
- Boulders through the detail ladder (VP-R5, needs VP6).
