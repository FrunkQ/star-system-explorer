---
name: sse4-postcards-build
description: Build state of track PC (postcards) from the builder thread "Build postcards and hold backdrops", 25 Nov 2026
metadata:
  type: project
---
25 Nov 2026 10:21 UTC: PR 509 merged to main. It holds PC1 (ui-contract pose.ts, postcard.ts, pose on sse.bookmark/1, ws.camera/ws.aim), PC3 and PC4 part 1 (shell/src/postcards/: wall on rail Look, card, Send, visits from paste / #postcard= link / file; "Keep as postcard" on the globe). Logs are in handover/packages/PC1.md and PC3.md; status is in handover/status/postcards.md (PR 555 adds the merged line).

- The CI line `node tests/ui/postcards.e2e.mjs --screens ui-screens/postcards` is in ci.yml (Alex's yes).
- The Run desk (PC5) and renderer (PC2, photographView) threads were told at 10:21 that they can start.
- PC4 part 2 is PR 561 (25 Nov 11:00 UTC): a sent postcard carries world.origin (universe id+seed, addresses, authored facts via goesInOrigin); a visiting kernel (shell/src/postcards/visiting.ts) opens it and draws the same picture. Asked Alex (question 1) whether the seed may travel; recommended yes.
- Next: PC6, the base set with the trailer thread (needs PC2 for non-globe views).
- Main moves fast: expect conflicts in the one-line ui:e2e list in package.json, and in the SCREENS/REGISTRY spreads in App.svelte and registry.ts. Merge at once when green.

Related: [[sse4-postcards]]
