---
name: sse4-cosmic-web
description: Cosmic web state after V9w (PR 368, merged 24 Nov 2026): 77% void space, 3D/True cube view, known gap in voids' matter share
metadata:
  type: project
  modified: 2026-11-24T04:03:44.532Z
---
V9w merged 24 Nov 2026 (PR 368). Alex's ask (23 Nov): voids close to the real 77% and a 3D "spiderweb" instead of a "wrinkled rug".

- `void_fraction` 0.77 in packs/base-cosmology is a share of SPACE: the T-web threshold uses Doroshkevich draws weighted by Zel'dovich room. The first try set it as a share of parcels (= matter), and the kernel thread caught it.
- Slice, cube (`web_slice` with `cube: true`) and the DM3 ledger read one web at the truncated Zel'dovich smoothing.
- Known gap (fudge register): voids hold about 40 to 50% of the matter against the measured 15%. The fix would be NEXUS+ or 2LPT.
- The view reads best at 100 Mpc with 48³ parcels (about 4 to 8 s a cube). 200 Mpc is coarse.
- Sent Alex before and after pictures; no answer yet.

Related: [[sse4-galaxy-histories]], [[sse4-landing-run-through]]
