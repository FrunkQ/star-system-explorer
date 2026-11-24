---
name: sse4-star-look
description: SSE4 track SG (thread "Build star glow and flares"): ordinary stars glow, churn, lumpy giants, flares and ejections in True; ownership and next steps
metadata:
  type: project
  modified: 2026-11-24T17:33:56.022Z
---
- Alex yes 24 Nov 2026 16:41. Status handover/status/star-look.md, log handover/packages/SG1-3.md, PR 439 (24 Nov).
- Owns renderer star-glow.ts, star-churn.ts, star-events.ts, star-now.ts (`drawStarNow`), plus star.ts and live-star.ts (handed over by the renderer thread; keep StarDisc defaults). In System3D only faceFor and the star's GpuGlow halo. gpu.ts stays the renderer's.
- Glow = CIE disability glare (eye scatter), exposure = worlds' daylight; ejections drawn coronagraph-style (log scale); `shownFlares` is the shared 3-flashes-a-second cap (exotic stars thread may reuse).
- Next: hook drawStarNow into the renderer's true-body.ts `trueLook` (gallery stars glow), SG4 Body StarDisc, SG5 star jets and shells (third V3 audit gap 8, coordinator 24 Nov). Brown dwarf bands belong to "Build giant bands, clouds and hazes".
- Related: [[sse4-renderer-track]], [[sse4-exotic-objects]].
