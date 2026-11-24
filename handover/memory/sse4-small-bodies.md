---
name: sse4-small-bodies
description: Track SB small bodies up close (thread "Build asteroids and rings up close", 24 Nov): method, seams, packages
metadata:
  type: project
  modified: 2026-11-24T18:31:59.555Z
---
Alex 24 Nov 18:02/18:05: belts and rings are a probability field until one member is looked at, then it resolves into a definite body (M/S/C/P..., rubble pile, contact binary, history); a Psyche-like core is pre-resolved from formation; small body kinds join the Add list.

- Method: `sse_small::resolve` (crate crates/sse-small) on top of `leftovers::Cloud::resolve`; numbers in packs/base-small-bodies (not in the contract golden); notables = rank 1 of a shattered world's debris cloud is its iron core. Status file handover/status/small-bodies.md; SB1 = PR 459.
- Seams: kernel OK'd SB2 ops `small_body` and `make_small_body` in a new crates/sse-kernel/src/small.rs, small golden change after their golden split PR 454; editing wants recipe input units as sheet Units ids and a ping when SB2 merges; renderer: mesh side ours in a new file (not small-bodies.ts, owned by the plumes thread), optional figure on TrueBody after PR 442, additive mesh field on GpuItem.
- Full `cargo test --workspace` locally runs over 25 min in this container; let CI run it.

Related: [[sse4-element-inventory]]
