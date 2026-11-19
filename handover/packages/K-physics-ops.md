# Kernel ops from the physics track: import_v3 and run_formation (19 Nov 2026)

Asked by the UI track through the coordinator: two chart room screens ("import my V3 save", "generate a system")
need kernel ops only the physics code can back. Names follow the kernel track's convention (snake_case, verb
first, additive under `sse.kernel/1`; see `K6-additions.md` on the kernel track's branch).

## Ops
- `import_v3 { save, universe_id?, reconstruct? }`: `save` is the text of a V3 `starmap.json` or `system.json`
  (unpack a `.sse.zip` first). Response `v3_imported { universe, facts, report, reconstructed }`:
  - `facts`: address, then field, then `{ value, evidence }`, as the importer reads them (P3);
  - `report`: the difference report (`sse.import-report/1`: counts, differences with tolerance and `via`,
    unclassified and dropped fields, honesty marks, warnings, state hash);
  - `reconstructed` (only with `reconstruct`: candidate histories to search, 1 to 64): one record per known
    system, the F9 summary plus `run` (the solved `SystemRun`, history labelled reconstructed) and `matches`.
  - A save that is not V3 gives `error { code: "import_failed" }`.
- `run_formation { seed, system?, block?, birth?, now_gyr?, envelope?, inherit_pins?, star_pins?, pins? }`:
  one system through the single pipeline. Inheritance from a cosmos `birth` (G0's `SystemBirth`, read at `now_gyr`,
  default 13.8) or a patch `envelope`; neither means the stand-in solar neighbourhood. `block` is `modern`
  (default), `classic` or `classic-corrected`. `pins` is F9's `Pins` (`worlds: [{label, mass_earth, a_au, e?}]`,
  `tries`). Response `formation_ran { system, pipeline, run, pinned?, surfaces }`: `run` is the `SystemRun`
  (history, growth, upheaval, leftovers, protoplanets, worlds, heat, events, notes); `pinned` the F9 fit
  (attempt, seed, fit, fit_first, matches, removed); `surfaces` one S0 surface envelope per world.
- Both are pure: nothing is written to an open universe. Recording an import or a generated system as chronicle
  events is for the screen that adopts it (set_authored, or a later "adopt" op if the kernel track prefers one).
- Cost classes: `import_v3` light (heavy with `reconstruct`), `run_formation` medium.

- `read_world_stack { body?, fixture?, stack? }` (added later on 19 Nov for the Cutaway (Body) view): a body's
  world stack (`sse.world-stack/1`, P4). Served from the P4 fixtures (`earth`, `europa`, `mars`, `tarn`, `titan`,
  `venus`) by `fixture` name, or by `body` when it is a fixture's address; a given `stack` is checked instead.
  Response `world_stack_read { body, source, stack, ladder, notes, problems, invalid }`: `source` is
  `fixture:<name>` or `given`; `stack` is the stack as stored (fixture `sources` kept); `ladder` the "above, on
  and within" lines (`depth`, `text`, `where`, `key`); `notes` and `problems` from `physics_notes()`; `invalid`
  from `validate()` (each `{at, message}`). Nothing else gives `error { code: "no_world_stack" }` listing the
  fixtures. Light. When the surface track builds stacks from history (S7 onwards), the same op serves them with
  `source` `built:<epoch>`; the response shape stays.

## Notes for the UI
- A `run_formation` response is 70 to 300 KB of JSON for a sun-like star. Some seeds form no planets: a high
  `history.inheritance.fuv_g0` (a nearby massive star) strips the disc in under a Myr; `run.growth.events` says
  so ("gas-gone"). Pin `star_pins.primary_mass` to get a sun-like system.
- `crates/sse-import-v3`: `Fact` now derives `Serialize` (needed for the response).

## Checks
- `cargo test -p sse-kernel`: same request gives the same response; a pinned Earth at 1 au is honoured; worlds
  and surfaces line up; bad block and bad save are protocol errors.
- Contract (`tests/contract/requests.json`): five new cases, then four for `read_world_stack` (re-blessed again:
  `hello` and the unknown-op message gain it, plus four lines). `tests/golden/contract.jsonl` re-blessed on
  purpose: the `hello` op list and the unknown-op error message (which lists the variants) gain the two ops, and
  five new lines. TS types regenerated (`packages/contracts/src/generated/Request.ts`, `Response.ts`).
- Harness: kernel API through a Web Worker, 115 of 115 responses identical to native (Node and Chromium);
  `npm test` and `npm run typecheck` pass.

## Integration
- The kernel now depends on `sse-formation`, `sse-import-v3`, `sse-cosmos` and `sse-surface`.
- PR 5 (kernel track) also adds variants to `protocol.rs` and lines to the contract; whichever merges second
  keeps both sets and re-blesses `contract.jsonl` and the generated TS.
