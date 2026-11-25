---
name: sse4-small-bodies
description: Track SB small bodies up close (thread "Build asteroids and rings up close"): method, what merged (PR 459, 517), seams, what is next
metadata:
  type: project
  modified: 2026-11-25T02:52:56.460Z
---
Alex 24 Nov 18:02/18:05: belts and rings are a probability field until one member is looked at, then it resolves into a definite body (M/S/C/P..., rubble pile, contact binary, history); a Psyche-like core is pre-resolved from formation; small body kinds join the Add list.

- Merged 25 Nov 02:43 UTC, PR 459: SB1 crate sse-small (`resolve` on top of `leftovers::Cloud::resolve`, notables = a shattered world's iron core), SB2 kernel ops `small_body` {system, member | cloud+rank | neither for notables, r_au?} and `make_small_body` {system, address, kind asteroid|comet-nucleus|contact-binary|binary-pair, diameter_km|mass_kg, a_au, e?}, SB3 renderer small-body.ts (TrueBody.figure, GpuItem.mesh), VP-R3 part 1 ring-structure.ts (TrueBody.mass_kg + moons; Tajeddine 2017 torque), SB4 part 1 comet.rs (activity from the sheet's vapour curves).
- PR 517 (25 Nov): System3D passes Body3D.mass_kg and moonsOf into trueFace; agreed with the chart room thread.
- Next: SB4 part 2 draw coma and tails (wait for plumeBrightness from the plumes thread); a belt member click needs a pick gesture first (True's belts are drawn dust with no member behind each point); SB5 S-inner/C-outer gradient; VP-R1/R2/R4/R5 wait on VP1, K-e, VP4, VP6; resolved members not yet listed at their own address for Traveller's resource books.
- Status file handover/status/small-bodies.md; logs handover/packages/SB1-SB4.md, VP-R3.md.
- Contract cases use universe seed "small" (seed "smallb" with kernel-form-9 forms no clouds).

Related: [[sse4-element-inventory]], [[sse4-light-track-local-ci]]
