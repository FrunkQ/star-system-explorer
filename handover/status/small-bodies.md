# Status: Small bodies up close (track SB)

Owner thread: "Build asteroids and rings up close" (took small bodies over from "Build plumes, frosts and small bodies" on 24 Nov 2026; that thread keeps plumes, frosts and hot glow). Brief: Alex, 24 Nov 2026 18:02 and 18:05 UTC: belts and rings are a probability field until you look at one member, which then resolves into a definite body (M, S, C, P and so on, rubble pile, contact binary, with a short history); a Psyche-like stripped core is pre-resolved because formation made it; contact binaries and the other kinds join the Add an object list.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

## The method (one way, no second)
`sse_formation::leftovers::Cloud::resolve(seed, k)` gives the k-th largest member's mass, size and orbit. `sse_small::resolve` (crate `crates/sse-small`) takes that and adds what the member is, from the seed and its address alone (stream `small.member.v1`): makeup, type label read off it, rubble pile or monolith, single, contact or separated pair, spin, figure, weathering, albedo and history. The largest member of a shattered world's debris is that world's bared core (`sse_small::notables`); nothing is stored, it is computed again on every ask. Every number is in `packs/base-small-bodies` (`provides.other.small_bodies`), cited in `docs/citations.md` (section "Small bodies") with simplifications in `docs/fudges.md` (rows SB-*).

## Seams agreed (24 Nov)
- Kernel ("Build kernel resolve and rules"): SB2 adds `crates/sse-kernel/src/small.rs` and dispatch lines in `lib.rs` and `protocol.rs`, a cost row in `ops()`, contract cases appended to `tests/contract/requests.json`, a perf-chunks entry. After their golden split (PR 454) a new op only appends golden lines. Check member addresses do not collide with moon addresses under a planet (BC1 puts 36 bodies in base-sol).
- Traveller: a resolved member's facts are `mass_kg`, `radius_m`, `makeup.{metal,rock,carbon,ice,gas}`, so the resource books (PR 400) mine it unchanged once the kernel lists it at its own address.
- Editing ("Build planet and star editing"): op `make_small_body` {seed, kind, diameter_km or mass_kg, a_au, around?}; recipes in `provides.other.small_bodies.recipes` {id, label, kernel_kind, inputs [{id, default, unit (a sheet Units id), kind}], citation}: small-body, comet, contact-binary, binary-asteroid. Tell them when SB2 merges.
- Renderer ("Build photoreal renderer"): the mesh side is this track's, in a new file (not `small-bodies.ts`, which the plumes thread owns), an optional `figure` on TrueBody in `true-body.ts` (after PR 442) and an additive optional `mesh` on GpuItem in `gpu.ts`. Every stand-in in the fudge register.
- Shape: the shared flattening block `sse.shape.spin_flattening` is "Build giant bands, clouds and hazes"'s; this track's breakup-side figure (Maclaurin, Jacobi) is in `sse-small/src/figure.rs` and becomes a formula block `sse.shape.spin_figure` in its own formulas file with SB2.

| Package | State | Log |
|---|---|---|
| SB1 the resolve: crate `sse-small`, pack `base-small-bodies`, notables (a shattered world's core), 14 checks (radar pair shares, spin barrier, Chandrasekhar figures, Arrokoth-like lobes, rings) | done (this PR) | packages/SB1.md |
| SB2 kernel ops `small_body` {system, member address} and `make_small_body` (Add list), facts for the resource books, `sse.shape.spin_figure` block; golden lane (appends lines after PR 454) | next | |
| SB3 True draws figures: ellipsoids, two lobes with a neck, relief from the kernel's tiles, weathering colour; new renderer file plus the optional fields above | queued (after SB2 and PR 442) | |
| SB4 comets whole (agreed with the plumes thread 24 Nov): sublimation from the energy balance, coma switch-on distance, dust and ion tails, and their drawing (takes over VP-P3; reuse `plumeBrightness` in `renderer/src/plumes.ts` for the dust tail) (render audit R32) | queued | |
| SB5 type gradient across a belt (fudge SB-belt-gradient), per-member makeup from where it formed; asks proven core | queued | |
| VP-R1 ring shadow on the planet and the planet's shadow on the rings (visual polish design 5.8) | queued (needs the renderer's VP1 finish) | |
| VP-R2 rings' dark side, opposition surge, forward scatter by particle size (5.9) | queued (needs ask K-c, ring particle size) | |
| VP-R3 gaps and wave trains at moons' computed resonances, wavy edges beside embedded moons (5.9) | queued (moons' orbits exist) | |
| VP-R4 ring particles near the camera, instanced (5.9) | queued (needs VP4) | |
| VP-R5 small-body shapes from spin and strength (this track's figure in `sse-small`, drawn by SB3), boulders through the detail ladder (5.10) | queued (needs VP6; folds into SB3) | |

Visual polish packages (VP-R1 to VP-R5) come from `handover/design/SSE4-visual-polish-design.md` section 12, accepted by Alex 24 Nov 2026 18:57 ("yes to all"). Its section 3 rule applies: every drawn detail is a fact, drawn from the numbers, or a camera effect; a flourish with no kernel number behind it is not drawn.
