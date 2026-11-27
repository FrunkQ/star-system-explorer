# GS2: golden split part two (frozen physics, reach side file)

Alex's yes, 24 Nov 2026 18:07 UTC. Part one (PR 454) masked the op list (golden line 1) and the
unknown-op message (line 41).

## What moved where

- **The contract's physics is frozen.** `tests/fixtures/packs/base-physics.json` is now a frozen
  copy. It no longer has to equal `packs/base-physics/pack.json`, and nobody re-copies it after a
  physics change. The contract (`tests/contract/requests.json`, `$pack:base-physics`) still loads
  it, from the Rust runner and the JS runner alike. A change to the live physics no longer moves
  `tests/golden/contract.jsonl`.
- **The live pack is tested on its own.** The test `base_physics_fixture_is_the_real_pack`
  (`crates/sse-kernel/src/tests.rs`), which checked that the copy matched the pack, is replaced by
  `the_live_base_physics_pack_loads`. The kernel unit tests that used the fixture now load the live
  pack: `tags_tests.rs`, `dressing_tests.rs`, `names_tests.rs`, `matter_tests.rs`,
  `trust_tests.rs` and `sol_tests.rs` in `sse-kernel`, and `early_tests.rs` in `sse-formation`.
  So the live physics stays under test, in those tests and in the area goldens.
- **The reach report's lists have their own file.** The constants reach line (golden line 318,
  `{"type":"reach",...}`) used to carry its `follows`, `estimated` and `cannot_follow` lists, and
  every new block changed that line. The line now keeps its head, and the lists are written as
  `"lists":"one line per block in tests/golden/contract-reach.txt"` (`contract::mask_ops` in Rust,
  `maskOps` in `packages/kernel-client/src/contract.ts`). The entries go one per line to
  `tests/golden/contract-reach.txt`, as `<constant> <list> <entry json>`
  (`contract::reach_entries`). The native contract test writes it on bless and compares it
  otherwise. A new block is now one new line there, so it merges with other new blocks.

## The rule for golden PRs from now on

A change to `packs/base-physics/pack.json` no longer moves `tests/golden/contract.jsonl`; it moves
only the area goldens that read the live pack. The contract golden still moves for a protocol
change (an op's request or reply, a new contract case) and for code or data compiled into the
kernel, such as the block formulas in `packs/base-physics/formulas/*.json`, which `sse-atoms`
builds in with `include_str!`. Adding a block adds a line to `contract-reach.txt` rather than
changing a contract line. Rebless with
`SSE_BLESS=1 CARGO_INCREMENTAL=0 cargo test -q --release -p sse-kernel contract`. A new physics
case that must be seen by the contract has to be placed in the frozen copy on purpose, with its
reason in that package's log.

## Golden change in this package

`tests/golden/contract.jsonl` line 318 now ends in the `lists` note instead of the three lists, and
`tests/golden/contract-reach.txt` is new (202 lines). No other hash moved.

## Part three: one golden per area (25 Nov)

The coordinator's call, on Alex's agreement to split the golden by area: nine golden PRs were queued
behind one another at about 80 minutes of CI each.

- `tests/contract/areas/<area>.json` holds an area's cases. `contract::run` plays each on a fresh kernel
  (natively in `contract_areas_match_their_goldens`, through a worker in
  `packages/kernel-client/test/contract.test.ts`, and in the browser harness through `pkg/fixtures/areas.json`).
  The golden is `tests/golden/areas/<area>.jsonl`. A golden with no request file fails.
- `tests/contract/requests.json` is closed at 380 cases (`the_core_sequence_is_closed`).
  `every_op_is_covered_by_the_contract` counts the areas too.
- Moved out of the core to prove it: the small bodies cases (PR 459) into `small-bodies`, and the K13 cases into
  `kernel`. Each area starts with the `load_stack` that preceded it. The moved responses are byte for byte the
  core lines they replace. The one exception is `stack_loaded`, which in the core also reported adopting into
  the universe already open; a fresh kernel has none.
- The core golden changes only at line 318 (the reach report, part two) and loses its last 16 lines.

Areas, one per owning thread (create the file the first time):

| Area file | Threads |
|---|---|
| `kernel` | Build kernel resolve and rules |
| `light` | Build light and nebulae; Build aurorae and night lights; Build star glow and flares |
| `climate` | Build shared climate and weather (seas, weather) |
| `giants` | Build giant bands, clouds and hazes |
| `galaxy` | Build galaxy charts and histories; Build galaxy arm fix and atmosphere advice; Build cosmic web view |
| `travel` | Build galaxy charts and travel screens (NAV, voyages, routes) |
| `spaceflight` | Build real spaceflight history pack; Build Paper spacecraft pack |
| `ships` | Build ship flow and power views; Build ship manifest and crew roster |
| `exotic` | Build exotic stars and black holes |
| `surface` | Build landing run-through; Build surface time-lapse; Build detail ladder and world textures |
| `small-bodies` | Build asteroids and rings up close |
| `names` | Build chart pencil and Atlas look; Build star atlas and editions; names gallery |
| `matter` | Build the master matter data sheet |
| `traveller` | Build Traveller pack |
| `run-desk` | Build Run desk playbook and players' side |
| `orac` | Build Orac, Minerva and Tars |
| `editing` | Build planet and star editing |
| `proven-core` | Build SSE4 proven core |

## 27 Nov 2026: the reach lists, one file per block
`tests/golden/contract-reach.txt` became `tests/golden/reach/<constant>/<block>.txt`, one file per block, holding
every line unchanged (285 at the split) (moved, not re-blessed; `contract.jsonl` did not move). Every PR that added blocks
appended to the one sorted list, and neighbouring additions conflicted. `check_reach_golden` fails when a block's
reach changes, when a block has no file (a new block: bless), and when a file names a block the report no longer
lists. The mask text in `contract.jsonl` still says `contract-reach.txt`; it changes with the next core golden change.
