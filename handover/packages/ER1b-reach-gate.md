# ER1b: the reach gate

Track ER. Design: `handover/design/SSE4-edits-reach-design.md` sections 7 ("The reach gate") and 12. Built 26 Nov 2026, after ER1 (PR 644).

## What it is

`crates/sse-kernel/src/reach_gate_tests.rs`, one test in the ordinary `cargo test` run (about 11 s).

1. **The shipped stack:** the live `base-physics`, the master data sheet and every base pack with a settings block an op could read (climate, surface, weather, nav, small bodies, cosmology, cosmos, life, time, place).
2. **The nudged stack:** the same packs with every editable number scaled by 1.001: every constant of nature and of the Cosmos group, every value (`v`) on the sheet's other tables, and every real number in every physics settings block (`sse_physics::is_physics_key`).
3. Every area contract case (`tests/contract/areas/*.json`) runs on both, with each `load_stack` given the stack. Answers are compared with every `*hash*` key masked. A case that is refused on the shipped stack counts for nothing.
4. An op whose answers do not move did not reach the edit.

The ledger `handover/packages/ER-reach-ledger.txt` (the second ledger beside ER1's `ER-side-doors.txt`) has one op per line:

- `still <op>`: runs, reads physics, does not follow (a side door);
- `quiet <op> # why`: runs and reads no physics;
- `untested <op>`: no area case runs it.

The test fails on a new side door (an op that does not move and is not listed), on a listed `still` or `quiet` op that now moves, and on a wrong `untested` line either way. So the ledger only shrinks. `SSE_REACH_SHRINK=1 cargo test -p sse-kernel reach_gate` deletes stale lines, and turns an `untested` op that has gained a case but does not follow into `still` (same count, different kind). `SSE_REACH_WRITE=1` wrote it once for ER1b.

## Seen red

On today's code the gate finds side doors. With the ledger edited, it fails three ways: a `still weather` line removed gives "NEW SIDE DOOR: weather does not move", a `still orbit_stability` line added gives "orbit_stability now follows the physics", and a removed `untested hello` gives "hello has no area contract case".

## What it found on 26 Nov 2026

Of the ops the area cases run, 4 follow the nudge (`hazard_map`, `read_shadow_light`, `orbit_stability`, `open_universe`), 9 read no physics (record and catalogue ops, each with its reason on its line), and **16 read physics but do not follow**:

`adopt`, `atmosphere_advice`, `class_sample`, `describe_star`, `elements_in`, `expects`, `formation_record`, `light_through_gas`, `make_small_body`, `preview_tree`, `read_scattering`, `rehost`, `small_body`, `start_job` and `step_job` (a sweep), and `weather`.

118 ops have no area case yet. They are listed `untested`. Each area's agent shrinks that list by adding cases.

## What the nudge leaves out, and why

- **G and c are not nudged.** The sheet makes the solar, Earth and Jupiter mass units from GM / G, and the light year from c. A unit never moves (Alex, 26 Nov, question 1), so a nudged G or c is refused at load. Each GM is nudged so that G alone changes nothing, but G itself is not nudged yet. G's reach is still tested by the MP1c doubled-G tests. ER4 should give the mass units and the light year their fixed nominal values, so that G and c can be nudged here too. That is a finding for ER4.
- Units, conventions and factors are not nudged (they never move).
- Triple points are not nudged: the phase table's own point is a fit that is checked against them.
- Counts that must balance are not nudged: reaction terms, formulas, charges, electrons. Whole numbers are not nudged either, because they are counts.
- **base-light is not on the stack.** Its own flow test refuses it on every stack today: Earth's effective temperature comes out at 255.1 K, against 254 ± 1. So no stack that loads can carry its settings. This is a real finding for the light track. Add base-light back to `STACK` once it is fixed.
- `open_universe` moves, but what in its answer moves was not traced. It may be stack facts rather than physics, so treat it as a weak pass.

## Checks

- `cargo test -p sse-kernel reach_gate` green with the ledger, red as above without it.
- `tests/golden/` unchanged.
