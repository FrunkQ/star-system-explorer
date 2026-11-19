# Formation pipeline (19 Nov 2026)

Asked by Alex via the coordinator: chain the formation stages into one pipeline and run the heat blocks in world
generation.

## What was built
- `crates/sse-formation/src/pipeline.rs`: `run(history, Block) -> SystemRun`. A formed `FormationHistory`
  (stages 1 to 3) goes through:
  - `Block::Modern`: F3 growth, F4 violent phase, F5 leftovers, then F6 early life on F5's worlds;
  - `Block::Classic`: SSE 3's Accrete with System Lab's default controls (as V3 ran it), then F6;
  - `Block::ClassicCorrected`: F2's corrected rule driven by the inheritance, then F6.
  A block that refuses a system (System Lab's ranges) leaves a sentence in `notes` and no worlds.
- Heat: every non-giant world runs the base-physics flow `sse.base.planet-heat` (the pack file itself, compiled
  once, `include_str!` of `packs/base-physics/pack.json`), with the world's age now, mass, core mass fraction,
  solid radius and the system's K-40 and r-process inheritance. `WorldHeat` keeps power, heat flow, the age
  geology falls quiet and whether it is active now. A dated `geology-quiet` event goes into `SystemRun.events`.
- `sse-formation` now depends on `sse-flow`, `sse-pack` and `sse-atoms` (were dev-dependencies).
- `PIPELINE_VERSION = formation-pipeline-v1`.

## Checks (`cargo test -p sse-formation pipeline`)
- Every block, six G0 systems: same input gives byte-identical output; events in time order; one heat record
  per solid world, none for giants.
- Twice the r-process never shortens geology and lengthens it whenever it is between 0 and 100 Gyr; Earth-sized
  worlds (0.5 to 2 M⊕) make 3 to 200 TW; the largest quarter of worlds outlive the smallest quarter.
- Golden `tests/golden/formation-pipeline.json` (four G0 systems), new; in the cross-device harness as
  `formation-pipeline`. No existing golden changed.

## Not yet
- Al-26 does not yet thin F3's water (the `planetesimal-water` flow exists; F3 would read it per band at
  planetesimal formation time, and that moves the F3 to F6 goldens, so it gets its own package).
- Explanations: `world_heat` keeps the numbers only; a caller wanting the "Why?" panel re-runs the same flow
  through the kernel's explain op with the same inputs.
- The kernel has no op that runs the pipeline yet; the first UI consumer wires it (one call, data out).
- `SystemRun.protoplanets` added for F7 (orbits for ageing); `formation-pipeline.json` digests re-blessed for it.
