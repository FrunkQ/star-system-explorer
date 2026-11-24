---
name: feedback-better-than-v3
description: Alex's rule - don't slavishly copy V3 features; if there's a better way, do it
metadata:
  type: feedback
  modified: 2026-11-19T14:34:00.873Z
---

Do not slavishly copy every V3 feature. Where there is a better way of doing something, do it (Alex, 19 Nov 2026 14:33). Applies to the sizes tool, time controls and everything else in SSE4.

**Why:** V3 parity is a floor of capability, not a design; SSE4 is a rewrite meant to improve.
**How to apply:** when porting a V3 tool (see docs/v3-parity.md), keep what it lets people do, but design the interaction fresh; say in the package log what was done differently and why.

Related: human-scale time controls asked the same day (days, months, years on the time ruler, a campaign "now"), queued on the UI track after PR 14.
