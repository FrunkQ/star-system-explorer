---
name: surface-timelapse-track
description: Thread "Build surface time-lapse" owns R6 part 2 (globe time-lapse from surface_frames) and the drawing side of W-sea2 to W-sea4; boundaries and contracts
metadata:
  type: project
  modified: 2026-11-24T17:28:10.423Z
---
Started 24 Nov 2026 (handed by "Build photoreal renderer"). Status file handover/status/surface-timelapse.md (holds the sea contract).
- Owns: packages/renderer/src/timelapse.ts + test, time-lapse parts of views/body (Body.svelte lapse API, BodyTimeline.svelte), frames() plumbing in runtime.ts, new packages/renderer/src/sea.ts, and the sea block of gpu-surface.ts (from "// The sea: wave slopes" to "const water =" plus its uniforms in draw()), SurfaceFrame.sea.optics, Land passing data.sea. Renderer keeps the rest; update docs/fudges.md row "The sea drawn on the ground (V10k)" when kernel numbers replace WAVE_TRAINS.
- R6c PR 436 (24 Nov): dry worlds, ice-age strip, cells reader. Proven core adds `cells` on ice-age events (after PR 417) and anchors catalogued fixed climate to today with (L/L_now)^1/4 for earlier frames.
- Sea contract with climate thread: same keys in weather report data.sea and survey.climate.sea.bands[] (null over land): W-sea2 bands_nm/absorb_per_m/backscatter_per_m/refractive_index/deep_rgb; W-sea3 trains (<=6 {wavelength_m, amplitude_m, period_s, dir_deg}), ripple_cutoff_m, superfluid, liquid_note; W-sea4 whitecap_fraction, foam_albedo, streak_dir_deg. Missing key = not sent yet.
- render.e2e's sample Earth has no orbit track (no ice ages); Sol Earth via start sheet door "sol" has them.
Related: [[weather-track]] [[sse4-renderer-track]]
