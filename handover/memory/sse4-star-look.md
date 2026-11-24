---
name: sse4-star-look
description: SSE4 track SG (thread "Build star glow and flares"): ordinary stars glow, churn, lumpy giants, flares and ejections in True; ownership, seams and what waits on whom
metadata:
  type: project
---
- Alex yes 24 Nov 2026 16:41. Status handover/status/star-look.md, log handover/packages/SG1-3.md. PR 439 merged; PR 460 (SG4, SG4b, SG5a, VP-S3, VP-S4, shared events) open 24 Nov.
- Owns renderer star-glow.ts, star-churn.ts, star-events.ts, star-now.ts (`drawStarNow`, `liveFace`), star-shells.ts, star-corona.ts, star.ts, live-star.ts. gpu.ts and true-body.ts stay the renderer thread's (TrueBody.face added by SG4b).
- Shared with "Build aurorae and night lights" (via coordinator): `readLiveStar(..., "system")` fills ejection targets from orbit facts (`targetsFromFacts`); `capFlashes` is the one 3-a-second flash cap.
- Waiting: SG5b on light thread's `jet`/`dust-shell` kinds in read_nebulae; VP-S1 on renderer finish.ts (then face gain, drop halo glare); VP-S2 on XO13 facts incl. star.gravity_darkening_beta, taken by kernel thread (golden lane). β is a block input: 0.25 radiative, ~0.19 fast spinners, 0.08 convective.
- Related: [[sse4-renderer-track]], [[sse4-exotic-objects]], [[sse4-visual-polish]].
