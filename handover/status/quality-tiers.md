# Status: Quality tiers and speed probe

Owner thread: "Build quality tiers and speed probe" (split from "Build photoreal renderer", 24 Nov 2026). Covers: VP4 and VP5 of `design/SSE4-visual-polish-design.md` (sections 8.1 to 8.5): the `gpu-ultra` tier, the speed probe, placement by measured speed, the heat watch, quality override, per-pass lines in the performance readout, the drawing bench rows; then system True, Land, ships and stations under recipes and the governor; then sharpen at rest and enlarging on Light.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Ownership agreed with the renderer thread (24 Nov): this thread owns `packages/renderer/src/probe.ts`, `probe-gpu.ts`, `bench-drawing.ts`, `recipes.ts` and `governor.ts`; `gpu.ts` is shared and additive-only; the shared finish (`finish.ts`, VP1) is the renderer thread's, and this thread adds the tier knob (`rungs`, glare size) to its `FinishFrame`. "Build detail ladder and world textures" owns `ladder.ts` and `texture-ladder.ts`.

| Package | State | Log |
|---|---|---|
| VP4 part 1: `gpu-ultra` in the Tier type and `TIER_ORDER`; the speed probe (capabilities, three scenes at one megapixel, timestamp or read-back timing, cached per adapter and browser); placement at the richest tier predicted within 60% of budget (WebGL2 can earn High, Ultra needs WebGPU with compute); heat watch; Quality Auto, Light, Full, Ultra and "Measure again" in the performance readout; per-pass estimated shares in the readout; the drawing rows in the bench and `docs/minimum-spec.md`. The Body globe is placed this way | done (this PR) | packages/VP4.md |
| VP4 part 2: system True, Land, ships and stations run on recipes and the governor, with the finish's glare rungs as the tier knob (B 3 at quarter size, H 6 at half); standard recipe per view timed in the bench | next: waits for VP1 `finish.ts` on main | packages/VP4.md |
| VP5: sharpen at rest in every True view (jittered accumulation, more steps when still, drop back on input; fade in under reduced motion); FSR1-style enlarging on Light | after VP4 part 2 | |
