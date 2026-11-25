# VP6: the detail ladder, one mechanism for "more as you close in"

Alex, 24 Nov 2026 18:39 UTC: "Add visual flourishes where they don't cost a fortune, eg more swirls when close up." Design: `handover/design/SSE4-visual-polish-design.md` sections 3 (three grades) and 6 (the ladder), accepted by Alex 24 Nov 18:57 ("yes to all"). Thread "Build detail ladder and world textures" (split from "Build photoreal renderer").

## What it does

A layer's detail below the kernel's facts is built from **rungs**, each a size on the body, halving (by default) from the layer's largest drawn size. Which rungs are drawn depends only on how finely the view sees the body:

- a rung is drawn once its features are 2 pixels across, faded in over a factor of two (`RUNG_MIN_PX`, `RUNG_FADE`);
- a rung larger than the view's span on the body fades out over a factor of two (it would be a near-uniform shift), so the cost per pixel stays about the same as you close in;
- the tier allows a number of rungs (`TIER_RUNGS`: basic 1, high 4, ultra 6, at rest 8, the shader's array); weights summed from the coarsest never pass it, and the rung beyond gets what is left, so the window slides with no pop. A test walks a giant from 50 px to 500,000 px across and checks no rung's weight jumps by more than 0.02 a step.

Each rung is a noise whose **largest size, amplitude and speed are kernel values** bound by the layer's declaration (`LadderLayer`; starters in `ladder-layers.json`). If any bound value is missing the layer is not drawn and `why` says which value (rule 3: a flourish with no number behind it is cut). The spectrum (Hurst exponent: amplitude 2^−H per halving) and a plain-number stretch are shape numbers, in the fudge register. Faster eddies at smaller sizes follow Kolmogorov (speed ∝ size^(1/3)), so finer rungs turn faster.

**Seed and clock.** Seeded from the body's address (`ladderSeed`), clocked by the universe time passed in: every seat sees the same detail in the same place, tomorrow too. Phases (gradient turn, flow phase) and each rung's domain origin are computed on the processor in f64 and handed over as small numbers. The origin folds in the caller's **anchor** (metres in the body's frame) modulo the noises' shared 867-cell period, so the shader only ever adds the caller's small local metres (rendering rule 3) and the detail stays fixed to the ground as the anchor moves (tested at Earth's radius). Simplex's lattice is skewed, so 289 along one axis is not a repeat; 867 = 3 × 289 is, for every kind.

**Noises** (`ladder-noise.ts`, written fresh from the published methods, written once over a tiny `Ops` interface so the same code runs on numbers and builds the TSL shader): simplex with turning gradients (flow noise) and its analytic gradient; curl (n × ∇ψ, tangent and divergence-free along the surface, tested); cells (Worley F1, F2 over 27 cells); ridged ((1 − |n|)²); flow (simplex carried along a direction with two-phase blending).

**HUD detail line** (`detailLine`): one sentence per view, "Curls at band edges: finer than 10,000 km is drawn from the wind's figures, not computed.", in the body's own units; layers with nothing on screen are left out; a layer not drawn gives its reason.

## Call shape (for the area threads)

```ts
import { ladderLayer, resolveLayer, rungs, discScale, ladderSeed, detailLine } from "@sse/renderer"; // or ../src/ladder.ts
import { ladderNode } from ".../renderer/src/ladder-node.ts"; // GPU code only (loads three.js)

const layer = resolveLayer(ladderLayer("giant.curls")!, facts);   // facts: { rhines_scale_m, deformation_radius_m, jet_shear_m_s, jet_speed_m_s, eddy_speed_m_s }
const lad = ladderNode("curl", localMetres /* vec3 node, metres from your anchor */, { normal /* vec3 node, for curl */, along /* vec3 node, for flow */ });
material.colorNode = yourFacts(warpedBy(lad.node));                // float for simplex/flow/cell/ridged; vec3 metres for curl
// each frame (or when the view, tier or clock moves):
if (layer.drawn) lad.set(rungs(layer, discScale(radius_m, disc_px, view_px), tier, t_s, ladderSeed(address), anchor_m), { axis: spinAxis, stretch: layer.stretch });
hud = detailLine([{ layer, shown }]);
```

**Per-latitude facts** (the climate thread's `jets` on the weather op's giant output: `lat_deg` and same-length `jet_speed_m_s`, `jet_shear_m_s`, `eddy_speed_m_s`, `rhines_scale_m`, `deformation_radius_m`): pass the arrays as they are. The rung table is one per body, so a profile is resolved at its peak magnitude (a ratio at its peak element-by-element ratio); `profileShare(layer.amplitude, facts)` gives each band's share (0..1), which the caller interpolates by latitude and multiplies into `lad.node`, so each band shows its own strength.

The processor twin `ladderAt(kind, rungs, at, { normal, along, axis, stretch })` gives the same values (canvas tier, CPU bakes, tests).

## Code

- `packages/renderer/src/ladder.ts`: `LadderLayer`, `Bound` (fact, times, over), `checkLayer`, `resolveLayer`, `LADDER_LAYERS`, `ladderLayer`, `TIER_RUNGS`, `MAX_RUNGS`, `MAX_STRETCH`, `discScale`, `rungs`, `rungOrigin`, `ladderSeed`, `profileShare`, `EDDY_EXPONENT`, `FLOW_REACH`, `detailLine`, `sizeWords`, `ladderAt`.
- `packages/renderer/src/ladder-noise.ts`: `Ops`, `NUM`, `simplex`, `curl`, `cell`, `ridged`, `PERIOD`, `WRAP`, `SIMPLEX_SCALE`, `NOISE_REFERENCES`.
- `packages/renderer/src/ladder-node.ts`: `ladderNode` (TSL, one shader for WebGPU and WebGL2; rungs in two uniform vec4 arrays of 8, a loop to the live count), `TSL` ops.
- `packages/renderer/src/ladder-layers.json`: six starter declarations (giant curls, cloud puffs, ground roughness, small craters, ice cracks, ring fine structure), each naming its owner thread and the facts it reads.
- Exports added to `packages/renderer/src/index.ts` (not the node, so pages without 3D never load three.js). `gpu.ts` untouched.

## Checks

- `packages/renderer/test/ladder.test.ts` (16 tests, in `npm run ui:test`): starters sound; missing values cut the layer and name the value (also a ratio's divisor); ratio bindings and the stretch cap; distant bodies draw nothing and rungs arrive at 2 px; tier caps (1, 4, 6, 8, 0 for text); smooth weights from 50 px to 500,000 px; coarse fade by span; seed and clock repeatable, bodies differ, finer rungs turn faster, still layers never move; ground from the fact floor down to the grain, amplitude 2^−H; anchors keep the detail on the ground at Earth's radius; simplex periodic over 867, analytic gradient matches finite differences, |n| ≤ 1; curl tangent and divergence-free; cells continuous, F1 ≤ F2, mean spacing; stretch makes features run across the axis; the HUD line; per-latitude profiles resolved at their peak with per-band shares.
- `tests/ui/ladder-gpu.e2e.mjs` (added to the ui job in `.github/workflows/ci.yml`, add-only): builds `tests/ui/ladder-gpu/` with Vite, draws all five kinds on the GPU into a float target with a far anchor, a late clock and a stretch, and compares every pixel with `ladderAt`. Cloud container, 24 Nov: WebGL2 and WebGPU both match to 2.3e-4 of the largest value for simplex, 3.4e-4 ridged, 3.3e-5 cells, 2.0e-4 flow, 7.9e-3 m of 32 m for curl. WebGPU is checked where the browser has it, skipped with a line where not.

## Cost

Per rung per pixel: simplex is 4 corners × (3 hash steps, one sin and cos pair); cells are 27 cells × 5 hash steps, so cells are the dear kind (keep them to one or two rungs on B). Measured costs on the reference phone are for the performance readout (VP4); nothing here runs until an area thread wires a layer in.

## Fudges and citations

Five rows in `docs/fudges.md` (Light, rendering and sound, after the O1 test row): the ladder itself, `EDDY_EXPONENT`, `FLOW_REACH`, `MAX_STRETCH`, the starters' shape numbers. Seven rows in `docs/citations/light-reference.md` (section "Detail ladder (VP6)"); two web pages could not be read and are marked unverified.

## Next

VP3 (texture ladder and relief normals) in this thread. Waiting builders: VP-A2 giant curls and VP-A4 cloud puffs (giants thread), VP-P1 and VP-P4 (plumes thread), VP10 ground (renderer), VP-R5 (rings thread), VP15 nebulae. Each adjusts its own declaration in `ladder-layers.json` and passes its facts.
