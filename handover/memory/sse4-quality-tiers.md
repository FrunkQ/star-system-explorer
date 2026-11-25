---
name: sse4-quality-tiers
description: SSE4 track VP4/VP5 (quality tiers, speed probe, TrueRun controller, sharpen at rest): state and seams as of 25 Nov 2026
metadata:
  type: project
  modified: 2026-11-25T05:50:53.118Z
---
- Thread "Build quality tiers and speed probe" owns VP4, VP5; status handover/status/quality-tiers.md, logs handover/packages/VP4.md and VP5.md.
- Done: PR 470 (probe, place(), HeatWatch, Quality choice, bench drawing rows). PR 533 merged 25 Nov 07:10 UTC: device.ts, true-run.ts TrueRun, still.ts RestClock/RestLayer, enlarge.ts, recipes system-true and land-true (scene "land", pass true-ground); finish.ts takes rungs/glare_scale/render_scale/accumulate and is built ONCE (uniforms; a rebuild drew black for seconds on WebGL2 while compiling); gpu.ts draw takes sphere_detail/jitter + createGpu(canvas,{timing}) gpuMs; System3D and Surface each run one TrueRun (data-quality="tier|level|rest|frames"); bench draws the standard frame per tier (`standard` in report).
- Gotchas: three's async readback is rAF-quantised (use gl.readPixels); stars must use the DRAWN pixel's angle at reduced render scale; node --test cannot import JSON so never export recipes.ts from renderer index.ts (views import recipes.ts directly); Halton jitter never flips a pixel whose edge sits exactly on a boundary.
- Next (part 3): ships/stations once the renderer thread puts them on the finish; recalibrate PROBE_SCENES and pass costs from AC-ProArt's next bench (asked Alex to rerun /harness/ bench).
- Ownership: this thread owns probe.ts, probe-gpu.ts, device.ts, true-run.ts, still.ts, enlarge.ts, bench-drawing.ts, recipes.ts, governor.ts; gpu.ts shared additive-only; finish.ts is the renderer thread's (knobs added by agreement).
- Related: [[sse4-visual-polish]], [[sse4-performance-readout]].
