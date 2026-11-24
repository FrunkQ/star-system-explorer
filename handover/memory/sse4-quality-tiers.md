---
name: sse4-quality-tiers
description: SSE4 track VP4/VP5 (quality tiers, speed probe, placement by measured speed): state and seams as of 24 Nov 2026
metadata:
  type: project
  modified: 2026-11-24T19:33:09.411Z
---
- Thread "Build quality tiers and speed probe" (split from "Build photoreal renderer" 24 Nov) owns VP4, VP5; status file handover/status/quality-tiers.md, log handover/packages/VP4.md.
- VP4 part 1 = PR 470 (24 Nov): gpu-ultra tier, PassCost.load (fill/march/geometry), probe-gpu.ts (3 scenes, timestamp or sync readPixels; three's async readback is rAF-quantised), probe.ts place() (60% margin, ULTRA_HEADROOM 0.3 until an Ultra recipe exists), HeatWatch, Quality pref sse4.render.quality, cache sse4.render.probe; Body globe placed this way; perf readout Quality + Measure again; drawing bench rows (vite lib build into tests/harness/pkg/drawing).
- Ownership: this thread owns probe.ts, probe-gpu.ts, bench-drawing.ts, recipes.ts, governor.ts; gpu.ts shared additive-only; renderer thread owns finish.ts (VP1: startFinish(renderer), FinishFrame {exposure, pixel_deg, glare?}) and moves gpu-surface/globe/ship/station onto it; VP4 part 2 adds rungs?/scale? to FinishFrame (B 3 rungs quarter size, H 6 half).
- Asked Alex: run /harness/ bench on AC-ProArt to set PROBE_SCENES reference_ms (estimates now).
- Related: [[sse4-visual-polish]], [[sse4-performance-readout]].
