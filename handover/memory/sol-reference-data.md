---
name: sol-reference-data
description: V3's default Solar System is a reference only (authored, maybe wrong); SSE4's own Sol gets its belt and spherical Oort cloud from physics
metadata:
  type: project
  modified: 2026-11-21T16:07:43.900Z
---

Alex, 21 Nov: the "default Solar System" (with an asteroid belt and an Oort cloud) is V3's own default data, "authored over time - maybe wrong". It lives in the V3 source on his machine (C:\Development) and in what the converter repo (FrunkQ/starsystem-convert) vendors. SSE4's sample `packages/shell/public/samples/synthetic-sol.json` has no belts.

**Why:** the Oort cloud was drawn as a flat belt; Alex: "really should be a sphere". U16 (UI track) draws any belt with `shape: "sphere"` as a shell; the kernel's clouds carry `shape` from F5's tilt model (Oort isotropic).
**How to apply:** use V3's Sol to compare, never copy its numbers; SSE4's own Sol (start sheet, top-level design step 3) takes its belt and round Oort cloud from physics. See [[ui-checks-before-commit]].
