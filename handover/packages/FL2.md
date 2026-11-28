# FL2: the settle kit (Carry, Exposure, Fade)

Status: built 28 Nov 2026, thread "Build the shared flourish system" (Opus). Design: `design/SSE4-visual-flourish-design.md`
section 4.4 and 8A. No kernel, pack or golden change; no view uses it yet (FL3 giants, FL5 the zoom).

## Files

- `packages/renderer/src/wind-table.ts`: the table a picture is carried along. RGBA floats per cell: the row's jet
  (radians of longitude a second), the turning part as it stood (radians of longitude and of latitude a second),
  unused. `windTable(w, h, jet, turn)` builds one from rules; `loopLength` works the loop out from the table itself;
  `windAt` samples it at a face's texel place. A use bakes its own table from its kernel read (FL3 from `want: "carry"`).
- `packages/renderer/src/settle-kit-cpu.ts`: the processor twins. `carryCpu`, `exposureCpu` (with `rowMeans`,
  `meanShare`), `fadeCpu`, `loopPhases`, `sampleAt`.
- `packages/renderer/src/settle-kit.ts`: the same as three.js TSL, one source for WebGPU and WebGL2. Node builders
  `carryNode`, `exposureNode`, `fadeNode` (for a view's own material), and `drawKit` (a whole face into a float
  target and back, for tests, the bench and baking). Imported directly, not through `index.ts`, because it pulls in
  `three/webgpu` (as `stirred-gpu.ts` is); the table and the twins are exported from `index.ts`.
- `bench-drawing.ts`: `kitFrames()`, the three pieces on a 512 × 256 face, and a `kit` field in the bench report.
- Register: FL-F4 (the kit's own simplifications). Citations: `docs/citations/settle-kit.md` (Max and Becker 1995,
  Vlachos 2010, both unverified until 1 Dec) and `max1995` on the master list.
- CI: its own line, `node tests/ui/settle-kit.e2e.mjs` (beside the stirred GPU check).

## How it works

- **Places in texels.** Every sum works in the face's texels (column, row; row 0 at the south pole), so a carry of
  zero seconds reads each texel at its own centre and the picture comes back untouched, to the bit, on both
  backends. The bilinear read lets the top row read itself for the same reason.
- **Carry.** The jets slide each row by its turn rate times the time, exact for any length of time (ten days at
  2e-5 rad/s is 2.75 turns). The turning part, sampled at the held picture's own place, runs as two loops half a
  loop apart: each one's offset goes from minus to plus half a loop and its weight is a triangle that is zero where
  it starts again, so the weights always sum to one and nothing jumps. At zero seconds the full weight is on the
  loop whose offset is zero.
- **The loop** is the shorter of the steepest shear time between rows (how fast neighbouring rows draw apart; a turn
  shared by every row moves nothing apart) and the time the fastest eddy takes to turn a radian (2 over the largest
  vorticity on the unit sphere). Rows within 3° of a pole are left out: there the face's pixels close to a point.
- **Exposure.** Eight reads back along the wind at the pixel over the time one picture stands for: the jets for all
  of it, the storms and eddies for at most half a loop (held straight for a day, they shifted whole bands north and
  south in the first screens). From half a turn in one picture it blends toward the row's mean, all of it at a whole
  turn; the means are worked out once per picture on the processor.
- **Fade.** a · (1 − w) + b · w, written out so w = 0 and w = 1 are exact.

## Tests

- `packages/renderer/test/settle-kit.test.ts` (7, in `ui:test`): zero carry untouched to the bit; a pure jet for ten
  days is the rows slid exactly; the loops weigh one between them and are weightless where they restart; carried on
  in steps of a 64th of a loop nothing jumps; the exposure at its limit is each row's mean and a calm exposure is the
  picture; the fade's ends exact; the loop from Jupiter-like jets (4.4 h from the steepest shear of 150 m/s jets in
  7.5° bands; the design's 16 h is a band's mean shear) and from a solid turn of a radian in 0.9 days (0.90 d).
- `tests/ui/settle-kit.e2e.mjs`: each piece on the GPU against its twin on a 64 × 32 patch, WebGL2 always, WebGPU
  where there is one. The design asked for one pixel; the twins agree to about 1e-4 of a pixel, and a whole pixel let
  a loop's turning be left out unseen, so the gate is a hundredth of a pixel. The zero carry and the fade's ends
  must be equal to the bit. `--bench` adds the timings, `--screens <dir>` saves a picture per backend,
  `SSE_CHANNEL=chrome` runs the installed Chrome (a real GPU).
- Red first (28 Nov, in the container): processor side, the loop offset left uncentred (zero carry and loops red),
  the jet's slide wrapped wrongly (slide and smoothness red), the limit at half the mean (exposure red); GPU side, one
  loop's turning left out (both eddy carries red at 0.18 and 0.08 px) and a second added to the jets' time (zero carry
  red, not equal to the bit).

## Measured

In the container (SwiftShader, software), 512 × 256 with upload and read-back: WebGL2 carry 83 ms, exposure 100 ms,
fade 50 ms; WebGPU 29, 33 and 13 ms. Software figures only; AC-ProArt's are owed (below).

Screens (container, software GPU): `sse4/handover-screens/FL2-screens/settle-kit-webgl2.png` and `-webgpu.png`, a
banded face with a spot, held, carried a day and three days, a day's exposure and a week's (at the limit).

## Owed

- The check on AC-ProArt's real GPU, WebGPU and WebGL2 (`SSE_CHANNEL=chrome node tests/ui/settle-kit.e2e.mjs --bench
  --screens ...`), its timings and screens. Recorded in the next package's log when done.
