# VP4: quality tiers, the speed probe and placement by measured speed

Design: `handover/design/SSE4-visual-polish-design.md` sections 8.1 to 8.3 (accepted by Alex 24 Nov 2026 18:57 UTC, questions 2 and 3: an Ultra tier switched on only by a measured probe, every Ultra pass with a plain form; devices placed by measured speed, not by WebGPU or WebGL2). Also `SSE4-surfaces-and-rendering.md` 6.2 and 6.3. Thread: "Build quality tiers and speed probe".

## Part 1 (this PR)

What it does:
- **`gpu-ultra`** joins `Tier` (`ui-contract/src/view.ts`) and `TIER_ORDER` (`renderer/src/recipe.ts`), between `still` and `gpu-high`, budget 16.7 ms. No recipe declares it yet, so nothing draws at Ultra until the first Ultra pass ships; a device's class can already be Ultra.
- **Load mix on pass costs.** `PassCost.load` gives the shares of the probe's three scenes that bound a pass (`fill`, `march`, `geometry`); `estimateMs(..., factors)` scales each GPU pass by its own mix (`passFactor`). The canvas and text tiers are not scaled. The existing passes got their mix: relief half geometry, half fill; graticule geometry; atmosphere march.
- **The speed probe** (`probe-gpu.ts`): capabilities read without drawing (WebGPU adapter features and limits, or WebGL2 extensions; memory hint, screen pixels, reduced motion, data saver). Then three scenes off screen at 1024 × 1024: Fill (sixteen texture reads a pixel), March (24-step ray march through a smooth density), Geometry (300,000 lit triangles), plus an empty frame. Timed by `timestamp-query` where present, otherwise by frames closed with a one-pixel read-back minus the empty frame. Two warm-up frames, then seven timed (at least three), capped at 300 ms a scene. Medians over the reference device's figures give the three device factors.
- **Cache** (`probe.ts`): `sse4.render.probe` in local storage, keyed by backend, adapter and user agent, so a new graphics adapter or browser version measures again. "Measure again" in More › Performance readout forgets it and measures.
- **Placement** (`probe.ts` `place`): the richest tier whose standard recipe is predicted within 60% of its frame budget (`PLACE_MARGIN`, the governor's "well under"). WebGL2 can earn High. Ultra needs WebGPU with compute, and either the recipe's own Ultra prediction or, until a recipe declares Ultra, the High frame within 30% of budget (`ULTRA_HEADROOM`, in the fudge register). Until the probe returns, a GPU device starts at Light and is placed again when it does.
- **The person's choice**: Quality Auto, Light, Full, Ultra in the performance readout (kept as `sse4.render.quality`); every open globe is placed again at once, and the tier note says the choice is theirs (and when it is above what the device measured).
- **Heat watch**: three governor steps down inside a minute lowers the ceiling one tier for the rest of the visit, on every view.
- **The Body globe** (`runtime.ts` `chooseTier`) now uses all of this on Auto instead of "WebGPU means High"; a forced tier on the globe's "Draw on" still forces.
- **Readout lines**: `render.pass` with detail `<pass> (<tier>, estimated share)`, the globe's measured drawing time split by declared cost (labelled as an estimate; fudge register). Added to the Drawing chunk's sources.
- **Bench**: the bench on `/harness/` now also runs the drawing bench (`bench-drawing.ts`, built by `scripts/build-harness.sh` into `tests/harness/pkg/drawing/` with three.js bundled) and writes `drawing` into the report: backend, adapter, method, the three scene times and factors, the class earned and each tier's predicted standard True frame at 1080p. `scripts/min-spec.mjs` writes a "Drawing: the speed probe by device" table from reports that carry it. The existing reports in `docs/perf-reports/` predate it, so the table appears with the first new bench report.

Checks: `packages/renderer/test/probe.test.ts` (15 tests: tier order, load mix, factors, no GPU, before the probe, WebGL2 earns High, slow WebGPU stays Light, the 60% line, Ultra by headroom and by recipe, choice overrides, heat watch, cache keyed by adapter and browser, quality kept, per-pass shares, bench row), `npm run ui:test`, `npm run ui:check`, `node tests/ui/render.e2e.mjs`, `node tests/ui/perf.e2e.mjs`, the bench in headless Chromium.

Fudge register rows added: the reference device's scene times (estimates until AC-ProArt's bench sets them), `ULTRA_HEADROOM`, per-pass estimated shares.

For Alex: run `/harness/` › Run the bench on AC-ProArt and the phone once this is live, and save the reports in `docs/perf-reports/`; AC-ProArt's drawing row then sets the reference figures.

## Part 2 (next)

System True (System3D's GPU path in `gpu.ts`), Land (`gpu-surface.ts`), ships and stations under recipes and the governor, once the renderer thread's shared finish (`finish.ts`, VP1) is on main: the finish's glare rungs and size are the tier knob (B 3 rungs at quarter size, H 6 at half), each view gets a standard recipe, and the bench times each standard recipe for real beside its prediction.

## Integration points
- `renderer/src/probe.ts` `place()` is the one placement for any view: pass the view's standard recipe and its pixel count.
- `runtime.ts` exports `deviceReadout`, `measureAgain`, `setQuality`, `quality` for any settings place (the chart room's one settings place, when it exists, can move the Quality control there).

## Part 2a (built 24 Nov evening, in the PR after 470)
- `device.ts`: the device's capabilities, measured speed, the visit's heat watch and the Quality choice, shared by every view (moved out of `runtime.ts`); `onPlacementChange` tells views to place themselves again.
- `true-run.ts` `TrueRun`: one controller per True view. `place()` by the device (or a forced tier), `frame(ms, now)` feeds the governor and heat watch and returns true when the settings change, `settings(atRest)` gives the tier, render scale and every pass's parameters (the still passes at full size when at rest), `status()` gives the badge, note, budget and rest state. Tests: `test/true-run.test.ts`.
- `recipes/system-true.json` (scene `system`, a new recipe scene): passes `true-bodies` (worlds, star, rings, belts; `sphere_detail`) and `true-finish` (the shared finish; `glare_rungs`, `glare_scale`), degrade order scale 0.85, fewer rungs, fall to Light, 0.75, fewer rungs, coarser globes, 0.6, 0.5. Exported as `SYSTEM_TRUE` (kept out of the Body's recipe list). The bench now predicts with it.
- Costs of the two new passes are estimates (fudge register row "reference device") until the bench times the recipe for real in part 2b.
